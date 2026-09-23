//! Updater settings action (port of packages/app/src/components/updater-action.ts).

#[derive(Clone, Debug, PartialEq)]
pub enum UpdaterStatus {
    Idle,
    Checking,
    Downloading,
    Ready,
    Installing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UpdaterState {
    pub status: UpdaterStatus,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    pub label: String,
    pub run: Option<String>,
}

fn action(label: &str, run: Option<&str>) -> Action {
    Action {
        label: label.to_string(),
        run: run.map(|value| value.to_string()),
    }
}

pub fn updater_action(state: Option<UpdaterState>) -> Action {
    let state = match state {
        Some(state) => state,
        None => return action("settings.updates.action.checkNow", None),
    };
    match state.status {
        UpdaterStatus::Checking => action("settings.updates.action.checking", None),
        UpdaterStatus::Downloading => action("settings.updates.action.downloading", None),
        UpdaterStatus::Ready => action("toast.update.action.installRestart", Some("install")),
        UpdaterStatus::Installing => action("settings.updates.action.installing", None),
        UpdaterStatus::Idle => action("settings.updates.action.checkNow", Some("check")),
    }
}
