//! Settings context helpers.
//!
//! Port of `packages/app/src/context/settings.ts` (upstream 18ef3cc).

/// The maximum delay accepted by browser timers.
pub const MAXIMUM_SUNSET_TIMEOUT: i64 = 2_147_483_647;

/// A layout transition projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutTransition {
    pub available: bool,
    pub notice: bool,
}

/// The initial agent picker visibility, or `None` when a preference exists.
pub fn initial_agent_visibility(
    preference: Option<bool>,
    existing_profile: bool,
    version: Option<&str>,
) -> Option<bool> {
    if preference.is_some() {
        return None;
    }
    Some(existing_profile || version.is_some())
}

/// Whether blank profiles default to the new layout designs.
pub fn new_layout_designs_default() -> bool {
    true
}

/// The layout transition state for the current profile.
pub fn layout_transition_state(
    existing_profile: bool,
    has_toggle: bool,
    sunset_scheduled: bool,
    dismissed: bool,
) -> LayoutTransition {
    LayoutTransition {
        available: existing_profile && has_toggle && !sunset_scheduled,
        notice: sunset_scheduled && !dismissed,
    }
}

/// Whether a web profile has existing state.
pub fn has_existing_web_state(settings: Option<&str>, version: Option<&str>) -> bool {
    settings.is_some() || version.is_some()
}

/// Resolve the new-layout preference, honoring an explicit choice and sunset.
pub fn resolve_new_layout_designs(sunset: bool, preference: Option<bool>, default: bool) -> bool {
    if sunset {
        return true;
    }
    preference.unwrap_or(default)
}

/// The next sunset check delay, capped at the browser timeout limit.
pub fn next_sunset_check_delay(target: i64, now: i64) -> i64 {
    (target - now).clamp(0, MAXIMUM_SUNSET_TIMEOUT)
}

/// Whether the new layout should be enabled for this upgrade.
pub fn should_enable_new_layout(previous: Option<&str>, current: &str) -> bool {
    let Some(current) = parse_version(current) else {
        return false;
    };
    let Some(threshold) = parse_version("1.17.19") else {
        return false;
    };
    match previous {
        None => current > threshold,
        Some(previous) => {
            let Some(previous) = parse_version(previous) else {
                return false;
            };
            previous <= threshold && current > previous
        }
    }
}

/// Whether the app is upgrading from an older version.
pub fn is_app_upgrade(previous: Option<&str>, current: &str) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    let (Some(previous), Some(current)) = (parse_version(previous), parse_version(current)) else {
        return false;
    };
    current > previous
}

/// Whether the tabs toast should be shown.
pub fn should_display_tabs_toast(
    previous: Option<&str>,
    current: &str,
    existing_install: bool,
) -> bool {
    is_app_upgrade(previous, current) || (previous.is_none() && existing_install)
}

fn parse_version(value: &str) -> Option<Vec<u64>> {
    let value = value.strip_prefix(['v', 'V']).unwrap_or(value);
    let parts: Option<Vec<u64>> = value
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect();
    let parts = parts?;
    if parts.is_empty() {
        return None;
    }
    Some(parts)
}
