//! Hierarchical, evidence-carrying state machines over spatially owned content.
use super::text_analysis::{
    line_join, paragraph_layout, LineJoin, ObservedText, ParagraphLayout, TextLine as Line,
};
use super::{
    allocation::{message, Allocation},
    value_macro::request_json,
};
use pdf_oxide::{extractors::PdfImage, geometry::Rect, layout::TextSpan};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
pub struct Policy {
    pub baseline_tolerance_em: f32,
    pub alignment_tolerance_em: f32,
    pub paragraph_gap_em: f32,
    pub max_context_depth: usize,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            baseline_tolerance_em: 0.2,
            alignment_tolerance_em: 0.75,
            paragraph_gap_em: 1.8,
            max_context_depth: 8,
        }
    }
}

pub struct Region {
    pub body: bool,
    pub context: Vec<String>,
    pub bbox: Rect,
    pub span_indices: Vec<usize>,
    pub obstacles: Vec<Rect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(super) enum ParagraphState {
    Idle,
    Accumulating,
    BoundaryCandidate,
    Confirmed,
    Deferred,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum TableState {
    Searching,
    Candidate,
    Validating,
    GeometryConfirmed,
    RecognizingCells,
    Complete,
    Deferred,
}
#[derive(Clone, Copy, Debug)]
enum TableEvent {
    CandidateFound,
    ValidateGeometry,
    GeometryAccepted,
    BeginCells,
    EndCells,
    Hold,
}

struct Trace<'a> {
    allocation: Allocation<'a>,
    context: Vec<String>,
    actor: &'static str,
    records: Vec<Value>,
}
impl Trace<'_> {
    fn record<S: std::fmt::Debug>(
        &mut self,
        from: S,
        to: S,
        event: &str,
        reason: &str,
        evidence: Value,
    ) -> Result<(), String> {
        let allocation = self.allocation;
        let mut record = request_json!(allocation,{"context":self.context,"actor":self.actor,
            "from":allocation.display(&format_args!("{from:?}"))?,
            "to":allocation.display(&format_args!("{to:?}"))?,
            "event":event,"reason":reason})?;
        allocation
            .set(&mut record, "evidence", evidence)
            .map_err(message)?;
        allocation.push(&mut self.records, record).map_err(message)
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ParagraphProposal {
    pub status: ParagraphState,
    pub reason: String,
    pub context: Vec<String>,
    pub lines: Vec<Line>,
    pub line_joins: Vec<LineJoin>,
    pub layout: Option<ParagraphLayout>,
    pub span_indices: Vec<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RegionRecognition {
    context: Vec<String>,
    pub paragraphs: Vec<ParagraphProposal>,
    transitions: Vec<Value>,
}

pub(super) struct PageRecognition {
    pub report: Value,
    pub cells: BTreeMap<(usize, usize), RegionRecognition>,
    pub body: Vec<BodyRecognition>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct BodyRecognition {
    pub bbox: Rect,
    pub span_indices: Vec<usize>,
    #[serde(flatten)]
    pub recognition: RegionRecognition,
}

fn valid_rect(r: Rect) -> bool {
    [r.x, r.y, r.width, r.height].iter().all(|v| v.is_finite()) && r.width >= 0.0 && r.height >= 0.0
}

fn make_lines(
    observed: &ObservedText,
    spans: &[TextSpan],
    span_clips: &[Option<std::sync::Arc<pdf_oxide::elements::PathClip>>],
    region: &Region,
    media_box: (f32, f32, f32, f32),
    policy: Policy,
) -> Result<Vec<Line>, String> {
    if !valid_rect(region.bbox) || region.span_indices.len() > 20000 {
        return Err("invalid region or span budget".into());
    }
    let allocation = observed.allocation();
    let indices = &region.span_indices;
    let mut sorted = allocation.copy_slice(indices).map_err(message)?;
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("duplicate span ownership in region".into());
    }
    for i in indices {
        let s = spans.get(*i).ok_or("span reference out of range")?;
        if !valid_rect(s.bbox)
            || !s.font_size.is_finite()
            || s.font_size <= 0.0
            || s.rotation_degrees != 0.0
            || s.wmode != 0
            || s.mirrored
        {
            return Err("unsupported span geometry".into());
        }
        let b = super::table_detection::span_ownership_bounds(
            s,
            span_clips.get(*i).and_then(|c| c.as_deref()),
            media_box,
        );
        if !contains(region.bbox, b) {
            return Err("span does not belong to supplied region".into());
        }
    }
    observed.lines(indices, policy.baseline_tolerance_em)
}

/// Enumerators and bullets that open a list item or a labelled block:
/// `①`, `❶`, `◦`, `1.`, `(1)`, `(가)`, `ㄱ.`, `<보기>`, `[1~3]`.
/// Inline reference marks such as `㉠` also start wrapped lines, so they
/// are not item evidence.
pub(super) fn starts_item(text: &str) -> bool {
    let text = text.trim_start();
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let closes = |rest: &str, close: char| {
        let mut rest = rest.chars();
        rest.next() == Some(close) && rest.next().is_none_or(char::is_whitespace)
    };
    match first {
        '\u{2460}'..='\u{2473}' | '\u{2776}'..='\u{277f}' => true,
        '◦' | '•' | '●' | '○' | '■' | '□' | '▪' | '◆' | '◇' | '▶' | '※' => {
            true
        }
        '0'..='9' => {
            let rest = text.trim_start_matches(|c: char| c.is_ascii_digit());
            text.len() - rest.len() <= 2 && (closes(rest, '.') || closes(rest, ')'))
        }
        // Syllable enumerators (`가.`, `다.`) are not used: a wrapped sentence
        // end such as `다. ` starts lines just as often.
        'ㄱ'..='ㅎ' => closes(chars.as_str(), '.') || closes(chars.as_str(), ')'),
        '(' => {
            let rest = chars.as_str();
            let inner =
                rest.trim_start_matches(|c: char| c.is_ascii_digit() || ('가'..='하').contains(&c));
            inner.len() < rest.len() && rest.len() - inner.len() <= 3 && closes(inner, ')')
        }
        '<' | '[' => {
            let close = if first == '<' { '>' } else { ']' };
            chars
                .as_str()
                .find(close)
                .is_some_and(|end| end > 0 && end <= 12)
        }
        _ => false,
    }
}

/// Width of the first word of `line`. Positioned PDF text often carries no
/// space characters, so a word also ends at a gap between source span boxes
/// that is at least a narrow word space wide.
fn first_word_width(
    line: &Line,
    spans: &[TextSpan],
    em: f32,
    allocation: Allocation<'_>,
) -> Result<f32, String> {
    let mut boxes = allocation
        .collect(
            line.indices
                .iter()
                .filter_map(|&i| spans.get(i))
                .map(|s| (s.bbox.x, s.bbox.x + s.bbox.width, s.text.as_str())),
        )
        .map_err(message)?;
    boxes.sort_by(|a, b| a.0.total_cmp(&b.0));
    let Some(start) = boxes.first().map(|b| b.0) else {
        return Ok(line.right - line.left);
    };
    for (k, &(left, right, text)) in boxes.iter().enumerate() {
        let body = if k == 0 { text.trim_start() } else { text };
        if let Some(space) = body.find(char::is_whitespace) {
            let chars = text.chars().count().max(1) as f32;
            let lead = (text.len() - body.len()) + space;
            let before = text[..lead].chars().count() as f32;
            return Ok(left + (right - left) * before / chars - start);
        }
        if boxes
            .get(k + 1)
            .is_some_and(|next| next.0 - right >= 0.15 * em)
        {
            return Ok(right - start);
        }
    }
    Ok(line.right - line.left)
}

struct ParagraphMachine<'a> {
    state: ParagraphState,
    owner: Rect,
    trace: Trace<'a>,
    pending: Vec<Line>,
    output: Vec<ParagraphProposal>,
    /// (left, right, baseline, block) of every observed line in this owner.
    /// A block is a run of lines without a paragraph-sized baseline gap. A
    /// paragraph's measure comes only from nearby lines of its own block that
    /// start on one of its own left edges.
    line_extents: Vec<(f32, f32, f32, usize)>,
}
impl ParagraphMachine<'_> {
    /// The widest right edge among lines of `line`'s block that start
    /// exactly on a left edge of the pending paragraph or of `line` and lie
    /// within a few lines of it. Blocks with another start, across a
    /// paragraph gap or farther away keep their own measure.
    fn measure_right(&self, line: &Line, em: f32) -> Result<f32, String> {
        let starts = self
            .trace
            .allocation
            .collect(self.pending.iter().chain([line]).map(|l| l.left))
            .map_err(message)?;
        let top = self.pending.first().map_or(line.baseline, |l| l.baseline);
        let block = self
            .line_extents
            .iter()
            .find(|e| e.2 == line.baseline && e.0 == line.left)
            .map(|e| e.3);
        Ok(self
            .line_extents
            .iter()
            .filter(|(l, _, baseline, b)| {
                Some(*b) == block
                    && *baseline >= top - 8.0 * em
                    && *baseline <= line.baseline + 8.0 * em
                    && starts.iter().any(|s| (l - s).abs() <= 0.5)
            })
            .map(|&(_, r, _, _)| r)
            .fold(line.right, f32::max))
    }
    /// A line wraps only when the next word does not fit after it. If the
    /// first word of `line` fits in the room `previous` left before the
    /// measure, the author broke the line: the paragraph ends there.
    /// Two short lines centered on the owner carry no wrap evidence.
    fn ends_short(
        &self,
        previous: &Line,
        line: &Line,
        em: f32,
        spans: &[TextSpan],
    ) -> Result<bool, String> {
        let measure = self.measure_right(line, em)?;
        let room = measure - previous.right;
        let word = first_word_width(line, spans, em, self.trace.allocation)?;
        let center = self.owner.x + self.owner.width / 2.;
        let centered = |l: &Line| ((l.left + l.right) / 2. - center).abs() <= 0.5;
        let centered_pair =
            centered(previous) && centered(line) && measure - line.right > 0.25 * em;
        // A word follows a separator about a quarter em wide.
        Ok(room >= word + 0.25 * em && !centered_pair)
    }

    fn transition(
        &mut self,
        to: ParagraphState,
        event: &str,
        reason: &str,
        evidence: Value,
    ) -> Result<(), String> {
        self.trace.record(self.state, to, event, reason, evidence)?;
        self.state = to;
        Ok(())
    }
    fn emit(&mut self, _spans: &[TextSpan], reason: &str) -> Result<(), String> {
        let allocation = self.trace.allocation;
        if self.pending.is_empty() {
            return Ok(());
        }
        let layout = paragraph_layout(&self.pending, self.owner);
        if layout.is_none() && self.state != ParagraphState::Deferred {
            self.transition(
                ParagraphState::Deferred,
                "HoldLayout",
                "line_origins_need_additional_paragraph_layout",
                request_json!(allocation, {})?,
            )?;
        }
        let lines = std::mem::take(&mut self.pending);
        let line_joins = allocation
            .collect((0..lines.len()).map(|i| {
                if i == 0 {
                    LineJoin::Start
                } else {
                    line_join(&lines[i - 1].preview_text, &lines[i].preview_text)
                }
            }))
            .map_err(message)?;
        let span_indices = allocation
            .collect(lines.iter().flat_map(|l| l.indices.iter().copied()))
            .map_err(message)?;
        allocation
            .push(
                &mut self.output,
                ParagraphProposal {
                    status: self.state,
                    reason: allocation.string(reason).map_err(message)?,
                    context: allocation.strings(&self.trace.context).map_err(message)?,
                    lines,
                    line_joins,
                    layout,
                    span_indices,
                },
            )
            .map_err(message)?;
        Ok(())
    }
    fn feed(
        &mut self,
        line: Line,
        spans: &[TextSpan],
        region: &Region,
        policy: Policy,
    ) -> Result<(), String> {
        let allocation = self.trace.allocation;
        if self.pending.is_empty() {
            self.transition(
                ParagraphState::Accumulating,
                "LineObserved",
                "start_candidate",
                request_json!(allocation,{"spanIndices":line.indices})?,
            )?;
            if line.disjoint {
                self.transition(
                    ParagraphState::Deferred,
                    "SegmentationRequired",
                    "disjoint_horizontal_runs",
                    request_json!(allocation, {})?,
                )?;
            }
            allocation.push(&mut self.pending, line).map_err(message)?;
            return Ok(());
        }
        if self.state == ParagraphState::Deferred {
            let previous = self.pending.last().unwrap();
            let em = previous.font.max(line.font);
            let clear_boundary = (region.body
                && (line.baseline - previous.baseline > policy.paragraph_gap_em * em
                    || region.obstacles.iter().any(|b| {
                        b.y >= previous.bottom - 0.5 && b.y + b.height <= line.top + 0.5
                    })))
                // Unresolved content ends with its short last line. The next
                // line is judged on its own evidence instead of being held.
                || (!previous.disjoint && !line.disjoint && self.ends_short(previous, &line, em, spans)?);
            if clear_boundary {
                self.emit(spans, "unresolved_region_ended")?;
                self.transition(
                    ParagraphState::Idle,
                    "ResumeAfterBoundary",
                    "independent_boundary_after_deferred_content",
                    request_json!(allocation, {})?,
                )?;
                self.feed(line, spans, region, policy)?;
                return Ok(());
            }
            self.transition(
                ParagraphState::Deferred,
                "LineObserved",
                "retain_unresolved_content",
                request_json!(allocation,{"spanIndices":line.indices})?,
            )?;
            allocation.push(&mut self.pending, line).map_err(message)?;
            return Ok(());
        }
        let previous = self.pending.last().unwrap();
        let em = previous.font.max(line.font);
        let gap = line.baseline - previous.baseline;
        let tol = policy.alignment_tolerance_em * em;
        let alignment = (line.left - previous.left).abs() <= tol
            || (line.right - previous.right).abs() <= tol
            || ((line.left + line.right) - (previous.left + previous.right)).abs() / 2.0 <= tol;
        let first_indent = self.pending.len() == 1
            && (region.body || line.left < previous.left)
            && line.left != previous.left
            && (previous.left - line.left).abs() <= 2.0 * em;
        let size_change = previous.font.max(line.font) / previous.font.min(line.font) > 1.25;
        let object_boundary = region
            .obstacles
            .iter()
            .any(|b| b.y >= previous.bottom - 0.5 && b.y + b.height <= line.top + 0.5);
        let short_end = !line.disjoint && self.ends_short(previous, &line, em, spans)?;
        // From the second line on, every continuation shares one left edge.
        // A different start begins the next paragraph's first line.
        let continuation_changed = self
            .pending
            .get(1)
            .is_some_and(|second| (line.left - second.left).abs() > 0.5);
        // An item marker on the paragraph's own first-line start opens the
        // next item. Inside a hanging paragraph a wrap starts on the
        // continuation edge instead, so it never matches.
        let item_start = starts_item(&line.preview_text)
            && self
                .pending
                .first()
                .is_some_and(|first| (line.left - first.left).abs() <= 0.5);
        let evidence = request_json!(allocation,{"baselineGap":gap,"fontScale":em,"alignmentCompatible":alignment,
            "firstLineIndent":first_indent,"fontSizeChanged":size_change,"objectBoundary":object_boundary,
            "previousLineShort":short_end,"continuationLeftChanged":continuation_changed,
            "itemMarker":item_start})?;
        if line.disjoint {
            self.transition(
                ParagraphState::BoundaryCandidate,
                "SegmentationRequired",
                "disjoint_horizontal_runs",
                allocation.value(&evidence).map_err(message)?,
            )?;
            self.transition(
                ParagraphState::Deferred,
                "Hold",
                "region_segmentation_unknown",
                evidence,
            )?;
        } else if object_boundary
            || gap > policy.paragraph_gap_em * em
            || (region.body && size_change)
            || short_end
            || continuation_changed
            || item_start
        {
            let reason = if object_boundary {
                if region.body {
                    "intervening_object"
                } else {
                    "intervening_image"
                }
            } else if region.body && size_change {
                "body_typographic_boundary"
            } else if gap > policy.paragraph_gap_em * em {
                "large_baseline_gap"
            } else if short_end {
                "short_last_line"
            } else if continuation_changed {
                "continuation_left_changed"
            } else {
                "item_marker"
            };
            self.transition(
                ParagraphState::BoundaryCandidate,
                "BoundaryObserved",
                reason,
                allocation.value(&evidence).map_err(message)?,
            )?;
            self.transition(
                ParagraphState::Confirmed,
                "AcceptBoundary",
                reason,
                evidence,
            )?;
            self.emit(spans, reason)?;
            self.transition(
                ParagraphState::Idle,
                "CandidateEmitted",
                "start_next_candidate",
                request_json!(allocation, {})?,
            )?;
            self.transition(
                ParagraphState::Accumulating,
                "LineObserved",
                "start_candidate",
                request_json!(allocation,{"spanIndices":line.indices})?,
            )?;
        } else if (!alignment && !first_indent) || size_change {
            self.transition(
                ParagraphState::BoundaryCandidate,
                "ConflictingClues",
                "alignment_or_size_conflict",
                allocation.value(&evidence).map_err(message)?,
            )?;
            self.transition(
                ParagraphState::Deferred,
                "Hold",
                "insufficient_boundary_evidence",
                evidence,
            )?;
        } else {
            self.transition(
                ParagraphState::Accumulating,
                "LineObserved",
                "compatible_continuation",
                evidence,
            )?;
        }
        allocation.push(&mut self.pending, line).map_err(message)?;
        Ok(())
    }
    fn finish(mut self, spans: &[TextSpan]) -> Result<RegionRecognition, String> {
        let allocation = self.trace.allocation;
        if !self.pending.is_empty() {
            if self.state != ParagraphState::Deferred {
                self.transition(
                    ParagraphState::Confirmed,
                    "RegionEnd",
                    "owner_region_ended",
                    request_json!(allocation, {})?,
                )?;
            } else {
                self.transition(
                    ParagraphState::Deferred,
                    "RegionEnd",
                    "retain_unresolved_content",
                    request_json!(allocation, {})?,
                )?;
            }
            self.emit(spans, "region_end")?;
        }
        Ok(RegionRecognition {
            context: self.trace.context,
            paragraphs: self.output,
            transitions: self.trace.records,
        })
    }
}

/// Processes exactly one spatial owner. Confirmed means a rule-based proposal, not authored PDF metadata.
fn recognize_region(
    observed: &ObservedText,
    spans: &[TextSpan],
    span_clips: &[Option<std::sync::Arc<pdf_oxide::elements::PathClip>>],
    region: &Region,
    media_box: (f32, f32, f32, f32),
    policy: Policy,
) -> Result<RegionRecognition, String> {
    let (left, bottom, right, top) = media_box;
    if ![left, bottom, right, top].iter().all(|v| v.is_finite())
        || right <= left
        || top <= bottom
        || !contains(Rect::new(0.0, 0.0, right - left, top - bottom), region.bbox)
        || region.obstacles.iter().any(|b| !contains(region.bbox, *b))
    {
        return Err("invalid region geometry or obstacle ownership".into());
    }
    if region.context.len() + 1 > policy.max_context_depth {
        return Err("recognition context depth exceeded".into());
    }
    if ![
        policy.baseline_tolerance_em,
        policy.alignment_tolerance_em,
        policy.paragraph_gap_em,
    ]
    .iter()
    .all(|v| v.is_finite() && *v > 0.0)
    {
        return Err("invalid recognition policy".into());
    }
    let allocation = observed.allocation();
    let lines = make_lines(observed, spans, span_clips, region, media_box, policy)?;
    let mut machine = ParagraphMachine {
        state: ParagraphState::Idle,
        owner: region.bbox,
        pending: Vec::new(),
        output: Vec::new(),
        line_extents: {
            let mut block = 0;
            let mut previous: Option<&Line> = None;
            let mut extents = Vec::new();
            for l in &lines {
                if previous.is_some_and(|p| {
                    l.baseline - p.baseline > policy.paragraph_gap_em * p.font.max(l.font)
                }) {
                    block += 1;
                }
                previous = Some(l);
                if !l.disjoint {
                    allocation
                        .push(&mut extents, (l.left, l.right, l.baseline, block))
                        .map_err(message)?;
                }
            }
            extents
        },
        trace: Trace {
            allocation,
            context: allocation.strings(&region.context).map_err(message)?,
            actor: "paragraph",
            records: Vec::new(),
        },
    };
    for line in lines {
        machine.feed(line, spans, region, policy)?;
    }
    machine.finish(spans)
}

fn contains(outer: Rect, inner: Rect) -> bool {
    valid_rect(outer)
        && valid_rect(inner)
        && inner.x >= outer.x - 0.5
        && inner.y >= outer.y - 0.5
        && inner.x + inner.width <= outer.x + outer.width + 0.5
        && inner.y + inner.height <= outer.y + outer.height + 0.5
}
fn rect(v: &Value) -> Result<Rect, String> {
    let field = |key: &str| {
        v[key]
            .as_f64()
            .map(|n| n as f32)
            .ok_or_else(|| format!("invalid rectangle field {key}"))
    };
    let r = Rect::new(field("x")?, field("y")?, field("width")?, field("height")?);
    if !valid_rect(r) {
        return Err("invalid rectangle".into());
    }
    Ok(r)
}
fn index(v: &Value) -> Result<usize, String> {
    v.as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or("invalid index".into())
}
fn indices(allocation: Allocation<'_>, v: &Value) -> Result<Vec<usize>, String> {
    allocation.try_collect::<_, String>(
        v.as_array()
            .ok_or("invalid reference array")?
            .iter()
            .map(index),
    )
}

fn validate_table(
    allocation: Allocation<'_>,
    table: &Value,
    span_count: usize,
    image_count: usize,
) -> Result<(), String> {
    let rows = index(&table["rows"])?;
    let cols = index(&table["columns"])?;
    if rows == 0
        || cols == 0
        || rows
            .checked_mul(cols)
            .is_none_or(|n| !(2..=4096).contains(&n))
    {
        return Err("invalid grid dimensions".into());
    }
    let mut occupied = allocation.vec(rows * cols).map_err(message)?;
    occupied.resize(rows * cols, false);
    let mut spans = BTreeSet::new();
    let mut images = BTreeSet::new();
    let bounds = rect(&table["bbox"])?;
    let axis = |v: &Value, len: usize| -> Result<Vec<f32>, String> {
        let a = allocation.try_collect::<_, String>(
            v.as_array().ok_or("axis missing")?.iter().map(|n| {
                n.as_f64()
                    .map(|n| n as f32)
                    .ok_or("invalid axis value".to_string())
            }),
        )?;
        if a.len() != len || a.iter().any(|n| !n.is_finite()) || a.windows(2).any(|w| w[1] <= w[0])
        {
            return Err("invalid grid axis".into());
        }
        Ok(a)
    };
    let xs = axis(&table["xBoundaries"], cols + 1)?;
    let ys = axis(&table["yBoundaries"], rows + 1)?;
    if (bounds.x - xs[0]).abs() > 0.5
        || (bounds.y - ys[0]).abs() > 0.5
        || (bounds.width - (xs[cols] - xs[0])).abs() > 0.5
        || (bounds.height - (ys[rows] - ys[0])).abs() > 0.5
    {
        return Err("table bounds do not match grid axes".into());
    }
    let cells = table["cells"].as_array().ok_or("cells missing")?;
    if (rows == 1 || cols == 1) && cells.len() < 2 {
        return Err("invalid grid dimensions".into());
    }
    for c in cells {
        let (r, col, rs, cs) = (
            index(&c["row"])?,
            index(&c["column"])?,
            index(&c["rowspan"])?,
            index(&c["colspan"])?,
        );
        if rs == 0
            || cs == 0
            || r >= rows
            || col >= cols
            || rs > rows - r
            || cs > cols - col
            || !contains(bounds, rect(&c["bbox"])?)
        {
            return Err("invalid cell extent".into());
        }
        let b = rect(&c["bbox"])?;
        if (b.x - xs[col]).abs() > 0.5
            || (b.y - ys[r]).abs() > 0.5
            || (b.width - (xs[col + cs] - xs[col])).abs() > 0.5
            || (b.height - (ys[r + rs] - ys[r])).abs() > 0.5
        {
            return Err("cell geometry does not match grid slot".into());
        }
        for y in r..r + rs {
            for x in col..col + cs {
                if std::mem::replace(&mut occupied[y * cols + x], true) {
                    return Err("overlapping cell ownership".into());
                }
            }
        }
        for i in indices(allocation, &c["spanIndices"])? {
            if !spans.contains(&i) {
                allocation.tree_entry::<usize, ()>().map_err(message)?;
            }
            if i >= span_count || !spans.insert(i) {
                return Err("duplicate or invalid span owner".into());
            }
        }
        for i in indices(allocation, &c["imageIndices"])? {
            if !images.contains(&i) {
                allocation.tree_entry::<usize, ()>().map_err(message)?;
            }
            if i >= image_count || !images.insert(i) {
                return Err("duplicate or invalid image owner".into());
            }
        }
    }
    if occupied.iter().any(|v| !*v) {
        return Err("uncovered grid slots".into());
    }
    super::table_topology::evidence(
        allocation,
        rows,
        cols,
        &allocation.try_collect::<_, String>(cells.iter().map(|c| {
            Ok((
                index(&c["row"])?,
                index(&c["column"])?,
                index(&c["rowspan"])?,
                index(&c["colspan"])?,
            ))
        }))?,
    )?;
    Ok(())
}

struct TableMachine<'a> {
    state: TableState,
    trace: Trace<'a>,
}
impl TableMachine<'_> {
    fn step(&mut self, event: TableEvent, reason: &str, evidence: Value) -> Result<(), String> {
        let allocation = self.trace.allocation;
        let next = match (self.state, event) {
            (TableState::Searching, TableEvent::CandidateFound) => TableState::Candidate,
            (TableState::Candidate, TableEvent::ValidateGeometry) => TableState::Validating,
            (TableState::Validating, TableEvent::GeometryAccepted) => TableState::GeometryConfirmed,
            (TableState::GeometryConfirmed, TableEvent::BeginCells) => TableState::RecognizingCells,
            (TableState::RecognizingCells, TableEvent::EndCells) => TableState::Complete,
            (TableState::Validating, TableEvent::Hold) => TableState::Deferred,
            _ => return Err("invalid table recognition transition".into()),
        };
        self.trace.record(
            self.state,
            next,
            &allocation
                .display(&format_args!("{event:?}"))
                .map_err(message)?,
            reason,
            evidence,
        )?;
        self.state = next;
        Ok(())
    }
}

