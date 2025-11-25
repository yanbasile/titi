//! Regression Test Suite
//!
//! This module provides a comprehensive regression test suite that ensures
//! all critical functionality continues to work correctly across code changes.
//!
//! The regression tests are organized into categories:
//! - Terminal core functionality (grid, parser, text extraction)
//! - Rendering functionality (glyph atlas, text rendering)
//! - Performance optimizations (dirty tracking)
//! - Integration tests (copy/paste, mouse support)
//!
//! Run with: cargo test --test regression

use titi::terminal::{Grid, CellStyle, Color};

// =============================================================================
// TERMINAL CORE REGRESSION TESTS
// =============================================================================

#[test]
fn regression_grid_basic_operations() {
    let mut grid = Grid::new(80, 24);

    // Test basic character insertion
    grid.put_char('H');
    grid.put_char('e');
    grid.put_char('l');
    grid.put_char('l');
    grid.put_char('o');

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 5, "Cursor should advance after each character");
    assert_eq!(y, 0, "Cursor should stay on first line");

    // Test newline
    grid.newline();
    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 0, "Newline should reset cursor to column 0");
    assert_eq!(y, 1, "Newline should advance to next line");
}

#[test]
fn regression_grid_scrolling() {
    let mut grid = Grid::new(80, 3);

    // Fill grid and force scrolling
    for i in 0..5 {
        for c in format!("Line {}", i).chars() {
            grid.put_char(c);
        }
        grid.newline();
    }

    // Verify scrollback exists
    assert!(grid.scrollback_len() > 0, "Scrollback should contain scrolled lines");

    // Test scroll navigation
    grid.scroll_back_up(1);
    assert!(!grid.is_at_bottom(), "Should not be at bottom after scrolling up");

    grid.scroll_to_bottom();
    assert!(grid.is_at_bottom(), "Should be at bottom after scroll_to_bottom");
}

#[test]
fn regression_grid_clear_operations() {
    let mut grid = Grid::new(80, 24);

    // Write text
    for c in "Test".chars() {
        grid.put_char(c);
    }

    // Clear screen
    grid.clear_screen();

    // Verify all cells are cleared
    for y in 0..24 {
        for x in 0..80 {
            if let Some(cell) = grid.get_cell(x, y) {
                assert_eq!(cell.c, ' ', "Cell should be space after clear");
            }
        }
    }
}

#[test]
fn regression_grid_resize() {
    let mut grid = Grid::new(80, 24);

    // Write text
    for c in "Hello World".chars() {
        grid.put_char(c);
    }

    // Resize
    grid.resize(100, 30);

    let (cols, rows) = grid.size();
    assert_eq!(cols, 100, "Columns should be updated");
    assert_eq!(rows, 30, "Rows should be updated");

    // Verify text is preserved (within old dimensions)
    if let Some(cell) = grid.get_cell(0, 0) {
        assert_eq!(cell.c, 'H', "Text should be preserved after resize");
    }
}

// =============================================================================
// DIRTY TRACKING REGRESSION TESTS
// =============================================================================

#[test]
fn regression_dirty_tracking_basic() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    // Single character should mark one cell dirty
    grid.put_char('a');

    assert!(!grid.is_all_dirty(), "Should not be all dirty");
    assert_eq!(grid.dirty_cells().len(), 1, "Should have exactly one dirty cell");
    assert!(grid.dirty_cells().contains(&(0, 0)));
}

#[test]
fn regression_dirty_tracking_scroll() {
    let mut grid = Grid::new(80, 24);
    grid.clear_dirty();

    grid.scroll_up(1);

    assert!(grid.is_all_dirty(), "Scroll should mark all dirty");
}

#[test]
fn regression_dirty_tracking_clear() {
    let mut grid = Grid::new(80, 24);

    // Mark some cells dirty
    grid.put_char('a');
    grid.put_char('b');

    // Clear dirty state
    grid.clear_dirty();

    assert!(!grid.is_all_dirty());
    assert_eq!(grid.dirty_cells().len(), 0);
}

// =============================================================================
// TEXT EXTRACTION REGRESSION TESTS
// =============================================================================

fn extract_text(grid: &Grid) -> String {
    let (cols, rows) = grid.size();
    let mut text = String::new();

    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            if let Some(cell) = grid.get_cell(col, row) {
                line.push(cell.c);
            }
        }
        let trimmed = line.trim_end();
        if !trimmed.is_empty() || row < rows - 1 {
            text.push_str(trimmed);
            if row < rows - 1 {
                text.push('\n');
            }
        }
    }

    text
}

#[test]
fn regression_text_extraction_simple() {
    let mut grid = Grid::new(80, 24);

    for c in "Hello World".chars() {
        grid.put_char(c);
    }

    let text = extract_text(&grid);
    assert!(text.lines().next().unwrap().starts_with("Hello World"));
}

