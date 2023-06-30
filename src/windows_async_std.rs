//! `AsyncStdSocketpairStream` and `async_std_socketpair_stream` for Windows.

use async_std::fs::File;
use async_std::io::{self, IoSlice, IoSliceMut, Read, Write};
use async_std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle};
use async_std::path::Path;
use io_extras::os::windows::{
    AsHandleOrSocket, AsRawHandleOrSocket, AsRawReadWriteHandleOrSocket, AsReadWriteHandleOrSocket,
    BorrowedHandleOrSocket, IntoRawHandleOrSocket, OwnedHandleOrSocket,
    RawHandleOrSocket,
};
use io_lifetimes::{AsHandle, BorrowedHandle, OwnedHandle};
use std::fmt::{self, Debug};
use std::os::windows::ffi::OsStrExt;
use std::pin::Pin;
use std::ptr;
use std::task::{Context, Poll};
use uuid::Uuid;
use windows_sys::Win32::Storage::FileSystem::{PIPE_ACCESS_DUPLEX, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_ATTRIBUTE_NORMAL, OPEN_EXISTING};
use windows_sys::Win32::Foundation::{INVALID_HANDLE_VALUE, ERROR_ACCESS_DENIED};
use windows_sys::Win32::System::Pipes::{PIPE_UNLIMITED_INSTANCES, PIPE_REJECT_REMOTE_CLIENTS};

/// A socketpair stream, which is a bidirectional bytestream much like a
/// [`TcpStream`] except that it does not have a name or address.
///
/// [`TcpStream`]: async_std::net::TcpStream
#[repr(transparent)]
#[derive(Clone)]
pub struct AsyncStdSocketpairStream(File);

impl AsyncStdSocketpairStream {
    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    pub fn peek(&self, _buf: &mut [u8]) -> io::Result<usize> {
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
pub async fn async_std_socketpair_stream(
) -> io::Result<(AsyncStdSocketpairStream, AsyncStdSocketpairStream)> {
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
    let first = unsafe { AsyncStdSocketpairStream::from_raw_handle(first_raw_handle) };

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
    let second = unsafe { AsyncStdSocketpairStream::from_raw_handle(second_raw_handle) };

    Ok((first, second))
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

impl AsRawHandle for AsyncStdSocketpairStream {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

impl AsHandle for AsyncStdSocketpairStream {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        self.0.as_handle()
    }
}

impl IntoRawHandle for AsyncStdSocketpairStream {
    #[inline]
    fn into_raw_handle(self) -> RawHandle {
        self.0.into_raw_handle()
    }
}

impl From<AsyncStdSocketpairStream> for OwnedHandle {
    #[inline]
    fn from(stream: AsyncStdSocketpairStream) -> Self {
        stream.0.into()
    }
}

impl FromRawHandle for AsyncStdSocketpairStream {
    #[inline]
    unsafe fn from_raw_handle(raw_handle: RawHandle) -> Self {
        Self(File::from_raw_handle(raw_handle))
    }
}

impl AsRawHandleOrSocket for AsyncStdSocketpairStream {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.0.as_raw_handle_or_socket()
    }
}

impl AsHandleOrSocket for AsyncStdSocketpairStream {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.0.as_handle_or_socket()
    }
}

impl IntoRawHandleOrSocket for AsyncStdSocketpairStream {
    #[inline]
    fn into_raw_handle_or_socket(self) -> RawHandleOrSocket {
        self.0.into_raw_handle_or_socket()
    }
}

impl From<AsyncStdSocketpairStream> for OwnedHandleOrSocket {
    #[inline]
    fn from(stream: AsyncStdSocketpairStream) -> Self {
        stream.0.into()
    }
}

impl AsRawReadWriteHandleOrSocket for AsyncStdSocketpairStream {
    #[inline]
    fn as_raw_read_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }

    #[inline]
    fn as_raw_write_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }
}

impl AsReadWriteHandleOrSocket for AsyncStdSocketpairStream {
    #[inline]
    fn as_read_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }

    #[inline]
    fn as_write_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }
}

impl Debug for AsyncStdSocketpairStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Just print the handles; don't try to print the path or any
        // information about it, because this information is otherwise
        // unavailable to safe Rust code.
        f.debug_struct("AsyncStdSocketpairStream")
            .field("raw_handle", &self.0.as_raw_handle())
            .finish()
    }
}
