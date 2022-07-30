use std::collections::{hash_map::Entry, HashMap};

use ipis::{
    core::anyhow::{anyhow, bail, Result},
    tokio::sync::Mutex,
};
use ipwis_modules_task_common_wasi::interrupt_id::InterruptId;
use wasmtime::Caller;

use crate::{
    interrupt_handler_state::IpwisInterruptHandler, interrupt_module::InterruptModule,
    memory::IpwisMemory, task_ctx::IpwisTaskCtx,
};

type IpwisInterruptModule =
    Box<dyn InterruptModule<IpwisMemory<&'static mut Caller<'static, IpwisTaskCtx>>>>;

#[derive(Default)]
pub struct InterruptManager {
    map: Mutex<HashMap<InterruptId, IpwisInterruptModule>>,
}

impl InterruptManager {
    pub async fn get(&self, id: &InterruptId) -> Result<IpwisInterruptHandler> {
        let map = self.map.lock().await;
        let module = map
            .get(id)
            .ok_or_else(|| anyhow!("failed to find the interrupt module: {id}"))?;

        module.spawn_handler().await
    }

    pub async fn put<T>(&self, module: T) -> Result<()>
    where
        T: InterruptModule<IpwisMemory<&'static mut Caller<'static, IpwisTaskCtx>>>,
    {
        match self.map.lock().await.entry(module.id()) {
            Entry::Vacant(e) => {
                e.insert(Box::new(module));
                Ok(())
            }
            Entry::Occupied(e) => bail!("duplicated interrupt module: {}", e.key()),
        }
    }
}
