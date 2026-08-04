//! v2 `<location>` geometry pass (rhwp-aware).
//!
//! Builds a [`LocationMap`] (`Prov -> Location`) by laying out every page of the
//! document with `DocumentCore::build_page_render_tree` and walking the
//! resulting `RenderNode` tree. Each render node carries model provenance
//! (`section_index` / `para_index` / `control_index`); we join those onto the
//! Semantic IR's [`Prov`] stamps so the writer can emit four `<location>`
//! head elements per resolved block.
//!
//! This is the **only** location-feature module that depends on rhwp, keeping
//! the IR and writer rhwp-agnostic. It is gated behind `ConvertOptions
//! ::with_location`: when disabled the whole pass is skipped (it re-layouts
//! every page and is therefore not free).
//!
//! # What gets a box (per verified mapping rules — see
//! `.omc/research/bbox-block-mapping-probe.md`)
//!
//! * **Body paragraphs / headings / lists**: `TextLine(section, para)` nodes
//!   found under a `Body`/`Column` subtree, unioned per `(section, para)`.
//!   `para == usize::MAX` (synthetic furniture lines) are skipped.
//! * **Tables**: top-level `Table(section, para, control)` nodes. The 1×1
//!   wrapper-flattening case (research §3) is handled by the model side: our IR
//!   tables carry the wrapper's `(s, p, c)`, which the render tree reports
//!   directly, so the key matches without special handling. (We additionally
//!   accept a render Table whose `(s, p, c)` matches even if its reported dims
//!   differ from the wrapper, since dims are not part of the key.)
//! * **Pictures**: `Image(section, para, control)` nodes that are NOT inside a
//!   table cell (`cell_context == None`) and NOT header/footer images
//!   (`header_footer_ref == None`). Cell-internal pictures carry no global
//!   provenance and are skipped.
//! * **Formulas**: `Equation(section, para, control)` nodes that are NOT inside
//!   a cell (`cell_index == None`) and NOT inside a note. Cell-internal
//!   equations are skipped (documented limitation).
//! * **Page headers/footers**: the per-page `Header` / `Footer` group node
//!   bounds, attached to the header/footer control by matching the body
//!   paragraph + control index that owns it. Because a header/footer group
//!   carries no provenance payload, we resolve the owning control by scanning
//!   the section's body paragraphs for the single Header/Footer control and key
//!   the box under that `(section, para, control)`.
//!
//! # Skipped for v2 (documented)
//!
//! * **Textbox-internal content** — rendered as rescued plain blocks with no
//!   stable provenance; recorded as nothing (geometry is optional in DocLang).
//! * **Footnote/endnote content** — `FootnoteArea` is a single per-page region
//!   covering possibly several notes; there is no per-note box, so footnote
//!   blocks get no location.
//! * **Cell-internal tables/pictures/formulas** — cell-local indices, no global
//!   provenance.
//!
//! # Multi-page blocks
//!
//! A block split across pages (threaded text, a table that breaks) is unioned
//! only within the first page on which it appears: we record the box for a
//! `Prov` the first time we see it and never widen it across page boundaries.
//! This keeps the box anchored to the block's leading segment.

use std::collections::HashMap;

use crate::model::control::Control;
use crate::model::document::Document;
use crate::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use crate::DocumentCore;

use crate::doclang::ir::prov::{Location, LocationMap, Prov};
use crate::doclang::loss::report::{LossEntry, LossKind, LossReport};

/// A pixel-space bounding box accumulator (min/max corners).
#[derive(Clone, Copy)]
struct PxBox {
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
}

impl PxBox {
    fn from_bbox(b: &BoundingBox) -> Self {
        PxBox {
            x0: b.x,
            y0: b.y,
            x1: b.x + b.width,
            y1: b.y + b.height,
        }
    }

    fn union(&mut self, b: &BoundingBox) {
        self.x0 = self.x0.min(b.x);
        self.y0 = self.y0.min(b.y);
        self.x1 = self.x1.max(b.x + b.width);
        self.y1 = self.y1.max(b.y + b.height);
    }
}

/// Per-page accumulation: which page first claimed each `Prov`, and the union
/// box (in px) within that page.
struct Accum {
    /// page index that first introduced this prov
    page: u32,
    /// page dimensions (px) for normalisation
    page_w: f64,
    page_h: f64,
    /// accumulated union box
    bx: PxBox,
}

