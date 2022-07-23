#![allow(clippy::missing_safety_doc)]

pub mod data;
pub mod extrinsics;
pub mod interrupt;
pub mod memory;
pub mod protection;
pub mod resource;
pub mod task;

pub mod modules {
    pub const MODULE_NAME_API: &str = "__ipwis_kernel_api";
    pub const MODULE_NAME_CUSTOM: &str = "__ipwis_custom";
}
