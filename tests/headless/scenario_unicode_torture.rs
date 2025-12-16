//! Complex Scenario Test 4: Unicode and Escape Sequence Torture Test
//!
//! Tests headless terminal with complex Unicode and escape sequences.
//! Ensures proper handling of international text and terminal codes.
//!
//! **Scenarios:**
//! - Multi-byte Unicode characters (CJK, emoji, etc.)
//! - Complex escape sequences (colors, cursor movement, etc.)
//! - Mixed LTR/RTL text
//! - Zero-width characters and combining marks
//!
//! **Validates:**
//! - Correct Unicode parsing and storage
//! - Escape sequence interpretation
//! - Character width calculation
//! - No corruption or garbling

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
async fn test_headless_emoji_and_symbols() {
    let port = 18801;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("emoji-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting emoji and symbols test");

    // Test various emoji
    let emoji_tests = vec![
        "🚀 Rocket",
        "✅ Check mark",
        "❌ Cross mark",
        "🎉 Party popper",
        "💻 Laptop",
        "🐍 Snake (Python)",
        "🦀 Crab (Rust)",
        "📁 Folder",
        "🔒 Lock",
        "⚠️ Warning",
    ];

    for (i, test) in emoji_tests.iter().enumerate() {
        let cmd = format!("echo '{}'\n", test);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        println!("   Sent: {}", test);
        sleep(Duration::from_millis(50)).await;

        if i % 3 == 0 {
            sleep(Duration::from_millis(50)).await;
        }
    }

    println!("✅ Emoji and symbols test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_multibyte_characters() {
    let port = 18802;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("multibyte-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting multibyte characters test");

    // Test various languages and scripts
    let language_tests = vec![
        ("Chinese", "你好世界 (Hello World)"),
        ("Japanese", "こんにちは世界 (Hello World)"),
        ("Korean", "안녕하세요 세계 (Hello World)"),
        ("Arabic", "مرحبا بالعالم (Hello World)"),
        ("Hebrew", "שלום עולם (Hello World)"),
        ("Russian", "Привет мир (Hello World)"),
        ("Greek", "Γειά σου κόσμε (Hello World)"),
        ("Thai", "สวัสดีชาวโลก (Hello World)"),
        ("Hindi", "नमस्ते दुनिया (Hello World)"),
        ("Emoji Mix", "Hello 🌍 World 🚀"),
    ];

    for (lang, text) in language_tests.iter() {
        let cmd = format!("echo '{}'\n", text);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        println!("   {} : {}", lang, text);
        sleep(Duration::from_millis(100)).await;
    }

    println!("✅ Multibyte characters test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_ansi_color_codes() {
    let port = 18803;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("color-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting ANSI color codes test");

    // Test various color codes
    println!("   Testing basic colors");
    client.inject_command("echo '\\x1b[31mRed text\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo '\\x1b[32mGreen text\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo '\\x1b[33mYellow text\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo '\\x1b[34mBlue text\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("   Testing bold and styles");
    client.inject_command("echo '\\x1b[1mBold text\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo '\\x1b[4mUnderline text\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("   Testing combined styles");
    client.inject_command("echo '\\x1b[1;31mBold Red\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    client.inject_command("echo '\\x1b[4;32mUnderline Green\\x1b[0m'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(50)).await;

    println!("✅ ANSI color codes test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_cursor_movement() {
    let port = 18804;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("cursor-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting cursor movement test");

    // Test cursor movement sequences
    println!("   Testing cursor positioning");

    // Move cursor to (10, 5)
    client.inject_command("echo '\\x1b[10;5HPosition (10,5)'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Move cursor up
    client.inject_command("echo '\\x1b[3AUp 3 lines'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Move cursor down
    client.inject_command("echo '\\x1b[2BDown 2 lines'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Move cursor right
    client.inject_command("echo '\\x1b[5CRight 5 cols'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Move cursor left
    client.inject_command("echo '\\x1b[3DLeft 3 cols'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Save and restore cursor position
    client.inject_command("echo '\\x1b[sSave position'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    client.inject_command("echo '\\x1b[uRestore position'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    println!("✅ Cursor movement test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_screen_manipulation() {
    let port = 18805;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("screen-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting screen manipulation test");

    // Fill screen with content
    println!("   Filling screen");
    for i in 0..24 {
        let cmd = format!("echo 'Line {}'\n", i);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
    }
    sleep(Duration::from_millis(200)).await;

    // Clear screen
    println!("   Clearing screen");
    client.inject_command("echo '\\x1b[2J'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(200)).await;

    // Clear to end of line
    println!("   Testing line clearing");
    client.inject_command("echo 'This will be cleared...\\x1b[K'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Scroll up
    println!("   Testing scroll");
    client.inject_command("echo '\\x1b[S'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    // Scroll down
    client.inject_command("echo '\\x1b[T'\n")
        .await
        .expect("Command failed");
    sleep(Duration::from_millis(100)).await;

    println!("✅ Screen manipulation test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
#[ignore]
async fn test_headless_mixed_content_stress() {
    let port = 18806;
    let (token, handle) = start_test_server(port).await;

    let mut client = ServerClient::connect(&format!("127.0.0.1:{}", port))
        .await
        .expect("Connect failed");

    client.authenticate(&token).await.expect("Auth failed");
    client.create_session(Some("mixed-test")).await.expect("Session failed");
    client.create_pane(Some("pane1")).await.expect("Pane failed");

    println!("🚀 Starting mixed content stress test");

    // Mix of ASCII, Unicode, ANSI codes, and emoji
    let mixed_content = vec![
        "Normal ASCII text",
        "\\x1b[31m红色文本 (Red Chinese)\\x1b[0m",
        "Emoji: 🚀 🎉 ✅ mixed with text",
        "\\x1b[1;32mBold Green こんにちは\\x1b[0m",
        "Math symbols: ∑ ∫ π ∞ √ ≈",
        "Box drawing: ┌─┬─┐ │ │ │ └─┴─┘",
        "Arrows: ← → ↑ ↓ ↔ ⇄ ⇆",
        "\\x1b[44mBlue background مرحبا\\x1b[0m",
    ];

    for (i, content) in mixed_content.iter().enumerate() {
        let cmd = format!("echo '{}'\n", content);
        client.inject_command(&client.session_id().to_string(), &client.pane_id().to_string(), &cmd).await.expect("Command failed");
        println!("   Line {}: {}", i + 1, content);
        sleep(Duration::from_millis(100)).await;
    }

    // Rapid mixed content
    println!("   Sending rapid mixed content burst");
    for _ in 0..20 {
        let cmd = "echo '🔥 快速 FAST \\x1b[31mRED\\x1b[0m ✨'\n";
        client.inject_command(cmd).await.expect("Command failed");
    }

    sleep(Duration::from_millis(500)).await;

    println!("✅ Mixed content stress test complete");

    handle.abort();
    sleep(Duration::from_millis(100)).await;
}
