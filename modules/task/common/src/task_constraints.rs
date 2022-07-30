use bytecheck::CheckBytes;
use ipis::{
    class::{metadata::ClassMetadata, Class},
    core::signed::IsSigned,
    object::{data::ObjectData, IntoObjectData},
};
use rkyv::{Archive, Deserialize, Serialize};

use crate::task_resource_constraints::TaskResourceConstraints;

#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct TaskConstraints {
    pub inputs: ObjectData,
    pub outputs: ClassMetadata,
    pub resources: TaskResourceConstraints,
}

impl TaskConstraints {
    pub fn new_sandbox() -> Self {
        Self {
            inputs: ().__into_object_data(),
            outputs: <() as Class>::__class_metadata(),
            resources: TaskResourceConstraints::UNLIMITED,
        }
    }
}

impl IsSigned for TaskConstraints {}
