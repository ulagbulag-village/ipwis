use core::sync::atomic::{AtomicU32, Ordering};
use std::{collections::BTreeMap, sync::Arc};

use ipis::{
    core::{
        account::GuarantorSigned,
        anyhow::{bail, Result},
        value::chrono::DateTime,
    },
    tokio::{self, sync::Mutex},
};
use ipwis_kernel_api::{
    memory::IpwisMemoryInner,
    wasmtime::{Engine, Module, Trap},
};
use ipwis_kernel_common::{
    data::ExternDataRef,
    extrinsics::InterruptArgs,
    memory::Memory,
    modules::{FUNC_NAME_SYSCALL, MODULE_NAME_API},
    protection::ProtectionMode,
    resource::ResourceId,
    task::{TaskCtx, TaskId, TaskState},
};

use crate::{
    ctx::{IpwisCtx, IpwisLinker, IpwisStore},
    interrupt::InterruptManager,
};

pub struct TaskStore<T> {
    api: Module,
    seed: TaskIdSeed,
    map: Mutex<BTreeMap<TaskId, T>>,
    interrupt_manager: Arc<InterruptManager>,
}

impl<T> TaskStore<T> {
    pub fn try_new(engine: &Engine, interrupt_manager: Arc<InterruptManager>) -> Result<Self> {
        Ok(Self {
            api: ::ipwis_kernel_api::load_module(engine)?,
            seed: Default::default(),
            map: Default::default(),
            interrupt_manager,
        })
    }

    async fn spawn_inner<F>(
        &self,
        linker: &mut IpwisLinker,
        module: &Module,
        resource_id: ResourceId,
        ctx: Arc<GuarantorSigned<TaskCtx>>,
        f: F,
    ) -> Result<TaskId>
    where
        F: FnOnce(Task) -> T,
    {
        let task_id = self.seed.generate()?;

        // create a new state
        let state = TaskState {
            resource_id,
            task_id,
            inputs: Default::default(), // uninitialized
            outputs: Default::default(),
            errors: Default::default(),
            created_date: DateTime::now(),
            protection_mode: ProtectionMode::Entry,
            is_working: true,
        };

        // create a new store
        let mut store = IpwisStore::new(
            linker.engine(),
            IpwisCtx::new(linker.engine(), ctx, state, self.interrupt_manager.clone())?,
        );
        let ctx = store.data().task.clone();
        let state = store.data().state.clone();

        // register API module
        let api = linker.instantiate_async(&mut store, &self.api).await?;
        linker.instance(&mut store, MODULE_NAME_API, api)?;

        // create an instance with given module and store
        let instance = linker.instantiate_async(&mut store, module).await?;

        // find main function
        let func = instance
            .get_func(&mut store, FUNC_NAME_SYSCALL)
            .expect("failed to find `syscall` func")
            .typed::<InterruptArgs, ExternDataRef, _>(&mut store)
            .expect("failed to parse `syscall` func");

        // external call
        // note: the inner schedule is controlled by `wasmtime` engine, not by this scheduler
        let handler = {
            let state = state.clone();
            let (inputs, outputs, errors) = {
                let mut memory = IpwisMemoryInner::with_instance(&instance, &mut store)?;

                let inputs = memory.dump_doubled_object(&ctx.constraints.inputs).await?;
                let outputs = memory.dump_doubled_null().await?;
                let errors = memory.dump_doubled_null().await?;
                (inputs, outputs, errors)
            };

            tokio::spawn(async move {
                func.call_async(
                    &mut store,
                    (0 /* nullptr */, inputs.ptr, outputs.ptr, errors.ptr),
                )
                .await?;

                state.lock().await.is_working = false;

                Ok(store)
            })
        };

        // instantiate and store the task
        let task = f(Task {
            ctx,
            state,
            handler,
        });
        {
            self.map.lock().await.insert(task_id, task);
        }

        Ok(task_id)
    }
}

impl TaskStore<Entry> {
    pub async fn spawn_entry(
        &self,
        linker: &IpwisLinker,
        module: &Module,
        id: ResourceId,
        ctx: Arc<GuarantorSigned<TaskCtx>>,
    ) -> Result<TaskId> {
        self.spawn_inner(&mut linker.clone(), module, id, ctx.clone(), |task| Entry {
            ctx,
            task,
        })
        .await
    }

    pub async fn poll_entry(&self, id: TaskId) -> Result<Option<IpwisCtx>> {
        let mut map = self.map.lock().await;
        match map.get(&id) {
            Some(entry) if !entry.task.state.lock().await.is_working => {
                let mut task = map.remove(&id).unwrap().await??.into_data();
                task.release().await?;
                Ok(Some(task))
            }
            Some(_) => Ok(None),
            None => bail!("failed to find the task: {id:x}"),
        }
    }
}

impl TaskStore<Task> {
    pub async fn spawn_task(
        &self,
        linker: &mut IpwisLinker,
        module: &Module,
        id: ResourceId,
        ctx: Arc<GuarantorSigned<TaskCtx>>,
    ) -> Result<TaskId> {
        self.spawn_inner(linker, module, id, ctx, |task| task).await
    }

    pub async fn release(&mut self) {
        // order: Task Seed -> SubTasks
        self.seed.release();
        for task in self.map.get_mut().values() {
            task.handler.abort();
        }
    }
}

#[derive(Debug)]
struct TaskIdSeed(AtomicU32);

impl Default for TaskIdSeed {
    fn default() -> Self {
        Self((Self::DROPPED + 1).into())
    }
}

impl TaskIdSeed {
    const DROPPED: ExternDataRef = 0;

    pub fn generate(&self) -> Result<TaskId> {
        let id = self.0.fetch_add(1, Ordering::SeqCst);
        if id == 0 {
            bail!("released task")
        } else {
            Ok(TaskId(self.0.fetch_add(1, Ordering::SeqCst)))
        }
    }

    fn release(&mut self) {
        *self.0.get_mut() = Self::DROPPED
    }
}

pub type Entry = ::ipwis_kernel_common::task::Entry<TaskResult>;
pub type Task = ::ipwis_kernel_common::task::Task<TaskResult>;
type TaskResult = Result<IpwisStore, Trap>;
