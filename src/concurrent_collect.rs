use std::cell::RefCell;

use crate::{acc::AccPtr, AccBoxPtr, Color};
pub struct ParCycleCollector {
    cycle_buffer: RefCell<Vec<AccPtr>>,
}

impl ParCycleCollector {
    pub fn increment(zelf: &dyn AccBoxPtr) {
        zelf.inc_strong();
    }

    pub fn decrement(zelf: &dyn AccBoxPtr) {
        zelf.dec_strong();
        if zelf.strong() == 0{
            ParCycleCollector::release(zelf)
        }
    }

    fn release(zelf: &dyn AccBoxPtr) {
        todo!()
    }

    fn possible_root(zelf: &dyn AccBoxPtr) {
        
    }

    fn scan_black(zelf: &dyn AccBoxPtr) {
        if zelf.color() != Color::Black {
            {
                *zelf.metadata().color.write().unwrap() = Color::Black;
            }

            zelf.trace(&mut |ch| ParCycleCollector::scan_black(ch))
        }
    }
}
