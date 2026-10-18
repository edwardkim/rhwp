//! Page-scoped observation and hypothesis ledger. Nothing here asserts authored cells.
//! Original paint is left to lowering; absent paint never proves a cell passage.
use super::text_analysis::{ObservedText, MAX_GLYPHS};
use pdf_oxide::{
    elements::{PathClipObservations, PathContent, PathOperation},
    extractors::PdfImage,
    geometry::Rect,
    layout::TextSpan,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

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
    let mut vertices = Vec::new();
    let mut closed = false;
    for op in ops {
        match *op {
            PathOperation::MoveTo(x, y) if vertices.is_empty() => vertices.push((x, y)),
            PathOperation::LineTo(x, y) => vertices.push((x, y)),
            PathOperation::ClosePath => closed = true,
            _ => return None,
        }
    }
    if vertices.last() == vertices.first() && vertices.len() > 1 {
        vertices.pop();
        closed = true;
    }
    if !closed || vertices.len() != 4 {
        return None;
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (
        f32::INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NEG_INFINITY,
    );
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
    values.sort_by(f32::total_cmp);
    let mut result = Vec::new();
    for v in values {
        if result.last().is_none_or(|previous| v - previous > EPS) {
            result.push(v);
        }
    }
    result
}

impl Ledger {
    pub(super) fn observe(
        spans: &[TextSpan],
        span_orders: &[Vec<usize>],
        images: &[PdfImage],
        image_orders: &[Vec<usize>],
        clips: &PathClipObservations,
        media: (f32, f32, f32, f32),
    ) -> Self {
        let mut result = Self {
            text: ObservedText::observe(spans, media),
            blocks: Vec::new(),
            regions: Vec::new(),
            candidates: Vec::new(),
            incomplete: BTreeSet::new(),
            clip_ids: BTreeMap::new(),
            source_spans: Vec::new(),
            source_images: Vec::new(),
            work: 0,
        };
        if result.text.glyph_observations_incomplete {
            result
                .incomplete
                .insert("glyph observation budget exceeded");
        }
        result.source_spans = spans
            .iter()
            .enumerate()
            .map(|(source_span, span)| {
                json!({"id":format!("span:{source_span}"),
                "sourceOrder":span_orders.get(source_span),"observation":span})
            })
            .collect();
        // T0 and region-local recognition consume the same observed row bands.
        // T0 stays independent of borders; owner-local recognition filters first.
        let mut by_span = vec![Vec::new(); spans.len()];
        for (gi, glyph) in result.text.glyphs.iter().enumerate() {
            by_span[glyph.source_span].push(gi);
        }
        let lines = result
            .text
            .lines(&result.text.all_span_indices(), 0.2)
            .expect("observation's own span indices are valid");
        for line in lines {
            let mut line: Vec<_> = line
                .indices
                .iter()
                .flat_map(|i| by_span[*i].iter().copied())
                .collect();
            line.sort_by(|a, b| {
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
                        result.blocks.push(block);
                    }
                    current = Some(Block {
                        id: format!("t0:{}", result.blocks.len()),
                        bounds: glyph.bounds,
                        glyph_ids: Vec::new(),
                    });
                }
                let block = current.as_mut().unwrap();
                block.bounds = union(block.bounds, glyph.bounds);
                block.glyph_ids.push(i);
            }
            if let Some(block) = current {
                result.blocks.push(block);
            }
        }
        result.source_images=images.iter().enumerate().map(|(i,image)|json!({
            "id":format!("image:{i}"),"sourceOrder":image_orders.get(i),
            "sourceBounds":image.bbox(),"visibleBounds":image.bbox().map(|b|page_rect(*b,media)),
            "pixelWidth":image.width(),"pixelHeight":image.height(),
            "visibility":"visible_bounds_observed_occlusion_unknown"
        })).collect();
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
            parents.insert(std::sync::Arc::as_ptr(&observed.clip) as usize, id);
            result.regions.push(Region {
                id: format!("clip:{id}"),
                kind: "clip_not_paint",
                source_order: observed.source_order.clone(),
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
                operations: observed.clip.operations.clone(),
            });
        }
        if clips.omitted > 0 {
            result
                .incomplete
                .insert("backend clip observations omitted");
        }
        result.clip_ids = parents;
        result
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
    ) {
        if self.candidates.len() == MAX_CANDIDATES {
            self.incomplete.insert("candidate budget exceeded");
            return;
        }
        if xs.len() < 2 || ys.len() < 2 {
            return;
        }
        let tiles = (xs.len() - 1).saturating_mul(ys.len() - 1);
        if tiles > MAX_TILES {
            self.incomplete.insert("candidate tile budget exceeded");
            return;
        }
        let mut glyphs = Vec::new();
        let mut conflicts = Vec::new();
        let mut owners = BTreeMap::<usize, Vec<usize>>::new();
        let mut ambiguous_owners = Vec::new();
        for gi in 0..self.text.glyphs.len() {
            if !self.step(1) {
                break;
            }
            let g = &self.text.glyphs[gi];
            if contains(region, g.bounds) {
                glyphs.push(gi);
                let columns: Vec<_> = xs
                    .windows(2)
                    .enumerate()
                    .filter(|(_, x)| {
                        g.bounds.x >= x[0] - EPS && g.bounds.x + g.bounds.width <= x[1] + EPS
                    })
                    .map(|(i, _)| i)
                    .collect();
                let rows: Vec<_> = ys
                    .windows(2)
                    .enumerate()
                    .filter(|(_, y)| {
                        g.bounds.y >= y[0] - EPS && g.bounds.y + g.bounds.height <= y[1] + EPS
                    })
                    .map(|(i, _)| i)
                    .collect();
                if columns.len() == 1 && rows.len() == 1 {
                    owners
                        .entry(rows[0] * (xs.len() - 1) + columns[0])
                        .or_default()
                        .push(gi);
                } else {
                    ambiguous_owners.push(gi);
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
                    conflicts.push(gi);
                }
            }
        }
        let id = format!("grid:{}", self.candidates.len());
        let image_indices: Vec<_> = self
            .source_images
            .iter()
            .enumerate()
            .filter_map(|(i, image)| {
                let b = &image["visibleBounds"];
                let n = |k: &str| b[k].as_f64().map(|n| n as f32);
                let b = Rect::new(n("x")?, n("y")?, n("width")?, n("height")?);
                contains(region, b).then_some(i)
            })
            .collect();
        let axis_evidence = |coordinates: &[f32], horizontal: bool| {
            coordinates.iter().map(|at|{
            let refs:Vec<_>=sources.iter().filter(|source|{
                let Some(index)=source.strip_prefix("clip:").and_then(|n|n.parse::<usize>().ok()) else{return true;};
                self.regions.get(index).and_then(|r|r.bounds).is_some_and(|r|
                    if horizontal {(r.y-at).abs()<=EPS || (r.y+r.height-at).abs()<=EPS}
                    else {(r.x-at).abs()<=EPS || (r.x+r.width-at).abs()<=EPS})
            }).collect();
            json!({"at":at,"sourceRefs":refs,"kind":"derived","rule":"fixed-reference-axis-snap-v1"})
        }).collect::<Vec<_>>()
        };
        let x_evidence = axis_evidence(&xs, false);
        let y_evidence = axis_evidence(&ys, true);
        self.candidates
            .push(json!({"id":id,"generator":generator,"bounds":region,
            "xBoundaries":xs,"yBoundaries":ys,"axisSources":sources,
            "axisEvidence":{"x":x_evidence,"y":y_evidence},"imageIndices":image_indices,
            "tileCount":tiles,"glyphIndices":glyphs,"axisCrossingGlyphs":conflicts,
            "tentativeTileOwnership":owners.into_iter().map(|(tile,glyphs)|json!({"tile":tile,"glyphIndices":glyphs})).collect::<Vec<_>>(),
            "ambiguousOwnershipGlyphs":ambiguous_owners,
            "status":"ambiguous","reason":"candidate axes alone do not establish cell separators",
            "alternatives":["separate_cells","connected_rectangular_cells"],
            "authoredStructure":"unknown"}));
    }

    pub(super) fn reconcile(&mut self, detection: &Value) {
        // Keep clipping and paint generators separate; never label clips as painted rules.
        let mut rects: Vec<(usize, Rect)> = Vec::new();
        for (i, region) in self.regions.iter().enumerate() {
            if let Some(b) = region.bounds {
                if !rects.iter().any(|(_, r)| *r == b) {
                    rects.push((i, b));
                }
            }
        }
        // Clip adjacency supplies local scopes. Containment alone does not connect cells.
        let mut visited = BTreeSet::new();
        for start in 0..rects.len() {
            if !visited.insert(start) {
                continue;
            }
            let mut group = vec![start];
            let mut cursor = 0;
            while cursor < group.len() {
                let a = rects[group[cursor]].1;
                cursor += 1;
                for (j, (_, b)) in rects.iter().enumerate() {
                    if visited.contains(&j) {
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
                        visited.insert(j);
                        group.push(j);
                    }
                }
            }
            if group.len() < 2 {
                continue;
            }
            let bounds = group.iter().map(|i| rects[*i].1).reduce(union).unwrap();
            let xs = axes(
                group
                    .iter()
                    .flat_map(|i| {
                        let b = rects[*i].1;
                        [b.x, b.x + b.width]
                    })
                    .collect(),
            );
            let ys = axes(
                group
                    .iter()
                    .flat_map(|i| {
                        let b = rects[*i].1;
                        [b.y, b.y + b.height]
                    })
                    .collect(),
            );
            let sources = group
                .iter()
                .map(|i| self.regions[rects[*i].0].id.clone())
                .collect();
            self.candidate("clip_adjacency", bounds, xs, ys, sources);
        }
        if let Some(tables) = detection["tables"].as_array() {
            for (index, table) in tables.iter().enumerate() {
                let Some(b) = table["bbox"].as_object() else {
                    continue;
                };
                let n = |k: &str| b[k].as_f64().unwrap_or_default() as f32;
                let xs = table["xBoundaries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect();
                let ys = table["yBoundaries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect();
                self.candidate(
                    "paint_boundary",
                    Rect::new(n("x"), n("y"), n("width"), n("height")),
                    xs,
                    ys,
                    vec!["tableDetection.sourceBoundarySegments".into()],
                );
                if let Some(candidate) = self.candidates.last_mut() {
                    candidate["sourceTableCandidate"] = json!(index);
                    candidate["geometryDecision"] = table["hierarchy"].clone();
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
                let mut group: Vec<_> = (i..end).collect();
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
                let mut xs = vec![bounds.x];
                for pair in group.windows(2) {
                    let (a, b) = (self.blocks[pair[0]].bounds, self.blocks[pair[1]].bounds);
                    if a.x + a.width < b.x {
                        xs.push((a.x + a.width + b.x) / 2.);
                    }
                }
                xs.push(bounds.x + bounds.width);
                let sources = group.iter().map(|i| self.blocks[*i].id.clone()).collect();
                self.candidate(
                    "text_gap_midpoints",
                    bounds,
                    axes(xs),
                    vec![bounds.y, bounds.y + bounds.height],
                    sources,
                );
            }
            i = end;
        }
        if !rects.is_empty() {
            let bounds = rects.iter().map(|(_, b)| *b).reduce(union).unwrap();
            let xs = axes(
                rects
                    .iter()
                    .flat_map(|(_, b)| [b.x, b.x + b.width])
                    .collect(),
            );
            let ys = axes(
                rects
                    .iter()
                    .flat_map(|(_, b)| [b.y, b.y + b.height])
                    .collect(),
            );
            self.candidate(
                "global_projection_comparison",
                bounds,
                xs,
                ys,
                rects
                    .iter()
                    .map(|(i, _)| self.regions[*i].id.clone())
                    .collect(),
            );
        }
        self.shared_edges(detection);
    }

    fn shared_edges(&mut self, detection: &Value) {
        let empty = Vec::new();
        let raw = detection["boundaryObservations"]
            .as_array()
            .unwrap_or(&empty);
        let logical = detection["logicalBoundaries"].as_array().unwrap_or(&empty);
        fn index(values: &[Value]) -> [Vec<&Value>; 2] {
            let mut grouped: [Vec<&Value>; 2] = std::array::from_fn(|_| Vec::new());
            for value in values {
                grouped[usize::from(value["horizontal"].as_bool().unwrap())].push(value);
            }
            for group in &mut grouped {
                group.sort_by(|a, b| {
                    a["at"]
                        .as_f64()
                        .unwrap()
                        .total_cmp(&b["at"].as_f64().unwrap())
                });
            }
            grouped
        }
        let raw_index = index(raw);
        let logical_index = index(logical);
        let near = |values: &Vec<&Value>, at: f32| {
            let start = values.partition_point(|v| v["at"].as_f64().unwrap() < f64::from(at - EPS));
            let end = values.partition_point(|v| v["at"].as_f64().unwrap() <= f64::from(at + EPS));
            (start, end)
        };
        for ci in 0..self.candidates.len() {
            let xs: Vec<_> = self.candidates[ci]["xBoundaries"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap() as f32)
                .collect();
            let ys: Vec<_> = self.candidates[ci]["yBoundaries"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap() as f32)
                .collect();
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
                        let mut cover = |values: &[&Value], record: bool| {
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
                                    support.push(edge["id"].clone());
                                }
                                intervals.push((a, b));
                            }
                            intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
                            let mut end = unit[0];
                            let mut total = 0.;
                            for (a, b) in intervals {
                                total += (b - a.max(end)).max(0.);
                                end = end.max(b);
                            }
                            (total, unknown)
                        };
                        let (observed, unknown) = cover(&raw_values[ra..rb], true);
                        let (regularized, _) = cover(&logical_values[la..lb], false);
                        let presence = if unknown {
                            "unknown_dash_coverage"
                        } else if observed == 0. {
                            "absent"
                        } else if observed >= unit[1] - unit[0] - 0.0001 {
                            "continuous"
                        } else {
                            "partial"
                        };
                        shared.push(json!({"horizontal":horizontal,"at":axis,"from":unit[0],"to":unit[1],
                            "paintPresence":presence,"rawCoveragePt":if unknown {None}else{Some(observed)},
                            "regularizedCoveragePt":regularized,"sourceBoundaryIds":support,
                            "separatorRole":if presence=="continuous" {"separator_candidate"}else{"undecided"},
                            "connection":"not_inferred_from_paint_absence"}));
                    }
                }
            }
            self.candidates[ci]["sharedEdges"] = json!(shared);
        }
    }

    fn compositions(&self) -> Vec<Value> {
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
                let mut shared = Vec::new();
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
                result.push(json!({"outer":a["id"],"inner":b["id"],"status":"ambiguous",
                    "sharedBoundarySides":shared,"alternatives":["H_flat","H_nested"],
                    "reason":"geometric containment or shared boundary does not prove authoring objects"}));
            }
        }
        result
    }

    pub(super) fn report(
        &self,
        detection: &Value,
        lowering: &Value,
        paths: &[PathContent],
        recognition: &Value,
        full: bool,
    ) -> Value {
        let mut report = json!({"schemaVersion":1,"ruleVersion":"tsg-observations-v1",
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
            "paragraphMappingPolicy":lowering["paragraphMappingPolicy"],
            "bodyParagraphMapping":lowering["bodyParagraphMapping"],
            "policy":{"axisTolerancePt":EPS,"maxGlyphs":MAX_GLYPHS,"maxClipRegions":MAX_REGIONS,
                "maxCandidates":MAX_CANDIDATES,"maxTiles":MAX_TILES,"maxWork":MAX_WORK,
                "lineBaselineEm":0.2,"inlineGapEm":0.5},
            "interpretation":"observations and competing hypotheses, not authored Excel/HWP objects"});
        if full {
            report["sourceTypography"] = lowering["sourceTypography"].clone();
            report["textRecognition"] = recognition.clone();
            report["glyphs"] = json!(self.text.glyphs);
            report["sourceSpans"] = json!(self.source_spans);
            report["sourceImages"] = json!(self.source_images);
            report["initialBlocks"] = json!(self.blocks);
            report["regions"] = json!(self.regions);
            report["candidates"] = json!(self.candidates);
            report["compositions"] = json!(self.compositions());
            report["paintPaths"] = json!(paths
                .iter()
                .enumerate()
                .map(|(i, path)| {
                    let mut p = json!(path);
                    p["id"] = json!(format!("paint:{i}"));
                    p["activeClipRegion"] =
                        json!(path.paint.clip_path.as_ref().and_then(|clip| self
                            .clip_ids
                            .get(&(std::sync::Arc::as_ptr(clip) as usize))));
                    p
                })
                .collect::<Vec<_>>());
            report["tableDetection"] = detection.clone();
        }
        report
    }
}
