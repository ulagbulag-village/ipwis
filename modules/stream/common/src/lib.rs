mod extrinsics;

use core::{
    pin::Pin,
    task::{Context, Poll},
};

use ipis::tokio::io::{self, AsyncRead, AsyncWrite, ReadBuf};
use ipwis_common::{data::ExternData, interrupt::InterruptId};

#[repr(C)]
pub struct ExternReader {
    id: InterruptId,
    len: u32,
}

unsafe impl Send for ExternReader {}
unsafe impl Sync for ExternReader {}

impl ExternReader {
    pub fn new(id: InterruptId, len: u32) -> Self {
        Self { id, len }
    }
}

impl AsyncRead for ExternReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        unsafe {
            let unfilled_buf = buf.unfilled_mut();
            let len = unfilled_buf.len();

            let mut buf_next = ExternData::from_slice_uninit_mut(unfilled_buf);

            // use the whole buffer
            buf.assume_init(len);

            // fill in buffer
            let filled_len = run_extrinsic(|error| {
                self::extrinsics::reader_next(error, self.id.0, buf_next.as_mut_ptr())
            })?;
            buf.set_filled(filled_len as usize);
        };
        Poll::Ready(Ok(()))
    }
}

#[repr(C)]
pub struct ExternWriter {
    id: InterruptId,
}

unsafe impl Send for ExternWriter {}
unsafe impl Sync for ExternWriter {}

impl ExternWriter {
    pub fn new(id: InterruptId) -> Self {
        Self { id }
    }
}

impl AsyncWrite for ExternWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let len = unsafe {
            let buf_next = ExternData::from_slice(buf);

            run_extrinsic(|error| {
                self::extrinsics::writer_next(error, self.id.0, buf_next.as_ptr())
            })?
        };
        Poll::Ready(Ok(len as usize))
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let () =
            unsafe { run_extrinsic(|error| self::extrinsics::writer_flush(error, self.id.0))? };
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let () =
            unsafe { run_extrinsic(|error| self::extrinsics::writer_shutdown(error, self.id.0))? };
        Poll::Ready(Ok(()))
    }
}

unsafe fn run_extrinsic<F, R>(f: F) -> Result<R, io::Error>
where
    F: FnOnce(u32) -> R,
{
    let mut error = ExternData::default();

    let ret = f(error.as_mut_ptr());

    error
        .assume_error()
        .map(|()| ret)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}
