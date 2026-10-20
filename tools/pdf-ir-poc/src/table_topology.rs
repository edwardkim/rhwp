//! Finite cellular chains over F2 for already reconstructed rectangular cell faces.
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn toggle<T: Ord>(set: &mut BTreeSet<T>, item: T) {
    if !set.remove(&item) {
        set.insert(item);
    }
}

/// Cell addresses are (row, column, rowspan, colspan). Axes are split at every grid slot.
pub fn evidence(
    rows: usize,
    cols: usize,
    cells: &[(usize, usize, usize, usize)],
) -> Result<Value, String> {
    if rows == 0
        || cols == 0
        || rows.checked_mul(cols).is_none_or(|n| n > 4096)
        || cells.len() > 4096
    {
        return Err("cell complex budget exceeded".into());
    }
    let mut edge_ids = BTreeMap::new();
    let mut boundaries = Vec::new();
    let mut incidences = 0usize;
    for &(r, c, rs, cs) in cells {
        if rs == 0 || cs == 0 || r >= rows || c >= cols || rs > rows - r || cs > cols - c {
            return Err("invalid cell complex face".into());
        }
        incidences += 2 * (rs + cs);
        if incidences > 65536 {
            return Err("cell complex incidence budget exceeded".into());
        }
        let mut boundary = BTreeSet::new();
        let mut add = |a, b| {
            let id = edge_ids.len();
            let key = if a < b { (a, b) } else { (b, a) };
            toggle(&mut boundary, *edge_ids.entry(key).or_insert(id));
        };
        let vertex = |y, x| y * (cols + 1) + x;
        for x in c..c + cs {
            add(vertex(r, x), vertex(r, x + 1));
            add(vertex(r + rs, x), vertex(r + rs, x + 1));
        }
        for y in r..r + rs {
            add(vertex(y, c), vertex(y + 1, c));
            add(vertex(y, c + cs), vertex(y + 1, c + cs));
        }
        boundaries.push(boundary);
    }
    let mut edges = vec![(0, 0); edge_ids.len()];
    for (edge, id) in edge_ids {
        edges[id] = edge;
    }
    let vertices: BTreeSet<_> = edges.iter().flat_map(|(a, b)| [*a, *b]).collect();
    let mut sets: Vec<_> = (0..(rows + 1) * (cols + 1)).collect();
    fn root(sets: &mut [usize], mut i: usize) -> usize {
        while sets[i] != i {
            sets[i] = sets[sets[i]];
            i = sets[i];
        }
        i
    }
    for &(a, b) in &edges {
        let a = root(&mut sets, a);
        let b = root(&mut sets, b);
        sets[b] = a;
    }
    let components = vertices
        .iter()
        .map(|i| root(&mut sets, *i))
        .collect::<BTreeSet<_>>()
        .len();
    let rank_one = vertices.len() - components;
    let mut pivots = BTreeMap::<usize, BTreeSet<usize>>::new();
    let mut work = 0usize;
    for mut boundary in boundaries {
        let mut vertex_boundary = BTreeSet::new();
        for &i in &boundary {
            toggle(&mut vertex_boundary, edges[i].0);
            toggle(&mut vertex_boundary, edges[i].1);
        }
        if !vertex_boundary.is_empty() {
            return Err("boundary of boundary is nonzero".into());
        }
        while let Some(&pivot) = boundary.last() {
            if let Some(previous) = pivots.get(&pivot) {
                work += previous.len();
                if work > 2_000_000 {
                    return Err("cell complex reduction budget exceeded".into());
                }
                for &i in previous {
                    toggle(&mut boundary, i);
                }
            } else {
                pivots.insert(pivot, boundary);
                break;
            }
        }
    }
    let rank_two = pivots.len();
    let cycles = edges.len() - rank_one;
    let betti = [components, cycles - rank_two, cells.len() - rank_two];
    if betti != [1, 0, 0] {
        return Err("cell complex is not an acyclic connected table disk".into());
    }
    Ok(
        json!({"field":"F2","vertices":vertices.len(),"edges":edges.len(),"faces":cells.len(),
        "rankBoundary1":rank_one,"rankBoundary2":rank_two,"boundarySquaredZero":true,
        "skeletonCycleRank":cycles,"betti":betti,"eulerCharacteristic":1,
        "scope":"filled cell complex; topology alone does not prove table semantics"}),
    )
}
