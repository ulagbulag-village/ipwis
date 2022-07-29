use ipis::{
    core::{account::GuarantorSigned, anyhow::Result},
    env::Infer,
    tokio,
};
use ipwis_kernel_common::{
    resource::ResourceManager,
    task::{TaskCtx, TaskId},
};

use crate::{ctx::IpwisCtx, scheduler::Scheduler};

pub struct Kernel<R> {
    resource_manager: R,
    scheduler: Scheduler,
}

impl<R> Kernel<R>
where
    R: ResourceManager,
{
    pub async fn boot() -> Result<Self>
    where
        R: for<'a> Infer<'a> + Send,
    {
        Ok(Self {
            resource_manager: R::infer().await,
            scheduler: Scheduler::new().await?,
        })
    }

    pub async fn spawn(
        &self,
        ctx: GuarantorSigned<TaskCtx>,
        program: &[u8],
    ) -> Result<Option<TaskId>> {
        match self.resource_manager.alloc(&ctx.constraints).await? {
            Some(id) => self.scheduler.spawn(id, ctx, program).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn spawn_local(
        &self,
        ctx: GuarantorSigned<TaskCtx>,
        local_path: impl AsRef<::std::path::Path>,
    ) -> Result<Option<TaskId>> {
        let program = tokio::fs::read(local_path).await?;
        self.spawn(ctx, &program).await
    }

    pub async fn poll(&self, id: TaskId) -> Result<Option<IpwisCtx>> {
        self.scheduler.poll(id).await
    }

    pub async fn wait(&self, id: TaskId) -> Result<IpwisCtx> {
        loop {
            match self.poll(id).await? {
                Some(ctx) => break Ok(ctx),
                None => tokio::task::yield_now().await,
            }
        }
    }
}
