//! `Control` variant dispatch (Phase A — block-level objects).
//!
//! Lowers a single rhwp [`Control`] (the inline/extended objects embedded in a
//! paragraph: tables, pictures, equations, footnotes, headers/footers, shapes,
//! …) into zero or more crate-IR [`Block`]s, plus optional marker / loss
//! signals for the orchestrator (T8).
//!
//! # Callback contract (decoupling from sibling modules)
//!
//! `convert_control` does **not** import `paragraph.rs` or `table.rs` directly,
//! because those modules are built concurrently / later.  Instead the caller
//! supplies two closures:
//!
//! - **`convert_paragraphs: &dyn Fn(&[Paragraph]) -> Vec<Block>`** — lowers a
//!   slice of rhwp paragraphs (footnote / header / cell-like bodies) into IR
//!   blocks.  Implemented by T5 (`paragraph.rs`).  Must recurse into nested
//!   controls itself.
//! - **`convert_table: &dyn Fn(&Table) -> ir::Table`** — lowers one rhwp
//!   [`Table`] into the IR table type.  Implemented by T6 (`table.rs`).
//! - **`resolve_picture: &dyn Fn(u16) -> Option<(Vec<u8>, String)>`** —
//!   resolves a `Picture.image_attr.bin_data_id` to its decoded bytes and
//!   lower-case extension.  Implemented by T8 as a thin wrapper over
//!   [`super::resources::bin_data_bytes`] (which owns the storage-id lookup);
//!   the callback exists because this module has no `&Document` handle.
//!
//! Bundling these into [`ControlCtx`] keeps the call sites short and lets the
//! orchestrator swap in real implementations once they exist.  Unit tests pass
//! trivial stubs.
//!
//! # Mode-aware policy
//!
//! Every variant follows one of these strategies, keyed off [`Mode`]:
//!
//! | Variant                | Outcome |
//! |------------------------|---------|
//! | `Table`                | `Block::Table` via `convert_table` callback |
//! | `Picture`              | `Block::Picture { data, extension }`; missing bin-data → `LossEntry(FloatingObject)` + skip |
//! | `Equation`             | `Block::Formula { raw_eqedit = script, latex: None }` (LaTeX wired in T13) |
//! | `Footnote`             | `Block::Footnote { number, content }` via callback |
//! | `Endnote`              | `Block::Footnote` (DocLang has no endnote) + `LossEntry(Other,"endnote mapped to footnote")` |
//! | `Header` / `Footer`    | `Block::PageHeader` / `PageFooter`, apply mapped from `HeaderFooterApply` |
//! | `Shape`                | text-rescue: extract carried paragraphs as plain blocks; always `LossEntry(TextBox/FloatingObject)`; preserve mode adds `Block::Custom` w/ geometry |
//! | `SectionDef`           | `ControlOutcome::SectionMarker` for T8 page/thread handling |
//! | `ColumnDef`            | `ControlOutcome::ColumnMarker` for T8 thread handling |
//! | `HiddenComment`        | text-rescue paragraphs (carries body) + `LossEntry(Other,"hidden comment")` |
//! | `Hyperlink`            | text-rescue: display text → plain block (HWP3-only; IR has no `<href>` inline, URL not yet extracted) + `LossEntry(Other)` |
//! | `Bookmark`             | `LossEntry(Other, "bookmark: <name>")` |
//! | `Ruby`                 | text-rescue: annotation text → plain block + `LossEntry(Other, "ruby: <text>")` |
//! | `CharOverlap`          | text-rescue: overlapped glyphs → plain block + `LossEntry(Other, "char overlap")` |
//! | `Field` / `Form`       | `LossEntry(Other, …)` describing the field/form |
//! | `AutoNumber` / `NewNumber` / `PageNumberPos` / `PageHide` | silently skipped (page-numbering chrome, no body content) |
//! | `Unknown`              | `LossEntry(Other, "unknown control 0x…")` |

use crate::model::control::{Control, FieldType};
use crate::model::header_footer::HeaderFooterApply;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{CommonObjAttr, ShapeObject};
use crate::model::table::Table;

use crate::doclang::ir::{self, Block, Formula};
use crate::doclang::loss::report::{LossEntry, LossKind, LossReport};
use crate::doclang::options::Mode;

