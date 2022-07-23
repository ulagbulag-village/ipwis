use crate::data::ExternDataRef;

#[link(wasm_import_module = "ipwis_kernel")]
extern "C" {
    pub fn syscall(
        handler: ExternDataRef,
        inputs: ExternDataRef,
        outputs: ExternDataRef,
        errors: ExternDataRef,
    );
}

pub type InterruptFn = unsafe extern "C" fn(
    handler: ExternDataRef,
    inputs: ExternDataRef,
    outputs: ExternDataRef,
    errors: ExternDataRef,
);
pub type InterruptArgs = (
    /* handler */ ExternDataRef,
    /* inputs */ ExternDataRef,
    /* outputs */ ExternDataRef,
    /* errors */ ExternDataRef,
);

pub const SYSCALL_OK: ExternDataRef = 0;
pub const SYSCALL_ERR_NORMAL: ExternDataRef = 1;
pub const SYSCALL_ERR_FATAL: ExternDataRef = 2;
