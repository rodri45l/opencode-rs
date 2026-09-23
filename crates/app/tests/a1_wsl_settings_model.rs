//! Port of packages/app/src/wsl/settings-model.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
enum Runtime {
    Starting,
    Ready,
    Failed(String),
    Stopped,
}

#[derive(Clone, Debug, PartialEq)]
struct OpencodeCheck {
    distro: String,
    resolved_path: Option<String>,
    version: Option<String>,
    expected_version: Option<String>,
    matches_desktop: Option<bool>,
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Distro {
    name: String,
    is_default: bool,
}

// Local stubs (fast wave): real module lands later.
fn wsl_runtime_retryable(_runtime: &Runtime) -> bool {
    false
}

fn wsl_opencode_action(_check: Option<&OpencodeCheck>) -> Option<String> {
    None
}

fn addable_probe_plan(
    _selected_distro: Option<&str>,
    _addable: &[Distro],
) -> Option<(String, Vec<String>)> {
    None
}

fn auto_probe_plan(_runtime: Option<&Runtime>) -> Option<(String, String)> {
    None
}

#[derive(Default)]
struct ProbeFailureGate {
    failed: Vec<String>,
}

impl ProbeFailureGate {
    // Local stubs (fast wave): real module lands later.
    fn accepts(&self, _key: &str) -> bool {
        false
    }
    fn settle(&mut self, _key: &str) {}
    fn reset(&mut self) {}
}

#[test]
#[ignore = "porting: wsl/settings-model not implemented"]
fn retries_only_settled_unsuccessful_runtimes() {
    assert!(!wsl_runtime_retryable(&Runtime::Starting));
    assert!(!wsl_runtime_retryable(&Runtime::Ready));
    assert!(wsl_runtime_retryable(&Runtime::Failed("boom".into())));
    assert!(wsl_runtime_retryable(&Runtime::Stopped));
}

#[test]
#[ignore = "porting: wsl/settings-model not implemented"]
fn offers_install_and_update_only_when_opencode_needs_attention() {
    assert_eq!(wsl_opencode_action(None), None);
    assert_eq!(
        wsl_opencode_action(Some(&OpencodeCheck {
            distro: "Debian".into(),
            resolved_path: None,
            version: None,
            expected_version: Some("1.2.3".into()),
            matches_desktop: None,
            error: None,
        })),
        Some("wsl.onboarding.installOpencode".to_string())
    );
    assert_eq!(
        wsl_opencode_action(Some(&OpencodeCheck {
            distro: "Debian".into(),
            resolved_path: Some("/usr/local/bin/opencode".into()),
            version: Some("1.2.2".into()),
            expected_version: Some("1.2.3".into()),
            matches_desktop: Some(false),
            error: None,
        })),
        Some("wsl.onboarding.updateOpencode".to_string())
    );
    assert_eq!(
        wsl_opencode_action(Some(&OpencodeCheck {
            distro: "Debian".into(),
            resolved_path: Some("/usr/local/bin/opencode".into()),
            version: Some("1.2.3".into()),
            expected_version: Some("1.2.3".into()),
            matches_desktop: Some(true),
            error: None,
        })),
        None
    );
}

#[test]
#[ignore = "porting: wsl/settings-model not implemented"]
fn plans_addable_distro_probes_with_the_selected_distro_first() {
    let addable = vec![
        Distro {
            name: "Debian".into(),
            is_default: true,
        },
        Distro {
            name: "Ubuntu".into(),
            is_default: false,
        },
    ];
    let plan = addable_probe_plan(Some("Ubuntu"), &addable);
    assert_eq!(
        plan,
        Some((
            "distro:Ubuntu|distro:Debian".to_string(),
            vec!["Ubuntu".to_string(), "Debian".to_string()]
        ))
    );
}

#[test]
#[ignore = "porting: wsl/settings-model not implemented"]
fn plans_bootstrap_probes_for_missing_runtime_and_initial_distro_lists() {
    assert_eq!(auto_probe_plan(None), None);
    assert_eq!(auto_probe_plan(Some(&Runtime::Starting)), None);
    assert_eq!(
        auto_probe_plan(Some(&Runtime::Ready)),
        Some(("distros".to_string(), "refreshDistros".to_string()))
    );
}

#[test]
#[ignore = "porting: wsl/settings-model not implemented"]
fn does_not_accept_the_same_failed_probe_command_until_reset() {
    let mut gate = ProbeFailureGate::default();
    assert!(gate.accepts("addable:distro:Debian"));
    gate.settle("addable:distro:Debian");
    assert!(!gate.accepts("addable:distro:Debian"));
    assert!(gate.accepts("addable:distro:Ubuntu"));
    gate.reset();
    assert!(gate.accepts("addable:distro:Debian"));
}
