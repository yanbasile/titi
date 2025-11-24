use super::{Color, Grid};
use std::sync::{Arc, Mutex};
use vte::{Params, Perform};

pub struct TerminalParser {
    grid: Arc<Mutex<Grid>>,
    performer: TerminalPerformer,
    vte_parser: vte::Parser,  // Keep parser state across calls
}

impl TerminalParser {
    pub fn new(grid: Arc<Mutex<Grid>>) -> Self {
        let performer = TerminalPerformer {
            grid: grid.clone(),
        };
        Self {
            grid,
            performer,
            vte_parser: vte::Parser::new(),
        }
    }

    pub fn parse(&mut self, data: &[u8]) {
        // Use the persistent parser to maintain state across calls
        for byte in data {
            self.vte_parser.advance(&mut self.performer, *byte);
        }
    }
}

struct TerminalPerformer {
    grid: Arc<Mutex<Grid>>,
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        let mut grid = self.grid.lock().unwrap();
        grid.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        let mut grid = self.grid.lock().unwrap();
        match byte {
            b'\n' => grid.newline(),
            b'\r' => grid.carriage_return(),
            b'\t' => grid.tab(),
            b'\x08' => grid.backspace(),
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {
        // DCS sequences - not implemented yet
    }

    fn put(&mut self, _byte: u8) {
        // DCS data - not implemented yet
    }

    fn unhook(&mut self) {
        // End of DCS - not implemented yet
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // OSC sequences (Operating System Command) - not implemented yet
        // These are used for things like setting window title, etc.
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'A' => {
                // Cursor up
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as i32;
                let mut grid = self.grid.lock().unwrap();
                grid.move_cursor(0, -n);
            }
            'B' => {
                // Cursor down
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as i32;
                let mut grid = self.grid.lock().unwrap();
                grid.move_cursor(0, n);
            }
            'C' => {
                // Cursor forward
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as i32;
                let mut grid = self.grid.lock().unwrap();
                grid.move_cursor(n, 0);
            }
            'D' => {
                // Cursor back
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(1) as i32;
                let mut grid = self.grid.lock().unwrap();
                grid.move_cursor(-n, 0);
            }
            'H' | 'f' => {
                // Cursor position
                let mut iter = params.iter();
                let y = iter
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1)
                    .saturating_sub(1) as usize;
                let x = iter
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1)
                    .saturating_sub(1) as usize;
                let mut grid = self.grid.lock().unwrap();
                grid.set_cursor(x, y);
            }
            'J' => {
                // Erase in display
                let n = params.iter().next().and_then(|p| p.first()).copied().unwrap_or(0);
                let mut grid = self.grid.lock().unwrap();
                match n {
                    0 | 1 | 2 | 3 => grid.clear_screen(),
                    _ => {}
                }
            }
            'K' => {
                // Erase in line
                let mut grid = self.grid.lock().unwrap();
                grid.clear_line();
            }
            'm' => {
                // SGR - Select Graphic Rendition
                self.handle_sgr(params);
            }
            'r' => {
                // Set scroll region
                let mut iter = params.iter();
                let top = iter
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(1)
                    .saturating_sub(1) as usize;
                let mut grid = self.grid.lock().unwrap();
                let bottom = iter
                    .next()
                    .and_then(|p| p.first())
                    .copied()
                    .unwrap_or(grid.size().1 as u16)
                    .saturating_sub(1) as usize;
                grid.set_scroll_region(top, bottom);
            }
            's' => {
                // Save cursor position
                let mut grid = self.grid.lock().unwrap();
                grid.save_cursor();
            }
            'u' => {
                // Restore cursor position
                let mut grid = self.grid.lock().unwrap();
                grid.restore_cursor();
            }
            _ => {
                // Unhandled CSI sequence
                log::debug!("Unhandled CSI: {}", c);
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences - not fully implemented
    }
}

