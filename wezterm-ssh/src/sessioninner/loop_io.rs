impl SessionInner {
    fn request_loop(&mut self, sess: &mut SessionWrap) -> anyhow::Result<()> {
        let mut sleep_delay = Duration::from_millis(100);

        loop {
            self.do_keepalive(sess)?;
            self.tick_io()?;
            self.drain_request_pipe();
            self.dispatch_pending_requests(sess)?;
            self.connect_pending_agent_forward_channels(sess);

            if self.channels.is_empty() && self.session_was_dropped {
                log::trace!(
                    "Stopping session loop as there are no more channels and Session was dropped"
                );
                return Ok(());
            }

            let mut poll_array = vec![
                pollfd {
                    fd: self.sender_read.as_socket_descriptor(),
                    events: POLLIN,
                    revents: 0,
                },
                pollfd {
                    fd: sess.as_socket_descriptor(),
                    events: sess.get_poll_flags(),
                    revents: 0,
                },
            ];
            let mut mapping = vec![];

            for info in self.channels.values() {
                for (fd_num, state) in info.descriptors.iter().enumerate() {
                    if let Some(fd) = state.fd.as_ref() {
                        poll_array.push(pollfd {
                            fd: fd.as_socket_descriptor(),
                            events: if fd_num == 0 {
                                POLLIN
                            } else if !state.buf.is_empty() || info.exited {
                                POLLOUT
                            } else {
                                0
                            },
                            revents: 0,
                        });
                        mapping.push((info.channel_id, fd_num));
                    }
                }
            }

            poll(&mut poll_array, Some(sleep_delay)).context("poll")?;
            sleep_delay += sleep_delay;

            for (idx, poll) in poll_array.iter().enumerate() {
                if poll.revents != 0 {
                    sleep_delay = Duration::from_millis(100);
                }
                if idx == 0 || idx == 1 {
                    // Dealt with at the top of the loop
                } else if poll.revents != 0 {
                    let (channel_id, fd_num) = mapping[idx - 2];
                    let info = self.channels.get_mut(&channel_id).unwrap();
                    let state = &mut info.descriptors[fd_num];
                    let fd = state.fd.as_mut().unwrap();

                    if fd_num == 0 {
                        // There's data we can read into the buffer
                        match read_into_buf(fd, &mut state.buf) {
                            Ok(_) => {}
                            Err(err) => {
                                log::debug!(
                                    "error reading from channel {channel_id} stdin pipe: {:#}",
                                    err
                                );
                                info.channel.close();
                                state.fd.take();
                            }
                        }
                    } else {
                        if info.exited && state.buf.is_empty() {
                            log::trace!("channel {channel_id} exited and we have no data to send to fd {fd_num}: close it!");
                            state.fd.take();
                        } else {
                            // We can write our buffered output
                            match write_from_buf(fd, &mut state.buf) {
                                Ok(_) => {}
                                Err(err) => {
                                    log::debug!(
                                        "error while writing to channel {} fd {}: {:#}",
                                        channel_id,
                                        fd_num,
                                        err
                                    );

                                    // Close it out
                                    state.fd.take();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Goal: if we have data to write to channels, try to send it.
    /// If we have room in our channel fd write buffers, try to fill it
    fn tick_io(&mut self) -> anyhow::Result<()> {
        let mut dead = vec![];
        for (id, chan) in self.channels.iter_mut() {
            if chan.exit.is_some() {
                if let Some(status) = chan.channel.exit_status() {
                    log::trace!("channel {id} has exit status {status:?}");
                    chan.exited = true;
                    let exit = chan.exit.take().unwrap();
                    smol::block_on(exit.send(status)).ok();
                }
            }

            let stdin = &mut chan.descriptors[0];
            if stdin.fd.is_some() && !stdin.buf.is_empty() {
                if let Err(err) = write_from_buf(&mut chan.channel.writer(), &mut stdin.buf)
                    .context("writing to channel")
                {
                    log::trace!(
                        "Failed to write data to channel {} stdin: {:#}, closing pipe",
                        id,
                        err
                    );
                    stdin.fd.take();
                }
            }

            for (idx, out) in chan
                .descriptors
                .get_mut(1..)
                .unwrap()
                .iter_mut()
                .enumerate()
            {
                if out.fd.is_none() {
                    continue;
                }
                let current_len = out.buf.len();
                let room = out.buf.capacity() - current_len;
                if room == 0 {
                    continue;
                }
                match read_into_buf(&mut chan.channel.reader(idx), &mut out.buf) {
                    Ok(_) => {}
                    Err(err) => {
                        if out.buf.is_empty() {
                            log::trace!(
                                "Failed to read data from channel {} stream {}: {:#}, closing pipe",
                                id,
                                idx,
                                err
                            );
                            out.fd.take();
                        } else {
                            log::trace!(
                                "Failed to read data from channel {} stream {}: {:#}, but \
                                         still have some buffer to drain",
                                id,
                                idx,
                                err
                            );
                        }
                    }
                }
            }

            if chan
                .descriptors
                .iter()
                .all(|descriptor| descriptor.fd.is_none())
            {
                log::trace!("all descriptors on channel {} are closed", id);
                dead.push(*id);
            }
        }
        for id in dead {
            self.channels.remove(&id);
        }
        Ok(())
    }

    fn drain_request_pipe(&mut self) {
        let mut buf = [0u8; 16];
        let _ = self.sender_read.read(&mut buf);
    }

    fn dispatch_pending_requests(&mut self, sess: &mut SessionWrap) -> anyhow::Result<()> {
        while self.dispatch_one_request(sess)? {}
        Ok(())
    }
}
