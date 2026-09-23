//! Port of packages/app/src/components/updater-action.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
enum UpdaterStatus {
    Idle,
    Checking,
    Downloading,
    Ready,
    Installing,
}

#[derive(Clone, Debug, PartialEq)]
struct UpdaterState {
    status: UpdaterStatus,
    version: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Action {
    label: String,
    run: Option<String>,
}

// Local stub (fast wave): real module lands later.
fn updater_action(_state: Option<UpdaterState>) -> Action {
    Action {
        label: String::new(),
        run: None,
    }
}

#[test]
#[ignore = "porting: components/updater-action not implemented"]
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
#[ignore = "porting: components/updater-action not implemented"]
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
