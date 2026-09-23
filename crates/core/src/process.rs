//! Process execution.
//!
//! Ports the observable behaviour of `packages/core/src/process.ts` (the
//! `AppProcess` service) without the Effect runtime: run a child process with
//! optional stdin, capture stdout/stderr (separately or merged in emission
//! order), truncate captured output, enforce a timeout, and expose the platform
//! handle for advanced use.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Error raised by process execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppProcessError {
    /// The command exited with a non-zero status.
    CommandFailed {
        /// The observed exit code.
        exit_code: i32,
    },
    /// The command exceeded its configured timeout.
    TimedOut,
    /// The command could not be spawned.
    Spawn(String),
}

impl AppProcessError {
    /// Build a command-failed error for `exit_code`.
    pub fn command_failed(exit_code: i32) -> Self {
        AppProcessError::CommandFailed { exit_code }
    }

    /// The exit code, when this is a command failure.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            AppProcessError::CommandFailed { exit_code } => Some(*exit_code),
            _ => None,
        }
    }
}

impl std::fmt::Display for AppProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppProcessError::CommandFailed { exit_code } => {
                write!(f, "Command failed (exit {exit_code})")
            }
            AppProcessError::TimedOut => write!(f, "Command timed out"),
            AppProcessError::Spawn(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppProcessError {}

/// A command to execute.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandSpec {
    /// The program to run.
    pub program: String,
    /// The arguments passed to the program.
    pub args: Vec<String>,
    /// Working directory, if any.
    pub cwd: Option<String>,
    /// Environment overrides.
    pub env: BTreeMap<String, String>,
    /// Whether to inherit the parent environment (defaults to `true`).
    pub extend_env: bool,
}

impl CommandSpec {
    /// Build a command for `program` with `args`, inheriting the environment.
    pub fn new(program: &str, args: &[&str]) -> Self {
        CommandSpec {
            program: program.to_string(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
            cwd: None,
            env: BTreeMap::new(),
            extend_env: true,
        }
    }

    /// Set the working directory.
    pub fn with_cwd(mut self, cwd: &str) -> Self {
        self.cwd = Some(cwd.to_string());
        self
    }

    /// Add an environment override.
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }

    /// The `${program} ${args.join(" ")}` description stored on a run result.
    pub fn description(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }

    fn build(&self, stdin: Option<Vec<u8>>) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        if !self.extend_env {
            command.env_clear();
        }
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if stdin.is_some() {
            command.stdin(Stdio::piped());
        } else {
            command.stdin(Stdio::null());
        }
        command
    }
}

/// Options for [`AppProcess::run`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunOptions {
    /// Bytes written to the child's stdin before it is closed.
    pub stdin: Option<Vec<u8>>,
    /// Maximum captured stdout bytes.
    pub max_output_bytes: Option<usize>,
    /// Maximum captured stderr bytes.
    pub max_error_bytes: Option<usize>,
    /// Merge stdout and stderr into `output` in emission order.
    pub combine_output: bool,
    /// Abort the child once this duration elapses.
    pub timeout: Option<Duration>,
}

impl RunOptions {
    /// Set stdin bytes.
    pub fn stdin(mut self, bytes: &[u8]) -> Self {
        self.stdin = Some(bytes.to_vec());
        self
    }

    /// Set the stdout capture limit.
    pub fn max_output_bytes(mut self, max: usize) -> Self {
        self.max_output_bytes = Some(max);
        self
    }

    /// Set the stderr capture limit.
    pub fn max_error_bytes(mut self, max: usize) -> Self {
        self.max_error_bytes = Some(max);
        self
    }

    /// Merge stdout and stderr in emission order.
    pub fn combine_output(mut self) -> Self {
        self.combine_output = true;
        self
    }

    /// Abort the child after `timeout`.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// The result of a completed run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Captured stdout bytes.
    pub stdout: Vec<u8>,
    /// Captured stderr bytes.
    pub stderr: Vec<u8>,
    /// Merged output bytes when `combine_output` was requested.
    pub output: Option<Vec<u8>>,
    /// The command description.
    pub command: String,
    /// Whether stdout was truncated.
    pub stdout_truncated: bool,
    /// Whether stderr was truncated.
    pub stderr_truncated: bool,
}

/// The `AppProcess` service.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppProcess;

