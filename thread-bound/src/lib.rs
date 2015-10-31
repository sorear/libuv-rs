#![deny(missing_docs)]
//! Module for tracking the current thread, and wrapping non-`Send` objects in
//! guards that perform runtime thread checks.

use std::sync::Arc;
use std::fmt::{self, Debug};
use std::ptr;
use std::mem;
use std::cell::Cell;

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

struct TBListHead {
    // do not move
    owner: ThreadId,
    head: TBHeader,
    destroying: Cell<bool>,
}

/// A container for values which must be accessed within a particular scope in a particular thread.
/// Values may be added using the `bind_value` method, which returns a capability; the capability
/// may freely move between threads, but can only be used in the thread of the binding.  A bound
/// value will be destroyed when the capability is destroyed if that happens in the allowed thread;
/// otherwise it will be destroyed when the list is.
pub struct TBList {
    list: Arc<TBListHead>,
}

struct TBHeader {
    // do not move
    next: Cell<*const TBHeader>,
    prev: Cell<*const TBHeader>,
    free: unsafe fn(*const TBHeader),
}

struct TBVar<T: 'static> {
    // do not move
    value: T,
    header: TBHeader,
}

/// A handle to a value of type `T` which is owned by a specific thread.  Code running on the same
/// thread can freely manipulate the value; attempts to manipulate the value from other threads
/// will panic at runtime.
///
/// Lifetime is subtle.  Since we do not assume `T` can be safely dropped from any thread other
/// than that where it was created, dropping a `ThreadBound<T>` will only immediately drop the
/// wrapped value in the originating thread.  Drops occuring on other threads will leave the value
/// in a thread-local list associated with the originating thread, and the value will live until
/// the originating thread exits, at which point it will be destroyed. Conversely, if the
/// originating thread exits while there are `ThreadBound<T>` capabilities alive in the heap, all
/// of the underlying values will be dropped (this causes the capabilities to become dangling
/// pointers, but that is OK as the only thread that could dereference them is exiting).
pub struct ThreadBound<T: 'static> {
    list: Arc<TBListHead>,
    var: *mut TBVar<T>,
}

unsafe impl<T: 'static> Send for ThreadBound<T> {}
unsafe impl<T: 'static> Sync for ThreadBound<T> {}

unsafe fn unlink_var(header: *const TBHeader) {
    // precondition: the list is being destroyed or the capability has just been
    // dropped
    // (either way there will be no future attempts to access this header)
    // println!("unlink header {:?}", header);
    let prev = (*header).prev.get();
    let next = (*header).next.get();
    (*next).prev.set(prev);
    (*prev).next.set(next);
}

// unsafe fn list_dumper(why: &'static str, lh: &TBListHead) {
//     let pstart = &lh.head as *const TBHeader;
//     let mut p2 = pstart;
//     println!("list dump {}", why);
//     loop {
// println!("header@{:?}, next={next:?}, prev={prev:?}, free={free:?}",
// p2,
// next = (*p2).next.get(), prev = (*p2).prev.get(), free =
// (*p2).free);
//         p2 = (*p2).next.get();
//         if p2 == pstart { break; }
//     }
// }

unsafe fn destroy_var(header: *const TBHeader) {
    // println!("destroy header {:?}", header);
    unlink_var(header);
    ((*header).free)(header);
}

impl Drop for TBList {
    fn drop(&mut self) {
        self.list.destroying.set(true);
        let headp = &self.list.head as *const TBHeader;

        unsafe {
            // list_dumper("A",&*self.list);
            while (*headp).next.get() != headp {
                destroy_var((*headp).next.get());
            }
        }
    }
}

impl<T: 'static> Drop for ThreadBound<T> {
    fn drop(&mut self) {
        if self.list.owner == current_thread_id() && !self.list.destroying.get() {
            unsafe {
                // list_dumper("B", &*self.list);
                destroy_var(&(*self.var).header);
            }
        }
        // otherwise the var leaks and will be cleaned up when the originating thread
        // exits.  we could provide a scrub() function to call from the originating
        // thread, at the cost of extra synchronization.
    }
}

