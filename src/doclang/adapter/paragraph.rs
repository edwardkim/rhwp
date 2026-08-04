//! Paragraph / heading / list mapping (Phase A — block).
//!
//! Lowers a single rhwp [`Paragraph`] into a sequence of crate-IR [`Block`]s.
//! The paragraph's [`ParaShape`] decides whether the text becomes a
//! [`Block::Heading`], a list item (a single-item [`Block::List`] that the
//! section assembler later merges with its neighbours), or an ordinary
//! [`Block::Paragraph`].  Inline content comes from [`extract_inlines`]; inline
//! *objects* (tables, pictures, equations, footnotes, …) are lowered from
//! `Paragraph.controls` via [`control::convert_control`].
//!
//! # List grouping
//!
//! HWP has no "list" container: each numbered / bulleted paragraph carries a
//! `head_type` of `Number` / `Bullet` independently.  DocLang, by contrast,
//! wraps consecutive items in one `<list>`.  Bridging this is a two-step design:
//!
//! 1. [`convert_paragraph`] emits a **single-item** `Block::List` for each
//!    list-item paragraph (`ordered = true` for `Number`, `false` for
//!    `Bullet`).  It does not look at neighbouring paragraphs.
//! 2. [`convert_paragraphs`] (the section-level driver, also the callback handed
//!    to `table.rs` / `control.rs` by T8) walks the resulting block stream and
//!    coalesces *runs* of adjacent single-item lists that share the same
//!    `ordered` flag into one `Block::List`.  Any non-list block (or a list of
//!    the opposite kind) closes the current run.
//!
//! Keeping the grouping in the driver — rather than threading look-behind state
//! through `convert_paragraph` — means cell / footnote / header bodies (which
//! also flow through `convert_paragraphs`) get list grouping for free.
//!
//! # In-text object positioning (v1 simplification)
//!
//! rhwp exposes `field_ranges` mapping text spans to `controls[]`, so objects
//! could be spliced at their exact in-text offset.  v1 takes the pragmatic
//! route: the paragraph's text block is emitted first, then every block-level
//! object from `controls[]` is appended in `controls` order.  This preserves
//! content and ordering between objects but not the precise interleaving of an
//! object that sits *inside* a line of text.
//!
//! TODO(v2): use `field_ranges` / `control_text_positions()` to split the text
//! block and interleave objects at their true character offset.
//!
//! # Shared loss accumulation
//!
//! The control callbacks (`convert_paragraphs`, `convert_table`) and
//! `convert_control` all need to append to the *same* [`LossReport`].  Rust will
//! not let three closures plus the dispatch loop hold `&mut LossReport`
//! simultaneously, so the report is wrapped in a [`RefCell`] for the duration of
//! the control walk; each borrow is short-lived and non-overlapping.

use std::cell::RefCell;

use crate::model::document::{DocInfo, Document};
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::style::HeadType;

use crate::doclang::ir::prov::Prov;
use crate::doclang::ir::{Block, ListItem};
use crate::doclang::loss::report::{LossEntry, LossKind, LossReport};
use crate::doclang::options::Mode;

use super::control::{self, ControlCtx, ControlOutcome};
use super::resources;

/// Lower one rhwp [`Paragraph`] into zero or more IR [`Block`]s.
///
/// The returned vector is, in order:
/// 1. an optional [`Block::PageBreak`] when the paragraph starts a new page
///    (`column_type == Page`);
/// 2. the paragraph's primary block (heading / single-item list / paragraph),
///    emitted only when it has inline content;
/// 3. block-level objects from `controls[]`, in control order (see the
///    module-level "in-text object positioning" note).
///
/// `document` resolves picture binary data; `doc_info` resolves char/para
/// shapes.  `mode` drives control-dispatch policy (lean vs preserve).
/// `footnote_counter` is the running 1-based footnote number, advanced for each
/// footnote/endnote emitted.  `location` is a human-readable source path
/// (e.g. `"s0/p3"`) for loss attribution.
pub(crate) fn convert_paragraph(
    para: &Paragraph,
    document: &Document,
    doc_info: &DocInfo,
    mode: Mode,
    footnote_counter: &mut usize,
    location: &str,
    loss: &mut LossReport,
) -> Vec<Block> {
    convert_paragraph_prov(
        para,
        document,
        doc_info,
        mode,
        footnote_counter,
        location,
        loss,
        None,
    )
}

