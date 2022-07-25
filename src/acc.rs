use crate::Color;
use std::sync::RwLock;
use std::{cell::Cell, ptr::NonNull};

/// A `Tracer` is a callback function that is invoked for each `CcBoxPtr` owned
/// by an instance of something.
pub type Atracer<'a> = dyn FnMut(&(dyn AccBoxPtr + 'static)) + 'a;

/// A trait that informs cycle collector how to find memory that is owned by a
/// `Trace` instance and managed by the cycle collector.
pub trait Atrace {
    /// Invoke the `Tracer` on each of the `CcBoxPtr`s owned by this `Trace`
    /// instance.
    ///
    /// Failing to invoke the tracer on every owned `CcBoxPtr` can lead to
    /// leaking cycles.
    fn trace(&self, tracer: &mut Atracer);
}
pub struct AccBoxMetaData {
    /// maybe concurrently access
    pub strong: usize,
    /// also maybe concurrently access
    pub weak: usize,
    pub buffered: bool,
    pub color: Color,
    // only be called by cycle collector, so no atomic needed
    pub crc: usize,
}


pub type AccPtr = NonNull<dyn AccBoxPtr>;

/// TODO: wrap MetaData with MPSC
/// so Mutator is serialized so no strange concurrent bug can happen
pub trait AccBoxPtr: Atrace {
    /// Get this `CcBoxPtr`'s [`CcBoxMetaData`]. Using RwLock to serialize
    fn metadata(&self) -> &RwLock<AccBoxMetaData>;

    fn get_ptr(&self) -> AccPtr;
    #[inline]
    fn strong(&self) -> usize {
        self.metadata().read().unwrap().strong
    }
    #[inline]
    fn weak(&self) -> usize {
        self.metadata().read().unwrap().weak
    }
    /// Return true if this node is in the buffer of possible cycle roots, false
    /// otherwise.
    #[inline]
    fn buffered(&self) -> bool {
        self.metadata().read().unwrap().buffered
    }

    /// Get the color of this node.
    #[inline]
    fn color(&self) -> Color {
        self.metadata().read().unwrap().color
    }

    fn crc(&self) -> usize {
        self.metadata().read().unwrap().crc
    }

    /// Only Increment this node's strong reference count.
    #[inline]
    fn inc_strong(&self) -> usize {
        let mut res = self.metadata().write().unwrap();
        res.strong += 1;
        res.strong
    }

    /// Only dec strong ref and do nothing more
    #[inline]
    fn dec_strong(&self) -> usize {
        let mut res = self.metadata().write().unwrap();
        res.strong -= 1;
        res.strong
    }

    /// Atomic Increment this node's weak reference count.
    #[inline]
    fn inc_weak(&self) -> usize {
        let mut res = self.metadata().write().unwrap();
        res.weak += 1;
        res.weak
    }

    /// Atomic Decrement this node's weak reference count and return new weak cnt
    #[inline]
    fn dec_weak(&self) -> usize {
        let mut res = self.metadata().write().unwrap();
        res.weak -= 1;
        res.weak
    }
}
