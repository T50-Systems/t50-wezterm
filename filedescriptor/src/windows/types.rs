use crate::{
    AsRawFileDescriptor, AsRawSocketDescriptor, Error, FileDescriptor, FromRawFileDescriptor,
    FromRawSocketDescriptor, IntoRawFileDescriptor, IntoRawSocketDescriptor, OwnedHandle, Pipe,
    Result, StdioDescriptor,
};
use std::io::{self, Error as IoError};
use std::os::windows::prelude::*;
use std::ptr;
use std::sync::Once;
use std::time::Duration;
use winapi::shared::ws2def::{AF_INET, INADDR_LOOPBACK, SOCKADDR_IN};
use winapi::um::fileapi::*;
use winapi::um::handleapi::*;
use winapi::um::minwinbase::SECURITY_ATTRIBUTES;
use winapi::um::namedpipeapi::{CreatePipe, GetNamedPipeInfo};
use winapi::um::processenv::{GetStdHandle, SetStdHandle};
use winapi::um::processthreadsapi::*;
use winapi::um::winbase::{FILE_TYPE_CHAR, FILE_TYPE_DISK, FILE_TYPE_PIPE};
use winapi::um::winnt::HANDLE;
use winapi::um::winsock2::{
    accept, bind, closesocket, connect, getsockname, getsockopt, htonl, ioctlsocket, listen, recv,
    send, WSAGetLastError, WSAPoll, WSASocketW, WSAStartup, INVALID_SOCKET, SOCKET, SOCK_STREAM,
    SOL_SOCKET, SO_ERROR, WSADATA, WSAENOTSOCK, WSA_FLAG_NO_HANDLE_INHERIT,
};
pub use winapi::um::winsock2::{POLLERR, POLLHUP, POLLIN, POLLOUT, WSAPOLLFD as pollfd};

/// `RawFileDescriptor` is a platform independent type alias for the
/// underlying platform file descriptor type.  It is primarily useful
/// for avoiding using `cfg` blocks in platform independent code.
pub type RawFileDescriptor = RawHandle;

/// `SocketDescriptor` is a platform independent type alias for the
/// underlying platform socket descriptor type.  It is primarily useful
/// for avoiding using `cfg` blocks in platform independent code.
pub type SocketDescriptor = SOCKET;

const STD_INPUT_HANDLE: u32 = 4294967286; // -10
const STD_OUTPUT_HANDLE: u32 = 4294967285; // -11
const STD_ERROR_HANDLE: u32 = 4294967284; // -12

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HandleType {
    Char,
    Disk,
    Pipe,
    Socket,
    Unknown,
}

impl Default for HandleType {
    fn default() -> Self {
        HandleType::Unknown
    }
}

impl<T: AsRawHandle> AsRawFileDescriptor for T {
    fn as_raw_file_descriptor(&self) -> RawFileDescriptor {
        self.as_raw_handle()
    }
}

impl<T: IntoRawHandle> IntoRawFileDescriptor for T {
    fn into_raw_file_descriptor(self) -> RawFileDescriptor {
        self.into_raw_handle()
    }
}

impl<T: FromRawHandle> FromRawFileDescriptor for T {
    unsafe fn from_raw_file_descriptor(handle: RawHandle) -> Self {
        Self::from_raw_handle(handle)
    }
}

impl<T: AsRawSocket> AsRawSocketDescriptor for T {
    fn as_socket_descriptor(&self) -> SocketDescriptor {
        self.as_raw_socket() as SocketDescriptor
    }
}

impl<T: IntoRawSocket> IntoRawSocketDescriptor for T {
    fn into_socket_descriptor(self) -> SocketDescriptor {
        self.into_raw_socket() as SocketDescriptor
    }
}

impl<T: FromRawSocket> FromRawSocketDescriptor for T {
    unsafe fn from_socket_descriptor(handle: SocketDescriptor) -> Self {
        Self::from_raw_socket(handle as _)
    }
}

unsafe impl Send for OwnedHandle {}
unsafe impl Sync for OwnedHandle {}

impl OwnedHandle {
    fn probe_handle_type_if_unknown(handle: RawHandle, handle_type: HandleType) -> HandleType {
        match handle_type {
            HandleType::Unknown => Self::probe_handle_type(handle),
            t => t,
        }
    }

    pub(crate) fn probe_handle_type(handle: RawHandle) -> HandleType {
        let handle = handle as HANDLE;
        match unsafe { GetFileType(handle) } {
            FILE_TYPE_CHAR => HandleType::Char,
            FILE_TYPE_DISK => HandleType::Disk,
            FILE_TYPE_PIPE => {
                // Could be a pipe or a socket.  Test if for pipeness
                let mut flags = 0;
                let mut out_buf = 0;
                let mut in_buf = 0;
                let mut inst = 0;
                if unsafe {
                    GetNamedPipeInfo(handle, &mut flags, &mut out_buf, &mut in_buf, &mut inst)
                } != 0
                {
                    HandleType::Pipe
                } else {
                    // It's probably a socket, but it may be a special device used
                    // when piping between WSL and native win32 apps.
                    let mut err = 0;
                    let mut errsize = std::mem::size_of_val(&err) as _;
                    if unsafe {
                        getsockopt(
                            handle as _,
                            SOL_SOCKET,
                            SO_ERROR,
                            &mut err as *mut _ as *mut i8,
                            &mut errsize,
                        ) != 0
                            && WSAGetLastError() == WSAENOTSOCK
                    } {
                        HandleType::Pipe
                    } else {
                        HandleType::Socket
                    }
                }
            }
            _ => HandleType::Unknown,
        }
    }

    fn is_socket_handle(&self) -> bool {
        match self.handle_type {
            HandleType::Socket => true,
            HandleType::Unknown => Self::probe_handle_type(self.handle) == HandleType::Socket,
            _ => false,
        }
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if self.handle != INVALID_HANDLE_VALUE as _ && !self.handle.is_null() {
            unsafe {
                if self.is_socket_handle() {
                    closesocket(self.handle as _);
                } else {
                    CloseHandle(self.handle as _);
                }
            };
        }
    }
}

impl FromRawHandle for OwnedHandle {
    unsafe fn from_raw_handle(handle: RawHandle) -> Self {
        OwnedHandle {
            handle,
            handle_type: Self::probe_handle_type(handle),
        }
    }
}

impl OwnedHandle {
    #[inline]
    pub(crate) fn dup_impl<F: AsRawFileDescriptor>(f: &F, handle_type: HandleType) -> Result<Self> {
        let handle = f.as_raw_file_descriptor();
        if handle == INVALID_HANDLE_VALUE as _ || handle.is_null() {
            return Ok(OwnedHandle {
                handle,
                handle_type,
            });
        }

        let handle_type = Self::probe_handle_type_if_unknown(handle, handle_type);

        let proc = unsafe { GetCurrentProcess() };
        let mut duped = INVALID_HANDLE_VALUE;
        let ok = unsafe {
            DuplicateHandle(
                proc,
                handle as *mut _,
                proc,
                &mut duped,
                0,
                0, // not inheritable
                winapi::um::winnt::DUPLICATE_SAME_ACCESS,
            )
        };
        if ok == 0 {
            Err(IoError::last_os_error().into())
        } else {
            Ok(OwnedHandle {
                handle: duped as *mut _,
                handle_type,
            })
        }
    }
}

impl AsRawHandle for OwnedHandle {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle
    }
}

impl IntoRawHandle for OwnedHandle {
    fn into_raw_handle(self) -> RawHandle {
        let handle = self.handle;
        std::mem::forget(self);
        handle
    }
}
