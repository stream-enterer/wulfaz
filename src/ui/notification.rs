use std::time::{Duration, Instant};

use super::draw::FontFamily;
use super::theme::Theme;
use super::widget::Widget;
use super::{Edges, Position, Sizing, WidgetId, WidgetTree};

/// Notification priority level (UI-302).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotificationPriority {
    Info,
    Important,
    Critical,
}

/// A single notification entry.
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub priority: NotificationPriority,
    pub created: Instant,
    pub duration: Duration,
}

/// Top-right notification stack (UI-302).
///
/// Manages a queue of notifications displayed as a Column of panels
/// anchored to the top-right of the screen. Notifications auto-dismiss
/// after their duration expires. Max 5 visible at a time; excess are
/// queued until space opens.
pub struct NotificationManager {
    notifications: Vec<Notification>,
    /// Root widget id of the notification column, if built.
    root: Option<WidgetId>,
    /// Maximum visible notifications.
    max_visible: usize,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            root: None,
            max_visible: 5,
        }
    }

    /// Push a new notification.
    pub fn push(
        &mut self,
        message: impl Into<String>,
        priority: NotificationPriority,
        now: Instant,
    ) {
        self.notifications.push(Notification {
            message: message.into(),
            priority,
            created: now,
            duration: Duration::from_secs(8),
        });
    }

    /// Push a notification with a custom duration.
    pub fn push_with_duration(
        &mut self,
        message: impl Into<String>,
        priority: NotificationPriority,
        now: Instant,
        duration: Duration,
    ) {
        self.notifications.push(Notification {
            message: message.into(),
            priority,
            created: now,
            duration,
        });
    }

    /// Remove expired notifications.
    pub fn tick(&mut self, now: Instant) {
        self.notifications
            .retain(|n| now.duration_since(n.created) < n.duration);
    }

    /// Dismiss a notification by index.
    pub fn dismiss(&mut self, index: usize) {
        if index < self.notifications.len() {
            self.notifications.remove(index);
        }
    }

    /// Number of active (non-expired) notifications.
    pub fn count(&self) -> usize {
        self.notifications.len()
    }

    /// Build the notification UI into the widget tree.
    /// Removes the previous build first. Sorts by priority (Critical first).
    /// Returns the root WidgetId.
    pub fn build(
        &mut self,
        tree: &mut WidgetTree,
        theme: &Theme,
        screen_w: f32,
    ) -> Option<WidgetId> {
        // Remove previous build.
        if let Some(old) = self.root.take() {
            tree.remove(old);
        }

        if self.notifications.is_empty() {
            return None;
        }

        // Sort: Critical first, then Important, then Info.
        self.notifications
            .sort_by(|a, b| b.priority.cmp(&a.priority));

        let visible_count = self.notifications.len().min(self.max_visible);
        let notif_w = 250.0;
        let notif_h = 40.0;
        let gap = 4.0;
        let margin_top = 8.0;
        let margin_right = 8.0;

        // Column root anchored to top-right.
        let col = tree.insert_root(Widget::Column {
            gap,
            align: super::widget::CrossAlign::End,
        });
        tree.set_position(
            col,
            Position::Fixed {
                x: screen_w - notif_w - margin_right,
                y: margin_top,
            },
        );
        tree.set_sizing(col, Sizing::Fixed(notif_w), Sizing::Fit);

        for notif in self.notifications.iter().take(visible_count) {
            let border_color = match notif.priority {
                NotificationPriority::Critical => theme.danger,
                NotificationPriority::Important => theme.gold,
                NotificationPriority::Info => theme.panel_border_color,
            };

            let panel = tree.insert(
                col,
                Widget::Panel {
                    bg_color: theme.bg_parchment,
                    border_color,
                    border_width: 1.0,
                    shadow_width: 0.0,
                },
            );
            tree.set_sizing(panel, Sizing::Percent(1.0), Sizing::Fixed(notif_h));
            tree.set_padding(panel, Edges::all(6.0));

            tree.insert(
                panel,
                Widget::Label {
                    text: notif.message.clone(),
                    color: theme.text_dark,
                    font_size: theme.font_body_size,
                    font_family: FontFamily::default(),
                    wrap: false,
                },
            );
        }

        self.root = Some(col);
        Some(col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{Size, WidgetTree};

    #[test]
    fn push_and_count() {
        let mut nm = NotificationManager::new();
        let now = Instant::now();
        nm.push("Hello", NotificationPriority::Info, now);
        nm.push("Warning", NotificationPriority::Important, now);
        assert_eq!(nm.count(), 2);
    }

    #[test]
    fn tick_expires_old_notifications() {
        let mut nm = NotificationManager::new();
        let now = Instant::now();
        nm.push_with_duration(
            "Short",
            NotificationPriority::Info,
            now,
            Duration::from_millis(100),
        );
        nm.push("Long", NotificationPriority::Info, now);

        // Advance past the short notification's duration.
        let later = now + Duration::from_millis(150);
        nm.tick(later);

        assert_eq!(nm.count(), 1, "short notification expired");
    }

    #[test]
    fn build_creates_column_with_panels() {
        let mut tree = WidgetTree::new();
        let mut nm = NotificationManager::new();
        let now = Instant::now();
        let theme = Theme::default();

        nm.push("Alert 1", NotificationPriority::Info, now);
        nm.push("Alert 2", NotificationPriority::Critical, now);
        nm.push("Alert 3", NotificationPriority::Important, now);

        let root = nm.build(&mut tree, &theme, 800.0);
        assert!(root.is_some());

        tree.layout(
            Size {
                width: 800.0,
                height: 600.0,
            },
            14.0,
        );

        let root = root.unwrap();
        let node = tree.get(root).unwrap();
        // 3 notification panels as children.
        assert_eq!(node.children.len(), 3);

        // Critical should be first (sorted by priority descending).
        let first_panel = tree.get(node.children[0]).unwrap();
        let first_label = tree.get(first_panel.children[0]).unwrap();
        if let Widget::Label { text, .. } = &first_label.widget {
            assert_eq!(text, "Alert 2", "Critical notification sorted first");
        } else {
            panic!("expected Label");
        }
    }

    #[test]
    fn max_visible_limits_displayed() {
        let mut tree = WidgetTree::new();
        let mut nm = NotificationManager::new();
        let now = Instant::now();
        let theme = Theme::default();

        for i in 0..8 {
            nm.push(format!("N{}", i), NotificationPriority::Info, now);
        }

        let root = nm.build(&mut tree, &theme, 800.0).unwrap();
        let node = tree.get(root).unwrap();
        assert_eq!(node.children.len(), 5, "max 5 visible");
    }

    #[test]
    fn build_empty_returns_none() {
        let mut tree = WidgetTree::new();
        let mut nm = NotificationManager::new();
        let theme = Theme::default();
        assert!(nm.build(&mut tree, &theme, 800.0).is_none());
    }

    #[test]
    fn dismiss_removes_by_index() {
        let mut nm = NotificationManager::new();
        let now = Instant::now();
        nm.push("A", NotificationPriority::Info, now);
        nm.push("B", NotificationPriority::Info, now);
        nm.push("C", NotificationPriority::Info, now);

        nm.dismiss(1); // remove "B"
        assert_eq!(nm.count(), 2);
    }
}
