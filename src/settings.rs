use std::path::Path;

use crate::loading;

/// Persistent user settings, saved to `settings.kdl` in the working directory.
pub struct Settings {
    /// Window width in logical pixels.
    pub window_width: f64,
    /// Window height in logical pixels.
    pub window_height: f64,
    /// Whether the window was maximized at quit.
    pub window_maximized: bool,
}

const PATH: &str = "settings.kdl";

const DEFAULT_WIDTH: f64 = 800.0;
const DEFAULT_HEIGHT: f64 = 600.0;

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_width: DEFAULT_WIDTH,
            window_height: DEFAULT_HEIGHT,
            window_maximized: false,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = Path::new(PATH);
        if !path.exists() {
            return Self::default();
        }

        let doc = match loading::parse_kdl_file(PATH) {
            Some(d) => d,
            None => return Self::default(),
        };

        let mut settings = Self::default();

        if let Some(window) = doc.get("window")
            && let Some(children) = window.children()
        {
            if let Some(v) = kdl_f64(children, "width") {
                settings.window_width = v;
            }
            if let Some(v) = kdl_f64(children, "height") {
                settings.window_height = v;
            }
            if let Some(v) = kdl_bool(children, "maximized") {
                settings.window_maximized = v;
            }
        }

        settings
    }

    pub fn save(&self) {
        let maximized = if self.window_maximized {
            "#true"
        } else {
            "#false"
        };
        let content = format!(
            "window {{\n    width {:.1}\n    height {:.1}\n    maximized {}\n}}\n",
            self.window_width, self.window_height, maximized,
        );

        loading::write_kdl_file(PATH, &content);
    }
}

fn kdl_f64(doc: &kdl::KdlDocument, key: &str) -> Option<f64> {
    let val = doc.get_arg(key)?;
    val.as_float()
        .or_else(|| val.as_integer().map(|i| i as f64))
}

fn kdl_bool(doc: &kdl::KdlDocument, key: &str) -> Option<bool> {
    doc.get_arg(key)?.as_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_parse() {
        let s = Settings {
            window_width: 1200.0,
            window_height: 800.0,
            window_maximized: false,
        };
        let maximized = if s.window_maximized {
            "#true"
        } else {
            "#false"
        };
        let content = format!(
            "window {{\n    width {:.1}\n    height {:.1}\n    maximized {}\n}}\n",
            s.window_width, s.window_height, maximized,
        );
        let doc: kdl::KdlDocument = content.parse().expect("should parse");
        let window = doc.get("window").expect("window node");
        let children = window.children().expect("children");
        assert_eq!(kdl_f64(children, "width"), Some(1200.0));
        assert_eq!(kdl_f64(children, "height"), Some(800.0));
        assert_eq!(kdl_bool(children, "maximized"), Some(false));
    }

    #[test]
    fn round_trip_maximized() {
        let s = Settings {
            window_width: 900.0,
            window_height: 700.0,
            window_maximized: true,
        };
        let maximized = if s.window_maximized {
            "#true"
        } else {
            "#false"
        };
        let content = format!(
            "window {{\n    width {:.1}\n    height {:.1}\n    maximized {}\n}}\n",
            s.window_width, s.window_height, maximized,
        );
        let doc: kdl::KdlDocument = content.parse().expect("should parse");
        let window = doc.get("window").expect("window node");
        let children = window.children().expect("children");
        assert_eq!(kdl_bool(children, "maximized"), Some(true));
    }
}
