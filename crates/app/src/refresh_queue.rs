//! Directory refresh queue (port of packages/app/src/context/global-sync/queue.ts).

use std::collections::BTreeMap;

use crate::global_sync_utils::directory_key;

#[derive(Default)]
pub struct RefreshQueue {
    queued: BTreeMap<String, String>,
    pub calls: Vec<String>,
}

impl RefreshQueue {
    pub fn push(&mut self, directory: &str) {
        if directory.is_empty() {
            return;
        }
        self.queued
            .insert(directory_key(directory), directory.to_string());
    }

    pub fn clear(&mut self, directory: &str) {
        self.queued.remove(&directory_key(directory));
    }

    pub fn flush(&mut self) {
        for (_, directory) in std::mem::take(&mut self.queued) {
            self.calls.push(directory);
        }
    }

    pub fn dispose(&mut self) {
        self.queued.clear();
    }
}
