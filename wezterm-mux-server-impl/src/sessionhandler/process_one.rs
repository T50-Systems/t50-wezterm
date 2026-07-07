use super::*;

#[derive(Clone)]
pub(super) struct ResponseSender {
    sender: PduSender,
    serial: u64,
    start: Instant,
}

impl ResponseSender {
    fn new(sender: PduSender, serial: u64, start: Instant) -> Self {
        Self {
            sender,
            serial,
            start,
        }
    }

    pub(super) fn send(&self, result: anyhow::Result<Pdu>) {
        let pdu = match result {
            Ok(pdu) => pdu,
            Err(err) => Pdu::ErrorResponse(ErrorResponse {
                reason: format!("Error: {err:#}"),
            }),
        };
        log::trace!("{} processing time {:?}", self.serial, self.start.elapsed());
        self.sender
            .send(DecodedPdu {
                pdu,
                serial: self.serial,
            })
            .ok();
    }
}

pub(super) fn catch<F>(f: F, response: ResponseSender)
where
    F: FnOnce() -> anyhow::Result<Pdu>,
{
    response.send(f());
}

impl SessionHandler {
    pub fn process_one(&mut self, decoded: DecodedPdu) {
        let start = Instant::now();
        let response = ResponseSender::new(self.to_write_tx.clone(), decoded.serial, start);

        if let Some(client_id) = &self.client_id {
            if decoded.pdu.is_user_input() {
                Mux::get().client_had_input(client_id);
            }
        }

        match decoded.pdu {
            Pdu::Ping(Ping {}) => response.send(Ok(Pdu::Pong(Pong {}))),
            Pdu::SetClientId(SetClientId {
                mut client_id,
                is_proxy,
            }) => {
                if is_proxy {
                    if self.proxy_client_id.is_none() {
                        // Copy proxy identity, but don't assign it to the mux;
                        // we'll use it to annotate the actual clients own
                        // identity when they send it
                        self.proxy_client_id.replace(client_id);
                    }
                } else {
                    // If this session is a proxy, override the incoming id with
                    // the proxy information so that it is clear what is going
                    // on from the `wezterm cli list-clients` information
                    if let Some(proxy_id) = &self.proxy_client_id {
                        client_id.ssh_auth_sock = proxy_id.ssh_auth_sock.clone();
                        // Note that this `via proxy pid` string is coupled
                        // with the logic in mux/src/ssh_agent
                        client_id.hostname =
                            format!("{} (via proxy pid {})", client_id.hostname, proxy_id.pid);
                    }

                    let client_id = Arc::new(client_id);
                    self.client_id.replace(client_id.clone());
                    spawn_into_main_thread(async move {
                        let mux = Mux::get();
                        mux.register_client(client_id);
                    })
                    .detach();
                }
                response.send(Ok(Pdu::UnitResponse(UnitResponse {})))
            }
            pdu => {
                let pdu = match self.process_one_pane_request(pdu, response.clone()) {
                    Some(pdu) => pdu,
                    None => return,
                };
                self.process_one_misc_request(pdu, response);
            }
        }
    }
}