#[test]
fn regression_text_extraction_multiline() {
    let mut grid = Grid::new(80, 3);

    for c in "Line1".chars() {
        grid.put_char(c);
    }
    grid.newline();

    for c in "Line2".chars() {
        grid.put_char(c);
    }
    grid.newline();

    for c in "Line3".chars() {
        grid.put_char(c);
    }

    let text = extract_text(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("Line1"));
    assert!(lines[1].starts_with("Line2"));
    assert!(lines[2].starts_with("Line3"));
}

#[test]
fn regression_text_extraction_unicode() {
    let mut grid = Grid::new(80, 2);

    for c in "Hello 世界".chars() {
        grid.put_char(c);
    }

    let text = extract_text(&grid);
    assert!(text.contains("Hello 世界"));
}

// =============================================================================
// INTEGRATION REGRESSION TESTS
// =============================================================================

#[test]
fn regression_copy_paste_workflow() {
    let mut grid = Grid::new(80, 24);

    // Simulate terminal output
    for c in "Hello from terminal".chars() {
        grid.put_char(c);
    }

    // Extract text (copy operation)
    let copied = extract_text(&grid);

    // Verify copied text
    assert!(copied.contains("Hello from terminal"));

    // Clear and paste back
    grid.clear_screen();

    for c in copied.trim().chars() {
        grid.put_char(c);
    }

    // Verify pasted text
    let pasted = extract_text(&grid);
    assert!(pasted.contains("Hello from terminal"));
}

#[test]
fn regression_scrollback_with_dirty_tracking() {
    let mut grid = Grid::new(80, 3);

    // Generate scrollback
    for i in 0..10 {
        for c in format!("Line {}", i).chars() {
            grid.put_char(c);
        }
        grid.newline();
    }

    grid.clear_dirty();

    // Scroll back should mark all dirty
    grid.scroll_back_up(5);
    assert!(grid.is_all_dirty(), "Scrollback navigation should mark all dirty");

    grid.clear_dirty();

    // Scroll to bottom should mark all dirty
    grid.scroll_to_bottom();
    assert!(grid.is_all_dirty(), "Scroll to bottom should mark all dirty");
}

#[test]
fn regression_style_persistence() {
    let mut grid = Grid::new(80, 24);

    // Set bold style
    let bold_style = CellStyle {
        fg: Color::Default,
        bg: Color::Default,
        bold: true,
        italic: false,
        underline: false,
        strikethrough: false,
        inverse: false,
    };

    grid.set_style(bold_style);
    grid.put_char('B');

    // Reset style
    grid.set_style(CellStyle::default());
    grid.put_char('N');

    // Verify styles
    if let Some(cell) = grid.get_cell(0, 0) {
        assert!(cell.style.bold, "First char should be bold");
    }

    if let Some(cell) = grid.get_cell(1, 0) {
        assert!(!cell.style.bold, "Second char should not be bold");
    }
}

// =============================================================================
// EDGE CASE REGRESSION TESTS
// =============================================================================

#[test]
fn regression_wrapping_at_edge() {
    let mut grid = Grid::new(5, 3);

    // Write exactly to edge
    for _ in 0..5 {
        grid.put_char('X');
    }

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 5, "Cursor should be at column 5");
    assert_eq!(y, 0, "Should still be on first line");

    // Next character should wrap
    grid.put_char('Y');

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 1, "Should wrap to column 1");
    assert_eq!(y, 1, "Should advance to line 1");
}

#[test]
fn regression_empty_grid_operations() {
    let mut grid = Grid::new(80, 24);

    // Operations on empty grid should not panic
    grid.clear_screen();
    grid.clear_line();
    grid.scroll_up(1);
    grid.scroll_back_up(10);
    grid.scroll_to_bottom();

    let text = extract_text(&grid);
    assert!(text.lines().all(|l| l.is_empty()));
}

#[test]
fn regression_single_cell_grid() {
    let mut grid = Grid::new(1, 1);

    grid.put_char('A');

    if let Some(cell) = grid.get_cell(0, 0) {
        assert_eq!(cell.c, 'A');
    }

    // Writing more should cause scrolling
    grid.put_char('B');

    // Should still work without panicking
    let (cols, rows) = grid.size();
    assert_eq!(cols, 1);
    assert_eq!(rows, 1);
}

#[test]
fn regression_unicode_edge_cases() {
    let mut grid = Grid::new(80, 24);

    // Emoji and special unicode
    let chars = vec!['😀', '🎉', '✨', '中', '文', '📝'];

    for c in chars {
        grid.put_char(c);
    }

    // Should not panic, though rendering may vary by font
    let text = extract_text(&grid);
    assert!(text.len() > 0);
}
