use crate::{
    AsRawFileDescriptor, AsRawSocketDescriptor, Error, FileDescriptor, FromRawFileDescriptor,
    FromRawSocketDescriptor, IntoRawFileDescriptor, IntoRawSocketDescriptor, OwnedHandle, Pipe,
    Result, StdioDescriptor,
};
use std::os::unix::prelude::*;

pub(crate) type HandleType = ();

/// `RawFileDescriptor` is a platform independent type alias for the
/// underlying platform file descriptor type.  It is primarily useful
/// for avoiding using `cfg` blocks in platform independent code.
pub type RawFileDescriptor = RawFd;

/// `SocketDescriptor` is a platform independent type alias for the
/// underlying platform socket descriptor type.  It is primarily useful
/// for avoiding using `cfg` blocks in platform independent code.
pub type SocketDescriptor = RawFd;

impl<T: AsRawFd> AsRawFileDescriptor for T {
    fn as_raw_file_descriptor(&self) -> RawFileDescriptor {
        self.as_raw_fd()
    }
}

impl<T: IntoRawFd> IntoRawFileDescriptor for T {
    fn into_raw_file_descriptor(self) -> RawFileDescriptor {
        self.into_raw_fd()
    }
}

impl<T: FromRawFd> FromRawFileDescriptor for T {
    unsafe fn from_raw_file_descriptor(fd: RawFileDescriptor) -> Self {
        Self::from_raw_fd(fd)
    }
}

impl<T: AsRawFd> AsRawSocketDescriptor for T {
    fn as_socket_descriptor(&self) -> SocketDescriptor {
        self.as_raw_fd()
    }
}

impl<T: IntoRawFd> IntoRawSocketDescriptor for T {
    fn into_socket_descriptor(self) -> SocketDescriptor {
        self.into_raw_fd()
    }
}

impl<T: FromRawFd> FromRawSocketDescriptor for T {
    unsafe fn from_socket_descriptor(fd: SocketDescriptor) -> Self {
        Self::from_raw_fd(fd)
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.handle);
        }
    }
}

impl std::os::fd::AsFd for OwnedHandle {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        unsafe { std::os::fd::BorrowedFd::borrow_raw(self.handle) }
    }
}

impl AsRawFd for OwnedHandle {
    fn as_raw_fd(&self) -> RawFd {
        self.handle
    }
}

impl IntoRawFd for OwnedHandle {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.handle;
        std::mem::forget(self);
        fd
    }
}

impl FromRawFd for OwnedHandle {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            handle: fd,
            handle_type: (),
        }
    }
}

impl OwnedHandle {
    /// Helper function to set the close-on-exec flag for a raw descriptor
    fn cloexec(&mut self) -> Result<()> {
        let flags = unsafe { libc::fcntl(self.handle, libc::F_GETFD) };
        if flags == -1 {
            return Err(Error::Fcntl(std::io::Error::last_os_error()));
        }
        let result = unsafe { libc::fcntl(self.handle, libc::F_SETFD, flags | libc::FD_CLOEXEC) };
        if result == -1 {
            Err(Error::Cloexec(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    fn non_atomic_dup(fd: RawFd) -> Result<Self> {
        let duped = unsafe { libc::dup(fd) };
        if duped == -1 {
            Err(Error::Dup {
                fd: fd.into(),
                source: std::io::Error::last_os_error(),
            })
        } else {
            let mut owned = OwnedHandle {
                handle: duped,
                handle_type: (),
            };
            owned.cloexec()?;
            Ok(owned)
        }
    }

    fn non_atomic_dup2(fd: RawFd, dest_fd: RawFd) -> Result<Self> {
        let duped = unsafe { libc::dup2(fd, dest_fd) };
        if duped == -1 {
            Err(Error::Dup2 {
                src_fd: fd.into(),
                dest_fd: dest_fd.into(),
                source: std::io::Error::last_os_error(),
            })
        } else {
            let mut owned = OwnedHandle {
                handle: duped,
                handle_type: (),
            };
            owned.cloexec()?;
            Ok(owned)
        }
    }

    #[inline]
    pub(crate) fn dup_impl<F: AsRawFileDescriptor>(
        fd: &F,
        handle_type: HandleType,
    ) -> Result<Self> {
        let fd = fd.as_raw_file_descriptor();
        let duped = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
        if duped == -1 {
            let err = std::io::Error::last_os_error();
            if let Some(libc::EINVAL) = err.raw_os_error() {
                // We may be running on eg: WSL or an old kernel that
                // doesn't support F_DUPFD_CLOEXEC; fall back.
                Self::non_atomic_dup(fd)
            } else {
                Err(Error::Dup {
                    fd: fd.into(),
                    source: err,
                })
            }
        } else {
            Ok(OwnedHandle {
                handle: duped,
                handle_type,
            })
        }
    }

    #[inline]
    pub(crate) unsafe fn dup2_impl<F: AsRawFileDescriptor>(fd: &F, dest_fd: RawFd) -> Result<Self> {
        let fd = fd.as_raw_file_descriptor();

        #[cfg(not(target_os = "linux"))]
        return Self::non_atomic_dup2(fd, dest_fd);

        #[cfg(target_os = "linux")]
        {
            let duped = libc::dup3(fd, dest_fd, libc::O_CLOEXEC);

            if duped == -1 {
                let err = std::io::Error::last_os_error();
                if let Some(libc::EINVAL) = err.raw_os_error() {
                    // We may be running on eg: WSL or an old kernel that
                    // doesn't support O_CLOEXEC; fall back.
                    Self::non_atomic_dup2(fd, dest_fd)
                } else {
                    Err(Error::Dup2 {
                        src_fd: fd.into(),
                        dest_fd: dest_fd.into(),
                        source: err,
                    })
                }
            } else {
                Ok(OwnedHandle {
                    handle: duped,
                    handle_type: (),
                })
            }
        }
    }

    pub(crate) fn probe_handle_type(_handle: RawFileDescriptor) -> HandleType {
        ()
    }
}
