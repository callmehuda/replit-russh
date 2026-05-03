mod auth;
mod handler;
mod pty;

use anyhow::Result;
use auth::AuthStore;
use futures_util::{SinkExt, StreamExt};
use handler::SshHandler;
use russh_keys::key::KeyPair;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

fn load_or_generate_host_key() -> Result<KeyPair> {
    let path = "host_key";
    if std::path::Path::new(path).exists() {
        info!("Loading existing host key from {path}");
        let key = russh_keys::load_secret_key(path, None)?;
        Ok(key)
    } else {
        info!("Generating new Ed25519 host key and saving to {path}");
        let key = KeyPair::generate_ed25519()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate Ed25519 key"))?;
        let mut file = std::fs::File::create(path)?;
        russh_keys::encode_pkcs8_pem(&key, &mut file)?;
        Ok(key)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8000);

    let host_key = load_or_generate_host_key()?;
    let auth = Arc::new(AuthStore::new());

    let config = Arc::new(russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(300)),
        auth_rejection_time: std::time::Duration::from_secs(1),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![host_key],
        ..Default::default()
    });

    let bind_addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("SSH-over-WebSocket server listening on {bind_addr}");
    info!("WebSocket path: /ssh  (proxy: wss://<host>/ssh)");

    loop {
        match listener.accept().await {
            Ok((stream, peer_addr)) => {
                info!(%peer_addr, "New TCP connection");
                let config = config.clone();
                let auth = auth.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, peer_addr, config, auth).await {
                        error!(%peer_addr, "Connection error: {e:#}");
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {e}");
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    peer_addr: SocketAddr,
    config: Arc<russh::server::Config>,
    auth: Arc<AuthStore>,
) -> Result<()> {
    info!(%peer_addr, "Upgrading to WebSocket");
    let ws_stream = accept_async(stream).await?;
    info!(%peer_addr, "WebSocket handshake complete — starting SSH");

    // Split the WebSocket into independent sink and source
    let (mut ws_sink, mut ws_source) = ws_stream.split();

    // Create a duplex pipe: russh uses one end, bridge tasks use the other
    // This avoids concurrent mutable access on the WebSocketStream
    let (russh_io, bridge_io) = tokio::io::duplex(128 * 1024);
    let (mut bridge_reader, mut bridge_writer) = tokio::io::split(bridge_io);

    // Task A: WebSocket → russh (incoming SSH bytes from client)
    let ws_to_russh = tokio::spawn(async move {
        while let Some(msg_result) = ws_source.next().await {
            match msg_result {
                Ok(Message::Binary(data)) => {
                    if bridge_writer.write_all(&data).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Text(text)) => {
                    // Some WebSocket bridges send text frames — handle as raw bytes
                    if bridge_writer.write_all(text.as_bytes()).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Ping(payload)) => {
                    // Pings are handled by tungstenite automatically, just continue
                    let _ = payload;
                }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
        info!("ws→russh bridge ended");
    });

    // Task B: russh → WebSocket (outgoing SSH bytes to client)
    let russh_to_ws = tokio::spawn(async move {
        let mut buf = vec![0u8; 32 * 1024];
        loop {
            match bridge_reader.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let msg = Message::Binary(buf[..n].to_vec().into());
                    if ws_sink.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        }
        let _ = ws_sink.close().await;
        info!("russh→ws bridge ended");
    });

    // Run the SSH server on the clean duplex stream (no WebSocket involved)
    let handler = SshHandler::new(auth, peer_addr.to_string());
    let session = russh::server::run_stream(config, russh_io, handler).await;

    let session = match session {
        Ok(s) => s,
        Err(e) => {
            ws_to_russh.abort();
            russh_to_ws.abort();
            return Err(e.into());
        }
    };

    // Wait for the SSH session to complete
    match session.await {
        Ok(_handler) => info!(%peer_addr, "SSH session ended cleanly"),
        Err(e) => error!(%peer_addr, "SSH session error: {e:#}"),
    }

    // Abort bridge tasks now that SSH session is done
    ws_to_russh.abort();
    russh_to_ws.abort();

    Ok(())
}
