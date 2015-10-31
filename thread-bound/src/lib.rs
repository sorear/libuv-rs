#![deny(missing_docs)]
//! Module for tracking the current thread, and wrapping non-`Send` objects in
//! guards that perform runtime thread checks.

use std::sync::Arc;
use std::fmt::{self, Debug};

/// A permanent identifier for a single thread, which will not be reused.
#[derive(Clone)]
pub struct ThreadId(Arc<()>);

impl PartialEq for ThreadId {
    fn eq(&self, other: &Self) -> bool {
        (&*self.0 as *const ()) == (&*other.0 as *const ())
    }
}
impl Eq for ThreadId {}

impl Debug for ThreadId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "thread({:?})", &*self.0 as *const ())
    }
}

// consider using https://github.com/rust-lang/rust/pull/29447 after it lands,
// or a u64 atomic where available
// is there a lock-free way to increment a 64-bit counter?
thread_local!(static THREAD_NONCE: ThreadId = ThreadId(Arc::new(())));

/// Returns a persistent identifier for the current thread.
pub fn current_thread_id() -> ThreadId {
    THREAD_NONCE.with(|r| r.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn same_thread_id() {
        let id1 = current_thread_id();
        let id2 = current_thread_id();

        assert_eq!(id1, id2);
    }

    #[test]
    fn different_thread_id() {
        let id1 = current_thread_id();
        let id2 = thread::spawn(|| current_thread_id()).join().unwrap();

        assert!(id1 != id2);
    }
}
