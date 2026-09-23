//! Port of packages/desktop/src/main/updater-subscriptions.test.ts (upstream 18ef3cc).
//! Behaviour pinned by packages/desktop/src/main/updater-subscriptions.ts:
//! setting a subscription for a renderer disposes the previous one, and
//! deleting disposes the current one.

use opencode_desktop::updater_subscriptions::create_updater_subscriptions;

#[test]
fn replaces_the_previous_renderer_subscription_on_reload() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut subscriptions = create_updater_subscriptions();
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
