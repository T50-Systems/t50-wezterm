impl FileDescriptor {
    #[inline]
    pub(crate) fn as_stdio_impl(&self) -> Result<std::process::Stdio> {
        let duped = self.handle.try_clone()?;
        let handle = duped.into_raw_handle();
        let stdio = unsafe { std::process::Stdio::from_raw_handle(handle) };
        Ok(stdio)
    }

    #[inline]
    pub(crate) fn as_file_impl(&self) -> Result<std::fs::File> {
        let duped = self.handle.try_clone()?;
        let handle = duped.into_raw_handle();
        let stdio = unsafe { std::fs::File::from_raw_handle(handle) };
        Ok(stdio)
    }

    #[inline]
    pub(crate) fn set_non_blocking_impl(&mut self, non_blocking: bool) -> Result<()> {
        if !self.handle.is_socket_handle() {
            return Err(Error::OnlySocketsNonBlocking);
        }

        let mut on = if non_blocking { 1 } else { 0 };
        let res = unsafe {
            ioctlsocket(
                self.as_raw_socket() as SOCKET,
                winapi::um::winsock2::FIONBIO,
                &mut on,
            )
        };
        if res != 0 {
            Err(Error::FionBio(std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }

    pub(crate) fn redirect_stdio_impl<F: AsRawFileDescriptor>(
        f: &F,
        stdio: StdioDescriptor,
    ) -> Result<Self> {
        let std_handle = match stdio {
            StdioDescriptor::Stdin => STD_INPUT_HANDLE,
            StdioDescriptor::Stdout => STD_OUTPUT_HANDLE,
            StdioDescriptor::Stderr => STD_ERROR_HANDLE,
        };

        let raw_std_handle = unsafe { GetStdHandle(std_handle) } as *mut _;
        let std_original = unsafe { FileDescriptor::from_raw_handle(raw_std_handle) };

        let cloned_handle = OwnedHandle::dup(f)?;
        if unsafe { SetStdHandle(std_handle, cloned_handle.into_raw_handle() as *mut _) } == 0 {
            Err(Error::SetStdHandle(std::io::Error::last_os_error()))
        } else {
            Ok(std_original)
        }
    }
}

impl IntoRawHandle for FileDescriptor {
    fn into_raw_handle(self) -> RawHandle {
        self.handle.into_raw_handle()
    }
}

impl AsRawHandle for FileDescriptor {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
    }
}

impl FromRawHandle for FileDescriptor {
    unsafe fn from_raw_handle(handle: RawHandle) -> FileDescriptor {
        Self {
            handle: OwnedHandle::from_raw_handle(handle),
        }
    }
}

impl IntoRawSocket for FileDescriptor {
    fn into_raw_socket(self) -> RawSocket {
        // FIXME: this isn't a guaranteed conversion!
        debug_assert!(self.handle.is_socket_handle());
        self.handle.into_raw_handle() as RawSocket
    }
}

impl AsRawSocket for FileDescriptor {
    fn as_raw_socket(&self) -> RawSocket {
        // FIXME: this isn't a guaranteed conversion!
        debug_assert!(self.handle.is_socket_handle());
        self.handle.as_raw_handle() as RawSocket
    }
}

impl AsSocket for FileDescriptor {
    fn as_socket(&self) -> BorrowedSocket {
        unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
    }
}

impl FromRawSocket for FileDescriptor {
    unsafe fn from_raw_socket(handle: RawSocket) -> FileDescriptor {
        Self {
            handle: OwnedHandle::from_raw_handle(handle as RawHandle),
        }
    }
}

impl io::Read for FileDescriptor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.handle.is_socket_handle() {
            // It's important to use the winsock functions to read/write
            // even though ReadFile and WriteFile technically work; only
            // the winsock functions respect non-blocking mode.
            let num_read = unsafe {
                recv(
                    self.as_socket_descriptor(),
                    buf.as_mut_ptr() as *mut _,
                    buf.len() as _,
                    0,
                )
            };
            if num_read < 0 {
                Err(IoError::last_os_error())
            } else {
                Ok(num_read as usize)
            }
        } else {
            let mut num_read = 0;
            let ok = unsafe {
                ReadFile(
                    self.handle.as_raw_handle() as *mut _,
                    buf.as_mut_ptr() as *mut _,
                    buf.len() as _,
                    &mut num_read,
                    ptr::null_mut(),
                )
            };
            if ok == 0 {
                let err = IoError::last_os_error();
                if err.kind() == std::io::ErrorKind::BrokenPipe {
                    Ok(0)
                } else {
                    Err(err)
                }
            } else {
                Ok(num_read as usize)
            }
        }
    }
}

impl io::Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.handle.is_socket_handle() {
            let num_wrote = unsafe {
                send(
                    self.as_socket_descriptor(),
                    buf.as_ptr() as *const _,
                    buf.len() as _,
                    0,
                )
            };
            if num_wrote < 0 {
                Err(IoError::last_os_error())
            } else {
                Ok(num_wrote as usize)
            }
        } else {
            let mut num_wrote = 0;
            let ok = unsafe {
                WriteFile(
                    self.handle.as_raw_handle() as *mut _,
                    buf.as_ptr() as *const _,
                    buf.len() as u32,
                    &mut num_wrote,
                    ptr::null_mut(),
                )
            };
            if ok == 0 {
                Err(IoError::last_os_error())
            } else {
                Ok(num_wrote as usize)
            }
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
