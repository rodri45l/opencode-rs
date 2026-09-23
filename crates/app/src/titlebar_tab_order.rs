//! Titlebar tab ordering (port of packages/app/src/components/titlebar-tab-order.ts).

use std::collections::HashSet;

pub fn adjacent_tab_key(order: &[&str], current: &str, delta: i64) -> Option<String> {
    if current.is_empty() || order.is_empty() {
        return None;
    }
    let index = order.iter().position(|tab| *tab == current)?;
    let len = order.len() as i64;
    let next = ((index as i64 + delta + len) % len) as usize;
    order.get(next).map(|tab| (*tab).to_string())
}

pub fn merge_visible_tab_order(all: &[&str], visible: &[&str], reordered: &[&str]) -> Vec<String> {
    let visible: HashSet<&str> = visible.iter().copied().collect();
    let mut next = reordered.iter();
    all.iter()
        .map(|key| {
            if visible.contains(key) {
                next.next()
                    .map(|value| (*value).to_string())
                    .unwrap_or_else(|| (*key).to_string())
            } else {
                (*key).to_string()
            }
        })
        .collect()
}
