use anyhow::Result;
use ipwis_modules_codec_common::{RawData, RawResult};
use serde::{de::DeserializeOwned, Serialize};
use wasmtime::*;
use wasmtime_wasi::WasiCtx;

pub async fn link(linker: &mut Linker<CodecCtx>) -> Result<()> {
    linker.func_wrap("ipwis-modules-codec", "__load", load_unchecked)?;
    Ok(())
}

pub struct CodecCtx {
    pub wasi: WasiCtx,
    data: Option<Vec<u8>>,
}

impl CodecCtx {
    pub fn new(wasi: WasiCtx) -> Self {
        Self { wasi, data: None }
    }

    pub fn func_wrap<T, R, F>(mut caller: Caller<Self>, data: u32, wasi: u32, f: F)
    where
        T: DeserializeOwned,
        R: Serialize,
        F: FnOnce(T) -> Result<R>,
    {
        let memory = Self::get_caller_memory(&mut caller);
        let data = unsafe { memory.data_ptr(&caller).offset(data as isize) as *const RawData };
        let wasi = unsafe { memory.data_ptr(&caller).offset(wasi as isize) as *mut RawResult };

        let bytes = unsafe {
            match memory
                .data(&caller)
                .get(((*data).buf as usize)..)
                .and_then(|s| s.get(..((*data).len as usize)))
            {
                Some(bytes) => bytes,
                None => {
                    (*wasi).ok = RawResult::RESULT_ERR_INPUT;
                    return;
                }
            }
        };

        let (ok, result) = match avusen::codec::decode(&bytes)
            .map_err(Into::into)
            .and_then(f)
        {
            Ok(e) => (RawResult::RESULT_OK, avusen::codec::encode(&e).unwrap()),
            Err(e) => (RawResult::RESULT_ERR_CALL, e.to_string().into_bytes()),
        };
        unsafe {
            (*wasi).ok = ok;
            (*wasi).len = result.len() as u32;
        }
        caller.data_mut().data.replace(result);
    }

    fn get_caller_memory<T>(caller: &mut Caller<T>) -> Memory {
        let memory = caller
            .get_export("memory")
            .map(|e| e.into_memory().unwrap());
        memory.unwrap()
    }
}

fn load_unchecked(mut caller: Caller<'_, CodecCtx>, buf: u32) -> u32 {
    let memory = CodecCtx::get_caller_memory(&mut caller);
    let ctx = caller.data_mut();

    if let Some(data) = ctx.data.take() {
        unsafe {
            let buf = memory.data_ptr(&caller).offset(buf as isize) as *mut u8;
            buf.copy_from(data.as_ptr(), data.len());
        }
        RawResult::DATA_OK
    } else {
        RawResult::DATA_ERR
    }
}
