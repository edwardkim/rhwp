//! ComputedStyles → `taffy::Style` conversion (LAYOUT-2).
//!
//! Phase LAYOUT delegates block, flex, and grid layout to taffy
//! (Dioxus, MIT). Inline formatting stays ours (LAYOUT-3) — taffy's
//! "leaf" nodes call back into a measurement closure for inline-level
//! content.
//!
//! This module owns the conversion from a [`ComputedStyles`] (CSS-5
//! output) plus a [`CalcContext`] (CSS-6 unit-resolution context) to
//! a single [`taffy::Style`] suitable for a node in a TaffyTree.
//! LAYOUT-2's runner module (`run_layout`, in this file) builds the
//! TaffyTree mirroring the [`BoxTree`] and runs `compute_layout`.

use taffy::prelude::*;
use taffy::style::Style as TaffyStyle;

use crate::html_css::css::{parse_property, CalcContext, ComputedStyles, Length, Unit, Value};

use super::box_tree::{BoxId, BoxKind, BoxTree, DisplayInside, DisplayOutside};

/// Convert one [`ComputedStyles`] to a [`taffy::Style`].
///
/// Coverage targets the v0.3.35 supported surface — block / flex / grid
/// containers with widths, paddings, margins, gaps, flex-* properties,
/// grid templates. Properties not yet typed by CSS-8 fall back to taffy
/// defaults, which is the right behaviour for "unsupported style ignored".
pub fn style_to_taffy(
    styles: &ComputedStyles<'_>,
    ctx: &CalcContext,
    outside: DisplayOutside,
    inside: DisplayInside,
) -> TaffyStyle {
    let mut s = TaffyStyle::DEFAULT;
    s.display = match inside {
        DisplayInside::Flex => Display::Flex,
        DisplayInside::Grid => Display::Grid,
        DisplayInside::None => Display::None,
        // Block + Flow / FlowRoot / InlineBlock / Contents all map to
        // Block in taffy's vocabulary; "inline" outer + block inner is
        // not a thing taffy models, so we approximate inline-block
        // boxes as Block (sized by content).
        _ => Display::Block,
    };

    s.box_sizing = match get_keyword(styles, "box-sizing").as_deref() {
        Some("border-box") => BoxSizing::BorderBox,
        _ => BoxSizing::ContentBox,
    };

    s.size = Size {
        width: dimension(styles, "width", ctx, true),
        height: dimension(styles, "height", ctx, false),
    };
    s.min_size = Size {
        width: size_bound(styles, "min-width", ctx, true),
        height: size_bound(styles, "min-height", ctx, false),
    };
    s.max_size = Size {
        width: size_bound(styles, "max-width", ctx, true),
        height: size_bound(styles, "max-height", ctx, false),
    };

    let padding_sides = box_shorthand_sides(styles, "padding", ctx);
    s.padding = Rect {
        left: length_percent(styles, "padding-left", ctx).or_else_default(padding_sides[3]),
        right: length_percent(styles, "padding-right", ctx).or_else_default(padding_sides[1]),
        top: length_percent(styles, "padding-top", ctx).or_else_default(padding_sides[0]),
        bottom: length_percent(styles, "padding-bottom", ctx).or_else_default(padding_sides[2]),
    };
    s.border = Rect {
        left: length_percent(styles, "border-left-width", ctx).or_else_default(None),
        right: length_percent(styles, "border-right-width", ctx).or_else_default(None),
        top: length_percent(styles, "border-top-width", ctx).or_else_default(None),
        bottom: length_percent(styles, "border-bottom-width", ctx).or_else_default(None),
    };
    let margin_sides = box_shorthand_sides_auto(styles, "margin", ctx);
    s.margin = Rect {
        left: length_percent_auto(styles, "margin-left", ctx).or_else_default(margin_sides[3]),
        right: length_percent_auto(styles, "margin-right", ctx).or_else_default(margin_sides[1]),
        top: length_percent_auto(styles, "margin-top", ctx).or_else_default(margin_sides[0]),
        bottom: length_percent_auto(styles, "margin-bottom", ctx).or_else_default(margin_sides[2]),
    };

    // Flex container properties.
    if matches!(inside, DisplayInside::Flex) {
        s.flex_direction = match get_keyword(styles, "flex-direction").as_deref() {
            Some("row-reverse") => FlexDirection::RowReverse,
            Some("column") => FlexDirection::Column,
            Some("column-reverse") => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        };
        s.flex_wrap = match get_keyword(styles, "flex-wrap").as_deref() {
            Some("wrap") => FlexWrap::Wrap,
            Some("wrap-reverse") => FlexWrap::WrapReverse,
            _ => FlexWrap::NoWrap,
        };
        s.justify_content = align_content_keyword(styles, "justify-content");
        s.align_items = align_items_keyword(styles, "align-items");
        s.align_content = align_content_keyword(styles, "align-content");
    }

    // Cross-cutting align/gap properties.
    s.gap = Size {
        width: length_percent(styles, "column-gap", ctx).or_else_default(None),
        height: length_percent(styles, "row-gap", ctx).or_else_default(None),
    };

    // Flex-item properties (applied even on non-flex parents — taffy
    // ignores them outside flex contexts).
    s.flex_grow = number(styles, "flex-grow").unwrap_or(0.0);
    s.flex_shrink = number(styles, "flex-shrink").unwrap_or(1.0);
    s.flex_basis = match length_value(styles, "flex-basis", ctx) {
        Some(v) => v.to_dimension(),
        None => Dimension::auto(),
    };

    let _ = outside; // outside informs the box tree, not taffy.
    s
}

