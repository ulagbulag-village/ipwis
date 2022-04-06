use anyhow::Result;
use avusen::function::Function;
pub use ipwis_modules_codec_api::{Codec, CodecImpl};
pub use ipwis_modules_ipwis_common::Ipwis;

#[derive(Copy, Clone)]
pub struct IpwisImpl;

impl Ipwis for IpwisImpl {
    fn call(&self, func: &Function) -> Result<String> {
        CodecImpl.call(func, self::extrinsics::__call)
    }
}

mod extrinsics {
    pub use ipwis_modules_codec_api::{RawData, RawResult};

    #[link(wasm_import_module = "ipwis-modules-ipwis")]
    extern "C" {
        pub fn __call(data: u32, result: u32);
    }
}
