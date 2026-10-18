//! Resolve absent-edge alternatives only from a complete, content-backed clip tiling.
//! This selects an editable interpretation, never the PDF author's original objects.
use super::{Cell, Grid, EPS};
use crate::parser::pdf::{table_inference, table_topology, text_analysis::ObservedText};
use pdf_oxide::{
    elements::{PathClip, PathClipObservations},
    geometry::Rect,
};
use serde_json::json;
use std::collections::BTreeMap;

mod empty_faces;
mod local_axes;
pub(super) use local_axes::derive as local_axes;

struct Witness {
    bounds: Rect,
    glyphs: Vec<usize>,
}

/// Share the actual text-show/clip association between axis proposals and owners.
fn observe(
    outer: Rect,
    text: &ObservedText,
    orders: &[Vec<usize>],
    clips: &PathClipObservations,
    media: (f32, f32, f32, f32),
) -> Result<Vec<Witness>, &'static str> {
    if text.glyph_observations_incomplete || clips.text_uses_omitted > 0 {
        return Err("text or active-clip observation incomplete");
    }
    let uses = clips
        .text_uses
        .iter()
        .map(|(order, clip)| (order, clip))
        .collect::<BTreeMap<_, _>>();
    let mut regions: Vec<Witness> = Vec::new();
    let mut work = 0usize;
    for (gi, glyph) in text.glyphs.iter().enumerate() {
        if glyph.text.is_whitespace() || !super::overlaps(glyph.bounds, outer) {
            continue;
        }
        if glyph.geometry_quality == "uniform_span_approximation"
            || !super::inside(glyph.bounds, outer)
        {
            return Err("content geometry cannot establish cell ownership");
        }
        let order = orders
            .get(glyph.source_span)
            .ok_or("missing span invocation path")?;
        let clip = uses.get(order).ok_or("missing text-paint clip witness")?;
        if clip.is_none() {
            return Err("text has no active cell clip");
        }
        let bounds = effective_clip(clip.as_deref(), media).ok_or("unsupported text clip chain")?;
        if !super::inside(bounds, outer) || !super::inside(glyph.bounds, bounds) {
            return Err("glyph or active clip crosses the local exterior");
        }
        work += regions.len() + 1;
        if work > 1_000_000 {
            return Err("cell region observation work exceeded");
        }
        if let Some(region) = regions.iter_mut().find(|r| r.bounds == bounds) {
            region.glyphs.push(gi);
        } else {
            if regions.len() == 512 {
                return Err("cell region observation count exceeded");
            }
            regions.push(Witness {
                bounds,
                glyphs: vec![gi],
            });
        }
    }
    Ok(regions)
}

fn effective_clip(mut clip: Option<&PathClip>, media: (f32, f32, f32, f32)) -> Option<Rect> {
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
) {
    if grid.uncertain_connections == 0 && grid.axis_evidence.is_null() {
        return;
    }
    grid.cell_region_evidence = json!({"status":"Deferred", "reason":"incomplete content-backed clip tiling",
        "authoredStructure":"unknown", "alternatives":["connected_faces","separate_cells"]});
    let mut run = || -> Result<_, &'static str> {
        let observed = observe(grid.bbox, text, orders, clips, media)?;
        let mut faces = BTreeMap::<(usize, usize, usize, usize), Vec<usize>>::new();
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
                return Err("glyph crosses its observed cell clip");
            }
            let address = (r, c, re - r, ce - c);
            // Validate a region once, using all its common glyph observations below.
            // A broad clip cannot erase a positively observed separating rule.
            for edge in grid
                .shared_edges
                .iter()
                .filter(|_| !faces.contains_key(&address))
            {
                work += 1;
                if work > 1_000_000 {
                    return Err("cell region observation work exceeded");
                }
                if edge["regularizedCoveragePt"].as_f64().unwrap_or(0.) <= 0. {
                    continue;
                }
                let horizontal = edge["horizontal"].as_bool().unwrap_or(false);
                let at = edge["at"].as_f64().unwrap_or(0.) as f32;
                let from = edge["from"].as_f64().unwrap_or(0.) as f32;
                let to = edge["to"].as_f64().unwrap_or(0.) as f32;
                let (lo, hi, start, end) = if horizontal {
                    (grid.ys[r], grid.ys[re], grid.xs[c], grid.xs[ce])
                } else {
                    (grid.xs[c], grid.xs[ce], grid.ys[r], grid.ys[re])
                };
                if at > lo + EPS && at < hi - EPS && from < end - EPS && to > start + EPS {
                    return Err("content clip contradicts a painted separator");
                }
            }
            faces.entry(address).or_default().extend(region.glyphs);
        }
        let (rows, cols) = (grid.ys.len() - 1, grid.xs.len() - 1);
        let mut owners = vec![None; rows * cols];
        for (id, &(r, c, rs, cs)) in faces.keys().enumerate() {
            for y in r..r + rs {
                for x in c..c + cs {
                    if owners[y * cols + x].replace(id).is_some() {
                        return Err("competing content clip regions overlap");
                    }
                }
            }
        }
        empty_faces::complete(grid, &mut faces, &mut owners, &mut work)?;
        if faces.len() < 2 {
            return Err("insufficient independent content regions");
        }
        let keys = faces.keys().copied().collect::<Vec<_>>();
        let topology = table_topology::evidence(rows, cols, &keys)
            .map_err(|_| "invalid content-backed cell topology")?;
        let evidence = faces.iter().map(|(&(r,c,rs,cs),glyphs)| json!({
            "row":r,"column":c,"rowspan":rs,"colspan":cs,
            "basis":if glyphs.is_empty() {"closed_paint_face"} else {"active_text_clip"},
            "glyphIds":glyphs.iter().map(|i| &text.glyphs[*i].id).collect::<Vec<_>>(),
            "sourceOrders":glyphs.iter().map(|i| &orders[text.glyphs[*i].source_span]).collect::<std::collections::BTreeSet<_>>()
        })).collect::<Vec<_>>();
        Ok((keys, topology, evidence, owners))
    };
    match run() {
        Err(reason) => grid.cell_region_evidence["reason"] = json!(reason),
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
                edge["separatorRole"] = json!(if same {
                    "content_region_passage"
                } else {
                    "unpainted_content_separator"
                });
            }
            grid.cells = keys
                .into_iter()
                .map(|(r, c, rs, cs)| Cell {
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
                })
                .collect();
            grid.topology = topology;
            grid.cell_region_evidence = json!({"status":"Selected", "basis":"complete content-backed active-clip tiling",
                "authoredStructure":"unknown", "alternatives":["connected_faces","separate_cells"],
                "regions":evidence,"unpaintedSeparatorsRemainUnpainted":true});
        }
    }
}
