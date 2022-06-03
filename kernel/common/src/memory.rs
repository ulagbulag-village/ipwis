use crate::data::{ExternData, ExternDataRef};

pub trait Memory: Send + Sync {
    unsafe fn is_null(&self, data: ExternDataRef) -> bool {
        (data as *const ExternData).is_null()
    }

    unsafe fn load(&self, data: ExternDataRef) -> &[u8] {
        self.load_raw(data as *const ExternData)
    }

    unsafe fn load_raw(&self, data: *const ExternData) -> &[u8];

    unsafe fn load_mut(&mut self, data: ExternDataRef) -> &mut [u8] {
        self.load_mut_raw(data as *const ExternData)
    }

    unsafe fn load_mut_raw(&mut self, data: *const ExternData) -> &mut [u8];

    unsafe fn dump(&mut self, data: &[u8]) -> ExternData;

    unsafe fn copy(&mut self, src: &[u8], dst: ExternDataRef) {
        self.copy_raw(src, dst as *mut ExternData)
    }

    unsafe fn copy_raw(&mut self, src: &[u8], dst: *mut ExternData);

    unsafe fn copy_error(&mut self, err: ::ipis::core::anyhow::Error, dst: ExternDataRef) {
        self.copy(&err.to_string().into_bytes(), dst);
    }

    unsafe fn set_len(&mut self, len: u32, dst: ExternDataRef) {
        self.set_len_raw(len, dst as *mut ExternData)
    }

    unsafe fn set_len_raw(&mut self, len: u32, dst: *mut ExternData) {
        (*dst).len = len;
    }
}
