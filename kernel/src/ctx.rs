use std::sync::Arc;

use ipis::{core::anyhow::Result, tokio::sync::Mutex};
use ipwis_kernel_api::wasmtime::{Caller, Engine, Linker, Store};
use ipwis_kernel_api::wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use ipwis_kernel_common::task::{TaskPtr, TaskState};

use crate::{
    interrupt::{InterruptHandlerStore, InterruptManager},
    task::{Task, TaskStore},
};

pub type IpwisCaller<'a> = Caller<'a, IpwisCtx>;
pub type IpwisLinker = Linker<IpwisCtx>;
pub type IpwisStore = Store<IpwisCtx>;

pub struct IpwisCtx {
    pub wasi: WasiCtx,
    pub task: TaskPtr,
    pub state: Arc<Mutex<TaskState>>,
    pub store: TaskStore<Task>,
    pub interrupt_handlers: InterruptHandlerStore,
}

impl IpwisCtx {
    pub fn new(
        engine: &Engine,
        ctx: TaskPtr,
        state: TaskState,
        interrupt_manager: Arc<InterruptManager>,
    ) -> Result<Self> {
        Ok(Self {
            // create a WASI context and put it in a Store; all instances in the store
            // share this context. `WasiCtxBuilder` provides a number of ways to
            // configure what the target program will have access to.
            wasi: WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_args()?
                .build(),
            task: ctx,
            state: Arc::new(Mutex::new(state)),
            store: TaskStore::try_new(engine, interrupt_manager.clone())?,
            interrupt_handlers: InterruptHandlerStore::with_manager(interrupt_manager),
        })
    }

    pub async fn release(&mut self) -> Result<()> {
        // order: Task -> Interrupt Store
        self.store.release().await;
        self.interrupt_handlers.release().await?;
        Ok(())
    }
}
