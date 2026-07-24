//! Inline formatting-run splitting (Phase A — inline).
//!
//! Lowers a single rhwp [`Paragraph`] into a sequence of crate-IR [`Inline`]
//! nodes: plain/styled text spans, tabs, and line breaks.  Inline *objects*
//! (tables, pictures, equations, footnotes, …) are **not** emitted here — they
//! are handled at block level via `Paragraph.controls` in later tasks.
//!
//! # `CharShapeRef.start_pos` unit — VERIFIED (Critic N1)
//!
//! `start_pos` is a **UTF-16 code-unit stream offset** measured from the start
//! of the paragraph's PARA_TEXT stream (NOT a visible-char index).
//!
//! Evidence from the authoritative rhwp source (`/tmp/rhwp-src/`):
//!
//! 1. The parser reads `start_pos` verbatim from the 8-byte PARA_CHAR_SHAPE
//!    record entries (`u32 start_pos` + `u32 char_shape_id`):
//!    `src/parser/body_text.rs:409-423` (`parse_para_char_shape`).
//! 2. The same scale is used for `char_offsets`: each text char records
//!    `code_unit_pos = (pos / 2)` where `pos` is the byte offset into the
//!    UTF-16LE PARA_TEXT stream — i.e. the UTF-16 code-unit index:
//!    `src/parser/body_text.rs:275,282,365,374`.  Surrogate-pair chars push a
//!    single `char_offsets` entry but advance `pos` by 4 bytes (= +2 code
//!    units), so a shape that begins after an emoji has a `start_pos` two
//!    higher than the char index (`body_text.rs:360-371`).
//! 3. rhwp's own renderer resolves runs with Interpretation A and explicitly
//!    documents `start_pos` as a "UTF-16 stream offset", finding the first
//!    `char_offsets` entry `>= start_pos` as the run's first visible char:
//!    `src/renderer/composer.rs:800-825` (Task #915 — an earlier
//!    "visible-char-index" interpretation, #884, was reverted as a bug).
//!
//! This module mimics Interpretation A exactly: `char_offsets` maps each
//! visible char index → its UTF-16 stream offset, and a run boundary at
//! `start_pos` becomes the visible char index of the first `char_offsets`
//! entry `>= start_pos`.

use crate::model::control::{Control, FieldType};
use crate::model::document::DocInfo;
use crate::model::paragraph::Paragraph;
use crate::model::style::CharShape;

use crate::doclang::ir::{Inline, StyleFlags};
use crate::doclang::loss::report::{LossEntry, LossKind, LossReport};

use super::resources;

/// Unicode OBJECT REPLACEMENT CHARACTER.
///
/// The rhwp HWP3 path inserts this as a placeholder for inline objects (the
/// HWP5 path leaves a `char_offsets` gap instead).  The actual objects are
/// emitted from `Paragraph.controls`, so we strip the marker here to avoid a
/// stray "?" glyph in the text stream.
const OBJECT_REPLACEMENT: char = '\u{FFFC}';

/// Lower one paragraph's text + char-shape runs into a sequence of [`Inline`]
/// nodes.
///
/// `loss` accumulates lean-mode loss entries; this function records at most one
/// [`LossKind::FontInfo`] and one [`LossKind::CharColor`] per paragraph (never
/// per run) using `location` to identify the source paragraph.
pub(crate) fn extract_inlines(
    para: &Paragraph,
    doc_info: &DocInfo,
    location: &str,
    loss: &mut LossReport,
) -> Vec<Inline> {
    // The whole paragraph is one segment (no cuts).
    extract_inline_segments(para, doc_info, location, loss, &[])
        .pop()
        .unwrap_or_default()
}

