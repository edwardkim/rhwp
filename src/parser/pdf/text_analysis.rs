//! Shared, page-local observations. Source text is never rewritten by observation.
//! These are decoded Unicode scalar references, not PDF font glyph IDs.
use pdf_oxide::{geometry::Rect, layout::TextSpan};
use serde::Serialize;
use std::ops::Range;

pub(super) const MAX_GLYPHS: usize = 32_768;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Glyph {
    pub id: String,
    pub source_span: usize,
    pub source_character: usize,
    pub source_utf16: u32,
    pub text: char,
    pub bounds: Rect,
    pub baseline: f32,
    pub font_size: f32,
    pub geometry_quality: &'static str,
    pub visibility: &'static str,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SourceTextRange {
    pub span: usize,
    pub scalars: Range<usize>,
    pub utf16: Range<u32>,
}

struct ObservedSpan {
    bounds: Rect,
    baseline: f32,
    font_size: f32,
    source: SourceTextRange,
    text: String,
}

pub(super) struct ObservedText {
    pub glyphs: Vec<Glyph>,
    spans: Vec<ObservedSpan>,
    pub glyph_observations_incomplete: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TextLine {
    #[serde(rename = "spanIndices")]
    pub indices: Vec<usize>,
    pub source_ranges: Vec<SourceTextRange>,
    pub baseline: f32,
    pub bbox: Rect,
    #[serde(rename = "fontSize")]
    pub font: f32,
    pub preview_text: String,
    #[serde(skip)]
    pub left: f32,
    #[serde(skip)]
    pub right: f32,
    #[serde(skip)]
    pub top: f32,
    #[serde(skip)]
    pub bottom: f32,
    #[serde(skip)]
    pub disjoint: bool,
}

#[derive(Clone, Copy, PartialEq, Serialize)]
pub(super) enum TextAlignment {
    Left,
    Center,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ParagraphLayout {
    pub alignment: TextAlignment,
    pub margin_left: f32,
    /// HWP convention: negative indent advances subsequent lines, not the first.
    pub indent: f32,
}

pub(super) fn paragraph_layout(lines: &[TextLine], owner: Rect) -> Option<ParagraphLayout> {
    let first = lines.first()?;
    // Same page-geometry tolerance as owner containment/snapping, in points.
    // Size-dependent glyph tolerances must not determine paragraph alignment.
    let tolerance = 0.5;
    if lines.len() > 1
        && lines.iter().all(|line| {
            ((line.left + line.right) / 2. - owner.x - owner.width / 2.).abs() <= tolerance
        })
    {
        return Some(ParagraphLayout {
            alignment: TextAlignment::Center,
            margin_left: 0.,
            indent: 0.,
        });
    }
    let continuation_left = lines.get(1).map_or(first.left, |line| line.left);
    if lines
        .iter()
        .skip(1)
        .any(|line| (line.left - continuation_left).abs() > tolerance)
    {
        return None;
    }
    Some(ParagraphLayout {
        alignment: TextAlignment::Left,
        margin_left: first.left.min(continuation_left) - owner.x,
        indent: first.left - continuation_left,
    })
}

/// A layout wrap is not a hard break. Preserve source whitespace first; added
/// separators carry an explicit inference label in the recognition report.
#[derive(Clone, Copy, Serialize)]
pub(super) enum LineJoin {
    Start,
    SourceWhitespace,
    InferredWordSpace,
    InferredCjkContinuation,
    InferredHyphenContinuation,
}

pub(super) fn line_join(previous: &str, next: &str) -> LineJoin {
    let cjk = |c: char| {
        matches!(c as u32,
        0x1100..=0x11ff | 0x2e80..=0x9fff | 0xac00..=0xd7af | 0xf900..=0xfaff | 0x20000..=0x323af)
    };
    match (previous.chars().last(), next.chars().next()) {
        (Some(a), Some(b)) if a.is_whitespace() || b.is_whitespace() => LineJoin::SourceWhitespace,
        (Some('-'), Some(_)) => LineJoin::InferredHyphenContinuation,
        (Some(a), Some(b)) if cjk(a) || cjk(b) => LineJoin::InferredCjkContinuation,
        (Some(_), Some(_)) => LineJoin::InferredWordSpace,
        _ => LineJoin::Start,
    }
}

impl ObservedText {
    pub fn observe(spans: &[TextSpan], media: (f32, f32, f32, f32)) -> Self {
        let mut observed = Self {
            glyphs: Vec::new(),
            spans: Vec::with_capacity(spans.len()),
            glyph_observations_incomplete: false,
        };
        for (i, span) in spans.iter().enumerate() {
            let count = span.text.chars().count();
            observed.spans.push(ObservedSpan {
                bounds: Rect::new(
                    span.bbox.x - media.0,
                    media.3 - span.bbox.y - span.bbox.height,
                    span.bbox.width,
                    span.bbox.height,
                ),
                baseline: media.3 - span.bbox.y,
                font_size: span.font_size,
                source: SourceTextRange {
                    span: i,
                    scalars: 0..count,
                    utf16: 0..span.text.encode_utf16().count() as u32,
                },
                text: span.text.clone(),
            });
            // Retain coarse source ranges even when the detailed diagnostic budget ends.
            if observed.glyph_observations_incomplete || count > MAX_GLYPHS - observed.glyphs.len()
            {
                observed.glyph_observations_incomplete = true;
                continue;
            }
            let offsets_valid = span.char_x_offsets.len() == count
                && span.char_x_offsets.iter().all(|x| {
                    x.is_finite()
                        && *x >= span.bbox.x - 0.5
                        && *x <= span.bbox.x + span.bbox.width + 0.5
                });
            let quality = if offsets_valid {
                "recorded_origins_advance_boxes"
            } else if span.char_widths.len() == count {
                "advance_width_boxes"
            } else {
                "uniform_span_approximation"
            };
            let mut utf16 = 0;
            for (source_character, ch) in span.to_chars().into_iter().enumerate() {
                observed.glyphs.push(Glyph {
                    id: format!("g:{i}:{source_character}"),
                    source_span: i,
                    source_character,
                    source_utf16: utf16,
                    text: ch.char,
                    bounds: Rect::new(
                        ch.bbox.x - media.0,
                        media.3 - ch.bbox.y - ch.bbox.height,
                        ch.bbox.width,
                        ch.bbox.height,
                    ),
                    baseline: media.3 - ch.origin_y,
                    font_size: span.font_size,
                    geometry_quality: quality,
                    visibility: "painted_run_occlusion_unknown",
                });
                utf16 += ch.char.len_utf16() as u32;
            }
        }
        observed
    }

