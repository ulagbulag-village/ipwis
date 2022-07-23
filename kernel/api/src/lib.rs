#[cfg(not(target_os = "wasi"))]
pub extern crate wasmtime;
#[cfg(not(target_os = "wasi"))]
pub extern crate wasmtime_wasi;

pub mod intrinsics;
#[cfg(not(target_os = "wasi"))]
pub mod memory;

#[cfg(not(target_os = "wasi"))]
pub fn load_module(
    engine: &::wasmtime::Engine,
) -> ::ipis::core::anyhow::Result<::wasmtime::Module> {
    const BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/output.wasm"));

    ::wasmtime::Module::from_binary(engine, BINARY)
}
