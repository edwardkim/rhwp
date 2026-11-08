//! The existing rowspan-label rule with request-owned index storage.
use crate::{error::Result, import_storage::ImportStorage, layout::TextSpan};
pub(crate) fn reorder(spans: &mut [TextSpan], allocation: ImportStorage<'_>) -> Result<()> {
    allocation.bytes(0)?;

    if spans.len() < 10 {
        return Ok(());
    }

    // Cluster by X proximity (15pt gap threshold). Walk spans ordered
    // by left edge; start a new cluster whenever the gap exceeds the
    // threshold.
    let mut by_x = allocation.collect(0..spans.len())?;
    allocation.sort_by(&mut by_x, |&a, &b| {
        crate::utils::safe_float_cmp(spans[a].bbox.x, spans[b].bbox.x)
    })?;
    const X_GAP: f32 = 15.0;
    let mut columns: Vec<Vec<usize>> = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    let mut last_x = f32::NEG_INFINITY;
    for &idx in &by_x {
        let x = spans[idx].bbox.x;
        if !cur.is_empty() && x - last_x > X_GAP {
            allocation.push(&mut columns, std::mem::take(&mut cur))?;
        }
        allocation.push(&mut cur, idx)?;
        last_x = x;
    }
    if !cur.is_empty() {
        allocation.push(&mut columns, cur)?;
    }
    if columns.len() < 2 {
        return Ok(());
    }

    // Max column size is our reference for "dense".
    let max_count = columns.iter().map(|c| c.len()).max().unwrap_or(0);
    if max_count < 6 {
        return Ok(());
    }

    // Sort columns by span count descending so we can pick the top
    // dense cluster for anchor detection.
    let mut col_order = allocation.collect(0..columns.len())?;
    allocation.sort_by(&mut col_order, |&a, &b| columns[b].len().cmp(&columns[a].len()))?;

    // A column is "dense" when it holds a strict majority of the
    // most populous column's spans. Pages with multiple dense data
    // columns (three or more) let us derive the data-row range by
    // intersecting their Y bands — headers and sub-headers populate
    // only a subset of columns at their Y and fall out.
    let dense_cols_count = columns.iter().filter(|c| c.len() * 2 > max_count).count();

    // Most populous column, used for anchor Y lookups regardless.
    let dense_col = &columns[col_order[0]];
    let mut dense_ys = allocation.collect(dense_col.iter().map(|&i| spans[i].bbox.y))?;
    allocation.sort_by(&mut dense_ys, |a, b| crate::utils::safe_float_cmp(*b, *a))?;

    // Compute the set of Y bands that count as "data". When several
    // dense columns are available, require a band to have support in
    // the top three; otherwise fall back to the single dense column's
    // own Y values.
    let band_of = |y: f32| (y / crate::utils::ROW_BAND_TOLERANCE_PT).round() as i32;
    let data_bands = data_bands(spans, &columns, &col_order, dense_cols_count, allocation)?;

    if data_bands.len() < 4 {
        return Ok(());
    }
    let band_pt = crate::utils::ROW_BAND_TOLERANCE_PT;
    let data_top = (*data_bands.last().unwrap() as f32) * band_pt + band_pt / 2.0;
    let data_bot = (*data_bands.first().unwrap() as f32) * band_pt - band_pt / 2.0;

    // Y-bands occupied by the dense column. Genuine rowspan labels are
    // vertically centred *between* data rows, so their Y-band must NOT
    // appear in this set. Spans whose Y aligns with the dense column are
    // line-continuation text on the same logical line, not labels.
    let mut dense_bands =
        allocation.collect(dense_col.iter().map(|&i| band_of(spans[i].bbox.y)))?;
    dense_bands.sort_unstable();
    dense_bands.dedup();

    // Numbered reference/bibliography lists render the leading marker
    // ("1.", "2.", "3.") in a narrow column to the left of the body
    // text. That marker column is sparse (one per entry) and its markers
    // sit between body rows, so they look exactly like rowspan labels to
    // the heuristic below — but they are NOT: each number belongs to its
    // own entry and promoting them scrambles the reference order. Detect
    // the pattern (>=3 numbered markers sharing a tight left-edge cluster
    // and spread down >=3 distinct rows = a vertical numbered list) and
    // exclude those markers from label promotion.
    let is_numbered_marker = |i: usize| -> bool {
        let t = spans[i].text.trim_start();
        let digits = t.chars().take_while(|c| c.is_ascii_digit()).count();
        (1..=3).contains(&digits) && t[digits..].starts_with(['.', ')'])
    };
    let numbered_excluded = {
        let markers = allocation.collect((0..spans.len()).filter(|&i| is_numbered_marker(i)))?;
        if markers.len() >= 3 {
            let mut xs = allocation.collect(markers.iter().map(|&i| spans[i].bbox.x))?;
            allocation.sort_by(&mut xs, |a, b| crate::utils::safe_float_cmp(*a, *b))?;
            let median_x = xs[xs.len() / 2];
            // Marker indices are collected in source-array order, so this filtered
            // membership list is already sorted for the lookups below.
            let cluster = allocation.collect(
                markers
                    .iter()
                    .copied()
                    .filter(|&i| (spans[i].bbox.x - median_x).abs() <= 6.),
            )?;
            let mut rows = allocation.collect(cluster.iter().map(|&i| band_of(spans[i].bbox.y)))?;
            rows.sort_unstable();
            rows.dedup();
            if cluster.len() >= 3 && rows.len() >= 3 {
                cluster
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    };

    // Collect "label" candidates: spans that sit in a "sparse"
    // column — one that holds meaningfully fewer spans than the
    // most populous column. A candidate only qualifies when it
    // sits strictly inside the data Y range AND the sparse column
    // it belongs to has at least two entries inside that range —
    // single-span sparse cells are almost always stray annotations,
    // not labels.
    let mut labels: Vec<usize> = Vec::new();
    for col in &columns {
        if col.len() < 2 || col.len() * 2 >= max_count {
            continue;
        }
        let in_data = allocation.collect(col.iter().copied().filter(|&i| {
            let y = spans[i].bbox.y;
            // Exclude spans on the same Y-band as the dense column:
            // those are line-continuation text, not rowspan labels.
            // Also exclude numbered-list markers (reference numbers),
            // which would otherwise be hoisted out of reading order.
            y > data_bot
                && y < data_top
                && dense_bands.binary_search(&band_of(y)).is_err()
                && numbered_excluded.binary_search(&i).is_err()
        }))?;
        if in_data.len() >= 2 {
            allocation.reserve(&mut labels, in_data.len())?;
            labels.extend(in_data);
        }
    }
    if labels.is_empty() {
        return Ok(());
    }

    // A stub label is a word. The clustering reasons in table terms — a
    // sparse column against a dense one — and on prose it finds that shape
    // anyway: a mathematics page whose lines are cut into many runs by
    // inline symbols offered three single glyphs of a display equation as
    // its sparse column, and promoting them to the head of their block
    // reordered the paragraph around them, so a trailing line came out
    // ahead of the line it continues (`criterionThe ridge regression ...`).
    //
    // ISO 32000-1:2008 §14.8.4.3.4 makes a table's row (TR) the element
    // that holds its cells (TD), and a stub cell carries the row's name —
    // text, not a lone glyph. Requiring most candidates to be more than one
    // character keeps the promotion on the shape it was written for.
    let multi_char = labels
        .iter()
        .filter(|&&i| spans[i].text.trim().chars().count() > 1)
        .count();
    if multi_char * 2 <= labels.len() {
        return Ok(());
    }
    allocation.sort_by(&mut labels, |&a, &b| {
        crate::utils::safe_float_cmp(spans[b].bbox.y, spans[a].bbox.y)
    })?;

    // Labels that sit at near-identical Y values almost always
    // annotate the same logical row block (e.g. a test-name in the
    // "name" column alongside a unit "×10⁹/L" in the "unit" column,
    // both vertically centred in the same 6-row group). Cluster
    // labels by Y proximity so each logical block is promoted as a
    // unit.
    const CLUSTER_GAP: f32 = 10.0;
    let mut clusters: Vec<Vec<usize>> = Vec::new();
    let mut cur: Vec<usize> = Vec::new();
    let mut last_y = f32::NAN;
    for &idx in &labels {
        let y = spans[idx].bbox.y;
        if !cur.is_empty() && (last_y - y).abs() > CLUSTER_GAP {
            allocation.push(&mut clusters, std::mem::take(&mut cur))?;
        }
        allocation.push(&mut cur, idx)?;
        last_y = y;
    }
    if !cur.is_empty() {
        allocation.push(&mut clusters, cur)?;
    }
    let cluster_ys = allocation.collect(
        clusters
            .iter()
            .map(|c| c.iter().map(|&i| spans[i].bbox.y).sum::<f32>() / c.len() as f32),
    )?;

    // For each cluster, compute the midpoint partition boundaries
    // against its immediate neighbour clusters and find the topmost
    // dense-column Y that falls inside the partition. Promote every
    // member of the cluster to the same anchor so they sort together
    // at the top of their row block.
    let mut promoted = allocation.vec(spans.len())?;
    promoted.resize(spans.len(), None);
    for (k, cluster) in clusters.iter().enumerate() {
        let c_y = cluster_ys[k];
        let upper = if k > 0 {
            (cluster_ys[k - 1] + c_y) / 2.0
        } else {
            f32::INFINITY
        };
        let lower = if k + 1 < clusters.len() {
            (c_y + cluster_ys[k + 1]) / 2.0
        } else {
            f32::NEG_INFINITY
        };
        let upper_clamped = upper.min(data_top);
        let lower_clamped = lower.max(data_bot - 1.0);
        let mut anchor = f32::NEG_INFINITY;
        for &y in &dense_ys {
            if y <= upper_clamped && y > lower_clamped && y > anchor {
                anchor = y;
            }
        }
        if anchor.is_finite() {
            for &i in cluster {
                promoted[i] = Some(anchor + 1.);
            }
        }
    }
    if promoted.iter().all(Option::is_none) {
        return Ok(());
    }

    // Re-sort spans using the promoted Ys for labels and, for everything
    // else, the baseline of the row the span is printed on. Keep the
    // row-aware comparator so the ordering stays consistent with the rest
    // of the pipeline.
    //
    // The unpromoted spans take their key from `snap_baselines_to_rows`
    // rather than `bbox.y`: this re-sort covers the whole page, so reading
    // the raw baseline here would undo the row grouping the caller just
    // established and split a mixed-size row apart again.
    let all = allocation.collect(0..spans.len())?;
    let row_baseline = crate::utils::try_snap_baselines_to_rows(spans, &all, allocation)?;
    let mut order: Vec<usize> = all;
    allocation.sort_by(&mut order, |&a, &b| {
        let ya = promoted[a].unwrap_or(row_baseline[a]);
        let yb = promoted[b].unwrap_or(row_baseline[b]);
        // Band and x come from the promoted key — that is what lifts a
        // label to the head of its row block. The final baseline tiebreak
        // must come from the baseline the page actually draws: a promoted
        // key is `anchor + 1.0`, an offset chosen to land inside the
        // anchor's band, and it is bookkeeping rather than a position. Let
        // it break the tie and it outranks a real baseline — a wrapped
        // table cell's continuation line, promoted to `anchor + 1.0` and
        // sharing its column's x exactly, sorted ahead of the line it
        // continues and split that row apart.
        crate::utils::row_band_then_x(ya, spans[a].bbox.x, yb, spans[b].bbox.x)
            .then_with(|| crate::utils::safe_float_cmp(spans[b].bbox.y, spans[a].bbox.y))
    })?;
    allocation.reorder(spans, &order)
}

fn data_bands(
    spans: &[TextSpan],
    columns: &[Vec<usize>],
    column_order: &[usize],
    dense_columns: usize,
    allocation: ImportStorage<'_>,
) -> Result<Vec<i32>> {
    let selected = dense_columns.clamp(1, 3);
    let count = column_order
        .iter()
        .take(selected)
        .try_fold(0usize, |count, &col| count.checked_add(columns[col].len()))
        .ok_or_else(|| allocation.reject("rowspan band size"))?;
    let mut pairs = allocation.vec(count)?;
    for (rank, &col) in column_order.iter().take(selected).enumerate() {
        for &index in &columns[col] {
            let band = (spans[index].bbox.y / crate::utils::ROW_BAND_TOLERANCE_PT).round() as i32;
            pairs.push((band, rank));
        }
    }
    // A column supports a band once even when it contains several source runs.
    pairs.sort_unstable();
    pairs.dedup();
    let mut bands = Vec::new();
    let mut start = 0;
    while start < pairs.len() {
        let band = pairs[start].0;
        let mut end = start + 1;
        while end < pairs.len() && pairs[end].0 == band {
            end += 1;
        }
        if end - start >= selected {
            allocation.push(&mut bands, band)?;
        }
        start = end;
    }
    Ok(bands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geometry::Rect,
        import_limits::{ImportBudget, PdfImportLimits},
    };
    #[test]
    fn a_column_supports_a_band_once_and_distinct_columns_supply_the_threshold() {
        let spans: Vec<_> = [60., 60., 90., 60., 60.]
            .into_iter()
            .map(|y| TextSpan {
                bbox: Rect::new(0., y, 1., 1.),
                ..Default::default()
            })
            .collect();
        let columns = vec![vec![0, 1, 2], vec![3], vec![4]];
        let allocation = ImportStorage(None);
        assert_eq!(data_bands(&spans, &columns, &[0, 1, 2], 3, allocation).unwrap(), vec![20]);
        assert_eq!(data_bands(&spans, &columns, &[0, 1, 2], 2, allocation).unwrap(), vec![20]);
        assert_eq!(data_bands(&spans, &columns, &[0, 1, 2], 1, allocation).unwrap(), vec![20, 30]);
    }
    #[test]
    fn a_failed_rowspan_request_preserves_owned_source_and_does_not_resume() {
        let mut spans: Vec<_> = (0..10)
            .map(|i| TextSpan {
                text: format!("source-{i}"),
                bbox: Rect::new(0., i as f32 * 20., 20., 10.),
                font_size: 10.,
                ..Default::default()
            })
            .collect();
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(reorder(&mut spans, ImportStorage(Some(&tiny))).is_err());
        assert_eq!(tiny.snapshot().decoded_bytes, 0);
        assert_eq!(
            spans
                .iter()
                .map(|span| span.text.clone())
                .collect::<Vec<_>>(),
            (0..10).map(|i| format!("source-{i}")).collect::<Vec<_>>()
        );
        assert!(reorder(&mut spans, ImportStorage(Some(&tiny))).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        reorder(&mut spans, ImportStorage(Some(&fresh))).unwrap();
        assert!(fresh.check().is_ok());
    }
}
