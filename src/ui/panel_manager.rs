use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    /// Panels waiting for their hide animation to finish before removal.
    closing: Vec<ClosingPanel>,
    /// Persisted scroll offsets for ScrollLists inside panels.
    /// Key format: `"panel_name"` (one scroll offset per panel).
    scroll_offsets: HashMap<String, f32>,
}

struct PanelEntry {
    root: WidgetId,
    closeable: bool,
}

struct ClosingPanel {
    name: String,
    root: WidgetId,
    deadline: Instant,
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            draw_order: Vec::new(),
            closing: Vec::new(),
            scroll_offsets: HashMap::new(),
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

    /// Close the topmost closeable panel (instant removal).
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

    /// Close the topmost closeable panel with a hide animation.
    /// The widget stays in the tree for `duration`, then is removed
    /// by `flush_closed()`. The caller should start the hide animation
    /// on the Animator separately.
    pub fn close_topmost_animated(
        &mut self,
        duration: Duration,
        now: Instant,
    ) -> Option<(String, WidgetId)> {
        let name = self
            .draw_order
            .iter()
            .rev()
            .find(|n| self.panels.get(n.as_str()).is_some_and(|e| e.closeable))
            .cloned()?;
        let root = self.close_animated(&name, duration, now)?;
        Some((name, root))
    }

    /// Begin an animated close. Moves the panel to the closing list
    /// instead of removing it immediately. The widget subtree stays in
    /// the tree until `flush_closed()` removes it after `duration`.
    /// The caller is responsible for starting the hide animation on the
    /// Animator before calling this.
    /// Returns the root WidgetId if the panel existed.
    pub fn close_animated(
        &mut self,
        name: &str,
        duration: Duration,
        now: Instant,
    ) -> Option<WidgetId> {
        let entry = self.panels.remove(name)?;
        self.draw_order.retain(|n| n != name);
        self.closing.push(ClosingPanel {
            name: name.to_string(),
            root: entry.root,
            deadline: now + duration,
        });
        Some(entry.root)
    }

    /// Remove widgets for panels whose hide animation has finished.
    /// Call once per frame.
    pub fn flush_closed(&mut self, tree: &mut WidgetTree, now: Instant) {
        self.closing.retain(|c| {
            if now >= c.deadline {
                tree.remove(c.root);
                false
            } else {
                true
            }
        });
    }

    /// Whether a panel with the given name is currently open
    /// (not counting panels in the closing state).
    pub fn is_open(&self, name: &str) -> bool {
        self.panels.contains_key(name)
    }

    /// Whether a panel is currently playing its close animation.
    pub fn is_closing(&self, name: &str) -> bool {
        self.closing.iter().any(|c| c.name == name)
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

    /// Store the scroll offset for a panel's ScrollList.
    /// Survives panel close/reopen cycles.
    pub fn save_scroll_offset(&mut self, name: &str, offset: f32) {
        self.scroll_offsets.insert(name.to_string(), offset);
    }

    /// Retrieve the persisted scroll offset for a panel (0.0 if never saved).
    pub fn scroll_offset(&self, name: &str) -> f32 {
        self.scroll_offsets.get(name).copied().unwrap_or(0.0)
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

    #[test]
    fn close_animated_defers_removal() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();
        let t0 = Instant::now();

        let root = make_panel(&mut tree, "panel");
        pm.open("panel", root, true);
        pm.close_animated("panel", Duration::from_millis(200), t0);

        // Panel is no longer "open" but widget still exists in tree.
        assert!(!pm.is_open("panel"));
        assert!(pm.is_closing("panel"));
        assert!(
            tree.get(root).is_some(),
            "widget still in tree during animation"
        );
    }

    #[test]
    fn flush_closed_removes_after_deadline() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();
        let t0 = Instant::now();

        let root = make_panel(&mut tree, "panel");
        pm.open("panel", root, true);
        pm.close_animated("panel", Duration::from_millis(200), t0);

        // Before deadline: widget survives.
        pm.flush_closed(&mut tree, t0 + Duration::from_millis(100));
        assert!(tree.get(root).is_some());
        assert!(pm.is_closing("panel"));

        // After deadline: widget removed.
        pm.flush_closed(&mut tree, t0 + Duration::from_millis(200));
        assert!(tree.get(root).is_none());
        assert!(!pm.is_closing("panel"));
    }

    #[test]
    fn scroll_offset_persists_across_close_reopen() {
        let mut pm = PanelManager::new();
        pm.save_scroll_offset("finder", 42.0);
        assert!((pm.scroll_offset("finder") - 42.0).abs() < 0.01);

        // Unknown panel returns 0.
        assert!(pm.scroll_offset("unknown").abs() < 0.01);
    }
}
