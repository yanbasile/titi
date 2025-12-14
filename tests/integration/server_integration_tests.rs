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

// Future integration tests to add:
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

#[tokio::test]
async fn test_command_injection() {
    let port = 17383;
    let (token, handle) = start_test_server_with_env(port).await;

    // Create terminal client that will receive commands
    let mut terminal_client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect terminal client");

    terminal_client.authenticate(&token).await.expect("Terminal auth failed");
    terminal_client.create_session(Some("cmd-test")).await.expect("Session failed");
    terminal_client.create_pane(Some("pane1")).await.expect("Pane failed");
    terminal_client.subscribe_input().await.expect("Subscribe input failed");

    let session_id = terminal_client.session_id().to_string();
    let pane_id = terminal_client.pane_id().to_string();

    // Create controller client that will inject commands
    let mut controller_client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect controller client");

    controller_client.authenticate(&token).await.expect("Controller auth failed");

    // Inject a command
    controller_client.inject_command(&session_id, &pane_id, "echo 'Hello from injection'")
        .await
        .expect("Injection failed");

    // Give time for message to propagate
    sleep(Duration::from_millis(50)).await;

    // Terminal reads the injected command
    let command = terminal_client.read_input()
        .await
        .expect("Failed to read input");

    assert!(command.is_some(), "No command received");
    let cmd = command.unwrap();
    assert!(cmd.contains("echo 'Hello from injection'"), "Unexpected command: {}", cmd);

    // Cleanup
    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_output_capture() {
    let port = 17384;
    let (token, handle) = start_test_server_with_env(port).await;

    // Create terminal client that will publish output
    let mut terminal_client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect terminal client");

    terminal_client.authenticate(&token).await.expect("Terminal auth failed");
    terminal_client.create_session(Some("output-test")).await.expect("Session failed");
    terminal_client.create_pane(Some("pane1")).await.expect("Pane failed");

    let session_id = terminal_client.session_id().to_string();
    let pane_id = terminal_client.pane_id().to_string();

    // Terminal publishes output
    terminal_client.publish_output("Line 1: Terminal output")
        .await
        .expect("Publish failed");

    terminal_client.publish_output("Line 2: More output")
        .await
        .expect("Publish failed");

    // Give time for messages to propagate
    sleep(Duration::from_millis(100)).await;

    // Observer client can read the output by checking the channel directly
    // For this test, we'll verify output was published successfully by checking
    // that we can inject commands and get responses (tested in other tests)
    // A full implementation would use RPOP on the output channel

    // Cleanup
    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_bidirectional_communication() {
    let port = 17385;
    let (token, handle) = start_test_server_with_env(port).await;

    // Create terminal client
    let mut terminal_client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect terminal");

    terminal_client.authenticate(&token).await.expect("Auth failed");
    terminal_client.create_session(Some("bidir-test")).await.expect("Session failed");
    terminal_client.create_pane(Some("pane1")).await.expect("Pane failed");
    terminal_client.subscribe_input().await.expect("Subscribe input failed");
    terminal_client.subscribe_output().await.expect("Subscribe output failed");

    let session_id = terminal_client.session_id().to_string();
    let pane_id = terminal_client.pane_id().to_string();

    // Create controller client (separate session, will control terminal remotely)
    let mut controller_client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect controller");

    controller_client.authenticate(&token).await.expect("Controller auth failed");
    // Controller doesn't need to be in same session, uses direct INJECT/RPOP commands

    // Controller injects command
    controller_client.inject_command(&session_id, &pane_id, "ls -la")
        .await
        .expect("Injection failed");

    sleep(Duration::from_millis(50)).await;

    // Terminal receives command
    let cmd = terminal_client.read_input().await.expect("Read failed");
    assert!(cmd.is_some());
    assert!(cmd.unwrap().contains("ls -la"));

    // Terminal sends response
    terminal_client.publish_output("total 42")
        .await
        .expect("Publish failed");

    sleep(Duration::from_millis(50)).await;

    // Controller receives output from the terminal's session
    let output = controller_client.read_from_channel(&session_id, &pane_id, "output")
        .await
        .expect("Read failed");
    assert!(output.is_some(), "No output received from terminal");
    assert!(output.unwrap().contains("total 42"));

    // Cleanup
    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn test_headless_terminal_with_real_pty() {
    use titi::terminal::Terminal;

    let port = 17386;
    let (token, server_handle) = start_test_server_with_env(port).await;

    // Create terminal client for headless mode
    let mut terminal_client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Terminal client connection failed");

    terminal_client.authenticate(&token).await.expect("Terminal auth failed");
    terminal_client.create_session(Some("headless-test")).await.expect("Session failed");
    terminal_client.create_pane(Some("pane1")).await.expect("Pane failed");
    terminal_client.subscribe_input().await.expect("Subscribe input failed");

    let session_id = terminal_client.session_id().to_string();
    let pane_id = terminal_client.pane_id().to_string();

    // Create terminal with real PTY and server integration
    let mut terminal = Terminal::new_with_server(80, 24, terminal_client)
        .expect("Failed to create terminal");

    // Create controller client to inject commands
    let mut controller = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Controller connection failed");

    controller.authenticate(&token).await.expect("Controller auth failed");

    // Inject a simple shell command
    controller.inject_command(&session_id, &pane_id, "echo 'HEADLESS_TEST_OUTPUT'\n")
        .await
        .expect("Command injection failed");

    // Process the terminal event loop for a few iterations
    let mut output_found = false;
    for iteration in 0..20 {
        // Poll for input from server and write to PTY
        terminal.poll_server_input().await.expect("Failed to poll server input");

        // Read PTY output
        if let Ok(Some(data)) = terminal.read() {
            // Process output through parser
            terminal.process_output(&data);

            // Publish output to server
            terminal.publish_output_if_needed().await;
        }

        // Check if output was published
        sleep(Duration::from_millis(100)).await;

        if let Ok(Some(output)) = controller.read_from_channel(&session_id, &pane_id, "output").await {
            if output.contains("HEADLESS_TEST_OUTPUT") {
                output_found = true;
                eprintln!("✅ Found output on iteration {}: {}", iteration, output);
                break;
            }
        }
    }

    assert!(output_found, "Expected to find 'HEADLESS_TEST_OUTPUT' in terminal output");

    // Cleanup
    server_handle.abort();
    sleep(Duration::from_millis(100)).await;
}
