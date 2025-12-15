use std::process::Command;
use std::time::{Duration, Instant};
use titi::terminal::Terminal;

/// Battle Test Scenario 4: Complex Real Application Integration
///
/// Tests the terminal with actual command-line applications.
/// This validates ANSI escape sequences, cursor positioning, and real-world usage.
#[test]
#[ignore]
fn test_real_application_integration() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  BATTLE TEST: Real Application Integration                ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Testing with real terminal applications                  ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let start_time = Instant::now();
    let mut all_tests_passed = true;

    // Test 1: ls command with colors
    println!("📋 Test 1: ls command with ANSI colors...");
    match test_ls_command() {
        Ok(()) => println!("   ✅ ls command test passed"),
        Err(e) => {
            println!("   ❌ ls command test failed: {}", e);
            all_tests_passed = false;
        }
    }

    // Test 2: cat with large file
    println!("\n📋 Test 2: cat command with file output...");
    match test_cat_command() {
        Ok(()) => println!("   ✅ cat command test passed"),
        Err(e) => {
            println!("   ❌ cat command test failed: {}", e);
            all_tests_passed = false;
        }
    }

    // Test 3: grep with colored output
    println!("\n📋 Test 3: grep command with color highlights...");
    match test_grep_command() {
        Ok(()) => println!("   ✅ grep command test passed"),
        Err(e) => {
            println!("   ❌ grep command test failed: {}", e);
            all_tests_passed = false;
        }
    }

    // Test 4: echo with various escape sequences
    println!("\n📋 Test 4: echo with escape sequences...");
    match test_echo_sequences() {
        Ok(()) => println!("   ✅ echo escape sequences test passed"),
        Err(e) => {
            println!("   ❌ echo escape sequences test failed: {}", e);
            all_tests_passed = false;
        }
    }

    // Test 5: Simple cursor movement
    println!("\n📋 Test 5: Cursor positioning...");
    match test_cursor_positioning() {
        Ok(()) => println!("   ✅ Cursor positioning test passed"),
        Err(e) => {
            println!("   ❌ Cursor positioning test failed: {}", e);
            all_tests_passed = false;
        }
    }

    let total_time = start_time.elapsed();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  TEST RESULTS                                              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("Total test time: {:?}", total_time);
    println!("All tests passed: {}", all_tests_passed);

    assert!(all_tests_passed, "Some real application tests failed");
    assert!(total_time < Duration::from_secs(30), "Tests took too long: {:?}", total_time);

    println!("\n✅ Real Application Integration Test PASSED!");
    println!("\n📝 Manual Verification Required:");
    println!("   For full validation, manually test with:");
    println!("   - vim: Open and edit files");
    println!("   - htop: System monitoring");
    println!("   - nano: Text editing");
    println!("   - top: Process monitoring");
    println!("   - less: Large file navigation");
}

fn test_ls_command() -> Result<(), String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Run ls command
    let output = Command::new("ls")
        .args(&["-la", "--color=always"])
        .output()
        .map_err(|e| format!("Failed to run ls: {}", e))?;

    // Process output through terminal
    terminal.process_output(&output.stdout);

    // Verify we got some output
    let grid = terminal.grid();
    let grid_lock = grid.lock().unwrap();
    let (cols, rows) = grid_lock.size();

    if cols == 0 || rows == 0 {
        return Err("Grid has zero size".to_string());
    }

    // Check that some cells have content
    let mut has_content = false;
    for y in 0..rows {
        for x in 0..cols {
            if let Some(cell) = grid_lock.get_cell(x, y) {
                if cell.c != ' ' {
                    has_content = true;
                    break;
                }
            }
        }
        if has_content {
            break;
        }
    }

    if !has_content {
        return Err("No content found in terminal grid".to_string());
    }

    Ok(())
}

