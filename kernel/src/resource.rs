use ipis::core::anyhow::Result;
use ipwis_kernel_common::task::TaskCtx;

#[derive(Default)]
pub struct ResourceManager {}

impl ResourceManager {
    pub async fn is_affordable(&self, ctx: &TaskCtx) -> Result<bool> {
        Ok(true)
    }
}
