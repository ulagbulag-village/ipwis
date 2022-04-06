use anyhow::{anyhow, bail, Result};
pub use ipwis_modules_codec_common::{Codec, RawData, RawResult};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Copy, Clone)]
pub struct CodecImpl;

impl Codec for CodecImpl {
    unsafe fn dump<T>(&self, data: &T, f: unsafe fn(RawData, &mut RawResult)) -> Result<RawResult>
    where
        T: Serialize,
    {
        let bytes = avusen::codec::encode(data)?;
        let data = RawData {
            buf: bytes.as_ptr() as u32,
            len: bytes.len() as u32,
        };

        let mut result = RawResult::default();
        f(data, &mut result);
        Ok(result)
    }

    unsafe fn dump_raw(
        &self,
        bytes: &[u8],
        f: unsafe fn(RawData, &mut RawResult),
    ) -> Result<RawResult> {
        let data = RawData {
            buf: bytes.as_ptr() as u32,
            len: bytes.len() as u32,
        };

        let mut result = RawResult::default();
        f(data, &mut result);
        Ok(result)
    }

    fn load<T>(&self, result: RawResult) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.load_raw(result)
            .and_then(|e| avusen::codec::decode(&e).map_err(Into::into))
    }

    fn load_raw(&self, result: RawResult) -> Result<Vec<u8>> {
        unsafe {
            let buf = || {
                let mut buf = Vec::with_capacity(result.len as usize);
                match self::extrinsics::__load(buf.as_mut_ptr()) {
                    RawResult::DATA_OK => {
                        buf.set_len(result.len as usize);
                        Ok(buf)
                    }
                    _ => Err(anyhow!("Failed to load the result of an external function")),
                }
            };

            match result.ok {
                RawResult::RESULT_OK => Ok(buf()?),
                RawResult::RESULT_ERR_INTERNAL => bail!("Failed to execute an external function"),
                RawResult::RESULT_ERR_CALL => Err(anyhow!(String::from_utf8_unchecked(buf()?))),
                RawResult::RESULT_ERR_INPUT => {
                    Err(anyhow!("Failed to parse the input of an external function"))
                }
                _ => unreachable!("Failed to parse the result of an external function"),
            }
        }
    }
}

mod extrinsics {
    #[link(wasm_import_module = "ipwis-modules-codec")]
    extern "C" {
        pub fn __load(buf: *const u8) -> u32;
    }
}
