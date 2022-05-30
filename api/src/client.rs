use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned},
        anyhow::Result,
    },
    env::Infer,
    futures::TryFutureExt,
};
use ipwis_common::Ipwis;
use ipwis_kernel::{
    common::task::{TaskCtx, TaskId, TaskPoll},
    kernel::Kernel,
};

pub type IpwisClient = IpwisClientInner<::ipiis_api::client::IpiisClient>;

pub struct IpwisClientInner<IpiisClient> {
    pub ipiis: IpiisClient,
    kernel: Kernel,
}

impl<IpiisClient> AsRef<::ipiis_api::client::IpiisClient> for IpwisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipiis_api::client::IpiisClient>,
{
    fn as_ref(&self) -> &::ipiis_api::client::IpiisClient {
        self.ipiis.as_ref()
    }
}

impl<IpiisClient> AsRef<::ipiis_api::server::IpiisServer> for IpwisClientInner<IpiisClient>
where
    IpiisClient: AsRef<::ipiis_api::server::IpiisServer>,
{
    fn as_ref(&self) -> &::ipiis_api::server::IpiisServer {
        self.ipiis.as_ref()
    }
}

#[async_trait]
impl<'a, IpiisClient> Infer<'a> for IpwisClientInner<IpiisClient>
where
    Self: Send,
    IpiisClient: Infer<'a, GenesisResult = IpiisClient> + Send,
    <IpiisClient as Infer<'a>>::GenesisArgs: Sized,
{
    type GenesisArgs = <IpiisClient as Infer<'a>>::GenesisArgs;
    type GenesisResult = Self;

    async fn try_infer() -> Result<Self> {
        IpiisClient::try_infer()
            .and_then(Self::with_ipiis_client)
            .await
    }

    async fn genesis(
        args: <Self as Infer<'a>>::GenesisArgs,
    ) -> Result<<Self as Infer<'a>>::GenesisResult> {
        IpiisClient::genesis(args)
            .and_then(Self::with_ipiis_client)
            .await
    }
}

impl<IpiisClient> IpwisClientInner<IpiisClient> {
    pub async fn with_ipiis_client(ipiis: IpiisClient) -> Result<Self> {
        Ok(Self {
            ipiis,
            kernel: Kernel::boot().await?,
        })
    }
}

#[async_trait]
impl<IpiisClient> Ipwis for IpwisClientInner<IpiisClient>
where
    IpiisClient: Send + Sync,
{
    async fn task_spawn(
        &self,
        ctx: GuaranteeSigned<TaskCtx>,
    ) -> Result<Option<GuaranteeSigned<TaskId>>> {
        let guarantee = ctx.guarantee.account;

        self.task_spawn_unchecked(Some(guarantee), ctx.data.data)
            .await
    }

    async fn task_spawn_unchecked(
        &self,
        guarantee: Option<AccountRef>,
        ctx: TaskCtx,
    ) -> Result<Option<GuaranteeSigned<TaskId>>> {
        todo!()
    }

    async fn task_poll(&self, id: GuarantorSigned<TaskId>) -> Result<GuaranteeSigned<TaskPoll>> {
        todo!()
    }
}