/// Like [`convert_paragraph`], but additionally stamps v2 `<location>`
/// provenance when `body_pos = Some((section, para))` is supplied. The body
/// driver passes the model `(section, para)` indices so the produced blocks can
/// later be joined to render-tree bounding boxes; nested callers (cells,
/// footnotes, headers — whose paragraph indices are *cell-local* and carry no
/// global provenance) pass `None`, leaving every block's `prov` as `None`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn convert_paragraph_prov(
    para: &Paragraph,
    document: &Document,
    doc_info: &DocInfo,
    mode: Mode,
    footnote_counter: &mut usize,
    location: &str,
    loss: &mut LossReport,
    body_pos: Option<(usize, usize)>,
) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();

    // Text-block provenance (no owning control) for the primary block.
    let text_prov = body_pos.map(|(s, p)| Prov::text(s, p));

    // (1) A paragraph flagged as a page break opens a fresh page before its
    // content.  Other column-break kinds (Section / MultiColumn / Column) are
    // layout markers handled at the section level, not here.
    if para.column_type == ColumnBreakType::Page {
        blocks.push(Block::PageBreak);
    }

    // (2)+(3) Primary text and the paragraph's block-level objects.
    //
    // Ordinary paragraphs interleave text and objects at the objects' true
    // in-text character positions (v2). Headings and list items keep the simpler
    // "text first, then trailing objects" layout: splitting a heading or list
    // item across an embedded object is not meaningful.
    let head_type = resources::para_shape(doc_info, para.para_shape_id).map(|ps| ps.head_type);
    match head_type {
        None | Some(HeadType::None) => {
            interleave_ordinary(
                para,
                document,
                doc_info,
                mode,
                footnote_counter,
                location,
                loss,
                &mut blocks,
                body_pos,
                text_prov,
            );
        }
        Some(ht) => {
            let inlines = super::inline::extract_inlines(para, doc_info, location, loss);
            if !inlines.is_empty() {
                let block = match ht {
                    HeadType::Outline => {
                        // para_level is 0-based; DocLang headings are 1..=6. Levels
                        // deeper than 6 are clamped — record the loss so deep
                        // hierarchies are not silently flattened without an audit trail.
                        let para_level = resources::para_shape(doc_info, para.para_shape_id)
                            .map(|ps| ps.para_level)
                            .unwrap_or(0);
                        let raw_level = para_level as u16 + 1;
                        let level = raw_level.clamp(1, 6) as u8;
                        if raw_level > 6 {
                            loss.push(LossEntry {
                                kind: LossKind::Other("heading-level-clamped".to_string()),
                                location: location.to_string(),
                                detail: format!(
                                    "outline level {} clamped to {} (DocLang headings are 1–6)",
                                    raw_level, level
                                ),
                            });
                        }
                        Block::Heading {
                            level,
                            content: inlines,
                            lost: None,
                            prov: text_prov,
                        }
                    }
                    HeadType::Number => single_item_list(true, inlines, text_prov),
                    HeadType::Bullet => single_item_list(false, inlines, text_prov),
                    HeadType::None => unreachable!("None handled in the other match arm"),
                };
                blocks.push(block);
            }
            if !para.controls.is_empty() {
                for group in build_control_groups(
                    para,
                    document,
                    doc_info,
                    mode,
                    footnote_counter,
                    location,
                    loss,
                    body_pos,
                ) {
                    blocks.extend(group);
                }
            }
        }
    }

    blocks
}

/// Lower an ordinary (non-heading, non-list) paragraph, interleaving its text
/// with its block-level objects at each object's true in-text character
/// position (from [`Paragraph::control_text_positions`]).
///
/// When every object sits at the end of the text (the common case) this produces
/// the same "text then objects" output as the heading/list path.
#[allow(clippy::too_many_arguments)]
fn interleave_ordinary(
    para: &Paragraph,
    document: &Document,
    doc_info: &DocInfo,
    mode: Mode,
    footnote_counter: &mut usize,
    location: &str,
    loss: &mut LossReport,
    blocks: &mut Vec<Block>,
    body_pos: Option<(usize, usize)>,
    text_prov: Option<Prov>,
) {
    let char_len = para.text.chars().count();

    // Per-control block groups (control order), prov-stamped + footnote-advanced.
    let groups = if para.controls.is_empty() {
        Vec::new()
    } else {
        build_control_groups(
            para,
            document,
            doc_info,
            mode,
            footnote_counter,
            location,
            loss,
            body_pos,
        )
    };

    // Only objects that actually produced blocks get an in-text slot; markers and
    // skipped controls (empty groups) must not split the text.
    let positions = if para.controls.is_empty() {
        Vec::new()
    } else {
        para.control_text_positions()
    };

    // Only true in-text *flow* objects (tables, pictures, formulas) are spliced
    // at their character position. Page furniture (headers/footers), notes
    // (footnotes/endnotes), rescued text boxes, etc. are not part of the text
    // flow, so they keep the trailing "after the paragraph" placement.
    let mut placed: Vec<(usize, Vec<Block>)> = Vec::new();
    let mut trailing: Vec<Block> = Vec::new();
    for (ci, g) in groups.into_iter().enumerate() {
        if g.is_empty() {
            continue;
        }
        if group_is_flow(&g) {
            let pos = positions.get(ci).copied().unwrap_or(char_len).min(char_len);
            placed.push((pos, g));
        } else {
            trailing.extend(g);
        }
    }
    // Stable sort keeps control order within the same position.
    placed.sort_by_key(|(p, _)| *p);

    let mut cuts: Vec<usize> = placed.iter().map(|(p, _)| *p).collect();
    cuts.dedup();

    let segments = super::inline::extract_inline_segments(para, doc_info, location, loss, &cuts);

    let mut pi = 0usize;
    for (k, seg) in segments.into_iter().enumerate() {
        if !seg.is_empty() {
            blocks.push(Block::Paragraph {
                content: seg,
                lost: None,
                prov: text_prov,
            });
        }
        if k < cuts.len() {
            let pos = cuts[k];
            while pi < placed.len() && placed[pi].0 == pos {
                blocks.append(&mut placed[pi].1);
                pi += 1;
            }
        }
    }
    // Safety net: never drop object content if some slot did not match a cut.
    while pi < placed.len() {
        blocks.append(&mut placed[pi].1);
        pi += 1;
    }

    // Non-flow objects trail the paragraph text, in control order.
    blocks.append(&mut trailing);
}

