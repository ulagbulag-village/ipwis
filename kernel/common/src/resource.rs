use bytecheck::CheckBytes;
use ipis::core::signed::IsSigned;
use rkyv::{Archive, Deserialize, Serialize};

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

pub type ResourceIdInner = u64;
