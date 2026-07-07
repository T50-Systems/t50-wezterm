#[derive(Debug)]
struct KillerInner {
    killer: Option<Box<dyn ChildKiller + Send + Sync>>,
    /// If we haven't populated `killer` by the time someone has called
    /// `kill`, then we use this to remember to kill as soon as we recv
    /// the child process.
    pending_kill: bool,
}

#[derive(Debug, Clone)]
struct WrappedSshChildKiller {
    inner: Arc<Mutex<KillerInner>>,
}

#[derive(Debug)]
pub(crate) struct WrappedSshChild {
    status: Option<AsyncReceiver<ExitStatus>>,
    rx: Receiver<SshChildProcess>,
    exited: Option<ExitStatus>,
    killer: WrappedSshChildKiller,
}

impl WrappedSshChild {
    fn check_connected(&mut self) {
        if self.status.is_none() {
            match self.rx.try_recv() {
                Ok(c) => {
                    self.got_child(c);
                }
                Err(TryRecvError::Empty) => {}
                Err(err) => {
                    log::debug!("WrappedSshChild::check_connected err: {:#?}", err);
                    self.exited.replace(ExitStatus::with_exit_code(1));
                }
            }
        }
    }

    fn got_child(&mut self, mut child: SshChildProcess) {
        {
            let mut killer = self.killer.inner.lock().unwrap();
            killer.killer.replace(child.clone_killer());
            if killer.pending_kill {
                let _ = child.kill().ok();
            }
        }

        let (tx, rx) = bounded(1);
        promise::spawn::spawn_into_main_thread(async move {
            if let Ok(status) = child.async_wait().await {
                tx.send(status).await.ok();
                let mux = Mux::get();
                mux.prune_dead_windows();
            }
        })
        .detach();
        self.status.replace(rx);
    }
}

impl portable_pty::Child for WrappedSshChild {
    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        if let Some(status) = self.exited.as_ref() {
            return Ok(Some(status.clone()));
        }

        self.check_connected();

        if let Some(rx) = self.status.as_mut() {
            match rx.try_recv() {
                Ok(status) => {
                    self.exited.replace(status.clone());
                    Ok(Some(status))
                }
                Err(smol::channel::TryRecvError::Empty) => Ok(None),
                Err(err) => {
                    log::debug!("WrappedSshChild::try_wait err: {:#?}", err);
                    let status = ExitStatus::with_exit_code(1);
                    self.exited.replace(status.clone());
                    Ok(Some(status))
                }
            }
        } else {
            Ok(None)
        }
    }

    fn wait(&mut self) -> std::io::Result<portable_pty::ExitStatus> {
        if let Some(status) = self.exited.as_ref() {
            return Ok(status.clone());
        }

        if self.status.is_none() {
            match smol::block_on(async { self.rx.recv() }) {
                Ok(c) => {
                    self.got_child(c);
                }
                Err(err) => {
                    log::debug!("WrappedSshChild err: {:#?}", err);
                    let status = ExitStatus::with_exit_code(1);
                    self.exited.replace(status.clone());
                    return Ok(status);
                }
            }
        }

        let rx = self.status.as_mut().unwrap();
        match smol::block_on(rx.recv()) {
            Ok(status) => {
                self.exited.replace(status.clone());
                Ok(status)
            }
            Err(err) => {
                log::error!("WrappedSshChild err: {:#?}", err);
                let status = ExitStatus::with_exit_code(1);
                self.exited.replace(status.clone());
                Ok(status)
            }
        }
    }

    fn process_id(&self) -> Option<u32> {
        None
    }

    #[cfg(windows)]
    fn as_raw_handle(&self) -> Option<std::os::windows::io::RawHandle> {
        None
    }
}

impl ChildKiller for WrappedSshChild {
    fn kill(&mut self) -> std::io::Result<()> {
        let mut killer = self.killer.inner.lock().unwrap();
        if let Some(killer) = killer.killer.as_mut() {
            killer.kill()
        } else {
            killer.pending_kill = true;
            Ok(())
        }
    }

    fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
        Box::new(self.killer.clone())
    }
}

impl ChildKiller for WrappedSshChildKiller {
    fn kill(&mut self) -> std::io::Result<()> {
        let mut killer = self.inner.lock().unwrap();
        if let Some(killer) = killer.killer.as_mut() {
            killer.kill()
        } else {
            killer.pending_kill = true;
            Ok(())
        }
    }

    fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
        Box::new(self.clone())
    }
}
