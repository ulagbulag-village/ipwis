use ipwis_modules_codec_api::{Codec, CodecImpl, RawData, RawResult};

pub fn main() {
    let result = CodecImpl.call::<_, ()>(&(), ensure_err);
    assert!(result.is_err());
    dbg!(&result);
}

unsafe extern "C" fn ensure_err(data: u32, result: u32) {
    let _data = data as *const RawData;
    let mut result = result as *mut RawResult;

    (*result).ok = 3;
}
