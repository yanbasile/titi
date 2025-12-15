use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use titi::terminal::{Grid, Terminal, TerminalParser};

/// Battle Test Scenario 6: Security & Edge Case Testing
///
/// Tests security boundaries and edge cases including:
/// - Malformed ANSI sequences
/// - Invalid UTF-8
/// - Extremely long lines
/// - Binary data injection
/// - Control character abuse
#[test]
#[ignore]
fn test_security_edge_cases() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  BATTLE TEST: Security & Edge Case Testing                ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Testing input validation and security boundaries         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let start_time = Instant::now();
    let mut test_results = Vec::new();

    // Test 1: Malformed ANSI sequences
    println!("🔒 Test 1: Malformed ANSI sequences...");
    match test_malformed_ansi() {
        Ok(()) => {
            println!("   ✅ Malformed ANSI handled safely");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Malformed ANSI test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 2: Invalid UTF-8 sequences
    println!("\n🔒 Test 2: Invalid UTF-8 sequences...");
    match test_invalid_utf8() {
        Ok(()) => {
            println!("   ✅ Invalid UTF-8 handled safely");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Invalid UTF-8 test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 3: Extremely long lines
    println!("\n🔒 Test 3: Extremely long lines (>100 KB)...");
    match test_extremely_long_lines() {
        Ok(()) => {
            println!("   ✅ Long lines handled safely");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Long lines test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 4: Binary data injection
    println!("\n🔒 Test 4: Binary data injection...");
    match test_binary_data() {
        Ok(()) => {
            println!("   ✅ Binary data handled safely");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Binary data test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 5: Control character abuse
    println!("\n🔒 Test 5: Control character abuse...");
    match test_control_character_abuse() {
        Ok(()) => {
            println!("   ✅ Control characters handled safely");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Control character abuse test failed: {}", e);
            test_results.push(false);
        }
    }

    // Test 6: Nested ANSI sequences
    println!("\n🔒 Test 6: Nested ANSI sequences...");
    match test_nested_ansi() {
        Ok(()) => {
            println!("   ✅ Nested ANSI sequences handled safely");
            test_results.push(true);
        }
        Err(e) => {
            println!("   ❌ Nested ANSI test failed: {}", e);
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

    assert_eq!(passed_tests, test_results.len(), "Some security tests failed");
    assert!(total_time < Duration::from_secs(120), "Tests took too long: {:?}", total_time);

    println!("\n✅ Security & Edge Case Testing PASSED!");
    println!("   No crashes, panics, or security vulnerabilities detected");
}

fn test_malformed_ansi() -> Result<(), String> {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid.clone());

    // Various malformed ANSI sequences
    let malformed_sequences: Vec<&[u8]> = vec![
        b"\x1b[",               // Incomplete CSI
        b"\x1b[999999999999999m", // Huge parameter
        b"\x1b[;;;;;;;;;m",     // Multiple empty parameters
        b"\x1b[?",              // Incomplete private mode
        b"\x1b]",               // Incomplete OSC
        b"\x1b[38;5;999m",      // Invalid color index
        b"\x1b[H\x1b[H\x1b[H",  // Repeated sequences
    ];

    for seq in malformed_sequences {
        parser.parse(seq);

        // Verify grid still functional
        let grid_lock = grid.lock().unwrap();
        let (cols, rows) = grid_lock.size();

        if cols == 0 || rows == 0 {
            return Err("Grid corrupted by malformed ANSI".to_string());
        }
    }

    Ok(())
}

fn test_invalid_utf8() -> Result<(), String> {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid.clone());

    // Invalid UTF-8 sequences
    let invalid_utf8_sequences = vec![
        vec![0xFF, 0xFE, 0xFD],           // Invalid bytes
        vec![0xC0, 0x80],                  // Overlong encoding
        vec![0xED, 0xA0, 0x80],            // Surrogate half
        vec![0xF4, 0x90, 0x80, 0x80],      // Out of range
    ];

    for seq in invalid_utf8_sequences {
        // Parser should handle invalid UTF-8 gracefully
        parser.parse(&seq);

        // Verify grid still works
        let grid_lock = grid.lock().unwrap();
        let (cols, rows) = grid_lock.size();

        if cols != 80 || rows != 24 {
            return Err("Grid corrupted by invalid UTF-8".to_string());
        }
    }

    Ok(())
}

fn test_extremely_long_lines() -> Result<(), String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Create extremely long line (100 KB)
    let long_line = "X".repeat(100_000) + "\n";

    let start = Instant::now();
    terminal.process_output(long_line.as_bytes());
    let elapsed = start.elapsed();

    if elapsed > Duration::from_secs(5) {
        return Err(format!("Processing took too long: {:?}", elapsed));
    }

    // Verify terminal still functional
    let grid = terminal.grid();
    let grid_lock = grid.lock().unwrap();
    let (cols, rows) = grid_lock.size();

    if cols != 80 || rows != 24 {
        return Err("Grid dimensions changed after long line".to_string());
    }

    Ok(())
}

fn test_binary_data() -> Result<(), String> {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid.clone());

    // Create binary data (random bytes)
    let mut binary_data = Vec::new();
    for i in 0..256 {
        binary_data.push(i as u8);
    }

    // Parser should handle binary data without panicking
    parser.parse(&binary_data);

    // Verify grid still functional
    let grid_lock = grid.lock().unwrap();
    let (cols, rows) = grid_lock.size();

    if cols != 80 || rows != 24 {
        return Err("Grid corrupted by binary data".to_string());
    }

    Ok(())
}

fn test_control_character_abuse() -> Result<(), String> {
    let mut terminal = Terminal::new(80, 24)
        .map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Send many control characters
    let control_chars = vec![
        b"\x00\x01\x02\x03\x04\x05\x06\x07",
        b"\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F",
        b"\x10\x11\x12\x13\x14\x15\x16\x17",
        b"\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F",
    ];

    for chars in control_chars {
        terminal.process_output(chars);
    }

    // Verify terminal still works
    let grid = terminal.grid();
    let grid_lock = grid.lock().unwrap();
    let (cols, rows) = grid_lock.size();

    if cols != 80 || rows != 24 {
        return Err("Grid corrupted by control characters".to_string());
    }

    Ok(())
}

fn test_nested_ansi() -> Result<(), String> {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid.clone());

    // Deeply nested ANSI sequences
    let mut nested_seq = String::new();
    for i in 0..100 {
        nested_seq.push_str(&format!("\x1b[{}m", 30 + (i % 8)));
    }
    nested_seq.push_str("Text");
    for _ in 0..100 {
        nested_seq.push_str("\x1b[0m");
    }

    parser.parse(nested_seq.as_bytes());

    // Verify grid still functional
    let grid_lock = grid.lock().unwrap();
    let (cols, rows) = grid_lock.size();

    if cols != 80 || rows != 24 {
        return Err("Grid corrupted by nested ANSI".to_string());
    }

    Ok(())
}