// ─────────────────────────────────────────────────────────────────────
// Property → taffy primitive helpers
// ─────────────────────────────────────────────────────────────────────

fn dimension(
    styles: &ComputedStyles<'_>,
    prop: &str,
    ctx: &CalcContext,
    _width_axis: bool,
) -> Dimension {
    match length_value(styles, prop, ctx) {
        Some(v) => v.to_dimension(),
        None => Dimension::auto(),
    }
}

/// `min-*` / `max-*` sizes, which taffy types as `LengthPercentageAuto`
/// rather than the `Dimension` used for `width` / `height`.
fn size_bound(
    styles: &ComputedStyles<'_>,
    prop: &str,
    ctx: &CalcContext,
    _width_axis: bool,
) -> LengthPercentageAuto {
    match length_value(styles, prop, ctx) {
        Some(v) => v.to_size_bound(),
        None => LengthPercentageAuto::auto(),
    }
}

fn length_percent(
    styles: &ComputedStyles<'_>,
    prop: &str,
    ctx: &CalcContext,
) -> Maybe<LengthPercentage> {
    match length_value(styles, prop, ctx) {
        Some(LengthOrPercent::Length(px)) => Maybe::Some(LengthPercentage::length(px)),
        Some(LengthOrPercent::Percent(p)) => Maybe::Some(LengthPercentage::percent(p / 100.0)),
        None => Maybe::None,
    }
}

/// Tristate so a missing longhand can fall through to a shorthand-side
/// default without smashing zero on top of it.
enum Maybe<T> {
    Some(T),
    None,
}

impl Maybe<LengthPercentage> {
    fn or_else_default(self, fallback: Option<LengthPercentage>) -> LengthPercentage {
        match self {
            Maybe::Some(v) => v,
            Maybe::None => fallback.unwrap_or_else(|| LengthPercentage::length(0.0)),
        }
    }
}

impl Maybe<LengthPercentageAuto> {
    fn or_else_default(self, fallback: Option<LengthPercentageAuto>) -> LengthPercentageAuto {
        match self {
            Maybe::Some(v) => v,
            Maybe::None => fallback.unwrap_or_else(|| LengthPercentageAuto::length(0.0)),
        }
    }
}

