#[cfg(not(test))]
#[macro_use]
extern crate soupa;

use soupa::soupa;
use std::sync::Arc;

fn test_body() {
    let a = Arc::new(123usize);

    let func = soupa!(move || *super { a.clone() });

    let _ = a;

    let x = func();
    assert_eq!(x, 123);
}

#[test]
fn test() {
    test_body();
}
