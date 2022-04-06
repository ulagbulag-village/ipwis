use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

pub trait Codec {
    fn call<T, R>(&self, data: &T, f: ExternFn) -> Result<R>
    where
        T: Serialize,
        R: DeserializeOwned,
    {
        self.load(unsafe { self.dump(data, f) }?)
    }

    fn call_raw<T>(&self, data: &T, f: ExternFn) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        self.load_raw(unsafe { self.dump(data, f) }?)
    }

    unsafe fn dump<T>(&self, data: &T, f: ExternFn) -> Result<RawResult>
    where
        T: Serialize;

    unsafe fn dump_raw(&self, bytes: &[u8], f: ExternFn) -> Result<RawResult>;

    fn load<T>(&self, result: RawResult) -> Result<T>
    where
        T: DeserializeOwned;

    fn load_raw(&self, result: RawResult) -> Result<Vec<u8>>;
}

pub type ExternFn = unsafe extern "C" fn(u32 /* RawData */, u32 /* RawResult */);

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct RawData {
    pub buf: u32,
    pub len: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct RawResult {
    pub ok: u32,
    pub len: u32,
}

impl RawResult {
    pub const DATA_OK: u32 = 0;
    pub const DATA_ERR: u32 = 1;

    pub const RESULT_ERR_INTERNAL: u32 = 0;
    pub const RESULT_OK: u32 = 1;
    pub const RESULT_ERR_CALL: u32 = 2;
    pub const RESULT_ERR_INPUT: u32 = 3;
}
