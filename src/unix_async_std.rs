//! `AsyncStdSocketpairStream` and `async_std_socketpair_stream` for Unix platforms.

use async_std::{
    io::{self, IoSlice, IoSliceMut, Read, Write},
    os::unix::net::UnixStream,
};
use std::{
    fmt::{self, Debug},
    pin::Pin,
    task::{Context, Poll},
};
use unsafe_io::{
    os::posish::{AsRawFd, AsRawReadWriteFd, FromRawFd, IntoRawFd, RawFd},
    OwnsRaw,
};

/// A socketpair stream, which is a bidirectional bytestream much like a
/// [`UnixStream`] except that it does not have a name or address.
#[repr(transparent)]
#[derive(Clone)]
pub struct AsyncStdSocketpairStream(UnixStream);

impl AsyncStdSocketpairStream {
    // No `peek` in `async_std`'s `UnixStream` yet.
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
        posish::io::fionread(self)
    }
}

/// Create a socketpair and return stream handles connected to each end.
#[inline]
pub async fn async_std_socketpair_stream(
) -> io::Result<(AsyncStdSocketpairStream, AsyncStdSocketpairStream)> {
    UnixStream::pair().map(|(a, b)| (AsyncStdSocketpairStream(a), AsyncStdSocketpairStream(b)))
}

impl Read for AsyncStdSocketpairStream {
    #[inline]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }

    #[inline]
    fn poll_read_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_read_vectored(cx, bufs)
    }
}

impl Write for AsyncStdSocketpairStream {
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
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

impl AsRawFd for AsyncStdSocketpairStream {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for AsyncStdSocketpairStream {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for AsyncStdSocketpairStream {
    #[inline]
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(UnixStream::from_raw_fd(raw_fd))
    }
}

impl AsRawReadWriteFd for AsyncStdSocketpairStream {
    #[inline]
    fn as_raw_read_fd(&self) -> RawFd {
        self.as_raw_fd()
    }

    #[inline]
    fn as_raw_write_fd(&self) -> RawFd {
        self.as_raw_fd()
    }
}

/// Safety: `SocketpairStream` wraps a `UnixStream` which owns its handle.
unsafe impl OwnsRaw for AsyncStdSocketpairStream {}

impl Debug for AsyncStdSocketpairStream {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Just print the fd numbers; don't try to print the path or any
        // information about it, because this information is otherwise
        // unavailable to safe Rust code.
        f.debug_struct("AsyncStdSocketpairStream")
            .field("raw_fd", &self.0.as_raw_fd())
            .finish()
    }
}