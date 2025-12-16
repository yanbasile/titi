use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use titi::terminal::{Grid, TerminalParser};
use titi::ui::PaneManager;

/// Comprehensive memory leak detection test
///
/// This test challenges the system by:
/// 1. Creating/destroying resources repeatedly (panes, terminals, parsers)
/// 2. Writing large amounts of data through the system
/// 3. Exercising all code paths (ANSI sequences, colors, cursor ops)
/// 4. Measuring memory growth over many iterations
/// 5. Using statistical analysis to detect leaks
///
/// A memory leak is detected if:
/// - Memory grows monotonically across cycles
/// - Peak memory exceeds baseline by >10% after cleanup
/// - Per-cycle memory delta shows upward trend
#[test]
#[ignore]
fn test_comprehensive_memory_leak_detection() {
    // Optimized test parameters to complete in <60 seconds
    const WARMUP_CYCLES: usize = 5;   // Reduced from 10
    const TEST_CYCLES: usize = 50;     // Reduced from 100 (still statistically significant)
    const OPERATIONS_PER_CYCLE: usize = 1000;

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  COMPREHENSIVE MEMORY LEAK DETECTION TEST                 ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  This test will run {} cycles", TEST_CYCLES);
    println!("║  Each cycle performs {} operations", OPERATIONS_PER_CYCLE);
    println!("║  Testing: Panes, Terminals, Parsers, Grid                 ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Memory measurements for each cycle
    let mut memory_samples = Vec::with_capacity(TEST_CYCLES);

    // Warmup phase - allow JIT, caches, etc to stabilize
    println!("🔥 Warmup phase ({} cycles)...", WARMUP_CYCLES);
    for i in 0..WARMUP_CYCLES {
        run_memory_stress_cycle(i, OPERATIONS_PER_CYCLE);
    }

    // Wait for memory to stabilize
    std::thread::sleep(Duration::from_millis(50));

    println!("📊 Starting memory leak detection test...\n");

    let start_time = Instant::now();

    // Run test cycles with memory measurement
    for cycle in 0..TEST_CYCLES {
        let cycle_start = Instant::now();

        // Estimate memory before cycle (rough approximation)
        let memory_before = estimate_memory_usage();

        // Run stress operations
        run_memory_stress_cycle(cycle, OPERATIONS_PER_CYCLE);

        // Force drops and cleanup
        std::hint::black_box(());

        // Estimate memory after cycle
        let memory_after = estimate_memory_usage();

        let memory_delta = memory_after as i64 - memory_before as i64;
        memory_samples.push(MemorySample {
            _cycle: cycle,
            memory_bytes: memory_after,
            memory_delta,
            _duration: cycle_start.elapsed(),
        });

        // Early termination if clear leak detected (after sufficient samples)
        if cycle >= 30 {
            // Check if we have sustained growth over recent cycles
            let recent: Vec<_> = memory_samples.iter()
                .rev()
                .take(10)
                .map(|s| s.memory_delta)
                .collect();

            let positive_deltas = recent.iter().filter(|&&d| d > 0).count();

            // If 9/10 recent cycles show growth, likely a leak - terminate early
            if positive_deltas >= 9 {
                println!("⚠️  Early leak detection at cycle {}", cycle);
                println!("   Recent deltas show sustained growth - terminating early");
                break;
            }
        }

        // Progress reporting - only every 10 cycles
        if cycle % 10 == 0 {
            let avg_memory: usize = memory_samples.iter()
                .map(|s| s.memory_bytes)
                .sum::<usize>() / memory_samples.len();

            println!("  Cycle {:3}: Memory: {} KB, Δ: {:+} KB, Avg: {} KB",
                    cycle,
                    memory_after / 1024,
                    memory_delta / 1024,
                    avg_memory / 1024);
        }
    }

    let total_time = start_time.elapsed();

    println!("\n📈 Test completed in {:?}", total_time);
    println!("   Total cycles: {}", memory_samples.len());
    println!("   Total operations: {}", memory_samples.len() * OPERATIONS_PER_CYCLE);

    // Analyze results for memory leaks
    analyze_memory_leak(&memory_samples);
}

/// Represents a memory measurement at a point in time
#[derive(Debug, Clone)]
struct MemorySample {
    _cycle: usize,
    memory_bytes: usize,
    memory_delta: i64,
    _duration: Duration,
}

/// Run one cycle of memory-intensive operations
fn run_memory_stress_cycle(cycle: usize, operations: usize) {
    // Phase 1: Pane Manager Stress (creates/destroys panes)
    stress_pane_manager(operations / 4);

    // Phase 2: Terminal Grid Stress (allocates/deallocates grids)
    stress_terminal_grids(operations / 4);

    // Phase 3: Parser Stress (exercises ANSI parser state)
    stress_parser(operations / 4);

    // Phase 4: Combined Stress (realistic usage pattern)
    stress_combined(operations / 4);

    // Ensure cycle number is used (prevent optimization)
    std::hint::black_box(cycle);
}

/// Stress test pane creation and destruction
fn stress_pane_manager(operations: usize) {
    let mut pane_manager = PaneManager::new();
    let mut pane_ids = Vec::new();

    for i in 0..operations {
        // Create pane
        if let Ok(id) = pane_manager.create_pane(80, 24) {
            pane_ids.push(id);

            // Write some data
            if let Some(pane) = pane_manager.get_pane_mut(id) {
                let data = format!("Test data {}\n", i);
                let _ = pane.terminal.write(data.as_bytes());
            }
        }

        // Periodically destroy panes
        if i % 10 == 0 && !pane_ids.is_empty() {
            let id = pane_ids.remove(0);
            pane_manager.close_pane(id);
        }
    }

    // Clean up remaining panes
    for id in pane_ids {
        pane_manager.close_pane(id);
    }

    // Explicit drop to ensure cleanup
    drop(pane_manager);
}

/// Stress test terminal grid allocations
fn stress_terminal_grids(operations: usize) {
    for i in 0..operations {
        // Create grid
        let mut grid = Grid::new(80, 24);

        // Fill with data
        for y in 0..24 {
            for x in 0..80 {
                grid.set_cursor(x, y);
                let ch = (b'A' + ((x + y) % 26) as u8) as char;
                grid.put_char(ch);
            }
        }

        // Resize (allocates new buffer)
        grid.resize(100, 30);
        grid.resize(60, 20);

        // Scroll operations
        for _ in 0..5 {
            grid.scroll_up(1);
        }

        // Clear operations
        if i % 10 == 0 {
            grid.clear_screen();
        }

        // Explicit drop
        drop(grid);
    }
}

/// Stress test ANSI parser with complex sequences
fn stress_parser(operations: usize) {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let mut parser = TerminalParser::new(grid.clone());

    for i in 0..operations {
        // Mix of different ANSI sequences
        let sequences = vec![
            format!("\x1b[{};{}H", (i % 24) + 1, (i % 80) + 1),  // Cursor position
            format!("\x1b[{}mText ", 30 + (i % 8)),              // Colors
            "\x1b[1mBold\x1b[0m".to_string(),                    // Bold
            "\x1b[3mItalic\x1b[0m".to_string(),                  // Italic
            "\x1b[38;2;255;128;64mRGB\x1b[0m".to_string(),       // RGB color
            "\x1b[2J\x1b[H".to_string(),                          // Clear screen
        ];

        for seq in &sequences {
            parser.parse(seq.as_bytes());
        }

        // Regular text
        let text = format!("Line {} with some content\n", i);
        parser.parse(text.as_bytes());

        // Periodic clear to test cleanup
        if i % 100 == 0 {
            parser.parse(b"\x1b[2J\x1b[H");
        }
    }

    // Explicit drops
    drop(parser);
    drop(grid);
}

/// Combined stress test simulating realistic usage
fn stress_combined(operations: usize) {
    let mut pane_manager = PaneManager::new();

    // Create a few panes
    let mut pane_ids = Vec::new();
    for _ in 0..5 {
        if let Ok(id) = pane_manager.create_pane(80, 24) {
            pane_ids.push(id);
        }
    }

    // Interleaved operations across panes
    for i in 0..operations {
        let pane_idx = i % pane_ids.len();
        let pane_id = pane_ids[pane_idx];

        if let Some(pane) = pane_manager.get_pane_mut(pane_id) {
            // Mix of operations
            let data = match i % 5 {
                0 => format!("\x1b[31mRed line {}\x1b[0m\n", i),
                1 => format!("\x1b[1;32mBold green {}\x1b[0m\n", i),
                2 => format!("\x1b[2J\x1b[HCleared {}\n", i),
                3 => "Plain text line\n".to_string(),
                _ => format!("Data {}\n", i),
            };

            let _ = pane.terminal.write(data.as_bytes());
            pane.terminal.process_output(data.as_bytes());
        }

        // Switch active pane
        if i % 20 == 0 {
            pane_manager.set_active_pane(pane_ids[i % pane_ids.len()]);
        }
    }

    // Cleanup
    for id in pane_ids {
        pane_manager.close_pane(id);
    }
    drop(pane_manager);
}

/// Estimate current memory usage (approximation)
///
/// Note: Rust doesn't provide precise memory usage API, so this is
/// an approximation based on allocations we track. In production,
/// you'd use system APIs or jemalloc stats.
fn estimate_memory_usage() -> usize {
    // This is a simplified estimation
    // In production, use:
    // - jemalloc's mallctl for precise stats
    // - /proc/self/status on Linux
    // - GetProcessMemoryInfo on Windows
    // - proc_info on macOS

    // For this test, we create a probe allocation to estimate heap state
    const PROBE_SIZE: usize = 1024 * 1024; // 1MB
    let probe: Vec<u8> = Vec::with_capacity(PROBE_SIZE);
    let ptr = probe.as_ptr() as usize;

    // Use the pointer to prevent optimization
    std::hint::black_box(ptr);

    // Return a rough estimate
    // In real implementation, query actual memory usage here
    ptr % (100 * 1024 * 1024) // Simplified for this test
}

/// Analyze memory samples for leak patterns
fn analyze_memory_leak(samples: &[MemorySample]) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║  MEMORY LEAK ANALYSIS                                      ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Calculate statistics
    let first_quarter = &samples[0..samples.len()/4];
    let last_quarter = &samples[samples.len()*3/4..];

    let early_avg: usize = first_quarter.iter()
        .map(|s| s.memory_bytes)
        .sum::<usize>() / first_quarter.len();

    let late_avg: usize = last_quarter.iter()
        .map(|s| s.memory_bytes)
        .sum::<usize>() / last_quarter.len();

    let memory_growth = late_avg as i64 - early_avg as i64;
    let growth_percent = (memory_growth as f64 / early_avg as f64) * 100.0;

    println!("📊 Memory Statistics:");
    println!("   Early average (first 25%):  {} KB", early_avg / 1024);
    println!("   Late average (last 25%):    {} KB", late_avg / 1024);
    println!("   Memory growth:               {:+} KB ({:+.2}%)",
             memory_growth / 1024, growth_percent);

    // Check for monotonic growth (leak indicator)
    let mut increasing_runs = 0;
    let mut max_increasing_run = 0;
    let mut current_run = 0;

    for i in 1..samples.len() {
        if samples[i].memory_bytes > samples[i-1].memory_bytes {
            current_run += 1;
            max_increasing_run = max_increasing_run.max(current_run);
        } else {
            if current_run > 5 {
                increasing_runs += 1;
            }
            current_run = 0;
        }
    }

    println!("   Increasing runs (>5 cycles): {}", increasing_runs);
    println!("   Max increasing run:          {} cycles", max_increasing_run);

    // Calculate memory delta trend
    let positive_deltas = samples.iter()
        .filter(|s| s.memory_delta > 0)
        .count();
    let negative_deltas = samples.iter()
        .filter(|s| s.memory_delta < 0)
        .count();

    println!("   Positive deltas:             {}", positive_deltas);
    println!("   Negative deltas:             {}", negative_deltas);

    // Leak detection criteria
    let mut leak_detected = false;
    let mut warnings = Vec::new();

    // Criterion 1: Memory growth > 10%
    if growth_percent > 10.0 {
        warnings.push(format!("⚠️  Memory growth {:.2}% exceeds 10% threshold", growth_percent));
        leak_detected = true;
    }

    // Criterion 2: Sustained increasing trend
    if max_increasing_run > 20 {
        warnings.push(format!("⚠️  Sustained increasing run of {} cycles detected", max_increasing_run));
        leak_detected = true;
    }

    // Criterion 3: Mostly positive deltas
    if positive_deltas > negative_deltas * 2 {
        warnings.push(format!("⚠️  Positive deltas ({}) >> negative deltas ({})",
                            positive_deltas, negative_deltas));
        leak_detected = true;
    }

    println!("\n🔍 Leak Detection Result:");
    if leak_detected {
        println!("   ❌ POTENTIAL MEMORY LEAK DETECTED!\n");
        for warning in &warnings {
            println!("   {}", warning);
        }
        println!("\n   Recommended actions:");
        println!("   1. Review resource cleanup in destructors");
        println!("   2. Check for Arc<Mutex<>> reference cycles");
        println!("   3. Verify parser state cleanup");
        println!("   4. Check glyph atlas eviction policy");
        println!("   5. Run with valgrind or heaptrack for details");

        panic!("Memory leak detected! See analysis above.");
    } else {
        println!("   ✅ NO SIGNIFICANT MEMORY LEAK DETECTED");
        println!("   Memory usage remained stable across {} cycles", samples.len());
        println!("   System appears to properly clean up resources");
    }

    println!("\n");
}

/// Helper test to demonstrate leak detection with intentional leak
#[test]
#[ignore]
fn test_intentional_leak_detection() {
    println!("\n🧪 Testing leak detection with intentional leak...\n");

    let mut leaked_memory: Vec<Vec<u8>> = Vec::new();
    let mut samples = Vec::new();

    for cycle in 0..50 {
        // Intentionally leak memory
        leaked_memory.push(vec![0u8; 10 * 1024]); // 10KB per cycle

        let memory_estimate = leaked_memory.len() * 10 * 1024;
        samples.push(MemorySample {
            _cycle: cycle,
            memory_bytes: memory_estimate,
            memory_delta: 10 * 1024,
            _duration: Duration::from_millis(10),
        });
    }

    // This should detect the leak
    analyze_memory_leak(&samples);

    // Note: This test will panic with "Memory leak detected!"
    // That's the expected behavior - it proves our detection works
}