impl AppProcess {
    /// Run `command` and capture its output.
    pub fn run(
        &self,
        command: &CommandSpec,
        options: &RunOptions,
    ) -> Result<RunResult, AppProcessError> {
        if options.combine_output {
            return self.run_combined(command, options);
        }
        let description = command.description();
        let mut child = command
            .build(options.stdin.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| AppProcessError::Spawn(error.to_string()))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let mut stdin = child.stdin.take();

        if let Some(bytes) = &options.stdin {
            let bytes = bytes.clone();
            let mut pipe = stdin.take();
            thread::spawn(move || {
                if let Some(pipe) = pipe.as_mut() {
                    let _ = pipe.write_all(&bytes);
                }
            });
        } else {
            drop(stdin.take());
        }

        let stdout_reader = stdout.map(spawn_reader);
        let stderr_reader = stderr.map(spawn_reader);

        let status = wait_with_timeout(&mut child, options.timeout)?;
        let exit_code = status.code().unwrap_or(-1);

        let stdout = join_reader(stdout_reader);
        let stderr = join_reader(stderr_reader);

        let (stdout, stdout_truncated) = truncate(&stdout, options.max_output_bytes);
        let (stderr, stderr_truncated) = truncate(&stderr, options.max_error_bytes);

        Ok(RunResult {
            exit_code,
            stdout,
            stderr,
            output: None,
            command: description,
            stdout_truncated,
            stderr_truncated,
        })
    }

    /// Run `command` with merged stdout/stderr captured in emission order.
    pub fn run_combined(
        &self,
        command: &CommandSpec,
        options: &RunOptions,
    ) -> Result<RunResult, AppProcessError> {
        let description = command.description();
        let mut command_builder = command.build(options.stdin.clone());
        let merged = merged_stdio(&mut command_builder)?;
        let mut child = command_builder
            .spawn()
            .map_err(|error| AppProcessError::Spawn(error.to_string()))?;
        drop(command_builder);

        let mut stdin = child.stdin.take();
        if let Some(bytes) = &options.stdin {
            let bytes = bytes.clone();
            let mut pipe = stdin.take();
            thread::spawn(move || {
                if let Some(pipe) = pipe.as_mut() {
                    let _ = pipe.write_all(&bytes);
                }
            });
        } else {
            drop(stdin.take());
        }

        let reader = spawn_reader(merged);
        let status = wait_with_timeout(&mut child, options.timeout)?;
        let exit_code = status.code().unwrap_or(-1);
        let output = join_reader(Some(reader));
        let (output, _) = truncate(&output, options.max_output_bytes);

        Ok(RunResult {
            exit_code,
            stdout: Vec::new(),
            stderr: Vec::new(),
            output: Some(output),
            command: description,
            stdout_truncated: false,
            stderr_truncated: false,
        })
    }

    /// Run `command` and return its stdout as a string.
    pub fn string(&self, command: &CommandSpec) -> Result<String, AppProcessError> {
        let result = self.run(command, &RunOptions::default())?;
        Ok(String::from_utf8_lossy(&result.stdout).into_owned())
    }

    /// Run `command` and return its stdout split into lines.
    pub fn lines(&self, command: &CommandSpec) -> Result<Vec<String>, AppProcessError> {
        let result = self.run(command, &RunOptions::default())?;
        Ok(split_lines(&result.stdout))
    }

    /// Run `command`, returning its stdout lines and enforcing `ok_exit_codes`.
    pub fn run_stream(
        &self,
        command: &CommandSpec,
        ok_exit_codes: Option<&[i32]>,
        timeout: Option<Duration>,
    ) -> Result<Vec<String>, AppProcessError> {
        let options = RunOptions {
            timeout,
            ..RunOptions::default()
        };
        let result = self.run(command, &options)?;
        if let Some(codes) = ok_exit_codes {
            if !codes.contains(&result.exit_code) {
                return Err(AppProcessError::command_failed(result.exit_code));
            }
        }
        Ok(split_lines(&result.stdout))
    }

    /// Spawn `command` without waiting for completion.
    pub fn spawn(&self, command: &CommandSpec) -> Result<Handle, AppProcessError> {
        self.spawn_with(command, None)
    }

    /// Spawn `command`, optionally writing `stdin` before closing the pipe.
    pub fn spawn_with(
        &self,
        command: &CommandSpec,
        stdin: Option<Vec<u8>>,
    ) -> Result<Handle, AppProcessError> {
        let mut child = command
            .build(stdin.clone())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| AppProcessError::Spawn(error.to_string()))?;
        let pid = child.id();
        if let Some(bytes) = stdin {
            if let Some(mut pipe) = child.stdin.take() {
                thread::spawn(move || {
                    let _ = pipe.write_all(&bytes);
                });
            }
        }
        let stdout = child.stdout.take().map(spawn_shared_reader);
        let stderr = child.stderr.take().map(spawn_shared_reader);
        Ok(Handle {
            child,
            pid,
            stdout,
            stderr,
        })
    }
}

/// A running child process handle.
pub struct Handle {
    child: Child,
    pid: u32,
    stdout: Option<SharedReader>,
    stderr: Option<SharedReader>,
}

