use ipis::{
    async_trait::async_trait,
    core::anyhow::{bail, Result},
};
use ipwis_kernel_common::{
    data::{ExternData, ExternDataRef},
    memory::Memory,
};
use wasmtime::{AsContext, AsContextMut, Caller, Instance, Trap};

use crate::intrinsics::memory::{self, IpwisAlloc, IpwisAllocZeroed, IpwisDealloc, IpwisRealloc};

#[allow(dead_code)] // TODO: make more **safe** functions to get rid of it
pub struct IpwisMemoryInner<S> {
    pub store: S,
    memory: ::wasmtime::Memory,
    alloc: IpwisAlloc,
    alloc_zeroed: IpwisAllocZeroed,
    dealloc: IpwisDealloc,
    realloc: IpwisRealloc,
}

impl<'a, 'c, T> IpwisMemoryInner<&'c mut Caller<'a, T>> {
    pub fn with_caller(caller: &'c mut Caller<'a, T>) -> Result<Self> {
        Ok(Self {
            memory: memory::caller::__builtin_memory(caller)?,
            alloc: memory::caller::__alloc(caller)?,
            alloc_zeroed: memory::caller::__alloc_zeroed(caller)?,
            dealloc: memory::caller::__dealloc(caller)?,
            realloc: memory::caller::__realloc(caller)?,
            store: caller,
        })
    }
}

impl<S> IpwisMemoryInner<S>
where
    S: AsContextMut,
{
    pub fn with_instance(instance: &Instance, mut store: S) -> Result<Self> {
        Ok(Self {
            memory: memory::instance::__builtin_memory(instance, &mut store)?,
            alloc: memory::instance::__alloc(instance, &mut store)?,
            alloc_zeroed: memory::instance::__alloc_zeroed(instance, &mut store)?,
            dealloc: memory::instance::__dealloc(instance, &mut store)?,
            realloc: memory::instance::__realloc(instance, &mut store)?,
            store,
        })
    }
}

#[async_trait]
impl<S> Memory for IpwisMemoryInner<S>
where
    S: AsContextMut + Send + Sync,
    S::Data: Send,
{
    fn host_check(&self, data: ExternData) -> Result<()> {
        if data.is_null() {
            bail!("data is null");
        }

        if ((data.ptr + data.len) as usize) >= self.size() {
            bail!("data overflow");
        }

        Ok(())
    }

    unsafe fn host_ptr_unchecked<T>(&self, ptr: ExternDataRef) -> *const T {
        self.memory.data_ptr(&self.store).add(ptr as usize) as *const T
    }

    unsafe fn host_ptr_mut_unchecked<T>(&mut self, ptr: ExternDataRef) -> *mut T {
        self.memory.data_ptr(&self.store).add(ptr as usize) as *mut T
    }

    async fn dump(&mut self, data: &[u8]) -> Result<ExternData> {
        let data_len = data.len();

        let len = data_len as ExternDataRef;
        let align = 1; // u8

        // safety: TODO: checking linear memory's limit
        let ptr = unsafe { self.alloc(len, align) }.await?;

        // safety: the source and destination are already checked
        unsafe { ::core::ptr::copy(data.as_ptr(), self.host_ptr_mut_unchecked(ptr), data_len) };
        Ok(ExternData { ptr, len })
    }
}

#[allow(dead_code)] // TODO: make more **safe** functions to get rid of it
impl<S, T> IpwisMemoryInner<S>
where
    S: AsContextMut<Data = T>,
    T: Send,
{
    async unsafe fn alloc(
        &mut self,
        size: ExternDataRef,
        align: ExternDataRef,
    ) -> Result<ExternDataRef, Trap>
    where
        T: Send,
    {
        self.alloc.call_async(&mut self.store, (size, align)).await
    }

    async unsafe fn alloc_zeroed(
        &mut self,
        size: ExternDataRef,
        align: ExternDataRef,
    ) -> Result<ExternDataRef, Trap>
    where
        T: Send,
    {
        self.alloc_zeroed
            .call_async(&mut self.store, (size, align))
            .await
    }

    async unsafe fn dealloc(
        &mut self,
        ptr: ExternDataRef,
        size: ExternDataRef,
        align: ExternDataRef,
    ) -> Result<(), Trap>
    where
        T: Send,
    {
        self.dealloc
            .call_async(&mut self.store, (ptr, size, align))
            .await
    }

    async unsafe fn realloc(
        &mut self,
        ptr: ExternDataRef,
        size: ExternDataRef,
        align: ExternDataRef,
        new_size: ExternDataRef,
    ) -> Result<ExternDataRef, Trap>
    where
        T: Send,
    {
        self.realloc
            .call_async(&mut self.store, (ptr, size, align, new_size))
            .await
    }
}