/// Like [`extract_inlines`] but splits the output at the given visible-char
/// `cuts`, returning `cuts.len() + 1` inline segments. Segment `k` covers chars
/// `[cuts[k-1], cuts[k])` (with implicit `0` / `char_len` bounds). `cuts` must be
/// sorted ascending, de-duplicated, and within `0..=char_len`.
///
/// Per-paragraph loss de-duplication (font / colour) is shared across all
/// segments, and a hyperlink span that straddles a cut is closed at the boundary
/// and re-opened in the next segment.
pub(crate) fn extract_inline_segments(
    para: &Paragraph,
    doc_info: &DocInfo,
    location: &str,
    loss: &mut LossReport,
    cuts: &[usize],
) -> Vec<Vec<Inline>> {
    let chars: Vec<char> = para.text.chars().collect();
    let n_segments = cuts.len() + 1;
    if chars.is_empty() {
        return vec![Vec::new(); n_segments];
    }

    // Per-paragraph loss de-duplication flags.
    let mut font_loss_recorded = false;
    let mut color_loss_recorded = false;

    let runs = build_runs(para, chars.len());

    // Per visible-char index, which hyperlink URI (if any) covers it. When the
    // document has no hyperlink fields this is all `None` and the emission below
    // is byte-identical to the href-free path.
    let (href_at, uris) = hyperlink_href_map(para, chars.len());

    let mut segments: Vec<Vec<Inline>> = Vec::with_capacity(n_segments);
    let mut inlines: Vec<Inline> = Vec::new(); // current segment
    let mut href_open: Option<(usize, Vec<Inline>)> = None;
    let mut cur_href: Option<usize> = None;
    let mut pending_text = String::new();
    let mut pending_style = StyleFlags::default();
    let mut gi = 0usize; // running visible-char index across runs
    let mut cut_idx = 0usize;

    for run in &runs {
        let style = match run
            .char_shape_id
            .and_then(|id| resources::char_shape(doc_info, id))
        {
            Some(cs) => {
                record_char_shape_loss(
                    cs,
                    doc_info,
                    location,
                    loss,
                    &mut font_loss_recorded,
                    &mut color_loss_recorded,
                );
                style_flags_from(cs)
            }
            None => StyleFlags::default(),
        };

        for &c in &chars[run.start..run.end] {
            // Close the current segment at any cut boundary we have reached.
            while cut_idx < cuts.len() && cuts[cut_idx] <= gi {
                {
                    let sink = href_open.as_mut().map(|(_, v)| v).unwrap_or(&mut inlines);
                    flush_text(sink, &mut pending_text, &pending_style);
                }
                if let Some((uri_idx, content)) = href_open.take() {
                    push_merged(
                        &mut inlines,
                        Inline::Href {
                            uri: uris[uri_idx].clone(),
                            content,
                        },
                    );
                }
                cur_href = None;
                segments.push(std::mem::take(&mut inlines));
                cut_idx += 1;
            }

            // Enter/leave a hyperlink span before emitting this char.
            let want = href_at.get(gi).copied().flatten();
            if want != cur_href {
                {
                    let sink = href_open.as_mut().map(|(_, v)| v).unwrap_or(&mut inlines);
                    flush_text(sink, &mut pending_text, &pending_style);
                }
                if let Some((uri_idx, content)) = href_open.take() {
                    push_merged(
                        &mut inlines,
                        Inline::Href {
                            uri: uris[uri_idx].clone(),
                            content,
                        },
                    );
                }
                if let Some(idx) = want {
                    href_open = Some((idx, Vec::new()));
                }
                cur_href = want;
            }

            let sink = href_open.as_mut().map(|(_, v)| v).unwrap_or(&mut inlines);
            match c {
                '\t' => {
                    flush_text(sink, &mut pending_text, &pending_style);
                    push_merged(sink, Inline::Tab);
                }
                '\n' => {
                    flush_text(sink, &mut pending_text, &pending_style);
                    push_merged(sink, Inline::LineBreak);
                }
                OBJECT_REPLACEMENT => {
                    // Object placeholder — handled via controls[] elsewhere.
                    flush_text(sink, &mut pending_text, &pending_style);
                }
                _ => {
                    // If the style changed since the last buffered char, flush
                    // the previous span first so each span carries one style.
                    if !pending_text.is_empty() && pending_style != style {
                        flush_text(sink, &mut pending_text, &pending_style);
                    }
                    pending_style = style;
                    pending_text.push(c);
                }
            }
            gi += 1;
        }
    }

    {
        let sink = href_open.as_mut().map(|(_, v)| v).unwrap_or(&mut inlines);
        flush_text(sink, &mut pending_text, &pending_style);
    }
    if let Some((uri_idx, content)) = href_open.take() {
        push_merged(
            &mut inlines,
            Inline::Href {
                uri: uris[uri_idx].clone(),
                content,
            },
        );
    }
    segments.push(std::mem::take(&mut inlines));

    // Any cuts at/after char_len produce trailing empty segments.
    while segments.len() < n_segments {
        segments.push(Vec::new());
    }
    segments
}