/// True if a control's block group is an in-text *flow* object (table, picture,
/// or formula) that should be spliced at its character position rather than
/// trailing the paragraph.
fn group_is_flow(group: &[Block]) -> bool {
    !group.is_empty()
        && group.iter().all(|b| {
            matches!(
                b,
                Block::Table(_) | Block::Picture { .. } | Block::Formula(_)
            )
        })
}

/// Lower a paragraph's `controls[]` into block-level objects, returning one
/// block group per control (in control order). Markers and skipped controls
/// yield an empty group, so the returned vector is index-aligned with
/// `para.controls` — callers can pair each group with its
/// [`Paragraph::control_text_positions`] entry.
///
/// The control callbacks (`convert_paragraphs`, `convert_table`) and
/// `convert_control` all need to write losses.  Rather than thread one
/// `&mut LossReport` through re-entrant callbacks (which Rust's borrow checker
/// forbids, and which would deadlock a `RefCell`), each callback and each
/// dispatch builds its own scratch [`LossReport`] and merges it into the shared
/// cell once the call returns — so no borrow is ever held across recursion.
#[allow(clippy::too_many_arguments)]
fn build_control_groups(
    para: &Paragraph,
    document: &Document,
    doc_info: &DocInfo,
    mode: Mode,
    footnote_counter: &mut usize,
    location: &str,
    loss: &mut LossReport,
    body_pos: Option<(usize, usize)>,
) -> Vec<Vec<Block>> {
    let loss_cell = RefCell::new(loss);

    let convert_paras = |paras: &[Paragraph]| -> Vec<Block> {
        let mut scratch = LossReport::new();
        let out = convert_paragraphs(paras, document, doc_info, mode, location, &mut scratch);
        loss_cell.borrow_mut().merge(scratch);
        out
    };
    let convert_tbl = |t: &crate::model::table::Table| {
        let mut scratch = LossReport::new();
        let out = super::table::convert_table(t, doc_info, location, &convert_paras, &mut scratch);
        loss_cell.borrow_mut().merge(scratch);
        out
    };
    let resolve_pic = |bin_data_id: u16| -> Option<(Vec<u8>, String)> {
        resources::bin_data_bytes(document, bin_data_id)
            .map(|(bytes, ext)| (bytes, ext.to_string()))
    };

    let ctx = ControlCtx {
        mode,
        location,
        convert_paragraphs: &convert_paras,
        convert_table: &convert_tbl,
        resolve_picture: &resolve_pic,
    };

    let mut groups: Vec<Vec<Block>> = Vec::with_capacity(para.controls.len());
    for (ci, ctrl) in para.controls.iter().enumerate() {
        let mut scratch = LossReport::new();
        let outcome = control::convert_control(ctrl, &ctx, *footnote_counter, &mut scratch);
        loss_cell.borrow_mut().merge(scratch);
        match outcome {
            ControlOutcome::Blocks(mut bs) => {
                // Advance the footnote counter for each footnote/endnote we
                // actually emitted so numbering stays sequential.
                bump_footnote_counter(&bs, footnote_counter);
                // Stamp v2 location provenance on the object blocks this control
                // produced: a control maps to `controls[ci]` of the owning
                // body paragraph `(section, para)`. Only meaningful on the body
                // path (`body_pos = Some`); nested controls leave `prov = None`.
                if let Some((section, para_idx)) = body_pos {
                    let prov = Prov::object(section, para_idx, ci);
                    for b in &mut bs {
                        stamp_object_prov(b, prov);
                    }
                }
                groups.push(bs);
            }
            // Layout markers carry no body content; the orchestrator (T8)
            // handles section / thread semantics from the rhwp tree. They still
            // occupy a control slot, so push an empty group to keep alignment.
            ControlOutcome::SectionMarker | ControlOutcome::ColumnMarker { .. } => {
                groups.push(Vec::new());
            }
        }
    }
    groups
}

