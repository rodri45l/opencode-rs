//! Home session click behaviour (port of packages/app/src/pages/home-session-open.ts).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Click {
    pub button: i64,
    pub mac: bool,
    pub meta: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

pub fn should_open_session_in_background(click: Click) -> bool {
    if click.button == 1 {
        return true;
    }
    if click.button != 0 {
        return false;
    }
    if click.shift || click.alt {
        return false;
    }
    if click.mac {
        return click.meta && !click.ctrl;
    }
    click.ctrl && !click.meta
}
