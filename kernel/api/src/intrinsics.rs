#[allow(clippy::missing_safety_doc)]
#[cfg(target_os = "wasi")]
pub(crate) mod memory {
    use std::alloc;

    use ipwis_kernel_common::data::ExternDataRef;

    #[no_mangle]
    pub unsafe extern "C" fn __alloc(size: usize, align: usize) -> *mut u8 {
        alloc::alloc(alloc::Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    pub unsafe extern "C" fn __alloc_zeroed(size: usize, align: usize) -> *mut u8 {
        alloc::alloc_zeroed(alloc::Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    pub unsafe extern "C" fn __dealloc(ptr: *mut u8, size: usize, align: usize) {
        alloc::dealloc(ptr, alloc::Layout::from_size_align_unchecked(size, align))
    }

    #[no_mangle]
    pub unsafe extern "C" fn __realloc(
        ptr: *mut u8,
        size: usize,
        align: usize,
        new_size: usize,
    ) -> *mut u8 {
        alloc::realloc(
            ptr,
            alloc::Layout::from_size_align_unchecked(size, align),
            new_size,
        )
    }

    #[no_mangle]
    pub unsafe extern "C" fn __ipwis_syscall(
        handler: ExternDataRef,
        inputs: ExternDataRef,
        outputs: ExternDataRef,
        errors: ExternDataRef,
    ) -> ExternDataRef {
        ::ipwis_kernel_common::extrinsics::__ipwis_syscall(handler, inputs, outputs, errors)
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) mod memory {
    use ipis::core::anyhow::{anyhow, Result};
    use ipwis_kernel_common::data::ExternDataRef;
    use wasmtime::{AsContextMut, Caller, Instance, TypedFunc};

    pub type IpwisAlloc = TypedFunc<(ExternDataRef, ExternDataRef), ExternDataRef>;

    pub type IpwisAllocZeroed = TypedFunc<(ExternDataRef, ExternDataRef), ExternDataRef>;

    pub type IpwisDealloc = TypedFunc<(ExternDataRef, ExternDataRef, ExternDataRef), ()>;

    pub type IpwisRealloc =
        TypedFunc<(ExternDataRef, ExternDataRef, ExternDataRef, ExternDataRef), ExternDataRef>;

    pub mod caller {
        use super::{super::common::caller as common, *};

        pub fn __builtin_memory<T>(caller: &mut Caller<T>) -> Result<::wasmtime::Memory> {
            caller
                .get_export("memory")
                .and_then(::wasmtime::Extern::into_memory)
                .ok_or_else(|| anyhow!("failed to find `memory` export"))
        }

        pub fn __alloc<T>(caller: &mut Caller<T>) -> Result<IpwisAlloc> {
            common::load_extern(caller, "__alloc")
        }

        pub fn __alloc_zeroed<T>(caller: &mut Caller<T>) -> Result<IpwisAllocZeroed> {
            common::load_extern(caller, "__alloc_zeroed")
        }

        pub fn __dealloc<T>(caller: &mut Caller<T>) -> Result<IpwisDealloc> {
            common::load_extern(caller, "__dealloc")
        }

        pub fn __realloc<T>(caller: &mut Caller<T>) -> Result<IpwisRealloc> {
            common::load_extern(caller, "__realloc")
        }
    }

    pub mod instance {
        use super::{super::common::instance as common, *};

        pub fn __builtin_memory<S>(instance: &Instance, mut store: S) -> Result<::wasmtime::Memory>
        where
            S: AsContextMut,
        {
            instance
                .get_export(&mut store, "memory")
                .and_then(::wasmtime::Extern::into_memory)
                .ok_or_else(|| anyhow!("failed to find `memory` export"))
        }

        pub fn __alloc<S>(instance: &Instance, store: S) -> Result<IpwisAlloc>
        where
            S: AsContextMut,
        {
            common::load_extern(instance, store, "__alloc")
        }

        pub fn __alloc_zeroed<S>(instance: &Instance, store: S) -> Result<IpwisAllocZeroed>
        where
            S: AsContextMut,
        {
            common::load_extern(instance, store, "__alloc_zeroed")
        }

        pub fn __dealloc<S>(instance: &Instance, store: S) -> Result<IpwisDealloc>
        where
            S: AsContextMut,
        {
            common::load_extern(instance, store, "__dealloc")
        }

        pub fn __realloc<S>(instance: &Instance, store: S) -> Result<IpwisRealloc>
        where
            S: AsContextMut,
        {
            common::load_extern(instance, store, "__realloc")
        }
    }
}

#[cfg(not(target_os = "wasi"))]
mod common {
    pub mod caller {
        pub fn load_extern<T, Params, Results>(
            caller: &mut ::wasmtime::Caller<T>,
            name: &'static str,
        ) -> ::ipis::core::anyhow::Result<::wasmtime::TypedFunc<Params, Results>>
        where
            Params: ::wasmtime::WasmParams,
            Results: ::wasmtime::WasmResults,
        {
            caller
                .get_export(name)
                .and_then(::wasmtime::Extern::into_func)
                .ok_or_else(|| ::ipis::core::anyhow::anyhow!("failed to find `{}` export", name))
                .and_then(|e| e.typed(&caller).map_err(Into::into))
        }
    }

    pub mod instance {
        pub fn load_extern<S, Params, Results>(
            instance: &::wasmtime::Instance,
            mut store: S,
            name: &'static str,
        ) -> ::ipis::core::anyhow::Result<::wasmtime::TypedFunc<Params, Results>>
        where
            S: ::wasmtime::AsContextMut,
            Params: ::wasmtime::WasmParams,
            Results: ::wasmtime::WasmResults,
        {
            instance
                .get_export(&mut store, name)
                .and_then(::wasmtime::Extern::into_func)
                .ok_or_else(|| ::ipis::core::anyhow::anyhow!("failed to find `{}` export", name))
                .and_then(|e| e.typed(&store).map_err(Into::into))
        }
    }
}
