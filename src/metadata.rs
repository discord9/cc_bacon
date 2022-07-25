use std::{cell::Cell, sync::Arc, fmt::Debug};

use enum_dispatch::enum_dispatch;

use crate::{Color, SyncCycleCollector, collect::SyncOrConcurrent};

#[enum_dispatch]
pub trait MetaData {
    fn strong(&self) -> usize;
    fn weak(&self) -> usize;
    fn buffered(&self) -> bool;
    fn color(&self) -> Color;
    fn inc_strong(&self) -> usize;
    fn dec_strong(&self) -> usize;
    fn inc_weak(&self) -> usize;
    fn dec_weak(&self) -> usize;
    fn set_buffered(&self, new: bool);
    fn set_color(&self, new: Color);
    fn is_atomic(&self) -> bool;
    fn root(&self) -> SyncOrConcurrent;
}

impl Debug for dyn MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metadata")
            .field("strong", &self.strong())
            .field("weak", &self.weak())
            .field("buffered", &self.buffered())
            .field("color", &self.color())
            .field("is_atomic", &self.is_atomic())
            .finish()
    }
}

#[enum_dispatch(MetaData)]
pub enum BoxMetaData {
    CcBoxMetaData,
    AccBoxMetaData,
}

pub struct CcBoxMetaData {
    strong: Cell<usize>,
    weak: Cell<usize>,
    buffered: Cell<bool>,
    color: Cell<Color>,
    root: Arc<SyncCycleCollector>,
}

impl BoxMetaData {
    pub fn with(root: SyncOrConcurrent) -> BoxMetaData {{
        match root{
            SyncOrConcurrent::sync_cc(cc)=>{
                BoxMetaData::CcBoxMetaData(CcBoxMetaData::with(cc))
            },
            SyncOrConcurrent::concurrent_cc(cc) => todo!()
        }
    }}
}

impl CcBoxMetaData {
    pub fn with(root: Arc<SyncCycleCollector>) -> Self {
        Self {
            strong: 1.into(),
            weak: 1.into(),
            buffered: false.into(),
            color: Color::Black.into(),
            root,
        }
    }
}

impl MetaData for CcBoxMetaData {
    fn strong(&self) -> usize {
        self.strong.get()
    }

    fn weak(&self) -> usize {
        self.weak.get()
    }

    fn buffered(&self) -> bool {
        self.buffered.get()
    }

    fn color(&self) -> Color {
        self.color.get()
    }

    fn inc_strong(&self) -> usize {
        self.strong.set(self.strong.get() + 1);
        self.strong()
    }

    fn dec_strong(&self) -> usize {
        self.strong.set(self.strong.get() - 1);
        self.strong()
    }

    fn inc_weak(&self) -> usize {
        self.weak.set(self.weak.get() + 1);
        self.weak()
    }

    fn dec_weak(&self) -> usize {
        self.weak.set(self.weak.get() + 1);
        self.weak()
    }

    fn set_buffered(&self, new: bool) {
        self.buffered.set(new)
    }

    fn set_color(&self, new: Color) {
        self.color.set(new)
    }

    fn is_atomic(&self) -> bool {
        false
    }

    fn root(&self) -> SyncOrConcurrent {
        SyncOrConcurrent::sync_cc(self.root.clone())
    }
}

pub struct AccBoxMetaData {
    strong: Cell<usize>,
    weak: Cell<usize>,
    buffered: Cell<bool>,
    color: Cell<Color>,
    root: Arc<SyncCycleCollector>,
    crc: Cell<usize>,
}

impl AccBoxMetaData {
    fn update_crc(&self) -> usize {
        self.crc.set(self.strong());
        self.crc.get()
    }
    fn crc(&self) -> usize {
        self.crc.get()
    }
    fn dec_crc(&self) {
        self.crc.set(self.crc() - 1);
    }
    fn inc_crc(&self) {
        self.crc.set(self.crc() - 1);
    }
}

impl MetaData for AccBoxMetaData {
    fn strong(&self) -> usize {
        todo!()
    }

    fn weak(&self) -> usize {
        todo!()
    }

    fn buffered(&self) -> bool {
        todo!()
    }

    fn color(&self) -> Color {
        todo!()
    }

    fn inc_strong(&self) -> usize {
        todo!()
    }

    fn dec_strong(&self) -> usize {
        todo!()
    }

    fn inc_weak(&self) -> usize {
        todo!()
    }

    fn dec_weak(&self) -> usize {
        todo!()
    }

    fn set_buffered(&self, new: bool) {
        todo!()
    }

    fn set_color(&self, new: Color) {
        todo!()
    }

    fn is_atomic(&self) -> bool {
        true
    }

    fn root(&self) -> SyncOrConcurrent {
        todo!()
    }
}
