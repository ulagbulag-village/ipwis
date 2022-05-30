#![allow(clippy::missing_safety_doc)]

pub mod reader {
    use ipis::core::anyhow::Result;
    use ipwis_api::{ctx::IpwisCaller, memory::IpwisMemory};

    pub unsafe fn next(caller: IpwisCaller, error: u32, id: u64, buf: u32) -> u32 {
        unsafe fn try_call(_id: u64, data: &mut [u8]) -> Result<u32> {
            data[0] = 19;
            data[1] = 23;
            Ok(2)
        }

        let mut memory = IpwisMemory::from_caller(caller);

        let data = memory.load_mut(buf);

        match try_call(id, data) {
            Ok(e) => e,
            Err(e) => memory.return_error(e, error),
        }
    }
}

pub mod writer {
    use ipis::core::anyhow::Result;
    use ipwis_api::{ctx::IpwisCaller, memory::IpwisMemory};

    pub unsafe fn next(caller: IpwisCaller, error: u32, id: u64, buf: u32) -> u32 {
        unsafe fn try_call(_id: u64, data: &mut [u8]) -> Result<u32> {
            data[0] = 19;
            data[1] = 23;
            Ok(2)
        }

        let mut memory = IpwisMemory::from_caller(caller);

        let data = memory.load_mut(buf);

        match try_call(id, data) {
            Ok(e) => e,
            Err(e) => memory.return_error(e, error),
        }
    }
}
