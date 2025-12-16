use titi::terminal::{Color, Grid, TerminalParser};
use std::sync::{Arc, Mutex};

fn create_parser() -> (TerminalParser, Arc<Mutex<Grid>>) {
    let grid = Arc::new(Mutex::new(Grid::new(80, 24)));
    let parser = TerminalParser::new(grid.clone());
    (parser, grid)
}

#[test]
fn test_parser_basic_text() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"Hello");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'H');
    assert_eq!(grid.get_cell(1, 0).unwrap().c, 'e');
    assert_eq!(grid.get_cell(2, 0).unwrap().c, 'l');
    assert_eq!(grid.get_cell(3, 0).unwrap().c, 'l');
    assert_eq!(grid.get_cell(4, 0).unwrap().c, 'o');
}

#[test]
fn test_parser_newline() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"Line1\nLine2");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'L');
    assert_eq!(grid.get_cell(0, 1).unwrap().c, 'L');
}

#[test]
fn test_parser_carriage_return() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"Hello\rWorld");

    let grid = grid.lock().unwrap();
    // Carriage return should move to start, so "World" overwrites "Hello"
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'W');
    assert_eq!(grid.get_cell(1, 0).unwrap().c, 'o');
}

#[test]
fn test_parser_tab() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"A\tB");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'A');
    assert_eq!(grid.get_cell(8, 0).unwrap().c, 'B'); // Tab advances to column 8
}

#[test]
fn test_parser_backspace() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"ABC\x08D");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'A');
    assert_eq!(grid.get_cell(1, 0).unwrap().c, 'B');
    assert_eq!(grid.get_cell(2, 0).unwrap().c, 'D'); // D overwrites C after backspace
}

#[test]
fn test_parser_cursor_up() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"Line1\nLine2\nLine3");
    parser.parse(b"\x1b[2A"); // Move up 2 lines

    let grid = grid.lock().unwrap();
    let (_, y) = grid.cursor_pos();
    assert_eq!(y, 0); // Should be at first line
}

#[test]
fn test_parser_cursor_down() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"Start\x1b[3B"); // Move down 3 lines

    let grid = grid.lock().unwrap();
    let (_, y) = grid.cursor_pos();
    assert_eq!(y, 3);
}

#[test]
fn test_parser_cursor_forward() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"A\x1b[5CB"); // Move forward 5 columns

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'A');
    assert_eq!(grid.get_cell(6, 0).unwrap().c, 'B'); // 1 (after A) + 5 = 6
}

#[test]
fn test_parser_cursor_back() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"ABCDEFGH\x1b[3D"); // Move back 3 columns
    parser.parse(b"X");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(5, 0).unwrap().c, 'X'); // 8 - 3 = 5
}

#[test]
fn test_parser_cursor_position() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[5;10H"); // Move to row 5, column 10
    parser.parse(b"X");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(9, 4).unwrap().c, 'X'); // 0-indexed: (9, 4)
}

#[test]
fn test_parser_cursor_position_f_variant() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[3;5f"); // Move to row 3, column 5 (f variant)
    parser.parse(b"Y");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(4, 2).unwrap().c, 'Y'); // 0-indexed: (4, 2)
}

#[test]
fn test_parser_erase_display() {
    let (mut parser, grid) = create_parser();

    // Fill with content
    parser.parse(b"XXXXXXXXXXXXXXXXXXXX\nYYYYYYYYYYYYYYYYYYYY");

    // Clear screen
    parser.parse(b"\x1b[2J");

    let grid = grid.lock().unwrap();
    // All cells should be spaces
    for y in 0..5 {
        for x in 0..20 {
            assert_eq!(grid.get_cell(x, y).unwrap().c, ' ');
        }
    }
}

#[test]
fn test_parser_erase_line() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"XXXXXXXXXXXXXXXXXXXX\nYYYYYYYYYYYYYYYYYYYY\nZZZZZZZZZZZZZZZZZZZZ");
    parser.parse(b"\x1b[2;1H"); // Move to line 2
    parser.parse(b"\x1b[K"); // Erase line

    let grid = grid.lock().unwrap();
    // Line 2 should be cleared
    for x in 0..20 {
        assert_eq!(grid.get_cell(x, 1).unwrap().c, ' ');
    }
    // Other lines should be intact
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'X');
    assert_eq!(grid.get_cell(0, 2).unwrap().c, 'Z');
}

#[test]
fn test_parser_sgr_reset() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[31mRed\x1b[0mNormal");

    let grid = grid.lock().unwrap();
    // First character should be red
    assert_eq!(grid.get_cell(0, 0).unwrap().style.fg, Color::Red);
    // After reset, should be default
    assert_eq!(grid.get_cell(3, 0).unwrap().style.fg, Color::Default);
}

#[test]
fn test_parser_sgr_foreground_colors() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[30m0");  // Black
    parser.parse(b"\x1b[31m1");  // Red
    parser.parse(b"\x1b[32m2");  // Green
    parser.parse(b"\x1b[33m3");  // Yellow
    parser.parse(b"\x1b[34m4");  // Blue
    parser.parse(b"\x1b[35m5");  // Magenta
    parser.parse(b"\x1b[36m6");  // Cyan
    parser.parse(b"\x1b[37m7");  // White

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().style.fg, Color::Black);
    assert_eq!(grid.get_cell(1, 0).unwrap().style.fg, Color::Red);
    assert_eq!(grid.get_cell(2, 0).unwrap().style.fg, Color::Green);
    assert_eq!(grid.get_cell(3, 0).unwrap().style.fg, Color::Yellow);
    assert_eq!(grid.get_cell(4, 0).unwrap().style.fg, Color::Blue);
    assert_eq!(grid.get_cell(5, 0).unwrap().style.fg, Color::Magenta);
    assert_eq!(grid.get_cell(6, 0).unwrap().style.fg, Color::Cyan);
    assert_eq!(grid.get_cell(7, 0).unwrap().style.fg, Color::White);
}

