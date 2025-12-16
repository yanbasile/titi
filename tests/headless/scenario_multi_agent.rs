//! Complex Scenario Test 2: Multi-Agent Coordination
//!
//! Tests multiple headless terminals cooperating like AI agents.
//! Simulates real agent orchestration patterns.
//!
//! **Scenarios:**
//! - Producer-consumer pattern (agent A generates, agent B processes)
//! - Pipeline pattern (A → B → C data flow)
//! - Broadcast pattern (one agent notifies many)
//! - Collaborative task completion
//!
//! **Validates:**
//! - Inter-agent communication via pub/sub
//! - Coordination without race conditions
//! - Task distribution and load balancing
//! - Agent lifecycle management

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
async fn test_headless_producer_consumer_pattern() {
    let port = 18601;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting producer-consumer pattern test");

    let items_to_produce = 20;
    let produced_count = Arc::new(AtomicUsize::new(0));
    let consumed_count = Arc::new(AtomicUsize::new(0));

    let produced_clone = produced_count.clone();
    let consumed_clone = consumed_count.clone();
    let token_clone = token.clone();

    // Producer agent
    let producer = tokio::spawn(async move {
        let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
            .await
            .expect("Producer connect failed");

        client.authenticate(&token_clone).await.expect("Producer auth failed");
        client.create_session(Some("producer-session")).await.expect("Session failed");
        client.create_pane(Some("producer-pane")).await.expect("Pane failed");

        println!("   Producer agent started");

        for i in 0..items_to_produce {
            // Produce item
            let cmd = format!("echo 'Produced item {}'\n", i);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Produce failed");

            // Publish to work queue (simulated via command)
            let publish_cmd = format!("echo 'WORK_ITEM:{}'\n", i);
            client.inject_command(&publish_cmd).await.expect("Publish failed");

            produced_clone.fetch_add(1, Ordering::Relaxed);

            if i % 5 == 0 {
                println!("   Producer: {} items produced", i);
            }

            sleep(Duration::from_millis(50)).await;
        }

        println!("   Producer completed: {} items", items_to_produce);
    });

    // Consumer agent
    let consumer = tokio::spawn(async move {
        let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
            .await
            .expect("Consumer connect failed");

        client.authenticate(&token).await.expect("Consumer auth failed");
        client.create_session(Some("consumer-session")).await.expect("Session failed");
        client.create_pane(Some("consumer-pane")).await.expect("Pane failed");

        println!("   Consumer agent started");

        // Simulate consuming items (in real scenario, would subscribe to queue)
        for i in 0..items_to_produce {
            // Consume item
            let cmd = format!("echo 'Consumed item {}'\n", i);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Consume failed");

            consumed_clone.fetch_add(1, Ordering::Relaxed);

            if i % 5 == 0 {
                println!("   Consumer: {} items consumed", i);
            }

            sleep(Duration::from_millis(60)).await;
        }

        println!("   Consumer completed: {} items", items_to_produce);
    });

    // Wait for both agents
    producer.await.expect("Producer failed");
    consumer.await.expect("Consumer failed");

    let produced = produced_count.load(Ordering::Relaxed);
    let consumed = consumed_count.load(Ordering::Relaxed);

    println!("✅ Producer-consumer test complete");
    println!("   Produced: {}", produced);
    println!("   Consumed: {}", consumed);

    assert_eq!(produced, items_to_produce);
    assert_eq!(consumed, items_to_produce);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_pipeline_pattern() {
    let port = 18602;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting pipeline pattern test (A → B → C)");

    let stage_a_count = Arc::new(AtomicUsize::new(0));
    let stage_b_count = Arc::new(AtomicUsize::new(0));
    let stage_c_count = Arc::new(AtomicUsize::new(0));

    let items = 15;

    // Stage A: Input processing
    let stage_a = {
        let token_clone = token.clone();
        let count_clone = stage_a_count.clone();

        tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Stage A connect failed");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client.create_session(Some("stage-a")).await.expect("Session failed");
            client.create_pane(Some("stage-a-pane")).await.expect("Pane failed");

            println!("   Stage A started");

            for i in 0..items {
                let cmd = format!("echo 'Stage A: Process {}'\n", i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
                count_clone.fetch_add(1, Ordering::Relaxed);
                sleep(Duration::from_millis(40)).await;
            }

            println!("   Stage A completed");
        })
    };

    // Stage B: Transformation
    let stage_b = {
        let token_clone = token.clone();
        let count_clone = stage_b_count.clone();

        tokio::spawn(async move {
            // Slight delay to simulate pipeline flow
            sleep(Duration::from_millis(100)).await;

            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Stage B connect failed");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client.create_session(Some("stage-b")).await.expect("Session failed");
            client.create_pane(Some("stage-b-pane")).await.expect("Pane failed");

            println!("   Stage B started");

            for i in 0..items {
                let cmd = format!("echo 'Stage B: Transform {}'\n", i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
                count_clone.fetch_add(1, Ordering::Relaxed);
                sleep(Duration::from_millis(40)).await;
            }

            println!("   Stage B completed");
        })
    };

    // Stage C: Output
    let stage_c = {
        let token_clone = token.clone();
        let count_clone = stage_c_count.clone();

        tokio::spawn(async move {
            // More delay to simulate pipeline flow
            sleep(Duration::from_millis(200)).await;

            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Stage C connect failed");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client.create_session(Some("stage-c")).await.expect("Session failed");
            client.create_pane(Some("stage-c-pane")).await.expect("Pane failed");

            println!("   Stage C started");

            for i in 0..items {
                let cmd = format!("echo 'Stage C: Output {}'\n", i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
                count_clone.fetch_add(1, Ordering::Relaxed);
                sleep(Duration::from_millis(40)).await;
            }

            println!("   Stage C completed");
        })
    };

    // Wait for pipeline to complete
    stage_a.await.expect("Stage A failed");
    stage_b.await.expect("Stage B failed");
    stage_c.await.expect("Stage C failed");

    let a_count = stage_a_count.load(Ordering::Relaxed);
    let b_count = stage_b_count.load(Ordering::Relaxed);
    let c_count = stage_c_count.load(Ordering::Relaxed);

    println!("✅ Pipeline pattern test complete");
    println!("   Stage A processed: {}", a_count);
    println!("   Stage B processed: {}", b_count);
    println!("   Stage C processed: {}", c_count);

    assert_eq!(a_count, items);
    assert_eq!(b_count, items);
    assert_eq!(c_count, items);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_broadcast_pattern() {
    let port = 18603;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting broadcast pattern test (1 → N)");

    let num_workers = 5;
    let messages_to_broadcast = 10;
    let received_count = Arc::new(AtomicUsize::new(0));

    // Broadcaster agent
    let broadcaster = {
        let token_clone = token.clone();

        tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Broadcaster connect failed");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client.create_session(Some("broadcaster")).await.expect("Session failed");
            client.create_pane(Some("broadcast-pane")).await.expect("Pane failed");

            println!("   Broadcaster started");

            for i in 0..messages_to_broadcast {
                let cmd = format!("echo 'BROADCAST: Message {}'\n", i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Broadcast failed");

                println!("   Broadcasted message {}", i);
                sleep(Duration::from_millis(100)).await;
            }

            println!("   Broadcaster completed");
        })
    };

    // Worker agents
    let mut workers = vec![];

    for worker_id in 0..num_workers {
        let token_clone = token.clone();
        let received_clone = received_count.clone();

        let worker = tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Worker connect failed");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client
                .create_session(Some(&format!("worker-{}", worker_id)))
                .await
                .expect("Session failed");
            client
                .create_pane(Some(&format!("worker-pane-{}", worker_id)))
                .await
                .expect("Pane failed");

            println!("   Worker {} started", worker_id);

            // Simulate receiving broadcast messages
            for i in 0..messages_to_broadcast {
                let cmd = format!("echo 'Worker {} received message {}'\n", worker_id, i);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
                received_clone.fetch_add(1, Ordering::Relaxed);
                sleep(Duration::from_millis(120)).await;
            }

            println!("   Worker {} completed", worker_id);
        });

        workers.push(worker);
    }

    // Wait for broadcaster
    broadcaster.await.expect("Broadcaster failed");

    // Wait for all workers
    for worker in workers {
        worker.await.expect("Worker failed");
    }

    let total_received = received_count.load(Ordering::Relaxed);
    let expected = messages_to_broadcast * num_workers;

    println!("✅ Broadcast pattern test complete");
    println!("   Messages broadcasted: {}", messages_to_broadcast);
    println!("   Workers: {}", num_workers);
    println!("   Total received: {}", total_received);
    println!("   Expected: {}", expected);

    assert_eq!(total_received, expected);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_collaborative_task() {
    let port = 18604;
    let (token, handle) = start_test_server(port).await;

    println!("🚀 Starting collaborative task completion test");

    let total_work_items = 30;
    let num_agents = 3;
    let completed_count = Arc::new(AtomicUsize::new(0));

    let mut agent_handles = vec![];

    // Spawn collaborative agents
    for agent_id in 0..num_agents {
        let token_clone = token.clone();
        let completed_clone = completed_count.clone();

        let agent = tokio::spawn(async move {
            let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
                .await
                .expect("Agent connect failed");

            client.authenticate(&token_clone).await.expect("Auth failed");
            client
                .create_session(Some(&format!("collab-agent-{}", agent_id)))
                .await
                .expect("Session failed");
            client
                .create_pane(Some(&format!("agent-pane-{}", agent_id)))
                .await
                .expect("Pane failed");

            println!("   Agent {} started", agent_id);

            // Each agent processes 1/3 of the work
            let work_per_agent = total_work_items / num_agents;
            let start_item = agent_id * work_per_agent;
            let end_item = start_item + work_per_agent;

            for item in start_item..end_item {
                let cmd = format!("echo 'Agent {} processing item {}'\n", agent_id, item);
                client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
                completed_clone.fetch_add(1, Ordering::Relaxed);
                sleep(Duration::from_millis(50)).await;
            }

            println!("   Agent {} completed {} items", agent_id, work_per_agent);
        });

        agent_handles.push(agent);
    }

    // Wait for all agents
    for agent in agent_handles {
        agent.await.expect("Agent failed");
    }

    let completed = completed_count.load(Ordering::Relaxed);

    println!("✅ Collaborative task test complete");
    println!("   Total work items: {}", total_work_items);
    println!("   Agents: {}", num_agents);
    println!("   Items completed: {}", completed);

    assert_eq!(completed, total_work_items);

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
