use ipwis_kernel_api::memory::IpwisMemoryInner;

use crate::ctx::IpwisCaller;

pub type IpwisMemory<'a> = IpwisMemoryInner<&'a mut IpwisCaller<'a>>;
