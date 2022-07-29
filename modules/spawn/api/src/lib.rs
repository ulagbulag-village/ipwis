#![allow(clippy::missing_safety_doc)]

// pub extern crate ipwis_modules_spawn_common as common;

use core::pin::Pin;

use ipis::{
    async_trait::async_trait,
    core::{
        anyhow::{anyhow, Result},
        signed::IsSigned,
    },
    pin::PinnedInner,
    rkyv::AlignedVec,
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};
use ipwis_kernel_common::{
        interrupt::{InterruptHandler, InterruptId, InterruptModule},
        memory::Memory,
        resource::{ResourceId,ResourceStore},
};
use ipwis_modules_spawn_common::{io, ExternReader, ExternWriter};

#[derive(Copy, Clone, Debug, Default)]
pub struct StreamModule;

#[async_trait]
impl<M> InterruptModule<M> for StreamHandler
where
    M: Memory,
{
    fn id(&self) -> InterruptId {
        io::OpCode::ID
    }

    async fn spawn_handler(&self) -> Result<Box<dyn InterruptHandler<M>>> {
        Ok(Box::new(StreamHandler::default()))
    }
}

#[derive(Default)]
pub struct StreamHandler {
    readers: ResourceStore<Pin<Box<dyn AsyncRead + Send + Sync>>>,
    writers: ResourceStore<Pin<Box<dyn AsyncWrite + Send + Sync>>>,
}

#[async_trait]
impl<M> InterruptHandler<M> for StreamHandler
where
    M: Memory,
{
    async unsafe fn handle_raw(&mut self, memory: &mut M, inputs: &[u8]) -> Result<AlignedVec> {
        match PinnedInner::deserialize_owned(inputs)? {
            io::OpCode::ReaderNext(req) => self
                .handle_reader_next(memory, req)
                .await?
                .to_bytes()
                .map_err(Into::into),
            io::OpCode::WriterNext(req) => self
                .handle_writer_next(memory, req)
                .await?
                .to_bytes()
                .map_err(Into::into),
            io::OpCode::WriterFlush(req) => self
                .handle_writer_flush(req)
                .await?
                .to_bytes()
                .map_err(Into::into),
            io::OpCode::WriterShutdown(req) => self
                .handle_writer_shutdown(req)
                .await?
                .to_bytes()
                .map_err(Into::into),
        }
    }

    async fn release(&mut self) -> Result<()> {
        self.readers.map.clear();
        for writer in self.writers.map.values_mut() {
            writer.shutdown().await?;
        }
        self.writers.map.clear();
        Ok(())
    }
}

impl StreamHandler {
    pub fn handle_reader_new(
        &mut self,
        reader: impl AsyncRead + Send + Sync + 'static,
        len: usize,
    ) -> Result<ExternReader> {
        let id = self.readers.insert(|_| Ok(Box::pin(reader)))?;
        let len = len.try_into()?;

        Ok(ExternReader::new(id, len))
    }

    async unsafe fn handle_reader_next<M>(
        &mut self,
        memory: &mut M,
        req: io::request::ReaderNext,
    ) -> Result<io::response::ReaderNext>
    where
        M: Memory,
    {
        let reader = self.get_reader(&req.id)?;
        let mut buf = memory.load_mut_raw(req.buf)?;

        Ok(io::response::ReaderNext {
            len: reader.read_buf(&mut buf).await?.try_into()?,
        })
    }

    fn get_reader(
        &mut self,
        id: &ResourceId,
    ) -> Result<&mut Pin<Box<dyn AsyncRead + Send + Sync>>> {
        self.readers
            .map
            .get_mut(id)
            .ok_or_else(|| anyhow!("failed to find the ExternReader: {:x}", id))
    }
}

impl StreamHandler {
    pub fn handle_writer_new(
        &mut self,
        writer: impl AsyncWrite + Send + Sync + 'static,
    ) -> Result<ExternWriter> {
        let id = self.writers.insert(|_| Ok(Box::pin(writer)))?;

        Ok(ExternWriter::new(id))
    }

    async unsafe fn handle_writer_next<M>(
        &mut self,
        memory: &mut M,
        req: io::request::WriterNext,
    ) -> Result<io::response::WriterNext>
    where
        M: Memory,
    {
        let writer = self.get_writer(&req.id)?;
        let mut buf = memory.load_raw(req.buf)?;

        Ok(io::response::WriterNext {
            len: writer.write_buf(&mut buf).await?.try_into()?,
        })
    }

    async fn handle_writer_flush(
        &mut self,
        req: io::request::WriterFlush,
    ) -> Result<io::response::WriterFlush> {
        let writer = self.get_writer(&req.id)?;

        writer
            .flush()
            .await
            .map(|()| io::response::WriterFlush {})
            .map_err(Into::into)
    }

    async fn handle_writer_shutdown(
        &mut self,
        req: io::request::WriterShutdown,
    ) -> Result<io::response::WriterShutdown> {
        let writer = self.get_writer(&req.id)?;

        writer
            .shutdown()
            .await
            .map(|()| io::response::WriterShutdown {})
            .map_err(Into::into)
    }

    fn get_writer(
        &mut self,
        id: &ResourceId,
    ) -> Result<&mut Pin<Box<dyn AsyncWrite + Send + Sync>>> {
        self.writers
            .map
            .get_mut(id)
            .ok_or_else(|| anyhow!("failed to find the ExternWriter: {:x}", id))
    }
}
