//! XY-Cut recursive spatial partitioning for multi-column text layout.
//!
//! This module implements the XY-Cut algorithm per PDF Spec Section 9.4 for
//! recursive geometric analysis without semantic heuristics. Uses projection
//! profiles to detect column boundaries in complex layouts.
//!
//! Per ISO 32000-1:2008:
//! - Section 9.4: Text Objects and coordinates
//! - Section 14.7: Logical Structure (prefers structure tree when available)
//!
//! # Algorithm Overview
//!
//! 1. Compute horizontal projection (white space density across X)
//! 2. Find valleys (gaps) where density < threshold
//! 3. Split region at widest valley (vertical line)
//! 4. Recursively partition left and right sub-regions
//! 5. Alternate to vertical projection if no horizontal valleys found
//! 6. Base case: Sort spans top-to-bottom, left-to-right
//!
//! # Performance
//!
//! Typical newspaper page: ~100 spans, < 5ms processing time
//! Recursive depth: O(log n) for balanced columns

use super::{ReadingOrderContext, ReadingOrderStrategy};
use crate::error::Result;
use crate::geometry::Rect;
use crate::layout::TextSpan;
use crate::pipeline::{OrderedTextSpan, ReadingOrderInfo};

/// Maximum density-array length for XY-cut projection profiles.
///
/// A normal PDF page is at most a few thousand points wide/tall. This limit of
/// 100 000 bins is generous (≈ 33× a 3000-point A0 page) while being small
/// enough to never cause an allocation problem. Spans whose bounding-box span
/// exceeds this limit are the result of a degenerate CTM; returning `None` from
/// the projection safely skips the split instead of attempting a multi-terabyte
/// allocation that would abort the process via `handle_alloc_error`.
const MAX_PROJECTION_SIZE: usize = 100_000;

/// Coarse classification of a region for the #534 multi-column-prose
/// fix. Used to gate the tight-gutter cut: tight cuts are only accepted on
/// regions that *positively* identify as prose, so the same XY-cut recursion
/// no longer corrupts table cells (the lesson — see lines 73–101).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegionKind {
    /// Tall stack of wide lines OR tall stack of half-column lines with
    /// substantial content per line. Safe to apply tight-gutter cuts.
    Prose,
    /// Short cells in a grid (mean characters per line < 8). Tight cuts
    /// here corrupt cell ordering — the canonical google_doc population
    /// table that reverted v0.3.53's two attempts is the prototype.
    Table,
    /// Anything else — too few lines, mixed shapes, decorative regions.
    /// Default to the behaviour (no tight cut).
    Mixed,
}

/// Contiguous run of bold-or-larger-font spans spanning ≥ 2 visual lines
/// that the XY-cut splitter must treat as an atomic block. Built by
/// `find_heading_runs` (v0.3.55 #543) BEFORE recursive partitioning,
/// then substituted into the partition input as a single wide synthetic
/// span so cluster-detection / valley-finding can't drive a vertical
/// cut THROUGH a wrapped heading.
///
/// After partition completes, `expand_blocks` projects the synthetic
/// placeholder back into its constituent original spans, preserving
/// each span's per-glyph metadata for downstream consumers
/// (markdown converter heading-level inference, layout-preserving
/// DOCX export, etc.).
#[derive(Debug, Clone)]
struct HeadingRun {
    /// Indices into the original `&[TextSpan]` slice, in reading order
    /// (top-to-bottom, left-to-right within a line).
    span_indices: Vec<usize>,
    /// Union of the constituent spans' bboxes. Substituted for each
    /// individual bbox during partition so the heading appears as one
    /// wide bbox.
    combined_bbox: Rect,
}

/// Union of the bboxes of `spans[indices]`. Empty index list yields a
/// zero-sized rect at the origin (never built in practice — guarded by
/// the caller).
fn union_bboxes(spans: &[TextSpan], indices: &[usize]) -> Rect {
    let mut x_min = f32::MAX;
    let mut y_min = f32::MAX;
    let mut x_max = f32::MIN;
    let mut y_max = f32::MIN;
    for &i in indices {
        let b = spans[i].bbox;
        x_min = x_min.min(b.left());
        x_max = x_max.max(b.right());
        y_min = y_min.min(b.top());
        y_max = y_max.max(b.bottom());
    }
    if x_min == f32::MAX {
        return Rect::default();
    }
    Rect::from_points(x_min, y_min, x_max, y_max)
}

/// XY-Cut recursive spatial partitioning strategy.
///
/// Detects columns using projection profiles and white space analysis.
/// Suitable for newspapers, academic papers, and multi-column layouts.
pub struct XYCutStrategy {
    /// Minimum number of spans in a region before attempting split (default: 5).
    /// Prevents excessive recursion on small regions.
    pub min_spans_for_split: usize,

    /// Valley threshold as fraction of peak projection density (default: 0.3).
    /// Lower values detect narrower gutters, higher values only detect wide gaps.
    pub valley_threshold: f32,

    /// Minimum valley width in points (default: 15.0).
    /// Prevents detecting single-character gaps as column boundaries.
    pub min_valley_width: f32,

    /// Enable horizontal partitioning first, fallback to vertical (default: true).
    ///
    /// Per PDF Spec ISO 32000-1:2008 §14.8.4 (Logical Structure reading order),
    /// column detection is the primary purpose of XY-Cut — horizontal-first
    /// (vertical cut line) splits columns before rows, matching Western
    /// top-down-left-to-right reading order in multi-column documents.
    /// Callers with row-dominant layouts can override via
    /// `with_prefer_horizontal(false)`.
    pub prefer_horizontal: bool,
}

/// Cap on `partition_indexed` recursion depth. Real layouts nest only a few
/// splits deep; this bound only fires on the singleton-peel pathology (many
/// distinct-Y header/footer strips) where unbounded depth is O(n² log n). Set
/// high enough that no real document reaches it.
const MAX_PARTITION_DEPTH: u32 = 64;

impl Default for XYCutStrategy {
    fn default() -> Self {
        Self {
            min_spans_for_split: 5,
            valley_threshold: 0.3,
            // 15pt. Issue #7 (multi-column prose interleaving on
            // issue_07_orphaned_fragments.pdf) was attempted TWICE and
            // REVERTED both times — the 70-PDF sweep caught data
            // corruption in google_doc_document.pdf's population table
            // ("273.879.7501" -> "1273.879.750") each time:
            //
            //   Attempt 1 — lower min_valley_width 15 -> 12 so the tight
            //   ~12pt two-column gutter is detected. Also split the
            //   table's ~12pt inter-cell gaps -> reordered digits.
            //
            //   Attempt 2 — a structural find_two_column_prose_split
            //   (exactly-two recurring left-edge clusters, wide columns,
            //   clean gutter) tried before the single-column check. It
            //   never fired on issue_07's WHOLE page (three left-edge
            //   clusters: full-width intro/footer @60 + left @82 + right
            //   @312, because is_single_column blocks band separation
            //   first), yet it DID fire on a 2-column sub-region of the
            //   google_doc table and reordered cells.
            //
            // Root cause: the same XY-Cut machinery orders both
            // prose-columns and table-cells. Any sensitivity increase
            // that catches issue_07's tight 2-column prose also splits
            // table cells and corrupts data. A correct #7 fix needs a
            // real table-vs-prose classifier (column cells are short
            // values; prose columns are tall stacks of wide lines) AND
            // recursive band-separation of full-width header/footer rows
            // before column detection — a substantial XY-Cut redesign,
            // validated against the full CI corpus, not a local tweak.
            min_valley_width: 15.0,
            prefer_horizontal: true,
        }
    }
}

