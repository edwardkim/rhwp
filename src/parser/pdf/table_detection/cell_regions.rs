//! Resolve absent-edge alternatives only from a complete, content-backed clip tiling.
//! This selects an editable interpretation, never the PDF author's original objects.
use super::{Cell, Grid, EPS};
use crate::parser::pdf::{
    allocation::{message, Allocation},
    value_macro::request_json,
};
use crate::parser::pdf::{table_inference, table_topology, text_analysis::ObservedText};
use pdf_oxide::{
    elements::{PathClip, PathClipObservations},
    geometry::Rect,
};
use serde_json::Value;
use std::collections::BTreeMap;

mod empty_faces;
mod local_axes;
pub(super) use local_axes::derive as local_axes;

struct Witness {
    bounds: Rect,
    glyphs: Vec<usize>,
    orders: std::collections::BTreeSet<Vec<usize>>,
}

struct Separator {
    horizontal: bool,
    at: f32,
    from: f32,
    to: f32,
}
impl Separator {
    fn crosses(&self, grid: &Grid, (r, c, rs, cs): (usize, usize, usize, usize)) -> bool {
        let (lo, hi, start, end) = if self.horizontal {
            (grid.ys[r], grid.ys[r + rs], grid.xs[c], grid.xs[c + cs])
        } else {
            (grid.xs[c], grid.xs[c + cs], grid.ys[r], grid.ys[r + rs])
        };
        self.at > lo + EPS && self.at < hi - EPS && self.from < end - EPS && self.to > start + EPS
    }
}
fn separators(allocation: Allocation<'_>, grid: &Grid) -> Result<Vec<Separator>, String> {
    allocation
        .collect(
            grid.shared_edges
                .iter()
                .filter(|e| e["regularizedCoveragePt"].as_f64().unwrap_or(0.) > 0.)
                .map(|e| Separator {
                    horizontal: e["horizontal"].as_bool().unwrap(),
                    at: e["at"].as_f64().unwrap() as f32,
                    from: e["from"].as_f64().unwrap() as f32,
                    to: e["to"].as_f64().unwrap() as f32,
                }),
        )
        .map_err(message)
}

fn insert_order(
    allocation: Allocation<'_>,
    orders: &mut std::collections::BTreeSet<Vec<usize>>,
    order: &[usize],
) -> Result<(), String> {
    if !orders.contains(order) {
        allocation.tree_entry::<Vec<usize>, ()>().map_err(message)?;
        orders.insert(allocation.copy_slice(order).map_err(message)?);
    }
    Ok(())
}

/// Share the actual text-show/clip association between axis proposals and owners.
fn observe(
    outer: Rect,
    text: &ObservedText,
    orders: &[Vec<usize>],
    clips: &PathClipObservations,
    media: (f32, f32, f32, f32),
) -> Result<Vec<Witness>, String> {
    let allocation = text.allocation();
    if text.glyph_observations_incomplete || clips.text_uses_omitted > 0 {
        return Err("text or active-clip observation incomplete".into());
    }
    let mut uses = BTreeMap::new();
    for (order, clip) in &clips.text_uses {
        if !uses.contains_key(order) {
            allocation.tree_entry::<usize, usize>().map_err(message)?;
        }
        uses.insert(order, clip);
    }
    let mut regions: Vec<Witness> = Vec::new();
    let mut work = 0usize;
    for (gi, glyph) in text.glyphs.iter().enumerate() {
        if !super::overlaps(glyph.bounds, outer) {
            continue;
        }
        if glyph.geometry_quality == "uniform_span_approximation"
            || !super::inside(glyph.bounds, outer)
        {
            return Err("content geometry cannot establish cell ownership".into());
        }
        let order = orders
            .get(glyph.source_span)
            .ok_or("missing span invocation path")?;
        let clip = uses.get(order).ok_or("missing text-paint clip witness")?;
        if clip.is_none() {
            return Err("text has no active cell clip".into());
        }
        let bounds = effective_clip(clip.as_deref(), media).ok_or("unsupported text clip chain")?;
        if !super::inside(bounds, outer)
            || (!glyph.text.is_whitespace() && !super::inside(glyph.bounds, bounds))
        {
            return Err("glyph or active clip crosses the local exterior".into());
        }
        work += regions.len() + 1;
        if work > 1_000_000 {
            return Err("cell region observation work exceeded".into());
        }
        if let Some(region) = regions.iter_mut().find(|r| r.bounds == bounds) {
            allocation.push(&mut region.glyphs, gi).map_err(message)?;
            insert_order(allocation, &mut region.orders, order)?;
        } else {
            if regions.len() == 512 {
                return Err("cell region observation count exceeded".into());
            }
            let mut observed_orders = std::collections::BTreeSet::new();
            insert_order(allocation, &mut observed_orders, order)?;
            allocation
                .push(
                    &mut regions,
                    Witness {
                        bounds,
                        glyphs: allocation.collect([gi]).map_err(message)?,
                        orders: observed_orders,
                    },
                )
                .map_err(message)?;
        }
    }
    Ok(regions)
}

