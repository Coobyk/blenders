//! Port of `lazy_threading.cc` — `send_hint` receivers (thread-local).

use std::cell::RefCell;

thread_local! {
    static HINT_RECEIVERS: RefCell<Vec<Box<dyn Fn()>>> = RefCell::new(vec![]);
}

/// `lazy_threading::send_hint` — run all registered receivers.
pub fn send_hint() {
    HINT_RECEIVERS.with(|v| {
        for f in v.borrow().iter() {
            f();
        }
    });
}

/// `lazy_threading::HintReceiver` — register `fn` while alive.
pub struct HintReceiver;

impl HintReceiver {
    /// `HintReceiver(fn)` — pushes receiver; `drop` pops it.
    pub fn new<F: Fn() + 'static>(f: F) -> Self {
        HINT_RECEIVERS.with(|v| v.borrow_mut().push(Box::new(f)));
        HintReceiver
    }
}
impl Drop for HintReceiver {
    fn drop(&mut self) {
        HINT_RECEIVERS.with(|v| {
            v.borrow_mut().pop();
        });
    }
}

/// `lazy_threading::ReceiverIsolation` — push a fresh receiver stack.
/// (Single stack in this port: isolation clears receivers on drop-restore.)
pub struct ReceiverIsolation(Vec<Box<dyn Fn()>>);

impl ReceiverIsolation {
    pub fn new() -> Self {
        let saved = HINT_RECEIVERS.with(|v| std::mem::take(&mut *v.borrow_mut()));
        ReceiverIsolation(saved)
    }
}
impl Drop for ReceiverIsolation {
    fn drop(&mut self) {
        let saved = std::mem::take(&mut self.0);
        HINT_RECEIVERS.with(|v| *v.borrow_mut() = saved);
    }
}