impl<S> IpwisMemoryInner<S>
where
    S: AsContext,
{
    fn size(&self) -> usize {
        self.memory.data_size(&self.store)
    }
}

#[cfg(not(target_os = "wasi"))]
#[cfg(test)]
mod tests {
    use ipis::tokio;
    use ipwis_kernel_common::{
        data::{ExternData, ExternDataRef},
        extrinsics::{InterruptArgs, SYSCALL_OK},
        memory::Memory,
        modules::{FUNC_NAME_SYSCALL, MODULE_NAME_API, MODULE_NAME_COMMON},
    };
    use wasmtime::{Caller, Config, Engine, Linker, Store, TypedFunc};
    use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

    use super::IpwisMemoryInner;

    #[tokio::test]
    async fn test_memory_alloc_dealloc() {
        // define the WASI functions globally on the `Config`.
        let engine = Engine::new(Config::new().async_support(true)).unwrap();
        let mut linker = Linker::<WasiCtx>::new(&engine);
        ::wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

        // Create a WASI context and put it in a Store; all instances in the store
        // share this context. `WasiCtxBuilder` provides a number of ways to
        // configure what the target program will have access to.
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()
            .unwrap()
            .build();
        let mut store = Store::new(&engine, wasi);

        // Define our test program
        async fn test(mut caller: Caller<'_, WasiCtx>) -> ExternDataRef {
            // instantiate the memory module
            let mut memory = IpwisMemoryInner::with_caller(&mut caller).unwrap();

            // get the memory size
            let size_old = memory.size();

            // define the alignment
            let step = 8_192;
            let num_iter = 32;
            let align = 1; // u8

            for size in (0..num_iter).map(|size| size * step) {
                unsafe {
                    // create a zeroed array
                    let ptr = memory.alloc_zeroed(size, align).await.unwrap();
                    let slice = memory.load_raw(ExternData { ptr, len: size }).unwrap();

                    // test the array is zeroed
                    assert!(slice.iter().all(|&item| item == 0));

                    // delete the array
                    memory.dealloc(ptr, size, align).await.unwrap();
                }
            }

            // get the growed memory size
            let size_new = memory.size();

            // test the arrays are deallocated
            assert!(size_new - size_old <= (step * num_iter) as usize);

            SYSCALL_OK
        }

        // Register our test program
        linker
            .func_wrap4_async(
                MODULE_NAME_COMMON,
                FUNC_NAME_SYSCALL,
                |caller,
                 _handler: ExternDataRef,
                 _inputs: ExternDataRef,
                 _outputs: ExternDataRef,
                 _errors: ExternDataRef| Box::new(test(caller)),
            )
            .unwrap();

        // Register API module.
        let module = super::super::load_module(&engine).unwrap();
        let instance = linker.instantiate_async(&mut store, &module).await.unwrap();
        linker
            .instance(&mut store, MODULE_NAME_API, instance)
            .unwrap();

        // Run our test program
        let func: TypedFunc<InterruptArgs, ExternDataRef> = instance
            .get_func(&mut store, FUNC_NAME_SYSCALL)
            .unwrap()
            .typed(&mut store)
            .unwrap();
        let output = func.call_async(&mut store, (0, 0, 0, 0)).await.unwrap();
        assert_eq!(output, SYSCALL_OK);
    }
}
