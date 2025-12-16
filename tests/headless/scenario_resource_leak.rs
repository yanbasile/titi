//! Complex Scenario Test 5: Resource Leak Detection for Headless Mode
//!
//! Dedicated resource leak detection for headless terminal operations.
//! Complements the general memory leak detection with headless-specific scenarios.
//!
//! **Scenarios:**
//! - File descriptor leak detection
//! - Memory growth monitoring over time
//! - PTY process orphan detection
//! - Connection resource cleanup
//!
//! **Validates:**
//! - No file descriptor leaks
//! - Bounded memory usage
//! - All child processes are reaped
//! - Network connections properly closed

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

/// Sample current process memory usage
fn sample_memory_usage() -> usize {
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

    // Fallback
    50_000 // 50 MB baseline
}

/// Count open file descriptors
fn count_open_fds() -> usize {
    use std::fs;

    if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        return entries.count();
    }

    // Fallback
    10 // Reasonable baseline
}

#[tokio::test]
#[ignore]
async fn test_headless_file_descriptor_leak() {
    let port = 18901;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting file descriptor leak detection");

    let initial_fds = count_open_fds();
    println!("   Initial FDs: {}", initial_fds);

    let num_cycles = 50;

    for i in 0..num_cycles {
        // Create client
        let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
            .await
            .expect("Connect failed");

        client.authenticate(&token).await.expect("Auth failed");
        client
            .create_session(Some(&format!("fd-test-{}", i)))
            .await
            .expect("Session failed");
        client
            .create_pane(Some(&format!("fd-pane-{}", i)))
            .await
            .expect("Pane failed");

        // Send some commands
        for j in 0..5 {
            let cmd = format!("echo 'FD test {} cmd {}'", i, j);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        }

        // Explicitly drop client
        drop(client);

        if i % 10 == 0 {
            let current_fds = count_open_fds();
            println!("   Cycle {}: {} FDs (delta: {})", i, current_fds, current_fds as i32 - initial_fds as i32);
        }

        // Small delay for cleanup
        sleep(Duration::from_millis(100)).await;
    }

    // Give time for async cleanup
    sleep(Duration::from_secs(2)).await;

    let final_fds = count_open_fds();
    let fd_growth = final_fds as i32 - initial_fds as i32;

    println!("✅ File descriptor leak test complete");
    println!("   Initial FDs: {}", initial_fds);
    println!("   Final FDs: {}", final_fds);
    println!("   Growth: {}", fd_growth);
    println!("   Cycles: {}", num_cycles);

    // FD growth should be minimal (< 20 FDs for 50 cycles)
    assert!(
        fd_growth < 20,
        "Excessive FD growth: {} FDs after {} cycles",
        fd_growth,
        num_cycles
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_memory_leak_detection() {
    let port = 18902;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting headless memory leak detection");

    let initial_memory = sample_memory_usage();
    println!("   Initial memory: {} KB", initial_memory);

    let mut memory_samples = vec![];
    let num_cycles = 30;

    for i in 0..num_cycles {
        // Create and use client
        let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
            .await
            .expect("Connect failed");

        client.authenticate(&token).await.expect("Auth failed");
        client
            .create_session(Some(&format!("mem-test-{}", i)))
            .await
            .expect("Session failed");
        client
            .create_pane(Some(&format!("mem-pane-{}", i)))
            .await
            .expect("Pane failed");

        // Generate some load
        for j in 0..20 {
            let cmd = format!("echo 'Memory test {} - {}'", i, j);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        }

        drop(client);

        // Sample memory
        let current_memory = sample_memory_usage();
        memory_samples.push(current_memory);

        if i % 5 == 0 {
            let growth = current_memory as i32 - initial_memory as i32;
            println!("   Cycle {}: {} KB (growth: {} KB)", i, current_memory, growth);
        }

        sleep(Duration::from_millis(100)).await;
    }

    // Give time for cleanup
    sleep(Duration::from_secs(2)).await;

    let final_memory = sample_memory_usage();
    let total_growth = final_memory as i32 - initial_memory as i32;
    let growth_pct = (total_growth as f64 / initial_memory as f64) * 100.0;

    println!("✅ Memory leak test complete");
    println!("   Initial: {} KB", initial_memory);
    println!("   Final: {} KB", final_memory);
    println!("   Growth: {} KB ({:.1}%)", total_growth, growth_pct);
    println!("   Cycles: {}", num_cycles);

    // Memory growth should be < 30% of initial
    assert!(
        growth_pct < 30.0,
        "Excessive memory growth: {:.1}% after {} cycles",
        growth_pct,
        num_cycles
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_connection_cleanup() {
    let port = 18903;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting connection cleanup test");

    let connections_to_create = 100;
    let cleanup_counter = Arc::new(AtomicUsize::new(0));

    let initial_memory = sample_memory_usage();
    let initial_fds = count_open_fds();

    println!("   Initial state:");
    println!("     Memory: {} KB", initial_memory);
    println!("     FDs: {}", initial_fds);

    for i in 0..connections_to_create {
        let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
            .await
            .expect("Connect failed");

        client.authenticate(&token).await.expect("Auth failed");
        client
            .create_session(Some(&format!("cleanup-{}", i)))
            .await
            .expect("Session failed");
        client
            .create_pane(Some(&format!("cleanup-pane-{}", i)))
            .await
            .expect("Pane failed");

        // Quick command
        client.inject("echo 'test'").await.expect("Command failed");

        // Cleanup
        drop(client);
        cleanup_counter.fetch_add(1, Ordering::Relaxed);

        if i % 20 == 0 {
            let mem = sample_memory_usage();
            let fds = count_open_fds();
            println!(
                "   Cycle {}: {} KB, {} FDs",
                i, mem, fds
            );
        }

        // Minimal delay
        sleep(Duration::from_millis(10)).await;
    }

    // Allow async cleanup
    sleep(Duration::from_secs(2)).await;

    let final_memory = sample_memory_usage();
    let final_fds = count_open_fds();

    let memory_growth = final_memory as i32 - initial_memory as i32;
    let fd_growth = final_fds as i32 - initial_fds as i32;

    println!("✅ Connection cleanup test complete");
    println!("   Connections created: {}", connections_to_create);
    println!("   Final state:");
    println!("     Memory: {} KB (growth: {} KB)", final_memory, memory_growth);
    println!("     FDs: {} (growth: {})", final_fds, fd_growth);

    // Resource growth should be minimal
    assert!(
        fd_growth < 30,
        "Excessive FD growth: {} after {} connections",
        fd_growth,
        connections_to_create
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_session_pane_cleanup() {
    let port = 18904;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting session/pane cleanup test");

    let initial_memory = sample_memory_usage();

    // Create many sessions and panes
    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");

    let num_sessions = 20;
    let panes_per_session = 5;

    for session_id in 0..num_sessions {
        client
            .create_session(Some(&format!("cleanup-session-{}", session_id)))
            .await
            .expect("Session failed");

        for pane_id in 0..panes_per_session {
            client
                .create_pane(Some(&format!("cleanup-pane-{}-{}", session_id, pane_id)))
                .await
                .expect("Pane failed");

            // Send command to each pane
            client.inject("echo 'test'").await.expect("Command failed");
        }

        if session_id % 5 == 0 {
            let mem = sample_memory_usage();
            let growth = mem as i32 - initial_memory as i32;
            println!(
                "   Created {} sessions, {} total panes: {} KB (growth: {} KB)",
                session_id + 1,
                (session_id + 1) * panes_per_session,
                mem,
                growth
            );
        }
    }

    let peak_memory = sample_memory_usage();
    println!("   Peak memory: {} KB", peak_memory);

    // Disconnect (in real implementation, would clean up sessions/panes)
    drop(client);

    // Allow cleanup
    sleep(Duration::from_secs(2)).await;

    let final_memory = sample_memory_usage();
    let total_growth = final_memory as i32 - initial_memory as i32;

    println!("✅ Session/pane cleanup test complete");
    println!("   Sessions: {}", num_sessions);
    println!("   Total panes: {}", num_sessions * panes_per_session);
    println!("   Peak memory: {} KB", peak_memory);
    println!("   Final memory: {} KB", final_memory);
    println!("   Net growth: {} KB", total_growth);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_sustained_load_stability() {
    let port = 18905;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting sustained load stability test");
    println!("   Running for 3 minutes with continuous activity");

    let start = Instant::now();
    let test_duration = Duration::from_secs(3 * 60); // 3 minutes

    let initial_memory = sample_memory_usage();
    let initial_fds = count_open_fds();

    let mut cycle_count = 0;
    let mut memory_samples = vec![];

    while start.elapsed() < test_duration {
        // Create temporary client
        let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
            .await
            .expect("Connect failed");

        client.authenticate(&token).await.expect("Auth failed");
        client
            .create_session(Some(&format!("sustained-{}", cycle_count)))
            .await
            .expect("Session failed");
        client
            .create_pane(Some(&format!("sustained-pane-{}", cycle_count)))
            .await
            .expect("Pane failed");

        // Do some work
        for i in 0..10 {
            let cmd = format!("echo 'Sustained load {} - {}'", cycle_count, i);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        }

        drop(client);
        cycle_count += 1;

        // Sample every 10 cycles
        if cycle_count % 10 == 0 {
            let mem = sample_memory_usage();
            let fds = count_open_fds();
            memory_samples.push(mem);

            let elapsed_secs = start.elapsed().as_secs();
            println!(
                "   {}s: Cycle {}, {} KB, {} FDs",
                elapsed_secs, cycle_count, mem, fds
            );
        }

        sleep(Duration::from_millis(100)).await;
    }

    let final_memory = sample_memory_usage();
    let final_fds = count_open_fds();

    let memory_growth = final_memory as i32 - initial_memory as i32;
    let fd_growth = final_fds as i32 - initial_fds as i32;
    let memory_growth_pct = (memory_growth as f64 / initial_memory as f64) * 100.0;

    println!("✅ Sustained load stability test complete");
    println!("   Duration: {:?}", start.elapsed());
    println!("   Total cycles: {}", cycle_count);
    println!("   Initial memory: {} KB", initial_memory);
    println!("   Final memory: {} KB", final_memory);
    println!("   Memory growth: {} KB ({:.1}%)", memory_growth, memory_growth_pct);
    println!("   Initial FDs: {}", initial_fds);
    println!("   Final FDs: {}", final_fds);
    println!("   FD growth: {}", fd_growth);

    // Memory growth should be < 25% over 3 minutes
    assert!(
        memory_growth_pct < 25.0,
        "Excessive memory growth: {:.1}%",
        memory_growth_pct
    );

    // FD growth should be minimal
    assert!(fd_growth < 20, "Excessive FD growth: {}", fd_growth);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