use super::geometry;

/// Conversion callbacks supplied by the orchestrator (T8) so that this module
/// does not depend on `paragraph.rs` / `table.rs` being finished.
///
/// See the module docs for the contract each closure must satisfy.
pub(crate) struct ControlCtx<'a> {
    /// Conversion mode (lean vs preserve).
    pub mode: Mode,
    /// Human-readable location prefix for loss entries
    /// (e.g. `"section[0]/para[3]/control[1]"`).
    pub location: &'a str,
    /// Lowers a paragraph slice into IR blocks (T5).
    pub convert_paragraphs: &'a dyn Fn(&[Paragraph]) -> Vec<Block>,
    /// Lowers an rhwp table into an IR table (T6).
    pub convert_table: &'a dyn Fn(&Table) -> ir::Table,
    /// Resolves a picture `bin_data_id` to `(bytes, extension)` (T8, wraps
    /// [`super::resources::bin_data_bytes`]).  Returns `None` when the embedded
    /// binary is absent.
    pub resolve_picture: &'a dyn Fn(u16) -> Option<(Vec<u8>, String)>,
}

/// Result of dispatching one [`Control`].
///
/// Most controls produce zero or more [`Block`]s ([`ControlOutcome::Blocks`]).
/// The two layout-defining controls produce *markers* the orchestrator (T8)
/// consumes for section / column-thread handling instead of body content.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ControlOutcome {
    /// Ordinary content blocks to splice into the current section in order.
    /// May be empty (control fully skipped / loss-recorded only).
    Blocks(Vec<Block>),
    /// `SectionDef` — start of a new page-layout region.  T8 decides whether to
    /// open a new IR `Section`.  Geometry/extras are deliberately not carried
    /// here; T8 reads them from the rhwp tree directly if needed.
    SectionMarker,
    /// `ColumnDef` — multi-column / single-column boundary.  `count` is the
    /// declared column count.  T8 maps this onto `<thread>` continuity.
    ColumnMarker { count: u16 },
}

impl ControlOutcome {
    /// Convenience constructor for a single block.
    fn one(block: Block) -> Self {
        ControlOutcome::Blocks(vec![block])
    }

    /// Convenience constructor for "nothing emitted".
    fn empty() -> Self {
        ControlOutcome::Blocks(Vec::new())
    }
}

