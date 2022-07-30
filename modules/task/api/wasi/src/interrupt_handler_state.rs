use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use ipis::{async_trait::async_trait, core::anyhow::Result};
use ipwis_modules_core_common::resource::Resource;
use ipwis_modules_task_common_wasi::interrupt_id::InterruptId;
use rkyv::AlignedVec;
use wasmtime::Caller;

use crate::{
    interrupt_handler::InterruptHandler, memory::IpwisMemory, task_ctx::IpwisTaskCtx,
    task_manager::IpwisTaskManager,
};

pub(crate) type IpwisInterruptHandler =
    Box<dyn InterruptHandler<IpwisMemory<&'static mut Caller<'static, IpwisTaskCtx>>>>;

pub struct InterruptHandlerState {
    manager: Arc<IpwisTaskManager>,
    map: HashMap<InterruptId, IpwisInterruptHandler>,
}

impl InterruptHandlerState {
    pub fn with_manager(manager: Arc<IpwisTaskManager>) -> Self {
        Self {
            manager,
            map: Default::default(),
        }
    }
}

impl InterruptHandlerState {
    pub async unsafe fn syscall_raw(
        &mut self,
        memory: &mut IpwisMemory<&'static mut Caller<'static, IpwisTaskCtx>>,
        handler: InterruptId,
        inputs: &[u8],
    ) -> Result<AlignedVec> {
        // load interrupt module
        if let Entry::Vacant(e) = self.map.entry(handler) {
            e.insert(self.manager.interrupt_manager.get(&handler).await?);
        }
        let handler = self.map.get_mut(&handler).unwrap();

        handler.handle_raw(memory, inputs).await
    }
}

#[async_trait]
impl Resource for InterruptHandlerState {
    async fn release(&mut self) -> Result<()> {
        for (_, mut handler) in self.map.drain() {
            handler.release().await?;
        }
        Ok(())
    }
}
