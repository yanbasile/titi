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

    pub fn navigate_up(&mut self) {
        if let Some(current_id) = self.active_pane {
            if let Some(next_id) = self.find_pane_in_direction(current_id, NavigationDirection::Up) {
                self.active_pane = Some(next_id);
            }
        }
    }

    pub fn navigate_down(&mut self) {
        if let Some(current_id) = self.active_pane {
            if let Some(next_id) = self.find_pane_in_direction(current_id, NavigationDirection::Down) {
                self.active_pane = Some(next_id);
            }
        }
    }

    pub fn navigate_left(&mut self) {
        if let Some(current_id) = self.active_pane {
            if let Some(next_id) = self.find_pane_in_direction(current_id, NavigationDirection::Left) {
                self.active_pane = Some(next_id);
            }
        }
    }

    pub fn navigate_right(&mut self) {
        if let Some(current_id) = self.active_pane {
            if let Some(next_id) = self.find_pane_in_direction(current_id, NavigationDirection::Right) {
                self.active_pane = Some(next_id);
            }
        }
    }

    fn find_pane_in_direction(&self, current_id: PaneId, direction: NavigationDirection) -> Option<PaneId> {
        // Calculate bounds for all panes (using a default size)
        let bounds = self.layout.calculate_bounds(1000.0, 1000.0);

        let current_bounds = bounds.get(&current_id)?;
        let (curr_x, curr_y, curr_w, curr_h) = *current_bounds;

        // Center of current pane
        let curr_center_x = curr_x + curr_w / 2.0;
        let curr_center_y = curr_y + curr_h / 2.0;

        let mut best_pane: Option<PaneId> = None;
        let mut best_distance = f32::MAX;

        for (pane_id, bounds) in bounds.iter() {
            if *pane_id == current_id {
                continue;
            }

            let (x, y, w, h) = *bounds;
            let center_x = x + w / 2.0;
            let center_y = y + h / 2.0;

            // Check if pane is in the correct direction
            let is_valid = match direction {
                NavigationDirection::Up => center_y < curr_center_y,
                NavigationDirection::Down => center_y > curr_center_y,
                NavigationDirection::Left => center_x < curr_center_x,
                NavigationDirection::Right => center_x > curr_center_x,
            };

            if !is_valid {
                continue;
            }

            // Calculate distance
            let dx = center_x - curr_center_x;
            let dy = center_y - curr_center_y;
            let distance = dx * dx + dy * dy;

            if distance < best_distance {
                best_distance = distance;
                best_pane = Some(*pane_id);
            }
        }

        best_pane
    }
}

#[derive(Debug, Clone, Copy)]
enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Default for PaneManager {
    fn default() -> Self {
        Self::new()
    }
}
