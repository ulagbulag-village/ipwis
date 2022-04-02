use ipwis_modules_dummy_api::{Dummy, DummyImpl};

pub fn main() {
    let a = 42;
    let b = DummyImpl.add_one(a);
    dbg!(a, b);
}
