use core::mem::MaybeUninit;

use ipis::core::anyhow::{bail, Result};

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct ExternData {
    pub ptr: u32,
    pub len: u32,
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
            ptr: ptr as u32,
            len: len as u32,
        }
    }

    pub fn with_raw_parts_mut(ptr: *mut u8, len: usize) -> Self {
        Self {
            ptr: ptr as u32,
            len: len as u32,
        }
    }

    pub fn as_ptr(&self) -> u32 {
        self as *const Self as u32
    }

    pub fn as_mut_ptr(&mut self) -> u32 {
        self as *mut Self as u32
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

pub type ExternDataRef = u32;
