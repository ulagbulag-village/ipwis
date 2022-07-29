use ipiis_api::{client::IpiisClient, common::Ipiis};
use ipis::{core::anyhow::Result, env::Infer, tokio};
use ipwis_api::resource::DummyResourceManager;
use ipwis_common::kernel::task::TaskCtx;
use ipwis_kernel::kernel::Kernel;

#[tokio::main]
async fn main() -> Result<()> {
    // create an IPIIS account
    let client = IpiisClient::infer().await;

    // boot a kernel
    let kernel = Kernel::<DummyResourceManager>::boot().await?;

    // prepare a program
    let my_program = include_bytes!("../../target/wasm32-wasi/debug/ipwis_demo.wasi.wasm");

    // create a task and sign
    let ctx = TaskCtx::new_sandbox();
    let ctx = client.sign(client.account_me().account_ref(), ctx)?;
    let ctx = client.sign_as_guarantor(ctx)?;

    let id = kernel.spawn(ctx, my_program).await?.unwrap();
    let ctx = kernel.wait(id).await?;

    Ok(())
}
