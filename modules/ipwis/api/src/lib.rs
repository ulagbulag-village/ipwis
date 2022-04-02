pub use ipwis_modules_ipwis_common::{Function, Ipwis};

#[derive(Copy, Clone)]
pub struct IpwisImpl;

impl Ipwis for IpwisImpl {
    fn call(&self, func: &Function) -> Result<String, String> {
        todo!()
        // unsafe { self::extrinsics::__ipwis_modules_dummy__call_raw(a) }
    }
}

mod extrinsics {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        pub fn __ipwis_modules_ipwis__call(method: &str, msg: &str) -> Result;
    }

    #[wasm_bindgen]
    pub struct Result {
        ok: bool,
        value: String,
    }    
}
