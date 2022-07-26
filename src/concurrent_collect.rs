use std::cell::RefCell;

use crate::{AccBoxPtr, CcPtr, Color};

#[derive(Debug)]
#[allow(unused)]
pub struct ParCycleCollector {
    cycle_buffer: RefCell<Vec<CcPtr>>,
}

impl ParCycleCollector {
    pub fn increment(zelf: &dyn AccBoxPtr) {
        zelf.inc_strong();
        ParCycleCollector::scan_black(zelf);
    }

    pub fn decrement(zelf: &dyn AccBoxPtr) {
        zelf.dec_strong();
        if zelf.strong() == 0 {
            ParCycleCollector::release(zelf)
        }
    }

    fn release(_zelf: &dyn AccBoxPtr) {
        todo!()
    }

    fn possible_root(_zelf: &dyn AccBoxPtr) {}

    fn scan_black(zelf: &dyn AccBoxPtr) {
        let mut meta = zelf.metadata().write().unwrap();
        if meta.color != Color::Black {
            meta.color = Color::Black;
            zelf.trace(&mut |ch| ParCycleCollector::scan_black(ch))
        }
    }
}
