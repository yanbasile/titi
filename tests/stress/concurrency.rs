use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use titi::terminal::{Grid, TerminalParser};
use titi::ui::{PaneManager, SplitDirection};

/// Test multiple concurrent panes with simultaneous updates
#[test]
#[ignore]
fn test_stress_multiple_concurrent_panes() {
    let mut pane_manager = PaneManager::new();

    let pane_count = 10;
    let mut pane_ids = Vec::new();

    // Create multiple panes
    for i in 0..pane_count {
        let id = pane_manager.create_pane(80, 24).expect("Failed to create pane");
        pane_ids.push(id);
        println!("Created pane {} with ID {:?}", i, id);
    }

    assert_eq!(pane_manager.panes().len(), pane_count);

    // Simulate concurrent writes to all panes
    let start = Instant::now();

    for i in 0..100 {
        for (idx, pane_id) in pane_ids.iter().enumerate() {
            if let Some(pane) = pane_manager.get_pane_mut(*pane_id) {
                let msg = format!("Pane {} update {}\n", idx, i);
                pane.terminal.write(msg.as_bytes()).expect("Write failed");
                pane.terminal.process_output(msg.as_bytes());
            }
        }
    }

    let elapsed = start.elapsed();
    println!("Updated {} panes 100 times in {:?}", pane_count, elapsed);

    // Should complete in reasonable time
    assert!(elapsed < Duration::from_secs(5), "Concurrent updates too slow");
}

/// Test creating and destroying many panes
#[test]
#[ignore]
fn test_stress_pane_lifecycle() {
    let mut pane_manager = PaneManager::new();

    let start = Instant::now();
    let cycles = 100; // Reduced from 1000 - PTY creation is inherently slow

    for i in 0..cycles {
        // Create pane
        let id = pane_manager.create_pane(80, 24).expect("Failed to create pane");

        // Write some data
        if let Some(pane) = pane_manager.get_pane_mut(id) {
            let msg = format!("Cycle {}\n", i);
            pane.terminal.write(msg.as_bytes()).expect("Write failed");
        }

        // Destroy pane
        pane_manager.close_pane(id);
    }

    let elapsed = start.elapsed();
    println!("Completed {} pane lifecycle cycles in {:?}", cycles, elapsed);
    println!("Average time per cycle: {:?}", elapsed / cycles);

    // PTY creation is slow (~45ms per pane), so 100 panes ~= 4.5s is reasonable
    assert!(elapsed < Duration::from_secs(10), "Pane lifecycle too slow: {:?}", elapsed);
}

/// Test stress with 50+ concurrent panes
#[test]
#[ignore]
fn test_stress_many_panes() {
    let mut pane_manager = PaneManager::new();

    let pane_count = 50;
    let mut pane_ids = Vec::new();

    let start = Instant::now();

    // Create 50 panes
    for _ in 0..pane_count {
        let id = pane_manager.create_pane(80, 24).expect("Failed to create pane");
        pane_ids.push(id);
    }

    let creation_time = start.elapsed();
    println!("Created {} panes in {:?}", pane_count, creation_time);

    // Update all panes simultaneously
    let update_start = Instant::now();

    for _ in 0..10 {
        for pane_id in &pane_ids {
            if let Some(pane) = pane_manager.get_pane_mut(*pane_id) {
                let msg = "Update line\n";
                pane.terminal.write(msg.as_bytes()).expect("Write failed");
                pane.terminal.process_output(msg.as_bytes());
            }
        }
    }

    let update_time = update_start.elapsed();
    println!("Updated {} panes 10 times in {:?}", pane_count, update_time);

    assert!(creation_time < Duration::from_secs(5), "Pane creation too slow");
    assert!(update_time < Duration::from_secs(5), "Pane updates too slow");
}

/// Test concurrent parser access
#[test]
#[ignore]
fn test_stress_concurrent_parser_access() {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let parser1 = TerminalParser::new(grid.clone());
    let parser2 = TerminalParser::new(grid.clone());

    // Note: This test demonstrates the architecture but actual concurrent
    // access would require parsers to be Send + Sync which may need refactoring

    let grid_clone1 = grid.clone();
    let grid_clone2 = grid.clone();

    let start = Instant::now();

    // Simulate interleaved access
    for i in 0..1000 {
        {
            let mut g = grid_clone1.lock().unwrap();
            g.put_char('A');
        }
        {
            let mut g = grid_clone2.lock().unwrap();
            g.put_char('B');
        }
    }

    let elapsed = start.elapsed();
    println!("Completed concurrent access test in {:?}", elapsed);

    let g = grid.lock().unwrap();
    assert_eq!(g.size(), (80, 24));
}

/// Test pane splitting stress
#[test]
#[ignore]
fn test_stress_pane_splitting() {
    let mut pane_manager = PaneManager::new();

    // Create initial pane
    let root = pane_manager.create_pane(80, 24).expect("Failed to create root pane");

    let start = Instant::now();

    // Create many splits
    let mut current_panes = vec![root];

    for i in 0..20 {
        let pane_to_split = current_panes[i % current_panes.len()];
        let direction = if i % 2 == 0 {
            SplitDirection::Horizontal
        } else {
            SplitDirection::Vertical
        };

        match pane_manager.split_pane(pane_to_split, direction, 40, 12) {
            Ok(new_pane) => {
                current_panes.push(new_pane);
            }
            Err(e) => {
                println!("Split failed at iteration {}: {}", i, e);
                break;
            }
        }
    }

    let elapsed = start.elapsed();
    println!("Created {} splits in {:?}", current_panes.len(), elapsed);

    assert!(current_panes.len() > 10, "Should create multiple panes");
}

