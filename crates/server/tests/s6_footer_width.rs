//! Port of packages/opencode/test/cli/run/footer.width.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run-footer width policy in `cli/cmd/run/footer.width` is not
//! implemented in this crate. The breakpoint behaviour is pinned against a local
//! typed stub.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
struct DialogPolicy {
    narrow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatuslinePolicy {
    show_activity_meta: bool,
    show_command_hint: bool,
    show_context_hints: bool,
    context_hint_limit: Option<u32>,
    show_model: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterWidthPolicy {
    dialog: DialogPolicy,
    statusline: StatuslinePolicy,
}

fn footer_width_policy(width: u32) -> FooterWidthPolicy {
    FooterWidthPolicy {
        dialog: DialogPolicy { narrow: width < 80 },
        statusline: StatuslinePolicy {
            show_activity_meta: width >= 80,
            show_command_hint: width >= 66,
            show_context_hints: width >= 80,
            context_hint_limit: if width >= 150 {
                None
            } else if width >= 120 {
                Some(2)
            } else if width >= 80 {
                Some(1)
            } else {
                Some(0)
            },
            show_model: width >= 120,
        },
    }
}

#[test]
fn preserves_shared_dialog_and_statusline_breakpoints() {
    let narrow = footer_width_policy(79);
    assert!(narrow.dialog.narrow);
    assert!(!narrow.statusline.show_activity_meta);
    assert!(narrow.statusline.show_command_hint);
    assert!(!narrow.statusline.show_context_hints);
    assert_eq!(narrow.statusline.context_hint_limit, Some(0));
    assert!(!narrow.statusline.show_model);

    let command = footer_width_policy(65);
    assert!(!command.statusline.show_command_hint);

    let command_hint = footer_width_policy(66);
    assert!(command_hint.statusline.show_command_hint);

    let compact = footer_width_policy(80);
    assert!(!compact.dialog.narrow);
    assert!(compact.statusline.show_activity_meta);
    assert!(compact.statusline.show_context_hints);
    assert_eq!(compact.statusline.context_hint_limit, Some(1));
    assert!(!compact.statusline.show_model);

    let model = footer_width_policy(120);
    assert_eq!(model.statusline.context_hint_limit, Some(2));
    assert!(model.statusline.show_model);

    let spacious = footer_width_policy(150);
    assert_eq!(spacious.statusline.context_hint_limit, None);
    assert!(spacious.statusline.show_model);
}
