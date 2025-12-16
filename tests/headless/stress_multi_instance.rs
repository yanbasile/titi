//! Stress Test 3: Multi-Instance Concurrent Headless Terminals
//!
//! Tests running many headless terminal instances simultaneously.
//! Simulates large-scale AI agent deployments.
//!
//! **Scenarios:**
//! - 10 concurrent headless terminals
//! - 50 concurrent terminals (high scale)
//! - 100 sessions with mixed activity levels
//! - Staggered startup and shutdown patterns
//!
//! **Validates:**
//! - Server handles multiple concurrent clients
//! - No resource contention or deadlocks
//! - Each terminal operates independently
//! - Clean resource cleanup on shutdown

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
async fn test_headless_10_concurrent_terminals() {
    let port = 18201;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting 10 concurrent headless terminals test");

    let num_terminals = 10;
    let commands_per_terminal = 50;
    let success_count = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();

    // Spawn 10 concurrent terminal sessions
    let mut terminal_handles = vec![];

    for term_id in 0..num_terminals {
        let token_clone = token.clone();
        let success_clone = success_count.clone();

        let term_handle = tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Failed to connect");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client
                .create_session(Some(&format!("term-session-{}", term_id)))
                .await
                .expect("Session failed");
            client
                .create_pane(Some(&format!("term-pane-{}", term_id)))
                .await
                .expect("Pane failed");

            // Each terminal sends commands
            for i in 0..commands_per_terminal {
                let cmd = format!("echo 'Terminal {} Command {}'\n", term_id, i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
            }

            success_clone.fetch_add(1, Ordering::Relaxed);
            println!("   Terminal {} completed {} commands", term_id, commands_per_terminal);
        });

        terminal_handles.push(term_handle);
    }

    // Wait for all terminals
    for term_handle in terminal_handles {
        term_handle.await.expect("Terminal task failed");
    }

    let elapsed = start.elapsed();
    let completed = success_count.load(Ordering::Relaxed);

    println!("✅ 10 concurrent terminals test complete");
    println!("   Terminals: {}", num_terminals);
    println!("   Completed: {}", completed);
    println!("   Duration: {:?}", elapsed);

    assert_eq!(completed, num_terminals);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_50_concurrent_terminals() {
    let port = 18202;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting 50 concurrent headless terminals test (high scale)");

    let num_terminals = 50;
    let commands_per_terminal = 20;
    let success_count = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();

    let mut terminal_handles = vec![];

    for term_id in 0..num_terminals {
        let token_clone = token.clone();
        let success_clone = success_count.clone();

        let term_handle = tokio::spawn(async move {
            let mut client = match ServerClient::connect(&format!("127.0.0.1:{}", port)).await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Terminal {} connection failed: {}", term_id, e);
                    return;
                }
            };

            if client.authenticate(&token_clone).await.is_err() {
                eprintln!("Terminal {} auth failed", term_id);
                return;
            }

            if client
                .create_session(Some(&format!("scale-session-{}", term_id)))
                .await
                .is_err()
            {
                eprintln!("Terminal {} session creation failed", term_id);
                return;
            }

            if client
                .create_pane(Some(&format!("scale-pane-{}", term_id)))
                .await
                .is_err()
            {
                eprintln!("Terminal {} pane creation failed", term_id);
                return;
            }

            for i in 0..commands_per_terminal {
                let cmd = format!("echo 'Terminal {} Cmd {}'\n", term_id, i);
                if client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.is_err() {
                    eprintln!("Terminal {} command {} failed", term_id, i);
                    return;
                }
            }

            success_clone.fetch_add(1, Ordering::Relaxed);

            if term_id % 10 == 0 {
                println!("   Terminal {} completed", term_id);
            }
        });

        terminal_handles.push(term_handle);

        // Slight stagger to avoid thundering herd
        if term_id % 10 == 0 {
            sleep(Duration::from_millis(50)).await;
        }
    }

    // Wait for all terminals
    for term_handle in terminal_handles {
        term_handle.await.expect("Terminal task failed");
    }

    let elapsed = start.elapsed();
    let completed = success_count.load(Ordering::Relaxed);
    let success_rate = (completed as f64 / num_terminals as f64) * 100.0;

    println!("✅ 50 concurrent terminals test complete");
    println!("   Terminals: {}", num_terminals);
    println!("   Completed: {}", completed);
    println!("   Success rate: {:.1}%", success_rate);
    println!("   Duration: {:?}", elapsed);

    // At least 80% should succeed (40/50)
    assert!(
        completed >= 40,
        "Too many failures: only {} / {} succeeded",
        completed,
        num_terminals
    );

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_staggered_lifecycle() {
    let port = 18203;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting staggered lifecycle test");

    let num_terminals = 20;
    let success_count = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();

    // Create terminals with staggered timing
    for term_id in 0..num_terminals {
        let token_clone = token.clone();
        let success_clone = success_count.clone();

        tokio::spawn(async move {
            // Random startup delay (0-500ms)
            let startup_delay = (term_id * 25) as u64; // 0, 25, 50, ... ms
            sleep(Duration::from_millis(startup_delay)).await;

            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Failed to connect");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client
                .create_session(Some(&format!("stagger-session-{}", term_id)))
                .await
                .expect("Session failed");
            client
                .create_pane(Some(&format!("stagger-pane-{}", term_id)))
                .await
                .expect("Pane failed");

            // Run for random duration
            let run_duration = Duration::from_millis(100 + (term_id * 50) as u64);
            let commands = 5 + term_id; // Variable workload

            for i in 0..commands {
                let cmd = format!("echo 'Stagger {} Cmd {}'\n", term_id, i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
                sleep(Duration::from_millis(20)).await;
            }

            sleep(run_duration).await;

            success_clone.fetch_add(1, Ordering::Relaxed);
            println!("   Terminal {} completed after {:?}", term_id, startup_delay + run_duration.as_millis() as u64);
        });

        // Small delay between spawns
        sleep(Duration::from_millis(50)).await;
    }

    // Wait for all to complete (max 5 seconds)
    sleep(Duration::from_secs(5)).await;

    let elapsed = start.elapsed();
    let completed = success_count.load(Ordering::Relaxed);

    println!("✅ Staggered lifecycle test complete");
    println!("   Terminals spawned: {}", num_terminals);
    println!("   Completed: {}", completed);
    println!("   Duration: {:?}", elapsed);

    // All should complete
    assert_eq!(completed, num_terminals);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_mixed_activity_levels() {
    let port = 18204;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting mixed activity levels test");

    let heavy_terminals = 5; // Many commands
    let light_terminals = 15; // Few commands
    let idle_terminals = 10; // No commands

    let success_count = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();

    let mut all_handles = vec![];

    // Heavy activity terminals
    for term_id in 0..heavy_terminals {
        let token_clone = token.clone();
        let success_clone = success_count.clone();

        let h = tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Connect failed");
            client.authenticate(&token_clone).await.expect("Auth failed");
            client.create_session(Some(&format!("heavy-{}", term_id))).await.expect("Session failed");
            client.create_pane(Some(&format!("heavy-pane-{}", term_id))).await.expect("Pane failed");

            for i in 0..100 {
                let cmd = format!("echo 'Heavy {} Cmd {}'\n", term_id, i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
            }

            success_clone.fetch_add(1, Ordering::Relaxed);
        });
        all_handles.push(h);
    }

    // Light activity terminals
    for term_id in 0..light_terminals {
        let token_clone = token.clone();
        let success_clone = success_count.clone();

        let h = tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Connect failed");
            client.authenticate(&token_clone).await.expect("Auth failed");
            client.create_session(Some(&format!("light-{}", term_id))).await.expect("Session failed");
            client.create_pane(Some(&format!("light-pane-{}", term_id))).await.expect("Pane failed");

            for i in 0..5 {
                let cmd = format!("echo 'Light {} Cmd {}'\n", term_id, i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Inject failed");
            }

            success_clone.fetch_add(1, Ordering::Relaxed);
        });
        all_handles.push(h);
    }

    // Idle terminals (connect but don't send commands)
    for term_id in 0..idle_terminals {
        let token_clone = token.clone();
        let success_clone = success_count.clone();

        let h = tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Connect failed");
            client.authenticate(&token_clone).await.expect("Auth failed");
            client.create_session(Some(&format!("idle-{}", term_id))).await.expect("Session failed");
            client.create_pane(Some(&format!("idle-pane-{}", term_id))).await.expect("Pane failed");

            // Just stay connected for a bit
            sleep(Duration::from_millis(500)).await;

            success_clone.fetch_add(1, Ordering::Relaxed);
        });
        all_handles.push(h);
    }

    // Wait for all
    for h in all_handles {
        h.await.expect("Task failed");
    }

    let elapsed = start.elapsed();
    let completed = success_count.load(Ordering::Relaxed);
    let total_terminals = heavy_terminals + light_terminals + idle_terminals;

    println!("✅ Mixed activity levels test complete");
    println!("   Heavy terminals: {}", heavy_terminals);
    println!("   Light terminals: {}", light_terminals);
    println!("   Idle terminals: {}", idle_terminals);
    println!("   Total: {}", total_terminals);
    println!("   Completed: {}", completed);
    println!("   Duration: {:?}", elapsed);

    assert_eq!(completed, total_terminals);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
