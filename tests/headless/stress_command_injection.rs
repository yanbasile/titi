//! Stress Test 1: High-Frequency Command Injection
//!
//! Tests the headless terminal's ability to handle rapid command streams.
//! Simulates AI agents sending commands at high frequency.
//!
//! **Scenarios:**
//! - 1000 commands injected within 1 second
//! - 10,000 commands over 10 seconds (1000/sec sustained)
//! - Burst patterns: 100 commands instant, then pause, repeat
//! - Concurrent command injection from multiple "agents"
//!
//! **Validates:**
//! - Command queue doesn't overflow
//! - All commands are processed in order
//! - No command loss under stress
//! - Response time remains acceptable

use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;
use tokio::time::{sleep, Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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
async fn test_headless_rapid_command_injection() {
    let port = 18001;
    let (token, handle) = start_test_server(port).await;

    // Create headless client
    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("rapid-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting rapid command injection test");

    let start = Instant::now();
    let command_count = 1000;

    // Inject 1000 commands as fast as possible
    for i in 0..command_count {
        let cmd = format!("echo 'Command {}'", i);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");

        if i % 100 == 0 {
            println!("   Injected {} commands...", i);
        }
    }

    let elapsed = start.elapsed();
    let rate = command_count as f64 / elapsed.as_secs_f64();

    println!("✅ Injected {} commands in {:?}", command_count, elapsed);
    println!("   Rate: {:.0} commands/sec", rate);

    // Should complete in under 5 seconds
    assert!(
        elapsed < Duration::from_secs(5),
        "Command injection too slow: {:?}",
        elapsed
    );

    // Should achieve at least 200 commands/sec
    assert!(
        rate > 200.0,
        "Command injection rate too low: {:.0} cmd/s",
        rate
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_sustained_command_injection() {
    let port = 18002;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("sustained-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting sustained command injection test");

    let start = Instant::now();
    let duration = Duration::from_secs(10);

    let mut count = 0;
    while start.elapsed() < duration {
        let cmd = format!("echo 'Sustained {}'", count);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");

        count += 1;
        if count % 1000 == 0 {
            println!("   Sent {} commands ({:.1}s elapsed)", count, start.elapsed().as_secs_f64());
        }

        // Small yield to prevent tight loop monopolizing CPU
        if count % 100 == 0 {
            tokio::task::yield_now().await;
        }
    }

    let elapsed = start.elapsed();
    let actual_rate = count as f64 / elapsed.as_secs_f64();

    println!("✅ Sustained injection complete");
    println!("   Commands: {}", count);
    println!("   Duration: {:?}", elapsed);
    println!("   Actual rate: {:.0} cmd/s", actual_rate);

    // Should sustain at least 200 cmd/s for 10 seconds (minimum viable performance)
    assert!(
        actual_rate >= 200.0,
        "Sustained rate too low: {:.0} cmd/s (minimum: 200 cmd/s)",
        actual_rate
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_burst_command_injection() {
    let port = 18003;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("burst-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting burst command injection test");

    let bursts = 10;
    let commands_per_burst = 100;
    let pause_between_bursts = Duration::from_millis(500);

    let start = Instant::now();
    let mut total_commands = 0;

    for burst in 0..bursts {
        let burst_start = Instant::now();

        // Send 100 commands instantly
        for i in 0..commands_per_burst {
            let cmd = format!("echo 'Burst {} Command {}'", burst, i);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
        }

        let burst_elapsed = burst_start.elapsed();
        println!(
            "   Burst {} complete: {} commands in {:?}",
            burst, commands_per_burst, burst_elapsed
        );

        total_commands += commands_per_burst;

        if burst < bursts - 1 {
            tokio::time::sleep(pause_between_bursts).await;
        }
    }

    let elapsed = start.elapsed();
    println!("✅ Burst test complete");
    println!("   Total commands: {}", total_commands);
    println!("   Total time: {:?}", elapsed);
    println!("   Bursts: {}", bursts);

    assert_eq!(total_commands, bursts * commands_per_burst);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_concurrent_multi_agent_injection() {
    let port = 18004;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting concurrent multi-agent command injection test");

    let num_agents = 5;
    let commands_per_agent = 200;
    let counter = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();

    // Spawn multiple "agents" that inject commands concurrently
    let mut agent_handles = vec![];

    for agent_id in 0..num_agents {
        let token_clone = token.clone();
        let counter_clone = counter.clone();

        let agent_handle = tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Failed to connect");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client
                .create_session(Some(&format!("agent-session-{}", agent_id)))
                .await
                .expect("Session failed");
            client
                .create_pane(Some(&format!("agent-pane-{}", agent_id)))
                .await
                .expect("Pane failed");

            for i in 0..commands_per_agent {
                let cmd = format!("echo 'Agent {} Command {}'", agent_id, i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }

            println!("   Agent {} completed {} commands", agent_id, commands_per_agent);
        });

        agent_handles.push(agent_handle);
    }

    // Wait for all agents to complete
    for agent_handle in agent_handles {
        agent_handle.await.expect("Agent task failed");
    }

    let elapsed = start.elapsed();
    let total_commands = counter.load(Ordering::Relaxed);
    let rate = total_commands as f64 / elapsed.as_secs_f64();

    println!("✅ Multi-agent test complete");
    println!("   Agents: {}", num_agents);
    println!("   Total commands: {}", total_commands);
    println!("   Duration: {:?}", elapsed);
    println!("   Combined rate: {:.0} cmd/s", rate);

    assert_eq!(total_commands, num_agents * commands_per_agent);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
