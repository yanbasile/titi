//! Basic Verification Test for Headless Infrastructure
//!
//! This test verifies that the basic server/client infrastructure works
//! without requiring full PTY integration.

use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_server_client_basic_communication() {
    println!("🚀 Testing basic server/client communication");

    // Start server
    let token = format!("test-token-{}", rand::random::<u32>());
    let auth = TokenAuth::from_token(token.clone()).unwrap();
    let server = RedititiTcpServer::new("127.0.0.1:19000".to_string(), auth);

    let server_handle = tokio::spawn(async move {
        let _ = server.run().await;
    });

    // Give server time to start
    sleep(Duration::from_millis(200)).await;

    println!("   ✓ Server started");

    // Connect client
    let mut client = ServerClient::connect("127.0.0.1:19000")
        .await
        .expect("Failed to connect to server");

    println!("   ✓ Client connected");

    // Authenticate
    client
        .authenticate(&token)
        .await
        .expect("Authentication failed");

    println!("   ✓ Client authenticated");

    // Create session
    let session_id = client
        .create_session(Some("test-session"))
        .await
        .expect("Failed to create session");

    println!("   ✓ Session created: {}", session_id);

    // Create pane
    let pane_id = client
        .create_pane(Some("test-pane"))
        .await
        .expect("Failed to create pane");

    println!("   ✓ Pane created: {}", pane_id);

    // Test inject command (this will publish to channel even without PTY)
    let result = client
        .inject(&"echo 'Hello World'\n".to_string())
        .await;

    match result {
        Ok(_) => println!("   ✓ Command injection successful"),
        Err(e) => println!("   ⚠  Command injection returned: {}", e),
    }

    // Cleanup
    server_handle.abort();
    sleep(Duration::from_millis(100)).await;

    println!("✅ Basic server/client communication test complete");
}

#[tokio::test]
async fn test_multiple_clients() {
    println!("🚀 Testing multiple concurrent clients");

    // Start server
    let token = format!("test-token-{}", rand::random::<u32>());
    let auth = TokenAuth::from_token(token.clone()).unwrap();
    let server = RedititiTcpServer::new("127.0.0.1:19001".to_string(), auth);

    let server_handle = tokio::spawn(async move {
        let _ = server.run().await;
    });

    sleep(Duration::from_millis(200)).await;

    // Create 5 clients
    let mut clients = vec![];
    for i in 0..5 {
        let mut client = ServerClient::connect("127.0.0.1:19001")
            .await
            .expect("Failed to connect");

        client.authenticate(&token).await.expect("Auth failed");
        client
            .create_session(Some(&format!("session-{}", i)))
            .await
            .expect("Session failed");
        client
            .create_pane(Some(&format!("pane-{}", i)))
            .await
            .expect("Pane failed");

        clients.push(client);
        println!("   ✓ Client {} connected and set up", i);
    }

    println!("✅ Multiple clients test complete: {} clients", clients.len());

    server_handle.abort();
    sleep(Duration::from_millis(100)).await;
}