/// Flush buffered text into `sink` as a `Text` (plain) or `Styled` node.
/// No-op when the buffer is empty.
fn flush_text(sink: &mut Vec<Inline>, text: &mut String, style: &StyleFlags) {
    if text.is_empty() {
        return;
    }
    let node = if style.is_plain() {
        Inline::Text(std::mem::take(text))
    } else {
        Inline::Styled(*style, vec![Inline::Text(std::mem::take(text))])
    };
    push_merged(sink, node);
}

/// Map each visible-char index to the index (into the returned `uris`) of the
/// hyperlink that covers it, or `None`.
///
/// Only [`FieldType::Hyperlink`] fields (and the HWP3 [`Control::Hyperlink`])
/// with a non-empty resolved URI participate; other field kinds are ignored.
/// Ranges are clamped to `char_len` defensively.
fn hyperlink_href_map(para: &Paragraph, char_len: usize) -> (Vec<Option<usize>>, Vec<String>) {
    let mut map = vec![None; char_len];
    let mut uris: Vec<String> = Vec::new();
    for fr in &para.field_ranges {
        let uri = match para.controls.get(fr.control_idx) {
            Some(Control::Field(f)) if f.field_type == FieldType::Hyperlink => {
                hyperlink_uri(&f.command)
            }
            Some(Control::Hyperlink(h)) => hyperlink_uri(&h.url),
            _ => continue,
        };
        if uri.is_empty() {
            continue;
        }
        let start = fr.start_char_idx.min(char_len);
        let end = fr.end_char_idx.min(char_len);
        if start >= end {
            continue;
        }
        let uri_idx = uris.len();
        uris.push(uri);
        for slot in &mut map[start..end] {
            *slot = Some(uri_idx);
        }
    }
    (map, uris)
}

/// Extract the URL from a HWP hyperlink field `command`.
///
/// The command is the URL optionally followed by `;`-separated parameters, with
/// `\` escaping `;` and `\` itself. We return the first (URL) segment, unescaped
/// and trimmed.
fn hyperlink_uri(command: &str) -> String {
    let mut out = String::new();
    let mut chars = command.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            }
            ';' => break,
            _ => out.push(c),
        }
    }
    out.trim().to_string()
}

/// A contiguous formatting run expressed as a half-open visible-char range.
struct Run {
    start: usize,
    end: usize,
    /// `char_shape_id` for this run, or `None` if no shape applies.
    char_shape_id: Option<u32>,
}

/// Compute formatting-run boundaries (visible-char indices).
///
/// For each `CharShapeRef`, the run begins at the first `char_offsets` entry
/// whose UTF-16 offset is `>= start_pos` (Interpretation A, mirroring rhwp's
/// `composer.rs:822-825`).  Runs are emitted in char-index order; if no shape
/// covers char 0 the leading text uses `None` (plain).
fn build_runs(para: &Paragraph, char_len: usize) -> Vec<Run> {
    // Map each char_shape's start_pos → visible char index, dedup, sort.
    let mut starts: Vec<(usize, u32)> = Vec::with_capacity(para.char_shapes.len());
    for cs in &para.char_shapes {
        let idx = para
            .char_offsets
            .iter()
            .position(|&off| off >= cs.start_pos)
            .unwrap_or(char_len);
        if idx < char_len {
            starts.push((idx, cs.char_shape_id));
        }
    }
    starts.sort_by_key(|&(idx, _)| idx);
    // On duplicate start index keep the last entry (most recent CharShapeRef),
    // matching rhwp's reverse-dedup in composer.rs:837-841.
    let mut deduped: Vec<(usize, u32)> = Vec::with_capacity(starts.len());
    for (idx, id) in starts {
        if let Some(last) = deduped.last_mut() {
            if last.0 == idx {
                last.1 = id;
                continue;
            }
        }
        deduped.push((idx, id));
    }

    let mut runs: Vec<Run> = Vec::new();

    // Leading text not covered by any shape → plain run.
    let first_start = deduped.first().map(|&(idx, _)| idx).unwrap_or(char_len);
    if first_start > 0 {
        runs.push(Run {
            start: 0,
            end: first_start,
            char_shape_id: None,
        });
    }

    for (i, &(start, id)) in deduped.iter().enumerate() {
        let end = deduped
            .get(i + 1)
            .map(|&(next, _)| next)
            .unwrap_or(char_len);
        if start < end {
            runs.push(Run {
                start,
                end,
                char_shape_id: Some(id),
            });
        }
    }

    if runs.is_empty() {
        runs.push(Run {
            start: 0,
            end: char_len,
            char_shape_id: None,
        });
    }

    runs
}

