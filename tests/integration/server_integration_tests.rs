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
// - test_server_connection_and_authentication()
// - test_session_and_pane_creation()
// - test_subscribe_and_publish()
// - test_command_injection_flow()
// - test_output_capture_flow()
// - test_multiple_clients()
// - test_reconnection()
// - test_full_headless_terminal_integration()
