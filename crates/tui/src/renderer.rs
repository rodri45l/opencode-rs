//! Renderer teardown helper.
//!
//! Port of packages/tui/src/util/renderer.ts `destroyRenderer` (upstream 18ef3cc).

/// Clear the terminal title, then destroy the renderer unless already destroyed.
pub fn destroy_renderer<SetTitle, Destroy>(
    is_destroyed: bool,
    mut set_title: SetTitle,
    mut destroy: Destroy,
) where
    SetTitle: FnMut(&str),
    Destroy: FnMut(),
{
    set_title("");
    if is_destroyed {
        return;
    }
    destroy();
}
