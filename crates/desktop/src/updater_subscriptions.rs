//! Per-renderer updater subscriptions.
//!
//! Port of `packages/desktop/src/main/updater-subscriptions.ts`
//! (upstream 18ef3cc): setting a subscription for a renderer disposes the
//! previous one, and deleting disposes the current one.

/// A set of per-renderer updater subscriptions.
#[derive(Default)]
pub struct UpdaterSubscriptions {
    subscriptions: Vec<(u64, Box<dyn FnMut()>)>,
}

impl UpdaterSubscriptions {
    fn remove(&mut self, renderer: u64) {
        if let Some(index) = self
            .subscriptions
            .iter()
            .position(|(id, _)| *id == renderer)
        {
            let (_, mut dispose) = self.subscriptions.remove(index);
            dispose();
        }
    }

    /// Replace the subscription for a renderer, disposing the previous one.
    pub fn set(&mut self, renderer: u64, dispose: Box<dyn FnMut()>) {
        self.remove(renderer);
        self.subscriptions.push((renderer, dispose));
    }

    /// Dispose the subscription for a renderer.
    pub fn delete(&mut self, renderer: u64) {
        self.remove(renderer);
    }
}

/// Create an empty subscription set.
pub fn create_updater_subscriptions() -> UpdaterSubscriptions {
    UpdaterSubscriptions::default()
}
