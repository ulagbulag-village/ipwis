use core::sync::atomic::{AtomicU64, Ordering};
use std::collections::BTreeMap;

use ipis::core::anyhow::Result;
use ipwis_kernel_common::{resource::ResourceId, task::TaskCtx};

#[derive(Default)]
pub(crate) struct ResourceManager {}

impl ResourceManager {
    pub async fn is_affordable(&self, ctx: &TaskCtx) -> Result<bool> {
        Ok(true)
    }
}

#[derive(Debug, Default)]
pub struct ResourceStore<R> {
    seed: ResourceIdSeed,
    map: BTreeMap<ResourceId, R>,
}

impl<R> ResourceStore<R> {
    pub fn insert<F>(&mut self, f: F) -> Result<ResourceId>
    where
        F: FnOnce(ResourceId) -> Result<R>,
    {
        let id = self.seed.generate();

        self.map.insert(id, f(id)?);

        Ok(id)
    }
}

#[derive(Debug)]
struct ResourceIdSeed(AtomicU64);

impl Default for ResourceIdSeed {
    fn default() -> Self {
        Self(1.into())
    }
}

impl ResourceIdSeed {
    pub fn generate(&self) -> ResourceId {
        ResourceId(self.0.fetch_add(1, Ordering::SeqCst))
    }
}
