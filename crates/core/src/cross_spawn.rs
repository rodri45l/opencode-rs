//! Cross-platform process spawning.
//!
//! Ports the observable behaviour of `packages/core/src/cross-spawn-spawner.ts`:
//! run a command and capture stdout/stderr (separately or merged), read its exit
//! code, provide stdin, kill a running child, and build/execute pipelines. The
//! Effect `ChildProcessSpawner` service is replaced by a synchronous spawner.

use std::collections::BTreeMap;
use std::path::Path;

use crate::process::{AppProcess, AppProcessError, CommandSpec, Handle as ProcessHandle};

/// A command to spawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Command {
    /// The program to run.
    pub program: String,
    /// The arguments passed to the program.
    pub args: Vec<String>,
    /// Working directory, if any.
    pub cwd: Option<String>,
    /// Environment overrides.
    pub env: BTreeMap<String, String>,
    /// Whether to inherit the parent environment.
    pub extend_env: bool,
    /// Whether to route through the platform shell.
    pub shell: bool,
    /// Bytes written to stdin before it is closed.
    pub stdin: Option<Vec<u8>>,
}

impl Command {
    /// Build a command for `program` with `args`.
    pub fn new(program: &str, args: &[&str]) -> Self {
        Command {
            program: program.to_string(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
            extend_env: true,
            ..Command::default()
        }
    }

    /// Set the working directory.
    pub fn with_cwd(mut self, cwd: &str) -> Self {
        self.cwd = Some(cwd.to_string());
        self
    }

    /// Add an environment override.
    pub fn with_env(mut self, key: &str, value: &str, extend_env: bool) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self.extend_env = extend_env;
        self
    }

    /// Provide stdin bytes.
    pub fn with_stdin(mut self, stdin: &[u8]) -> Self {
        self.stdin = Some(stdin.to_vec());
        self
    }

    fn spec(&self) -> CommandSpec {
        if self.shell {
            let line = if self.args.is_empty() {
                self.program.clone()
            } else {
                format!("{} {}", self.program, self.args.join(" "))
            };
            #[cfg(windows)]
            let (program, args) = ("cmd".to_string(), vec!["/C".to_string(), line]);
            #[cfg(not(windows))]
            let (program, args) = ("sh".to_string(), vec!["-c".to_string(), line]);
            return CommandSpec {
                program,
                args,
                cwd: self.cwd.clone(),
                env: self.env.clone(),
                extend_env: self.extend_env,
            };
        }
        CommandSpec {
            program: self.program.clone(),
            args: self.args.clone(),
            cwd: self.cwd.clone(),
            env: self.env.clone(),
            extend_env: self.extend_env,
        }
    }
}

/// Which stream is piped from one stage into the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeFrom {
    /// Pipe stdout.
    Stdout,
    /// Pipe stderr.
    Stderr,
    /// Pipe merged stdout and stderr.
    All,
}

/// A multi-stage pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    /// The ordered command stages.
    pub stages: Vec<Command>,
    /// The stream piped between stages.
    pub from: PipeFrom,
}

/// Build a two-stage pipeline.
pub fn pipe_to(first: Command, next: Command, from: PipeFrom) -> Pipeline {
    Pipeline {
        stages: vec![first, next],
        from,
    }
}

/// Build a pipeline from an ordered list of stages.
pub fn pipe_chain(stages: Vec<Command>, from: PipeFrom) -> Pipeline {
    Pipeline { stages, from }
}

/// A spawned child handle.
pub struct Handle {
    inner: ProcessHandle,
}

impl Handle {
    /// The operating-system process id.
    pub fn pid(&self) -> u32 {
        self.inner.pid()
    }

    /// Wait for the child and return its exit code.
    pub fn exit_code(&mut self) -> i32 {
        self.inner.wait().unwrap_or(-1)
    }

    /// Whether the child is still running.
    pub fn running(&mut self) -> bool {
        self.inner.is_running()
    }

    /// Wait for the child and return its stdout bytes.
    pub fn stdout(&mut self) -> Vec<u8> {
        self.inner.stdout()
    }

    /// Wait for the child and return its stderr bytes.
    pub fn stderr(&mut self) -> Vec<u8> {
        self.inner.stderr()
    }

    /// Wait for the child and return its merged output bytes.
    pub fn all(&mut self) -> Vec<u8> {
        self.inner.all()
    }

    /// Terminate the child.
    pub fn kill(&mut self) -> std::io::Result<()> {
        self.inner.kill()
    }
}

/// The cross-platform spawner.
#[derive(Debug, Clone, Copy, Default)]
pub struct Spawner;

impl Spawner {
    /// Run `command` and return its trimmed stdout as a string.
    pub fn string(&self, command: &Command) -> Result<String, AppProcessError> {
        let result = AppProcess.run(&command.spec(), &Default::default())?;
        Ok(decode_byte_stream(&[result.stdout]))
    }

    /// Spawn `command` without waiting for completion.
    pub fn spawn(&self, command: &Command) -> Result<Handle, AppProcessError> {
        let inner = AppProcess.spawn_with(&command.spec(), command.stdin.clone())?;
        Ok(Handle { inner })
    }

    /// Run `command` and return its exit code.
    pub fn exit_code(&self, command: &Command) -> Result<i32, AppProcessError> {
        let result = AppProcess.run(&command.spec(), &Default::default())?;
        Ok(result.exit_code)
    }

    /// Execute every stage of `pipeline`, feeding the selected stream forward.
    pub fn pipeline(&self, pipeline: &Pipeline) -> Result<Handle, AppProcessError> {
        if pipeline.stages.is_empty() {
            return Err(AppProcessError::Spawn("empty pipeline".to_string()));
        }
        let mut input: Option<Vec<u8>> = None;
        let mut last: Option<Handle> = None;
        for (index, stage) in pipeline.stages.iter().enumerate() {
            let mut stage = stage.clone();
            if index > 0 {
                stage.stdin = input.take();
            }
            let handle = self.spawn(&stage)?;
            if index + 1 < pipeline.stages.len() {
                let mut handle = handle;
                input = Some(match pipeline.from {
                    PipeFrom::Stdout => handle.stdout(),
                    PipeFrom::Stderr => handle.stderr(),
                    PipeFrom::All => handle.all(),
                });
            } else {
                last = Some(handle);
            }
        }
        Ok(last.expect("non-empty pipeline"))
    }
}

/// Concatenate byte chunks, UTF-8 decode, and trim surrounding whitespace.
pub fn decode_byte_stream(chunks: &[Vec<u8>]) -> String {
    let mut out = Vec::new();
    for chunk in chunks {
        out.extend_from_slice(chunk);
    }
    String::from_utf8_lossy(&out).trim().to_string()
}

/// Merge a base environment with command overrides honouring `extend_env`.
pub fn resolve_env(
    base: &BTreeMap<String, String>,
    cmd: &BTreeMap<String, String>,
    extend_env: bool,
) -> BTreeMap<String, String> {
    let mut out = if extend_env {
        base.clone()
    } else {
        BTreeMap::new()
    };
    for (key, value) in cmd {
        out.insert(key.clone(), value.clone());
    }
    out
}

/// Whether a process is still alive (`kill(pid, 0)` semantics).
pub fn alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    #[cfg(target_os = "linux")]
    {
        Path::new(&format!("/proc/{pid}")).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        false
    }
}
