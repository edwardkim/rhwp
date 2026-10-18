//! Detection only: vector-rule topology, independent of text wrapping and IR lowering.
use pdf_oxide::{
    elements::{PathContent, PathOperation},
    extractors::PdfImage,
    geometry::Rect,
    layout::TextSpan,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

mod cell_regions;

const EPS: f32 = 0.5;
const MAX_EDGES: usize = 512;
const MAX_SOURCE_EDGES: usize = 20_000;
const MAX_TILES: usize = 4096;

#[derive(Clone, Copy)]
struct Edge {
    horizontal: bool,
    at: f32,
    from: f32,
    to: f32,
    source_path: usize,
    source_segment: usize,
    stroke_width: f32,
    source_operation: usize,
    paint_kind: &'static str,
    periodic_gap: f32,
}

struct Sets(Vec<usize>);
impl Sets {
    fn new(n: usize) -> Self {
        Self((0..n).collect())
    }
    fn root(&mut self, mut i: usize) -> usize {
        while self.0[i] != i {
            self.0[i] = self.0[self.0[i]];
            i = self.0[i];
        }
        i
    }
    fn join(&mut self, a: usize, b: usize) {
        let a = self.root(a);
        let b = self.root(b);
        self.0[b] = a;
    }
    fn groups(&mut self) -> Vec<Vec<usize>> {
        let mut groups = BTreeMap::<usize, Vec<usize>>::new();
        for i in 0..self.0.len() {
            let root = self.root(i);
            groups.entry(root).or_default().push(i);
        }
        groups.into_values().collect()
    }
}

fn edges(paths: &[PathContent], left: f32, top: f32) -> Result<Vec<Edge>, String> {
    use pdf_oxide::extractors::image_clip::ImageClip;
    let mut edges = Vec::new();
    for (source_path, path) in paths.iter().enumerate() {
        let matrix = path.matrix.unwrap_or([1., 0., 0., 1., 0., 0.]);
        let transform = |x: f32, y: f32| {
            (
                matrix[0] * x + matrix[2] * y + matrix[4],
                matrix[1] * x + matrix[3] * y + matrix[5],
            )
        };
        let mut add = |a: (f32, f32),
                       b: (f32, f32),
                       operation: usize,
                       width: f32,
                       kind: &'static str|
         -> Result<(), String> {
            let (a, b) = (transform(a.0, a.1), transform(b.0, b.1));
            if ![a.0, a.1, b.0, b.1, width].iter().all(|v| v.is_finite()) {
                return Err("non-finite rule geometry".into());
            }
            // Axis tolerance must not erase short dashes or turn short diagonals into rules.
            let (dx, dy) = ((a.0 - b.0).abs(), (a.1 - b.1).abs());
            let horizontal = if dy == 0. && dx > 0. {
                true
            } else if dx == 0. && dy > 0. {
                false
            } else {
                return Ok(());
            };
            let (at, mut from, mut to) = if horizontal {
                (top - a.1, a.0.min(b.0) - left, a.0.max(b.0) - left)
            } else {
                (a.0 - left, top - a.1.max(b.1), top - a.1.min(b.1))
            };
            match path.paint.clip {
                ImageClip::Empty | ImageClip::Unsupported | ImageClip::Polygon(_) => return Ok(()),
                ImageClip::Rectangle(clip) => {
                    let clip = super::table_inference::page_rect(clip, (left, 0., 0., top));
                    let (lo, hi, start, end) = if horizontal {
                        (clip.y, clip.y + clip.height, clip.x, clip.x + clip.width)
                    } else {
                        (clip.x, clip.x + clip.width, clip.y, clip.y + clip.height)
                    };
                    if at < lo || at > hi {
                        return Ok(());
                    }
                    from = from.max(start);
                    to = to.min(end);
                }
                ImageClip::Unbounded => {}
            }
            if to <= from {
                return Ok(());
            }
            if edges.len() == MAX_SOURCE_EDGES {
                return Err("ruled-table source edge budget exceeded".into());
            }
            edges.push(Edge {
                horizontal,
                at,
                from,
                to,
                source_path,
                source_segment: edges.len(),
                stroke_width: width,
                source_operation: operation,
                paint_kind: kind,
                periodic_gap: 0.,
            });
            Ok(())
        };
        if path.stroke_color.is_some() && path.paint.stroke_alpha > 0. {
            let stroke_kind = if path.dash_pattern.is_some() {
                "dash_carrier"
            } else {
                "stroke"
            };
            let mut cursor = None;
            let mut start = None;
            for (oi, op) in path.operations.iter().enumerate() {
                match *op {
                    PathOperation::MoveTo(x, y) => {
                        cursor = Some((x, y));
                        start = cursor;
                    }
                    PathOperation::LineTo(x, y) => {
                        if let Some(p) = cursor {
                            add(p, (x, y), oi, path.stroke_width, stroke_kind)?;
                        }
                        cursor = Some((x, y));
                    }
                    PathOperation::Rectangle(x, y, w, h) => {
                        for (a, b) in [
                            ((x, y), (x + w, y)),
                            ((x + w, y), (x + w, y + h)),
                            ((x + w, y + h), (x, y + h)),
                            ((x, y + h), (x, y)),
                        ] {
                            add(a, b, oi, path.stroke_width, stroke_kind)?;
                        }
                        cursor = Some((x, y));
                        start = cursor;
                    }
                    PathOperation::ClosePath => {
                        if let (Some(a), Some(b)) = (cursor, start) {
                            add(a, b, oi, path.stroke_width, stroke_kind)?;
                        }
                        cursor = start;
                    }
                    PathOperation::CurveTo(..) => {
                        cursor = None;
                        start = None;
                    }
                }
            }
        }
        if path.fill_color.is_some() && path.paint.fill_alpha > 0. {
            if let Some(b) = super::table_inference::rectangle(&path.operations) {
                // A thin filled band is boundary evidence, not proof of its semantic role.
                if b.width >= 4. * b.height && b.height <= 2. {
                    add(
                        (b.x, b.y + b.height / 2.),
                        (b.x + b.width, b.y + b.height / 2.),
                        0,
                        b.height,
                        "fill_band",
                    )?;
                } else if b.height >= 4. * b.width && b.width <= 2. {
                    add(
                        (b.x + b.width / 2., b.y),
                        (b.x + b.width / 2., b.y + b.height),
                        0,
                        b.width,
                        "fill_band",
                    )?;
                }
            }
        }
    }
    Ok(edges)
}

/// Compact geometric duplicates/contiguous paint and separately evidenced periodic runs
/// before the quadratic grid stage. Original contributors are retained unchanged.
fn normalize_edges(source: &[Edge]) -> Result<(Vec<Edge>, Vec<Edge>), String> {
    let mut sorted = source.to_vec();
    sorted.sort_by(|a, b| {
        a.horizontal
            .cmp(&b.horizontal)
            .then(a.at.total_cmp(&b.at))
            .then(a.from.total_cmp(&b.from))
            .then(a.to.total_cmp(&b.to))
    });
    let mut lines: Vec<Vec<Edge>> = Vec::new();
    for edge in sorted {
        if let Some(line) = lines.last_mut().filter(|line| {
            line[0].horizontal == edge.horizontal && (line[0].at - edge.at).abs() <= EPS
        }) {
            line.push(edge);
        } else {
            lines.push(vec![edge]);
        }
    }
    let mut canonical = Vec::new();
    for mut line in lines {
        let at = line[0].at;
        line.sort_by(|a, b| a.from.total_cmp(&b.from).then(a.to.total_cmp(&b.to)));
        let mut intervals: Vec<Edge> = Vec::new();
        for edge in line {
            if let Some(previous) = intervals.last_mut().filter(|p| edge.from <= p.to) {
                previous.to = previous.to.max(edge.to);
            } else {
                intervals.push(Edge { at, ..edge });
            }
        }
        let mut i = 0;
        while i < intervals.len() {
            let first = intervals[i];
            let mut end = i + 1;
            if i + 1 < intervals.len() {
                let length = first.to - first.from;
                let gap = intervals[i + 1].from - first.to;
                let same = |a: f32, b: f32| (a - b).abs() <= 0.001_f32.max(b.abs() * 0.05);
                while end < intervals.len()
                    && same(intervals[end].to - intervals[end].from, length)
                    && same(intervals[end].from - intervals[end - 1].to, gap)
                {
                    end += 1;
                }
            }
            if end - i >= 4 {
                canonical.push(Edge {
                    to: intervals[end - 1].to,
                    paint_kind: "periodic_carrier",
                    periodic_gap: intervals[i + 1].from - first.to,
                    ..first
                });
                i = end;
            } else {
                canonical.push(first);
                i += 1;
            }
            if canonical.len() > MAX_EDGES {
                return Err("ruled-table canonical edge budget exceeded".into());
            }
        }
    }
    let projected: Vec<_> = canonical
        .iter()
        .map(|e| project_junctions(*e, &canonical))
        .collect();
    let logical = source
        .iter()
        .map(|raw| {
            canonical
                .iter()
                .zip(&projected)
                .find(|(e, _)| {
                    e.horizontal == raw.horizontal
                        && (e.at - raw.at).abs() <= EPS
                        && raw.from >= e.from - EPS
                        && raw.to <= e.to + EPS
                })
                .map_or(*raw, |(_, e)| Edge {
                    source_path: raw.source_path,
                    source_segment: raw.source_segment,
                    source_operation: raw.source_operation,
                    ..*e
                })
        })
        .collect();
    Ok((logical, projected))
}

fn axes(values: Vec<f32>) -> Vec<f32> {
    super::table_inference::axes(values)
}

fn geometry_key(edge: &Edge) -> (bool, u32, u32, u32) {
    let bits = |v: f32| if v == 0. { 0 } else { v.to_bits() };
    (
        edge.horizontal,
        bits(edge.at),
        bits(edge.from),
        bits(edge.to),
    )
}

fn project_junctions(edge: Edge, source: &[Edge]) -> Edge {
    let endpoint = |value: f32| {
        source
            .iter()
            .filter(|other| {
                // Endpoints and the measured repeat gap are f32 observations.
                // Their arithmetic must not reject an exactly touching junction
                // solely because (edge.to + gap) rounds below the crossing axis.
                // This is a roundoff bound, not an extra geometric snap radius.
                let rounding = 2.0
                    * f32::EPSILON
                    * (edge.from.abs() + edge.to.abs() + edge.periodic_gap.abs() + other.at.abs());
                other.horizontal != edge.horizontal
                    && other.at >= edge.from - edge.periodic_gap - rounding
                    && other.at <= edge.to + edge.periodic_gap + rounding
                    && edge.at >= other.from - EPS
                    && edge.at <= other.to + EPS
                    && (other.at - value).abs() <= other.stroke_width / 2. + EPS + edge.periodic_gap
            })
            .min_by(|a, b| {
                (a.at - value)
                    .abs()
                    .total_cmp(&(b.at - value).abs())
                    .then(a.at.total_cmp(&b.at))
            })
            .map_or(value, |other| other.at)
    };
    let (from, to) = (endpoint(edge.from), endpoint(edge.to));
    if to - from > EPS {
        Edge { from, to, ..edge }
    } else {
        edge
    }
}

fn component_groups(edges: &[Edge]) -> Vec<Vec<usize>> {
    let mut sets = Sets::new(edges.len());
    for a in 0..edges.len() {
        for b in a + 1..edges.len() {
            let (a_edge, b_edge) = (edges[a], edges[b]);
            let touches = if a_edge.horizontal == b_edge.horizontal {
                (a_edge.at - b_edge.at).abs() <= EPS
                    && a_edge.from <= b_edge.to + EPS
                    && b_edge.from <= a_edge.to + EPS
            } else {
                let (h, v) = if a_edge.horizontal {
                    (a_edge, b_edge)
                } else {
                    (b_edge, a_edge)
                };
                v.at >= h.from - EPS
                    && v.at <= h.to + EPS
                    && h.at >= v.from - EPS
                    && h.at <= v.to + EPS
            };
            if touches {
                sets.join(a, b);
            }
        }
    }
    sets.groups()
}

fn boundary_contributor(edge: &Edge, grid: &Grid, component: &[Edge]) -> bool {
    let mut intervals = Vec::new();
    for cell in &grid.cells {
        let b = cell.bbox;
        let (axes, from, to) = if edge.horizontal {
            ([b.y, b.y + b.height], b.x, b.x + b.width)
        } else {
            ([b.x, b.x + b.width], b.y, b.y + b.height)
        };
        if axes.iter().any(|at| (at - edge.at).abs() <= EPS) {
            intervals.push((from, to));
        }
    }
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let Some(&(start, _)) = intervals.first() else {
        return false;
    };
    let end = intervals.iter().map(|(_, to)| *to).fold(start, f32::max);
    // Endpoints can reach into the perpendicular border's painted join, not arbitrary artwork.
    let join_extent = |at: f32| {
        component
            .iter()
            .filter(|other| {
                other.horizontal != edge.horizontal
                    && (other.at - at).abs() <= EPS
                    && edge.at >= other.from - EPS
                    && edge.at <= other.to + EPS
            })
            .map(|other| other.stroke_width / 2.)
            .fold(0., f32::max)
    };
    if edge.from < start - join_extent(start) - EPS || edge.to > end + join_extent(end) + EPS {
        return false;
    }
    let target_start = edge.from.max(start);
    let target_end = edge.to.min(end);
    if target_end <= target_start {
        return false;
    }
    let mut cursor = target_start;
    for (from, to) in intervals {
        let (from, to) = (from.max(target_start), to.min(target_end));
        if to <= cursor {
            continue;
        }
        if from > cursor + EPS {
            return false;
        }
        cursor = cursor.max(to);
    }
    cursor >= target_end - EPS
}

fn coverage(edges: &[Edge], horizontal: bool, at: f32, from: f32, to: f32) -> f32 {
    let mut intervals: Vec<_> = edges
        .iter()
        .filter(|e| e.horizontal == horizontal && (e.at - at).abs() <= EPS)
        .map(|e| (e.from.max(from), e.to.min(to)))
        .filter(|(a, b)| b > a)
        .collect();
    intervals.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut total = 0.0;
    let mut end = from;
    for (a, b) in intervals {
        if a > end {
            total += (b - a).max(0.0);
        } else {
            total += (b - end).max(0.0);
        }
        end = end.max(b);
    }
    total
}

fn complete(edges: &[Edge], horizontal: bool, at: f32, from: f32, to: f32) -> bool {
    coverage(edges, horizontal, at, from, to) >= to - from - EPS
}

fn inside(inner: Rect, outer: Rect) -> bool {
    inner.x >= outer.x - EPS
        && inner.y >= outer.y - EPS
        && inner.x + inner.width <= outer.x + outer.width + EPS
        && inner.y + inner.height <= outer.y + outer.height + EPS
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

struct Cell {
    row: usize,
    col: usize,
    rows: usize,
    cols: usize,
    bbox: Rect,
    spans: Vec<usize>,
    images: Vec<usize>,
}
struct Grid {
    bbox: Rect,
    xs: Vec<f32>,
    ys: Vec<f32>,
    cells: Vec<Cell>,
    topology: Value,
    source_edges: Vec<Edge>,
    search_region: Option<Rect>,
    shared_edges: Vec<Value>,
    uncertain_connections: usize,
    regularized: bool,
    cell_region_evidence: Value,
    axis_evidence: Value,
}

/// Paint observations on a proposed axis set remain distinct from cell ownership.
fn grid_shared_edges(
    edges: &[Edge],
    xs: &[f32],
    ys: &[f32],
    allow_partial: bool,
) -> Result<(Vec<Value>, usize, Sets), String> {
    let (cols, rows) = (xs.len() - 1, ys.len() - 1);
    let mut sets = Sets::new(rows * cols);
    // Connected tiles are only one hypothesis. Missing paint never confirms a passage.
    let mut uncertain_connections = 0;
    let mut shared_edges = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            if col + 1 < cols {
                let length = ys[row + 1] - ys[row];
                let covered = coverage(edges, false, xs[col + 1], ys[row], ys[row + 1]);
                shared_edges.push(json!({"horizontal":false,"at":xs[col+1],"from":ys[row],"to":ys[row+1],
                    "regularizedCoveragePt":covered,"separatorRole":if covered==0. {"undecided"} else {"separator_candidate"},
                    "alternatives":if covered==0. {vec!["separate_cells","connected_cells"]} else {vec!["separate_cells"]}}));
                if covered == 0. {
                    uncertain_connections += 1;
                    sets.join(row * cols + col, row * cols + col + 1);
                } else if !allow_partial && covered < length - EPS {
                    return Err("partial internal vertical boundary".into());
                }
            }
            if row + 1 < rows {
                let length = xs[col + 1] - xs[col];
                let covered = coverage(edges, true, ys[row + 1], xs[col], xs[col + 1]);
                shared_edges.push(json!({"horizontal":true,"at":ys[row+1],"from":xs[col],"to":xs[col+1],
                    "regularizedCoveragePt":covered,"separatorRole":if covered==0. {"undecided"} else {"separator_candidate"},
                    "alternatives":if covered==0. {vec!["separate_cells","connected_cells"]} else {vec!["separate_cells"]}}));
                if covered == 0. {
                    uncertain_connections += 1;
                    sets.join(row * cols + col, (row + 1) * cols + col);
                } else if !allow_partial && covered < length - EPS {
                    return Err("partial internal horizontal boundary".into());
                }
            }
        }
    }
    Ok((shared_edges, uncertain_connections, sets))
}

