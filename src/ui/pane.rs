use crate::terminal::Terminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneId(pub usize);

pub struct Pane {
    pub id: PaneId,
    pub terminal: Terminal,
    pub title: String,
}

impl Pane {
    pub fn new(id: PaneId, terminal: Terminal) -> Self {
        Self {
            id,
            terminal,
            title: format!("Terminal {}", id.0),
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}
