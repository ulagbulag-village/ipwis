use std::sync::Arc;

use ipis::{core::anyhow::Result, tokio::sync::Mutex};
use ipwis_kernel_common::task::{TaskCtx, TaskState};
use wasmtime::{Caller, Linker, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

use crate::{
    interrupt::InterruptHandlerStore,
    task::{Task, TaskStore},
};

pub type IpwisCaller<'a> = Caller<'a, IpwisCtx>;
pub type IpwisLinker = Linker<IpwisCtx>;
pub type IpwisStore = Store<IpwisCtx>;

pub struct IpwisCtx {
    pub wasi: WasiCtx,
    pub task: *const TaskCtx,
    pub state: Arc<Mutex<TaskState>>,
    pub store: TaskStore<Task>,
}

/// # Safety
///
/// It's thread-safe as the task is read-only and is owned by Entry.
unsafe impl Send for IpwisCtx {}

impl IpwisCtx {
    pub fn new(
        ctx: *const TaskCtx,
        state: TaskState,
        interrupt_handlers: Arc<InterruptHandlerStore>,
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
            store: TaskStore::new(interrupt_handlers),
        })
    }
}