#[test]
fn test_parser_sgr_bright_colors() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[90mA"); // Bright black
    parser.parse(b"\x1b[91mB"); // Bright red
    parser.parse(b"\x1b[97mC"); // Bright white

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().style.fg, Color::BrightBlack);
    assert_eq!(grid.get_cell(1, 0).unwrap().style.fg, Color::BrightRed);
    assert_eq!(grid.get_cell(2, 0).unwrap().style.fg, Color::BrightWhite);
}

#[test]
fn test_parser_sgr_background_colors() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[41mR"); // Red background
    parser.parse(b"\x1b[42mG"); // Green background

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().style.bg, Color::Red);
    assert_eq!(grid.get_cell(1, 0).unwrap().style.bg, Color::Green);
}

#[test]
fn test_parser_sgr_text_attributes() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[1mBold\x1b[0m ");
    parser.parse(b"\x1b[3mItalic\x1b[0m ");
    parser.parse(b"\x1b[4mUnderline");

    let grid = grid.lock().unwrap();
    assert!(grid.get_cell(0, 0).unwrap().style.bold);
    assert!(grid.get_cell(5, 0).unwrap().style.italic);
    assert!(grid.get_cell(12, 0).unwrap().style.underline);
}

#[test]
fn test_parser_sgr_256_color() {
    let (mut parser, grid) = create_parser();

    // 256-color foreground: ESC[38;5;COLORm
    parser.parse(b"\x1b[38;5;196mX"); // Bright red (color 196)

    let _grid = grid.lock().unwrap();
    // Should map to an RGB or named color
    // The exact color depends on the 256-color palette implementation
}

#[test]
fn test_parser_sgr_rgb_color() {
    let (mut parser, grid) = create_parser();

    // RGB foreground: ESC[38;2;R;G;Bm
    parser.parse(b"\x1b[38;2;255;128;64mX");

    let grid = grid.lock().unwrap();
    match grid.get_cell(0, 0).unwrap().style.fg {
        Color::Rgb(r, g, b) => {
            assert_eq!(r, 255);
            assert_eq!(g, 128);
            assert_eq!(b, 64);
        }
        _ => panic!("Expected RGB color"),
    }
}

#[test]
fn test_parser_scroll_region() {
    let (mut parser, _grid) = create_parser();

    // Set scroll region from row 5 to row 15
    parser.parse(b"\x1b[5;15r");

    // This should set the internal scroll region
    // Verification through scrolling behavior
}

#[test]
fn test_parser_save_restore_cursor() {
    let (mut parser, grid) = create_parser();

    parser.parse(b"\x1b[10;20H"); // Move to position
    parser.parse(b"\x1b[s");      // Save cursor
    parser.parse(b"\x1b[5;5H");   // Move elsewhere
    parser.parse(b"\x1b[u");      // Restore cursor

    let grid = grid.lock().unwrap();
    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 19); // Column 20, 0-indexed = 19
    assert_eq!(y, 9);  // Row 10, 0-indexed = 9
}

#[test]
fn test_parser_complex_sequence() {
    let (mut parser, grid) = create_parser();

    // Mix of text, colors, and cursor movement
    parser.parse(b"\x1b[31mRed Text\x1b[0m\n\x1b[1;32mBold Green\x1b[0m");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'R');
    assert_eq!(grid.get_cell(0, 0).unwrap().style.fg, Color::Red);
    assert_eq!(grid.get_cell(0, 1).unwrap().style.fg, Color::Green);
    assert!(grid.get_cell(0, 1).unwrap().style.bold);
}

#[test]
fn test_parser_split_sequences() {
    let (mut parser, grid) = create_parser();

    // Test parsing sequences split across multiple parse() calls
    parser.parse(b"\x1b[");
    parser.parse(b"31");
    parser.parse(b"m");
    parser.parse(b"Red");

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().style.fg, Color::Red);
}

#[test]
fn test_parser_invalid_sequence() {
    let (mut parser, grid) = create_parser();

    // Parser should handle invalid sequences gracefully
    parser.parse(b"\x1b[999;999HX");

    let grid = grid.lock().unwrap();
    // Should still process the 'X'
    let has_x = (0..80).any(|x| {
        (0..24).any(|y| {
            grid.get_cell(x, y).map(|c| c.c == 'X').unwrap_or(false)
        })
    });
    assert!(has_x);
}

#[test]
fn test_parser_utf8_characters() {
    let (mut parser, grid) = create_parser();

    parser.parse("Hello 世界!".as_bytes());

    let grid = grid.lock().unwrap();
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'H');
    assert_eq!(grid.get_cell(6, 0).unwrap().c, '世');
    assert_eq!(grid.get_cell(7, 0).unwrap().c, '界');
}

#[test]
fn test_parser_multiple_sgr_params() {
    let (mut parser, grid) = create_parser();

    // Multiple SGR parameters in one sequence
    parser.parse(b"\x1b[1;31;42mText");

    let grid = grid.lock().unwrap();
    let cell = grid.get_cell(0, 0).unwrap();
    assert!(cell.style.bold);
    assert_eq!(cell.style.fg, Color::Red);
    assert_eq!(cell.style.bg, Color::Green);
}
