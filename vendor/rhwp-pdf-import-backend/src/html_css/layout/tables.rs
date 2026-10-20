//! Table layout (LAYOUT-7) — closes Phase LAYOUT.
//!
//! CSS 2.1 Chapter 17 table layout, restrained to the v0.3.35
//! supported surface:
//!
//! - Automatic and fixed table layout algorithms.
//! - `border-collapse: collapse | separate`.
//! - `<thead>` repetition handled by the paginator (PAGINATE-2);
//!   here we just record which row-group is which.
//! - Row-level page breaks only — no mid-cell splits in v0.3.35.
//!
//! Out of scope:
//! - colspan/rowspan beyond simple cases (deferred).
//! - column groups affecting widths (parsed-and-ignored).
//! - Border-collapse conflict resolution beyond "max width wins"
//!   (CSS 2.1 §17.6.2's full table is post-release).

/// Section role of a table row group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowGroupKind {
    /// `<thead>` — repeats on every page that the table spans.
    Header,
    /// `<tbody>` — body rows, splittable.
    Body,
    /// `<tfoot>` — bottom-aligned, kept on the final table page.
    Footer,
}

/// One cell's intrinsic width hints (used by automatic layout).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellHint {
    /// Minimum content width in px (longest unbreakable word).
    pub min_px: f32,
    /// Maximum (preferred) content width in px (one-line layout).
    pub max_px: f32,
    /// Number of columns this cell spans.
    pub colspan: u32,
}

impl Default for CellHint {
    fn default() -> Self {
        Self {
            min_px: 0.0,
            max_px: 0.0,
            colspan: 1,
        }
    }
}

/// Row of cell hints. `cells.len()` includes spanned slots; cells with
/// colspan>1 occupy the leading slot, the rest are placeholders with
/// `colspan=0`.
#[derive(Debug, Clone, Default)]
pub struct RowHint {
    /// One entry per column slot.
    pub cells: Vec<CellHint>,
}

/// Computed table layout output: per-column widths in px + per-row
/// heights.
#[derive(Debug, Clone, Default)]
pub struct TableLayout {
    /// Column widths (Σ = table content width).
    pub column_widths: Vec<f32>,
    /// Row heights, ordered top-to-bottom across all groups.
    pub row_heights: Vec<f32>,
}

/// Table-layout algorithm choice (CSS `table-layout` property).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutAlgorithm {
    /// `table-layout: auto` (default) — column widths from content
    /// hints.
    #[default]
    Auto,
    /// `table-layout: fixed` — first row determines widths, rest
    /// inherit. Cheaper because subsequent cells don't influence
    /// column sizing.
    Fixed,
}

/// Compute column widths for a table given per-cell hints and the
/// available container width.
pub fn compute_column_widths(
    rows: &[&RowHint],
    available_px: f32,
    algorithm: LayoutAlgorithm,
) -> Vec<f32> {
    let column_count = rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
    if column_count == 0 {
        return Vec::new();
    }
    match algorithm {
        LayoutAlgorithm::Fixed => {
            // First row's max widths dominate; distribute leftover
            // space proportionally.
            let empty = RowHint::default();
            let first = rows.first().copied().unwrap_or(&empty);
            let mut widths: Vec<f32> = (0..column_count)
                .map(|i| first.cells.get(i).map(|c| c.max_px).unwrap_or(0.0))
                .collect();
            normalize_to_available(&mut widths, available_px);
            widths
        },
        LayoutAlgorithm::Auto => {
            let mut min_w = vec![0.0_f32; column_count];
            let mut max_w = vec![0.0_f32; column_count];
            for row in rows {
                for (i, cell) in row.cells.iter().enumerate() {
                    if cell.colspan == 0 {
                        // Spanned slot — handled by the leading cell.
                        continue;
                    }
                    if cell.colspan == 1 {
                        if cell.min_px > min_w[i] {
                            min_w[i] = cell.min_px;
                        }
                        if cell.max_px > max_w[i] {
                            max_w[i] = cell.max_px;
                        }
                    } else {
                        // Distribute the spanning cell's min/max
                        // across the columns it covers, proportionally.
                        let cols = (cell.colspan as usize).min(column_count - i);
                        for k in 0..cols {
                            let share = cell.min_px / cols as f32;
                            if share > min_w[i + k] {
                                min_w[i + k] = share;
                            }
                            let share_max = cell.max_px / cols as f32;
                            if share_max > max_w[i + k] {
                                max_w[i + k] = share_max;
                            }
                        }
                    }
                }
            }
            let total_min: f32 = min_w.iter().sum();
            let total_max: f32 = max_w.iter().sum();
            if total_max <= available_px {
                // Everything fits at preferred widths.
                max_w
            } else if total_min >= available_px {
                // Even minimum widths don't fit — return min widths
                // (table will overflow).
                min_w
            } else {
                // Linear interpolation between min and max.
                let extra = available_px - total_min;
                let span = total_max - total_min;
                min_w
                    .iter()
                    .zip(max_w.iter())
                    .map(|(&mn, &mx)| {
                        if span <= 0.0 {
                            mn
                        } else {
                            mn + (mx - mn) * (extra / span)
                        }
                    })
                    .collect()
            }
        },
    }
}

