//! Reference-counted map (port of packages/app/src/utils/refcount.ts).
//!
//! The reference ties lifetime to SolidJS owners; here we expose the same
//! acquire/release accounting directly, keyed through [`path_key`].

use std::collections::HashMap;

use crate::path_key::path_key;

#[derive(Default)]
pub struct RefCountMap {
    counts: HashMap<String, usize>,
    removed: Vec<String>,
}

impl RefCountMap {
    pub fn acquire(&mut self, key: &str) {
        let id = path_key(key);
        *self.counts.entry(id).or_insert(0) += 1;
    }

    pub fn release(&mut self, key: &str) {
        let id = path_key(key);
        match self.counts.get_mut(&id) {
            Some(count) if *count > 1 => *count -= 1,
            Some(_) => {
                self.counts.remove(&id);
                self.removed.push(id);
            }
            None => {}
        }
    }

    pub fn removed(&self) -> &[String] {
        &self.removed
    }
}
