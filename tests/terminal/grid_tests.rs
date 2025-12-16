use titi::terminal::{CellStyle, Color, Grid};

#[test]
fn test_grid_new_initializes_with_empty_cells() {
    let grid = Grid::new(80, 24);
    let (cols, rows) = grid.size();

    assert_eq!(cols, 80);
    assert_eq!(rows, 24);

    // Check that all cells are default (space character)
    for y in 0..rows {
        for x in 0..cols {
            let cell = grid.get_cell(x, y).unwrap();
            assert_eq!(cell.c, ' ');
        }
    }
}

#[test]
fn test_grid_put_char_at_cursor() {
    let mut grid = Grid::new(80, 24);

    grid.put_char('H');
    grid.put_char('e');
    grid.put_char('l');
    grid.put_char('l');
    grid.put_char('o');

    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'H');
    assert_eq!(grid.get_cell(1, 0).unwrap().c, 'e');
    assert_eq!(grid.get_cell(2, 0).unwrap().c, 'l');
    assert_eq!(grid.get_cell(3, 0).unwrap().c, 'l');
    assert_eq!(grid.get_cell(4, 0).unwrap().c, 'o');

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 5);
    assert_eq!(y, 0);
}

#[test]
fn test_grid_cursor_wraps_at_end_of_line() {
    let mut grid = Grid::new(5, 3);

    for _ in 0..5 {
        grid.put_char('X');
    }

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 5);
    assert_eq!(y, 0);

    // Next character should wrap
    grid.put_char('Y');
    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 1);
    assert_eq!(y, 1);
    assert_eq!(grid.get_cell(0, 1).unwrap().c, 'Y');
}

#[test]
fn test_grid_newline_moves_cursor_down() {
    let mut grid = Grid::new(80, 24);

    grid.put_char('A');
    let (_, y1) = grid.cursor_pos();

    grid.newline();
    let (x, y2) = grid.cursor_pos();

    assert_eq!(y2, y1 + 1);
    assert_eq!(x, 0); // Newline resets cursor to column 0 (Unix terminal behavior)
}

#[test]
fn test_grid_carriage_return_moves_to_start() {
    let mut grid = Grid::new(80, 24);

    grid.put_char('A');
    grid.put_char('B');
    grid.put_char('C');

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 3);

    grid.carriage_return();
    let (x2, y2) = grid.cursor_pos();
    assert_eq!(x2, 0);
    assert_eq!(y2, y);
}

#[test]
fn test_grid_backspace_moves_cursor_back() {
    let mut grid = Grid::new(80, 24);

    grid.put_char('A');
    grid.put_char('B');
    grid.put_char('C');

    let (x, _) = grid.cursor_pos();
    grid.backspace();
    let (x2, _) = grid.cursor_pos();

    assert_eq!(x2, x - 1);
}

#[test]
fn test_grid_tab_advances_to_next_tab_stop() {
    let mut grid = Grid::new(80, 24);

    grid.tab();
    let (x, _) = grid.cursor_pos();
    assert_eq!(x, 8); // Default tab stops are every 8 columns

    grid.tab();
    let (x, _) = grid.cursor_pos();
    assert_eq!(x, 16);
}

#[test]
fn test_grid_set_cursor_position() {
    let mut grid = Grid::new(80, 24);

    grid.set_cursor(10, 5);
    let (x, y) = grid.cursor_pos();

    assert_eq!(x, 10);
    assert_eq!(y, 5);
}

#[test]
fn test_grid_set_cursor_clamps_to_bounds() {
    let mut grid = Grid::new(80, 24);

    grid.set_cursor(100, 50);
    let (x, y) = grid.cursor_pos();

    assert_eq!(x, 79); // Should clamp to max column
    assert_eq!(y, 23); // Should clamp to max row
}

#[test]
fn test_grid_move_cursor_relative() {
    let mut grid = Grid::new(80, 24);

    grid.set_cursor(10, 10);
    grid.move_cursor(5, 3);

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 15);
    assert_eq!(y, 13);
}

#[test]
fn test_grid_move_cursor_negative() {
    let mut grid = Grid::new(80, 24);

    grid.set_cursor(10, 10);
    grid.move_cursor(-5, -3);

    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 5);
    assert_eq!(y, 7);
}

#[test]
fn test_grid_clear_screen_fills_with_spaces() {
    let mut grid = Grid::new(80, 24);

    // Fill with some characters
    for i in 0..100 {
        grid.put_char(('A' as u8 + (i % 26)) as char);
    }

    grid.clear_screen();

    // Check all cells are spaces
    for y in 0..24 {
        for x in 0..80 {
            assert_eq!(grid.get_cell(x, y).unwrap().c, ' ');
        }
    }
}

