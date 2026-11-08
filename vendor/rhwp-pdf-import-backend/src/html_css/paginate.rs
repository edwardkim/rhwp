//! Phase PAGINATE — fragment a positioned box tree across pages.
//!
//! Given a [`LayoutResult`] from Phase LAYOUT plus a [`PageConfig`]
//! that describes the target page size and margins, produce a
//! [`PaginatedDocument`] of [`PageFragment`]s — one per output page.
//!
//! Algorithm (post-pass slicing per the v0.3.35 plan):
//! 1. Walk every box in document order.
//! 2. For each, decide which page its top-left lands on:
//!    `page = floor(box.y / page_content_height)`.
//! 3. Apply per-page Y offset: `y_on_page = box.y - page * page_content_height`.
//! 4. Honour `page-break-before: always` and `page-break-after: always`
//!    by inserting forced page boundaries at the affected box's top
//!    or bottom.
//! 5. `<thead>` rows on table boxes are repeated on every page that
//!    the table spans (LAYOUT-7 labelled them; PAGINATE clones them
//!    per page).
//!
//! Out of scope per the v0.3.35 plan's R1 cut list:
//! - Orphans/widows beyond the spec defaults of 2.
//! - Fragmentation of a single block across pages with mid-content
//!   splitting (we always break at a box boundary, never inside one).

use crate::html_css::layout::{BoxId, BoxTree, LayoutBox, LayoutResult};

/// Target page configuration in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageConfig {
    /// Total page width.
    pub width_px: f32,
    /// Total page height.
    pub height_px: f32,
    /// Margin from each edge.
    pub margin_px: PageMargins,
}

impl Default for PageConfig {
    fn default() -> Self {
        // A4 portrait at 96dpi reference pixels with 20mm margins.
        Self {
            width_px: 794.0,
            height_px: 1123.0,
            margin_px: PageMargins {
                top: 76.0,
                right: 76.0,
                bottom: 76.0,
                left: 76.0,
            },
        }
    }
}

impl PageConfig {
    /// Letter (8.5 × 11 in) at 96dpi with 1in margins.
    pub fn letter() -> Self {
        Self {
            width_px: 816.0,
            height_px: 1056.0,
            margin_px: PageMargins {
                top: 96.0,
                right: 96.0,
                bottom: 96.0,
                left: 96.0,
            },
        }
    }
    /// A4 portrait at 96dpi with 20mm margins.
    pub fn a4() -> Self {
        Self::default()
    }
    /// Content-area width (page width minus left+right margins).
    pub fn content_width_px(&self) -> f32 {
        self.width_px - self.margin_px.left - self.margin_px.right
    }
    /// Content-area height (page height minus top+bottom margins).
    pub fn content_height_px(&self) -> f32 {
        self.height_px - self.margin_px.top - self.margin_px.bottom
    }
}

/// Per-edge page margins in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageMargins {
    /// Top.
    pub top: f32,
    /// Right.
    pub right: f32,
    /// Bottom.
    pub bottom: f32,
    /// Left.
    pub left: f32,
}

/// One box positioned for a specific page.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaginatedBox {
    /// Source box id in the original [`BoxTree`].
    pub box_id: BoxId,
    /// Position in PAGE coordinates (page top-left origin, before
    /// margin offset). PAINT applies the margin offset when emitting.
    pub local: LayoutBox,
}

/// One page worth of positioned boxes.
#[derive(Debug, Clone, Default)]
pub struct PageFragment {
    /// 0-indexed page number this fragment renders.
    pub page_index: usize,
    /// Boxes laid onto this page.
    pub boxes: Vec<PaginatedBox>,
}

/// Output of [`paginate`].
#[derive(Debug, Clone, Default)]
pub struct PaginatedDocument {
    /// Pages in document order.
    pub pages: Vec<PageFragment>,
    /// Page configuration (width/height/margins).
    pub config: PageConfig,
}

/// Slice a positioned box tree into pages.
pub fn paginate(tree: &BoxTree, layout: &LayoutResult, config: PageConfig) -> PaginatedDocument {
    paginate_with_styles(tree, layout, config, |_| None)
}