impl TBList {
    fn new() -> Self {
        unsafe fn unreachable(_head: *const TBHeader) {
            panic!("attempt to free TBList header");
        }
        let lheader = Arc::new(TBListHead {
            owner: current_thread_id(),
            destroying: Cell::new(false),
            head: TBHeader {
                next: Cell::new(ptr::null_mut()),
                prev: Cell::new(ptr::null_mut()),
                free: unreachable,
            },
        });
        let headp = &lheader.head as *const TBHeader;
        // println!("insert header {:?}", headp);
        unsafe {
            (*headp).next.set(headp);
            (*headp).prev.set(headp);
            // list_dumper("C", &*lheader);
        }
        TBList { list: lheader }
    }

    /// Creates a new scope for value bindings.  The lifetime of a scope list is forced to be a
    /// dynamic scope so that it will be properly nested with all `with` and `with_mut` borrows.
    pub fn scope<F, R>(f: F) -> R
        where F: FnOnce(&mut Self) -> R
    {
        f(&mut Self::new())
    }

    /// Transfers ownership of a value into a scope, and returns a capability which can be used to
    /// access that value but only within the same thread and during the same scope.
    pub fn bind<T: 'static>(&self, value: T) -> ThreadBound<T> {
        unsafe fn free<T: 'static>(header: *const TBHeader) {
            let offset = &(*(0 as *const TBVar<T>)).header as *const TBHeader as usize;
            let varp = ((header as usize) - offset) as *mut TBVar<T>;
            // println!("free/box {:?}", varp);
            mem::drop(Box::from_raw(varp));
        }

        let new_var: *mut TBVar<T> = Box::into_raw(Box::new(TBVar {
            header: TBHeader {
                next: Cell::new(ptr::null_mut()),
                prev: Cell::new(ptr::null_mut()),
                free: free::<T>,
            },
            value: value,
        }));

        unsafe {
            let new_head = &(*new_var).header as *const TBHeader;
            let list_head = &self.list.head as *const TBHeader;

            (*new_head).next.set(list_head);
            (*new_head).prev.set((*list_head).prev.get());
            (*(*list_head).prev.get()).next.set(new_head);
            (*list_head).prev.set(new_head);
            // list_dumper("D", &*self.list);
        }

        ThreadBound {
            var: new_var,
            list: self.list.clone(),
        }
    }
}

thread_local!(static THREAD_LIST_KEY: TBList = TBList::new());

