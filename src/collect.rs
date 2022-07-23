//! impl bacon's cycle collector: http://link.springer.com/10.1007/3-540-45337-7_12
use std::{cell::Cell, fmt::Debug, sync::{Weak, Arc}};

use core::cell::RefCell;
use core::ptr::NonNull;

use crate::{Color, CcBoxData, CcBoxPtr};
// TODO: understand NonNull can be safe?
pub type CcRef = NonNull<dyn CcBoxPtr>;

/// one CycleCollector for one virtual Machine
pub struct CycleCollector {
    roots: RefCell<Vec<CcRef>>,
}

pub type RootsRef = Arc<CycleCollector>;

impl CycleCollector {
    pub fn add_root(&self, box_ptr: CcRef) {
        let mut vec = self.roots.borrow_mut();
        vec.push(box_ptr);
    }

    pub fn collect_cycles(&self) {
        self.mark_roots();
        self.scan_roots();
        self.collect_roots();
    }

    fn mark_roots(&self) {
        let mut new_roots: Vec<_> = self
            .roots
            .borrow_mut()
            .drain(..)
            .filter(|s| {
                // TODO: check if this is safe!
                let s = unsafe { s.as_ref() };
                if s.color() == Color::Purple {
                    s.mark_gray();
                    true
                } else {
                    s.data().buffered.set(false);
                    if s.color() == Color::Black && s.strong() == 0 {
                        s.free();
                    }
                    false
                }
            })
            .collect();
        self.roots.borrow_mut().append(&mut new_roots);
    }

    fn scan_roots(&self) {
        for s in self.roots.borrow_mut().iter() {
            // TODO: check if this is safe!
            let s = unsafe { s.as_ref() };
            s.scan();
        }
    }

    fn collect_roots(&self) {
        self
            .roots
            .borrow_mut()
            .drain(..)
            .map(|s| {
                // TODO: check if this is safe!
                let s = unsafe { s.as_ref() };
                s.data().buffered.set(false);
                s.collect_white();
            })
            .count();
    }
}