/// Expand `padding`/`border-width`/etc. shorthand into per-side values.
/// CSS shorthand: 1 value = all four; 2 = (top/bottom, left/right);
/// 3 = (top, left/right, bottom); 4 = (top, right, bottom, left).
/// Returned in [top, right, bottom, left] order.
fn box_shorthand_sides(
    styles: &ComputedStyles<'_>,
    shorthand: &str,
    ctx: &CalcContext,
) -> [Option<LengthPercentage>; 4] {
    let Some(rv) = styles.get(shorthand) else {
        return [None, None, None, None];
    };
    let Ok(Value::List(items)) = parse_property(shorthand, &rv.value) else {
        return [None, None, None, None];
    };
    let lp = |v: &Value| -> Option<LengthPercentage> {
        if let Value::Length(l) = v {
            match l {
                Length::Dim {
                    value,
                    unit: Unit::Percent,
                } => Some(LengthPercentage::percent(*value / 100.0)),
                Length::Dim { value, unit } => {
                    Some(LengthPercentage::length(unit.to_px(*value, ctx)))
                },
                Length::Auto => None,
                Length::Calc { name, body } => Length::Calc {
                    name: name.clone(),
                    body: body.clone(),
                }
                .resolve(ctx)
                .map(LengthPercentage::length),
            }
        } else {
            None
        }
    };
    match items.len() {
        1 => {
            let a = lp(&items[0]);
            [a, a, a, a]
        },
        2 => {
            let (tb, lr) = (lp(&items[0]), lp(&items[1]));
            [tb, lr, tb, lr]
        },
        3 => [lp(&items[0]), lp(&items[1]), lp(&items[2]), lp(&items[1])],
        4 => [lp(&items[0]), lp(&items[1]), lp(&items[2]), lp(&items[3])],
        _ => [None, None, None, None],
    }
}

fn box_shorthand_sides_auto(
    styles: &ComputedStyles<'_>,
    shorthand: &str,
    ctx: &CalcContext,
) -> [Option<LengthPercentageAuto>; 4] {
    let Some(rv) = styles.get(shorthand) else {
        return [None, None, None, None];
    };
    let Ok(Value::List(items)) = parse_property(shorthand, &rv.value) else {
        return [None, None, None, None];
    };
    let lpa = |v: &Value| -> Option<LengthPercentageAuto> {
        if let Value::Length(l) = v {
            match l {
                Length::Dim {
                    value,
                    unit: Unit::Percent,
                } => Some(LengthPercentageAuto::percent(*value / 100.0)),
                Length::Dim { value, unit } => {
                    Some(LengthPercentageAuto::length(unit.to_px(*value, ctx)))
                },
                Length::Auto => Some(LengthPercentageAuto::auto()),
                Length::Calc { name, body } => Length::Calc {
                    name: name.clone(),
                    body: body.clone(),
                }
                .resolve(ctx)
                .map(LengthPercentageAuto::length),
            }
        } else {
            None
        }
    };
    match items.len() {
        1 => {
            let a = lpa(&items[0]);
            [a, a, a, a]
        },
        2 => {
            let (tb, lr) = (lpa(&items[0]), lpa(&items[1]));
            [tb, lr, tb, lr]
        },
        3 => [
            lpa(&items[0]),
            lpa(&items[1]),
            lpa(&items[2]),
            lpa(&items[1]),
        ],
        4 => [
            lpa(&items[0]),
            lpa(&items[1]),
            lpa(&items[2]),
            lpa(&items[3]),
        ],
        _ => [None, None, None, None],
    }
}

fn length_percent_auto(
    styles: &ComputedStyles<'_>,
    prop: &str,
    ctx: &CalcContext,
) -> Maybe<LengthPercentageAuto> {
    if is_auto(styles, prop) {
        return Maybe::Some(LengthPercentageAuto::auto());
    }
    match length_value(styles, prop, ctx) {
        Some(LengthOrPercent::Length(px)) => Maybe::Some(LengthPercentageAuto::length(px)),
        Some(LengthOrPercent::Percent(p)) => Maybe::Some(LengthPercentageAuto::percent(p / 100.0)),
        None => Maybe::None,
    }
}

fn is_auto(styles: &ComputedStyles<'_>, prop: &str) -> bool {
    matches!(get_keyword(styles, prop).as_deref(), Some("auto"))
        || matches!(
            styles
                .get(prop)
                .and_then(|rv| crate::html_css::css::parse_length(&rv.value, prop).ok()),
            Some(Length::Auto)
        )
}

#[derive(Clone, Copy)]
enum LengthOrPercent {
    Length(f32),
    Percent(f32),
}

impl LengthOrPercent {
    fn to_dimension(self) -> Dimension {
        match self {
            LengthOrPercent::Length(px) => Dimension::length(px),
            LengthOrPercent::Percent(p) => Dimension::percent(p / 100.0),
        }
    }

    fn to_size_bound(self) -> LengthPercentageAuto {
        match self {
            LengthOrPercent::Length(px) => LengthPercentageAuto::length(px),
            LengthOrPercent::Percent(p) => LengthPercentageAuto::percent(p / 100.0),
        }
    }
}

