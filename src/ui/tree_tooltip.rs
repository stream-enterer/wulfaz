use super::WidgetId;
use super::draw::TextMeasurer;
use super::geometry::{Constraints, Edges, Position, Size};
use super::node::ZTier;
use super::theme::Theme;
use super::tree::WidgetTree;
use super::widget::{self, Widget};

impl WidgetTree {
    /// Compute tooltip position with edge-flipping.
    ///
    /// Pure geometry — no `&self` needed.
    pub(crate) fn compute_tooltip_position(
        cursor: (f32, f32),
        tooltip_size: Size,
        screen: Size,
        nesting_level: usize,
        theme: &Theme,
    ) -> (f32, f32) {
        let nest = nesting_level as f32;
        let off_x = theme.tooltip_offset_x + nest * theme.tooltip_nesting_offset;
        let off_y = theme.tooltip_offset_y + nest * theme.tooltip_nesting_offset;

        let mut x = cursor.0 + off_x;
        let mut y = cursor.1 + off_y;

        // Flip horizontally if clipping right edge.
        if x + tooltip_size.width > screen.width {
            x = cursor.0 - tooltip_size.width - off_x;
        }
        // Flip vertically if clipping bottom edge.
        if y + tooltip_size.height > screen.height {
            y = cursor.1 - tooltip_size.height - off_y;
        }

        // Clamp to screen bounds.
        x = x.clamp(0.0, (screen.width - tooltip_size.width).max(0.0));
        y = y.clamp(0.0, (screen.height - tooltip_size.height).max(0.0));

        // Snap to whole pixels so glyph positions stay stable as the tooltip
        // follows the cursor (avoids subpixel shimmer in the text).
        (x.round(), y.round())
    }

    /// Create tooltip chrome: Panel(ZTier::Tooltip) + Column.
    /// Returns (panel_id, column_id). Caller inserts content into column_id.
    pub fn insert_tooltip_chrome(&mut self, theme: &Theme) -> (WidgetId, WidgetId) {
        let panel = self.insert_root_with_tier(
            Widget::Panel {
                bg_color: theme.tooltip_bg_color,
                border_color: theme.tooltip_border_color,
                border_width: theme.tooltip_border_width,
                shadow_width: theme.tooltip_shadow_width,
            },
            ZTier::Tooltip,
        );
        self.set_padding(panel, Edges::all(theme.tooltip_padding));
        self.set_constraints(panel, Constraints::loose(theme.tooltip_max_width, f32::MAX));

        let col = self.insert(
            panel,
            Widget::Column {
                gap: theme.label_gap,
                align: widget::CrossAlign::Start,
            },
        );

        (panel, col)
    }

    /// Measure tooltip panel and position it near the cursor with edge-flipping.
    pub fn position_tooltip(
        &mut self,
        panel: WidgetId,
        cursor: (f32, f32),
        screen: Size,
        nesting_level: usize,
        theme: &Theme,
        tm: &mut dyn TextMeasurer,
    ) {
        let measured = self.measure_node(panel, tm);
        let tooltip_w = measured.width + theme.tooltip_padding * 2.0;
        let tooltip_h = measured.height + theme.tooltip_padding * 2.0;
        let (tx, ty) = Self::compute_tooltip_position(
            cursor,
            Size {
                width: tooltip_w,
                height: tooltip_h,
            },
            screen,
            nesting_level,
            theme,
        );
        self.set_position(panel, Position::Fixed { x: tx, y: ty });
    }
}
