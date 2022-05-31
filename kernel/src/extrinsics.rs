use ipis::{core::anyhow::Result, rkyv::AlignedVec};
use ipwis_kernel_common::{data::ExternDataRef, interrupt::InterruptId};

use crate::{
    ctx::{IpwisCaller, IpwisLinker},
    memory::IpwisMemory,
};

pub fn register(linker: &mut IpwisLinker) -> Result<()> {
    linker.func_wrap("ipwis_kernel", "syscall", syscall)?;
    Ok(())
}

fn syscall(
    mut caller: IpwisCaller,
    handler: ExternDataRef,
    inputs: ExternDataRef,
    outputs: ExternDataRef,
    errors: ExternDataRef,
) {
    let mut memory = unsafe {
        // allow interior mutability
        IpwisMemory::from_caller(::core::mem::transmute::<_, &mut IpwisCaller>(&mut caller))
    };

    unsafe fn try_handle(
        caller: &mut IpwisCaller,
        memory: &mut IpwisMemory<&mut IpwisCaller>,
        handler: ExternDataRef,
        inputs: ExternDataRef,
    ) -> Result<AlignedVec> {
        let handler = {
            let data = ::core::mem::transmute(memory.load(handler)); // ignore `memory` lifetime
            InterruptId(::core::str::from_utf8(data)?)
        };
        let inputs: &[u8] = {
            ::core::mem::transmute(memory.load(inputs)) // ignore `memory` lifetime
        };

        caller
            .data_mut()
            .store
            .handle_interrupt_raw(handler, inputs)
    }

    unsafe {
        match try_handle(&mut caller, &mut memory, handler, inputs) {
            Ok(buf) => memory.copy(&buf, outputs),
            Err(error) => memory.copy_error(error, errors),
        }
    }
}
