// mod memory {
//     #[allow(clippy::missing_safety_doc)]
//     #[link(wasm_import_module = "__ipwis_kernel_api")]
//     extern "C" {
//         pub fn __alloc(size: usize, align: usize) -> *mut u8;
//         pub fn __alloc_zeroed(size: usize, align: usize) -> *mut u8;
//         pub fn __dealloc(ptr: *mut u8, size: usize, align: usize);
//         pub fn __realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8;
//     }
// }

// #[no_mangle]
// pub fn __ipwis_syscall(handler: *const u8, inputs: *const u8, outputs: *mut u8, errors: *mut u8) -> u32 {
//     let msg = "Hello, world!".to_string();
//     println!("{msg}");

//     0
// }
