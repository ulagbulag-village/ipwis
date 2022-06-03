use ipwis_kernel_common::data::{ExternData, ExternDataRef};
use wasmtime::{AsContext, AsContextMut, Caller, Extern, Instance, Memory, TypedFunc};

use crate::ctx::IpwisCaller;

pub type IpwisMemory<'a> = IpwisMemoryInner<&'a mut IpwisCaller<'a>>;

pub struct IpwisMemoryInner<S> {
    pub store: S,
    alloc: Alloc,
    memory: Memory,
}

impl<'a, 'c, T> IpwisMemoryInner<&'c mut Caller<'a, T>> {
    pub fn from_caller(caller: &'c mut Caller<'a, T>) -> Self {
        Self {
            alloc: {
                let ext = caller.get_export("alloc");
                get_alloc_from_extern(&caller, ext)
            },
            memory: {
                let ext = caller.get_export("memory");
                get_memory_from_extern(ext)
            },
            store: caller,
        }
    }
}

impl<S> IpwisMemoryInner<S>
where
    S: AsContextMut,
{
    pub fn from_instance(mut store: S, instance: &Instance) -> Self {
        Self {
            alloc: {
                let ext = instance.get_export(&mut store, "alloc");
                get_alloc_from_extern(&store, ext)
            },
            memory: {
                let ext = instance.get_export(&mut store, "memory");
                get_memory_from_extern(ext)
            },
            store,
        }
    }
}

impl<S> ::ipwis_kernel_common::memory::Memory for IpwisMemoryInner<S>
where
    S: AsContextMut + Send + Sync,
{
    unsafe fn load_raw(&self, data: *const ExternData) -> &[u8] {
        let ptr = self
            .memory
            .data_ptr(&self.store)
            .offset((*data).ptr as isize) as *mut u8;
        let len = (*data).len as usize;

        match self
            .memory
            .data(&self.store)
            .get((ptr as usize)..)
            .and_then(|s| s.get(..len))
        {
            Some(slice) => slice,
            None => panic!("invalid memory access"),
        }
    }

    unsafe fn load_mut_raw(&mut self, data: *const ExternData) -> &mut [u8] {
        let ptr = self
            .memory
            .data_ptr(&self.store)
            .offset((*data).ptr as isize) as *mut u8;
        let len = (*data).len as usize;

        match self
            .memory
            .data_mut(&mut self.store)
            .get_mut((ptr as usize)..)
            .and_then(|s| s.get_mut(..len))
        {
            Some(slice) => slice,
            None => panic!("invalid memory access"),
        }
    }

    unsafe fn dump(&mut self, data: &[u8]) -> ExternData {
        let len = data.len();
        let ptr = self
            .alloc
            .call(&mut self.store, len as ExternDataRef)
            .expect("failed to allocate memory: {len}") as *mut u8;

        ::core::ptr::copy(data.as_ptr(), ptr, len);

        ExternData {
            ptr: ptr as ExternDataRef,
            len: len as ExternDataRef,
        }
    }

    unsafe fn copy_raw(&mut self, src: &[u8], dst: *mut ExternData) {
        *dst = self.dump(src);
    }
}

fn get_alloc_from_extern<S>(store: S, ext: Option<Extern>) -> Alloc
where
    S: AsContext,
{
    ext.and_then(Extern::into_func)
        .expect("failed to find `alloc` export")
        .typed(store)
        .expect("failed to load `alloc` export")
}

fn get_memory_from_extern(ext: Option<Extern>) -> Memory {
    ext.and_then(Extern::into_memory)
        .expect("failed to find `memory` export")
}

type Alloc = TypedFunc<u32, u32>;