fn length_value(
    styles: &ComputedStyles<'_>,
    prop: &str,
    ctx: &CalcContext,
) -> Option<LengthOrPercent> {
    let rv = styles.get(prop)?;
    let parsed = crate::html_css::css::parse_length(&rv.value, prop).ok()?;
    match parsed {
        Length::Dim {
            value,
            unit: Unit::Percent,
        } => Some(LengthOrPercent::Percent(value)),
        Length::Dim { value, unit } => Some(LengthOrPercent::Length(unit.to_px(value, ctx))),
        Length::Auto => None,
        Length::Calc { name, body } => {
            // Re-evaluate calc here. Calc's resolve() handles this for
            // us when called via the typed Length API; we just need to
            // adapt back into a px f32.
            Length::Calc { name, body }
                .resolve(ctx)
                .map(LengthOrPercent::Length)
        },
    }
}

fn number(styles: &ComputedStyles<'_>, prop: &str) -> Option<f32> {
    match parse_property(prop, &styles.get(prop)?.value).ok()? {
        Value::Number(n) => Some(n),
        _ => None,
    }
}

fn get_keyword(styles: &ComputedStyles<'_>, prop: &str) -> Option<String> {
    let rv = styles.get(prop)?;
    match parse_property(prop, &rv.value).ok()? {
        Value::Keyword(s) => Some(s),
        _ => None,
    }
}

fn align_items_keyword(styles: &ComputedStyles<'_>, prop: &str) -> Option<AlignItems> {
    Some(match get_keyword(styles, prop)?.as_str() {
        "flex-start" | "start" => AlignItems::START,
        "flex-end" | "end" => AlignItems::END,
        "center" => AlignItems::CENTER,
        "stretch" => AlignItems::STRETCH,
        "baseline" => AlignItems::BASELINE,
        _ => return None,
    })
}

fn align_content_keyword(styles: &ComputedStyles<'_>, prop: &str) -> Option<AlignContent> {
    Some(match get_keyword(styles, prop)?.as_str() {
        "flex-start" | "start" => AlignContent::START,
        "flex-end" | "end" => AlignContent::END,
        "center" => AlignContent::CENTER,
        "stretch" => AlignContent::STRETCH,
        "space-between" => AlignContent::SPACE_BETWEEN,
        "space-around" => AlignContent::SPACE_AROUND,
        "space-evenly" => AlignContent::SPACE_EVENLY,
        _ => return None,
    })
}

// ─────────────────────────────────────────────────────────────────────
// Layout runner
// ─────────────────────────────────────────────────────────────────────

/// Layout result for one box: position relative to the containing block,
/// and size in px. Positions are top-left origin (HTML semantics);
/// the PAINT phase flips Y when emitting PDF.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutBox {
    /// X in px from parent's content-box origin.
    pub x: f32,
    /// Y in px from parent's content-box origin.
    pub y: f32,
    /// Width in px (border-box).
    pub width: f32,
    /// Height in px (border-box).
    pub height: f32,
}

