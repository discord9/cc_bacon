

use std::ptr::NonNull;

use crate::{CcBoxMetaData, Color, CycleCollector, Trace};
pub type CcPtr = NonNull<dyn CcBoxPtr>;
pub trait CcBoxPtr: Trace {
    /// Get this `CcBoxPtr`'s [`CcBoxMetaData`].
    fn metadata(&self) -> &CcBoxMetaData;

    fn get_ptr(&self) -> CcPtr;

    /// Get the color of this node.
    #[inline]
    fn color(&self) -> Color {
        self.metadata().color.get()
    }

    /// Return true if this node is in the buffer of possible cycle roots, false
    /// otherwise.
    #[inline]
    fn buffered(&self) -> bool {
        self.metadata().buffered.get()
    }

    /// Return the strong reference count.
    #[inline]
    fn strong(&self) -> usize {
        self.metadata().strong.get()
    }

    /// Only Increment this node's strong reference count.
    #[inline]
    fn inc_strong(&self) {
        self.metadata().strong.set(self.strong() + 1);
    }

    /// Only dec strong ref and do nothing more
    #[inline]
    fn dec_strong(&self) {
        self.metadata().strong.set(self.strong() - 1);
    }

    /// Get this node's weak reference count, including the "strong weak"
    /// reference.
    #[inline]
    fn weak(&self) -> usize {
        self.metadata().weak.get()
    }

    /// Increment this node's weak reference count.
    #[inline]
    fn inc_weak(&self) {
        self.metadata().weak.set(self.weak() + 1);
    }

    /// Decrement this node's weak reference count.
    #[inline]
    fn dec_weak(&self) {
        self.metadata().weak.set(self.weak() - 1);
    }
}

/// .
pub fn collect_cycles(roots: &CycleCollector) {
    roots.collect_cycles();
}