impl XYCutStrategy {
    /// Create a new XY-Cut strategy with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom valley threshold (0.0-1.0).
    pub fn with_valley_threshold(mut self, threshold: f32) -> Self {
        self.valley_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Create with custom minimum valley width.
    pub fn with_min_valley_width(mut self, width: f32) -> Self {
        self.min_valley_width = width.max(1.0);
        self
    }

    /// Enable or disable horizontal partitioning first preference.
    pub fn with_prefer_horizontal(mut self, prefer: bool) -> Self {
        self.prefer_horizontal = prefer;
        self
    }

    /// Core recursive partitioning algorithm.
    ///
    /// Public for use by MarkdownConverter's ColumnAware reading order mode.
    ///
    /// (#543): runs a pre-pass that detects multi-line heading runs
    /// (bold or larger-than-body font, ≥ 2 wrapped lines with matching
    /// X-extent) and locks them as atomic blocks the recursive splitter
    /// cannot split. Without this, a wrapped heading whose tail lines
    /// Y-overlap with adjacent-column dense content (table caption, table
    /// row, image label) gets bucketed across columns: line 1 glued to the
    /// body paragraph, line 2..N orphaned into the wrong block — and the
    /// markdown converter then promotes the orphan tail to a phantom
    /// heading (`### …`) in the wrong location.
    pub fn partition_region(&self, spans: &[TextSpan]) -> Vec<Vec<TextSpan>> {
        let heading_runs = self.find_heading_runs(spans);
        if heading_runs.is_empty() {
            // Hot path: no headings found, skip the synthesize/expand
            // pair entirely so the cost is bounded to one O(n log n) sort
            // inside find_heading_runs.
            let indices: Vec<usize> = (0..spans.len()).collect();
            let index_groups = self.partition_indexed(spans, &indices);
            return index_groups
                .into_iter()
                .map(|group| group.into_iter().map(|i| spans[i].clone()).collect())
                .collect();
        }

        // Build synthetic span list: each heading run collapses to ONE
        // wide span carrying the union bbox; non-heading spans pass
        // through unchanged. The synthetic list is shorter than the
        // original by (sum_of_run_sizes - num_runs).
        let (synthetic, synthetic_origin) = self.synthesize_for_partition(spans, &heading_runs);
        let synth_indices: Vec<usize> = (0..synthetic.len()).collect();
        let synth_groups = self.partition_indexed(&synthetic, &synth_indices);

        // Project synthetic-space groups back into original-span space:
        // each synthetic span that came from a heading run gets expanded
        // back into its constituent original spans (in their original
        // reading-order sequence within the run).
        self.expand_blocks(synth_groups, spans, &synthetic_origin)
    }

    /// Detect contiguous bold/large-font runs that span ≥ 2 lines with
    /// matching X-extent (i.e. wrapped subsection headings).
    ///
    /// Per the fix-543 plan §A.2: two adjacent spans (in reading
    /// order) are considered to belong to the same heading run when
    /// ALL of the following hold:
    ///
    /// 1. Both are heading-like (bold, OR font_size > median × 1.15).
    /// 2. Same font_size (within 0.5 pt epsilon).
    /// 3. Same bold flag.
    /// 4. Next span's left edge is within `[prev.left, prev.left + 6pt]`
    ///    (wrapped heading lines often re-indent by up to ~6pt).
    /// 5. Next span sits ≤ 1.5 × line-height below the previous span
    ///    (a single-line gap; double-line gaps are paragraph breaks).
    ///
    /// `median_font_size` is computed across non-bold spans so heavy
    /// bold runs don't bias the body-size estimate upward.
    fn find_heading_runs(&self, spans: &[TextSpan]) -> Vec<HeadingRun> {
        if spans.len() < 2 {
            return Vec::new();
        }

        // Median body font size from NON-bold spans only. Bold spans
        // typically sit at heading sizes (bigger than body), so including
        // them biases the median high and we'd miss bold headings whose
        // size sits between body and the heavier weight tier.
        let mut non_bold_sizes: Vec<f32> = spans
            .iter()
            .filter(|s| !s.font_weight.is_bold())
            .map(|s| s.font_size)
            .filter(|&sz| sz > 0.0)
            .collect();
        let median_body = if non_bold_sizes.is_empty() {
            // Fallback: all spans bold (or zero-size). Use overall median.
            let mut sizes: Vec<f32> = spans
                .iter()
                .map(|s| s.font_size)
                .filter(|&sz| sz > 0.0)
                .collect();
            if sizes.is_empty() {
                return Vec::new();
            }
            sizes.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
            sizes[sizes.len() / 2]
        } else {
            non_bold_sizes.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
            non_bold_sizes[non_bold_sizes.len() / 2]
        };
        let heading_size_floor = median_body * 1.15;

        let is_heading_like =
            |s: &TextSpan| -> bool { s.font_weight.is_bold() || s.font_size > heading_size_floor };

        // Sort indices by reading order (top of page first; Rect::top()
        // is the SMALLER Y of the normalized rect — see comment at
        // line ~885 — so larger Y = higher on page in PDF coords;
        // we want DESCENDING Y here).
        let mut order: Vec<usize> = (0..spans.len()).collect();
        order.sort_by(|&a, &b| {
            let y_cmp = crate::utils::safe_float_cmp(spans[b].bbox.top(), spans[a].bbox.top());
            if y_cmp != std::cmp::Ordering::Equal {
                return y_cmp;
            }
            crate::utils::safe_float_cmp(spans[a].bbox.left(), spans[b].bbox.left())
        });

        // Cluster reading-order-adjacent heading-like spans into runs.
        // The same line may carry multiple bold spans (one per Tj
        // segment); we collapse runs across lines, not within a line.
        let indent_tolerance = 6.0_f32;
        let font_eps = 0.5_f32;
        let mut runs: Vec<Vec<usize>> = Vec::new();
        let mut current: Vec<usize> = Vec::new();

        for &idx in &order {
            let span = &spans[idx];
            if !is_heading_like(span) {
                if !current.is_empty() {
                    runs.push(std::mem::take(&mut current));
                }
                continue;
            }

            if current.is_empty() {
                current.push(idx);
                continue;
            }

            let last_idx = *current.last().unwrap();
            let last = &spans[last_idx];

            // (2) same font size, (3) same bold flag.
            let size_ok = (span.font_size - last.font_size).abs() <= font_eps;
            let bold_ok = span.font_weight.is_bold() == last.font_weight.is_bold();

            // Same-line: top within 1 pt of last's top — fold without
            // applying indent/leading checks (both spans belong to the
            // SAME wrapped-heading line, e.g. two bold Tj segments).
            let same_line = (span.bbox.top() - last.bbox.top()).abs() <= 1.0;

            if size_ok && bold_ok && same_line {
                current.push(idx);
                continue;
            }

            // Different line: enforce indent (4) + leading (5).
            // line_height = max of the two spans' bbox heights, plus a
            // floor of font_size to handle ascender-only / descender-only
            // glyphs with collapsed bboxes.
            let line_h = last
                .bbox
                .height
                .max(span.bbox.height)
                .max(last.font_size)
                .max(1.0);
            let leading_tolerance = line_h * 1.5;

            // PDF coords: y grows up, so the wrapped line sits at a
            // SMALLER bbox.top than the previous line. The gap between
            // last's bottom and span's top should fit inside the leading
            // tolerance.
            let last_bottom = last.bbox.top(); // smaller-Y edge in PDF coords (see find_vertical_split comment)
            let span_top = span.bbox.top();
            let vertical_gap = (last_bottom - span_top).abs();

            let indent_ok = span.bbox.left() >= last.bbox.left() - indent_tolerance
                && span.bbox.left() <= last.bbox.left() + indent_tolerance;
            let leading_ok = vertical_gap <= leading_tolerance;

            if size_ok && bold_ok && indent_ok && leading_ok {
                current.push(idx);
            } else {
                runs.push(std::mem::take(&mut current));
                current.push(idx);
            }
        }
        if !current.is_empty() {
            runs.push(current);
        }

        // A run becomes a HeadingRun only when it spans ≥ 2 distinct
        // lines. Single-line bold spans (inline emphasis, lone short
        // headings) don't need locking — XY-cut handles them correctly
        // already, and locking them would be a no-op for the splitter
        // but adds overhead.
        runs.into_iter()
            .filter_map(|span_indices| {
                if span_indices.len() < 2 {
                    return None;
                }
                let mut distinct_lines = std::collections::BTreeSet::new();
                for &i in &span_indices {
                    distinct_lines.insert(spans[i].bbox.top().round() as i32);
                }
                if distinct_lines.len() < 2 {
                    return None;
                }
                Some(HeadingRun {
                    combined_bbox: union_bboxes(spans, &span_indices),
                    span_indices,
                })
            })
            .collect()
    }

    /// Build a synthetic span list where each detected `HeadingRun`
    /// collapses to ONE wide synthetic span carrying the union bbox.
    /// Non-heading spans pass through unchanged.
    ///
    /// Returns:
    /// - `synthetic`: the input to `partition_indexed`.
    /// - `synthetic_origin[k]`: indices of ORIGINAL spans backing
    ///   synthetic span `k`. Length 1 for pass-throughs, ≥ 2 for
    ///   heading-run placeholders. Used by `expand_blocks` to project
    ///   partition output back into original-span space.
    fn synthesize_for_partition(
        &self,
        spans: &[TextSpan],
        runs: &[HeadingRun],
    ) -> (Vec<TextSpan>, Vec<Vec<usize>>) {
        // Mark each original span with the heading-run it belongs to
        // (or None for pass-through).
        let mut in_run: Vec<Option<usize>> = vec![None; spans.len()];
        for (r_idx, run) in runs.iter().enumerate() {
            for &i in &run.span_indices {
                in_run[i] = Some(r_idx);
            }
        }

        let mut synthetic: Vec<TextSpan> = Vec::with_capacity(spans.len());
        let mut origins: Vec<Vec<usize>> = Vec::with_capacity(spans.len());
        let mut emitted_run = vec![false; runs.len()];

        for (i, span) in spans.iter().enumerate() {
            match in_run[i] {
                None => {
                    synthetic.push(span.clone());
                    origins.push(vec![i]);
                },
                Some(r_idx) if !emitted_run[r_idx] => {
                    // Emit the run as a synthetic placeholder at the
                    // position of its first-encountered span.
                    let run = &runs[r_idx];
                    let mut placeholder = span.clone();
                    placeholder.bbox = run.combined_bbox;
                    // Concatenate the run's text with single spaces so
                    // is_single_column_region's core-width estimate is
                    // proportional to the actual heading length, not the
                    // single first-line fragment.
                    let mut combined_text = String::new();
                    for (k, &si) in run.span_indices.iter().enumerate() {
                        if k > 0 {
                            combined_text.push(' ');
                        }
                        combined_text.push_str(&spans[si].text);
                    }
                    placeholder.text = combined_text;
                    synthetic.push(placeholder);
                    origins.push(run.span_indices.clone());
                    emitted_run[r_idx] = true;
                },
                Some(_) => { /* already emitted — skip later spans of the run */ },
            }
        }

        (synthetic, origins)
    }

    /// Project partition groups from synthetic-span space back into
    /// original-span space, expanding each heading-run placeholder into
    /// its constituent original spans (in their original ordering).
    fn expand_blocks(
        &self,
        synth_groups: Vec<Vec<usize>>,
        original: &[TextSpan],
        synthetic_origin: &[Vec<usize>],
    ) -> Vec<Vec<TextSpan>> {
        synth_groups
            .into_iter()
            .map(|group| {
                let mut out = Vec::with_capacity(group.len());
                for synth_idx in group {
                    for &orig_idx in &synthetic_origin[synth_idx] {
                        out.push(original[orig_idx].clone());
                    }
                }
                out
            })
            .collect()
    }

    /// Index-based recursive partitioning — returns groups of indices into the input span slice.
    ///
    /// Avoids cloning TextSpan at every recursive split level. Spans are only
    /// read through shared reference; indices are partitioned instead.
    fn partition_indexed(&self, all_spans: &[TextSpan], indices: &[usize]) -> Vec<Vec<usize>> {
        self.partition_indexed_depth(all_spans, indices, 0)
    }

    /// Depth-bounded recursive partition. `find_vertical_split_indexed` permits
    /// singleton peels, so without a cap a page with many distinct-Y
    /// header/footer strips can recurse O(n) deep (O(n² log n) work);
    /// `MAX_PARTITION_DEPTH` bounds it.
    fn partition_indexed_depth(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
        depth: u32,
    ) -> Vec<Vec<usize>> {
        if indices.is_empty() {
            return Vec::new();
        }

        // Base case: small region, don't split further via the recursive
        // partitioner. Below `min_spans_for_split`, the statistical
        // prose/table classifiers (`classify_region_kind`,
        // `detect_two_column_prose`, `detect_narrow_gutter_prose`) all have
        // their own internal minimum-span floors (6/8/24) far above this
        // one and unconditionally decline to classify — so a flat
        // "impose Y-then-X row-major order" was applied even to a genuine
        // sparse 2-column page (#979: a 2-column, 2-row prose page —
        // 4 spans — read back row-major/interleaved instead of
        // column-major).
        //
        // A geometric gutter check alone can't distinguish "sparse 2-column
        // prose" from "a 2x2 row-major table" at this scale either — both
        // produce an identical clean-gutter signature, and the table-row
        // guard inside `find_horizontal_split_indexed` needs >=3 rows to
        // reach confidence, so it's a no-op at 2 rows. Rather than commit
        // to a (left, right) column grouping we can't justify being
        // correct, fall back to the page's own content-stream emission
        // order when a clean gutter exists at all (PDFium parity, per the
        // reporter's own cross-tool probe: PDFium performs no prose/table
        // decision here either, it just follows stream order). This
        // relies on the empirical tendency of table generators to emit
        // cells row-major and column-generators to emit column-major, in
        // the absence of any other geometric signal being decidable at
        // this scale. Falls back to the flat Y-then-X sort when no clean
        // gutter is found at all (ordinary short single-column snippets).
        if indices.len() < self.min_spans_for_split {
            if self
                .find_horizontal_split_indexed(all_spans, indices)
                .is_some()
            {
                let mut stream_order: Vec<usize> = indices.to_vec();
                stream_order.sort_by_key(|&i| all_spans[i].sequence);
                return vec![stream_order];
            }
            return vec![self.sort_indices(all_spans, indices)];
        }

        // Depth cap: bail to a flat sort rather than recurse unbounded.
        if depth >= MAX_PARTITION_DEPTH {
            return vec![self.sort_indices(all_spans, indices)];
        }

        // (#534): two-column-prose probe BEFORE the
        // single-column short-circuit. Tight gutters (~10-15pt) that
        // sit below `min_valley_width` defeat the standard projection-
        // valley detector, and the wide+dense heuristic inside
        // `is_single_column_region` mis-classifies the body as one
        // column because each line's bbox spans the narrow gutter.
        // The probe positively identifies the 2-column-prose shape
        // (gutter-radius left-edge clusters + ≥6 narrow lines +
        // classify_region_kind == Prose) and only fires when ALL of
        // those signals agree. Critically, the Prose gate prevents
        // the false positive that reverted v0.3.53's attempts on a
        // 2-column sub-region of the google_doc population table
        // (mean_chars < 8 → Table → bail).
        //
        // **Band-separation first**: when the probe would fire AND a
        // clean vertical band-separation (top header / body / bottom
        // footer) is available, peel the band off BEFORE the column
        // cut. Without this step, full-width header / footer rows
        // get absorbed into one of the two column halves and end up
        // mid-page in reading order — the failure mode on the
        // 1256-page French Bible from #536 where the chapter-header
        // band and page-number footer were full-width and span the
        // gutter. The signal for "band": a vertical split whose
        // smaller side has ≤ 25 % of the region's spans (a tight
        // band relative to the body it sits next to).
        // Two-column-prose detector based on line-start clustering.
        // When it fires, peel any wide Y-band first (title / authors
        // / abstract / footer often span the gutter) before the
        // column cut, so they don't get fragmented across columns.
        // Each peeled band is re-classified inside the recursive
        // call.
        //
        // Classify once and pass to both prose detectors below; each gated on
        // `classify_region_kind == Prose` and re-ran the same line clustering.
        let region_kind = self.classify_region_kind(all_spans, indices);
        if let Some(gutter_x) = self.detect_two_column_prose(all_spans, indices, region_kind) {
            if let Some((above, below)) = self.find_vertical_split_indexed(all_spans, indices) {
                log::debug!(
                    "XY-cut: peeling Y-band before column cut, above={} below={}",
                    above.len(),
                    below.len()
                );
                let mut result = self.partition_indexed_depth(all_spans, &above, depth + 1);
                result.extend(self.partition_indexed_depth(all_spans, &below, depth + 1));
                return result;
            }
            let (left, right): (Vec<usize>, Vec<usize>) = indices
                .iter()
                .copied()
                .partition(|&i| all_spans[i].bbox.left() < gutter_x);
            if !left.is_empty() && !right.is_empty() {
                log::debug!(
                    "XY-cut: two-column-prose detected, gutter_x={:.1}, left={} right={}",
                    gutter_x,
                    left.len(),
                    right.len()
                );
                let mut result = self.partition_indexed_depth(all_spans, &left, depth + 1);
                result.extend(self.partition_indexed_depth(all_spans, &right, depth + 1));
                return result;
            }
        }

        // Narrow-gutter prose detector — second pass for layouts
        // where the line-start cluster shape is masked by outlier
        // singletons (title / caption / equation rows scattering
        // extra clusters that block the primary detector). Cuts
        // directly at the gap-cluster centre WITHOUT peeling a
        // Y-band first: for these pages `find_vertical_split`
        // tends to fire on mid-body paragraph gaps and bisect
        // the body across the peel — both halves then lose
        // enough gutter signal that the column cut never reaches
        // them on recursion.
        if let Some(gutter_x) = self.detect_narrow_gutter_prose(all_spans, indices, region_kind) {
            let (left, right): (Vec<usize>, Vec<usize>) = indices
                .iter()
                .copied()
                .partition(|&i| all_spans[i].bbox.left() < gutter_x);
            if !left.is_empty() && !right.is_empty() {
                log::debug!(
                    "XY-cut: narrow-gutter prose detected, gutter_x={:.1}, left={} right={}",
                    gutter_x,
                    left.len(),
                    right.len()
                );
                let mut result = self.partition_indexed_depth(all_spans, &left, depth + 1);
                result.extend(self.partition_indexed_depth(all_spans, &right, depth + 1));
                return result;
            }
        }

        // Detect single-column body text up-front and skip all spatial
        // splits. Real body text has density dips (indented code, short
        // last-lines, paragraph breaks) that would otherwise trigger
        // spurious horizontal (column) or vertical (row) splits,
        // scrambling reading order. The subsequent sort-by-Y already
        // handles row order within a column.
        if self.is_single_column_region(all_spans, indices) {
            return vec![self.sort_indices(all_spans, indices)];
        }

        let split_h =
            |s: &Self, sp: &[TextSpan], idx: &[usize]| s.find_horizontal_split_indexed(sp, idx);
        let split_v =
            |s: &Self, sp: &[TextSpan], idx: &[usize]| s.find_vertical_split_indexed(sp, idx);

        let first_split = if self.prefer_horizontal {
            split_h
        } else {
            split_v
        };
        let second_split = if self.prefer_horizontal {
            split_v
        } else {
            split_h
        };

        if let Some((a, b)) = first_split(self, all_spans, indices) {
            let mut result = self.partition_indexed_depth(all_spans, &a, depth + 1);
            result.extend(self.partition_indexed_depth(all_spans, &b, depth + 1));
            return result;
        }

        if let Some((a, b)) = second_split(self, all_spans, indices) {
            let mut result = self.partition_indexed_depth(all_spans, &a, depth + 1);
            result.extend(self.partition_indexed_depth(all_spans, &b, depth + 1));
            return result;
        }

        // No split found, return as single group
        vec![self.sort_indices(all_spans, indices)]
    }

    /// Classifier verdict for a region — used to gate the tight-gutter
    /// column-split path (#534) so the same XY-cut recursion no longer
    /// corrupts table cells (the lesson).
    ///
    /// See the inline post-mortem at lines 73–101: two prior attempts at
    /// the multi-column-prose fix were reverted by the 70-PDF sweep when
    /// they accidentally fired on a 2-column sub-region of a real table
    /// and reordered digits. The fix has to *positively identify prose*
    /// before allowing the tight cut — not merely *fail to identify
    /// table*. This classifier is that positive identification.
    fn classify_region_kind(&self, all_spans: &[TextSpan], indices: &[usize]) -> RegionKind {
        // Cheap shape check first.
        if indices.len() < 6 {
            return RegionKind::Mixed;
        }

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        for &i in indices {
            x_min = x_min.min(all_spans[i].bbox.left());
            x_max = x_max.max(all_spans[i].bbox.right());
        }
        let region_width = x_max - x_min;
        if region_width <= 10.0 {
            return RegionKind::Mixed;
        }

        // Cluster spans into lines by rounded Y.
        let mut lines: std::collections::BTreeMap<i32, (f32, f32, usize)> =
            std::collections::BTreeMap::new();
        for &i in indices {
            let s = &all_spans[i];
            let y_key = s.bbox.top().round() as i32;
            let nonws_chars = s.text.chars().filter(|c| !c.is_whitespace()).count();
            let entry = lines.entry(y_key).or_insert((f32::MAX, f32::MIN, 0));
            entry.0 = entry.0.min(s.bbox.left());
            entry.1 = entry.1.max(s.bbox.right());
            entry.2 += nonws_chars;
        }

        let line_count = lines.len();
        if line_count < 6 {
            // Too few lines to be a substantial prose body. Headings,
            // captions, single paragraphs all land here — leave them to
            // the default XY-cut behaviour.
            return RegionKind::Mixed;
        }

        // Per-line statistics: average char count and the count of
        // "narrow" lines whose extent < 0.6 × region_width (a column-half
        // line) and "wide" lines whose extent ≥ 0.6 × region_width (a
        // body-text or table-row line). Table cells are narrow; tables
        // have many such narrow lines but with very short content.
        let mut total_chars = 0usize;
        let mut narrow_lines = 0usize;
        let mut wide_lines = 0usize;
        for (left, right, chars) in lines.values() {
            total_chars += chars;
            let extent = (*right - *left).max(0.0);
            if extent < region_width * 0.6 {
                narrow_lines += 1;
            } else {
                wide_lines += 1;
            }
        }
        let mean_chars = total_chars as f32 / line_count as f32;

        // PROSE: tall stack of wide lines OR tall stack of half-column
        // lines with substantial content per line.
        //   - mean_chars > 20: real prose, not table cells
        //   - line_count ≥ 6: substantial column
        //   - either:
        //     * majority of lines are wide (single-column body), OR
        //     * majority of lines are narrow with mean_chars > 20
        //       (two half-column lines with prose content)
        let mostly_wide = wide_lines * 2 > line_count;
        let mostly_narrow = narrow_lines * 2 > line_count;
        if mean_chars > 20.0 && (mostly_wide || mostly_narrow) {
            return RegionKind::Prose;
        }

        // SHORT-LINE PROSE (#536 short-verse two-column bodies): the
        // `mean_chars > 20` guard above deliberately rejected short-verse
        // two-column bodies (Bible / lexicon editions — a verse fragment
        // per column-line is often < 20 non-whitespace chars) along with
        // short-cell tables. The guard was doing two jobs at once. Here we
        // re-admit ONLY the short-line case that carries a *strong central
        // gutter corridor* a short-cell table cannot fake: a single
        // persistent vertical gutter near the region centre, present on a
        // high fraction of lines, with balanced left/right char mass and
        // ≤ 2 left-edge clusters. A label+data table fails this on
        // concentration/coverage (its gaps scatter across cell
        // boundaries), centre (the dominant gap sits off-centre),
        // char-balance (the label column is tiny), or left-edge clusters
        // (≥ 3 columns). The long-line accept path above is byte-unchanged.
        if mean_chars <= 20.0
            && self.short_line_central_corridor_prose(all_spans, indices, x_min, region_width)
        {
            return RegionKind::Prose;
        }

        // TABLE: lots of narrow lines, short content per line (mean_chars
        // < 8). The google_doc_document.pdf population table —
        // the canonical regression that reverted attempts 1 & 2 — sits
        // squarely here (digit-only cells, ≤ 7 chars each).
        if mean_chars < 8.0 {
            return RegionKind::Table;
        }

        // Anything in between (e.g. captions with headings, mixed
        // figure-and-text bands) → don't risk the tight cut.
        RegionKind::Mixed
    }

    /// Short-line two-column-prose admission (#536, v0.3.58 Part 1a).
    ///
    /// Called from `classify_region_kind` ONLY for the short-line case
    /// (`mean_chars <= 20`) that the long-line prose guard rejects. A
    /// short-verse two-column body (verse-per-line bibles/lexicons) has
    /// short lines yet a strong, table-independent central gutter; a
    /// short-cell numeric table has short lines and NO such corridor.
    ///
    /// Returns `true` only when ALL of the following hold — each one a
    /// length-independent discriminator a short-cell label+data table
    /// cannot satisfy:
    ///   - a single persistent vertical gutter exists: per-line largest
    ///     within-line gap clusters at one X (10 pt radius) covering
    ///     **≥ 70 %** of gap-bearing lines (concentration) and present on
    ///     **≥ 60 %** of all lines (coverage) — a table's dominant gap
    ///     scatters across cell boundaries and appears on a minority of
    ///     rows;
    ///   - that gutter sits near the region centre: offset ∈
    ///     **[0.30, 0.70]·region_width** — a label+data table's dominant
    ///     gap sits off-centre;
    ///   - **left/right char balance:** non-whitespace char mass on each
    ///     side of the gutter is **≥ 35 %** of the total — a label column
    ///     is lopsided (one side is tiny numeric labels);
    ///   - **≤ 2 left-edge clusters** left of the gutter (30 pt radius) —
    ///     a real two-column body starts each column at one X; an
    ///     N-column table has ≥ 3 left-edge clusters (the fix-534
    ///     `left_edge_clusters >= 3 → Mixed` rule).
    fn short_line_central_corridor_prose(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
        x_min: f32,
        region_width: f32,
    ) -> bool {
        if region_width <= 0.0 {
            return false;
        }

        // Re-cluster spans into lines, keeping PER-SPAN (left, right, chars)
        // so we can find the within-line gutter gap and split char mass.
        let mut lines: std::collections::BTreeMap<i32, Vec<(f32, f32, usize)>> =
            std::collections::BTreeMap::new();
        for &i in indices {
            let s = &all_spans[i];
            let y_key = s.bbox.top().round() as i32;
            let nonws = s.text.chars().filter(|c| !c.is_whitespace()).count();
            lines
                .entry(y_key)
                .or_default()
                .push((s.bbox.left(), s.bbox.right(), nonws));
        }
        let total_lines = lines.len();
        if total_lines == 0 {
            return false;
        }

        // Per-line: largest within-line gap and its midpoint X. A gap of
        // ≥ 6 pt suppresses ordinary 2–5 pt word spacing.
        const MIN_GAP_PT: f32 = 6.0;
        let mut gap_positions: Vec<f32> = Vec::new();
        for line_spans in lines.values() {
            if line_spans.len() < 2 {
                continue;
            }
            let mut sorted = line_spans.clone();
            sorted.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
            let mut largest_gap = 0.0_f32;
            let mut largest_mid = 0.0_f32;
            for w in sorted.windows(2) {
                let gap = w[1].0 - w[0].1;
                if gap > largest_gap {
                    largest_gap = gap;
                    largest_mid = (w[0].1 + w[1].0) * 0.5;
                }
            }
            if largest_gap >= MIN_GAP_PT {
                gap_positions.push(largest_mid);
            }
        }
        if gap_positions.is_empty() {
            return false;
        }

        // Cluster gap positions (10 pt radius) → dominant corridor.
        const CLUSTER_RADIUS_PT: f32 = 10.0;
        let mut sorted_gaps = gap_positions.clone();
        sorted_gaps.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        let mut best_size = 0usize;
        let mut best_center = 0.0_f32;
        for &pivot in &sorted_gaps {
            let lo = pivot - CLUSTER_RADIUS_PT;
            let hi = pivot + CLUSTER_RADIUS_PT;
            let mut count = 0usize;
            let mut sum = 0.0_f32;
            for &g in &sorted_gaps {
                if g >= lo && g <= hi {
                    count += 1;
                    sum += g;
                }
            }
            if count > best_size {
                best_size = count;
                best_center = sum / count as f32;
            }
        }
        if best_size == 0 {
            return false;
        }

        // Concentration ≥ 70 % of gap-bearing lines at one X.
        if best_size * 10 < gap_positions.len() * 7 {
            return false;
        }
        // Coverage ≥ 60 % of ALL lines carry the corridor.
        if best_size * 10 < total_lines * 6 {
            return false;
        }
        // Centre: gutter offset ∈ [0.30, 0.70]·region_width.
        let gutter_offset = best_center - x_min;
        if gutter_offset < region_width * 0.30 || gutter_offset > region_width * 0.70 {
            return false;
        }

        // Left/right non-whitespace char balance about the corridor:
        // each side ≥ 35 % of total. A label-column table is lopsided.
        let mut left_chars = 0usize;
        let mut right_chars = 0usize;
        for line_spans in lines.values() {
            for &(l, r, chars) in line_spans {
                let mid = (l + r) * 0.5;
                if mid < best_center {
                    left_chars += chars;
                } else {
                    right_chars += chars;
                }
            }
        }
        let total_chars = left_chars + right_chars;
        if total_chars == 0 {
            return false;
        }
        if (left_chars as f32) < total_chars as f32 * 0.35
            || (right_chars as f32) < total_chars as f32 * 0.35
        {
            return false;
        }

        // ≤ 2 left-edge clusters left of the corridor (30 pt radius). A
        // real two-column body starts its left column at one X (one
        // cluster, maybe two counting a paragraph indent); an N-column
        // table left of the corridor has several cell-start X's → ≥ 3
        // clusters. Cluster EVERY span left-edge that lies left of the
        // corridor (not just each line's minimum) so multi-column cell
        // starts are not collapsed into one cluster.
        const LEFT_CLUSTER_RADIUS_PT: f32 = 30.0;
        let mut clusters: Vec<(f32, usize)> = Vec::new();
        for line_spans in lines.values() {
            for &(l, _, _) in line_spans {
                if l >= best_center {
                    continue;
                }
                if let Some(c) = clusters
                    .iter_mut()
                    .find(|(c, _)| (*c - l).abs() <= LEFT_CLUSTER_RADIUS_PT)
                {
                    let count = c.1 as f32;
                    c.0 = (c.0 * count + l) / (count + 1.0);
                    c.1 += 1;
                } else {
                    clusters.push((l, 1));
                }
            }
        }
        // Drop singleton/noise clusters (< 2 lines) before counting, so a
        // lone outlier left-edge doesn't inflate the count.
        let dominant_left_clusters = clusters.iter().filter(|(_, n)| *n >= 2).count();
        if dominant_left_clusters >= 3 {
            return false;
        }

        true
    }

    /// Two-column-prose probe (#534) — does this region look like two
    /// side-by-side columns of prose with a tight gutter (~10-15pt)?
    ///
    /// Called from `is_single_column_region` when the wide+dense
    /// heuristic would otherwise short-circuit the region as
    /// single-column. Distinguishing signal: most lines fit inside
    /// **one** half of the region width (column-half lines), and the
    /// left edges cluster into exactly **two** groups separated by
    /// approximately half the region width.
    ///
    /// Gated on `classify_region_kind == Prose` so the same machinery
    /// doesn't fire on a 2-column sub-region of a table (the v0.3.53
    /// failure mode).
    ///
    /// Returns `Some(gutter_x)` when a 2-column prose layout is
    /// detected — the caller treats that as a non-single-column verdict
    /// and lets `find_horizontal_split_indexed` cut at the gutter.
    fn detect_two_column_prose(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
        region_kind: RegionKind,
    ) -> Option<f32> {
        // Cheap shape check first.
        if indices.len() < 8 {
            return None;
        }

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        for &i in indices {
            x_min = x_min.min(all_spans[i].bbox.left());
            x_max = x_max.max(all_spans[i].bbox.right());
        }
        let region_width = x_max - x_min;
        if region_width < 200.0 {
            // Real two-column bodies span at least ~200pt (the
            // narrowest two-column layout in the corpus is ~250pt for a
            // letter-page body inside ~250pt margins).
            return None;
        }

        // Cluster spans into lines by rounded Y. Keep PER-SPAN
        // (left, right) data so we can detect within-line gaps —
        // the canonical multi-column interleave on issue_07 puts
        // a left-col span (left=82) and a right-col span (left=312)
        // on the same Y baseline. The whole-line bbox.right -
        // bbox.left = 358 pt looks "wide" (358 > 0.6 × 500 = 300)
        // even though each side is a narrow column half.
        let mut lines_spans: std::collections::BTreeMap<i32, Vec<(f32, f32)>> =
            std::collections::BTreeMap::new();
        for &i in indices {
            let s = &all_spans[i];
            let y_key = s.bbox.top().round() as i32;
            lines_spans
                .entry(y_key)
                .or_default()
                .push((s.bbox.left(), s.bbox.right()));
        }
        if lines_spans.len() < 6 {
            return None;
        }

        // For each line, find the largest gap between adjacent spans.
        // A line is treated as multiple "half-lines" if a gap ≥ 10 pt
        // splits it; each side of the gap contributes its leftmost-x
        // to `narrow_lefts`. This is the lesson: the row-by-
        // row interleave shape on issue_07 spans the gutter as bbox
        // but has a clear gap within each line.
        let narrow_threshold = region_width * 0.6;
        let intra_line_gap_threshold = 10.0_f32;
        let mut narrow_lefts: Vec<f32> = Vec::new();
        // Count "narrow" lines for the majority check — a line with
        // a within-line gap contributes 1 to this count regardless of
        // how many half-lines it produces, so the majority threshold
        // stays comparable to single-column reasoning.
        let mut narrow_line_count = 0usize;
        for line_spans in lines_spans.values() {
            let mut sorted = line_spans.clone();
            sorted.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
            // Detect largest within-line gap.
            let mut largest_gap = 0.0_f32;
            let mut split_idx: Option<usize> = None;
            for (i, w) in sorted.windows(2).enumerate() {
                let gap = w[1].0 - w[0].1;
                if gap > largest_gap {
                    largest_gap = gap;
                    split_idx = Some(i);
                }
            }
            let line_left = sorted.first().map(|(l, _)| *l).unwrap_or(0.0);
            let line_right = sorted.last().map(|(_, r)| *r).unwrap_or(0.0);
            let line_extent = (line_right - line_left).max(0.0);

            if let Some(si) = split_idx {
                if largest_gap >= intra_line_gap_threshold {
                    // Within-line gap detected — treat each side as
                    // its own narrow half-line.
                    narrow_lefts.push(line_left);
                    // The right-side starts at sorted[si + 1].0
                    if let Some(&(right_side_left, _)) = sorted.get(si + 1) {
                        narrow_lefts.push(right_side_left);
                    }
                    narrow_line_count += 1;
                    continue;
                }
            }

            if line_extent < narrow_threshold {
                narrow_lefts.push(line_left);
                narrow_line_count += 1;
            }
        }
        // Majority of lines must be narrow — otherwise this isn't a
        // 2-column body, it's a single-column body with a few short
        // last-lines.
        if narrow_line_count * 2 < lines_spans.len() {
            return None;
        }

        // Cluster the narrow left-edges. Two clusters separated by
        // approximately half the region width = 2-column prose.
        let cluster_radius = 30.0_f32;
        let mut clusters: Vec<(f32, usize)> = Vec::new();
        for &x in &narrow_lefts {
            if let Some(c) = clusters
                .iter_mut()
                .find(|(c, _)| (*c - x).abs() <= cluster_radius)
            {
                // Running mean
                let count = c.1 as f32;
                c.0 = (c.0 * count + x) / (count + 1.0);
                c.1 += 1;
            } else {
                clusters.push((x, 1));
            }
        }

        // Want exactly 2 substantial clusters separated by ~half-width.
        // ≥ 3 clusters = either a table or a band-mixed region — bail.
        if clusters.len() != 2 {
            return None;
        }
        // Sort by x.
        clusters.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
        let (c1_x, c1_n) = clusters[0];
        let (c2_x, c2_n) = clusters[1];

        // Each cluster needs substantial coverage — ≥ 3 lines, or 20 %
        // of the line count, whichever is larger. Reject lopsided
        // shapes (header + body-paragraph).
        let min_cluster = 3usize.max(narrow_lefts.len() / 5);
        if c1_n < min_cluster || c2_n < min_cluster {
            return None;
        }

        // Gap between cluster centres ≥ 30 % of region width (the
        // gutter + right-column left-margin). For a tight gutter of
        // ~12pt with two ~250pt columns the gap is ~250pt out of 512pt
        // → ~49 %, well above the floor.
        let gap = c2_x - c1_x;
        if gap < region_width * 0.30 {
            return None;
        }

        // Positive identification of prose — required by the
        // classifier to avoid the google_doc 2-col table
        // sub-region false positive.
        if region_kind != RegionKind::Prose {
            return None;
        }

        // Gutter midpoint as the cut. The cluster centres are the left
        // edges of the two columns; the gutter sits between the right
        // edge of column 1 and the left edge of column 2. We don't
        // track right edges per cluster, so approximate the gutter
        // centre as halfway between the two cluster centres — that's
        // close enough; the actual partition uses `bbox.left()` per
        // span so individual spans land cleanly on either side.
        let gutter_x = (c1_x + c2_x) * 0.5;
        Some(gutter_x)
    }

    /// Second-pass 2-column-prose detector for the narrow-gutter case
    /// that `detect_two_column_prose` (the line-start-cluster detector)
    /// misses.
    ///
    /// Two-column papers that emit body text at character-cluster
    /// granularity (each glyph its own span) confuse the line-start
    /// detector: titles, captions, and equation labels contribute
    /// outlier singleton clusters in addition to the two body
    /// columns, so the `clusters.len() != 2` gate rejects. Their
    /// gutters are also often narrower than `min_valley_width` so
    /// the primary projection-valley path in
    /// `find_horizontal_split_indexed` rejects as well.
    ///
    /// Distinguishing signal that works regardless of outlier rows:
    /// the **largest within-line gap** on each body line lives at
    /// roughly the same X coordinate (the gutter) across a strong
    /// majority of lines. Cluster those gap positions; if one cluster
    /// covers ≥ 60 % of the body lines AND the region classifies as
    /// `Prose`, the page is two-column prose and the cluster centre
    /// is the gutter X.
    ///
    /// Returns the gutter X coordinate (an actual gap position, not
    /// a midpoint estimate) when the pattern is detected.
    ///
    /// The Prose-classifier gate keeps tables out: table rows have
    /// their largest gap at variable X across rows (different cell
    /// widths), so the gap-position cluster never dominates.
    fn detect_narrow_gutter_prose(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
        region_kind: RegionKind,
    ) -> Option<f32> {
        if indices.len() < 24 {
            return None;
        }
        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        for &i in indices {
            x_min = x_min.min(all_spans[i].bbox.left());
            x_max = x_max.max(all_spans[i].bbox.right());
        }
        let region_width = x_max - x_min;
        if region_width < 200.0 {
            return None;
        }

        // Cluster spans into lines by rounded Y.
        let mut lines: std::collections::BTreeMap<i32, Vec<(f32, f32)>> =
            std::collections::BTreeMap::new();
        for &i in indices {
            let s = &all_spans[i];
            let y_key = s.bbox.top().round() as i32;
            lines
                .entry(y_key)
                .or_default()
                .push((s.bbox.left(), s.bbox.right()));
        }
        if lines.len() < 12 {
            return None;
        }

        // For each line, find the largest within-line gap (≥ 6 pt
        // suppresses ordinary word-spacing of 2–5 pt). Record the gap's
        // midpoint X.
        const MIN_GAP_PT: f32 = 6.0;
        let mut gap_positions: Vec<f32> = Vec::new();
        for line_spans in lines.values() {
            if line_spans.len() < 2 {
                continue;
            }
            let mut sorted = line_spans.clone();
            sorted.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
            let mut largest_gap = 0.0_f32;
            let mut largest_mid = 0.0_f32;
            for w in sorted.windows(2) {
                let gap = w[1].0 - w[0].1;
                if gap > largest_gap {
                    largest_gap = gap;
                    largest_mid = (w[0].1 + w[1].0) * 0.5;
                }
            }
            if largest_gap >= MIN_GAP_PT {
                gap_positions.push(largest_mid);
            }
        }

        // Need at least 12 gap-bearing lines to cluster — fewer is
        // statistical noise.
        if gap_positions.len() < 12 {
            return None;
        }

        // Cluster the gap positions with a 10 pt radius (tight; the
        // gutter is at one specific X with minor line-to-line drift).
        // Sliding-window two-pointer scan over the sorted positions —
        // both `left` and `right` only advance forward, so total
        // work is O(n) instead of the previous O(n²) pivot scan
        // (thesis-style PDFs with hundreds of gap-bearing rows pay
        // visibly in that nested loop).
        const CLUSTER_RADIUS_PT: f32 = 10.0;
        let mut sorted_gaps = gap_positions.clone();
        sorted_gaps.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        // Prefix sums let us read window-sum in O(1) given (left, right).
        let mut prefix: Vec<f32> = Vec::with_capacity(sorted_gaps.len() + 1);
        prefix.push(0.0);
        for &x in &sorted_gaps {
            prefix.push(prefix.last().unwrap() + x);
        }
        let mut best_size = 0usize;
        let mut best_center = 0.0_f32;
        let mut left = 0usize;
        let mut right = 0usize;
        for &pivot in &sorted_gaps {
            while left < sorted_gaps.len() && sorted_gaps[left] < pivot - CLUSTER_RADIUS_PT {
                left += 1;
            }
            while right < sorted_gaps.len() && sorted_gaps[right] <= pivot + CLUSTER_RADIUS_PT {
                right += 1;
            }
            let count = right - left;
            let sum = prefix[right] - prefix[left];
            if count > best_size {
                best_size = count;
                best_center = sum / count as f32;
            }
        }

        // Concentration: ≥ 70 % of gap-bearing lines cluster at the
        // same X. Distinguishes 2-col prose (one gutter) from
        // tables (gaps at several cell boundaries, lower
        // concentration).
        if best_size * 10 < gap_positions.len() * 7 {
            return None;
        }
        if best_size < 12 {
            return None;
        }
        if best_size * 5 < lines.len() {
            return None;
        }

        // Sanity: the gutter must lie comfortably inside the region.
        let gutter_offset = best_center - x_min;
        if gutter_offset < region_width * 0.2 || gutter_offset > region_width * 0.8 {
            return None;
        }

        // Prose gate — same safety as `detect_two_column_prose`.
        // Tables with narrow cell gaps fail the classifier
        // (`mean_chars < 8` → `Table`), preventing the gap-cluster
        // signal from misfiring on tabular content. Short-verse
        // two-column bodies (#536) now also pass this gate: although
        // their `mean_chars <= 20`, `classify_region_kind`'s short-line
        // central-corridor admission arm returns `Prose` for them, so a
        // routed short-verse body is cut here rather than re-collapsed.
        //
        if region_kind != RegionKind::Prose {
            return None;
        }

        Some(best_center)
    }

    /// Heuristic: does the region look like a single column of body text?
    ///
    /// Called **before** horizontal split attempts. When true, the region
    /// is returned as a single sorted group, bypassing both horizontal
    /// (column) and vertical (row) splits. This prevents XY-Cut from
    /// fragmenting body text at density dips caused by indentation or
    /// short last-lines.
    ///
    /// Detection: cluster spans into lines by rounded top-Y, then count
    /// lines that are both **wide** (extent ≥ 60% region width) and
    /// **dense** (covered ratio ≥ 80%). Body-text lines satisfy both.
    /// Aligned multi-column rows look "wide" because their extent spans
    /// the gutter, but fail the density check because the gutter is empty.
    fn is_single_column_region(&self, all_spans: &[TextSpan], indices: &[usize]) -> bool {
        if indices.len() < 3 {
            return false;
        }
        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        for &i in indices {
            x_min = x_min.min(all_spans[i].bbox.left());
            x_max = x_max.max(all_spans[i].bbox.right());
        }
        let region_width = x_max - x_min;
        if region_width <= 10.0 {
            return true;
        }

        // Store both bbox.right and core_right for each span. bbox.right
        // can be over-estimated by extractors (trailing whitespace,
        // stretched advance widths) which makes multi-column lines look
        // like one wide continuous run; core_right (char_count × em) is
        // a conservative fallback used ONLY when adjacent bbox edges
        // overlap (a signal of bbox inflation).
        //
        let mut lines: std::collections::BTreeMap<i32, Vec<(f32, f32, f32)>> =
            std::collections::BTreeMap::new();
        for &i in indices {
            let s = &all_spans[i];
            let y_key = s.bbox.top().round() as i32;
            let char_count = s.text.chars().filter(|c| !c.is_whitespace()).count().max(1) as f32;
            let approx_char_width = (s.font_size * 0.45).max(2.5);
            let core_right = s.bbox.left() + char_count * approx_char_width;
            lines
                .entry(y_key)
                .or_default()
                .push((s.bbox.left(), s.bbox.right(), core_right));
        }
        if lines.len() < 3 {
            return false;
        }

        // A real column gutter recurs at roughly the SAME X position
        // across multiple lines. Sparse title-page layouts (Title /
        // Subtitle / Byline) also have wide inter-word gaps, but their
        // gap positions are scattered — not a gutter. Collect all gap
        // positions (mid-gap X), then check whether a consistent cluster
        // of gap positions appears on ≥30% of lines.
        //
        // Gap uses bbox.right, but if adjacent bboxes OVERLAP (classic
        // signature of extractor-inflated bbox widths), re-check with
        // conservative core_right estimates so column detection is not
        // defeated by trailing whitespace inflation.
        let max_gap = self.min_valley_width;
        let mut gap_positions: Vec<f32> = Vec::new();
        for line_spans in lines.values() {
            let mut sorted = line_spans.clone();
            sorted.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
            for w in sorted.windows(2) {
                let bbox_gap = w[1].0 - w[0].1;
                let (effective_gap, gap_end_left) = if bbox_gap < 0.0 {
                    (w[1].0 - w[0].2, w[0].2)
                } else {
                    (bbox_gap, w[0].1)
                };
                if effective_gap >= max_gap {
                    gap_positions.push((gap_end_left + w[1].0) * 0.5);
                }
            }
        }
        // Centered-block guard: a CENTERED title/subtitle/
        // byline block (each line horizontally centered, varying widths)
        // produces accidental gap clusters that look like a column
        // gutter — but it is NOT columnar, and treating it as columns
        // scrambles reading order ("Quarterly Inventory Review" centered
        // title read as 3 columns → "Quarterly" / "Spring" / ... ).
        //
        // The distinguishing signal: a REAL multi-column layout has the
        // left column starting at a consistent left edge across rows
        // (low variance of per-line leftmost x). Centered text has its
        // leftmost x scattered (each line centered with a different
        // width). Compute the spread of per-line leftmost edges; if it
        // is large relative to the region width, the block is centered,
        // not columnar, so do NOT treat the gap cluster as a gutter.
        // Centered iff the per-line leftmost edges do NOT share a common
        // left margin. A left-aligned layout (single column OR real
        // multi-column) has most rows starting at the same x (the left
        // margin), so the largest cluster of leftmost edges covers a
        // majority of lines. Centered text has each line's leftmost edge
        // scattered (different per line), so no cluster dominates.
        //
        // Using a cluster fraction (not raw spread) is robust to rows
        // that only contain right-column content — those push the spread
        // up but do not change the fact that the left margin still
        // dominates the remaining rows. (Raw spread mis-classified the
        // two-column test where the last row held only a right cell.)
        let looks_centered = {
            let mins: Vec<f32> = lines
                .values()
                .map(|ls| ls.iter().map(|(l, _, _)| *l).fold(f32::MAX, f32::min))
                .collect();
            if mins.len() < 2 {
                false
            } else {
                let tol = 10.0_f32;
                // Largest count of leftmost-edges within ±tol of any single edge.
                // Sort once + binary-search the window instead of the O(k^2)
                // all-pairs scan; the max count is a multiset property so this is
                // identical to the pairwise version.
                let largest = {
                    let mut sorted = mins.clone();
                    sorted.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
                    sorted
                        .iter()
                        .map(|&a| {
                            let lo = sorted.partition_point(|&x| x < a - tol);
                            let hi = sorted.partition_point(|&x| x <= a + tol);
                            hi - lo
                        })
                        .max()
                        .unwrap_or(0)
                };
                // Centered when no left-margin cluster covers a majority.
                (largest as f32) < (mins.len() as f32) * 0.5
            }
        };

        // A SMALL centered block (title / subtitle / byline — few lines,
        // scattered leftmost edges) is treated as a single column so its
        // lines stay in top-to-bottom order and a centered multi-word
        // title is not split into per-word "columns". Gated
        // to <= 6 lines so it only catches title-page-style blocks: a
        // real multi-column body has many lines and is never classified
        // centered here (its left column starts at a consistent margin,
        // giving a small leftmost-spread anyway).
        if looks_centered && lines.len() <= 6 {
            return true;
        }

        // Cluster gap positions: count, for each observed gap, how many
        // other gaps fall within ±20pt. If any cluster contains gaps
        // from ≥30% of lines, it's a genuine column gutter.
        if !gap_positions.is_empty() && !looks_centered {
            let cluster_radius = 20.0_f32;
            // Require ≥3 gap positions (or 20% of lines, whichever is
            // larger) clustered within ±20pt. 20% accommodates pages
            // where header/footer/title rows dilute the body-line count
            // but a real multi-column body still dominates.
            let min_cluster = (3usize).max(lines.len() / 5);
            // Sort once + binary-search each gap's ±radius window instead of the
            // O(k^2) all-pairs scan. Returns false iff some gap's window holds
            // >= min_cluster gaps — identical to the pairwise version.
            let mut sorted_gaps = gap_positions.clone();
            sorted_gaps.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
            for &pos in &sorted_gaps {
                let lo = sorted_gaps.partition_point(|&p| p < pos - cluster_radius);
                let hi = sorted_gaps.partition_point(|&p| p <= pos + cluster_radius);
                if hi - lo >= min_cluster {
                    return false;
                }
            }
        }

        // With no column gutter found on any line, check that the majority
        // of lines are wide AND densely covered. This catches clean body
        // text where every line covers most of the region width.
        let width_threshold = region_width * 0.6;
        let mut wide_dense_lines = 0usize;
        for line_spans in lines.values() {
            let mut sorted = line_spans.clone();
            sorted.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));
            let extent_left = sorted.first().unwrap().0;
            let extent_right = sorted.iter().map(|(_, r, _)| *r).fold(f32::MIN, f32::max);
            let extent = extent_right - extent_left;
            if extent < width_threshold {
                continue;
            }
            // Use core_right (char-count estimate) rather than bbox.right
            // for coverage. bbox.right is inflated by tab characters and
            // trailing whitespace — tab-expanded table rows would otherwise
            // score 100% coverage and be misidentified as dense body text.
            let mut covered = 0.0f32;
            let mut last_end = f32::MIN;
            for &(l, _, cr) in &sorted {
                let effective_right = cr.min(extent_right);
                let start = l.max(last_end);
                if effective_right > start {
                    covered += effective_right - start;
                    last_end = effective_right;
                }
            }
            if covered >= extent * 0.8 {
                wide_dense_lines += 1;
            }
        }
        wide_dense_lines * 2 >= lines.len()
    }

    /// Find vertical line (X-axis) split using index-based partitioning.
    ///
    /// Rejects lopsided splits where one side contains fewer than ~10% of
    /// the region's spans — those come from single-column pages where
    /// indentation or stray content creates a spurious density dip at
    /// one edge of the projection, not from a real column boundary.
    fn find_horizontal_split_indexed(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
    ) -> Option<(Vec<usize>, Vec<usize>)> {
        let profile = self.horizontal_projection_indexed(all_spans, indices)?;

        // The corridor the cut is taken through: the profile's valley where
        // one was found, or a band the width of the valley floor around a
        // trough between two peaks.
        let (split_x, corridor) = if let Some((vs, ve, vw)) = self.find_valley(&profile) {
            if vw < self.min_valley_width {
                return None;
            }
            let (lo, hi) = (profile.x_min + vs as f32, profile.x_min + ve as f32);
            ((lo + hi) / 2.0, (lo, hi))
        } else {
            let x = self.find_split_between_peaks(&profile)?;
            let half = self.min_valley_width / 2.0;
            (x, (x - half, x + half))
        };

        // Reject splits where either resulting sub-column would be
        // narrower than ~60 pt (about 6 body-text characters at
        // 10 pt). Without this check, XY-cut recursion sub-splits
        // a single body column into sliver sub-blocks at internal
        // whitespace valleys (paragraph indentation, justified-line
        // trailing gaps, isolated short words), turning what should
        // be a clean column-major emit of a multi-column page into
        // a band-chunked stream. PDF spec §9.4.4 mentions "natural
        // reading order" but does not mandate a
        // minimum column width; this is a descriptive heuristic —
        // a real body column holds at least ~6 characters.
        const MIN_RESULT_WIDTH_PT: f32 = 60.0;
        let mut left_x_min = f32::MAX;
        let mut left_x_max = f32::MIN;
        let mut right_x_min = f32::MAX;
        let mut right_x_max = f32::MIN;
        for &i in indices {
            let l = all_spans[i].bbox.left();
            let r = all_spans[i].bbox.right();
            if l < split_x {
                left_x_min = left_x_min.min(l);
                left_x_max = left_x_max.max(r);
            } else {
                right_x_min = right_x_min.min(l);
                right_x_max = right_x_max.max(r);
            }
        }
        let left_w = left_x_max - left_x_min;
        let right_w = right_x_max - right_x_min;
        if left_w < MIN_RESULT_WIDTH_PT || right_w < MIN_RESULT_WIDTH_PT {
            return None;
        }

        // A span the projection never counted leaves no ink in the density
        // array, so a "valley" can be an artefact of the exclusion rather
        // than a real corridor. `horizontal_projection_indexed` drops every
        // span wider than 55% of the region so that a banner headline does
        // not hide the columns beneath it. That is right — but only when the
        // banner is peeled off first. Cutting straight through a run that
        // physically crosses the corridor splits one printed line between two
        // groups and emits its halves far apart.
        //
        // ISO 32000-1:2008 §9.4.4 computes the glyph displacement along the
        // writing axis and sets the other axis's component to 0, so a
        // horizontal run occupies one unbroken interval in X. If that
        // interval covers the candidate corridor on both sides, the corridor
        // is not empty and there is no column boundary here. Refuse the cut;
        // the recursion falls back to a row split, which is what peels a
        // banner off the columns underneath it.
        //
        // The measurement uses the same core-width estimate as the
        // projection (character count x ~0.45 em, clamped to `bbox.right()`)
        // rather than the raw bbox, because extractor bboxes overreach to the
        // right on trailing whitespace and stretched advances. `STRADDLE_TOL`
        // keeps a one-glyph overhang from counting as a crossing.
        const STRADDLE_TOL: f32 = 10.0;
        let region_width = (left_x_max.max(right_x_max) - left_x_min.min(right_x_min)).max(1.0);
        let crosses: Vec<bool> = indices
            .iter()
            .map(|&i| {
                let s = &all_spans[i];
                let (l, r) = (s.bbox.left(), s.bbox.right());
                // Only spans the projection skipped can hide ink from it.
                if r - l <= region_width * 0.55 {
                    return false;
                }
                let chars = s.text.chars().filter(|c| !c.is_whitespace()).count().max(1) as f32;
                let core_right = (l + chars * (s.font_size * 0.45).max(2.5)).min(r);
                l < split_x - STRADDLE_TOL && core_right > split_x + STRADDLE_TOL
            })
            .collect();
        let crossing_count = crosses.iter().filter(|c| **c).count();
        // A crossing run alone is not enough to refuse the cut. A banner
        // headline sitting *above* two real columns crosses every candidate
        // corridor between them, and refusing there costs the column split on
        // ordinary two-column prose — which then reads row-major and glues a
        // hyphenated word to the facing column's continuation.
        //
        // Exactly one benign shape exists: a single banner over two columns
        // that run together down the page. Both halves of that matter.
        //
        // A corridor crossed again and again is not a corridor. A contents
        // page whose every subchapter title spans the measure crossed one
        // candidate 22 times; two columns of prose under one heading crossed
        // theirs once.
        //
        // And a real column runs most of the region's height, so two of them
        // are near-coextensive: the prose page's halves spanned 18..635 and
        // 20..635 of a 617 pt region. The bands this guard exists for do not —
        // a newspaper masthead's side holds a nameplate and a dateline over
        // 28% of the region, and a contents page's page-number column is a
        // short band nested inside the full-measure titles beside it.
        //
        // A single banner is measured as furniture, not as a member of the
        // column its left edge happens to land in. It is bucketed by that
        // edge like everything else, so its own height counted as that
        // column's and stretched the region both sides are scored against —
        // backwards for the one shape this allowance exists for. A banner
        // printed 35 pt above two 112 pt columns of equal height put the left
        // side at 152 pt of a 159 pt region and the right at 112, and the two
        // columns scored 70 % against each other when they are the same
        // height. The columns are what has to be near-coextensive.
        //
        // Only for a single crossing. Where a corridor is crossed again and
        // again the crossing runs ARE the page's content — a contents page's
        // full-measure titles are its column — so those keep their height.
        // A side that is nothing but the banner is not a column either.
        let banner_is_furniture = crossing_count == 1;
        let mut left_lo = f32::MAX;
        let mut left_hi = f32::MIN;
        let mut right_lo = f32::MAX;
        let mut right_hi = f32::MIN;
        let (mut left_n, mut right_n) = (0usize, 0usize);
        // The vertical extent of what each side holds BESIDES the crossing
        // runs, for the test below.
        let mut rows_left = (f32::MAX, f32::MIN);
        let mut rows_right = (f32::MAX, f32::MIN);
        for (&i, &crossing) in indices.iter().zip(crosses.iter()) {
            let b = &all_spans[i].bbox;
            let (lo, hi) = (b.y, b.y + b.height.abs());
            if !crossing {
                let side = if b.left() < split_x {
                    &mut rows_left
                } else {
                    &mut rows_right
                };
                side.0 = side.0.min(lo);
                side.1 = side.1.max(hi);
            }
            if banner_is_furniture && crossing {
                continue;
            }
            if b.left() < split_x {
                left_lo = left_lo.min(lo);
                left_hi = left_hi.max(hi);
                left_n += 1;
            } else {
                right_lo = right_lo.min(lo);
                right_hi = right_hi.max(hi);
                right_n += 1;
            }
        }
        // A corridor is a gutter only where the rows on either side of it
        // leave it empty. The profile's valley is where the density falls
        // under a fraction of the peak, and that is right for finding a
        // gutter beside a dense column — a stray stub or a folio in the
        // gutter must not hide it. But where one side of a region is much
        // denser than the other, a band holding a row or two of ordinary
        // words scores as a valley too, and the valley reaches into the
        // ragged line ends of the column beside it.
        //
        // Measured on the page that exposed it: a paragraph set one run per
        // word (the justifier's word gaps exceed the merge threshold) beneath
        // a letter-spaced listing and above the page's footnotes. The listing
        // and footnote rows made the left mass dense, and everything right of
        // x≈165 fell under the threshold: the valley ran 165..236, seventy
        // points wide, with `is`, `no` and `file` inside it on the
        // paragraph's own rows. Its centre landed in the word gap after `no`,
        // the full lines around it crossed the corridor, the two halves
        // measured 0.79 of the region — a column by the bar below — and the
        // cut was taken: `pro-` was emitted three lines from `ceeds`, and
        // `proceeds` left the page.
        //
        // What tells that corridor from a gutter is the rows. On every row
        // the corridor divides — a row with runs on both sides of it — the
        // corridor held letters: the paragraph's short words on its rows,
        // the listing's spaced glyphs on the rest. On the page's real gutter,
        // measured the same way, two of forty such rows hold letters in the
        // valley (a line end the valley's fringe reaches, a listing fragment)
        // and the rest hold nothing. A title set one run per word across the
        // head of a two-column paper puts words in its gutter on one row;
        // the thirty rows of column beneath it put nothing there. So the cut
        // is refused when the rows the corridor divides hold letters in it
        // on at least half of them, and on at least two. Digits alone do not
        // count: a folio or a verse number centred between two columns is
        // furniture in the gutter, not text across it.
        //
        // ISO 32000-1:2008 §9.4.4 (docs/spec/pdf.md:17396): a horizontal
        // run occupies one unbroken interval on the writing axis, so a run
        // of letters whose interval lies inside the corridor is proof there
        // is text there and no column boundary on that row.
        //
        // A row is divided by the corridor only when both its sides carry
        // letters, and the letters inside the corridor count only when they
        // continue the row's own text — a word space or less after the run
        // to their left. A chart beside a paragraph shares its rows and puts
        // an axis title in the corridor, but that title sits three or four
        // ems from the paragraph's line end; a column of tick numerals is
        // not the other half of a paragraph's lines at all. The paragraph
        // that exposed this has `no` five points after `is` and `file` ten
        // points after `no`: the corridor runs through the middle of a line.
        let band = |y: f32| (y / crate::utils::ROW_BAND_TOLERANCE_PT).round() as i32;
        // Per row: (left, ink right, has letters, font size), non-crossing.
        let mut rows: std::collections::BTreeMap<i32, Vec<(f32, f32, bool, f32)>> =
            std::collections::BTreeMap::new();
        for (&i, &crossing) in indices.iter().zip(crosses.iter()) {
            if crossing {
                continue;
            }
            let s = &all_spans[i];
            let letters = s.text.chars().any(|c| c.is_alphabetic());
            let chars = s.text.chars().filter(|c| !c.is_whitespace()).count().max(1) as f32;
            let ink_right =
                (s.bbox.left() + chars * (s.font_size * 0.45).max(2.5)).min(s.bbox.right());
            rows.entry(band(s.bbox.y)).or_default().push((
                s.bbox.left(),
                ink_right,
                letters,
                s.font_size,
            ));
        }
        let (mut divided, mut divided_with_text) = (0usize, 0usize);
        for runs in rows.values() {
            let letters_left = runs.iter().any(|r| r.2 && r.0 < split_x);
            let letters_right = runs.iter().any(|r| r.2 && r.0 >= split_x);
            if !(letters_left && letters_right) {
                continue;
            }
            divided += 1;
            let continues_the_row = runs.iter().any(|r| {
                if !r.2 || r.0 < corridor.0 || r.1 > corridor.1 {
                    return false;
                }
                let word_space = 1.5 * r.3;
                runs.iter()
                    .any(|l| l.1 <= r.0 + 0.5 && r.0 - l.1 <= word_space && l.0 < r.0)
            });
            if continues_the_row {
                divided_with_text += 1;
            }
        }
        const ROWS_THAT_FILL_A_CORRIDOR: usize = 2;
        let corridor_holds_text =
            divided_with_text >= ROWS_THAT_FILL_A_CORRIDOR && divided_with_text * 2 >= divided;
        if corridor_holds_text {
            return None;
        }
        let region_height = (left_hi.max(right_hi) - left_lo.min(right_lo)).max(1.0);
        let shorter_side = (left_hi - left_lo).min(right_hi - right_lo);
        // A band that stops far short of the region is not a column. What
        // counts as "far short" cannot be a fifth, though: columns of running
        // text are only near-coextensive when nothing interrupts them. On a
        // news page a photograph, an advertisement or the end of the story
        // stops one column well above the other, and the two sides are then
        // genuinely uneven — one measured pair ran 749.6 pt against 597.6 pt
        // of a 755.6 pt region, a 0.79 ratio, while a second page's columns
        // sat at the same 0.79. Both are ordinary three-column news pages
        // whose columns this guard refused, taking no cut at all and leaving
        // the columns to be read across the page.
        //
        // The shapes the guard exists to refuse are far shorter than that: a
        // masthead's side, holding only a nameplate and a dateline, covers
        // 28% of the region, and a contents page's page-number column is a
        // short band nested inside the full-measure titles beside it.
        //
        // Nothing observed falls between those and a real column, and the
        // observed columns spread wider than any near-coextensive reading
        // admits — 0.746 and 0.480 on a two-column typescript, 0.79 on two
        // news pages. So the line is what separates a column from a band
        // rather than what separates a column from its neighbour: a column
        // covers at least half the region it is a column of; a run of
        // furniture beside one does not.
        //
        // Excluding the crossing runs from the two sides' extents was tried
        // instead, on the theory that several banners over real columns are
        // still furniture. It moves the run counts and not the extents — the
        // crossing runs are not what reaches furthest on either side — and
        // left every affected page exactly where it was.
        const COLUMN_HEIGHT_FRACTION: f32 = 0.5;
        let sides_are_columns =
            left_n > 0 && right_n > 0 && shorter_side >= region_height * COLUMN_HEIGHT_FRACTION;
        if crossing_count > 0 && !sides_are_columns {
            return None;
        }

        // Partition by span LEFT EDGE (where the glyphs actually start),
        // not bbox.right() and not center. Extractor bboxes overreach to
        // the right (trailing whitespace / stretched advance widths), and
        // for wide single-column body spans the center can also drift
        // past the split. Left edge is anchored to the true glyph start
        // and reliably places each span into its actual column.
        let (left, right): (Vec<usize>, Vec<usize>) = indices
            .iter()
            .partition(|&&i| all_spans[i].bbox.left() < split_x);

        if left.is_empty() || right.is_empty() {
            return None;
        }

        // Real column splits produce balanced partitions. A 95/5 split is
        // almost always from edge dips or stray content, not a column.
        let min_side = (indices.len() / 10).max(2);
        if left.len() < min_side || right.len() < min_side {
            return None;
        }

        // Table-row guard (#7 / PMC8025747). A genuine column gutter is a
        // vertical CORRIDOR: the left column's glyphs END before the gutter
        // and the right column's glyphs BEGIN after it, so the two sides are
        // X-disjoint. A data-table row, by contrast, starts at the left
        // margin but its cells run the FULL width of the region; partitioning
        // such rows by left edge throws the wide rows into `left` while the
        // right-hand cells (their own spans) land in `right`. Taking the cut
        // anyway slices the table's rows into shattered left/right cell groups
        // — the canonical PMC8025747 p2 failure (a prose column stacked above
        // a full-width data table), and the google_doc population-table hazard
        // the post-mortem at lines 73–101 records.
        //
        // Table-row SIGNATURE: SEVERAL left-side rows each span the ENTIRE
        // right column — their glyph content reaches past the right column's
        // far edge (`right_x_max`). A data table has MANY full-width rows (the
        // header and every data row run the whole region width), so when rows
        // are bucketed into `left` by their left edge, multiple of them blanket
        // the whole right column. By contrast:
        //   * a genuine left prose / reference column ENDS before the gutter,
        //     so its lines stop well short of `right_x_max` (never counted);
        //   * a single wide mis-split OCR line (alice) yields at most one or
        //     two straddling spans — never the recurring full-width pattern;
        //   * the single-column google_doc population table short-circuits at
        //     `is_single_column_region` and never reaches here.
        // Requiring ≥ 3 such rows is what isolates the real table from those
        // cases, so the guard is SUBTRACTIVE: it only ever REJECTS a column
        // cut that would shred a table (the recursion then falls back to a row
        // cut and reads the table row-major), never adds or reorders anything.
        //
        // `core_right` (left edge + char count × ~0.5 em, capped at
        // `bbox.right`) is used instead of `bbox.right` alone so
        // trailing-whitespace / advance-width bbox inflation on a real left
        // column's last word is not mistaken for a glyph crossing the gutter.
        // `overlap_tol` (~ one body em) lets a single straddling glyph slip
        // past.
        let mut right_x_max = f32::MIN;
        let mut max_font = 0.0f32;
        for &i in &right {
            right_x_max = right_x_max.max(all_spans[i].bbox.right());
            max_font = max_font.max(all_spans[i].bbox.height.abs());
        }
        let overlap_tol = max_font.max(10.0);
        // A row that blankets the right column is only evidence of a TABLE if
        // the right column has something on that row. A table row does — its
        // right-hand cells are their own spans at the same y. A full-measure
        // BAND does not: a title, heading, abstract or caption spanning the
        // measure is alone on its row, and the spans bucketed into `right` sit
        // above or below it.
        //
        // Without this the guard mistakes ordinary two-column furniture for a
        // table. A journal page's title, authors, abstract, running head and
        // captions all blanket the right column, three of them are enough to
        // refuse the cut, and the body then reads as one leaf spanning the
        // page — every physical row emitted as one line, splicing the columns
        // into each other.
        //
        // A row's other cells need not land in `right` to exist. Where the
        // split falls to the RIGHT of a row's first cell boundary — a contents
        // page's section numbers, a schedule's label column — the row's own
        // stub is bucketed into `left` beside the full-measure run it labels,
        // and the run is the rest of the row. Reading only `right` for company
        // makes the guard blind to exactly the rows it exists to protect: a
        // contents page's `10.3.1` rail was cut away from its titles and the
        // titles re-emitted a column-run later with no numbers on them.
        //
        // So a blanketing run is furniture only when it is ALONE on its row:
        // nothing in `right` shares its band, AND nothing on that band is
        // printed to its left. A title, abstract or caption spanning the
        // measure satisfies both — it is the first and only thing on its row.
        // A labelled row satisfies neither.
        let band_of =
            |i: usize| (all_spans[i].bbox.y / crate::utils::ROW_BAND_TOLERANCE_PT).round() as i32;
        let shares_a_row_with_right = |i: usize| -> bool {
            let a = band_of(i);
            right.iter().any(|&j| band_of(j) == a)
        };
        let is_labelled_from_the_left = |i: usize| -> bool {
            let a = band_of(i);
            let x = all_spans[i].bbox.left();
            left.iter()
                .any(|&j| j != i && band_of(j) == a && all_spans[j].bbox.left() < x)
        };
        let full_width_left_rows = left
            .iter()
            .filter(|&&i| {
                if !shares_a_row_with_right(i) && !is_labelled_from_the_left(i) {
                    return false;
                }
                let s = &all_spans[i];
                // Count EVERY character, not just the non-whitespace ones. An
                // inter-word space consumes an advance exactly as a glyph does,
                // so excluding spaces under-measures justified prose by about a
                // ninth of its width — enough that a full-measure body line
                // reaching the right margin scored short of it and was not
                // counted. Clamping to `bbox.right()` keeps the estimate from
                // over-reaching instead: the result is never wider than the ink
                // actually is, which is what the trailing-whitespace concern
                // above is really about.
                let chars = s.text.chars().count().max(1) as f32;
                let approx_char_width = (s.font_size * 0.45).max(2.5);
                let core_right = (s.bbox.left() + chars * approx_char_width).min(s.bbox.right());
                core_right >= right_x_max - overlap_tol
            })
            .count();
        // The count has to be read against the side it came from, not on its
        // own: three rows out of a hundred is a heading, a footnote and a
        // caption over two columns of prose, and vetoing there costs those
        // columns their cut and leaves them read across the page.
        //
        // The threshold is bounded on both sides by measurement, and the window
        // is narrow:
        //
        //   3 of 100 left rows (0.03) — a two-column page carrying a heading,
        //     a footnote and a caption. Must NOT veto.
        //   6 of 49 left rows (0.122) — a contents page whose shallower entries
        //     run from title through dot leader to page number, reaching across
        //     the channel the deeper entries' indentation opens. Must veto, or
        //     the page reads as a rail of bare section numbers followed by
        //     titles with no numbers on them.
        //
        // A tenth sits between them. Nothing observed falls in the gap.
        //
        // The count still has to clear 3 in absolute terms, so a short region
        // whose two or three lines happen to be wide cannot veto on a fraction
        // alone.
        const TABLE_ROW_SHARE: f32 = 0.10;
        let table_shaped = full_width_left_rows as f32 >= left.len() as f32 * TABLE_ROW_SHARE;
        if full_width_left_rows >= 3 && table_shaped {
            // Several left rows, and a real share of them, each blanket the
            // right column ⇒ this is a table-row slice, not a column gutter.
            // Don't take the column cut; the recursion falls back to a row
            // (horizontal) split and reads the table row-major.
            return None;
        }

        Some((left, right))
    }

    /// Fallback column split: find the deepest trough between the two
    /// strongest density peaks. Used when the standard valley detection
    /// fails because narrow table-cell spans partially fill the gutter.
    ///
    /// Returns the split X coordinate (absolute, not relative to x_min) if
    /// a genuine trough exists — i.e., the minimum between the peaks is ≤
    /// 50% of the weaker peak density.
    fn find_split_between_peaks(&self, profile: &ProjectionProfile) -> Option<f32> {
        let density = &profile.density;
        let n = density.len();
        if n < 3 {
            return None;
        }

        // Smooth with a small box filter (window = min_valley_width) to
        // average out individual narrow peaks before finding mass centres.
        let smooth_window = (self.min_valley_width as usize).max(3);
        let half = smooth_window / 2;

        // Smooth into a reused thread-local buffer instead of a fresh `Vec` per
        // failed-valley node. Window-mean is unchanged. (Confirmed not a source
        // of the p.692 non-determinism: the buffer is cleared+refilled to exactly
        // `n` each call and never read out of range.)
        thread_local! {
            static SMOOTH_SCRATCH: std::cell::RefCell<Vec<f32>> =
                const { std::cell::RefCell::new(Vec::new()) };
        }
        SMOOTH_SCRATCH.with(|cell| {
            let mut smoothed = cell.borrow_mut();
            smoothed.clear();
            smoothed.extend((0..n).map(|i| {
                let s = i.saturating_sub(half);
                let e = (i + half + 1).min(n);
                let sum: f32 = density[s..e].iter().sum();
                sum / (e - s) as f32
            }));

            // Find the strongest peak in each half. Use `safe_float_cmp` for
            // NaN-safe total ordering — matches the comparator used elsewhere
            // in the reading-order code so `density` sentinel values can't
            // reach a `partial_cmp` that maps them to `Equal`.
            let mid = n / 2;
            let left_peak =
                (0..mid).max_by(|&a, &b| crate::utils::safe_float_cmp(smoothed[a], smoothed[b]))?;
            let right_peak =
                (mid..n).max_by(|&a, &b| crate::utils::safe_float_cmp(smoothed[a], smoothed[b]))?;

            if smoothed[left_peak] == 0.0 || smoothed[right_peak] == 0.0 {
                return None;
            }

            // Find the minimum density in the interior between the two peaks.
            let search_start = left_peak.min(right_peak) + 1;
            let search_end = left_peak.max(right_peak);
            if search_start >= search_end {
                return None;
            }

            let trough_pos = (search_start..search_end)
                .min_by(|&a, &b| crate::utils::safe_float_cmp(smoothed[a], smoothed[b]))?;

            // Only use if trough is a genuine valley: ≤ 50% of the weaker peak.
            let weaker_peak = smoothed[left_peak].min(smoothed[right_peak]);
            if smoothed[trough_pos] > weaker_peak * 0.5 {
                return None;
            }

            // Trough must be at least min_valley_width from both edges.
            if trough_pos < self.min_valley_width as usize
                || trough_pos + self.min_valley_width as usize > n
            {
                return None;
            }

            Some(profile.x_min + trough_pos as f32)
        })
    }

    /// Find horizontal line (Y-axis) split using index-based partitioning.
    ///
    /// Returns `(above, below)` where `above` holds spans whose rectangle
    /// edge is at larger Y (higher on page in PDF coordinates) and must be
    /// processed first in reading order. PDF Spec ISO 32000-1:2008 §8.3.2.3
    /// defines the default user-space coordinate system with origin at the
    /// lower-left corner and Y increasing upward.
    fn find_vertical_split_indexed(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
    ) -> Option<(Vec<usize>, Vec<usize>)> {
        let profile = self.vertical_projection_indexed(all_spans, indices)?;
        let (valley_start, valley_end, valley_width) = self.find_valley(&profile)?;

        if valley_width < self.min_valley_width {
            return None;
        }

        let split_y = profile.y_min + (valley_start + valley_end) as f32 / 2.0;

        // `Rect::top()` returns `self.y`, the SMALLER Y coordinate of the
        // normalized rectangle — the method name follows a screen-coordinate
        // convention (Y grows downward) but PDF user space has Y growing
        // upward, so in PDF terms `bbox.top()` is actually the LOWER edge of
        // the glyph's bounding box. The predicate `bbox.top() >= split_y`
        // therefore classifies a span into `above` only when its *lowest*
        // point is already above the split line, i.e. the entire span sits
        // above the cut. Since `split_y` is the midpoint of a horizontal
        // projection valley (an empty band by construction), spans should
        // not straddle it in practice; any that do (e.g. a tall header
        // glyph whose ascenders dip into the valley) fall into `below`.
        let (above, below): (Vec<usize>, Vec<usize>) = indices
            .iter()
            .partition(|&&i| all_spans[i].bbox.top() >= split_y);

        if above.is_empty() || below.is_empty() {
            return None;
        }

        // Row (vertical) splits legitimately produce singleton top
        // partitions for lone headers/titles, so we accept down to 1
        // span per side. The column (horizontal) split is stricter since
        // single-span columns are almost always spurious.
        let min_side = (indices.len() / 10).max(1);
        if above.len() < min_side || below.len() < min_side {
            return None;
        }

        Some((above, below))
    }

    /// Calculate horizontal projection profile from indexed spans.
    fn horizontal_projection_indexed(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
    ) -> Option<ProjectionProfile> {
        if indices.is_empty() {
            return None;
        }

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        for &i in indices {
            let span = &all_spans[i];
            x_min = x_min.min(span.bbox.left());
            x_max = x_max.max(span.bbox.right());
            y_min = y_min.min(span.bbox.top());
            y_max = y_max.max(span.bbox.bottom());
        }

        let width = (x_max - x_min).ceil() as usize;
        if width > MAX_PROJECTION_SIZE {
            log::warn!(
                "XY-cut: horizontal projection width {} exceeds MAX_PROJECTION_SIZE {}, skipping region (degenerate CTM?)",
                width,
                MAX_PROJECTION_SIZE
            );
            return None;
        }
        let mut density = vec![0.0; width];

        // Text extractors frequently over-estimate span bbox widths
        // (trailing whitespace, stretched advance widths). That makes a
        // full-width projection falsely fill the inter-column gutter on
        // multi-column pages. We project each span's TEXT CORE footprint
        // anchored to its LEFT edge (where glyphs actually start), with
        // length proportional to character count. The left edge is
        // reliable; the right edge is not.
        //
        // Additionally, spans whose core width exceeds 55% of the region
        // width are full-width elements (section headers, figure captions,
        // table titles) that span both columns. Including them fills the
        // inter-column gutter in the density array and prevents valley
        // detection. They are excluded from the projection; the column
        // split boundary will still assign them correctly by left edge.
        let region_width = (x_max - x_min).max(1.0);
        for &i in indices {
            let span = &all_spans[i];
            let height = span.bbox.bottom() - span.bbox.top();
            let char_count = span
                .text
                .chars()
                .filter(|c| !c.is_whitespace())
                .count()
                .max(1);
            // 0.45em per char is a reasonable average across common PDF
            // fonts (Helvetica/Times/Arial at body size) and narrower
            // than the 0.5em advance used for monospace.
            let approx_char_width = (span.font_size * 0.45).max(2.5);
            let core_width = char_count as f32 * approx_char_width;
            let span_width = span.bbox.right() - span.bbox.left();
            // Skip full-width elements (captions, headers, table rows) whose
            // bbox spans more than 55% of the region — they fill the gutter.
            if span_width > region_width * 0.55 {
                continue;
            }
            // Skip isolated single-character/digit spans (table cell values
            // like 'G', 'T', '1', 'A') that scatter across the full X range
            // and fill the column gutter in the density profile. Body text
            // spans always contain multiple characters.
            if char_count < 2 {
                continue;
            }
            let core_left = span.bbox.left();
            let core_right = (core_left + core_width).min(span.bbox.right());
            let x_start = (core_left - x_min).max(0.0).ceil() as usize;
            let x_end = (core_right - x_min).ceil() as usize;

            for j in x_start..x_end.min(width) {
                density[j] += height;
            }
        }

        Some(ProjectionProfile {
            density,
            x_min,
            y_min,
        })
    }

    /// Calculate vertical projection profile from indexed spans.
    fn vertical_projection_indexed(
        &self,
        all_spans: &[TextSpan],
        indices: &[usize],
    ) -> Option<ProjectionProfile> {
        if indices.is_empty() {
            return None;
        }

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        for &i in indices {
            let span = &all_spans[i];
            x_min = x_min.min(span.bbox.left());
            x_max = x_max.max(span.bbox.right());
            y_min = y_min.min(span.bbox.top());
            y_max = y_max.max(span.bbox.bottom());
        }

        let height = (y_max - y_min).ceil() as usize;
        if height > MAX_PROJECTION_SIZE {
            log::warn!(
                "XY-cut: vertical projection height {} exceeds MAX_PROJECTION_SIZE {}, skipping region (degenerate CTM?)",
                height,
                MAX_PROJECTION_SIZE
            );
            return None;
        }
        let mut density = vec![0.0; height];

        for &i in indices {
            let span = &all_spans[i];
            let y_start = (span.bbox.top() - y_min).max(0.0).ceil() as usize;
            let y_end = (span.bbox.bottom() - y_min).ceil() as usize;
            let w = span.bbox.right() - span.bbox.left();

            for j in y_start..y_end.min(height) {
                density[j] += w;
            }
        }

        Some(ProjectionProfile {
            density,
            x_min,
            y_min,
        })
    }

    /// Find the widest valley (white space gap) in projection profile.
    ///
    /// Only considers INTERIOR valleys — gaps sandwiched between two
    /// non-empty regions. Leading/trailing empty bands (margin space
    /// outside the actual content extent) are ignored; they represent
    /// page margins, not column gutters, and picking them would produce
    /// meaningless splits.
    fn find_valley(&self, profile: &ProjectionProfile) -> Option<(usize, usize, f32)> {
        if profile.density.is_empty() {
            return None;
        }

        // Find peak density
        let peak = profile.density.iter().copied().fold(0.0, f32::max);

        if peak == 0.0 {
            return None;
        }

        // Find the content extent (first and last non-empty positions).
        // Valleys outside this extent are leading/trailing margins.
        let first_nonzero = profile.density.iter().position(|&d| d > 0.0)?;
        let last_nonzero = profile.density.iter().rposition(|&d| d > 0.0)?;

        // Find valleys (regions below threshold)
        let threshold = peak * self.valley_threshold;
        let mut valleys = Vec::new();
        let mut in_valley = false;
        let mut valley_start = 0;

        for (i, &density) in profile.density.iter().enumerate() {
            if density < threshold {
                if !in_valley {
                    valley_start = i;
                    in_valley = true;
                }
            } else if in_valley {
                valleys.push((valley_start, i));
                in_valley = false;
            }
        }

        if in_valley {
            valleys.push((valley_start, profile.density.len()));
        }

        // Merge adjacent interior valley segments separated by a narrow
        // bridge (≤ half the minimum valley width). A callout box or small
        // figure positioned in the column gutter creates a density bump
        // that splits what should be a single valley into two fragments.
        // Bridging re-joins them so the gap is still recognised as a
        // column boundary.
        let bridge_limit = (self.min_valley_width / 2.0).ceil() as usize;
        let interior: Vec<(usize, usize)> = valleys
            .into_iter()
            .filter(|&(start, end)| start > first_nonzero && end <= last_nonzero + 1)
            .collect();
        let mut merged: Vec<(usize, usize)> = Vec::with_capacity(interior.len());
        for seg in interior {
            if let Some(last) = merged.last_mut() {
                if seg.0 <= last.1 + bridge_limit {
                    last.1 = last.1.max(seg.1);
                    continue;
                }
            }
            merged.push(seg);
        }
        merged
            .into_iter()
            .map(|(start, end)| (start, end, (end - start) as f32))
            .max_by(|a, b| crate::utils::safe_float_cmp(a.2, b.2))
    }

    /// Test-only wrapper for horizontal projection on a contiguous slice.
    #[cfg(test)]
    fn horizontal_projection(&self, spans: &[TextSpan]) -> Option<ProjectionProfile> {
        let indices: Vec<usize> = (0..spans.len()).collect();
        self.horizontal_projection_indexed(spans, &indices)
    }

    /// Test-only wrapper for vertical projection on a contiguous slice.
    #[cfg(test)]
    fn vertical_projection(&self, spans: &[TextSpan]) -> Option<ProjectionProfile> {
        let indices: Vec<usize> = (0..spans.len()).collect();
        self.vertical_projection_indexed(spans, &indices)
    }

    /// Sort spans in reading order (top-to-bottom, left-to-right).
    #[cfg(test)]
    fn sort_spans<'a>(&self, spans: &'a [TextSpan]) -> Vec<&'a TextSpan> {
        let mut sorted: Vec<_> = spans.iter().collect();

        sorted.sort_by(|a, b| {
            // Sort by Y (top) first, descending (top of page first)
            let y_cmp = crate::utils::safe_float_cmp(b.bbox.top(), a.bbox.top());
            if y_cmp != std::cmp::Ordering::Equal {
                return y_cmp;
            }
            // Same Y level, sort by X (left) ascending
            crate::utils::safe_float_cmp(a.bbox.left(), b.bbox.left())
        });

        sorted
    }

    /// Sort indices in reading order (top-to-bottom, left-to-right).
    ///
    /// The row key is the baseline, banded, rather than `bbox.top()` compared
    /// for exact equality. Two things were wrong with the old key. Exact
    /// equality is not a row test at all — any sub-point difference put two
    /// glyphs of one line into different "rows", so the `x` tiebreak never
    /// ran and the order degenerated to a pure descending sort; on an OCR text
    /// layer, whose per-word baselines jitter by a couple of points, that
    /// emitted whole lines backwards. And `top()` is the wrong edge: it moves
    /// with the font size, so on a line mixing a 2 pt punctuation span with an
    /// 8 pt word the tops differ by more than the line spacing while the
    /// baselines agree to a fraction of a point. ISO 32000-1:2008 §9.4.4 puts
    /// the glyph displacement along the writing axis, which makes the baseline
    /// — not the ascender — what identifies a line.
    ///
    /// `row_aware_span_cmp` is the same comparator the single-column geometric
    /// path already uses, and its `i32` band key keeps the ordering a valid
    /// total order.
    fn sort_indices(&self, all_spans: &[TextSpan], indices: &[usize]) -> Vec<usize> {
        let row_baseline = crate::utils::snap_baselines_to_rows(all_spans, indices);
        // Sort positions within `indices`, because `row_baseline` is parallel
        // to `indices` rather than to `all_spans`.
        let mut order: Vec<usize> = (0..indices.len()).collect();
        order.sort_by(|&a, &b| {
            // Band and x from the row key; the last word goes to the baseline
            // the page draws. Two spans sharing a row key compare equal on any
            // tiebreak taken from that key, which would leave their order to
            // the sequence they arrived in.
            crate::utils::row_band_then_x_axis(
                all_spans[indices[a]].rotation_degrees,
                row_baseline[a],
                all_spans[indices[a]].bbox.left(),
                all_spans[indices[b]].rotation_degrees,
                row_baseline[b],
                all_spans[indices[b]].bbox.left(),
            )
            .then_with(|| {
                crate::utils::safe_float_cmp(
                    all_spans[indices[b]].bbox.y,
                    all_spans[indices[a]].bbox.y,
                )
            })
        });
        order.into_iter().map(|k| indices[k]).collect()
    }
}

