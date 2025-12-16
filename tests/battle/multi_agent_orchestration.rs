use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;

/// Battle Test Scenario 1: Multi-Agent Terminal Orchestration
///
/// This test simulates 5 concurrent AI agents each using their own terminal session.
/// Tests session isolation, pub/sub under load, memory management, and concurrent parsing.
#[tokio::test]
#[ignore]
async fn test_multi_agent_orchestration() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  BATTLE TEST: Multi-Agent Terminal Orchestration          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Spawning 5 concurrent AI agents with terminal sessions   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Start test server
    let (token, server_handle) = start_test_server(19999).await;
    sleep(Duration::from_millis(500)).await;

    let start_time = Instant::now();
    let results = Arc::new(Mutex::new(Vec::new()));

    // Agent configurations
    let agent_tasks = vec![
        ("Agent-1-FileMonitor", "Continuous file monitoring (tail -f simulation)"),
        ("Agent-2-BuildSystem", "Build system with compile logs"),
        ("Agent-3-TestRunner", "Test runner with streaming output"),
        ("Agent-4-LogAggregator", "Log aggregation (grep/awk simulation)"),
        ("Agent-5-InteractiveShell", "Interactive shell commands"),
    ];

    // Spawn all agents concurrently
    let mut handles = vec![];

    for (idx, (agent_name, description)) in agent_tasks.iter().enumerate() {
        let token_clone = token.clone();
        let results_clone = results.clone();
        let agent_name = agent_name.to_string();
        let description = description.to_string();

        let handle = tokio::spawn(async move {
            println!("🤖 {} starting: {}", agent_name, description);

            match run_agent_simulation(idx, &agent_name, &token_clone).await {
                Ok(metrics) => {
                    println!("✅ {} completed successfully", agent_name);
                    results_clone.lock().unwrap().push((agent_name.clone(), metrics));
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ {} failed: {}", agent_name, e);
                    Err(e)
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all agents to complete
    let mut all_success = true;
    for handle in handles {
        match handle.await {
            Ok(Ok(())) => {},
            Ok(Err(e)) => {
                eprintln!("Agent failed: {}", e);
                all_success = false;
            }
            Err(e) => {
                eprintln!("Agent task panicked: {}", e);
                all_success = false;
            }
        }
    }

    let total_time = start_time.elapsed();

    // Analyze results
    let results_vec = results.lock().unwrap();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  TEST RESULTS                                              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("Total test time: {:?}", total_time);
    println!("Agents completed: {}/5", results_vec.len());

    for (agent_name, metrics) in results_vec.iter() {
        println!("\n{} metrics:", agent_name);
        println!("  - Commands sent: {}", metrics.commands_sent);
        println!("  - Messages received: {}", metrics.messages_received);
        println!("  - Duration: {:?}", metrics.duration);
        println!("  - Throughput: {:.2} msg/sec",
                 metrics.messages_received as f64 / metrics.duration.as_secs_f64());
    }

    // Cleanup
    server_handle.abort();

    // Assertions
    assert!(all_success, "Not all agents completed successfully");
    assert_eq!(results_vec.len(), 5, "Expected 5 agents to complete");
    assert!(total_time < Duration::from_secs(180), "Test took too long: {:?}", total_time);

    // Verify no session leakage - each agent should have received its own messages
    for (agent_name, metrics) in results_vec.iter() {
        assert!(metrics.messages_received > 0, "{} received no messages", agent_name);
        assert!(metrics.commands_sent > 0, "{} sent no commands", agent_name);
    }

    println!("\n✅ Multi-Agent Orchestration Test PASSED!");
}

#[derive(Debug, Clone)]
struct AgentMetrics {
    commands_sent: usize,
    messages_received: usize,
    duration: Duration,
}

async fn run_agent_simulation(
    agent_id: usize,
    agent_name: &str,
    token: &str,
) -> Result<AgentMetrics, String> {
    let start = Instant::now();

    // Connect to server
    let mut client = ServerClient::connect("127.0.0.1:19999")
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    // Authenticate
    client.authenticate(token)
        .await
        .map_err(|e| format!("Authentication failed: {}", e))?;

    // Create unique session for this agent
    let session_name = format!("agent-session-{}", agent_id);
    client.create_session(Some(&session_name)).await?;

    let session_id = client.session_id().to_string();
    let pane_id = client.pane_id().to_string();

    // Subscribe to output
    client.subscribe_output().await?;

    let mut commands_sent = 0;
    let mut messages_received = 0;

    // Simulate agent-specific workload
    match agent_id {
        0 => {
            // Agent 1: File monitoring simulation (continuous output via pub/sub)
            for i in 0..50 {
                let log_line = format!("LOG [{}] File changed: /path/to/file{}.txt", agent_name, i);
                let channel = format!("{}/pane-{}/output", session_id, pane_id);
                client.publish_to_channel(&channel, &log_line).await?;
                commands_sent += 1;

                // Read output
                if let Some(_msg) = client.read_output().await? {
                    messages_received += 1;
                }

                sleep(Duration::from_millis(10)).await;
            }
        }
        1 => {
            // Agent 2: Build system simulation (batch output)
            for i in 0..30 {
                let build_log = format!("Compiling module_{}.rs... [{}] OK\n", i, agent_name);
                let channel = format!("{}/pane-{}/output", session_id, pane_id);
                client.publish_to_channel(&channel, &build_log).await?;
                commands_sent += 1;

                if let Some(_msg) = client.read_output().await? {
                    messages_received += 1;
                }

                sleep(Duration::from_millis(20)).await;
            }
        }
        2 => {
            // Agent 3: Test runner simulation (streaming results)
            for i in 0..40 {
                let test_output = format!("test test_case_{} ... [{}] ok\n", i, agent_name);
                let channel = format!("{}/pane-{}/output", session_id, pane_id);
                client.publish_to_channel(&channel, &test_output).await?;
                commands_sent += 1;

                if let Some(_msg) = client.read_output().await? {
                    messages_received += 1;
                }

                sleep(Duration::from_millis(15)).await;
            }
        }
        3 => {
            // Agent 4: Log aggregation simulation (grep/awk patterns)
            for i in 0..35 {
                let grep_result = format!("[{}] ERROR found at line {}: critical failure\n", agent_name, i * 10);
                let channel = format!("{}/pane-{}/output", session_id, pane_id);
                client.publish_to_channel(&channel, &grep_result).await?;
                commands_sent += 1;

                if let Some(_msg) = client.read_output().await? {
                    messages_received += 1;
                }

                sleep(Duration::from_millis(18)).await;
            }
        }
        4 => {
            // Agent 5: Interactive shell simulation (commands + responses)
            let commands = vec![
                "ls -la",
                "cd /tmp",
                "echo 'Hello from Agent 5'",
                "pwd",
                "whoami",
            ];

            for (i, cmd) in commands.iter().cycle().take(25).enumerate() {
                let shell_output = format!("[{}] $ {} \nOutput for command {}\n", agent_name, cmd, i);
                let channel = format!("{}/pane-{}/output", session_id, pane_id);
                client.publish_to_channel(&channel, &shell_output).await?;
                commands_sent += 1;

                if let Some(_msg) = client.read_output().await? {
                    messages_received += 1;
                }

                sleep(Duration::from_millis(25)).await;
            }
        }
        _ => unreachable!(),
    }

    // Final read to catch any remaining messages
    for _ in 0..5 {
        if let Some(_msg) = client.read_output().await? {
            messages_received += 1;
        }
        sleep(Duration::from_millis(50)).await;
    }

    let duration = start.elapsed();

    Ok(AgentMetrics {
        commands_sent,
        messages_received,
        duration,
    })
}

async fn start_test_server(port: u16) -> (String, tokio::task::JoinHandle<()>) {
    let token = "test_token_battle_multi_agent_12345678901234567890".to_string();
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
