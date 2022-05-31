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
