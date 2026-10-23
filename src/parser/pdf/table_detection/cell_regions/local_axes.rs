//! Resolve local axes from oriented rules and a complete content-clip tiling.
use super::*;
use crate::parser::pdf::table_detection::{
    axes, boundary_contributor, complete, grid_shared_edges, Edge, MAX_TILES,
};

pub(in crate::parser::pdf::table_detection) fn derive(
    component: &[Edge],
    source: &[Edge],
    text: &ObservedText,
    orders: &[Vec<usize>],
    clips: &PathClipObservations,
    media: (f32, f32, f32, f32),
) -> Result<Grid, String> {
    // Exterior axes are ruling centerlines. Horizontal overhangs used for
    // painted joins must not move a vertical side to their endpoint.
    let mut xs = axes(
        component
            .iter()
            .filter(|e| !e.horizontal)
            .map(|e| e.at)
            .collect(),
    );
    let mut ys = axes(
        component
            .iter()
            .filter(|e| e.horizontal)
            .map(|e| e.at)
            .collect(),
    );
    if xs.len() < 2 || ys.len() < 2 {
        return Err("no closed paint exterior".into());
    }
    let (x, right, y, bottom) = (xs[0], *xs.last().unwrap(), ys[0], *ys.last().unwrap());
    let paint_exterior_closed = complete(component, true, y, x, right)
        && complete(component, true, bottom, x, right)
        && complete(component, false, x, y, bottom)
        && complete(component, false, right, y, bottom);
    // A continued cell need not paint the page-fragment's closing edge. Its
    // active content clip can establish an editable region without inventing
    // the missing ink. Selection still requires complete, noncompeting tiling
    // and rejects any clip that crosses a positively painted separator.
    let outer = Rect::new(x, y, right - x, bottom - y);
    let regions = observe(outer, text, orders, clips, media)?;
    if regions.is_empty() {
        return Err("no active content regions".into());
    }
    // Insets used only to clip text padding must not invent an empty frame
    // around an otherwise paint-qualified table. A local tiling proposal must
    // reach the independently known exterior; holes inside remain undecided.
    let min_x = regions
        .iter()
        .map(|r| r.bounds.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = regions
        .iter()
        .map(|r| r.bounds.y)
        .fold(f32::INFINITY, f32::min);
    let max_x = regions
        .iter()
        .map(|r| r.bounds.x + r.bounds.width)
        .fold(f32::NEG_INFINITY, f32::max);
    let max_y = regions
        .iter()
        .map(|r| r.bounds.y + r.bounds.height)
        .fold(f32::NEG_INFINITY, f32::max);
    if (min_x - x).abs() > EPS
        || (min_y - y).abs() > EPS
        || (max_x - right).abs() > EPS
        || (max_y - bottom).abs() > EPS
    {
        return Err("content regions do not span the paint exterior".into());
    }
    // Internal axes originate from oriented paint or witnessed clips, never from
    // arbitrary artwork endpoints or text-gap midpoints.
    xs = axes(
        component
            .iter()
            .filter(|e| !e.horizontal)
            .map(|e| e.at)
            .collect(),
    );
    ys = axes(
        component
            .iter()
            .filter(|e| e.horizontal)
            .map(|e| e.at)
            .collect(),
    );
    // Keep the independently observed paint axes as the reference. Clip
    // quantization inside EPS must not move the exterior or add a thin tile.
    for region in &regions {
        for at in [region.bounds.x, region.bounds.x + region.bounds.width] {
            if xs.iter().all(|v| (*v - at).abs() > EPS) {
                xs.push(at);
            }
        }
        for at in [region.bounds.y, region.bounds.y + region.bounds.height] {
            if ys.iter().all(|v| (*v - at).abs() > EPS) {
                ys.push(at);
            }
        }
    }
    // Empty column/row regions can have their own local subdivision. Require
    // two positively ruled sides and no already-owned content; a glyph inset
    // or a free artwork clip cannot introduce axes. Selection below still
    // requires complete, noncompeting empty-region coverage.
    let mut empty_axis_scopes = Vec::new();
    if clips.omitted == 0 {
        for scope in &clips.clips {
            let Some(b) = effective_clip(Some(&scope.clip), media) else {
                continue;
            };
            if !super::super::inside(b, outer)
                || regions.iter().any(|r| {
                    b.x < r.bounds.x + r.bounds.width - EPS
                        && r.bounds.x < b.x + b.width - EPS
                        && b.y < r.bounds.y + r.bounds.height - EPS
                        && r.bounds.y < b.y + b.height - EPS
                })
            {
                continue;
            }
            let vertical = complete(component, false, b.x, b.y, b.y + b.height)
                && complete(component, false, b.x + b.width, b.y, b.y + b.height);
            let horizontal = complete(component, true, b.y, b.x, b.x + b.width)
                && complete(component, true, b.y + b.height, b.x, b.x + b.width);
            if vertical {
                for at in [b.y, b.y + b.height] {
                    if ys.iter().all(|v| (*v - at).abs() > EPS) {
                        ys.push(at);
                    }
                }
            }
            if horizontal {
                for at in [b.x, b.x + b.width] {
                    if xs.iter().all(|v| (*v - at).abs() > EPS) {
                        xs.push(at);
                    }
                }
            }
            if vertical || horizontal {
                empty_axis_scopes.push((b, scope.source_order.clone()));
            }
        }
    }
    let (xs, ys) = (axes(xs), axes(ys));
    let (cols, rows) = (xs.len() - 1, ys.len() - 1);
    if rows
        .checked_mul(cols)
        .is_none_or(|n| !(2..=MAX_TILES).contains(&n))
    {
        return Err("local content tile budget exceeded".into());
    }
    let (shared_edges, uncertain_connections, _) = grid_shared_edges(component, &xs, &ys, true)?;
    let cells = (0..rows)
        .flat_map(|r| (0..cols).map(move |c| (r, c)))
        .map(|(r, c)| Cell {
            row: r,
            col: c,
            rows: 1,
            cols: 1,
            bbox: Rect::new(xs[c], ys[r], xs[c + 1] - xs[c], ys[r + 1] - ys[r]),
            spans: Vec::new(),
            images: Vec::new(),
        })
        .collect();
    let evidence = regions
        .iter()
        .map(|region| {
            json!({"bounds":region.bounds,
        "sourceOrders":region.glyphs.iter().map(|i| &orders[text.glyphs[*i].source_span])
            .collect::<std::collections::BTreeSet<_>>() })
        })
        .collect::<Vec<_>>();
    let axis_sources = |values: &[f32], horizontal: bool| {
        values
            .iter()
            .map(|at| {
                let paint_refs = component
                    .iter()
                    .filter(|e| e.horizontal == horizontal && (e.at - at).abs() <= EPS)
                    .map(|e| format!("boundary:{}", e.source_segment))
                    .collect::<std::collections::BTreeSet<_>>();
                let clip_orders = regions
                    .iter()
                    .filter(|r| {
                        let (lo, hi) = if horizontal {
                            (r.bounds.y, r.bounds.y + r.bounds.height)
                        } else {
                            (r.bounds.x, r.bounds.x + r.bounds.width)
                        };
                        (lo - at).abs() <= EPS || (hi - at).abs() <= EPS
                    })
                    .flat_map(|r| {
                        r.glyphs
                            .iter()
                            .map(|i| &orders[text.glyphs[*i].source_span])
                    })
                    .collect::<std::collections::BTreeSet<_>>();
                json!({"at":at,"kind":"derived","rule":"paint-centerline-or-active-content-clip",
            "sourceRefs":paint_refs,"activeClipSourceOrders":clip_orders,
            "emptyRegionClipScopes":empty_axis_scopes.iter().filter(|(b,_)| {
                let (lo,hi)=if horizontal{(b.y,b.y+b.height)}else{(b.x,b.x+b.width)};
                (lo-at).abs()<=EPS || (hi-at).abs()<=EPS
            }).map(|(_,order)|order).collect::<std::collections::BTreeSet<_>>()})
            })
            .collect::<Vec<_>>()
    };
    let axis_evidence = json!({"basis":"oriented paint axes plus complete active content clips",
        "paintExteriorClosed":paint_exterior_closed,
        "x":axis_sources(&xs,false),"y":axis_sources(&ys,true),
        "contentRegions":evidence,"paintIsNotInvented":true});
    let mut grid = Grid {
        bbox: outer,
        xs,
        ys,
        cells,
        topology: serde_json::Value::Null,
        source_edges: Vec::new(),
        search_region: None,
        shared_edges,
        uncertain_connections,
        regularized: component
            .iter()
            .any(|e| matches!(e.paint_kind, "periodic_carrier" | "dash_carrier")),
        cell_region_evidence: serde_json::Value::Null,
        axis_evidence,
    };
    grid.source_edges = source
        .iter()
        .filter(|e| boundary_contributor(e, &grid, component))
        .copied()
        .collect();
    super::select(&mut grid, text, orders, clips, media);
    // A well-formed but incomplete finer proposal must stay visible and
    // deferred. Falling back to a coarser confirmed face would lose the very
    // content separation that required this comparison.
    grid.source_edges = source
        .iter()
        .filter(|e| boundary_contributor(e, &grid, component))
        .copied()
        .collect();
    Ok(grid)
}
