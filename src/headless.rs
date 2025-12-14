//! Headless mode for running terminals without GPU rendering
//!
//! This module provides a headless runtime for Titi that runs without
//! GPU rendering, designed for automation, CI/CD, and server environments.
//! It's the core component for orchestrating multiple AI agents via redititi.

use crate::terminal::Terminal;
use crate::server_client::ServerClient;
use tokio::time::{self, Duration};
use anyhow::Result;

/// Configuration for headless mode
pub struct HeadlessConfig {
    /// Server address (e.g., "localhost:6379")
    pub server_addr: String,
    /// Authentication token
    pub token: String,
    /// Optional session name (creates new if None)
    pub session_name: Option<String>,
    /// Optional pane name
    pub pane_name: Option<String>,
    /// Terminal columns
    pub cols: u16,
    /// Terminal rows
    pub rows: u16,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            server_addr: "localhost:6379".to_string(),
            token: String::new(),
            session_name: None,
            pane_name: None,
            cols: 80,
            rows: 24,
        }
    }
}

/// Run terminal in headless mode
///
/// This function runs a terminal without GPU rendering, communicating
/// with a redititi server for command injection and screen capture.
///
/// # Arguments
/// * `config` - Headless configuration
///
/// # Returns
/// * `Result<()>` - Ok on graceful shutdown, Err on failure
///
/// # Example
/// ```no_run
/// use titi::headless::{run_headless, HeadlessConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = HeadlessConfig {
///         server_addr: "localhost:6379".to_string(),
///         token: "your_token_here".to_string(),
///         session_name: Some("my-session".to_string()),
///         pane_name: Some("my-pane".to_string()),
///         cols: 80,
///         rows: 24,
///     };
///
///     run_headless(config).await
/// }
/// ```
pub async fn run_headless(config: HeadlessConfig) -> Result<()> {
    log::info!("Starting headless mode");
    log::info!("Connecting to server at {}", config.server_addr);

    // Connect to redititi server
    let mut client = ServerClient::connect(&config.server_addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to connect: {}", e))?;

    log::info!("Connected to server, authenticating...");

    // Authenticate
    client
        .authenticate(&config.token)
        .await
        .map_err(|e| anyhow::anyhow!("Authentication failed: {}", e))?;

    log::info!("Authenticated successfully");

    // Create or join session
    let session_id = client
        .create_session(config.session_name.as_deref())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create session: {}", e))?;

    log::info!("Session created: {}", session_id);

    // Create pane
    let pane_id = client
        .create_pane(config.pane_name.as_deref())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create pane: {}", e))?;

    log::info!("Pane created: {}", pane_id);

    // Subscribe to input channel
    client
        .subscribe_input()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to subscribe to input: {}", e))?;

    log::info!("Subscribed to input channel");

    // Create terminal with server integration
    let mut terminal = Terminal::new_with_server(config.cols, config.rows, client)?;

    log::info!("Terminal created, entering main loop");

    // Main event loop
    let mut frame_count = 0u64;
    let mut last_log = std::time::Instant::now();

    loop {
        // Read from PTY (non-blocking)
        match terminal.read()? {
            Some(output) => {
                // Process escape sequences and update grid
                terminal.process_output(&output);

                // Publish dirty lines to server
                terminal.publish_output_if_needed().await;

                frame_count += 1;
            }
            None => {
                // No output available, continue
            }
        }

        // Poll for input commands from server
        if let Err(e) = terminal.poll_server_input().await {
            log::error!("Failed to poll server input: {}", e);
        }

        // Log heartbeat every 60 seconds
        if last_log.elapsed() >= Duration::from_secs(60) {
            log::info!("Headless terminal running (frames processed: {})", frame_count);
            last_log = std::time::Instant::now();
        }

        // Small sleep to avoid busy loop (10ms = 100Hz polling rate)
        time::sleep(Duration::from_millis(10)).await;
    }
}

/// Builder for HeadlessConfig
pub struct HeadlessConfigBuilder {
    config: HeadlessConfig,
}

impl HeadlessConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: HeadlessConfig::default(),
        }
    }

    pub fn server_addr(mut self, addr: impl Into<String>) -> Self {
        self.config.server_addr = addr.into();
        self
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.config.token = token.into();
        self
    }

    pub fn session_name(mut self, name: impl Into<String>) -> Self {
        self.config.session_name = Some(name.into());
        self
    }

    pub fn pane_name(mut self, name: impl Into<String>) -> Self {
        self.config.pane_name = Some(name.into());
        self
    }

    pub fn size(mut self, cols: u16, rows: u16) -> Self {
        self.config.cols = cols;
        self.config.rows = rows;
        self
    }

    pub fn build(self) -> HeadlessConfig {
        self.config
    }
}

impl Default for HeadlessConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_config_builder() {
        let config = HeadlessConfigBuilder::new()
            .server_addr("localhost:6380")
            .token("test_token")
            .session_name("test-session")
            .pane_name("test-pane")
            .size(100, 30)
            .build();

        assert_eq!(config.server_addr, "localhost:6380");
        assert_eq!(config.token, "test_token");
        assert_eq!(config.session_name, Some("test-session".to_string()));
        assert_eq!(config.pane_name, Some("test-pane".to_string()));
        assert_eq!(config.cols, 100);
        assert_eq!(config.rows, 30);
    }

    #[test]
    fn test_headless_config_default() {
        let config = HeadlessConfig::default();
        assert_eq!(config.server_addr, "localhost:6379");
        assert_eq!(config.cols, 80);
        assert_eq!(config.rows, 24);
    }
}
