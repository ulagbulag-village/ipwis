use bytecheck::CheckBytes;
use ipis::core::{signed::IsSigned, value::chrono::DateTime};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub struct TaskResourceConstraints {
    pub due_date: DateTime,
}

impl TaskResourceConstraints {
    pub const UNLIMITED: Self = TaskResourceConstraints {
        due_date: DateTime::MAX_DATETIME,
    };
}

impl IsSigned for TaskResourceConstraints {}