/// Internal projection profile representation.
struct ProjectionProfile {
    /// Density values (height or width accumulated per bin)
    density: Vec<f32>,

    /// Origin coordinates
    x_min: f32,
    y_min: f32,
}

impl ReadingOrderStrategy for XYCutStrategy {
    fn apply(
        &self,
        spans: Vec<TextSpan>,
        _context: &ReadingOrderContext,
    ) -> Result<Vec<OrderedTextSpan>> {
        // (#543): detect multi-line heading runs and route the
        // partition through synthetic-span space so the splitter treats
        // each wrapped heading as a single atomic block. When no
        // headings are found we use the original index-only path that
        // avoids span clones during recursion.
        let heading_runs = self.find_heading_runs(&spans);

        let index_groups: Vec<Vec<usize>> = if heading_runs.is_empty() {
            let indices: Vec<usize> = (0..spans.len()).collect();
            self.partition_indexed(&spans, &indices)
        } else {
            let (synthetic, synthetic_origin) =
                self.synthesize_for_partition(&spans, &heading_runs);
            let synth_indices: Vec<usize> = (0..synthetic.len()).collect();
            let synth_groups = self.partition_indexed(&synthetic, &synth_indices);
            // Project synthetic-space groups back to ORIGINAL-span
            // indices (so the move-out below works on the input Vec).
            synth_groups
                .into_iter()
                .map(|group| {
                    let mut out = Vec::with_capacity(group.len());
                    for synth_idx in group {
                        out.extend(synthetic_origin[synth_idx].iter().copied());
                    }
                    out
                })
                .collect()
        };

        // Build result — moves spans out by index (no extra clone)
        let mut ordered = Vec::with_capacity(spans.len());
        // Convert spans to indexable storage for O(1) moves
        let mut span_slots: Vec<Option<TextSpan>> = spans.into_iter().map(Some).collect();
        let mut order_index = 0usize;

        for (group_idx, group) in index_groups.iter().enumerate() {
            for &i in group {
                if let Some(span) = span_slots[i].take() {
                    ordered.push(
                        OrderedTextSpan::with_info(span, order_index, ReadingOrderInfo::xycut())
                            .with_group(group_idx),
                    );
                    order_index += 1;
                }
            }
        }

        Ok(ordered)
    }

