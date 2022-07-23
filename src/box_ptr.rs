use crate::{CcBoxMetaData, CcRef, Color, CycleCollector, Trace};

pub trait CcBoxPtr: Trace {
    /// Get this `CcBoxPtr`'s CcBoxData.
    fn data(&self) -> &CcBoxMetaData;

    fn get_ptr(&self) -> CcRef;

    /// Get the color of this node.
    #[inline]
    fn color(&self) -> Color {
        self.data().color.get()
    }

    /// Return true if this node is in the buffer of possible cycle roots, false
    /// otherwise.
    #[inline]
    fn buffered(&self) -> bool {
        self.data().buffered.get()
    }

    /// Return the strong reference count.
    #[inline]
    fn strong(&self) -> usize {
        self.data().strong.get()
    }

    /// cresponding to `Increment(S)`in paper, change color to Black
    #[inline]
    fn increment(&self) {
        self.inc_strong();
        self.data().color.set(Color::Black);
    }

    /// Only Increment this node's strong reference count.
    #[inline]
    fn inc_strong(&self) {
        self.data().strong.set(self.strong() + 1);
    }

    /// Only dec strong ref and do nothing more
    #[inline]
    fn dec_strong(&self) {
        self.data().strong.set(self.strong() - 1);
    }

    /// Decrement this node's strong reference count.
    ///
    /// crosponding to `Decrement(S)`in paper
    #[inline]
    fn decrement(&self) {
        self.dec_strong();
        if self.strong() == 0 {
            self.release()
        } else {
            self.possible_root()
        }
    }

    /// .
    fn release(&self) {
        debug_assert_eq!(self.strong(), 0);
        self.trace(&mut |ch| ch.dec_strong());
        self.data().color.set(Color::Black);
        if !self.buffered() {
            self.free();
        }
    }

    fn possible_root(&self) {
        if self.color() != Color::Purple {
            self.data().color.set(Color::Purple);
            if !self.buffered() {
                self.data().buffered.set(true);
                if let Some(root) = self.data().root.upgrade() {
                    root.add_root(self.get_ptr());
                }else {
                    // if roots already didn't exist, free？
                    self.free();
                    todo!()
                }
            }
        }
    }

    fn mark_gray(&self) {
        if self.color() != Color::Gray {
            self.data().color.set(Color::Gray);
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
                self.data().color.set(Color::White);
                self.trace(&mut |ch| {
                    ch.scan();
                })
            }
        }
    }

    fn scan_black(&self) {
        self.data().color.set(Color::Black);
        self.trace(&mut |ch| {
            ch.inc_strong();
            if ch.color() != Color::Black {
                ch.scan_black();
            }
        })
    }

    fn collect_white(&self) {
        if self.color() != Color::White && !self.buffered() {
            self.data().color.set(Color::Black);
            self.trace(&mut |ch| ch.collect_white());
            self.free();
        }
    }

    /// free the object, drop?(TODO: make it work or what)
    fn free(&self);

    /// Get this node's weak reference count, including the "strong weak"
    /// reference.
    #[inline]
    fn weak(&self) -> usize {
        self.data().weak.get()
    }

    /// Increment this node's weak reference count.
    #[inline]
    fn inc_weak(&self) {
        self.data().weak.set(self.weak() + 1);
    }

    /// Decrement this node's weak reference count.
    #[inline]
    fn dec_weak(&self) {
        self.data().weak.set(self.weak() - 1);
    }
}

pub fn collect_cycles(roots: &mut CycleCollector) {
    roots.collect_cycles();
}
