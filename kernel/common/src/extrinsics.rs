use crate::{data::ExternDataRef, interrupt::InterruptIdInner};

#[link(wasm_import_module = "ipwis_kernel")]
extern "C" {
    pub fn syscall(
        id: InterruptIdInner,
        inputs: ExternDataRef,
        outputs: ExternDataRef,
        errors: ExternDataRef,
    );
}

pub type InterruptFn = unsafe extern "C" fn(
    id: InterruptIdInner,
    inputs: ExternDataRef,
    outputs: ExternDataRef,
    errors: ExternDataRef,
);
pub type InterruptArgs = (
    /* id */ InterruptIdInner,
    /* inputs */ ExternDataRef,
    /* outputs */ ExternDataRef,
    /* errors */ ExternDataRef,
);
