pub use ipwis_modules_dummy_common::Dummy;

#[derive(Copy, Clone)]
pub struct DummyImpl;

impl Dummy for DummyImpl {
    fn add_one(&self, a: i32) -> i32 {
        unsafe { self::extrinsics::__ipwis_modules_dummy__add_one(a) }
    }
}

mod extrinsics {
    extern "C" {
        pub fn __ipwis_modules_dummy__add_one(a: i32) -> i32;
    }
}