/// Build the full `Prov -> Location` map for `data`.
///
/// `document` is the already-parsed model (used to resolve which body control a
/// per-page Header/Footer group belongs to). Best-effort: a page that fails to
/// lay out contributes no boxes rather than aborting the whole conversion.
///
/// Failures are surfaced through `loss` so a caller can tell *why* `<location>`
/// coverage is empty/partial instead of seeing a silent empty map:
/// - the whole layout core failing to build (no boxes at all), or
/// - individual pages failing to lay out (partial coverage).
pub(crate) fn build_location_map(
    data: &[u8],
    document: &Document,
    loss: &mut LossReport,
) -> LocationMap {
    let core = match DocumentCore::from_bytes(data) {
        Ok(c) => c,
        Err(_) => {
            loss.push(LossEntry {
                kind: LossKind::Other("location-unavailable".to_string()),
                location: "document".to_string(),
                detail: "layout core failed to build; no <location> coordinates emitted"
                    .to_string(),
            });
            return LocationMap::new();
        }
    };
    build_from_core(&core, document, loss)
}

/// Inner builder once a [`DocumentCore`] is available (kept separate so it can
/// be unit-tested with a synthetic core / document in the future).
fn build_from_core(core: &DocumentCore, document: &Document, loss: &mut LossReport) -> LocationMap {
    let mut acc: HashMap<Prov, Accum> = HashMap::new();
    let mut failed_pages: Vec<u32> = Vec::new();

    // Precompute, per section, the (para, control) of its sole Header/Footer
    // control so the per-page Header/Footer group bounds can be attached.
    let hf = header_footer_controls(document);

    // Precompute model table dimensions by provenance so the 1×1 wrapper
    // flattening rule (research §3) can validate render Table nodes.
    let table_dims = body_table_dims(document);

    for page in 0..core.page_count() {
        let tree = match core.build_page_render_tree(page) {
            Ok(t) => t,
            Err(_) => {
                failed_pages.push(page);
                continue;
            }
        };
        let (page_w, page_h, section) = page_dims(&tree.root);
        let ctx = Ctx {
            page,
            page_w,
            page_h,
            section,
            in_cell: false,
            in_textbox: false,
            in_note: false,
        };
        walk(&tree.root, ctx, &hf, &table_dims, &mut acc);
    }

    if !failed_pages.is_empty() {
        loss.push(LossEntry {
            kind: LossKind::Other("location-page-layout-failed".to_string()),
            location: "document".to_string(),
            detail: format!(
                "{} of {} page(s) failed to lay out; <location> coverage is partial (pages {:?})",
                failed_pages.len(),
                core.page_count(),
                failed_pages
            ),
        });
    }

    acc.into_iter()
        .map(|(prov, a)| {
            (
                prov,
                Location::from_px_box_on_page(
                    a.page,
                    a.bx.x0,
                    a.bx.y0,
                    a.bx.x1 - a.bx.x0,
                    a.bx.y1 - a.bx.y0,
                    a.page_w,
                    a.page_h,
                ),
            )
        })
        .collect()
}

/// Per-section header/footer owning control: `section -> (header_prov, footer_prov)`.
struct HfControls {
    header: HashMap<usize, Prov>,
    footer: HashMap<usize, Prov>,
}

/// Model table dimensions (`row_count`, `col_count`) for every top-level body
/// table, keyed by its `(section, para, control)` provenance. Used by the
/// 1×1 wrapper flattening rule to validate render Table nodes.
fn body_table_dims(document: &Document) -> HashMap<Prov, (u16, u16)> {
    let mut out = HashMap::new();
    for (si, section) in document.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Control::Table(t) = ctrl {
                    out.insert(Prov::object(si, pi, ci), (t.row_count, t.col_count));
                }
            }
        }
    }
    out
}

fn header_footer_controls(document: &Document) -> HfControls {
    let mut header = HashMap::new();
    let mut footer = HashMap::new();
    for (si, section) in document.sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Header(_) => {
                        header.entry(si).or_insert_with(|| Prov::object(si, pi, ci));
                    }
                    Control::Footer(_) => {
                        footer.entry(si).or_insert_with(|| Prov::object(si, pi, ci));
                    }
                    _ => {}
                }
            }
        }
    }
    HfControls { header, footer }
}

