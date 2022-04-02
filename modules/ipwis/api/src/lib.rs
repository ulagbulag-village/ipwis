use anyhow::Result;
use avusen::{function::Function, result::BytesResult};
pub use ipwis_modules_ipwis_common::Ipwis;

#[derive(Copy, Clone)]
pub struct IpwisImpl;

impl Ipwis for IpwisImpl {
    fn call(&self, func: &Function) -> Result<String> {
        let bytes = avusen::encode(func)?;
        let buf = bytes.as_ptr();
        let len = bytes.len();

        unsafe {
            let mut ret = BytesResult::default();

            self::extrinsics::call(buf, len, &mut ret);
            dbg!(&ret.ok);
            dbg!(&ret.len);

            let mut ret_buf = Vec::with_capacity(ret.len as usize);
            self::extrinsics::load(ret_buf.as_mut_ptr());
            ret_buf.set_len(ret.len as usize);

            if ret.ok == 0 {
                Ok(String::from_utf8_unchecked(ret_buf))
            } else {
                Err(anyhow::anyhow!(String::from_utf8_unchecked(ret_buf)))
            }
        }
    }
}

mod extrinsics {
    use avusen::result::BytesResult;

    #[link(wasm_import_module = "ipwis-modules-ipwis")]
    extern "C" {
        pub fn call(buf: *const u8, len: usize, ret: &mut BytesResult);
        pub fn load(buf: *const u8);
    }
}
