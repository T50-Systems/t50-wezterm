impl SessionInner {
    fn dispatch_one_request(&mut self, sess: &mut SessionWrap) -> anyhow::Result<bool> {
        match self.rx_req.try_recv() {
            Err(TryRecvError::Closed) => anyhow::bail!("all clients are closed"),
            Err(TryRecvError::Empty) => Ok(false),
            Ok(req) => {
                sess.set_blocking(true);
                let res = match req {
                    SessionRequest::SessionDropped => {
                        self.session_was_dropped = true;
                        Ok(true)
                    }
                    SessionRequest::NewPty(newpty, reply) => {
                        dispatch(reply, || self.new_pty(sess, newpty), "NewPty")
                    }
                    SessionRequest::ResizePty(resize, Some(reply)) => {
                        dispatch(reply, || self.resize_pty(resize), "resize_pty")
                    }
                    SessionRequest::ResizePty(resize, None) => {
                        if let Err(err) = self.resize_pty(resize) {
                            log::error!("error in resize_pty: {:#}", err);
                        }
                        Ok(true)
                    }
                    SessionRequest::Exec(exec, reply) => {
                        dispatch(reply, || self.exec(sess, exec), "exec")
                    }
                    SessionRequest::SignalChannel(info) => {
                        if let Err(err) = self.signal_channel(&info) {
                            log::error!("{:?} -> error: {:#}", info, err);
                        }
                        Ok(true)
                    }
                    SessionRequest::Sftp(SftpRequest::OpenWithMode(msg, reply)) => {
                        dispatch(reply, || self.open_with_mode(sess, &msg), "OpenWithMode")
                    }
                    SessionRequest::Sftp(SftpRequest::OpenDir(path, reply)) => {
                        dispatch(reply, || self.open_dir(sess, path), "OpenDir")
                    }
                    SessionRequest::Sftp(SftpRequest::File(FileRequest::Write(msg, reply))) => {
                        dispatch(
                            reply,
                            || {
                                let file = self
                                    .files
                                    .get_mut(&msg.file_id)
                                    .ok_or_else(|| anyhow!("invalid file_id"))?;
                                file.writer().write_all(&msg.data)?;
                                Ok(())
                            },
                            "write_file",
                        )
                    }
                    SessionRequest::Sftp(SftpRequest::File(FileRequest::Read(msg, reply))) => {
                        dispatch(
                            reply,
                            || {
                                let file = self
                                    .files
                                    .get_mut(&msg.file_id)
                                    .ok_or_else(|| anyhow!("invalid file_id"))?;

                                // TODO: Move this somewhere to avoid re-allocating buffer
                                let mut buf = vec![0u8; msg.max_bytes];
                                let n = file.reader().read(&mut buf)?;
                                buf.truncate(n);
                                Ok(buf)
                            },
                            "read_file",
                        )
                    }
                    SessionRequest::Sftp(SftpRequest::File(FileRequest::Close(file_id, reply))) => {
                        dispatch(
                            reply,
                            || {
                                self.files.remove(&file_id);
                                Ok(())
                            },
                            "close_file",
                        )
                    }
                    SessionRequest::Sftp(SftpRequest::Dir(DirRequest::Close(dir_id, reply))) => {
                        dispatch(
                            reply,
                            || {
                                self.dirs
                                    .remove(&dir_id)
                                    .ok_or_else(|| anyhow!("invalid dir_id"))?;
                                Ok(())
                            },
                            "close_dir",
                        )
                    }
                    SessionRequest::Sftp(SftpRequest::Dir(DirRequest::ReadDir(dir_id, reply))) => {
                        dispatch(
                            reply,
                            || {
                                let dir = self
                                    .dirs
                                    .get_mut(&dir_id)
                                    .ok_or_else(|| anyhow!("invalid dir_id"))?;
                                dir.read_dir()
                            },
                            "read_dir",
                        )
                    }
                    SessionRequest::Sftp(SftpRequest::File(FileRequest::Flush(file_id, reply))) => {
                        dispatch(
                            reply,
                            || {
                                let file = self
                                    .files
                                    .get_mut(&file_id)
                                    .ok_or_else(|| anyhow!("invalid file_id"))?;
                                file.writer().flush()?;
                                Ok(())
                            },
                            "flush_file",
                        )
                    }
                    SessionRequest::Sftp(SftpRequest::File(FileRequest::SetMetadata(
                        msg,
                        reply,
                    ))) => dispatch(
                        reply,
                        || {
                            let file = self
                                .files
                                .get_mut(&msg.file_id)
                                .ok_or_else(|| anyhow!("invalid file_id"))?;
                            file.set_metadata(msg.metadata)
                        },
                        "set_metadata_file",
                    ),
                    SessionRequest::Sftp(SftpRequest::File(FileRequest::Metadata(
                        file_id,
                        reply,
                    ))) => dispatch(
                        reply,
                        || {
                            let file = self
                                .files
                                .get_mut(&file_id)
                                .ok_or_else(|| anyhow!("invalid file_id"))?;
                            file.metadata()
                        },
                        "metadata_file",
                    ),
                    SessionRequest::Sftp(SftpRequest::File(FileRequest::Fsync(file_id, reply))) => {
                        dispatch(
                            reply,
                            || {
                                let file = self
                                    .files
                                    .get_mut(&file_id)
                                    .ok_or_else(|| anyhow!("invalid file_id"))?;
                                file.fsync()
                            },
                            "fsync",
                        )
                    }

                    SessionRequest::Sftp(SftpRequest::ReadDir(path, reply)) => {
                        dispatch(reply, || self.init_sftp(sess)?.read_dir(&path), "read_dir")
                    }
                    SessionRequest::Sftp(SftpRequest::CreateDir(msg, reply)) => dispatch(
                        reply,
                        || self.init_sftp(sess)?.create_dir(&msg.filename, msg.mode),
                        "create_dir",
                    ),
                    SessionRequest::Sftp(SftpRequest::RemoveDir(path, reply)) => dispatch(
                        reply,
                        || self.init_sftp(sess)?.remove_dir(&path),
                        "remove_dir",
                    ),
                    SessionRequest::Sftp(SftpRequest::Metadata(path, reply)) => {
                        dispatch(reply, || self.init_sftp(sess)?.metadata(&path), "metadata")
                    }
                    SessionRequest::Sftp(SftpRequest::SymlinkMetadata(path, reply)) => dispatch(
                        reply,
                        || self.init_sftp(sess)?.symlink_metadata(&path),
                        "symlink_metadata",
                    ),
                    SessionRequest::Sftp(SftpRequest::SetMetadata(msg, reply)) => dispatch(
                        reply,
                        || {
                            self.init_sftp(sess)?
                                .set_metadata(&msg.filename, msg.metadata)
                        },
                        "set_metadata",
                    ),
                    SessionRequest::Sftp(SftpRequest::Symlink(msg, reply)) => dispatch(
                        reply,
                        || self.init_sftp(sess)?.symlink(&msg.path, &msg.target),
                        "symlink",
                    ),
                    SessionRequest::Sftp(SftpRequest::ReadLink(path, reply)) => dispatch(
                        reply,
                        || self.init_sftp(sess)?.read_link(&path),
                        "read_link",
                    ),
                    SessionRequest::Sftp(SftpRequest::Canonicalize(path, reply)) => dispatch(
                        reply,
                        || self.init_sftp(sess)?.canonicalize(&path),
                        "canonicalize",
                    ),
                    SessionRequest::Sftp(SftpRequest::Rename(msg, reply)) => dispatch(
                        reply,
                        || self.init_sftp(sess)?.rename(&msg.src, &msg.dst, msg.opts),
                        "rename",
                    ),
                    SessionRequest::Sftp(SftpRequest::RemoveFile(path, reply)) => {
                        dispatch(reply, || self.init_sftp(sess)?.unlink(&path), "remove_file")
                    }
                };
                sess.set_blocking(false);
                res
            }
        }
    }

    fn connect_pending_agent_forward_channels(&mut self, sess: &mut SessionWrap) {
        fn process_one(sess: &mut SessionInner, channel: ChannelWrap) -> anyhow::Result<()> {
            let identity_agent = sess
                .identity_agent()
                .ok_or_else(|| anyhow!("no identity agent in config"))?;
            let mut fd = {
                use wezterm_uds::UnixStream;
                #[cfg(unix)]
                {
                    FileDescriptor::new(UnixStream::connect(&identity_agent)?)
                }
                #[cfg(windows)]
                unsafe {
                    use std::os::windows::io::{FromRawSocket, IntoRawSocket};
                    FileDescriptor::from_raw_socket(
                        UnixStream::connect(&identity_agent)?.into_raw_socket(),
                    )
                }
            };
            fd.set_non_blocking(true)?;

            let read_from_agent = fd;
            let write_to_agent = read_from_agent.try_clone()?;
            let channel_id = sess.next_channel_id;
            sess.next_channel_id += 1;
            let info = ChannelInfo {
                channel_id,
                channel,
                exit: None,
                exited: false,
                descriptors: [
                    DescriptorState {
                        fd: Some(read_from_agent),
                        buf: VecDeque::with_capacity(8192),
                    },
                    DescriptorState {
                        fd: Some(write_to_agent),
                        buf: VecDeque::with_capacity(8192),
                    },
                    DescriptorState {
                        fd: None,
                        buf: VecDeque::with_capacity(8192),
                    },
                ],
            };
            sess.channels.insert(channel_id, info);
            Ok(())
        }
        while let Some(channel) = sess.accept_agent_forward() {
            if let Err(err) = process_one(self, channel) {
                log::error!("error connecting agent forward: {:#}", err);
            }
        }
    }
}
