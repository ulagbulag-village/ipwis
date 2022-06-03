pub extern crate ipwis_kernel_common as kernel;

use ipiis_common::{define_io, external_call, Ipiis, ServerResult};
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned},
        anyhow::Result,
    },
};
use ipwis_kernel_common::task::{TaskCtx, TaskId, TaskPoll};

#[async_trait]
pub trait Ipwis {
    async fn task_spawn(
        &self,
        ctx: GuaranteeSigned<TaskCtx>,
    ) -> Result<Option<GuaranteeSigned<TaskId>>>;

    async fn task_spawn_unchecked(
        &self,
        guarantee: Option<AccountRef>,
        ctx: TaskCtx,
    ) -> Result<Option<GuaranteeSigned<TaskId>>>;

    async fn task_poll(&self, id: GuarantorSigned<TaskId>) -> Result<GuaranteeSigned<TaskPoll>>;
}

#[async_trait]
impl<IpiisClient> Ipwis for IpiisClient
where
    IpiisClient: Ipiis + Send + Sync,
{
    async fn task_spawn(
        &self,
        ctx: GuaranteeSigned<TaskCtx>,
    ) -> Result<Option<GuaranteeSigned<TaskId>>> {
        // next target
        let target = ctx.guarantor;

        // external call
        let (id,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => Spawn,
            sign: ctx,
            inputs: { },
            outputs: { id, },
        );

        // unpack response
        Ok(id)
    }

    async fn task_spawn_unchecked(
        &self,
        _guarantee: Option<AccountRef>,
        ctx: TaskCtx,
    ) -> Result<Option<GuaranteeSigned<TaskId>>> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // sign data
        let ctx = self.sign(target, ctx.clone())?;

        // external call
        self.task_spawn(ctx).await
    }

    async fn task_poll(&self, id: GuarantorSigned<TaskId>) -> Result<GuaranteeSigned<TaskPoll>> {
        // next target
        let target = self.get_account_primary(KIND.as_ref()).await?;

        // external call
        let (poll,) = external_call!(
            client: self,
            target: KIND.as_ref() => &target,
            request: crate::io => Poll,
            sign: self.sign(target, ())?,
            inputs: {
                id: id,
            },
            outputs: { poll, },
        );

        // unpack response
        Ok(poll)
    }
}

define_io! {
    Spawn {
        inputs: { },
        input_sign: GuaranteeSigned<TaskCtx>,
        outputs: {
            id: Option<GuaranteeSigned<TaskId>>,
        },
        output_sign: GuarantorSigned<TaskCtx>,
        generics: { },
    },
    Poll {
        inputs: {
            id: GuarantorSigned<TaskId>,
        },
        input_sign: GuaranteeSigned<()>,
        outputs: {
            poll: GuaranteeSigned<TaskPoll>,
        },
        output_sign: GuarantorSigned<()>,
        generics: { },
    },
}

::ipis::lazy_static::lazy_static! {
    pub static ref KIND: Option<::ipis::core::value::hash::Hash> = Some(
        ::ipis::core::value::hash::Hash::with_str("__ipis__ipwis__"),
    );
}