/// Dispatch a single rhwp [`Control`] to its IR representation.
///
/// `footnote_number` is the 1-based sequential number the orchestrator assigns
/// to the next footnote/endnote (HWP's own `number` field is not always
/// reliable for DocLang's sequential model); it is used only for
/// `Footnote`/`Endnote` variants.
pub(crate) fn convert_control(
    ctrl: &Control,
    ctx: &ControlCtx<'_>,
    footnote_number: usize,
    loss: &mut LossReport,
) -> ControlOutcome {
    match ctrl {
        // ---- Block-level typed content -----------------------------------
        Control::Table(t) => ControlOutcome::one(Block::Table((ctx.convert_table)(t.as_ref()))),

        Control::Picture(p) => {
            convert_picture(p.as_ref().image_attr.bin_data_id, &p.common, ctx, loss)
        }

        Control::Equation(e) => ControlOutcome::one(Block::Formula(Formula {
            raw_eqedit: e.script.clone(),
            // LaTeX conversion is wired in T13 (eqedit module); not here.
            latex: None,
            // Provenance is stamped by the body driver (paragraph.rs) which
            // knows the (section, para, control) position.
            prov: None,
        })),

        Control::Footnote(f) => ControlOutcome::one(Block::Footnote {
            number: footnote_number,
            content: (ctx.convert_paragraphs)(&f.paragraphs),
            prov: None,
        }),

        Control::Endnote(e) => {
            // DocLang v0.6 has no <endnote> element — map to a footnote so the
            // content survives, but record the demotion so lean-mode audits and
            // a future preserve path can recover the distinction.
            loss.push(LossEntry {
                kind: LossKind::Other("endnote".to_string()),
                location: ctx.location.to_string(),
                detail: "endnote mapped to footnote (no DocLang endnote element)".to_string(),
            });
            ControlOutcome::one(Block::Footnote {
                number: footnote_number,
                content: (ctx.convert_paragraphs)(&e.paragraphs),
                prov: None,
            })
        }

        Control::Header(h) => ControlOutcome::one(Block::PageHeader {
            content: (ctx.convert_paragraphs)(&h.paragraphs),
            apply: map_apply(h.apply_to),
            prov: None,
        }),

        Control::Footer(f) => ControlOutcome::one(Block::PageFooter {
            content: (ctx.convert_paragraphs)(&f.paragraphs),
            apply: map_apply(f.apply_to),
            prov: None,
        }),

        // ---- Shapes / text-box-like --------------------------------------
        Control::Shape(s) => convert_shape(s.as_ref(), ctx, loss),

        // ---- Layout markers handed to the orchestrator (T8) --------------
        Control::SectionDef(_) => ControlOutcome::SectionMarker,
        Control::ColumnDef(c) => ControlOutcome::ColumnMarker {
            count: c.column_count,
        },

        // ---- Body-carrying auxiliary content -----------------------------
        Control::HiddenComment(h) => {
            // Hidden comment carries paragraphs; rescue the text so it is not
            // silently dropped, and record the loss of its "hidden" semantics.
            loss.push(LossEntry {
                kind: LossKind::Other("hidden comment".to_string()),
                location: ctx.location.to_string(),
                detail: "hidden comment text rescued as plain blocks".to_string(),
            });
            ControlOutcome::Blocks((ctx.convert_paragraphs)(&h.paragraphs))
        }

        // ---- Loss-recorded metadata controls -----------------------------
        Control::Hyperlink(h) => {
            // This variant is the HWP3 path only: the display text lives in the
            // control (not the paragraph body run) and rhwp does not yet extract
            // the URL (it sets `url = ""`). HWP5/HWPX hyperlinks instead arrive as
            // `Control::Field` and keep their anchor text in the body run.
            //
            // HWP5/HWPX hyperlinks are emitted as inline `<href>` (via the
            // paragraph's field_ranges in inline.rs), but this HWP3 path has no
            // extracted URL to anchor one. For now rescue the display text as a
            // plain block so it is not silently dropped, and record the loss of
            // the link semantics / URL.
            loss.push(LossEntry {
                kind: LossKind::Other("hyperlink".to_string()),
                location: ctx.location.to_string(),
                detail: format!(
                    "hyperlink url={:?} text={:?} (anchor text rescued, link semantics dropped)",
                    h.url, h.text
                ),
            });
            ControlOutcome::Blocks(rescue_text_block(&h.text))
        }

        Control::Bookmark(b) => {
            loss.push(LossEntry {
                kind: LossKind::Other("bookmark".to_string()),
                location: ctx.location.to_string(),
                detail: format!("bookmark: {}", b.name),
            });
            ControlOutcome::empty()
        }

        Control::Ruby(r) => {
            // 덧말: an annotation rendered above its base text (the base text is in
            // the body run; this carries only the annotation). Rescue the
            // annotation text so it is not dropped; record the loss of its ruby
            // positioning/association.
            loss.push(LossEntry {
                kind: LossKind::Other("ruby".to_string()),
                location: ctx.location.to_string(),
                detail: format!("ruby (덧말): {} (annotation text rescued)", r.ruby_text),
            });
            ControlOutcome::Blocks(rescue_text_block(&r.ruby_text))
        }

        Control::CharOverlap(c) => {
            // 글자겹침: the overlapped glyphs are stored in the control, not the
            // body run. Rescue them as plain text so the characters survive;
            // record the loss of the overlap rendering.
            let overlapped: String = c.chars.iter().collect();
            loss.push(LossEntry {
                kind: LossKind::Other("char overlap".to_string()),
                location: ctx.location.to_string(),
                detail: format!("char overlap (글자겹침): {} (text rescued)", overlapped),
            });
            ControlOutcome::Blocks(rescue_text_block(&overlapped))
        }

        Control::Field(f) => {
            // Hyperlink fields are now represented inline as <href> (resolved via
            // the paragraph's field_ranges in inline.rs), so recording a
            // block-level loss for them would be misleading. Other field kinds
            // have no inline representation yet and are still recorded.
            if f.field_type != FieldType::Hyperlink {
                loss.push(LossEntry {
                    kind: LossKind::Other("field".to_string()),
                    location: ctx.location.to_string(),
                    detail: format!("field type={} command={:?}", f.field_type_str(), f.command),
                });
            }
            ControlOutcome::empty()
        }

        Control::Form(f) => {
            loss.push(LossEntry {
                kind: LossKind::Other("form".to_string()),
                location: ctx.location.to_string(),
                detail: format!("form object name={:?}", f.name),
            });
            ControlOutcome::empty()
        }

        Control::Unknown(u) => {
            loss.push(LossEntry {
                kind: LossKind::Other("unknown control".to_string()),
                location: ctx.location.to_string(),
                detail: format!("unknown control ctrl_id=0x{:08x}", u.ctrl_id),
            });
            ControlOutcome::empty()
        }

        // ---- Page-numbering chrome: no body content, silently skipped ----
        // These describe automatic numbering / page-number placement that
        // DocLang regenerates on render; there is nothing meaningful to carry.
        Control::AutoNumber(_)
        | Control::NewNumber(_)
        | Control::PageNumberPos(_)
        | Control::PageHide(_) => ControlOutcome::empty(),
    }
}

