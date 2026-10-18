//! Residual regions require their own positive enclosure; emptiness is not evidence.
use super::*;
type Address = (usize, usize, usize, usize);

pub(super) fn complete(
    grid: &Grid,
    faces: &mut BTreeMap<Address, Vec<usize>>,
    owners: &mut [Option<usize>],
    work: &mut usize,
) -> Result<(), &'static str> {
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
        sides.insert(
            (
                h,
                r.ok_or("unknown row axis")?,
                c.ok_or("unknown column axis")?,
            ),
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
                return Err("cell region observation work exceeded");
            }
            let h = |row: usize, col: usize| {
                row == 0
                    || row == rows
                    || sides
                        .get(&(true, row, col))
                        .is_some_and(|n| *n >= grid.xs[col + 1] - grid.xs[col] - EPS)
            };
            let v = |row: usize, col: usize| {
                col == 0
                    || col == cols
                    || sides
                        .get(&(false, row, col))
                        .is_some_and(|n| *n >= grid.ys[row + 1] - grid.ys[row] - EPS)
            };
            // Do not join empty tiles through an absent side: a blank merged
            // region and blank cells with hidden sides cannot be distinguished
            // by missing ink. Each accepted blank tile needs all four sides.
            if !h(r, c) || !h(r + 1, c) || !v(r, c) || !v(r, c + 1) {
                return Err("unwitnessed cell region lacks an independent painted enclosure");
            }
            owners[tile] = Some(faces.len());
            faces.insert((r, c, 1, 1), Vec::new());
        }
    }
    Ok(())
}