    /// T0 and owner-local recognition use the same recorded origins and line rule.
    /// Owner filtering precedes grouping: a row band never joins adjacent cells.
    pub fn lines(&self, indices: &[usize], tolerance_em: f32) -> Result<Vec<TextLine>, String> {
        if indices.iter().any(|i| *i >= self.spans.len()) {
            return Err("span reference out of range".into());
        }
        let mut indices = indices.to_vec();
        indices.sort_by(|a, b| {
            self.spans[*a]
                .baseline
                .total_cmp(&self.spans[*b].baseline)
                .then(self.spans[*a].bounds.x.total_cmp(&self.spans[*b].bounds.x))
                .then(a.cmp(b))
        });
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for i in indices {
            let s = &self.spans[i];
            if let Some(group) = groups.last_mut().filter(|g| {
                let first = &self.spans[g[0]];
                (first.baseline - s.baseline).abs()
                    <= tolerance_em * first.font_size.min(s.font_size)
            }) {
                group.push(i);
            } else {
                groups.push(vec![i]);
            }
        }
        Ok(groups
            .into_iter()
            .map(|mut indices| {
                indices.sort_by(|a, b| {
                    self.spans[*a]
                        .bounds
                        .x
                        .total_cmp(&self.spans[*b].bounds.x)
                        .then(a.cmp(b))
                });
                let spans: Vec<_> = indices.iter().map(|i| &self.spans[*i]).collect();
                let font = spans.iter().map(|s| s.font_size).fold(0., f32::max);
                let left = spans
                    .iter()
                    .map(|s| s.bounds.x)
                    .fold(f32::INFINITY, f32::min);
                let right = spans
                    .iter()
                    .map(|s| s.bounds.x + s.bounds.width)
                    .fold(f32::NEG_INFINITY, f32::max);
                let top = spans
                    .iter()
                    .map(|s| s.bounds.y)
                    .fold(f32::INFINITY, f32::min);
                let bottom = spans
                    .iter()
                    .map(|s| s.bounds.y + s.bounds.height)
                    .fold(f32::NEG_INFINITY, f32::max);
                TextLine {
                    baseline: spans[0].baseline,
                    source_ranges: spans.iter().map(|s| s.source.clone()).collect(),
                    preview_text: spans.iter().map(|s| s.text.as_str()).collect(),
                    disjoint: spans
                        .windows(2)
                        .any(|s| s[1].bounds.x - (s[0].bounds.x + s[0].bounds.width) > 3. * font),
                    bbox: Rect::new(left, top, right - left, bottom - top),
                    indices,
                    left,
                    right,
                    top,
                    bottom,
                    font,
                }
            })
            .collect())
    }