#[test]
fn test_grid_clear_line_fills_line_with_spaces() {
    let mut grid = Grid::new(80, 24);

    grid.set_cursor(0, 5);
    for _ in 0..80 {
        grid.put_char('X');
    }

    grid.set_cursor(0, 5);
    grid.clear_line();

    // Check line 5 is all spaces
    for x in 0..80 {
        assert_eq!(grid.get_cell(x, 5).unwrap().c, ' ');
    }

    // Check other lines are unchanged
    assert_eq!(grid.get_cell(0, 4).unwrap().c, ' ');
}

#[test]
fn test_grid_scroll_up_moves_content() {
    let mut grid = Grid::new(10, 5);

    // Fill each row with a different character
    for row in 0..5 {
        grid.set_cursor(0, row);
        let ch = ('A' as u8 + row as u8) as char;
        for _ in 0..10 {
            grid.put_char(ch);
        }
    }

    grid.scroll_up(1);

    // First row should now contain what was in second row
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'B');
    // Last row should be empty
    assert_eq!(grid.get_cell(0, 4).unwrap().c, ' ');
}

#[test]
fn test_grid_resize_smaller_truncates() {
    let mut grid = Grid::new(80, 24);

    grid.put_char('H');
    grid.put_char('e');
    grid.put_char('l');
    grid.put_char('l');
    grid.put_char('o');

    grid.resize(40, 12);

    let (cols, rows) = grid.size();
    assert_eq!(cols, 40);
    assert_eq!(rows, 12);

    // Content should still be there
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'H');
    assert_eq!(grid.get_cell(4, 0).unwrap().c, 'o');
}

#[test]
fn test_grid_resize_larger_preserves_content() {
    let mut grid = Grid::new(10, 5);

    grid.put_char('T');
    grid.put_char('e');
    grid.put_char('s');
    grid.put_char('t');

    grid.resize(20, 10);

    let (cols, rows) = grid.size();
    assert_eq!(cols, 20);
    assert_eq!(rows, 10);

    // Original content should be preserved
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'T');
    assert_eq!(grid.get_cell(1, 0).unwrap().c, 'e');
    assert_eq!(grid.get_cell(2, 0).unwrap().c, 's');
    assert_eq!(grid.get_cell(3, 0).unwrap().c, 't');
}

#[test]
fn test_grid_save_and_restore_cursor() {
    let mut grid = Grid::new(80, 24);

    grid.set_cursor(10, 5);
    grid.save_cursor();

    grid.set_cursor(20, 15);
    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 20);
    assert_eq!(y, 15);

    grid.restore_cursor();
    let (x, y) = grid.cursor_pos();
    assert_eq!(x, 10);
    assert_eq!(y, 5);
}

#[test]
fn test_grid_set_scroll_region() {
    let mut grid = Grid::new(80, 24);

    grid.set_scroll_region(5, 15);

    // Scrolling should only affect the region
    // This is tested indirectly through scroll operations
}

#[test]
fn test_cell_style_colors() {
    let mut grid = Grid::new(80, 24);

    let mut style = CellStyle::default();
    style.fg = Color::Red;
    style.bg = Color::Blue;

    grid.set_style(style);
    grid.put_char('X');

    let cell = grid.get_cell(0, 0).unwrap();
    assert_eq!(cell.c, 'X');
    assert_eq!(cell.style.fg, Color::Red);
    assert_eq!(cell.style.bg, Color::Blue);
}

#[test]
fn test_cell_style_attributes() {
    let mut grid = Grid::new(80, 24);

    let mut style = CellStyle::default();
    style.bold = true;
    style.italic = true;
    style.underline = true;

    grid.set_style(style);
    grid.put_char('B');

    let cell = grid.get_cell(0, 0).unwrap();
    assert!(cell.style.bold);
    assert!(cell.style.italic);
    assert!(cell.style.underline);
}

#[test]
fn test_grid_scrolling_with_region() {
    let mut grid = Grid::new(10, 10);

    // Set scroll region from row 2 to row 7
    grid.set_scroll_region(2, 7);

    // Fill region with identifiable content
    for row in 2..=7 {
        grid.set_cursor(0, row);
        let ch = ('A' as u8 + row as u8) as char;
        grid.put_char(ch);
    }

    // Rows outside region should not be affected by scroll
    grid.set_cursor(0, 0);
    grid.put_char('X');
    grid.set_cursor(0, 9);
    grid.put_char('Y');

    grid.scroll_up(1);

    // Check that rows outside scroll region are unchanged
    assert_eq!(grid.get_cell(0, 0).unwrap().c, 'X');
    assert_eq!(grid.get_cell(0, 9).unwrap().c, 'Y');
}
