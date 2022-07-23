use std::sync::Arc;

use super::*;

#[test]
fn test_new_cc(){
    let root = Arc::new(CycleCollector::new());
    let five = Cc::new(5i32, &root);
}
