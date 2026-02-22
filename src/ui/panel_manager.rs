use std::collections::HashMap;

use super::{WidgetId, WidgetTree};

/// Tracks open UI panels by name (UI-306).
///
/// Multiple panels can be open simultaneously (character panel, outliner,
/// event log). Panels are drawn in `draw_order` sequence within their
/// Z-tier. Raising a panel moves it to the end of the draw order (topmost
/// within the Panel tier). Modals (ZTier::Modal) always draw above all panels.
pub struct PanelManager {
    /// Map from panel name to its root WidgetId.
    panels: HashMap<String, PanelEntry>,
    /// Draw order — last entry is topmost within the Panel tier.
    draw_order: Vec<String>,
}

struct PanelEntry {
    root: WidgetId,
    closeable: bool,
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            draw_order: Vec::new(),
        }
    }

    /// Register an open panel. `root` must already be in the tree.
    pub fn open(&mut self, name: impl Into<String>, root: WidgetId, closeable: bool) {
        let name = name.into();
        // Close existing panel with the same name.
        if self.panels.contains_key(&name) {
            self.draw_order.retain(|n| *n != name);
        }
        self.panels
            .insert(name.clone(), PanelEntry { root, closeable });
        self.draw_order.push(name);
    }

    /// Close a panel by name. Removes the widget subtree from the tree.
    /// Returns the root WidgetId if the panel existed.
    pub fn close(&mut self, name: &str, tree: &mut WidgetTree) -> Option<WidgetId> {
        let entry = self.panels.remove(name)?;
        self.draw_order.retain(|n| n != name);
        tree.remove(entry.root);
        Some(entry.root)
    }

    /// Bring a panel to the front (last in draw order within its Z-tier).
    /// Does NOT change the Z-tier — panels stay at ZTier::Panel.
    pub fn raise(&mut self, name: &str) {
        if self.panels.contains_key(name) {
            self.draw_order.retain(|n| n != name);
            self.draw_order.push(name.to_string());
        }
    }

    /// Close the topmost closeable panel.
    /// Returns the name and root WidgetId if a panel was closed.
    pub fn close_topmost(&mut self, tree: &mut WidgetTree) -> Option<(String, WidgetId)> {
        // Walk draw order in reverse to find the topmost closeable panel.
        let name = self
            .draw_order
            .iter()
            .rev()
            .find(|n| self.panels.get(n.as_str()).is_some_and(|e| e.closeable))
            .cloned()?;
        let root = self.close(&name, tree)?;
        Some((name, root))
    }

    /// Whether a panel with the given name is currently open.
    pub fn is_open(&self, name: &str) -> bool {
        self.panels.contains_key(name)
    }

    /// Get the root WidgetId of a panel by name.
    pub fn root_id(&self, name: &str) -> Option<WidgetId> {
        self.panels.get(name).map(|e| e.root)
    }

    /// Current draw order (front-to-back names).
    pub fn draw_order(&self) -> &[String] {
        &self.draw_order
    }

    /// Number of open panels.
    pub fn len(&self) -> usize {
        self.panels.len()
    }

    /// Whether no panels are open.
    pub fn is_empty(&self) -> bool {
        self.panels.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::widget::Widget;
    use crate::ui::{Position, Sizing, WidgetTree};

    fn make_panel(tree: &mut WidgetTree, name: &str) -> WidgetId {
        let _ = name; // used only for identification in tests
        let panel = tree.insert_root(Widget::Panel {
            bg_color: [0.5; 4],
            border_color: [1.0; 4],
            border_width: 2.0,
            shadow_width: 0.0,
        });
        tree.set_position(panel, Position::Fixed { x: 50.0, y: 50.0 });
        tree.set_sizing(panel, Sizing::Fixed(200.0), Sizing::Fixed(150.0));
        panel
    }

    #[test]
    fn open_and_close() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();

        let root = make_panel(&mut tree, "character");
        pm.open("character", root, true);
        assert!(pm.is_open("character"));
        assert_eq!(pm.len(), 1);
        assert_eq!(pm.root_id("character"), Some(root));

        pm.close("character", &mut tree);
        assert!(!pm.is_open("character"));
        assert_eq!(pm.len(), 0);
        assert!(tree.get(root).is_none(), "subtree removed");
    }

    #[test]
    fn raise_moves_to_top_of_draw_order() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();

        let a = make_panel(&mut tree, "a");
        pm.open("a", a, true);
        let b = make_panel(&mut tree, "b");
        pm.open("b", b, true);
        let c = make_panel(&mut tree, "c");
        pm.open("c", c, true);

        assert_eq!(pm.draw_order(), &["a", "b", "c"]);

        pm.raise("a");
        assert_eq!(pm.draw_order(), &["b", "c", "a"]);
    }

    #[test]
    fn close_topmost_closes_last_closeable() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();

        let a = make_panel(&mut tree, "a");
        pm.open("a", a, true);
        let b = make_panel(&mut tree, "b");
        pm.open("b", b, false); // not closeable
        let c = make_panel(&mut tree, "c");
        pm.open("c", c, true);

        // Topmost closeable is "c".
        let closed = pm.close_topmost(&mut tree);
        assert_eq!(closed.as_ref().map(|(n, _)| n.as_str()), Some("c"));

        // Next topmost closeable is "a" (b is not closeable).
        let closed = pm.close_topmost(&mut tree);
        assert_eq!(closed.as_ref().map(|(n, _)| n.as_str()), Some("a"));

        // Only "b" remains (not closeable).
        assert!(pm.close_topmost(&mut tree).is_none());
        assert_eq!(pm.len(), 1);
    }

    #[test]
    fn reopen_same_name_replaces() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();

        let root1 = make_panel(&mut tree, "x");
        pm.open("x", root1, true);
        let root2 = make_panel(&mut tree, "x");
        pm.open("x", root2, true);

        assert_eq!(pm.len(), 1);
        assert_eq!(pm.root_id("x"), Some(root2));
        // Draw order should have only one entry for "x".
        assert_eq!(pm.draw_order().len(), 1);
    }
}
