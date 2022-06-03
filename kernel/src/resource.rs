use core::sync::atomic::{AtomicU32, Ordering};
use std::collections::HashMap;

use ipis::core::anyhow::Result;
use ipwis_kernel_common::resource::ResourceId;

#[derive(Debug)]
pub struct ResourceStore<R> {
    seed: ResourceIdSeed,
    pub map: HashMap<ResourceId, R>,
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

impl<R> Default for ResourceStore<R> {
    fn default() -> Self {
        Self {
            seed: Default::default(),
            map: Default::default(),
        }
    }
}

#[derive(Debug)]
struct ResourceIdSeed(AtomicU32);

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
