use ipwis_modules_codec_api::{Codec, CodecImpl, RawData, RawResult};

pub fn main() {
    let result = CodecImpl.call::<_, ()>(&(), ensure_err);
    assert!(result.is_err());
    dbg!(&result);
}

unsafe fn ensure_err(_: RawData, result: &mut RawResult) {
    result.ok = 3;
}
