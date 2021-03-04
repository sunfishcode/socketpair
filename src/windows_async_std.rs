//! `AsyncSocketpairStream` and `async_socketpair_stream` for Windows.

use async_std::{
    fs::File,
    io::{self, IoSlice, IoSliceMut, Read, Write},
    os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle},
    path::Path,
};
use std::{
    convert::TryInto,
    os::windows::ffi::OsStrExt,
    pin::Pin,
    ptr,
    task::{Context, Poll},
};
use unsafe_io::{
    os::windows::{
        AsRawHandleOrSocket, AsRawReadWriteHandleOrSocket, IntoRawHandleOrSocket, RawHandleOrSocket,
    },
    OwnsRaw,
};
use uuid::Uuid;
use winapi::{
    shared::winerror::ERROR_ACCESS_DENIED,
    um::{
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        namedpipeapi::CreateNamedPipeW,
        winbase::{
            FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX, PIPE_READMODE_BYTE,
            PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES,
        },
        winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE},
    },
};

/// A socketpair stream, which is a bidirectional bytestream much like a
/// [`TcpStream`] except that it does not have a name or address.
///
/// [`TcpStream`]: async_std::net::TcpStream
///
/// TODO: Make this `Clone` once async-std releases with
/// https://github.com/async-rs/async-std/pull/937
#[repr(transparent)]
pub struct AsyncSocketpairStream(File);

impl AsyncSocketpairStream {
    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    pub fn peek(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        // TODO: Implement async peek on Windows
        Ok(0)
    }

    /// Return the number of bytes which are ready to be read immediately.
    pub fn num_ready_bytes(&self) -> io::Result<u64> {
        // TODO: Implement async num_ready_bytes on Windows
        Ok(0)
    }
}

/// Create a socketpair and return stream handles connected to each end.
pub async fn async_socketpair_stream() -> io::Result<(AsyncSocketpairStream, AsyncSocketpairStream)>
{
    let (first_raw_handle, path) = loop {
        let name = format!("\\\\.\\pipe\\{}", Uuid::new_v4());
        let mut path = Path::new(&name)
            .as_os_str()
            .encode_wide()
            .collect::<Vec<_>>();
        path.push(0);
        let first_raw_handle = unsafe {
            CreateNamedPipeW(
                path.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                4096,
                4096,
                0,
                ptr::null_mut(),
            )
        };
        if first_raw_handle != INVALID_HANDLE_VALUE {
            break (first_raw_handle, path);
        }
        let err = io::Error::last_os_error();
        if err.raw_os_error() != Some(ERROR_ACCESS_DENIED.try_into().unwrap()) {
            return Err(err);
        }
    };
    let first = unsafe { AsyncSocketpairStream::from_raw_handle(first_raw_handle) };

    let second_raw_handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    };
    if second_raw_handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let second = unsafe { AsyncSocketpairStream::from_raw_handle(second_raw_handle) };

    Ok((first, second))
}

impl Read for AsyncSocketpairStream {
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

impl Write for AsyncSocketpairStream {
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

impl AsRawHandle for AsyncSocketpairStream {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

impl IntoRawHandle for AsyncSocketpairStream {
    #[inline]
    fn into_raw_handle(self) -> RawHandle {
        self.0.into_raw_handle()
    }
}

impl FromRawHandle for AsyncSocketpairStream {
    #[inline]
    unsafe fn from_raw_handle(raw_handle: RawHandle) -> Self {
        Self(File::from_raw_handle(raw_handle))
    }
}

impl AsRawHandleOrSocket for AsyncSocketpairStream {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.0.as_raw_handle_or_socket()
    }
}

impl IntoRawHandleOrSocket for AsyncSocketpairStream {
    #[inline]
    fn into_raw_handle_or_socket(self) -> RawHandleOrSocket {
        self.0.into_raw_handle_or_socket()
    }
}

impl AsRawReadWriteHandleOrSocket for AsyncSocketpairStream {
    #[inline]
    fn as_raw_read_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }

    #[inline]
    fn as_raw_write_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }
}

/// Safety: `AsyncSocketpairStream` wraps a `TcpStream` which owns its handle.
unsafe impl OwnsRaw for AsyncSocketpairStream {}

impl std::fmt::Debug for AsyncSocketpairStream {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Just print the handles; don't try to print the path or any
        // information about it, because this information is otherwise
        // unavailable to safe Rust code.
        f.debug_struct("AsyncSocketpairStream")
            .field("raw_handle", &self.0.as_raw_handle())
            .finish()
    }
}
