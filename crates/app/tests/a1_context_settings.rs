//! Port of packages/app/src/context/settings.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/app/src/context/settings.ts.

use opencode_app::context_settings::{
    has_existing_web_state, initial_agent_visibility, is_app_upgrade, layout_transition_state,
    new_layout_designs_default, next_sunset_check_delay, resolve_new_layout_designs,
    should_display_tabs_toast, should_enable_new_layout, LayoutTransition, MAXIMUM_SUNSET_TIMEOUT,
};

#[test]
fn shows_the_picker_for_existing_profiles_and_hides_it_for_first_time_installs() {
    assert_eq!(initial_agent_visibility(None, true, None), Some(true));
    assert_eq!(initial_agent_visibility(None, false, None), Some(false));
}

#[test]
fn shows_the_picker_when_updating_from_a_recent_release() {
    assert_eq!(
        initial_agent_visibility(None, false, Some("1.18.8")),
        Some(true)
    );
}

#[test]
fn preserves_the_preference_after_initialization() {
    assert_eq!(
        initial_agent_visibility(Some(true), true, Some("1.18.8")),
        None
    );
    assert_eq!(initial_agent_visibility(Some(true), false, None), None);
}

#[test]
fn blank_profiles_default_to_the_new_layout() {
    assert!(new_layout_designs_default());
}

#[test]
fn hides_the_transition_until_a_sunset_is_scheduled() {
    assert_eq!(
        layout_transition_state(false, true, false, false),
        LayoutTransition {
            available: false,
            notice: false
        }
    );
}

#[test]
fn existing_profiles_can_switch_before_sunset() {
    assert_eq!(
        layout_transition_state(true, true, false, false),
        LayoutTransition {
            available: true,
            notice: false
        }
    );
}

#[test]
fn classifies_web_profiles_from_existing_settings_or_a_recorded_version() {
    assert!(has_existing_web_state(Some("{}"), None));
    assert!(has_existing_web_state(None, Some("1.17.19")));
    assert!(!has_existing_web_state(None, None));
}

#[test]
fn preserves_explicit_and_default_layout_preferences() {
    assert!(!resolve_new_layout_designs(false, Some(false), true));
    assert!(!resolve_new_layout_designs(false, None, false));
    assert!(resolve_new_layout_designs(false, None, true));
}

#[test]
fn sunset_replaces_the_toggle_with_a_dismissible_notice() {
    assert_eq!(
        layout_transition_state(true, true, true, false),
        LayoutTransition {
            available: false,
            notice: true
        }
    );
    assert_eq!(
        layout_transition_state(true, true, true, true),
        LayoutTransition {
            available: false,
            notice: false
        }
    );
    assert!(resolve_new_layout_designs(true, Some(false), false));
}

#[test]
fn caps_checks_for_sunsets_beyond_the_browser_timeout_limit() {
    assert_eq!(
        next_sunset_check_delay(MAXIMUM_SUNSET_TIMEOUT + 1_000, 0),
        MAXIMUM_SUNSET_TIMEOUT
    );
    assert_eq!(next_sunset_check_delay(10_000, 9_000), 1_000);
    assert_eq!(next_sunset_check_delay(9_000, 10_000), 0);
}

#[test]
fn enables_the_new_layout_when_upgrading_from_1_17_19_or_earlier() {
    assert!(should_enable_new_layout(Some("v1.17.19"), "1.17.20"));
    assert!(should_enable_new_layout(Some("1.16.9"), "2.0.0"));
}

#[test]
fn enables_the_new_layout_when_no_previous_version_was_recorded() {
    assert!(should_enable_new_layout(None, "1.17.20"));
}

#[test]
fn detects_upgrades_only_when_a_previous_version_is_older() {
    assert!(is_app_upgrade(Some("1.17.19"), "1.17.20"));
    assert!(!is_app_upgrade(None, "1.17.20"));
    assert!(!is_app_upgrade(Some("1.17.20"), "1.17.20"));
    assert!(!is_app_upgrade(Some("1.17.21"), "1.17.20"));
}

#[test]
fn shows_the_tabs_toast_for_upgrades_and_existing_installs_without_a_recorded_version() {
    assert!(should_display_tabs_toast(Some("1.17.19"), "1.17.20", false));
    assert!(should_display_tabs_toast(None, "1.17.20", true));
    assert!(!should_display_tabs_toast(None, "1.17.20", false));
}

#[test]
fn does_not_enable_the_new_layout_without_a_qualifying_upgrade() {
    assert!(!should_enable_new_layout(Some("1.17.19"), "1.17.19"));
    assert!(!should_enable_new_layout(Some("1.17.20"), "1.17.21"));
    assert!(!should_enable_new_layout(None, "1.17.19"));
    assert!(!should_enable_new_layout(Some("dev"), "1.17.20"));
}
