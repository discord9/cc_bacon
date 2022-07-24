//! impl bacon's cycle collector: <http://link.springer.com/10.1007/3-540-45337-7_12>
use std::fmt::Debug;
use std::sync::Arc;

use core::cell::RefCell;


use crate::{CcBoxPtr, Color, dealloc, CcPtr};
// TODO: understand NonNull can be safe?


/// one CycleCollector for one virtual Machine

pub struct CycleCollector {
    roots: RefCell<Vec<CcPtr>>,
}

impl Debug for CycleCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.roots.borrow().iter().map(|raw| {
                let ptr = unsafe { raw.as_ref() };
                (raw, ptr.metadata())
            }))
            .finish()
    }
}

pub type RootsRef = Arc<CycleCollector>;

impl Default for CycleCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl CycleCollector {
    pub fn new() -> Self {
        Self {
            roots: Vec::new().into(),
        }
    }

    /// # Safety
    /// should have already dropped
    pub unsafe fn free(zelf: &dyn CcBoxPtr) {
        dealloc::free(zelf.get_ptr())
    }

    /// cresponding to `Increment(S)`in paper, change color to Black
    #[inline]
    pub fn increment(zelf: &dyn CcBoxPtr) {
        zelf.inc_strong();
        zelf.metadata().color.set(Color::Black);
    }

    /// Decrement this node's strong reference count.
    ///
    /// crosponding to `Decrement(S)`in paper
    #[inline]
    pub fn decrement(zelf: &dyn CcBoxPtr) {
        #[cfg(test)]
        dbg!("Before dec: ", zelf.strong());
        #[cfg(test)]
        dbg!(zelf.get_ptr());
        if zelf.strong() > 0 {
            zelf.dec_strong();
            if zelf.strong() == 0 {
                Self::release(zelf);
            } else {
                Self::possible_root(zelf);
            }
        }
        #[cfg(test)]
        dbg!("After dec: ", zelf.strong());
    }

    /// .
    fn release(zelf: &dyn CcBoxPtr) {
        debug_assert_eq!(zelf.strong(), 0);
        // self.trace(&mut |ch| ch.decrement());
        let obj = unsafe { zelf.get_ptr().as_ref() };
        obj.trace(&mut |ch| Self::decrement(ch));
        zelf.metadata().color.set(Color::Black);
        if !zelf.buffered() {
            unsafe { CycleCollector::free(zelf) }
        }
    }

    fn possible_root(zelf: &dyn CcBoxPtr) {
        if zelf.color() != Color::Purple {
            zelf.metadata().color.set(Color::Purple);
            if !zelf.buffered() {
                zelf.metadata().buffered.set(true);
                zelf.metadata().root.add_root(zelf.get_ptr());
            }
        }
    }

    pub fn add_root(&self, box_ptr: CcPtr) {
        let mut vec = self.roots.borrow_mut();
        vec.push(box_ptr);
    }

    pub fn collect_cycles(&self) {
        #[cfg(test)]
        println!("Call mark_roots() with roots: {:#?}", self);
        self.mark_roots();
        #[cfg(test)]
        println!("Call scan_roots() with roots: {:#?}", self);
        self.scan_roots();
        #[cfg(test)]
        println!("Call collect_roots() with roots: {:#?}", self);
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
                    Self::mark_gray(s);
                    true
                } else {
                    s.metadata().buffered.set(false);
                    if s.color() == Color::Black && s.strong() == 0 {
                        unsafe { CycleCollector::free(s) }
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
            Self::scan(s)
        }
    }

    fn collect_roots(&self) {
        self.roots
            .borrow_mut()
            .drain(..)
            .map(|s| {
                // TODO: check if this is safe!
                let s = unsafe { s.as_ref() };
                s.metadata().buffered.set(false);
                Self::collect_white(s);
            })
            .count();
    }

    fn mark_gray(zelf: &dyn CcBoxPtr) {
        if zelf.color() != Color::Gray {
            zelf.metadata().color.set(Color::Gray);
            zelf.trace(&mut |ch| {
                ch.dec_strong();
                Self::mark_gray(ch);
            });
        }
    }

    fn scan(zelf: &dyn CcBoxPtr) {
        if zelf.color() == Color::Gray {
            if zelf.strong() > 0 {
                Self::scan_black(zelf);
            } else {
                zelf.metadata().color.set(Color::White);
                zelf.trace(&mut |ch| {
                    Self::scan(ch);
                })
            }
        }
    }

    fn scan_black(zelf: &dyn CcBoxPtr) {
        zelf.metadata().color.set(Color::Black);
        zelf.trace(&mut |ch| {
            ch.inc_strong();
            if ch.color() != Color::Black {
                Self::scan_black(ch);
            }
        })
    }

    fn collect_white(zelf: &dyn CcBoxPtr) {
        if zelf.color() == Color::White && !zelf.buffered() {
            zelf.metadata().color.set(Color::Black);
            zelf.trace(&mut |ch| Self::collect_white(ch));
            unsafe { CycleCollector::free(zelf) }
        }
    }
}
