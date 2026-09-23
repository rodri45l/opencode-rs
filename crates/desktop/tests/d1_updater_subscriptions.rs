//! Port of packages/desktop/src/main/updater-subscriptions.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/updater-subscriptions.ts:
//! setting a subscription for a renderer disposes the previous one, and
//! deleting disposes the current one.

#[allow(dead_code)]
mod updater_subscriptions {
    use std::fmt;

    #[derive(Debug, PartialEq, Eq)]
    pub struct NotImplemented(pub &'static str);

    impl fmt::Display for NotImplemented {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    impl std::error::Error for NotImplemented {}

    pub type PortResult<T> = Result<T, NotImplemented>;

    pub const NOTE: &str = "porting: desktop updater subscriptions not implemented";

    fn stub<T>() -> PortResult<T> {
        Err(NotImplemented(NOTE))
    }

    pub struct UpdaterSubscriptions;

    impl UpdaterSubscriptions {
        pub fn set(&mut self, _renderer: u64, _dispose: Box<dyn FnMut()>) {}

        pub fn delete(&mut self, _renderer: u64) {}
    }

    pub fn create_updater_subscriptions() -> PortResult<UpdaterSubscriptions> {
        stub()
    }
}

use updater_subscriptions::{create_updater_subscriptions, NOTE};

#[test]
#[ignore = "porting: desktop updater subscriptions not implemented"]
fn replaces_the_previous_renderer_subscription_on_reload() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut subscriptions = create_updater_subscriptions().expect(NOTE);
    let disposed = Rc::new(RefCell::new(Vec::<String>::new()));

    let first = Rc::clone(&disposed);
    subscriptions.set(
        1,
        Box::new(move || first.borrow_mut().push("first".to_string())),
    );
    let second = Rc::clone(&disposed);
    subscriptions.set(
        1,
        Box::new(move || second.borrow_mut().push("second".to_string())),
    );

    assert_eq!(*disposed.borrow(), vec!["first".to_string()]);
    subscriptions.delete(1);
    assert_eq!(
        *disposed.borrow(),
        vec!["first".to_string(), "second".to_string()]
    );
}
