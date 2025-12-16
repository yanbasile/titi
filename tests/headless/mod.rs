//! Headless Mode Test Suite
//!
//! Comprehensive stress tests and complex scenario tests for headless terminal operation.
//! These tests validate terminal automation, AI agent orchestration, and server integration.
//!
//! # Test Categories
//!
//! ## Stress Tests (5):
//! 1. **High-Frequency Command Injection** - Rapid command streams
//! 2. **Large Output Volume** - Multi-MB PTY output handling
//! 3. **Multi-Instance Concurrency** - Many headless terminals simultaneously
//! 4. **Long-Running Stability** - Hours of continuous operation
//! 5. **Rapid Lifecycle** - Fast session/pane creation/destruction
//!
//! ## Complex Scenario Tests (5):
//! 1. **Network Resilience** - Connection loss and recovery
//! 2. **Multi-Agent Coordination** - Cooperative headless instances
//! 3. **Interactive Shell Programs** - vim, less, interactive prompts
//! 4. **Unicode Torture Test** - Complex characters and escape sequences
//! 5. **Resource Leak Detection** - Memory and file descriptor validation

use titi::headless::{HeadlessConfig, HeadlessConfigBuilder};
use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;
use tokio::time::{sleep, timeout, Duration, Instant};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Test helper: Start a test server and return token + handle
async fn start_headless_test_server(port: u16) -> (String, JoinHandle<()>) {
    let token = format!("test-token-{}", rand::random::<u32>());
    let auth = TokenAuth::from_token(token.clone()).unwrap();
    let server = RedititiTcpServer::new(format!("127.0.0.1:{}", port), auth);

    let handle = tokio::spawn(async move {
        let _ = server.run().await;
    });

    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    (token, handle)
}

/// Test helper: Create a configured headless client
async fn create_headless_client(
    port: u16,
    token: &str,
    session_name: &str,
    pane_name: &str,
) -> Result<ServerClient, String> {
    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port)).await?;
    client.authenticate(token).await?;
    client.create_session(Some(session_name)).await?;
    client.create_pane(Some(pane_name)).await?;
    client.subscribe_input().await?;
    Ok(client)
}
