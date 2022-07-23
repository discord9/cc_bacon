use std::{
    alloc::{dealloc, Layout},
    ptr::NonNull,
};

use crate::{CcBoxPtr, CcPtr};

unsafe fn deallocate(ptr: NonNull<dyn CcBoxPtr>) {
    dealloc(ptr.cast().as_ptr(), Layout::for_value(ptr.as_ref()));
}

/// Deallocate the box if possible. `s` should already have been dropped.
pub unsafe fn free(mut s: CcPtr) {
    debug_assert!(s.as_mut().strong() == 0);
    debug_assert!(!s.as_mut().buffered());

    // Remove the implicit "strong weak" pointer now that we've destroyed
    // the contents.
    s.as_mut().dec_weak();

    if s.as_mut().weak() == 0 {
        deallocate(s);
    }
}
