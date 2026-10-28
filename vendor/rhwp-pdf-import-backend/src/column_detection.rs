//! Existing multi-column classification with request-owned temporary storage.
use crate::{error::Result, import_storage::ImportStorage};

type Line = (i32, (Vec<(f32, f32)>, usize));
fn group_lines(
    mut entries: Vec<(i32, (f32, f32), usize)>,
    allocation: ImportStorage<'_>,
) -> Result<Vec<Line>> {
    allocation.sort_by(&mut entries, |a, b| a.0.cmp(&b.0))?;
    let mut lines: Vec<Line> = Vec::new();
    for (key, bounds, chars) in entries {
        if lines.last().is_none_or(|line| line.0 != key) {
            allocation.push(&mut lines, (key, (Vec::new(), 0)))?;
        }
        let line = &mut lines.last_mut().unwrap().1;
        allocation.push(&mut line.0, bounds)?;
        line.1 += chars;
    }
    Ok(lines)
}
/// Heuristic: does this page have two or more vertical text columns?
///
/// Used by `extract_spans` to decide whether to pay the XY-cut cost
/// (correct but slower on large pages) or stick with the cheap row-
/// aware sort. The check bins span X-centers into a small histogram
/// and looks for two dense bands separated by a gutter whose spans
/// vertically overlap with each other — that's the defining shape
/// of a multi-column layout (newspaper / academic / dashboard) as
/// opposed to sparse side-notes that flank a single column.
///
/// False negatives (missed multi-column page) just mean we use the
/// old reading order. False positives (single column routed through
/// XY-cut) cost a bit of CPU but produce the same or better result.
/// Both sides degrade gracefully.
/// True when the page splits into side-by-side columns separated by a clean
/// vertical gutter that no text span crosses.
///
/// This is the small-page companion to the histogram detector in
/// [`Self::is_multi_column_page`]: a two-column page with only a handful of
/// wrapped lines per column (a short article, a synthetic fixture) carries
/// too few spans for a projection histogram to classify, yet the gutter is
/// perfectly unambiguous. We recover it directly:
///
/// 1. Drop spans whose width exceeds 60 % of the content width — full-bleed
///    headings/footers legitimately straddle the gutter and must not veto it
///    (the recursive XY-Cut handles them with a horizontal cut first).
/// 2. Sweep the remaining boxes left-to-right merging their X extents; a
///    forward jump of ≥ `MIN_GUTTER_PT` between the running right edge and
///    the next box's left edge is an empty channel that no span crosses.
/// 3. Accept only when ≥ 2 spans sit on each side (genuine columns, not a
///    stray indent or page number) and the two sides' vertical ranges
///    overlap (columns sit beside each other, ruling out stacked blocks).
pub(crate) fn has_clean_column_gutter(
    spans: &[crate::layout::TextSpan],
    allocation: ImportStorage<'_>,
) -> Result<bool> {
    /// Minimum empty-channel width. Real column gutters run ≥ 18pt; ordinary
    /// inter-word/inter-cell gaps are both narrower and crossed by spans on
    /// other lines, so they never survive the sweep.
    const MIN_GUTTER_PT: f32 = 18.0;

    // (x0, x1, y0, y1) for every non-empty, finite span.
    let mut boxes = allocation.collect(
        spans
            .iter()
            .filter(|s| {
                !s.text.trim().is_empty()
                    && s.bbox.x.is_finite()
                    && s.bbox.y.is_finite()
                    && s.bbox.width.is_finite()
                    && s.bbox.height.is_finite()
                    && s.bbox.width > 0.0
            })
            .map(|s| (s.bbox.x, s.bbox.x + s.bbox.width, s.bbox.y, s.bbox.y + s.bbox.height)),
    )?;
    if boxes.len() < 4 {
        return Ok(false);
    }

    let content_min_x = boxes.iter().map(|b| b.0).fold(f32::INFINITY, f32::min);
    let content_max_x = boxes.iter().map(|b| b.1).fold(f32::NEG_INFINITY, f32::max);
    let content_w = content_max_x - content_min_x;
    if content_w < 100.0 {
        return Ok(false); // a single narrow column cannot hold a gutter
    }

    // Exclude full-width headings/footers from the gutter sweep.
    boxes.retain(|b| (b.1 - b.0) <= 0.6 * content_w);
    if boxes.len() < 4 {
        return Ok(false);
    }
    allocation.sort_by(&mut boxes, |a, b| crate::utils::safe_float_cmp(a.0, b.0))?;

    // Grid-row guard. A two-column body has exactly one text run per column
    // on a line, so a row carries at most one wide internal gap (the
    // gutter). A table / form row instead has several cells, i.e. two or
    // more wide internal gaps. Group the (heading-excluded) boxes into rows
    // by baseline and, when the majority of multi-box rows carry ≥ 2
    // significant internal gaps, treat the page as a grid and bail — a
    // single wide middle gap on a 2×N cell grid would otherwise read as a
    // lone gutter. Mirrors the grid-row discriminator on the histogram path.
    const MIN_GAP_PT: f32 = 6.0;
    let mut entries = allocation.vec(boxes.len())?;
    for &(x0, x1, _y0, y1) in &boxes {
        entries.push((y1.round() as i32, (x0, x1), 0));
    }
    let rows = group_lines(entries, allocation)?;
    let (mut multi_gap_rows, mut counted_rows) = (0usize, 0usize);
    for (_, (cells, _)) in &rows {
        if cells.len() < 2 {
            continue;
        }
        let mut s = allocation.collect(cells.iter().copied())?;
        allocation.sort_by(&mut s, |a, b| crate::utils::safe_float_cmp(a.0, b.0))?;
        let gaps = s
            .windows(2)
            .filter(|w| w[1].0 - w[0].1 >= MIN_GAP_PT)
            .count();
        counted_rows += 1;
        if gaps >= 2 {
            multi_gap_rows += 1;
        }
    }
    if counted_rows > 0 && multi_gap_rows * 2 >= counted_rows {
        return Ok(false); // grid / form, not a two-column body
    }

    // Sweep-merge X extents and collect EVERY ≥ MIN_GUTTER_PT forward jump
    // (a vertical channel no span crosses). A genuine two-column body has
    // exactly ONE such corridor — the gutter; the lines inside each column
    // overlap horizontally, so a column's own extents merge into a single
    // contiguous run. A short-cell table (numeric grid, form) instead leaves
    // a corridor between every cell column, so two or more qualifying gaps
    // means a grid / multi-region layout, not two columns — reject it
    // (matching the grid-row discriminator on the histogram path).
    let mut cover_right = boxes[0].1;
    let mut gutter_splits = allocation.vec(boxes.len())?;
    for i in 1..boxes.len() {
        if boxes[i].0 - cover_right >= MIN_GUTTER_PT {
            gutter_splits.push(i);
        }
        cover_right = cover_right.max(boxes[i].1);
    }
    if gutter_splits.len() != 1 {
        return Ok(false); // 0 = single column; ≥ 2 = grid / multi-region
    }

    let (left, right) = boxes.split_at(gutter_splits[0]);
    if left.len() < 2 || right.len() < 2 {
        return Ok(false);
    }
    // Vertical ranges of the two sides must overlap — otherwise the
    // "columns" are vertically stacked blocks (e.g. a body block above a
    // sidebar), which read fine row-aware.
    let l_y0 = left.iter().map(|b| b.2).fold(f32::INFINITY, f32::min);
    let l_y1 = left.iter().map(|b| b.3).fold(f32::NEG_INFINITY, f32::max);
    let r_y0 = right.iter().map(|b| b.2).fold(f32::INFINITY, f32::min);
    let r_y1 = right.iter().map(|b| b.3).fold(f32::NEG_INFINITY, f32::max);
    let overlap = l_y1.min(r_y1) - l_y0.max(r_y0);
    let min_height = (l_y1 - l_y0).min(r_y1 - r_y0);
    Ok(min_height > 0.0 && overlap > 0.5 * min_height)
}

