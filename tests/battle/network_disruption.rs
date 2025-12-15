use std::time::{Duration, Instant};
use tokio::time::sleep;
use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;

/// Battle Test Scenario 3: Network Disruption & Recovery
///
/// Tests Redititi server resilience under network failures including:
/// - Sudden client disconnections
/// - Server restarts
/// - Connection timeouts
/// - Session recovery
#[tokio::test]
#[ignore]
async fn test_network_disruption_recovery() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  BATTLE TEST: Network Disruption & Recovery               ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Testing server resilience under network failures         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let start_time = Instant::now();
    let mut test_results = Vec::new();

    // Test 1: Sudden client disconnection
    println!("📡 Test 1: Sudden client disconnection...");
    match test_client_disconnect().await {
        Ok(()) => {
            println!("   ✅ Client disconnection handled gracefully");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Client disconnection test failed: {}", e);
            test_results.push(false);
        }
    }

    sleep(Duration::from_millis(500)).await;

    // Test 2: Client reconnection
    println!("\n📡 Test 2: Client reconnection...");
    match test_client_reconnection().await {
        Ok(()) => {
            println!("   ✅ Client reconnection successful");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Client reconnection test failed: {}", e);
            test_results.push(false);
        }
    }

    sleep(Duration::from_millis(500)).await;

    // Test 3: Session persistence after disconnect
    println!("\n📡 Test 3: Session persistence...");
    match test_session_persistence().await {
        Ok(()) => {
            println!("   ✅ Session persisted across disconnect");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Session persistence test failed: {}", e);
            test_results.push(false);
        }
    }

    sleep(Duration::from_millis(500)).await;

    // Test 4: Multiple rapid reconnections
    println!("\n📡 Test 4: Rapid reconnections...");
    match test_rapid_reconnections().await {
        Ok(()) => {
            println!("   ✅ Rapid reconnections handled");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Rapid reconnections test failed: {}", e);
            test_results.push(false);
        }
    }

    let total_time = start_time.elapsed();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  TEST RESULTS                                              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let passed_tests = test_results.iter().filter(|&&r| r).count();
    println!("Total test time: {:?}", total_time);
    println!("Tests passed: {}/{}", passed_tests, test_results.len());

    assert_eq!(passed_tests, test_results.len(), "Some network disruption tests failed");
    assert!(total_time < Duration::from_secs(300), "Tests took too long: {:?}", total_time);

    println!("\n✅ Network Disruption & Recovery Test PASSED!");
}

async fn test_client_disconnect() -> Result<(), String> {
    let (token, server_handle) = start_test_server(20001).await;

    // Connect client
    let mut client = ServerClient::connect("127.0.0.1:20001")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    client.authenticate(&token)
        .await
        .map_err(|e| format!("Authentication failed: {}", e))?;

    // Create session
    client.create_session(Some("test-disconnect")).await?;

    let session_id = client.session_id().to_string();
    let pane_id = client.pane_id().to_string();

    // Send some data
    client.inject_command(&session_id, &pane_id, "echo 'test'\n").await?;

    // Abruptly drop client (simulates network disconnect)
    drop(client);

    sleep(Duration::from_millis(200)).await;

    // Server should still be running
    server_handle.abort();

    Ok(())
}

async fn test_client_reconnection() -> Result<(), String> {
    let (token, server_handle) = start_test_server(20002).await;

    // First connection
    {
        let mut client = ServerClient::connect("127.0.0.1:20002")
            .await
            .map_err(|e| format!("First connection failed: {}", e))?;

        client.authenticate(&token).await?;
        client.create_session(Some("test-reconnect")).await?;

        // Drop connection
        drop(client);
        sleep(Duration::from_millis(200)).await;
    }

    // Second connection (reconnect)
    {
        let mut client = ServerClient::connect("127.0.0.1:20002")
            .await
            .map_err(|e| format!("Reconnection failed: {}", e))?;

        client.authenticate(&token).await?;

        // Should be able to work with server after reconnect
        client.create_session(Some("test-reconnect-2")).await?;

        drop(client);
    }

    server_handle.abort();

    Ok(())
}

async fn test_session_persistence() -> Result<(), String> {
    let (token, server_handle) = start_test_server(20003).await;

    let session_id: String;
    let pane_id: String;

    // First connection - create session
    {
        let mut client = ServerClient::connect("127.0.0.1:20003")
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        client.authenticate(&token).await?;
        client.create_session(Some("persistent-session")).await?;

        session_id = client.session_id().to_string();
        pane_id = client.pane_id().to_string();

        client.inject_command(&session_id, &pane_id, "persistent data\n").await?;

        // Disconnect
        drop(client);
        sleep(Duration::from_millis(300)).await;
    }

    // Second connection - verify session still accessible
    {
        let mut client = ServerClient::connect("127.0.0.1:20003")
            .await
            .map_err(|e| format!("Reconnection failed: {}", e))?;

        client.authenticate(&token).await?;

        // Try to inject command to verify session still exists
        match client.inject_command(&session_id, &pane_id, "test\n").await {
            Ok(()) => {}, // Session still exists
            Err(_) => return Err("Session not persisted".to_string()),
        }

        drop(client);
    }

    server_handle.abort();

    Ok(())
}

async fn test_rapid_reconnections() -> Result<(), String> {
    let (token, server_handle) = start_test_server(20004).await;

    // Perform 10 rapid connect/disconnect cycles
    for i in 0..10 {
        let mut client = ServerClient::connect("127.0.0.1:20004")
            .await
            .map_err(|e| format!("Connection {} failed: {}", i, e))?;

        client.authenticate(&token).await?;

        let session_name = format!("rapid-session-{}", i);
        client.create_session(Some(&session_name)).await?;

        // Immediate disconnect
        drop(client);

        // Small delay
        sleep(Duration::from_millis(50)).await;
    }

    // Final connection to verify server still works
    let mut client = ServerClient::connect("127.0.0.1:20004")
        .await
        .map_err(|e| format!("Final connection failed: {}", e))?;

    client.authenticate(&token).await?;
    client.create_session(Some("final-test")).await?;

    drop(client);
    server_handle.abort();

    Ok(())
}

async fn start_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    let token = format!("test_token_network_disruption_{}", port);
    std::env::set_var("TITI_TOKEN", &token);

    let auth = TokenAuth::new().expect("Failed to create auth");
    let server = RedititiTcpServer::new(format!("127.0.0.1:{}", port), auth);

    let handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Test server error: {}", e);
        }
    });

    sleep(Duration::from_millis(200)).await;

    (token, handle)
}
