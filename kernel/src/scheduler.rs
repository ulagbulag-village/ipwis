use std::sync::Arc;

use ipis::core::{account::GuarantorSigned, anyhow::Result};
use ipwis_kernel_common::task::{TaskCtx, TaskId};
use wasmtime::{Config, Engine, Module};

use crate::{
    ctx::{IpwisCtx, IpwisLinker, IpwisStore},
    interrupt::InterruptHandlerStore,
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
        ::wasmtime_wasi::add_to_linker(&mut linker, |ctx| &mut ctx.wasi)?;

        // register extrinsics
        crate::extrinsics::register(&mut linker)?;

        // create the other modules
        let interrupt_handlers: Arc<InterruptHandlerStore> = Default::default();
        let tasks = TaskStore::new(interrupt_handlers);

        Ok(Self { linker, tasks })
    }

    pub async fn spawn(&self, ctx: GuarantorSigned<TaskCtx>, program: &[u8]) -> Result<TaskId> {
        // load a module from given binary
        let module = Module::from_binary(self.linker.engine(), program)?;

        // spawn
        self.tasks.spawn_entry(&self.linker, &module, ctx).await
    }

    pub async fn lock_and_wait_raw(&self, id: TaskId) -> Result<IpwisCtx> {
        self.tasks
            .lock_and_wait(id)
            .await
            .map(IpwisStore::into_data)
    }
}
