use ipis::{core::anyhow::Result, log::warn, rkyv::AlignedVec};
use ipwis_kernel_common::{
    data::ExternDataRef,
    extrinsics::{SYSCALL_ERR_FATAL, SYSCALL_ERR_NORMAL, SYSCALL_OK},
    interrupt::InterruptId,
    memory::Memory,
    modules::{FUNC_NAME_SYSCALL, MODULE_NAME_COMMON},
};

use crate::{
    ctx::{IpwisCaller, IpwisLinker},
    memory::IpwisMemory,
};

pub fn register(linker: &mut IpwisLinker) -> Result<()> {
    linker.func_wrap4_async(
        MODULE_NAME_COMMON,
        FUNC_NAME_SYSCALL,
        |caller, handler, inputs, outputs, errors| {
            Box::new(syscall(caller, handler, inputs, outputs, errors))
        },
    )?;
    Ok(())
}

async fn syscall(
    mut caller: IpwisCaller<'_>,
    handler: ExternDataRef,
    inputs: ExternDataRef,
    outputs: ExternDataRef,
    errors: ExternDataRef,
) -> ExternDataRef {
    let mut memory = unsafe {
        // allow interior mutability
        match IpwisMemory::with_caller(::core::mem::transmute::<_, &mut IpwisCaller>(&mut caller)) {
            Ok(memory) => memory,
            Err(error) => {
                warn!("{}", error);
                return SYSCALL_ERR_FATAL;
            }
        }
    };

    async unsafe fn try_handle<'a>(
        caller: &mut IpwisCaller<'a>,
        memory: &mut IpwisMemory<'static>,
        handler: ExternDataRef,
        inputs: ExternDataRef,
    ) -> Result<AlignedVec> {
        let handler = {
            let data = ::core::mem::transmute(memory.load(handler)?); // ignore `memory` lifetime
            InterruptId(::core::str::from_utf8(data)?)
        };
        let inputs: &[u8] = {
            ::core::mem::transmute(memory.load(inputs)?) // ignore `memory` lifetime
        };

        caller
            .data_mut()
            .interrupt_handlers
            .handle_raw(memory, handler, inputs)
            .await
    }

    unsafe {
        match try_handle(&mut caller, &mut memory, handler, inputs).await {
            Ok(buf) => match memory.dump_to(&buf, outputs).await {
                Ok(()) => SYSCALL_OK,
                Err(error) => {
                    warn!("{}", error);
                    SYSCALL_ERR_FATAL
                }
            },
            Err(error) => match memory.dump_error_to(error, errors).await {
                Ok(()) => SYSCALL_ERR_NORMAL,
                Err(error) => {
                    warn!("{}", error);
                    SYSCALL_ERR_FATAL
                }
            },
        }
    }
}
