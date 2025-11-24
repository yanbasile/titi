use super::Color;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellStyle {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub inverse: bool,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            fg: Color::Default,
            bg: Color::Default,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            inverse: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub style: CellStyle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            style: CellStyle::default(),
        }
    }
}

pub struct Grid {
    cells: Vec<Cell>,
    cols: usize,
    rows: usize,
    cursor_x: usize,
    cursor_y: usize,
    current_style: CellStyle,
    cursor_visible: bool,
    scroll_top: usize,
    scroll_bottom: usize,
    saved_cursor: (usize, usize),
    // Scrollback buffer
    scrollback: Vec<Vec<Cell>>,
    max_scrollback: usize,
    scroll_offset: usize, // 0 = at bottom (current), >0 = scrolled back
    // Dirty tracking for performance
    dirty_cells: HashSet<(usize, usize)>, // (col, row) of dirty cells
    all_dirty: bool, // True if entire screen needs redraw
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = vec![Cell::default(); cols * rows];
        Self {
            cells,
            cols,
            rows,
            cursor_x: 0,
            cursor_y: 0,
            current_style: CellStyle::default(),
            cursor_visible: true,
            scroll_top: 0,
            scroll_bottom: rows - 1,
            saved_cursor: (0, 0),
            scrollback: Vec::new(),
            max_scrollback: 10000, // Store up to 10000 lines
            scroll_offset: 0,
            dirty_cells: HashSet::new(),
            all_dirty: true, // Start with full redraw
        }
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut new_cells = vec![Cell::default(); cols * rows];

        let min_rows = self.rows.min(rows);
        let min_cols = self.cols.min(cols);

        for y in 0..min_rows {
            for x in 0..min_cols {
                let old_idx = y * self.cols + x;
                let new_idx = y * cols + x;
                new_cells[new_idx] = self.cells[old_idx].clone();
            }
        }

        self.cells = new_cells;
        self.cols = cols;
        self.rows = rows;
        self.cursor_x = self.cursor_x.min(cols - 1);
        self.cursor_y = self.cursor_y.min(rows - 1);
        self.scroll_bottom = rows - 1;

