#![allow(clippy::missing_safety_doc)]

use core::pin::Pin;

use ipis::{
    async_trait::async_trait,
    core::{anyhow::Result, signed::IsSigned},
    pin::PinnedInner,
    rkyv::AlignedVec,
    tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};
use ipwis_modules_core_common::{resource::Resource, resource_store::ResourceStore};
use ipwis_modules_stream_common::{io, ExternReader, ExternWriter};
use ipwis_modules_task_api_wasi::{
    interrupt_handler::InterruptHandler, interrupt_module::InterruptModule, memory::Memory,
};
use ipwis_modules_task_common_wasi::interrupt_id::InterruptId;

#[derive(Copy, Clone, Debug, Default)]
pub struct StreamModule;

#[async_trait]
impl<M> InterruptModule<M> for StreamModule
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
            io::OpCode::ReaderNew(req) => self
                .handle_reader_new(memory, req)
                .await?
                .to_bytes()
                .map_err(Into::into),
            io::OpCode::ReaderNext(req) => self
                .handle_reader_next(memory, req)
                .await?
                .to_bytes()
                .map_err(Into::into),
            io::OpCode::ReaderRelease(req) => self
                .handle_reader_release(req)
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
            io::OpCode::WriterRelease(req) => self
                .handle_writer_release(req)
                .await?
                .to_bytes()
                .map_err(Into::into),
        }
    }
}

#[async_trait]
impl Resource for StreamHandler {
    async fn release(&mut self) -> Result<()> {
        self.readers.release().await?;
        self.writers.release().await?;
        Ok(())
    }
}

impl StreamHandler {
    pub fn new_reader(
        &mut self,
        reader: impl AsyncRead + Send + Sync + 'static,
        len: usize,
    ) -> Result<ExternReader> {
        let id = self.readers.put(Box::pin(reader));
        let len = len.try_into()?;

        Ok(ExternReader::new(id, len))
    }

    async unsafe fn handle_reader_new<M>(
        &mut self,
        memory: &mut M,
        req: io::request::ReaderNew,
    ) -> Result<io::response::ReaderNew>
    where
        M: Memory,
    {
        // safety: the lifetime only depends on the client
        let buf: &[u8] = ::core::mem::transmute(memory.load(req.buf)?);
        let id = self.readers.put(Box::pin(buf));

        Ok(io::response::ReaderNew { id })
    }

    async unsafe fn handle_reader_next<M>(
        &mut self,
        memory: &mut M,
        req: io::request::ReaderNext,
    ) -> Result<io::response::ReaderNext>
    where
        M: Memory,
    {
        let reader = self.readers.get_mut(&req.id)?;
        let mut buf = memory.load_mut(req.buf)?;

        Ok(io::response::ReaderNext {
            len: reader.read_buf(&mut buf).await?.try_into()?,
        })
    }

    async unsafe fn handle_reader_release(
        &mut self,
        req: io::request::ReaderRelease,
    ) -> Result<io::response::ReaderRelease> {
        self.readers
            .release_one(&req.id)
            .await
            .map(|()| io::response::ReaderRelease {})
    }
}

impl StreamHandler {
    pub fn handle_writer_new(
        &mut self,
        writer: impl AsyncWrite + Send + Sync + 'static,
    ) -> Result<ExternWriter> {
        let id = self.writers.put(Box::pin(writer));

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
        let writer = self.writers.get_mut(&req.id)?;
        let mut buf = memory.load(req.buf)?;

        Ok(io::response::WriterNext {
            len: writer.write_buf(&mut buf).await?.try_into()?,
        })
    }

    async fn handle_writer_flush(
        &mut self,
        req: io::request::WriterFlush,
    ) -> Result<io::response::WriterFlush> {
        let writer = self.writers.get_mut(&req.id)?;

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
        let writer = self.writers.get_mut(&req.id)?;

        writer
            .shutdown()
            .await
            .map(|()| io::response::WriterShutdown {})
            .map_err(Into::into)
    }

    async unsafe fn handle_writer_release(
        &mut self,
        req: io::request::WriterRelease,
    ) -> Result<io::response::WriterRelease> {
        self.writers
            .release_one(&req.id)
            .await
            .map(|()| io::response::WriterRelease {})
    }
}
