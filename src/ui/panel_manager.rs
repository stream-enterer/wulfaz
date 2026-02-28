use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::action::PanelKind;
use super::{WidgetId, WidgetTree};

/// Tracks open UI panels by kind (UI-306).
///
/// Multiple panels can be open simultaneously (character panel, outliner,
/// event log). Panels are drawn in `draw_order` sequence within their
/// Z-tier. Raising a panel moves it to the end of the draw order (topmost
/// within the Panel tier). Modals (ZTier::Modal) always draw above all panels.
pub struct PanelManager {
    /// Map from panel kind to its root WidgetId.
    panels: HashMap<PanelKind, PanelEntry>,
    /// Draw order — last entry is topmost within the Panel tier.
    draw_order: Vec<PanelKind>,
    /// Panels waiting for their hide animation to finish before removal.
    closing: Vec<ClosingPanel>,
}

struct PanelEntry {
    root: WidgetId,
    closeable: bool,
}

struct ClosingPanel {
    kind: PanelKind,
    root: WidgetId,
    deadline: Instant,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelManager {
    pub fn new() -> Self {
        Self {
            panels: HashMap::new(),
            draw_order: Vec::new(),
            closing: Vec::new(),
        }
    }

    /// Register an open panel. `root` must already be in the tree.
    pub fn open(&mut self, kind: PanelKind, root: WidgetId, closeable: bool) {
        // Close existing panel with the same kind.
        if self.panels.contains_key(&kind) {
            self.draw_order.retain(|k| *k != kind);
        }
        self.panels.insert(kind, PanelEntry { root, closeable });
        self.draw_order.push(kind);
    }

    /// Close a panel by kind. Removes the widget subtree from the tree.
    /// Returns the root WidgetId if the panel existed.
    pub fn close(&mut self, kind: PanelKind, tree: &mut WidgetTree) -> Option<WidgetId> {
        let entry = self.panels.remove(&kind)?;
        self.draw_order.retain(|k| *k != kind);
        tree.remove(entry.root);
        Some(entry.root)
    }

    /// Bring a panel to the front (last in draw order within its Z-tier).
    /// Does NOT change the Z-tier — panels stay at ZTier::Panel.
    pub fn raise(&mut self, kind: PanelKind) {
        if self.panels.contains_key(&kind) {
            self.draw_order.retain(|k| *k != kind);
            self.draw_order.push(kind);
        }
    }

    /// Close the topmost closeable panel (instant removal).
    /// Returns the kind and root WidgetId if a panel was closed.
    pub fn close_topmost(&mut self, tree: &mut WidgetTree) -> Option<(PanelKind, WidgetId)> {
        // Walk draw order in reverse to find the topmost closeable panel.
        let kind = self
            .draw_order
            .iter()
            .rev()
            .find(|k| self.panels.get(k).is_some_and(|e| e.closeable))
            .copied()?;
        let root = self.close(kind, tree)?;
        Some((kind, root))
    }

    /// Close the topmost closeable panel with a hide animation.
    /// The widget stays in the tree for `duration`, then is removed
    /// by `flush_closed()`. The caller should start the hide animation
    /// on the Animator separately.
    pub fn close_topmost_animated(
        &mut self,
        duration: Duration,
        now: Instant,
    ) -> Option<(PanelKind, WidgetId)> {
        let kind = self
            .draw_order
            .iter()
            .rev()
            .find(|k| self.panels.get(k).is_some_and(|e| e.closeable))
            .copied()?;
        let root = self.close_animated(kind, duration, now)?;
        Some((kind, root))
    }

    /// Begin an animated close. Moves the panel to the closing list
    /// instead of removing it immediately. The widget subtree stays in
    /// the tree until `flush_closed()` removes it after `duration`.
    /// The caller is responsible for starting the hide animation on the
    /// Animator before calling this.
    /// Returns the root WidgetId if the panel existed.
    pub fn close_animated(
        &mut self,
        kind: PanelKind,
        duration: Duration,
        now: Instant,
    ) -> Option<WidgetId> {
        let entry = self.panels.remove(&kind)?;
        self.draw_order.retain(|k| *k != kind);
        self.closing.push(ClosingPanel {
            kind,
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

    /// Whether a panel with the given kind is currently open
    /// (not counting panels in the closing state).
    pub fn is_open(&self, kind: PanelKind) -> bool {
        self.panels.contains_key(&kind)
    }

    /// Whether a panel is currently playing its close animation.
    pub fn is_closing(&self, kind: PanelKind) -> bool {
        self.closing.iter().any(|c| c.kind == kind)
    }

    /// Get the root WidgetId of a panel by kind.
    pub fn root_id(&self, kind: PanelKind) -> Option<WidgetId> {
        self.panels.get(&kind).map(|e| e.root)
    }

    /// Current draw order (front-to-back panel kinds).
    pub fn draw_order(&self) -> &[PanelKind] {
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

    fn make_panel(tree: &mut WidgetTree) -> WidgetId {
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

        let root = make_panel(&mut tree);
        pm.open(PanelKind::CharacterPanel, root, true);
        assert!(pm.is_open(PanelKind::CharacterPanel));
        assert_eq!(pm.len(), 1);
        assert_eq!(pm.root_id(PanelKind::CharacterPanel), Some(root));

        pm.close(PanelKind::CharacterPanel, &mut tree);
        assert!(!pm.is_open(PanelKind::CharacterPanel));
        assert_eq!(pm.len(), 0);
        assert!(tree.get(root).is_none(), "subtree removed");
    }

    #[test]
    fn raise_moves_to_top_of_draw_order() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();

        let a = make_panel(&mut tree);
        pm.open(PanelKind::Sidebar, a, true);
        let b = make_panel(&mut tree);
        pm.open(PanelKind::Outliner, b, true);
        let c = make_panel(&mut tree);
        pm.open(PanelKind::CharacterPanel, c, true);

        assert_eq!(
            pm.draw_order(),
            &[
                PanelKind::Sidebar,
                PanelKind::Outliner,
                PanelKind::CharacterPanel
            ]
        );

        pm.raise(PanelKind::Sidebar);
        assert_eq!(
            pm.draw_order(),
            &[
                PanelKind::Outliner,
                PanelKind::CharacterPanel,
                PanelKind::Sidebar
            ]
        );
    }

    #[test]
    fn close_topmost_closes_last_closeable() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();

        let a = make_panel(&mut tree);
        pm.open(PanelKind::Sidebar, a, true);
        let b = make_panel(&mut tree);
        pm.open(PanelKind::Outliner, b, false); // not closeable
        let c = make_panel(&mut tree);
        pm.open(PanelKind::CharacterPanel, c, true);

        // Topmost closeable is CharacterPanel.
        let closed = pm.close_topmost(&mut tree);
        assert_eq!(
            closed.as_ref().map(|(k, _)| *k),
            Some(PanelKind::CharacterPanel)
        );

        // Next topmost closeable is Sidebar (Outliner is not closeable).
        let closed = pm.close_topmost(&mut tree);
        assert_eq!(closed.as_ref().map(|(k, _)| *k), Some(PanelKind::Sidebar));

        // Only Outliner remains (not closeable).
        assert!(pm.close_topmost(&mut tree).is_none());
        assert_eq!(pm.len(), 1);
    }

    #[test]
    fn reopen_same_kind_replaces() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();

        let root1 = make_panel(&mut tree);
        pm.open(PanelKind::CharacterFinder, root1, true);
        let root2 = make_panel(&mut tree);
        pm.open(PanelKind::CharacterFinder, root2, true);

        assert_eq!(pm.len(), 1);
        assert_eq!(pm.root_id(PanelKind::CharacterFinder), Some(root2));
        // Draw order should have only one entry.
        assert_eq!(pm.draw_order().len(), 1);
    }

    #[test]
    fn close_animated_defers_removal() {
        let mut tree = WidgetTree::new();
        let mut pm = PanelManager::new();
        let t0 = Instant::now();

        let root = make_panel(&mut tree);
        pm.open(PanelKind::Settings, root, true);
        pm.close_animated(PanelKind::Settings, Duration::from_millis(200), t0);

        // Panel is no longer "open" but widget still exists in tree.
        assert!(!pm.is_open(PanelKind::Settings));
        assert!(pm.is_closing(PanelKind::Settings));
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

        let root = make_panel(&mut tree);
        pm.open(PanelKind::SaveLoad, root, true);
        pm.close_animated(PanelKind::SaveLoad, Duration::from_millis(200), t0);

        // Before deadline: widget survives.
        pm.flush_closed(&mut tree, t0 + Duration::from_millis(100));
        assert!(tree.get(root).is_some());
        assert!(pm.is_closing(PanelKind::SaveLoad));

        // After deadline: widget removed.
        pm.flush_closed(&mut tree, t0 + Duration::from_millis(200));
        assert!(tree.get(root).is_none());
        assert!(!pm.is_closing(PanelKind::SaveLoad));
    }
}