    fn name(&self) -> &'static str {
        "XYCutStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    fn make_span(x: f32, y: f32, width: f32, height: f32) -> TextSpan {
        make_span_text(x, y, width, height, "test", 12.0)
    }

    /// Like make_span but with realistic body-text density (~72 non-whitespace chars
    /// at 12pt, matching a full Letter-width column). Used when is_single_column_region
    /// must correctly identify a wide single-column page as not multi-column.
    fn make_body_span(x: f32, y: f32, width: f32, height: f32) -> TextSpan {
        // 72 non-whitespace characters at 12pt → core_width = 72 × 5.4 = 388.8pt
        // which is 83% of a 468pt column — enough to pass the 80% dense check.
        let text = "abcdefghijklmnopqrstuvwxyz".repeat(3); // 78 non-whitespace chars
        make_span_text(x, y, width, height, &text, 12.0)
    }

    fn make_span_text(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        text: &str,
        font_size: f32,
    ) -> TextSpan {
        use crate::layout::{Color, FontWeight};

        TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: text.to_string(),
            bbox: Rect::new(x, y, width, height),
            font_size,
            font_name: "Arial".to_string(),
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            split_boundary_before: false,
            offset_semantic: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 100.0,
            primary_detected: false,
            char_widths: vec![],
            char_x_offsets: Vec::new(),
            heading_level: None,
            rotation_degrees: 0.0,
            wmode: 0,
            rtl_draw_logical: false,
            mirrored: false,
            page_rotation_applied: 0,
        }
    }

    #[test]
    fn test_single_column_no_split() {
        let strategy = XYCutStrategy::new();
        let spans = vec![
            make_span(10.0, 100.0, 50.0, 10.0), // Line 1
            make_span(10.0, 85.0, 50.0, 10.0),  // Line 2
            make_span(10.0, 70.0, 50.0, 10.0),  // Line 3
        ];

        let groups = strategy.partition_region(&spans);
        assert_eq!(groups.len(), 1); // No split for single column
        assert_eq!(groups[0].len(), 3);
    }

    /// Realistic A4/Letter single-column page: 60 lines of body text,
    /// 14pt leading, one paragraph gap (30pt) mid-page. Only one body
    /// column exists, so XY-Cut must return exactly one group and
    /// preserve top-to-bottom reading order. A density-dip split at the
    /// paragraph gap would fragment the page and non-monotonically
    /// interleave paragraph contents.
    #[test]
    fn test_single_column_body_text_no_fragmentation() {
        let strategy = XYCutStrategy::new();
        // Simulate 60 lines of body text at x=72..540 (letter page, 1" margins).
        // Each line is a single span; line height 12pt, leading 14pt.
        let mut spans = Vec::new();
        let line_height = 12.0;
        let leading = 14.0;
        let left = 72.0;
        let right = 540.0;
        let width = right - left;
        let mut y = 720.0; // start near top of letter page
        for i in 0..60 {
            // Insert a paragraph gap in the middle (30pt, larger than min_valley_width=15pt)
            if i == 30 {
                y -= 30.0;
            }
            // Use realistic body text density (78 non-whitespace chars at 12pt) so
            // is_single_column_region correctly classifies the region as single-column.
            spans.push(make_body_span(left, y, width, line_height));
            y -= leading;
        }

        let groups = strategy.partition_region(&spans);
        assert_eq!(
            groups.len(),
            1,
            "single-column body text must not be split by XY-Cut (got {} groups)",
            groups.len()
        );
        assert_eq!(groups[0].len(), 60, "all 60 spans must be preserved");

        // Verify the group preserves monotonic top-to-bottom reading order
        // (each subsequent span's Y should be <= previous Y).
        let mut last_y = f32::MAX;
        for s in &groups[0] {
            assert!(
                s.bbox.top() <= last_y + 0.01,
                "reading order must be top-to-bottom: {} > {}",
                s.bbox.top(),
                last_y
            );
            last_y = s.bbox.top();
        }
    }

    /// After a vertical (row) split, the partition at higher Y (top of
    /// page in PDF coords) must be processed first in reading order so
    /// that header content appears before body content.
    #[test]
    fn test_vertical_split_preserves_top_to_bottom_order() {
        use crate::pipeline::reading_order::{ReadingOrderContext, ReadingOrderStrategy};

        let mut strategy = XYCutStrategy::new();
        strategy.min_spans_for_split = 2;

        // Header line at high Y (top of page in PDF coords).
        // Body block at lower Y values. Gap between them > min_valley_width.
        let make = |text: &str, x: f32, y: f32, w: f32| {
            let mut s = make_span(x, y, w, 12.0);
            s.text = text.to_string();
            s
        };
        // Two columns at y ∈ {200, 180, 160} (body), header at y=400.
        // Horizontal split will find the column gutter first; within each
        // column the header must still come out first in reading order.
        let spans = vec![
            make("HEADER LEFT", 50.0, 400.0, 200.0),
            make("HEADER RIGHT", 300.0, 400.0, 200.0),
            make("body-L1", 50.0, 200.0, 150.0),
            make("body-R1", 300.0, 200.0, 150.0),
            make("body-L2", 50.0, 180.0, 150.0),
            make("body-R2", 300.0, 180.0, 150.0),
        ];
        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).unwrap();

        let texts: Vec<&str> = ordered.iter().map(|o| o.span.text.as_str()).collect();
        // First output must be from y=400 (header), not y=180 (body bottom).
        assert!(texts[0].contains("HEADER"), "expected HEADER first, got sequence {:?}", texts);
    }

    /// Build the span set that exposes the full-measure-line guard.
    ///
    /// The region runs x 72..523.28 (451.28 pt wide). `include_body` adds
    /// eleven justified body lines that span the whole measure; the short
    /// fragments are present either way and are what create the density
    /// valley, since the projection discards anything wider than 55% of the
    /// region — which on a single-column page is every real body line.
    fn full_measure_page(include_body: bool) -> Vec<TextSpan> {
        const LEFT: f32 = 72.0;
        const RIGHT: f32 = 523.28;
        const FS: f32 = 10.9;
        let mut spans = Vec::new();

        if include_body {
            // "word " × 18 + "end" = 93 characters, 18 of them spaces.
            // At 0.45 em the full count reaches the right margin
            // (72 + 93 × 4.905 = 528 pt, capped at the bbox) while the
            // non-whitespace count alone reaches only 440 pt — short of the
            // 512.38 pt bar. That difference is the whole defect.
            let prose = format!("{}end", "word ".repeat(18));
            let mut y = 600.0;
            for _ in 0..11 {
                spans.push(make_span_text(LEFT, y, RIGHT - LEFT, FS, &prose, FS));
                y -= 14.0;
            }
        }

        // Two clusters of short fragments with a 108 pt channel between them
        // at x 222..330. This is the relative density dip the valley test
        // accepts; it is not an empty corridor on the real page either.
        let mut y = 440.0;
        for _ in 0..6 {
            spans.push(make_span_text(LEFT, y, 150.0, FS, "left frag", FS));
            spans.push(make_span_text(330.0, y, RIGHT - 330.0, FS, "right frag", FS));
            y -= 14.0;
        }
        spans
    }

    /// A column gutter is a corridor the lines do not cross. When full-measure
    /// body lines carry ink across the candidate cut, there is no gutter there
    /// whatever the projection says, and the cut must be refused.
    ///
    /// The guard for this already existed but estimated a span's ink width from
    /// its *non-whitespace* character count. Inter-word spaces consume an
    /// advance too, so justified prose measured about a ninth short and lines
    /// that genuinely reached the right margin went uncounted.
    #[test]
    fn test_cut_crossed_by_full_measure_lines_is_refused() {
        let strategy = XYCutStrategy::new();
        let spans = full_measure_page(true);
        let indices: Vec<usize> = (0..spans.len()).collect();

        assert!(
            strategy
                .find_horizontal_split_indexed(&spans, &indices)
                .is_none(),
            "eleven body lines span the whole measure, so no column cut is legal"
        );
    }

    /// Counter-case, and the reason the test above is not vacuous: with the
    /// full-measure lines removed the fragment geometry is unchanged, the same
    /// valley is found, and the cut IS taken. So the assertion above turns on
    /// the guard rather than on the valley never being detected at all.
    #[test]
    fn test_same_fragment_geometry_alone_is_still_cut() {
        let strategy = XYCutStrategy::new();
        let spans = full_measure_page(false);
        let indices: Vec<usize> = (0..spans.len()).collect();

        let split = strategy.find_horizontal_split_indexed(&spans, &indices);
        assert!(
            split.is_some(),
            "without the body lines this geometry is a genuine two-column split"
        );
        let (left, right) = split.unwrap();
        assert_eq!(left.len(), 6, "six fragments belong to the left column");
        assert_eq!(right.len(), 6, "six fragments belong to the right column");
    }

    /// Single-column page with a tall header band ("Title" or "Chapter
    /// heading") at the top. XY-Cut may validly split the header from
    /// the body (vertical Y-split) but must not further split the body
    /// into per-paragraph chunks.
    #[test]
    fn test_single_column_with_header_at_most_two_groups() {
        let strategy = XYCutStrategy::new();
        let mut spans = Vec::new();

        // Tall header band
        spans.push(make_span(72.0, 750.0, 468.0, 24.0));

        // 40 lines of body text below, separated by a ~50pt gap
        let mut y = 670.0;
        for _ in 0..40 {
            spans.push(make_span(72.0, y, 468.0, 12.0));
            y -= 14.0;
        }

        let groups = strategy.partition_region(&spans);
        assert!(
            groups.len() <= 2,
            "single-column with header should produce at most 2 groups, got {}",
            groups.len()
        );
        let total: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total, 41);
    }

    #[test]
    fn test_two_column_split() {
        let mut strategy = XYCutStrategy::new();
        strategy.min_spans_for_split = 2; // Lower threshold for testing

        let spans = vec![
            // Left column (x: 10-60)
            make_span(10.0, 100.0, 50.0, 10.0),
            make_span(10.0, 85.0, 50.0, 10.0),
            // Right column (x: 100-150) - wide gap of 40 points
            make_span(100.0, 100.0, 50.0, 10.0),
            make_span(100.0, 85.0, 50.0, 10.0),
        ];

        let groups = strategy.partition_region(&spans);
        // With wide gap and lower threshold, should split into 2 columns or keep as 1 group
        assert!(!groups.is_empty(), "Expected at least 1 group");
        // Verify all spans are preserved
        let total_spans: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total_spans, 4, "Expected all 4 spans to be preserved");
    }

    #[test]
    fn test_three_column_layout() {
        let strategy = XYCutStrategy::new();
        // Realistic column widths (≥ 60 pt per column, ≥ 6 body chars at
        // 10 pt — find_horizontal_split rejects narrower splits since
        // body columns are never sliver-wide).
        let spans = vec![
            // Column 1 (x: 10-110, 100pt wide)
            make_span(10.0, 100.0, 100.0, 10.0),
            make_span(10.0, 85.0, 100.0, 10.0),
            // Column 2 (x: 180-280, 100pt wide; 70pt gutter)
            make_span(180.0, 100.0, 100.0, 10.0),
            make_span(180.0, 85.0, 100.0, 10.0),
            // Column 3 (x: 350-450, 100pt wide; 70pt gutter)
            make_span(350.0, 100.0, 100.0, 10.0),
            make_span(350.0, 85.0, 100.0, 10.0),
        ];

        let groups = strategy.partition_region(&spans);
        // Should recursively split into at least 2 groups
        assert!(groups.len() >= 2, "Expected at least 2 groups, got {}", groups.len());
    }

    #[test]
    fn test_small_region_no_split() {
        let strategy = XYCutStrategy::new();
        let spans = vec![make_span(10.0, 100.0, 50.0, 10.0)];

        let groups = strategy.partition_region(&spans);
        assert_eq!(groups.len(), 1); // Single span region
        assert_eq!(groups[0].len(), 1);
    }

    #[test]
    fn test_sort_order() {
        let strategy = XYCutStrategy::new();
        let spans = vec![
            make_span(100.0, 70.0, 50.0, 10.0),  // Lower right
            make_span(10.0, 100.0, 50.0, 10.0),  // Upper left
            make_span(100.0, 100.0, 50.0, 10.0), // Upper right
            make_span(10.0, 70.0, 50.0, 10.0),   // Lower left
        ];

        let sorted = strategy.sort_spans(&spans);

        // Expect: upper left, upper right, lower left, lower right
        assert_eq!(sorted[0].bbox.top(), 100.0); // Upper
        assert_eq!(sorted[0].bbox.left(), 10.0); // Left
        assert_eq!(sorted[1].bbox.top(), 100.0); // Upper
        assert_eq!(sorted[1].bbox.left(), 100.0); // Right
    }

    #[test]
    fn test_horizontal_projection() {
        let strategy = XYCutStrategy::new();
        let spans = vec![
            make_span(10.0, 100.0, 30.0, 10.0),  // x: 10-40
            make_span(100.0, 100.0, 30.0, 10.0), // x: 100-130
        ];

        if let Some(profile) = strategy.horizontal_projection(&spans) {
            // Should have density peaks around x=25 and x=115
            assert!(!profile.density.is_empty());
            assert!(profile.density.len() >= 120); // Total width from 10 to 130 = 120

            // Gap is between local x=30 and x=90 (relative to x_min=10)
            // So in density array indices [30..90]
            let gap_start = 30;
            let gap_end = 90;
            if gap_end <= profile.density.len() {
                let gap_region = &profile.density[gap_start..gap_end];
                let gap_density: f32 = gap_region.iter().sum();
                assert!(gap_density < 1.0); // Gap should be mostly empty
            }
        }
    }

    #[test]
    fn test_vertical_projection() {
        let strategy = XYCutStrategy::new();
        let spans = vec![
            make_span(10.0, 100.0, 50.0, 20.0), // y: 100-120
            make_span(10.0, 50.0, 50.0, 20.0),  // y: 50-70
        ];

        if let Some(profile) = strategy.vertical_projection(&spans) {
            // Should have density peaks around y=110 and y=60
            assert!(!profile.density.is_empty());
            // Large gap between 70 and 100
            assert!(profile.density.len() > 50);
        }
    }

    #[test]
    fn test_narrow_gap_rejected() {
        let strategy = XYCutStrategy::new();
        let spans = vec![
            make_span(10.0, 100.0, 30.0, 10.0), // x: 10-40
            make_span(45.0, 100.0, 30.0, 10.0), // x: 45-75, gap: 5 points
        ];

        let groups = strategy.partition_region(&spans);
        // Gap is too narrow (< 15 points), should not split
        assert_eq!(groups.len(), 1);
    }

    /// Regression test for Bug 2: degenerate CTM places spans at ~100 trillion PDF points.
    /// horizontal_projection_indexed must return None instead of attempting a
    /// ~100-trillion-element vec allocation (which triggers handle_alloc_error → abort).
    #[test]
    fn test_degenerate_ctm_horizontal_projection_returns_none() {
        let strategy = XYCutStrategy::new();
        // Observed crash coordinate: 99_992_777_785_344 PDF points on a ~3968-point page.
        let degenerate_x: f32 = 99_992_777_785_344.0;
        let spans = vec![
            make_span(10.0, 100.0, 30.0, 10.0),
            make_span(degenerate_x, 100.0, 30.0, 10.0),
        ];

        // Must not panic or abort — projection should return None for oversized region.
        let result = strategy.horizontal_projection(&spans);
        assert!(
            result.is_none(),
            "expected None for projection spanning ~100 trillion points, got Some"
        );
    }

    /// Vertical projection must also return None for degenerate CTM y-coordinates.
    #[test]
    fn test_degenerate_ctm_vertical_projection_returns_none() {
        let strategy = XYCutStrategy::new();
        let degenerate_y: f32 = 99_992_777_785_344.0;
        let spans = vec![
            make_span(10.0, 100.0, 30.0, 10.0),
            make_span(10.0, degenerate_y, 30.0, 10.0),
        ];

        let result = strategy.vertical_projection(&spans);
        assert!(
            result.is_none(),
            "expected None for projection spanning ~100 trillion points, got Some"
        );
    }

    /// Issue #1: a CENTERED title/subtitle/byline block (each line
    /// centered, scattered leftmost edges) must NOT be split into
    /// per-word "columns". The centered "Quarterly Inventory Review"
    /// title (3 large words at the same Y with wide gaps) plus centered
    /// subtitle/byline previously aligned accidentally into fake columns,
    /// scrambling reading order. The centered-block guard must keep the
    /// whole block as ONE group so the title line stays intact.
    #[test]
    fn test_issue1_centered_title_block_not_split_into_columns() {
        let strat = XYCutStrategy::new();
        // Centered title (y=612, fs=28), subtitle (y=572), byline (y=532).
        // Leftmost edges scattered: 145 / 185 / 210 (centered, not columnar).
        let spans = vec![
            make_span_text(145.0, 612.0, 115.0, 28.0, "Quarterly", 28.0),
            make_span_text(300.0, 612.0, 115.0, 28.0, "Inventory", 28.0),
            make_span_text(430.0, 612.0, 92.0, 28.0, "Review", 28.0),
            make_span_text(185.0, 572.0, 40.0, 14.0, "Spring", 14.0),
            make_span_text(238.0, 572.0, 31.0, 14.0, "2025", 14.0),
            make_span_text(300.0, 572.0, 70.0, 14.0, "Distribution", 14.0),
            make_span_text(210.0, 532.0, 45.0, 10.0, "Northwind", 10.0),
            make_span_text(290.0, 532.0, 34.0, 10.0, "Traders", 10.0),
        ];
        let groups = strat.partition_region(&spans);
        assert_eq!(
            groups.len(),
            1,
            "centered title block must stay one group, got {} groups",
            groups.len()
        );
        // The three title words must appear in document order within the group.
        let g0: Vec<&str> = groups[0].iter().map(|s| s.text.as_str()).collect();
        let qi = g0.iter().position(|t| *t == "Quarterly").unwrap();
        let ii = g0.iter().position(|t| *t == "Inventory").unwrap();
        let ri = g0.iter().position(|t| *t == "Review").unwrap();
        assert!(qi < ii && ii < ri, "title words out of order: {:?}", g0);
    }

    /// XYCut must assign distinct group_id values to spans in different
    /// spatial partitions so that converters can keep each column's content
    /// contiguous instead of interleaving by Y-coordinate.
    #[test]
    fn test_xycut_group_id_two_column_layout() {
        use crate::pipeline::reading_order::{ReadingOrderContext, ReadingOrderStrategy};

        let mut strategy = XYCutStrategy::new();
        strategy.min_spans_for_split = 2; // lower threshold for small test

        // Left column (x=50-200) Right column (x=400-550)
        //   "Description" y=100 "Amount" y=100
        //   "Widget A" y=120 "$150.00" y=120
        //   "Widget B" y=140 "Discount" y=140
        //                                   "$25.00" y=160
        let make = |text: &str, x: f32, y: f32, w: f32| {
            let mut s = make_span(x, y, w, 12.0);
            s.text = text.to_string();
            s
        };
        let spans = vec![
            make("Description", 50.0, 100.0, 150.0),
            make("Amount", 400.0, 100.0, 150.0),
            make("Widget A", 50.0, 120.0, 150.0),
            make("$150.00", 400.0, 120.0, 150.0),
            make("Widget B", 50.0, 140.0, 150.0),
            make("Discount", 400.0, 140.0, 150.0),
            make("$25.00", 400.0, 160.0, 150.0),
        ];

        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).unwrap();

        // Every span must have a group_id assigned.
        assert!(
            ordered.iter().all(|s| s.group_id.is_some()),
            "all spans should have group_id set by XYCut"
        );

        // Left-column spans must share one group_id, right-column another.
        let left_groups: Vec<usize> = ordered
            .iter()
            .filter(|s| s.span.bbox.left() < 300.0)
            .map(|s| s.group_id.unwrap())
            .collect();
        let right_groups: Vec<usize> = ordered
            .iter()
            .filter(|s| s.span.bbox.left() >= 300.0)
            .map(|s| s.group_id.unwrap())
            .collect();

        // Within each column, group_id must be the same.
        assert!(
            left_groups.windows(2).all(|w| w[0] == w[1]),
            "left column spans should share the same group_id: {:?}",
            left_groups
        );
        assert!(
            right_groups.windows(2).all(|w| w[0] == w[1]),
            "right column spans should share the same group_id: {:?}",
            right_groups
        );

        // The two columns must have different group_ids.
        assert_ne!(
            left_groups[0], right_groups[0],
            "left and right columns should have different group_ids"
        );

        // Verify reading order keeps each column contiguous: all left-column
        // spans should appear before (or after) all right-column spans.
        let left_orders: Vec<usize> = ordered
            .iter()
            .filter(|s| s.span.bbox.left() < 300.0)
            .map(|s| s.reading_order)
            .collect();
        let right_orders: Vec<usize> = ordered
            .iter()
            .filter(|s| s.span.bbox.left() >= 300.0)
            .map(|s| s.reading_order)
            .collect();
        let left_max = *left_orders.iter().max().unwrap();
        let right_min = *right_orders.iter().min().unwrap();
        let left_min = *left_orders.iter().min().unwrap();
        let right_max = *right_orders.iter().max().unwrap();
        // Either all left before all right, or all right before all left.
        assert!(
            left_max < right_min || right_max < left_min,
            "columns must be contiguous in reading order: left={:?} right={:?}",
            left_orders,
            right_orders
        );
    }

    /// Plain-text rendering must keep group_id-separated columns as
    /// contiguous blocks, not interleave them by Y-coordinate.
    #[test]
    fn test_group_id_plain_text_no_interleave() {
        use crate::pipeline::converters::OutputConverter;
        use crate::pipeline::converters::PlainTextConverter;
        use crate::pipeline::reading_order::{ReadingOrderContext, ReadingOrderStrategy};
        use crate::pipeline::TextPipelineConfig;

        let mut strategy = XYCutStrategy::new();
        strategy.min_spans_for_split = 2;

        let make = |text: &str, x: f32, y: f32, w: f32| {
            let mut s = make_span(x, y, w, 12.0);
            s.text = text.to_string();
            s
        };
        let spans = vec![
            make("Description", 50.0, 100.0, 150.0),
            make("Amount", 400.0, 100.0, 150.0),
            make("Widget A", 50.0, 120.0, 150.0),
            make("$150.00", 400.0, 120.0, 150.0),
            make("Widget B", 50.0, 140.0, 150.0),
            make("Discount", 400.0, 140.0, 150.0),
            make("$25.00", 400.0, 160.0, 150.0),
        ];

        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).unwrap();

        let converter = PlainTextConverter::new();
        let config = TextPipelineConfig::default();
        let text = converter.convert(&ordered, &config).unwrap();

        // With Y-position-based merging, same-Y spans from left and right columns
        // are placed on the same line. This produces better label-value pairing:
        // "Description Amount" on one line, "Widget A $150.00" on the next.
        assert!(text.contains("Description"), "missing Description:\n{text}");
        assert!(text.contains("Amount"), "missing Amount:\n{text}");
        assert!(text.contains("Widget A"), "missing Widget A:\n{text}");
        assert!(text.contains("$150.00"), "missing $150.00:\n{text}");

        // Same-Y spans should be on the same line
        for line in text.lines() {
            if line.contains("Description") {
                assert!(
                    line.contains("Amount"),
                    "Description and Amount should be on same line:\n{text}"
                );
            }
        }
    }

    /// Builder for a bold heading span at a given font size. Used by the
    /// fix-543 tests to construct the "bold/large-font run spanning ≥ 2
    /// lines" shape the pre-partition heading lock must catch.
    fn make_bold_span(x: f32, y: f32, width: f32, text: &str, font_size: f32) -> TextSpan {
        use crate::layout::FontWeight;
        let mut s = make_span_text(x, y, width, font_size, text, font_size);
        s.font_weight = FontWeight::Bold;
        s
    }

    /// fix-543 unit: `find_heading_runs` must detect a 2-line bold
    /// heading whose wrapped tail line sits below the first line with
    /// matching X-extent. Single-line bold spans or paragraph-gap
    /// shapes must NOT be returned.
    #[test]
    fn find_heading_runs_detects_2_line_bold_heading() {
        let strategy = XYCutStrategy::new();

        // Body baseline (12pt regular) — establishes the median.
        let mut spans = Vec::new();
        let body_left = 72.0;
        let body_width = 200.0;
        let mut y = 720.0;
        for _ in 0..10 {
            spans.push(make_body_span(body_left, y, body_width, 12.0));
            y -= 14.0;
        }

        // 2-line bold heading at 14pt, same left margin, adjacent lines.
        spans.push(make_bold_span(body_left, 500.0, 180.0, "2.3 Performance and", 14.0));
        spans.push(make_bold_span(
            body_left,
            484.0,
            180.0,
            "Advantages of Vari-linear Network",
            14.0,
        ));

        let runs = strategy.find_heading_runs(&spans);
        assert_eq!(runs.len(), 1, "expected exactly one heading run, got {runs:?}");
        assert_eq!(runs[0].span_indices.len(), 2, "expected the run to cover both heading lines");

        // A LONE bold span (no second line) must NOT be locked: that
        // case is a single-line heading that XY-cut already handles.
        let mut spans_single = vec![make_body_span(body_left, 720.0, body_width, 12.0); 5];
        spans_single.push(make_bold_span(body_left, 500.0, 180.0, "Lone Heading", 14.0));
        let runs_single = strategy.find_heading_runs(&spans_single);
        assert!(runs_single.is_empty(), "single-line bold runs must not produce a HeadingRun");
    }

    /// fix-543 unit: the canonical repro shape — left-column 2-line
    /// bold heading whose wrapped tail line Y-overlaps right-column
    /// dense content (table caption + rows). Pre-fix, line 2 of the
    /// heading was bucketed into the RIGHT block; post-fix the lock
    /// keeps both heading lines in the LEFT block, adjacent to the
    /// left-column body paragraph.
    #[test]
    fn partition_keeps_heading_in_left_block() {
        let strategy = XYCutStrategy::new();

        // Geometry: two columns, ~260pt each with a ~30pt gutter.
        // Left col x ∈ [72, 332], right col x ∈ [362, 622].
        let left_col_x = 72.0_f32;
        let right_col_x = 362.0_f32;
        let col_width = 260.0_f32;

        let mut spans = Vec::new();

        // Left column: 2-line bold heading at Y=500/484, then 8 body
        // lines below at Y=460..360 (so the body paragraph anchors the
        // left block in reading order).
        spans.push(make_bold_span(left_col_x, 500.0, 180.0, "2.3 Performance and", 14.0));
        spans.push(make_bold_span(
            left_col_x,
            484.0,
            220.0,
            "Advantages of Vari-linear Network",
            14.0,
        ));
        let mut y = 460.0_f32;
        for _ in 0..8 {
            spans.push(make_body_span(left_col_x, y, col_width, 12.0));
            y -= 14.0;
        }

        // Right column: dense table-caption-style content that
        // Y-overlaps the heading's second line (Y=484). The
        // pre-fix block-assignment step pulled the heading's tail
        // into THIS column because the geometry was alone in
        // deciding bucket membership.
        spans.push(make_body_span(right_col_x, 500.0, col_width, 12.0));
        spans.push(make_body_span(right_col_x, 484.0, col_width, 12.0));
        spans.push(make_body_span(right_col_x, 468.0, col_width, 12.0));
        spans.push(make_body_span(right_col_x, 452.0, col_width, 12.0));
        spans.push(make_body_span(right_col_x, 436.0, col_width, 12.0));
        spans.push(make_body_span(right_col_x, 420.0, col_width, 12.0));
        spans.push(make_body_span(right_col_x, 404.0, col_width, 12.0));

        // Tag the heading lines so we can assert they land together.
        // (Done via the existing text; the bold builder sets distinct
        // strings.)
        let groups = strategy.partition_region(&spans);

        // Find the group holding "2.3 Performance and".
        let heading_first_group = groups
            .iter()
            .position(|g| g.iter().any(|s| s.text.contains("2.3 Performance and")))
            .expect("heading line 1 must land in some group");
        let heading_second_group = groups
            .iter()
            .position(|g| {
                g.iter()
                    .any(|s| s.text.contains("Advantages of Vari-linear Network"))
            })
            .expect("heading line 2 must land in some group");

        assert_eq!(
            heading_first_group, heading_second_group,
            "both heading lines must end up in the SAME block — pre-fix \
             they split across left/right column blocks"
        );

        // And that group must also contain left-column body content
        // (not right-column body content). All bbox.left() in the
        // group should be in the left-column band.
        let group = &groups[heading_first_group];
        for s in group {
            assert!(
                s.bbox.left() < right_col_x,
                "heading + body group must stay in the LEFT column; \
                 stray span at x={} (right_col starts at {}): {:?}",
                s.bbox.left(),
                right_col_x,
                s.text
            );
        }
    }

    /// fix-543 unit: the markdown converter must not promote an orphan
    /// heading-tail line to a phantom `### …`. With the pre-partition
    /// lock in place, the orphan does not exist in the first place,
    /// so the full heading appears as a single block immediately
    /// before its body paragraph in the markdown output.
    #[test]
    fn markdown_emits_single_heading_no_orphan() {
        use crate::pipeline::converters::{MarkdownOutputConverter, OutputConverter};
        use crate::pipeline::reading_order::ReadingOrderContext;
        use crate::pipeline::TextPipelineConfig;

        let strategy = XYCutStrategy::new();

        // Mini repro: left-column heading + body, right-column body,
        // Y-overlap on the wrapped heading's tail line.
        let left_col_x = 72.0_f32;
        let right_col_x = 362.0_f32;
        let col_width = 260.0_f32;

        let mut spans = Vec::new();
        spans.push(make_bold_span(left_col_x, 500.0, 180.0, "Performance and", 14.0));
        spans.push(make_bold_span(
            left_col_x,
            484.0,
            220.0,
            "Advantages of Vari-linear Network",
            14.0,
        ));
        let mut y = 460.0_f32;
        for _ in 0..6 {
            spans.push(make_body_span(left_col_x, y, col_width, 12.0));
            y -= 14.0;
        }
        for ky in [500.0, 484.0, 468.0, 452.0, 436.0, 420.0] {
            spans.push(make_body_span(right_col_x, ky, col_width, 12.0));
        }

        let context = ReadingOrderContext::new();
        let ordered = strategy.apply(spans, &context).expect("apply");
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let md = converter.convert(&ordered, &config).expect("markdown");

        // Both heading halves must appear, and BOTH must precede any
        // right-column content. Pre-fix, the wrapped-heading tail was
        // bucketed into the right-column block and emitted AFTER the
        // right column's body, then promoted to a fresh heading level
        // by `heading_level_ratio` (since it lost its body
        // continuation) — that's the phantom `### …` in the wrong
        // location. Post-fix the lock keeps both heading lines in the
        // left block adjacent to each other.
        let pos_first = md
            .find("Performance and")
            .expect("heading line 1 must appear in markdown");
        let pos_second = md
            .find("Advantages of Vari-linear Network")
            .expect("heading line 2 must appear in markdown");

        assert!(pos_first < pos_second, "heading line 1 must precede line 2:\n{md}");
        // Both heading lines must sit within the FIRST 30% of the
        // emitted markdown — they belong at the very top of the
        // left-column block, not floating somewhere after the
        // right-column body. (Pre-fix: the orphan tail landed deep
        // into the document after the right-column body — easily past
        // the 30 % mark.)
        let cap = (md.len() as f32 * 0.30) as usize;
        assert!(
            pos_second < cap.max(40),
            "heading-line-2 emitted late in the document — likely the \
             pre-fix orphan-in-wrong-column behaviour. pos_second={}, \
             cap={}, md=\n{md}",
            pos_second,
            cap
        );
    }

    /// A 2-column body where the gutter is narrower than
    /// `min_valley_width` AND the line-start cluster shape carries
    /// outlier singletons (title / caption / equation labels) so
    /// `detect_two_column_prose` bails on `clusters.len() != 2`.
    /// The narrow-gutter prose detector should catch this via
    /// gap-position clustering.
    #[test]
    fn test_narrow_gutter_prose_with_outlier_singletons() {
        let strategy = XYCutStrategy::new();
        let make_word = |x: f32, y: f32, text: &str| {
            // 12 pt font, ~5.4 pt/char advance.
            let w = (text.chars().count() as f32 * 5.4).max(3.0);
            make_span_text(x, y, w, 12.0, text, 12.0)
        };

        let mut spans = Vec::new();
        // 14 body lines with a tight gutter at x ≈ 295 (gap is
        // ~10 pt: left column ends at ~285, right column starts at
        // ~305). Each side has multiple per-word spans so the line
        // density is realistic.
        for i in 0..14 {
            let y = 600.0 - (i as f32) * 14.0;
            let left_words = [
                "Dwarf",
                "spheroidal",
                "galaxies",
                "of",
                "the",
                "Local",
                "Group",
                "are",
            ];
            let mut x = 40.0;
            for w in left_words {
                spans.push(make_word(x, y, w));
                x += (w.chars().count() as f32 * 5.4) + 2.5; // word + small space
            }
            let right_words = [
                "The",
                "Schwarzschild",
                "modeling",
                "technique",
                "offers",
                "another",
                "approach",
                "to",
            ];
            let mut x = 305.0;
            for w in right_words {
                spans.push(make_word(x, y, w));
                x += (w.chars().count() as f32 * 5.4) + 2.5;
            }
        }
        // Outlier singletons (title / caption / equation labels)
        // whose left edges don't align with either column. Under
        // detect_two_column_prose these produce extra clusters and
        // block detection — the narrow-gutter detector should still
        // catch the body via gap-position clustering.
        spans.push(make_word(145.0, 700.0, "Title text spanning"));
        spans.push(make_word(214.0, 680.0, "Caption that wraps somewhere"));
        spans.push(make_word(455.0, 670.0, "(1)"));
        spans.push(make_word(505.0, 660.0, "(2)"));

        let groups = strategy.partition_region(&spans);
        assert!(
            groups.len() >= 2,
            "expected at least 2 groups (column split) for narrow-gutter 2-col body \
             with outlier singletons; got {} group(s)",
            groups.len()
        );

        // The left and right body columns must be partitioned into
        // separate groups: no group should contain BOTH a body span
        // at x < 200 AND a body span at x ≥ 305.
        for (gi, g) in groups.iter().enumerate() {
            let has_left = g.iter().any(|s| s.bbox.left() < 200.0);
            let has_right = g.iter().any(|s| s.bbox.left() >= 305.0);
            assert!(
                !(has_left && has_right),
                "group {} contains spans from both columns — the column split did \
                 not separate them: {:?}",
                gi,
                g.iter()
                    .map(|s| (s.text.clone(), s.bbox.left()))
                    .collect::<Vec<_>>()
            );
        }
    }

    /// Negative: a single-column body with one large figure caption
    /// produces a strong within-line gap on the caption row but no
    /// recurring gap pattern across body lines. The narrow-gutter
    /// detector must NOT fire (would scramble reading order).
    #[test]
    fn test_narrow_gutter_prose_negative_single_col_with_caption() {
        let strategy = XYCutStrategy::new();
        let make_word = |x: f32, y: f32, text: &str| {
            let w = (text.chars().count() as f32 * 5.4).max(3.0);
            make_span_text(x, y, w, 12.0, text, 12.0)
        };

        let mut spans = Vec::new();
        // 14 single-column body lines (no within-line gutter).
        for i in 0..14 {
            let y = 600.0 - (i as f32) * 14.0;
            let words = [
                "This",
                "is",
                "an",
                "ordinary",
                "single",
                "column",
                "body",
                "paragraph",
                "with",
                "no",
                "interior",
                "gutter",
                "or",
                "wide",
                "gap",
            ];
            let mut x = 40.0;
            for w in words {
                spans.push(make_word(x, y, w));
                x += (w.chars().count() as f32 * 5.4) + 2.5;
            }
        }
        // One row that DOES have a within-line gap (figure caption
        // with a label on the right). This single outlier must not
        // make the page look 2-column.
        spans.push(make_word(40.0, 410.0, "Figure"));
        spans.push(make_word(80.0, 410.0, "caption"));
        spans.push(make_word(300.0, 410.0, "(continued)"));

        let groups = strategy.partition_region(&spans);
        // For a true single-column page, partition_region should
        // return either ONE group or a small number from row/header
        // splits — never a column split that lands left-side spans
        // in one group and right-side spans in another.
        // Count groups that contain at least one body span (x < 100):
        let body_groups = groups
            .iter()
            .filter(|g| {
                g.iter()
                    .any(|s| s.bbox.left() < 100.0 && s.text != "Figure")
            })
            .count();
        assert!(
            body_groups <= 1,
            "narrow-gutter detector wrongly column-split a single-column body: \
             body spans landed in {} groups",
            body_groups
        );
    }

    /// #536 Part 1a (xycut mirror): a short-verse two-column body —
    /// short tokens per column-line (`mean_chars <= 20`) but a strong
    /// balanced central gutter — must classify as `Prose` (so it gets
    /// cut) and be accepted by `detect_narrow_gutter_prose`, even though
    /// the long-line `mean_chars > 20` guard would reject it.
    #[test]
    fn test_short_verse_two_column_classified_prose_and_cut() {
        let strategy = XYCutStrategy::new();
        let make_word = |x: f32, y: f32, text: &str| {
            let w = (text.chars().count() as f32 * 5.4).max(3.0);
            make_span_text(x, y, w, 12.0, text, 12.0)
        };

        // Two columns: left starts at x=40, right at x=240. Each verse
        // line carries two short, EQUAL-length 4-char tokens per side
        // (8 non-whitespace chars/side → 16 chars/line, so mean_chars
        // ≤ 20 and the long-line prose guard does NOT apply — this
        // exercises the new short-line admission arm). The left column's
        // right edge lands consistently near x≈94 and the gutter gap
        // midpoint is stable at ≈167 every line (region x≈40..≈294,
        // width≈254, gutter offset ≈0.50·width). Stable gap → high
        // corridor concentration; equal token counts → balanced char
        // mass; two tight left-column start X's (40, 72) within one
        // column → ≤ 2 left-edge clusters. Uniform token widths keep the
        // within-line gap midpoint inside the 10 pt clustering radius.
        let left_lines = [
            ["comm", "lalu"],
            ["crea", "ciel"],
            ["terr", "etai"],
            ["info", "vide"],
            ["surf", "labi"],
            ["espr", "leau"],
        ];
        let right_lines = [
            ["EtD1", "ditq"],
            ["lumi", "soit"],
            ["etla", "fut1"],
            ["Dieu", "vitq"],
            ["bonn", "ilse"],
            ["aral", "obsc"],
        ];
        let mut spans = Vec::new();
        // 24 lines total (4 verse-stanzas of 6) so the body clears the
        // ≥12 gap-bearing-line floor in detect_narrow_gutter_prose.
        for rep in 0..4 {
            for i in 0..6 {
                let y = 600.0 - ((rep * 6 + i) as f32) * 14.0;
                // Left column: two 4-char words at x=40 and x=72.
                spans.push(make_word(40.0, y, left_lines[i][0]));
                spans.push(make_word(72.0, y, left_lines[i][1]));
                // Right column: two 4-char words at x=240 and x=272.
                spans.push(make_word(240.0, y, right_lines[i][0]));
                spans.push(make_word(272.0, y, right_lines[i][1]));
            }
        }

        let indices: Vec<usize> = (0..spans.len()).collect();
        assert_eq!(
            strategy.classify_region_kind(&spans, &indices),
            RegionKind::Prose,
            "short-verse two-column body with a strong balanced central \
             corridor must classify as Prose despite mean_chars <= 20"
        );
        assert!(
            strategy
                .detect_narrow_gutter_prose(
                    &spans,
                    &indices,
                    strategy.classify_region_kind(&spans, &indices),
                )
                .is_some(),
            "detect_narrow_gutter_prose must accept the routed short-verse \
             body (gutter found) so it is cut at the gutter"
        );
    }

    /// #536 Part 1a (xycut mirror) — negative: a short-cell multi-column
    /// numeric table (four narrow digit columns → short cells with ≥ 3
    /// left-edge clusters and scattered within-line gaps) must STILL
    /// classify as `Table` and NOT be accepted for cutting.
    #[test]
    fn test_short_cell_label_table_still_table_not_cut() {
        let strategy = XYCutStrategy::new();
        let make_word = |x: f32, y: f32, text: &str| {
            let w = (text.chars().count() as f32 * 5.4).max(3.0);
            make_span_text(x, y, w, 12.0, text, 12.0)
        };

        // A lopsided label+data table: a tiny numeric label column at
        // x=40 (a single digit, ~1 char) and a wide data column at x=100
        // (~8 chars). The within-line gutter is consistent, so the
        // corridor concentration/coverage/centre guards alone would NOT
        // reject it — but the left/right non-whitespace char balance is
        // grossly lopsided (label side ≈ 11 % of chars, well under the
        // 35 % floor), the length-independent table discriminator. A
        // genuine two-column verse body has balanced sides; this table
        // does not, so the short-line admission must reject it.
        // mean_chars ≈ 9 (≥ 8), so it does NOT fall through the
        // `mean_chars < 8 → Table` branch either — the balance check is
        // what keeps it out of Prose.
        let mut spans = Vec::new();
        let labels = ["7", "8", "9", "5", "3", "1"];
        let data = ["12345678", "23456781", "34567812", "45678123"];
        for i in 0..24 {
            let y = 600.0 - (i as f32) * 14.0;
            // Narrow label column (off to the far left, tiny char mass).
            spans.push(make_word(40.0, y, labels[i % labels.len()]));
            // Wide data column.
            spans.push(make_word(100.0, y, data[i % data.len()]));
        }

        let indices: Vec<usize> = (0..spans.len()).collect();
        assert_ne!(
            strategy.classify_region_kind(&spans, &indices),
            RegionKind::Prose,
            "lopsided label+data table must NOT be admitted as Prose \
             (left/right char mass is unbalanced — label column is tiny)"
        );
        assert!(
            strategy
                .detect_narrow_gutter_prose(
                    &spans,
                    &indices,
                    strategy.classify_region_kind(&spans, &indices),
                )
                .is_none(),
            "detect_narrow_gutter_prose must reject the short-cell table \
             (no central-corridor Prose admission) so it is NOT cut"
        );
    }

    #[test]
    fn test_degenerate_ctm_partition_region_does_not_abort() {
        let strategy = XYCutStrategy::new();
        let degenerate_x: f32 = 99_992_777_785_344.0;
        let spans = vec![
            make_span(10.0, 100.0, 30.0, 10.0),
            make_span(10.0, 85.0, 30.0, 10.0),
            make_span(10.0, 70.0, 30.0, 10.0),
            make_span(10.0, 55.0, 30.0, 10.0),
            make_span(10.0, 40.0, 30.0, 10.0),
            make_span(degenerate_x, 100.0, 30.0, 10.0),
        ];

        // Must complete without panicking and preserve all spans.
        let groups = strategy.partition_region(&spans);
        let total: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total, spans.len(), "all spans must be preserved");
    }

    /// Many distinct-Y single spans is the singleton-peel pathology. With the
    /// depth cap, `partition_region` must still terminate and preserve every
    /// span (the cap falls back to a flat sort, which keeps all indices).
    #[test]
    fn test_partition_indexed_depth_guard_preserves_all_spans() {
        let mut strategy = XYCutStrategy::new();
        strategy.min_spans_for_split = 2; // force maximum splitting

        // 300 spans, each on its own Y band — deeper than MAX_PARTITION_DEPTH.
        let spans: Vec<TextSpan> = (0..300)
            .map(|i| make_span(10.0, (i as f32) * 11.0, 30.0, 10.0))
            .collect();

        let groups = strategy.partition_region(&spans);
        let total: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total, spans.len(), "depth guard must not drop spans");
    }

    /// A news page's columns are not coextensive: a photograph, an
    /// advertisement or the end of a story stops one column well above its
    /// neighbour. Asking the two sides to end within a fifth of each other
    /// refuses the cut on ordinary three-column news pages, and the columns
    /// are then read straight across — splicing unrelated sentences and
    /// breaking any word hyphenated at the seam.
    ///
    /// The banner is wider than the region's 55% bound, so the density
    /// profile leaves it out and the corridor between the columns reads as
    /// empty; it still crosses that corridor, which is what brings the guard
    /// into play. The two sides here sit at a 0.60 height ratio — inside the
    /// span the corpus actually shows for real columns (0.48 to 0.79) and
    /// below every near-coextensive reading of them.
    #[test]
    fn uneven_columns_under_a_banner_still_take_the_cut() {
        let strategy = XYCutStrategy::new();
        let line = "abcdefghij klmnopqrst uvwxyzabcd efghijklmn opqrstuvwx";
        let mut spans = vec![make_span_text(
            55.0,
            730.0,
            470.0,
            16.0,
            &"W".repeat(40),
            16.0,
        )];

        let mut y = 100.0;
        while y <= 700.0 {
            spans.push(make_span_text(55.0, y, 200.0, 8.0, line, 8.0));
            y += 12.0;
        }
        // The right-hand column stops early, as it would above a photograph.
        let mut y = 343.0;
        while y <= 694.0 {
            spans.push(make_span_text(325.0, y, 200.0, 8.0, line, 8.0));
            y += 12.0;
        }

        let indices: Vec<usize> = (0..spans.len()).collect();
        let split = strategy.find_horizontal_split_indexed(&spans, &indices);
        assert!(
            split.is_some(),
            "two columns of prose at a 0.60 height ratio are still two \
             columns; refusing the cut leaves them to be read across the page"
        );
    }

    /// The shape of the page that exposed the populated-corridor defect: a
    /// letter-spaced listing whose multi-glyph fragments sit on every row,
    /// and beneath it a paragraph set one run per word — three full lines
    /// interleaved with three per-word lines whose words after the gap start
    /// at x=235. The listing's rows make the left mass dense, so any column
    /// covered by a single paragraph row is under the valley threshold.
    ///
    /// `listing_reaches_the_words`: the listing's left fragments end at
    /// x≈161, before the paragraph's short words, so the valley runs from
    /// there to 235 and holds `is`, `no`, `then` and `like` (the defect); or
    /// they run to x≈212, past those words, so the valley is 212..235 and
    /// holds nothing — an ordinary corridor beside a dense block.
    fn paragraph_split_at_a_word_gap(listing_reaches_the_words: bool) -> Vec<TextSpan> {
        let size = 10.0;
        let gap = 0.81 * size;
        let w = |s: &str| s.chars().count() as f32 * 0.5 * size;
        let mut spans = Vec::new();
        // The listing's left fragments: 24 glyphs reach x≈161 at the
        // profile's 0.45 em per glyph; 39 reach x≈205 (clamped to the box),
        // past the paragraph's short words on all but one row. The boxes
        // stay under 55% of the region's width, or the profile drops them.
        let (text, scale) = if listing_reaches_the_words {
            ("Content-Length: 195034 bytes", 0.8)
        } else {
            ("Content-Length: 195034 bytes and more still", 0.62)
        };
        for (i, y) in [310.0, 300.0, 290.0, 280.0, 270.0].iter().enumerate() {
            spans.push(make_span_text(72.0, *y, w(text) * scale, 8.0, text, 8.0));
            if i % 2 == 1 {
                spans.push(make_span_text(235.0, *y, 46.0, 8.0, "octet-stream", 8.0));
            }
        }
        let full = "When loaded in Internet Explorer, the browser,";
        let rows: [(&[&str], &[&str]); 3] = [
            (&["noticing", "that", "there", "is", "no"], &["file", "extension,", "pro-"]),
            (&["sniffing", "it,", "and", "then", "a"], &["header", "it", "was", "sent"]),
            (&["Anything", "that", "looks", "like", "an"], &["image", "at", "all", "is"]),
        ];
        for (i, y) in [260.0, 236.0, 212.0].iter().enumerate() {
            spans.push(make_span_text(72.0, *y, 224.0, size, full, size));
            let (before, after) = rows[i];
            let y = y - 12.0;
            let mut x = 72.0;
            for word in before {
                spans.push(make_span_text(x, y, w(word), size, word, size));
                x += w(word) + gap;
            }
            let mut x = 235.0;
            for word in after {
                spans.push(make_span_text(x, y, w(word), size, word, size));
                x += w(word) + gap;
            }
        }
        spans
    }

    /// The defect: the corridor the profile found holds the paragraph's own
    /// short words on three rows. The cut must be refused, or `pro-` is
    /// emitted apart from `ceeds`.
    #[test]
    fn test_corridor_that_holds_words_is_not_a_gutter() {
        let strategy = XYCutStrategy::new();
        let spans = paragraph_split_at_a_word_gap(true);
        let indices: Vec<usize> = (0..spans.len()).collect();
        assert!(
            strategy
                .find_horizontal_split_indexed(&spans, &indices)
                .is_none(),
            "a valley that holds whole words is a word gap under a dense block, \
             not a gutter; the full lines around it cross it between its rows"
        );
    }

    /// The same block with the corridor empty — the listing's fragments run
    /// past the paragraph's short words, so the valley is the bare gap
    /// before x=235 — is a corridor beside a dense block, and the cut stands.
    /// This pins that the refusal is about what the corridor holds, not
    /// about the full lines between the rows, which are here too.
    #[test]
    fn test_same_block_with_an_empty_corridor_still_takes_the_cut() {
        let strategy = XYCutStrategy::new();
        let spans = paragraph_split_at_a_word_gap(false);
        let indices: Vec<usize> = (0..spans.len()).collect();
        assert!(
            strategy
                .find_horizontal_split_indexed(&spans, &indices)
                .is_some(),
            "an empty corridor between a dense block and a column is a gutter, \
             whatever crosses it"
        );
    }

    /// The counter-direction, and the shape the guard exists for: a masthead
    /// side carrying only a nameplate and a dateline covers a fraction of the
    /// region and is not a column. That cut must still be refused, or the
    /// nameplate is emitted after the whole body instead of at the top of the
    /// page where it is printed.
    #[test]
    fn test_masthead_side_under_a_banner_is_still_refused() {
        let strategy = XYCutStrategy::new();
        let line = "abcdefghij klmnopqrst uvwxyzabcd efghijklmn opqrstuvwx";
        let mut spans = vec![make_span_text(
            55.0,
            730.0,
            470.0,
            16.0,
            &"W".repeat(40),
            16.0,
        )];

        let mut y = 100.0;
        while y <= 700.0 {
            spans.push(make_span_text(55.0, y, 200.0, 8.0, line, 8.0));
            y += 12.0;
        }
        // Nameplate and dateline only: about 28% of the region's height.
        let mut y = 530.0;
        while y <= 700.0 {
            spans.push(make_span_text(325.0, y, 200.0, 8.0, line, 8.0));
            y += 12.0;
        }

        let indices: Vec<usize> = (0..spans.len()).collect();
        let split = strategy.find_horizontal_split_indexed(&spans, &indices);
        assert!(
            split.is_none(),
            "a side holding only a nameplate and a dateline is not a column, \
             and the gap beside it is not a gutter"
        );
    }

    /// Build a two-column region whose left side carries `full_width` lines
    /// that reach across the right column, and `plain` that stop at the
    /// gutter. The right column always gets a line on every row, so the
    /// full-measure lines genuinely share a row with it.
    ///
    /// Left column 72..272, right column 320..520, so the corridor is 48 pt
    /// wide and `right_x_max` is 520. A full-measure line is 100% of the
    /// region's width, over the 55% bound, so the density profile leaves it
    /// out and the corridor is still found.
    fn two_columns_with_full_measure_lines(full_width: usize, plain: usize) -> Vec<TextSpan> {
        // 0.45 em per char at 8 pt is 3.6 pt: 56 chars spans 202 pt (the
        // column), 125 chars spans 450 pt (the whole region).
        let short = "a".repeat(56);
        let wide = "a".repeat(125);
        let mut spans = Vec::new();
        let mut y = 700.0_f32;
        for row in 0..(full_width + plain) {
            if row < full_width {
                spans.push(make_span_text(72.0, y, 448.0, 8.0, &wide, 8.0));
            } else {
                spans.push(make_span_text(72.0, y, 200.0, 8.0, &short, 8.0));
            }
            spans.push(make_span_text(320.0, y, 200.0, 8.0, &short, 8.0));
            y -= 12.0;
        }
        spans
    }

    /// A heading, a footnote and a caption running the full measure over two
    /// columns of prose are three full-width rows, and three is what the
    /// table-row veto counted. A data table's rows are most of it, so the
    /// count only means anything against the side it came from — measured on
    /// a real page, 3 of 100 left rows vetoed the cut and the two columns
    /// were then read straight across.
    #[test]
    fn three_full_measure_lines_over_a_prose_page_do_not_veto_the_cut() {
        let strategy = XYCutStrategy::new();
        let spans = two_columns_with_full_measure_lines(3, 97);
        let indices: Vec<usize> = (0..spans.len()).collect();
        assert!(
            strategy
                .find_horizontal_split_indexed(&spans, &indices)
                .is_some(),
            "three full-measure lines among a hundred are a heading and a \
             caption, not a table; the columns must still be cut apart"
        );
    }

    /// The counter-direction, and the shape the veto exists for: when the
    /// rows that blanket the right column are most of the side, the region is
    /// a table and cutting it vertically shreds its rows.
    #[test]
    fn test_page_whose_rows_are_mostly_full_measure_still_vetoes_the_cut() {
        let strategy = XYCutStrategy::new();
        let spans = two_columns_with_full_measure_lines(30, 6);
        let indices: Vec<usize> = (0..spans.len()).collect();
        assert!(
            strategy
                .find_horizontal_split_indexed(&spans, &indices)
                .is_none(),
            "rows that blanket the right column and are most of the side are \
             a table's rows; a vertical cut through them shreds every row"
        );
    }
}
