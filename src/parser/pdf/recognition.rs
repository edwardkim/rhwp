//! Hierarchical, evidence-carrying state machines over spatially owned content.
use super::text_analysis::{
    line_join, paragraph_layout, LineJoin, ObservedText, ParagraphLayout, TextLine as Line,
};
use pdf_oxide::{extractors::PdfImage, geometry::Rect, layout::TextSpan};
use serde::Serialize;
use serde_json::{json, Value};
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

struct Trace {
    context: Vec<String>,
    actor: &'static str,
    records: Vec<Value>,
}
impl Trace {
    fn record<S: std::fmt::Debug>(
        &mut self,
        from: S,
        to: S,
        event: &str,
        reason: &str,
        evidence: Value,
    ) {
        self.records.push(json!({"context":self.context,"actor":self.actor,
            "from":format!("{from:?}"),"to":format!("{to:?}"),"event":event,"reason":reason,"evidence":evidence}));
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
    let indices = region.span_indices.clone();
    let unique: BTreeSet<_> = indices.iter().copied().collect();
    if unique.len() != indices.len() {
        return Err("duplicate span ownership in region".into());
    }
    for i in &indices {
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
    observed.lines(&indices, policy.baseline_tolerance_em)
}

struct ParagraphMachine {
    state: ParagraphState,
    owner: Rect,
    trace: Trace,
    pending: Vec<Line>,
    output: Vec<ParagraphProposal>,
}
impl ParagraphMachine {
    fn transition(&mut self, to: ParagraphState, event: &str, reason: &str, evidence: Value) {
        self.trace.record(self.state, to, event, reason, evidence);
        self.state = to;
    }
    fn emit(&mut self, _spans: &[TextSpan], reason: &str) {
        if self.pending.is_empty() {
            return;
        }
        let layout = paragraph_layout(&self.pending, self.owner);
        if layout.is_none() && self.state != ParagraphState::Deferred {
            self.transition(
                ParagraphState::Deferred,
                "HoldLayout",
                "line_origins_need_additional_paragraph_layout",
                json!({}),
            );
        }
        let lines = std::mem::take(&mut self.pending);
        let line_joins = (0..lines.len())
            .map(|i| {
                if i == 0 {
                    LineJoin::Start
                } else {
                    line_join(&lines[i - 1].preview_text, &lines[i].preview_text)
                }
            })
            .collect();
        let span_indices = lines
            .iter()
            .flat_map(|l| l.indices.iter().copied())
            .collect();
        self.output.push(ParagraphProposal {
            status: self.state,
            reason: reason.into(),
            context: self.trace.context.clone(),
            lines,
            line_joins,
            layout,
            span_indices,
        });
    }
    fn feed(&mut self, line: Line, spans: &[TextSpan], region: &Region, policy: Policy) {
        if self.pending.is_empty() {
            self.transition(
                ParagraphState::Accumulating,
                "LineObserved",
                "start_candidate",
                json!({"spanIndices":line.indices}),
            );
            if line.disjoint {
                self.transition(
                    ParagraphState::Deferred,
                    "SegmentationRequired",
                    "disjoint_horizontal_runs",
                    json!({}),
                );
            }
            self.pending.push(line);
            return;
        }
        if self.state == ParagraphState::Deferred {
            let previous = self.pending.last().unwrap();
            let clear_boundary = region.body
                && (line.baseline - previous.baseline
                    > policy.paragraph_gap_em * previous.font.max(line.font)
                    || region
                        .obstacles
                        .iter()
                        .any(|b| b.y >= previous.bottom - 0.5 && b.y + b.height <= line.top + 0.5));
            if clear_boundary {
                self.emit(spans, "unresolved_region_ended");
                self.transition(
                    ParagraphState::Idle,
                    "ResumeAfterBoundary",
                    "independent_boundary_after_deferred_content",
                    json!({}),
                );
                self.feed(line, spans, region, policy);
                return;
            }
            self.transition(
                ParagraphState::Deferred,
                "LineObserved",
                "retain_unresolved_content",
                json!({"spanIndices":line.indices}),
            );
            self.pending.push(line);
            return;
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
        let evidence = json!({"baselineGap":gap,"fontScale":em,"alignmentCompatible":alignment,
            "firstLineIndent":first_indent,"fontSizeChanged":size_change,"objectBoundary":object_boundary});
        if line.disjoint {
            self.transition(
                ParagraphState::BoundaryCandidate,
                "SegmentationRequired",
                "disjoint_horizontal_runs",
                evidence.clone(),
            );
            self.transition(
                ParagraphState::Deferred,
                "Hold",
                "region_segmentation_unknown",
                evidence,
            );
        } else if object_boundary
            || gap > policy.paragraph_gap_em * em
            || (region.body && size_change)
        {
            let reason = if object_boundary {
                if region.body {
                    "intervening_object"
                } else {
                    "intervening_image"
                }
            } else if region.body && size_change {
                "body_typographic_boundary"
            } else {
                "large_baseline_gap"
            };
            self.transition(
                ParagraphState::BoundaryCandidate,
                "BoundaryObserved",
                reason,
                evidence.clone(),
            );
            self.transition(
                ParagraphState::Confirmed,
                "AcceptBoundary",
                reason,
                evidence,
            );
            self.emit(spans, reason);
            self.transition(
                ParagraphState::Idle,
                "CandidateEmitted",
                "start_next_candidate",
                json!({}),
            );
            self.transition(
                ParagraphState::Accumulating,
                "LineObserved",
                "start_candidate",
                json!({"spanIndices":line.indices}),
            );
        } else if (!alignment && !first_indent) || size_change {
            self.transition(
                ParagraphState::BoundaryCandidate,
                "ConflictingClues",
                "alignment_or_size_conflict",
                evidence.clone(),
            );
            self.transition(
                ParagraphState::Deferred,
                "Hold",
                "insufficient_boundary_evidence",
                evidence,
            );
        } else {
            self.transition(
                ParagraphState::Accumulating,
                "LineObserved",
                "compatible_continuation",
                evidence,
            );
        }
        self.pending.push(line);
    }
    fn finish(mut self, spans: &[TextSpan]) -> RegionRecognition {
        if !self.pending.is_empty() {
            if self.state != ParagraphState::Deferred {
                self.transition(
                    ParagraphState::Confirmed,
                    "RegionEnd",
                    "owner_region_ended",
                    json!({}),
                );
            } else {
                self.transition(
                    ParagraphState::Deferred,
                    "RegionEnd",
                    "retain_unresolved_content",
                    json!({}),
                );
            }
            self.emit(spans, "region_end");
        }
        RegionRecognition {
            context: self.trace.context,
            paragraphs: self.output,
            transitions: self.trace.records,
        }
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
    let lines = make_lines(observed, spans, span_clips, region, media_box, policy)?;
    let mut machine = ParagraphMachine {
        state: ParagraphState::Idle,
        owner: region.bbox,
        pending: Vec::new(),
        output: Vec::new(),
        trace: Trace {
            context: region.context.clone(),
            actor: "paragraph",
            records: Vec::new(),
        },
    };
    for line in lines {
        machine.feed(line, spans, region, policy);
    }
    Ok(machine.finish(spans))
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
fn indices(v: &Value) -> Result<Vec<usize>, String> {
    v.as_array()
        .ok_or("invalid reference array")?
        .iter()
        .map(index)
        .collect()
}

fn validate_table(table: &Value, span_count: usize, image_count: usize) -> Result<(), String> {
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
    let mut occupied = vec![false; rows * cols];
    let mut spans = BTreeSet::new();
    let mut images = BTreeSet::new();
    let bounds = rect(&table["bbox"])?;
    let axis = |v: &Value, len: usize| -> Result<Vec<f32>, String> {
        let a: Vec<_> = v
            .as_array()
            .ok_or("axis missing")?
            .iter()
            .map(|n| {
                n.as_f64()
                    .map(|n| n as f32)
                    .ok_or("invalid axis value".to_string())
            })
            .collect::<Result<_, _>>()?;
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
        for i in indices(&c["spanIndices"])? {
            if i >= span_count || !spans.insert(i) {
                return Err("duplicate or invalid span owner".into());
            }
        }
        for i in indices(&c["imageIndices"])? {
            if i >= image_count || !images.insert(i) {
                return Err("duplicate or invalid image owner".into());
            }
        }
    }
    if occupied.iter().any(|v| !*v) {
        return Err("uncovered grid slots".into());
    }
    super::table_topology::evidence(
        rows,
        cols,
        &cells
            .iter()
            .map(|c| {
                Ok((
                    index(&c["row"])?,
                    index(&c["column"])?,
                    index(&c["rowspan"])?,
                    index(&c["colspan"])?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?,
    )?;
    Ok(())
}

struct TableMachine {
    state: TableState,
    trace: Trace,
}
impl TableMachine {
    fn step(&mut self, event: TableEvent, reason: &str, evidence: Value) -> Result<(), String> {
        let next = match (self.state, event) {
            (TableState::Searching, TableEvent::CandidateFound) => TableState::Candidate,
            (TableState::Candidate, TableEvent::ValidateGeometry) => TableState::Validating,
            (TableState::Validating, TableEvent::GeometryAccepted) => TableState::GeometryConfirmed,
            (TableState::GeometryConfirmed, TableEvent::BeginCells) => TableState::RecognizingCells,
            (TableState::RecognizingCells, TableEvent::EndCells) => TableState::Complete,
            (TableState::Validating, TableEvent::Hold) => TableState::Deferred,
            _ => return Err("invalid table recognition transition".into()),
        };
        self.trace
            .record(self.state, next, &format!("{event:?}"), reason, evidence);
        self.state = next;
        Ok(())
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
    let (left, bottom, right, top) = media_box;
    if ![left, bottom, right, top].iter().all(|v| v.is_finite()) || right <= left || top <= bottom {
        return Err("invalid recognition page".into());
    }
    let page_bounds = Rect::new(0.0, 0.0, right - left, top - bottom);
    let image_boxes: Vec<_> = images
        .iter()
        .map(|i| {
            let b = i.bbox().ok_or("image lacks geometry")?;
            Ok(Rect::new(
                b.x - left,
                top - b.y - b.height,
                b.width,
                b.height,
            ))
        })
        .collect::<Result<_, String>>()?;
    let candidates = detection["tables"]
        .as_array()
        .ok_or("table candidates missing")?;
    if candidates.len() > 128 {
        return Err("table recognition budget exceeded".into());
    }
    let validations: Vec<_> = candidates
        .iter()
        .map(|t| validate_table(t, spans.len(), images.len()))
        .collect();
    let valid_ids: Vec<_> = validations
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.is_ok().then_some(i))
        .collect();
    let geometry = valid_ids
        .iter()
        .map(|i| {
            let t = &candidates[*i];
            Ok((
                rect(&t["bbox"])?,
                t["cells"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| rect(&c["bbox"]))
                    .collect::<Result<Vec<_>, String>>()?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let composed = super::table_hierarchy::compose(&geometry)?;
    let mut placements = vec![
        super::table_hierarchy::Placement {
            parent: None,
            depth: 0,
            embedding: None,
            deferred: None
        };
        candidates.len()
    ];
    for (local, &global) in valid_ids.iter().enumerate() {
        placements[global] = composed[local].clone();
        placements[global].parent = composed[local].parent.map(|(p, c)| (valid_ids[p], c));
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
                if let Ok(ids) = indices(&c["spanIndices"]) {
                    for i in ids {
                        *span_owners.entry(i).or_default() += 1;
                    }
                }
                if let Ok(ids) = indices(&c["imageIndices"]) {
                    for i in ids {
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
            reserved.insert(index(&a["index"])?);
        }
    }
    for (ti, t) in candidates.iter().enumerate() {
        let mut chain = vec![ti];
        let mut at = ti;
        for _ in 0..16 {
            let Some((parent, _)) = placements[at].parent else {
                break;
            };
            chain.push(parent);
            at = parent;
        }
        chain.reverse();
        let mut context = vec!["page".to_string()];
        for (level, &table) in chain.iter().enumerate() {
            if level > 0 {
                context.push(format!("cell:{}", placements[table].parent.unwrap().1));
            }
            context.push(format!("table:{table}"));
        }
        let mut machine = TableMachine {
            state: TableState::Searching,
            trace: Trace {
                context: context.clone(),
                actor: "table",
                records: Vec::new(),
            },
        };
        machine.step(
            TableEvent::CandidateFound,
            "spatial_detector",
            json!({"candidate":ti}),
        )?;
        machine.step(
            TableEvent::ValidateGeometry,
            "check_grid_and_owners",
            json!({}),
        )?;
        let validation = validations[ti].clone().and_then(|_| {
            if let Some(reason) = &placements[ti].deferred {
                return Err(reason.clone());
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
                let expected = placements[ti]
                    .parent
                    .map(|(table, cell)| json!({"table":table,"cell":cell}));
                if t["hierarchy"]["parent"] != json!(expected) {
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
                if let Ok(ids) = indices(&c["spanIndices"]) {
                    reserved.extend(ids.into_iter().filter(|i| *i < spans.len()));
                }
            }
        }
        if let Err(reason) = validation {
            machine.step(TableEvent::Hold, &reason, json!({}))?;
            tables.push(json!({"candidate":ti,"status":"Deferred","reason":reason,"transitions":machine.trace.records}));
            continue;
        }
        machine.step(
            TableEvent::GeometryAccepted,
            "consistent_grid_and_unique_owners",
            json!({"rows":t["rows"],"columns":t["columns"]}),
        )?;
        machine.step(TableEvent::BeginCells, "enter_child_contexts", json!({}))?;
        let mut children = Vec::new();
        for (ci, c) in t["cells"].as_array().unwrap().iter().enumerate() {
            let mut cell_context = context.clone();
            cell_context.push(format!("cell:{ci}"));
            let mut obstacles: Vec<_> = indices(&c["imageIndices"])?
                .iter()
                .map(|i| image_boxes[*i])
                .collect();
            let child_tables: Vec<_> = placements
                .iter()
                .enumerate()
                .filter_map(|(child, p)| (p.parent == Some((ti, ci))).then_some(child))
                .collect();
            for child in &child_tables {
                obstacles.push(rect(&candidates[*child]["bbox"])?);
            }
            let region = Region {
                body: false,
                context: cell_context,
                bbox: rect(&c["bbox"])?,
                span_indices: indices(&c["spanIndices"])?,
                obstacles,
            };
            let result = recognize_region(observed, spans, span_clips, &region, media_box, policy)?;
            children
                .push(json!({"cell":ci,"row":c["row"],"column":c["column"],"imageIndices":c["imageIndices"],"childTableIndices":child_tables,"recognition":result}));
            cells.insert((ti, ci), result);
        }
        machine.step(
            TableEvent::EndCells,
            "child_contexts_closed",
            json!({"cells":children.len()}),
        )?;
        let mut table = json!({"candidate":ti,"status":"GeometryConfirmed","semanticStatus":"unverified",
            "parent":placements[ti].parent.map(|(table,cell)|json!({"table":table,"cell":cell})),
            "depth":placements[ti].depth});
        // These JSON trees already own their values. Passing them through
        // json! serializes and allocates every nested glyph/transition again.
        table["cells"] = Value::Array(children);
        table["transitions"] = Value::Array(machine.trace.records);
        tables.push(table);
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
                    tables[held]["status"] = json!("Deferred");
                    tables[held]["reason"] = json!("unresolved nested table context");
                    tables[held]["transitions"].as_array_mut().unwrap().push(json!({
                        "actor":"table","from":"Complete","to":"Deferred","event":"HoldSubtree",
                        "reason":"unresolved nested table context","evidence":{"child":i,"parent":parent}
                    }));
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let remaining: Vec<_> = spans
        .iter()
        .enumerate()
        .filter(|(i, s)| !s.text.is_empty() && s.rotation_degrees == 0.0 && !reserved.contains(i))
        .map(|(i, _)| i)
        .collect();
    let mut body = Vec::new();
    let partitions = super::body_regions::partition(spans, span_clips, &remaining, media_box);
    let partition_error = partitions.as_ref().err().cloned();
    let partitions = partitions.unwrap_or_else(|_| {
        if remaining.is_empty() {
            Vec::new()
        } else {
            vec![super::body_regions::Partition {
                bounds: page_bounds,
                spans: remaining,
            }]
        }
    });
    for (i, partition) in partitions.into_iter().enumerate() {
        let mut obstacles: Vec<_> = image_boxes.iter().copied().collect();
        for t in candidates {
            if let Ok(b) = rect(&t["bbox"]) {
                obstacles.push(b);
            }
        }
        // Only the geometric intersection belongs to this owner. The original
        // image/table and its paint are not clipped or modified.
        let clipped = obstacles
            .into_iter()
            .filter_map(|b| {
                let x = b.x.max(partition.bounds.x);
                let y = b.y.max(partition.bounds.y);
                let right = (b.x + b.width).min(partition.bounds.x + partition.bounds.width);
                let bottom = (b.y + b.height).min(partition.bounds.y + partition.bounds.height);
                (right > x && bottom > y).then(|| Rect::new(x, y, right - x, bottom - y))
            })
            .collect();
        let region = Region {
            body: true,
            context: vec!["page".into(), format!("body-region:{i}")],
            bbox: partition.bounds,
            span_indices: partition.spans.clone(),
            obstacles: clipped,
        };
        let mut recognized =
            recognize_region(observed, spans, span_clips, &region, media_box, policy)?;
        if let Some(reason) = &partition_error {
            for p in &mut recognized.paragraphs {
                p.status = ParagraphState::Deferred;
                p.reason = reason.clone();
            }
        }
        body.push(BodyRecognition {
            bbox: region.bbox,
            span_indices: partition.spans,
            recognition: recognized,
        });
    }
    regions.extend(body.iter().map(|r| json!(r)));
    let mut report = json!({"algorithm":"hierarchical-efsm-v4-body-regions","acceptance":"rule-based proposals, not authored paragraph/table metadata",
        "policy":{"baselineToleranceEm":policy.baseline_tolerance_em,"alignmentToleranceEm":policy.alignment_tolerance_em,
            "paragraphGapEm":policy.paragraph_gap_em,"maxContextDepth":policy.max_context_depth},
        "unresolvedContent":ambiguous,"emission":"typed proposals; actual IR consumption is reported by lowering",
        "limits":["body whitespace partitions require repeated gutter evidence; ambiguous runs retained","nested cell-slot structure inferred; lowering may preserve unresolved content as positioned spans","tagged semantics not reconstructed"]});
    report["tables"] = Value::Array(tables);
    report["bodyRegions"] = Value::Array(regions);
    Ok(PageRecognition {
        report,
        cells,
        body,
    })
}