impl Handle {
    /// The operating-system process id.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Whether the child is still running.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Wait for the child to exit and return its exit code.
    pub fn wait(&mut self) -> Result<i32, AppProcessError> {
        let status = self
            .child
            .wait()
            .map_err(|error| AppProcessError::Spawn(error.to_string()))?;
        Ok(status.code().unwrap_or(-1))
    }

    /// Terminate the child.
    pub fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    /// Wait for the child and return its stdout bytes.
    pub fn stdout(&mut self) -> Vec<u8> {
        let _ = self.wait();
        join_shared_reader(self.stdout.take())
    }

    /// Wait for the child and return its stderr bytes.
    pub fn stderr(&mut self) -> Vec<u8> {
        let _ = self.wait();
        join_shared_reader(self.stderr.take())
    }

    /// Wait for the child and return merged stdout+stderr bytes.
    pub fn all(&mut self) -> Vec<u8> {
        let mut merged = self.stdout();
        merged.extend(self.stderr());
        merged
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

type Reader = thread::JoinHandle<Vec<u8>>;
type SharedReader = (Arc<Mutex<Vec<u8>>>, thread::JoinHandle<()>);

fn spawn_reader(mut reader: impl Read + Send + 'static) -> Reader {
    thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer);
        buffer
    })
}

fn join_reader(reader: Option<Reader>) -> Vec<u8> {
    reader
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default()
}

fn spawn_shared_reader(mut reader: impl Read + Send + 'static) -> SharedReader {
    let shared = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&shared);
    let handle = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = reader.read_to_end(&mut buffer);
        if let Ok(mut guard) = sink.lock() {
            guard.extend_from_slice(&buffer);
        }
    });
    (shared, handle)
}

fn join_shared_reader(reader: Option<SharedReader>) -> Vec<u8> {
    match reader {
        Some((shared, handle)) => {
            let _ = handle.join();
            match shared.lock() {
                Ok(guard) => guard.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            }
        }
        None => Vec::new(),
    }
}

fn wait_with_timeout(
    child: &mut Child,
    timeout: Option<Duration>,
) -> Result<std::process::ExitStatus, AppProcessError> {
    match timeout {
        None => child
            .wait()
            .map_err(|error| AppProcessError::Spawn(error.to_string())),
        Some(limit) => {
            let start = Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => return Ok(status),
                    Ok(None) => {}
                    Err(error) => return Err(AppProcessError::Spawn(error.to_string())),
                }
                if start.elapsed() >= limit {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(AppProcessError::TimedOut);
                }
                thread::sleep(Duration::from_millis(5));
            }
        }
    }
}

#[cfg(unix)]
fn merged_stdio(command: &mut Command) -> Result<impl Read + Send + 'static, AppProcessError> {
    use std::os::fd::OwnedFd;
    let (read_end, write_end) = std::os::unix::net::UnixStream::pair()
        .map_err(|error| AppProcessError::Spawn(error.to_string()))?;
    let write_fd: OwnedFd = write_end.into();
    let stderr_fd = write_fd
        .try_clone()
        .map_err(|error| AppProcessError::Spawn(error.to_string()))?;
    command.stdout(Stdio::from(write_fd));
    command.stderr(Stdio::from(stderr_fd));
    Ok(read_end)
}

#[cfg(not(unix))]
fn merged_stdio(command: &mut Command) -> Result<impl Read + Send + 'static, AppProcessError> {
    let _ = command;
    Err(AppProcessError::Spawn(
        "combined output is unsupported on this platform".to_string(),
    ))
}

/// Truncate a byte buffer to `max`, reporting whether bytes were dropped.
pub fn truncate(data: &[u8], max: Option<usize>) -> (Vec<u8>, bool) {
    match max {
        Some(max) if data.len() > max => (data[..max].to_vec(), true),
        _ => (data.to_vec(), false),
    }
}

/// Split decoded output into lines, dropping the trailing empty element
/// produced by a terminating newline.
pub fn split_lines(data: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(data);
    let mut out: Vec<String> = text.split('\n').map(|line| line.to_string()).collect();
    if matches!(out.last(), Some(last) if last.is_empty()) {
        out.pop();
    }
    out
}

/// The `${program} ${args.join(" ")}` description stored on a run result.
pub fn command_description(program: &str, args: &[&str]) -> String {
    CommandSpec::new(program, args).description()
}

/// Turn a successful result into `Ok`, or a command-failed error.
pub fn require_success(result: RunResult) -> Result<RunResult, AppProcessError> {
    if result.exit_code == 0 {
        Ok(result)
    } else {
        Err(AppProcessError::command_failed(result.exit_code))
    }
}

/// Turn a result whose exit code is in `codes` into `Ok`.
pub fn require_exit_in(codes: &[i32], result: RunResult) -> Result<RunResult, AppProcessError> {
    if codes.contains(&result.exit_code) {
        Ok(result)
    } else {
        Err(AppProcessError::command_failed(result.exit_code))
    }
}
