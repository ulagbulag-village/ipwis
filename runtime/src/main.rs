//! Example of instantiating of instantiating a wasm module which uses WASI
//! imports.

// You can execute this example with `cargo run --example wasi`

use anyhow::Result;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::new(Config::new().async_support(true))?;
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Link the external modules
    ipfis_modules_dummy::link(&mut linker).await?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(
        &engine,
        "./target/wasm32-wasi/debug/ipfis-modules-dummy-runner.wasi.wasm",
    )?;
    linker.module_async(&mut store, "", &module).await?;

    let func = linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store);
    let ret = func.unwrap().call_async(&mut store, ()).await?;
    dbg!(ret);

    Ok(())
}
