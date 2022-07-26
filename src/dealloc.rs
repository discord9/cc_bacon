use std::{
    alloc::{dealloc, Layout},
    ptr::NonNull,
};

use crate::{metadata::MetaData, CcBoxPtr, CcPtr};

pub unsafe fn deallocate(ptr: NonNull<dyn CcBoxPtr>) {
    #[cfg(test)]
    println!("Actually deallocate in here for {:?}", ptr);
    dealloc(ptr.cast().as_ptr(), Layout::for_value(ptr.as_ref()));
}

/// Deallocate the box if possible. `s` should already have been dropped.
pub unsafe fn free(s: CcPtr) {
    #[cfg(test)]
    println!("Called free in here for {:?}", s);
    debug_assert_eq!(s.as_ref().metadata().strong(), 0);
    debug_assert!(!s.as_ref().metadata().buffered());

    // Remove the implicit "strong weak" pointer now that we've destroyed
    // the contents.
    s.as_ref().metadata().dec_weak();

    if s.as_ref().weak() == 0 {
        deallocate(s);
    }
}
