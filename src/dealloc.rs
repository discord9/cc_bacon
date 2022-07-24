use std::{
    alloc::{dealloc, Layout},
    ptr::NonNull,
};

use crate::{CcBoxPtr, CcPtr};

unsafe fn deallocate(ptr: NonNull<dyn CcBoxPtr>) {
    dealloc(ptr.cast().as_ptr(), Layout::for_value(ptr.as_ref()));
}

/// Deallocate the box if possible. `s` should already have been dropped.
pub unsafe fn free(s: CcPtr) {
    #[cfg(test)]
    dbg!("Called free in here for", s);
    debug_assert_eq!(s.as_ref().strong(), 0);
    debug_assert!(!s.as_ref().buffered());

    // Remove the implicit "strong weak" pointer now that we've destroyed
    // the contents.
    s.as_ref().dec_weak();

    if s.as_ref().weak() == 0 {
        deallocate(s);
    }
}
