use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use std::{collections::HashMap, sync::Arc};

use bytecheck::CheckBytes;
use ipis::{
    class::{metadata::ClassMetadata, Class},
    core::{
        account::{GuaranteeSigned, GuarantorSigned},
        anyhow::Result,
        signed::IsSigned,
        value::{chrono::DateTime, text::Text},
    },
    object::{data::ObjectData, IntoObjectData},
    path::Path,
    tokio::{self, sync::Mutex},
};
use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    data::{ExternData, ExternDataRef},
    protection::ProtectionMode,
    resource::{ResourceConstraints, ResourceId},
};

pub struct Entry<R> {
    pub ctx: Box<GuarantorSigned<TaskCtx>>,
    pub task: Task<R>,
}

impl<R> Future for Entry<R> {
    type Output = <Task<R> as Future>::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.task).poll(cx)
    }
}

pub struct Task<R> {
    pub ctx: TaskPtr,
    pub state: Arc<Mutex<TaskState>>,
    pub handler: tokio::task::JoinHandle<R>,
}

impl<R> Future for Task<R> {
    type Output = Result<R, tokio::task::JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.handler).poll(cx)
    }
}

#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(bound(serialize = "
    __S: ::rkyv::ser::ScratchSpace + ::rkyv::ser::Serializer,
"))]
#[archive_attr(derive(Debug, PartialEq))]
pub struct TaskCtx {
    pub constraints: TaskConstraints,
    pub program: Option<GuaranteeSigned<Path>>,
    #[omit_bounds]
    pub reserved: HashMap<String, TaskCtx>,
    #[omit_bounds]
    pub children: HashMap<String, TaskCtx>,
    #[omit_bounds]
    pub exceptions: Vec<TaskCtx>,
}

impl TaskCtx {
    pub fn new_sandbox() -> Self {
        Self {
            constraints: TaskConstraints {
                inputs: ().__into_object_data(),
                outputs: <() as Class>::__class_metadata(),
                resources: ResourceConstraints::UNLIMITED,
            },
            program: None,
            reserved: Default::default(),
            children: Default::default(),
            exceptions: Default::default(),
        }
    }
}

impl IsSigned for TaskCtx {}

impl<__C> CheckBytes<__C> for ArchivedTaskCtx
where
    __C: ::rkyv::validation::ArchiveContext,
    <__C as ::rkyv::Fallible>::Error: ::std::error::Error,
{
    type Error = ::bytecheck::StructCheckError;

    unsafe fn check_bytes<'__bytecheck>(
        value: *const Self,
        context: &mut __C,
    ) -> Result<&'__bytecheck Self, Self::Error> {
        CheckBytes::<__C>::check_bytes(::core::ptr::addr_of!((*value).constraints), context)
            .map_err(|e| ::bytecheck::StructCheckError {
                field_name: stringify!(constraints),
                inner: ::bytecheck::ErrorBox::new(e),
            })?;
        CheckBytes::<__C>::check_bytes(::core::ptr::addr_of!((*value).program), context).map_err(
            |e| ::bytecheck::StructCheckError {
                field_name: stringify!(program),
                inner: ::bytecheck::ErrorBox::new(e),
            },
        )?;
        CheckBytes::<__C>::check_bytes(::core::ptr::addr_of!((*value).reserved), context).map_err(
            |e| ::bytecheck::StructCheckError {
                field_name: stringify!(reserved),
                inner: ::bytecheck::ErrorBox::new(e),
            },
        )?;
        CheckBytes::<__C>::check_bytes(::core::ptr::addr_of!((*value).children), context).map_err(
            |e| ::bytecheck::StructCheckError {
                field_name: stringify!(children),
                inner: ::bytecheck::ErrorBox::new(e),
            },
        )?;
        CheckBytes::<__C>::check_bytes(::core::ptr::addr_of!((*value).exceptions), context)
            .map_err(|e| ::bytecheck::StructCheckError {
                field_name: stringify!(exceptions),
                inner: ::bytecheck::ErrorBox::new(e),
            })?;
        Ok(&*value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TaskPtr(*const TaskCtx);

/// # Safety
///
/// It's thread-safe as the task is read-only and is owned by Entry.
unsafe impl Send for TaskPtr {}
unsafe impl Sync for TaskPtr {}

impl ::core::ops::Deref for TaskPtr {
    type Target = TaskCtx;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl TaskPtr {
    pub const fn new(ctx: &TaskCtx) -> Self {
        Self(ctx)
    }
}

#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct TaskConstraints {
    pub inputs: ObjectData,
    pub outputs: ClassMetadata,
    pub resources: ResourceConstraints,
}

impl IsSigned for TaskConstraints {}

#[derive(Copy, Clone, Debug)]
pub struct TaskState {
    pub resource_id: ResourceId,
    pub task_id: TaskId,
    pub inputs: ExternData,
    pub outputs: ExternData,
    pub errors: ExternData,
    pub created_date: DateTime,
    pub protection_mode: ProtectionMode,
    pub is_working: bool,
}

#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub enum TaskPoll {
    Pending,
    Ready(Box<ObjectData>),
    Trap(Text),
}

impl IsSigned for TaskPoll {}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Archive, Serialize, Deserialize,
)]
#[archive(compare(PartialEq, PartialOrd))]
#[archive_attr(derive(CheckBytes, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[repr(C)]
pub struct TaskId(pub ExternDataRef);

impl IsSigned for TaskId {}

impl ::core::fmt::LowerHex for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ::core::fmt::LowerHex::fmt(&self.0, f)
    }
}
