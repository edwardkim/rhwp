//! Residual regions require their own positive enclosure; emptiness is not evidence.
use super::*;
type Address = (usize, usize, usize, usize);

/// An exporter may emit an empty cell's rectangular clip scope without any
/// text-show. Use such scopes only to complete unowned parts of an already
/// content-backed grid. They cannot add axes, displace content, cross a painted
/// separator or choose between competing partitions. This is inferred empty
/// structure, not a claim that a clipping operator is authored table metadata.
pub(super) fn from_clip_scopes(
    allocation: Allocation<'_>,
    grid: &Grid,
    faces: &mut BTreeMap<Address, Vec<usize>>,
    owners: &mut [Option<usize>],
    clips: &PathClipObservations,
    media: (f32, f32, f32, f32),
    work: &mut usize,
    separators: &[super::Separator],
) -> Result<BTreeMap<Address, std::collections::BTreeSet<Vec<usize>>>, String> {
    if clips.omitted > 0 {
        return Err("empty clip scope observations incomplete".into());
    }
    let cols = grid.xs.len() - 1;
    let axis = |vs: &[f32], at: f32| vs.iter().position(|v| (*v - at).abs() <= EPS);
    let mut proposed = BTreeMap::<Address, std::collections::BTreeSet<Vec<usize>>>::new();
    for scope in &clips.clips {
        *work += grid.xs.len() + grid.ys.len();
        if *work > 1_000_000 {
            return Err("cell region observation work exceeded".into());
        }
        let Some(b) = super::effective_clip(Some(&scope.clip), media) else {
            continue;
        };
        if !super::super::inside(b, grid.bbox) {
            continue;
        }
        let (Some(c), Some(ce), Some(r), Some(re)) = (
            axis(&grid.xs, b.x),
            axis(&grid.xs, b.x + b.width),
            axis(&grid.ys, b.y),
            axis(&grid.ys, b.y + b.height),
        ) else {
            continue;
        };
        if c >= ce || r >= re || (r..re).any(|y| (c..ce).any(|x| owners[y * cols + x].is_some())) {
            continue;
        }
        let crosses = separators
            .iter()
            .any(|e| e.crosses(grid, (r, c, re - r, ce - c)));
        *work += separators.len();
        if *work > 1_000_000 {
            return Err("cell region observation work exceeded".into());
        }
        if !crosses {
            let address = (r, c, re - r, ce - c);
            if !proposed.contains_key(&address) {
                allocation
                    .tree_entry::<Address, std::collections::BTreeSet<Vec<usize>>>()
                    .map_err(message)?;
            }
            insert_order(
                allocation,
                proposed.entry(address).or_default(),
                &scope.source_order,
            )?;
        }
    }
    let mut pending = allocation.vec(owners.len()).map_err(message)?;
    pending.resize(owners.len(), None);
    for (id, &(r, c, rs, cs)) in proposed.keys().enumerate() {
        for y in r..r + rs {
            for x in c..c + cs {
                if pending[y * cols + x].replace(id).is_some() {
                    return Err("competing empty clip partitions".into());
                }
            }
        }
    }
    for &(r, c, rs, cs) in proposed.keys() {
        let id = faces.len();
        if !faces.contains_key(&(r, c, rs, cs)) {
            allocation
                .tree_entry::<Address, Vec<usize>>()
                .map_err(message)?;
        }
        faces.insert((r, c, rs, cs), Vec::new());
        for y in r..r + rs {
            for x in c..c + cs {
                owners[y * cols + x] = Some(id);
            }
        }
    }
    Ok(proposed)
}

pub(super) fn complete(
    allocation: crate::parser::pdf::allocation::Allocation<'_>,
    grid: &Grid,
    faces: &mut BTreeMap<Address, Vec<usize>>,
    owners: &mut [Option<usize>],
    work: &mut usize,
) -> Result<(), String> {
    let (rows, cols) = (grid.ys.len() - 1, grid.xs.len() - 1);
    let mut sides = BTreeMap::new();
    for edge in &grid.shared_edges {
        let h = edge["horizontal"]
            .as_bool()
            .ok_or("missing edge orientation")?;
        let at = edge["at"].as_f64().ok_or("missing edge axis")? as f32;
        let from = edge["from"].as_f64().ok_or("missing edge start")? as f32;
        let (r, c) = if h {
            (
                grid.ys.iter().position(|v| (*v - at).abs() <= EPS),
                grid.xs.iter().position(|v| (*v - from).abs() <= EPS),
            )
        } else {
            (
                grid.ys.iter().position(|v| (*v - from).abs() <= EPS),
                grid.xs.iter().position(|v| (*v - at).abs() <= EPS),
            )
        };
        let key = (
            h,
            r.ok_or("unknown row axis")?,
            c.ok_or("unknown column axis")?,
        );
        if !sides.contains_key(&key) {
            allocation
                .tree_entry::<(bool, usize, usize), f32>()
                .map_err(message)?;
        }
        sides.insert(
            key,
            edge["regularizedCoveragePt"]
                .as_f64()
                .ok_or("missing paint coverage")? as f32,
        );
    }
    for r in 0..rows {
        for c in 0..cols {
            let tile = r * cols + c;
            if owners[tile].is_some() {
                continue;
            }
            *work += 1;
            if *work > 1_000_000 {
                return Err("cell region observation work exceeded".into());
            }
            let h = |row: usize, col: usize| -> Result<bool, String> {
                if row == 0 || row == rows {
                    super::super::complete(
                        allocation,
                        &grid.source_edges,
                        true,
                        grid.ys[row],
                        grid.xs[col],
                        grid.xs[col + 1],
                    )
                } else {
                    Ok(sides
                        .get(&(true, row, col))
                        .is_some_and(|n| *n >= grid.xs[col + 1] - grid.xs[col] - EPS))
                }
            };
            let v = |row: usize, col: usize| -> Result<bool, String> {
                if col == 0 || col == cols {
                    super::super::complete(
                        allocation,
                        &grid.source_edges,
                        false,
                        grid.xs[col],
                        grid.ys[row],
                        grid.ys[row + 1],
                    )
                } else {
                    Ok(sides
                        .get(&(false, row, col))
                        .is_some_and(|n| *n >= grid.ys[row + 1] - grid.ys[row] - EPS))
                }
            };
            // Do not join empty tiles through an absent side: a blank merged
            // region and blank cells with hidden sides cannot be distinguished
            // by missing ink. Each accepted blank tile needs all four sides.
            if !h(r, c)? || !h(r + 1, c)? || !v(r, c)? || !v(r, c + 1)? {
                return Err(
                    "unwitnessed cell region lacks an independent painted enclosure".into(),
                );
            }
            owners[tile] = Some(faces.len());
            if !faces.contains_key(&(r, c, 1, 1)) {
                allocation
                    .tree_entry::<Address, Vec<usize>>()
                    .map_err(message)?;
            }
            faces.insert((r, c, 1, 1), Vec::new());
        }
    }
    Ok(())
}