        // Mark all as dirty after resize
        self.all_dirty = true;
    }

    pub fn put_char(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.cursor_y += 1;
            if self.cursor_y > self.scroll_bottom {
                self.scroll_up(1);
                self.cursor_y = self.scroll_bottom;
            }
        }

        let idx = self.cursor_y * self.cols + self.cursor_x;
        if idx < self.cells.len() {
            self.cells[idx] = Cell {
                c,
                style: self.current_style,
            };
            // Mark cell as dirty
            self.dirty_cells.insert((self.cursor_x, self.cursor_y));
        }
        self.cursor_x += 1;
    }

    pub fn newline(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y > self.scroll_bottom {
            self.scroll_up(1);
            self.cursor_y = self.scroll_bottom;
        }
        // In Unix terminals, newline typically includes carriage return
        self.cursor_x = 0;
    }

    pub fn carriage_return(&mut self) {
        self.cursor_x = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    pub fn tab(&mut self) {
        let next_tab = ((self.cursor_x / 8) + 1) * 8;
        self.cursor_x = next_tab.min(self.cols - 1);
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        self.cursor_x = x.min(self.cols - 1);
        self.cursor_y = y.min(self.rows - 1);
    }

    pub fn move_cursor(&mut self, dx: i32, dy: i32) {
        let new_x = (self.cursor_x as i32 + dx).clamp(0, self.cols as i32 - 1) as usize;
        let new_y = (self.cursor_y as i32 + dy).clamp(0, self.rows as i32 - 1) as usize;
        self.cursor_x = new_x;
        self.cursor_y = new_y;
    }

    pub fn clear_screen(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
        // Mark all as dirty
        self.all_dirty = true;
    }

    pub fn clear_line(&mut self) {
        let start = self.cursor_y * self.cols;
        let end = start + self.cols;
        for idx in start..end {
            if idx < self.cells.len() {
                self.cells[idx] = Cell::default();
            }
        }
        // Mark entire line as dirty
        for x in 0..self.cols {
            self.dirty_cells.insert((x, self.cursor_y));
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        let start_row = self.scroll_top;
        let end_row = self.scroll_bottom + 1;

        for _ in 0..lines {
            // Save the top line to scrollback before scrolling
            if start_row == 0 {
                let mut line = Vec::with_capacity(self.cols);
                for x in 0..self.cols {
                    let idx = start_row * self.cols + x;
                    if idx < self.cells.len() {
                        line.push(self.cells[idx].clone());
                    }
                }

                // Add to scrollback
                self.scrollback.push(line);

                // Limit scrollback size
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.remove(0);
                }
            }

            // Move rows up
            for y in start_row..(end_row - 1) {
                for x in 0..self.cols {
                    let src_idx = (y + 1) * self.cols + x;
                    let dst_idx = y * self.cols + x;
                    if src_idx < self.cells.len() && dst_idx < self.cells.len() {
                        self.cells[dst_idx] = self.cells[src_idx].clone();
                    }
                }
            }

            // Clear bottom row
            let bottom_row = end_row - 1;
            for x in 0..self.cols {
                let idx = bottom_row * self.cols + x;
                if idx < self.cells.len() {
                    self.cells[idx] = Cell::default();
                }
            }
        }

        // Reset scroll offset when new content arrives
        self.scroll_offset = 0;

        // Mark all as dirty after scroll
        self.all_dirty = true;
    }

    pub fn set_style(&mut self, style: CellStyle) {
        self.current_style = style;
    }

    pub fn get_current_style(&self) -> CellStyle {
        self.current_style
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.cols || y >= self.rows {
            return None;
        }

        // If scrolled back, get from scrollback buffer
        if self.scroll_offset > 0 {
            // Calculate which line in scrollback to show
            let scrollback_line = self.scrollback.len().saturating_sub(self.scroll_offset) + y;

            if scrollback_line < self.scrollback.len() {
                // Get from scrollback
                return self.scrollback[scrollback_line].get(x);
            } else {
                // This line is beyond scrollback, return empty
                return None;
            }
        }

        // Not scrolled back, get from current cells
        let idx = y * self.cols + x;
        self.cells.get(idx)
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.cols, self.rows)
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn save_cursor(&mut self) {
        self.saved_cursor = (self.cursor_x, self.cursor_y);
    }

    pub fn restore_cursor(&mut self) {
        (self.cursor_x, self.cursor_y) = self.saved_cursor;
    }

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        self.scroll_top = top.min(self.rows - 1);
        self.scroll_bottom = bottom.min(self.rows - 1);
    }

    pub fn scroll_back_up(&mut self, lines: usize) {
        let max_offset = self.scrollback.len();
        let old_offset = self.scroll_offset;
        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
        // Mark all dirty if scroll changed
        if old_offset != self.scroll_offset {
            self.all_dirty = true;
        }
    }

    pub fn scroll_back_down(&mut self, lines: usize) {
        let old_offset = self.scroll_offset;
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        // Mark all dirty if scroll changed
        if old_offset != self.scroll_offset {
            self.all_dirty = true;
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        let old_offset = self.scroll_offset;
        self.scroll_offset = 0;
        // Mark all dirty if scroll changed
        if old_offset != self.scroll_offset {
            self.all_dirty = true;
        }
    }

    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset == 0
    }

    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    // Dirty tracking methods
    pub fn is_all_dirty(&self) -> bool {
        self.all_dirty
    }

    pub fn dirty_cells(&self) -> &HashSet<(usize, usize)> {
        &self.dirty_cells
    }

    pub fn clear_dirty(&mut self) {
        self.all_dirty = false;
        self.dirty_cells.clear();
    }

    pub fn mark_all_dirty(&mut self) {
        self.all_dirty = true;
    }
}
