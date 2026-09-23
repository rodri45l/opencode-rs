//! Port of packages/opencode/test/cli/run/footer.menu.test.ts (upstream 18ef3cc).
//!
//! RED-first: the run-footer menu scroll model in `cli/cmd/run/footer.menu` is
//! not implemented in this crate. The scroll-before-edge behaviour is pinned
//! against a local typed stub.

#![allow(dead_code)]

const FOOTER_MENU_ROWS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FooterMenuState {
    count: usize,
    limit: usize,
    selected: usize,
    offset: usize,
}

impl FooterMenuState {
    fn move_by(&mut self, delta: i32) {
        if self.count == 0 || self.limit == 0 {
            return;
        }
        let last = self.count - 1;
        let next = (self.selected as i64 + delta as i64).clamp(0, last as i64) as usize;
        self.selected = next;
        if self.limit > 2 {
            while self.selected.saturating_sub(self.offset) >= self.limit - 2 {
                self.offset += 1;
            }
        }
        while self.offset > 0 && self.selected <= self.offset + 1 {
            self.offset -= 1;
        }
    }
    fn selected(&self) -> usize {
        self.selected
    }
    fn offset(&self) -> usize {
        self.offset
    }
}

fn create_footer_menu_state(count: usize, limit: usize) -> FooterMenuState {
    FooterMenuState {
        count,
        limit,
        selected: 0,
        offset: 0,
    }
}

#[test]
fn scrolls_before_the_selected_row_hits_the_bottom_edge() {
    let mut menu = create_footer_menu_state(20, FOOTER_MENU_ROWS);
    for _ in 0..6 {
        menu.move_by(1);
    }
    assert_eq!(menu.selected(), 6);
    assert_eq!(menu.offset(), 1);
}

#[test]
fn scrolls_before_the_selected_row_hits_the_top_edge() {
    let mut menu = create_footer_menu_state(20, FOOTER_MENU_ROWS);
    for _ in 0..13 {
        menu.move_by(1);
    }
    for _ in 0..4 {
        menu.move_by(-1);
    }
    assert_eq!(menu.selected(), 9);
    assert_eq!(menu.offset(), 7);
}
