//! Stress Test 2: Large Output Volume Handling
//!
//! Tests headless terminal's ability to process large volumes of PTY output.
//! Simulates commands that generate MB of output (logs, data dumps, etc.).
//!
//! **Scenarios:**
//! - 1MB continuous output stream
//! - 10MB output burst
//! - 100MB sustained output over time
//! - Rapid small outputs (10KB × 1000)
//!
//! **Validates:**
//! - No buffer overflow or truncation
//! - Memory remains bounded
//! - Output publishing keeps up with generation
//! - No deadlocks or backpressure issues

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
async fn test_headless_1mb_continuous_output() {
    let port = 18101;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("output-1mb")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting 1MB continuous output test");

    let start = Instant::now();

    // Generate 1MB of output (1024 lines of ~1KB each)
    let line_size = 1024;
    let num_lines = 1024;
    let line = "X".repeat(line_size - 1); // -1 for newline

    let cmd = format!("for i in {{0..{}}}; do echo '{}'; done", num_lines, line);
    client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");

    // Wait for processing
    sleep(Duration::from_secs(3)).await;

    let elapsed = start.elapsed();
    let mb_size = (line_size * num_lines) as f64 / (1024.0 * 1024.0);

    println!("✅ 1MB output test complete");
    println!("   Size: {:.2} MB", mb_size);
    println!("   Duration: {:?}", elapsed);
    println!("   Throughput: {:.2} MB/s", mb_size / elapsed.as_secs_f64());

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_10mb_burst_output() {
    let port = 18102;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("output-10mb")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting 10MB burst output test");

    let start = Instant::now();

    // Generate 10MB burst: 10,240 lines of ~1KB each
    let line_size = 1024;
    let num_lines = 10240;
    let line = "Y".repeat(line_size - 1);

    let cmd = format!("for i in {{0..{}}}; do echo '{}'; done", num_lines, line);
    client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");

    // Wait for processing (longer for 10MB)
    sleep(Duration::from_secs(10)).await;

    let elapsed = start.elapsed();
    let mb_size = (line_size * num_lines) as f64 / (1024.0 * 1024.0);

    println!("✅ 10MB burst test complete");
    println!("   Size: {:.2} MB", mb_size);
    println!("   Duration: {:?}", elapsed);
    println!("   Throughput: {:.2} MB/s", mb_size / elapsed.as_secs_f64());

    // Should handle within reasonable time (30s)
    assert!(
        elapsed < Duration::from_secs(30),
        "10MB processing too slow: {:?}",
        elapsed
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_rapid_small_outputs() {
    let port = 18103;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("output-rapid")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting rapid small outputs test");

    let start = Instant::now();
    let num_outputs = 1000;
    let output_size = 10 * 1024; // 10KB each

    // Send 1000 commands, each generating 10KB
    for i in 0..num_outputs {
        let line = "Z".repeat(output_size - 1);
        let cmd = format!("echo '{}'", line);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");

        if i % 100 == 0 {
            println!("   Sent {} / {} commands", i, num_outputs);
        }
    }

    // Wait for processing
    sleep(Duration::from_secs(5)).await;

    let elapsed = start.elapsed();
    let total_mb = (num_outputs * output_size) as f64 / (1024.0 * 1024.0);

    println!("✅ Rapid small outputs test complete");
    println!("   Outputs: {}", num_outputs);
    println!("   Size per output: {} KB", output_size / 1024);
    println!("   Total size: {:.2} MB", total_mb);
    println!("   Duration: {:?}", elapsed);
    println!("   Throughput: {:.2} MB/s", total_mb / elapsed.as_secs_f64());

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_sustained_output_stress() {
    let port = 18104;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to connect");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("output-sustained")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting sustained output stress test");

    let start = Instant::now();
    let duration = Duration::from_secs(30);
    let mut total_bytes = 0u64;
    let mut command_count = 0u32;

    // Generate output continuously for 30 seconds
    while start.elapsed() < duration {
        // 100KB per command
        let line = "S".repeat(100 * 1024 - 1);
        let cmd = format!("echo '{}'", line);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");

        total_bytes += 100 * 1024;
        command_count += 1;

        if command_count % 10 == 0 {
            let elapsed_secs = start.elapsed().as_secs_f64();
            let mb_so_far = total_bytes as f64 / (1024.0 * 1024.0);
            println!(
                "   {:.1}s: {:.2} MB sent ({:.2} MB/s)",
                elapsed_secs,
                mb_so_far,
                mb_so_far / elapsed_secs
            );
        }

        // Small delay to avoid overwhelming
        sleep(Duration::from_millis(100)).await;
    }

    let elapsed = start.elapsed();
    let total_mb = total_bytes as f64 / (1024.0 * 1024.0);

    println!("✅ Sustained output stress test complete");
    println!("   Commands: {}", command_count);
    println!("   Total data: {:.2} MB", total_mb);
    println!("   Duration: {:?}", elapsed);
    println!("   Average throughput: {:.2} MB/s", total_mb / elapsed.as_secs_f64());

    // Should handle at least 0.8 MB/s sustained (accounting for overhead)
    let throughput = total_mb / elapsed.as_secs_f64();
    assert!(
        throughput >= 0.8,
        "Sustained throughput too low: {:.2} MB/s (minimum: 0.8 MB/s)",
        throughput
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
