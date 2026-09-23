//! WSL settings model (port of packages/app/src/wsl/settings-model.ts).

#[derive(Clone, Debug, PartialEq)]
pub enum Runtime {
    Starting,
    Ready,
    Failed(String),
    Stopped,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpencodeCheck {
    pub distro: String,
    pub resolved_path: Option<String>,
    pub version: Option<String>,
    pub expected_version: Option<String>,
    pub matches_desktop: Option<bool>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Distro {
    pub name: String,
    pub is_default: bool,
}

pub fn wsl_runtime_retryable(runtime: &Runtime) -> bool {
    matches!(runtime, Runtime::Failed(_) | Runtime::Stopped)
}

pub fn wsl_opencode_action(check: Option<&OpencodeCheck>) -> Option<String> {
    let check = check?;
    if check.resolved_path.is_none() {
        return Some("wsl.onboarding.installOpencode".to_string());
    }
    if check.matches_desktop == Some(false) {
        return Some("wsl.onboarding.updateOpencode".to_string());
    }
    None
}

pub fn addable_probe_plan(
    selected_distro: Option<&str>,
    addable: &[Distro],
) -> Option<(String, Vec<String>)> {
    if addable.is_empty() {
        return None;
    }
    let mut ordered: Vec<String> = Vec::new();
    if let Some(selected) = selected_distro {
        ordered.extend(
            addable
                .iter()
                .filter(|distro| distro.name == selected)
                .map(|distro| distro.name.clone()),
        );
    }
    ordered.extend(
        addable
            .iter()
            .filter(|distro| Some(distro.name.as_str()) != selected_distro)
            .map(|distro| distro.name.clone()),
    );
    if ordered.is_empty() {
        return None;
    }
    let key = ordered
        .iter()
        .map(|name| format!("distro:{name}"))
        .collect::<Vec<_>>()
        .join("|");
    Some((key, ordered))
}

pub fn auto_probe_plan(runtime: Option<&Runtime>) -> Option<(String, String)> {
    match runtime {
        None => None,
        Some(Runtime::Starting) => None,
        Some(Runtime::Ready) => Some(("distros".to_string(), "refreshDistros".to_string())),
        Some(Runtime::Failed(_)) | Some(Runtime::Stopped) => None,
    }
}

#[derive(Default)]
pub struct ProbeFailureGate {
    failed: Option<String>,
}

impl ProbeFailureGate {
    pub fn accepts(&self, key: &str) -> bool {
        self.failed.as_deref() != Some(key)
    }

    pub fn settle(&mut self, key: &str) {
        self.failed = Some(key.to_string());
    }

    pub fn reset(&mut self) {
        self.failed = None;
    }
}
