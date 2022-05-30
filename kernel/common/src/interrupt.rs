use ipis::core::anyhow::Result;

use crate::data::ExternData;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct InterruptId(pub InterruptIdInner);

impl ::core::fmt::LowerHex for InterruptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ::core::fmt::LowerHex::fmt(&self.0, f)
    }
}

impl InterruptId {
    pub unsafe fn syscall_unchecked(&self, inputs: &mut ExternData) -> Result<ExternData> {
        // initiate I/O placeholders
        let mut outputs = ExternData::default();
        let mut errors = ExternData::default();

        // execute syscall
        crate::extrinsics::syscall(
            self.0,
            inputs.as_mut_ptr(),
            outputs.as_mut_ptr(),
            errors.as_mut_ptr(),
        );

        // parse result
        errors.assume_error().map(|()| outputs)
    }
}

pub type InterruptIdInner = u32;
