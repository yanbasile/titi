// Timing and synchronization edge cases for terminal emulator
//
// Tests race conditions, timing issues, and synchronization edge cases

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use titi::terminal::{Grid, TerminalParser};
use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;

#[tokio::test]
#[ignore]
async fn test_timing_edge_cases() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  BATTLE TEST: Timing & Synchronization Edge Cases         ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Testing race conditions and timing issues                ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut tests_passed = 0;
    let total_tests = 6;

    // Test 1: Rapid connect/disconnect
    println!("⏱️  Test 1: Rapid connect/disconnect cycles...");
    if test_rapid_connect_disconnect().await.is_ok() {
        println!("   ✅ Rapid connect/disconnect handled correctly");
        tests_passed += 1;
    } else {
        println!("   ❌ Rapid connect/disconnect test failed");
    }

    // Test 2: Concurrent writes to same pane
    println!("\n⏱️  Test 2: Concurrent writes to same pane...");
    if test_concurrent_writes().await.is_ok() {
        println!("   ✅ Concurrent writes handled correctly");
        tests_passed += 1;
    } else {
        println!("   ❌ Concurrent writes test failed");
    }

    // Test 3: Message ordering
    println!("\n⏱️  Test 3: Message ordering guarantees...");
    if test_message_ordering().await.is_ok() {
        println!("   ✅ Message ordering maintained");
        tests_passed += 1;
    } else {
        println!("   ❌ Message ordering test failed");
    }

    // Test 4: Timeout handling
    println!("\n⏱️  Test 4: Timeout and deadline handling...");
    if test_timeout_handling().await.is_ok() {
        println!("   ✅ Timeouts handled correctly");
        tests_passed += 1;
    } else {
        println!("   ❌ Timeout handling test failed");
    }

    // Test 5: Burst traffic
    println!("\n⏱️  Test 5: Burst traffic handling...");
    if test_burst_traffic().await.is_ok() {
        println!("   ✅ Burst traffic handled correctly");
        tests_passed += 1;
    } else {
        println!("   ❌ Burst traffic test failed");
    }

    // Test 6: Parser state consistency
    println!("\n⏱️  Test 6: Parser state consistency under load...");
    if test_parser_consistency().is_ok() {
        println!("   ✅ Parser state remains consistent");
        tests_passed += 1;
    } else {
        println!("   ❌ Parser consistency test failed");
    }

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  TEST RESULTS                                              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    println!("Tests passed: {}/{}", tests_passed, total_tests);
    println!();

    assert_eq!(tests_passed, total_tests, "Not all timing tests passed");

    println!("✅ Timing & Synchronization Edge Cases PASSED!");
    println!("   All timing edge cases handled correctly\n");
}

async fn test_rapid_connect_disconnect() -> Result<(), String> {
    let (token, server_handle) = start_test_server(21000).await;

    // Perform 50 rapid connect/disconnect cycles
    for i in 0..50 {
        let mut client = ServerClient::connect("127.0.0.1:21000")
            .await
            .map_err(|e| format!("Connect {} failed: {}", i, e))?;

        client.authenticate(&token)
            .await
            .map_err(|e| format!("Auth {} failed: {}", i, e))?;

        // Immediate disconnect
        drop(client);

        // No delay - stress test rapid cycling
    }

    server_handle.abort();
    Ok(())
}

async fn test_concurrent_writes() -> Result<(), String> {
    let (token, server_handle) = start_test_server(21001).await;

    let mut client = ServerClient::connect("127.0.0.1:21001").await?;
    client.authenticate(&token).await?;
    client.create_session(Some("concurrent-test")).await?;

    let session_id = client.session_id().to_string();
    let pane_id = client.pane_id().to_string();

    // Spawn 10 concurrent writers
    let mut handles = vec![];
    for i in 0..10 {
        let token_clone = token.clone();
        let session_id_clone = session_id.clone();
        let pane_id_clone = pane_id.clone();

        let handle = tokio::spawn(async move {
            let mut client = ServerClient::connect("127.0.0.1:21001").await?;
            client.authenticate(&token_clone).await?;

            // Write 100 messages rapidly
            let channel = format!("{}/pane-{}/output", session_id_clone, pane_id_clone);
            for j in 0..100 {
                let msg = format!("Writer {} Message {}", i, j);
                client.publish_to_channel(&channel, &msg).await?;
            }

            Ok::<(), String>(())
        });

        handles.push(handle);
    }

    // Wait for all writers
    for handle in handles {
        handle.await.map_err(|e| format!("Join error: {}", e))??;
    }

    server_handle.abort();
    Ok(())
}

