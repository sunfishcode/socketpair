//! `TokioSocketpairStream` and `tokio_socketpair_stream` for Unix platforms.

use io_lifetimes::{AsFd, BorrowedFd};
use std::{
    fmt::{self, Debug},
    io::IoSlice,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{self, AsyncRead, AsyncWrite, ReadBuf},
    net::UnixStream,
};
use unsafe_io::os::rsix::{AsRawFd, AsRawReadWriteFd, AsReadWriteFd, RawFd};

/// A socketpair stream, which is a bidirectional bytestream much like a
/// [`UnixStream`] except that it does not have a name or address.
#[repr(transparent)]
pub struct TokioSocketpairStream(UnixStream);

impl TokioSocketpairStream {
    // No `peek` in `tokio`'s `UnixStream` yet.
    /*
    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    #[inline]
    pub fn peek(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.peek(buf)
    }
    */

    /// Return the number of bytes which are ready to be read immediately.
    #[inline]
    pub fn num_ready_bytes(&self) -> io::Result<u64> {
        Ok(rsix::io::ioctl_fionread(self)?)
    }
}

/// Create a socketpair and return stream handles connected to each end.
#[inline]
pub async fn tokio_socketpair_stream() -> io::Result<(TokioSocketpairStream, TokioSocketpairStream)>
{
    UnixStream::pair().map(|(a, b)| (TokioSocketpairStream(a), TokioSocketpairStream(b)))
}

impl AsyncRead for TokioSocketpairStream {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for TokioSocketpairStream {
    #[inline]
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write_vectored(cx, bufs)
    }

    #[inline]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl AsRawFd for TokioSocketpairStream {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl AsFd for TokioSocketpairStream {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawReadWriteFd for TokioSocketpairStream {
    #[inline]
    fn as_raw_read_fd(&self) -> RawFd {
        self.as_raw_fd()
    }

    #[inline]
    fn as_raw_write_fd(&self) -> RawFd {
        self.as_raw_fd()
    }
}

impl AsReadWriteFd for TokioSocketpairStream {
    #[inline]
    fn as_read_fd(&self) -> BorrowedFd<'_> {
        self.as_fd()
    }

    #[inline]
    fn as_write_fd(&self) -> BorrowedFd<'_> {
        self.as_fd()
    }
}

impl Debug for TokioSocketpairStream {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Just print the fd numbers; don't try to print the path or any
        // information about it, because this information is otherwise
        // unavailable to safe Rust code.
        f.debug_struct("TokioSocketpairStream")
            .field("raw_fd", &self.0.as_raw_fd())
            .finish()
    }
}