impl<T: 'static> ThreadBound<T> {
    /// Wraps a value of type `T` in a capability bound to the current thread.  This uses a
    /// thread-local instance of `TBList` with the lifetime of the calling thread.
    pub fn new(inner: T) -> Self {
        THREAD_LIST_KEY.with(|tl| tl.bind(inner))
    }

    /// Returns true if this capability can be used without panicking.  This will remain true on
    /// the same thread as long as the thread does not enter the TLS destruction phase.
    pub fn accessible(&self) -> bool {
        return self.list.owner == current_thread_id() && !self.list.destroying.get();
    }

    fn check_access(&self) {
        if self.list.owner != current_thread_id() {
            panic!("Attempt to access ThreadBound from incorrect thread");
        }

        if self.list.destroying.get() {
            panic!("Attempt to access ThreadBound during TLS destruction phase");
        }
    }

    /// Consumes this capability to regain ownership of the underlying value.
    ///
    /// # Panics
    ///
    /// Panics if the capability is not accessible.
    pub fn into_inner(self) -> T {
        self.check_access();
        unsafe {
            unlink_var(&(*self.var).header as *const TBHeader);
            // yes, this is a move, so it invalidates the next/prev
            // println!("from_raw {:?}",self.var);
            let var_guts: TBVar<T> = *Box::from_raw(self.var);
            mem::forget(self); // no double free
            var_guts.value
        }
    }

    /// Temporarily acquires shared access to the guarded value.  It would not be safe to return a
    /// reference as by the standard `Deref` traits, as the lifetime of the capability could be
    /// much longer than the usable life of the referred value.  By only exposing borrowing and
    /// scoping as functions, we force the lifetimes of values and capabilities to be properly
    /// nested with each other in a thread, so a single liveness check at the beginning can
    /// suffice.
    ///
    /// # Panics
    ///
    /// Panics if the capability is not accessible.
    pub fn with<F, R>(&self, fun: F) -> R
        where F: FnOnce(&T) -> R
    {
        self.check_access();
        fun(unsafe { &(*self.var).value })
    }

    /// Temporarily acquires exclusive access to the guarded value.  See safety notes on `with`.
    ///
    /// # Panics
    ///
    /// Panics if the capability is not accessible.
    pub fn with_mut<F, R>(&mut self, fun: F) -> R
        where F: FnOnce(&mut T) -> R
    {
        self.check_access();
        fun(unsafe { &mut (*self.var).value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::mem;
    use std::sync::{Arc, Mutex};

    #[test]
    fn same_thread_id() {
        let id1 = current_thread_id();
        let id2 = current_thread_id();

        assert_eq!(id1, id2);
    }

    #[test]
    fn thread_id_formatting() {
        let strg = format!("{:?}", current_thread_id());
        assert!(strg.starts_with("thread("));
    }

    #[test]
    fn different_thread_id() {
        let id1 = current_thread_id();
        let id2 = thread::spawn(|| current_thread_id()).join().unwrap();

        assert!(id1 != id2);
    }

    #[test]
    fn round_tripping() {
        let y = 42;
        let mut cap = ThreadBound::new(y);

        assert!(cap.accessible());

        cap.with(|r1| assert_eq!(*r1, 42));
        cap.with_mut(|r2| assert_eq!(*r2, 42));
        let r3: i32 = cap.into_inner();
        assert_eq!(r3, 42);
    }

    #[test]
    fn wrong_thread_throws() {
        let mut cap = ThreadBound::new(42);
        let joinh = thread::spawn(move || cap.with_mut(|r| *r = 55));
        let res: Box<&'static str> = joinh.join().err().unwrap().downcast().unwrap();
        assert_eq!(*res, "Attempt to access ThreadBound from incorrect thread")
    }

    #[test]
    fn wrong_thread_inaccessible() {
        let cap = ThreadBound::new(42);
        assert!(!thread::spawn(move || cap.accessible()).join().unwrap());
    }

    struct Trap(Arc<Mutex<bool>>);
    impl Drop for Trap {
        fn drop(&mut self) {
            (*self.0.lock().unwrap()) = true;
        }
    }

    #[test]
    fn inaccessible_after_teardown() {
        let cap = TBList::scope(|list| list.bind(42));
        assert!(!cap.accessible());

        let joinh = thread::spawn(|| {
            let mut cap2 = TBList::scope(|list| list.bind(42));
            cap2.with_mut(|r| *r = 55);
        });
        let error: Box<&'static str> = joinh.join().err().unwrap().downcast().unwrap();

        assert_eq!(*error,
                   "Attempt to access ThreadBound during TLS destruction phase");
    }

    #[test]
    fn sync_drop() {
        let was_dropped = Arc::new(Mutex::new(false));
        let cap = ThreadBound::new(Trap(was_dropped.clone()));
        mem::drop(cap);
        assert!(*was_dropped.lock().unwrap());
    }

    #[test]
    fn async_drop() {
        let was_dropped = Arc::new(Mutex::new(false));
        let was_dropped_2 = was_dropped.clone();
        let early_dropped = Arc::new(Mutex::new(false));
        let early_dropped_2 = early_dropped.clone();
        let thr = thread::spawn(move || {
            let cap = ThreadBound::new(Trap(was_dropped_2.clone()));
            thread::spawn(move || mem::drop(cap)).join().unwrap();
            (*early_dropped_2.lock().unwrap()) = *was_dropped_2.lock().unwrap();
        });
        thr.join().unwrap();
        assert!(!*early_dropped.lock().unwrap());
        assert!(*was_dropped.lock().unwrap());
    }
}
