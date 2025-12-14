//! Command handlers
//!
//! Implements handlers for all Redis-like commands.

use super::channels::{ChannelManager, ConnectionId};
use super::protocol::Response;
use super::registry::Registry;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct CommandHandler {
    registry: Arc<RwLock<Registry>>,
    channels: Arc<ChannelManager>,
}

impl CommandHandler {
    pub fn new(registry: Arc<RwLock<Registry>>, channels: Arc<ChannelManager>) -> Self {
        Self { registry, channels }
    }

    pub async fn handle_command(
        &self,
        command: &str,
        args: Vec<String>,
        conn_id: ConnectionId,
    ) -> Response {
        match command {
            // Session management
            "LIST" if args.get(0).map(|s| s.as_str()) == Some("SESSIONS") => {
                self.handle_list_sessions().await
            }
            "LIST" if args.get(0).map(|s| s.as_str()) == Some("PANES") => {
                if let Some(session_id) = args.get(1) {
                    self.handle_list_panes(session_id).await
                } else {
                    Response::Error("LIST PANES requires session_id".to_string())
                }
            }
            "CREATE" if args.get(0).map(|s| s.as_str()) == Some("SESSION") => {
                let name = args.get(1).map(|s| s.to_string());
                let pane_name = args.get(2).map(|s| s.to_string());
                self.handle_create_session(name, pane_name).await
            }
            "CREATE" if args.get(0).map(|s| s.as_str()) == Some("PANE") => {
                if let Some(session_id) = args.get(1) {
                    let name = args.get(2).map(|s| s.to_string());
                    self.handle_create_pane(session_id, name).await
                } else {
                    Response::Error("CREATE PANE requires session_id".to_string())
                }
            }
            "CLOSE" if args.get(0).map(|s| s.as_str()) == Some("PANE") => {
                if let (Some(session_id), Some(pane_id)) = (args.get(1), args.get(2)) {
                    self.handle_close_pane(session_id, pane_id).await
                } else {
                    Response::Error("CLOSE PANE requires session_id and pane_id".to_string())
                }
            }
            "CLOSE" if args.get(0).map(|s| s.as_str()) == Some("SESSION") => {
                if let Some(session_id) = args.get(1) {
                    self.handle_close_session(session_id).await
                } else {
                    Response::Error("CLOSE SESSION requires session_id".to_string())
                }
            }

            // Channel operations
            "SUBSCRIBE" => {
                if let Some(channel) = args.get(0) {
                    self.handle_subscribe(channel, conn_id).await
                } else {
                    Response::Error("SUBSCRIBE requires channel name".to_string())
                }
            }
            "UNSUBSCRIBE" => {
                if let Some(channel) = args.get(0) {
                    self.handle_unsubscribe(channel, conn_id).await
                } else {
                    Response::Error("UNSUBSCRIBE requires channel name".to_string())
                }
            }
            "PUBLISH" => {
                if let (Some(channel), Some(message)) = (args.get(0), args.get(1)) {
                    let content = args[1..].join(" ");
                    self.handle_publish(channel, &content).await
                } else {
                    Response::Error("PUBLISH requires channel and message".to_string())
                }
            }

            // Command injection
            "INJECT" => {
                if let (Some(target), Some(command)) = (args.get(0), args.get(1)) {
                    let cmd = args[1..].join(" ");
                    let mode = args.last().and_then(|m| {
                        if m == "NOWAIT" || m == "QUEUE" || m == "BATCH" {
                            Some(m.as_str())
                        } else {
                            None
                        }
                    });
                    self.handle_inject(target, &cmd, mode).await
                } else {
                    Response::Error("INJECT requires target and command".to_string())
                }
            }

            // Screen capture
            "CAPTURE" => {
                if let Some(target) = args.get(0) {
                    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("FULL");
                    self.handle_capture(target, mode).await
                } else {
                    Response::Error("CAPTURE requires target".to_string())
                }
            }

            // Queue operations
            "LLEN" => {
                if let Some(channel) = args.get(0) {
                    self.handle_llen(channel).await
                } else {
                    Response::Error("LLEN requires channel name".to_string())
                }
            }
            "RPOP" => {
                if let Some(channel) = args.get(0) {
                    self.handle_rpop(channel).await
                } else {
                    Response::Error("RPOP requires channel name".to_string())
                }
            }

            _ => Response::Error(format!("Unknown command: {}", command)),
        }
    }

    async fn handle_list_sessions(&self) -> Response {
        let registry = self.registry.read().await;
        let sessions = registry.list_sessions();
        Response::Array(sessions)
    }

    async fn handle_list_panes(&self, session_id: &str) -> Response {
        let registry = self.registry.read().await;
        match registry.list_panes(session_id) {
            Some(panes) => Response::Array(panes),
            None => Response::Error(format!("Session '{}' not found", session_id)),
        }
    }

