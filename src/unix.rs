//! `SocketpairStream` and `socketpair_stream` for Posix-ish platforms.

use io_lifetimes::{AsFd, BorrowedFd, FromFd, IntoFd, OwnedFd};
use std::{
    fmt::{self, Arguments, Debug},
    io::{self, IoSlice, IoSliceMut, Read, Write},
    os::unix::net::UnixStream,
};
use unsafe_io::os::posish::{AsRawFd, AsRawReadWriteFd, FromRawFd, IntoRawFd, RawFd};
#[cfg(not(unix_socket_peek))]
use {io_lifetimes::AsSocketlike, std::net::TcpStream};

/// A socketpair stream, which is a bidirectional bytestream much like a
/// [`UnixStream`] except that it does not have a name or address.
#[repr(transparent)]
pub struct SocketpairStream(UnixStream);

impl SocketpairStream {
    /// Creates a new independently owned handle to the underlying socket.
    #[inline]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    #[inline]
    pub fn peek(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        #[cfg(unix_socket_peek)]
        {
            self.0.peek(buf)
        }

        #[cfg(not(unix_socket_peek))]
        {
            self.0.as_socketlike_view::<TcpStream>().peek(buf)
        }
    }

    /// Return the number of bytes which are ready to be read immediately.
    #[inline]
    pub fn num_ready_bytes(&self) -> io::Result<u64> {
        Ok(posish::io::ioctl_fionread(self)?)
    }
}

/// Create a socketpair and return stream handles connected to each end.
#[inline]
pub fn socketpair_stream() -> io::Result<(SocketpairStream, SocketpairStream)> {
    UnixStream::pair().map(|(a, b)| (SocketpairStream(a), SocketpairStream(b)))
}

impl Read for SocketpairStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
        self.0.read_vectored(bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_read_vectored(&self) -> bool {
        self.0.is_read_vectored()
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        self.0.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        self.0.read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        self.0.read_exact(buf)
    }
}

impl Write for SocketpairStream {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice]) -> io::Result<usize> {
        self.0.write_vectored(bufs)
    }

    #[cfg(can_vector)]
    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.0.is_write_vectored()
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.0.write_all(buf)
    }

    #[cfg(write_all_vectored)]
    #[inline]
    fn write_all_vectored(&mut self, bufs: &mut [IoSlice]) -> io::Result<()> {
        self.0.write_all_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        self.0.write_fmt(fmt)
    }
}

impl AsRawFd for SocketpairStream {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl AsFd for SocketpairStream {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl IntoRawFd for SocketpairStream {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl IntoFd for SocketpairStream {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        self.0.into_fd()
    }
}

impl FromRawFd for SocketpairStream {
    #[inline]
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(UnixStream::from_raw_fd(raw_fd))
    }
}

impl FromFd for SocketpairStream {
    #[inline]
    fn from_fd(fd: OwnedFd) -> Self {
        Self(UnixStream::from_fd(fd))
    }
}

impl AsRawReadWriteFd for SocketpairStream {
    #[inline]
    fn as_raw_read_fd(&self) -> RawFd {
        self.as_raw_fd()
    }

    #[inline]
    fn as_raw_write_fd(&self) -> RawFd {
        self.as_raw_fd()
    }
}

impl Debug for SocketpairStream {
    #[allow(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Just print the fd numbers; don't try to print the path or any
        // information about it, because this information is otherwise
        // unavailable to safe Rust code.
        f.debug_struct("SocketpairStream")
            .field("raw_fd", &self.0.as_raw_fd())
            .finish()
    }
}
