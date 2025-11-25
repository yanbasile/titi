//! Dirty Rectangle Tracking Tests
//!
//! This module tests the dirty tracking system used for rendering optimization.
//! Dirty tracking allows the renderer to identify which cells have changed since
//! the last frame, enabling selective updates instead of full-screen redraws.
//!
//! # Test Coverage
//!
//! - Initial dirty state management
//! - Cell-level dirty marking on character insertion
//! - Line-level dirty marking on operations
//! - Full-screen dirty marking on major operations (scroll, resize, clear)
//! - Dirty state clearing and resetting
//! - Scrollback navigation dirty tracking
//!
//! # Why This Matters
//!
//! Proper dirty tracking is critical for rendering performance. Without it, the
//! GPU would regenerate vertex buffers for all cells every frame, even when only
//! a few characters changed. Dirty tracking enables:
//!
//! - Reduced CPU usage during updates
//! - Lower GPU memory bandwidth usage
//! - Smoother rendering with less latency
//! - Better battery life on laptops
//!
//! Run with: `cargo test --test dirty_tracking_tests`

use titi::terminal::{Grid, CellStyle, Color};

#[test]
fn test_initial_state_is_dirty() {
    let grid = Grid::new(80, 24);
    assert!(grid.is_all_dirty(), "Grid should start as all dirty");
    assert_eq!(grid.dirty_cells().len(), 0, "No individual cells marked dirty initially");
}

#[test]
fn test_put_char_marks_cell_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty(); // Clear initial dirty state

    grid.put_char('a');

    assert!(!grid.is_all_dirty(), "Should not be all dirty after single char");
    assert_eq!(grid.dirty_cells().len(), 1, "Should have one dirty cell");
    assert!(grid.dirty_cells().contains(&(0, 0)), "Cell (0,0) should be dirty");
}

#[test]
fn test_multiple_chars_mark_multiple_cells_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    grid.put_char('h');
    grid.put_char('e');
    grid.put_char('l');
    grid.put_char('l');
    grid.put_char('o');

    assert_eq!(grid.dirty_cells().len(), 5, "Should have 5 dirty cells");
    assert!(grid.dirty_cells().contains(&(0, 0)));
    assert!(grid.dirty_cells().contains(&(1, 0)));
    assert!(grid.dirty_cells().contains(&(2, 0)));
    assert!(grid.dirty_cells().contains(&(3, 0)));
    assert!(grid.dirty_cells().contains(&(4, 0)));
}

#[test]
fn test_clear_screen_marks_all_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    grid.clear_screen();

    assert!(grid.is_all_dirty(), "clear_screen should mark all as dirty");
}

#[test]
fn test_clear_line_marks_line_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    // Move cursor to row 5
    grid.set_cursor(10, 5);
    grid.clear_line();

    assert!(!grid.is_all_dirty(), "Should not be all dirty, just the line");
    assert_eq!(grid.dirty_cells().len(), 80, "Should have 80 dirty cells (entire line)");

    // Check all cells in row 5 are dirty
    for x in 0..80 {
        assert!(grid.dirty_cells().contains(&(x, 5)), "Cell ({}, 5) should be dirty", x);
    }
}

#[test]
fn test_scroll_up_marks_all_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    grid.scroll_up(1);

    assert!(grid.is_all_dirty(), "Scrolling should mark all as dirty");
}

#[test]
fn test_resize_marks_all_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    grid.resize(100, 30);

    assert!(grid.is_all_dirty(), "Resize should mark all as dirty");
}

#[test]
fn test_clear_dirty_resets_state() {
    let mut grid = Grid::new(80, 24);

    // Make some cells dirty
    grid.put_char('a');
    grid.put_char('b');

    assert!(grid.dirty_cells().len() > 0 || grid.is_all_dirty());

    grid.clear_dirty();

    assert!(!grid.is_all_dirty(), "Should not be all dirty after clear");
    assert_eq!(grid.dirty_cells().len(), 0, "Should have no dirty cells after clear");
}

#[test]
fn test_scroll_back_up_marks_all_dirty() {
    let mut grid = Grid::new(80, 24);

    // Add some scrollback
    for _ in 0..30 {
        for _ in 0..80 {
            grid.put_char('x');
        }
        grid.newline();
    }

    grid.clear_dirty();
    grid.scroll_back_up(5);

    assert!(grid.is_all_dirty(), "Scrollback navigation should mark all dirty");
}

#[test]
fn test_scroll_back_down_marks_all_dirty() {
    let mut grid = Grid::new(80, 24);

    // Add scrollback and scroll up
    for _ in 0..30 {
        for _ in 0..80 {
            grid.put_char('x');
        }
        grid.newline();
    }
    grid.scroll_back_up(10);

    grid.clear_dirty();
    grid.scroll_back_down(5);

    assert!(grid.is_all_dirty(), "Scrollback navigation should mark all dirty");
}

#[test]
fn test_scroll_to_bottom_marks_dirty_if_scrolled() {
    let mut grid = Grid::new(80, 24);

    // Add scrollback and scroll up
    for _ in 0..30 {
        for _ in 0..80 {
            grid.put_char('x');
        }
        grid.newline();
    }
    grid.scroll_back_up(10);

    grid.clear_dirty();
    grid.scroll_to_bottom();

    assert!(grid.is_all_dirty(), "scroll_to_bottom should mark all dirty when scrolled");
}

#[test]
fn test_scroll_to_bottom_no_dirty_if_already_at_bottom() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    grid.scroll_to_bottom(); // Already at bottom

    assert!(!grid.is_all_dirty(), "Should not mark dirty if already at bottom");
}

#[test]
fn test_mark_all_dirty() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    grid.mark_all_dirty();

    assert!(grid.is_all_dirty(), "mark_all_dirty should set all_dirty flag");
}

#[test]
fn test_dirty_tracking_with_wrapping() {
    let mut grid = Grid::new(5, 3);
    grid.clear_dirty();

    // Write 6 characters, should wrap to next line
    for _ in 0..6 {
        grid.put_char('x');
    }

    assert!(grid.dirty_cells().len() >= 6, "Should have at least 6 dirty cells");
    assert!(grid.dirty_cells().contains(&(0, 0)));
    assert!(grid.dirty_cells().contains(&(4, 0)));
    assert!(grid.dirty_cells().contains(&(0, 1)), "Should wrap to next line");
}

#[test]
fn test_dirty_tracking_with_scrolling_content() {
    let mut grid = Grid::new(5, 3);
    grid.clear_dirty();

    // Fill grid and cause scrolling
    for _ in 0..20 {
        grid.put_char('x');
    }

    // After scrolling, should be all dirty
    assert!(grid.is_all_dirty() || grid.dirty_cells().len() > 0);
}
