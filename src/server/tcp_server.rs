//! TCP server for handling client connections
//!
//! Async TCP server that handles authentication and command routing.

use super::auth::TokenAuth;
use super::channels::{ChannelManager, ConnectionId};
use super::commands::CommandHandler;
use super::protocol::{Protocol, Response};
use super::registry::Registry;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

pub struct TcpServer {
    addr: String,
    auth: Arc<TokenAuth>,
    registry: Arc<RwLock<Registry>>,
    channels: Arc<ChannelManager>,
    command_handler: Arc<CommandHandler>,
}

impl TcpServer {
    pub fn new(addr: String, auth: TokenAuth) -> Self {
        let auth = Arc::new(auth);
        let registry = Arc::new(RwLock::new(Registry::new()));
        let channels = Arc::new(ChannelManager::new());
        let command_handler = Arc::new(CommandHandler::new(registry.clone(), channels.clone()));

        Self {
            addr,
            auth,
            registry,
            channels,
            command_handler,
        }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(&self.addr).await?;
        log::info!("Server listening on {}", self.addr);
        log::info!("Token: {}", self.auth.token());
        log::info!("Token file: {:?}", self.auth.token_path());

        loop {
            let (socket, addr) = listener.accept().await?;
            log::debug!("New connection from: {}", addr);

            let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::SeqCst);
            let auth = self.auth.clone();
            let command_handler = self.command_handler.clone();
            let channels = self.channels.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, conn_id, auth, command_handler, channels).await {
                    log::error!("Connection {} error: {}", conn_id, e);
                }
            });
        }
    }

    async fn handle_connection(
        socket: TcpStream,
        conn_id: ConnectionId,
        auth: Arc<TokenAuth>,
        command_handler: Arc<CommandHandler>,
        channels: Arc<ChannelManager>,
    ) -> Result<(), std::io::Error> {
        let (reader, mut writer) = socket.into_split();
        let mut reader = BufReader::new(reader);
        let mut authenticated = false;
        let mut auth_attempts = 0;
        const MAX_AUTH_ATTEMPTS: u32 = 3;

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                // Connection closed
                log::debug!("Connection {} closed", conn_id);
                break;
            }

            // Parse command
            let (command, args) = match Protocol::parse_command(&line) {
                Ok(parsed) => parsed,
                Err(e) => {
                    let response = Response::Error(e);
                    writer.write_all(response.serialize().as_bytes()).await?;
                    continue;
                }
            };

            // Handle authentication
            if !authenticated {
                if command == "AUTH" {
                    if let Some(token) = args.get(0) {
                        if auth.validate(token) {
                            authenticated = true;
                            let response = Response::Ok;
                            writer.write_all(response.serialize().as_bytes()).await?;
                            log::info!("Connection {} authenticated", conn_id);
                        } else {
                            auth_attempts += 1;
                            let response = Response::Error("Invalid token".to_string());
                            writer.write_all(response.serialize().as_bytes()).await?;

                            if auth_attempts >= MAX_AUTH_ATTEMPTS {
                                log::warn!("Connection {} exceeded auth attempts", conn_id);
                                break;
                            }
                        }
                    } else {
                        let response = Response::Error("AUTH requires token".to_string());
                        writer.write_all(response.serialize().as_bytes()).await?;
                    }
                } else {
                    let response = Response::Error("Not authenticated. Use AUTH command first".to_string());
                    writer.write_all(response.serialize().as_bytes()).await?;
                }
                continue;
            }

            // Handle authenticated commands
            let response = command_handler.handle_command(&command, args, conn_id).await;
            writer.write_all(response.serialize().as_bytes()).await?;
        }

        // Cleanup: unsubscribe from all channels
        channels.unsubscribe_all(conn_id).await;
        log::debug!("Connection {} cleaned up", conn_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    async fn start_test_server() -> (TcpServer, String, String) {
        let auth = TokenAuth {
            token: "test_token_123".to_string(),
            token_path: std::path::PathBuf::new(),
        };
        let token = auth.token().to_string();
        let addr = "127.0.0.1:0".to_string(); // Port 0 = random available port
        let server = TcpServer::new(addr, auth);

        (server, token, "127.0.0.1:16379".to_string())
    }

    #[tokio::test]
    async fn test_authentication_required() {
        // This test would require a running server
        // For now, just test the protocol parsing
        let (cmd, args) = Protocol::parse_command("AUTH mytoken").unwrap();
        assert_eq!(cmd, "AUTH");
        assert_eq!(args[0], "mytoken");
    }
}
