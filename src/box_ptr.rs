use std::ptr::NonNull;

use crate::{metadata::MetaData, BoxMetaData, Color, SyncCycleCollector, Trace};
pub type CcPtr = NonNull<dyn CcBoxPtr>;
pub trait CcBoxPtr: Trace {
    /// Get this `CcBoxPtr`'s [`CcBoxMetaData`].
    fn metadata(&self) -> &BoxMetaData;

    fn get_ptr(&self) -> CcPtr;

    #[inline]
    fn strong(&self) -> usize {
        self.metadata().strong()
    }
    #[inline]
    fn weak(&self) -> usize {
        self.metadata().weak()
    }
    #[inline]
    fn buffered(&self) -> bool {
        self.metadata().buffered()
    }
    #[inline]
    fn color(&self) -> Color {
        self.metadata().color()
    }
}

/// .
pub fn collect_cycles(roots: &SyncCycleCollector) {
    roots.collect_cycles();
}