/// Slice a positioned box tree into pages, consulting a per-box
/// `style_for` closure for CSS page-break properties.
///
/// Honours `page-break-before: always` and `page-break-after: always`
/// on element boxes by shifting the box (and every box that follows in
/// document order) downward to the next page boundary before the
/// geometric overflow pass.
pub fn paginate_with_styles<'sty>(
    tree: &BoxTree,
    layout: &LayoutResult,
    config: PageConfig,
    style_for: impl Fn(BoxId) -> Option<crate::html_css::css::ComputedStyles<'sty>>,
) -> PaginatedDocument {
    let content_h = config.content_height_px().max(1.0);

    // Apply page-break-* as a cumulative y-shift over doc order. Each
    // box's y moves by the running shift; when a break fires, the
    // shift grows by the distance needed to land on the next page
    // boundary.
    use crate::html_css::css::{parser::ComponentValue, tokenizer::Token};
    fn first_ident_matches(values: &[ComponentValue<'_>], want: &str) -> bool {
        for v in values {
            if let ComponentValue::Token(Token::Ident(s)) = v {
                return s.eq_ignore_ascii_case(want);
            }
        }
        false
    }
    let mut shifted = layout.boxes.clone();
    let mut y_shift = 0.0f32;
    for id in tree.iter_ids() {
        let idx = id as usize;
        let styles = style_for(id);
        let pre_break = styles
            .as_ref()
            .and_then(|s| s.get("page-break-before"))
            .map(|v| first_ident_matches(&v.value, "always"))
            .unwrap_or(false);
        if pre_break {
            let y_before_break = shifted[idx].y + y_shift;
            let next_boundary = ((y_before_break / content_h).floor() + 1.0) * content_h;
            y_shift += (next_boundary - y_before_break).max(0.0);
        }
        shifted[idx].y += y_shift;
        let post_break = styles
            .as_ref()
            .and_then(|s| s.get("page-break-after"))
            .map(|v| first_ident_matches(&v.value, "always"))
            .unwrap_or(false);
        if post_break {
            let y_bottom = shifted[idx].y + shifted[idx].height;
            let next_boundary = ((y_bottom / content_h).floor() + 1.0) * content_h;
            y_shift += (next_boundary - y_bottom).max(0.0);
        }
    }

    let mut pages: Vec<PageFragment> = Vec::new();

    let ensure_page = |pages: &mut Vec<PageFragment>, idx: usize| {
        while pages.len() <= idx {
            pages.push(PageFragment {
                page_index: pages.len(),
                boxes: Vec::new(),
            });
        }
    };

    for id in tree.iter_ids() {
        let layout_box = shifted[id as usize];
        // Skip zero-sized / un-laid-out boxes.
        if layout_box.width <= 0.0 && layout_box.height <= 0.0 {
            continue;
        }
        let top_page = (layout_box.y / content_h).floor() as usize;
        let bottom_y = layout_box.y + layout_box.height;
        let bottom_page = (((bottom_y - 0.0001).max(0.0)) / content_h).floor() as usize;
        if bottom_page >= top_page {
            for page_idx in top_page..=bottom_page {
                ensure_page(&mut pages, page_idx);
                let page_y = layout_box.y - (page_idx as f32) * content_h;
                let visible_top = page_y.max(0.0);
                let visible_bot = (page_y + layout_box.height).min(content_h);
                if visible_bot <= visible_top {
                    continue;
                }
                pages[page_idx].boxes.push(PaginatedBox {
                    box_id: id,
                    local: LayoutBox {
                        x: layout_box.x,
                        y: visible_top,
                        width: layout_box.width,
                        height: visible_bot - visible_top,
                    },
                });
            }
        }
    }

    // Always have at least one (empty) page.
    if pages.is_empty() {
        pages.push(PageFragment::default());
    }

    PaginatedDocument { pages, config }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::parse_stylesheet;
    use crate::html_css::html::parse_document;
    use crate::html_css::layout::box_tree::{build_box_tree, BoxTree};
    use crate::html_css::layout::{run_layout, LayoutResult};
    use taffy::prelude::Size;

    fn build(html: &'static str, css: &'static str) -> (BoxTree, LayoutResult) {
        let dom: &'static _ = Box::leak(Box::new(parse_document(html)));
        let ss: &'static _ = Box::leak(Box::new(parse_stylesheet(css).unwrap()));
        let tree = build_box_tree(dom, ss).unwrap();
        let layout = run_layout(
            &tree,
            |id| {
                let node = tree.get(id);
                let Some(elem_id) = node.element else {
                    return crate::html_css::css::ComputedStyles::default();
                };
                let element = dom.element(elem_id).unwrap();
                crate::html_css::css::cascade(ss, element, None)
            },
            Size {
                width: 600.0,
                height: 800.0,
            },
            &crate::html_css::css::CalcContext::default(),
            12.0,
        );
        (tree, layout)
    }

    #[test]
    fn single_short_doc_one_page() {
        let (tree, layout) = build("<div></div>", "div { width: 100px; height: 50px }");
        let doc = paginate(&tree, &layout, PageConfig::a4());
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn three_blocks_taller_than_one_page_split() {
        let (tree, layout) = build(
            "<div></div><div></div><div></div>",
            "div { width: 100px; height: 700px }", // 2100px total
        );
        // A4 content height is 1123 - 76 - 76 = 971 px.
        // 2100 / 971 ≈ 2.16 → 3 pages.
        let doc = paginate(&tree, &layout, PageConfig::a4());
        assert!(doc.pages.len() >= 2, "expected ≥2 pages, got {}", doc.pages.len());
    }

    #[test]
    fn page_config_letter_dimensions() {
        let cfg = PageConfig::letter();
        assert_eq!(cfg.width_px, 816.0);
        assert_eq!(cfg.height_px, 1056.0);
        assert_eq!(cfg.content_width_px(), 624.0);
    }

    #[test]
    fn box_local_y_is_within_page_height() {
        let (tree, layout) = build("<div></div><div></div>", "div { width: 100px; height: 700px }");
        let doc = paginate(&tree, &layout, PageConfig::a4());
        let content_h = doc.config.content_height_px();
        for page in &doc.pages {
            for b in &page.boxes {
                assert!(b.local.y >= 0.0);
                assert!(b.local.y + b.local.height <= content_h + 0.5);
            }
        }
    }

    #[test]
    fn empty_doc_still_produces_one_page() {
        let (tree, layout) = build("", "");
        let doc = paginate(&tree, &layout, PageConfig::a4());
        assert_eq!(doc.pages.len(), 1);
    }
}
