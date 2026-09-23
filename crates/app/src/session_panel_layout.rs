//! Session panel layout (port of packages/app/src/pages/session/session-panel-layout.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub visible: bool,
    pub stacked: bool,
}

pub fn session_panel_layout(review: bool, terminal: bool, files: bool) -> Layout {
    Layout {
        visible: review || terminal || files,
        stacked: review && terminal,
    }
}
