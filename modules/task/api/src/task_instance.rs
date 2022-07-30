use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use ipis::tokio::{self, sync::Mutex};

use crate::{task_manager::TaskManager, task_state::TaskState};

pub struct TaskInstance<R, T>
where
    T: TaskManager,
{
    pub state: Arc<Mutex<TaskState<T>>>,
    pub handler: tokio::task::JoinHandle<R>,
}

impl<R, T> Future for TaskInstance<R, T>
where
    T: TaskManager,
{
    type Output = Result<R, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.handler).poll(cx)
    }
}