impl TerminalPerformer {
    fn handle_sgr(&mut self, params: &Params) {
        // Get current style from grid's internal state
        let mut current_style = {
            let grid = self.grid.lock().unwrap();
            grid.get_current_style()
        };

        let mut param_iter = params.iter();

        while let Some(param) = param_iter.next() {
            let value = param.first().copied().unwrap_or(0);

            match value {
                0 => {
                    // Reset
                    current_style = Default::default();
                }
                1 => current_style.bold = true,
                3 => current_style.italic = true,
                4 => current_style.underline = true,
                7 => current_style.inverse = true,
                9 => current_style.strikethrough = true,
                22 => current_style.bold = false,
                23 => current_style.italic = false,
                24 => current_style.underline = false,
                27 => current_style.inverse = false,
                29 => current_style.strikethrough = false,
                30 => current_style.fg = Color::Black,
                31 => current_style.fg = Color::Red,
                32 => current_style.fg = Color::Green,
                33 => current_style.fg = Color::Yellow,
                34 => current_style.fg = Color::Blue,
                35 => current_style.fg = Color::Magenta,
                36 => current_style.fg = Color::Cyan,
                37 => current_style.fg = Color::White,
                38 => {
                    // Extended foreground color
                    if let Some(next) = param_iter.next() {
                        if let Some(&mode) = next.first() {
                            if mode == 5 {
                                // 256 color
                                if let Some(color_param) = param_iter.next() {
                                    if let Some(&color) = color_param.first() {
                                        current_style.fg = self.map_256_color(color as u8);
                                    }
                                }
                            } else if mode == 2 {
                                // RGB color
                                let r = param_iter.next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                                let g = param_iter.next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                                let b = param_iter.next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                                current_style.fg = Color::Rgb(r, g, b);
                            }
                        }
                    }
                }
                39 => current_style.fg = Color::Default,
                40 => current_style.bg = Color::Black,
                41 => current_style.bg = Color::Red,
                42 => current_style.bg = Color::Green,
                43 => current_style.bg = Color::Yellow,
                44 => current_style.bg = Color::Blue,
                45 => current_style.bg = Color::Magenta,
                46 => current_style.bg = Color::Cyan,
                47 => current_style.bg = Color::White,
                48 => {
                    // Extended background color (similar to 38)
                    if let Some(next) = param_iter.next() {
                        if let Some(&mode) = next.first() {
                            if mode == 5 {
                                if let Some(color_param) = param_iter.next() {
                                    if let Some(&color) = color_param.first() {
                                        current_style.bg = self.map_256_color(color as u8);
                                    }
                                }
                            } else if mode == 2 {
                                let r = param_iter.next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                                let g = param_iter.next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                                let b = param_iter.next().and_then(|p| p.first()).copied().unwrap_or(0) as u8;
                                current_style.bg = Color::Rgb(r, g, b);
                            }
                        }
                    }
                }
                49 => current_style.bg = Color::Default,
                90 => current_style.fg = Color::BrightBlack,
                91 => current_style.fg = Color::BrightRed,
                92 => current_style.fg = Color::BrightGreen,
                93 => current_style.fg = Color::BrightYellow,
                94 => current_style.fg = Color::BrightBlue,
                95 => current_style.fg = Color::BrightMagenta,
                96 => current_style.fg = Color::BrightCyan,
                97 => current_style.fg = Color::BrightWhite,
                100 => current_style.bg = Color::BrightBlack,
                101 => current_style.bg = Color::BrightRed,
                102 => current_style.bg = Color::BrightGreen,
                103 => current_style.bg = Color::BrightYellow,
                104 => current_style.bg = Color::BrightBlue,
                105 => current_style.bg = Color::BrightMagenta,
                106 => current_style.bg = Color::BrightCyan,
                107 => current_style.bg = Color::BrightWhite,
                _ => {}
            }
        }

        let mut grid = self.grid.lock().unwrap();
        grid.set_style(current_style);
    }

    fn map_256_color(&self, color: u8) -> Color {
        match color {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            8 => Color::BrightBlack,
            9 => Color::BrightRed,
            10 => Color::BrightGreen,
            11 => Color::BrightYellow,
            12 => Color::BrightBlue,
            13 => Color::BrightMagenta,
            14 => Color::BrightCyan,
            15 => Color::BrightWhite,
            16..=231 => {
                // 216 color cube
                let idx = color - 16;
                let r = (idx / 36) * 51;
                let g = ((idx % 36) / 6) * 51;
                let b = (idx % 6) * 51;
                Color::Rgb(r, g, b)
            }
            232..=255 => {
                // Grayscale
                let gray = (color - 232) * 10 + 8;
                Color::Rgb(gray, gray, gray)
            }
        }
    }
}
