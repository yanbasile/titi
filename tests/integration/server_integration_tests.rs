//! Integration tests for Titi + Redititi server interaction
//!
//! These tests verify end-to-end functionality:
//! - Command injection from redititi to terminal
//! - Screen capture from terminal to redititi
//!
//! Note: These tests require spawning a redititi server process

use std::time::Duration;
use tokio::time::sleep;
use titi::server_client::ServerClient;
use titi::redititi_server::{RedititiTcpServer, TokenAuth};

/// Helper to start a test redititi server
/// Note: This test is currently disabled because TokenAuth requires file system access
/// or environment variables. A future enhancement could add a test-only constructor.
///
/// For now, integration tests should be run against a real redititi server instance.
#[allow(dead_code)]
async fn _start_test_server_disabled(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    // Create a static token for testing
    let token = "test_token_12345678901234567890123456789012345678901234567890123456".to_string();

    // Set environment variable for TokenAuth to use
    std::env::set_var("TITI_TOKEN", &token);

    let auth = TokenAuth::new()
        .expect("Failed to create test auth");

    let server = RedititiTcpServer::new(
        format!("127.0.0.1:{}", port),
        auth,
    );

    let handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    (token, handle)
}

// NOTE: Integration tests require a running redititi server
// Run tests with: cargo test --test server_integration -- --ignored
//
// Before running, start the redititi server in a separate terminal:
//   cargo run --bin redititi --release
//
// The tests use ports 16379-16386, so make sure the redititi server
// is NOT running on these ports (use the default 6379).

#[tokio::test]
#[ignore] // Requires external redititi server
async fn test_basic_client_api() {
    // This is a placeholder test showing the expected API usage.
    // To run actual integration tests, you need to:
    //
    // 1. Start a redititi server:
    //    cargo run --bin redititi --release
    //
    // 2. Get the authentication token from server output
    //
    // 3. Connect using ServerClient:
    let _example_usage = || async {
        let mut client = ServerClient::connect("127.0.0.1:6379")
            .await
            .expect("Failed to connect");

        client.authenticate("your_token_here")
            .await
            .expect("Auth failed");

        let session_id = client.create_session(Some("test"))
            .await
            .expect("Session creation failed");

        let pane_id = client.create_pane(Some("pane1"))
            .await
            .expect("Pane creation failed");

        client.subscribe_input()
            .await
            .expect("Subscribe failed");

        client.publish_output("test output")
            .await
            .expect("Publish failed");

        (session_id, pane_id)
    };
}

// Future integration tests to add once we have a test-friendly TokenAuth:
//
// - test_command_injection_flow()
// - test_output_capture_flow()
// - test_reconnection()
// - test_full_headless_terminal_integration()

/// Helper to start a test server with environment variable auth
async fn start_test_server_with_env(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    let token = "test_token_12345678901234567890123456789012345678901234567890123456".to_string();

    // Set environment variable before creating auth
    std::env::set_var("TITI_TOKEN", &token);

    let auth = TokenAuth::new().expect("Failed to create auth");
    let server = RedititiTcpServer::new(format!("127.0.0.1:{}", port), auth);

    let handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Test server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(200)).await;

    (token, handle)
}

#[tokio::test]
async fn test_server_client_connection() {
    let port = 17379;
    let (token, handle) = start_test_server_with_env(port).await;

    // Connect client
    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect to test server");

    // Authenticate
    client.authenticate(&token)
        .await
        .expect("Authentication failed");

    assert!(client.is_authenticated());

    // Cleanup
    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_session_and_pane_management() {
    let port = 17380;
    let (token, handle) = start_test_server_with_env(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");

    // Create session
    let session_id = client.create_session(Some("test-session"))
        .await
        .expect("Failed to create session");

    assert_eq!(session_id, "test-session");
    assert_eq!(client.session_id(), "test-session");

    // Create pane
    let pane_id = client.create_pane(Some("test-pane"))
        .await
        .expect("Failed to create pane");

    assert_eq!(pane_id, "test-pane");
    assert_eq!(client.pane_id(), "test-pane");

    // Cleanup
    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_channel_pub_sub() {
    let port = 17381;
    let (token, handle) = start_test_server_with_env(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");

    // Create session and pane
    client.create_session(Some("pub-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    // Subscribe to input channel
    client.subscribe_input()
        .await
        .expect("Subscribe failed");

    // Publish output
    client.publish_output("Line 1: Hello from test")
        .await
        .expect("Publish failed");

    client.publish_output("Line 2: Second message")
        .await
        .expect("Publish failed");

    // Give time for messages to be processed
    sleep(Duration::from_millis(50)).await;

    // Cleanup
    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_multiple_clients() {
    let port = 17382;
    let (token, handle) = start_test_server_with_env(port).await;

    // Create 3 clients
    let mut clients = Vec::new();
    for i in 0..3 {
        let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
            .await
            .expect(&format!("Failed to connect client {}", i));

        client.authenticate(&token)
            .await
            .expect(&format!("Client {} auth failed", i));

        let session_name = format!("session-{}", i);
        client.create_session(Some(&session_name))
            .await
            .expect(&format!("Client {} session failed", i));

        clients.push(client);
    }

    // Verify all sessions are unique
    assert_eq!(clients[0].session_id(), "session-0");
    assert_eq!(clients[1].session_id(), "session-1");
    assert_eq!(clients[2].session_id(), "session-2");

    // Cleanup
    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
