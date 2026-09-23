//! Worktree directory state (port of packages/app/src/utils/worktree.ts).

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum WorktreeState {
    Pending,
    Ready,
    Failed(String),
}

#[derive(Default)]
pub struct Worktree {
    states: HashMap<(String, String), WorktreeState>,
}

fn normalize(directory: &str) -> String {
    directory.trim_end_matches(['/', '\\']).to_string()
}

impl Worktree {
    fn key(scope: &str, directory: &str) -> (String, String) {
        (scope.to_string(), normalize(directory))
    }

    pub fn get(&self, scope: &str, directory: &str) -> Option<WorktreeState> {
        self.states.get(&Self::key(scope, directory)).cloned()
    }

    pub fn pending(&mut self, scope: &str, directory: &str) {
        let key = Self::key(scope, directory);
        if let Some(current) = self.states.get(&key) {
            if *current != WorktreeState::Pending {
                return;
            }
        }
        self.states.insert(key, WorktreeState::Pending);
    }

    pub fn ready(&mut self, scope: &str, directory: &str) {
        self.states
            .insert(Self::key(scope, directory), WorktreeState::Ready);
    }

    pub fn failed(&mut self, scope: &str, directory: &str, message: &str) {
        self.states.insert(
            Self::key(scope, directory),
            WorktreeState::Failed(message.to_string()),
        );
    }
}
