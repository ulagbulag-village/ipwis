//! Example of instantiating of instantiating a wasm module which uses WASI
//! imports.

// You can execute this example with `cargo run --example wasi`

use anyhow::Result;
use ipwis_modules_codec::CodecCtx;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::new(Config::new().async_support(true))?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s: &mut CodecCtx| &mut s.wasi)?;

    // Link the external modules
    ipwis_modules_codec::link(&mut linker).await?;
    // ipwis_modules_dummy::link(&mut linker).await?;
    // ipwis_modules_ipwis::link(&mut linker).await?;

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(
        &engine,
        "./target/wasm32-wasi/debug/ipwis-modules-codec-example.wasi.wasm",
    )?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, CodecCtx::new(wasi));
    linker.module_async(&mut store, "", &module).await?;

    let func = linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store);
    let ret = func.unwrap().call_async(&mut store, ()).await?;
    dbg!(ret);

    Ok(())
}
