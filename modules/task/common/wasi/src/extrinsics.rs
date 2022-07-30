pub mod memory {
    pub const MEMORY: &str = "memory";
    pub const IPWIS_ALLOC: &str = "__ipwis_alloc";
    pub const IPWIS_ALLOC_ZEROED: &str = "__ipwis_alloc_zeroed";
    pub const IPWIS_DEALLOC: &str = "__ipwis_dealloc";
    pub const IPWIS_REALLOC: &str = "__ipwis_realloc";
}

pub mod syscall {
    use crate::extern_data::ExternDataRef;

    pub const MODULE: &str = "__ipwis_syscall";

    pub const SYSCALL: &str = "__ipwis_syscall";
    pub const SYSCALL_OK: ExternDataRef = 0;
    pub const SYSCALL_ERR_NORMAL: ExternDataRef = 1;
    pub const SYSCALL_ERR_FATAL: ExternDataRef = 2;

    #[cfg(target_os = "wasi")]
    #[link(wasm_import_module = "__ipwis_syscall")]
    extern "C" {
        pub(crate) fn __ipwis_syscall(
            handler: ExternDataRef,
            inputs: ExternDataRef,
            outputs: ExternDataRef,
            errors: ExternDataRef,
        ) -> ExternDataRef;
    }
}
