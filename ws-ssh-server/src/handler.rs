use crate::auth::AuthStore;
use async_trait::async_trait;
use portable_pty::MasterPty;
use russh::server::{Auth, Handler, Msg, Session};
use russh::{Channel, ChannelId, CryptoVec};
use russh_keys::key::PublicKey;
use std::io::Read;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct SshHandler {
    auth: Arc<AuthStore>,
    pty_writer: Option<Box<dyn std::io::Write + Send>>,
    pty_master: Option<Box<dyn MasterPty + Send>>,
    pty_child: Option<Box<dyn portable_pty::Child + Send + Sync>>,
    pty_cols: u16,
    pty_rows: u16,
    peer_addr: String,
}

impl SshHandler {
    pub fn new(auth: Arc<AuthStore>, peer_addr: String) -> Self {
        Self {
            auth,
            pty_writer: None,
            pty_master: None,
            pty_child: None,
            pty_cols: 80,
            pty_rows: 24,
            peer_addr,
        }
    }
}

#[async_trait]
impl Handler for SshHandler {
    type Error = anyhow::Error;

    async fn auth_password(
        &mut self,
        user: &str,
        password: &str,
    ) -> Result<Auth, Self::Error> {
        let ok = self.auth.verify_password(user, password);
        if ok {
            info!(peer = %self.peer_addr, user = %user, "Auth password accepted");
            Ok(Auth::Accept)
        } else {
            warn!(peer = %self.peer_addr, user = %user, "Auth password rejected");
            Ok(Auth::Reject {
                proceed_with_methods: None,
            })
        }
    }

    async fn auth_publickey(
        &mut self,
        user: &str,
        key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        let ok = self.auth.verify_pubkey(user, key);
        if ok {
            info!(peer = %self.peer_addr, user = %user, "Auth pubkey accepted");
            Ok(Auth::Accept)
        } else {
            warn!(peer = %self.peer_addr, user = %user, "Auth pubkey rejected");
            Ok(Auth::Reject {
                proceed_with_methods: None,
            })
        }
    }

    async fn channel_open_session(
        &mut self,
        _channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        info!(peer = %self.peer_addr, "Channel open session");
        Ok(true)
    }

    async fn pty_request(
        &mut self,
        _channel: ChannelId,
        _term: &str,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(russh::Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        self.pty_cols = col_width as u16;
        self.pty_rows = row_height as u16;
        info!(
            peer = %self.peer_addr,
            cols = self.pty_cols,
            rows = self.pty_rows,
            "PTY request"
        );
        session.request_success();
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!(peer = %self.peer_addr, "Shell request — spawning PTY");

        let (pty_session, reader) = crate::pty::PtySession::new(self.pty_cols, self.pty_rows)?;

        self.pty_master = Some(pty_session.master);
        self.pty_writer = Some(pty_session.writer);
        self.pty_child = Some(pty_session.child);

        let handle = session.handle();
        let peer = self.peer_addr.clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            let mut reader = reader;
            let mut buf = vec![0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = CryptoVec::from_slice(&buf[..n]);
                        if let Err(_) = rt.block_on(handle.data(channel, data)) {
                            error!(peer = %peer, "Failed to send data to SSH channel");
                            break;
                        }
                    }
                    Err(e) => {
                        error!(peer = %peer, "PTY read error: {e}");
                        break;
                    }
                }
            }
            info!(peer = %peer, "PTY reader loop ended, closing channel");
            let _ = rt.block_on(handle.eof(channel));
            let _ = rt.block_on(handle.close(channel));
        });

        session.request_success();
        Ok(())
    }

    async fn data(
        &mut self,
        _channel: ChannelId,
        data: &[u8],
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        if let Some(writer) = &mut self.pty_writer {
            use std::io::Write;
            if let Err(e) = writer.write_all(data) {
                error!(peer = %self.peer_addr, "Failed to write to PTY: {e}");
            }
        }
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        _channel: ChannelId,
        col_width: u32,
        row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let cols = col_width as u16;
        let rows = row_height as u16;
        info!(peer = %self.peer_addr, cols, rows, "Window change request");

        if let Some(master) = &self.pty_master {
            if let Err(e) = master.resize(portable_pty::PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            }) {
                error!(peer = %self.peer_addr, "Failed to resize PTY: {e}");
            }
        }

        session.request_success();
        Ok(())
    }

    async fn channel_close(
        &mut self,
        _channel: ChannelId,
        _session: &mut Session,
    ) -> Result<(), Self::Error> {
        info!(peer = %self.peer_addr, "Channel close");
        if let Some(child) = &mut self.pty_child {
            let _ = child.kill();
        }
        Ok(())
    }
}
