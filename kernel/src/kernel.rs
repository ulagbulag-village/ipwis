use ipis::{
    core::{account::GuarantorSigned, anyhow::Result},
    env::Infer,
    tokio,
};
use ipwis_kernel_common::{
    resource::ResourceManager,
    task::{TaskCtx, TaskId},
};

use crate::{ctx::IpwisCtx, scheduler::Scheduler};

pub struct Kernel<R> {
    resource_manager: R,
    scheduler: Scheduler,
}

impl<R> Kernel<R>
where
    R: ResourceManager,
{
    pub async fn boot() -> Result<Self>
    where
        R: for<'a> Infer<'a> + Send,
    {
        Ok(Self {
            resource_manager: R::infer().await,
            scheduler: Scheduler::new().await?,
        })
    }

    pub async fn spawn(
        &self,
        ctx: GuarantorSigned<TaskCtx>,
        program: &[u8],
    ) -> Result<Option<TaskId>> {
        match self.resource_manager.alloc(&ctx.constraints).await? {
            Some(id) => self.scheduler.spawn(id, ctx, program).await.map(Some),
            None => Ok(None),
        }
    }

    pub async fn spawn_local(
        &self,
        ctx: GuarantorSigned<TaskCtx>,
        local_path: impl AsRef<::std::path::Path>,
    ) -> Result<Option<TaskId>> {
        let program = tokio::fs::read(local_path).await?;
        self.spawn(ctx, &program).await
    }

    pub async fn poll(&self, id: TaskId) -> Result<Option<IpwisCtx>> {
        self.scheduler.poll(id).await
    }

    // pub async fn start(self) -> Result<()> {
    //     // define the WASI functions globally on the `Config`.
    //     let engine = Engine::new(Config::new().async_support(true))?;
    //     let mut linker = IpwisLinker::new(&engine);
    //     wasmtime_wasi::add_to_linker(&mut linker, |s| &mut s.wasi)?;

    //     // linker.func_wrap("ipwis_module_stream", "next", ipiis_reader__next)?;

    //     // Instantiate our module with the imports we've created, and run it.
    //     let module =
    //         Module::from_file(&engine, "../target/wasm32-wasi/debug/ipwis_demo.wasi.wasm")?;

    //     // Create a WASI context and put it in a Store; all instances in the store
    //     // share this context. `WasiCtxBuilder` provides a number of ways to
    //     // configure what the target program will have access to.
    //     let wasi = WasiCtxBuilder::new()
    //         .inherit_stdio()
    //         .inherit_args()?
    //         .build();
    //     let ctx = IpwisCtx { wasi };
    //     let mut store = Store::new(&engine, ctx);
    //     // linker.module_async(&mut store, "", &module).await?;

    //     // linker.func_wrap("", "foo", foo)?;
    //     // let func = match linker.get(&mut store, "", "foo") {
    //     //     Some(external) => match external {
    //     //         Extern::Func(func) => func.typed::<(u32, u32), u32, _>(&store).unwrap(),
    //     //         _ => panic!(),
    //     //     },
    //     //     _ => panic!(),
    //     // };
    //     // let ret = func.call_async(&mut store, (13, 29)).await?;
    //     // dbg!(ret);

    //     // let func = match linker.get(&mut store, "", "goo") {
    //     //     Some(external) => match external {
    //     //         Extern::Func(func) => func.typed::<(u32, u32), u32, _>(&store).unwrap(),
    //     //         _ => panic!(),
    //     //    }
    //     //    _ => panic!(),
    //     // };
    //     // let ret = func.call_async(&mut store, (13, 29)).await?;
    //     // dbg!(ret);

    //     let func = match linker.get(&mut store, "", "read_sum_sync") {
    //         Some(external) => match external {
    //             Extern::Func(func) => func.typed::<(u64, u32), u32, _>(&store).unwrap(),
    //             _ => panic!(),
    //         },
    //         _ => panic!(),
    //     };
    //     let ret = func.call_async(&mut store, (13, 2)).await?;
    //     dbg!(ret);

    //     Ok(())
    // }
}
