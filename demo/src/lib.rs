#![allow(clippy::missing_safety_doc)]

use ipis::{futures, tokio::io::AsyncReadExt};
use ipwis_modules_stream_common::{ExternId, ExternReader};

#[no_mangle]
pub async unsafe fn read_sum_async(id: ExternId, len: u32) -> u32 {
    let mut reader = ExternReader::new(id, len);

    let mut data = vec![];
    reader.read_to_end(&mut data).await.unwrap();
    dbg!(len, &data);

    data.into_iter().map(|e| e as u32).sum()
}

#[no_mangle]
pub unsafe extern "C" fn read_sum_sync(id: ExternId, len: u32) -> u32 {
    futures::executor::block_on(read_sum_async(id, len))
}
