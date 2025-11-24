use super::PaneId;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub enum LayoutNode {
    Pane(PaneId),
    Split {
        direction: SplitDirection,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

pub struct Layout {
    root: Option<LayoutNode>,
}

impl Layout {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn set_root(&mut self, pane_id: PaneId) {
        self.root = Some(LayoutNode::Pane(pane_id));
    }

    pub fn split(&mut self, target: PaneId, new_pane: PaneId, direction: SplitDirection) {
        if let Some(root) = &mut self.root {
            Self::split_node(root, target, new_pane, direction);
        }
    }

    fn split_node(
        node: &mut LayoutNode,
        target: PaneId,
        new_pane: PaneId,
        direction: SplitDirection,
    ) -> bool {
        match node {
            LayoutNode::Pane(id) if *id == target => {
                let old_node = std::mem::replace(node, LayoutNode::Pane(PaneId(0)));
                *node = LayoutNode::Split {
                    direction,
                    ratio: 0.5,
                    first: Box::new(old_node),
                    second: Box::new(LayoutNode::Pane(new_pane)),
                };
                true
            }
            LayoutNode::Split { first, second, .. } => {
                Self::split_node(first, target, new_pane, direction)
                    || Self::split_node(second, target, new_pane, direction)
            }
            _ => false,
        }
    }

    pub fn remove(&mut self, pane_id: PaneId) {
        if let Some(root) = &mut self.root {
            if Self::remove_node(root, pane_id) {
                self.root = None;
            }
        }
    }

    fn remove_node(node: &mut LayoutNode, target: PaneId) -> bool {
        match node {
            LayoutNode::Pane(id) => *id == target,
            LayoutNode::Split { first, second, .. } => {
                if Self::remove_node(first, target) {
                    *node = (**second).clone();
                    false
                } else if Self::remove_node(second, target) {
                    *node = (**first).clone();
                    false
                } else {
                    false
                }
            }
        }
    }

    pub fn root(&self) -> Option<&LayoutNode> {
        self.root.as_ref()
    }

    pub fn calculate_bounds(&self, width: f32, height: f32) -> HashMap<PaneId, (f32, f32, f32, f32)> {
        let mut bounds = HashMap::new();
        if let Some(root) = &self.root {
            Self::calculate_node_bounds(root, 0.0, 0.0, width, height, &mut bounds);
        }
        bounds
    }

    fn calculate_node_bounds(
        node: &LayoutNode,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        bounds: &mut HashMap<PaneId, (f32, f32, f32, f32)>,
    ) {
        match node {
            LayoutNode::Pane(id) => {
                bounds.insert(*id, (x, y, width, height));
            }
            LayoutNode::Split {
                direction,
                ratio,
                first,
                second,
            } => match direction {
                SplitDirection::Horizontal => {
                    let split_x = x + width * ratio;
                    Self::calculate_node_bounds(first, x, y, width * ratio, height, bounds);
                    Self::calculate_node_bounds(
                        second,
                        split_x,
                        y,
                        width * (1.0 - ratio),
                        height,
                        bounds,
                    );
                }
                SplitDirection::Vertical => {
                    let split_y = y + height * ratio;
                    Self::calculate_node_bounds(first, x, y, width, height * ratio, bounds);
                    Self::calculate_node_bounds(
                        second,
                        x,
                        split_y,
                        width,
                        height * (1.0 - ratio),
                        bounds,
                    );
                }
            },
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
