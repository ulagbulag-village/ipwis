#[allow(clippy::missing_safety_doc)]
#[cfg(target_os = "wasi")]
pub(crate) mod memory {
    use std::alloc;

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
    pub unsafe extern "C" fn __ipwis_test() {
        _ipwis_test()
    }

    #[link(wasm_import_module = "__ipwis_custom")]
    extern "C" {
        fn _ipwis_test();
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) mod memory {
    use ipis::core::anyhow::{anyhow, Result};
    use ipwis_kernel_common::data::ExternDataRef;
    use wasmtime::{Caller, TypedFunc};

    pub type IpwisAlloc = TypedFunc<(ExternDataRef, ExternDataRef), ExternDataRef>;

    pub fn __builtin_memory<T>(caller: &mut Caller<T>) -> Result<::wasmtime::Memory> {
        caller
            .get_export("memory")
            .and_then(::wasmtime::Extern::into_memory)
            .ok_or_else(|| anyhow!("failed to find `memory` export"))
    }

    pub fn __alloc<T>(caller: &mut Caller<T>) -> Result<IpwisAlloc> {
        super::load_extern(caller, "__alloc")
    }

    pub type IpwisAllocZeroed = TypedFunc<(ExternDataRef, ExternDataRef), ExternDataRef>;

    pub fn __alloc_zeroed<T>(caller: &mut Caller<T>) -> Result<IpwisAllocZeroed> {
        super::load_extern(caller, "__alloc_zeroed")
    }

    pub type IpwisDealloc = TypedFunc<(ExternDataRef, ExternDataRef, ExternDataRef), ()>;

    pub fn __dealloc<T>(caller: &mut Caller<T>) -> Result<IpwisDealloc> {
        super::load_extern(caller, "__dealloc")
    }

    pub type IpwisRealloc =
        TypedFunc<(ExternDataRef, ExternDataRef, ExternDataRef, ExternDataRef), ExternDataRef>;

    pub fn __realloc<T>(caller: &mut Caller<T>) -> Result<IpwisRealloc> {
        super::load_extern(caller, "__realloc")
    }
}

#[cfg(not(target_os = "wasi"))]
fn load_extern<T, Params, Results>(
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
