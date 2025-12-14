mod pty;
mod parser;
mod grid;

pub use pty::Pty;
pub use parser::TerminalParser;
pub use grid::{Cell, Grid, CellStyle};

use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use crate::server_client::ServerClient;
use tokio::sync::RwLock;

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
    server_client: Option<Arc<RwLock<ServerClient>>>,
    publish_output: bool,
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
            server_client: None,
            publish_output: false,
        })
    }

    /// Create a new terminal with server integration
    pub fn new_with_server(
        cols: u16,
        rows: u16,
        server_client: ServerClient,
    ) -> anyhow::Result<Self> {
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
            server_client: Some(Arc::new(RwLock::new(server_client))),
            publish_output: true,
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

    /// Poll for input commands from server and write to PTY
    /// Should be called from the main event loop periodically
    pub async fn poll_server_input(&mut self) -> anyhow::Result<()> {
        if let Some(client) = &self.server_client {
            let mut client_guard = client.write().await;

            // Poll for input commands (non-blocking)
            match client_guard.read_input().await {
                Ok(Some(cmd)) => {
                    // Write command to PTY
                    self.pty.write(cmd.as_bytes())?;
                }
                Ok(None) => {
                    // Queue empty, nothing to do
                }
                Err(e) => {
                    log::warn!("Failed to read server input: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Publish output to server if enabled
    pub async fn publish_output_if_needed(&self) {
        if !self.publish_output {
            return;
        }

        if let Some(client) = &self.server_client {
            // Get dirty lines from grid
            let dirty_lines = self.get_dirty_lines();

            if !dirty_lines.is_empty() {
                // Publish each dirty line
                let client_guard = client.read().await;
                for (line_num, content) in dirty_lines {
                    let output = format!("L{}: {}", line_num, content);
                    if let Err(e) = client_guard.publish_output(&output).await {
                        log::error!("Failed to publish output: {}", e);
                    }
                }
            }
        }
    }

    /// Extract dirty lines from grid as strings
    fn get_dirty_lines(&self) -> Vec<(usize, String)> {
        let mut grid = self.grid.lock().unwrap();
        let mut dirty_lines = Vec::new();

        if grid.is_all_dirty() {
            // Full screen is dirty, get all lines
            let (cols, rows) = grid.size();
            for row in 0..rows {
                let line = self.extract_line_text(&grid, row, cols);
                dirty_lines.push((row, line));
            }
        } else {
            // Only specific cells are dirty, group by row
            let dirty_cells = grid.dirty_cells();
            let mut dirty_rows: std::collections::HashSet<usize> = std::collections::HashSet::new();

            for &(_, row) in dirty_cells {
                dirty_rows.insert(row);
            }

            let (cols, _) = grid.size();
            for &row in &dirty_rows {
                let line = self.extract_line_text(&grid, row, cols);
                dirty_lines.push((row, line));
            }
        }

        // Clear dirty flags after extraction
        grid.clear_dirty();

        dirty_lines
    }

    /// Extract text from a single line
    fn extract_line_text(&self, grid: &Grid, row: usize, cols: usize) -> String {
        let mut line = String::new();
        for col in 0..cols {
            if let Some(cell) = grid.get_cell(col, row) {
                line.push(cell.c);
            }
        }
        line.trim_end().to_string()
    }
}
