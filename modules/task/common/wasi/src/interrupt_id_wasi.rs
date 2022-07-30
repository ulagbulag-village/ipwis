use bytecheck::CheckBytes;
use ipis::{
    core::{
        anyhow::Result,
        signed::{IsSigned, Serializer},
    },
    pin::PinnedInner,
};
use rkyv::{
    de::deserializers::SharedDeserializeMap, validation::validators::DefaultValidator, Archive,
    Deserialize, Serialize,
};

use crate::{extern_data::ExternData, extrinsics::syscall, interrupt_id::InterruptId};

impl InterruptId {
    pub unsafe fn syscall<I, O>(&self, inputs: &mut I) -> Result<O>
    where
        I: Serialize<Serializer> + IsSigned + Send + Sync,
        O: Archive,
        <O as Archive>::Archived:
            for<'a> CheckBytes<DefaultValidator<'a>> + Deserialize<O, SharedDeserializeMap>,
    {
        let inputs = inputs.to_bytes()?;

        let outputs = self.syscall_raw(&inputs)?;

        PinnedInner::deserialize_owned(outputs)
    }

    pub unsafe fn syscall_raw(&self, inputs: &[u8]) -> Result<Vec<u8>> {
        // initiate I/O placeholders
        let handler = ExternData::from_slice(self.0.as_bytes());
        let inputs = ExternData::from_slice(inputs);
        let mut outputs = ExternData::default();
        let mut errors = ExternData::default();

        // execute syscall
        syscall::__ipwis_syscall(
            handler.as_ptr(),
            inputs.as_ptr(),
            outputs.as_mut_ptr(),
            errors.as_mut_ptr(),
        );

        // try parsing error
        errors.assume_error()?;

        // parse result
        let ptr = outputs.ptr as *mut u8;
        let len = outputs.len as usize;
        Ok(Vec::from_raw_parts(ptr, len, len))
    }
}