pub(crate) fn is_multi_column_page(
    spans: &[crate::layout::TextSpan],
    allocation: ImportStorage<'_>,
) -> Result<bool> {
    // Clean-gutter detector (handles short pages the histogram gates below
    // reject for lack of spans). A genuine empty vertical channel that no
    // span crosses, with multi-line content of overlapping vertical extent
    // on both sides, is the unambiguous geometric signature of side-by-side
    // columns — recoverable for untagged pages only from layout (XY-Cut,
    // ISO 32000-1 §9.4, since there is no logical-structure hint).
    if has_clean_column_gutter(spans, allocation)? {
        return Ok(true);
    }

    if spans.len() < 12 {
        return Ok(false); // too few to confidently split into columns
    }

    // Primary detector: line-start-X bimodality.
    //
    // The span-center histogram further down is noisy for word-level
    // spans (every X position has many word starts on multi-word
    // body-text lines). The reliable signal is the X position at
    // which each *line* begins — a two-column body has a strong
    // peak at the left-column-start X plus a strong peak at the
    // right-column-start X, with a clear empty gutter between
    // them. We cluster spans into lines by Y (1pt tolerance), pick
    // the leftmost X per line, and look for ≥ 2 peaks separated by
    // a gutter of ≥ 30pt with zero line-starts in it.
    if has_bimodal_line_starts(spans, allocation)? {
        return Ok(true);
    }

    let mut x_centers = allocation.collect(spans.iter().map(|s| s.bbox.x + s.bbox.width * 0.5))?;
    allocation.sort_by(&mut x_centers, |a, b| crate::utils::safe_float_cmp(*a, *b))?;

    // Degenerate CTM guard: drop centers more than MAX_EXTENT from the
    // median so a rogue span ~1e16 doesn't explode the histogram.
    const MAX_EXTENT_FROM_MEDIAN: f32 = 5_000.0;
    let median = x_centers[x_centers.len() / 2];
    x_centers.retain(|c| (*c - median).abs() <= MAX_EXTENT_FROM_MEDIAN);
    if x_centers.len() < 12 {
        return Ok(false);
    }

    let min = *x_centers.first().unwrap();
    let max = *x_centers.last().unwrap();
    let width = max - min;
    if width < 100.0 {
        return Ok(false); // spans cluster in a single vertical line — not columns
    }

    // Bin into 40 buckets; find peaks (≥ mean × 1.5) separated by at
    // least one empty bucket.
    const BUCKETS: usize = 40;
    let bucket_width = width / BUCKETS as f32;
    if bucket_width <= 0.0 {
        return Ok(false);
    }
    let mut hist = [0usize; BUCKETS];
    for c in &x_centers {
        let idx = (((c - min) / bucket_width) as usize).min(BUCKETS - 1);
        hist[idx] += 1;
    }

    let total: usize = hist.iter().sum();
    let mean = total as f32 / BUCKETS as f32;
    let threshold = (mean * 1.5).max(3.0);

    let mut peaks = 0usize;
    let mut in_peak = false;
    for &count in &hist {
        if count as f32 >= threshold {
            if !in_peak {
                peaks += 1;
                in_peak = true;
            }
        } else if count == 0 {
            in_peak = false;
        }
    }

    if peaks < 2 {
        return Ok(false);
    }

    // Confirmation: the peaks must have vertical overlap. If one "column"
    // is a footer and the other is the body, they don't interact — row-
    // aware is fine. Split spans into left-half vs right-half and check
    // their Y ranges overlap.
    let mid_x = (min + max) / 2.0;
    let mut left_y_min = f32::INFINITY;
    let mut left_y_max = f32::NEG_INFINITY;
    let mut right_y_min = f32::INFINITY;
    let mut right_y_max = f32::NEG_INFINITY;
    for s in spans {
        let cx = s.bbox.x + s.bbox.width * 0.5;
        if (cx - median).abs() > MAX_EXTENT_FROM_MEDIAN {
            continue;
        }
        let y_top = s.bbox.y + s.bbox.height;
        if cx < mid_x {
            left_y_min = left_y_min.min(s.bbox.y);
            left_y_max = left_y_max.max(y_top);
        } else {
            right_y_min = right_y_min.min(s.bbox.y);
            right_y_max = right_y_max.max(y_top);
        }
    }
    let left_span = (left_y_max - left_y_min).max(0.0);
    let right_span = (right_y_max - right_y_min).max(0.0);
    let overlap = left_y_max.min(right_y_max) - left_y_min.max(right_y_min);
    let min_span = left_span.min(right_span);
    if !(min_span > 0.0 && overlap > 0.5 * min_span) {
        return Ok(false);
    }

    // Require each half to contain enough spans to represent genuine body
    // text columns. Copyright pages, title pages, and other sparse layouts
    // can produce two X-center peaks with only 2–7 spans per "column" —
    // these are not true multi-column body text.
    let left_count = spans
        .iter()
        .filter(|s| {
            let cx = s.bbox.x + s.bbox.width * 0.5;
            (cx - median).abs() <= MAX_EXTENT_FROM_MEDIAN && cx < mid_x
        })
        .count();
    let right_count = spans.len() - left_count;
    if left_count.min(right_count) < 15 {
        return Ok(false);
    }

    // Font-aware column-shape gate.
    //
    // Real two-column body text has tight column-edge alignment:
    // most spans on each side share one dominant X position
    // (the column start), with a handful of indented or
    // section-header outliers. Scattered-fragment layouts spread
    // their spans evenly across many X positions on each side.
    //
    // Measure the fraction of side-spans that fall into the
    // largest X-cluster (cluster gap = `dominant_em`). Body text
    // typically scores ≥ 0.5; scattered layouts score < 0.4.
    // Reject pages where either side fails the threshold so
    // XY-cut doesn't mis-route scattered content as multi-column.
    let stats = crate::layout::PageFontStats::try_from_spans(spans, allocation)?;
    let cluster_gap = stats.dominant_em.max(4.0);
    let dominant_cluster_fraction = |take: &dyn Fn(f32) -> bool| -> Result<f32> {
        let mut xs = allocation.collect(
            spans
                .iter()
                .filter(|s| {
                    let cx = s.bbox.x + s.bbox.width * 0.5;
                    (cx - median).abs() <= MAX_EXTENT_FROM_MEDIAN && take(cx)
                })
                .map(|s| s.bbox.x),
        )?;
        let total = xs.len();
        if total == 0 {
            return Ok(0.0);
        }
        allocation.sort_by(&mut xs, |a, b| crate::utils::safe_float_cmp(*a, *b))?;
        let mut best = 1usize;
        let mut current = 1usize;
        let mut last = xs[0];
        for &x in &xs[1..] {
            if x - last <= cluster_gap {
                current += 1;
                if current > best {
                    best = current;
                }
            } else {
                current = 1;
            }
            last = x;
        }
        Ok(best as f32 / total as f32)
    };
    const MIN_DOMINANT_FRACTION: f32 = 0.5;
    let left_frac = dominant_cluster_fraction(&|cx| cx < mid_x)?;
    let right_frac = dominant_cluster_fraction(&|cx| cx >= mid_x)?;
    if left_frac >= MIN_DOMINANT_FRACTION && right_frac >= MIN_DOMINANT_FRACTION {
        return Ok(true);
    }

    // Additive accept path (no change to the gate above): shared-baseline
    // two-column bodies — academic references / bibliographies — read
    // left+right on the SAME Y line, so the row-aware sort interleaves
    // them. Their word-granular left edges scatter, so the dominant-
    // cluster gate above misses them. But they exhibit ONE persistent
    // vertical gutter corridor (the signal poppler/MuPDF use, independent
    // of line length). Detect it via within-line gap projection, prose-
    // guarded so numeric / short-cell tables — which also reach here —
    // stay on the row-aware path. See #607.
    has_persistent_gutter_corridor(spans, median, MAX_EXTENT_FROM_MEDIAN, allocation)
}