impl LayoutBox {
    /// Empty/zero box.
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Per-box-tree-id layout output.
#[derive(Debug, Clone, Default)]
pub struct LayoutResult {
    /// Indexed by [`BoxId`] — `boxes[id as usize]` is that box's
    /// position & size, or LayoutBox::zero() if the box wasn't laid
    /// out (e.g. text content, which inline formatting handles).
    pub boxes: Vec<LayoutBox>,
}

/// Compute layout for a [`BoxTree`] given a fixed available size and
/// a base context for unit resolution.
///
/// Inline-level boxes (text and inline elements) get `LayoutBox::zero()`
/// for now — LAYOUT-3 (inline formatter) populates them.
///
/// `style_for` returns the computed style for a given box id; the
/// caller (Phase API) wires it to the cascade. The 'sty lifetime
/// allows the closure to borrow the source stylesheet.
/// Sum the character count of every descendant Text box under
/// `box_id`. Used to give block-level containers an intrinsic height
/// before Taffy layout — without this every `<p>` would be a 0×0 leaf
/// (text children are skipped by Taffy assembly) and sibling `<p>`s
/// would all stack at y=0. Issue #248 release(v0.3.37) B1.
fn collect_text_chars(tree: &BoxTree, id: BoxId) -> usize {
    let mut acc = 0usize;
    let n = tree.get(id);
    if let BoxKind::Text(s) = &n.kind {
        // Compress runs of whitespace to one logical space so the
        // estimate matches what the inline formatter (LAYOUT-3) would
        // produce after `white-space: normal` collapsing.
        let mut last_ws = false;
        for c in s.chars() {
            let is_ws = c.is_whitespace();
            if is_ws {
                if !last_ws {
                    acc += 1;
                }
            } else {
                acc += 1;
            }
            last_ws = is_ws;
        }
    }
    for &c in &n.children {
        acc += collect_text_chars(tree, c);
    }
    acc
}

/// Estimate the wrapped-text height for a block container holding
/// inline text. Approximate but sufficient to make sibling block
/// elements stack vertically at distinct baselines (the precise
/// per-glyph positioning is the inline formatter's job in PAINT).
///
/// Heuristic: char width ≈ 0.5 × font_size, line-height ≈ 1.2 ×
/// font_size, computed wrapped lines = ceil(chars / chars_per_line).
fn estimate_block_text_height(
    tree: &BoxTree,
    box_id: BoxId,
    available_width_px: f32,
    font_size_px: f32,
) -> f32 {
    let chars = collect_text_chars(tree, box_id);
    if chars == 0 {
        return 0.0;
    }
    let char_width = (font_size_px * 0.5).max(1.0);
    let chars_per_line = (available_width_px / char_width).max(1.0);
    let lines = ((chars as f32) / chars_per_line).ceil().max(1.0);
    lines * font_size_px * 1.2
}

/// Run Taffy layout over `tree`, returning per-box positions and sizes.
///
/// `body_font_size_px` is the inherited body font-size used as a fallback
/// when estimating intrinsic text height for inline-bearing blocks.
pub fn run_layout<'sty>(
    tree: &BoxTree,
    style_for: impl Fn(BoxId) -> ComputedStyles<'sty>,
    available: Size<f32>,
    ctx: &CalcContext,
    body_font_size_px: f32,
) -> LayoutResult {
    use taffy::prelude::TaffyTree;

    let mut taffy: TaffyTree<()> = TaffyTree::new();
    let mut tid: Vec<Option<NodeId>> = vec![None; tree.boxes.len()];

    // Build taffy nodes bottom-up so children exist before parents.
    let mut order = tree.iter_ids();
    order.reverse();
    for id in order {
        let node = tree.get(id);
        // Skip boxes that don't get a taffy node (text/inline-level).
        let participates = matches!(node.outside, DisplayOutside::Block | DisplayOutside::ListItem)
            && !matches!(node.kind, BoxKind::Text(_));
        if !participates {
            continue;
        }
        let style = if matches!(node.kind, BoxKind::AnonymousBlock) {
            // Block-flow anonymous wrapper. Taffy's block layout
            // shrinks `width: auto` to content unless it's told to
            // stretch; for the synthetic root we want it to take the
            // full available width so descendant blocks inherit a
            // sensible containing-block width. Set width: 100% which
            // resolves against the available space at compute_layout
            // time.
            let mut s = TaffyStyle::DEFAULT;
            s.display = Display::Block;
            s.size = Size {
                width: Dimension::percent(1.0),
                height: Dimension::auto(),
            };
            // Stamp text-content height so an anonymous block
            // wrapping inline text has a real intrinsic height.
            let h = estimate_block_text_height(tree, id, available.width, body_font_size_px);
            if h > 0.0 {
                s.min_size.height = LengthPercentageAuto::length(h);
            }
            s
        } else {
            let mut computed = style_to_taffy(&style_for(id), ctx, node.outside, node.inside);
            // Block-level boxes with `width: auto` should stretch to
            // fill their containing block — that's how CSS block
            // layout works. Taffy's `Dimension::auto()` shrinks to
            // content; convert to percent(1.0) so non-replaced blocks
            // fill horizontally as expected.
            if matches!(node.outside, DisplayOutside::Block | DisplayOutside::ListItem)
                && computed.size.width == Dimension::auto()
            {
                computed.size.width = Dimension::percent(1.0);
            }
            // Inline content height: when this block has only inline /
            // text descendants (no block children that would otherwise
            // contribute their own heights), give it an intrinsic
            // wrapped-text height so sibling blocks stack vertically
            // (B1 fix). Without this, every `<p>` becomes a 0×0
            // Taffy leaf and they all sit at y=0.
            if matches!(node.outside, DisplayOutside::Block | DisplayOutside::ListItem)
                && computed.size.height == Dimension::auto()
                && computed.min_size.height == LengthPercentageAuto::auto()
            {
                let h = estimate_block_text_height(tree, id, available.width, body_font_size_px);
                if h > 0.0 {
                    computed.min_size.height = LengthPercentageAuto::length(h);
                }
            }
            computed
        };

        let child_taffy_nodes: Vec<NodeId> = node
            .children
            .iter()
            .filter_map(|&c| tid[c as usize])
            .collect();
        let new_node = if child_taffy_nodes.is_empty() {
            taffy.new_leaf(style).expect("taffy leaf")
        } else {
            taffy
                .new_with_children(style, &child_taffy_nodes)
                .expect("taffy node")
        };
        tid[id as usize] = Some(new_node);
    }

    // Compute against available size from the root's taffy node.
    if let Some(root_node) = tid[BoxTree::ROOT as usize] {
        let _ = taffy.compute_layout(
            root_node,
            Size {
                width: AvailableSpace::Definite(available.width),
                height: AvailableSpace::MaxContent,
            },
        );
    }

    // Materialise per-BoxId positions. Taffy's positions are relative
    // to the parent; we walk the tree and accumulate to absolute
    // coordinates.
    let mut out = LayoutResult {
        boxes: vec![LayoutBox::zero(); tree.boxes.len()],
    };
    walk_absolute(tree, &taffy, &tid, BoxTree::ROOT, 0.0, 0.0, available.width, &mut out);
    out
}

