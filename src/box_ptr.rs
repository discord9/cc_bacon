

use crate::{CcBoxMetaData, CcPtr, Color, CycleCollector, Trace};

pub trait CcBoxPtr: Trace {
    /// Get this `CcBoxPtr`'s [`CcBoxMetaData`].
    fn metadata(&self) -> &CcBoxMetaData;

    fn get_ptr(&self) -> CcPtr;

    /// Get the color of this node.
    #[inline]
    fn color(&self) -> Color {
        self.metadata().color.get()
    }

    /// Return true if this node is in the buffer of possible cycle roots, false
    /// otherwise.
    #[inline]
    fn buffered(&self) -> bool {
        self.metadata().buffered.get()
    }

    /// Return the strong reference count.
    #[inline]
    fn strong(&self) -> usize {
        self.metadata().strong.get()
    }

    /// cresponding to `Increment(S)`in paper, change color to Black
    #[inline]
    fn increment(&self) {
        self.inc_strong();
        self.metadata().color.set(Color::Black);
    }

    /// Only Increment this node's strong reference count.
    #[inline]
    fn inc_strong(&self) {
        self.metadata().strong.set(self.strong() + 1);
    }

    /// Only dec strong ref and do nothing more
    #[inline]
    fn dec_strong(&self) {
        self.metadata().strong.set(self.strong() - 1);
    }

    /// Decrement this node's strong reference count.
    ///
    /// crosponding to `Decrement(S)`in paper
    #[inline]
    fn decrement(&self) {
        dbg!("Before dec: {}", self.strong());
        dbg!(self.get_ptr());
        if self.strong() > 0 {
            self.dec_strong();
            if self.strong() == 0 {
                self.release()
            } else {
                dbg!("call possible root");
                self.possible_root()
            }
        }
        dbg!("After dec: {}", self.strong());
        if let Some(root) = self.metadata().root.upgrade() {
            dbg!("Root: {}", root);
        }
    }

    /// .
    fn release(&self) {
        debug_assert_eq!(self.strong(), 0);
        // self.trace(&mut |ch| ch.decrement());
        let obj = unsafe {
            self.get_ptr().as_ref()
        };
        obj.trace(&mut |ch| ch.decrement());
        self.metadata().color.set(Color::Black);
        if !self.buffered() {
            self.free();
        }
    }

    fn free(&self);

    fn possible_root(&self) {
        if self.color() != Color::Purple {
            self.metadata().color.set(Color::Purple);
            if !self.buffered() {
                self.metadata().buffered.set(true);
                if let Some(root) = self.metadata().root.upgrade() {
                    root.add_root(self.get_ptr());
                } else {
                    // if roots already didn't exist, free？
                    self.free();
                }
            }
        }
    }

    fn mark_gray(&self) {
        if self.color() != Color::Gray {
            self.metadata().color.set(Color::Gray);
            self.trace(&mut |ch| {
                ch.dec_strong();
                ch.mark_gray();
            });
        }
    }

    fn scan(&self) {
        if self.color() == Color::Gray {
            if self.strong() > 0 {
                todo!()
            } else {
                self.metadata().color.set(Color::White);
                self.trace(&mut |ch| {
                    ch.scan();
                })
            }
        }
    }

    fn scan_black(&self) {
        self.metadata().color.set(Color::Black);
        self.trace(&mut |ch| {
            ch.inc_strong();
            if ch.color() != Color::Black {
                ch.scan_black();
            }
        })
    }

    fn collect_white(&self) {
        if self.color() == Color::White && !self.buffered() {
            self.metadata().color.set(Color::Black);
            self.trace(&mut |ch| ch.collect_white());
            self.free();
        }
    }

    /// Get this node's weak reference count, including the "strong weak"
    /// reference.
    #[inline]
    fn weak(&self) -> usize {
        self.metadata().weak.get()
    }

    /// Increment this node's weak reference count.
    #[inline]
    fn inc_weak(&self) {
        self.metadata().weak.set(self.weak() + 1);
    }

    /// Decrement this node's weak reference count.
    #[inline]
    fn dec_weak(&self) {
        self.metadata().weak.set(self.weak() - 1);
    }
}

/// .
pub fn collect_cycles(roots: &CycleCollector) {
    roots.collect_cycles();
}
