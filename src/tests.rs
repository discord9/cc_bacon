use std::{sync::Arc, cell::RefCell};

use super::*;

struct TestObj{
    to: RefCell<Option<Cc<TestObj>>>
}

impl Trace for TestObj{
    fn trace(&self, tracer: &mut Tracer) {
        if let Some(to) = self.to.borrow_mut().into(){
            to.trace(tracer)
        }
    }
}
#[test]
fn test_new_cc(){
    let root = Arc::new(CycleCollector::new());
    let _five = Cc::new(5i32, &root);
}

#[test]
fn test_self_ref_cc(){
    let root = Arc::new(CycleCollector::new());
    // let _five = Cc::new(5i32, &root);
    let cycle = Cc::new(TestObj{to:None.into()}, &root);
    *cycle.to.borrow_mut() = Some(cycle.clone());
    //drop(cycle);
    //root.collect_cycles();
    dbg!(root);
}