fn walk_absolute(
    tree: &BoxTree,
    taffy: &taffy::TaffyTree<()>,
    tid: &[Option<taffy::NodeId>],
    box_id: BoxId,
    parent_x: f32,
    parent_y: f32,
    parent_width: f32,
    out: &mut LayoutResult,
) {
    let mut self_x = parent_x;
    let mut self_y = parent_y;
    let mut self_width = parent_width;
    if let Some(t) = tid[box_id as usize] {
        if let Ok(layout) = taffy.layout(t) {
            self_x = parent_x + layout.location.x;
            self_y = parent_y + layout.location.y;
            self_width = layout.size.width;
            out.boxes[box_id as usize] = LayoutBox {
                x: self_x,
                y: self_y,
                width: layout.size.width,
                height: layout.size.height,
            };
        }
    } else {
        // No taffy node for this box (text/inline). Inherit position
        // and width from the parent so the paginator and paint phase
        // see real coordinates. Height is a small default so paginate
        // doesn't filter it out; LAYOUT-3's inline formatter will
        // refine when it's wired in via the measure-callback.
        out.boxes[box_id as usize] = LayoutBox {
            x: self_x,
            y: self_y,
            width: self_width,
            height: 16.0,
        };
    }
    for &child in &tree.get(box_id).children {
        walk_absolute(tree, taffy, tid, child, self_x, self_y, self_width, out);
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::{cascade, parse_stylesheet, ComputedStyles};
    use crate::html_css::html::parse_document;
    use crate::html_css::layout::box_tree::build_box_tree;

    fn build(
        html: &'static str,
        css: &'static str,
    ) -> (
        BoxTree,
        &'static crate::html_css::css::Stylesheet<'static>,
        &'static crate::html_css::html::Dom,
    ) {
        let dom: &'static _ = Box::leak(Box::new(parse_document(html)));
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let tree = build_box_tree(dom, ss).unwrap();
        (tree, ss, dom)
    }

    fn style_for_factory<'s>(
        ss: &'s crate::html_css::css::Stylesheet<'s>,
        dom: &'s crate::html_css::html::Dom,
        tree: &'s BoxTree,
    ) -> impl Fn(BoxId) -> ComputedStyles<'s> + 's {
        move |id: BoxId| {
            let node = tree.get(id);
            let Some(elem_id) = node.element else {
                return ComputedStyles::default();
            };
            let element = dom.element(elem_id).unwrap();
            cascade(ss, element, None)
        }
    }

    #[test]
    fn block_takes_full_width() {
        let (tree, ss, dom) = build("<div></div>", "");
        let style_for = style_for_factory(ss, dom, &tree);
        let res = run_layout(
            &tree,
            style_for,
            Size {
                width: 600.0,
                height: 400.0,
            },
            &CalcContext::default(),
            12.0,
        );
        // Find the div's BoxId.
        let div_id = tree
            .iter_ids()
            .into_iter()
            .find(|&id| matches!(tree.get(id).kind, BoxKind::Element))
            .unwrap();
        // Default block element with no width takes the parent's full
        // available width.
        assert_eq!(res.boxes[div_id as usize].width, 600.0);
    }

    #[test]
    fn explicit_width_respected() {
        let (tree, ss, dom) = build("<div></div>", "div { width: 200px }");
        let style_for = style_for_factory(ss, dom, &tree);
        let res = run_layout(
            &tree,
            style_for,
            Size {
                width: 600.0,
                height: 400.0,
            },
            &CalcContext::default(),
            12.0,
        );
        let div_id = tree
            .iter_ids()
            .into_iter()
            .find(|&id| matches!(tree.get(id).kind, BoxKind::Element))
            .unwrap();
        assert_eq!(res.boxes[div_id as usize].width, 200.0);
    }

    #[test]
    fn padding_increases_outer_size_under_content_box() {
        let (tree, ss, dom) = build("<div></div>", "div { width: 200px; padding: 10px }");
        let style_for = style_for_factory(ss, dom, &tree);
        let res = run_layout(
            &tree,
            style_for,
            Size {
                width: 600.0,
                height: 400.0,
            },
            &CalcContext::default(),
            12.0,
        );
        let div_id = tree
            .iter_ids()
            .into_iter()
            .find(|&id| matches!(tree.get(id).kind, BoxKind::Element))
            .unwrap();
        // content-box width 200 + 10 left + 10 right = 220
        assert_eq!(res.boxes[div_id as usize].width, 220.0);
    }

    #[test]
    fn border_box_includes_padding_in_width() {
        let (tree, ss, dom) =
            build("<div></div>", "div { width: 200px; padding: 10px; box-sizing: border-box }");
        let style_for = style_for_factory(ss, dom, &tree);
        let res = run_layout(
            &tree,
            style_for,
            Size {
                width: 600.0,
                height: 400.0,
            },
            &CalcContext::default(),
            12.0,
        );
        let div_id = tree
            .iter_ids()
            .into_iter()
            .find(|&id| matches!(tree.get(id).kind, BoxKind::Element))
            .unwrap();
        // box-sizing: border-box ⇒ width 200 includes the padding
        assert_eq!(res.boxes[div_id as usize].width, 200.0);
    }

    #[test]
    fn block_stacking() {
        let (tree, ss, dom) = build("<div></div><div></div>", "div { width: 100px; height: 50px }");
        let style_for = style_for_factory(ss, dom, &tree);
        let res = run_layout(
            &tree,
            style_for,
            Size {
                width: 600.0,
                height: 400.0,
            },
            &CalcContext::default(),
            12.0,
        );
        let divs: Vec<BoxId> = tree
            .iter_ids()
            .into_iter()
            .filter(|&id| matches!(tree.get(id).kind, BoxKind::Element))
            .collect();
        assert_eq!(divs.len(), 2);
        // Second div sits below the first.
        assert_eq!(res.boxes[divs[0] as usize].y, 0.0);
        assert_eq!(res.boxes[divs[1] as usize].y, 50.0);
    }

    #[test]
    fn flex_row_distributes_horizontally() {
        let (tree, ss, dom) = build(
            "<div><span></span><span></span></div>",
            "div { display: flex; width: 600px; height: 100px } \
             span { display: block; width: 100px; height: 100px }",
        );
        let style_for = style_for_factory(ss, dom, &tree);
        let res = run_layout(
            &tree,
            style_for,
            Size {
                width: 600.0,
                height: 400.0,
            },
            &CalcContext::default(),
            12.0,
        );
        let elements: Vec<BoxId> = tree
            .iter_ids()
            .into_iter()
            .filter(|&id| matches!(tree.get(id).kind, BoxKind::Element))
            .collect();
        // 1 div + 2 spans
        assert_eq!(elements.len(), 3);
        // Spans are children of the flex container, side by side.
        let span_xs: Vec<f32> = elements[1..]
            .iter()
            .map(|&id| res.boxes[id as usize].x)
            .collect();
        // Both spans have x ≥ 0 and their xs differ (one to the right
        // of the other).
        assert!(span_xs.iter().all(|&x| x >= 0.0));
        assert_ne!(span_xs[0], span_xs[1]);
    }
}
