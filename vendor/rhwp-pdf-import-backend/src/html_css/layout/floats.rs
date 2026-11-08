//! Floats (LAYOUT-4).
//!
//! v0.3.35 ships the data structures and a basic line-shortening
//! helper. Full float-aware line-box interaction (`shape-outside`,
//! complex avoidance) is deferred per the plan's R1 cut list — for
//! the v0.3.35 first cut, floats render in-flow so simple
//! invoice/report HTML doesn't crash, and v0.3.36 adds proper
//! avoidance.

use crate::html_css::css::ComputedStyles;

use super::box_tree::BoxId;

/// `float` keyword space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloatSide {
    /// `float: none` — default, in-flow.
    #[default]
    None,
    /// `float: left` — pull to the left edge.
    Left,
    /// `float: right` — pull to the right edge.
    Right,
}

/// `clear` keyword space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Clear {
    /// `clear: none` — default.
    #[default]
    None,
    /// Below preceding left floats.
    Left,
    /// Below preceding right floats.
    Right,
    /// Below all preceding floats.
    Both,
}

/// One floating box recorded by the layout engine.
#[derive(Debug, Clone, Copy)]
pub struct FloatBox {
    /// Source box id.
    pub box_id: BoxId,
    /// Which side it floats on.
    pub side: FloatSide,
    /// Top y in the parent's content coordinate system.
    pub top_y: f32,
    /// Bottom y (top + height).
    pub bottom_y: f32,
    /// Width in px (border-box).
    pub width: f32,
}

/// Per-element float state collected during layout.
#[derive(Debug, Clone, Default)]
pub struct FloatContext {
    /// Currently active floats, in source order.
    pub floats: Vec<FloatBox>,
}

impl FloatContext {
    /// Inset for a line at vertical position `line_y` from the left
    /// edge of the containing block.
    pub fn left_inset(&self, line_y: f32) -> f32 {
        self.floats
            .iter()
            .filter(|f| f.side == FloatSide::Left && f.top_y <= line_y && line_y < f.bottom_y)
            .map(|f| f.width)
            .sum()
    }

    /// Inset from the right edge.
    pub fn right_inset(&self, line_y: f32) -> f32 {
        self.floats
            .iter()
            .filter(|f| f.side == FloatSide::Right && f.top_y <= line_y && line_y < f.bottom_y)
            .map(|f| f.width)
            .sum()
    }

    /// Effective content width for a line at `line_y` given the
    /// containing block's full content width.
    pub fn line_width(&self, line_y: f32, content_width: f32) -> f32 {
        (content_width - self.left_inset(line_y) - self.right_inset(line_y)).max(0.0)
    }

    /// Y at which `clear: side` should jump to — the first y at or
    /// after `current_y` where no float on the cleared side is active.
    pub fn clear_y(&self, current_y: f32, clear: Clear) -> f32 {
        let mut y = current_y;
        for f in &self.floats {
            let interferes = match clear {
                Clear::Left => f.side == FloatSide::Left,
                Clear::Right => f.side == FloatSide::Right,
                Clear::Both => f.side != FloatSide::None,
                Clear::None => false,
            };
            if interferes && f.bottom_y > y {
                y = f.bottom_y;
            }
        }
        y
    }
}

/// Read `float` from a [`ComputedStyles`].
pub fn read_float(styles: &ComputedStyles<'_>) -> FloatSide {
    let Some(rv) = styles.get("float") else {
        return FloatSide::None;
    };
    let s = match crate::html_css::css::parse_property("float", &rv.value) {
        Ok(crate::html_css::css::Value::Keyword(s)) => s,
        _ => return FloatSide::None,
    };
    match s.as_str() {
        "left" => FloatSide::Left,
        "right" => FloatSide::Right,
        _ => FloatSide::None,
    }
}

/// Read `clear` from a [`ComputedStyles`].
pub fn read_clear(styles: &ComputedStyles<'_>) -> Clear {
    let Some(rv) = styles.get("clear") else {
        return Clear::None;
    };
    let s = match crate::html_css::css::parse_property("clear", &rv.value) {
        Ok(crate::html_css::css::Value::Keyword(s)) => s,
        _ => return Clear::None,
    };
    match s.as_str() {
        "left" => Clear::Left,
        "right" => Clear::Right,
        "both" => Clear::Both,
        _ => Clear::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with(left: Vec<(f32, f32, f32)>, right: Vec<(f32, f32, f32)>) -> FloatContext {
        let mut c = FloatContext::default();
        let mut id = 1u32;
        for (top, bot, w) in left {
            c.floats.push(FloatBox {
                box_id: id,
                side: FloatSide::Left,
                top_y: top,
                bottom_y: bot,
                width: w,
            });
            id += 1;
        }
        for (top, bot, w) in right {
            c.floats.push(FloatBox {
                box_id: id,
                side: FloatSide::Right,
                top_y: top,
                bottom_y: bot,
                width: w,
            });
            id += 1;
        }
        c
    }

    #[test]
    fn no_floats_no_inset() {
        let c = FloatContext::default();
        assert_eq!(c.left_inset(0.0), 0.0);
        assert_eq!(c.right_inset(0.0), 0.0);
        assert_eq!(c.line_width(0.0, 600.0), 600.0);
    }

    #[test]
    fn one_left_float_shrinks_line() {
        let c = ctx_with(vec![(0.0, 100.0, 80.0)], vec![]);
        assert_eq!(c.left_inset(50.0), 80.0);
        assert_eq!(c.line_width(50.0, 600.0), 520.0);
        // Below the float, line returns to full width.
        assert_eq!(c.line_width(101.0, 600.0), 600.0);
    }

    #[test]
    fn left_and_right_floats_both_shrink() {
        let c = ctx_with(vec![(0.0, 100.0, 80.0)], vec![(0.0, 50.0, 60.0)]);
        assert_eq!(c.line_width(25.0, 600.0), 460.0);
        // Below the right float but still under left.
        assert_eq!(c.line_width(60.0, 600.0), 520.0);
    }

    #[test]
    fn clear_left_jumps_to_bottom() {
        let c = ctx_with(vec![(0.0, 100.0, 80.0)], vec![]);
        assert_eq!(c.clear_y(50.0, Clear::Left), 100.0);
        // Already past it.
        assert_eq!(c.clear_y(101.0, Clear::Left), 101.0);
    }

    #[test]
    fn clear_both_jumps_past_max() {
        let c = ctx_with(vec![(0.0, 80.0, 50.0)], vec![(0.0, 120.0, 40.0)]);
        assert_eq!(c.clear_y(0.0, Clear::Both), 120.0);
    }
}