/// Map a picture's bin-data reference to a `Block::Picture`, or record a loss
/// and skip when the embedded bytes are absent.
fn convert_picture(
    bin_data_id: u16,
    common: &CommonObjAttr,
    ctx: &ControlCtx<'_>,
    loss: &mut LossReport,
) -> ControlOutcome {
    // NOTE: the lookup is delegated to `ctx.resolve_picture` (T8 wires it to
    // `resources::bin_data_bytes`). A prior agent confirmed
    // `BinDataContent.id == storage_id` directly, so that helper is a single
    // linear scan rather than a two-table hop (see resources.rs docs).
    match (ctx.resolve_picture)(bin_data_id) {
        Some((data, extension)) => ControlOutcome::one(Block::Picture {
            data,
            extension,
            geometry: Some(geometry::from_common_attr(common)),
            lost: None,
            prov: None,
        }),
        None => {
            loss.push(LossEntry {
                kind: LossKind::FloatingObject,
                location: ctx.location.to_string(),
                detail: format!("picture skipped: bin-data id {} not found", bin_data_id),
            });
            ControlOutcome::empty()
        }
    }
}

/// Lower a drawing `Shape`.
///
/// Lean-mode **text-rescue policy**: shapes (text boxes, grouped objects) have
/// no DocLang counterpart, so the object itself is always loss-recorded.  But
/// if the shape carries text paragraphs (a 글상자 / text box, or a group that
/// contains one), we extract that text via the paragraph callback and return it
/// as plain blocks so the words are not silently dropped.  In preserve mode we
/// additionally attach a `Block::Custom` carrying the geometry so a round-trip
/// tool can rebuild the floating object.
fn convert_shape(
    shape: &ShapeObject,
    ctx: &ControlCtx<'_>,
    loss: &mut LossReport,
) -> ControlOutcome {
    let rescued = rescue_shape_paragraphs(shape);
    let has_text = !rescued.is_empty();

    loss.push(LossEntry {
        kind: if has_text {
            LossKind::TextBox
        } else {
            LossKind::FloatingObject
        },
        location: ctx.location.to_string(),
        detail: format!(
            "{} ({})",
            shape.shape_name(),
            if has_text { "text rescued" } else { "no text" },
        ),
    });

    let mut blocks = (ctx.convert_paragraphs)(&rescued);

    if ctx.mode == Mode::Preserve {
        // Serialise geometry as a simple key=value payload string so the
        // preserve-mode writer can emit it inside <custom ns="hwp:geometry">.
        let geo = geometry::from_common_attr(shape.common());
        let payload = format!(
            "shape={};width={};height={};h_offset={};v_offset={};treat_as_char={}",
            shape.shape_name(),
            geo.width,
            geo.height,
            geo.h_offset,
            geo.v_offset,
            geo.treat_as_char,
        );
        blocks.push(Block::Custom {
            namespace: "hwp:geometry".to_string(),
            payload,
        });
    }

    ControlOutcome::Blocks(blocks)
}