    pub fn all_span_indices(&self) -> Vec<usize> {
        self.spans
            .iter()
            .enumerate()
            .filter_map(|(i, span)| (!span.source.scalars.is_empty()).then_some(i))
            .collect()
    }
}

// P2 typed character plan: source references survive inferred word separators.
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ContentCharacter {
    pub span: usize,
    pub source_character: Option<usize>,
    pub source_utf16: Option<u32>,
    pub character: char,
    pub origin: f32,
    pub nominal_advance: f32,
    pub advance: f32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LineContent {
    pub text: String,
    pub characters: Vec<ContentCharacter>,
}

/// Consume observed glyph origins, not span partitioning, for word separators.
/// This is an inference: a PDF geometric gap does not prove an authored space.
pub(super) fn reconstruct_line(
    spans: &[pdf_oxide::layout::TextSpan],
    indices: &[usize],
) -> Result<LineContent, String> {
    let mut source = Vec::new();
    for &span_index in indices {
        let span = spans.get(span_index).ok_or("invalid source span")?;
        let count = span.text.chars().count();
        if source.len() + count > MAX_GLYPHS
            || span.char_x_offsets.len() != count
            || span.char_widths.len() != count
        {
            return Err("complete recorded glyph metrics unavailable".into());
        }
        let mut utf16 = 0;
        for (index, character) in span.text.chars().enumerate() {
            let origin = span.char_x_offsets[index];
            let nominal = span.char_widths[index];
            if !origin.is_finite()
                || !nominal.is_finite()
                || nominal <= 0.
                || origin < span.bbox.x - 0.5
                || origin > span.bbox.x + span.bbox.width + 0.5
            {
                return Err("unverified glyph geometry".into());
            }
            source.push(ContentCharacter {
                span: span_index,
                source_character: Some(index),
                source_utf16: Some(utf16),
                character,
                origin,
                nominal_advance: nominal,
                advance: nominal,
            });
            utf16 += character.len_utf16() as u32;
        }
    }
    // Line recognition already selected an owner and sorted source runs.
    // Do not independently interleave overlapping runs or sort their scalars.
    let mut result = Vec::with_capacity(source.len());
    for (i, current) in source.iter().enumerate() {
        let mut current = current.clone();
        if let Some(next) = source.get(i + 1) {
            let advance = next.origin - current.origin;
            if advance <= 0. || !advance.is_finite() {
                return Err("backtracking glyph origins need separate shaping".into());
            }
            let gap = advance - current.nominal_advance;
            let font = spans[current.span]
                .font_size
                .max(spans[next.span].font_size);
            if gap > font * 0.2
                && !current.character.is_whitespace()
                && !next.character.is_whitespace()
            {
                let separator = ContentCharacter {
                    span: current.span,
                    source_character: None,
                    source_utf16: None,
                    character: ' ',
                    origin: current.origin + current.nominal_advance,
                    nominal_advance: gap,
                    advance: gap,
                };
                result.push(current);
                result.push(separator);
                continue;
            }
            current.advance = advance;
        }
        result.push(current);
    }
    Ok(LineContent {
        text: result.iter().map(|c| c.character).collect(),
        characters: result,
    })
}
