mod pty;
mod parser;
mod grid;

pub use pty::Pty;
pub use parser::TerminalParser;
pub use grid::{Cell, Grid, CellStyle};

use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Default,
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone)]
pub enum TerminalEvent {
    Output(Vec<u8>),
    Resize(u16, u16),
    Exit,
}

pub struct Terminal {
    pub grid: Arc<Mutex<Grid>>,
    pty: Pty,
    parser: TerminalParser,
    event_tx: Sender<TerminalEvent>,
    event_rx: Receiver<TerminalEvent>,
}

impl Terminal {
    pub fn new(cols: u16, rows: u16) -> anyhow::Result<Self> {
        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        let grid = Arc::new(Mutex::new(Grid::new(cols as usize, rows as usize)));
        let pty = Pty::new(cols, rows)?;
        let parser = TerminalParser::new(grid.clone());

        Ok(Self {
            grid,
            pty,
            parser,
            event_tx,
            event_rx,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.pty.write(data)?;
        Ok(())
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.pty.resize(cols, rows)?;
        let mut grid = self.grid.lock().unwrap();
        grid.resize(cols as usize, rows as usize);
        Ok(())
    }

    pub fn read(&mut self) -> anyhow::Result<Option<Vec<u8>>> {
        self.pty.read()
    }

    pub fn process_output(&mut self, data: &[u8]) {
        self.parser.parse(data);
    }

    pub fn grid(&self) -> Arc<Mutex<Grid>> {
        self.grid.clone()
    }

    pub fn scroll_back_up(&mut self, lines: usize) {
        let mut grid = self.grid.lock().unwrap();
        grid.scroll_back_up(lines);
    }

    pub fn scroll_back_down(&mut self, lines: usize) {
        let mut grid = self.grid.lock().unwrap();
        grid.scroll_back_down(lines);
    }

    pub fn scroll_to_bottom(&mut self) {
        let mut grid = self.grid.lock().unwrap();
        grid.scroll_to_bottom();
    }
}