fn normalize_to_available(widths: &mut [f32], available: f32) {
    let total: f32 = widths.iter().sum();
    if total <= 0.0 {
        let n = widths.len() as f32;
        if n > 0.0 {
            let each = available / n;
            for w in widths.iter_mut() {
                *w = each;
            }
        }
        return;
    }
    if (total - available).abs() < 0.001 {
        return;
    }
    let scale = available / total;
    for w in widths.iter_mut() {
        *w *= scale;
    }
}

/// Compute per-row heights from per-cell content heights. A row's
/// height is the max of its cells' heights.
pub fn compute_row_heights(rows: &[Vec<f32>]) -> Vec<f32> {
    rows.iter()
        .map(|cells| cells.iter().copied().fold(0.0_f32, f32::max))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(min: f32, max: f32) -> CellHint {
        CellHint {
            min_px: min,
            max_px: max,
            colspan: 1,
        }
    }
    fn row(cells: Vec<CellHint>) -> RowHint {
        RowHint { cells }
    }

    #[test]
    fn auto_layout_fits_at_max() {
        let rows = [row(vec![cell(20.0, 100.0), cell(30.0, 120.0)])];
        let row_refs: Vec<&RowHint> = rows.iter().collect();
        let w = compute_column_widths(&row_refs, 600.0, LayoutAlgorithm::Auto);
        // 100 + 120 = 220 <= 600, so widths are at max.
        assert_eq!(w, vec![100.0, 120.0]);
    }

    #[test]
    fn auto_layout_squeezes_when_overconstrained() {
        let rows = [row(vec![cell(50.0, 400.0), cell(50.0, 400.0)])];
        let row_refs: Vec<&RowHint> = rows.iter().collect();
        let w = compute_column_widths(&row_refs, 500.0, LayoutAlgorithm::Auto);
        // total min = 100; total max = 800; available = 500.
        // extra = 400, span = 700. Each col gets 50 + 350 * (400/700) = 50 + 200 = 250.
        assert!((w[0] - 250.0).abs() < 0.5);
        assert!((w[1] - 250.0).abs() < 0.5);
    }

    #[test]
    fn auto_layout_below_min_returns_min() {
        let rows = [row(vec![cell(200.0, 300.0), cell(200.0, 300.0)])];
        let row_refs: Vec<&RowHint> = rows.iter().collect();
        let w = compute_column_widths(&row_refs, 100.0, LayoutAlgorithm::Auto);
        // total min = 400 > 100 → return min widths (table overflows).
        assert_eq!(w, vec![200.0, 200.0]);
    }

    #[test]
    fn fixed_layout_uses_first_row_then_normalizes() {
        let rows = [
            row(vec![cell(0.0, 100.0), cell(0.0, 200.0)]),
            row(vec![cell(0.0, 999.0), cell(0.0, 999.0)]), // ignored
        ];
        let row_refs: Vec<&RowHint> = rows.iter().collect();
        let w = compute_column_widths(&row_refs, 600.0, LayoutAlgorithm::Fixed);
        // First row dictates 1:2 ratio; normalised to 600 → 200, 400.
        assert!((w[0] - 200.0).abs() < 0.5);
        assert!((w[1] - 400.0).abs() < 0.5);
    }

    #[test]
    fn rowgroup_kind_distinguishes() {
        assert_ne!(RowGroupKind::Header, RowGroupKind::Body);
        assert_ne!(RowGroupKind::Body, RowGroupKind::Footer);
    }

    #[test]
    fn row_heights_take_max() {
        let h = compute_row_heights(&[vec![20.0, 30.0, 25.0], vec![40.0, 10.0]]);
        assert_eq!(h, vec![30.0, 40.0]);
    }
}
