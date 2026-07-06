impl Pipe {
    pub fn new() -> Result<Pipe> {
        let mut sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 0,
        };
        let mut read: HANDLE = INVALID_HANDLE_VALUE as _;
        let mut write: HANDLE = INVALID_HANDLE_VALUE as _;
        if unsafe { CreatePipe(&mut read, &mut write, &mut sa, 0) } == 0 {
            Err(Error::Pipe(IoError::last_os_error()))
        } else {
            Ok(Pipe {
                read: FileDescriptor {
                    handle: OwnedHandle {
                        handle: read as _,
                        handle_type: HandleType::Pipe,
                    },
                },
                write: FileDescriptor {
                    handle: OwnedHandle {
                        handle: write as _,
                        handle_type: HandleType::Pipe,
                    },
                },
            })
        }
    }
}

fn init_winsock() {
    static START: Once = Once::new();
    START.call_once(|| unsafe {
        let mut data: WSADATA = std::mem::zeroed();
        let ret = WSAStartup(
            0x202, // version 2.2
            &mut data,
        );
        assert_eq!(ret, 0, "failed to initialize winsock");
    });
}

fn socket(af: i32, sock_type: i32, proto: i32) -> Result<FileDescriptor> {
    let s = unsafe {
        WSASocketW(
            af,
            sock_type,
            proto,
            ptr::null_mut(),
            0,
            WSA_FLAG_NO_HANDLE_INHERIT,
        )
    };
    if s == INVALID_SOCKET {
        Err(Error::Socket(IoError::last_os_error()))
    } else {
        Ok(FileDescriptor {
            handle: OwnedHandle {
                handle: s as _,
                handle_type: HandleType::Socket,
            },
        })
    }
}

#[doc(hidden)]
pub fn socketpair_impl() -> Result<(FileDescriptor, FileDescriptor)> {
    init_winsock();

    let s = socket(AF_INET, SOCK_STREAM, 0)?;

    let mut in_addr: SOCKADDR_IN = unsafe { std::mem::zeroed() };
    in_addr.sin_family = AF_INET as _;
    unsafe {
        *in_addr.sin_addr.S_un.S_addr_mut() = htonl(INADDR_LOOPBACK);
    }

    unsafe {
        if bind(
            s.as_raw_handle() as _,
            std::mem::transmute(&in_addr),
            std::mem::size_of_val(&in_addr) as _,
        ) != 0
        {
            return Err(Error::Bind(IoError::last_os_error()));
        }
    }

    let mut addr_len = std::mem::size_of_val(&in_addr) as i32;

    unsafe {
        if getsockname(
            s.as_raw_handle() as _,
            std::mem::transmute(&mut in_addr),
            &mut addr_len,
        ) != 0
        {
            return Err(Error::Getsockname(IoError::last_os_error()));
        }
    }

    unsafe {
        if listen(s.as_raw_handle() as _, 1) != 0 {
            return Err(Error::Listen(IoError::last_os_error()));
        }
    }

    let client = socket(AF_INET, SOCK_STREAM, 0)?;

    unsafe {
        if connect(
            client.as_raw_handle() as _,
            std::mem::transmute(&in_addr),
            addr_len,
        ) != 0
        {
            return Err(Error::Connect(IoError::last_os_error()));
        }
    }

    let server = unsafe { accept(s.as_raw_handle() as _, ptr::null_mut(), ptr::null_mut()) };
    if server == INVALID_SOCKET {
        return Err(Error::Accept(IoError::last_os_error()));
    }
    let server = FileDescriptor {
        handle: OwnedHandle {
            handle: server as _,
            handle_type: HandleType::Socket,
        },
    };

    Ok((server, client))
}

#[doc(hidden)]
pub fn poll_impl(pfd: &mut [pollfd], duration: Option<Duration>) -> Result<usize> {
    let poll_result = unsafe {
        WSAPoll(
            pfd.as_mut_ptr(),
            pfd.len() as _,
            duration
                .map(|wait| wait.as_millis() as libc::c_int)
                .unwrap_or(-1),
        )
    };
    if poll_result < 0 {
        Err(std::io::Error::last_os_error().into())
    } else {
        Ok(poll_result as usize)
    }
}