/// Collect any paragraphs carried by a shape (text box) or its grouped
/// children, recursively.  Returns an empty vec for purely graphical shapes.
fn rescue_shape_paragraphs(shape: &ShapeObject) -> Vec<Paragraph> {
    let mut out = Vec::new();
    // A text box stores its body in DrawingObjAttr.text_box.
    if let Some(drawing) = shape.drawing() {
        if let Some(tb) = &drawing.text_box {
            out.extend(tb.paragraphs.iter().cloned());
        }
    }
    // Grouped objects may contain text-bearing children.
    if let ShapeObject::Group(g) = shape {
        for child in &g.children {
            out.extend(rescue_shape_paragraphs(child));
        }
    }
    out
}

/// Wrap a rescued plain-text string in a single unformatted [`Block::Paragraph`].
///
/// Used by controls whose text content has no DocLang counterpart (HWP3
/// hyperlink display text, ruby annotations, overlapped glyphs): the text is
/// preserved as a plain block rather than dropped. Whitespace-only / empty input
/// yields no block so we never emit a blank paragraph.
fn rescue_text_block(text: &str) -> Vec<Block> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Vec::new()
    } else {
        vec![Block::Paragraph {
            content: vec![ir::Inline::Text(trimmed.to_string())],
            lost: None,
            prov: None,
        }]
    }
}