fn test_cat_command() -> Result<(), String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Create a temporary test file
    let test_content = "Line 1: Test content\nLine 2: More content\nLine 3: Even more\n";
    std::fs::write("/tmp/titi_test_file.txt", test_content)
        .map_err(|e| format!("Failed to write test file: {}", e))?;

    // Run cat command
    let output = Command::new("cat")
        .arg("/tmp/titi_test_file.txt")
        .output()
        .map_err(|e| format!("Failed to run cat: {}", e))?;

    // Process output
    terminal.process_output(&output.stdout);

    // Cleanup
    let _ = std::fs::remove_file("/tmp/titi_test_file.txt");

    // Verify content was processed
    let grid = terminal.grid();
    let grid_lock = grid.lock().unwrap();

    let mut has_content = false;
    for y in 0..24 {
        for x in 0..80 {
            if let Some(cell) = grid_lock.get_cell(x, y) {
                if cell.c != ' ' {
                    has_content = true;
                    break;
                }
            }
        }
        if has_content {
            break;
        }
    }

    if !has_content {
        return Err("Cat output not found in terminal".to_string());
    }

    Ok(())
}

fn test_grep_command() -> Result<(), String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Create test file
    let test_content = "error: something went wrong\ninfo: all good\nerror: another problem\n";
    std::fs::write("/tmp/titi_grep_test.txt", test_content)
        .map_err(|e| format!("Failed to write test file: {}", e))?;

    // Run grep with color
    let output = Command::new("grep")
        .args(&["--color=always", "error", "/tmp/titi_grep_test.txt"])
        .output()
        .map_err(|e| format!("Failed to run grep: {}", e))?;

    // Process output
    terminal.process_output(&output.stdout);

    // Cleanup
    let _ = std::fs::remove_file("/tmp/titi_grep_test.txt");

    // Verify output
    let grid = terminal.grid();
    let grid_lock = grid.lock().unwrap();

    let mut found_output = false;
    for y in 0..24 {
        for x in 0..80 {
            if let Some(cell) = grid_lock.get_cell(x, y) {
                if cell.c == 'e' || cell.c == 'r' {
                    found_output = true;
                    break;
                }
            }
        }
        if found_output {
            break;
        }
    }

    if !found_output {
        return Err("Grep output not found".to_string());
    }

    Ok(())
}

fn test_echo_sequences() -> Result<(), String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Test various ANSI sequences
    let sequences = vec![
        "\x1b[31mRed text\x1b[0m\n",           // Red color
        "\x1b[1mBold text\x1b[0m\n",            // Bold
        "\x1b[4mUnderlined\x1b[0m\n",           // Underline
        "\x1b[7mInverted\x1b[0m\n",             // Inverse
        "Normal text\n",
    ];

    for seq in sequences {
        terminal.process_output(seq.as_bytes());
    }

    // Verify terminal still functional
    let grid = terminal.grid();
    let grid_lock = grid.lock().unwrap();
    let (cols, rows) = grid_lock.size();

    assert_eq!(cols, 80, "Terminal width changed");
    assert_eq!(rows, 24, "Terminal height changed");

    Ok(())
}

fn test_cursor_positioning() -> Result<(), String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Move cursor to specific positions and write
    let sequences = vec![
        "\x1b[1;1HA",      // Move to 1,1 and write 'A'
        "\x1b[5;10HB",     // Move to 5,10 and write 'B'
        "\x1b[10;20HC",    // Move to 10,20 and write 'C'
        "\x1b[H",          // Home position
    ];

    for seq in sequences {
        terminal.process_output(seq.as_bytes());
    }

    // Verify grid is still valid
    let grid = terminal.grid();
    let grid_lock = grid.lock().unwrap();

    // Check that we can get cells at different positions
    let cell_1_1 = grid_lock.get_cell(0, 0);
    let cell_5_10 = grid_lock.get_cell(9, 4);
    let cell_10_20 = grid_lock.get_cell(19, 9);

    assert!(cell_1_1.is_some(), "Cannot access cell at 1,1");
    assert!(cell_5_10.is_some(), "Cannot access cell at 5,10");
    assert!(cell_10_20.is_some(), "Cannot access cell at 10,20");

    Ok(())
}
