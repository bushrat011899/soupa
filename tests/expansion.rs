#[macro_use]
extern crate soupa;

mod expand {
    mod a;
    mod b;
    mod c;
    mod d;
}

#[test]
pub fn pass() {
    macrotest::expand("tests/expand/*.rs");
}
