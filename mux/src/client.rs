use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

pub use wezterm_mux_protocol::client::{ClientId, ClientInfo};

static CLIENT_ID: AtomicUsize = AtomicUsize::new(0);
lazy_static::lazy_static! {
    static ref EPOCH: u64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
}

pub fn new_client_id() -> ClientId {
    ClientId {
        hostname: hostname::get()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|_| "localhost".to_string()),
        username: config::username_from_env().unwrap_or_else(|_| "somebody".to_string()),
        pid: unsafe { libc::getpid() as u32 },
        epoch: *EPOCH,
        id: CLIENT_ID.fetch_add(1, Ordering::Relaxed),
        ssh_auth_sock: default_ssh_auth_sock(),
    }
}

fn default_ssh_auth_sock() -> Option<String> {
    match &config::configuration().default_ssh_auth_sock {
        Some(value) => Some(value.to_string()),
        None => std::env::var("SSH_AUTH_SOCK").ok(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn generated_client_ids_preserve_process_identity() {
        let first = super::new_client_id();
        let second = super::new_client_id();

        assert_eq!(first.pid, std::process::id());
        assert_eq!(second.pid, first.pid);
        assert_eq!(second.epoch, first.epoch);
        assert!(second.id > first.id);
    }
}
