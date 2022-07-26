use std::{cell::RefCell, sync::Arc};

use super::*;

struct TestObj {
    to: RefCell<Option<Cc<TestObj>>>,
}

impl Trace for TestObj {
    fn trace(&self, tracer: &mut Tracer) {
        if let Some(to) = self.to.borrow().into() {
            to.trace(tracer)
        }
    }
}
#[test]
fn test_new_cc() {
    let root = Arc::new(SyncCycleCollector::new());
    let five = Cc::new(5i32, &root);
    drop(five);
}

#[test]
fn test_dead() {
    let root = Arc::new(SyncCycleCollector::new());
    let x = Cc::new(5, &root);
    let y = x.downgrade();
    drop(x);
    assert!(y.upgrade().is_none());
}

#[test]
fn test_self_ref_cc() {
    let root = Arc::new(SyncCycleCollector::new());
    // let _five = Cc::new(5i32, &root);
    let cycle = Cc::new(TestObj { to: None.into() }, &root);
    *cycle.to.borrow_mut() = Some(cycle.clone());
    //drop(cycle);
    //root.collect_cycles();
    dbg!(root);
}

#[test]
fn test_simple_ref_cycle() {
    let root = Arc::new(SyncCycleCollector::new());
    // let _five = Cc::new(5i32, &root);
    let obj1 = Cc::new(TestObj { to: None.into() }, &root);
    let obj2 = Cc::new(TestObj { to: None.into() }, &root);
    *obj1.to.borrow_mut() = Some(obj2.clone());
    *obj2.to.borrow_mut() = Some(obj1.clone());
    dbg!("Cycle made");
    dbg!("dropping obj2 now");
    drop(obj2);
    dbg!("obj2 dropped");
    dbg!("dropping obj1 now");
    drop(obj1);
    dbg!("obj1 dropped");
    //*cycle.to.borrow_mut() = Some(cycle.clone());
    //drop(cycle);
    //root.collect_cycles();
    dbg!(root);
}
