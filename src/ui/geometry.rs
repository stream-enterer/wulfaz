// ---------------------------------------------------------------------------
// Geometry primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Returns true if the point (px, py) is inside this rectangle.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    /// Compute the intersection of two rectangles. Returns None if they don't overlap.
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        if x2 > x1 && y2 > y1 {
            Some(Rect {
                x: x1,
                y: y1,
                width: x2 - x1,
                height: y2 - y1,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Constraints {
    pub min_width: f32,
    pub min_height: f32,
    pub max_width: f32,
    pub max_height: f32,
}

impl Constraints {
    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            min_height: height,
            max_width: width,
            max_height: height,
        }
    }

    pub fn loose(max_width: f32, max_height: f32) -> Self {
        Self {
            min_width: 0.0,
            min_height: 0.0,
            max_width,
            max_height,
        }
    }

    pub fn clamp(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }
}

/// Padding / margin edges (top, right, bottom, left — CSS order).
#[derive(Debug, Clone, Copy, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn all(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

// ---------------------------------------------------------------------------
// Positioning mode
// ---------------------------------------------------------------------------

/// How a widget is positioned within its parent.
#[derive(Debug, Clone, Copy)]
pub enum Position {
    /// Fixed pixel offset from parent's content origin.
    Fixed { x: f32, y: f32 },
    /// Percentage of parent's content area (0.0–1.0).
    Percent { x: f32, y: f32 },
    /// Centered in parent's content area.
    /// Computed as `(parent_w - widget_w) / 2` after size resolution.
    Center,
}

impl Default for Position {
    fn default() -> Self {
        Position::Fixed { x: 0.0, y: 0.0 }
    }
}

/// How a widget's width/height is determined.
#[derive(Debug, Clone, Copy, Default)]
pub enum Sizing {
    /// Fixed pixel size.
    Fixed(f32),
    /// Percentage of parent's content dimension (0.0–1.0).
    Percent(f32),
    /// Fit to content (intrinsic size from measure).
    #[default]
    Fit,
}