/// Lower a slice of paragraphs into block content, applying list grouping.
///
/// This is the section-level driver and the callback handed to `table.rs` /
/// `control.rs` for cell / footnote / header bodies.  It lowers each paragraph
/// via [`convert_paragraph`], then merges adjacent single-item lists of the same
/// kind into combined `Block::List`s (see the module-level "list grouping" note).
///
/// A fresh footnote counter starts at 1 for each call; callers that need a
/// document-wide sequence drive [`convert_paragraph`] directly (see T8).
pub(crate) fn convert_paragraphs(
    paras: &[Paragraph],
    document: &Document,
    doc_info: &DocInfo,
    mode: Mode,
    location: &str,
    loss: &mut LossReport,
) -> Vec<Block> {
    let mut footnote_counter = 1usize;
    let mut raw: Vec<Block> = Vec::new();
    for (i, para) in paras.iter().enumerate() {
        let para_loc = format!("{}/p{}", location, i);
        raw.extend(convert_paragraph(
            para,
            document,
            doc_info,
            mode,
            &mut footnote_counter,
            &para_loc,
            loss,
        ));
    }
    group_lists(raw)
}

/// Build a single-item `Block::List` wrapping the given inline content.
///
/// `prov` is the model provenance of the originating paragraph; it is carried
/// on the `List` block so the section-level grouping can anchor the merged
/// list at its first item (see [`group_lists`]). The inner paragraph keeps the
/// same provenance so a cell/footnote re-grouping path stays consistent.
fn single_item_list(
    ordered: bool,
    content: Vec<crate::doclang::ir::Inline>,
    prov: Option<Prov>,
) -> Block {
    Block::List {
        ordered,
        items: vec![ListItem {
            content: vec![Block::Paragraph {
                content,
                lost: None,
                prov,
            }],
        }],
        lost: None,
        prov,
    }
}

/// Stamp v2 location provenance onto an object block produced by a control.
///
/// Only the block kinds that can carry a `<location>` and that originate
/// directly from a paragraph control are stamped: tables, pictures, formulas,
/// footnotes, and page headers/footers. Text-rescue paragraphs (from shapes /
/// hidden comments) and `Custom` blocks keep `prov = None` — their geometry has
/// no stable provenance (textbox-internal content is skipped for v2).
fn stamp_object_prov(block: &mut Block, prov: Prov) {
    match block {
        Block::Table(t) => t.prov = Some(prov),
        Block::Formula(f) => f.prov = Some(prov),
        Block::Picture { prov: p, .. }
        | Block::Footnote { prov: p, .. }
        | Block::PageHeader { prov: p, .. }
        | Block::PageFooter { prov: p, .. } => *p = Some(prov),
        // Text-rescue paragraphs/headings/lists and other blocks: no stable
        // control provenance, leave as-is.
        _ => {}
    }
}

/// Section-level list grouping (T8 entry point).
///
/// The orchestrator collects every block produced by a section's paragraphs and
/// passes the flat stream here so that consecutive single-item lists coalesce
/// across paragraph boundaries.  Thin public wrapper over [`group_lists`].
pub(crate) fn group_section_lists(blocks: Vec<Block>) -> Vec<Block> {
    group_lists(blocks)
}

/// Whether a paragraph's `column_type` opens a new *column chunk* in a
/// multi-column flow (as opposed to a page break or a section boundary).
///
/// In HWP a single section laid out in multiple columns is still a *linear*
/// paragraph stream; the points where the text jumps to the next column are
/// marked on the paragraph that begins the new column with
/// [`ColumnBreakType::Column`] (단 나누기) or [`ColumnBreakType::MultiColumn`]
/// (다단 나누기).  DocLang has no column layout, so we model the columns as one
/// logical `<thread>`: the content before the first break is the thread start,
/// and each subsequent column chunk is a thread continuation (see
/// [`assemble_section_with_threads`]).
///
/// [`ColumnBreakType::Page`] is a page break (handled as `Block::PageBreak`, not
/// a column boundary) and [`ColumnBreakType::Section`] is a section boundary
/// (sections are already independent — rhwp splits them — so it never opens a
/// thread chunk).
fn is_column_chunk_break(column_type: ColumnBreakType) -> bool {
    matches!(
        column_type,
        ColumnBreakType::Column | ColumnBreakType::MultiColumn
    )
}

