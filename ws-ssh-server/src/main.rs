mod auth;
mod handler;
mod pty;
mod transport;

use anyhow::Result;
use auth::AuthStore;
use handler::SshHandler;
use russh_keys::key::KeyPair;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tracing::{error, info};
use transport::WsTransport;

fn load_or_generate_host_key() -> Result<KeyPair> {
    let path = "host_key";
    if std::path::Path::new(path).exists() {
        info!("Loading existing host key from {path}");
        let key = russh_keys::load_secret_key(path, None)?;
        Ok(key)
    } else {
        info!("Generating new Ed25519 host key and saving to {path}");
        let key = KeyPair::generate_ed25519().ok_or_else(|| anyhow::anyhow!("Failed to generate Ed25519 key"))?;
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

    let host_key = load_or_generate_host_key()?;
    let auth = Arc::new(AuthStore::new());

    let config = Arc::new(russh::server::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(300)),
        auth_rejection_time: std::time::Duration::from_secs(1),
        auth_rejection_time_initial: Some(std::time::Duration::from_secs(0)),
        keys: vec![host_key],
        ..Default::default()
    });

    let bind_addr = "0.0.0.0:8022";
    let listener = TcpListener::bind(bind_addr).await?;
    info!("SSH-over-WebSocket server listening on {bind_addr}");
    info!("Test with: websocat -b tcp-l:127.0.0.1:2222 ws://localhost:8022");
    info!("Then: ssh -o StrictHostKeyChecking=no -p 2222 admin@localhost");
    info!("Credentials: admin/secret123 or guest/guest");

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
    info!(%peer_addr, "Upgrading connection to WebSocket");
    let ws_stream = accept_async(stream).await?;
    info!(%peer_addr, "WebSocket handshake complete");

    let transport = WsTransport::new(ws_stream);
    let handler = SshHandler::new(auth, peer_addr.to_string());

    russh::server::run_stream(config, transport, handler).await?;

    info!(%peer_addr, "SSH session ended");
    Ok(())
}
