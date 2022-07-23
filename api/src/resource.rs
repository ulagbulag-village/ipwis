use ipis::{async_trait::async_trait, core::anyhow::Result, env::Infer, tokio::sync::Mutex};
use ipwis_common::kernel::{
    resource::{ResourceId, ResourceManager},
    task::TaskConstraints,
};
use ipwis_kernel::resource::ResourceStore;

#[derive(Default)]
pub struct DummyResourceManager {
    store: Mutex<ResourceStore<()>>,
}

#[async_trait]
impl<'a> Infer<'a> for DummyResourceManager {
    type GenesisArgs = ();
    type GenesisResult = Self;

    async fn try_infer() -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Default::default())
    }

    async fn genesis(
        (): <Self as Infer<'a>>::GenesisArgs,
    ) -> Result<<Self as Infer<'a>>::GenesisResult> {
        <Self as Infer<'a>>::try_infer().await
    }
}

#[async_trait]
impl ResourceManager for DummyResourceManager {
    async fn alloc(&self, _constraints: &TaskConstraints) -> Result<Option<ResourceId>> {
        self.store.lock().await.insert(|_| Ok(())).map(Some)
    }
}
