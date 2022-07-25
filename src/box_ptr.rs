

use std::ptr::NonNull;

use crate::{CcBoxMetaData, Color, SyncCycleCollector, Trace, BoxMetaData, metadata::MetaData};
pub type CcPtr = NonNull<dyn CcBoxPtr>;
pub trait CcBoxPtr: Trace {
    /// Get this `CcBoxPtr`'s [`CcBoxMetaData`].
    fn metadata(&self) -> &BoxMetaData;

    fn get_ptr(&self) -> CcPtr;

    fn strong(&self) -> usize {
        self.metadata().strong()
    }

    fn weak(&self) -> usize {
        self.metadata().weak()
    }

    fn buffered(&self) -> bool {
        self.metadata().buffered()
    }

    fn color(&self) -> Color {
        self.metadata().color()
    }
}

/// .
pub fn collect_cycles(roots: &SyncCycleCollector) {
    roots.collect_cycles();
}
