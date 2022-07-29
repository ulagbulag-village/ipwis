use std::sync::Arc;

use ipis::core::{account::GuarantorSigned, anyhow::Result};
use ipwis_kernel_api::wasmtime::{Config, Engine, Module};
use ipwis_kernel_common::{
    resource::ResourceId,
    task::{TaskCtx, TaskId},
};

use crate::{
    ctx::{IpwisCtx, IpwisLinker},
    interrupt::InterruptManager,
    task::{Entry, TaskStore},
};

pub struct Scheduler {
    linker: IpwisLinker,
    tasks: TaskStore<Entry>,
}

impl Scheduler {
    pub async fn new() -> Result<Self> {
        // define the WASI functions globally on the `Config`.
        let engine = Engine::new(Config::new().async_support(true))?;

        let mut linker = IpwisLinker::new(&engine);
        ::ipwis_kernel_api::wasmtime_wasi::add_to_linker(&mut linker, |ctx| &mut ctx.wasi)?;

        // register extrinsics
        crate::extrinsics::register(&mut linker)?;

        // create the other modules
        let interrupt_manager: Arc<InterruptManager> = Default::default();
        let tasks = TaskStore::try_new(&engine, interrupt_manager)?;

        Ok(Self { linker, tasks })
    }

    pub async fn spawn(
        &self,
        id: ResourceId,
        ctx: GuarantorSigned<TaskCtx>,
        program: &[u8],
    ) -> Result<TaskId> {
        // load a module from given binary
        let module = Module::from_binary(self.linker.engine(), program)?;

        // spawn
        self.tasks
            .spawn_entry(&self.linker, &module, id, ctx.into())
            .await
    }

    pub async fn poll(&self, id: TaskId) -> Result<Option<IpwisCtx>> {
        self.tasks.poll_entry(id).await
    }
}
