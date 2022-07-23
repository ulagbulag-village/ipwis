use ipiis_api::common::Ipiis;
use ipis::{
    async_trait::async_trait,
    core::{
        account::{AccountRef, GuaranteeSigned, GuarantorSigned},
        anyhow::{bail, Result},
        value::text::Text,
    },
    env::Infer,
    futures::TryFutureExt,
};
use ipsis_common::Ipsis;
use ipwis_common::Ipwis;
use ipwis_kernel::{
    common::task::{TaskCtx, TaskId, TaskPoll},
    kernel::Kernel,
};

pub type IpwisClient = IpwisClientInner<::ipiis_api::client::IpiisClient>;

pub struct IpwisClientInner<IpiisClient> {
    pub ipiis: IpiisClient,
    kernel: Kernel<super::resource::DummyResourceManager>,
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
    IpiisClient: Ipiis + Ipsis + Send + Sync,
{
    async fn task_spawn(
        &self,
        ctx: GuaranteeSigned<TaskCtx>,
    ) -> Result<Option<GuaranteeSigned<TaskId>>> {
        let ctx = self.ipiis.sign_as_guarantor(ctx)?;
        let guarantee = ctx.guarantor.account;

        match &ctx.program {
            Some(program) => {
                let program: Vec<u8> = self.ipiis.get(program).await?;
                match self.kernel.spawn(ctx, &program).await? {
                    Some(id) => self.ipiis.sign(guarantee, id).map(Some),
                    None => Ok(None),
                }
            }
            None => bail!("Empty program"),
        }
    }

    async fn task_spawn_unchecked(
        &self,
        guarantee: Option<AccountRef>,
        ctx: TaskCtx,
    ) -> Result<Option<GuaranteeSigned<TaskId>>> {
        let guarantee = guarantee.unwrap_or_else(|| self.ipiis.account_me().account_ref());

        let ctx = self.ipiis.sign(guarantee, ctx)?;
        self.task_spawn(ctx).await
    }

    async fn task_poll(&self, id: GuarantorSigned<TaskId>) -> Result<GuaranteeSigned<TaskPoll>> {
        let poll = match self.kernel.poll(id.data.data.data).await {
            Ok(Some(ctx)) => TaskPoll::Ready(todo!()),
            Ok(None) => TaskPoll::Pending,
            Err(err) => TaskPoll::Trap(Text::with_en_us(err.to_string())),
        };

        self.ipiis.sign(id.guarantor.account, poll)
    }
}
