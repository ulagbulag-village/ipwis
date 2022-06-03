use core::{
    pin::Pin,
    task::{Context, Poll},
};

use bytecheck::CheckBytes;
use ipis::tokio::{
    self,
    io::{AsyncRead, AsyncWrite, ReadBuf},
};
use ipwis_kernel_common::{data::ExternData, interrupt::InterruptId, resource::ResourceId};
use rkyv::{
    de::deserializers::SharedDeserializeMap, validation::validators::DefaultValidator, Archive,
    Deserialize, Serialize,
};

#[repr(C)]
pub struct ExternReader {
    id: ResourceId,
    len: u32,
}

impl ExternReader {
    pub fn new(id: ResourceId, len: u32) -> Self {
        Self { id, len }
    }
}

impl AsyncRead for ExternReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        unsafe {
            // fill in buffer
            let opcode = self::io::request::ReaderNext {
                id: self.id,
                buf: {
                    let unfilled_buf = buf.unfilled_mut();
                    let len = unfilled_buf.len();

                    let unfilled_buf = ExternData::from_slice_uninit_mut(unfilled_buf);

                    // use the whole buffer
                    buf.assume_init(len);

                    unfilled_buf
                },
            };
            buf.set_filled(opcode.syscall()?.len as usize);
        };
        Poll::Ready(Ok(()))
    }
}

#[repr(C)]
pub struct ExternWriter {
    id: ResourceId,
}

impl ExternWriter {
    pub fn new(id: ResourceId) -> Self {
        Self { id }
    }
}

impl AsyncWrite for ExternWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, tokio::io::Error>> {
        let len = unsafe {
            let opcode = self::io::request::WriterNext {
                id: self.id,
                buf: ExternData::from_slice(buf),
            };
            opcode.syscall()?.len
        };
        Poll::Ready(Ok(len as usize))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), tokio::io::Error>> {
        let () = unsafe {
            let opcode = self::io::request::WriterFlush { id: self.id };
            opcode.syscall()?;
        };
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<(), tokio::io::Error>> {
        let () = unsafe {
            let opcode = self::io::request::WriterShutdown { id: self.id };
            opcode.syscall()?;
        };
        Poll::Ready(Ok(()))
    }
}

pub mod io {
    use super::*;

    #[derive(Archive, Serialize, Deserialize)]
    #[archive_attr(derive(CheckBytes))]
    pub enum OpCode {
        ReaderNext(self::request::ReaderNext),
        WriterNext(self::request::WriterNext),
        WriterFlush(self::request::WriterFlush),
        WriterShutdown(self::request::WriterShutdown),
    }

    impl ::ipis::core::signed::IsSigned for OpCode {}

    impl OpCode {
        pub const ID: InterruptId = InterruptId("ipwis_modules_stream");

        unsafe fn syscall<O>(mut self) -> Result<O, tokio::io::Error>
        where
            O: Archive,
            <O as Archive>::Archived:
                for<'a> CheckBytes<DefaultValidator<'a>> + Deserialize<O, SharedDeserializeMap>,
        {
            Self::ID
                .syscall(&mut self)
                .map_err(|e| tokio::io::Error::new(tokio::io::ErrorKind::Other, e))
        }
    }

    pub mod request {
        use super::*;

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct ReaderNext {
            pub id: ResourceId,
            pub buf: ExternData,
        }

        impl ::ipis::core::signed::IsSigned for ReaderNext {}

        impl ReaderNext {
            pub(crate) unsafe fn syscall(
                self,
            ) -> Result<super::response::ReaderNext, tokio::io::Error> {
                super::OpCode::ReaderNext(self).syscall()
            }
        }

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct WriterNext {
            pub id: ResourceId,
            pub buf: ExternData,
        }

        impl ::ipis::core::signed::IsSigned for WriterNext {}

        impl WriterNext {
            pub(crate) unsafe fn syscall(
                self,
            ) -> Result<super::response::WriterNext, tokio::io::Error> {
                super::OpCode::WriterNext(self).syscall()
            }
        }

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct WriterFlush {
            pub id: ResourceId,
        }

        impl ::ipis::core::signed::IsSigned for WriterFlush {}

        impl WriterFlush {
            pub(crate) unsafe fn syscall(
                self,
            ) -> Result<super::response::WriterFlush, tokio::io::Error> {
                super::OpCode::WriterFlush(self).syscall()
            }
        }

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct WriterShutdown {
            pub id: ResourceId,
        }

        impl ::ipis::core::signed::IsSigned for WriterShutdown {}

        impl WriterShutdown {
            pub(crate) unsafe fn syscall(
                self,
            ) -> Result<super::response::WriterShutdown, tokio::io::Error> {
                super::OpCode::WriterShutdown(self).syscall()
            }
        }
    }

    pub mod response {
        use super::*;

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct ReaderNext {
            pub len: u32,
        }

        impl ::ipis::core::signed::IsSigned for ReaderNext {}

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct WriterNext {
            pub len: u32,
        }

        impl ::ipis::core::signed::IsSigned for WriterNext {}

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct WriterFlush {}

        impl ::ipis::core::signed::IsSigned for WriterFlush {}

        #[derive(Archive, Serialize, Deserialize)]
        #[archive_attr(derive(CheckBytes))]
        pub struct WriterShutdown {}

        impl ::ipis::core::signed::IsSigned for WriterShutdown {}
    }
}