fn grid(edges: &[Edge], allow_linear: bool) -> Result<Grid, String> {
    let xs = axes(
        edges
            .iter()
            .filter(|e| !e.horizontal)
            .map(|e| e.at)
            .collect(),
    );
    let ys = axes(
        edges
            .iter()
            .filter(|e| e.horizontal)
            .map(|e| e.at)
            .collect(),
    );
    if xs.len() < 2 || ys.len() < 2 || (!allow_linear && (xs.len() < 3 || ys.len() < 3)) {
        return Err("not a two-dimensional multi-cell grid".into());
    }
    let (cols, rows) = (xs.len() - 1, ys.len() - 1);
    if cols * rows > MAX_TILES {
        return Err("ruled-table tile budget exceeded".into());
    }
    if !complete(edges, true, ys[0], xs[0], xs[cols])
        || !complete(edges, true, ys[rows], xs[0], xs[cols])
        || !complete(edges, false, xs[0], ys[0], ys[rows])
        || !complete(edges, false, xs[cols], ys[0], ys[rows])
    {
        return Err("open outer boundary".into());
    }
    let (shared_edges, uncertain_connections, mut sets) =
        grid_shared_edges(edges, &xs, &ys, false)?;
    let mut cells = Vec::new();
    for group in sets.groups() {
        let row = group.iter().map(|i| i / cols).min().unwrap();
        let col = group.iter().map(|i| i % cols).min().unwrap();
        let end_row = group.iter().map(|i| i / cols).max().unwrap() + 1;
        let end_col = group.iter().map(|i| i % cols).max().unwrap() + 1;
        if group.len() != (end_row - row) * (end_col - col) {
            return Err("non-rectangular cell topology".into());
        }
        cells.push(Cell {
            row,
            col,
            rows: end_row - row,
            cols: end_col - col,
            bbox: Rect::new(
                xs[col],
                ys[row],
                xs[end_col] - xs[col],
                ys[end_row] - ys[row],
            ),
            spans: Vec::new(),
            images: Vec::new(),
        });
    }
    if cells.len() < 2 {
        return Err("no distinct cells".into());
    }
    cells.sort_by_key(|c| (c.row, c.col));
    for c in &cells {
        let (right, bottom) = (c.bbox.x + c.bbox.width, c.bbox.y + c.bbox.height);
        if !complete(edges, true, c.bbox.y, c.bbox.x, right)
            || !complete(edges, true, bottom, c.bbox.x, right)
            || !complete(edges, false, c.bbox.x, c.bbox.y, bottom)
            || !complete(edges, false, right, c.bbox.y, bottom)
        {
            return Err("cell face boundary lacks source rule coverage".into());
        }
    }
    let topology = super::table_topology::evidence(
        rows,
        cols,
        &cells
            .iter()
            .map(|c| (c.row, c.col, c.rows, c.cols))
            .collect::<Vec<_>>(),
    )?;
    Ok(Grid {
        bbox: Rect::new(xs[0], ys[0], xs[cols] - xs[0], ys[rows] - ys[0]),
        xs,
        ys,
        cells,
        topology,
        source_edges: Vec::new(),
        search_region: None,
        shared_edges,
        uncertain_connections,
        regularized: edges
            .iter()
            .any(|e| matches!(e.paint_kind, "periodic_carrier" | "dash_carrier")),
        cell_region_evidence: Value::Null,
        axis_evidence: Value::Null,
    })
}

