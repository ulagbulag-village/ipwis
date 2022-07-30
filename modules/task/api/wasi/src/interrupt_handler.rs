use ipis::{async_trait::async_trait, core::anyhow::Result};
use ipwis_modules_core_common::resource::Resource;
use rkyv::AlignedVec;

use crate::memory::Memory;

#[async_trait]
pub trait InterruptHandler<M>
where
    Self: Resource + Send + Sync + 'static,
    M: Memory,
{
    async unsafe fn handle_raw(&mut self, memory: &mut M, inputs: &[u8]) -> Result<AlignedVec>;
}
