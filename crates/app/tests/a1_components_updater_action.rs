//! Port of packages/app/src/components/updater-action.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.

use opencode_app::updater_action::{updater_action, Action, UpdaterState, UpdaterStatus};

#[test]
fn disables_update_actions_when_the_platform_has_no_updater() {
    assert_eq!(
        updater_action(None),
        Action {
            label: "settings.updates.action.checkNow".into(),
            run: None
        }
    );
}

#[test]
fn projects_updater_transitions_into_one_settings_action() {
    assert_eq!(
        updater_action(Some(UpdaterState {
            status: UpdaterStatus::Idle,
            version: None
        })),
        Action {
            label: "settings.updates.action.checkNow".into(),
            run: Some("check".into())
        }
    );
    assert_eq!(
        updater_action(Some(UpdaterState {
            status: UpdaterStatus::Checking,
            version: None
        })),
        Action {
            label: "settings.updates.action.checking".into(),
            run: None
        }
    );
    assert_eq!(
        updater_action(Some(UpdaterState {
            status: UpdaterStatus::Downloading,
            version: Some("2.0.0".into())
        })),
        Action {
            label: "settings.updates.action.downloading".into(),
            run: None
        }
    );
    assert_eq!(
        updater_action(Some(UpdaterState {
            status: UpdaterStatus::Ready,
            version: Some("2.0.0".into())
        })),
        Action {
            label: "toast.update.action.installRestart".into(),
            run: Some("install".into())
        }
    );
    assert_eq!(
        updater_action(Some(UpdaterState {
            status: UpdaterStatus::Installing,
            version: Some("2.0.0".into())
        })),
        Action {
            label: "settings.updates.action.installing".into(),
            run: None
        }
    );
}
