#![allow(clippy::missing_safety_doc)]

use ipis::{futures, tokio::io::AsyncReadExt};
use ipwis_common::kernel::{resource::ResourceId, data::ExternDataRef};
use ipwis_modules_stream_common::ExternReader;

// #[no_mangle]
// pub async unsafe fn read_sum_async(id: ResourceId, len: u32) -> u32 {
//     let mut reader = ExternReader::new(id, len);

//     let mut data = vec![];
//     reader.read_to_end(&mut data).await.unwrap();
//     dbg!(len, &data);

//     data.into_iter().map(|e| e as u32).sum()
// }

// #[no_mangle]
// pub unsafe extern "C" fn read_sum_sync(id: ResourceId, len: u32) -> u32 {
//     futures::executor::block_on(read_sum_async(id, len))
// }

#[no_mangle]
pub unsafe extern "C" fn malloc(size: ExternDataRef, alignment: ExternDataRef) -> *mut u8 {
    let layout = ::std::alloc::Layout::from_size_align_unchecked(size as usize, alignment as usize);
    ::std::alloc::alloc(layout)
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut u8, size: ExternDataRef, alignment: ExternDataRef) {
    let layout = ::std::alloc::Layout::from_size_align_unchecked(size as usize, alignment as usize);
    ::std::alloc::dealloc(ptr, layout);
}

pub fn main() {
    let msg = "Hello, world!".to_string();
    println!("{msg}");
}
