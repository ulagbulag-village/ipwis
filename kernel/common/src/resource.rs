use std::{collections::HashMap, sync::atomic::{AtomicU32, Ordering}};

use bytecheck::CheckBytes;
use ipis::{
    async_trait::async_trait,
    core::{anyhow::Result, signed::IsSigned, value::chrono::DateTime},
};
use rkyv::{Archive, Deserialize, Serialize};

use crate::{data::ExternDataRef, task::TaskConstraints};

#[async_trait]
pub trait ResourceManager {
    async fn alloc(&self, constraints: &TaskConstraints) -> Result<Option<ResourceId>>;
}

#[derive(Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct ResourceConstraints {
    pub due_date: DateTime,
}

impl ResourceConstraints {
    pub const UNLIMITED: Self = ResourceConstraints {
        due_date: DateTime::MAX_DATETIME,
    };
}


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

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[repr(C)]
pub struct ResourceId(pub ExternDataRef);

impl IsSigned for ResourceId {}

impl ::core::fmt::LowerHex for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ::core::fmt::LowerHex::fmt(&self.0, f)
    }
}
