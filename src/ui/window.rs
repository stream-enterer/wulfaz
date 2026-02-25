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

/// Handles returned by `build_confirmation_dialog`.
pub struct ConfirmationDialog {
    /// Root Panel widget (push this onto ModalStack).
    pub root: WidgetId,
    /// Accept button.
    pub accept_btn: WidgetId,
    /// Cancel button.
    pub cancel_btn: WidgetId,
}

/// Build a standard confirmation dialog (UI-301).
///
/// Layout: WindowFrame with message label + [Cancel] [Expand] [Accept] button row.
/// Buttons have pre-wired `on_click` callbacks: `"dialog::cancel"` and `"dialog::accept"`.
/// The caller pushes `dialog.root` onto the ModalStack with appropriate `ModalOptions`.
pub fn build_confirmation_dialog(
    tree: &mut WidgetTree,
    theme: &Theme,
    title: &str,
    message: &str,
    accept_text: &str,
    cancel_text: &str,
) -> ConfirmationDialog {
    let width = 400.0;
    let frame = build_window_frame(tree, theme, title, width, Sizing::Fit, false);

    // Message label (wrapped)
    tree.insert(
        frame.content,
        Widget::Label {
            text: message.to_string(),
            color: theme.text_medium,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
            wrap: true,
        },
    );

    // Button row: [Cancel] [Expand] [Accept]
    let button_row = tree.insert(
        frame.content,
        Widget::Row {
            gap: theme.label_gap * 2.0,
            align: CrossAlign::Center,
        },
    );
    tree.set_sizing(button_row, Sizing::Percent(1.0), Sizing::Fit);

    let btn_pad = Edges {
        top: theme.button_pad_v,
        right: theme.button_pad_h,
        bottom: theme.button_pad_v,
        left: theme.button_pad_h,
    };

    let cancel_btn = tree.insert(
        button_row,
        Widget::Button {
            text: cancel_text.to_string(),
            color: theme.text_medium,
            bg_color: theme.tab_inactive_color,
            border_color: theme.panel_border_color,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_on_click(cancel_btn, crate::ui::UiAction::DialogCancel);
    tree.set_padding(cancel_btn, btn_pad);

    tree.insert(button_row, Widget::Expand);

    let accept_btn = tree.insert(
        button_row,
        Widget::Button {
            text: accept_text.to_string(),
            color: theme.text_medium,
            bg_color: theme.tab_inactive_color,
            border_color: theme.gold,
            font_size: theme.font_body_size,
            font_family: FontFamily::Serif,
        },
    );
    tree.set_on_click(accept_btn, crate::ui::UiAction::DialogAccept);
    tree.set_padding(accept_btn, btn_pad);

    ConfirmationDialog {
        root: frame.root,
        accept_btn,
        cancel_btn,
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

    #[test]
    fn confirmation_dialog_structure() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let dialog =
            build_confirmation_dialog(&mut tree, &theme, "Confirm", "Are you sure?", "Yes", "No");

        // Root is a Panel
        let root_node = tree.get(dialog.root).unwrap();
        assert!(matches!(root_node.widget, Widget::Panel { .. }));

        // Accept button exists and has correct callback
        let accept_node = tree.get(dialog.accept_btn).unwrap();
        assert!(matches!(
            accept_node.on_click,
            Some(crate::ui::UiAction::DialogAccept)
        ));
        if let Widget::Button { text, .. } = &accept_node.widget {
            assert_eq!(text, "Yes");
        } else {
            panic!("accept_btn should be a Button");
        }

        // Cancel button exists and has correct callback
        let cancel_node = tree.get(dialog.cancel_btn).unwrap();
        assert!(matches!(
            cancel_node.on_click,
            Some(crate::ui::UiAction::DialogCancel)
        ));
        if let Widget::Button { text, .. } = &cancel_node.widget {
            assert_eq!(text, "No");
        } else {
            panic!("cancel_btn should be a Button");
        }
    }

    #[test]
    fn confirmation_dialog_has_message_and_buttons() {
        let theme = Theme::default();
        let mut tree = WidgetTree::new();
        let dialog = build_confirmation_dialog(
            &mut tree,
            &theme,
            "Delete",
            "This cannot be undone.",
            "Delete",
            "Cancel",
        );

        // Navigate: root -> frame_col -> content -> [message, button_row]
        let root_node = tree.get(dialog.root).unwrap();
        let frame_col = root_node.children[0];
        let col_node = tree.get(frame_col).unwrap();
        // col children: header, separator, content
        let content_id = col_node.children[2];
        let content_node = tree.get(content_id).unwrap();
        // content children: message label, button_row
        assert_eq!(content_node.children.len(), 2);

        // First child is the message label
        let msg = tree.get(content_node.children[0]).unwrap();
        if let Widget::Label { text, .. } = &msg.widget {
            assert_eq!(text, "This cannot be undone.");
        } else {
            panic!("first content child should be message Label");
        }

        // Second child is the button row
        let row = tree.get(content_node.children[1]).unwrap();
        assert!(matches!(row.widget, Widget::Row { .. }));
        // Row has: cancel, expand, accept
        assert_eq!(row.children.len(), 3);
    }
}
