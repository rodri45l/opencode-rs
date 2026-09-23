//! Port of packages/opencode/test/lsp/jdtls-root.test.ts (upstream 18ef3cc).
//! Behaviour pinned by src/lsp/server.ts `JDTLS.root`; see docs/TEST-PORT.md.
//!
//! Ported: the pure filesystem root resolution — Maven `<module>` chain walking
//! (including multi-segment, `./` and trailing-slash normalization, comments,
//! mismatch and broken chains), Gradle markers (settings/build, `.kts`, gradlew
//! precedence over pom.xml), Eclipse `.project`, sibling isolation, and the
//! no-marker `None` case.
//! Dropped: none — every upstream case is pure filesystem logic.

use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct NotImplemented(&'static str);

fn nope<T>(topic: &'static str) -> Result<T, NotImplemented> {
    Err(NotImplemented(topic))
}

fn jdtls_root(_file: &Path, _directory: &Path) -> Result<Option<PathBuf>, NotImplemented> {
    nope("lsp jdtls-root")
}

fn base() -> PathBuf {
    std::env::temp_dir().join("opencode-rs-jdtls-s4")
}

fn mkdirp(p: &Path) {
    fs::create_dir_all(p).unwrap();
}

fn touch(p: &Path) {
    mkdirp(p.parent().unwrap());
    fs::write(p, "").unwrap();
}

fn write(p: &Path, content: &str) {
    mkdirp(p.parent().unwrap());
    fs::write(p, content).unwrap();
}

fn java_src(dir: &Path) -> PathBuf {
    dir.join("src/main/java/com/example/App.java")
}

fn workspace(name: &str) -> PathBuf {
    let root = base().join(name);
    mkdirp(&root);
    root
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn single_module_maven_project_returns_pom_directory() {
    let root = workspace("single-maven");
    touch(&root.join("pom.xml"));
    let file = java_src(&root);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn multi_module_maven_project_follows_module_chain_to_top_level_pom() {
    let root = workspace("multi-maven");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>module-a</module></modules></project>",
    );
    let child = root.join("module-a");
    touch(&child.join("pom.xml"));
    let file = java_src(&child);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn maven_project_inside_a_nested_directory() {
    let workspace = workspace("maven-workspace");
    let project = workspace.join("my-maven-app");
    touch(&project.join("pom.xml"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn nested_independent_maven_project_stops_at_its_own_pom() {
    let workspace = workspace("nested-independent");
    write(
        &workspace.join("pom.xml"),
        "<project><modules><module>module-a</module></modules></project>",
    );
    let project = workspace.join("tools/sample");
    touch(&project.join("pom.xml"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn three_level_maven_module_chain_resolves_to_top_level() {
    let root = workspace("three-level");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>apps</module></modules></project>",
    );
    let apps = root.join("apps");
    write(
        &apps.join("pom.xml"),
        "<project><modules><module>my-app</module></modules></project>",
    );
    let app = apps.join("my-app");
    touch(&app.join("pom.xml"));
    let file = java_src(&app);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn three_level_maven_chain_stops_when_module_link_is_broken() {
    let root = workspace("broken-chain");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>apps</module></modules></project>",
    );
    let apps = root.join("apps");
    touch(&apps.join("pom.xml"));
    let app = apps.join("my-app");
    touch(&app.join("pom.xml"));
    let file = java_src(&app);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(app));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn module_with_dot_slash_prefix_is_normalized() {
    let root = workspace("dot-slash-module");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>./module-a</module></modules></project>",
    );
    let child = root.join("module-a");
    touch(&child.join("pom.xml"));
    let file = java_src(&child);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn module_with_trailing_slash_is_normalized() {
    let root = workspace("trailing-slash-module");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>module-a/</module></modules></project>",
    );
    let child = root.join("module-a");
    touch(&child.join("pom.xml"));
    let file = java_src(&child);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn gradle_project_with_settings_gradle_in_a_subdirectory() {
    let workspace = workspace("gradle-sub");
    let project = workspace.join("gradle-app");
    touch(&project.join("settings.gradle"));
    touch(&project.join("build.gradle"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn gradle_project_with_only_build_gradle_in_a_subdirectory() {
    let workspace = workspace("gradle-build-sub");
    let project = workspace.join("gradle-app");
    touch(&project.join("build.gradle"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn gradle_monorepo_settings_takes_precedence_over_nested_pom() {
    let workspace = workspace("gradle-monorepo");
    let gradle_root = workspace.join("gradle-project");
    touch(&gradle_root.join("settings.gradle"));
    touch(&gradle_root.join("gradlew"));
    let sub = gradle_root.join("module-a");
    touch(&sub.join("pom.xml"));
    let file = java_src(&sub);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(gradle_root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn settings_gradle_kts_is_recognized() {
    let workspace = workspace("gradle-kts-settings");
    let project = workspace.join("gradle-app");
    touch(&project.join("settings.gradle.kts"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn build_gradle_kts_is_recognized() {
    let workspace = workspace("gradle-kts-build");
    let project = workspace.join("gradle-app");
    touch(&project.join("build.gradle.kts"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn gradlew_without_settings_gradle_in_a_subdirectory_is_recognized() {
    let workspace = workspace("gradlew-sub");
    let project = workspace.join("gradle-app");
    touch(&project.join("gradlew"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn pom_xml_is_excluded_when_gradlew_is_present_at_same_level() {
    let workspace = workspace("gradle-excludes-maven");
    let project = workspace.join("mixed-project");
    touch(&project.join("pom.xml"));
    touch(&project.join("gradlew"));
    let file = java_src(&project);
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn eclipse_project_with_dot_project_in_a_subdirectory() {
    let workspace = workspace("eclipse-sub");
    let project = workspace.join("eclipse-app");
    touch(&project.join(".project"));
    touch(&project.join(".classpath"));
    let file = project.join("src/com/example/App.java");
    touch(&file);
    assert_eq!(jdtls_root(&file, &workspace).unwrap(), Some(project));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn java_file_with_no_build_markers_returns_none() {
    let root = workspace("no-build");
    let file = root.join("src/App.java");
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), None);
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn module_multi_segment_path_matches_nested_directory() {
    let root = workspace("multi-seg-module");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>tools/sample</module></modules></project>",
    );
    let child = root.join("tools/sample");
    touch(&child.join("pom.xml"));
    let file = java_src(&child);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn module_declaration_mismatch_does_not_falsely_match() {
    let root = workspace("module-mismatch");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>module-a</module></modules></project>",
    );
    let child = root.join("module-b");
    touch(&child.join("pom.xml"));
    let file = java_src(&child);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(child));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn multiple_module_declarations_allow_second_module_to_traverse_up() {
    let root = workspace("multi-modules");
    write(
        &root.join("pom.xml"),
        "<project><modules><module>module-a</module><module>module-b</module></modules></project>",
    );
    touch(&root.join("module-a/pom.xml"));
    let child_b = root.join("module-b");
    touch(&child_b.join("pom.xml"));
    let file = java_src(&child_b);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn xml_commented_module_is_not_matched() {
    let root = workspace("commented-module");
    write(
        &root.join("pom.xml"),
        "<project><modules><!-- <module>module-a</module> --></modules></project>",
    );
    let child = root.join("module-a");
    touch(&child.join("pom.xml"));
    let file = java_src(&child);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(child));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn pom_xml_at_ctx_directory_itself_is_found() {
    let root = workspace("pom-at-ctx");
    touch(&root.join("pom.xml"));
    let file = java_src(&root);
    touch(&file);
    assert_eq!(jdtls_root(&file, &root).unwrap(), Some(root));
}

#[test]
#[ignore = "porting: lsp jdtls-root not implemented"]
fn maven_and_gradle_sibling_projects_do_not_interfere() {
    let workspace = workspace("mixed-siblings");
    let gradle_dir = workspace.join("gradle-project");
    touch(&gradle_dir.join("settings.gradle"));
    let gradle_src = gradle_dir.join("src/main/java/com/example/GradleApp.java");
    touch(&gradle_src);
    let maven_dir = workspace.join("maven-project");
    touch(&maven_dir.join("pom.xml"));
    let maven_src = maven_dir.join("src/main/java/com/example/MavenApp.java");
    touch(&maven_src);

    assert_eq!(
        jdtls_root(&gradle_src, &workspace).unwrap(),
        Some(gradle_dir)
    );
    assert_eq!(jdtls_root(&maven_src, &workspace).unwrap(), Some(maven_dir));
}
