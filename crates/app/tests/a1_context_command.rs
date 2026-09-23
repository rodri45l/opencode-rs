//! Port of packages/app/src/context/command.test.ts (upstream 18ef3cc).
//! Behaviour pinned by the reference test; see docs/TEST-PORT.md.
#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq)]
struct CommandOption {
    id: String,
    title: String,
    hidden: bool,
    disabled: bool,
    when: Option<bool>,
}

#[derive(Clone, Debug, PartialEq)]
struct Registration {
    key: Option<String>,
    options: Vec<CommandOption>,
}

fn option(id: &str, title: &str) -> CommandOption {
    CommandOption {
        id: id.into(),
        title: title.into(),
        hidden: false,
        disabled: false,
        when: None,
    }
}

// Local stubs (fast wave): real module lands later.
fn command_palette_options(_options: &[CommandOption]) -> Vec<CommandOption> {
    Vec::new()
}

fn add_command_registration(
    _registrations: Vec<Registration>,
    _next: Registration,
) -> Vec<Registration> {
    Vec::new()
}

fn active_command_registrations(_registrations: &[Registration]) -> Vec<Registration> {
    Vec::new()
}

fn resolve_keybind_option(_options: &[CommandOption]) -> Option<CommandOption> {
    None
}

fn palette_options() -> Vec<CommandOption> {
    let mut hidden = option("hidden", "Hidden");
    hidden.hidden = true;
    let mut disabled = option("disabled", "Disabled");
    disabled.disabled = true;
    vec![
        option("settings.open", "Open settings"),
        option("session.undo", "Undo"),
        option("file.open", "Open file"),
        hidden,
        disabled,
    ]
}

#[test]
#[ignore = "porting: context/command not implemented"]
fn keeps_visible_enabled_commands() {
    assert_eq!(
        command_palette_options(&palette_options())
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>(),
        vec!["settings.open".to_string(), "session.undo".to_string()]
    );
}

#[test]
#[ignore = "porting: context/command not implemented"]
fn shadows_keyed_registrations_while_retaining_the_previous_owner() {
    let one = Registration {
        key: Some("layout".into()),
        options: vec![option("one", "One")],
    };
    let two = Registration {
        key: Some("layout".into()),
        options: vec![option("two", "Two")],
    };
    let registrations = add_command_registration(vec![one.clone()], two.clone());
    let active = active_command_registrations(&registrations);
    assert_eq!(registrations.len(), 2);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].options, two.options);

    let restored = active_command_registrations(std::slice::from_ref(&one));
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].options, one.options);
}

#[test]
#[ignore = "porting: context/command not implemented"]
fn keeps_unkeyed_registrations_additive() {
    let one = Registration {
        key: None,
        options: vec![option("one", "One")],
    };
    let two = Registration {
        key: None,
        options: vec![option("two", "Two")],
    };
    let next = active_command_registrations(&add_command_registration(vec![one], two));
    assert_eq!(next.len(), 2);
}

#[test]
#[ignore = "porting: context/command not implemented"]
fn prefers_a_matching_contextual_command_over_the_global_fallback() {
    let fallback = option("tab.close", "Close tab");
    let mut contextual = option("terminal.close", "Close terminal");
    contextual.when = Some(true);
    let resolved = resolve_keybind_option(&[fallback, contextual.clone()]);
    assert_eq!(resolved, Some(contextual));
}

#[test]
#[ignore = "porting: context/command not implemented"]
fn uses_the_global_fallback_outside_the_command_context() {
    let fallback = option("tab.close", "Close tab");
    let mut contextual = option("terminal.close", "Close terminal");
    contextual.when = Some(false);
    let resolved = resolve_keybind_option(&[fallback.clone(), contextual]);
    assert_eq!(resolved, Some(fallback));
}
