use std::sync::Arc;

use ipiis_api::{client::IpiisClient, common::Ipiis};
use ipis::{
    core::anyhow::{bail, Result},
    env::Infer,
    tokio,
};
use ipwis_modules_task_api::task_manager::TaskManager;
use ipwis_modules_task_api_wasi::task_manager::IpwisTaskManager;
use ipwis_modules_task_common::task::Task;

#[tokio::main]
async fn main() -> Result<()> {
    // create an IPIIS account
    let client = IpiisClient::infer().await;

    // prepare a task manager
    let manager = Arc::new(IpwisTaskManager::try_new().await?);

    // register some interrupt modules
    manager
        .interrupt_manager
        .put(::ipwis_modules_stream_api::StreamModule::default())
        .await?;

    // prepare a program
    const BINARY: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/output.wasm"));

    // create a task and sign
    let task = Task::new_sandbox();
    let task = client.sign(client.account_me().account_ref(), task)?;
    let task = client.sign_as_guarantor(task)?;

    // spawn a task
    let instance = manager.spawn_raw(task, BINARY).await?;

    // wait for result
    match instance.await? {
        Ok(outputs) => {
            println!("{:?}", outputs);
            Ok(())
        }
        Err(errors) => bail!("{}", errors.msg),
    }
}