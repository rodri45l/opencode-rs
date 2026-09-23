//! Scoped value cache with LRU + TTL eviction
//! (port of packages/app/src/utils/scoped-cache.ts).

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    pub count: i64,
}

/// A small cache that lazily creates values, evicts the least-recently-used
/// entry when `max_entries` is exceeded, and expires entries older than
/// `ttl_ms`. Entries are stored in LRU order (front is least recent).
pub struct ScopedCache {
    max_entries: Option<usize>,
    ttl_ms: Option<i64>,
    clock: i64,
    counter: i64,
    disposed: Vec<String>,
    store: Vec<(String, Entry, i64)>,
}

impl ScopedCache {
    pub fn new(max_entries: Option<usize>, ttl_ms: Option<i64>) -> Self {
        ScopedCache {
            max_entries,
            ttl_ms,
            clock: 0,
            counter: 0,
            disposed: Vec::new(),
            store: Vec::new(),
        }
    }

    fn expired(&self, touched_at: i64) -> bool {
        match self.ttl_ms {
            Some(ttl) => self.clock - touched_at >= ttl,
            None => false,
        }
    }

    fn record_dispose(&mut self, entry: &Entry) {
        if self.ttl_ms.is_some() {
            self.disposed.push(format!("{}:{}", entry.key, entry.count));
        } else {
            self.disposed.push(entry.key.clone());
        }
    }

    fn sweep(&mut self) {
        if self.ttl_ms.is_none() {
            return;
        }
        let mut expired = Vec::new();
        self.store.retain(|(key, entry, touched_at)| {
            if self.clock - *touched_at >= self.ttl_ms.unwrap() {
                expired.push((key.clone(), entry.clone()));
                false
            } else {
                true
            }
        });
        for (_, entry) in expired {
            self.record_dispose(&entry);
        }
    }

    fn position(&self, key: &str) -> Option<usize> {
        self.store.iter().position(|(k, _, _)| k == key)
    }

    pub fn get(&mut self, key: &str) -> Entry {
        self.sweep();
        if let Some(index) = self.position(key) {
            let (_, entry, touched_at) = self.store.remove(index);
            if !self.expired(touched_at) {
                let value = entry.clone();
                self.store.push((key.to_string(), entry, self.clock));
                return value;
            }
            self.record_dispose(&entry);
        }

        self.counter += 1;
        let entry = Entry {
            key: key.to_string(),
            count: self.counter,
        };
        self.store
            .push((key.to_string(), entry.clone(), self.clock));
        self.prune();
        entry
    }

    pub fn peek(&mut self, key: &str) -> Option<Entry> {
        self.sweep();
        self.store
            .iter()
            .find(|(k, _, _)| k == key)
            .map(|(_, entry, _)| entry.clone())
    }

    pub fn delete(&mut self, key: &str) -> Option<Entry> {
        let index = self.position(key)?;
        let (_, entry, _) = self.store.remove(index);
        self.record_dispose(&entry);
        Some(entry)
    }

    pub fn clear(&mut self) {
        let drained: Vec<Entry> = self.store.drain(..).map(|(_, entry, _)| entry).collect();
        for entry in drained {
            self.record_dispose(&entry);
        }
    }

    fn prune(&mut self) {
        let max = match self.max_entries {
            Some(max) => max,
            None => return,
        };
        while self.store.len() > max {
            let (_, entry, _) = self.store.remove(0);
            self.record_dispose(&entry);
        }
    }

    pub fn set_clock(&mut self, clock: i64) {
        self.clock = clock;
    }

    pub fn disposed(&self) -> &[String] {
        &self.disposed
    }
}
