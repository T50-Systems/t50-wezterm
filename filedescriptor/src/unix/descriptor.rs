impl std::io::Read for FileDescriptor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = unsafe { libc::read(self.handle.handle, buf.as_mut_ptr() as *mut _, buf.len()) };
        if size == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(size as usize)
        }
    }
}

impl std::io::Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size = unsafe { libc::write(self.handle.handle, buf.as_ptr() as *const _, buf.len()) };
        if size == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(size as usize)
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::os::fd::AsFd for FileDescriptor {
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.handle.as_fd()
    }
}

impl AsRawFd for FileDescriptor {
    fn as_raw_fd(&self) -> RawFd {
        self.handle.as_raw_fd()
    }
}

impl IntoRawFd for FileDescriptor {
    fn into_raw_fd(self) -> RawFd {
        self.handle.into_raw_fd()
    }
}

impl FromRawFd for FileDescriptor {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            handle: OwnedHandle::from_raw_fd(fd),
        }
    }
}

impl FileDescriptor {
    #[inline]
    pub(crate) fn as_stdio_impl(&self) -> Result<std::process::Stdio> {
        let duped = OwnedHandle::dup(self)?;
        let fd = duped.into_raw_fd();
        let stdio = unsafe { std::process::Stdio::from_raw_fd(fd) };
        Ok(stdio)
    }

    #[inline]
    pub(crate) fn as_file_impl(&self) -> Result<std::fs::File> {
        let duped = OwnedHandle::dup(self)?;
        let fd = duped.into_raw_fd();
        let stdio = unsafe { std::fs::File::from_raw_fd(fd) };
        Ok(stdio)
    }

    #[inline]
    pub(crate) fn set_non_blocking_impl(&mut self, non_blocking: bool) -> Result<()> {
        let on = if non_blocking { 1 } else { 0 };
        let res = unsafe { libc::ioctl(self.handle.as_raw_file_descriptor(), libc::FIONBIO, &on) };
        if res != 0 {
            Err(Error::FionBio(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    /// Attempt to duplicate the underlying handle from an object that is
    /// representable as the system `RawFileDescriptor` type and assign it to
    /// a destination file descriptor. It then returns a `FileDescriptor`
    /// wrapped around the duplicate.  Since the duplication requires kernel
    /// resources that may not be available, this is a potentially fallible operation.
    /// The returned handle has a separate lifetime from the source, but
    /// references the same object at the kernel level.
    pub unsafe fn dup2<F: AsRawFileDescriptor>(f: &F, dest_fd: RawFd) -> Result<Self> {
        OwnedHandle::dup2_impl(f, dest_fd).map(|handle| Self { handle })
    }

    /// Helper function to unset the close-on-exec flag for a raw descriptor
    fn no_cloexec(fd: RawFd) -> Result<()> {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags == -1 {
            return Err(Error::Fcntl(std::io::Error::last_os_error()));
        }
        let result = unsafe { libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) };
        if result == -1 {
            Err(Error::Cloexec(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn redirect_stdio_impl<F: AsRawFileDescriptor>(
        f: &F,
        stdio: StdioDescriptor,
    ) -> Result<Self> {
        let std_descriptor = match stdio {
            StdioDescriptor::Stdin => libc::STDIN_FILENO,
            StdioDescriptor::Stdout => libc::STDOUT_FILENO,
            StdioDescriptor::Stderr => libc::STDERR_FILENO,
        };

        let std_original = FileDescriptor::dup(&std_descriptor)?;
        // Assign f into std_descriptor, then convert to an fd so that
        // we don't close it when the returned FileDescriptor is dropped.
        // Then we discard/ignore the fd because it is nominally owned by
        // the stdio machinery for the process
        let _ = unsafe { FileDescriptor::dup2(f, std_descriptor) }?.into_raw_fd();
        Self::no_cloexec(std_descriptor)?;

        Ok(std_original)
    }
}