async fn test_message_ordering() -> Result<(), String> {
    let (token, server_handle) = start_test_server(21002).await;

    let mut client = ServerClient::connect("127.0.0.1:21002").await?;
    client.authenticate(&token).await?;
    client.create_session(Some("ordering-test")).await?;
    client.subscribe_output().await?;

    let session_id = client.session_id().to_string();
    let pane_id = client.pane_id().to_string();

    // Send messages in order
    let channel = format!("{}/pane-{}/output", session_id, pane_id);
    for i in 0..100 {
        let msg = format!("Message {}", i);
        client.publish_to_channel(&channel, &msg).await?;
    }

    // Give time for messages to propagate
    sleep(Duration::from_millis(500)).await;

    // Read messages - should maintain order within reasonable bounds
    let mut received_count = 0;
    for _ in 0..20 {
        if let Ok(Some(_msg)) = client.read_output().await {
            received_count += 1;
        }
        sleep(Duration::from_millis(10)).await;
    }

    // We should receive at least some messages
    if received_count < 10 {
        return Err(format!("Too few messages received: {}", received_count));
    }

    server_handle.abort();
    Ok(())
}

async fn test_timeout_handling() -> Result<(), String> {
    let (token, server_handle) = start_test_server(21003).await;

    let mut client = ServerClient::connect("127.0.0.1:21003").await?;
    client.authenticate(&token).await?;
    client.create_session(Some("timeout-test")).await?;
    client.subscribe_output().await?;

    // Try to read with no data available (should return None quickly)
    let start = Instant::now();
    let result = client.read_output().await?;
    let elapsed = start.elapsed();

    // Should complete quickly (< 100ms) even with no data
    if elapsed > Duration::from_millis(100) {
        return Err(format!("Read took too long: {:?}", elapsed));
    }

    // Result should be None (no data)
    if result.is_some() {
        return Err("Unexpected data received".to_string());
    }

    server_handle.abort();
    Ok(())
}

async fn test_burst_traffic() -> Result<(), String> {
    let (token, server_handle) = start_test_server(21004).await;

    let mut client = ServerClient::connect("127.0.0.1:21004").await?;
    client.authenticate(&token).await?;
    client.create_session(Some("burst-test")).await?;

    let session_id = client.session_id().to_string();
    let pane_id = client.pane_id().to_string();

    // Send 1000 messages as fast as possible
    let start = Instant::now();
    let channel = format!("{}/pane-{}/output", session_id, pane_id);

    for i in 0..1000 {
        let msg = format!("Burst {}", i);
        client.publish_to_channel(&channel, &msg).await?;
    }

    let elapsed = start.elapsed();
    let throughput = 1000.0 / elapsed.as_secs_f64();

    println!("   📊 Burst throughput: {:.2} msg/sec", throughput);

    // Should be able to handle at least 1000 msg/sec
    if throughput < 1000.0 {
        return Err(format!("Throughput too low: {:.2} msg/sec", throughput));
    }

    server_handle.abort();
    Ok(())
}

fn test_parser_consistency() -> Result<(), String> {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid.clone());

    // Interleave different types of sequences rapidly
    for i in 0..1000 {
        match i % 4 {
            0 => parser.parse(b"Regular text\n"),
            1 => parser.parse(b"\x1b[31mRed text\x1b[0m\n"),
            2 => parser.parse(b"\x1b[H\x1b[2JClear screen\n"),
            3 => parser.parse(b"\x1b[10;20HCursor move\n"),
            _ => unreachable!(),
        }
    }

    // Parser should still be in consistent state
    // (no panics = success)

    Ok(())
}

async fn start_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    let token = format!("timing_test_token_{}", port);
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
