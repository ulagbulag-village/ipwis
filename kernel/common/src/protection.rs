use bytecheck::CheckBytes;
use ipis::core::signed::IsSigned;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[repr(C)]
pub enum ProtectionMode {
    Worker,
    Entry,
    Interrupt,
    Kernel,
}

impl IsSigned for ProtectionMode {}