fn parent_value(
    allocation: Allocation<'_>,
    parent: Option<(usize, usize)>,
) -> Result<Value, String> {
    match parent {
        Some((table, cell)) => Ok(request_json!(allocation,{"table":table,"cell":cell})?),
        None => Ok(Value::Null),
    }
}

pub fn recognize_page(
    observed: &ObservedText,
    spans: &[TextSpan],
    span_clips: &[Option<std::sync::Arc<pdf_oxide::elements::PathClip>>],
    images: &[PdfImage],
    detection: &Value,
    media_box: (f32, f32, f32, f32),
    policy: Policy,
) -> Result<PageRecognition, String> {
    let allocation = observed.allocation();
    let (left, bottom, right, top) = media_box;
    if ![left, bottom, right, top].iter().all(|v| v.is_finite()) || right <= left || top <= bottom {
        return Err("invalid recognition page".into());
    }
    let page_bounds = Rect::new(0.0, 0.0, right - left, top - bottom);
    let image_boxes = allocation.try_collect::<_, String>(images.iter().map(|i| {
        let b = i.bbox().ok_or("image lacks geometry")?;
        Ok(Rect::new(
            b.x - left,
            top - b.y - b.height,
            b.width,
            b.height,
        ))
    }))?;
    let candidates = detection["tables"]
        .as_array()
        .ok_or("table candidates missing")?;
    if candidates.len() > 128 {
        return Err("table recognition budget exceeded".into());
    }
    let validations = allocation
        .collect(
            candidates
                .iter()
                .map(|t| validate_table(allocation, t, spans.len(), images.len())),
        )
        .map_err(message)?;
    let valid_ids = allocation
        .collect(
            validations
                .iter()
                .enumerate()
                .filter_map(|(i, v)| v.is_ok().then_some(i)),
        )
        .map_err(message)?;
    let geometry = allocation.try_collect::<_, String>(valid_ids.iter().map(|i| {
        let t = &candidates[*i];
        Ok((
            rect(&t["bbox"])?,
            allocation.try_collect::<_, String>(
                t["cells"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| rect(&c["bbox"])),
            )?,
        ))
    }))?;
    let composed = super::table_hierarchy::compose(allocation, &geometry)?;
    let mut placements = allocation.vec(candidates.len()).map_err(message)?;
    placements.resize_with(candidates.len(), || super::table_hierarchy::Placement {
        parent: None,
        depth: 0,
        embedding: None,
        deferred: None,
    });
    for (local, mut placement) in composed.into_iter().enumerate() {
        placement.parent = placement.parent.map(|(p, c)| (valid_ids[p], c));
        placements[valid_ids[local]] = placement;
    }
    let mut tables = Vec::new();
    let mut regions = Vec::new();
    let mut cells = BTreeMap::new();
    let mut reserved = BTreeSet::new();
    let mut span_owners = BTreeMap::<usize, usize>::new();
    let mut image_owners = BTreeMap::<usize, usize>::new();
    for t in candidates {
        if let Some(cells) = t["cells"].as_array() {
            for c in cells {
                if let Ok(ids) = indices(allocation, &c["spanIndices"]) {
                    for i in ids {
                        if !span_owners.contains_key(&i) {
                            allocation.tree_entry::<usize, usize>().map_err(message)?;
                        }
                        *span_owners.entry(i).or_default() += 1;
                    }
                }
                if let Ok(ids) = indices(allocation, &c["imageIndices"]) {
                    for i in ids {
                        if !image_owners.contains_key(&i) {
                            allocation.tree_entry::<usize, usize>().map_err(message)?;
                        }
                        *image_owners.entry(i).or_default() += 1;
                    }
                }
            }
        }
    }
    let ambiguous = detection["ambiguousContent"]
        .as_array()
        .ok_or("ambiguity records missing")?;
    for a in ambiguous {
        if a["kind"] == "span" {
            let i = index(&a["index"])?;
            if !reserved.contains(&i) {
                allocation.tree_entry::<usize, ()>().map_err(message)?;
            }
            reserved.insert(i);
        }
    }
    for (ti, t) in candidates.iter().enumerate() {
        let mut chain = allocation.vec(17).map_err(message)?;
        chain.push(ti);
        let mut at = ti;
        for _ in 0..16 {
            let Some((parent, _)) = placements[at].parent else {
                break;
            };
            chain.push(parent);
            at = parent;
        }
        chain.reverse();
        let mut context = allocation.vec(chain.len() * 2).map_err(message)?;
        context.push(allocation.string("page").map_err(message)?);
        for (level, &table) in chain.iter().enumerate() {
            if level > 0 {
                context.push(
                    allocation
                        .id("cell", &[placements[table].parent.unwrap().1])
                        .map_err(message)?,
                );
            }
            context.push(allocation.id("table", &[table]).map_err(message)?);
        }
        let mut machine = TableMachine {
            state: TableState::Searching,
            trace: Trace {
                allocation,
                context: allocation.strings(&context).map_err(message)?,
                actor: "table",
                records: Vec::new(),
            },
        };
        machine.step(
            TableEvent::CandidateFound,
            "spatial_detector",
            request_json!(allocation,{"candidate":ti})?,
        )?;
        machine.step(
            TableEvent::ValidateGeometry,
            "check_grid_and_owners",
            request_json!(allocation, {})?,
        )?;
        let initial_validation = match &validations[ti] {
            Ok(()) => Ok(()),
            Err(reason) => Err(allocation.string(reason).map_err(message)?),
        };
        let validation = initial_validation.and_then(|_| {
            if let Some(reason) = &placements[ti].deferred {
                return Err(allocation.string(reason).map_err(message)?);
            }
            // A single row/column is still a table when the same geometry,
            // topology and unique-content ownership checks hold. The previous
            // root-only dimension veto discarded valid local cell results.
            if context.len() + 2 > policy.max_context_depth {
                return Err("recognition context depth exceeded".into());
            }
            if t["hierarchy"]["status"] == "Deferred" {
                return Err("detector deferred table composition".into());
            }
            if !t["hierarchy"].is_null() {
                let expected = parent_value(allocation, placements[ti].parent)?;
                if t["hierarchy"]["parent"] != expected {
                    return Err("detector parent disagrees with cell-slot geometry".into());
                }
            }
            if !contains(page_bounds, rect(&t["bbox"])?) {
                return Err("table outside page".into());
            }
            let refs = t["cells"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|c| c["spanIndices"].as_array().unwrap());
            if refs
                .into_iter()
                .any(|i| index(i).is_ok_and(|i| span_owners.get(&i) != Some(&1)))
            {
                return Err("cross-table span ownership".into());
            }
            let image_refs = t["cells"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|c| c["imageIndices"].as_array().unwrap());
            if image_refs
                .into_iter()
                .any(|i| index(i).is_ok_and(|i| image_owners.get(&i) != Some(&1)))
            {
                return Err("cross-table image ownership".into());
            }
            let bounds = rect(&t["bbox"])?;
            for a in ambiguous {
                let i = index(&a["index"])?;
                let b = if a["kind"] == "span" {
                    let s = spans.get(i).ok_or("invalid ambiguous span reference")?;
                    Rect::new(
                        s.bbox.x - left,
                        top - s.bbox.y - s.bbox.height,
                        s.bbox.width,
                        s.bbox.height,
                    )
                } else if a["kind"] == "image" {
                    *image_boxes
                        .get(i)
                        .ok_or("invalid ambiguous image reference")?
                } else {
                    return Err("unknown ambiguity kind".into());
                };
                if b.x < bounds.x + bounds.width
                    && bounds.x < b.x + b.width
                    && b.y < bounds.y + bounds.height
                    && bounds.y < b.y + b.height
                {
                    return Err("ambiguous spatial ownership requires review".into());
                }
            }
            Ok(())
        });
        // Reserve content even in a deferred table; never reinterpret it as page prose.
        if let Some(cells) = t["cells"].as_array() {
            for c in cells {
                if let Ok(ids) = indices(allocation, &c["spanIndices"]) {
                    for i in ids.into_iter().filter(|i| *i < spans.len()) {
                        if !reserved.contains(&i) {
                            allocation.tree_entry::<usize, ()>().map_err(message)?;
                        }
                        reserved.insert(i);
                    }
                }
            }
        }
        if let Err(reason) = validation {
            machine.step(TableEvent::Hold, &reason, request_json!(allocation, {})?)?;
            let mut table =
                request_json!(allocation,{"candidate":ti,"status":"Deferred","reason":reason})?;
            allocation
                .set(
                    &mut table,
                    "transitions",
                    Value::Array(machine.trace.records),
                )
                .map_err(message)?;
            allocation.push(&mut tables, table).map_err(message)?;
            continue;
        }
        machine.step(
            TableEvent::GeometryAccepted,
            "consistent_grid_and_unique_owners",
            request_json!(allocation,{"rows":t["rows"],"columns":t["columns"]})?,
        )?;
        machine.step(
            TableEvent::BeginCells,
            "enter_child_contexts",
            request_json!(allocation, {})?,
        )?;
        let mut children = Vec::new();
        for (ci, c) in t["cells"].as_array().unwrap().iter().enumerate() {
            let mut cell_context = allocation.strings(&context).map_err(message)?;
            allocation
                .push(
                    &mut cell_context,
                    allocation.id("cell", &[ci]).map_err(message)?,
                )
                .map_err(message)?;
            let mut obstacles = allocation
                .collect(
                    indices(allocation, &c["imageIndices"])?
                        .iter()
                        .map(|i| image_boxes[*i]),
                )
                .map_err(message)?;
            let child_tables = allocation
                .collect(
                    placements
                        .iter()
                        .enumerate()
                        .filter_map(|(child, p)| (p.parent == Some((ti, ci))).then_some(child)),
                )
                .map_err(message)?;
            for child in &child_tables {
                allocation
                    .push(&mut obstacles, rect(&candidates[*child]["bbox"])?)
                    .map_err(message)?;
            }
            let region = Region {
                body: false,
                context: cell_context,
                bbox: rect(&c["bbox"])?,
                span_indices: indices(allocation, &c["spanIndices"])?,
                obstacles,
            };
            let result = recognize_region(observed, spans, span_clips, &region, media_box, policy)?;
            allocation.reserve(&mut children, 1).map_err(message)?;
            children.push(request_json!(allocation,{"cell":ci,"row":c["row"],"column":c["column"],"imageIndices":c["imageIndices"],"childTableIndices":child_tables,"recognition":result})?);
            allocation
                .tree_entry::<(usize, usize), RegionRecognition>()
                .map_err(message)?;
            cells.insert((ti, ci), result);
        }
        machine.step(
            TableEvent::EndCells,
            "child_contexts_closed",
            request_json!(allocation,{"cells":children.len()})?,
        )?;
        let parent = parent_value(allocation, placements[ti].parent)?;
        let mut table = request_json!(allocation,{"candidate":ti,"status":"GeometryConfirmed","semanticStatus":"unverified",
            "parent":parent,
            "depth":placements[ti].depth})?;
        // These JSON trees already own their values. Passing them through
        // json! serializes and allocates every nested glyph/transition again.
        allocation
            .set(&mut table, "cells", Value::Array(children))
            .map_err(message)?;
        allocation
            .set(
                &mut table,
                "transitions",
                Value::Array(machine.trace.records),
            )
            .map_err(message)?;
        allocation.push(&mut tables, table).map_err(message)?;
    }
    for _ in 0..=16 {
        let mut changed = false;
        for i in 0..placements.len() {
            if let Some((parent, _)) = placements[i].parent {
                if (tables[i]["status"] == "Deferred") != (tables[parent]["status"] == "Deferred") {
                    let held = if tables[i]["status"] == "Deferred" {
                        parent
                    } else {
                        i
                    };
                    allocation
                        .set(
                            &mut tables[held],
                            "status",
                            allocation.value("Deferred").map_err(message)?,
                        )
                        .map_err(message)?;
                    allocation
                        .set(
                            &mut tables[held],
                            "reason",
                            allocation
                                .value("unresolved nested table context")
                                .map_err(message)?,
                        )
                        .map_err(message)?;
                    allocation.push(tables[held]["transitions"].as_array_mut().unwrap(),request_json!(allocation,{
                        "actor":"table","from":"Complete","to":"Deferred","event":"HoldSubtree",
                        "reason":"unresolved nested table context","evidence":{"child":i,"parent":parent}
                    })?).map_err(message)?;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let remaining = allocation
        .collect(
            spans
                .iter()
                .enumerate()
                .filter(|(i, s)| {
                    !s.text.is_empty() && s.rotation_degrees == 0.0 && !reserved.contains(i)
                })
                .map(|(i, _)| i),
        )
        .map_err(message)?;
    let mut body = Vec::new();
    let partitions =
        super::body_regions::partition(allocation, spans, span_clips, &remaining, media_box);
    allocation.0.check().map_err(|e| e.to_string())?;
    let partition_error = match &partitions {
        Err(reason) => Some(allocation.string(reason).map_err(message)?),
        Ok(_) => None,
    };
    let partitions = match partitions {
        Ok(partitions) => partitions,
        Err(_) if remaining.is_empty() => Vec::new(),
        Err(_) => allocation
            .collect([super::body_regions::Partition {
                bounds: page_bounds,
                spans: remaining,
            }])
            .map_err(message)?,
    };
    let mut obstacles = allocation.copy_slice(&image_boxes).map_err(message)?;
    for t in candidates {
        if let Ok(b) = rect(&t["bbox"]) {
            allocation.push(&mut obstacles, b).map_err(message)?;
        }
    }
    for (i, partition) in partitions.into_iter().enumerate() {
        // Only the geometric intersection belongs to this owner. The original
        // image/table and its paint are not clipped or modified.
        let clipped = allocation
            .collect(obstacles.iter().copied().filter_map(|b| {
                let x = b.x.max(partition.bounds.x);
                let y = b.y.max(partition.bounds.y);
                let right = (b.x + b.width).min(partition.bounds.x + partition.bounds.width);
                let bottom = (b.y + b.height).min(partition.bounds.y + partition.bounds.height);
                (right > x && bottom > y).then(|| Rect::new(x, y, right - x, bottom - y))
            }))
            .map_err(message)?;
        let region = Region {
            body: true,
            context: allocation
                .collect([
                    allocation.string("page").map_err(message)?,
                    allocation.id("body-region", &[i]).map_err(message)?,
                ])
                .map_err(message)?,
            bbox: partition.bounds,
            span_indices: allocation.copy_slice(&partition.spans).map_err(message)?,
            obstacles: clipped,
        };
        let mut recognized =
            recognize_region(observed, spans, span_clips, &region, media_box, policy)?;
        if let Some(reason) = &partition_error {
            for p in &mut recognized.paragraphs {
                p.status = ParagraphState::Deferred;
                p.reason = allocation.string(reason).map_err(message)?;
            }
        }
        allocation
            .push(
                &mut body,
                BodyRecognition {
                    bbox: region.bbox,
                    span_indices: partition.spans,
                    recognition: recognized,
                },
            )
            .map_err(message)?;
    }
    for region in &body {
        allocation
            .push(&mut regions, allocation.value(region).map_err(message)?)
            .map_err(message)?;
    }
    let mut report = request_json!(allocation,{"algorithm":"hierarchical-efsm-v4-body-regions","acceptance":"rule-based proposals, not authored paragraph/table metadata",
        "policy":{"baselineToleranceEm":policy.baseline_tolerance_em,"alignmentToleranceEm":policy.alignment_tolerance_em,
            "paragraphGapEm":policy.paragraph_gap_em,"maxContextDepth":policy.max_context_depth},
        "unresolvedContent":ambiguous,"emission":"typed proposals; actual IR consumption is reported by lowering",
        "limits":["body whitespace partitions require repeated gutter evidence; ambiguous runs retained","nested cell-slot structure inferred; lowering may preserve unresolved content as positioned spans","tagged semantics not reconstructed"]})?;
    allocation
        .set(&mut report, "tables", Value::Array(tables))
        .map_err(message)?;
    allocation
        .set(&mut report, "bodyRegions", Value::Array(regions))
        .map_err(message)?;
    Ok(PageRecognition {
        report,
        cells,
        body,
    })
}
