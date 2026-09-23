//! Debounced scroll persistence (port of packages/app/src/context/layout-scroll.ts).
//!
//! The reference debounces through `setTimeout`; here time is advanced
//! explicitly via [`ScrollPersistence::advance`].

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

pub type Snapshot = BTreeMap<String, BTreeMap<String, Point>>;

pub struct ScrollPersistence {
    pub snapshot: Snapshot,
    pub writes: Vec<Snapshot>,
    cache: Snapshot,
    dirty: bool,
    last_change: Option<i64>,
    clock: i64,
    debounce_ms: i64,
}

impl ScrollPersistence {
    pub fn new(snapshot: Snapshot, debounce_ms: i64) -> Self {
        ScrollPersistence {
            snapshot,
            writes: Vec::new(),
            cache: Snapshot::new(),
            dirty: false,
            last_change: None,
            clock: 0,
            debounce_ms,
        }
    }

    fn seed(&mut self, session: &str) {
        let next = self.snapshot.get(session).cloned().unwrap_or_default();
        match self.cache.get(session) {
            None => {
                self.cache.insert(session.to_string(), next);
            }
            Some(current) if current.is_empty() && !next.is_empty() => {
                self.cache.insert(session.to_string(), next);
            }
            _ => {}
        }
    }

    pub fn set_scroll(&mut self, session: &str, key: &str, point: Point) {
        self.seed(session);
        let previous = self
            .cache
            .get(session)
            .and_then(|inner| inner.get(key))
            .copied();
        if previous == Some(point) {
            return;
        }
        self.cache
            .entry(session.to_string())
            .or_default()
            .insert(key.to_string(), point);
        self.dirty = true;
        self.last_change = Some(self.clock);
    }

    pub fn advance(&mut self, ms: i64) {
        self.clock += ms;
        if self.dirty
            && self
                .last_change
                .map(|last| self.clock - last >= self.debounce_ms)
                .unwrap_or(false)
        {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        self.writes.push(self.cache.clone());
    }

    pub fn scroll(&mut self, session: &str, key: &str) -> Option<Point> {
        self.seed(session);
        self.cache
            .get(session)
            .and_then(|inner| inner.get(key))
            .copied()
            .or_else(|| {
                self.snapshot
                    .get(session)
                    .and_then(|inner| inner.get(key))
                    .copied()
            })
    }

    pub fn dispose(&mut self) {
        self.dirty = false;
        self.last_change = None;
    }
}
