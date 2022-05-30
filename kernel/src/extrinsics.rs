use ipis::core::anyhow::Result;
use ipwis_kernel_common::{data::ExternDataRef, interrupt::InterruptIdInner};

use crate::ctx::{IpwisCaller, IpwisLinker};

pub fn register(linker: &mut IpwisLinker) -> Result<()> {
    linker.func_wrap("ipwis_kernel", "syscall", syscall)?;
    Ok(())
}

fn syscall(
    caller: IpwisCaller,
    id: InterruptIdInner,
    inputs: ExternDataRef,
    outputs: ExternDataRef,
    errors: ExternDataRef,
) {
    todo!()
}
