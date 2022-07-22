//! impl bacon's cycle collector: http://link.springer.com/10.1007/3-540-45337-7_12
use std::{cell::Cell, fmt::Debug, sync::Weak};

use core::cell::RefCell;
use core::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// In use or free
    Black,

    /// Possible member of cycle
    Gray,

    /// Member of garbage cycle
    White,

    /// Possible root of cycle
    Purple,

    /// Acyclic
    Green,

    /// Candidate cycle undergoing -computation
    Red,

    /// Candidate cycle awaiting epoch boundary
    Orange,
}

// TODO: understand NonNull can be safe?
type CcRef = NonNull<dyn CcBoxPtr>;

/// one CycleCollector for one virtual Machine
pub struct CycleCollector {
    roots: RefCell<Vec<CcRef>>,
}

// type RootsRef = Arc<CycleCollector>;

impl CycleCollector {
    pub fn add_root(&self, box_ptr: CcRef) {
        let mut vec = self.roots.borrow_mut();
        vec.push(box_ptr);
    }

    pub fn collect_cycles(&self) {
        self.mark_roots();
        self.scan_roots();
        self.collect_roots();
    }

    fn mark_roots(&self) {
        let mut new_roots: Vec<_> = self
            .roots
            .borrow_mut()
            .drain(..)
            .filter(|s| {
                // TODO: check if this is safe!
                let s = unsafe { s.as_ref() };
                if s.color() == Color::Purple {
                    s.mark_gray();
                    true
                } else {
                    s.data().buffered.set(false);
                    if s.color() == Color::Black && s.strong() == 0 {
                        s.free();
                    }
                    false
                }
            })
            .collect();
        self.roots.borrow_mut().append(&mut new_roots);
    }

    fn scan_roots(&self) {
        for s in self.roots.borrow_mut().iter() {
            // TODO: check if this is safe!
            let s = unsafe { s.as_ref() };
            s.scan();
        }
    }

    fn collect_roots(&self) {}
}

#[doc(hidden)]
pub struct CcBoxData {
    strong: Cell<usize>,
    weak: Cell<usize>,
    buffered: Cell<bool>,
    color: Cell<Color>,
    root: Weak<CycleCollector>,
}

impl CcBoxData {
    /*
    There is an implicit weak pointer owned by all the strong
    pointers, which ensures that the weak destructor never frees
    the allocation while the strong destructor is running, even
    if the weak pointer is stored inside the strong one.
    */
    pub fn new(root: Weak<CycleCollector>) -> Self {
        Self {
            strong: 1.into(),
            weak: 1.into(),
            buffered: false.into(),
            color: Color::Black.into(),
            root,
        }
    }
}

/// A trait that informs cycle collector how to find memory that is owned by a
/// `Trace` instance and managed by the cycle collector.
pub trait Trace {
    /// Invoke the `Tracer` on each of the `CcBoxPtr`s owned by this `Trace`
    /// instance.
    ///
    /// Failing to invoke the tracer on every owned `CcBoxPtr` can lead to
    /// leaking cycles.
    fn trace(&self, tracer: &mut Tracer);
}

/// A `Tracer` is a callback function that is invoked for each `CcBoxPtr` owned
/// by an instance of something.
pub type Tracer<'a> = dyn FnMut(&(dyn CcBoxPtr + 'static)) + 'a;

pub trait CcBoxPtr: Trace + Debug {
    /// Get this `CcBoxPtr`'s CcBoxData.
    fn data(&self) -> &CcBoxData;

    fn get_mut(&self) -> CcRef;

    /// Get the color of this node.
    #[inline]
    fn color(&self) -> Color {
        self.data().color.get()
    }

    /// Return true if this node is in the buffer of possible cycle roots, false
    /// otherwise.
    #[inline]
    fn buffered(&self) -> bool {
        self.data().buffered.get()
    }

    /// Return the strong reference count.
    #[inline]
    fn strong(&self) -> usize {
        self.data().strong.get()
    }

    /// cresponding to `Increment(S)`in paper
    #[inline]
    fn increment(&self) {
        self.inc_strong();
        self.data().color.set(Color::Black);
    }

    /// Only Increment this node's strong reference count.
    #[inline]
    fn inc_strong(&self) {
        self.data().strong.set(self.strong() + 1);
    }

    /// Only dec strong ref and do nothing more
    #[inline]
    fn dec_strong(&self) {
        self.data().strong.set(self.strong() - 1);
    }

    /// Decrement this node's strong reference count.
    ///
    /// crosponding to `Decrement(S)`in paper
    #[inline]
    fn decrement(&self) {
        self.dec_strong();
        if self.strong() == 0 {
            self.release()
        } else {
            self.possible_root()
        }
    }

    /// .
    fn release(&self) {
        self.trace(&mut |ch| ch.dec_strong());
        self.data().color.set(Color::Black);
        if !self.buffered() {
            self.free();
        }
    }

    fn possible_root(&self) {
        if self.color() != Color::Purple {
            self.data().color.set(Color::Purple);
            if !self.buffered() {
                self.data().buffered.set(true);
                {
                    if let Some(root) = self.data().root.upgrade() {
                        root.add_root(self.get_mut());
                    }
                }
            }
        }
    }

    fn mark_gray(&self) {
        if self.color() != Color::Gray {
            self.data().color.set(Color::Gray);
            self.trace(&mut |ch| {
                ch.dec_strong();
                ch.mark_gray();
            });
        }
    }

    fn scan(&self) {
        if self.color() == Color::Gray {
            if self.strong() > 0 {
                todo!()
            } else {
                self.data().color.set(Color::White);
                self.trace(&mut |ch| {
                    ch.scan();
                })
            }
        }
    }

    fn scan_black(&self) {
        self.data().color.set(Color::Black);
        self.trace(&mut |ch| {
            ch.inc_strong();
            if ch.color() != Color::Black {
                ch.scan_black();
            }
        })
    }

    fn collect_white(&self) {
        if self.color() != Color::White && !self.buffered() {
            self.data().color.set(Color::Black);
            self.trace(&mut |ch| ch.collect_white());
            self.free();
        }
    }

    /// free the object, drop?(TODO: make it work or what)
    fn free(&self);

    /// Get this node's weak reference count, including the "strong weak"
    /// reference.
    #[inline]
    fn weak(&self) -> usize {
        self.data().weak.get()
    }

    /// Increment this node's weak reference count.
    #[inline]
    fn inc_weak(&self) {
        self.data().weak.set(self.weak() + 1);
    }

    /// Decrement this node's weak reference count.
    #[inline]
    fn dec_weak(&self) {
        self.data().weak.set(self.weak() - 1);
    }
}

pub fn collect_cycles(roots: &mut CycleCollector) {
    roots.collect_cycles();
}