/// Assemble one body section's blocks, threading multi-column content.
///
/// `per_para` is the block stream produced for each of the section's paragraphs,
/// in paragraph order (so `per_para[i]` are the blocks lowered from
/// `paras[i]`).  When the section contains any column-chunk break (see
/// [`is_column_chunk_break`]) the entire section is treated as a single logical
/// `<thread>`:
///
/// * a fresh `thread_id` is drawn from `thread_counter` (monotonic per
///   document) and rendered as `"col-{n}"`;
/// * a [`Block::ThreadStart`] is inserted before the section's first block;
/// * a [`Block::ThreadContinuation`] is inserted before the first block of every
///   column chunk after the first.
///
/// Because DocLang cannot express the column layout itself (count / gap /
/// separator), one [`LossKind::SectionSettings`] entry is recorded per
/// multi-column section, carrying the declared column count when a `ColumnDef`
/// control is present in the section.
///
/// Sections without any column-chunk break are returned unchanged (after list
/// grouping), so single-column documents never gain a `<thread>` marker.
pub(crate) fn assemble_section_with_threads(
    paras: &[crate::model::paragraph::Paragraph],
    per_para: Vec<Vec<Block>>,
    thread_counter: &mut usize,
    location: &str,
    loss: &mut LossReport,
) -> Vec<Block> {
    debug_assert_eq!(paras.len(), per_para.len());

    // Does this section flow across columns at all?
    let has_columns = paras.iter().any(|p| is_column_chunk_break(p.column_type));
    if !has_columns {
        // Common case: no column threading — flatten and group lists as before.
        let flat: Vec<Block> = per_para.into_iter().flatten().collect();
        return group_section_lists(flat);
    }

    // One thread id for the whole multi-column section.
    let thread_id = format!("col-{}", *thread_counter);
    *thread_counter += 1;

    // Record the column layout as a single SectionSettings loss (DocLang has no
    // way to express column count / gap / separators).
    let column_count = section_column_count(paras);
    loss.push(LossEntry {
        kind: LossKind::SectionSettings,
        location: location.to_string(),
        detail: match column_count {
            Some(n) => format!(
                "multi-column layout ({} columns) flattened into one <thread id=\"{}\">; column count/gap not representable in DocLang",
                n, thread_id
            ),
            None => format!(
                "multi-column layout flattened into one <thread id=\"{}\">; column count/gap not representable in DocLang",
                thread_id
            ),
        },
    });

    // Walk the per-paragraph block stream, inserting the thread head before the
    // first block overall (ThreadStart) and before the first block of each
    // subsequent column chunk (ThreadContinuation).  `pending_head` defers the
    // marker until we actually have a block to attach it to, so empty column
    // chunks (a break paragraph that produced no blocks) carry the marker
    // forward to the next non-empty chunk rather than emitting a dangling thread.
    let mut flat: Vec<Block> = Vec::new();
    let mut emitted_start = false;
    let mut pending_head: Option<Block> = None;

    for (para, blocks) in paras.iter().zip(per_para) {
        if is_column_chunk_break(para.column_type) {
            // A new column chunk begins here.  The very first chunk is the
            // ThreadStart; later chunks are continuations.
            pending_head = Some(if emitted_start {
                Block::ThreadContinuation {
                    thread_id: thread_id.clone(),
                }
            } else {
                Block::ThreadStart {
                    thread_id: thread_id.clone(),
                }
            });
            emitted_start = true;
        }
        for block in blocks {
            // The section's first emitted block must carry ThreadStart even if
            // the leading paragraph itself was not flagged as a column break
            // (the first column chunk is implicit, before the first break).
            if !emitted_start && pending_head.is_none() {
                pending_head = Some(Block::ThreadStart {
                    thread_id: thread_id.clone(),
                });
                emitted_start = true;
            }
            if let Some(head) = pending_head.take() {
                flat.push(head);
            }
            flat.push(block);
        }
    }

    group_section_lists(flat)
}

/// Best-effort column count for a section: the largest `ColumnDef.column_count`
/// declared by any paragraph's controls (the `cold` control usually sits on the
/// section's first paragraph).  `None` when no `ColumnDef` is present.
fn section_column_count(paras: &[crate::model::paragraph::Paragraph]) -> Option<u16> {
    let mut max: Option<u16> = None;
    for para in paras {
        for ctrl in &para.controls {
            if let crate::model::control::Control::ColumnDef(c) = ctrl {
                max = Some(max.map_or(c.column_count, |m| m.max(c.column_count)));
            }
        }
    }
    max
}

