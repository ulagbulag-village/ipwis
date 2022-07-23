use bytecheck::CheckBytes;
use ipis::{
    async_trait::async_trait,
    core::{
        anyhow::Result,
        signed::{IsSigned, Serializer},
    },
    pin::PinnedInner,
};
use rkyv::{
    de::deserializers::SharedDeserializeMap, validation::validators::DefaultValidator, AlignedVec,
    Archive, Deserialize, Serialize,
};

use crate::{data::ExternData, memory::Memory};

#[async_trait]
pub trait InterruptHandler<M>
where
    Self: Send + Sync,
    M: Memory,
{
    async unsafe fn handle_raw(&mut self, memory: &mut M, inputs: &[u8]) -> Result<AlignedVec>;

    async fn release(&mut self) -> Result<()>;
}

#[async_trait]
pub trait InterruptFallbackHandler<M>
where
    Self: InterruptHandler<M> + Send + Sync,
    M: Memory,
{
    async fn handle_fallback(
        &self,
        memory: &mut M,
        id: InterruptId,
        inputs: &[u8],
    ) -> Result<AlignedVec>;
}

#[async_trait]
pub trait InterruptModule<M>
where
    Self: Send + Sync,
    M: Memory,
{
    fn id(&self) -> InterruptId;

    async fn spawn_handler(&self) -> Result<Box<dyn InterruptHandler<M>>>;
}

#[async_trait]
pub trait InterruptFallbackModule<M>
where
    Self: InterruptModule<M> + Send + Sync,
    M: Memory,
{
    async fn spawn_fallback(&self) -> Result<Box<dyn InterruptFallbackHandler<M>>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InterruptId(pub &'static str);

impl ::core::fmt::Display for InterruptId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InterruptHandler({})", &self.0)
    }
}

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
        crate::extrinsics::syscall(
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
