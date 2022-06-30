use bytecheck::CheckBytes;
use ipis::{
    async_trait::async_trait,
    core::{anyhow::Result, signed::IsSigned, value::chrono::DateTime},
};
use rkyv::{Archive, Deserialize, Serialize};

use crate::task::TaskConstraints;

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

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[repr(C)]
pub struct ResourceId(pub ResourceIdInner);

impl IsSigned for ResourceId {}

impl ::core::fmt::LowerHex for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ::core::fmt::LowerHex::fmt(&self.0, f)
    }
}

pub type ResourceIdInner = u32;
