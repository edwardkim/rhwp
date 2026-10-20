//! Detection only: vector-rule topology, independent of text wrapping and IR lowering.
use pdf_oxide::{
    elements::{PathContent, PathOperation},
    extractors::PdfImage,
    geometry::Rect,
    layout::TextSpan,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

const EPS: f32 = 0.5;
const MAX_EDGES: usize = 512;
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
    let mut edges = Vec::new();
    let mut invalid = false;
    let mut overflow = false;
    let mut add = |x1: f32, y1: f32, x2: f32, y2: f32, source_path: usize, stroke_width: f32| {
        if ![x1, y1, x2, y2].iter().all(|v| v.is_finite()) {
            invalid = true;
            return;
        }
        if edges.len() >= MAX_EDGES {
            overflow = true;
            return;
        }
        let (x1, x2, y1, y2) = (x1 - left, x2 - left, top - y1, top - y2);
        let source_segment = edges.len();
        if (y1 - y2).abs() <= EPS && (x1 - x2).abs() > EPS {
            edges.push(Edge {
                horizontal: true,
                at: (y1 + y2) / 2.0,
                from: x1.min(x2),
                to: x1.max(x2),
                source_path,
                source_segment,
                stroke_width,
            });
        } else if (x1 - x2).abs() <= EPS && (y1 - y2).abs() > EPS {
            edges.push(Edge {
                horizontal: false,
                at: (x1 + x2) / 2.0,
                from: y1.min(y2),
                to: y1.max(y2),
                source_path,
                source_segment,
                stroke_width,
            });
        }
    };
    for (source_path, path) in paths.iter().enumerate() {
        if path.stroke_color.is_none() || path.matrix.is_some() {
            continue;
        }
        let mut cursor = None;
        let mut start = None;
        for op in &path.operations {
            match *op {
                PathOperation::MoveTo(x, y) => {
                    cursor = Some((x, y));
                    start = cursor;
                }
                PathOperation::LineTo(x, y) => {
                    if let Some((px, py)) = cursor {
                        add(px, py, x, y, source_path, path.stroke_width);
                    }
                    cursor = Some((x, y));
                }
                PathOperation::Rectangle(x, y, w, h) => {
                    add(x, y, x + w, y, source_path, path.stroke_width);
                    add(x + w, y, x + w, y + h, source_path, path.stroke_width);
                    add(x + w, y + h, x, y + h, source_path, path.stroke_width);
                    add(x, y + h, x, y, source_path, path.stroke_width);
                    cursor = Some((x, y));
                    start = cursor;
                }
                PathOperation::ClosePath => {
                    if let (Some((x, y)), Some((sx, sy))) = (cursor, start) {
                        add(x, y, sx, sy, source_path, path.stroke_width);
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
    if invalid {
        return Err("non-finite rule geometry".into());
    }
    if overflow {
        return Err("ruled-table edge budget exceeded".into());
    }
    Ok(edges)
}

fn axes(mut values: Vec<f32>) -> Vec<f32> {
    values.sort_by(f32::total_cmp);
    let mut groups: Vec<Vec<f32>> = Vec::new();
    for v in values {
        if let Some(group) = groups.last_mut().filter(|g| v - g[0] <= EPS) {
            group.push(v);
        } else {
            groups.push(vec![v]);
        }
    }
    groups
        .iter()
        .map(|g| g.iter().sum::<f32>() / g.len() as f32)
        .collect()
}

fn same_geometry(a: &Edge, b: &Edge) -> bool {
    a.horizontal == b.horizontal && a.at == b.at && a.from == b.from && a.to == b.to
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
                other.horizontal != edge.horizontal
                    && other.at >= edge.from
                    && other.at <= edge.to
                    && edge.at >= other.from - EPS
                    && edge.at <= other.to + EPS
                    && (other.at - value).abs() <= other.stroke_width / 2. + EPS
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
    if target_end - target_start <= EPS {
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
        if a > end + EPS {
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
    let mut sets = Sets::new(rows * cols);
    // A missing internal rule joins neighboring tiles; a partial rule is ambiguous, not an invented merge.
    for row in 0..rows {
        for col in 0..cols {
            if col + 1 < cols {
                let length = ys[row + 1] - ys[row];
                let covered = coverage(edges, false, xs[col + 1], ys[row], ys[row + 1]);
                if covered <= EPS {
                    sets.join(row * cols + col, row * cols + col + 1);
                } else if covered < length - EPS {
                    return Err("partial internal vertical boundary".into());
                }
            }
            if row + 1 < rows {
                let length = xs[col + 1] - xs[col];
                let covered = coverage(edges, true, ys[row + 1], xs[col], xs[col + 1]);
                if covered <= EPS {
                    sets.join(row * cols + col, (row + 1) * cols + col);
                } else if covered < length - EPS {
                    return Err("partial internal horizontal boundary".into());
                }
            }
        }
    }
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
    let topology = crate::table_topology::evidence(
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
    })
}

fn supported_grid(
    component: &[Edge],
    source: &[Edge],
    logical: &[Edge],
    allow_linear: bool,
) -> Result<Grid, String> {
    let mut g = grid(component, allow_linear)?;
    let contributors = source
        .iter()
        .filter(|edge| {
            logical.iter().any(|logical| {
                logical.source_segment == edge.source_segment
                    && component.iter().any(|other| same_geometry(logical, other))
            })
        })
        .copied()
        .collect::<Vec<_>>();
    g.source_edges = contributors
        .iter()
        .filter(|edge| boundary_contributor(edge, &g, &contributors))
        .copied()
        .collect();
    Ok(g)
}

/// Returns geometric table candidates and original content indices, not rhwp Table IR.
pub fn detect_ruled_tables(
    spans: &[TextSpan],
    paths: &[PathContent],
    images: &[PdfImage],
    media_box: (f32, f32, f32, f32),
) -> Result<Value, String> {
    let (left, bottom, right, top) = media_box;
    if ![left, bottom, right, top].iter().all(|v| v.is_finite()) || right <= left || top <= bottom {
        return Err("invalid page geometry".into());
    }
    let (left, _, _, top) = media_box;
    let source_edges = edges(paths, left, top)?;
    let logical_edges = source_edges
        .iter()
        .map(|edge| project_junctions(*edge, &source_edges))
        .collect::<Vec<_>>();
    let mut edges = Vec::new();
    // This is geometric coverage normalization, not a claim that distinct paints are interchangeable.
    for edge in &logical_edges {
        if !edges.iter().any(|other| same_geometry(edge, other)) {
            edges.push(*edge);
        }
    }
    let mut grids = Vec::new();
    let mut rejected = Vec::new();
    for group in component_groups(&edges) {
        let component: Vec<_> = group.iter().map(|i| edges[*i]).collect();
        match supported_grid(&component, &source_edges, &logical_edges, false) {
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
                if let Ok(mut g) = supported_grid(&component, &source_edges, &logical_edges, true) {
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
    let placements = crate::table_hierarchy::compose(
        &grids
            .iter()
            .map(|g| (g.bbox, g.cells.iter().map(|c| c.bbox).collect()))
            .collect::<Vec<_>>(),
    )?;
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
                            && crate::table_hierarchy::ancestor(&placements, *other, *gi)
                    })
                    && grids.iter().enumerate().all(|(other, g)| {
                        !overlaps(bbox, g.bbox)
                            || other == *gi
                            || !crate::table_hierarchy::ancestor(&placements, *gi, other)
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
    Ok(json!({
        "algorithm":"ruling-cellular-topology-v2","coordinateSystem":"points, top-left of MediaBox",
        "scope":"stroked axis-aligned ruled grids; geometric candidates, not semantic proof",
        "tolerancePt":EPS,"edgeCount":source_edges.len(),"canonicalEdgeCount":edges.len(),
        "boundaryNormalization":"exact geometric duplicates share coverage; original paint contributors retained",
        "junctionModel":"centerline intersections supported by perpendicular stroke ink and quantization tolerance",
        "nestedSearch":"verified-parent-cell regions; unclaimed complete evidence only",
        "tables":grids.iter().enumerate().map(|(gi,g)|json!({
            "bbox":g.bbox,"rows":g.ys.len()-1,"columns":g.xs.len()-1,
            "xBoundaries":g.xs,"yBoundaries":g.ys,
            "topology":g.topology,
            "searchRegion":g.search_region,
            "sourceBoundarySegments":g.source_edges.iter().map(|edge|json!({
                "path":edge.source_path,"segment":edge.source_segment,
                "horizontal":edge.horizontal,"at":edge.at,"from":edge.from,"to":edge.to,
                "strokeWidth":edge.stroke_width
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
        })).collect::<Vec<_>>(),
        "contentOrder":"original extraction index; semantic reading order deferred",
        "composition":"guarded cell-slot substitution; affine unit-square embeddings",
        "rejectedComponents":rejected,"unassignedContent":unassigned,"ambiguousContent":ambiguous
    }))
}
