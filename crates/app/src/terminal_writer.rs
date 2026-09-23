//! Terminal output writer (port of packages/app/src/utils/terminal-writer.ts).
//!
//! The reference coalesces chunks across a scheduled microtask. Here the
//! scheduler is driven explicitly so the batching logic can be tested offline.

#[derive(Default)]
pub struct TerminalWriter {
    pub calls: Vec<String>,
    pub scheduled: Vec<bool>,
    chunks: Vec<String>,
}

impl TerminalWriter {
    pub fn push(&mut self, data: &str) {
        if data.is_empty() {
            return;
        }
        self.chunks.push(data.to_string());
        if self.scheduled.is_empty() {
            self.scheduled.push(true);
        }
    }

    pub fn flush(&mut self) {
        if self.chunks.is_empty() {
            return;
        }
        let joined = self.chunks.join("");
        self.chunks.clear();
        self.calls.push(joined);
        self.scheduled.clear();
    }

    pub fn run_scheduled(&mut self) {
        if !self.chunks.is_empty() {
            let joined = self.chunks.join("");
            self.chunks.clear();
            self.calls.push(joined);
        }
        self.scheduled.clear();
    }
}