fn supported_grid(
    component: &[Edge],
    source: &[Edge],
    _logical: &[Edge],
    allow_linear: bool,
) -> Result<Grid, String> {
    let mut g = grid(component, allow_linear)?;
    let contributors = source.iter().copied().collect::<Vec<_>>();
    g.source_edges = source
        .iter()
        .filter(|edge| boundary_contributor(edge, &g, &contributors))
        .copied()
        .collect();
    for shared in &mut g.shared_edges {
        let horizontal = shared["horizontal"].as_bool().unwrap();
        let at = shared["at"].as_f64().unwrap() as f32;
        let from = shared["from"].as_f64().unwrap() as f32;
        let to = shared["to"].as_f64().unwrap() as f32;
        let covered = coverage(source, horizontal, at, from, to);
        let dash_unknown = source.iter().any(|edge| {
            edge.paint_kind == "dash_carrier"
                && edge.horizontal == horizontal
                && (edge.at - at).abs() <= EPS
                && edge.from < to
                && edge.to > from
        });
        shared["rawCoveragePt"] = json!(if dash_unknown { None } else { Some(covered) });
        shared["paintPresence"] = json!(if dash_unknown {
            "unknown_dash_coverage"
        } else if covered == 0. {
            "absent"
        } else if covered >= to - from - 0.0001 {
            "continuous"
        } else {
            "partial"
        });
    }
    Ok(g)
}

