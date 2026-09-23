//! Port of packages/httpapi-codegen/test/write.test.ts (upstream 18ef3cc).
//!
//! Tier W: the reference injects an Effect `FileSystem.makeNoop`; the behavioural
//! assertions (file set, manifest contents, stale-file removal, and path-safety
//! rejections) are re-expressed against the Rust [`OutputFs`] port. The Effect
//! service plumbing is dropped.
//!
//! Until `write` is implemented every behavioural test is red-first and ignored.

use std::cell::RefCell;

use opencode_protocol::{write, GeneratedFile, GenerationError, Output, OutputFs, MANIFEST_NAME};

#[derive(Default)]
struct FakeFs {
    manifest: Option<String>,
    /// Paths that exist.
    existing: Vec<String>,
    /// Paths that exist and are symbolic links.
    symlinks: Vec<String>,
    writes: RefCell<Vec<(String, String)>>,
    removes: RefCell<Vec<String>>,
}

impl FakeFs {
    fn with_manifest(manifest: &str) -> Self {
        Self {
            manifest: Some(manifest.to_string()),
            ..Self::default()
        }
    }

    fn with_symlink(path: &str) -> Self {
        Self {
            existing: vec![path.to_string()],
            symlinks: vec![path.to_string()],
            ..Self::default()
        }
    }

    fn writes(&self) -> Vec<(String, String)> {
        self.writes.borrow().clone()
    }

    fn removes(&self) -> Vec<String> {
        self.removes.borrow().clone()
    }
}

impl OutputFs for FakeFs {
    fn exists(&self, path: &str) -> bool {
        self.existing.iter().any(|p| p == path)
    }

    fn is_symlink(&self, path: &str) -> bool {
        self.symlinks.iter().any(|p| p == path)
    }

    fn read_manifest(&self, _path: &str) -> Result<Option<String>, GenerationError> {
        Ok(self.manifest.clone())
    }

    fn write_file(&self, path: &str, content: &str) -> Result<(), GenerationError> {
        self.writes
            .borrow_mut()
            .push((path.to_string(), content.to_string()));
        Ok(())
    }

    fn remove_file(&self, path: &str) -> Result<(), GenerationError> {
        self.removes.borrow_mut().push(path.to_string());
        Ok(())
    }
}

fn output(files: &[(&str, &str)]) -> Output {
    Output {
        operations: Vec::new(),
        files: files
            .iter()
            .map(|(path, content)| GeneratedFile::new(*path, *content))
            .collect(),
    }
}

#[test]
fn writes_compiled_files_and_the_owned_manifest() {
    let fs = FakeFs::default();
    let out = output(&[("session.ts", "export const session = {}")]);

    write(&out, "/generated", &fs).expect("write");

    let expected_manifest = "[\n  \"session.ts\"\n]\n";
    assert_eq!(
        fs.writes(),
        vec![
            (
                "/generated/session.ts".to_string(),
                "export const session = {}\n".to_string()
            ),
            (
                format!("/generated/{MANIFEST_NAME}"),
                expected_manifest.to_string()
            ),
        ]
    );
}

#[test]
fn removes_only_stale_files_owned_by_the_previous_manifest() {
    let fs = FakeFs::with_manifest(r#"["old.ts", "session.ts"]"#);
    let out = output(&[("session.ts", "")]);

    write(&out, "/generated", &fs).expect("write");

    assert_eq!(fs.removes(), vec!["/generated/old.ts".to_string()]);
}

#[test]
fn rejects_unsafe_and_duplicate_output_paths_before_writing() {
    let fs = FakeFs::default();
    let out = output(&[("../outside.ts", ""), ("client.ts", ""), ("CLIENT.ts", "")]);

    let err = write(&out, "/generated", &fs).unwrap_err();

    assert!(matches!(err, GenerationError::UnsafeOutputPath { .. }));
    assert!(err.reason().starts_with("Unsafe output path"));
    assert!(fs.writes().is_empty());
}

#[test]
fn rejects_case_insensitive_duplicate_output_paths() {
    let fs = FakeFs::default();
    let out = output(&[("client.ts", ""), ("CLIENT.ts", "")]);

    let err = write(&out, "/generated", &fs).unwrap_err();

    assert_eq!(err.reason(), "Duplicate output path: CLIENT.ts");
    assert!(fs.writes().is_empty());
}

#[test]
fn reserves_the_private_manifest_path() {
    let fs = FakeFs::default();
    let out = output(&[(MANIFEST_NAME, "")]);

    let err = write(&out, "/generated", &fs).unwrap_err();

    assert!(err.reason().contains("Unsafe output path"));
}

#[test]
fn rejects_existing_symbolic_link_output_targets() {
    let fs = FakeFs::with_symlink("/generated/session.ts");
    let out = output(&[("session.ts", "")]);

    let err = write(&out, "/generated", &fs).unwrap_err();

    assert_eq!(err.reason(), "Unsafe output path: session.ts");
}