    async fn handle_create_session(&self, name: Option<String>, pane_name: Option<String>) -> Response {
        let mut registry = self.registry.write().await;
        match registry.create_session(name) {
            Ok(session_id) => {
                // Create first pane
                match registry.create_pane(&session_id, pane_name) {
                    Ok(pane_id) => {
                        Response::OkWithData(format!("session-id:{} pane-id:{}", session_id, pane_id))
                    }
                    Err(e) => Response::Error(e),
                }
            }
            Err(e) => Response::Error(e),
        }
    }

    async fn handle_create_pane(&self, session_id: &str, name: Option<String>) -> Response {
        let mut registry = self.registry.write().await;
        match registry.create_pane(session_id, name) {
            Ok(pane_id) => Response::OkWithData(format!("pane-id:{}", pane_id)),
            Err(e) => Response::Error(e),
        }
    }

    async fn handle_close_pane(&self, session_id: &str, pane_id: &str) -> Response {
        let mut registry = self.registry.write().await;
        match registry.remove_pane(session_id, pane_id) {
            Ok(_) => Response::Ok,
            Err(e) => Response::Error(e),
        }
    }

    async fn handle_close_session(&self, session_id: &str) -> Response {
        let mut registry = self.registry.write().await;
        match registry.remove_session(session_id) {
            Ok(_) => Response::Ok,
            Err(e) => Response::Error(e),
        }
    }

    async fn handle_subscribe(&self, channel: &str, conn_id: ConnectionId) -> Response {
        self.channels.subscribe(channel, conn_id).await;
        Response::Ok
    }

    async fn handle_unsubscribe(&self, channel: &str, conn_id: ConnectionId) -> Response {
        self.channels.unsubscribe(channel, conn_id).await;
        Response::Ok
    }

    async fn handle_publish(&self, channel: &str, content: &str) -> Response {
        let count = self.channels.publish(channel, content.to_string()).await;
        Response::OkWithData(format!("published to {} subscribers", count))
    }

    async fn handle_inject(&self, target: &str, command: &str, mode: Option<&str>) -> Response {
        // Parse target (session-id/pane-id)
        let parts: Vec<&str> = target.split('/').collect();
        if parts.len() != 2 {
            return Response::Error("Invalid target format. Use: session-id/pane-id".to_string());
        }

        let channel = format!("{}/input", target);
        let message = format!("{}\\n", command); // Auto-append newline

        self.channels.publish(&channel, message).await;
        Response::Ok
    }

    async fn handle_capture(&self, target: &str, mode: &str) -> Response {
        // Request capture from terminal via channel
        let channel = format!("{}/capture-request", target);
        let request = json!({
            "mode": mode,
        });

        self.channels.publish(&channel, request.to_string()).await;

        // In real implementation, would wait for response on capture-response channel
        // For now, return placeholder
        Response::Json(json!({
            "session": target.split('/').next().unwrap_or(""),
            "pane": target.split('/').nth(1).unwrap_or(""),
            "mode": mode,
            "status": "requested"
        }))
    }

    async fn handle_llen(&self, channel: &str) -> Response {
        let len = self.channels.queue_length(channel).await;
        Response::String(len.to_string())
    }

    async fn handle_rpop(&self, channel: &str) -> Response {
        match self.channels.pop_message(channel).await {
            Some(msg) => Response::String(msg.content),
            None => Response::String("(nil)".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_sessions() {
        let registry = Arc::new(RwLock::new(Registry::new()));
        let channels = Arc::new(ChannelManager::new());
        let handler = CommandHandler::new(registry.clone(), channels);

        // Create a session
        {
            let mut reg = registry.write().await;
            reg.create_session(Some("test-session".to_string())).unwrap();
        }

        let response = handler.handle_command("LIST", vec!["SESSIONS".to_string()], 1).await;
        match response {
            Response::Array(sessions) => {
                assert_eq!(sessions.len(), 1);
                assert_eq!(sessions[0], "test-session");
            }
            _ => panic!("Expected Array response"),
        }
    }

    #[tokio::test]
    async fn test_create_session() {
        let registry = Arc::new(RwLock::new(Registry::new()));
        let channels = Arc::new(ChannelManager::new());
        let handler = CommandHandler::new(registry, channels);

        let response = handler.handle_command(
            "CREATE",
            vec!["SESSION".to_string(), "my-session".to_string()],
            1
        ).await;

        match response {
            Response::OkWithData(data) => {
                assert!(data.contains("session-id:my-session"));
            }
            _ => panic!("Expected OkWithData response"),
        }
    }

    #[tokio::test]
    async fn test_subscribe_publish() {
        let registry = Arc::new(RwLock::new(Registry::new()));
        let channels = Arc::new(ChannelManager::new());
        let handler = CommandHandler::new(registry, channels.clone());

        // Subscribe
        handler.handle_command("SUBSCRIBE", vec!["test-channel".to_string()], 1).await;

        // Publish
        let response = handler.handle_command(
            "PUBLISH",
            vec!["test-channel".to_string(), "Hello".to_string()],
            1
        ).await;

        match response {
            Response::OkWithData(data) => {
                assert!(data.contains("1 subscribers"));
            }
            _ => panic!("Expected OkWithData response"),
        }
    }
}
