use core::{
    future::Future,
    sync::atomic::{AtomicU32, Ordering},
};
use std::{collections::BTreeMap, sync::Arc};

use ipis::{
    core::{
        account::GuarantorSigned,
        anyhow::{anyhow, Result},
        signed::IsSigned,
        value::chrono::DateTime,
    },
    tokio::{self, sync::Mutex},
};
use ipwis_kernel_common::{
    extrinsics::InterruptArgs,
    protection::ProtectionMode,
    task::{TaskCtx, TaskId, TaskState},
};
use wasmtime::{Module, Trap};

use crate::{
    ctx::{IpwisCtx, IpwisLinker, IpwisStore},
    interrupt::InterruptHandlerStore,
    memory::IpwisMemory,
};

pub struct TaskStore<T> {
    seed: TaskIdSeed,
    map: Mutex<BTreeMap<TaskId, T>>,
    interrupt_handlers: Arc<InterruptHandlerStore>,
}

impl<T> TaskStore<T> {
    pub fn new(interrupt_handlers: Arc<InterruptHandlerStore>) -> Self {
        Self {
            seed: Default::default(),
            map: Default::default(),
            interrupt_handlers,
        }
    }

    async fn spawn_inner<F>(
        &self,
        linker: &IpwisLinker,
        module: &Module,
        ctx: *const TaskCtx,
        f: F,
    ) -> Result<TaskId>
    where
        F: FnOnce(Task) -> T,
    {
        let id = self.seed.generate();

        // create a new state
        let state = TaskState {
            id,
            inputs: Default::default(), // uninitialized
            outputs: Default::default(),
            errors: Default::default(),
            created_date: DateTime::now(),
            protection_mode: ProtectionMode::Entry,
        };

        // create a new store
        let mut store = IpwisStore::new(
            linker.engine(),
            IpwisCtx::new(ctx, state, self.interrupt_handlers.clone())?,
        );
        let ctx = store.data().task;
        let state = store.data().state.clone();

        // create an instance with given module and store
        let instance = linker.instantiate_async(&mut store, module).await?;

        // copy inputs into instance
        let inputs = unsafe {
            let inputs = (*ctx).constraints.inputs.to_bytes()?;

            let mut memory = IpwisMemory::from_instance(&mut store, &instance);
            memory.dump(&inputs)
        };
        {
            state.lock().await.inputs = inputs;
        }

        // find main function
        let func = instance
            .get_func(&mut store, "_ipwis_call")
            .expect("failed to find `_ipwis_call` func")
            .typed::<InterruptArgs, (), _>(&mut store)
            .expect("failed to parse `_ipwis_call` func");

        // external call
        // note: the inner schedule is controlled by `wasmtime` engine, not by this scheduler
        let handler = tokio::spawn(async move {
            func.call_async(&mut store, (0, 0, 0, 0))
                .await
                .map(|()| store)
        });

        // instantiate and store the task
        let task = f(Task {
            ctx,
            state,
            handler,
        });
        {
            self.map.lock().await.insert(id, task);
        }

        Ok(id)
    }

    pub async fn lock_and_wait(&self, id: TaskId) -> Result<IpwisStore>
    where
        T: Future<Output = Result<TaskResult, tokio::task::JoinError>>,
    {
        let task = self
            .map
            .lock()
            .await
            .remove(&id)
            .ok_or_else(|| anyhow!("failed to find the task: {id:x}"))?;

        task.await
            .map_err(Into::into)
            .and_then(|e| e.map_err(Into::into))
    }
}

impl TaskStore<Entry> {
    pub async fn spawn_entry(
        &self,
        linker: &IpwisLinker,
        module: &Module,
        ctx: GuarantorSigned<TaskCtx>,
    ) -> Result<TaskId> {
        let ctx = Box::new(ctx);

        self.spawn_inner(
            linker,
            module,
            (&ctx.data.data.data) as *const TaskCtx,
            |task| Entry { ctx, task },
        )
        .await
    }
}

impl TaskStore<Task> {
    pub async fn spawn_task(
        &self,
        linker: &IpwisLinker,
        module: &Module,
        ctx: *const TaskCtx,
    ) -> Result<TaskId> {
        self.spawn_inner(linker, module, ctx, |task| task).await
    }
}

#[derive(Debug)]
struct TaskIdSeed(AtomicU32);

impl Default for TaskIdSeed {
    fn default() -> Self {
        Self(1.into())
    }
}

impl TaskIdSeed {
    pub fn generate(&self) -> TaskId {
        TaskId(self.0.fetch_add(1, Ordering::SeqCst))
    }
}

pub type Entry = ::ipwis_kernel_common::task::Entry<TaskResult>;
pub type Task = ::ipwis_kernel_common::task::Task<TaskResult>;
type TaskResult = Result<IpwisStore, Trap>;
