use ipwis_modules_dummy_api::add_one;

pub fn main() {
    let a = 42;
    let b = add_one(a);
    dbg!(a, b);
}
