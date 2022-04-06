pub use ipwis_modules_dummy_common::Dummy;

#[derive(Copy, Clone)]
pub struct DummyImpl;

impl Dummy for DummyImpl {
    fn add_one(&self, a: i32) -> i32 {
        unsafe { self::extrinsics::__add_one(a) }
    }
}

mod extrinsics {
    #[link(wasm_import_module = "ipwis-modules-dummy")]
    extern "C" {
        pub fn __add_one(a: i32) -> i32;
    }
}
