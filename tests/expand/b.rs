#[cfg(not(test))]
#[macro_use]
extern crate soupa;

use soupa::soupa;
use std::sync::Arc;

fn test_body() {
    let foo = Arc::new(123usize);

    let func = soupa!(move || {
        let foo = {
            {
                // Note that a super block at any depth will be found and lifted
                super { foo.clone() }
            }
        };
        *foo
    });

    let _ = foo;

    let x = func();
    assert_eq!(x, 123);
}

#[test]
fn test() {
    test_body();
}
