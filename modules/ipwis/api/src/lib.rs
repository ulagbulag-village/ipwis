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
            let mut ret_buf = Vec::with_capacity(64);
            ret.buf = ret_buf.as_mut_ptr() as u32;

            self::extrinsics::call(buf, len, &mut ret);
            ret_buf.set_len(ret.len as usize);
            dbg!(&ret.ok);
            dbg!(&ret.len);
            dbg!(&ret_buf);

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
    }
}
