use ipis::{async_trait::async_trait, core::anyhow::Result};

#[async_trait]
pub trait ResourceManager {
    type Key;
    type Value;

    async fn get(
        &self,
        key: &<Self as ResourceManager>::Key,
    ) -> Result<Option<<Self as ResourceManager>::Value>>;

    async fn put(
        &self,
        value: <Self as ResourceManager>::Value,
    ) -> Result<<Self as ResourceManager>::Key>;
}
