//! JDTLS workspace-root resolution.
//!
//! Ports the observable behaviour of `JDTLS.root` in
//! `packages/opencode/src/lsp/server.ts`: Gradle wrapper/settings/build
//! markers, the Maven `<module>` chain walk, and the Eclipse `.project`
//! fallback.

use std::fs;
use std::path::{Path, PathBuf};

/// Resolve the JDTLS root for `file` within the instance `directory`.
pub fn jdtls_root(file: &Path, directory: &Path) -> Option<PathBuf> {
    let start = file.parent()?;

    let settings_markers = ["settings.gradle", "settings.gradle.kts"];
    let gradle_markers = ["gradlew", "gradlew.bat"];

    if let Some(root) = strict_nearest_root(start, directory, &gradle_markers, &settings_markers) {
        return Some(root);
    }
    if let Some(root) = strict_nearest_root(start, directory, &settings_markers, &[]) {
        return Some(root);
    }
    if let Some(root) =
        strict_nearest_root(start, directory, &["build.gradle", "build.gradle.kts"], &[])
    {
        return Some(root);
    }

    let poms = find_up("pom.xml", start, directory);
    if !poms.is_empty() {
        let mut root = poms[0].parent().map(Path::to_path_buf)?;
        for pom in poms.iter().skip(1) {
            let parent_dir = pom.parent().map(Path::to_path_buf)?;
            let relative = relative_path(&parent_dir, &root);
            let content = fs::read_to_string(pom).unwrap_or_default();
            if is_module_of(&content, &relative) {
                root = parent_dir;
            } else {
                break;
            }
        }
        return Some(root);
    }

    strict_nearest_root(start, directory, &[".project", ".classpath"], &[])
}

fn strict_nearest_root(
    start: &Path,
    stop: &Path,
    include: &[&str],
    exclude: &[&str],
) -> Option<PathBuf> {
    if !exclude.is_empty() && !find_up_any(start, stop, exclude).is_empty() {
        return None;
    }
    let found = find_up_any(start, stop, include);
    found
        .first()
        .and_then(|path| path.parent().map(Path::to_path_buf))
}

fn find_up_any(start: &Path, stop: &Path, targets: &[&str]) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut current = start.to_path_buf();
    loop {
        for target in targets {
            let candidate = current.join(target);
            if candidate.exists() {
                result.push(candidate);
            }
        }
        if current == stop {
            break;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => break,
        }
    }
    result
}

fn find_up(name: &str, start: &Path, stop: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join(name);
        if candidate.exists() {
            result.push(candidate);
        }
        if current == stop {
            break;
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent.to_path_buf(),
            _ => break,
        }
    }
    result
}

fn relative_path(parent: &Path, child: &Path) -> String {
    child
        .strip_prefix(parent)
        .unwrap_or(child)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_module_of(pom_content: &str, module_path: &str) -> bool {
    let normalized = module_path.replace('\\', "/");
    let normalized = normalized.trim_end_matches('/').to_string();
    if normalized.is_empty() {
        return false;
    }

    let mut rest = pom_content;
    while let Some(start) = rest.find("<modules>") {
        let after = &rest[start + "<modules>".len()..];
        let Some(end) = after.find("</modules>") else {
            break;
        };
        let block = &after[..end];
        let block = strip_xml_comments(block);
        for declaration in module_declarations(&block) {
            let declaration = declaration
                .replace('\\', "/")
                .trim_start_matches("./")
                .trim_end_matches('/')
                .to_string();
            if declaration == normalized {
                return true;
            }
        }
        rest = &after[end + "</modules>".len()..];
    }
    false
}

fn strip_xml_comments(input: &str) -> String {
    let mut out = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        match rest[start..].find("-->") {
            Some(end) => rest = &rest[start + end + 3..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn module_declarations(block: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut rest = block;
    while let Some(start) = rest.find("<module>") {
        let after = &rest[start + "<module>".len()..];
        let Some(end) = after.find("</module>") else {
            break;
        };
        result.push(after[..end].trim().to_string());
        rest = &after[end + "</module>".len()..];
    }
    result
}
