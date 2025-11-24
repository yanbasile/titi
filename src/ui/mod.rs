mod pane;
mod layout;

pub use pane::{Pane, PaneId};
pub use layout::{Layout, LayoutNode, SplitDirection};

use crate::terminal::Terminal;
use std::collections::HashMap;

pub struct PaneManager {
    panes: HashMap<PaneId, Pane>,
    layout: Layout,
    active_pane: Option<PaneId>,
    next_id: usize,
}

impl PaneManager {
    pub fn new() -> Self {
        Self {
            panes: HashMap::new(),
            layout: Layout::new(),
            active_pane: None,
            next_id: 0,
        }
    }

    pub fn create_pane(&mut self, cols: u16, rows: u16) -> anyhow::Result<PaneId> {
        let id = PaneId(self.next_id);
        self.next_id += 1;

        let terminal = Terminal::new(cols, rows)?;
        let pane = Pane::new(id, terminal);

        self.panes.insert(id, pane);

        if self.active_pane.is_none() {
            self.active_pane = Some(id);
            self.layout.set_root(id);
        }

        Ok(id)
    }

    pub fn split_pane(
        &mut self,
        pane_id: PaneId,
        direction: SplitDirection,
        cols: u16,
        rows: u16,
    ) -> anyhow::Result<PaneId> {
        let new_id = self.create_pane(cols, rows)?;
        self.layout.split(pane_id, new_id, direction);
        Ok(new_id)
    }

    pub fn close_pane(&mut self, pane_id: PaneId) {
        self.panes.remove(&pane_id);
        self.layout.remove(pane_id);

        if self.active_pane == Some(pane_id) {
            self.active_pane = self.panes.keys().next().copied();
        }
    }

    pub fn get_pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.get(&id)
    }

    pub fn get_pane_mut(&mut self, id: PaneId) -> Option<&mut Pane> {
        self.panes.get_mut(&id)
    }

    pub fn active_pane(&self) -> Option<PaneId> {
        self.active_pane
    }

    pub fn set_active_pane(&mut self, id: PaneId) {
        if self.panes.contains_key(&id) {
            self.active_pane = Some(id);
        }
    }

    pub fn panes(&self) -> &HashMap<PaneId, Pane> {
        &self.panes
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }
}

impl Default for PaneManager {
    fn default() -> Self {
        Self::new()
    }
}
