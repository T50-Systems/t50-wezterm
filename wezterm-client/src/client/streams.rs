#[async_trait(?Send)]
pub trait AsyncReadAndWrite: Unpin + AsyncRead + AsyncWrite + std::fmt::Debug + Send {
    async fn wait_for_readable(&self) -> anyhow::Result<()>;
}

#[async_trait(?Send)]
impl<T> AsyncReadAndWrite for Async<T>
where
    T: std::fmt::Debug,
    T: std::io::Write,
    T: std::io::Read,
    T: Send,
    T: async_io::IoSafe,
{
    async fn wait_for_readable(&self) -> anyhow::Result<()> {
        Ok(self.readable().await?)
    }
}

#[derive(Debug)]
struct Reconnectable {
    config: ClientDomainConfig,
    stream: Option<Box<dyn AsyncReadAndWrite>>,
    tls_creds: Option<GetTlsCredsResponse>,
}

struct SshStream {
    stdin: FileDescriptor,
    stdout: FileDescriptor,
}

unsafe impl async_io::IoSafe for SshStream {}

impl std::fmt::Debug for SshStream {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "SshStream {{...}}")
    }
}

#[cfg(unix)]
impl AsFd for SshStream {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.stdout.as_fd()
    }
}

#[cfg(unix)]
impl AsRawFd for SshStream {
    fn as_raw_fd(&self) -> RawFd {
        self.stdout.as_raw_fd()
    }
}

#[cfg(windows)]
impl AsRawSocket for SshStream {
    fn as_raw_socket(&self) -> RawSocket {
        self.stdout.as_raw_socket()
    }
}

#[cfg(windows)]
impl AsSocket for SshStream {
    fn as_socket(&self) -> BorrowedSocket {
        self.stdout.as_socket()
    }
}

impl Read for SshStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.stdout.read(buf)
    }
}

impl Write for SshStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.stdin.write(buf)
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stdin.flush()
    }
}
