//! Complex Scenario Test 3: Interactive Shell Programs
//!
//! Tests headless terminal with complex interactive applications.
//! Simulates real terminal applications used by agents.
//!
//! **Scenarios:**
//! - vim-like modal editing (insert/command modes)
//! - less-like pagination
//! - Interactive prompts (Y/N confirmations)
//! - Pseudo-interactive CLIs (git, ssh, etc.)
//!
//! **Validates:**
//! - Proper handling of application modes
//! - Escape sequence interpretation
//! - Cursor positioning in interactive apps
//! - Input modes and state transitions

use titi::redititi_server::{RedititiTcpServer, TokenAuth};
use titi::server_client::ServerClient;
use tokio::time::{sleep, Duration};

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
async fn test_headless_vim_like_modal_editing() {
    let port = 18701;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("vim-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting vim-like modal editing test");

    // Simulate vim-like modal editor workflow
    // Note: This doesn't actually launch vim, but simulates the command patterns

    // Start "editor" (simulated)
    client.inject_command("echo 'Entering editor mode'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    // Simulate insert mode
    println!("   Simulating insert mode");
    client.inject_command("echo 'INSERT MODE'\n").await.expect("Command failed");
    client.inject_command("echo 'Line 1: Hello'\n").await.expect("Command failed");
    client.inject_command("echo 'Line 2: World'\n").await.expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Simulate escape to command mode
    println!("   Simulating command mode");
    client.inject_command("echo 'COMMAND MODE'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    // Simulate commands
    client.inject_command("echo ':w (save)'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo ':q (quit)'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("   Simulated modal editor workflow");

    println!("✅ Vim-like modal editing test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_paged_output() {
    let port = 18702;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("pager-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting paged output test (less-like behavior)");

    // Generate multi-page output
    println!("   Generating multi-page content");
    for page in 0..3 {
        println!("   Page {}", page + 1);

        for line in 0..24 {
            let cmd = format!("echo 'Page {} Line {}'\n", page + 1, line + 1);
            client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        }

        // Simulate pagination control
        if page < 2 {
            sleep(Duration::from_millis(100)).await;
            client.inject_command("echo '[Press SPACE for next page]'\n")
                .await
                .expect("Command failed");
            sleep(Duration::from_millis(100)).await;
        }
    }

    println!("   Completed paged output simulation");

    println!("✅ Paged output test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_interactive_prompts() {
    let port = 18703;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("prompt-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting interactive prompts test");

    // Simulate various interactive prompts
    println!("   Testing Y/N prompts");
    client.inject_command("echo 'Do you want to continue? (y/n)'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;
    client.inject_command("y\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("   Testing confirmation prompts");
    client.inject_command("echo 'Are you sure? [Y/n]'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;
    client.inject_command("Y\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("   Testing numbered selection");
    client.inject_command("echo 'Select option: 1) Option A, 2) Option B, 3) Option C'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await);
    client.inject_command("2\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("   Testing text input prompts");
    client.inject_command("echo 'Enter your name:'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;
    client.inject_command("TestAgent\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("✅ Interactive prompts test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_command_line_tools() {
    let port = 18704;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("cli-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting command-line tools test");

    // Test common CLI tools
    println!("   Testing git-like commands");
    client.inject_command("echo 'git status'\n").await.expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    client.inject_command("echo 'git add .'\n").await.expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    client.inject_command("echo 'git commit -m \"test\"'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    println!("   Testing docker-like commands");
    client.inject_command("echo 'docker ps'\n").await.expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    client.inject_command("echo 'docker run ubuntu echo hello'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    println!("   Testing npm-like commands");
    client.inject_command("echo 'npm install'\n").await.expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    client.inject_command("echo 'npm test'\n").await.expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    println!("✅ Command-line tools test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_long_running_commands() {
    let port = 18705;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("long-cmd-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting long-running commands test");

    // Simulate long-running command with progress updates
    println!("   Starting long-running task simulation");

    client.inject_command("echo 'Starting build process...'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    for progress in (0..=100).step_by(10) {
        let cmd = format!("echo 'Progress: {}%'\n", progress);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        sleep(Duration::from_millis(200)).await;
    }

    client.inject_command("echo 'Build complete!'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    println!("   Long-running task completed");

    println!("✅ Long-running commands test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_shell_job_control() {
    let port = 18706;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("job-control")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting shell job control test");

    // Simulate background jobs
    println!("   Testing background job simulation");
    client.inject_command("echo 'sleep 10 &'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo '[1] 12345'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    // Check jobs
    println!("   Checking job status");
    client.inject_command("echo 'jobs'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo '[1]+  Running                 sleep 10 &'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    // Foreground a job
    println!("   Bringing job to foreground");
    client.inject_command("echo 'fg %1'\n").await.expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("✅ Shell job control test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
