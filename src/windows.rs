//! `SocketpairStream` and `socketpair_stream` for Windows.

use io_lifetimes::{AsHandle, BorrowedHandle, IntoHandle, OwnedHandle};
use std::cmp::min;
use std::convert::TryInto;
use std::fmt::{self, Arguments, Debug};
use std::fs::File;
use std::io::{self, IoSlice, IoSliceMut, Read, Write};
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle};
use std::path::Path;
use std::ptr;
use unsafe_io::os::windows::{
    AsHandleOrSocket, AsRawHandleOrSocket, AsRawReadWriteHandleOrSocket, AsReadWriteHandleOrSocket,
    BorrowedHandleOrSocket, IntoHandleOrSocket, IntoRawHandleOrSocket, OwnedHandleOrSocket,
    RawHandleOrSocket,
};
use uuid::Uuid;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::winerror::ERROR_ACCESS_DENIED;
use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::namedpipeapi::{CreateNamedPipeW, PeekNamedPipe};
use winapi::um::winbase::{
    FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES,
};
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE};

/// A socketpair stream, which is a bidirectional bytestream much like a
/// [`TcpStream`] except that it does not have a name or address.
///
/// [`TcpStream`]: std::net::TcpStream
#[repr(transparent)]
pub struct SocketpairStream(File);

impl SocketpairStream {
    /// Creates a new independently owned handle to the underlying socket.
    #[inline]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.0.try_clone().map(Self)
    }

    /// Receives data on the socket from the remote address to which it is
    /// connected, without removing that data from the queue. On success,
    /// returns the number of bytes peeked.
    pub fn peek(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut bytes_read = MaybeUninit::<u32>::uninit();
        let len = min(buf.len(), u32::MAX as usize) as u32;
        let res = unsafe {
            PeekNamedPipe(
                self.0.as_raw_handle(),
                buf.as_mut_ptr() as LPVOID,
                len,
                bytes_read.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if res == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { bytes_read.assume_init() } as usize)
    }

    /// Return the number of bytes which are ready to be read immediately.
    pub fn num_ready_bytes(&self) -> io::Result<u64> {
        let mut bytes_avail = MaybeUninit::<u32>::uninit();
        let res = unsafe {
            PeekNamedPipe(
                self.0.as_raw_handle(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                bytes_avail.as_mut_ptr(),
                ptr::null_mut(),
            )
        };
        if res == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(u64::from(unsafe { bytes_avail.assume_init() }))
    }
}

/// Create a socketpair and return stream handles connected to each end.
pub fn socketpair_stream() -> io::Result<(SocketpairStream, SocketpairStream)> {
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
    let first = unsafe { SocketpairStream::from_raw_handle(first_raw_handle) };

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
    let second = unsafe { SocketpairStream::from_raw_handle(second_raw_handle) };

    Ok((first, second))
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

impl AsRawHandle for SocketpairStream {
    #[inline]
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

impl AsHandle for SocketpairStream {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        self.0.as_handle()
    }
}

impl IntoRawHandle for SocketpairStream {
    #[inline]
    fn into_raw_handle(self) -> RawHandle {
        self.0.into_raw_handle()
    }
}

impl IntoHandle for SocketpairStream {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        self.0.into_handle()
    }
}

impl FromRawHandle for SocketpairStream {
    #[inline]
    unsafe fn from_raw_handle(raw_handle: RawHandle) -> Self {
        Self(File::from_raw_handle(raw_handle))
    }
}

impl AsRawHandleOrSocket for SocketpairStream {
    #[inline]
    fn as_raw_handle_or_socket(&self) -> RawHandleOrSocket {
        self.0.as_raw_handle_or_socket()
    }
}

impl AsHandleOrSocket for SocketpairStream {
    #[inline]
    fn as_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.0.as_handle_or_socket()
    }
}

impl IntoRawHandleOrSocket for SocketpairStream {
    #[inline]
    fn into_raw_handle_or_socket(self) -> RawHandleOrSocket {
        self.0.into_raw_handle_or_socket()
    }
}

impl IntoHandleOrSocket for SocketpairStream {
    #[inline]
    fn into_handle_or_socket(self) -> OwnedHandleOrSocket {
        self.0.into_handle_or_socket()
    }
}

impl AsRawReadWriteHandleOrSocket for SocketpairStream {
    #[inline]
    fn as_raw_read_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }

    #[inline]
    fn as_raw_write_handle_or_socket(&self) -> RawHandleOrSocket {
        self.as_raw_handle_or_socket()
    }
}

impl AsReadWriteHandleOrSocket for SocketpairStream {
    #[inline]
    fn as_read_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }

    #[inline]
    fn as_write_handle_or_socket(&self) -> BorrowedHandleOrSocket<'_> {
        self.as_handle_or_socket()
    }
}

impl Debug for SocketpairStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Just print the handles; don't try to print the path or any
        // information about it, because this information is otherwise
        // unavailable to safe Rust code.
        f.debug_struct("SocketpairStream")
            .field("raw_handle", &self.0.as_raw_handle())
            .finish()
    }
}
