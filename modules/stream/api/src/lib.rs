#![allow(clippy::missing_safety_doc)]

// pub extern crate ipwis_modules_stream_common as common;

use ipis::{
    core::{anyhow::Result, signed::IsSigned},
    pin::PinnedInner,
    rkyv::AlignedVec,
};
use ipwis_kernel::{
    common::interrupt::{InterruptHandler, InterruptId},
    resource::ResourceStore,
};
use ipwis_modules_stream_common::io;

#[derive(Default)]
pub struct StreamHandler {
    store: ResourceStore<()>,
}

impl InterruptHandler for StreamHandler {
    fn id(&self) -> InterruptId {
        io::OpCode::ID
    }

    fn handle_raw(&self, inputs: &[u8]) -> Result<AlignedVec> {
        match PinnedInner::deserialize_owned(inputs)? {
            io::OpCode::ReaderNext(req) => {
                self.handle_reader_next(req)?.to_bytes().map_err(Into::into)
            }
            io::OpCode::WriterNext(req) => {
                self.handle_writer_next(req)?.to_bytes().map_err(Into::into)
            }
            io::OpCode::WriterFlush(req) => self
                .handle_writer_flush(req)?
                .to_bytes()
                .map_err(Into::into),
            io::OpCode::WriterShutdown(req) => self
                .handle_writer_shutdown(req)?
                .to_bytes()
                .map_err(Into::into),
        }
    }
}

impl StreamHandler {
    fn handle_reader_next(&self, req: io::request::ReaderNext) -> Result<io::response::ReaderNext> {
        todo!()
    }

    fn handle_writer_next(&self, req: io::request::WriterNext) -> Result<io::response::WriterNext> {
        todo!()
    }

    fn handle_writer_flush(
        &self,
        req: io::request::WriterFlush,
    ) -> Result<io::response::WriterFlush> {
        todo!()
    }

    fn handle_writer_shutdown(
        &self,
        req: io::request::WriterShutdown,
    ) -> Result<io::response::WriterShutdown> {
        todo!()
    }
}
