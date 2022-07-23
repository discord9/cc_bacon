mod box_ptr;
mod collect;
#[cfg(test)]
mod tests;
mod trace;
use std::{
    cell::Cell,
    fmt::Debug,
    ptr::NonNull,
    sync::{Arc, Weak}, ops::Deref,
};

pub use box_ptr::CcBoxPtr;
use collect::RootsRef;
pub use collect::{CcRef, CycleCollector};
pub use trace::{Trace, Tracer};

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

#[doc(hidden)]
pub struct CcBoxMetaData {
    strong: Cell<usize>,
    weak: Cell<usize>,
    buffered: Cell<bool>,
    color: Cell<Color>,
    root: Weak<CycleCollector>,
}

impl CcBoxMetaData {
    /*
    There is an implicit weak pointer owned by all the strong
    pointers, which ensures that the weak destructor never frees
    the allocation while the strong destructor is running, even
    if the weak pointer is stored inside the strong one.
    */
    pub fn with(root: Weak<CycleCollector>) -> Self {
        Self {
            strong: 1.into(),
            weak: 1.into(),
            buffered: false.into(),
            color: Color::Black.into(),
            root,
        }
    }
}

struct CcBox<T: Trace> {
    value: T,
    metadata: CcBoxMetaData,
}

impl<T: Trace> Trace for Cc<T> {
    fn trace(&self, tracer: &mut Tracer) {
        unsafe {
            tracer(self._ptr.as_ref());
        }
    }
}

impl<T: Trace> Trace for CcBox<T> {
    fn trace(&self, tracer: &mut Tracer) {
        Trace::trace(&self.value, tracer)
    }
}

impl<T: 'static + Trace> CcBoxPtr for CcBox<T> {
    fn data(&self) -> &CcBoxMetaData {
        &self.metadata
    }

    fn get_ptr(&self) -> CcRef {
        // CcBox's mutability is interior, so?
        NonNull::from(self)
    }

    fn free(&self) {
        unreachable!()
    }
}

#[doc(hidden)]
impl<T: Trace> CcBoxPtr for Cc<T> {
    #[inline(always)]
    fn data(&self) -> &CcBoxMetaData {
        unsafe {
            self._ptr.as_ref().data()
        }
    }

    fn get_ptr(&self) -> CcRef {
        self._ptr
    }

    fn free(&self) {
        todo!()
    }
}

/// A reference-counted pointer type over an immutable value.
///
/// See the [module level documentation](./) for more details.
pub struct Cc<T: 'static + Trace> {
    // FIXME #12808: strange names to try to avoid interfering with field
    // accesses of the contained type via Deref
    _ptr: NonNull<CcBox<T>>,
}

impl<T: Trace> Cc<T> {
    pub fn new(value: T, roots: &RootsRef) -> Cc<T> {
        unsafe {
            Cc {
                _ptr: NonNull::new_unchecked(Box::into_raw(Box::new(CcBox {
                    value,
                    metadata: CcBoxMetaData::with(Arc::downgrade(roots)),
                }))),
            }
        }
    }
}

impl<T: Trace> Deref for Cc<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        if self.strong() > 0 {
            unsafe { &self._ptr.as_ref().value }
        } else {
            panic!("Invalid access during cycle collection");
        }
    }
}

impl<T: Trace> Clone for Cc<T> {
    fn clone(&self) -> Self {
        self.inc_strong();
        Cc { _ptr: self._ptr }
    }
}