/// Map rhwp [`HeaderFooterApply`] onto the IR [`ir::HeaderFooterApply`].
///
/// HWP distinguishes Both/Even/Odd; DocLang additionally has `First`, which HWP
/// expresses through master pages rather than this enum, so it is unused here.
fn map_apply(apply: HeaderFooterApply) -> ir::HeaderFooterApply {
    match apply {
        HeaderFooterApply::Both => ir::HeaderFooterApply::All,
        HeaderFooterApply::Even => ir::HeaderFooterApply::Even,
        HeaderFooterApply::Odd => ir::HeaderFooterApply::Odd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::{Bookmark, CharOverlap, Equation, Hyperlink, Ruby};
    use crate::model::footnote::{Endnote, Footnote};
    use crate::model::header_footer::Header;
    use crate::model::image::Picture;
    use crate::model::paragraph::Paragraph;
    use crate::model::shape::{DrawingObjAttr, RectangleShape, TextBox};

    /// Build a `ControlCtx` whose paragraph callback emits one empty paragraph
    /// block per input paragraph (so recursion is observable) and whose table
    /// callback yields a 1x1 table.
    fn stub_ctx<'a>(
        mode: Mode,
        convert_paragraphs: &'a dyn Fn(&[Paragraph]) -> Vec<Block>,
        convert_table: &'a dyn Fn(&Table) -> ir::Table,
        resolve_picture: &'a dyn Fn(u16) -> Option<(Vec<u8>, String)>,
    ) -> ControlCtx<'a> {
        ControlCtx {
            mode,
            location: "test/loc",
            convert_paragraphs,
            convert_table,
            resolve_picture,
        }
    }

    /// A `resolve_picture` closure that never finds anything.
    fn no_picture(_id: u16) -> Option<(Vec<u8>, String)> {
        None
    }

    fn count_paragraphs(ps: &[Paragraph]) -> Vec<Block> {
        ps.iter()
            .map(|_| Block::Paragraph {
                content: Vec::new(),
                lost: None,
                prov: None,
            })
            .collect()
    }

    fn dummy_table(_t: &Table) -> ir::Table {
        ir::Table {
            rows: 1,
            cols: 1,
            cells: Vec::new(),
            caption: None,
            prov: None,
        }
    }

    #[test]
    fn equation_script_passthrough() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let eq = Equation {
            script: "1 over 2".to_string(),
            ..Default::default()
        };
        let out = convert_control(&Control::Equation(Box::new(eq)), &ctx, 1, &mut loss);
        match out {
            ControlOutcome::Blocks(blocks) => match &blocks[0] {
                Block::Formula(f) => {
                    assert_eq!(f.raw_eqedit, "1 over 2");
                    assert_eq!(f.latex, None);
                }
                other => panic!("expected Formula, got {other:?}"),
            },
            other => panic!("expected Blocks, got {other:?}"),
        }
        assert!(loss.is_empty());
    }

    #[test]
    fn picture_happy_path() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        // A resolver that returns PNG bytes for id 7 only.
        let resolve = |id: u16| -> Option<(Vec<u8>, String)> {
            if id == 7 {
                Some((vec![0x89, 0x50], "png".to_string()))
            } else {
                None
            }
        };
        // Mode::Preserve so geometry is attached.
        let ctx = stub_ctx(Mode::Preserve, &cp, &ct, &resolve);
        let mut loss = LossReport::new();
        let mut pic = Picture::default();
        pic.image_attr.bin_data_id = 7;
        let out = convert_control(&Control::Picture(Box::new(pic)), &ctx, 1, &mut loss);
        match out {
            ControlOutcome::Blocks(blocks) => match &blocks[0] {
                Block::Picture {
                    data,
                    extension,
                    geometry,
                    ..
                } => {
                    assert_eq!(data, &vec![0x89, 0x50]);
                    assert_eq!(extension, "png");
                    assert!(geometry.is_some());
                }
                other => panic!("expected Picture, got {other:?}"),
            },
            other => panic!("expected Blocks, got {other:?}"),
        }
        assert!(loss.is_empty());
    }

    #[test]
    fn picture_missing_bindata_records_loss_and_skips() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let mut pic = Picture::default();
        pic.image_attr.bin_data_id = 99; // resolver returns None
        let out = convert_control(&Control::Picture(Box::new(pic)), &ctx, 1, &mut loss);
        assert_eq!(out, ControlOutcome::Blocks(Vec::new()));
        assert_eq!(loss.len(), 1);
        assert_eq!(loss.iter().next().unwrap().kind, LossKind::FloatingObject);
    }

    #[test]
    fn footnote_recurses_via_callback() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let fnote = Footnote {
            number: 3,
            paragraphs: vec![Paragraph::default(), Paragraph::default()],
            ..Default::default()
        };
        let out = convert_control(&Control::Footnote(Box::new(fnote)), &ctx, 5, &mut loss);
        match out {
            ControlOutcome::Blocks(blocks) => match &blocks[0] {
                Block::Footnote {
                    number, content, ..
                } => {
                    assert_eq!(*number, 5); // sequential number, not HWP's 3
                    assert_eq!(content.len(), 2); // both paragraphs lowered
                }
                other => panic!("expected Footnote, got {other:?}"),
            },
            other => panic!("expected Blocks, got {other:?}"),
        }
        assert!(loss.is_empty());
    }

    #[test]
    fn endnote_records_loss_entry() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let en = Endnote {
            number: 1,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        };
        let out = convert_control(&Control::Endnote(Box::new(en)), &ctx, 2, &mut loss);
        match out {
            ControlOutcome::Blocks(blocks) => {
                assert!(matches!(blocks[0], Block::Footnote { .. }));
            }
            other => panic!("expected Blocks, got {other:?}"),
        }
        assert_eq!(loss.len(), 1);
        assert_eq!(
            loss.iter().next().unwrap().kind,
            LossKind::Other("endnote".to_string())
        );
    }

    #[test]
    fn header_apply_mapping() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let header = Header {
            apply_to: HeaderFooterApply::Odd,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        };
        let out = convert_control(&Control::Header(Box::new(header)), &ctx, 1, &mut loss);
        match out {
            ControlOutcome::Blocks(blocks) => match &blocks[0] {
                Block::PageHeader { apply, content, .. } => {
                    assert_eq!(*apply, ir::HeaderFooterApply::Odd);
                    assert_eq!(content.len(), 1);
                }
                other => panic!("expected PageHeader, got {other:?}"),
            },
            other => panic!("expected Blocks, got {other:?}"),
        }
    }

    #[test]
    fn shape_text_rescue() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        // A rectangle whose drawing attrs carry a text box with two paragraphs.
        let rect = RectangleShape {
            drawing: DrawingObjAttr {
                text_box: Some(TextBox {
                    paragraphs: vec![Paragraph::default(), Paragraph::default()],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let shape = ShapeObject::Rectangle(rect);
        let out = convert_control(&Control::Shape(Box::new(shape)), &ctx, 1, &mut loss);
        match out {
            ControlOutcome::Blocks(blocks) => {
                // Lean mode: only the two rescued paragraphs, no Custom block.
                assert_eq!(blocks.len(), 2);
            }
            other => panic!("expected Blocks, got {other:?}"),
        }
        assert_eq!(loss.len(), 1);
        assert_eq!(loss.iter().next().unwrap().kind, LossKind::TextBox);
    }

    #[test]
    fn shape_preserve_appends_custom_geometry() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Preserve, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let rect = RectangleShape::default(); // no text box → graphical only
        let shape = ShapeObject::Rectangle(rect);
        let out = convert_control(&Control::Shape(Box::new(shape)), &ctx, 1, &mut loss);
        match out {
            ControlOutcome::Blocks(blocks) => {
                assert_eq!(blocks.len(), 1);
                match &blocks[0] {
                    Block::Custom { namespace, .. } => assert_eq!(namespace, "hwp:geometry"),
                    other => panic!("expected Custom, got {other:?}"),
                }
            }
            other => panic!("expected Blocks, got {other:?}"),
        }
        assert_eq!(loss.iter().next().unwrap().kind, LossKind::FloatingObject);
    }

    #[test]
    fn column_def_yields_marker() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let cd = crate::model::page::ColumnDef {
            column_count: 3,
            ..Default::default()
        };
        let out = convert_control(&Control::ColumnDef(cd), &ctx, 1, &mut loss);
        assert_eq!(out, ControlOutcome::ColumnMarker { count: 3 });
    }

    #[test]
    fn bookmark_records_loss() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let out = convert_control(
            &Control::Bookmark(Bookmark {
                name: "ch1".to_string(),
            }),
            &ctx,
            1,
            &mut loss,
        );
        assert_eq!(out, ControlOutcome::empty());
        assert_eq!(loss.len(), 1);
    }

    #[test]
    fn hyperlink_rescues_text_and_records_loss() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let out = convert_control(
            &Control::Hyperlink(Hyperlink {
                url: "https://x".to_string(),
                text: "Example".to_string(),
            }),
            &ctx,
            1,
            &mut loss,
        );
        // Display text survives as a plain paragraph; link semantics are loss.
        assert_eq!(
            out,
            ControlOutcome::Blocks(vec![Block::Paragraph {
                content: vec![ir::Inline::Text("Example".to_string())],
                lost: None,
                prov: None,
            }])
        );
        assert_eq!(loss.len(), 1);
    }

    #[test]
    fn hyperlink_empty_text_emits_no_block() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let out = convert_control(
            &Control::Hyperlink(Hyperlink {
                url: "https://x".to_string(),
                text: "   ".to_string(),
            }),
            &ctx,
            1,
            &mut loss,
        );
        // No display text → no rescued block, but the loss is still recorded.
        assert_eq!(out, ControlOutcome::empty());
        assert_eq!(loss.len(), 1);
    }

    #[test]
    fn ruby_rescues_annotation_and_records_loss() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let out = convert_control(
            &Control::Ruby(Ruby {
                ruby_text: "가나다".to_string(),
                ..Default::default()
            }),
            &ctx,
            1,
            &mut loss,
        );
        assert_eq!(
            out,
            ControlOutcome::Blocks(vec![Block::Paragraph {
                content: vec![ir::Inline::Text("가나다".to_string())],
                lost: None,
                prov: None,
            }])
        );
        assert_eq!(loss.len(), 1);
    }

    #[test]
    fn char_overlap_rescues_glyphs_and_records_loss() {
        let cp = count_paragraphs;
        let ct = dummy_table;
        let np = no_picture;
        let ctx = stub_ctx(Mode::Lean, &cp, &ct, &np);
        let mut loss = LossReport::new();
        let out = convert_control(
            &Control::CharOverlap(CharOverlap {
                chars: vec!['企', '業'],
                ..Default::default()
            }),
            &ctx,
            1,
            &mut loss,
        );
        assert_eq!(
            out,
            ControlOutcome::Blocks(vec![Block::Paragraph {
                content: vec![ir::Inline::Text("企業".to_string())],
                lost: None,
                prov: None,
            }])
        );
        assert_eq!(loss.len(), 1);
    }
}