pub(super) fn effective_clip(
    mut clip: Option<&PathClip>,
    media: (f32, f32, f32, f32),
) -> Option<Rect> {
    let mut bounds = Rect::new(media.0, media.1, media.2 - media.0, media.3 - media.1);
    let mut depth = 0;
    while let Some(current) = clip {
        depth += 1;
        if depth > 32 || current.operations.len() > 8 {
            return None;
        }
        // PathClip coordinates already include the CTM, including Form matrices.
        let b = table_inference::rectangle(&current.operations)?;
        let x = bounds.x.max(b.x);
        let y = bounds.y.max(b.y);
        let right = (bounds.x + bounds.width).min(b.x + b.width);
        let top = (bounds.y + bounds.height).min(b.y + b.height);
        if right <= x || top <= y {
            return None;
        }
        bounds = Rect::new(x, y, right - x, top - y);
        clip = current.parent.as_deref();
    }
    Some(table_inference::page_rect(bounds, media))
}

/// All interior glyphs must have a directly observed text-paint clip. Missing,
/// nonrectangular, partial and competing scopes remain positional, with a reason.
pub(super) fn select(
    grid: &mut Grid,
    text: &ObservedText,
    orders: &[Vec<usize>],
    clips: &PathClipObservations,
    media: (f32, f32, f32, f32),
) -> Result<(), String> {
    let allocation = text.allocation();
    if grid.uncertain_connections == 0 && grid.axis_evidence.is_null() {
        return Ok(());
    }
    grid.cell_region_evidence = request_json!(allocation,{"status":"Deferred", "reason":"incomplete content-backed clip tiling",
        "authoredStructure":"unknown", "alternatives":["connected_faces","separate_cells"]})?;
    let mut run = || -> Result<_, String> {
        let mut observed = observe(grid.bbox, text, orders, clips, media)?;
        // A real empty text-show has no glyph entry, but its live cell-sized
        // clip is still positive region evidence. Only existing axes qualify;
        // an arbitrary padding/artwork clip must not invent blank-cell axes.
        let mut span_orders = allocation.collect(orders.iter()).map_err(message)?;
        span_orders.sort_unstable();
        span_orders.dedup();
        for (order, clip) in &clips.text_uses {
            if span_orders.binary_search(&order).is_ok() {
                continue;
            }
            let Some(bounds) = clip.as_deref().and_then(|c| effective_clip(Some(c), media)) else {
                continue;
            };
            if !super::inside(bounds, grid.bbox)
                || [bounds.x, bounds.x + bounds.width]
                    .iter()
                    .any(|at| grid.xs.iter().all(|v| (*v - at).abs() > EPS))
                || [bounds.y, bounds.y + bounds.height]
                    .iter()
                    .any(|at| grid.ys.iter().all(|v| (*v - at).abs() > EPS))
            {
                continue;
            }
            if let Some(region) = observed.iter_mut().find(|r| r.bounds == bounds) {
                insert_order(allocation, &mut region.orders, order)?;
            } else {
                if observed.len() == 512 {
                    return Err("cell region observation count exceeded".into());
                }
                let mut observed_orders = std::collections::BTreeSet::new();
                insert_order(allocation, &mut observed_orders, order)?;
                allocation
                    .push(
                        &mut observed,
                        Witness {
                            bounds,
                            glyphs: Vec::new(),
                            orders: observed_orders,
                        },
                    )
                    .map_err(message)?;
            }
        }
        let mut faces = BTreeMap::<(usize, usize, usize, usize), Vec<usize>>::new();
        let separators = separators(allocation, grid)?;
        let mut clip_orders = BTreeMap::<_, std::collections::BTreeSet<Vec<usize>>>::new();
        let mut work = 0usize;
        for region in observed {
            let bounds = region.bounds;
            let axis = |values: &[f32], at: f32| {
                let mut matching = values
                    .iter()
                    .enumerate()
                    .filter(|(_, v)| (**v - at).abs() <= EPS);
                let (index, _) = matching.next()?;
                matching.next().is_none().then_some(index)
            };
            let c = axis(&grid.xs, bounds.x).ok_or("clip does not match local column axes")?;
            let r = axis(&grid.ys, bounds.y).ok_or("clip does not match local row axes")?;
            let ce = axis(&grid.xs, bounds.x + bounds.width)
                .ok_or("clip does not match local column end")?;
            let re = axis(&grid.ys, bounds.y + bounds.height)
                .ok_or("clip does not match local row end")?;
            if r >= re || c >= ce {
                return Err("glyph crosses its observed cell clip".into());
            }
            let address = (r, c, re - r, ce - c);
            // Validate a region once, using all its common glyph observations below.
            // A broad clip cannot erase a positively observed separating rule.
            if !faces.contains_key(&address) {
                work += separators.len();
                if work > 1_000_000 {
                    return Err("cell region observation work exceeded".into());
                }
                if separators.iter().any(|e| e.crosses(grid, address)) {
                    return Err("content clip contradicts a painted separator".into());
                }
            }
            if !faces.contains_key(&address) {
                allocation
                    .tree_entry::<(usize, usize, usize, usize), Vec<usize>>()
                    .map_err(message)?;
            }
            let glyphs = faces.entry(address).or_default();
            allocation
                .reserve(glyphs, region.glyphs.len())
                .map_err(message)?;
            glyphs.extend(region.glyphs);
            if !clip_orders.contains_key(&address) {
                allocation.tree_entry::<(usize,usize,usize,usize),std::collections::BTreeSet<Vec<usize>>>().map_err(message)?;
            }
            let target = clip_orders.entry(address).or_default();
            for order in region.orders {
                if !target.contains(&order) {
                    allocation.tree_entry::<Vec<usize>, ()>().map_err(message)?;
                    target.insert(order);
                }
            }
        }
        let (rows, cols) = (grid.ys.len() - 1, grid.xs.len() - 1);
        let mut owners = allocation.vec(rows * cols).map_err(message)?;
        owners.resize(rows * cols, None);
        for (id, &(r, c, rs, cs)) in faces.keys().enumerate() {
            for y in r..r + rs {
                for x in c..c + cs {
                    if owners[y * cols + x].replace(id).is_some() {
                        return Err("competing content clip regions overlap".into());
                    }
                }
            }
        }
        let empty_scopes = empty_faces::from_clip_scopes(
            allocation,
            grid,
            &mut faces,
            &mut owners,
            clips,
            media,
            &mut work,
            &separators,
        )?;
        empty_faces::complete(text.allocation(), grid, &mut faces, &mut owners, &mut work)?;
        if faces.len() < 2 {
            return Err("insufficient independent content regions".into());
        }
        let keys = allocation.collect(faces.keys().copied()).map_err(message)?;
        let topology = table_topology::evidence(text.allocation(), rows, cols, &keys)
            .map_err(|_| "invalid content-backed cell topology")?;
        let mut evidence = allocation.vec(faces.len()).map_err(message)?;
        let empty_orders = std::collections::BTreeSet::new();
        for (&(r, c, rs, cs), glyphs) in &faces {
            let ids = allocation
                .collect(glyphs.iter().map(|i| &text.glyphs[*i].id))
                .map_err(message)?;
            let source_orders = clip_orders
                .get(&(r, c, rs, cs))
                .or_else(|| empty_scopes.get(&(r, c, rs, cs)))
                .unwrap_or(&empty_orders);
            evidence.push(request_json!(allocation,{
                "row":r,"column":c,"rowspan":rs,"colspan":cs,
                "basis":if clip_orders.contains_key(&(r,c,rs,cs)) {"active_text_clip"}
                    else if empty_scopes.contains_key(&(r,c,rs,cs)) {"empty_grid_region_clip_scope"}else{"closed_paint_face"},
                "glyphIds":ids,"sourceOrders":source_orders
            })?);
        }
        Ok((keys, topology, evidence, owners))
    };
    match run() {
        Err(reason) => allocation
            .set(
                &mut grid.cell_region_evidence,
                "reason",
                allocation.value(&reason).map_err(message)?,
            )
            .map_err(message)?,
        Ok((keys, topology, evidence, owners)) => {
            let cols = grid.xs.len() - 1;
            for edge in &mut grid.shared_edges {
                if edge["regularizedCoveragePt"].as_f64().unwrap_or(0.) != 0. {
                    continue;
                }
                let horizontal = edge["horizontal"].as_bool().unwrap();
                let at = edge["at"].as_f64().unwrap() as f32;
                let from = edge["from"].as_f64().unwrap() as f32;
                let (r, c) = if horizontal {
                    (
                        grid.ys.iter().position(|v| (*v - at).abs() <= EPS).unwrap(),
                        grid.xs
                            .iter()
                            .position(|v| (*v - from).abs() <= EPS)
                            .unwrap(),
                    )
                } else {
                    (
                        grid.ys
                            .iter()
                            .position(|v| (*v - from).abs() <= EPS)
                            .unwrap(),
                        grid.xs.iter().position(|v| (*v - at).abs() <= EPS).unwrap(),
                    )
                };
                let same = if horizontal {
                    owners[(r - 1) * cols + c] == owners[r * cols + c]
                } else {
                    owners[r * cols + c - 1] == owners[r * cols + c]
                };
                allocation
                    .set(
                        edge,
                        "separatorRole",
                        allocation
                            .value(&if same {
                                "content_region_passage"
                            } else {
                                "unpainted_content_separator"
                            })
                            .map_err(message)?,
                    )
                    .map_err(message)?;
            }
            grid.cells = allocation
                .collect(keys.into_iter().map(|(r, c, rs, cs)| Cell {
                    row: r,
                    col: c,
                    rows: rs,
                    cols: cs,
                    bbox: Rect::new(
                        grid.xs[c],
                        grid.ys[r],
                        grid.xs[c + cs] - grid.xs[c],
                        grid.ys[r + rs] - grid.ys[r],
                    ),
                    spans: Vec::new(),
                    images: Vec::new(),
                }))
                .map_err(message)?;
            grid.topology = topology;
            grid.cell_region_evidence = request_json!(allocation,{"status":"Selected", "basis":"complete content-region tiling with separately identified empty clip scopes",
                "authoredStructure":"unknown", "alternatives":["connected_faces","separate_cells"],
                "unpaintedSeparatorsRemainUnpainted":true})?;
            allocation
                .set(
                    &mut grid.cell_region_evidence,
                    "regions",
                    Value::Array(evidence),
                )
                .map_err(message)?;
        }
    }
    Ok(())
}
