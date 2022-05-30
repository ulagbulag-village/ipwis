use std::collections::BTreeMap;

use ipwis_kernel_common::interrupt::InterruptId;

#[derive(Debug, Default)]
pub struct InterruptHandlerStore {
    map: BTreeMap<InterruptId, InterruptHandler>,
}

#[derive(Debug)]
pub struct InterruptHandler {
    pub module: String,
    pub func: String,
}