/// Coalesce runs of adjacent single-kind lists into combined lists.
///
/// A maximal run of consecutive `Block::List`s that all share the same
/// `ordered` flag is merged into one `Block::List` whose `items` are the
/// concatenation of the run's items.  Any other block — or a list of the
/// opposite kind — terminates the current run.
fn group_lists(blocks: Vec<Block>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::with_capacity(blocks.len());
    for block in blocks {
        match block {
            Block::List {
                ordered,
                mut items,
                lost,
                prov,
            } => {
                if let Some(Block::List {
                    ordered: prev_ordered,
                    items: prev_items,
                    ..
                }) = out.last_mut()
                {
                    if *prev_ordered == ordered {
                        // Merge into the run; the run keeps the first list's
                        // `prov` (its leading item) so the location anchors the
                        // grouped list at its first line.
                        prev_items.append(&mut items);
                        continue;
                    }
                }
                out.push(Block::List {
                    ordered,
                    items,
                    lost,
                    prov,
                });
            }
            other => out.push(other),
        }
    }
    out
}

/// Count footnote/endnote blocks just produced and advance the running counter
/// so subsequent footnotes get the next sequential number.
fn bump_footnote_counter(blocks: &[Block], counter: &mut usize) {
    for b in blocks {
        if let Block::Footnote { .. } = b {
            *counter += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doclang::ir::Inline;
    use crate::model::document::{DocInfo, Document};
    use crate::model::paragraph::{ColumnBreakType, Paragraph};
    use crate::model::style::{HeadType, ParaShape};

    /// Build a plain-text paragraph referencing `para_shape_id`, with computed
    /// UTF-16 `char_offsets` so [`extract_inlines`] yields its text.
    fn para(text: &str, para_shape_id: u16) -> Paragraph {
        let mut char_offsets = Vec::new();
        let mut off: u32 = 0;
        for c in text.chars() {
            char_offsets.push(off);
            off += if (c as u32) > 0xFFFF { 2 } else { 1 };
        }
        Paragraph {
            text: text.to_string(),
            char_offsets,
            para_shape_id,
            ..Default::default()
        }
    }

    /// DocInfo whose para_shapes[i] carries head_type/para_level per the input.
    fn doc_info(shapes: &[(HeadType, u8)]) -> DocInfo {
        DocInfo {
            para_shapes: shapes
                .iter()
                .map(|&(head_type, para_level)| ParaShape {
                    head_type,
                    para_level,
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn plain_paragraph() {
        let di = doc_info(&[(HeadType::None, 0)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let mut fc = 1;
        let out = convert_paragraph(
            &para("hello", 0),
            &doc,
            &di,
            Mode::Lean,
            &mut fc,
            "s0/p0",
            &mut loss,
        );
        assert_eq!(
            out,
            vec![Block::Paragraph {
                content: vec![Inline::Text("hello".to_string())],
                lost: None,
                prov: None,
            }]
        );
    }

    #[test]
    fn outline_becomes_heading_level_plus_one() {
        // para_level 0 → heading level 1; para_level 2 → level 3.
        let di = doc_info(&[(HeadType::Outline, 0), (HeadType::Outline, 2)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let mut fc = 1;
        let h1 = convert_paragraph(
            &para("Title", 0),
            &doc,
            &di,
            Mode::Lean,
            &mut fc,
            "s0/p0",
            &mut loss,
        );
        let h3 = convert_paragraph(
            &para("Sub", 1),
            &doc,
            &di,
            Mode::Lean,
            &mut fc,
            "s0/p1",
            &mut loss,
        );
        assert!(matches!(h1[0], Block::Heading { level: 1, .. }));
        assert!(matches!(h3[0], Block::Heading { level: 3, .. }));
    }

    #[test]
    fn outline_level_clamped_to_six() {
        // para_level 6 → level 7 → clamped to 6.
        let di = doc_info(&[(HeadType::Outline, 6)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let mut fc = 1;
        let out = convert_paragraph(
            &para("Deep", 0),
            &doc,
            &di,
            Mode::Lean,
            &mut fc,
            "s0/p0",
            &mut loss,
        );
        assert!(matches!(out[0], Block::Heading { level: 6, .. }));
    }

    #[test]
    fn number_and_bullet_become_single_item_lists() {
        let di = doc_info(&[(HeadType::Number, 0), (HeadType::Bullet, 0)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let mut fc = 1;
        let n = convert_paragraph(
            &para("one", 0),
            &doc,
            &di,
            Mode::Lean,
            &mut fc,
            "s0/p0",
            &mut loss,
        );
        let b = convert_paragraph(
            &para("dot", 1),
            &doc,
            &di,
            Mode::Lean,
            &mut fc,
            "s0/p1",
            &mut loss,
        );
        match &n[0] {
            Block::List { ordered, items, .. } => {
                assert!(*ordered);
                assert_eq!(items.len(), 1);
            }
            other => panic!("expected ordered list, got {other:?}"),
        }
        match &b[0] {
            Block::List { ordered, items, .. } => {
                assert!(!*ordered);
                assert_eq!(items.len(), 1);
            }
            other => panic!("expected bullet list, got {other:?}"),
        }
    }

    #[test]
    fn page_break_paragraph_prefixes_page_break() {
        let di = doc_info(&[(HeadType::None, 0)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let mut fc = 1;
        let mut p = para("after break", 0);
        p.column_type = ColumnBreakType::Page;
        let out = convert_paragraph(&p, &doc, &di, Mode::Lean, &mut fc, "s0/p0", &mut loss);
        assert_eq!(out[0], Block::PageBreak);
        assert!(matches!(out[1], Block::Paragraph { .. }));
    }

    #[test]
    fn consecutive_bullets_grouped_into_one_list() {
        let di = doc_info(&[(HeadType::Bullet, 0)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let paras = vec![para("a", 0), para("b", 0), para("c", 0)];
        let out = convert_paragraphs(&paras, &doc, &di, Mode::Lean, "s0", &mut loss);
        assert_eq!(out.len(), 1);
        match &out[0] {
            Block::List { ordered, items, .. } => {
                assert!(!*ordered);
                assert_eq!(items.len(), 3, "three bullets coalesce into one list");
            }
            other => panic!("expected one grouped list, got {other:?}"),
        }
    }

    #[test]
    fn mixed_ordered_and_bullet_runs_stay_separate() {
        // Number, Number, Bullet → ordered list (2 items) then bullet list (1).
        let di = doc_info(&[(HeadType::Number, 0), (HeadType::Bullet, 0)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let paras = vec![para("1", 0), para("2", 0), para("dot", 1)];
        let out = convert_paragraphs(&paras, &doc, &di, Mode::Lean, "s0", &mut loss);
        assert_eq!(out.len(), 2);
        match &out[0] {
            Block::List {
                ordered: true,
                items,
                ..
            } => assert_eq!(items.len(), 2),
            other => panic!("expected ordered list of 2, got {other:?}"),
        }
        match &out[1] {
            Block::List {
                ordered: false,
                items,
                ..
            } => assert_eq!(items.len(), 1),
            other => panic!("expected bullet list of 1, got {other:?}"),
        }
    }

    #[test]
    fn paragraph_between_lists_splits_runs() {
        // Bullet, Paragraph, Bullet → list, paragraph, list (no merge across).
        let di = doc_info(&[(HeadType::Bullet, 0), (HeadType::None, 0)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let paras = vec![para("a", 0), para("text", 1), para("b", 0)];
        let out = convert_paragraphs(&paras, &doc, &di, Mode::Lean, "s0", &mut loss);
        assert_eq!(out.len(), 3);
        assert!(matches!(out[0], Block::List { ordered: false, .. }));
        assert!(matches!(out[1], Block::Paragraph { .. }));
        assert!(matches!(out[2], Block::List { ordered: false, .. }));
    }

    // ---- Multi-column thread assembly --------------------------------------

    /// Lower a section's paragraphs (as `build_sir` does) into a per-paragraph
    /// block stream, then run it through [`assemble_section_with_threads`].
    fn assemble(
        paras: &[Paragraph],
        di: &DocInfo,
        thread_counter: &mut usize,
        loss: &mut LossReport,
    ) -> Vec<Block> {
        let doc = Document::default();
        let mut per_para: Vec<Vec<Block>> = Vec::new();
        let mut fc = 1usize;
        for (i, p) in paras.iter().enumerate() {
            let loc = format!("s0/p{}", i);
            per_para.push(convert_paragraph(
                p,
                &doc,
                di,
                Mode::Lean,
                &mut fc,
                &loc,
                loss,
            ));
        }
        assemble_section_with_threads(paras, per_para, thread_counter, "s0", loss)
    }

    #[test]
    fn single_column_section_emits_no_thread() {
        let di = doc_info(&[(HeadType::None, 0)]);
        let mut tc = 0usize;
        let mut loss = LossReport::new();
        let paras = vec![para("a", 0), para("b", 0)];
        let out = assemble(&paras, &di, &mut tc, &mut loss);
        assert!(
            !out.iter().any(|b| matches!(
                b,
                Block::ThreadStart { .. } | Block::ThreadContinuation { .. }
            )),
            "single-column section must not gain a <thread> marker"
        );
        assert_eq!(tc, 0, "no thread id consumed");
        assert!(loss.is_empty(), "no SectionSettings loss for single-column");
    }

    #[test]
    fn column_break_threads_section_into_start_and_continuation() {
        let di = doc_info(&[(HeadType::None, 0)]);
        let mut tc = 0usize;
        let mut loss = LossReport::new();
        // First chunk: "a", "b"; column break opens second chunk: "c".
        let mut p_break = para("c", 0);
        p_break.column_type = ColumnBreakType::Column;
        let paras = vec![para("a", 0), para("b", 0), p_break];
        let out = assemble(&paras, &di, &mut tc, &mut loss);

        // Expected order: ThreadStart, <a>, <b>, ThreadContinuation, <c>.
        assert!(matches!(out[0], Block::ThreadStart { ref thread_id } if thread_id == "col-0"));
        let cont_idx = out
            .iter()
            .position(|b| matches!(b, Block::ThreadContinuation { .. }))
            .expect("a continuation marker");
        match &out[cont_idx] {
            Block::ThreadContinuation { thread_id } => assert_eq!(thread_id, "col-0"),
            _ => unreachable!(),
        }
        // The continuation must come after the first two paragraphs and before
        // the third, with exactly one start and one continuation.
        let starts = out
            .iter()
            .filter(|b| matches!(b, Block::ThreadStart { .. }))
            .count();
        let conts = out
            .iter()
            .filter(|b| matches!(b, Block::ThreadContinuation { .. }))
            .count();
        assert_eq!((starts, conts), (1, 1));
        assert_eq!(tc, 1, "one thread id consumed");
    }

    #[test]
    fn multicolumn_break_also_threads() {
        let di = doc_info(&[(HeadType::None, 0)]);
        let mut tc = 0usize;
        let mut loss = LossReport::new();
        let mut p_break = para("second", 0);
        p_break.column_type = ColumnBreakType::MultiColumn;
        let paras = vec![para("first", 0), p_break];
        let out = assemble(&paras, &di, &mut tc, &mut loss);
        assert!(matches!(out[0], Block::ThreadStart { .. }));
        assert!(out
            .iter()
            .any(|b| matches!(b, Block::ThreadContinuation { .. })));
    }

    #[test]
    fn column_section_records_section_settings_loss_with_count() {
        let di = doc_info(&[(HeadType::None, 0)]);
        let mut tc = 0usize;
        let mut loss = LossReport::new();
        // A ColumnDef(count=2) on the first paragraph; a column break later.
        let mut p0 = para("a", 0);
        p0.controls.push(crate::model::control::Control::ColumnDef(
            crate::model::page::ColumnDef {
                column_count: 2,
                ..Default::default()
            },
        ));
        let mut p1 = para("b", 0);
        p1.column_type = ColumnBreakType::Column;
        let paras = vec![p0, p1];
        let _ = assemble(&paras, &di, &mut tc, &mut loss);
        let entry = loss
            .iter()
            .find(|e| e.kind == LossKind::SectionSettings)
            .expect("a SectionSettings loss for the multi-column layout");
        assert!(
            entry.detail.contains("2 columns"),
            "detail names the column count: {}",
            entry.detail
        );
    }

    #[test]
    fn section_break_alone_does_not_thread() {
        // ColumnBreakType::Section marks an independent section boundary, never a
        // column chunk — it must not introduce a thread.
        let di = doc_info(&[(HeadType::None, 0)]);
        let mut tc = 0usize;
        let mut loss = LossReport::new();
        let mut p0 = para("a", 0);
        p0.column_type = ColumnBreakType::Section;
        let paras = vec![p0, para("b", 0)];
        let out = assemble(&paras, &di, &mut tc, &mut loss);
        assert!(!out.iter().any(|b| matches!(
            b,
            Block::ThreadStart { .. } | Block::ThreadContinuation { .. }
        )));
        assert_eq!(tc, 0);
    }

    #[test]
    fn thread_ids_are_unique_across_sections() {
        // Two multi-column sections assembled with a shared counter get distinct
        // ids (col-0, col-1) — verifies document-wide monotonicity.
        let di = doc_info(&[(HeadType::None, 0)]);
        let mut tc = 0usize;
        let mut loss = LossReport::new();
        let mut b1 = para("b", 0);
        b1.column_type = ColumnBreakType::Column;
        let out0 = assemble(&[para("a", 0), b1], &di, &mut tc, &mut loss);
        let mut d1 = para("d", 0);
        d1.column_type = ColumnBreakType::Column;
        let out1 = assemble(&[para("c", 0), d1], &di, &mut tc, &mut loss);
        let id0 = match &out0[0] {
            Block::ThreadStart { thread_id } => thread_id.clone(),
            _ => panic!(),
        };
        let id1 = match &out1[0] {
            Block::ThreadStart { thread_id } => thread_id.clone(),
            _ => panic!(),
        };
        assert_eq!(id0, "col-0");
        assert_eq!(id1, "col-1");
        assert_ne!(id0, id1);
    }

    #[test]
    fn empty_paragraph_emits_no_block() {
        let di = doc_info(&[(HeadType::None, 0)]);
        let doc = Document::default();
        let mut loss = LossReport::new();
        let mut fc = 1;
        let out = convert_paragraph(
            &para("", 0),
            &doc,
            &di,
            Mode::Lean,
            &mut fc,
            "s0/p0",
            &mut loss,
        );
        assert!(out.is_empty());
    }
}
