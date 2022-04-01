pub fn add_one(a: i32) -> i32 {
    unsafe { self::extrinsics::__ipfis_modules_dummy__add_one(a) }
}

mod extrinsics {
    extern "C" {
        pub fn __ipfis_modules_dummy__add_one(a: i32) -> i32;
    }
}