/// Derive the six DocLang-expressible flags from a resolved [`CharShape`].
fn style_flags_from(cs: &CharShape) -> StyleFlags {
    use crate::model::style::UnderlineType;
    StyleFlags {
        bold: cs.bold,
        italic: cs.italic,
        underline: cs.underline_type != UnderlineType::None,
        strike: cs.strikethrough,
        superscript: cs.superscript,
        subscript: cs.subscript,
    }
}

/// Record lean-mode loss for properties a [`CharShape`] carries that DocLang
/// cannot express (non-default text colour, font face/size).
///
/// At most one entry of each kind is recorded per paragraph; the boolean flags
/// guard against per-run spam.
fn record_char_shape_loss(
    cs: &CharShape,
    doc_info: &DocInfo,
    location: &str,
    loss: &mut LossReport,
    font_recorded: &mut bool,
    color_recorded: &mut bool,
) {
    // text_color is ColorRef = u32; default/black is 0.  Any non-zero value is
    // colour information DocLang cannot express.
    if !*color_recorded && cs.text_color != 0 {
        loss.push(LossEntry {
            kind: LossKind::CharColor,
            location: location.to_string(),
            detail: format!("text_color=0x{:06X}", cs.text_color & 0x00FF_FFFF),
        });
        *color_recorded = true;
    }

    if !*font_recorded {
        // `font_ids` carries one face per language slot (한글/영문/한자/일어/기타/
        // 기호/사용자). A face in *any* slot is information DocLang cannot express,
        // so scan all of them rather than only slot 0 (한글) — otherwise an
        // English/CJK/symbol-only face goes unreported. Report the first resolved
        // face along with its slot.
        let resolved = cs.font_ids.iter().enumerate().find_map(|(lang, &fid)| {
            resources::font_name(doc_info, lang, fid)
                .filter(|n| !n.is_empty())
                .map(|n| (lang, n))
        });
        if resolved.is_some() || cs.base_size != 0 {
            let detail = match resolved {
                Some((lang, name)) => {
                    format!(
                        "font={} (lang slot {}), base_size={}",
                        name, lang, cs.base_size
                    )
                }
                None => format!("font_id={}, base_size={}", cs.font_ids[0], cs.base_size),
            };
            loss.push(LossEntry {
                kind: LossKind::FontInfo,
                location: location.to_string(),
                detail,
            });
            *font_recorded = true;
        }
    }
}

