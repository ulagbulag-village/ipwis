use anyhow::Result;
use avusen::{function::Function, source::Source};
use ipwis_modules_codec::CodecCtx;
use wasmtime::*;

pub async fn link(linker: &mut Linker<CodecCtx>) -> Result<()> {
    linker.func_wrap(
        "ipwis-modules-ipwis",
        "__call",
        |caller: Caller<CodecCtx>, data, wasi| CodecCtx::func_wrap(caller, data, wasi, call),
    )?;
    Ok(())
}

fn call(func: Function) -> Result<String> {
    match func.program {
        Source::Ipfs { host, .. } => Ok(host.unwrap()),
    }
}
