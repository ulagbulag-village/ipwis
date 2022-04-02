use anyhow::Result;
use wasmtime::*;
use wasmtime_wasi::WasiCtx;

#[no_mangle]
pub async fn link(linker: &mut Linker<WasiCtx>) -> Result<()> {
    linker.func_wrap("ipwis-modules-dummy", "add_one", add_one)?;
    Ok(())
}

fn add_one(a: i32) -> i32 {
    a + 1
}
