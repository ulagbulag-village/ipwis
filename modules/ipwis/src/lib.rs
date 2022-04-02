use anyhow::Result;
use avusen::{function::Function, result::BytesResult, source::Source};
use wasmtime::*;
use wasmtime_wasi::WasiCtx;

#[no_mangle]
pub async fn link(linker: &mut Linker<WasiCtx>) -> Result<()> {
    linker.func_wrap("ipwis-modules-ipwis", "call", call_unchecked)?;
    Ok(())
}

fn get_caller_memory<T>(caller: &mut Caller<T>) -> Memory {
    let memory = caller
        .get_export("memory")
        .map(|e| e.into_memory().unwrap());
    memory.unwrap()
}

fn call_unchecked(mut caller: Caller<'_, WasiCtx>, buf: u32, len: u32, ret: u32) {
    let memory = get_caller_memory(&mut caller);
    let func = memory
        .data(&caller)
        .get((buf as usize)..)
        .and_then(|s| s.get(..(len as usize)))
        .unwrap();

    unsafe {
        let result = call(func);
        if let Ok(value) = &result {
            dbg!(value);
        }
        let (ok, value) = match result {
            Ok(value) => (0, value),
            Err(value) => (1, value.to_string()),
        };
        let value = value.as_bytes();

        let ret = memory.data_ptr(&caller).offset(ret as isize) as *mut BytesResult;
        (*ret).ok = ok;
        (*ret).len = value.len() as u32;
        dbg!((*ret).ok);
        dbg!((*ret).len);

        let buf = memory.data_ptr(&caller).offset((*ret).buf as isize) as *mut u8;
        buf.copy_from(value.as_ptr(), value.len() as usize);

        dbg!(3);
    }
}

fn call(func: &[u8]) -> Result<String> {
    let func: Function = avusen::decode(func)?;

    match func.program {
        Source::Ipfs { host, .. } => Ok(host.unwrap()),
    }
}
