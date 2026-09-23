//! Port of packages/opencode/test/config/markdown.test.ts (upstream 18ef3cc).
//!
//! Behaviour pinned: `ConfigMarkdown.files` extracts `@path` references,
//! ignoring backtick-quoted references, email addresses, trailing punctuation,
//! and supporting extensions, hidden paths, absolute and `~` paths.
//!
//! The frontmatter-parsing suites of the reference file require YAML fixtures
//! and are tracked as skipped (see PORT-STATUS.s5.json).
#![allow(dead_code)]

// Fast-wave local stubs: `config::markdown` is not implemented in this crate yet.
mod config_markdown {
    pub fn files(_template: &str) -> Result<Vec<(String, String)>, &'static str> {
        Err("porting: ConfigMarkdown.files not implemented")
    }
}

const TEMPLATE: &str = "This is a @valid/path/to/a/file and it should also match at
  the beginning of a line:

  @another-valid/path/to/a/file

  but this is not:

     - Adds a \"Co-authored-by:\" footer which clarifies which AI agent
       helped create this commit, using an appropriate `noreply@...`
       or `noreply@anthropic.com` email address.

  We also need to deal with files followed by @commas, ones
  with @file-extensions.md, even @multiple.extensions.bak,
  hidden directories like @.config/ or files like @.bashrc
  and ones at the end of a sentence like @foo.md.

  Also shouldn't forget @/absolute/paths.txt with and @/without/extensions,
  as well as @~/home-files and @~/paths/under/home.txt.

  If the reference is `@quoted/in/backticks` then it shouldn't match at all.";

fn matches() -> Vec<(String, String)> {
    config_markdown::files(TEMPLATE).unwrap()
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_exactly_12_file_references() {
    assert_eq!(matches().len(), 12);
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_valid_path_to_a_file() {
    assert_eq!(matches()[0].1, "valid/path/to/a/file");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_another_valid_path_to_a_file() {
    assert_eq!(matches()[1].1, "another-valid/path/to/a/file");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_paths_ignoring_comma_after() {
    assert_eq!(matches()[2].1, "commas");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_a_path_with_a_file_extension_and_comma_after() {
    assert_eq!(matches()[3].1, "file-extensions.md");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_a_path_with_multiple_dots_and_comma_after() {
    assert_eq!(matches()[4].1, "multiple.extensions.bak");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_hidden_directory() {
    assert_eq!(matches()[5].1, ".config/");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_hidden_file() {
    assert_eq!(matches()[6].1, ".bashrc");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_a_file_ignoring_period_at_end_of_sentence() {
    assert_eq!(matches()[7].1, "foo.md");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_an_absolute_path_with_an_extension() {
    assert_eq!(matches()[8].1, "/absolute/paths.txt");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_an_absolute_path_without_an_extension() {
    assert_eq!(matches()[9].1, "/without/extensions");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_an_absolute_path_in_home_directory() {
    assert_eq!(matches()[10].1, "~/home-files");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_extract_an_absolute_path_under_home_directory() {
    assert_eq!(matches()[11].1, "~/paths/under/home.txt");
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_not_match_when_preceded_by_backtick() {
    let backtick_test = "This `@should/not/match` should be ignored";
    let backtick_matches = config_markdown::files(backtick_test).unwrap();
    assert_eq!(backtick_matches.len(), 0);
}

#[test]
#[ignore = "porting: config-markdown not implemented"]
fn should_not_match_email_addresses() {
    let email_test = "Contact user@example.com for help";
    let email_matches = config_markdown::files(email_test).unwrap();
    assert_eq!(email_matches.len(), 0);
}
