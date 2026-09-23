//! Line comment session state (port of packages/app/src/context/comments.tsx).

use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct LineComment {
    pub id: String,
    pub file: String,
    pub comment: String,
    pub time: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Focus {
    pub file: String,
    pub id: String,
}

#[derive(Default)]
pub struct CommentSession {
    pub files: BTreeMap<String, Vec<LineComment>>,
    focus: Option<Focus>,
    active: Option<Focus>,
    counter: u64,
}

impl CommentSession {
    pub fn list(&self, file: &str) -> Vec<LineComment> {
        self.files.get(file).cloned().unwrap_or_default()
    }

    pub fn all(&self) -> Vec<LineComment> {
        let mut all: Vec<LineComment> = self
            .files
            .values()
            .flat_map(|comments| comments.iter().cloned())
            .collect();
        all.sort_by_key(|comment| comment.time);
        all
    }

    pub fn add(&mut self, file: &str, comment: &str) -> LineComment {
        self.counter += 1;
        let time = self
            .files
            .values()
            .flat_map(|comments| comments.iter())
            .map(|comment| comment.time)
            .max()
            .unwrap_or(0)
            + 1;
        let next = LineComment {
            id: format!("comment-{}", self.counter),
            file: file.to_string(),
            comment: comment.to_string(),
            time,
        };
        self.files
            .entry(file.to_string())
            .or_default()
            .push(next.clone());
        self.focus = Some(Focus {
            file: file.to_string(),
            id: next.id.clone(),
        });
        next
    }

    pub fn remove(&mut self, file: &str, id: &str) {
        if let Some(comments) = self.files.get_mut(file) {
            comments.retain(|comment| comment.id != id);
        }
        if self
            .focus
            .as_ref()
            .map(|focus| focus.file == file && focus.id == id)
            .unwrap_or(false)
        {
            self.focus = None;
        }
    }

    pub fn clear(&mut self) {
        self.files.clear();
        self.focus = None;
        self.active = None;
    }

    pub fn update(&mut self, file: &str, id: &str, comment: &str) {
        if let Some(comments) = self.files.get_mut(file) {
            for item in comments.iter_mut() {
                if item.id == id {
                    item.comment = comment.to_string();
                }
            }
        }
    }

    pub fn replace(&mut self, comments: Vec<LineComment>) {
        self.files.clear();
        for comment in comments {
            self.files
                .entry(comment.file.clone())
                .or_default()
                .push(comment);
        }
        self.focus = None;
        self.active = None;
    }

    pub fn set_focus(&mut self, focus: Option<Focus>) {
        self.focus = focus;
    }

    pub fn set_active(&mut self, active: Option<Focus>) {
        self.active = active;
    }

    pub fn focus(&self) -> Option<Focus> {
        self.focus.clone()
    }

    pub fn active(&self) -> Option<Focus> {
        self.active.clone()
    }
}