pub(crate) fn has_persistent_gutter_corridor(
    spans: &[crate::layout::TextSpan],
    median: f32,
    max_extent: f32,
    allocation: ImportStorage<'_>,
) -> Result<bool> {
    // Group spans into lines by rounded Y baseline; carry left/right
    // extents for gap projection and char count for the prose guard.
    let mut entries = allocation.vec(spans.len())?;
    let mut x_min = f32::MAX;
    let mut x_max = f32::MIN;
    for s in spans {
        let cx = s.bbox.x + s.bbox.width * 0.5;
        if (cx - median).abs() > max_extent {
            continue; // degenerate-CTM guard, same as the caller
        }
        let y_key = (s.bbox.y + s.bbox.height).round() as i32;
        entries.push((
            y_key,
            (s.bbox.x, s.bbox.x + s.bbox.width),
            s.text.chars().filter(|c| !c.is_whitespace()).count(),
        ));
        x_min = x_min.min(s.bbox.x);
        x_max = x_max.max(s.bbox.x + s.bbox.width);
    }
    let lines = group_lines(entries, allocation)?;
    let region_width = x_max - x_min;
    if lines.len() < 12 || region_width < 200.0 {
        return Ok(false);
    }

    let total_chars: usize = lines.iter().map(|(_, (_, c))| *c).sum();
    let mean_chars = total_chars as f32 / lines.len() as f32;

    // Largest within-line gap per line (≥ 6 pt suppresses word spacing);
    // record the gap midpoint X. Also flag full-width lines with no internal
    // gutter (display equations, full-width headings) so they neither support
    // nor veto the corridor — they are excluded from the coverage denominator
    // (Part 1b: display-math robustness, #536/arxiv_math).
    const MIN_GAP_PT: f32 = 6.0;
    let mut gap_positions = allocation.vec(lines.len())?;
    let mut full_width_lines = 0usize;
    let mut multi_gap_lines = 0usize;
    for (_, (line_spans, _)) in &lines {
        if line_spans.is_empty() {
            continue;
        }
        let mut sorted = allocation.collect(line_spans.iter().copied())?;
        allocation.sort_by(&mut sorted, |a, b| crate::utils::safe_float_cmp(a.0, b.0))?;
        let line_left = sorted.first().map(|s| s.0).unwrap_or(0.0);
        let line_right = sorted.last().map(|s| s.1).unwrap_or(0.0);
        let mut largest_gap = 0.0_f32;
        let mut largest_mid = 0.0_f32;
        let mut significant_gaps = 0usize;
        for w in sorted.windows(2) {
            let gap = w[1].0 - w[0].1;
            if gap >= MIN_GAP_PT {
                significant_gaps += 1;
            }
            if gap > largest_gap {
                largest_gap = gap;
                largest_mid = (w[0].1 + w[1].0) * 0.5;
            }
        }
        if (line_right - line_left) >= region_width * 0.9 && largest_gap < MIN_GAP_PT {
            full_width_lines += 1;
        }
        // A line with two or more wide internal gaps is a grid row (≥ 3
        // cells), not a two-column body line (one gutter). Used by the
        // short-line table discriminator below.
        if significant_gaps >= 2 {
            multi_gap_lines += 1;
        }
        if largest_gap >= MIN_GAP_PT {
            gap_positions.push(largest_mid);
        }
    }
    if gap_positions.len() < 12 {
        return Ok(false);
    }
    // Coverage denominator excludes full-width display rows.
    let eff_lines = lines.len().saturating_sub(full_width_lines).max(1);

    // Cluster gap midpoints (10 pt radius); find the dominant corridor.
    const CLUSTER_RADIUS_PT: f32 = 10.0;
    allocation.sort_by(&mut gap_positions, |a, b| crate::utils::safe_float_cmp(*a, *b))?;
    let mut best_size = 0usize;
    let mut best_center = 0.0_f32;
    let mut left = 0usize;
    let mut right = 0usize;
    let mut prefix = allocation.vec(
        gap_positions
            .len()
            .checked_add(1)
            .ok_or_else(|| allocation.reject("column prefix size"))?,
    )?;
    prefix.push(0.0);
    for &x in &gap_positions {
        prefix.push(prefix.last().unwrap() + x);
    }
    for &pivot in &gap_positions {
        while left < gap_positions.len() && gap_positions[left] < pivot - CLUSTER_RADIUS_PT {
            left += 1;
        }
        while right < gap_positions.len() && gap_positions[right] <= pivot + CLUSTER_RADIUS_PT {
            right += 1;
        }
        let count = right - left;
        if count > best_size {
            best_size = count;
            best_center = (prefix[right] - prefix[left]) / count as f32;
        }
    }

    // Gutter must sit near the page centre (0.30–0.70). A true two-column
    // body splits down the middle; a table's dominant gap (label column vs
    // data, or one of several cell boundaries) sits off-centre.
    let gutter_offset = best_center - x_min;
    let centre_ok = gutter_offset >= region_width * 0.30 && gutter_offset <= region_width * 0.70;
    if best_size < 16 || !centre_ok {
        return Ok(false);
    }

    if mean_chars > 20.0 {
        // Long-line two-column prose (the v0.3.57 accept path, unchanged
        // except the coverage denominator now excludes display rows):
        // concentration ≥ 62 %, coverage ≥ 50 % of (effective) lines.
        return Ok(best_size * 50 >= gap_positions.len() * 31 && best_size * 2 >= eff_lines);
    }

    // Short-line bodies (verse / lexicon / dictionary editions, #536): the
    // raw `mean_chars` floor used to reject these along with short-cell
    // tables. Admit them only under STRICTER, length-independent guards a
    // short-cell table cannot satisfy (Part 1a).
    let strict_concentration = best_size * 10 >= gap_positions.len() * 7; // ≥ 70 %
    let strict_coverage = best_size * 5 >= eff_lines * 3; // ≥ 60 % of lines
    if !(strict_concentration && strict_coverage) {
        return Ok(false);
    }
    // Column char-mass balance: each side of the gutter must carry ≥ 35 % of
    // the non-whitespace characters. A narrow label / verse-number column
    // paired with wide data is lopsided and rejected.
    let (mut left_chars, mut right_chars) = (0usize, 0usize);
    for s in spans {
        let cx = s.bbox.x + s.bbox.width * 0.5;
        if (cx - median).abs() > max_extent {
            continue;
        }
        let n = s.text.chars().filter(|c| !c.is_whitespace()).count();
        if cx < best_center {
            left_chars += n;
        } else {
            right_chars += n;
        }
    }
    let total = (left_chars + right_chars).max(1) as f32;
    if (left_chars as f32) < total * 0.35 || (right_chars as f32) < total * 0.35 {
        return Ok(false);
    }
    // Grid-row discriminator: a two-column body has ONE wide gap per line
    // (the gutter); a multi-cell numeric table has ≥ 2 wide gaps on most
    // rows (cell boundaries). Reject when the majority of lines are grid
    // rows — this is what keeps short-cell tables off the XY-cut path
    // without the raw `mean_chars` floor that also blocked short verse.
    Ok(multi_gap_lines * 2 <= eff_lines)
}