/// The 1×1 wrapper-flattening decision (research note §3).
///
/// rhwp's layout engine flattens a *trivial* 1×1 wrapper table and reports a
/// single render node that carries the **outer** ref `(s, p, c)` but the
/// **inner** (nested) table's dimensions. Our IR keys tables by `(s, p, c)`
/// only, so the outer ref always matches regardless of dims — but when we need
/// to confirm a render Table node corresponds to a given model table, this rule
/// says: a render node matches the model table at `(s, p, c)` when either the
/// dimensions agree, OR the model table is a 1×1 wrapper (the flattened case).
///
/// Returns `true` when the render node should be accepted as the box for the
/// model table identified by `(model_rows, model_cols)`.
fn table_dims_match(model_rows: u16, model_cols: u16, render_rows: u16, render_cols: u16) -> bool {
    if model_rows == render_rows && model_cols == render_cols {
        return true;
    }
    // 1×1 wrapper flattening: rhwp reports the nested table's dims under the
    // wrapper's ref. Accept the box for the wrapper.
    model_rows == 1 && model_cols == 1
}

/// Extract `(width, height, section_index)` from the page root node.
fn page_dims(root: &RenderNode) -> (f64, f64, usize) {
    if let RenderNodeType::Page(p) = &root.node_type {
        (p.width, p.height, p.section_index)
    } else {
        (root.bbox.width, root.bbox.height, 0)
    }
}

/// Walk context: tracks page dims, the active section, and whether we are inside
/// a table cell / textbox / note subtree (which suppresses provenance joins for
/// content lacking global provenance).
#[derive(Clone, Copy)]
struct Ctx {
    page: u32,
    page_w: f64,
    page_h: f64,
    section: usize,
    in_cell: bool,
    in_textbox: bool,
    in_note: bool,
}

fn walk(
    node: &RenderNode,
    ctx: Ctx,
    hf: &HfControls,
    table_dims: &HashMap<Prov, (u16, u16)>,
    acc: &mut HashMap<Prov, Accum>,
) {
    let mut next = ctx;
    match &node.node_type {
        // Header / Footer group: attach the group bounds to the owning control.
        RenderNodeType::Header => {
            if let Some(prov) = hf.header.get(&ctx.section) {
                record(acc, *prov, &node.bbox, ctx);
            }
            // Header content lines have no stable per-line provenance for v2.
            next.in_textbox = true;
        }
        RenderNodeType::Footer => {
            if let Some(prov) = hf.footer.get(&ctx.section) {
                record(acc, *prov, &node.bbox, ctx);
            }
            next.in_textbox = true;
        }
        // Textbox / master-page / footnote-area subtrees: their text lines carry
        // section-local or cell-local indices that collide with body provenance,
        // so we must not treat them as body content. Footnote content is skipped.
        RenderNodeType::TextBox | RenderNodeType::MasterPage => next.in_textbox = true,
        RenderNodeType::FootnoteArea => next.in_note = true,

        // Body text line: union per (section, para) when in real body context.
        RenderNodeType::TextLine(t) => {
            if !ctx.in_cell && !ctx.in_textbox && !ctx.in_note {
                if let (Some(s), Some(p)) = (t.section_index, t.para_index) {
                    if p != usize::MAX && s != usize::MAX {
                        record(acc, Prov::text(s, p), &node.bbox, ctx);
                    }
                }
            }
        }

        // Table: top-level (non-cell) tables get a box keyed by (s, p, c).
        RenderNodeType::Table(t) => {
            if !ctx.in_cell && !ctx.in_textbox {
                if let (Some(s), Some(p), Some(c)) =
                    (t.section_index, t.para_index, t.control_index)
                {
                    let prov = Prov::object(s, p, c);
                    // Validate against the model table dims, applying the 1×1
                    // wrapper flattening rule (research §3). When the prov is
                    // not a known body table (e.g. nested), skip.
                    let ok = table_dims
                        .get(&prov)
                        .map(|&(mr, mc)| table_dims_match(mr, mc, t.row_count, t.col_count))
                        .unwrap_or(false);
                    if ok {
                        record(acc, prov, &node.bbox, ctx);
                    }
                }
            }
            // Descend into cells with the cell flag set so nested content is
            // not mistaken for body content.
            next.in_cell = true;
        }
        RenderNodeType::TableCell(_) => next.in_cell = true,

        // Picture: only body pictures (not in a cell, not header/footer images).
        RenderNodeType::Image(i) => {
            if !ctx.in_cell
                && !ctx.in_textbox
                && i.cell_context.is_none()
                && i.header_footer_ref.is_none()
            {
                if let (Some(s), Some(p), Some(c)) =
                    (i.section_index, i.para_index, i.control_index)
                {
                    record(acc, Prov::object(s, p, c), &node.bbox, ctx);
                }
            }
        }

        // Equation: only body formulas (not cell-internal, not note-internal).
        RenderNodeType::Equation(e)
            if !ctx.in_cell
                && !ctx.in_textbox
                && !ctx.in_note
                && e.cell_index.is_none()
                && e.note_ref.is_none() =>
        {
            if let (Some(s), Some(p), Some(c)) = (e.section_index, e.para_index, e.control_index) {
                record(acc, Prov::object(s, p, c), &node.bbox, ctx);
            }
        }

        _ => {}
    }

    for child in &node.children {
        walk(child, next, hf, table_dims, acc);
    }
}

