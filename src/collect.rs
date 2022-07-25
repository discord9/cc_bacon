//! impl bacon's cycle collector: <http://link.springer.com/10.1007/3-540-45337-7_12>
use std::fmt::Debug;
use std::sync::Arc;

use core::cell::RefCell;


use crate::{CcBoxPtr, Color, dealloc, CcPtr, metadata::MetaData, concurrent_collect::ParCycleCollector};
// TODO: understand NonNull can be safe?

/// all possible types of collector currently only sync/par, maybe add on the fly(2006)
#[derive(Debug)]
pub enum SyncOrConcurrent {
    sync_cc( Arc<SyncCycleCollector>),
    concurrent_cc(Arc<ParCycleCollector>)
}

impl From<Arc<ParCycleCollector>> for SyncOrConcurrent {
    fn from(v: Arc<ParCycleCollector>) -> Self {
        Self::concurrent_cc(v)
    }
}

impl From<Arc<SyncCycleCollector>> for SyncOrConcurrent {
    fn from(v: Arc<SyncCycleCollector>) -> Self {
        Self::sync_cc(v)
    }
}


impl SyncOrConcurrent {
    fn add_root(&self, elem: CcPtr) {
        match self{
            Self::sync_cc(cc) => {
                cc.add_root(elem)
            },
            Self::concurrent_cc(cc) => todo!()
        }
    }

    pub fn try_into_sync_cc(self) -> Result<Arc<SyncCycleCollector>, Self> {
        if let Self::sync_cc(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_concurrent_cc(self) -> Result<Arc<ParCycleCollector>, Self> {
        if let Self::concurrent_cc(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

/// one CycleCollector for one virtual Machine
pub struct SyncCycleCollector {
    roots: RefCell<Vec<CcPtr>>,
}

impl Debug for SyncCycleCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.roots.borrow().iter().map(|raw| {
                let ptr = unsafe { raw.as_ref() };
                (raw, ptr.metadata() as &dyn MetaData)
            }))
            .finish()
    }
}

pub type RootsRef = Arc<SyncCycleCollector>;

impl Default for SyncCycleCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SyncCycleCollector {
    /// cresponding to `Increment(S)`in paper, change color to Black
    #[inline]
    pub fn increment(zelf: &dyn CcBoxPtr) {
        zelf.metadata().inc_strong();
        zelf.metadata().set_color(Color::Black);
    }

    /// Decrement this node's strong reference count.
    ///
    /// crosponding to `Decrement(S)`in paper
    #[inline]
    pub fn decrement(zelf: &dyn CcBoxPtr) {
        #[cfg(test)]
        dbg!("Before dec: ", zelf.metadata().strong());
        #[cfg(test)]
        dbg!(zelf.get_ptr());
        if zelf.strong() > 0 {
            zelf.metadata().dec_strong();
            if zelf.strong() == 0 {
                Self::release(zelf);
            } else {
                Self::possible_root(zelf);
            }
        }
        #[cfg(test)]
        dbg!("After dec: ", zelf.strong());
    }

    #[inline]
    fn add_root(&self, box_ptr: CcPtr) {
        let mut vec = self.roots.borrow_mut();
        vec.push(box_ptr);
    }
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

    /// .
    fn release(zelf: &dyn CcBoxPtr) {
        #[cfg(test)]
        println!("Call release with {:#?}", zelf.get_ptr());
        debug_assert_eq!(zelf.strong(), 0);
        // self.trace(&mut |ch| ch.decrement());
        let obj = unsafe { zelf.get_ptr().as_ref() };
        obj.trace(&mut |ch| Self::decrement(ch));
        zelf.metadata().set_color(Color::Black);
        if !zelf.buffered() {
            unsafe { Self::free(zelf) }
        }
    }

    fn possible_root(zelf: &dyn CcBoxPtr) {
        if zelf.color() != Color::Purple {
            zelf.metadata().set_color(Color::Purple);
            if !zelf.buffered() {
                zelf.metadata().set_buffered(true);
                zelf.metadata().root().add_root(zelf.get_ptr());
            }
        }
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
                    s.metadata().set_buffered(false);
                    if s.color() == Color::Black && s.strong() == 0 {
                        unsafe { Self::free(s) }
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
                s.metadata().set_buffered(false);
                Self::collect_white(s);
            })
            .count();
    }

    fn mark_gray(zelf: &dyn CcBoxPtr) {
        if zelf.color() != Color::Gray {
            zelf.metadata().set_color(Color::Gray);
            zelf.trace(&mut |ch| {
                ch.metadata().dec_strong();
                Self::mark_gray(ch);
            });
        }
    }

    fn scan(zelf: &dyn CcBoxPtr) {
        if zelf.color() == Color::Gray {
            if zelf.strong() > 0 {
                Self::scan_black(zelf);
            } else {
                zelf.metadata().set_color(Color::White);
                zelf.trace(&mut |ch| {
                    Self::scan(ch);
                })
            }
        }
    }

    fn scan_black(zelf: &dyn CcBoxPtr) {
        zelf.metadata().set_color(Color::Black);
        zelf.trace(&mut |ch| {
            ch.metadata().inc_strong();
            if ch.color() != Color::Black {
                Self::scan_black(ch);
            }
        })
    }

    fn collect_white(zelf: &dyn CcBoxPtr) {
        if zelf.color() == Color::White && !zelf.buffered() {
            zelf.metadata().set_color(Color::Black);
            zelf.trace(&mut |ch| Self::collect_white(ch));
            unsafe { Self::free(zelf) }
        }
    }
}