/// Preserve paint-qualified candidates. Only a complete, content-backed local
/// interpretation may rescue a rejected or ambiguous paint component.
fn resolved_grid(
    component: &[Edge],
    source: &[Edge],
    logical: &[Edge],
    text: &super::text_analysis::ObservedText,
    orders: &[Vec<usize>],
    clips: &pdf_oxide::elements::PathClipObservations,
    media: (f32, f32, f32, f32),
) -> Result<Grid, String> {
    let original = supported_grid(component, source, logical, true).map(|mut grid| {
        cell_regions::select(&mut grid, text, orders, clips, media);
        grid
    });
    if original
        .as_ref()
        .is_ok_and(|g| g.cell_region_evidence["status"] == "Selected")
    {
        return original;
    }
    match cell_regions::local_axes(component, source, text, orders, clips, media) {
        Ok(candidate) => {
            // A coarse painted face can contain several directly witnessed
            // content regions even when its own paint topology has no holes.
            // Keep an already equivalent paint-qualified result unchanged.
            if let Ok(grid) = original {
                let addresses = |g: &Grid| {
                    g.cells
                        .iter()
                        .map(|c| (c.row, c.col, c.rows, c.cols))
                        .collect::<Vec<_>>()
                };
                if grid.xs == candidate.xs
                    && grid.ys == candidate.ys
                    && addresses(&grid) == addresses(&candidate)
                {
                    return Ok(grid);
                }
            }
            Ok(candidate)
        }
        Err(reason) => match original {
            Ok(mut grid) => {
                if !grid.cell_region_evidence.is_null() {
                    grid.cell_region_evidence["localAxisAttempt"] = json!(reason);
                }
                Ok(grid)
            }
            Err(paint_reason) => Err(format!("{paint_reason}; local content regions: {reason}")),
        },
    }
}