/// Record a box for `prov`. The first page to introduce a prov owns it; later
/// nodes on the same page widen the union, nodes on later pages are ignored
/// (multi-page blocks use the first-page segment only).
fn record(acc: &mut HashMap<Prov, Accum>, prov: Prov, bbox: &BoundingBox, ctx: Ctx) {
    match acc.entry(prov) {
        std::collections::hash_map::Entry::Vacant(v) => {
            v.insert(Accum {
                page: ctx.page,
                page_w: ctx.page_w,
                page_h: ctx.page_h,
                bx: PxBox::from_bbox(bbox),
            });
        }
        std::collections::hash_map::Entry::Occupied(mut o) => {
            if o.get().page == ctx.page {
                o.get_mut().bx.union(bbox);
            }
            // Different (later) page: keep the first-page box only.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pxbox_union_widens_both_corners() {
        let mut b = PxBox::from_bbox(&BoundingBox::new(10.0, 10.0, 5.0, 5.0));
        b.union(&BoundingBox::new(20.0, 0.0, 5.0, 30.0));
        assert_eq!(b.x0, 10.0);
        assert_eq!(b.y0, 0.0);
        assert_eq!(b.x1, 25.0);
        assert_eq!(b.y1, 30.0);
    }

    #[test]
    fn flattening_rule_exact_dims_match() {
        assert!(table_dims_match(15, 9, 15, 9));
        assert!(!table_dims_match(15, 9, 6, 3));
    }

    #[test]
    fn flattening_rule_1x1_wrapper_accepts_mismatched_render_dims() {
        // research §3: model 1×1 wrapper, render reports nested 6×3 dims.
        assert!(table_dims_match(1, 1, 6, 3));
        assert!(table_dims_match(1, 1, 2, 2));
        // A non-1×1 model with mismatched dims is NOT flattened.
        assert!(!table_dims_match(2, 2, 1, 1));
    }

    #[test]
    fn header_footer_controls_picks_first_per_section() {
        use crate::model::document::{Document, Section};
        use crate::model::paragraph::Paragraph;

        let mut p = Paragraph::default();
        // `Box::default()` infers the boxed Header/Footer payload type.
        p.controls.push(Control::Header(Box::default()));
        p.controls.push(Control::Footer(Box::default()));
        let doc = Document {
            sections: vec![Section {
                paragraphs: vec![p],
                ..Default::default()
            }],
            ..Default::default()
        };
        let hf = header_footer_controls(&doc);
        assert_eq!(hf.header.get(&0), Some(&Prov::object(0, 0, 0)));
        assert_eq!(hf.footer.get(&0), Some(&Prov::object(0, 0, 1)));
    }

    #[test]
    fn record_first_page_wins_for_multipage_block() {
        let mut acc: HashMap<Prov, Accum> = HashMap::new();
        let prov = Prov::text(0, 0);
        let ctx0 = Ctx {
            page: 0,
            page_w: 100.0,
            page_h: 100.0,
            section: 0,
            in_cell: false,
            in_textbox: false,
            in_note: false,
        };
        record(
            &mut acc,
            prov,
            &BoundingBox::new(10.0, 10.0, 5.0, 5.0),
            ctx0,
        );
        // Same page widens.
        record(
            &mut acc,
            prov,
            &BoundingBox::new(10.0, 20.0, 5.0, 5.0),
            ctx0,
        );
        // Later page is ignored.
        let mut ctx1 = ctx0;
        ctx1.page = 1;
        record(
            &mut acc,
            prov,
            &BoundingBox::new(0.0, 0.0, 100.0, 100.0),
            ctx1,
        );
        let a = acc.get(&prov).unwrap();
        assert_eq!(a.page, 0);
        assert_eq!(a.bx.y1, 25.0); // widened by the same-page record only
    }
}
