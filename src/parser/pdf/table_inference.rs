//! Page-scoped observation and hypothesis ledger. Nothing here asserts authored cells.
//! Original paint is left to lowering; absent paint never proves a cell passage.
use super::{
    allocation::Allocation,
    text_analysis::{ObservedText, MAX_GLYPHS},
    value_macro::request_json,
    PdfImportError,
};
use pdf_oxide::import_limits::ImportBudget;
use pdf_oxide::{
    elements::{PathClipObservations, PathContent, PathOperation},
    extractors::PdfImage,
    geometry::Rect,
    layout::TextSpan,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

const EPS: f32 = 0.5;
const MAX_REGIONS: usize = 512;
const MAX_CANDIDATES: usize = 128;
const MAX_TILES: usize = 4096;
const MAX_WORK: usize = 1_000_000;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Block {
    id: String,
    bounds: Rect,
    glyph_ids: Vec<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Region {
    id: String,
    kind: &'static str,
    source_order: Vec<usize>,
    source_bounds: Option<Rect>,
    bounds: Option<Rect>,
    transform: [f32; 6],
    parent: Option<usize>,
    parent_status: &'static str,
    even_odd: bool,
    operations: Vec<PathOperation>,
}

pub(super) struct Ledger {
    pub text: ObservedText,
    blocks: Vec<Block>,
    regions: Vec<Region>,
    candidates: Vec<Value>,
    incomplete: BTreeSet<&'static str>,
    clip_ids: BTreeMap<usize, usize>,
    source_spans: Vec<Value>,
    source_images: Vec<Value>,
    work: usize,
    request: Arc<ImportBudget>,
}

pub(super) fn page_rect(b: Rect, media: (f32, f32, f32, f32)) -> Rect {
    Rect::new(b.x - media.0, media.3 - b.y - b.height, b.width, b.height)
}

pub(super) fn contains(a: Rect, b: Rect) -> bool {
    b.x >= a.x - EPS
        && b.y >= a.y - EPS
        && b.x + b.width <= a.x + a.width + EPS
        && b.y + b.height <= a.y + a.height + EPS
}

fn union(a: Rect, b: Rect) -> Rect {
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Rect::new(
        x,
        y,
        (a.x + a.width).max(b.x + b.width) - x,
        (a.y + a.height).max(b.y + b.height) - y,
    )
}

/// Recognise a rectangular path, rather than treating any path's bbox as a clip.
pub(super) fn rectangle(ops: &[PathOperation]) -> Option<Rect> {
    if let [PathOperation::Rectangle(x, y, w, h)] = ops {
        return (w.abs() > 0. && h.abs() > 0.)
            .then(|| Rect::new(x.min(x + w), y.min(y + h), w.abs(), h.abs()));
    }
    let mut vertices = [(0., 0.); 5];
    let mut count = 0;
    let mut closed = false;
    for op in ops {
        match *op {
            PathOperation::MoveTo(x, y) if count == 0 => {
                vertices[count] = (x, y);
                count += 1;
            }
            PathOperation::LineTo(x, y) if count < vertices.len() => {
                vertices[count] = (x, y);
                count += 1;
            }
            PathOperation::ClosePath => closed = true,
            _ => return None,
        }
    }
    if count > 1 && vertices[count - 1] == vertices[0] {
        count -= 1;
        closed = true;
    }
    if !closed || count != 4 {
        return None;
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    );
    let vertices = &vertices[..count];
    for (i, &(x, y)) in vertices.iter().enumerate() {
        let (xx, yy) = vertices[(i + 1) % 4];
        if !x.is_finite() || !y.is_finite() || (x != xx && y != yy) {
            return None;
        }
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    (max_x > min_x && max_y > min_y).then(|| Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
}

/// Fixed-reference snapping: duplicate paints do not shift an axis by averaging.
pub(super) fn axes(mut values: Vec<f32>) -> Vec<f32> {
    values.retain(|v| v.is_finite());
    values.sort_unstable_by(f32::total_cmp);
    values.dedup_by(|current, previous| *current - *previous <= EPS);
    values
}

impl Ledger {
    pub(super) fn observe(
        spans: &[TextSpan],
        span_orders: &[Vec<usize>],
        images: &[PdfImage],
        image_orders: &[Vec<usize>],
        clips: &PathClipObservations,
        media: (f32, f32, f32, f32),
        request: std::sync::Arc<pdf_oxide::import_limits::ImportBudget>,
        include_source_details: bool,
    ) -> Result<Self, super::PdfImportError> {
        let mut result = Self {
            text: ObservedText::observe(spans, media, std::sync::Arc::clone(&request))?,
            blocks: Vec::new(),
            regions: Vec::new(),
            candidates: Vec::new(),
            incomplete: BTreeSet::new(),
            clip_ids: BTreeMap::new(),
            source_spans: Vec::new(),
            source_images: Vec::new(),
            work: 0,
            request: Arc::clone(&request),
        };
        if result.text.glyph_observations_incomplete {
            result
                .incomplete
                .insert("glyph observation budget exceeded");
        }
        let allocation = super::allocation::Allocation(&request);
        #[derive(Serialize)]
        struct SourceSpan<'a> {
            id: String,
            #[serde(rename = "sourceOrder")]
            source_order: Option<&'a Vec<usize>>,
            observation: &'a TextSpan,
        }
        if include_source_details {
            result.source_spans = allocation.vec(spans.len())?;
            for (source_span, span) in spans.iter().enumerate() {
                result.source_spans.push(allocation.value(&SourceSpan {
                    id: allocation.id("span", &[source_span])?,
                    source_order: span_orders.get(source_span),
                    observation: span,
                })?);
            }
        }
        // T0 and region-local recognition consume the same observed row bands.
        // T0 stays independent of borders; owner-local recognition filters first.
        let mut by_span = allocation.vec(spans.len())?;
        by_span.resize_with(spans.len(), Vec::new);
        for (gi, glyph) in result.text.glyphs.iter().enumerate() {
            allocation.push(&mut by_span[glyph.source_span], gi)?;
        }
        let lines = result
            .text
            .lines(&result.text.all_span_indices()?, 0.2)
            .map_err(|_| {
                super::PdfImportError::budget("line observation allocation or ownership failed")
            })?;
        for line in lines {
            let mut line = allocation.collect(
                line.indices
                    .iter()
                    .flat_map(|i| by_span[*i].iter().copied()),
            )?;
            line.sort_unstable_by(|a, b| {
                result.text.glyphs[*a]
                    .bounds
                    .x
                    .total_cmp(&result.text.glyphs[*b].bounds.x)
                    .then(a.cmp(b))
            });
            let mut current: Option<Block> = None;
            for i in line {
                let glyph = &result.text.glyphs[i];
                let join = current.as_ref().is_some_and(|block| {
                    let previous = &result.text.glyphs[*block.glyph_ids.last().unwrap()];
                    glyph.bounds.x - (block.bounds.x + block.bounds.width)
                        <= previous.font_size.min(glyph.font_size) * 0.5
                });
                if !join {
                    if let Some(block) = current.take() {
                        allocation.push(&mut result.blocks, block)?;
                    }
                    current = Some(Block {
                        id: allocation.id("t0", &[result.blocks.len()])?,
                        bounds: glyph.bounds,
                        glyph_ids: Vec::new(),
                    });
                }
                let block = current.as_mut().unwrap();
                block.bounds = union(block.bounds, glyph.bounds);
                allocation.push(&mut block.glyph_ids, i)?;
            }
            if let Some(block) = current {
                allocation.push(&mut result.blocks, block)?;
            }
        }
        result.source_images = allocation.vec(images.len())?;
        for (i, image) in images.iter().enumerate() {
            result.source_images.push(request_json!(allocation, {
                "id": allocation.id("image", &[i])?, "sourceOrder": image_orders.get(i),
                "sourceBounds": image.bbox(), "visibleBounds": image.bbox().map(|b|page_rect(*b,media)),
                "pixelWidth":image.width(), "pixelHeight":image.height(),
                "visibility":"visible_bounds_observed_occlusion_unknown"
            })?);
        }
        let mut parents = BTreeMap::new();
        let mut clip_operations = 0usize;
        for observed in &clips.clips {
            clip_operations = clip_operations.saturating_add(observed.clip.operations.len());
            if clip_operations > 20_000 {
                result
                    .incomplete
                    .insert("clip operation observation budget exceeded");
                break;
            }
            if result.regions.len() == MAX_REGIONS {
                result.incomplete.insert("clip region budget exceeded");
                break;
            }
            let source_bounds = rectangle(&observed.clip.operations);
            let parent = observed
                .clip
                .parent
                .as_ref()
                .and_then(|p| parents.get(&(std::sync::Arc::as_ptr(p) as usize)).copied());
            let id = result.regions.len();
            allocation.tree_entry::<usize, usize>()?;
            parents.insert(std::sync::Arc::as_ptr(&observed.clip) as usize, id);
            allocation.reserve(&mut result.regions, 1)?;
            result.regions.push(Region {
                id: allocation.id("clip", &[id])?,
                kind: "clip_not_paint",
                source_order: allocation.copy_slice(&observed.source_order)?,
                source_bounds,
                bounds: source_bounds.map(|b| page_rect(b, media)),
                transform: observed.transform,
                parent,
                parent_status: if observed.clip.parent.is_none() {
                    "none"
                } else if parent.is_some() {
                    "recorded"
                } else {
                    "unobserved_parent"
                },
                even_odd: observed.clip.even_odd,
                operations: allocation.collect(observed.clip.operations.iter().cloned())?,
            });
        }
        if clips.omitted > 0 {
            result
                .incomplete
                .insert("backend clip observations omitted");
        }
        result.clip_ids = parents;
        if include_source_details {
            let mut uses = BTreeMap::new();
            for (order, clip) in &clips.text_uses {
                if !uses.contains_key(order) {
                    allocation.tree_entry::<usize, usize>()?;
                }
                uses.insert(order, clip);
            }
            for (i, source) in result.source_spans.iter_mut().enumerate() {
                if let Some((order, clip)) = span_orders
                    .get(i)
                    .and_then(|order| uses.get(order).map(|clip| (order, *clip)))
                {
                    let witness = request_json!(allocation, {"sourceOrder":order,
                        "kind":"clip_active_at_text_paint",
                        "clipRegion":clip.as_ref().and_then(|c| result.clip_ids.get(&(Arc::as_ptr(c) as usize))),
                        "hasClip":clip.is_some()})?;
                    allocation.set(source, "activeClipWitness", witness)?;
                }
            }
        }
        if clips.text_uses_omitted > 0 {
            result.incomplete.insert("backend text clip uses omitted");
        }
        Ok(result)
    }

    fn step(&mut self, amount: usize) -> bool {
        self.work = self.work.saturating_add(amount);
        if self.work > MAX_WORK {
            self.incomplete.insert("hypothesis work budget exceeded");
            false
        } else {
            true
        }
    }

    fn candidate(
        &mut self,
        generator: &str,
        region: Rect,
        xs: Vec<f32>,
        ys: Vec<f32>,
        sources: Vec<String>,
    ) -> Result<(), PdfImportError> {
        if self.candidates.len() == MAX_CANDIDATES {
            self.incomplete.insert("candidate budget exceeded");
            return Ok(());
        }
        if xs.len() < 2 || ys.len() < 2 {
            return Ok(());
        }
        let tiles = (xs.len() - 1).saturating_mul(ys.len() - 1);
        if tiles > MAX_TILES {
            self.incomplete.insert("candidate tile budget exceeded");
            return Ok(());
        }
        let request = Arc::clone(&self.request);
        let allocation = Allocation(&request);
        let mut glyphs = Vec::new();
        let mut conflicts = Vec::new();
        let mut owners = BTreeMap::<usize, Vec<usize>>::new();
        let mut ambiguous_owners = Vec::new();
        // Only the existence of exactly one containing row/column is consumed.
        // Keep those two iterator results on the stack instead of collecting
        // every overlap for every glyph.
        fn unique(mut indices: impl Iterator<Item = usize>) -> Option<usize> {
            let first = indices.next()?;
            indices.next().is_none().then_some(first)
        }
        for gi in 0..self.text.glyphs.len() {
            if !self.step(1) {
                break;
            }
            let g = &self.text.glyphs[gi];
            if contains(region, g.bounds) {
                allocation.push(&mut glyphs, gi)?;
                let column = unique(
                    xs.windows(2)
                        .enumerate()
                        .filter(|(_, x)| {
                            g.bounds.x >= x[0] - EPS && g.bounds.x + g.bounds.width <= x[1] + EPS
                        })
                        .map(|(i, _)| i),
                );
                let row = unique(
                    ys.windows(2)
                        .enumerate()
                        .filter(|(_, y)| {
                            g.bounds.y >= y[0] - EPS && g.bounds.y + g.bounds.height <= y[1] + EPS
                        })
                        .map(|(i, _)| i),
                );
                if let (Some(column), Some(row)) = (column, row) {
                    let tile = row * (xs.len() - 1) + column;
                    if !owners.contains_key(&tile) {
                        allocation.tree_entry::<usize, Vec<usize>>()?;
                    }
                    allocation.push(owners.entry(tile).or_default(), gi)?;
                } else {
                    allocation.push(&mut ambiguous_owners, gi)?;
                }
                if !g.text.is_whitespace()
                    && (xs
                        .iter()
                        .skip(1)
                        .take(xs.len() - 2)
                        .any(|x| *x > g.bounds.x + EPS && *x < g.bounds.x + g.bounds.width - EPS)
                        || ys.iter().skip(1).take(ys.len() - 2).any(|y| {
                            *y > g.bounds.y + EPS && *y < g.bounds.y + g.bounds.height - EPS
                        }))
                {
                    allocation.push(&mut conflicts, gi)?;
                }
            }
        }
        let id = allocation.id("grid", &[self.candidates.len()])?;
        let image_indices =
            allocation.collect(self.source_images.iter().enumerate().filter_map(
                |(i, image)| {
                    let b = &image["visibleBounds"];
                    let n = |k: &str| b[k].as_f64().map(|n| n as f32);
                    let b = Rect::new(n("x")?, n("y")?, n("width")?, n("height")?);
                    contains(region, b).then_some(i)
                },
            ))?;
        let axis_evidence =
            |coordinates: &[f32], horizontal: bool| -> Result<Value, PdfImportError> {
                let mut evidence = allocation.vec(coordinates.len())?;
                for at in coordinates {
                    let refs = allocation.collect(sources.iter().filter(|source| {
                        let Some(index) = source
                            .strip_prefix("clip:")
                            .and_then(|n| n.parse::<usize>().ok())
                        else {
                            return true;
                        };
                        self.regions
                            .get(index)
                            .and_then(|r| r.bounds)
                            .is_some_and(|r| {
                                if horizontal {
                                    (r.y - at).abs() <= EPS || (r.y + r.height - at).abs() <= EPS
                                } else {
                                    (r.x - at).abs() <= EPS || (r.x + r.width - at).abs() <= EPS
                                }
                            })
                    }))?;
                    evidence.push(request_json!(allocation, {"at":at,"sourceRefs":refs,
                    "kind":"derived","rule":"fixed-reference-axis-snap-v1"})?);
                }
                Ok(Value::Array(evidence))
            };
        let x_evidence = axis_evidence(&xs, false)?;
        let y_evidence = axis_evidence(&ys, true)?;
        let mut axis = Value::Object(allocation.object()?);
        allocation.set(&mut axis, "x", x_evidence)?;
        allocation.set(&mut axis, "y", y_evidence)?;
        let mut ownership = allocation.vec(owners.len())?;
        for (tile, glyphs) in owners {
            ownership.push(request_json!(allocation,{"tile":tile,"glyphIndices":glyphs})?);
        }
        let mut candidate = request_json!(allocation, {"id":id,"generator":generator,"bounds":region,
            "xBoundaries":xs,"yBoundaries":ys,"axisSources":sources,"imageIndices":image_indices,
            "tileCount":tiles,"glyphIndices":glyphs,"axisCrossingGlyphs":conflicts,
            "ambiguousOwnershipGlyphs":ambiguous_owners,
            "status":"ambiguous","reason":"candidate axes alone do not establish cell separators",
            "alternatives":["separate_cells","connected_rectangular_cells"],"authoredStructure":"unknown"})?;
        allocation.set(&mut candidate, "axisEvidence", axis)?;
        allocation.set(
            &mut candidate,
            "tentativeTileOwnership",
            Value::Array(ownership),
        )?;
        allocation.push(&mut self.candidates, candidate)?;
        Ok(())
    }

    pub(super) fn reconcile(&mut self, detection: &Value) -> Result<(), PdfImportError> {
        let request = Arc::clone(&self.request);
        let allocation = Allocation(&request);
        // Keep clipping and paint generators separate; never label clips as painted rules.
        let mut rects = allocation.vec::<(usize, Rect)>(self.regions.len())?;
        for (i, region) in self.regions.iter().enumerate() {
            if let Some(b) = region.bounds {
                if !rects.iter().any(|(_, r)| *r == b) {
                    rects.push((i, b));
                }
            }
        }
        // Clip adjacency supplies local scopes. Containment alone does not connect cells.
        let mut visited = allocation.vec::<bool>(rects.len())?;
        visited.resize(rects.len(), false);
        for start in 0..rects.len() {
            if visited[start] {
                continue;
            }
            visited[start] = true;
            let mut group = allocation.collect([start])?;
            let mut cursor = 0;
            while cursor < group.len() {
                let a = rects[group[cursor]].1;
                cursor += 1;
                for (j, (_, b)) in rects.iter().enumerate() {
                    if visited[j] {
                        continue;
                    }
                    if !self.step(1) {
                        break;
                    }
                    let adjacent = (((a.x + a.width - b.x).abs() <= EPS
                        || (b.x + b.width - a.x).abs() <= EPS)
                        && (a.y + a.height).min(b.y + b.height) > a.y.max(b.y) + EPS)
                        || (((a.y + a.height - b.y).abs() <= EPS
                            || (b.y + b.height - a.y).abs() <= EPS)
                            && (a.x + a.width).min(b.x + b.width) > a.x.max(b.x) + EPS);
                    if adjacent {
                        visited[j] = true;
                        allocation.push(&mut group, j)?;
                    }
                }
            }
            if group.len() < 2 {
                continue;
            }
            let bounds = group.iter().map(|i| rects[*i].1).reduce(union).unwrap();
            let xs = axes(allocation.collect(group.iter().flat_map(|i| {
                let b = rects[*i].1;
                [b.x, b.x + b.width]
            }))?);
            let ys = axes(allocation.collect(group.iter().flat_map(|i| {
                let b = rects[*i].1;
                [b.y, b.y + b.height]
            }))?);
            let mut sources = allocation.vec(group.len())?;
            for i in &group {
                sources.push(allocation.string(&self.regions[rects[*i].0].id)?);
            }
            self.candidate("clip_adjacency", bounds, xs, ys, sources)?;
        }
        if let Some(tables) = detection["tables"].as_array() {
            for (index, table) in tables.iter().enumerate() {
                let Some(b) = table["bbox"].as_object() else {
                    continue;
                };
                let n = |k: &str| b[k].as_f64().unwrap_or_default() as f32;
                let xs = allocation.collect(
                    table["xBoundaries"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap() as f32),
                )?;
                let ys = allocation.collect(
                    table["yBoundaries"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_f64().unwrap() as f32),
                )?;
                self.candidate(
                    if table["axisEvidence"].is_object() {
                        "active_content_clip_tiling"
                    } else {
                        "paint_boundary"
                    },
                    Rect::new(n("x"), n("y"), n("width"), n("height")),
                    xs,
                    ys,
                    allocation
                        .collect([allocation.string("tableDetection.sourceBoundarySegments")?])?,
                )?;
                if let Some(candidate) = self.candidates.last_mut() {
                    allocation.set(candidate, "sourceTableCandidate", allocation.value(&index)?)?;
                    allocation.set(
                        candidate,
                        "geometryDecision",
                        allocation.value(&table["hierarchy"])?,
                    )?;
                    if table["axisEvidence"].is_object() {
                        allocation.set(
                            candidate,
                            "axisEvidence",
                            allocation.value(&table["axisEvidence"])?,
                        )?;
                    }
                    allocation.set(
                        candidate,
                        "selectedAxisEvidence",
                        allocation.value(&table["axisEvidence"])?,
                    )?;
                }
            }
        }
        // Text-only scopes are proposals, not votes independent of the T0 input.
        let mut i = 0;
        while i < self.blocks.len() && self.candidates.len() < MAX_CANDIDATES {
            let first = &self.blocks[i];
            let mut end = i + 1;
            while end < self.blocks.len()
                && (self.blocks[end].bounds.y - first.bounds.y).abs() <= EPS
            {
                end += 1;
            }
            if end - i > 1 {
                let mut group = allocation.collect(i..end)?;
                allocation.bytes(group.len() * std::mem::size_of::<usize>())?;
                group.sort_by(|a, b| {
                    self.blocks[*a]
                        .bounds
                        .x
                        .total_cmp(&self.blocks[*b].bounds.x)
                });
                let bounds = group
                    .iter()
                    .map(|i| self.blocks[*i].bounds)
                    .reduce(union)
                    .unwrap();
                let mut xs = allocation.vec(group.len() + 1)?;
                xs.push(bounds.x);
                for pair in group.windows(2) {
                    let (a, b) = (self.blocks[pair[0]].bounds, self.blocks[pair[1]].bounds);
                    if a.x + a.width < b.x {
                        xs.push((a.x + a.width + b.x) / 2.);
                    }
                }
                xs.push(bounds.x + bounds.width);
                let mut sources = allocation.vec(group.len())?;
                for i in &group {
                    sources.push(allocation.string(&self.blocks[*i].id)?);
                }
                self.candidate(
                    "text_gap_midpoints",
                    bounds,
                    axes(xs),
                    allocation.collect([bounds.y, bounds.y + bounds.height])?,
                    sources,
                )?;
            }
            i = end;
        }
        if !rects.is_empty() {
            let bounds = rects.iter().map(|(_, b)| *b).reduce(union).unwrap();
            let xs =
                axes(allocation.collect(rects.iter().flat_map(|(_, b)| [b.x, b.x + b.width]))?);
            let ys =
                axes(allocation.collect(rects.iter().flat_map(|(_, b)| [b.y, b.y + b.height]))?);
            let mut sources = allocation.vec(rects.len())?;
            for (i, _) in &rects {
                sources.push(allocation.string(&self.regions[*i].id)?);
            }
            self.candidate("global_projection_comparison", bounds, xs, ys, sources)?;
        }
        self.shared_edges(detection)?;
        Ok(())
    }

    fn shared_edges(&mut self, detection: &Value) -> Result<(), PdfImportError> {
        let request = Arc::clone(&self.request);
        let allocation = Allocation(&request);
        let empty = Vec::new();
        let raw = detection["boundaryObservations"]
            .as_array()
            .unwrap_or(&empty);
        let logical = detection["logicalBoundaries"].as_array().unwrap_or(&empty);
        fn index<'a>(
            allocation: Allocation<'_>,
            values: &'a [Value],
        ) -> Result<[Vec<&'a Value>; 2], PdfImportError> {
            let mut grouped: [Vec<&Value>; 2] = std::array::from_fn(|_| Vec::new());
            for value in values {
                allocation.push(
                    &mut grouped[usize::from(value["horizontal"].as_bool().unwrap())],
                    value,
                )?;
            }
            for group in &mut grouped {
                allocation.bytes(group.len() * std::mem::size_of::<&Value>())?;
                group.sort_by(|a, b| {
                    a["at"]
                        .as_f64()
                        .unwrap()
                        .total_cmp(&b["at"].as_f64().unwrap())
                });
            }
            Ok(grouped)
        }
        let raw_index = index(allocation, raw)?;
        let logical_index = index(allocation, logical)?;
        let near = |values: &Vec<&Value>, at: f32| {
            let start = values.partition_point(|v| v["at"].as_f64().unwrap() < f64::from(at - EPS));
            let end = values.partition_point(|v| v["at"].as_f64().unwrap() <= f64::from(at + EPS));
            (start, end)
        };
        for ci in 0..self.candidates.len() {
            let xs = allocation.collect(
                self.candidates[ci]["xBoundaries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap() as f32),
            )?;
            let ys = allocation.collect(
                self.candidates[ci]["yBoundaries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_f64().unwrap() as f32),
            )?;
            let mut shared = Vec::new();
            for horizontal in [false, true] {
                let (at, across) = if horizontal { (&ys, &xs) } else { (&xs, &ys) };
                for &axis in at.iter().skip(1).take(at.len() - 2) {
                    for unit in across.windows(2) {
                        let raw_values = &raw_index[usize::from(horizontal)];
                        let logical_values = &logical_index[usize::from(horizontal)];
                        let (ra, rb) = near(raw_values, axis);
                        let (la, lb) = near(logical_values, axis);
                        if !self.step(rb - ra + lb - la + 1) {
                            break;
                        }
                        let mut support = Vec::new();
                        let mut cover =
                            |values: &[&Value],
                             record: bool|
                             -> Result<(f32, bool), PdfImportError> {
                                let mut intervals = Vec::new();
                                let mut unknown = false;
                                for edge in values {
                                    if edge["horizontal"].as_bool() != Some(horizontal)
                                        || (edge["at"].as_f64().unwrap() as f32 - axis).abs() > EPS
                                    {
                                        continue;
                                    }
                                    let a = (edge["from"].as_f64().unwrap() as f32).max(unit[0]);
                                    let b = (edge["to"].as_f64().unwrap() as f32).min(unit[1]);
                                    if a >= b {
                                        continue;
                                    }
                                    if edge["paintKind"] == "dash_carrier" {
                                        unknown = true;
                                    }
                                    if record {
                                        allocation
                                            .push(&mut support, allocation.value(&edge["id"])?)?;
                                    }
                                    allocation.push(&mut intervals, (a, b))?;
                                }
                                allocation
                                    .bytes(intervals.len() * std::mem::size_of::<(f32, f32)>())?;
                                intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
                                let mut end = unit[0];
                                let mut total = 0.;
                                for (a, b) in intervals {
                                    total += (b - a.max(end)).max(0.);
                                    end = end.max(b);
                                }
                                Ok((total, unknown))
                            };
                        let (observed, unknown) = cover(&raw_values[ra..rb], true)?;
                        let (regularized, _) = cover(&logical_values[la..lb], false)?;
                        let presence = if unknown {
                            "unknown_dash_coverage"
                        } else if observed == 0. {
                            "absent"
                        } else if observed >= unit[1] - unit[0] - 0.0001 {
                            "continuous"
                        } else {
                            "partial"
                        };
                        allocation.push(&mut shared,request_json!(allocation,{"horizontal":horizontal,"at":axis,"from":unit[0],"to":unit[1],
                            "paintPresence":presence,"rawCoveragePt":if unknown {None}else{Some(observed)},
                            "regularizedCoveragePt":regularized,"sourceBoundaryIds":support,
                            "separatorRole":if presence=="continuous" {"separator_candidate"}else{"undecided"},
                            "connection":"not_inferred_from_paint_absence"})?)?;
                    }
                }
            }
            allocation.set(
                &mut self.candidates[ci],
                "sharedEdges",
                Value::Array(shared),
            )?;
        }
        Ok(())
    }

    fn compositions(&self) -> Result<Vec<Value>, PdfImportError> {
        let allocation = Allocation(&self.request);
        let mut result = Vec::new();
        let bounds = |v: &Value| {
            Rect::new(
                v["x"].as_f64().unwrap() as f32,
                v["y"].as_f64().unwrap() as f32,
                v["width"].as_f64().unwrap() as f32,
                v["height"].as_f64().unwrap() as f32,
            )
        };
        for (i, a) in self.candidates.iter().enumerate() {
            for (j, b) in self.candidates.iter().enumerate() {
                if i == j {
                    continue;
                }
                let (aa, bb) = (bounds(&a["bounds"]), bounds(&b["bounds"]));
                if !contains(aa, bb) || aa.width * aa.height <= bb.width * bb.height {
                    continue;
                }
                let mut shared = allocation.vec(4)?;
                for (name, distance) in [
                    ("left", aa.x - bb.x),
                    ("top", aa.y - bb.y),
                    ("right", aa.x + aa.width - bb.x - bb.width),
                    ("bottom", aa.y + aa.height - bb.y - bb.height),
                ] {
                    if distance.abs() <= EPS {
                        shared.push(name);
                    }
                }
                allocation.push(&mut result,request_json!(allocation,{"outer":a["id"],"inner":b["id"],"status":"ambiguous",
                    "sharedBoundarySides":shared,"alternatives":["H_flat","H_nested"],
                    "reason":"geometric containment or shared boundary does not prove authoring objects"})?)?;
            }
        }
        Ok(result)
    }

    pub(super) fn report(
        self,
        detection: &Value,
        lowering: &Value,
        paths: &[PathContent],
        recognition: &Value,
        full: bool,
    ) -> Result<Value, PdfImportError> {
        let allocation = Allocation(&self.request);
        let mut report = request_json!(allocation,{"schemaVersion":1,"ruleVersion":"tsg-observations-v1",
            "coordinateSystem":"page-local top-left points; source geometry in page user space",
            "regionGeometry":"declared clip paths; effective visibility also depends on parent intersections",
            "idScope":"source page and deterministic extraction order",
            "executionState":"EXPORTED",
            "status":if self.incomplete.is_empty() && detection["deferredReason"].is_null(){"observed"}else{"incomplete"},
            "incompleteReasons":self.incomplete,"glyphCount":self.text.glyphs.len(),
            "initialBlockCount":self.blocks.len(),"clipCount":self.regions.len(),
            "imageCount":self.source_images.len(),
            "candidateCount":self.candidates.len(),"paintPathCount":paths.len(),
            "detectionDeferredReason":detection["deferredReason"],
            "rejectedComponents":detection["rejectedComponents"],
            "loweringDeferred":lowering["deferredTables"],
            "semanticTables":lowering["semanticTables"],
            "boundaryPaintPolicy":lowering["boundaryPaintPolicy"],
            "nativeBorderCells":lowering["nativeBorderCells"],
            "nativeBorderPaintFragments":lowering["nativeBorderPaintFragments"],
            "nativeCellBackgrounds":lowering["nativeCellBackgrounds"],
            "externalBoundaryPaint":lowering["externalBoundaryPaint"],
            "paragraphMappingPolicy":lowering["paragraphMappingPolicy"],
            "bodyParagraphMapping":lowering["bodyParagraphMapping"],
            "policy":{"axisTolerancePt":EPS,"maxGlyphs":MAX_GLYPHS,"maxClipRegions":MAX_REGIONS,
                "maxCandidates":MAX_CANDIDATES,"maxTiles":MAX_TILES,"maxWork":MAX_WORK,
                "lineBaselineEm":0.2,"inlineGapEm":0.5},
            "interpretation":"observations and competing hypotheses, not authored Excel/HWP objects"})?;
        let compositions = if full {
            Some(self.compositions()?)
        } else {
            None
        };
        if full {
            allocation.set(
                &mut report,
                "sourceTypography",
                allocation.value(&lowering["sourceTypography"])?,
            )?;
            allocation.set(
                &mut report,
                "textRecognition",
                allocation.value(recognition)?,
            )?;
            allocation.set(&mut report, "glyphs", allocation.value(&self.text.glyphs)?)?;
            allocation.set(&mut report, "sourceSpans", Value::Array(self.source_spans))?;
            allocation.set(
                &mut report,
                "sourceImages",
                Value::Array(self.source_images),
            )?;
            allocation.set(
                &mut report,
                "initialBlocks",
                allocation.value(&self.blocks)?,
            )?;
            allocation.set(&mut report, "regions", allocation.value(&self.regions)?)?;
            allocation.set(&mut report, "candidates", Value::Array(self.candidates))?;
            allocation.set(
                &mut report,
                "compositions",
                Value::Array(compositions.unwrap()),
            )?;
            let mut paint = allocation.vec(paths.len())?;
            for (i, path) in paths.iter().enumerate() {
                let mut p = allocation.value(path)?;
                allocation.set(
                    &mut p,
                    "id",
                    allocation.value(&allocation.id("paint", &[i])?)?,
                )?;
                allocation.set(
                    &mut p,
                    "activeClipRegion",
                    allocation.value(
                        &path
                            .paint
                            .clip_path
                            .as_ref()
                            .and_then(|clip| self.clip_ids.get(&(Arc::as_ptr(clip) as usize))),
                    )?,
                )?;
                paint.push(p);
            }
            allocation.set(&mut report, "paintPaths", Value::Array(paint))?;
            allocation.set(&mut report, "tableDetection", allocation.value(detection)?)?;
        }
        Ok(report)
    }
}
