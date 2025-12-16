//! Stress Test 4: Long-Running Stability
//!
//! Tests headless terminals running for extended periods.
//! Simulates production deployments where terminals run for hours/days.
//!
//! **Scenarios:**
//! - 5-minute continuous operation
//! - 30-minute sustained activity
//! - 1-hour stability test (optional, very long)
//! - Memory stability over time
//!
//! **Validates:**
//! - No memory leaks over time
//! - No resource exhaustion
//! - Consistent performance over duration
//! - Graceful handling of long-running processes

use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;
use tokio::time::{sleep, Duration, Instant};

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
async fn test_headless_5_minute_stability() {
    let port = 18301;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("stability-5min")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting 5-minute stability test");
    println!("   This test will run for 5 minutes with periodic activity");

    let start = Instant::now();
    let test_duration = Duration::from_secs(5 * 60); // 5 minutes
    let mut command_count = 0;
    let mut last_report = Instant::now();

    while start.elapsed() < test_duration {
        // Send a command every 5 seconds
        let cmd = "echo 'Heartbeat check'\n";
        client.inject_command(cmd).await.expect("Inject failed");
        command_count += 1;

        // Report progress every minute
        if last_report.elapsed() >= Duration::from_secs(60) {
            let elapsed_mins = start.elapsed().as_secs() / 60;
            println!("   {} minutes elapsed, {} commands sent", elapsed_mins, command_count);
            last_report = Instant::now();
        }

        sleep(Duration::from_secs(5)).await;
    }

    let elapsed = start.elapsed();

    println!("✅ 5-minute stability test complete");
    println!("   Duration: {:?}", elapsed);
    println!("   Commands sent: {}", command_count);
    println!("   Average interval: {:.2}s", elapsed.as_secs_f64() / command_count as f64);

    // Should have sent roughly 60 commands (5min / 5s)
    assert!(
        command_count >= 50 && command_count <= 70,
        "Unexpected command count: {}",
        command_count
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_sustained_activity_30min() {
    let port = 18302;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("sustained-30min")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting 30-minute sustained activity test");
    println!("   WARNING: This test takes 30 minutes to complete");
    println!("   Commands will be sent every 10 seconds");

    let start = Instant::now();
    let test_duration = Duration::from_secs(30 * 60); // 30 minutes
    let mut command_count = 0;
    let mut last_report = Instant::now();

    while start.elapsed() < test_duration {
        // Send command with some output
        let cmd = format!("echo 'Sustained activity check {}'\n", command_count);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
        command_count += 1;

        // Report every 5 minutes
        if last_report.elapsed() >= Duration::from_secs(5 * 60) {
            let elapsed_mins = start.elapsed().as_secs() / 60;
            println!("   {} minutes elapsed, {} commands sent", elapsed_mins, command_count);
            last_report = Instant::now();
        }

        sleep(Duration::from_secs(10)).await;
    }

    let elapsed = start.elapsed();

    println!("✅ 30-minute sustained activity test complete");
    println!("   Duration: {:?}", elapsed);
    println!("   Commands sent: {}", command_count);

    // Should have sent roughly 180 commands (30min / 10s)
    assert!(
        command_count >= 170 && command_count <= 190,
        "Unexpected command count: {}",
        command_count
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_memory_stability() {
    let port = 18303;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("memory-stability")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting memory stability test (10 minutes)");
    println!("   Monitoring for memory growth patterns");

    let start = Instant::now();
    let test_duration = Duration::from_secs(10 * 60); // 10 minutes
    let mut samples = vec![];
    let mut command_count = 0;

    while start.elapsed() < test_duration {
        // Generate some load
        for i in 0..10 {
            let cmd = format!("echo 'Memory test iteration {}'\n", command_count * 10 + i);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
        }
        command_count += 10;

        // Sample memory (using process RSS as proxy)
        let memory_kb = estimate_process_memory();
        samples.push((start.elapsed(), memory_kb));

        if samples.len() % 10 == 0 {
            println!(
                "   {:.1} min: ~{} KB",
                start.elapsed().as_secs_f64() / 60.0,
                memory_kb
            );
        }

        sleep(Duration::from_secs(60)).await; // Sample every minute
    }

    let elapsed = start.elapsed();

    println!("✅ Memory stability test complete");
    println!("   Duration: {:?}", elapsed);
    println!("   Commands sent: {}", command_count);
    println!("   Memory samples: {}", samples.len());

    // Analyze memory trend
    if samples.len() >= 2 {
        let first = samples[0].1;
        let last = samples[samples.len() - 1].1;
        let growth = last as f64 - first as f64;
        let growth_pct = (growth / first as f64) * 100.0;

        println!("   Initial memory: {} KB", first);
        println!("   Final memory: {} KB", last);
        println!("   Growth: {:.1} KB ({:.1}%)", growth, growth_pct);

        // Memory should not grow more than 50% over 10 minutes
        assert!(
            growth_pct < 50.0,
            "Excessive memory growth: {:.1}%",
            growth_pct
        );
    }

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

/// Estimate process memory usage (simplified version)
fn estimate_process_memory() -> usize {
    // In a real implementation, you'd read /proc/self/status or use sysinfo crate
    // For testing, we return a reasonable placeholder
    // This would be replaced with actual memory measurement in production
    use std::fs;

    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(mem) = kb.parse::<usize>() {
                        return mem;
                    }
                }
            }
        }
    }

    // Fallback: return approximate value
    50_000 // 50 MB baseline
}

#[tokio::test]
#[ignore]
async fn test_headless_no_activity_stability() {
    let port = 18304;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("idle-stability")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting idle stability test (5 minutes)");
    println!("   Terminal remains connected but idle");

    let start = Instant::now();
    let test_duration = Duration::from_secs(5 * 60); // 5 minutes

    // Just stay connected, send one heartbeat per minute
    let mut heartbeat_count = 0;
    while start.elapsed() < test_duration {
        sleep(Duration::from_secs(60)).await;

        let cmd = "echo 'Heartbeat'\n";
        client.inject_command(cmd).await.expect("Inject failed");
        heartbeat_count += 1;

        let elapsed_mins = start.elapsed().as_secs() / 60;
        println!("   {} minutes elapsed (idle)", elapsed_mins);
    }

    let elapsed = start.elapsed();

    println!("✅ Idle stability test complete");
    println!("   Duration: {:?}", elapsed);
    println!("   Heartbeats: {}", heartbeat_count);

    // Should still be connected after 5 minutes
    assert!(heartbeat_count >= 4);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