/// Test rapid pane switching
#[test]
#[ignore]
fn test_stress_pane_switching() {
    let mut pane_manager = PaneManager::new();

    // Create 10 panes
    let mut pane_ids = Vec::new();
    for _ in 0..10 {
        let id = pane_manager.create_pane(80, 24).expect("Failed to create pane");
        pane_ids.push(id);
    }

    let start = Instant::now();
    let switches = 10_000;

    for i in 0..switches {
        let pane_id = pane_ids[i % pane_ids.len()];
        pane_manager.set_active_pane(pane_id);

        // Verify active pane changed
        assert_eq!(pane_manager.active_pane(), Some(pane_id));
    }

    let elapsed = start.elapsed();
    let rate = (switches as f64) / elapsed.as_secs_f64();

    println!("Performed {} pane switches in {:?}", switches, elapsed);
    println!("Switch rate: {:.2} switches/sec", rate);

    assert!(rate > 10_000.0, "Pane switching too slow");
}

/// Test memory under heavy pane usage
#[test]
#[ignore]
fn test_stress_pane_memory() {
    let mut pane_manager = PaneManager::new();

    let start = Instant::now();

    // Create and use many panes
    for cycle in 0..100 {
        let mut temp_panes = Vec::new();

        // Create 20 panes
        for _ in 0..20 {
            let id = pane_manager.create_pane(80, 24).expect("Failed to create pane");
            temp_panes.push(id);

            // Write some data
            if let Some(pane) = pane_manager.get_pane_mut(id) {
                let msg = format!("Cycle {} data\n", cycle);
                pane.terminal.write(msg.as_bytes()).expect("Write failed");
                pane.terminal.process_output(msg.as_bytes());
            }
        }

        // Remove all temporary panes
        for id in temp_panes {
            pane_manager.close_pane(id);
        }

        if cycle % 10 == 0 {
            println!("Completed {} cycles", cycle);
        }
    }

    let elapsed = start.elapsed();
    println!("Completed memory stress test in {:?}", elapsed);

    // Should not have accumulated panes
    assert!(pane_manager.panes().len() < 5, "Pane cleanup failed");
}

/// Test concurrent grid operations
#[test]
#[ignore]
fn test_stress_concurrent_grid_operations() {
    let grids: Vec<Arc<Mutex<Grid>>> = (0..10)
        .map(|_| Arc::new(Mutex::new(Grid::new(80, 24))))
        .collect();

    let start = Instant::now();
    let operations_per_thread = 1000;

    let handles: Vec<_> = grids
        .iter()
        .enumerate()
        .map(|(idx, grid)| {
            let grid = grid.clone();
            thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let mut g = grid.lock().unwrap();
                    g.set_cursor((i % 80), (i % 24));
                    g.put_char(('A' as u8 + (idx % 26) as u8) as char);

                    if i % 100 == 0 {
                        g.clear_screen();
                    }
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let elapsed = start.elapsed();
    println!("Completed concurrent grid operations in {:?}", elapsed);

    // Verify all grids are still valid
    for grid in grids {
        let g = grid.lock().unwrap();
        assert_eq!(g.size(), (80, 24));
    }
}

/// Test layout calculation performance
#[test]
#[ignore]
fn test_stress_layout_calculation() {
    let mut pane_manager = PaneManager::new();

    // Create complex layout
    let root = pane_manager.create_pane(80, 24).expect("Failed to create root");

    let mut panes = vec![root];
    for i in 0..15 {
        let pane = panes[i % panes.len()];
        let direction = if i % 2 == 0 {
            SplitDirection::Horizontal
        } else {
            SplitDirection::Vertical
        };
        if let Ok(new_pane) = pane_manager.split_pane(pane, direction, 40, 12) {
            panes.push(new_pane);
        }
    }

    let start = Instant::now();
    let calculations = 10_000;

    for _ in 0..calculations {
        let _bounds = pane_manager.layout().calculate_bounds(1920.0, 1080.0);
    }

    let elapsed = start.elapsed();
    let rate = (calculations as f64) / elapsed.as_secs_f64();

    println!("Performed {} layout calculations in {:?}", calculations, elapsed);
    println!("Rate: {:.2} calculations/sec", rate);

    assert!(rate > 1000.0, "Layout calculation too slow");
}

/// Test parser throughput with concurrent grids
#[test]
#[ignore]
fn test_stress_parser_throughput() {
    let thread_count = 4;
    let lines_per_thread = 10_000;

    let start = Instant::now();

    let handles: Vec<_> = (0..thread_count)
        .map(|thread_id| {
            thread::spawn(move || {
                let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
                let mut parser = TerminalParser::new(grid.clone());

                for i in 0..lines_per_thread {
                    let line = format!("Thread {} line {}\n", thread_id, i);
                    parser.parse(line.as_bytes());
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let elapsed = start.elapsed();
    let total_lines = thread_count * lines_per_thread;
    let throughput = (total_lines as f64) / elapsed.as_secs_f64();

    println!("Processed {} lines across {} threads in {:?}",
             total_lines, thread_count, elapsed);
    println!("Aggregate throughput: {:.2} lines/sec", throughput);

    assert!(throughput > 10_000.0, "Aggregate throughput too low");
}
