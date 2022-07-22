//! impl bacon's cycle collector: http://link.springer.com/10.1007/3-540-45337-7_12
use std::{
    cell::Cell,
    fmt::Debug,
    sync::{Mutex, Weak, Arc},
};

use core::cell::RefCell;
use core::ptr::NonNull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// In use or free
    Black,

    /// Possible member of cycle
    Gray,

    /// Member of garbage cycle
    White,

    /// Possible root of cycle
    Purple,

    /// Acyclic
    Green,

    /// Candidate cycle undergoing -computation
    Red,

    /// Candidate cycle awaiting epoch boundary
    Orange,
}

/// one Roots for one virtual Machine

pub struct Roots {
    roots: RefCell<Vec<NonNull<dyn CcBoxPtr>>>,
}

type RootsRef = Arc<Roots>;

impl Roots{
    pub fn add_root(&self, box_ptr: NonNull<dyn CcBoxPtr>) {
        let mut vec = self.roots.borrow_mut();
        vec.push(box_ptr);
}
}


#[doc(hidden)]
pub struct CcBoxData {
    strong: Cell<usize>,
    weak: Cell<usize>,
    buffered: Cell<bool>,
    color: Cell<Color>,
    root: Weak<Roots>,
}

/// A trait that informs cycle collector how to find memory that is owned by a
/// `Trace` instance and managed by the cycle collector.
pub trait Trace {
    /// Invoke the `Tracer` on each of the `CcBoxPtr`s owned by this `Trace`
    /// instance.
    ///
    /// Failing to invoke the tracer on every owned `CcBoxPtr` can lead to
    /// leaking cycles.
    fn trace(&self, tracer: &mut Tracer);
}

/// A `Tracer` is a callback function that is invoked for each `CcBoxPtr` owned
/// by an instance of something.
pub type Tracer<'a> = dyn FnMut(&(dyn CcBoxPtr + 'static)) + 'a;

impl Debug for dyn CcBoxPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "(strong={}, weak={}, buffered={}, color={:?})",
            self.strong(),
            self.weak(),
            self.buffered(),
            self.color()
        ))
    }
}

pub trait CcBoxPtr: Trace + Debug {
    /// Get this `CcBoxPtr`'s CcBoxData.
    fn data(&self) -> &CcBoxData;

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

    /// Increment this node's strong reference count.
    #[inline]
    fn inc_strong(&self) {
        self.data().strong.set(self.strong() + 1);
        self.data().color.set(Color::Black);
    }

    /// Decrement this node's strong reference count.
    /// 
    /// cresponding to `Decrement(S)`in paper
    #[inline]
    fn dec_strong(&self) {
        self.data().strong.set(self.strong() - 1);
        if self.strong() == 0{
            self.release()
        }else{
            self.possible_root()
        }
    }

    /// .
    fn release(&self){
        self.trace(&mut |ch|{
            ch.dec_strong()
        });
        self.data().color.set(Color::Black);
        if !self.buffered(){
            self.free();
        }
    }

    fn possible_root(&self){
        if self.color() != Color::Purple{
            self.data().color.set(Color::Purple);
            if !self.buffered(){
                self.data().buffered.set(true);
                {
                    if let Some(root) = self.data().root.upgrade(){
                        let ptr: NonNull<dyn CcBoxPtr> = NonNull::new(self as *mut _).expect("ptr is null!");;
                        root.add_root(ptr);
                        
                    }
                }
                
            }
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

#[inline]
fn noop() {}

pub fn collect_cycles(roots: &mut Roots) {}
