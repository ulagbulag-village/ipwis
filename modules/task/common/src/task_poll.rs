use bytecheck::CheckBytes;
use ipis::{
    core::{signed::IsSigned, value::text::Text},
    object::data::ObjectData,
};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Archive, Serialize, Deserialize)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(CheckBytes, Debug, PartialEq))]
pub enum TaskPoll {
    Pending,
    Ready(Box<ObjectData>),
    Trap(Text),
}

impl IsSigned for TaskPoll {}
