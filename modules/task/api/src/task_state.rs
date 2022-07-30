use std::sync::Arc;

use ipis::core::{account::GuarantorSigned, value::chrono::DateTime};
use ipwis_modules_task_common::task::Task;

use crate::task_manager::TaskManager;

#[derive(Clone, Debug)]
pub struct TaskState<T>
where
    T: TaskManager,
{
    pub manager: Arc<T>,
    pub task: GuarantorSigned<Task>,
    pub created_date: DateTime,
    pub is_working: bool,
}
