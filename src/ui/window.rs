//! Shared window frame builder (UI-600).
//!
//! Produces a standard panel layout: Panel -> Column -> [Header Row, Separator, Content Column].
//! All closeable windows get an "X" button. Screens add their content to `frame.content`.

use super::theme::Theme;
use super::widget::CrossAlign;
use super::{Edges, FontFamily, Sizing, Widget, WidgetId, WidgetTree};

/// Handles returned by `build_window_frame` for the caller to attach content.
pub struct WindowFrame {
    /// Root Panel widget.
    pub root: WidgetId,
    /// Header Row (title + optional close button).
    pub header: WidgetId,
    /// Title Label.
    pub title: WidgetId,
    /// Content Column -- add screen-specific widgets here.
    pub content: WidgetId,
    /// Close button ("X"), present only when `closeable` is true.
    pub close_btn: Option<WidgetId>,
    /// Usable content width (panel width minus padding on both sides).
    pub content_width: f32,
}

/// Build a standard window frame.
///
/// `width` -- panel width in pixels.
/// `height` -- panel height sizing (Fixed or Fit).
/// `closeable` -- whether to include an "X" close button in the header.
pub fn build_window_frame(
    tree: &mut WidgetTree,
    theme: &Theme,
    title_text: &str,
    width: f32,
    height: Sizing,
    closeable: bool,
) -> WindowFrame {
    let content_width = width - theme.panel_padding * 2.0;

    // Root panel
    let root = tree.insert_root(Widget::Panel {
        bg_color: theme.bg_parchment,
        border_color: theme.panel_border_color,
        border_width: theme.panel_border_width,
        shadow_width: theme.panel_shadow_width,
    });
    tree.set_sizing(root, Sizing::Fixed(width), height);
    tree.set_padding(root, Edges::all(theme.panel_padding));

    // Frame column -- holds header, separator, content
    let frame_col = tree.insert(
        root,
        Widget::Column {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(frame_col, Sizing::Percent(1.0), Sizing::Fit);

    // Header row
    let header = tree.insert(
        frame_col,
        Widget::Row {
            gap: theme.label_gap,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(header, Sizing::Percent(1.0), Sizing::Fit);

    // Title label
    let title = tree.insert(
        header,
        Widget::Label {
            text: title_text.to_string(),
            color: theme.gold,
            font_size: theme.font_header_size,
            font_family: FontFamily::Serif,
            wrap: false,
        },
    );

    // Spacer pushes close button to right edge (UI-601).
    if closeable {
        tree.insert(header, Widget::Expand);
    }

    // Close button (optional)
    let close_btn = if closeable {
        Some(tree.insert(
            header,
            Widget::Button {
                text: "X".to_string(),
                color: theme.danger,
                bg_color: [0.0, 0.0, 0.0, 0.0],
                border_color: theme.danger,
                font_size: theme.font_data_size,
                font_family: FontFamily::Mono,
            },
        ))
    } else {
        None
    };

    // Separator
    let sep = tree.insert(
        frame_col,
        Widget::Separator {
            color: theme.gold,
            thickness: theme.separator_thickness,
            horizontal: true,
        },
    );
    tree.set_sizing(sep, Sizing::Percent(1.0), Sizing::Fit);

    // Content column
    let content = tree.insert(
        frame_col,
        Widget::Column {
            gap: theme.label_gap,
            align: CrossAlign::Start,
        },
    );
    tree.set_sizing(content, Sizing::Percent(1.0), Sizing::Fit);

    WindowFrame {
        root,
        header,
        title,
        content,
        close_btn,
        content_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_tree_structure() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let frame = build_window_frame(&mut tree, &theme, "Test", 300.0, Sizing::Fit, true);

        // Root exists and is a Panel
        let root_node = tree.get(frame.root).unwrap();
        assert!(matches!(root_node.widget, Widget::Panel { .. }));

        // Root has one child: frame_col
        assert_eq!(root_node.children.len(), 1);
        let frame_col = root_node.children[0];
        let col_node = tree.get(frame_col).unwrap();

        // frame_col has 3 children: header, separator, content
        assert_eq!(col_node.children.len(), 3);

        // Header is a Row
        let header_node = tree.get(col_node.children[0]).unwrap();
        assert!(matches!(header_node.widget, Widget::Row { .. }));

        // Separator
        let sep_node = tree.get(col_node.children[1]).unwrap();
        assert!(matches!(sep_node.widget, Widget::Separator { .. }));

        // Content is a Column
        let content_node = tree.get(col_node.children[2]).unwrap();
        assert!(matches!(content_node.widget, Widget::Column { .. }));
    }

    #[test]
    fn close_button_present_when_closeable() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let frame = build_window_frame(&mut tree, &theme, "Test", 300.0, Sizing::Fit, true);

        assert!(frame.close_btn.is_some());
        let btn_node = tree.get(frame.close_btn.unwrap()).unwrap();
        if let Widget::Button { text, .. } = &btn_node.widget {
            assert_eq!(text, "X");
        } else {
            panic!("Expected Button widget");
        }
    }

    #[test]
    fn no_close_button_when_not_closeable() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let frame = build_window_frame(&mut tree, &theme, "Test", 300.0, Sizing::Fit, false);

        assert!(frame.close_btn.is_none());
        // Header should have only the title label
        let header_node = tree.get(frame.header).unwrap();
        assert_eq!(header_node.children.len(), 1);
    }

    #[test]
    fn content_width_calculation() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let frame = build_window_frame(&mut tree, &theme, "Test", 300.0, Sizing::Fit, false);
        let expected = 300.0 - theme.panel_padding * 2.0;
        assert!((frame.content_width - expected).abs() < 0.01);
    }
}