/// Append `node`, merging it into the previous node when both are `Text`/plain
/// or identically-`Styled`, so adjacent runs with identical formatting collapse.
fn push_merged(inlines: &mut Vec<Inline>, node: Inline) {
    if let Some(last) = inlines.last_mut() {
        match (last, &node) {
            (Inline::Text(prev), Inline::Text(next)) => {
                prev.push_str(next);
                return;
            }
            (Inline::Styled(p_flags, p_children), Inline::Styled(n_flags, _))
                if p_flags == n_flags =>
            {
                if let Inline::Styled(_, n_children) = node {
                    // Merge inner text where possible.
                    for child in n_children {
                        push_merged(p_children, child);
                    }
                }
                return;
            }
            _ => {}
        }
    }
    inlines.push(node);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::DocInfo;
    use crate::model::paragraph::{CharShapeRef, Paragraph};
    use crate::model::style::CharShape;

    /// Build a paragraph from `text`, computing `char_offsets` as the UTF-16
    /// stream offsets (mirroring the parser).  `shapes` are `(start_pos,
    /// char_shape_id)` pairs in UTF-16 code-unit space.
    fn make_para(text: &str, shapes: &[(u32, u32)]) -> Paragraph {
        let mut char_offsets = Vec::new();
        let mut off: u32 = 0;
        for c in text.chars() {
            char_offsets.push(off);
            off += if (c as u32) > 0xFFFF { 2 } else { 1 };
        }
        Paragraph {
            text: text.to_string(),
            char_offsets,
            char_shapes: shapes
                .iter()
                .map(|&(start_pos, char_shape_id)| CharShapeRef {
                    start_pos,
                    char_shape_id,
                })
                .collect(),
            ..Default::default()
        }
    }

    fn bold_shape() -> CharShape {
        CharShape {
            bold: true,
            ..Default::default()
        }
    }

    fn doc_info(shapes: Vec<CharShape>) -> DocInfo {
        DocInfo {
            char_shapes: shapes,
            ..Default::default()
        }
    }

    #[test]
    fn single_run_plain() {
        let para = make_para("hello", &[(0, 0)]);
        let di = doc_info(vec![CharShape::default()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(out, vec![Inline::Text("hello".to_string())]);
        assert!(loss.is_empty());
    }

    #[test]
    fn single_run_bold() {
        let para = make_para("hello", &[(0, 0)]);
        let di = doc_info(vec![bold_shape()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![Inline::Styled(
                StyleFlags {
                    bold: true,
                    ..Default::default()
                },
                vec![Inline::Text("hello".to_string())]
            )]
        );
    }

    #[test]
    fn multi_run_style_change_mid_text() {
        // "ab" plain, "cd" bold. start_pos 2 → char index 2.
        let para = make_para("abcd", &[(0, 0), (2, 1)]);
        let di = doc_info(vec![CharShape::default(), bold_shape()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![
                Inline::Text("ab".to_string()),
                Inline::Styled(
                    StyleFlags {
                        bold: true,
                        ..Default::default()
                    },
                    vec![Inline::Text("cd".to_string())]
                ),
            ]
        );
    }

    #[test]
    fn surrogate_pair_utf16_offset_conversion() {
        // "A😀B": 😀 (U+1F600) is one char but two UTF-16 code units.
        // char_offsets = [0, 1, 3].  A bold run beginning AFTER the emoji must
        // use start_pos = 3 (UTF-16), which maps to char index 2 ("B").
        let para = make_para("A😀B", &[(0, 0), (3, 1)]);
        let di = doc_info(vec![CharShape::default(), bold_shape()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![
                Inline::Text("A😀".to_string()),
                Inline::Styled(
                    StyleFlags {
                        bold: true,
                        ..Default::default()
                    },
                    vec![Inline::Text("B".to_string())]
                ),
            ]
        );
    }

    #[test]
    fn surrogate_pair_boundary_at_emoji() {
        // Bold run begins AT the emoji: start_pos = 1 → char index 1.
        let para = make_para("A😀B", &[(0, 0), (1, 1)]);
        let di = doc_info(vec![CharShape::default(), bold_shape()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![
                Inline::Text("A".to_string()),
                Inline::Styled(
                    StyleFlags {
                        bold: true,
                        ..Default::default()
                    },
                    vec![Inline::Text("😀B".to_string())]
                ),
            ]
        );
    }

    #[test]
    fn adjacent_identical_runs_merged() {
        // Two consecutive bold runs (different char_shape_id but identical
        // flags) collapse into one Styled node.
        let para = make_para("abcd", &[(0, 0), (2, 1)]);
        let di = doc_info(vec![bold_shape(), bold_shape()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![Inline::Styled(
                StyleFlags {
                    bold: true,
                    ..Default::default()
                },
                vec![Inline::Text("abcd".to_string())]
            )]
        );
    }

    #[test]
    fn tab_and_linebreak_control_chars() {
        let para = make_para("a\tb\nc", &[(0, 0)]);
        let di = doc_info(vec![CharShape::default()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![
                Inline::Text("a".to_string()),
                Inline::Tab,
                Inline::Text("b".to_string()),
                Inline::LineBreak,
                Inline::Text("c".to_string()),
            ]
        );
    }

    #[test]
    fn object_replacement_char_stripped() {
        let para = make_para("a\u{FFFC}b", &[(0, 0)]);
        let di = doc_info(vec![CharShape::default()]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        // Marker removed; surrounding text merges.
        assert_eq!(out, vec![Inline::Text("ab".to_string())]);
    }

    #[test]
    fn empty_text() {
        let para = make_para("", &[]);
        let di = doc_info(vec![]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert!(out.is_empty());
        assert!(loss.is_empty());
    }

    #[test]
    fn no_char_shapes_defaults_plain() {
        let para = make_para("plain", &[]);
        let di = doc_info(vec![]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(out, vec![Inline::Text("plain".to_string())]);
    }

    #[test]
    fn color_and_font_loss_recorded_once_per_paragraph() {
        // Two runs, both carrying colour + font info; loss recorded once each.
        let cs0 = CharShape {
            text_color: 0x0000_00FF, // red-ish (BGR), non-zero
            base_size: 1000,
            font_ids: [1, 0, 0, 0, 0, 0, 0],
            ..Default::default()
        };
        let mut cs1 = cs0.clone();
        cs1.text_color = 0x00FF_0000;

        let font = crate::model::style::Font {
            name: "Nanum".to_string(),
            ..Default::default()
        };
        let mut di = doc_info(vec![cs0, cs1]);
        di.font_faces = vec![vec![crate::model::style::Font::default(), font]];

        let para = make_para("abcd", &[(0, 0), (2, 1)]);
        let mut loss = LossReport::new();
        let _ = extract_inlines(&para, &di, "s0/p0", &mut loss);

        let color_count = loss
            .iter()
            .filter(|e| e.kind == LossKind::CharColor)
            .count();
        let font_count = loss.iter().filter(|e| e.kind == LossKind::FontInfo).count();
        assert_eq!(color_count, 1, "colour loss must be recorded once");
        assert_eq!(font_count, 1, "font loss must be recorded once");
    }

    #[test]
    fn nested_flags_bold_italic() {
        let cs = CharShape {
            bold: true,
            italic: true,
            ..Default::default()
        };
        let di = doc_info(vec![cs]);
        let para = make_para("x", &[(0, 0)]);
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![Inline::Styled(
                StyleFlags {
                    bold: true,
                    italic: true,
                    ..Default::default()
                },
                vec![Inline::Text("x".to_string())]
            )]
        );
    }

    #[test]
    fn hyperlink_field_range_wraps_anchor_in_href() {
        use crate::model::control::{Control, Field, FieldType};
        use crate::model::paragraph::FieldRange;

        // "see Open here" — anchor "Open" spans char indices 4..8.
        let mut para = make_para("see Open here", &[]);
        para.controls = vec![Control::Field(Field {
            field_type: FieldType::Hyperlink,
            command: "http://x.com".to_string(),
            ..Default::default()
        })];
        para.field_ranges = vec![FieldRange {
            start_char_idx: 4,
            end_char_idx: 8,
            control_idx: 0,
            end_field_id: 0,
        }];

        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        assert_eq!(
            out,
            vec![
                Inline::Text("see ".to_string()),
                Inline::Href {
                    uri: "http://x.com".to_string(),
                    content: vec![Inline::Text("Open".to_string())],
                },
                Inline::Text(" here".to_string()),
            ]
        );
    }

    #[test]
    fn non_hyperlink_field_does_not_wrap() {
        use crate::model::control::{Control, Field, FieldType};
        use crate::model::paragraph::FieldRange;

        let mut para = make_para("a Open b", &[]);
        para.controls = vec![Control::Field(Field {
            field_type: FieldType::ClickHere,
            command: "hint".to_string(),
            ..Default::default()
        })];
        para.field_ranges = vec![FieldRange {
            start_char_idx: 2,
            end_char_idx: 6,
            control_idx: 0,
            end_field_id: 0,
        }];

        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let out = extract_inlines(&para, &di, "s0/p0", &mut loss);
        // No href wrapping for non-hyperlink fields; plain text only.
        assert_eq!(out, vec![Inline::Text("a Open b".to_string())]);
    }

    #[test]
    fn hyperlink_uri_takes_first_segment_unescaped() {
        assert_eq!(super::hyperlink_uri("http://x.com;1;0"), "http://x.com");
        assert_eq!(super::hyperlink_uri(r"a\;b;rest"), "a;b");
        assert_eq!(super::hyperlink_uri("plain"), "plain");
        assert_eq!(super::hyperlink_uri("  spaced  ;x"), "spaced");
    }

    #[test]
    fn segments_split_text_at_cuts() {
        // "abcde" split at cut position 2 -> ["ab", "cde"].
        let para = make_para("abcde", &[]);
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let segs = extract_inline_segments(&para, &di, "s0/p0", &mut loss, &[2]);
        assert_eq!(
            segs,
            vec![
                vec![Inline::Text("ab".to_string())],
                vec![Inline::Text("cde".to_string())],
            ]
        );
    }

    #[test]
    fn segments_with_no_cuts_match_extract_inlines() {
        let para = make_para("hello world", &[]);
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let segs = extract_inline_segments(&para, &di, "s0/p0", &mut loss, &[]);
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0], vec![Inline::Text("hello world".to_string())]);
    }

    #[test]
    fn cut_at_zero_yields_empty_leading_segment() {
        // Cut at 0 -> empty first segment, then the whole text.
        let para = make_para("xy", &[]);
        let di = DocInfo::default();
        let mut loss = LossReport::new();
        let segs = extract_inline_segments(&para, &di, "s0/p0", &mut loss, &[0]);
        assert_eq!(segs, vec![Vec::new(), vec![Inline::Text("xy".to_string())]]);
    }
}
