use assert_fs::prelude::*;
use assert_fs::TempDir;
use rstest::*;
use std::collections::HashMap;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::LazyLock;
use std::time::Duration;
use wezterm_ssh::{Config, Session, SessionEvent};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// NOTE: OpenSSH's sshd requires absolute path
const BIN_PATH_STR: &str = "/usr/sbin/sshd";

pub fn sshd_available() -> bool {
    Path::new(BIN_PATH_STR).exists()
}

/// Ask the kernel to assign a free port.
/// We pass this to sshd and tell it to listen on that port.
/// This is racy, as releasing the socket technically makes
/// that port available to others using the same technique.
fn allocate_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0 failed");
    listener.local_addr().unwrap().port()
}

const USERNAME: LazyLock<String> = LazyLock::new(whoami::username);

pub struct SshKeygen;

impl SshKeygen {
    // ssh-keygen -t rsa -f $ROOT/id_rsa -N "" -q
    pub fn generate_rsa(path: impl AsRef<Path>, passphrase: impl AsRef<str>) -> IoResult<bool> {
        let res = Command::new("ssh-keygen")
            .args(&["-m", "PEM"])
            .args(&["-t", "rsa"])
            .arg("-f")
            .arg(path.as_ref())
            .arg("-N")
            .arg(passphrase.as_ref())
            .arg("-q")
            .status()
            .map(|status| status.success())?;

        #[cfg(unix)]
        if res {
            // chmod 600 id_rsa* -> ida_rsa + ida_rsa.pub
            std::fs::metadata(path.as_ref().with_extension("pub"))?
                .permissions()
                .set_mode(0o600);
            std::fs::metadata(path)?.permissions().set_mode(0o600);
        }

        Ok(res)
    }
}

pub struct SshAgent {
    child: Child,
}

impl Drop for SshAgent {
    fn drop(&mut self) {
        self.child.kill().ok();
    }
}

impl SshAgent {
    fn with_sock(path: &Path) -> IoResult<Self> {
        let child = Command::new("ssh-agent")
            .arg("-a")
            .arg(path)
            .arg("-D")
            .spawn()?;
        Ok(Self { child })
    }
}
