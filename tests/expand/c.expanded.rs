#![no_std]
#[macro_use]
extern crate soupa;
use soupa::soupa;
use std::sync::Arc;
fn test_body() {
    let a = Arc::new(123usize);
    let func = {
        let __soupa_temp__a = { a.clone() };
        move || *__soupa_temp__a
    };
    let _ = a;
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
