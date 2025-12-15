use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use titi::terminal::{Grid, Terminal, TerminalParser};
use titi::ui::PaneManager;

/// Battle Test Scenario 5: Resource Exhaustion Testing
///
/// Pushes the system to its limits to test:
/// - Maximum pane creation
/// - Scrollback capacity
/// - Rapid-fire commands
/// - Large ANSI sequences
/// - Graceful degradation
#[test]
#[ignore]
fn test_resource_exhaustion() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  BATTLE TEST: Resource Exhaustion                         ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Pushing system to limits and testing graceful handling   ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let start_time = Instant::now();
    let mut test_results = Vec::new();

    // Test 1: Maximum pane creation
    println!("💥 Test 1: Maximum pane creation (100+ panes)...");
    match test_maximum_panes() {
        Ok(count) => {
            println!("   ✅ Created {} panes successfully", count);
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Maximum panes test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 2: Scrollback capacity
    println!("\n💥 Test 2: Scrollback buffer capacity (10,000 lines)...");
    match test_scrollback_capacity() {
        Ok(lines) => {
            println!("   ✅ Processed {} lines in scrollback", lines);
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Scrollback capacity test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 3: Rapid-fire commands
    println!("\n💥 Test 3: Rapid-fire commands (10,000 commands)...");
    match test_rapid_fire_commands() {
        Ok(rate) => {
            println!("   ✅ Processed at {:.2} commands/sec", rate);
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Rapid-fire commands test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 4: Large ANSI sequences
    println!("\n💥 Test 4: Large ANSI sequences (10 KB+)...");
    match test_large_ansi_sequences() {
        Ok(size) => {
            println!("   ✅ Processed {} KB ANSI sequences", size / 1024);
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Large ANSI sequences test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 5: Memory pressure
    println!("\n💥 Test 5: Memory pressure test...");
    match test_memory_pressure() {
        Ok(()) => {
            println!("   ✅ System stable under memory pressure");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Memory pressure test failed: {}", e);
            test_results.push(false);
        }
    }

    let total_time = start_time.elapsed();

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  TEST RESULTS                                              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let passed_tests = test_results.iter().filter(|&&r| r).count();
    println!("Total test time: {:?}", total_time);
    println!("Tests passed: {}/{}", passed_tests, test_results.len());

    assert_eq!(passed_tests, test_results.len(), "Some resource exhaustion tests failed");
    assert!(total_time < Duration::from_secs(300), "Tests took too long: {:?}", total_time);

    println!("\n✅ Resource Exhaustion Test PASSED!");
    println!("   System demonstrated graceful degradation under extreme load");
}

fn test_maximum_panes() -> Result<usize, String> {
    let mut pane_manager = PaneManager::new();
    let target_panes = 100;
    let mut created_count = 0;

    let start = Instant::now();

    for i in 0..target_panes {
        match pane_manager.create_pane(80, 24) {
            Ok(_id) => {
                created_count += 1;

                // Write some data to each pane to ensure it's functional
                if let Some(pane) = pane_manager.get_pane_mut(_id) {
                    let msg = format!("Pane {} initialized\n", i);
                    let _ = pane.terminal.write(msg.as_bytes());
                }
            }
            Err(e) => {
                // If we hit a limit, that's okay as long as we got a reasonable number
                if created_count < 50 {
                    return Err(format!("Failed to create enough panes: {} ({})", created_count, e));
                }
                break;
            }
        }

        // Stop if taking too long
        if start.elapsed() > Duration::from_secs(120) {
            break;
        }
    }

    // Cleanup all panes
    let pane_ids: Vec<_> = pane_manager.panes().keys().copied().collect();
    for pane_id in pane_ids {
        pane_manager.close_pane(pane_id);
    }

    if created_count < 50 {
        Err(format!("Only created {} panes, expected at least 50", created_count))
    } else {
        Ok(created_count)
    }
}

fn test_scrollback_capacity() -> Result<usize, String> {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid.clone());

    let target_lines = 10_000;
    let start = Instant::now();

    // Fill scrollback to capacity
    for i in 0..target_lines {
        let line = format!("Scrollback line {}: Testing capacity\n", i);
        parser.parse(line.as_bytes());

        // Stop if taking too long
        if start.elapsed() > Duration::from_secs(30) {
            return Err(format!("Timeout after {} lines", i));
        }
    }

    let elapsed = start.elapsed();
    let rate = target_lines as f64 / elapsed.as_secs_f64();

    println!("   Scrollback fill rate: {:.2} lines/sec", rate);

    // Verify grid is still functional
    let grid_lock = grid.lock().unwrap();
    let (cols, rows) = grid_lock.size();

    if cols != 80 || rows != 24 {
        return Err("Grid dimensions changed".to_string());
    }

    Ok(target_lines)
}

fn test_rapid_fire_commands() -> Result<f64, String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    let command_count = 10_000;
    let start = Instant::now();

    for i in 0..command_count {
        let cmd = format!("command {}\n", i);
        terminal.process_output(cmd.as_bytes());
    }

    let elapsed = start.elapsed();
    let rate = command_count as f64 / elapsed.as_secs_f64();

    if elapsed > Duration::from_secs(10) {
        return Err(format!("Processing too slow: {:?}", elapsed));
    }

    Ok(rate)
}

fn test_large_ansi_sequences() -> Result<usize, String> {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid);

    // Create large ANSI sequence (10 KB)
    let mut large_sequence = String::new();

    // Add many color changes
    for i in 0..500 {
        large_sequence.push_str(&format!("\x1b[{}mColor {} ", 30 + (i % 8), i));
    }
    large_sequence.push_str("\n");

    let sequence_size = large_sequence.len();

    if sequence_size < 10_000 {
        return Err(format!("Sequence too small: {} bytes", sequence_size));
    }

    let start = Instant::now();

    // Process the large sequence
    parser.parse(large_sequence.as_bytes());

    let elapsed = start.elapsed();

    if elapsed > Duration::from_secs(5) {
        return Err(format!("Processing too slow: {:?}", elapsed));
    }

    Ok(sequence_size)
}

fn test_memory_pressure() -> Result<(), String> {
    // Create multiple terminals simultaneously
    let mut terminals = Vec::new();

    for _ in 0..10 {
        let terminal = Terminal::new(80, 24)
            .map_err(|e| format!("Failed to create terminal: {}", e))?;
        terminals.push(terminal);
    }

    // Fill each terminal with data
    for (idx, terminal) in terminals.iter_mut().enumerate() {
        for i in 0..1000 {
            let line = format!("Terminal {} line {}\n", idx, i);
            terminal.process_output(line.as_bytes());
        }
    }

    // Verify all terminals still functional
    for terminal in &terminals {
        let grid = terminal.grid();
        let grid_lock = grid.lock().unwrap();
        let (cols, rows) = grid_lock.size();

        if cols != 80 || rows != 24 {
            return Err("Terminal corrupted under memory pressure".to_string());
        }
    }

    // Cleanup
    drop(terminals);

    Ok(())
}
