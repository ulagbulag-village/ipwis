use core::mem::MaybeUninit;

use bytecheck::CheckBytes;
use ipis::core::anyhow::{bail, Result};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Default, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug))]
pub struct ExternData {
    pub ptr: ExternDataRef,
    pub len: ExternDataRef,
}

impl ExternData {
    pub fn from_slice(slice: &[u8]) -> Self {
        let ptr = slice.as_ptr();
        let len = slice.len();

        Self::with_raw_parts(ptr as *const u8, len)
    }

    pub fn from_slice_mut(slice: &mut [u8]) -> Self {
        let ptr = slice.as_mut_ptr();
        let len = slice.len();

        Self::with_raw_parts_mut(ptr as *mut u8, len)
    }

    pub fn from_slice_uninit_mut(slice: &mut [MaybeUninit<u8>]) -> Self {
        let ptr = slice.as_mut_ptr();
        let len = slice.len();

        Self::with_raw_parts_mut(ptr as *mut u8, len)
    }

    pub fn with_raw_parts(ptr: *const u8, len: usize) -> Self {
        Self {
            ptr: ptr as ExternDataRef,
            len: len as ExternDataRef,
        }
    }

    pub fn with_raw_parts_mut(ptr: *mut u8, len: usize) -> Self {
        Self {
            ptr: ptr as ExternDataRef,
            len: len as ExternDataRef,
        }
    }

    pub fn is_null(&self) -> bool {
        self.ptr == 0
    }

    pub fn as_ptr(&self) -> ExternDataRef {
        self as *const Self as ExternDataRef
    }

    pub fn as_mut_ptr(&mut self) -> ExternDataRef {
        self as *mut Self as ExternDataRef
    }

    pub fn as_bytes(self) -> [u8; ::core::mem::size_of::<ExternDataRef>() * 2] {
        // note: wasm uses little-endian
        unsafe { ::core::mem::transmute([self.ptr.to_le_bytes(), self.len.to_le_bytes()]) }
    }

    pub unsafe fn into_slice<'a, T>(self) -> &'a [T] {
        self.try_into_slice().unwrap_or_default()
    }

    pub unsafe fn try_into_slice<'a, T>(self) -> Option<&'a [T]> {
        let ptr = self.ptr as usize as *const T;
        let len = self.len as usize;

        if ptr.is_null() {
            None
        } else {
            Some(::core::slice::from_raw_parts(ptr, len))
        }
    }

    pub unsafe fn into_vec<T>(self) -> Vec<T> {
        self.try_into_vec().unwrap_or_default()
    }

    pub unsafe fn try_into_vec<T>(self) -> Option<Vec<T>> {
        let ptr = self.ptr as usize as *mut T;
        let len = self.len as usize;

        if ptr.is_null() {
            None
        } else {
            Some(Vec::from_raw_parts(ptr, len, len))
        }
    }

    pub unsafe fn assume_error(self) -> Result<()> {
        // consume owner
        match self.try_into_vec() {
            Some(vec) => bail!(String::from_utf8_unchecked(vec)),
            None => Ok(()),
        }
    }
}

#[cfg(target_os = "wasi")]
pub type ExternDataRef = usize;
#[cfg(not(target_os = "wasi"))] // wasm32-wasi
pub type ExternDataRef = u32;
