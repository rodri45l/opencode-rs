//! Model option ordering.
//!
//! Port of packages/tui/src/component/dialog-model.tsx `sortModelOptions`
//! behaviour (upstream 18ef3cc).

/// A sortable model option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOption {
    pub title: String,
    pub release_date: String,
    pub footer: Option<String>,
}

/// Sort model options, either newest-first or free-first then newest-first.
pub fn sort_model_options(mut options: Vec<ModelOption>, newest_first: bool) -> Vec<ModelOption> {
    options.sort_by(|left, right| {
        if !newest_first {
            let left_free = left.footer.as_deref() != Some("Free");
            let right_free = right.footer.as_deref() != Some("Free");
            let ordering = left_free.cmp(&right_free);
            if ordering != std::cmp::Ordering::Equal {
                return ordering;
            }
        }
        right
            .release_date
            .cmp(&left.release_date)
            .then_with(|| left.title.cmp(&right.title))
    });
    options
}
