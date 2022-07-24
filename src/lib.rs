mod box_ptr;
mod collect;
mod dealloc;
#[cfg(test)]
mod tests;
mod trace;
use std::{
    cell::Cell,
    fmt::Debug,
    ops::Deref,
    ptr::NonNull,
    sync::{Arc},
};

pub use box_ptr::{collect_cycles, CcBoxPtr};
use collect::RootsRef;
pub use collect::{CcPtr, CycleCollector};

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
    root: Arc<CycleCollector>,
}

impl Debug for CcBoxMetaData{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metadata")
        .field("strong", &self.strong.get())
        .field("weak", &self.weak.get())
        .field("buffered", &self.buffered.get())
        .field("color", &self.color.get())
        .finish()
    }
}

impl CcBoxMetaData {
    /*
    There is an implicit weak pointer owned by all the strong
    pointers, which ensures that the weak destructor never frees
    the allocation while the strong destructor is running, even
    if the weak pointer is stored inside the strong one.
    */
    pub fn with(root: Arc<CycleCollector>) -> Self {
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
        #[cfg(test)]
        println!("Call trace from Cc: {:?}", self.get_ptr());
        /*Trace::trace(unsafe {self._ptr.as_ref() }, tracer)
         */
        unsafe {
            tracer(self._ptr.as_ref());
        }
    }
}

impl<T:'static +  Trace> Trace for CcBox<T> {
    fn trace(&self, tracer: &mut Tracer) {
        #[cfg(test)]
        println!("Call trace from CcBox: {:?}", self.get_ptr());
        Trace::trace(&self.value, tracer)
    }
}

impl<T: 'static + Trace> CcBoxPtr for CcBox<T> {
    fn metadata(&self) -> &CcBoxMetaData {
        &self.metadata
    }

    fn get_ptr(&self) -> CcPtr {
        // CcBox's mutability is interior, so?
        NonNull::from(self)
    }
}

#[doc(hidden)]
impl<T: Trace> CcBoxPtr for Cc<T> {
    #[inline(always)]
    fn metadata(&self) -> &CcBoxMetaData {
        unsafe { self._ptr.as_ref().metadata() }
    }

    fn get_ptr(&self) -> CcPtr {
        self._ptr
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
                    metadata: CcBoxMetaData::with(roots.clone()),
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

impl<T: Trace> Drop for Cc<T> {
    fn drop(&mut self) {
        #[cfg(test)]
        dbg!("Cc Drop here.");
        CycleCollector::decrement(self);
        self.metadata().root.collect_cycles();
    }
}
