//! Closed row intervals, validated before staging or calling any editor mutation.
use super::{invalid, template_rows::RepeatTableRowsRequest};
use crate::{
    error::HwpError,
    model::table::{Table, MAX_TABLE_GRID_CELLS},
};

pub(super) struct Geometry {
    pub cells: Vec<usize>,
    pub zones: Vec<usize>,
    pub added: u16,
    pub rows: u16,
    pub height: u32,
    pub grid_bytes: usize,
}

/// Validated cells have nonzero spans and no overlaps, so per-row anchor counts
/// fit the format's UINT16. Avoid scanning all cells again for every new row.
pub(super) fn rebuild_row_sizes(t: &mut Table) {
    let mut counts = vec![0u16; usize::from(t.row_count)];
    for c in &t.cells {
        counts[usize::from(c.row)] += 1;
    }
    t.row_sizes = counts.into_iter().map(|n| n as i16).collect();
}

pub(super) fn inspect(t: &Table, r: &RepeatTableRowsRequest) -> Result<Geometry, HwpError> {
    r.limits.validate()?;
    if r.start_row >= r.end_row || r.end_row > t.row_count || t.col_count == 0 {
        return Err(invalid(
            "rows/source: expected a nonempty in-range row interval",
        ));
    }
    if r.insert_before > t.row_count
        || (r.start_row < r.insert_before && r.insert_before < r.end_row)
    {
        return Err(invalid("rows/destination: out of range or inside source"));
    }
    let n = r.records.len();
    if n > r.limits.max_copies {
        return Err(invalid("rows/copy budget exceeded"));
    }
    let added = usize::from(r.end_row - r.start_row)
        .checked_mul(n)
        .and_then(|v| u16::try_from(v).ok())
        .ok_or_else(|| invalid("rows/row count overflow"))?;
    let rows = t
        .row_count
        .checked_add(added)
        .ok_or_else(|| invalid("rows/row count overflow"))?;
    let slots = usize::from(rows) * usize::from(t.col_count);
    let grid_bytes = slots
        .checked_mul(std::mem::size_of::<Option<usize>>())
        .filter(|v| slots <= MAX_TABLE_GRID_CELLS && *v <= r.limits.max_structure_bytes)
        .ok_or_else(|| invalid("rows/grid budget exceeded"))?;
    let mut result = Geometry {
        cells: vec![],
        zones: vec![],
        added,
        rows,
        height: t.common.height,
        grid_bytes,
    };
    // Empty records still validate the address/limits but do not inspect source content.
    if n == 0 {
        return Ok(result);
    }
    if t.cells.len() > r.limits.max_nodes || t.zones.len() > r.limits.max_nodes {
        return Err(invalid("rows/structure node budget exceeded"));
    }
    let mut occupied = vec![false; usize::from(t.row_count) * usize::from(t.col_count)];
    for (i, c) in t.cells.iter().enumerate() {
        let end = c.row.checked_add(c.row_span).filter(|v| *v <= t.row_count);
        let right = c.col.checked_add(c.col_span).filter(|v| *v <= t.col_count);
        let (Some(end), Some(right)) = (end, right) else {
            return Err(invalid(format!("rows/Cell({i}): span outside table")));
        };
        if c.row_span == 0 || c.col_span == 0 {
            return Err(invalid(format!("rows/Cell({i}): zero span")));
        }
        for boundary in [r.start_row, r.end_row, r.insert_before] {
            if c.row < boundary && boundary < end {
                return Err(invalid(format!(
                    "rows/Cell({i}): merge crosses row boundary {boundary}"
                )));
            }
        }
        for row in c.row..end {
            for col in c.col..right {
                let slot =
                    &mut occupied[usize::from(row) * usize::from(t.col_count) + usize::from(col)];
                if *slot {
                    return Err(invalid(format!("rows/Cell({i}): overlapping cells")));
                }
                *slot = true;
            }
        }
        if c.row >= r.start_row && end <= r.end_row {
            if c.is_header {
                return Err(invalid(format!(
                    "rows/Cell({i}): title cells cannot be repeated as data"
                )));
            }
            result.cells.push(i);
        }
    }
    if occupied.iter().any(|v| !v) {
        return Err(invalid("rows/grid: uncovered coordinate"));
    }
    if usize::from(r.insert_before) < t.leading_header_rows().len() {
        return Err(invalid("rows/destination: insertion would displace the leading title block"));
    }
    result
        .cells
        .sort_by_key(|i| (t.cells[*i].row, t.cells[*i].col));
    for (i, z) in t.zones.iter().enumerate() {
        if z.start_row > z.end_row
            || z.end_row >= t.row_count
            || z.start_col > z.end_col
            || z.end_col >= t.col_count
        {
            return Err(invalid(format!("rows/Zone({i}): invalid inclusive bounds")));
        }
        let end = z.end_row + 1;
        let intersects = z.start_row < r.end_row && r.start_row < end;
        if (intersects && !(z.start_row >= r.start_row && end <= r.end_row))
            || (z.start_row < r.insert_before && r.insert_before < end)
        {
            return Err(invalid(format!(
                "rows/Zone({i}): zone crosses source or insertion boundary"
            )));
        }
        if intersects {
            result.zones.push(i);
        }
    }
    let zones = result
        .zones
        .len()
        .checked_mul(n)
        .and_then(|v| v.checked_add(t.zones.len()));
    if zones.is_none_or(|v| v > usize::from(u16::MAX)) {
        return Err(invalid("rows/zone count overflow"));
    }
    // Preserve the existing displayed height of stretched rows, as insert_row does.
    let heights = t
        .stretched_row_heights()
        .unwrap_or_else(|| t.get_row_heights());
    let source_height = heights[usize::from(r.start_row)..usize::from(r.end_row)]
        .iter()
        .try_fold(0u32, |sum, h| sum.checked_add(*h))
        .ok_or_else(|| invalid("rows/height overflow"))?;
    let base_height = heights.iter().try_fold(0u32, |sum,h| sum.checked_add(*h))
        .ok_or_else(|| invalid("rows/height overflow"))?;
    result.height = source_height
        .checked_mul(n as u32)
        .and_then(|h| base_height.checked_add(h))
        .ok_or_else(|| invalid("rows/height overflow"))?;
    Ok(result)
}