/// True if the spans cluster into lines whose leftmost X positions
/// form ≥ 2 distinct peaks separated by a clear gutter.
///
/// Body-level word spans fill the X axis continuously, so the
/// span-center histogram cannot tell two-column body text apart
/// from a single-column page with varied line lengths. The line-
/// start histogram does: in two-column body text most lines start
/// at one of two X positions (left-column-start or right-column-
/// start), and the wide gutter between the columns produces a
/// long zero-count stretch.
pub(crate) fn has_bimodal_line_starts(
    spans: &[crate::layout::TextSpan],
    allocation: ImportStorage<'_>,
) -> Result<bool> {
    const Y_BAND: f32 = 2.0;
    const BIN_PT: f32 = 5.0;
    const MIN_PEAK_COUNT: usize = 4;
    const MIN_GUTTER_PT: f32 = 30.0;

    if spans.len() < 24 {
        return Ok(false);
    }

    // Cluster spans into lines by Y (descending so top-of-page first).
    let mut lines = allocation.vec(spans.len())?; // (y, line_x_min)
    let mut sorted = allocation.collect(spans.iter())?;
    allocation.sort_by(&mut sorted, |a, b| {
        crate::utils::safe_float_cmp(b.bbox.y, a.bbox.y)
            .then_with(|| crate::utils::safe_float_cmp(a.bbox.x, b.bbox.x))
    })?;

    let mut current_y: Option<f32> = None;
    let mut current_xmin: f32 = f32::INFINITY;
    for s in &sorted {
        match current_y {
            Some(y) if (y - s.bbox.y).abs() <= Y_BAND => {
                current_xmin = current_xmin.min(s.bbox.x);
            },
            _ => {
                if let Some(y) = current_y {
                    if current_xmin.is_finite() {
                        lines.push((y, current_xmin));
                    }
                }
                current_y = Some(s.bbox.y);
                current_xmin = s.bbox.x;
            },
        }
    }
    if let Some(y) = current_y {
        if current_xmin.is_finite() {
            lines.push((y, current_xmin));
        }
    }
    if lines.len() < 16 {
        return Ok(false);
    }

    // Bin line-start X positions.
    let xmin = lines.iter().map(|(_, x)| *x).fold(f32::INFINITY, f32::min);
    let xmax = lines
        .iter()
        .map(|(_, x)| *x)
        .fold(f32::NEG_INFINITY, f32::max);
    if !(xmin.is_finite() && xmax.is_finite()) || xmax - xmin < MIN_GUTTER_PT {
        return Ok(false);
    }
    let bin_count = (((xmax - xmin) / BIN_PT).ceil() as usize).max(1);
    if bin_count > 4096 {
        return Ok(false); // degenerate CTM
    }
    let mut hist = allocation.vec(bin_count)?;
    hist.resize(bin_count, 0usize);
    for (_, x) in &lines {
        let idx = (((x - xmin) / BIN_PT) as usize).min(bin_count - 1);
        hist[idx] += 1;
    }

    // Scan for ≥ 2 peaks (count ≥ MIN_PEAK_COUNT) with a long
    // zero-count run between them.
    let mut peaks = allocation.vec(bin_count)?; // bin indices (peak center)
    let mut in_peak = false;
    let mut peak_start = 0usize;
    for (i, &c) in hist.iter().enumerate() {
        if c >= MIN_PEAK_COUNT {
            if !in_peak {
                peak_start = i;
                in_peak = true;
            }
        } else if c == 0 && in_peak {
            peaks.push((peak_start + i.saturating_sub(1)) / 2);
            in_peak = false;
        }
    }
    if in_peak {
        peaks.push((peak_start + hist.len() - 1) / 2);
    }
    if peaks.len() < 2 {
        return Ok(false);
    }

    // Check gutter: at least one pair of consecutive peaks must
    // have ≥ MIN_GUTTER_PT zero-count between them.
    let gutter_bins = (MIN_GUTTER_PT / BIN_PT) as usize;
    for w in peaks.windows(2) {
        let a = w[0];
        let b = w[1];
        if b <= a {
            continue;
        }
        let zeros = hist[a + 1..b].iter().filter(|&&c| c == 0).count();
        if zeros >= gutter_bins {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod resource_tests {
    use super::*;
    use crate::{
        geometry::Rect,
        import_limits::{ImportBudget, PdfImportLimits},
        layout::{PageFontStats, TextSpan},
    };

    #[test]
    fn a_resource_failure_is_not_a_single_column_decision_and_fresh_requests_recover() {
        let spans: Vec<_> = [(20.0, 100.0), (250.0, 100.0), (20.0, 80.0), (250.0, 80.0)]
            .into_iter()
            .map(|(x, y)| TextSpan {
                text: "column body".into(),
                font_name: "Body".into(),
                font_size: 12.0,
                bbox: Rect::new(x, y, 100.0, 12.0),
                ..Default::default()
            })
            .collect();
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(is_multi_column_page(&spans, ImportStorage(Some(&tiny))).is_err());
        assert!(is_multi_column_page(&[], ImportStorage(Some(&tiny))).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert!(is_multi_column_page(&spans, ImportStorage(Some(&fresh))).unwrap());
        assert_eq!(
            spans.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
            vec!["column body"; 4]
        );
    }

    #[test]
    fn font_statistics_propagate_failure_and_keep_the_public_statistics_contract() {
        let spans = [TextSpan {
            text: "weighted font".into(),
            font_name: "Body".into(),
            font_size: 12.0,
            bbox: Rect::new(20.0, 80.0, 100.0, 12.0),
            ..Default::default()
        }];
        let tiny = ImportBudget::new(PdfImportLimits {
            decoded_bytes: 1,
            ..Default::default()
        })
        .unwrap();
        assert!(PageFontStats::try_from_spans(&spans, ImportStorage(Some(&tiny))).is_err());
        assert!(PageFontStats::try_from_spans(&[], ImportStorage(Some(&tiny))).is_err());
        let fresh = ImportBudget::new(PdfImportLimits::default()).unwrap();
        assert_eq!(
            PageFontStats::try_from_spans(&spans, ImportStorage(Some(&fresh))).unwrap(),
            PageFontStats::from_spans(&spans)
        );
    }
}
