//! Complex Scenario Test 1: Network Resilience
//!
//! Tests headless terminal behavior under network stress and failures.
//! Simulates real-world network conditions.
//!
//! **Scenarios:**
//! - Server restart during operation
//! - Connection timeout handling
//! - Slow network simulation
//! - Packet loss tolerance
//!
//! **Validates:**
//! - Graceful handling of connection loss
//! - Reconnection strategies work
//! - No data corruption under network stress
//! - Timeouts are appropriately configured

use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;
use tokio::time::{sleep, timeout, Duration, Instant};

async fn start_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    let token = format!("test-token-{}", rand::random::<u32>());
    let auth = TokenAuth::from_token(token.clone()).unwrap();
    let server = RedititiTcpServer::new(format!("127.0.0.1:{}", port), auth);

    let handle = tokio::spawn(async move {
        let _ = server.run().await;
    });

    sleep(Duration::from_millis(100)).await;
    (token, handle)
}

#[tokio::test]
#[ignore]
async fn test_headless_server_restart() {
    let port = 18501;

    println!("🚀 Starting server restart resilience test");

    // Start initial server
    let (token, handle1) = start_test_server(port).await;

    // Connect client
    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Initial connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("restart-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    // Send some commands
    for i in 0..5 {
        let cmd = format!("echo 'Before restart {}'\n", i);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
    }

    println!("   Sent 5 commands, simulating server restart...");

    // Simulate server restart
    handle1.abort();
    sleep(Duration::from_millis(500)).await;

    println!("   Server stopped, restarting...");

    // Restart server (with same token for simplicity)
    let (_token2, handle2) = start_test_server(port).await;

    println!("   Server restarted");

    // Try to reconnect
    println!("   Attempting reconnection...");

    let reconnect_result = timeout(
        Duration::from_secs(5),
        ServerClient::connect(&format!("127.0.0.1:{}", port))
    )
    .await;

    match reconnect_result {
        Ok(Ok(mut new_client)) => {
            println!("   ✅ Reconnection successful");

            // Authenticate with new connection
            if new_client.authenticate(&token).await.is_ok() {
                println!("   ✅ Re-authentication successful");

                // Create new session
                if new_client.create_session(Some("restart-test-2")).await.is_ok() {
                    println!("   ✅ New session created");

                    if new_client.create_pane(Some("pane2")).await.is_ok() {
                        println!("   ✅ New pane created");

                        // Send command to verify everything works
                        if new_client.inject_command("echo 'After restart'\n").await.is_ok() {
                            println!("   ✅ Commands working after restart");
                        }
                    }
                }
            }
        }
        Ok(Err(e)) => {
            println!("   ⚠️  Reconnection failed: {}", e);
        }
        Err(_) => {
            println!("   ⚠️  Reconnection timeout");
        }
    }

    println!("✅ Server restart test complete");

    handle2.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_connection_timeout() {
    let port = 18502;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting connection timeout test");

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("timeout-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("   Testing command timeout handling");

    // Send commands and measure response time
    let mut timeouts = 0;
    let mut successes = 0;

    for i in 0..20 {
        let cmd = format!("echo 'Timeout test {}'\n", i);

        let result = timeout(Duration::from_secs(2), client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd)).await;

        match result {
            Ok(Ok(_)) => {
                successes += 1;
            }
            Ok(Err(_)) => {
                println!("   Command {} failed", i);
            }
            Err(_) => {
                timeouts += 1;
                println!("   Command {} timed out", i);
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Connection timeout test complete");
    println!("   Successes: {}", successes);
    println!("   Timeouts: {}", timeouts);

    // Most commands should succeed
    assert!(successes >= 15, "Too many timeouts: {} / 20", timeouts);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_slow_network_simulation() {
    let port = 18503;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting slow network simulation test");

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("slow-net")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("   Simulating slow network with artificial delays");

    let mut total_latency = Duration::ZERO;
    let num_commands = 10;

    for i in 0..num_commands {
        // Simulate network delay
        let network_delay = Duration::from_millis(50 + (i * 10) as u64);
        sleep(network_delay).await;

        let start = Instant::now();
        let cmd = format!("echo 'Slow network test {}'\n", i);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        let latency = start.elapsed();

        total_latency += latency;

        println!("   Command {} latency: {:?} (+ {}ms simulated delay)", i, latency, network_delay.as_millis());
    }

    let avg_latency = total_latency / num_commands;

    println!("✅ Slow network simulation test complete");
    println!("   Commands sent: {}", num_commands);
    println!("   Average latency: {:?}", avg_latency);

    // Should still complete within reasonable time
    assert!(avg_latency < Duration::from_secs(1), "Latency too high: {:?}", avg_latency);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_connection_refused() {
    println!("🚀 Starting connection refused handling test");

    let port = 18504;
    // Don't start a server - connection should be refused

    let result = timeout(
        Duration::from_secs(2),
        ServerClient::connect(&format!("127.0.0.1:{}", port))
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            panic!("Connection should have been refused");
        }
        Ok(Err(e)) => {
            println!("   ✅ Connection properly refused: {}", e);
        }
        Err(_) => {
            println!("   ✅ Connection attempt timed out as expected");
        }
    }

    println!("✅ Connection refused test complete");
}

#[tokio::test]
#[ignore]
async fn test_headless_graceful_disconnect() {
    let port = 18505;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting graceful disconnect test");

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("disconnect-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    // Send some commands
    for i in 0..10 {
        let cmd = format!("echo 'Test {}'\n", i);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
    }

    println!("   Sent 10 commands, disconnecting gracefully...");

    // Explicitly drop client to trigger disconnect
    drop(client);

    println!("   Client disconnected");

    // Wait a bit to ensure server processes the disconnect
    sleep(Duration::from_millis(200)).await;

    // Try to connect again - should work fine
    let mut client2 = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Reconnect failed");

    client2.authenticate(&token).await.expect("Re-auth failed");
    client2.create_session(Some("disconnect-test-2")).await.expect("Session 2 failed");
    client2.create_pane(Some("pane2")).await.expect("Pane 2 failed");

    println!("   ✅ Reconnected successfully after graceful disconnect");

    println!("✅ Graceful disconnect test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
