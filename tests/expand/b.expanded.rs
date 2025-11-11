#![no_std]
#[macro_use]
extern crate soupa;
use soupa::soupa;
use std::sync::Arc;
fn test_body() {
    let foo = Arc::new(123usize);
    let func = {
        let __soupa_temp__a = { foo.clone() };
        move || {
            let foo = { { __soupa_temp__a } };
            *foo
        }
    };
    let _ = foo;
    let x = func();
    match (&x, &123) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                let kind = ::core::panicking::AssertKind::Eq;
                ::core::panicking::assert_failed(
                    kind,
                    &*left_val,
                    &*right_val,
                    ::core::option::Option::None,
                );
            }
        }
    };
}
