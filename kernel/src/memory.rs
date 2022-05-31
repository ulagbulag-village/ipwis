use ipwis_kernel_common::data::{ExternData, ExternDataRef};
use wasmtime::{AsContext, AsContextMut, Caller, Extern, Instance, Memory, TypedFunc};

pub struct IpwisMemory<S> {
    pub store: S,
    alloc: Alloc,
    memory: Memory,
}

impl<'a, 'c, T> IpwisMemory<&'c mut Caller<'a, T>> {
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

impl<S> IpwisMemory<S>
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

impl<S> IpwisMemory<S>
where
    S: AsContextMut,
{
    pub unsafe fn is_null(&self, data: ExternDataRef) -> bool {
        (data as *const ExternData).is_null()
    }

    pub unsafe fn load(&self, data: ExternDataRef) -> &[u8] {
        self.load_raw(data as *const ExternData)
    }

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

    pub unsafe fn load_mut(&mut self, data: ExternDataRef) -> &mut [u8] {
        self.load_mut_raw(data as *const ExternData)
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

    pub unsafe fn dump(&mut self, data: &[u8]) -> ExternData {
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

    pub unsafe fn copy(&mut self, src: &[u8], dst: ExternDataRef) {
        self.copy_raw(src, dst as *mut ExternData)
    }

    unsafe fn copy_raw(&mut self, src: &[u8], dst: *mut ExternData) {
        *dst = self.dump(src);
    }

    pub unsafe fn copy_error(&mut self, err: ::ipis::core::anyhow::Error, dst: ExternDataRef) {
        self.copy(&err.to_string().into_bytes(), dst);
    }

    pub unsafe fn set_len(&mut self, len: u32, dst: ExternDataRef) {
        self.set_len_raw(len, dst as *mut ExternData)
    }

    unsafe fn set_len_raw(&mut self, len: u32, dst: *mut ExternData) {
        (*dst).len = len;
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