/// Returns geometric table candidates and original content indices, not rhwp Table IR.
pub fn detect_ruled_tables(
    spans: &[TextSpan],
    paths: &[PathContent],
    images: &[PdfImage],
    media_box: (f32, f32, f32, f32),
    text: &super::text_analysis::ObservedText,
    span_orders: &[Vec<usize>],
    clips: &pdf_oxide::elements::PathClipObservations,
) -> Result<Value, String> {
    let (left, bottom, right, top) = media_box;
    if ![left, bottom, right, top].iter().all(|v| v.is_finite()) || right <= left || top <= bottom {
        return Err("invalid page geometry".into());
    }
    let (left, _, _, top) = media_box;
    // Table inference is optional. Stop its work at the existing cap, while
    // preserving original paths/text/images for positional lowering. Invalid
    // source geometry is still fatal; no inferred table is asserted here.
    let (source_edges, logical_edges, edges, deferred_reason) = match edges(paths, left, top) {
        Ok(source) => match normalize_edges(&source) {
            Ok((logical, canonical)) => (source, logical, canonical, None),
            Err(reason) => (source, Vec::new(), Vec::new(), Some(reason)),
        },
        Err(reason) if reason.contains("budget exceeded") => {
            (Vec::new(), Vec::new(), Vec::new(), Some(reason))
        }
        Err(reason) => return Err(reason),
    };
    let mut grids = Vec::new();
    let mut rejected = Vec::new();
    for group in component_groups(&edges) {
        let component: Vec<_> = group.iter().map(|i| edges[*i]).collect();
        match resolved_grid(
            &component,
            &source_edges,
            &logical_edges,
            text,
            span_orders,
            clips,
            media_box,
        ) {
            Ok(g) => grids.push(g),
            Err(reason) => rejected.push(json!({"edgeCount":component.len(),"reason":reason})),
        }
    }
    // Search the unclaimed evidence in already-verified cell regions; never clip crossing strokes.
    loop {
        let start_count = grids.len();
        if start_count > 128 {
            return Err("nested table candidate budget exceeded".into());
        }
        let mut regions = grids
            .iter()
            .flat_map(|g| g.cells.iter().map(|c| c.bbox))
            .collect::<Vec<_>>();
        regions.sort_by(|a, b| {
            (a.width * a.height)
                .total_cmp(&(b.width * b.height))
                .then(a.y.total_cmp(&b.y))
                .then(a.x.total_cmp(&b.x))
        });
        let mut claimed = grids
            .iter()
            .flat_map(|g| &g.source_edges)
            .map(|owned| geometry_key(&logical_edges[owned.source_segment]))
            .collect::<BTreeSet<_>>();
        for region in regions {
            let local = edges
                .iter()
                .filter(|edge| {
                    let bounds = if edge.horizontal {
                        Rect::new(edge.from, edge.at, edge.to - edge.from, 0.)
                    } else {
                        Rect::new(edge.at, edge.from, 0., edge.to - edge.from)
                    };
                    !claimed.contains(&geometry_key(edge)) && inside(bounds, region)
                })
                .copied()
                .collect::<Vec<_>>();
            for group in component_groups(&local) {
                let component = group.iter().map(|i| local[*i]).collect::<Vec<_>>();
                if let Ok(mut g) = resolved_grid(
                    &component,
                    &source_edges,
                    &logical_edges,
                    text,
                    span_orders,
                    clips,
                    media_box,
                ) {
                    if !inside(g.bbox, region)
                        || grids.iter().any(|existing| existing.bbox == g.bbox)
                    {
                        continue;
                    }
                    g.search_region = Some(region);
                    claimed.extend(
                        g.source_edges
                            .iter()
                            .map(|owned| geometry_key(&logical_edges[owned.source_segment])),
                    );
                    grids.push(g);
                    if grids.len() > 128 {
                        return Err("nested table candidate budget exceeded".into());
                    }
                }
            }
        }
        if grids.len() == start_count {
            break;
        }
    }
    grids.sort_by(|a, b| {
        a.bbox
            .y
            .total_cmp(&b.bbox.y)
            .then(a.bbox.x.total_cmp(&b.bbox.x))
    });
    let mut placements = super::table_hierarchy::compose(
        &grids
            .iter()
            .map(|g| (g.bbox, g.cells.iter().map(|c| c.bbox).collect()))
            .collect::<Vec<_>>(),
    )?;
    for (i, g) in grids.iter().enumerate() {
        if (g.uncertain_connections > 0 || !g.axis_evidence.is_null())
            && g.cell_region_evidence["status"] != "Selected"
        {
            placements[i].deferred = Some(
                "absent paint does not establish merged cells; competing separation retained"
                    .into(),
            );
        }
    }
    for _ in 0..=16 {
        let mut changed = false;
        for i in 0..placements.len() {
            if let Some((parent, _)) = placements[i].parent {
                if placements[i].deferred.is_some() != placements[parent].deferred.is_some() {
                    let held = if placements[i].deferred.is_none() {
                        i
                    } else {
                        parent
                    };
                    placements[held].deferred = Some("unresolved table hypothesis family".into());
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut unassigned = Vec::new();
    let mut ambiguous = Vec::new();
    let mut assign = |kind: &str, index: usize, bbox: Rect| {
        let mut owners = Vec::new();
        for (gi, g) in grids.iter().enumerate() {
            if !inside(bbox, g.bbox) {
                continue;
            }
            for (ci, c) in g.cells.iter().enumerate() {
                if inside(bbox, c.bbox) {
                    owners.push((gi, ci));
                }
            }
        }
        let deepest = owners
            .iter()
            .copied()
            .filter(|(gi, _)| {
                placements[*gi].deferred.is_none()
                    && owners.iter().all(|(other, _)| {
                        placements[*other].deferred.is_none()
                            && super::table_hierarchy::ancestor(&placements, *other, *gi)
                    })
                    && grids.iter().enumerate().all(|(other, g)| {
                        !overlaps(bbox, g.bbox)
                            || other == *gi
                            || !super::table_hierarchy::ancestor(&placements, *gi, other)
                    })
            })
            .collect::<Vec<_>>();
        if deepest.len() == 1 {
            let (gi, ci) = deepest[0];
            let cell = &mut grids[gi].cells[ci];
            if kind == "span" {
                cell.spans.push(index);
            } else {
                cell.images.push(index);
            }
        } else if grids.iter().any(|g| overlaps(bbox, g.bbox)) {
            ambiguous.push(json!({"kind":kind,"index":index,"ownerCount":owners.len()}));
        } else {
            unassigned.push(json!({"kind":kind,"index":index}));
        }
    };
    for (i, s) in spans.iter().enumerate() {
        if !s.text.is_empty() {
            if ![s.bbox.x, s.bbox.y, s.bbox.width, s.bbox.height]
                .iter()
                .all(|v| v.is_finite())
                || s.bbox.width < 0.0
                || s.bbox.height < 0.0
                || s.rotation_degrees != 0.0
                || s.wmode != 0
                || s.mirrored
            {
                return Err("unsupported text geometry in table detection".into());
            }
            assign(
                "span",
                i,
                Rect::new(
                    s.bbox.x - left,
                    top - s.bbox.y - s.bbox.height,
                    s.bbox.width,
                    s.bbox.height,
                ),
            );
        }
    }
    for (i, image) in images.iter().enumerate() {
        if let Some(b) = image.bbox() {
            if ![b.x, b.y, b.width, b.height].iter().all(|v| v.is_finite())
                || b.width <= 0.0
                || b.height <= 0.0
            {
                return Err("invalid image geometry".into());
            }
            assign(
                "image",
                i,
                Rect::new(b.x - left, top - b.y - b.height, b.width, b.height),
            );
        } else {
            return Err("image placement unavailable for table ownership".into());
        }
    }
    let tables = grids.iter().enumerate().map(|(gi,g)|json!({
            "bbox":g.bbox,"rows":g.ys.len()-1,"columns":g.xs.len()-1,
            "xBoundaries":g.xs,"yBoundaries":g.ys,
            "topology":g.topology,"axisEvidence":g.axis_evidence,
            "searchRegion":g.search_region,
            "sharedEdges":g.shared_edges,"uncertainConnections":g.uncertain_connections,
            "regularizedPeriodicBoundary":g.regularized,
            "cellInterpretation":if g.cell_region_evidence["status"]=="Selected" {
                "content-backed active-clip tiling; authored structure unknown"
            } else if !g.axis_evidence.is_null() { "unresolved active-content local grid" }
            else {"connected-face hypothesis; absent edges retain separation alternative"},
            "cellRegionEvidence":g.cell_region_evidence,
            "sourceBoundarySegments":g.source_edges.iter().map(|edge|json!({
                "path":edge.source_path,"segment":edge.source_segment,
                "horizontal":edge.horizontal,"at":edge.at,"from":edge.from,"to":edge.to,
                "strokeWidth":edge.stroke_width,"operation":edge.source_operation,"paintKind":edge.paint_kind
            })).collect::<Vec<_>>(),
            "sourcePathIndices":g.source_edges.iter().map(|edge|edge.source_path)
                .collect::<std::collections::BTreeSet<_>>().into_iter().filter(|path|
                    source_edges.iter().filter(|edge|edge.source_path==*path).all(|edge|
                        g.source_edges.iter().any(|owned|owned.source_segment==edge.source_segment)))
                .collect::<Vec<_>>(),
            "hierarchy":{"status":if placements[gi].deferred.is_some(){"Deferred"}else{"Confirmed"},
                "reason":placements[gi].deferred,"depth":placements[gi].depth,
                "parent":placements[gi].parent.map(|(table,cell)|json!({"table":table,"cell":cell})),
                "embedding":placements[gi].embedding.map(|e|json!({"x":e.x,"y":e.y,"width":e.width,"height":e.height}))},
            "cells":g.cells.iter().enumerate().map(|(ci,c)|json!({"row":c.row,"column":c.col,
                "rowspan":c.rows,"colspan":c.cols,"bbox":c.bbox,
                "spanIndices":c.spans,"imageIndices":c.images,
                "childTableIndices":placements.iter().enumerate().filter_map(|(child,p)|
                    (p.parent==Some((gi,ci))).then_some(child)).collect::<Vec<_>>()
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>();
    Ok(json!({
        "algorithm":"ruling-cellular-topology-v3","coordinateSystem":"points, top-left of MediaBox",
        "deferredReason":deferred_reason,"sourcePathCount":paths.len(),
        "scope":"stroke/fill-band geometric hypotheses; missing paint is not a merge proof",
        "tolerancePt":EPS,"edgeCount":source_edges.len(),"canonicalEdgeCount":edges.len(),
        "policy":{"maxSourceEdges":MAX_SOURCE_EDGES,"maxCanonicalEdges":MAX_EDGES,
            "maxFillBandThicknessPt":2.,"minFillBandAspect":4.,"periodicMinimumUnits":4,
            "periodicRelativeTolerance":0.05,"periodicAbsoluteTolerancePt":0.001},
        "boundaryNormalization":"exact geometric duplicates share coverage; original paint contributors retained",
        "boundaryObservations":source_edges.iter().map(|e|json!({
            "id":format!("boundary:{}",e.source_segment),"path":e.source_path,"operation":e.source_operation,
            "horizontal":e.horizontal,"at":e.at,"from":e.from,"to":e.to,"paintKind":e.paint_kind
        })).collect::<Vec<_>>(),
        "logicalBoundaries":edges.iter().enumerate().map(|(i,e)|json!({
            "id":format!("logical:{i}"),"horizontal":e.horizontal,"at":e.at,"from":e.from,"to":e.to,
            "paintKind":e.paint_kind,"periodicGapPt":e.periodic_gap,
            "sourceBoundaryIds":logical_edges.iter().enumerate()
                .filter(|(_,logical)| geometry_key(logical)==geometry_key(e))
                .map(|(raw,_)|format!("boundary:{raw}")).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),
        "junctionModel":"centerline intersections supported by perpendicular stroke ink and quantization tolerance",
        "nestedSearch":"verified-parent-cell regions; unclaimed complete evidence only",
        "tables":tables,
        "contentOrder":"original extraction index; semantic reading order deferred",
        "composition":"guarded cell-slot substitution; affine unit-square embeddings",
        "rejectedComponents":rejected,"unassignedContent":unassigned,"ambiguousContent":ambiguous
    }))
}
