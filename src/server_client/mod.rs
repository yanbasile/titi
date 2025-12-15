//! Client for connecting to redititi server
//!
//! This module provides a client for Titi terminals to connect to the redititi
//! automation server, enabling command injection and screen capture.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Client for connecting to redititi server
pub struct ServerClient {
    reader: Arc<RwLock<BufReader<tokio::net::tcp::OwnedReadHalf>>>,
    writer: Arc<RwLock<tokio::net::tcp::OwnedWriteHalf>>,
    session_id: String,
    pane_id: String,
    authenticated: bool,
}

impl ServerClient {
    /// Connect to redititi server
    pub async fn connect(addr: &str) -> Result<Self, String> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);

        Ok(Self {
            reader: Arc::new(RwLock::new(reader)),
            writer: Arc::new(RwLock::new(write_half)),
            session_id: String::new(),
            pane_id: String::new(),
            authenticated: false,
        })
    }

    /// Authenticate with token
    pub async fn authenticate(&mut self, token: &str) -> Result<(), String> {
        self.send_command(&format!("AUTH {}", token)).await?;
        let response = self.read_response().await?;

        if response.starts_with("+OK") {
            self.authenticated = true;
            Ok(())
        } else {
            Err(format!("Authentication failed: {}", response))
        }
    }

    /// Create or join session
    pub async fn create_session(&mut self, name: Option<&str>) -> Result<String, String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let cmd = if let Some(n) = name {
            format!("CREATE SESSION {}", n)
        } else {
            "CREATE SESSION".to_string()
        };

        self.send_command(&cmd).await?;
        let response = self.read_response().await?;

        if let Some(data) = response.strip_prefix("+OK ") {
            // Parse response: "session-id:xxx pane-id:yyy"
            let parts: Vec<&str> = data.trim().split_whitespace().collect();
            for part in parts {
                if let Some(id) = part.strip_prefix("session-id:") {
                    self.session_id = id.to_string();
                } else if let Some(id) = part.strip_prefix("pane-id:") {
                    self.pane_id = id.to_string();
                }
            }
            Ok(self.session_id.clone())
        } else {
            Err(format!("Failed to create session: {}", response))
        }
    }

    /// Create pane in current session
    pub async fn create_pane(&mut self, name: Option<&str>) -> Result<String, String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        if self.session_id.is_empty() {
            return Err("No session created".to_string());
        }

        let cmd = if let Some(n) = name {
            format!("CREATE PANE {} {}", self.session_id, n)
        } else {
            format!("CREATE PANE {}", self.session_id)
        };

        self.send_command(&cmd).await?;
        let response = self.read_response().await?;

        if let Some(data) = response.strip_prefix("+OK ") {
            // Parse response: "pane-id:xxx"
            if let Some(id) = data.trim().strip_prefix("pane-id:") {
                self.pane_id = id.to_string();
                Ok(self.pane_id.clone())
            } else {
                Err(format!("Invalid pane response format: {}", data))
            }
        } else {
            Err(format!("Failed to create pane: {}", response))
        }
    }

    /// Subscribe to input channel
    pub async fn subscribe_input(&mut self) -> Result<(), String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let channel = format!("{}/pane-{}/input", self.session_id, self.pane_id);
        self.send_command(&format!("SUBSCRIBE {}", channel)).await?;
        let response = self.read_response().await?;

        if response.starts_with("+OK") {
            Ok(())
        } else {
            Err(format!("Failed to subscribe to input: {}", response))
        }
    }

    /// Publish output to channel
    pub async fn publish_output(&self, data: &str) -> Result<(), String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let channel = format!("{}/pane-{}/output", self.session_id, self.pane_id);
        let cmd = format!("PUBLISH {} {}", channel, data);
        self.send_command(&cmd).await?;

        // Don't wait for response for publish (fire and forget for performance)
        Ok(())
    }

    /// Publish to arbitrary channel (for testing and advanced use cases)
    pub async fn publish_to_channel(&self, channel: &str, data: &str) -> Result<(), String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let cmd = format!("PUBLISH {} {}", channel, data);
        self.send_command(&cmd).await?;

        // Don't wait for response for publish (fire and forget for performance)
        Ok(())
    }

    /// Read message from input channel (non-blocking)
    pub async fn read_input(&mut self) -> Result<Option<String>, String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let channel = format!("{}/pane-{}/input", self.session_id, self.pane_id);
        self.send_command(&format!("RPOP {}", channel)).await?;
        let response = self.read_response().await?;

        if response.starts_with("-ERR") {
            Ok(None)  // Queue empty
        } else if let Some(msg) = response.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            Ok(Some(msg.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Inject command into a terminal (for external clients controlling terminals)
    pub async fn inject_command(&self, session_id: &str, pane_id: &str, command: &str) -> Result<(), String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let target = format!("{}/pane-{}", session_id, pane_id);
        let cmd = format!("INJECT {} {}", target, command);
        self.send_command(&cmd).await?;
        let response = self.read_response().await?;

        if response.starts_with("+OK") {
            Ok(())
        } else {
            Err(format!("Failed to inject command: {}", response))
        }
    }

    /// Subscribe to output channel to read terminal output
    pub async fn subscribe_output(&mut self) -> Result<(), String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let channel = format!("{}/pane-{}/output", self.session_id, self.pane_id);
        self.send_command(&format!("SUBSCRIBE {}", channel)).await?;
        let response = self.read_response().await?;

        if response.starts_with("+OK") {
            Ok(())
        } else {
            Err(format!("Failed to subscribe to output: {}", response))
        }
    }

    /// Read message from output channel (non-blocking)
    pub async fn read_output(&mut self) -> Result<Option<String>, String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let channel = format!("{}/pane-{}/output", self.session_id, self.pane_id);
        self.send_command(&format!("RPOP {}", channel)).await?;
        let response = self.read_response().await?;

        if response.starts_with("-ERR") {
            Ok(None)  // Queue empty
        } else if let Some(msg) = response.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            Ok(Some(msg.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get pane ID
    pub fn pane_id(&self) -> &str {
        &self.pane_id
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }

    /// Read from a specific channel (for monitoring other sessions)
    pub async fn read_from_channel(&self, session_id: &str, pane_id: &str, channel_type: &str) -> Result<Option<String>, String> {
        if !self.authenticated {
            return Err("Not authenticated".to_string());
        }

        let channel = format!("{}/pane-{}/{}", session_id, pane_id, channel_type);
        self.send_command(&format!("RPOP {}", channel)).await?;
        let response = self.read_response().await?;

        if response.starts_with("-ERR") {
            Ok(None)  // Queue empty
        } else if let Some(msg) = response.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            Ok(Some(msg.to_string()))
        } else {
            Ok(None)
        }
    }

    // Helper methods
    async fn send_command(&self, cmd: &str) -> Result<(), String> {
        let mut writer = self.writer.write().await;
        writer
            .write_all(cmd.as_bytes())
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| format!("Failed to send newline: {}", e))?;
        Ok(())
    }

    async fn read_response(&self) -> Result<String, String> {
        let mut reader = self.reader.write().await;
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        Ok(line.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running redititi server
    // For now, we test the API structure

    #[test]
    fn test_server_client_creation() {
        // Test that ServerClient can be constructed
        // Actual connection test requires async runtime and server
    }

    #[test]
    fn test_session_pane_ids() {
        // We can test the struct fields
        // but full functionality requires integration tests
    }

    // TODO: Add integration tests that start a test redititi server
}
