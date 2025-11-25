//! Text Extraction Tests
//!
//! This module tests the text extraction functionality used for copy/paste operations.
//! Text extraction reads the visible terminal grid and converts it to plain text,
//! preserving formatting while trimming unnecessary trailing spaces.
//!
//! # Test Coverage
//!
//! - Simple single-line text extraction
//! - Multi-line text with proper newline handling
//! - Trailing space trimming
//! - Empty line preservation (between content)
//! - Internal space preservation
//! - Special character handling
//! - Unicode and full-width character support
//! - Text extraction after grid operations (clear, resize, scroll)
//!
//! # Integration with Copy/Paste
//!
//! These tests validate the core logic used by the Ctrl+Shift+C keyboard shortcut.
//! When a user copies text from the terminal:
//!
//! 1. The `get_visible_text()` function extracts all visible cells
//! 2. Each row is trimmed of trailing spaces
//! 3. Empty lines within content are preserved with newlines
//! 4. The resulting text is placed in the system clipboard
//!
//! # Why Proper Text Extraction Matters
//!
//! - **Usability**: Users expect copied text to match what they see
//! - **Formatting**: Preserve structure while removing terminal artifacts
//! - **Unicode**: Handle international characters and emojis correctly
//! - **Efficiency**: Extract only visible content, not internal state
//!
//! Run with: `cargo test --test text_extraction_tests`

use titi::terminal::{Grid, Cell};

fn get_visible_text_from_grid(grid: &Grid) -> String {
    let (cols, rows) = grid.size();
    let mut text = String::new();

    for row in 0..rows {
        let mut line = String::new();
        for col in 0..cols {
            if let Some(cell) = grid.get_cell(col, row) {
                line.push(cell.c);
            }
        }
        // Trim trailing spaces from each line
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
fn test_extract_simple_text() {
    let mut grid = Grid::new(80, 24);

    // Write "Hello"
    for c in "Hello".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].starts_with("Hello"), "First line should start with 'Hello'");
}

#[test]
fn test_extract_multiple_lines() {
    let mut grid = Grid::new(80, 3);

    // Write first line
    for c in "Line 1".chars() {
        grid.put_char(c);
    }
    grid.newline();

    // Write second line
    for c in "Line 2".chars() {
        grid.put_char(c);
    }
    grid.newline();

    // Write third line
    for c in "Line 3".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines.len(), 3, "Should have 3 lines");
    assert!(lines[0].starts_with("Line 1"));
    assert!(lines[1].starts_with("Line 2"));
    assert!(lines[2].starts_with("Line 3"));
}

#[test]
fn test_extract_text_trims_trailing_spaces() {
    let mut grid = Grid::new(80, 2);

    // Write "Hello" followed by spaces (default cells are spaces)
    for c in "Hello".chars() {
        grid.put_char(c);
    }
    grid.newline();

    for c in "World".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines[0], "Hello", "Should trim trailing spaces");
    assert_eq!(lines[1], "World", "Should trim trailing spaces");
}

#[test]
fn test_extract_empty_grid() {
    let grid = Grid::new(80, 24);

    let text = get_visible_text_from_grid(&grid);

    // Empty grid should produce lines with no content (just newlines)
    assert!(text.lines().all(|line| line.is_empty()), "All lines should be empty");
}

#[test]
fn test_extract_with_empty_lines() {
    let mut grid = Grid::new(80, 5);

    // Line 0: text
    for c in "First".chars() {
        grid.put_char(c);
    }
    grid.newline();

    // Line 1: empty (skip)
    grid.newline();

    // Line 2: empty (skip)
    grid.newline();

    // Line 3: text
    for c in "Last".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines.len(), 4, "Should preserve empty lines in the middle");
    assert!(lines[0].starts_with("First"));
    assert_eq!(lines[1], "");
    assert_eq!(lines[2], "");
    assert!(lines[3].starts_with("Last"));
}

#[test]
fn test_extract_preserves_internal_spaces() {
    let mut grid = Grid::new(80, 2);

    // Write "Hello   World" with multiple internal spaces
    for c in "Hello   World".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].contains("Hello   World"), "Should preserve internal spaces");
}

#[test]
fn test_extract_special_characters() {
    let mut grid = Grid::new(80, 3);

    // Write special characters
    for c in "!@#$%^&*()".chars() {
        grid.put_char(c);
    }
    grid.newline();

    for c in "[]{}|\\/<>?".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].starts_with("!@#$%^&*()"));
    assert!(lines[1].starts_with("[]{}|\\/<>?"));
}

#[test]
fn test_extract_unicode() {
    let mut grid = Grid::new(80, 3);

    // Write unicode characters
    for c in "Hello 世界".chars() {
        grid.put_char(c);
    }
    grid.newline();

    for c in "こんにちは".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].starts_with("Hello 世界"), "Should handle Unicode correctly");
    assert!(lines[1].starts_with("こんにちは"), "Should handle Japanese characters");
}

#[test]
fn test_extract_after_clear() {
    let mut grid = Grid::new(80, 24);

    // Write some text
    for c in "Hello".chars() {
        grid.put_char(c);
    }

    // Clear and reset cursor
    grid.clear_screen();
    grid.set_cursor(0, 0);

    // Write new text
    for c in "World".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].starts_with("World"));
    assert!(!text.contains("Hello"), "Old text should be cleared");
}

#[test]
fn test_extract_full_width_characters() {
    let mut grid = Grid::new(80, 2);

    // Full-width characters (CJK)
    for c in "全角文字".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].starts_with("全角文字"), "Should handle full-width characters");
}

#[test]
fn test_extract_mixed_content() {
    let mut grid = Grid::new(80, 4);

    // Mix of ASCII, unicode, numbers, symbols
    for c in "ASCII 123".chars() {
        grid.put_char(c);
    }
    grid.newline();

    for c in "Unicode: café".chars() {
        grid.put_char(c);
    }
    grid.newline();

    for c in "Symbols: →←↑↓".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    assert!(lines[0].starts_with("ASCII 123"));
    assert!(lines[1].starts_with("Unicode: café"));
    assert!(lines[2].starts_with("Symbols: →←↑↓"));
}

#[test]
fn test_extract_wrapping_text() {
    let mut grid = Grid::new(10, 3);

    // Write text longer than width to cause wrapping
    for c in "This is a long text".chars() {
        grid.put_char(c);
    }

    let text = get_visible_text_from_grid(&grid);

    // Text should wrap across multiple lines
    assert!(text.contains("This is a"));
}

#[test]
fn test_extract_tabs() {
    let mut grid = Grid::new(80, 2);

    grid.put_char('A');
    grid.tab(); // Should move to next tab stop
    grid.put_char('B');

    let text = get_visible_text_from_grid(&grid);
    let lines: Vec<&str> = text.lines().collect();

    // Tab creates spaces, so we should see 'A' followed by spaces, then 'B'
    assert!(lines[0].starts_with("A"));
    assert!(lines[0].contains("B"));
}
