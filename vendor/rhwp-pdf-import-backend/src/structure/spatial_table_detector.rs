//! Spatial table detection from PDF text layout.
//!
//! Implements table detection according to ISO 32000-1:2008 Section 5.2 (Coordinate Systems).
//! Uses X and Y coordinate clustering to identify table structure in PDFs that lack explicit
//! table markup in the structure tree.

use crate::layout::text_block::TextSpan;
use crate::structure::table_extractor::{span_text_for_cell, Table, TableCell, TableRow};
use std::collections::HashMap;

/// Disjoint-set (union-find) with path compression.
struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn find(&mut self, i: usize) -> usize {
        let mut curr = i;
        while self.parent[curr] != curr {
            self.parent[curr] = self.parent[self.parent[curr]];
            curr = self.parent[curr];
        }
        curr
    }

    fn union(&mut self, i: usize, j: usize) {
        let ri = self.find(i);
        let rj = self.find(j);
        if ri != rj {
            self.parent[ri] = rj;
        }
    }

    fn groups(&mut self) -> HashMap<usize, Vec<usize>> {
        let mut result: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..self.parent.len() {
            let root = self.find(i);
            result.entry(root).or_default().push(i);
        }
        result
    }
}

/// Strategy for detecting table boundaries (v0.3.14).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TableStrategy {
    /// Use only vector lines to define boundaries.
    #[serde(rename = "lines")]
    Lines,
    /// Use only text alignment to define boundaries.
    #[serde(rename = "text")]
    Text,
    /// Use both text and lines (hybrid approach).
    #[default]
    #[serde(rename = "both")]
    Both,
}

/// Configuration for spatial table detection.
#[derive(Debug, Clone, PartialEq)]
pub struct TableDetectionConfig {
    /// Whether table detection is enabled.
    pub enabled: bool,
    /// Strategy for horizontal boundary detection.
    pub horizontal_strategy: TableStrategy,
    /// Strategy for vertical boundary detection.
    pub vertical_strategy: TableStrategy,
    /// X-coordinate tolerance for column grouping.
    pub column_tolerance: f32,
    /// Y-coordinate tolerance for row grouping.
    pub row_tolerance: f32,
    /// Minimum number of cells required for a valid table.
    pub min_table_cells: usize,
    /// Minimum number of columns required for a valid table.
    pub min_table_columns: usize,
    /// Ratio of regular rows required for a valid table structure.
    pub regular_row_ratio: f32,
    /// Maximum number of columns allowed before rejecting as false positive.
    pub max_table_columns: usize,
    /// Merge threshold for post-clustering column merge pass.
    /// Adjacent columns whose centers are within this distance are merged.
    pub column_merge_threshold: f32,
    /// Minimum gap between Y-range groups of vertical lines to trigger a cluster split.
    /// Default: 20.0. Use smaller values (e.g. 4.0) for strict mode, larger (e.g. 40.0)
    /// for relaxed mode where V-lines at mixed Y-ranges should stay together.
    pub v_split_gap: f32,
    /// Enable text-only spatial detection as a fallback when no ruling lines are found.
    ///
    /// When `true` and the page has no table-relevant paths (no ruling lines or
    /// rectangles), the detector falls through to `detect_tables_from_spans_column_aware`
    /// rather than returning an empty result.  This is the right default for structured
    /// output callers (`to_markdown`, `to_html`) that explicitly want tabular layout
    /// and is also relied on by the public `extract_tables` API for line-less PDFs.
    /// Set to `false` from callers that want the conservative
    /// "no ruling lines → no tables" behaviour (e.g. plain-text extraction paths
    /// that explicitly opt out — see `extract_page_tables`).
    ///
    /// False-positive prose / TOC / underline tables that this default would
    /// previously have surfaced are filtered post-detection by the
    /// `looks_like_prose_table` shape gate and a ≥ 3-row evidence requirement
    /// on text-only and h-rule paths.
    ///
    /// Default: `true`.
    pub text_fallback: bool,
}

impl Default for TableDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            horizontal_strategy: TableStrategy::Both,
            vertical_strategy: TableStrategy::Both,
            column_tolerance: 15.0,
            row_tolerance: 2.8,
            min_table_cells: 4,
            min_table_columns: 2,
            regular_row_ratio: 0.3,
            max_table_columns: 15,
            column_merge_threshold: 25.0,
            v_split_gap: 20.0,
            text_fallback: true,
        }
    }
}

impl TableDetectionConfig {
    /// Create a strict table detection configuration.
    pub fn strict() -> Self {
        Self {
            enabled: true,
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            column_tolerance: 2.0,
            row_tolerance: 1.0,
            min_table_cells: 6,
            min_table_columns: 3,
            regular_row_ratio: 0.8,
            max_table_columns: 12,
            column_merge_threshold: 10.0,
            v_split_gap: 4.0,
            text_fallback: true,
        }
    }

    /// Create a relaxed table detection configuration.
    pub fn relaxed() -> Self {
        Self {
            enabled: true,
            horizontal_strategy: TableStrategy::Text,
            vertical_strategy: TableStrategy::Text,
            column_tolerance: 10.0,
            row_tolerance: 5.0,
            min_table_cells: 4,
            min_table_columns: 2,
            regular_row_ratio: 0.3,
            max_table_columns: 20,
            column_merge_threshold: 30.0,
            v_split_gap: 40.0,
            text_fallback: true,
        }
    }
}

/// Validate that an extracted table is not a false positive.
///
/// Rejects:
/// - Tables with too many empty cells (> 60%).
/// - 2-column tables that contain a **continuation-row signature**: any
///   row whose left-hand cell is empty while the right-hand cell is
///   non-empty. Product data sheets draw faint cell backgrounds behind
///   label/value rows, which the spatial detector can cluster into tiny
///   2-column tables; when the right-hand value wraps onto a second line,
///   the continuation row leaves an empty left-hand label cell beside
///   the wrapped value text. This exact shape is a reliable false-positive
///   signal. Sparse 2-column tables with *legitimately* missing right-hand
///   values (e.g. "Fax: ", "N/A" rows) are NOT rejected by this rule.
fn is_valid_table(table: &Table) -> bool {
    if table.rows.is_empty() || table.col_count == 0 {
        return false;
    }

    let total_cells = table.rows.len() * table.col_count;
    let empty_cells = table
        .rows
        .iter()
        .flat_map(|r| &r.cells)
        .filter(|c| c.text.trim().is_empty())
        .count();
    let empty_ratio = empty_cells as f32 / total_cells.max(1) as f32;

    if empty_ratio > 0.6 {
        return false;
    }

    // Narrow false-positive signature: a 2-column "table" emitted from
    // label/value rows with faint cell backgrounds, where the right-hand
    // value wraps onto a continuation line. The continuation row has an
    // empty left label cell next to a non-empty right value cell. Reject
    // only this specific shape so legitimate sparse 2-column tables
    // (missing values on the right, blank section headers, etc.) still
    // validate.
    if table.col_count == 2 {
        let has_continuation_row = table.rows.iter().any(|r| {
            r.cells.len() == 2
                && r.cells[0].text.trim().is_empty()
                && !r.cells[1].text.trim().is_empty()
        });
        if has_continuation_row {
            return false;
        }
    }

    true
}

/// Additional gate applied to SPATIAL-only table detection (no explicit
/// lines/rulings): reject "word-per-cell" false positives where a
/// paragraph's visual gaps accidentally align into columns.
///
/// Signature: >=5 columns AND >70% of non-empty cells contain only a
/// single word. Real data tables have multi-word labels, numeric values,
/// or dense content; a paragraph mis-read as a table reads as a sentence
/// when the cells are concatenated.
///
/// This gate is NOT applied when rulings/lines define the table — in
/// that case the author explicitly marked the structure and we trust it
/// even if cells are single-character (census forms, sparse grids).
fn passes_spatial_quality_gate(table: &Table) -> bool {
    if table.col_count < 5 {
        return true;
    }
    let non_empty: Vec<&str> = table
        .rows
        .iter()
        .flat_map(|r| &r.cells)
        .map(|c| c.text.trim())
        .filter(|t| !t.is_empty())
        .collect();
    if non_empty.is_empty() {
        return true;
    }
    // A genuine numeric data table (financial / metrics slides) is legitimately
    // almost all single tokens — every cell is a *number* — so the generic
    // single-word prose gate below would wrongly reject it and flatten it into a
    // bold label plus run-on numbers. Bypass the gate
    // ONLY when the table is clearly numeric-DOMINATED (≥50% of non-empty cells
    // are data values). This is deliberately strict: number-heavy prose (an
    // academic page with inline citations/equations whose words happen to align
    // into columns) stays below 50% numeric and is still held to the prose gate,
    // so the bypass does not manufacture false tables.
    let data_values = non_empty.iter().filter(|t| is_data_value(t)).count();
    if data_values * 2 >= non_empty.len() {
        return true;
    }
    // Otherwise: high single-word density is the signature of prose split into
    // one-word columns by aligned inter-word gaps — reject.
    let single_word_count = non_empty
        .iter()
        .filter(|t| t.split_whitespace().count() <= 1)
        .count();
    let ratio = single_word_count as f32 / non_empty.len() as f32;
    ratio <= 0.7
}

/// A numeric / data value token: digits plus the usual numeric punctuation
/// (decimal point, thousands comma, percent, sign, currency). Requires at least
/// one digit so a bare `+` or `$` is not treated as data. Used so numeric-table
/// cells do not read as prose fragments in the spatial quality gate.
fn is_data_value(t: &str) -> bool {
    !t.is_empty()
        && t.chars().any(|c| c.is_ascii_digit())
        && t.chars().all(|c| {
            c.is_ascii_digit()
                || matches!(
                    c,
                    '.' | ',' | '%' | '+' | '-' | '\u{2212}' | '$' | '\u{20AC}' | '\u{00A3}'
                )
        })
}

/// Reject a spatial (no-rulings) "table" whose rows are wrapped paragraph
/// lines — a flowing prose page (heading + body paragraph + footer) whose
/// inter-word gaps coincidentally aligned into columns.
///
/// Signature: at least one row, when its non-empty cells are concatenated
/// left-to-right, crosses a SENTENCE boundary mid-row — an alphanumeric
/// character, a sentence terminator, a space, then a new word (e.g. "...to
/// 23,500. Stockout rate..."). Real data-table rows hold values/labels, not
/// running sentences that span a terminator into the next clause, so this
/// almost never fires on genuine tables. Only applied to spatial tables (the
/// caller is the no-rulings path); ruled tables are author-marked and
/// trusted.
///
/// Case is checked via the Unicode general category (`char::is_uppercase` /
/// `is_lowercase`), not ASCII-only, so cased non-Latin scripts (Greek,
/// Cyrillic, Armenian, …) get the same "new sentence starts with a capital"
/// signal Latin does. Scripts with no case distinction at all (Bengali,
/// Devanagari, …) can't use that signal, so their sentence-final danda
/// (`।`, `॥`) is instead treated as a terminator in its own right: a danda
/// followed by a space and another letter mid-row is itself the
/// discriminator, since a genuine data cell doesn't embed a sentence stop
/// followed by more prose in the same row.
fn looks_like_prose_paragraph(table: &Table) -> bool {
    const CASELESS_TERMINATORS: [char; 2] = ['।', '॥'];

    for row in &table.rows {
        let joined = row
            .cells
            .iter()
            .map(|c| c.text.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let chars: Vec<char> = joined.chars().collect();
        for i in 0..chars.len() {
            // Cased terminator: the exact prior-release strict signature — a
            // lowercase/digit, then `.`/`!`/`?`, then space + capital +
            // lowercase (a genuine new sentence). Kept ASCII with immediate
            // neighbours on purpose: broadening it (any alphanumeric before,
            // skip-spaces, Unicode case) over-rejected genuine data tables
            // whose cells contain abbreviations or capitalised codes ("Fig.
            // 3", "Dr. Smith", "A. Test") as prose.
            if matches!(chars[i], '.' | '!' | '?')
                && i >= 1
                && (chars[i - 1].is_ascii_lowercase() || chars[i - 1].is_ascii_digit())
                && i + 3 < chars.len()
                && chars[i + 1] == ' '
                && chars[i + 2].is_ascii_uppercase()
                && chars[i + 3].is_ascii_lowercase()
            {
                return true;
            }
            // Caseless terminator (Bengali/Devanagari danda ।/॥): scripts with
            // no case distinction can't use the capital-start signal, so a
            // danda followed (allowing a stray positioning gap) by another
            // letter mid-row is itself the tell — a genuine data cell doesn't
            // embed a sentence stop followed by more prose in the same row.
            if CASELESS_TERMINATORS.contains(&chars[i]) {
                let Some(&prev) = chars[..i].iter().rev().find(|c| **c != ' ') else {
                    continue;
                };
                if !prev.is_alphanumeric() {
                    continue;
                }
                let mut after = chars[i + 1..].iter().copied().skip_while(|c| *c == ' ');
                if after.next().is_some_and(|c| c.is_alphabetic()) {
                    return true;
                }
            }
        }
    }

    // Punctuation-independent fallback: a wrapped-prose row whose column
    // break fell mid-clause carries no sentence terminator at all (a title
    // caption, a clause with no `.`/`!`/`?`), so the checks above miss it.
    // Real data-cell values are atomic (numbers, codes, capitalised labels)
    // and essentially never start with a lowercase letter; running-sentence
    // fragments frequently do ("text with", "a smaller", "on"). This mirrors
    // `looks_like_prose_table`'s per-cell signal in document.rs, but applies
    // PER ROW with no minimum cell count — that function's ≥10-cell floor
    // (needed there to avoid over-rejecting small genuine tables under a
    // table-wide ratio) is exactly what lets a small (2–3 row) fabricated
    // table slip through untested. A 2-cell floor plus a majority-of-row
    // requirement keeps ordinary short data rows (a units column, a coded
    // abbreviation) safe: those cells are numeric/capitalised, not lowercase
    // clause fragments, so they rarely cross the 50% bar even at n=2.
    for row in &table.rows {
        let cells: Vec<&str> = row
            .cells
            .iter()
            .map(|c| c.text.trim())
            .filter(|t| !t.is_empty())
            .collect();
        if cells.len() < 2 {
            continue;
        }
        let lower_starts = cells
            .iter()
            .filter(|c| c.chars().next().is_some_and(char::is_lowercase))
            .count();
        if lower_starts as f32 / cells.len() as f32 > 0.5 {
            return true;
        }
    }

    // Vertically-stacked single-character lines: a rotated axis label or
    // figure legend, drawn one glyph per line so its bbox flattens into the
    // page's normal (unrotated) column grid, reads as a cell whose text is
    // mostly 1-character lines joined by `\n` (e.g. "k\nc\no\nc\nE-Box" for a
    // vertically-drawn "clock"-style label). A genuine multi-line data cell
    // wraps at WORD boundaries — its lines hold whole words, not lone
    // letters — so this shape is essentially unique to misread rotated text.
    for row in &table.rows {
        for cell in &row.cells {
            let lines: Vec<&str> = cell
                .text
                .split('\n')
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            if lines.len() < 3 {
                continue;
            }
            let single_char_lines = lines.iter().filter(|l| l.chars().count() == 1).count();
            if single_char_lines as f32 / lines.len() as f32 > 0.5 {
                return true;
            }
        }
    }

    false
}

/// Reject a spatial (no-rulings) "table" that is actually CJK prose split into
/// columns by the wrap geometry of an unspaced script. CJK (Chinese, Japanese,
/// Korean Han/kana) writes without inter-word spaces, so wrapped prose lines
/// align into a grid the column detector mistakes for a table. A genuine CJK
/// data cell is short (a label or a number); a prose fragment is a long run of
/// ideographs/kana. Reject when any cell holds a run of `MIN_CJK_RUN` or more
/// consecutive CJK characters. Ruled / author-marked tables never reach here.
fn looks_like_cjk_prose(table: &Table) -> bool {
    const MIN_CJK_RUN: usize = 12;
    fn is_cjk(c: char) -> bool {
        matches!(
            c as u32,
            0x3040..=0x30FF      // Hiragana + Katakana
            | 0x3400..=0x4DBF    // CJK Ext A
            | 0x4E00..=0x9FFF    // CJK Unified
            | 0xF900..=0xFAFF    // CJK Compatibility
            | 0xFF66..=0xFF9F    // Halfwidth Katakana
            | 0xAC00..=0xD7AF    // Hangul Syllables
        )
    }
    table.rows.iter().flat_map(|r| &r.cells).any(|c| {
        let mut run = 0usize;
        for ch in c.text.chars() {
            if is_cjk(ch) {
                run += 1;
                if run >= MIN_CJK_RUN {
                    return true;
                }
            } else if !ch.is_whitespace() {
                run = 0;
            }
        }
        false
    })
}

/// Reject a spatial (no-rulings) "table" that is actually a bulleted list whose
/// markers and bodies aligned into columns. A genuine data table never carries
/// a cell that is *only* a bullet glyph; an untagged list rendered into two
/// columns (`• | Ship the API.`) does. Catches the structured-document
/// false-positive where a heading + bulleted list + prose are mis-fused into a
/// grid. Ruled / author-marked tables never reach this path.
fn looks_like_bulleted_list(table: &Table) -> bool {
    /// An unambiguous bullet glyph (never a legitimate data value).
    fn is_bullet_glyph(c: char) -> bool {
        matches!(
            c,
            '\u{2022}'
                | '\u{2023}'
                | '\u{2043}'
                | '\u{2219}'
                | '\u{25AA}'
                | '\u{25CF}'
                | '\u{25E6}'
                | '\u{00B7}'
                | '\u{2024}'
        )
    }
    fn is_list_item(t: &str) -> bool {
        let mut chars = t.chars();
        match chars.next() {
            // Leads with a bullet glyph (lone "•" or "• item") → a list item.
            Some(c) if is_bullet_glyph(c) => true,
            // A lone "*" cell is also a marker (but "*5" footnote-data is not).
            Some('*') => chars.next().is_none(),
            _ => false,
        }
    }
    table
        .rows
        .iter()
        .flat_map(|r| &r.cells)
        .filter(|c| !c.text.trim().is_empty())
        .any(|c| is_list_item(c.text.trim()))
}

/// Detect page column regions from an X-projection histogram of text spans.
///
/// Builds a histogram of horizontal coverage (2pt buckets), then identifies
/// runs of empty buckets as candidate column gutters.  Only gaps wider than
/// 20pt **and** at least 4% of the total page X-extent are treated as true
/// column boundaries, preventing internal table whitespace from being
/// misidentified as page column gutters.
///
/// Returns a list of `(x_min, x_max)` column regions sorted left-to-right.
fn detect_page_columns(spans: &[TextSpan]) -> Vec<(f32, f32)> {
    if spans.is_empty() {
        return Vec::new();
    }

    // 1. Find page X extent, excluding degenerate outliers.
    //
    // Per PDF 32000-1:2008 §8.3.2.3, user space is an infinite plane and
    // the CTM can produce arbitrarily large coordinates. The visible region
    // is defined by MediaBox/CropBox. Degenerate CTM transforms (e.g.,
    // rotated dvips pages) can produce span coordinates ~1e16 pt wide,
    // which would cause a multi-petabyte histogram allocation.
    //
    // Strategy: compute the median X center, then exclude any span whose
    // center is more than MAX_EXTENT from the median. This fixed safety
    // bound covers all standard page sizes while rejecting pathological
    // outliers; pages wider than 10,000pt fall back to single column.
    const MAX_EXTENT_FROM_MEDIAN: f32 = 5_000.0;

    let mut x_centers: Vec<f32> = spans
        .iter()
        .map(|s| s.bbox.x + s.bbox.width * 0.5)
        .collect();
    x_centers.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let median_x = x_centers[x_centers.len() / 2];

    let mut page_x_min = f32::MAX;
    let mut page_x_max = f32::MIN;
    for s in spans {
        let center = s.bbox.x + s.bbox.width * 0.5;
        if (center - median_x).abs() > MAX_EXTENT_FROM_MEDIAN {
            continue; // skip degenerate outlier
        }
        let left = s.bbox.x;
        let right = s.bbox.x + s.bbox.width;
        if left < page_x_min {
            page_x_min = left;
        }
        if right > page_x_max {
            page_x_max = right;
        }
    }

    if page_x_min >= page_x_max {
        // All spans were outliers or no valid extent
        return vec![(
            spans.iter().map(|s| s.bbox.x).fold(f32::MAX, f32::min),
            spans
                .iter()
                .map(|s| s.bbox.x + s.bbox.width)
                .fold(f32::MIN, f32::max),
        )];
    }

    let page_width = page_x_max - page_x_min;

    // Final safety: if width is still unreasonable after outlier filtering,
    // skip column detection entirely. Typical pages are ≤2400pt (A0).
    if page_width > 10_000.0 {
        log::warn!(
            "detect_page_columns: page_width {:.0} still exceeds safe limit after \
             outlier filtering, falling back to single column",
            page_width,
        );
        return vec![(page_x_min, page_x_max)];
    }

    // 2. Build histogram with 2pt buckets.
    let bucket_size = 2.0_f32;
    let n_buckets = ((page_width) / bucket_size).ceil() as usize + 1;
    let mut histogram = vec![0u32; n_buckets];

    for s in spans {
        let center = s.bbox.x + s.bbox.width * 0.5;
        if (center - median_x).abs() > MAX_EXTENT_FROM_MEDIAN {
            continue;
        }
        let left = s.bbox.x;
        let right = s.bbox.x + s.bbox.width;
        let b_start = ((left - page_x_min) / bucket_size).floor() as usize;
        let b_end = ((right - page_x_min) / bucket_size).ceil() as usize;
        for b in b_start..b_end.min(n_buckets) {
            histogram[b] += 1;
        }
    }

    // 3. Collect all gaps (runs of empty buckets) with their positions and widths.
    let min_gap_pt = 20.0_f32;
    let min_gap_buckets = ((min_gap_pt / bucket_size).ceil() as usize).max(1);

    struct Gap {
        start_bucket: usize,
        len_buckets: usize,
    }

    let mut gaps = Vec::new();
    let mut gap_start: Option<usize> = None;

    for (i, &count) in histogram.iter().enumerate() {
        if count == 0 {
            if gap_start.is_none() {
                gap_start = Some(i);
            }
        } else if let Some(gs) = gap_start {
            let gap_len = i - gs;
            if gap_len >= min_gap_buckets {
                gaps.push(Gap {
                    start_bucket: gs,
                    len_buckets: gap_len,
                });
            }
            gap_start = None;
        }
    }

    // 4. For each gap, determine the "immediate region" on each side
    //    (bounded by adjacent gaps or page edges).  A gap qualifies as a
    //    page column gutter only if at least one of its immediate regions
    //    contains a span wider than `min_paragraph_width` (80pt).  This
    //    prevents inter-cell whitespace in tables from being treated as
    //    column gutters.
    let min_paragraph_width = 80.0_f32;
    let qualifying_indices: Vec<usize> = (0..gaps.len())
        .filter(|&gi| {
            let gap_left_x = page_x_min + gaps[gi].start_bucket as f32 * bucket_size;
            let gap_right_x =
                page_x_min + (gaps[gi].start_bucket + gaps[gi].len_buckets) as f32 * bucket_size;

            // Left boundary: right edge of previous gap, or page_x_min.
            let left_bound = if gi > 0 {
                page_x_min
                    + (gaps[gi - 1].start_bucket + gaps[gi - 1].len_buckets) as f32 * bucket_size
            } else {
                page_x_min
            };
            // Right boundary: left edge of next gap, or page_x_max.
            let right_bound = if gi + 1 < gaps.len() {
                page_x_min + gaps[gi + 1].start_bucket as f32 * bucket_size
            } else {
                page_x_max
            };

            // Check left immediate region [left_bound, gap_left_x].
            let has_wide_left = spans.iter().any(|s| {
                let center = s.bbox.x + s.bbox.width / 2.0;
                center >= left_bound && center <= gap_left_x && s.bbox.width >= min_paragraph_width
            });

            // Check right immediate region [gap_right_x, right_bound].
            let has_wide_right = spans.iter().any(|s| {
                let center = s.bbox.x + s.bbox.width / 2.0;
                center >= gap_right_x
                    && center <= right_bound
                    && s.bbox.width >= min_paragraph_width
            });

            has_wide_left || has_wide_right
        })
        .collect();
    let qualifying_gaps: Vec<&Gap> = qualifying_indices.iter().map(|&i| &gaps[i]).collect();

    if qualifying_gaps.is_empty() {
        // No qualifying gap → single column.
        return vec![(page_x_min, page_x_max)];
    }

    // 5. Build column regions from gaps.
    let mut columns = Vec::new();

    // Find first occupied bucket.
    let first_occ = match histogram.iter().position(|&c| c > 0) {
        Some(b) => b,
        None => return Vec::new(),
    };
    let mut region_start = first_occ;

    for gap in &qualifying_gaps {
        // Close current region at the gap start.
        if gap.start_bucket > region_start {
            let x_min = page_x_min + region_start as f32 * bucket_size;
            let x_max = page_x_min + gap.start_bucket as f32 * bucket_size;
            columns.push((x_min, x_max));
        }
        region_start = gap.start_bucket + gap.len_buckets;
    }

    // Close last region.
    let last_occ = histogram
        .iter()
        .rposition(|&c| c > 0)
        .unwrap_or(n_buckets - 1);
    if region_start <= last_occ {
        let x_min = page_x_min + region_start as f32 * bucket_size;
        let x_max = page_x_min + (last_occ + 1) as f32 * bucket_size;
        columns.push((x_min, x_max));
    }

    columns
}

/// Column-aware text-only table detection.
///
/// Detects page columns first (via X-projection histogram), then runs
/// `detect_tables_from_spans()` independently on each column partition.
/// This prevents multi-column academic layouts from being misinterpreted
/// as wide tables spanning the whole page.
pub fn detect_tables_from_spans_column_aware(
    spans: &[TextSpan],
    config: &TableDetectionConfig,
) -> Vec<Table> {
    if !config.enabled || spans.is_empty() {
        return Vec::new();
    }

    let page_cols = detect_page_columns(spans);

    // Single column (or none) → delegate directly.
    if page_cols.len() <= 1 {
        return detect_tables_from_spans(spans, config);
    }

    // Multiple columns → partition spans and detect per column.
    let mut all_tables = Vec::new();
    for &(col_x_min, col_x_max) in &page_cols {
        let col_spans: Vec<TextSpan> = spans
            .iter()
            .filter(|s| {
                let span_center = s.bbox.x + s.bbox.width / 2.0;
                span_center >= col_x_min && span_center <= col_x_max
            })
            .cloned()
            .collect();
        if col_spans.is_empty() {
            continue;
        }
        let mut tables = detect_tables_from_spans(&col_spans, config);
        all_tables.append(&mut tables);
    }

    all_tables
}

/// Detect tables from spatial layout of text spans.
pub fn detect_tables_from_spans(spans: &[TextSpan], config: &TableDetectionConfig) -> Vec<Table> {
    if !config.enabled || spans.is_empty() {
        return Vec::new();
    }

    // The pitch cap is a REFINEMENT, so probe it the same way the finer
    // text-edge lattice is probed below: adopt it only when the grid it
    // produces still passes every emission gate, and otherwise keep the
    // uncapped columns.
    //
    // Without that it is a cliff. Once the cap has split a page's columns,
    // that column set is the only one tried; it either survives the gates or
    // the page becomes prose, and nothing preserves the set that would have
    // produced a table. Measured across the corpus, the cap moved 255 table
    // pages to prose and 59 the other way — the same mechanism in both
    // directions, so the gains were not evidence the rule was right.
    let columns_uncapped =
        detect_columns(spans, config.column_tolerance, config.column_merge_threshold, None);
    let columns_capped =
        detect_columns(spans, config.column_tolerance, config.column_merge_threshold, Some(config));
    let mut columns = if columns_capped.len() == columns_uncapped.len() {
        columns_capped
    } else {
        let probe_rows = detect_rows(spans, config.row_tolerance);
        let capped_holds = probe_rows.len() >= 2
            && columns_capped.len() >= config.min_table_columns.max(2)
            && columns_capped.len() <= config.max_table_columns
            && {
                let g = assign_spans_to_cells(spans, &columns_capped, &probe_rows);
                validate_table_structure_internal(&g, config) && {
                    let t = grid_to_table(&g, spans, None);
                    is_valid_table(&t)
                        && passes_spatial_quality_gate(&t)
                        && !looks_like_prose_paragraph(&t)
                }
            };
        if capped_holds {
            columns_capped
        } else {
            columns_uncapped
        }
    };

    // Greedy X-center clustering fragments a single logical cell whose
    // words are internally spaced (e.g. an agenda row "Receiving Dock
    // Inspection" laid out with wide inter-word gaps) into one column
    // per word. detect_text_edge_columns instead keeps only X edges that
    // recur across >= 3 distinct rows, so single-row word positions are
    // rejected and the true column grid (Time / Activity / Team) is
    // recovered. Cross-row recurrence is a strictly stronger column
    // signal than one row's word spacing, so prefer the text-edge result
    // whenever it yields a valid, strictly-smaller column set.
    //
    // Safety: for tables with < 3 rows, text-edge can keep no column
    // (every edge appears in < 3 rows) so it returns fewer than
    // min_table_columns and the guard below leaves greedy untouched —
    // small genuine tables are unaffected.
    // If greedy clustering produced too many columns, try text-edge
    // detection which looks for X positions that recur across multiple rows.
    if columns.len() > config.max_table_columns {
        let te_columns = detect_text_edge_columns(spans, config);
        if te_columns.len() >= config.min_table_columns.max(2) && te_columns.len() < columns.len() {
            columns = te_columns;
        }
    }

    // Borderless numeric lattice (ML / results tables). When the column gap
    // is below `column_merge_threshold`, greedy clustering fuses a dense grid
    // of short numeric cells laid out on a regular ~20pt pitch, so two values
    // share one cell ("0.69 0.76"). The text-edge detector keeps only X edges
    // that recur across >=3 rows, which on a numeric lattice recovers every
    // column. Prefer it when the spans are predominantly numeric and it splits
    // a coarser greedy set into more (still bounded) columns. The numeric-
    // predominance gate keeps prose / label-value tables (e.g. Google-Docs
    // exports) on the greedy path untouched.
    let numeric_spans = spans
        .iter()
        .filter(|s| is_numeric_cell(s.text.trim()))
        .count();
    if numeric_spans >= 10 && columns.len() <= config.max_table_columns {
        let te_columns = detect_text_edge_columns(spans, config);
        if te_columns.len() > columns.len()
            && te_columns.len() >= 5
            && te_columns.len() <= config.max_table_columns
            && is_regular_lattice(&te_columns)
        {
            // Adopt the finer lattice ONLY when it still forms a fully valid
            // grid. A sparse split that fails the quality gate would otherwise
            // drop the whole table to prose — worse than the merged-column
            // baseline. Probing here means the refinement can only refine a
            // table that stays valid, never demote one.
            let probe_rows = detect_rows(spans, config.row_tolerance);
            if probe_rows.len() >= 2 {
                let probe_grid = assign_spans_to_cells(spans, &te_columns, &probe_rows);
                if validate_table_structure_internal(&probe_grid, config) {
                    let probe_table = grid_to_table(&probe_grid, spans, None);
                    if is_valid_table(&probe_table)
                        && passes_spatial_quality_gate(&probe_table)
                        && !looks_like_prose_paragraph(&probe_table)
                    {
                        columns = te_columns;
                    }
                }
            }
        }
    }

    if columns.len() < config.min_table_columns.max(2) || columns.len() > config.max_table_columns {
        return Vec::new();
    }

    let rows = detect_rows(spans, config.row_tolerance);
    if rows.len() < 2 {
        return Vec::new();
    }

    // Baseline gate (CRITICAL): the ORIGINAL (unfiltered) columns must
    // already form a table that passes EVERY emission gate baseline
    // uses — structural validation AND the final is_valid_table /
    // passes_spatial_quality_gate checks. The row-coverage cleanup
    // below only REFINES a table that would have been emitted anyway;
    // it must never CREATE a table from content baseline treated as
    // prose. Without checking the FINAL gates here, dropping phantom
    // columns can flip a borderline case that baseline rejected on the
    // quality gate into a spurious table (observed on annots.pdf link
    // lists and right_to_left_01.pdf Arabic prose in the 70-PDF sweep).
    let orig_grid = assign_spans_to_cells(spans, &columns, &rows);
    if !validate_table_structure_internal(&orig_grid, config) {
        return Vec::new();
    }
    let orig_table = grid_to_table(&orig_grid, spans, None);
    if !is_valid_table(&orig_table)
        || !passes_spatial_quality_gate(&orig_table)
        || looks_like_prose_paragraph(&orig_table)
        || looks_like_bulleted_list(&orig_table)
        || looks_like_cjk_prose(&orig_table)
    {
        return Vec::new();
    }

    // Issue #6/#5: drop "phantom" columns created by a single cell whose
    // words are spaced apart (e.g. an agenda "Receiving Dock Inspection"
    // laid out with wide gaps → one greedy column per word). A genuine
    // table column carries content in MOST rows; a per-word phantom
    // appears in only one or two. Keep only columns whose spans occupy
    // at least 60% of rows (min 2). Phantom-column spans are then
    // re-assigned to the nearest surviving column by assign_spans_to_cells,
    // re-joining the words into their true cell. Skipped for small
    // tables (< 3 rows) where every column legitimately spans all rows.
    if rows.len() >= 3 {
        columns = filter_columns_by_row_coverage(&columns, &rows, spans);
        if columns.len() < config.min_table_columns.max(2) {
            return Vec::new();
        }
    }

    let grid = assign_spans_to_cells(spans, &columns, &rows);
    if !validate_table_structure_internal(&grid, config) {
        return Vec::new();
    }

    let table = grid_to_table(&grid, spans, None);
    if !is_valid_table(&table)
        || !passes_spatial_quality_gate(&table)
        || looks_like_prose_paragraph(&table)
        || looks_like_bulleted_list(&table)
        || looks_like_cjk_prose(&table)
    {
        return Vec::new();
    }
    vec![table]
}

#[derive(Debug, Clone)]
struct ColumnCluster {
    x_center: f32,
    x_min: f32,
    x_max: f32,
    span_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
struct RowCluster {
    y_center: f32,
    y_min: f32,
    y_max: f32,
    span_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
struct GridStructure {
    columns: Vec<ColumnCluster>,
    rows: Vec<RowCluster>,
    cells: Vec<Vec<Vec<usize>>>,
}

impl GridStructure {
    fn is_row_empty(&self, row_idx: usize) -> bool {
        self.cells[row_idx].iter().all(|cell| cell.is_empty())
    }

    fn is_column_empty(&self, col_idx: usize) -> bool {
        for row in &self.cells {
            if !row[col_idx].is_empty() {
                return false;
            }
        }
        true
    }

    fn trim_empty_columns(&self) -> GridStructure {
        let num_rows = self.cells.len();
        let num_cols = self.columns.len();

        let mut first_col = 0;
        while first_col < num_cols && self.is_column_empty(first_col) {
            first_col += 1;
        }

        let mut last_col = num_cols;
        while last_col > first_col && self.is_column_empty(last_col - 1) {
            last_col -= 1;
        }

        if first_col >= last_col {
            return self.clone();
        }

        let mut active_cols = Vec::new();
        for c in first_col..last_col {
            let col_width = self.columns[c].x_max - self.columns[c].x_min;
            if col_width < 2.0 && self.is_column_empty(c) {
                continue;
            }
            active_cols.push(c);
        }

        if active_cols.is_empty() {
            return self.clone();
        }

        let new_columns: Vec<ColumnCluster> = active_cols
            .iter()
            .map(|&c| self.columns[c].clone())
            .collect();

        let mut new_cells = Vec::with_capacity(num_rows);
        for r in 0..num_rows {
            let row_cells = active_cols
                .iter()
                .map(|&c| self.cells[r][c].clone())
                .collect();
            new_cells.push(row_cells);
        }

        GridStructure {
            columns: new_columns,
            rows: self.rows.clone(),
            cells: new_cells,
        }
    }
}

#[derive(Debug, Clone)]
struct CellMergeInfo {
    colspan: u32,
    rowspan: u32,
    covered: bool,
}

/// Issue #6/#5: keep only columns that carry content in a meaningful
/// fraction of rows. A real table column appears in most rows; a
/// "phantom" column produced by spaced words inside a single cell (e.g.
/// "Receiving Dock Inspection" with wide inter-word gaps) appears in
/// only one or two rows. Each column's distinct-row coverage is the
/// number of rows in which at least one of its spans falls.
///
/// Threshold: >= ceil(0.6 * num_rows), floored at 2. Phantom columns
/// (coverage 1) are removed; their spans get re-assigned to the nearest
/// surviving column downstream, rejoining the words into one cell.
fn filter_columns_by_row_coverage(
    columns: &[ColumnCluster],
    rows: &[RowCluster],
    spans: &[TextSpan],
) -> Vec<ColumnCluster> {
    let num_rows = rows.len();
    if num_rows < 3 {
        return columns.to_vec();
    }
    // Minimum distinct rows a column must touch to be "real".
    let min_cov = (((num_rows as f32) * 0.6).ceil() as usize).max(2);

    // Pre-resolve each span's row index (nearest row center within y-extent).
    let span_row = |sidx: usize| -> Option<usize> {
        let cy = spans[sidx].bbox.center().y;
        rows.iter().position(|r| cy <= r.y_max && cy >= r.y_min)
    };

    let kept: Vec<ColumnCluster> = columns
        .iter()
        .filter(|col| {
            let mut seen: Vec<usize> = col
                .span_indices
                .iter()
                .filter_map(|&s| span_row(s))
                .collect();
            seen.sort_unstable();
            seen.dedup();
            seen.len() >= min_cov
        })
        .cloned()
        .collect();

    // Safety: never return fewer than 2 columns from here — if the
    // coverage filter would collapse the table, fall back to the
    // original columns (the caller's min-columns guard then decides).
    if kept.len() >= 2 {
        kept
    } else {
        columns.to_vec()
    }
}

fn detect_columns(
    spans: &[TextSpan],
    column_tolerance: f32,
    merge_threshold: f32,
    cap_to_pitch: Option<&TableDetectionConfig>,
) -> Vec<ColumnCluster> {
    // Sort span indices by X coordinate before clustering for deterministic results.
    let mut sorted_indices: Vec<usize> = (0..spans.len()).collect();
    sorted_indices
        .sort_by(|&a, &b| crate::utils::safe_float_cmp(spans[a].bbox.left(), spans[b].bbox.left()));

    let mut columns: Vec<ColumnCluster> = Vec::new();
    for idx in sorted_indices {
        let x = spans[idx].bbox.left();
        let mut found = false;
        for col in &mut columns {
            if (x - col.x_center).abs() < column_tolerance {
                col.span_indices.push(idx);
                col.x_min = col.x_min.min(x);
                col.x_max = col.x_max.max(x);
                // Update running average so the cluster center tracks
                // the actual midpoint.
                let n = col.span_indices.len() as f32;
                col.x_center = col.x_center * ((n - 1.0) / n) + x / n;
                found = true;
                break;
            }
        }
        if !found {
            columns.push(ColumnCluster {
                x_center: x,
                x_min: x,
                x_max: x,
                span_indices: vec![idx],
            });
        }
    }

    // Sort columns by center before merge pass.
    columns.sort_by(|a, b| crate::utils::safe_float_cmp(a.x_center, b.x_center));

    // `merge_threshold` is a fixed absolute-point gap; on its own it breaks
    // down as column count grows and the table's actual pitch shrinks (a
    // dense numeric table with a 12pt column pitch has every adjacent pair
    // fused by even a modest fixed threshold, collapsing distinct columns
    // into one). Scale the *effective* threshold down for narrow-pitch
    // tables by capping it at a fraction of the table's own median
    // inter-column gap — same ratio-based pitch reasoning as
    // `is_regular_lattice`'s on-pitch band (`[0.6, 1.6] * median`): a gap
    // under `0.6 * median` is off-pitch-small relative to this table's own
    // columns and should merge regardless of the fixed threshold, while a
    // sparse table with few, widely-spaced columns keeps the full
    // `merge_threshold` (no median signal to scale from).
    //
    // The premise is that the median gap is this table's pitch, so it has to
    // hold before it is used. On a layout with no pitch -- most often prose the
    // detector has mistaken for a table -- the median is the middle of a spread
    // of unrelated gaps, and capping at 0.6 of it forbids merging fragments
    // that are one column: a word is split from the number beside it
    // (`Line 1` -> `Line` | `1`), and because neither half then matches the flow
    // span it came from, the row is emitted a second time as prose.
    //
    // `is_regular_lattice` is the gate rather than the pitch test alone: that
    // test tolerates two off-pitch gaps, which on a handful of columns is every
    // gap it has, so it answers "yes" for any small irregular group. The
    // lattice predicate carries the column-count floor that makes the tolerance
    // mean something, and it is the same predicate this ratio was borrowed
    // from, so the two stay in step.
    // Measure the pitch on the columns that actually recur down the page, not
    // on the greedy pre-merge clusters. Those clusters are one per distinct
    // word-start x, so they include the running head, the page number, every
    // indent level and every wrapped continuation: an indented legal index
    // produced 13 of them where the table has 4 columns, and a leader-dot fee
    // table 17 for 4. Their centres sit on a ~21-27 pt "pitch" that is just the
    // average word-start spacing of prose, so the gate answered "lattice" about
    // pages that have none, capped the merge threshold, and split the real
    // columns apart until the emission gates rejected the page as prose.
    //
    // `detect_text_edge_columns` keeps only x edges recurring across several
    // rows, which is what a column is. A dense numeric lattice still has those
    // on a regular pitch, so the case this cap exists for keeps it; an indented
    // index has too few to clear the lattice floor and keeps the fixed
    // threshold.
    // Cap the merge threshold to the table's own pitch, measured on the columns
    // that actually recur down the page rather than on the greedy clusters
    // built above. Those are one per distinct word-start x, so they include the
    // running head, the page number, every indent level and every wrapped
    // continuation: an indented legal index gave 13 of them where the table has
    // 4 columns, a leader-dot fee table 17 for 4. Their centres sit on a
    // ~21-27 pt "pitch" that is only the average word-start spacing of prose,
    // so the gate said "lattice" about pages that have none, capped the
    // threshold, split the real columns apart, and the emission gates then
    // rejected the page as prose — 255 table pages lost across the corpus.
    //
    // `detect_text_edge_columns` keeps only x edges recurring across several
    // rows, which is what a column is. A dense numeric lattice still has those
    // on a regular pitch and keeps the cap; an indented index has too few to
    // clear the lattice floor and keeps the fixed threshold.
    let pitch_columns = cap_to_pitch
        .map(|c| detect_text_edge_columns(spans, c))
        .unwrap_or_default();
    let effective_merge_threshold = if is_regular_lattice(&pitch_columns) {
        let mut gaps: Vec<f32> = pitch_columns
            .windows(2)
            .map(|w| w[1].x_center - w[0].x_center)
            .filter(|g| *g > 0.0)
            .collect();
        if gaps.is_empty() {
            merge_threshold
        } else {
            gaps.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
            let median_gap = gaps[gaps.len() / 2];
            merge_threshold.min(median_gap * 0.6)
        }
    } else {
        merge_threshold
    };

    // Post-clustering merge pass: merge adjacent columns whose centers are
    // within merge_threshold of each other or whose X ranges overlap.
    let mut merged: Vec<ColumnCluster> = Vec::new();
    for col in columns {
        let should_merge = merged.last().is_some_and(|prev: &ColumnCluster| {
            (col.x_center - prev.x_center).abs() < effective_merge_threshold
                || col.x_min <= prev.x_max
        });
        if should_merge {
            let prev = merged.last_mut().unwrap();
            prev.x_min = prev.x_min.min(col.x_min);
            prev.x_max = prev.x_max.max(col.x_max);
            let total = prev.span_indices.len() as f32 + col.span_indices.len() as f32;
            prev.x_center = prev.x_center * (prev.span_indices.len() as f32 / total)
                + col.x_center * (col.span_indices.len() as f32 / total);
            prev.span_indices.extend(col.span_indices);
        } else {
            merged.push(col);
        }
    }

    // Final sort by x_center.
    merged.sort_by(|a, b| crate::utils::safe_float_cmp(a.x_center, b.x_center));
    merged
}

/// Text-edge column detection inspired by pdfplumber/Tabula "Stream" mode.
///
/// Instead of greedily clustering span X-centres, this approach:
/// 1. Collects left-edge and right-edge X positions of every span.
/// 2. Snaps nearby X values into clusters (within `snap_tolerance`).
/// 3. Keeps only X positions that appear in `min_row_count` or more distinct
///    text rows — these are consistent alignment edges.
/// 4. Returns [`ColumnCluster`]s whose centres sit at those surviving edges.
///
/// The resulting columns are fewer and more faithful to the visual grid of
/// forms that have no vector lines.
/// A short numeric cell: optional sign, digits with an optional single decimal
/// point, optional trailing `%`. Accepts `0.69`, `100`, `-1.2`, `52%`; rejects
/// words and identifiers. Used to recognise a borderless data grid.
fn is_numeric_cell(t: &str) -> bool {
    if t.is_empty() || t.len() > 8 {
        return false;
    }
    let t = t.strip_suffix('%').unwrap_or(t);
    let t = t.strip_prefix(['+', '-', '\u{2212}']).unwrap_or(t);
    let mut seen_dot = false;
    let mut seen_digit = false;
    for c in t.chars() {
        match c {
            '0'..='9' => seen_digit = true,
            '.' if !seen_dot => seen_dot = true,
            _ => return false,
        }
    }
    seen_digit
}

/// True when the column centres sit on a near-constant pitch — the signature of
/// a numeric data lattice rather than prose that happened to align. Requires
/// ≥5 columns and tolerates up to two off-pitch gaps (e.g. a wider row-label
/// column at the left edge).
/// Whether `gaps` describe a regular pitch: most of them sit in the
/// `[0.6, 1.6] * median` band around their own median.
///
/// Both callers below reason from "this table has a pitch". That premise has
/// to be checked before it is used: on an irregular layout the median is just
/// the middle of a spread of unrelated gaps, and a band around it means
/// nothing.
fn gaps_are_on_pitch(gaps: &[f32]) -> bool {
    if gaps.is_empty() {
        return false;
    }
    let mut sorted = gaps.to_vec();
    sorted.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let median = sorted[sorted.len() / 2];
    if median <= 0.0 {
        return false;
    }
    let on_pitch = gaps
        .iter()
        .filter(|&&g| g >= median * 0.6 && g <= median * 1.6)
        .count();
    // Proportional, not a fixed allowance. `on_pitch + 2 >= gaps.len()` means
    // something different at every width: with 4 gaps it lets half of them be
    // arbitrary, and with 30 it is nearly unreachable. A page then fell on one
    // side of the gate or the other according to how many incidental x
    // clusters it happened to carry — a running head, a page number, one
    // wrapped line — rather than according to whether it is a lattice.
    on_pitch * 5 >= gaps.len() * 4
}

fn is_regular_lattice(cols: &[ColumnCluster]) -> bool {
    if cols.len() < 5 {
        return false;
    }
    let mut centers: Vec<f32> = cols.iter().map(|c| c.x_center).collect();
    centers.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let gaps: Vec<f32> = centers.windows(2).map(|w| w[1] - w[0]).collect();
    gaps_are_on_pitch(&gaps)
}

fn detect_text_edge_columns(
    spans: &[TextSpan],
    config: &TableDetectionConfig,
) -> Vec<ColumnCluster> {
    if spans.is_empty() {
        return Vec::new();
    }

    let snap_tolerance = config.column_tolerance;
    let min_row_count: usize = 3;

    // --- 1. Collect (x, y_row_key) for left and right edges ----------
    // We bucket Y into rows using row_tolerance so that we can count
    // *distinct* rows per X cluster.
    let row_tol = config.row_tolerance;

    // Assign each span to a row id (simple greedy 1-D clustering on Y).
    let mut row_ids: Vec<usize> = Vec::with_capacity(spans.len());
    let mut row_centres: Vec<f32> = Vec::new();
    for span in spans {
        let y = span.bbox.center().y;
        let mut assigned = None;
        for (rid, rc) in row_centres.iter().enumerate() {
            if (y - rc).abs() < row_tol {
                assigned = Some(rid);
                break;
            }
        }
        match assigned {
            Some(rid) => row_ids.push(rid),
            None => {
                row_ids.push(row_centres.len());
                row_centres.push(y);
            },
        }
    }

    // --- 2. Build edge observations: (x_value, row_id) ---------------
    let mut edge_obs: Vec<(f32, usize)> = Vec::with_capacity(spans.len() * 2);
    for (i, span) in spans.iter().enumerate() {
        edge_obs.push((span.bbox.left(), row_ids[i]));
        edge_obs.push((span.bbox.right(), row_ids[i]));
    }
    // Sort by X for deterministic clustering.
    edge_obs.sort_by(|a, b| crate::utils::safe_float_cmp(a.0, b.0));

    // --- 3. Cluster X positions (snap within tolerance) ---------------
    struct XCluster {
        x_center: f32,
        count: usize,
        rows: Vec<usize>, // row ids (may contain duplicates; we dedupe later)
    }

    let mut x_clusters: Vec<XCluster> = Vec::new();
    for &(x, rid) in &edge_obs {
        let mut found = false;
        for cl in &mut x_clusters {
            if (x - cl.x_center).abs() < snap_tolerance {
                let n = cl.count as f32;
                cl.x_center = cl.x_center * (n / (n + 1.0)) + x / (n + 1.0);
                cl.count += 1;
                cl.rows.push(rid);
                found = true;
                break;
            }
        }
        if !found {
            x_clusters.push(XCluster {
                x_center: x,
                count: 1,
                rows: vec![rid],
            });
        }
    }

    // --- 4. Filter: keep edges that appear in >= min_row_count distinct rows
    let mut edges: Vec<f32> = Vec::new();
    for cl in &mut x_clusters {
        cl.rows.sort_unstable();
        cl.rows.dedup();
        if cl.rows.len() >= min_row_count {
            edges.push(cl.x_center);
        }
    }
    edges.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));

    // Deduplicate edges that ended up very close after averaging.
    let mut deduped: Vec<f32> = Vec::new();
    for &e in &edges {
        if deduped
            .last()
            .is_some_and(|prev| (e - prev).abs() < snap_tolerance)
        {
            // merge: keep midpoint
            let prev = deduped.last_mut().unwrap();
            *prev = (*prev + e) / 2.0;
        } else {
            deduped.push(e);
        }
    }

    // --- 5. Convert surviving edges to ColumnClusters ----------------
    // Each edge becomes a column; assign spans whose left-edge is closest.
    let mut columns: Vec<ColumnCluster> = deduped
        .iter()
        .map(|&x| ColumnCluster {
            x_center: x,
            x_min: x,
            x_max: x,
            span_indices: Vec::new(),
        })
        .collect();

    if columns.is_empty() {
        return columns;
    }

    for (idx, span) in spans.iter().enumerate() {
        let sx = span.bbox.left();
        let best = columns
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| ((sx - c.x_center).abs() * 1000.0) as i32)
            .map(|(i, _)| i)
            .unwrap_or(0);
        columns[best].span_indices.push(idx);
        columns[best].x_min = columns[best].x_min.min(sx);
        columns[best].x_max = columns[best].x_max.max(sx);
    }

    // Drop columns that received no spans.
    columns.retain(|c| !c.span_indices.is_empty());

    // Final sort.
    columns.sort_by(|a, b| crate::utils::safe_float_cmp(a.x_center, b.x_center));
    columns
}

fn detect_rows(spans: &[TextSpan], row_tolerance: f32) -> Vec<RowCluster> {
    // Sort span indices by Y coordinate before clustering for deterministic results.
    let mut sorted_indices: Vec<usize> = (0..spans.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        crate::utils::safe_float_cmp(spans[a].bbox.center().y, spans[b].bbox.center().y)
    });

    let mut rows: Vec<RowCluster> = Vec::new();
    for idx in sorted_indices {
        let y = spans[idx].bbox.center().y;
        let mut found = false;
        for row in &mut rows {
            if (y - row.y_center).abs() < row_tolerance {
                row.span_indices.push(idx);
                row.y_min = row.y_min.min(y);
                row.y_max = row.y_max.max(y);
                // Update running average (same rationale as detect_columns)
                let n = row.span_indices.len() as f32;
                row.y_center = row.y_center * ((n - 1.0) / n) + y / n;
                found = true;
                break;
            }
        }
        if !found {
            rows.push(RowCluster {
                y_center: y,
                y_min: y,
                y_max: y,
                span_indices: vec![idx],
            });
        }
    }
    rows.sort_by(|a, b| crate::utils::safe_float_cmp(b.y_center, a.y_center));
    rows
}

fn assign_spans_to_cells(
    spans: &[TextSpan],
    columns: &[ColumnCluster],
    rows: &[RowCluster],
) -> GridStructure {
    let num_cols = columns.len();
    let num_rows = rows.len();
    let mut cells: Vec<Vec<Vec<usize>>> = vec![vec![Vec::new(); num_cols]; num_rows];
    for (idx, span) in spans.iter().enumerate() {
        let span_x = span.bbox.center().x;
        let span_y = span.bbox.center().y;
        let col_idx = columns
            .iter()
            .enumerate()
            .min_by_key(|(_, col)| ((span_x - col.x_center).abs() * 1000.0) as i32)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let row_idx = rows
            .iter()
            .enumerate()
            .min_by_key(|(_, row)| ((span_y - row.y_center).abs() * 1000.0) as i32)
            .map(|(i, _)| i)
            .unwrap_or(0);
        cells[row_idx][col_idx].push(idx);
    }
    GridStructure {
        columns: columns.to_vec(),
        rows: rows.to_vec(),
        cells,
    }
}

/// Maximum number of detected columns the split-column detector can
/// analyse. Grids wider than this skip the check; extremely wide
/// candidates are rare and have other defences upstream.
const MAX_MASK_COLUMNS: usize = 128;

/// Minimum share of modal rows a column-component must contain to
/// count as "significant" for split-detection purposes. Chosen to
/// admit the original split-flow shape (the DB10 reproducer's modal
/// rows split evenly across two halves) while avoiding obvious
/// overfitting. Heuristic, not corpus-calibrated.
const MIN_SPLIT_GROUP_ROW_SHARE: f32 = 0.20;

fn validate_table_structure_internal(grid: &GridStructure, config: &TableDetectionConfig) -> bool {
    let num_cols = grid.columns.len();
    let total_cells: usize = grid
        .cells
        .iter()
        .flat_map(|row| row.iter().take(num_cols))
        .map(|cell| if cell.is_empty() { 0 } else { 1 })
        .sum();
    if total_cells < config.min_table_cells {
        return false;
    }
    let cell_counts: Vec<usize> = grid
        .cells
        .iter()
        .map(|row| {
            row.iter()
                .take(num_cols)
                .filter(|cell| !cell.is_empty())
                .count()
        })
        .collect();
    if cell_counts.is_empty() {
        return false;
    }
    let most_common_count = *cell_counts
        .iter()
        .max_by_key(|&&count| cell_counts.iter().filter(|&&c| c == count).count())
        .unwrap_or(&0);
    if most_common_count == 0 {
        return false;
    }
    let regular_rows = cell_counts
        .iter()
        .filter(|&&count| count == most_common_count)
        .count();
    if (regular_rows as f32 / cell_counts.len() as f32) < config.regular_row_ratio {
        return false;
    }

    if has_split_modal_column_groups(grid, most_common_count) {
        return false;
    }

    true
}

/// Returns `true` when the modal rows of `grid` partition into two or
/// more disconnected column-co-occurrence components, each backed by a
/// significant share of modal rows. This signature catches "two prose
/// flows mis-clustered as one table" without rejecting hierarchical
/// tables whose modal data rows are sparse but internally connected.
///
/// The check operates only on rows whose populated-cell count equals
/// `most_common_count`. For each such row, the populated columns form
/// a co-occurrence clique. The union of those cliques forms a graph
/// over columns; its connected components are computed via bitmask
/// flood-fill. If two or more components each contain at least two
/// columns and are supported by at least `MIN_SPLIT_GROUP_ROW_SHARE`
/// of the modal rows, the grid is rejected as split-flow.
///
/// Heuristic, not corpus-calibrated.
fn has_split_modal_column_groups(grid: &GridStructure, most_common_count: usize) -> bool {
    let num_cols = grid.columns.len();

    // A meaningful split needs at least 4 columns (two groups of >=2)
    // and at least 2 populated cells per modal row.
    if !(4..=MAX_MASK_COLUMNS).contains(&num_cols) || most_common_count < 2 {
        return false;
    }

    // Collect column-occupancy masks for the modal rows. Bounded by
    // `num_cols` so `most_common_count` (computed over the same bounded
    // slice upstream) and `populated` here share one column universe;
    // also keeps every `1u128 << idx` shift in range of the u128 mask.
    let modal_masks: Vec<u128> = grid
        .cells
        .iter()
        .filter_map(|row| {
            let populated = row
                .iter()
                .take(num_cols)
                .filter(|cell| !cell.is_empty())
                .count();

            if populated != most_common_count {
                return None;
            }

            let mut mask = 0u128;
            for (idx, cell) in row.iter().take(num_cols).enumerate() {
                if !cell.is_empty() {
                    mask |= 1u128 << idx;
                }
            }

            if mask.count_ones() >= 2 {
                Some(mask)
            } else {
                None
            }
        })
        .collect();

    // Need enough modal rows to make the share threshold meaningful.
    if modal_masks.len() < 4 {
        return false;
    }

    // Floor at 2 rows so a single-row outlier with a wide/narrow mask
    // can never be classified as its own "significant" component
    // — when modal_masks.len() == 4 the share alone would round to 1.
    let min_component_rows =
        (((modal_masks.len() as f32) * MIN_SPLIT_GROUP_ROW_SHARE).ceil() as usize).max(2);

    // Build column adjacency: two columns are adjacent iff they ever
    // co-occur in the same modal row.
    let mut adjacency: Vec<u128> = vec![0u128; num_cols];
    let mut active_columns: u128 = 0;

    for &mask in &modal_masks {
        active_columns |= mask;
        let mut bits = mask;
        while bits != 0 {
            let bit = bits & bits.wrapping_neg();
            let col = bit.trailing_zeros() as usize;
            adjacency[col] |= mask;
            bits &= !bit;
        }
    }

    // Walk connected components by bitmask flood-fill. Count how many
    // are "significant" (>=2 columns AND >=min_component_rows modal
    // rows containing at least one of their columns).
    let mut remaining = active_columns;
    let mut significant_components = 0usize;

    while remaining != 0 {
        let seed_bit = remaining & remaining.wrapping_neg();
        let mut component: u128 = 0;
        let mut frontier: u128 = seed_bit;

        while frontier != 0 {
            let bit = frontier & frontier.wrapping_neg();
            frontier &= !bit;

            if component & bit != 0 {
                continue;
            }

            component |= bit;
            let col = bit.trailing_zeros() as usize;
            frontier |= adjacency[col] & !component;
        }

        remaining &= !component;

        let component_cols = component.count_ones() as usize;
        let component_row_support = modal_masks
            .iter()
            .filter(|&&mask| mask & component != 0)
            .count();

        if component_cols >= 2 && component_row_support >= min_component_rows {
            significant_components += 1;
            if significant_components >= 2 {
                return true;
            }
        }
    }

    false
}

/// Backward compatibility: Indices of spans belonging to a table.
#[derive(Debug, Clone)]
pub struct DetectedTable {
    /// Indices of spans that belong to this table.
    pub span_indices: Vec<usize>,
}

/// Backward compatibility: Table detector wrapper.
pub struct SpatialTableDetector {
    /// Configuration for this detector.
    pub config: TableDetectionConfig,
}

impl SpatialTableDetector {
    /// Create a new detector with config.
    pub fn with_config(config: TableDetectionConfig) -> Self {
        Self { config }
    }
    /// Detect tables (wrapper).
    pub fn detect_tables(&self, spans: &[TextSpan]) -> Vec<DetectedTable> {
        detect_tables_from_spans_column_aware(spans, &self.config)
            .into_iter()
            .flat_map(|_| None)
            .collect()
    }
    /// Detect tables using visual lines and text (hybrid).
    pub fn detect_tables_hybrid(
        &self,
        spans: &[TextSpan],
        lines: &[crate::elements::PathContent],
    ) -> Vec<Table> {
        detect_tables_with_lines(spans, lines, &self.config)
    }
}

fn cluster_values(values: &[f32], tolerance: f32) -> Vec<f32> {
    let mut clusters: Vec<f32> = Vec::new();
    let mut counts: Vec<u32> = Vec::new();
    for &v in values {
        if let Some(idx) = clusters.iter().position(|&c| (v - c).abs() < tolerance) {
            counts[idx] += 1;
            clusters[idx] += (v - clusters[idx]) / counts[idx] as f32;
        } else {
            clusters.push(v);
            counts.push(1);
        }
    }
    clusters
}

struct LineCluster {
    lines: Vec<usize>,
    bbox: crate::geometry::Rect,
}

impl LineCluster {
    fn new(line_idx: usize, bbox: crate::geometry::Rect) -> Self {
        Self {
            lines: vec![line_idx],
            bbox,
        }
    }
    fn add(&mut self, line_idx: usize, bbox: crate::geometry::Rect) {
        self.lines.push(line_idx);
        self.bbox = self.bbox.union(&bbox);
    }
}

fn group_lines_into_clusters(
    lines: &[crate::elements::PathContent],
    config: &TableDetectionConfig,
) -> Vec<LineCluster> {
    if lines.is_empty() {
        return Vec::new();
    }
    // All clustering geometry below works on RENDERED extents: a table rule
    // encoded as a 1 pt segment with a table-height stroke width must
    // cluster with (and span) the rules its drawn bar actually touches, not
    // the ones near its geometric speck. Computed once — pure
    // arithmetic per path.
    let rendered: Vec<crate::geometry::Rect> = lines.iter().map(|p| p.rendered_bbox()).collect();
    let mut uf = UnionFind::new(lines.len());
    let mut valid_indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, path)| path.is_table_primitive())
        .map(|(i, _)| i)
        .collect();

    // Optimization: Sort by X-coordinate to enable sweep-line early exit (O(n log n))
    valid_indices.sort_by(|&a, &b| crate::utils::safe_float_cmp(rendered[a].x, rendered[b].x));

    const EXPANSION: f32 = 3.0;
    for i in 0..valid_indices.len() {
        let idx_a = valid_indices[i];
        let bbox_a = &rendered[idx_a];
        let expanded_a = crate::geometry::Rect::new(
            bbox_a.x - EXPANSION,
            bbox_a.y - EXPANSION,
            bbox_a.width + EXPANSION * 2.0,
            bbox_a.height + EXPANSION * 2.0,
        );

        for j in (i + 1)..valid_indices.len() {
            let idx_b = valid_indices[j];
            let bbox_b = &rendered[idx_b];

            // Optimization: If the next path's X-start is beyond our search threshold,
            // no subsequent paths in the sorted list can possibly intersect.
            if bbox_b.x > expanded_a.x + expanded_a.width {
                break;
            }

            let expanded_b = crate::geometry::Rect::new(
                bbox_b.x - EXPANSION,
                bbox_b.y - EXPANSION,
                bbox_b.width + EXPANSION * 2.0,
                bbox_b.height + EXPANSION * 2.0,
            );

            if expanded_a.intersects(&expanded_b) {
                uf.union(idx_a, idx_b);
            }
        }
    }
    let mut cluster_map: HashMap<usize, LineCluster> = HashMap::new();
    for i in valid_indices {
        let root = uf.find(i);
        let bbox = rendered[i];
        cluster_map
            .entry(root)
            .and_modify(|c| c.add(i, bbox))
            .or_insert_with(|| LineCluster::new(i, bbox));
    }

    // Post-processing: split clusters whose vertical lines occupy distinct Y-ranges.
    // This prevents a small bordered table (e.g. an invoice header) from merging
    // with a large main table that happens to be nearby vertically.
    // Deterministic order: `cluster_map` is a HashMap (per-process-randomized
    // iteration), so sort clusters by their first (smallest) line index — each
    // cluster's `lines` Vec is already ascending — to keep downstream table
    // boundary order stable across runs.
    let mut raw_clusters: Vec<LineCluster> = cluster_map.into_values().collect();
    raw_clusters.sort_by_key(|c| c.lines.first().copied().unwrap_or(usize::MAX));
    let mut result: Vec<LineCluster> = Vec::with_capacity(raw_clusters.len());
    const LINE_AXIS_TOL: f32 = 2.0;
    let v_split_gap = config.v_split_gap;

    for cluster in raw_clusters {
        // Collect Y-ranges of vertical lines in this cluster.
        let mut v_ranges: Vec<(usize, f32, f32)> = Vec::new(); // (line_idx, y_min, y_max)
        for &idx in &cluster.lines {
            let path = &lines[idx];
            if path.is_vertical_line(LINE_AXIS_TOL) && rendered[idx].height.abs() > 5.0 {
                let y_min = rendered[idx].y;
                let y_max = rendered[idx].y + rendered[idx].height;
                let (y_min, y_max) = if y_min <= y_max {
                    (y_min, y_max)
                } else {
                    (y_max, y_min)
                };
                v_ranges.push((idx, y_min, y_max));
            }
        }

        // Need at least 2 V-lines in different ranges to consider splitting.
        if v_ranges.len() < 2 {
            result.push(cluster);
            continue;
        }

        // Sort V-lines by their y_min and group into non-overlapping Y-range bands.
        v_ranges.sort_by(|a, b| crate::utils::safe_float_cmp(a.1, b.1));
        let mut bands: Vec<(f32, f32)> = Vec::new(); // merged Y-range bands
        let mut band_start = v_ranges[0].1;
        let mut band_end = v_ranges[0].2;
        for &(_, y_min, y_max) in &v_ranges[1..] {
            if y_min > band_end + v_split_gap {
                bands.push((band_start, band_end));
                band_start = y_min;
                band_end = y_max;
            } else {
                band_end = band_end.max(y_max);
            }
        }
        bands.push((band_start, band_end));

        if bands.len() < 2 {
            // All V-lines share one contiguous Y-range; no split needed.
            result.push(cluster);
            continue;
        }

        // Split: assign each line in the cluster to the band it best fits.
        let mut sub_clusters: Vec<Vec<usize>> = vec![Vec::new(); bands.len()];
        for &idx in &cluster.lines {
            let bbox = &rendered[idx];
            let line_y_mid = bbox.y + bbox.height * 0.5;
            // Find the band whose range contains (or is closest to) the line's midpoint.
            let mut best_band = 0;
            let mut best_dist = f32::MAX;
            for (bi, &(b_min, b_max)) in bands.iter().enumerate() {
                let dist = if line_y_mid >= b_min && line_y_mid <= b_max {
                    0.0
                } else {
                    (line_y_mid - b_min).abs().min((line_y_mid - b_max).abs())
                };
                if dist < best_dist {
                    best_dist = dist;
                    best_band = bi;
                }
            }
            sub_clusters[best_band].push(idx);
        }

        // Build LineCluster from each non-empty sub-cluster.
        for sub in sub_clusters {
            if sub.is_empty() {
                continue;
            }
            let first_bbox = rendered[sub[0]];
            let mut lc = LineCluster::new(sub[0], first_bbox);
            for &idx in &sub[1..] {
                lc.add(idx, rendered[idx]);
            }
            result.push(lc);
        }
    }

    result
}

/// (C) WS0.3b — detect a header text row sitting just ABOVE the grid's top
/// ruling (a header boxed only by the page-top, or unruled above the grid).
///
/// `row_ys` is sorted descending (`row_ys[0]` = the grid's top edge) and
/// `col_xs` sorted ascending (the detected column boundaries). If a tight band
/// immediately above the top ruling — up to ~1.5x the median grid row height —
/// holds text whose cells align to the EXISTING columns and that both spans
/// at least 2 distinct columns and reaches across at least half the extent, this
/// returns the y of a new top boundary that brackets exactly that header row.
/// Otherwise `None` (table unchanged).
///
/// The distinct-columns + horizontal-span gates keep a centred title or a
/// left-aligned caption above an already-correct table from being mistaken for
/// a header row (their words cluster together instead of spanning the columns),
/// so this never alters a table that already detects correctly.
fn detect_header_row_above(spans: &[TextSpan], row_ys: &[f32], col_xs: &[f32]) -> Option<f32> {
    /// Header band reaches at most this multiple of the median row height above
    /// the grid's top ruling.
    const WINDOW_ROWS: f32 = 1.5;
    /// Header cells must reach across at least this fraction of the column
    /// extent (rejects titles/captions whose words cluster together).
    const MIN_SPAN_FRAC: f32 = 0.5;

    if row_ys.len() < 2 || col_xs.len() < 2 {
        return None;
    }
    let grid_top = row_ys[0];
    let col_lo = *col_xs.first().unwrap();
    let col_hi = *col_xs.last().unwrap();
    let col_extent = col_hi - col_lo;
    if col_extent <= 0.0 {
        return None;
    }

    // Median grid row height from consecutive H-ruling gaps.
    let mut gaps: Vec<f32> = row_ys.windows(2).map(|w| (w[0] - w[1]).abs()).collect();
    gaps.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    let median_row_h = gaps[gaps.len() / 2];
    if median_row_h <= 0.0 {
        return None;
    }
    let window_top = grid_top + WINDOW_ROWS * median_row_h;

    // Gather aligned candidate spans in the band immediately above the top ruling.
    let mut cols_hit: Vec<usize> = Vec::new();
    let mut header_max_top = f32::NEG_INFINITY;
    let mut cx_min = f32::INFINITY;
    let mut cx_max = f32::NEG_INFINITY;
    let mut cy_min = f32::INFINITY;
    let mut cy_max = f32::NEG_INFINITY;
    for span in spans {
        let cy = span.bbox.center().y;
        if cy <= grid_top || cy > window_top {
            continue;
        }
        let cx = span.bbox.center().x;
        if cx < col_lo || cx > col_hi {
            continue; // outside the column extent → not aligned to a column
        }
        let Some(ci) = (0..col_xs.len() - 1).find(|&c| cx >= col_xs[c] && cx <= col_xs[c + 1])
        else {
            continue;
        };
        if !cols_hit.contains(&ci) {
            cols_hit.push(ci);
        }
        header_max_top = header_max_top.max(span.bbox.y + span.bbox.height);
        cx_min = cx_min.min(cx);
        cx_max = cx_max.max(cx);
        cy_min = cy_min.min(cy);
        cy_max = cy_max.max(cy);
    }

    // Gate: >= 2 distinct aligned columns, spanning >= half the column extent,
    // and forming a single tight row.
    if cols_hit.len() < 2 {
        return None;
    }
    if (cx_max - cx_min) < MIN_SPAN_FRAC * col_extent {
        return None;
    }
    if (cy_max - cy_min) > median_row_h {
        return None;
    }

    // New top boundary sits just above the header text so its row band brackets
    // exactly the header spans.
    Some(header_max_top + 1.0)
}

fn detect_tables_in_cluster(
    spans: &[TextSpan],
    all_lines: &[crate::elements::PathContent],
    cluster: &LineCluster,
    config: &TableDetectionConfig,
) -> Vec<Table> {
    const MIN_LINE_LENGTH: f32 = 5.0;
    const LINE_AXIS_TOL: f32 = 2.0;
    let mut h_ys: Vec<f32> = Vec::new();
    let mut v_xs: Vec<f32> = Vec::new();
    for &idx in &cluster.lines {
        let path = &all_lines[idx];
        // Rendered extents: a stroke-width-encoded rule's center and length
        // come from the drawn bar, not the geometric speck.
        let bbox = path.rendered_bbox();
        if path.is_horizontal_line(LINE_AXIS_TOL) && bbox.width > MIN_LINE_LENGTH {
            h_ys.push(bbox.center().y);
        }
        if path.is_vertical_line(LINE_AXIS_TOL) && bbox.height.abs() > MIN_LINE_LENGTH {
            v_xs.push(bbox.center().x);
        }
    }
    let mut row_ys = cluster_values(&h_ys, config.row_tolerance);
    let mut col_xs = cluster_values(&v_xs, config.column_tolerance);
    if row_ys.len() < 2 || col_xs.len() < 2 {
        return Vec::new();
    }
    row_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a));
    col_xs.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    // (C) WS0.3b — include a header text row sitting just ABOVE the grid's top
    // ruling (boxed by page-top / unruled above the grid) when its cells align
    // to the already-detected columns. Adds ONE row boundary above the top and
    // widens the span-assignment region to reach it; never adds columns. Returns
    // `None` (unchanged) unless the tight gate in `detect_header_row_above` holds.
    let mut assign_bbox = cluster.bbox;
    let mut inserted_header_row = false;
    if let Some(header_top) = detect_header_row_above(spans, &row_ys, &col_xs) {
        row_ys.insert(0, header_top);
        inserted_header_row = true;
        let new_height = (header_top - assign_bbox.y).max(assign_bbox.height);
        assign_bbox =
            crate::geometry::Rect::new(assign_bbox.x, assign_bbox.y, assign_bbox.width, new_height);
    }
    let num_rows = row_ys.len() - 1;
    let num_cols = col_xs.len() - 1;
    if num_cols < config.min_table_columns || num_cols > config.max_table_columns {
        return Vec::new();
    }
    let mut cells: Vec<Vec<Vec<usize>>> = vec![vec![Vec::new(); num_cols]; num_rows];
    let mut assigned_any = false;
    for (orig_idx, span) in spans.iter().enumerate() {
        if !assign_bbox.intersects(&span.bbox) {
            continue;
        }
        let cx = span.bbox.center().x;
        let cy = span.bbox.center().y;
        let row_idx = (0..num_rows).find(|&r| cy <= row_ys[r] && cy >= row_ys[r + 1]);
        let col_idx = (0..num_cols).find(|&c| cx >= col_xs[c] && cx <= col_xs[c + 1]);
        if let (Some(r), Some(c)) = (row_idx, col_idx) {
            cells[r][c].push(orig_idx);
            assigned_any = true;
        }
    }
    if !assigned_any {
        return Vec::new();
    }
    let columns: Vec<ColumnCluster> = (0..num_cols)
        .map(|c| ColumnCluster {
            x_center: (col_xs[c] + col_xs[c + 1]) / 2.0,
            x_min: col_xs[c],
            x_max: col_xs[c + 1],
            span_indices: Vec::new(),
        })
        .collect();
    let all_rows: Vec<RowCluster> = (0..num_rows)
        .map(|r| RowCluster {
            y_center: (row_ys[r] + row_ys[r + 1]) / 2.0,
            y_min: row_ys[r + 1],
            y_max: row_ys[r],
            span_indices: Vec::new(),
        })
        .collect();
    let grid_full = GridStructure {
        columns: columns.clone(),
        rows: all_rows.clone(),
        cells: cells.clone(),
    };
    let mut tables = Vec::new();
    let mut current_start_row = 0;
    while current_start_row < num_rows {
        if grid_full.is_row_empty(current_start_row) {
            current_start_row += 1;
            continue;
        }
        let mut current_end_row = current_start_row;
        while current_end_row < num_rows {
            if grid_full.is_row_empty(current_end_row) {
                break;
            }
            current_end_row += 1;
        }
        if current_end_row > current_start_row {
            let sub_cells = cells[current_start_row..current_end_row].to_vec();
            let sub_rows = all_rows[current_start_row..current_end_row].to_vec();
            let mut grid = GridStructure {
                columns: columns.clone(),
                rows: sub_rows,
                cells: sub_cells,
            };
            grid = grid.trim_empty_columns();
            if validate_table_structure_internal(&grid, config) {
                // (C) WS0.3b — the inserted header row (global row 0) lives in the
                // first non-empty run, so it is local row 0 of this sub-table when
                // `current_start_row == 0`. Protect it from colspan merging.
                let protected_header_rows =
                    usize::from(inserted_header_row && current_start_row == 0);
                let mut table = grid_to_table(
                    &grid,
                    spans,
                    Some(detect_merged_cells_visually(
                        &grid,
                        spans,
                        cluster,
                        all_lines,
                        protected_header_rows,
                    )),
                );
                let mut min_y = f32::INFINITY;
                let mut max_y = f32::NEG_INFINITY;
                for r in &grid.rows {
                    min_y = min_y.min(r.y_min);
                    max_y = max_y.max(r.y_max);
                }
                table.bbox = Some(crate::geometry::Rect::new(
                    cluster.bbox.x,
                    min_y,
                    cluster.bbox.width,
                    max_y - min_y,
                ));
                let mut header_rows_detected = 0;
                let table_width = cluster.bbox.width;
                for r in 0..table.rows.len().min(3) {
                    let row_bottom = grid.rows[r].y_min;
                    let has_separator = cluster.lines.iter().any(|&idx| {
                        let path = &all_lines[idx];
                        let rendered = path.rendered_bbox();
                        path.is_horizontal_line(LINE_AXIS_TOL)
                            && rendered.width > table_width * 0.8
                            && (rendered.center().y - row_bottom).abs() < config.row_tolerance
                    });
                    if has_separator {
                        header_rows_detected = r + 1;
                    } else if r == 0 && table.rows[r].has_colspan() {
                        header_rows_detected = 1;
                    } else {
                        break;
                    }
                }
                if header_rows_detected > 0 {
                    table.has_header = true;
                    for r in 0..header_rows_detected {
                        if r < table.rows.len() {
                            table.rows[r].is_header = true;
                            for cell in &mut table.rows[r].cells {
                                cell.is_header = true;
                            }
                        }
                    }
                }
                tables.push(table);
            }
        }
        current_start_row = current_end_row + 1;
    }
    tables
}

fn detect_merged_cells_visually(
    grid: &GridStructure,
    spans: &[TextSpan],
    cluster: &LineCluster,
    all_lines: &[crate::elements::PathContent],
    protected_header_rows: usize,
) -> Vec<Vec<CellMergeInfo>> {
    let num_rows = grid.cells.len();
    let num_cols = grid.columns.len();
    const LINE_TOLERANCE: f32 = 2.0;
    let mut merge_info: Vec<Vec<CellMergeInfo>> = (0..num_rows)
        .map(|_| {
            (0..num_cols)
                .map(|_| CellMergeInfo {
                    colspan: 1,
                    rowspan: 1,
                    covered: false,
                })
                .collect()
        })
        .collect();
    for r in 0..num_rows {
        // (C) WS0.3b — a header row reconstructed from the unruled strip ABOVE
        // the grid has no vertical rulings in its band, which would otherwise
        // colspan-merge its distinct column cells into one (dropping every cell
        // but the first). Its cells were already verified to align to separate
        // columns, so skip colspan merging for these leading rows.
        if r < protected_header_rows {
            continue;
        }
        let mut c = 0;
        while c < num_cols {
            if merge_info[r][c].covered {
                c += 1;
                continue;
            }
            let mut colspan = 1;
            let mut cell_text_width: f32 = 0.0;
            for &idx in &grid.cells[r][c] {
                cell_text_width = cell_text_width.max(spans[idx].bbox.width);
            }
            let mut total_cell_width = grid.columns[c].x_max - grid.columns[c].x_min;
            for next_c in (c + 1)..num_cols {
                let separator_x = grid.columns[next_c].x_min;
                let y_min = grid.rows[r].y_min;
                let y_max = grid.rows[r].y_max;
                let has_separator = cluster.lines.iter().any(|&idx| {
                    let path = &all_lines[idx];
                    // Rendered extents: a stroke-width-encoded column rule
                    // crosses every row its drawn bar spans, not just the
                    // band around its geometric midline.
                    let rendered = path.rendered_bbox();
                    path.is_vertical_line(LINE_TOLERANCE)
                        && (rendered.center().x - separator_x).abs() < LINE_TOLERANCE
                        && rendered.y < y_max
                        && (rendered.y + rendered.height) > y_min
                });
                if !has_separator || (cell_text_width > total_cell_width + 2.0) {
                    colspan += 1;
                    total_cell_width += grid.columns[next_c].x_max - grid.columns[next_c].x_min;
                } else {
                    break;
                }
            }
            if colspan > 1 {
                merge_info[r][c].colspan = colspan;
                for i in 1..colspan {
                    merge_info[r][c + i as usize].covered = true;
                }
            }
            c += colspan as usize;
        }
    }
    for c in 0..num_cols {
        let mut r = 0;
        while r < num_rows {
            if merge_info[r][c].covered {
                r += 1;
                continue;
            }
            let mut rowspan = 1;
            let current_colspan = merge_info[r][c].colspan;
            for next_r in (r + 1)..num_rows {
                let separator_y = grid.rows[next_r].y_max;
                let x_min = grid.columns[c].x_min;
                let x_max = grid.columns[c + current_colspan as usize - 1].x_max;
                let has_separator = cluster.lines.iter().any(|&idx| {
                    let path = &all_lines[idx];
                    // Rendered extents, mirroring the colspan check above: a
                    // row rule encoded as a short vertical segment with a
                    // table-width stroke spans every column its drawn bar
                    // crosses.
                    let rendered = path.rendered_bbox();
                    path.is_horizontal_line(LINE_TOLERANCE)
                        && (rendered.center().y - separator_y).abs() < LINE_TOLERANCE
                        && rendered.x < x_max
                        && (rendered.x + rendered.width) > x_min
                });
                if !has_separator {
                    rowspan += 1;
                } else {
                    break;
                }
            }
            if rowspan > 1 {
                merge_info[r][c].rowspan = rowspan;
                for i in 1..rowspan {
                    merge_info[r + i as usize][c].covered = true;
                    for j in 1..current_colspan {
                        merge_info[r + i as usize][c + j as usize].covered = true;
                    }
                }
            }
            r += rowspan as usize;
        }
    }
    merge_info
}

// ---------------------------------------------------------------------------
// Intersection-based table detection (Tabula/pdfplumber/PyMuPDF pipeline)
// ---------------------------------------------------------------------------

/// Snap tolerance: parallel lines within this distance share a coordinate.
const SNAP_TOL: f32 = 3.0;
/// Join tolerance: collinear segments within this gap are merged.
const JOIN_TOL: f32 = 3.0;
/// Minimum edge length after merging; shorter edges are discarded.
const MIN_EDGE_LEN: f32 = 5.0;
/// Minimum number of short segments at the same coordinate to consider them a
/// dotted/dashed line candidate.
const DOTTED_MIN_SEGMENTS: usize = 3;
/// Minimum total span (in pt) of collinear short segments to reconstitute them
/// as a single continuous edge.
const DOTTED_MIN_SPAN: f32 = 50.0;
/// Snap precision for grouping dotted-line segments by coordinate (0.1 pt).
const DOTTED_COORD_SNAP: f32 = 10.0; // multiplier: coord * DOTTED_COORD_SNAP → i32 key

/// A horizontal or vertical edge (segment).
#[derive(Debug, Clone, Copy)]
struct Edge {
    /// For H edges: the shared y coordinate. For V edges: the shared x coordinate.
    coord: f32,
    /// Start of the range (min x for H, min y for V).
    start: f32,
    /// End of the range (max x for H, max y for V).
    end: f32,
}

/// An intersection point on the grid.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Intersection {
    x: f32,
    y: f32,
}

/// A rectangular cell defined by four corner intersections.
#[derive(Debug, Clone, Copy)]
struct IntersectionCell {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

/// Extract horizontal and vertical edges from path content, decomposing rectangles.
fn extract_edges(lines: &[crate::elements::PathContent]) -> (Vec<Edge>, Vec<Edge>) {
    const LINE_AXIS_TOL: f32 = 2.0;
    let mut h_edges: Vec<Edge> = Vec::new();
    let mut v_edges: Vec<Edge> = Vec::new();

    for path in lines {
        let bbox = &path.bbox;
        if path.is_horizontal_line(LINE_AXIS_TOL) {
            // Rendered extents so a stroke-width-encoded rule contributes
            // the edge its drawn bar covers, not its geometric speck
            // Identical to `bbox` for ordinary thin rules.
            let rendered = path.rendered_bbox();
            h_edges.push(Edge {
                coord: rendered.center().y,
                start: rendered.left(),
                end: rendered.right(),
            });
        } else if path.is_vertical_line(LINE_AXIS_TOL) {
            let rendered = path.rendered_bbox();
            v_edges.push(Edge {
                coord: rendered.center().x,
                start: rendered.top(),
                end: rendered.bottom(),
            });
        } else if path.is_rectangle() {
            // Decompose rectangle into 4 edges.
            let (l, r, t, b) = (bbox.left(), bbox.right(), bbox.top(), bbox.bottom());
            h_edges.push(Edge {
                coord: t,
                start: l,
                end: r,
            });
            h_edges.push(Edge {
                coord: b,
                start: l,
                end: r,
            });
            v_edges.push(Edge {
                coord: l,
                start: t,
                end: b,
            });
            v_edges.push(Edge {
                coord: r,
                start: t,
                end: b,
            });
        }
    }
    (h_edges, v_edges)
}

/// Snap parallel edges within `SNAP_TOL` to the same coordinate, join collinear
/// segments within `JOIN_TOL`, and discard edges shorter than `MIN_EDGE_LEN`.
fn snap_and_merge(edges: &mut Vec<Edge>) {
    snap_edges(edges);
    join_collinear_edges(edges);
    reconstitute_dotted_lines(edges);
}

/// Phase 1: Sort edges by coord and snap nearby coordinates (within `SNAP_TOL`)
/// to the first coordinate in each group.
fn snap_edges(edges: &mut [Edge]) {
    if edges.is_empty() {
        return;
    }
    // Sort by coord so nearby lines are adjacent.
    edges.sort_by(|a, b| crate::utils::safe_float_cmp(a.coord, b.coord));

    let mut i = 0;
    while i < edges.len() {
        let base_coord = edges[i].coord;
        let mut j = i + 1;
        while j < edges.len() && (edges[j].coord - base_coord).abs() <= SNAP_TOL {
            edges[j].coord = base_coord;
            j += 1;
        }
        i = j;
    }
}

/// Phase 2: Sort by (coord, start) and merge overlapping or adjacent collinear
/// segments into single edges.
fn join_collinear_edges(edges: &mut Vec<Edge>) {
    if edges.is_empty() {
        return;
    }
    // Sort by coord then start so a single sweep handles chains of touching
    // segments regardless of the order they were originally collected.
    edges.sort_by(|a, b| {
        crate::utils::safe_float_cmp(a.coord, b.coord)
            .then_with(|| crate::utils::safe_float_cmp(a.start, b.start))
    });

    let mut merged: Vec<Edge> = Vec::new();
    for &edge in edges.iter() {
        // Use SNAP_TOL for the coord comparison (not f32::EPSILON) so that
        // edges whose coords were snapped from slightly different originals
        // still join correctly.
        let should_merge = merged.last().is_some_and(|prev: &Edge| {
            (prev.coord - edge.coord).abs() <= SNAP_TOL && edge.start <= prev.end + JOIN_TOL
        });
        if should_merge {
            let prev = merged.last_mut().unwrap();
            prev.end = prev.end.max(edge.end);
        } else {
            merged.push(edge);
        }
    }

    *edges = merged;
}

/// Phase 3: Group short segments (below `MIN_EDGE_LEN`) by coordinate.  When a
/// group has >= `DOTTED_MIN_SEGMENTS` members spanning >= `DOTTED_MIN_SPAN`
/// points, replace them with a single long edge. Short segments that do not
/// qualify are discarded.
fn reconstitute_dotted_lines(edges: &mut Vec<Edge>) {
    let mut dotted_groups: HashMap<i32, Vec<Edge>> = HashMap::new();
    let mut long_edges: Vec<Edge> = Vec::new();

    for &edge in edges.iter() {
        if (edge.end - edge.start) >= MIN_EDGE_LEN {
            long_edges.push(edge);
        } else {
            let key = (edge.coord * DOTTED_COORD_SNAP).round() as i32;
            dotted_groups.entry(key).or_default().push(edge);
        }
    }

    // Iterate in sorted key order: `dotted_groups` is a HashMap (per-process-
    // randomized), and the reconstituted edges are appended to `long_edges`
    // (which becomes `*edges`), so HashMap order would leak into edge order and,
    // downstream, table-cell/region order. Sorting the snapped-coordinate keys
    // makes it deterministic.
    let mut dotted_keys: Vec<i32> = dotted_groups.keys().copied().collect();
    dotted_keys.sort_unstable();
    for key in dotted_keys {
        let segments = &dotted_groups[&key];
        if segments.len() >= DOTTED_MIN_SEGMENTS {
            let min_start = segments
                .iter()
                .map(|e| e.start)
                .min_by(|a, b| crate::utils::safe_float_cmp(*a, *b))
                .unwrap();
            let max_end = segments
                .iter()
                .map(|e| e.end)
                .max_by(|a, b| crate::utils::safe_float_cmp(*a, *b))
                .unwrap();
            let total_span = max_end - min_start;
            if total_span >= DOTTED_MIN_SPAN {
                // Use the coordinate of the first segment (they are all snapped
                // to the same value within SNAP_TOL anyway).
                long_edges.push(Edge {
                    coord: segments[0].coord,
                    start: min_start,
                    end: max_end,
                });
            }
        }
    }

    // No additional short-edge discard needed: long_edges already excludes
    // short segments that were not reconstituted.
    *edges = long_edges;
}

/// Remove orphan edges that have no plausible counterpart in the other axis.
///
/// For each H-edge, keep it only if at least one V-edge has an X coordinate
/// within the H-edge's X-range (with generous tolerance).
/// For each V-edge, keep it only if at least one H-edge has an X-range that
/// overlaps with the V-edge's X coordinate (with generous tolerance).
///
/// This is purely an X-range overlap check. The Y-axis relationship is
/// intentionally ignored because the extended grid projects V-line X positions
/// across all H-line Y positions regardless of whether they share Y ranges
/// (e.g., Census tables where H-lines and V-lines occupy different Y regions).
fn filter_edges_by_coverage(h_edges: &mut Vec<Edge>, v_edges: &mut Vec<Edge>) {
    // Compute a generous X-axis tolerance: 50% of the total X span of all edges.
    let all_x_min = h_edges
        .iter()
        .map(|e| e.start)
        .chain(v_edges.iter().map(|e| e.coord))
        .fold(f32::INFINITY, f32::min);
    let all_x_max = h_edges
        .iter()
        .map(|e| e.end)
        .chain(v_edges.iter().map(|e| e.coord))
        .fold(f32::NEG_INFINITY, f32::max);
    let x_span = (all_x_max - all_x_min).max(1.0);
    let x_tol = x_span * 0.5;

    // Keep H-edges that have at least one V-edge whose X coord falls within
    // [start - x_tol, end + x_tol].
    h_edges.retain(|h| {
        v_edges
            .iter()
            .any(|v| v.coord >= h.start - x_tol && v.coord <= h.end + x_tol)
    });

    // Keep V-edges that have at least one H-edge whose X-range overlaps
    // with this V-edge's X coordinate (within tolerance).
    v_edges.retain(|v| {
        h_edges
            .iter()
            .any(|h| v.coord >= h.start - x_tol && v.coord <= h.end + x_tol)
    });
}

/// Find all intersection points where an H edge and a V edge actually cross.
fn find_intersections(h_edges: &[Edge], v_edges: &[Edge]) -> Vec<Intersection> {
    let mut pts: Vec<Intersection> = Vec::new();
    for h in h_edges {
        for v in v_edges {
            // H edge spans x=[h.start, h.end] at y=h.coord
            // V edge spans y=[v.start, v.end] at x=v.coord
            if v.coord >= h.start - SNAP_TOL
                && v.coord <= h.end + SNAP_TOL
                && h.coord >= v.start - SNAP_TOL
                && h.coord <= v.end + SNAP_TOL
            {
                pts.push(Intersection {
                    x: v.coord,
                    y: h.coord,
                });
            }
        }
    }
    // Deduplicate (snap-level)
    pts.sort_by(|a, b| {
        crate::utils::safe_float_cmp(a.x, b.x).then_with(|| crate::utils::safe_float_cmp(a.y, b.y))
    });
    pts.dedup_by(|a, b| (a.x - b.x).abs() <= SNAP_TOL && (a.y - b.y).abs() <= SNAP_TOL);
    pts
}

/// Build cells from intersection points.
/// A cell exists when all four corners (x1,y1), (x2,y1), (x1,y2), (x2,y2) are present
/// and there is no intermediate intersection between them on either axis.
fn build_cells_from_intersections(pts: &[Intersection]) -> Vec<IntersectionCell> {
    use std::collections::BTreeSet;

    // Collect unique sorted X and Y coordinates.
    let mut xs: Vec<f32> = pts.iter().map(|p| p.x).collect();
    let mut ys: Vec<f32> = pts.iter().map(|p| p.y).collect();
    xs.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    xs.dedup_by(|a, b| (*a - *b).abs() <= SNAP_TOL);
    ys.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= SNAP_TOL);

    // Build a fast lookup: quantize to (xi, yi) indices.
    let x_idx = |xv: f32| -> Option<usize> { xs.iter().position(|&c| (c - xv).abs() <= SNAP_TOL) };
    let y_idx = |yv: f32| -> Option<usize> { ys.iter().position(|&c| (c - yv).abs() <= SNAP_TOL) };

    let nx = xs.len();
    let ny = ys.len();
    // present[yi * nx + xi] = true if intersection exists
    let mut present: BTreeSet<usize> = BTreeSet::new();
    for p in pts {
        if let (Some(xi), Some(yi)) = (x_idx(p.x), y_idx(p.y)) {
            present.insert(yi * nx + xi);
        }
    }

    let has = |xi: usize, yi: usize| -> bool { present.contains(&(yi * nx + xi)) };

    let mut cells = Vec::new();
    for yi in 0..ny {
        for xi in 0..nx {
            if !has(xi, yi) {
                continue;
            }
            // Find next X with an intersection on the same Y row.
            let next_xi = ((xi + 1)..nx).find(|&nxi| has(nxi, yi));
            // Find next Y with an intersection on the same X column.
            let next_yi = ((yi + 1)..ny).find(|&nyi| has(xi, nyi));

            if let (Some(nxi), Some(nyi)) = (next_xi, next_yi) {
                // Check diagonal corner exists.
                if has(nxi, nyi) {
                    cells.push(IntersectionCell {
                        x1: xs[xi],
                        y1: ys[yi],
                        x2: xs[nxi],
                        y2: ys[nyi],
                    });
                }
            }
        }
    }
    cells
}

/// Build grid cells from the Cartesian product of H-edge Y-positions and V-edge X-positions.
///
/// This "extended grid" approach handles the case where horizontal and vertical lines
/// don't physically intersect (e.g., H-lines in a header area and V tick marks in a data
/// area). Instead of requiring actual crossings, we project every unique V-line X coordinate
/// across every unique H-line Y coordinate to create virtual grid intersections.
fn build_extended_grid_cells(h_edges: &[Edge], v_edges: &[Edge]) -> Vec<IntersectionCell> {
    // Collect unique Y positions from H edges (the row boundaries).
    let mut ys: Vec<f32> = h_edges.iter().map(|e| e.coord).collect();
    ys.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    ys.dedup_by(|a, b| (*a - *b).abs() <= SNAP_TOL);

    // Collect unique X positions from V edges (the column boundaries).
    let mut xs: Vec<f32> = v_edges.iter().map(|e| e.coord).collect();
    xs.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    xs.dedup_by(|a, b| (*a - *b).abs() <= SNAP_TOL);

    if xs.len() < 2 || ys.len() < 2 {
        return Vec::new();
    }

    // Build cells from every adjacent pair of X and Y values.
    let mut cells = Vec::new();
    for yi in 0..ys.len() - 1 {
        for xi in 0..xs.len() - 1 {
            cells.push(IntersectionCell {
                x1: xs[xi],
                y1: ys[yi],
                x2: xs[xi + 1],
                y2: ys[yi + 1],
            });
        }
    }
    cells
}

/// Group cells that share edges into tables using union-find.
fn group_cells_into_tables(cells: &[IntersectionCell]) -> Vec<Vec<usize>> {
    if cells.is_empty() {
        return Vec::new();
    }
    let n = cells.len();
    let mut uf = UnionFind::new(n);

    // Sweep-line prune for the O(n²) edge-adjacency scan (the hot loop on dense
    // ruled pages — CFR regulatory megafiles, #26). BOTH adjacency tests below
    // require the cells' y-extents to touch within SNAP_TOL: horizontal
    // adjacency needs y1≈y1 (so cj.y1 ≤ ci.y2 + SNAP_TOL), and vertical
    // adjacency needs cj.y1 ≈ ci.y2 (also ≤ ci.y2 + SNAP_TOL). Iterating cells
    // in ascending-y1 order lets us `break` the inner loop once a candidate's
    // y1 clears ci.y2 + SNAP_TOL — every later candidate has an even larger y1
    // and cannot share an edge. We `union` by ORIGINAL index, and union is
    // order-independent, so the resulting partition is byte-identical to the
    // full O(n²) scan; only provably-non-adjacent pairs are skipped.
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| crate::utils::safe_float_cmp(cells[a].y1, cells[b].y1));
    for a in 0..n {
        let i = order[a];
        let ci = &cells[i];
        let y_limit = ci.y2 + SNAP_TOL;
        for &j in order.iter().skip(a + 1) {
            let cj = &cells[j];
            if cj.y1 > y_limit {
                break; // sorted by y1 → no later cell can touch ci's y-extent
            }
            let shares_edge = // Horizontal adjacency (share a vertical edge)
                (((ci.x2 - cj.x1).abs() <= SNAP_TOL || (ci.x1 - cj.x2).abs() <= SNAP_TOL)
                    && (ci.y1 - cj.y1).abs() <= SNAP_TOL
                    && (ci.y2 - cj.y2).abs() <= SNAP_TOL)
                || // Vertical adjacency (share a horizontal edge)
                (((ci.y2 - cj.y1).abs() <= SNAP_TOL || (ci.y1 - cj.y2).abs() <= SNAP_TOL)
                    && (ci.x1 - cj.x1).abs() <= SNAP_TOL
                    && (ci.x2 - cj.x2).abs() <= SNAP_TOL);
            if shares_edge {
                uf.union(i, j);
            }
        }
    }

    // Collect groups in a DETERMINISTIC order. `groups()` returns a HashMap
    // whose iteration order is randomized per-process (Rust `RandomState`), so
    // `into_values().collect()` would yield table clusters in a different order
    // each run — leaking non-deterministic reading order on multi-table / figure
    // pages (e.g. matrix-figure pages with several detected regions). Each
    // group's `Vec` is already ascending (built `for i in 0..n`); sort the outer
    // list by each group's first (smallest) cell index so table order is stable.
    let mut groups: Vec<Vec<usize>> = uf.groups().into_values().collect();
    groups.sort_by_key(|g| g.first().copied().unwrap_or(usize::MAX));
    groups
}

/// Split table rows that contain text spans at multiple distinct Y positions into sub-rows.
///
/// This handles the hybrid case where column boundaries come from vertical lines but there
/// are no horizontal lines between individual rows. In that scenario the intersection-based
/// pipeline produces a single mega-row; this function detects multiple Y-clusters within
/// each row and splits accordingly.
fn split_rows_by_text_positions(
    table_rows: Vec<TableRow>,
    row_cell_span_indices: &[Vec<Vec<usize>>],
    spans: &[TextSpan],
    config: &TableDetectionConfig,
) -> Vec<TableRow> {
    let mut result: Vec<TableRow> = Vec::new();

    for (row_idx, row) in table_rows.into_iter().enumerate() {
        let cell_indices = &row_cell_span_indices[row_idx];

        // Collect all Y-centers from every span in this row across all columns.
        let mut all_ys: Vec<f32> = Vec::new();
        for col_spans in cell_indices {
            for &idx in col_spans {
                if let Some(s) = spans.get(idx) {
                    all_ys.push(s.bbox.center().y);
                }
            }
        }

        if all_ys.len() <= 1 {
            // 0 or 1 span total -- nothing to split.
            result.push(row);
            continue;
        }

        // Cluster Y positions using the configured row_tolerance.
        all_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        let mut y_clusters: Vec<f32> = Vec::new();
        for &y in &all_ys {
            let merged = y_clusters
                .last()
                .is_some_and(|&last| (y - last).abs() < config.row_tolerance);
            if merged {
                // Update cluster center as running average.
                let last = y_clusters.last_mut().unwrap();
                *last = (*last + y) / 2.0;
            } else {
                y_clusters.push(y);
            }
        }

        if y_clusters.len() <= 1 {
            // All spans are on the same Y line -- no split needed.
            result.push(row);
            continue;
        }

        // Sort clusters descending (higher y = top of page in PDF coords, displayed first).
        y_clusters.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a));

        let num_cols = row.cells.len();

        // Build one new row per Y-cluster.
        for &cluster_y in &y_clusters {
            let mut new_row = TableRow::new(row.is_header);
            for ci in 0..num_cols {
                // Collect spans in this cell that belong to this Y-cluster.
                let matching_indices: Vec<usize> = cell_indices[ci]
                    .iter()
                    .copied()
                    .filter(|&idx| {
                        spans
                            .get(idx)
                            .map(|s| {
                                let sy = s.bbox.center().y;
                                // Assign span to nearest cluster.
                                y_clusters
                                    .iter()
                                    .min_by_key(|&&cy| ((sy - cy).abs() * 1000.0) as i32)
                                    .is_some_and(|&nearest| (nearest - cluster_y).abs() < 0.01)
                            })
                            .unwrap_or(false)
                    })
                    .collect();

                let cell_text = extract_cell_text(&matching_indices, spans);
                let mcids: Vec<u32> = matching_indices
                    .iter()
                    .filter_map(|&idx| spans.get(idx).and_then(|s| s.mcid))
                    .collect();

                // Compute bbox from matching spans, fall back to original cell bbox.
                let cell_bbox = if matching_indices.is_empty() {
                    row.cells[ci].bbox
                } else {
                    let mut b = spans[matching_indices[0]].bbox;
                    for &idx in &matching_indices[1..] {
                        b = b.union(&spans[idx].bbox);
                    }
                    Some(b)
                };

                let cell_spans = matching_indices
                    .iter()
                    .filter_map(|&idx| spans.get(idx).cloned())
                    .collect::<Vec<_>>();

                new_row.cells.push(TableCell {
                    text: cell_text,
                    spans: cell_spans,
                    colspan: 1,
                    rowspan: 1,
                    mcids,
                    bbox: cell_bbox,
                    is_header: row.is_header,
                });
            }
            result.push(new_row);
        }
    }

    result
}

/// Strip form-template numbering artifacts and decorative separators from table rows.
///
/// PDF form templates sometimes embed single-digit numbering (e.g. "1", "5") as
/// separate text spans that get concatenated into cell text as a prefix. They also
/// use rows or cells filled with dashes/underscores as decorative separators.
/// This function:
/// 1. Removes entire rows where every cell is either empty or a lone single digit.
/// 2. Strips a leading single-digit prefix from cell text when the remainder looks
///    like real content (starts with a letter, `$`, or contains `/` or `-`).
/// 3. Clears cells that contain only dashes/underscores (decorative separators).
/// 4. Removes rows where all cells are empty after separator stripping.
fn strip_form_numbering_artifacts(table_rows: &mut Vec<TableRow>) {
    // Phase 1: Remove rows where ALL cells are either empty or a lone single
    // digit (1-9), AND at least one cell actually contains a digit.  Rows that
    // are completely empty are left intact so the downstream empty-row splitting
    // logic can use them as table separators.
    table_rows.retain(|row| {
        let all_empty_or_digit = row.cells.iter().all(|c| {
            let t = c.text.trim();
            t.is_empty()
                || (t.len() == 1
                    && t.as_bytes()
                        .first()
                        .is_some_and(|b| b.is_ascii_digit() && *b != b'0'))
        });
        let has_digit = row.cells.iter().any(|c| {
            let t = c.text.trim();
            t.len() == 1
                && t.as_bytes()
                    .first()
                    .is_some_and(|b| b.is_ascii_digit() && *b != b'0')
        });
        !(all_empty_or_digit && has_digit)
    });

    // Phase 2: Strip leading single-digit prefix from individual cells.
    // Track whether any stripping occurred for Phase 3.
    // Only strip when the remainder clearly looks like form data (currency, dates,
    // codes with dashes/slashes), NOT when it could be a natural phrase like
    // "3 items".
    for row in table_rows.iter_mut() {
        let mut stripped_any = false;
        for cell in &mut row.cells {
            let text = cell.text.trim();
            if text.len() < 3 {
                continue; // Need at least digit + space + char
            }
            let bytes = text.as_bytes();
            if bytes[0].is_ascii_digit() && bytes[0] != b'0' && bytes[1] == b' ' {
                let rest = text[2..].trim_start();
                if !rest.is_empty() {
                    let first = rest.as_bytes()[0];
                    // Strip when remainder starts with '$' (currency) or starts
                    // with a digit (date like "Apr 11" won't, but codes like
                    // "12111 - ..." will), or contains '-' or '/' (dates, codes).
                    let looks_like_data = first == b'$'
                        || first.is_ascii_digit()
                        || (first.is_ascii_alphabetic()
                            && (rest.contains('-') || rest.contains('/') || rest.contains(',')));
                    if looks_like_data {
                        cell.text = rest.to_string();
                        stripped_any = true;
                    }
                }
            }
        }

        // Phase 3: In rows where prefixes were stripped, clear remaining
        // lone single-digit cells (they're the same numbering artifact
        // but had no content after the digit).
        if stripped_any {
            for cell in &mut row.cells {
                let t = cell.text.trim();
                if t.len() == 1 && t.as_bytes()[0].is_ascii_digit() {
                    cell.text.clear();
                }
            }
        }
    }

    // Phase 4: Clear cells that contain only dashes and/or underscores
    // (decorative line separators in form templates, e.g. "------", "____").
    for row in table_rows.iter_mut() {
        for cell in &mut row.cells {
            let t = cell.text.trim();
            if !t.is_empty() && t.chars().all(|c| c == '-' || c == '_') {
                cell.text.clear();
            }
        }
    }

    // Note: rows that become fully empty after Phase 4 (e.g. all-dash rows)
    // are intentionally left in place.  The downstream empty-row splitting
    // logic in detect_tables_from_intersections uses them as table separators.
}

/// Detect tables from intersections of horizontal and vertical edges, then assign text.
///
/// This implements the universal pipeline used by Tabula, pdfplumber, and PyMuPDF:
/// `Edges -> Snap/Merge -> Intersections -> Cells -> Table Groups`
fn detect_tables_from_intersections(
    spans: &[TextSpan],
    lines: &[crate::elements::PathContent],
    config: &TableDetectionConfig,
) -> Vec<Table> {
    let groups = build_grid_from_lines(lines, config);

    let mut tables = Vec::new();
    for (group_cells, xs, ys, num_cols) in &groups {
        let Some((table_rows, row_cell_span_indices)) =
            assign_spans_to_intersection_grid(group_cells, xs, ys, *num_cols, spans)
        else {
            continue;
        };
        let sub_tables = finalize_intersection_tables(
            table_rows,
            &row_cell_span_indices,
            spans,
            config,
            *num_cols,
        );
        tables.extend(sub_tables);
    }

    merge_vertically_adjacent_tables(&mut tables);

    // Post-merge: split tables at section dividers — full-width horizontal
    // lines that indicate separate form sections within a single grid.
    // Use merged H-edges (to detect full-width lines) but only snap (do NOT
    // join) V-edges — joining would merge separate per-section V-segments
    // into a single long edge, hiding the section boundary discontinuity.
    let (mut h_edges, mut v_edges) = extract_edges(lines);
    snap_and_merge(&mut h_edges);
    snap_edges(&mut v_edges); // snap only, don't join
    tables = split_tables_at_section_dividers(tables, &h_edges, &v_edges, config);

    tables
}

/// Steps 1-4: extract edges, find intersections, build cells, and group them
/// into per-table cell groups with their grid boundaries.
///
/// Returns one `(group_cells, xs, ys, num_cols)` tuple per table group.
fn build_grid_from_lines(
    lines: &[crate::elements::PathContent],
    config: &TableDetectionConfig,
) -> Vec<(Vec<IntersectionCell>, Vec<f32>, Vec<f32>, usize)> {
    // Step 1: Extract and preprocess edges.
    let (mut h_edges, mut v_edges) = extract_edges(lines);
    snap_and_merge(&mut h_edges);
    snap_and_merge(&mut v_edges);

    if h_edges.len() < 2 || v_edges.len() < 2 {
        return Vec::new();
    }

    // Step 2: Find intersections.
    let intersections = find_intersections(&h_edges, &v_edges);

    // Step 2b: When intersections are sparse (< 4), filter out orphan edges
    // that have no plausible counterpart before building the extended grid.
    // This prevents unrelated edges (e.g., decorative lines far from the table)
    // from polluting the grid.
    if intersections.len() < 4 {
        filter_edges_by_coverage(&mut h_edges, &mut v_edges);
        if h_edges.len() < 2 || v_edges.len() < 2 {
            return Vec::new();
        }
    }

    // Step 3: Build cells.
    let cells = if intersections.len() >= 4 {
        let c = build_cells_from_intersections(&intersections);
        if c.is_empty() {
            // Lines exist but don't form real intersection cells — try extended grid.
            build_extended_grid_cells(&h_edges, &v_edges)
        } else {
            c
        }
    } else {
        // H and V lines don't physically cross (e.g. Census table: H-lines in
        // header area, V tick marks in data area). Build a virtual grid by
        // projecting all V-line X positions across all H-line Y positions.
        build_extended_grid_cells(&h_edges, &v_edges)
    };
    if cells.is_empty() {
        return Vec::new();
    }

    // Step 4: Group cells into tables and compute grid boundaries per group.
    let table_groups = group_cells_into_tables(&cells);
    let mut result = Vec::new();
    for group in &table_groups {
        let group_cells: Vec<IntersectionCell> = group.iter().map(|&i| cells[i]).collect();

        // Determine unique sorted X and Y boundaries for this table.
        let mut xs: Vec<f32> = Vec::new();
        let mut ys: Vec<f32> = Vec::new();
        for c in &group_cells {
            xs.push(c.x1);
            xs.push(c.x2);
            ys.push(c.y1);
            ys.push(c.y2);
        }
        xs.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        xs.dedup_by(|a, b| (*a - *b).abs() <= SNAP_TOL);
        ys.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        ys.dedup_by(|a, b| (*a - *b).abs() <= SNAP_TOL);

        let num_cols = if xs.len() >= 2 {
            xs.len() - 1
        } else {
            continue;
        };
        if ys.len() < 2 {
            continue;
        }

        if num_cols < config.min_table_columns || num_cols > config.max_table_columns {
            continue;
        }

        result.push((group_cells, xs, ys, num_cols));
    }
    result
}

/// Assign text spans to grid cells and build table rows with per-cell span
/// indices. Returns `None` when the grid is degenerate.
fn assign_spans_to_intersection_grid(
    group_cells: &[IntersectionCell],
    xs: &[f32],
    ys: &[f32],
    num_cols: usize,
    spans: &[TextSpan],
) -> Option<(Vec<TableRow>, Vec<Vec<Vec<usize>>>)> {
    let num_rows = if ys.len() >= 2 {
        ys.len() - 1
    } else {
        return None;
    };

    // Map each cell to a (row, col) position.
    let col_of =
        |x: f32| -> Option<usize> { (0..num_cols).find(|&c| (xs[c] - x).abs() <= SNAP_TOL) };
    let row_of =
        |y: f32| -> Option<usize> { (0..num_rows).find(|&r| (ys[r] - y).abs() <= SNAP_TOL) };

    // Track which grid positions have cells.
    let mut grid_has_cell = vec![vec![false; num_cols]; num_rows];
    for c in group_cells {
        if let (Some(ci), Some(ri)) = (col_of(c.x1), row_of(c.y1)) {
            grid_has_cell[ri][ci] = true;
        }
    }

    // Assign text spans to grid cells based on center point. Prefer the exact
    // interval before applying snap tolerance: expanding every interval makes
    // points near an internal boundary match both neighbours, and `find()`
    // would always bias them into the earlier row or column.
    let mut grid_spans: Vec<Vec<Vec<usize>>> = vec![vec![Vec::new(); num_cols]; num_rows];
    for (idx, span) in spans.iter().enumerate() {
        let cx = span.bbox.center().x;
        let cy = span.bbox.center().y;
        let col_idx = grid_interval_for_point(cx, xs);
        let row_idx = grid_interval_for_point(cy, ys);
        if let (Some(ci), Some(ri)) = (col_idx, row_idx) {
            if grid_has_cell[ri][ci] {
                grid_spans[ri][ci].push(idx);
            }
        }
    }

    // Build rows sorted top-to-bottom for the table.
    // In PDF coordinates, higher y = higher on page, so sort rows descending by y.
    let mut row_order: Vec<usize> = (0..num_rows).collect();
    row_order.sort_by(|&a, &b| crate::utils::safe_float_cmp(ys[b], ys[a]));

    let mut table_rows = Vec::new();
    // Track span indices per cell alongside table_rows for text-based row splitting.
    let mut row_cell_span_indices: Vec<Vec<Vec<usize>>> = Vec::new();
    for &ri in &row_order {
        let mut row = TableRow::new(false);
        let mut cell_indices_for_row: Vec<Vec<usize>> = Vec::new();
        for ci in 0..num_cols {
            if !grid_has_cell[ri][ci] {
                // Still emit empty cell so column count stays consistent.
                row.cells.push(TableCell {
                    text: String::new(),
                    spans: Vec::new(),
                    colspan: 1,
                    rowspan: 1,
                    mcids: Vec::new(),
                    bbox: Some(crate::geometry::Rect::new(
                        xs[ci],
                        ys[ri],
                        xs[ci + 1] - xs[ci],
                        ys[ri + 1] - ys[ri],
                    )),
                    is_header: false,
                });
                cell_indices_for_row.push(Vec::new());
                continue;
            }
            let cell_text = extract_cell_text(&grid_spans[ri][ci], spans);
            let mcids: Vec<u32> = grid_spans[ri][ci]
                .iter()
                .filter_map(|&idx| spans.get(idx).and_then(|s| s.mcid))
                .collect();
            let cell_bbox = crate::geometry::Rect::new(
                xs[ci],
                ys[ri],
                xs[ci + 1] - xs[ci],
                ys[ri + 1] - ys[ri],
            );
            let cell_spans = grid_spans[ri][ci]
                .iter()
                .filter_map(|&idx| spans.get(idx).cloned())
                .collect::<Vec<_>>();

            row.cells.push(TableCell {
                text: cell_text,
                spans: cell_spans,
                colspan: 1,
                rowspan: 1,
                mcids,
                bbox: Some(cell_bbox),
                is_header: false,
            });
            cell_indices_for_row.push(grid_spans[ri][ci].clone());
        }
        table_rows.push(row);
        row_cell_span_indices.push(cell_indices_for_row);
    }

    Some((table_rows, row_cell_span_indices))
}

/// Return the grid interval containing `point`.
///
/// Internal boundaries are half-open and belong to the interval on their
/// right. Snap tolerance is reserved for points just outside the grid, where
/// there is no neighbouring interval to compete for ownership.
fn grid_interval_for_point(point: f32, boundaries: &[f32]) -> Option<usize> {
    let interval_count = boundaries.len().checked_sub(1)?;
    if interval_count == 0 || !point.is_finite() {
        return None;
    }

    if point < boundaries[0] {
        return (boundaries[0] - point <= SNAP_TOL).then_some(0);
    }
    if point > boundaries[interval_count] {
        return (point - boundaries[interval_count] <= SNAP_TOL).then_some(interval_count - 1);
    }

    (0..interval_count).find(|&index| {
        point >= boundaries[index]
            && (point < boundaries[index + 1]
                || (index + 1 == interval_count && point <= boundaries[index + 1]))
    })
}

/// Row splitting, form-artifact stripping, empty-row splitting, and bbox
/// computation. Produces the final `Table` entries for one table group.
fn finalize_intersection_tables(
    table_rows: Vec<TableRow>,
    row_cell_span_indices: &[Vec<Vec<usize>>],
    spans: &[TextSpan],
    config: &TableDetectionConfig,
    num_cols: usize,
) -> Vec<Table> {
    // Hybrid row splitting: if a row contains text spans at multiple distinct
    // Y positions (no horizontal lines between them), split into sub-rows
    // based on text Y-clustering.
    let mut table_rows =
        split_rows_by_text_positions(table_rows, row_cell_span_indices, spans, config);

    // Post-process: strip form template numbering artifacts.
    // Form templates sometimes embed single-digit numbering (e.g. "1", "5") as
    // separate text spans that get concatenated to cell text as a prefix.
    strip_form_numbering_artifacts(&mut table_rows);

    // Split on completely empty rows (same strategy as cluster-based approach).
    let mut tables = Vec::new();
    let mut sub_start = 0;
    while sub_start < table_rows.len() {
        // Skip leading empty rows.
        let row_is_empty = |r: &TableRow| r.cells.iter().all(|c| c.text.is_empty());
        if row_is_empty(&table_rows[sub_start]) {
            sub_start += 1;
            continue;
        }
        let mut sub_end = sub_start + 1;
        while sub_end < table_rows.len() && !row_is_empty(&table_rows[sub_end]) {
            sub_end += 1;
        }
        let sub_rows: Vec<TableRow> = table_rows[sub_start..sub_end].to_vec();
        let filled: usize = sub_rows
            .iter()
            .flat_map(|r| r.cells.iter())
            .filter(|c| !c.text.is_empty())
            .count();
        if filled >= config.min_table_cells {
            // Compute bbox from the cells in this sub-table.
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            for r in &sub_rows {
                for c in &r.cells {
                    if let Some(b) = c.bbox {
                        min_x = min_x.min(b.left());
                        min_y = min_y.min(b.top());
                        max_x = max_x.max(b.right());
                        max_y = max_y.max(b.bottom());
                    }
                }
            }
            let sub_bbox = if min_x.is_finite() {
                Some(crate::geometry::Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
            } else {
                None
            };
            tables.push(Table {
                rows: sub_rows,
                has_header: false,
                col_count: num_cols,
                bbox: sub_bbox,
            });
        }
        sub_start = sub_end;
    }
    tables
}

/// Minimum fraction of the table width that an H-edge must span to qualify
/// as a section divider.
const SECTION_DIVIDER_WIDTH_RATIO: f32 = 0.80;

/// Split each table at interior horizontal edges that span nearly the full
/// table width ("section dividers").  Returns a new list of tables where each
/// original table may have been broken into multiple smaller ones.
fn split_tables_at_section_dividers(
    tables: Vec<Table>,
    h_edges: &[Edge],
    v_edges: &[Edge],
    config: &TableDetectionConfig,
) -> Vec<Table> {
    let mut result = Vec::new();
    for table in tables {
        let parts = split_table_at_section_dividers(table, h_edges, v_edges, config);
        result.extend(parts);
    }
    result
}

/// Split a single table at section divider lines.
///
/// A section divider is a full-width H-edge at a Y position where few or no
/// V-edges cross through — indicating that the vertical lines stop at that
/// boundary (separate bordered sections stacked vertically).
fn split_table_at_section_dividers(
    table: Table,
    h_edges: &[Edge],
    v_edges: &[Edge],
    config: &TableDetectionConfig,
) -> Vec<Table> {
    let Some(bbox) = table.bbox else {
        return vec![table];
    };
    if table.rows.len() < 2 {
        return vec![table];
    }

    let table_width = bbox.right() - bbox.left();
    if table_width <= 0.0 {
        return vec![table];
    }

    // Collect Y-coordinates of H-edges that qualify as section dividers:
    // - span >= SECTION_DIVIDER_WIDTH_RATIO of the table width
    // - fall within the table's vertical range (not at the very top or bottom)
    // - few V-edges cross through that Y (sections have separate vertical grids)
    let top = bbox.top();
    let bottom = bbox.bottom();
    let margin = 2.0; // pts – ignore edges at the very top/bottom boundary

    // Count how many V-edges (within the table's X-range) cross each candidate Y.
    // V-edges have coord=X, start=minY, end=maxY.
    let table_left = bbox.left();
    let table_right = bbox.right();
    let relevant_v_edges: Vec<&Edge> = v_edges
        .iter()
        .filter(|e| e.coord >= table_left - SNAP_TOL && e.coord <= table_right + SNAP_TOL)
        .collect();

    let mut divider_ys: Vec<f32> = Vec::new();
    for edge in h_edges {
        // Edge must overlap the table's horizontal extent significantly.
        let overlap_start = edge.start.max(table_left);
        let overlap_end = edge.end.min(table_right);
        let overlap = overlap_end - overlap_start;
        if overlap < table_width * SECTION_DIVIDER_WIDTH_RATIO {
            continue;
        }
        // Edge must be interior (not the top or bottom border of the table).
        let y = edge.coord;
        if y <= top + margin || y >= bottom - margin {
            continue;
        }
        // Count V-edges that cross through this Y-coordinate (i.e., their
        // vertical span straddles it with clearance on both sides).
        let cross_margin = SNAP_TOL + 1.0;
        let crossings = relevant_v_edges
            .iter()
            .filter(|v| v.start < y - cross_margin && v.end > y + cross_margin)
            .count();
        // A true section divider has no (or very few) V-edges crossing through.
        // Regular grid row boundaries have many V-edges crossing.
        if crossings <= 1 {
            divider_ys.push(y);
        }
    }

    if divider_ys.is_empty() {
        return vec![table];
    }

    divider_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    divider_ys.dedup_by(|a, b| (*a - *b).abs() <= SNAP_TOL);

    // Find which row indices the dividers fall between.
    // A divider Y falls "between row i and row i+1" if it sits between the
    // bottom of row i and the top of row i+1 (with tolerance).
    //
    // Build a list of row bboxes (top, bottom) for matching.
    let row_bounds: Vec<Option<(f32, f32)>> = table
        .rows
        .iter()
        .map(|row| {
            let mut rmin = f32::INFINITY;
            let mut rmax = f32::NEG_INFINITY;
            for c in &row.cells {
                if let Some(b) = c.bbox {
                    rmin = rmin.min(b.top());
                    rmax = rmax.max(b.bottom());
                }
            }
            if rmin.is_finite() {
                Some((rmin, rmax))
            } else {
                None
            }
        })
        .collect();

    // Determine split-after indices: row indices after which to split.
    // A divider at Y should split after the row whose bottom (max Y) is at
    // or near that Y, OR before the row whose top (min Y) is at or near Y.
    let mut split_after: Vec<usize> = Vec::new();
    let tol = SNAP_TOL + 2.0; // generous tolerance for matching divider to row boundary
    for &dy in &divider_ys {
        // Find the row whose bottom edge is closest to dy (from above or at dy).
        let mut best_idx: Option<usize> = None;
        let mut best_dist = f32::INFINITY;
        for (i, bounds) in row_bounds.iter().enumerate() {
            if i >= table.rows.len().saturating_sub(1) {
                continue; // don't split after the last row
            }
            let Some((row_top, row_bot)) = bounds else {
                continue;
            };
            // Check if divider is near this row's bottom or near the next
            // row's top.
            let dist_to_bot = (dy - row_bot).abs();
            let dist_to_top = (dy - row_top).abs();
            let min_dist = dist_to_bot.min(dist_to_top);
            if min_dist <= tol && min_dist < best_dist {
                // Split after this row if divider is at its bottom,
                // or split after (i-1) if divider is at its top.
                if dist_to_bot <= dist_to_top {
                    best_idx = Some(i);
                } else if i > 0 {
                    best_idx = Some(i - 1);
                }
                best_dist = min_dist;
            }
        }
        if let Some(idx) = best_idx {
            split_after.push(idx);
        }
    }
    split_after.sort_unstable();
    split_after.dedup();

    if split_after.is_empty() {
        return vec![table];
    }

    // Perform the splits.
    let num_cols = table.col_count;
    let all_rows = table.rows;
    let mut sub_tables = Vec::new();
    let mut start = 0;
    for &split_idx in &split_after {
        let end = split_idx + 1;
        if end > start {
            sub_tables.push(&all_rows[start..end]);
        }
        start = end;
    }
    if start < all_rows.len() {
        sub_tables.push(&all_rows[start..]);
    }

    let mut result = Vec::new();
    for sub_rows_slice in sub_tables {
        let sub_rows: Vec<TableRow> = sub_rows_slice.to_vec();
        let filled: usize = sub_rows
            .iter()
            .flat_map(|r| r.cells.iter())
            .filter(|c| !c.text.is_empty())
            .count();
        if filled < config.min_table_cells {
            continue;
        }
        // Compute bbox for sub-table.
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for r in &sub_rows {
            for c in &r.cells {
                if let Some(b) = c.bbox {
                    min_x = min_x.min(b.left());
                    min_y = min_y.min(b.top());
                    max_x = max_x.max(b.right());
                    max_y = max_y.max(b.bottom());
                }
            }
        }
        let sub_bbox = if min_x.is_finite() {
            Some(crate::geometry::Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        };
        result.push(Table {
            rows: sub_rows,
            has_header: false,
            col_count: num_cols,
            bbox: sub_bbox,
        });
    }

    if result.is_empty() {
        // Don't lose data; return original if all sub-tables were too small.
        return vec![Table {
            rows: all_rows,
            has_header: false,
            col_count: num_cols,
            bbox: Some(bbox),
        }];
    }

    result
}

/// Maximum vertical gap (in points) between two table bboxes to consider them
/// adjacent and merge them into a single table.
const ADJACENT_TABLE_MERGE_GAP: f32 = 20.0;

/// Maximum allowed column count difference for merging vertically adjacent tables.
/// Tables whose column counts differ by more than this are not merged.
const MERGE_COL_DIFF_TOLERANCE: usize = 2;

/// Merge tables that are vertically adjacent (small gap between bottom of one
/// and top of another) and have similar column counts (difference <= `MERGE_COL_DIFF_TOLERANCE`).
/// When column counts differ, the narrower table's rows are padded with empty cells.
fn merge_vertically_adjacent_tables(tables: &mut Vec<Table>) {
    if tables.len() < 2 {
        return;
    }

    // Sort tables by the top-Y of their bbox (highest Y first in PDF coords).
    tables.sort_by(|a, b| {
        let ay = a.bbox.map_or(f32::NEG_INFINITY, |bb| bb.top());
        let by = b.bbox.map_or(f32::NEG_INFINITY, |bb| bb.top());
        crate::utils::safe_float_cmp(ay, by)
    });

    let mut merged: Vec<Table> = Vec::new();
    for table in tables.drain(..) {
        let should_merge = merged.last().is_some_and(|prev: &Table| {
            let col_diff = (prev.col_count as isize - table.col_count as isize).unsigned_abs();
            if col_diff > MERGE_COL_DIFF_TOLERANCE {
                return false;
            }
            match (prev.bbox, table.bbox) {
                (Some(pb), Some(tb)) => {
                    // Vertical gap: distance between bottom of prev and top of current.
                    let gap = (tb.top() - pb.bottom())
                        .abs()
                        .min((pb.top() - tb.bottom()).abs());
                    gap <= ADJACENT_TABLE_MERGE_GAP
                },
                _ => false,
            }
        });

        if should_merge {
            let prev = merged.last_mut().unwrap();
            let target_cols = prev.col_count.max(table.col_count);

            // Pad existing rows in prev if the new table has more columns.
            if prev.col_count < target_cols {
                let pad = target_cols - prev.col_count;
                for row in &mut prev.rows {
                    for _ in 0..pad {
                        row.cells.push(TableCell {
                            text: String::new(),
                            spans: Vec::new(),
                            colspan: 1,
                            rowspan: 1,
                            mcids: Vec::new(),
                            bbox: None,
                            is_header: row.is_header,
                        });
                    }
                }
            }

            // Pad incoming rows if they have fewer columns.
            let mut incoming_rows = table.rows;
            if table.col_count < target_cols {
                let pad = target_cols - table.col_count;
                for row in &mut incoming_rows {
                    for _ in 0..pad {
                        row.cells.push(TableCell {
                            text: String::new(),
                            spans: Vec::new(),
                            colspan: 1,
                            rowspan: 1,
                            mcids: Vec::new(),
                            bbox: None,
                            is_header: row.is_header,
                        });
                    }
                }
            }

            prev.rows.extend(incoming_rows);
            prev.col_count = target_cols;
            // Update bbox to encompass both.
            if let (Some(pb), Some(tb)) = (prev.bbox, table.bbox) {
                let min_x = pb.left().min(tb.left());
                let min_y = pb.top().min(tb.top());
                let max_x = pb.right().max(tb.right());
                let max_y = pb.bottom().max(tb.bottom());
                prev.bbox =
                    Some(crate::geometry::Rect::new(min_x, min_y, max_x - min_x, max_y - min_y));
            }
            prev.has_header = prev.has_header || table.has_header;
        } else {
            merged.push(table);
        }
    }

    *tables = merged;
}

/// True when the page carries vertical-ruling evidence that should route
/// table detection through the grid pipelines instead of the
/// horizontal-rule-bounded fallback.
///
/// A vertical line counts as RULING evidence only when its drawn bar
/// crosses at least TWO of the horizontal rules: a ruling vertical bounds
/// cells BETWEEN row rules, so it spans from one rule to another (a
/// stroke-width-encoded column bar crosses every row rule of its table).
/// Anything that crosses fewer says nothing about how the page's tables
/// are ruled: an isolated heavy-stroked speck (tick mark, list dash)
/// crosses nothing, and the short dash segments of a decorative dashed
/// BOX border cross at most the one rule their box happens to overlap.
/// Both were disabling the horizontal-rule fallback page-wide, which is
/// precisely what scattered booktabs tables on pages carrying a
/// dash-bordered affiliation box. Rectangles (decomposed into edges) keep
/// their pre-existing veto.
fn has_vertical_ruling_evidence(lines: &[crate::elements::PathContent], h_edges: &[Edge]) -> bool {
    const LINE_AXIS_TOL: f32 = 2.0;
    lines.iter().any(|path| {
        if path.is_horizontal_line(LINE_AXIS_TOL) {
            return false;
        }
        if path.is_vertical_line(LINE_AXIS_TOL) {
            let r = path.rendered_bbox();
            // Count DISTINCT rule levels crossed, not raw edges: one dashed
            // rule is several collinear edges at the same y, and a border
            // "joint" speck sitting on it would otherwise read as crossing
            // two rules while touching only one.
            let mut crossed_ys: Vec<f32> = h_edges
                .iter()
                .filter(|h| {
                    r.y <= h.coord
                        && (r.y + r.height) >= h.coord
                        && (r.x + r.width) >= h.start
                        && r.x <= h.end
                })
                .map(|h| h.coord)
                .collect();
            crossed_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
            crossed_ys.dedup_by(|a, b| (*a - *b).abs() <= LINE_AXIS_TOL);
            return crossed_ys.len() >= 2;
        }
        path.is_rectangle()
    })
}

/// Group wide H-edges into x-range-coherent FAMILIES, in emission order,
/// so only rules that could bound the same table ever pair up. This fixes
/// two failure shapes in one move: a page of scattered fraction bars never
/// forms a family (no fake table from unrelated equation rules), and a
/// decorative dashed border whose y-position interleaves a real table's
/// rules lands in its own family instead of splitting the table's rules
/// apart in the adjacent-pair walk (which silently dropped the table on
/// real dash-boxed pages).
fn x_coherent_rule_families<'a>(wide: &[&'a Edge]) -> Vec<Vec<&'a Edge>> {
    // A table's boundary rules line up: booktabs top/sub-header/bottom
    // rules share their x-range to within a point (overlap/union ≈ 1.0),
    // while unrelated wide strokes — displayed-equation fraction bars,
    // decorative borders — share at most a common left margin
    // (overlap/union ≲ 0.8 even when x-starts coincide, since widths
    // differ). 0.85 splits the two populations with margin on both sides.
    const X_COHERENCE: f32 = 0.85;

    let mut uf = UnionFind::new(wide.len());
    for i in 0..wide.len() {
        for j in (i + 1)..wide.len() {
            let (a, b) = (wide[i], wide[j]);
            let overlap = a.end.min(b.end) - a.start.max(b.start);
            let union = a.end.max(b.end) - a.start.min(b.start);
            if union > 0.0 && overlap / union >= X_COHERENCE {
                uf.union(i, j);
            }
        }
    }
    // Keyed by union-find root in a BTreeMap, not a HashMap: HashMap iteration
    // order is randomized per process, and the sort below cannot recover from
    // that on its own. Families are grouped by X-RANGE COHERENCE, so `coord`
    // (the Y of a horizontal rule) is not unique across families — two rule
    // families side by side, or a decorative border sharing a table's top-rule
    // Y, tie on `a[0].coord`. `sort_by` is stable, so a tie leaves the input
    // order intact, and with a HashMap that input order is per-process random:
    // the two families then emit their tables in a different order each run,
    // which reorders the rendered table blocks in the assembled page text.
    // The BTreeMap makes the collected order deterministic; the root tiebreak
    // makes the emitted order a total order over the families, so it depends
    // only on page geometry and edge input order. Each family's `Vec` is pushed
    // in ascending `wide` index order, so `a[0]` is still the family's first
    // rule and non-tied pages sort exactly as before (byte-identical).
    let mut families: std::collections::BTreeMap<usize, Vec<&'a Edge>> =
        std::collections::BTreeMap::new();
    for (i, e) in wide.iter().enumerate() {
        families.entry(uf.find(i)).or_default().push(e);
    }
    let mut families: Vec<(usize, Vec<&'a Edge>)> = families.into_iter().collect();
    families.sort_by(|(root_a, a), (root_b, b)| {
        crate::utils::safe_float_cmp(a[0].coord, b[0].coord).then_with(|| root_a.cmp(root_b))
    });
    families.into_iter().map(|(_root, f)| f).collect()
}

/// Detect tables in regions bounded by horizontal rules (H-lines) when no vertical
/// lines are present.  Groups H-edges by Y-position to find horizontal table
/// boundaries, then runs text-edge detection on the spans within each bounded
/// region.  This is the "H-lines define regions, text defines columns" hybrid.
fn detect_tables_from_horizontal_rules(
    spans: &[TextSpan],
    h_edges: &[Edge],
    config: &TableDetectionConfig,
) -> Vec<Table> {
    const MIN_RULE_WIDTH: f32 = 100.0;
    const Y_SNAP: f32 = 4.0;

    // Keep only wide H-edges.
    let wide: Vec<&Edge> = h_edges
        .iter()
        .filter(|e| (e.end - e.start) >= MIN_RULE_WIDTH)
        .collect();
    if wide.len() < 2 {
        return Vec::new();
    }

    let families = x_coherent_rule_families(&wide);

    let mut tables = Vec::new();

    for family in &families {
        // Cluster this family's edges by Y-coordinate (snap within Y_SNAP).
        let mut y_coords: Vec<f32> = Vec::new();
        for e in family {
            let merged = y_coords
                .iter_mut()
                .find(|y| (e.coord - **y).abs() <= Y_SNAP);
            if merged.is_none() {
                y_coords.push(e.coord);
            }
        }
        y_coords.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a)); // descending (top first in PDF coords)

        if y_coords.len() < 2 {
            continue;
        }

        // For each cluster, compute the X-range (union of the FAMILY's
        // edges in that cluster).
        let x_range_for_y = |target_y: f32| -> (f32, f32) {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            for e in family {
                if (e.coord - target_y).abs() <= Y_SNAP {
                    if e.start < min_x {
                        min_x = e.start;
                    }
                    if e.end > max_x {
                        max_x = e.end;
                    }
                }
            }
            (min_x, max_x)
        };

        // Consider adjacent Y-pairs within the family as potential table regions.
        for pair in y_coords.windows(2) {
            let y_top = pair[0];
            let y_bot = pair[1];
            // Both H-lines must span significant width and overlap in X.
            let (x1_start, x1_end) = x_range_for_y(y_top);
            let (x2_start, x2_end) = x_range_for_y(y_bot);
            let x_overlap_start = x1_start.max(x2_start);
            let x_overlap_end = x1_end.min(x2_end);
            if x_overlap_end - x_overlap_start < MIN_RULE_WIDTH {
                continue;
            }

            // Collect spans within this Y-range and X-range (with small padding).
            let pad = 2.0;
            let mut region_spans: Vec<TextSpan> = Vec::new();
            let mut outside_width = 0.0f32;
            let mut inside_width = 0.0f32;
            for s in spans {
                let cy = s.bbox.center().y;
                if cy > y_top + pad || cy < y_bot - pad {
                    continue;
                }
                let cx = s.bbox.center().x;
                if cx >= x_overlap_start - pad && cx <= x_overlap_end + pad {
                    inside_width += s.bbox.width.max(0.0);
                    region_spans.push(s.clone());
                } else {
                    outside_width += s.bbox.width.max(0.0);
                }
            }

            if region_spans.is_empty() {
                continue;
            }

            // A pair of rules bounds a table only if the band's text is
            // horizontally CONTAINED by the rules: a table's boundary
            // rules span the rows they rule, while a fraction bar floats
            // inside surrounding math that continues to its left and
            // right (relation symbols, equation numbers). X-range-coherent
            // vinculums from an aligned multi-step derivation pass the
            // family check above, but the text spilling past the bars
            // gives them away — when a third of the band's text mass lies
            // outside the rules, they don't bound anything. (Division-free
            // so a band of zero-width spans compares 0 > 0 instead of
            // taking a NaN branch.)
            if outside_width > (outside_width + inside_width) * 0.3 {
                continue;
            }

            // Letter-spaced monospace guard: framed code and console
            // listings (zines, technical reports) draw each glyph on a
            // terminal-font grid, so the band's "words" are mostly single
            // characters whose aligned x positions look exactly like
            // column boundaries — identifiers shatter into single letters
            // (`s e g f a u l t`), addresses into single digits
            // (`0 0 : 1 4`). A real table's cells are words and numbers:
            // one-third single LETTERS or one-half single characters of
            // any kind is spread-out text, not a grid. (The digit
            // threshold is the looser of the two so genuine single-digit
            // table columns, which sit among multi-char label cells, stay
            // under it.)
            let word_count = region_spans.len();
            let mut single_any = 0usize;
            let mut single_alpha = 0usize;
            for rs in &region_spans {
                let mut chars = rs.text.trim().chars();
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    single_any += 1;
                    if c.is_alphabetic() {
                        single_alpha += 1;
                    }
                }
            }
            if word_count > 0 && (single_alpha * 3 >= word_count || single_any * 2 >= word_count) {
                continue;
            }

            let mut detected = detect_tables_from_spans(&region_spans, config);
            tables.append(&mut detected);
        }
    }

    // Two families can bracket the same text — a dash-bordered decorative
    // box drawn around (or through) a ruled table gives both the box's
    // border family and the table's rule family a region over the same
    // rows, and each detects its own copy. Keep the TIGHTER detection when
    // two overlap: the looser region also swallows neighbouring lines
    // (footnotes, captions) as junk rows.
    let mut keep: Vec<bool> = vec![true; tables.len()];
    for i in 0..tables.len() {
        for j in (i + 1)..tables.len() {
            if !keep[i] || !keep[j] {
                continue;
            }
            let (Some(a), Some(b)) = (tables[i].bbox, tables[j].bbox) else {
                continue;
            };
            let ov_w = (a.x + a.width).min(b.x + b.width) - a.x.max(b.x);
            let ov_h = (a.y + a.height).min(b.y + b.height) - a.y.max(b.y);
            if ov_w <= 0.0 || ov_h <= 0.0 {
                continue;
            }
            let ov_area = ov_w * ov_h;
            let min_area = (a.width * a.height).min(b.width * b.height);
            if min_area > 0.0 && ov_area / min_area > 0.5 {
                // Overlapping duplicates: drop the larger (looser) one.
                if a.width * a.height >= b.width * b.height {
                    keep[i] = false;
                } else {
                    keep[j] = false;
                }
            }
        }
    }
    let mut it = keep.iter();
    tables.retain(|_| *it.next().unwrap_or(&true));

    tables
}

/// Detect tables using vector lines and text spans (main entry point for hybrid detection).
pub fn detect_tables_with_lines(
    spans: &[TextSpan],
    lines: &[crate::elements::PathContent],
    config: &TableDetectionConfig,
) -> Vec<Table> {
    if !config.enabled || spans.is_empty() {
        return Vec::new();
    }
    match (config.horizontal_strategy, config.vertical_strategy) {
        (TableStrategy::Text, TableStrategy::Text) => {
            return detect_tables_from_spans_column_aware(spans, config)
        },
        (TableStrategy::Lines, TableStrategy::Lines) => {
            // Try intersection-based detection first; fall back to cluster-based.
            let tables = detect_tables_from_intersections(spans, lines, config);
            if !tables.is_empty() {
                return tables.into_iter().filter(is_valid_table).collect();
            }
            let clusters = group_lines_into_clusters(lines, config);
            let mut tables = Vec::new();
            for cluster in clusters {
                tables.append(&mut detect_tables_in_cluster(spans, lines, &cluster, config));
            }
            return tables.into_iter().filter(is_valid_table).collect();
        },
        _ => {},
    }
    // Both / hybrid strategy: try intersection-based first, then cluster, then H-rule bounded,
    // then text fallback.
    let mut final_tables = detect_tables_from_intersections(spans, lines, config);
    if final_tables.is_empty() {
        let clusters = group_lines_into_clusters(lines, config);
        for cluster in clusters {
            final_tables.append(&mut detect_tables_in_cluster(spans, lines, &cluster, config));
        }
    }
    // When intersection and cluster pipelines found nothing, try H-rule bounded detection:
    // use horizontal lines as table region boundaries with text-edge column detection.
    if final_tables.is_empty() {
        let (mut h_edges, _) = extract_edges(lines);
        if !h_edges.is_empty() && !has_vertical_ruling_evidence(lines, &h_edges) {
            snap_and_merge(&mut h_edges);
            final_tables = detect_tables_from_horizontal_rules(spans, &h_edges, config);
            // A logical table ruled between row *bands* (a rule under the
            // header, or between groups of rows) is emitted here as one
            // fragment per band. Merge vertically-adjacent same-column
            // fragments BEFORE the min-row filter so a table cut into e.g. a
            // [3, 2] pair rejoins into [5] instead of losing its short band to
            // the guard below. Bands sit ~one inter-row pitch apart (the rule
            // stroke + leading), so scale the vertical tolerance to the
            // fragments' median row height rather than the abutting-fragment
            // default. Safety rests on the unchanged column gating in
            // `can_merge_tables` (equal col_count + matched X-start/width): a
            // lone spurious 2-row prose strip has no same-column neighbour, so
            // it stays short and is still dropped — the guard's intent holds.
            let row_h = median_fragment_row_height(&final_tables);
            let y_tol = (row_h * 1.5).max(3.0);
            final_tables = consolidate_adjacent_table_fragments_with_tol(final_tables, 2.0, y_tol);
            // H-rule bounded detection lacks vertical-line evidence —
            // columns come from text-edge clustering alone (same shape as
            // the text-only fallback below).  Two-row results are
            // virtually always prose that happens to live between
            // decorative rules (annotation underlines, page borders);
            // require three rows of evidence before promoting.
            final_tables.retain(|t| t.rows.len() >= 3);
        }
    }
    // Filter out invalid line-based tables BEFORE overlap checking so that
    // spurious line-based tables don't shadow valid text-based ones.
    final_tables.retain(is_valid_table);
    // The intersection grid is built from closed cells, so a rule-bounded
    // band whose rows are closed on only some sides never becomes a group.
    // A booktabs table whose row-groups are separated by full-width rules,
    // with hairlines between a few of its columns and a shaded last row per
    // group, yields a grid for the groups whose shaded rows and hairlines
    // close their cells — and nothing for a group's unshaded rows: they are
    // bounded above and below by rules and crossed by the hairlines, but
    // their outer cells have no side. The grid's own slice of that group is
    // its one shaded row, which the section-divider split then isolates and
    // the validity filter drops, so the group's rows fall to the prose flow
    // while the two groups beside it read as tables.
    //
    // ISO 32000-1:2008 §14.8.4.3.4 (docs/spec/pdf.md:37805) makes a row the
    // element that holds its cells, and a row-group bounded by rules is a
    // run of such rows whether or not every cell is boxed. So the rule
    // bands are read as well, with the detector the rules-only page uses,
    // and a band's table is kept where no grid table already covers it.
    if !final_tables.is_empty() {
        // Rules only: a shaded row's rectangle contributes an edge too, and
        // that edge would split a group's band at the shaded row.
        let rules: Vec<crate::elements::PathContent> = lines
            .iter()
            .filter(|p| p.is_horizontal_line(2.0))
            .cloned()
            .collect();
        let (mut h_edges, _) = extract_edges(&rules);
        if h_edges.len() >= 2 {
            snap_and_merge(&mut h_edges);
            let mut banded = detect_tables_from_horizontal_rules(spans, &h_edges, config);
            // Bands the grid already reads are dropped BEFORE adjacent
            // bands are consolidated: every rule pair of the table is a
            // band, and consolidating them first fuses the whole table into
            // one fragment that overlaps a grid table and is thrown away
            // with the group the grid missed.
            // Covered means a grid table spans most of the BAND's height:
            // the grid's one-row slice of a shaded row overlaps the band
            // it was cut from, and must not count as reading it.
            let covered = |band: &Table| {
                let Some(bb) = band.bbox else { return true };
                final_tables.iter().any(|t| {
                    t.bbox.is_some_and(|lb| {
                        let overlap = (lb.y + lb.height).min(bb.y + bb.height) - lb.y.max(bb.y);
                        overlap > 0.5 * bb.height
                            && lb.x < bb.x + bb.width
                            && bb.x < lb.x + lb.width
                    })
                })
            };
            // And a band is read only where it continues a grid table:
            // spanning most of THAT table's measure, and no more than a
            // couple of rows away. A form's field labels sit between rules
            // with no grid table beside them; a model summary's three lines
            // sit under a layers table but cover a fifth of its width. A
            // row-group of the same table runs its whole measure.
            let continues_a_grid_table = |band: &Table| {
                let Some(bb) = band.bbox else { return false };
                final_tables.iter().any(|t| {
                    t.bbox.is_some_and(|lb| {
                        let x_overlap = (lb.x + lb.width).min(bb.x + bb.width) - lb.x.max(bb.x);
                        let gap = if bb.y > lb.y + lb.height {
                            bb.y - (lb.y + lb.height)
                        } else {
                            lb.y - (bb.y + bb.height)
                        };
                        x_overlap >= 0.8 * lb.width && gap <= 24.0
                    })
                })
            };
            banded.retain(|b| !covered(b) && continues_a_grid_table(b));
            let row_h = median_fragment_row_height(&banded);
            let y_tol = (row_h * 1.5).max(3.0);
            banded = consolidate_adjacent_table_fragments_with_tol(banded, 2.0, y_tol);
            banded.retain(|t| t.rows.len() >= 3);
            banded.retain(is_valid_table);
            final_tables.extend(banded);
        }
    }

    // Only allow text-based fallback if BOTH strategies permit it AND the caller
    // explicitly enabled text-only detection (config.text_fallback=true).
    // This prevents extract_text() callers (text_fallback=false) from
    // spuriously running span-column detection alongside ruling-line tables:
    // report-style PDFs with decorative horizontal rules (e.g. swimming results)
    // would otherwise have all their data detected as a text table that renders
    // the page content a second time, causing duplicate extraction.
    // Callers that want text-based table detection (to_markdown, to_html) set
    // config.text_fallback=true explicitly.
    let allow_text_fallback = config.text_fallback
        && config.horizontal_strategy != TableStrategy::Lines
        && config.vertical_strategy != TableStrategy::Lines;

    if allow_text_fallback {
        let text_candidates = detect_tables_from_spans_column_aware(spans, config);
        for text_table in text_candidates {
            if !passes_spatial_quality_gate(&text_table) {
                continue;
            }
            // Text-only detection (no ruling lines) infers columns from word
            // x-alignment alone — two rows of column-aligned words is the
            // signature of ordinary prose (a title + a wrapped body line),
            // not a table.  Require at least three rows of evidence before
            // promoting a span cluster to a table.
            if text_table.rows.len() < 3 {
                continue;
            }
            if let Some(text_bbox) = text_table.bbox {
                let overlaps = final_tables.iter().any(|t| {
                    if let Some(line_bbox) = t.bbox {
                        line_bbox.intersects(&text_bbox)
                            || line_bbox.contains_rect(&text_bbox)
                            || text_bbox.contains_rect(&line_bbox)
                    } else {
                        false
                    }
                });
                if !overlaps {
                    final_tables.push(text_table);
                }
            }
        }
    }
    final_tables
}

fn grid_to_table(
    grid: &GridStructure,
    spans: &[TextSpan],
    visual_merge_info: Option<Vec<Vec<CellMergeInfo>>>,
) -> Table {
    let num_rows = grid.cells.len();
    let num_cols = grid.columns.len();
    let merge_info = visual_merge_info.unwrap_or_else(|| detect_merged_cells(grid, spans));
    let header_row_idx = detect_header_row(grid, spans);
    let mut table_rows = Vec::new();
    for (row_idx, row) in grid.cells.iter().enumerate() {
        let is_header = header_row_idx == Some(row_idx);
        let mut table_row = TableRow::new(is_header);
        for (col_idx, cell_span_indices) in row.iter().enumerate() {
            let mi = &merge_info[row_idx][col_idx];
            if mi.covered {
                continue;
            }
            let cell_text = extract_cell_text(cell_span_indices, spans);
            let mut cell_bbox = None;
            if !cell_span_indices.is_empty() {
                let mut b = spans[cell_span_indices[0]].bbox;
                for &idx in &cell_span_indices[1..] {
                    b = b.union(&spans[idx].bbox);
                }
                cell_bbox = Some(b);
            }
            let mcids = cell_span_indices
                .iter()
                .filter_map(|&idx| spans.get(idx).and_then(|s| s.mcid))
                .collect::<Vec<_>>();
            let cell_spans = cell_span_indices
                .iter()
                .filter_map(|&idx| spans.get(idx).cloned())
                .collect::<Vec<_>>();

            table_row.cells.push(TableCell {
                text: cell_text,
                spans: cell_spans,
                colspan: mi.colspan.min((num_cols - col_idx) as u32),
                rowspan: mi.rowspan.min((num_rows - row_idx) as u32),
                mcids,
                bbox: cell_bbox,
                is_header,
            });
        }
        table_rows.push(table_row);
    }
    let all_span_indices: Vec<usize> = grid
        .cells
        .iter()
        .flat_map(|row| row.iter().flat_map(|cell| cell.iter().copied()))
        .collect();
    let mut bbox = None;
    if !all_span_indices.is_empty() {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for &idx in &all_span_indices {
            if let Some(s) = spans.get(idx) {
                min_x = min_x.min(s.bbox.x);
                min_y = min_y.min(s.bbox.y);
                max_x = max_x.max(s.bbox.x + s.bbox.width);
                max_y = max_y.max(s.bbox.y + s.bbox.height);
            }
        }
        bbox = Some(crate::geometry::Rect::new(min_x, min_y, max_x - min_x, max_y - min_y));
    }
    Table {
        rows: table_rows,
        has_header: header_row_idx.is_some(),
        col_count: num_cols,
        bbox,
    }
}

fn extract_cell_text(cell_span_indices: &[usize], spans: &[TextSpan]) -> String {
    if cell_span_indices.is_empty() {
        return String::new();
    }
    // Keep the span reference (not just text) so we can decide spacing based
    // on the geometric gap and CJK/fullwidth-operator boundary state, exactly
    // like the inline-flow path does in pipeline/converters/mod.rs.  Without
    // this, the previous `line.join(" ")` was unconditionally inserting a
    // space between every adjacent span on the same row, splitting compound
    // tokens like `40000≤Q＜55000` into `40000≤Q ＜55000` and dropping word-F1
    // for table-heavy CJK documents (issue 484, issue-336).
    let mut span_entries: Vec<(f32, &TextSpan, String)> = cell_span_indices
        .iter()
        .filter_map(|&idx| {
            spans
                .get(idx)
                .map(|s| (s.bbox.center().y, s, span_text_for_cell(s)))
        })
        .collect();
    if span_entries.is_empty() {
        return String::new();
    }
    if span_entries.len() == 1 {
        return span_entries.remove(0).2;
    }
    // Order the cell's members by row band, then left to right — the same
    // comparator the reading-order passes use.
    //
    // The key used to be `bbox.center().y` descending with no x tiebreak. A
    // centre moves with font size, so a raised marker in a smaller font (`*` at
    // 4.98 pt beside body text at 7.98 pt) sorted to the FRONT of the cell
    // while staying inside the line grouping, and the backward gap to the
    // member now behind it produced no separator: `142.56 ± 59.19*^` read back
    // as `59.19*^142.56 ±`.
    //
    // Keying on the baseline was tried before the glyph-width fix and had to be
    // reverted; with widths taken from the measured offsets the geometry this
    // sees is no longer distorted.
    span_entries.sort_by(|a, b| {
        crate::utils::row_aware_span_cmp(a.1.bbox.y, a.1.bbox.x, b.1.bbox.y, b.1.bbox.x)
    });

    // Group into rows by y proximity, then within a row decide separator per
    // pair of spans using the same gap/CJK rules as inline text assembly.
    let mut lines: Vec<Vec<(&TextSpan, String)>> = Vec::new();
    let mut current_line: Vec<(&TextSpan, String)> =
        vec![(span_entries[0].1, span_entries[0].2.clone())];
    let mut current_y = span_entries[0].0;
    for (y, span, text) in &span_entries[1..] {
        if (current_y - y).abs() <= 2.0 {
            current_line.push((span, text.clone()));
        } else {
            lines.push(current_line);
            current_line = vec![(span, text.clone())];
            current_y = *y;
        }
    }
    lines.push(current_line);

    lines
        .iter()
        .map(|line| {
            let mut out = String::new();
            for (i, (span, text)) in line.iter().enumerate() {
                if i > 0 {
                    let (prev_span, _) = line[i - 1];
                    let separator = cell_span_separator(prev_span, span);
                    out.push_str(separator);
                }
                out.push_str(text);
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Decide what (if any) separator to put between two spans within the same
/// table cell row.  Mirrors the inline-flow has_horizontal_gap logic from
/// pipeline/converters/mod.rs: insert a space only when there is a real
/// horizontal gap that exceeds the inter-glyph kerning floor AND the
/// boundary is not a CJK ↔ CJK / CJK ↔ fullwidth-operator pair (issue 485).
fn cell_span_separator(prev: &TextSpan, current: &TextSpan) -> &'static str {
    // Already-present whitespace at the join point — never duplicate.
    if prev.text.ends_with(' ') || current.text.starts_with(' ') {
        return "";
    }

    let prev_end_x = prev.bbox.x + prev.bbox.width;
    let gap = current.bbox.x - prev_end_x;
    let font_size = prev.font_size.max(current.font_size).max(1.0);

    // Sub-em gap: glyphs are touching or overlapping (typical inter-glyph
    // advance).  Don't insert a space — adjacent characters in the same
    // word/expression must stay glued.  This is what `40000≤Q` + `＜55000`
    // hits: gap is essentially zero (the source PDF emits the operator as
    // its own positioned Tj) but the two spans are part of one compound
    // token in pdftotext's output.
    let space_threshold = font_size * 0.15;
    if gap <= space_threshold {
        return "";
    }
    // No upper bound: a very large gap (≥ 5 em) used to be treated as a
    // column boundary and yield no separator, but the caller concatenates
    // span text when this returns "" — so tokens like `3.80%` and `4.41%`
    // were rendered as `3.80%4.41%` on wide rate tables.  Mirroring the
    // inline-flow rule (`pipeline::converters::has_horizontal_gap`), any
    // gap above the inter-glyph threshold now gets at least a single
    // space.

    // CJK / fullwidth-operator suppression — same rule as
    // pipeline::converters::has_horizontal_gap.  pdftotext keeps an
    // ideograph + adjacent fullwidth/math operator without a separator.
    let is_cjk = |c: char| {
        matches!(
            c as u32,
            0x3040..=0x309F     // Hiragana
            | 0x30A0..=0x30FF   // Katakana
            | 0x4E00..=0x9FFF   // CJK Unified Ideographs
            | 0xAC00..=0xD7AF   // Hangul
            | 0x3400..=0x4DBF   // CJK Extension A
            | 0x20000..=0x2A6DF // CJK Extension B
        )
    };
    let is_fw_op = |c: char| {
        matches!(
            c as u32,
            0xFF0B | 0xFF0D | 0xFF1A | 0xFF1B
            | 0xFF1C..=0xFF1E       // ＜ ＝ ＞
            | 0x2260 | 0x2248
            | 0x2264..=0x2265       // ≤ ≥
            | 0x00B5 | 0x03BC       // µ μ
            | 0x00B1 | 0x00D7 | 0x00F7
        )
    };
    let prev_tail = prev.text.chars().next_back();
    let curr_head = current.text.chars().next();
    if let (Some(p), Some(c)) = (prev_tail, curr_head) {
        let p_cjk = is_cjk(p);
        let c_cjk = is_cjk(c);
        if (p_cjk || is_fw_op(p)) && (c_cjk || is_fw_op(c)) && (p_cjk || c_cjk) {
            return "";
        }
    }

    " "
}

/// Consolidate vertically-adjacent tables that share an identical column
/// structure into a single multi-row table.
///
/// Issue 484/486/487 root cause: when a logical multi-row table is drawn
/// with a horizontal ruling line between every pair of rows (rather than
/// only at the top and bottom), the line-based detector emits one Table
/// per row strip. Each fragment is a 1- or 2-row table that fails
/// `is_real_grid()` (which requires ≥2 rows) and gets dropped, after
/// which the cells fall through to the paragraph flow with column-based
/// reading order — producing orphan `<p>40000≤Q</p>` / `<p>＜55000</p>`
/// pairs instead of `<table><td>40000≤Q＜55000</td></tr></table>`.
///
/// Two fragments are merge-candidates when:
///   * both have a `bbox`
///   * X start matches within `X_TOLERANCE`
///   * width matches within `X_TOLERANCE`
///   * column counts are equal
///   * the lower fragment's top edge (`bbox.y + bbox.height`) is within
///     `Y_TOLERANCE` of the upper fragment's bottom edge (`bbox.y`)
///
/// Sort tables top-down (PDF y-up: largest top-Y first) and merge runs
/// of consecutive fragments that satisfy the criteria. The merged table
/// preserves the union of all rows and a bbox spanning both fragments.
pub fn consolidate_adjacent_table_fragments(tables: Vec<Table>) -> Vec<Table> {
    consolidate_adjacent_table_fragments_with_tol(tables, 2.0, 3.0)
}

/// Median per-row height across table fragments, estimated as each
/// fragment's bbox height divided by its row count. Used to scale the
/// vertical merge tolerance for rule-split bands. Returns 0.0 when no
/// fragment carries usable geometry.
fn median_fragment_row_height(tables: &[Table]) -> f32 {
    let mut heights: Vec<f32> = tables
        .iter()
        .filter_map(|t| {
            let b = t.bbox?;
            let n = t.rows.len();
            if n == 0 || b.height <= 0.0 {
                None
            } else {
                Some(b.height / n as f32)
            }
        })
        .collect();
    if heights.is_empty() {
        return 0.0;
    }
    heights.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
    heights[heights.len() / 2]
}

/// Like [`consolidate_adjacent_table_fragments`] but with a caller-chosen
/// vertical merge tolerance. The default 3.0 pt tolerance assumes fragments
/// abut (a ruling line between every row leaves ~0 gap). The H-rule-bounded
/// detector, however, emits one fragment per rule-delimited band, so two
/// bands of the SAME logical table are separated by a full inter-row pitch
/// (rule stroke + leading) — often ~10-15 pt. A larger `y_tol` lets those
/// bands rejoin. Safety rests on the unchanged column gating in
/// `can_merge_tables` (equal col_count + X-start ≤ x_tol + width ≤ x_tol):
/// two genuinely distinct tables sharing all three within one row-height are
/// vanishingly rare, whereas bands of one ruled table match them exactly.
pub fn consolidate_adjacent_table_fragments_with_tol(
    tables: Vec<Table>,
    x_tolerance: f32,
    y_tolerance: f32,
) -> Vec<Table> {
    if tables.len() < 2 {
        return tables;
    }

    // Sort by top-Y descending (top of page first in PDF y-up coordinates).
    let mut sorted = tables;
    sorted.sort_by(|a, b| {
        let a_top = a.bbox.map(|b| b.y + b.height).unwrap_or(f32::NEG_INFINITY);
        let b_top = b.bbox.map(|b| b.y + b.height).unwrap_or(f32::NEG_INFINITY);
        crate::utils::safe_float_cmp(b_top, a_top)
    });

    let mut consolidated: Vec<Table> = Vec::with_capacity(sorted.len());
    for table in sorted {
        let merge_into_last = consolidated
            .last()
            .map(|last| can_merge_tables(last, &table, x_tolerance, y_tolerance))
            .unwrap_or(false);
        if merge_into_last {
            // Safety: merge_into_last is only true when consolidated.last()
            // returned Some, so last_mut() must also return Some.
            if let Some(last) = consolidated.last_mut() {
                merge_table_into(last, table);
            }
        } else {
            consolidated.push(table);
        }
    }
    consolidated
}

fn can_merge_tables(upper: &Table, lower: &Table, x_tol: f32, y_tol: f32) -> bool {
    let (Some(u_bbox), Some(l_bbox)) = (upper.bbox, lower.bbox) else {
        return false;
    };
    if upper.col_count != lower.col_count || upper.col_count == 0 {
        return false;
    }
    if (u_bbox.x - l_bbox.x).abs() > x_tol {
        return false;
    }
    if (u_bbox.width - l_bbox.width).abs() > x_tol {
        return false;
    }
    // upper sits ABOVE lower in PDF y-up: upper.bbox.y is the BOTTOM of
    // upper, lower.bbox.y + lower.bbox.height is the TOP of lower.
    // For them to be vertically adjacent, the upper.bottom must be close
    // to the lower.top.  We allow a small NEGATIVE gap (overlap) up to
    // half the smaller table's height — the line-based detector
    // occasionally produces bboxes that overhang the adjacent table by a
    // few points when ruling-rule strokes have non-zero thickness or
    // include the line's drawn extent above/below the baseline.  Real
    // distinct tables almost always have a meaningful positive gap.
    let upper_bottom = u_bbox.y;
    let lower_top = l_bbox.y + l_bbox.height;
    let gap = upper_bottom - lower_top;
    if gap > y_tol {
        return false;
    }
    let min_height = u_bbox.height.min(l_bbox.height);
    if -gap > min_height * 0.5 {
        return false;
    }
    true
}

fn merge_table_into(upper: &mut Table, lower: Table) {
    if let (Some(ub), Some(lb)) = (upper.bbox, lower.bbox) {
        let new_y = ub.y.min(lb.y);
        let new_top = (ub.y + ub.height).max(lb.y + lb.height);
        let new_x = ub.x.min(lb.x);
        let new_right = (ub.x + ub.width).max(lb.x + lb.width);
        upper.bbox = Some(crate::geometry::Rect {
            x: new_x,
            y: new_y,
            width: new_right - new_x,
            height: new_top - new_y,
        });
    }
    upper.rows.extend(lower.rows);
}

fn detect_merged_cells(grid: &GridStructure, spans: &[TextSpan]) -> Vec<Vec<CellMergeInfo>> {
    let num_rows = grid.cells.len();
    let num_cols = grid.columns.len();
    let mut merge_info: Vec<Vec<CellMergeInfo>> = (0..num_rows)
        .map(|_| {
            (0..num_cols)
                .map(|_| CellMergeInfo {
                    colspan: 1,
                    rowspan: 1,
                    covered: false,
                })
                .collect()
        })
        .collect();
    for row_idx in 0..num_rows {
        for col_idx in 0..num_cols {
            if grid.cells[row_idx][col_idx].is_empty() {
                continue;
            }
            let cell_right = grid.cells[row_idx][col_idx]
                .iter()
                .filter_map(|&idx| spans.get(idx).map(|s| s.bbox.right()))
                .fold(f32::NEG_INFINITY, f32::max);
            if cell_right == f32::NEG_INFINITY {
                continue;
            }
            let mut extra_cols = 0u32;
            for next_col in (col_idx + 1)..num_cols {
                if !grid.cells[row_idx][next_col].is_empty() {
                    break;
                }
                if cell_right > grid.columns[next_col].x_center {
                    extra_cols += 1;
                } else {
                    break;
                }
            }
            if extra_cols > 0 {
                merge_info[row_idx][col_idx].colspan = 1 + extra_cols;
                for c in 1..=(extra_cols as usize) {
                    merge_info[row_idx][col_idx + c].covered = true;
                }
            }
        }
    }
    for col_idx in 0..num_cols {
        for row_idx in 0..num_rows {
            if grid.cells[row_idx][col_idx].is_empty() || merge_info[row_idx][col_idx].covered {
                continue;
            }
            let cell_bottom = grid.cells[row_idx][col_idx]
                .iter()
                .filter_map(|&idx| spans.get(idx).map(|s| s.bbox.bottom()))
                .fold(f32::INFINITY, f32::min);
            if cell_bottom == f32::INFINITY {
                continue;
            }
            let mut extra_rows = 0u32;
            for next_row in (row_idx + 1)..num_rows {
                if !grid.cells[next_row][col_idx].is_empty() {
                    break;
                }
                if cell_bottom < grid.rows[next_row].y_center {
                    extra_rows += 1;
                } else {
                    break;
                }
            }
            if extra_rows > 0 {
                merge_info[row_idx][col_idx].rowspan = 1 + extra_rows;
                for r in 1..=(extra_rows as usize) {
                    merge_info[row_idx + r][col_idx].covered = true;
                }
            }
        }
    }
    merge_info
}

fn detect_header_row(grid: &GridStructure, spans: &[TextSpan]) -> Option<usize> {
    if grid.cells.len() < 2 {
        return None;
    }
    let first_row_spans: Vec<&TextSpan> = grid.cells[0]
        .iter()
        .flat_map(|cell| cell.iter().filter_map(|&idx| spans.get(idx)))
        .collect();
    if first_row_spans.is_empty() {
        return None;
    }
    let data_row_spans: Vec<&TextSpan> = grid.cells[1..]
        .iter()
        .flat_map(|row| {
            row.iter()
                .flat_map(|cell| cell.iter().filter_map(|&idx| spans.get(idx)))
        })
        .collect();
    if data_row_spans.is_empty() {
        return None;
    }
    let first_row_bold_ratio = first_row_spans
        .iter()
        .filter(|s| s.font_weight.is_bold())
        .count() as f32
        / first_row_spans.len() as f32;
    let data_bold_ratio = data_row_spans
        .iter()
        .filter(|s| s.font_weight.is_bold())
        .count() as f32
        / data_row_spans.len() as f32;
    if first_row_bold_ratio > 0.5 && data_bold_ratio < 0.3 {
        return Some(0);
    }
    let first_row_avg_size: f32 =
        first_row_spans.iter().map(|s| s.font_size).sum::<f32>() / first_row_spans.len() as f32;
    let data_avg_size: f32 =
        data_row_spans.iter().map(|s| s.font_size).sum::<f32>() / data_row_spans.len() as f32;
    if first_row_avg_size > data_avg_size + 1.5 {
        return Some(0);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::text_block::{Color, FontWeight};

    #[test]
    fn test_is_numeric_cell() {
        for ok in ["0.69", "100", "-1.2", "52%", "0", "1.00", "\u{2212}3.5"] {
            assert!(is_numeric_cell(ok), "{ok:?} should be numeric");
        }
        for no in [
            "Ours",
            "GLUE",
            "v3",
            "1e9",
            "0.6.6",
            "",
            "12345678.9",
            "p<0.05",
        ] {
            assert!(!is_numeric_cell(no), "{no:?} should NOT be numeric");
        }
    }

    fn col_at(x: f32) -> ColumnCluster {
        ColumnCluster {
            x_center: x,
            x_min: x - 3.0,
            x_max: x + 3.0,
            span_indices: Vec::new(),
        }
    }

    #[test]
    fn grid_interval_prefers_exact_side_near_internal_boundary() {
        let boundaries = [0.0, 40.0, 70.0, 100.0];

        assert_eq!(grid_interval_for_point(68.5, &boundaries), Some(1));
        assert_eq!(grid_interval_for_point(71.0, &boundaries), Some(2));
        assert_eq!(grid_interval_for_point(70.0, &boundaries), Some(2));
        assert_eq!(grid_interval_for_point(-2.0, &boundaries), Some(0));
        assert_eq!(grid_interval_for_point(102.0, &boundaries), Some(2));
        assert_eq!(grid_interval_for_point(104.0, &boundaries), None);
    }

    #[test]
    fn intersection_grid_keeps_superscripts_and_boundary_text_in_their_cells() {
        let group_cells = [
            IntersectionCell {
                x1: 0.0,
                y1: 0.0,
                x2: 40.0,
                y2: 20.0,
            },
            IntersectionCell {
                x1: 40.0,
                y1: 0.0,
                x2: 70.0,
                y2: 20.0,
            },
            IntersectionCell {
                x1: 70.0,
                y1: 0.0,
                x2: 100.0,
                y2: 20.0,
            },
        ];
        let spans = vec![
            create_test_span("273", 10.0, 5.0, 27.0, 10.0),
            create_test_span("1", 37.0, 5.0, 2.0, 6.0),
            create_test_span("83", 45.0, 5.0, 21.0, 10.0),
            create_test_span("2", 66.0, 5.0, 2.0, 6.0),
            // Its center is one point into the third cell but within SNAP_TOL
            // of the second cell's right boundary.
            create_test_span("Europe", 66.0, 5.0, 10.0, 10.0),
        ];

        let (rows, _) = assign_spans_to_intersection_grid(
            &group_cells,
            &[0.0, 40.0, 70.0, 100.0],
            &[0.0, 20.0],
            3,
            &spans,
        )
        .expect("synthetic grid should be valid");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells[0].text, "2731");
        assert_eq!(rows[0].cells[1].text, "832");
        assert_eq!(rows[0].cells[2].text, "Europe");
    }

    #[test]
    fn test_is_regular_lattice() {
        // Regular ~20pt pitch with one wider row-label gap on the left.
        let regular: Vec<ColumnCluster> = [113.0, 150.0, 170.0, 190.0, 210.0, 230.0]
            .iter()
            .map(|&x| col_at(x))
            .collect();
        assert!(is_regular_lattice(&regular));

        // Fewer than 5 columns → not a lattice.
        let small: Vec<ColumnCluster> = [100.0, 200.0, 300.0].iter().map(|&x| col_at(x)).collect();
        assert!(!is_regular_lattice(&small));

        // Irregular gaps (prose that happened to align) → rejected.
        let irregular: Vec<ColumnCluster> = [100.0, 140.0, 320.0, 330.0, 500.0, 505.0]
            .iter()
            .map(|&x| col_at(x))
            .collect();
        assert!(!is_regular_lattice(&irregular));
    }

    #[test]
    fn test_is_data_value() {
        for v in ["5,012", "+2%", "240", "-1.5", "1,000.50", "67", "\u{2212}3"] {
            assert!(is_data_value(v), "{v:?} should be a data value");
        }
        for w in ["FY22", "Mercury", "Body", "", "YoY", "$", "+", "Q1"] {
            assert!(!is_data_value(w), "{w:?} should NOT be a data value");
        }
    }

    #[test]
    fn test_quality_gate_admits_numeric_table_rejects_prose_split() {
        let row = |cells: &[&str]| TableRow {
            cells: cells.iter().map(|c| prose_cell(c)).collect(),
            is_header: false,
        };
        // A dense numeric metrics table: ~all single-token numeric cells. Must
        // PASS (the prose ratio excludes data values).
        let mut numeric = Table::new();
        numeric.col_count = 8;
        for r in [
            ["Body", "FY22", "FY23", "FY24", "FY25", "YoY", "Plan", "Var"],
            [
                "Mercury Transits",
                "5,012",
                "5,210",
                "5,488",
                "5,612",
                "+2%",
                "5,600",
                "+12",
            ],
            [
                "Venus Phases",
                "1,840",
                "1,902",
                "1,975",
                "2,041",
                "+3%",
                "2,030",
                "+11",
            ],
        ] {
            numeric.rows.push(row(&r));
        }
        assert!(
            passes_spatial_quality_gate(&numeric),
            "dense numeric table must pass the spatial quality gate"
        );

        // Prose accidentally split into single-word columns must still be REJECTED.
        let mut prose = Table::new();
        prose.col_count = 6;
        for _ in 0..3 {
            prose
                .rows
                .push(row(&["the", "quick", "brown", "fox", "jumps", "over"]));
        }
        assert!(
            !passes_spatial_quality_gate(&prose),
            "word-dominated single-word split must still be rejected"
        );
    }

    fn prose_cell(text: &str) -> TableCell {
        TableCell {
            text: text.to_string(),
            spans: Vec::new(),
            colspan: 1,
            rowspan: 1,
            mcids: Vec::new(),
            bbox: None,
            is_header: false,
        }
    }

    #[test]
    fn test_looks_like_cjk_prose() {
        let row = |cells: &[&str]| TableRow {
            cells: cells.iter().map(|c| prose_cell(c)).collect(),
            is_header: false,
        };
        // CJK prose mis-split into columns: a long ideograph/kana run in a cell.
        let mut prose = Table::new();
        prose.col_count = 2;
        prose.rows.push(row(&[
            "\u{30CD}\u{30B3}",
            "\u{72ED}\u{7FA9}\u{306B}\u{306F}\u{98DF}\u{8089}\u{76EE}\u{30CD}\u{30B3}\u{79D1}\u{30CD}\u{30B3}\u{5C5E}",
        ]));
        assert!(looks_like_cjk_prose(&prose));

        // A genuine CJK data table — short label + number cells — is NOT prose.
        let mut table = Table::new();
        table.col_count = 2;
        table.rows.push(row(&["\u{58F2}\u{4E0A}", "1,234"]));
        table.rows.push(row(&["\u{5229}\u{76CA}", "567"]));
        assert!(!looks_like_cjk_prose(&table));

        // Latin prose has no long CJK run.
        let mut latin = Table::new();
        latin.col_count = 2;
        latin.rows.push(row(&["Region", "North"]));
        assert!(!looks_like_cjk_prose(&latin));
    }

    #[test]
    fn test_looks_like_bulleted_list_rejects_bullet_cells() {
        let row = |cells: &[&str]| TableRow {
            cells: cells.iter().map(|c| prose_cell(c)).collect(),
            is_header: false,
        };
        // A bulleted list mis-fused into two columns: lone bullet markers in
        // the first column → recognise as a list, not a table.
        let mut list = Table::new();
        list.col_count = 2;
        list.rows.push(row(&["\u{2022}", "Ship the API."]));
        list.rows.push(row(&["\u{2022}", "Write the docs."]));
        assert!(looks_like_bulleted_list(&list));

        // A genuine two-column data table has no lone-bullet cell.
        let mut table = Table::new();
        table.col_count = 2;
        table.rows.push(row(&["Region", "Q1"]));
        table.rows.push(row(&["North", "120"]));
        assert!(!looks_like_bulleted_list(&table));

        // A cell that merely *starts* with a dash but carries text (a value or
        // a negative number) is not a bullet marker.
        let mut dash = Table::new();
        dash.col_count = 2;
        dash.rows.push(row(&["Delta", "-12"]));
        assert!(!looks_like_bulleted_list(&dash));
    }

    /// #09 prose gate: a wrapped paragraph mis-split into a table — a row
    /// crossing a sentence boundary ("...to 23,500. Stockout rate...") must
    /// be recognised as prose and rejected.
    #[test]
    fn test_looks_like_prose_paragraph_detects_sentence_crossing_row() {
        let mut t = Table::new();
        t.col_count = 4;
        t.rows.push(TableRow {
            cells: vec![
                prose_cell("Total SKU count grew 15%"),
                prose_cell("quarter-over-quarter to"),
                prose_cell("23,500."),
                prose_cell("Stockout rate improved by 200 basis"),
            ],
            is_header: false,
        });
        assert!(looks_like_prose_paragraph(&t));
    }

    /// Caseless scripts (Bengali here) have no capital-letter signal at all,
    /// so their sentence-final danda ('।') must be treated as a terminator
    /// in its own right. Real-world PDFs sometimes render the danda with a
    /// stray gap before it ("প্রাণী ।"), so the check must tolerate that.
    #[test]
    fn test_looks_like_prose_paragraph_detects_bengali_danda_crossing_row() {
        let mut t = Table::new();
        t.col_count = 4;
        t.rows.push(TableRow {
            cells: vec![
                prose_cell("বিড়াল একটি গার্হস্থ্য প্রজাতি"),
                prose_cell("স্তন্যপায়ী প্রাণী"),
                prose_cell("। এটি"),
                prose_cell("ফেলিডা পরিবারের একমাত্র গৃহপালিত প্রজাতি"),
            ],
            is_header: false,
        });
        assert!(looks_like_prose_paragraph(&t));
    }

    /// REGRESSION GUARD: a genuine data table (short value/label cells, no
    /// sentence crossing a row) must NOT be flagged as prose.
    #[test]
    fn test_looks_like_prose_paragraph_keeps_real_table() {
        let mut t = Table::new();
        t.col_count = 4;
        for cells in [
            ["Zone", "Pallets stored", "11,100", "-2.5%"],
            ["A", "Utilization", "87%", "-3pp"],
            ["B", "Damage rate", "0.3%", "-0.2pp"],
        ] {
            t.rows.push(TableRow {
                cells: cells.iter().map(|c| prose_cell(c)).collect(),
                is_header: false,
            });
        }
        assert!(!looks_like_prose_paragraph(&t));
    }

    #[test]
    fn test_line_clustering_multiple_tables() {
        let lines = vec![
            make_rect_path(10.0, 100.0, 50.0, 20.0),
            make_rect_path(10.0, 50.0, 50.0, 20.0), // Far away vertically
        ];
        let config = TableDetectionConfig::default();
        let clusters = group_lines_into_clusters(&lines, &config);
        assert_eq!(
            clusters.len(),
            2,
            "Should find 2 separate table regions with optimized clustering"
        );
    }

    #[test]
    fn test_line_clustering_horizontal_separation() {
        let lines = vec![
            make_rect_path(10.0, 100.0, 50.0, 20.0), // Table 1: x=10..60
            make_rect_path(80.0, 100.0, 50.0, 20.0), // Table 2: x=80..130 (20pt gap)
        ];
        let config = TableDetectionConfig::default();
        let clusters = group_lines_into_clusters(&lines, &config);
        assert_eq!(
            clusters.len(),
            2,
            "Should find 2 separate table regions even if nearby horizontally"
        );
    }

    fn create_test_span(text: &str, x: f32, y: f32, width: f32, height: f32) -> TextSpan {
        TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: text.to_string(),
            bbox: Rect::new(x, y, width, height),
            font_name: "TestFont".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            split_boundary_before: false,
            offset_semantic: false,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 1.0,
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
    fn make_h_line(x: f32, y: f32, width: f32) -> crate::elements::PathContent {
        crate::elements::PathContent::line(x, y, x + width, y)
    }
    fn make_v_line(x: f32, y: f32, height: f32) -> crate::elements::PathContent {
        crate::elements::PathContent::line(x, y, x, y + height)
    }
    fn make_line_path(x1: f32, y1: f32, x2: f32, y2: f32) -> crate::elements::PathContent {
        crate::elements::PathContent::line(x1, y1, x2, y2)
    }
    fn make_rect_path(x: f32, y: f32, w: f32, h: f32) -> crate::elements::PathContent {
        crate::elements::PathContent::rect(x, y, w, h)
    }

    /// Issue #6/#5: an agenda-style table has 3 real columns (Time @72,
    /// Activity @200, Team @420). The Activity cell holds multiple words
    /// laid out with wide gaps ("Receiving Dock Inspection"), each at a
    /// distinct X that occurs in only ONE row. Greedy column clustering
    /// turns every word X into a column; the cross-row text-edge
    /// detector must instead recover the 3 real columns whose edges
    /// recur across rows. Asserts the detected table has 3 columns, not
    /// one-per-word.
    #[test]
    fn test_issue6_agenda_words_not_split_into_columns() {
        // y descending = rows top→bottom. 4 rows incl. header.
        let spans = vec![
            // Header row.
            create_test_span("Time", 72.0, 638.6, 24.4, 12.0),
            create_test_span("Activity", 200.0, 638.6, 34.8, 12.0),
            create_test_span("Team", 420.0, 638.6, 28.1, 12.0),
            // Row 1: Activity = "Receiving Dock Inspection" (3 word spans).
            create_test_span("06:00 - 07:00", 72.0, 610.6, 61.1, 12.0),
            create_test_span("Receiving", 200.0, 610.6, 43.9, 12.0),
            create_test_span("Dock", 249.9, 610.6, 22.8, 12.0),
            create_test_span("Inspection", 278.7, 610.6, 45.6, 12.0),
            create_test_span("Inbound Team", 420.0, 610.6, 65.7, 12.0),
            // Row 2: Activity = "Bulk Putaway Slotting".
            create_test_span("07:00 - 09:00", 72.0, 582.6, 61.1, 12.0),
            create_test_span("Bulk", 200.0, 582.6, 19.5, 12.0),
            create_test_span("Putaway", 225.4, 582.6, 38.3, 12.0),
            create_test_span("Slotting", 282.5, 582.6, 33.4, 12.0),
            create_test_span("Warehouse Ops", 420.0, 582.6, 73.5, 12.0),
            // Row 3: Activity = "Pick Wave Processing".
            create_test_span("09:00 - 11:00", 72.0, 554.6, 61.1, 12.0),
            create_test_span("Pick", 200.0, 554.6, 18.9, 12.0),
            create_test_span("Wave", 230.0, 554.6, 24.0, 12.0),
            create_test_span("Processing", 262.0, 554.6, 48.0, 12.0),
            create_test_span("Fulfillment", 420.0, 554.6, 55.0, 12.0),
        ];
        let config = TableDetectionConfig::default();
        let tables = detect_tables_from_spans(&spans, &config);
        // Either no table (acceptable — agenda is borderline tabular) or
        // a table with the 3 real columns. What must NOT happen: a table
        // with one column per Activity word (>= 5 columns).
        if let Some(t) = tables.first() {
            let ncols = t.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
            assert!(
                ncols <= 4,
                "agenda must not fragment Activity words into columns; got {} cols",
                ncols
            );
        }
    }

    #[test]
    fn test_lines_strategy_no_lines_returns_empty() {
        let spans = vec![
            create_test_span("A", 10.0, 100.0, 10.0, 10.0),
            create_test_span("B", 50.0, 100.0, 10.0, 10.0),
            create_test_span("C", 10.0, 80.0, 10.0, 10.0),
            create_test_span("D", 50.0, 80.0, 10.0, 10.0),
        ];
        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            ..TableDetectionConfig::default()
        };
        assert!(detect_tables_with_lines(&spans, &[], &config).is_empty());
    }

    #[test]
    fn test_horizontal_lines_only_strategy_no_false_positives() {
        // Regression test: horizontal_strategy: "lines" should NOT fall back to
        // text-based detection when there are no lines on the page.
        let spans = vec![
            create_test_span("A", 10.0, 100.0, 10.0, 10.0),
            create_test_span("B", 50.0, 100.0, 10.0, 10.0),
            create_test_span("C", 10.0, 80.0, 10.0, 10.0),
            create_test_span("D", 50.0, 80.0, 10.0, 10.0),
        ];
        // Test with horizontal_strategy: Lines but vertical_strategy: Both (the default)
        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Both,
            ..TableDetectionConfig::default()
        };
        // Should return empty because there are no horizontal lines to define rows
        assert!(detect_tables_with_lines(&spans, &[], &config).is_empty());
    }

    /// Regression test for issue #486: text-only spatial fallback for line-less tables.
    ///
    /// `text_fallback = true` is now the default on `TableDetectionConfig` (the
    /// prose-shape filter and ≥3-row guard suppress the false positives that
    /// previously motivated a `false` default).  With the default and the `Both`
    /// strategy, `detect_tables_with_lines` with an empty lines slice falls
    /// through to the text-based path and detects the grid from span alignment
    /// alone.  Callers that explicitly want the conservative
    /// "no ruling lines → no tables" behaviour set `text_fallback = false` and
    /// the `extract_page_tables` early-return guard in document.rs short-circuits
    /// before this code is reached.
    ///
    /// This test directly calls `detect_tables_with_lines` with an empty lines
    /// slice to verify that the text-based path inside it finds the table.
    #[test]
    fn test_text_fallback_detects_lineless_grid() {
        // Simulate a 3-column, 4-row sailing-score table with no ruling lines.
        // Columns at x=10, 50, 90; rows at y=200, 180, 160, 140.
        let spans = vec![
            // Row 1
            create_test_span("Pos", 10.0, 200.0, 25.0, 10.0),
            create_test_span("Boat", 50.0, 200.0, 25.0, 10.0),
            create_test_span("Pts", 90.0, 200.0, 20.0, 10.0),
            // Row 2
            create_test_span("1", 10.0, 180.0, 25.0, 10.0),
            create_test_span("Alpha", 50.0, 180.0, 25.0, 10.0),
            create_test_span("14", 90.0, 180.0, 20.0, 10.0),
            // Row 3
            create_test_span("2", 10.0, 160.0, 25.0, 10.0),
            create_test_span("Beta", 50.0, 160.0, 25.0, 10.0),
            create_test_span("17", 90.0, 160.0, 20.0, 10.0),
            // Row 4
            create_test_span("3", 10.0, 140.0, 25.0, 10.0),
            create_test_span("Gamma", 50.0, 140.0, 25.0, 10.0),
            create_test_span("21", 90.0, 140.0, 20.0, 10.0),
        ];

        // With the Both strategy, NO lines, and text_fallback explicitly
        // enabled, the text-based fallback inside detect_tables_with_lines
        // fires and finds the grid.  Issue 484: the default no longer enables
        // text_fallback to avoid spurious tables on report-style PDFs that
        // would otherwise be double-emitted by extract_text.
        let config = TableDetectionConfig {
            text_fallback: true,
            ..TableDetectionConfig::default()
        };
        let tables = detect_tables_with_lines(&spans, &[], &config);
        assert_eq!(
            tables.len(),
            1,
            "Text-only fallback in detect_tables_with_lines should detect the grid (got {:?} tables)",
            tables.len()
        );
        let t = &tables[0];
        assert_eq!(t.col_count, 3, "Should detect 3 columns");
        assert_eq!(t.rows.len(), 4, "Should detect 4 rows");
    }

    /// Verify that when no lines are present and `text_fallback = false` (the default),
    /// the guard in `extract_page_tables` (outside `detect_tables_with_lines`) would
    /// prevent the text path from running.  We simulate this at the config level: using
    /// a `Lines`-only strategy ensures `detect_tables_with_lines` returns nothing when
    /// paths are empty — confirming the safety contract for the public API path.
    #[test]
    fn test_text_fallback_disabled_lines_strategy_returns_empty() {
        let spans = vec![
            create_test_span("Pos", 10.0, 200.0, 25.0, 10.0),
            create_test_span("Boat", 50.0, 200.0, 25.0, 10.0),
            create_test_span("Pts", 90.0, 200.0, 20.0, 10.0),
            create_test_span("1", 10.0, 180.0, 25.0, 10.0),
            create_test_span("Alpha", 50.0, 180.0, 25.0, 10.0),
            create_test_span("14", 90.0, 180.0, 20.0, 10.0),
        ];
        // Lines-only strategy: no lines → no tables.  This is what the public
        // extract_tables() API uses after the early-return guard fires.
        let config = TableDetectionConfig::strict(); // strict() uses Lines/Lines
        let tables = detect_tables_with_lines(&spans, &[], &config);
        assert!(
            tables.is_empty(),
            "Lines-only strategy with no ruling lines should return no tables"
        );
    }

    #[test]
    fn test_table_splitting_on_empty_row() {
        let spans = vec![
            create_test_span("T1-11", 20.0, 115.0, 10.0, 10.0),
            create_test_span("T1-12", 40.0, 115.0, 10.0, 10.0),
            create_test_span("T1-21", 20.0, 95.0, 10.0, 10.0),
            create_test_span("T1-22", 40.0, 95.0, 10.0, 10.0),
            create_test_span("T2-11", 20.0, 35.0, 10.0, 10.0),
            create_test_span("T2-12", 40.0, 35.0, 10.0, 10.0),
            create_test_span("T2-21", 20.0, 15.0, 10.0, 10.0),
            create_test_span("T2-22", 40.0, 15.0, 10.0, 10.0),
        ];
        let lines = vec![
            make_h_line(10.0, 130.0, 50.0),
            make_h_line(10.0, 110.0, 50.0),
            make_h_line(10.0, 90.0, 50.0),
            make_v_line(10.0, 90.0, 40.0),
            make_v_line(30.0, 90.0, 40.0),
            make_v_line(60.0, 90.0, 40.0),
            make_h_line(10.0, 50.0, 50.0),
            make_h_line(10.0, 30.0, 50.0),
            make_h_line(10.0, 10.0, 50.0),
            make_v_line(10.0, 10.0, 40.0),
            make_v_line(30.0, 10.0, 40.0),
            make_v_line(60.0, 10.0, 40.0),
            make_v_line(10.0, 50.0, 40.0),
        ];
        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Both,
            vertical_strategy: TableStrategy::Both,
            ..TableDetectionConfig::default()
        };
        assert_eq!(detect_tables_with_lines(&spans, &lines, &config).len(), 2);
    }

    #[test]
    fn test_detect_columns_invoice_4_columns() {
        // Simulate invoice: Date | Description | Charges | Credits
        let spans = vec![
            create_test_span("01/01", 50.0, 100.0, 50.0, 10.0),
            create_test_span("Widget", 130.0, 100.0, 220.0, 10.0),
            create_test_span("$100", 500.0, 100.0, 50.0, 10.0),
            create_test_span("$0", 600.0, 100.0, 50.0, 10.0),
            create_test_span("02/15", 50.0, 80.0, 50.0, 10.0),
            create_test_span("Service fee", 130.0, 80.0, 220.0, 10.0),
            create_test_span("$250", 500.0, 80.0, 50.0, 10.0),
            create_test_span("$50", 600.0, 80.0, 50.0, 10.0),
            create_test_span("03/20", 50.0, 60.0, 50.0, 10.0),
            create_test_span("Consulting", 130.0, 60.0, 220.0, 10.0),
            create_test_span("$500", 500.0, 60.0, 50.0, 10.0),
            create_test_span("$100", 600.0, 60.0, 50.0, 10.0),
        ];
        let config = TableDetectionConfig::default();
        let columns = detect_columns(
            &spans,
            config.column_tolerance,
            config.column_merge_threshold,
            Some(&config),
        );
        assert_eq!(
            columns.len(),
            4,
            "Invoice with 4 distinct column groups should produce exactly 4 columns, got {}",
            columns.len()
        );
    }

    #[test]
    fn test_detect_columns_merges_nearby_clusters() {
        // Spans at x=130, x=135, x=140 within same logical column
        let spans = vec![
            create_test_span("A", 50.0, 100.0, 30.0, 10.0),
            create_test_span("B", 130.0, 100.0, 30.0, 10.0),
            create_test_span("C", 50.0, 80.0, 30.0, 10.0),
            create_test_span("D", 135.0, 80.0, 30.0, 10.0),
            create_test_span("E", 50.0, 60.0, 30.0, 10.0),
            create_test_span("F", 140.0, 60.0, 30.0, 10.0),
        ];
        let config = TableDetectionConfig::default();
        let columns = detect_columns(
            &spans,
            config.column_tolerance,
            config.column_merge_threshold,
            Some(&config),
        );
        assert_eq!(
            columns.len(),
            2,
            "Spans at x=130/135/140 should merge into 1 column, plus x=50 = 2 total, got {}",
            columns.len()
        );
    }

    /// The pitch-scaled cap must not fire on a layout that has no pitch.
    ///
    /// Six lines of `Word 1`-style prose: a label at a fixed left edge and a
    /// short number just past it. The label widths differ, so the column
    /// centres are spread irregularly and there is no pitch to scale to. The
    /// cap applied anyway forbade merging the number into its label's column,
    /// splitting `Line 1` into `Line` | `1`; the row then no longer matched the
    /// flow span it came from and was emitted a second time as prose.
    ///
    /// Geometry is taken from a real page: labels at x=90 of widths 19, 43 and
    /// 52, each followed by a ~6pt number about 3pt later.
    #[test]
    fn test_detect_columns_no_pitch_keeps_the_fixed_threshold() {
        let mut spans = Vec::new();
        for (row, (label, w)) in [
            ("Line", 19.0f32),
            ("Line", 19.0),
            ("Line", 19.0),
            ("Underline", 43.0),
            ("BoldLine", 43.0),
            ("ItalicLine", 52.0),
        ]
        .iter()
        .enumerate()
        {
            let y = 100.0 - row as f32 * 12.0;
            spans.push(create_test_span(label, 90.0, y, *w, 10.0));
            spans.push(create_test_span("1", 90.0 + w + 3.0, y, 6.0, 10.0));
        }
        let config = TableDetectionConfig::default();
        let columns = detect_columns(
            &spans,
            config.column_tolerance,
            config.column_merge_threshold,
            Some(&config),
        );
        assert!(
            columns.len() <= 2,
            "an irregular label/number layout has no pitch to scale to, so the \
             fixed merge threshold must still join each number to its label; \
             got {} columns",
            columns.len()
        );
    }

    #[test]
    fn test_detect_columns_dense_pitch_stays_distinct() {
        // A dense numeric table: 6 columns on a 20pt pitch, each column's
        // spans sharing an exact left-x (so the *first* clustering pass,
        // gated on column_tolerance=15, keeps every column separate — this
        // isolates the merge-threshold pass specifically). Under the
        // default config's *fixed* 25pt column_merge_threshold, every
        // adjacent 20pt gap is below the threshold, so the old code fused
        // all 6 into 1. The threshold must scale down with this table's own
        // (much smaller than 25pt) pitch so the columns survive distinct.
        let mut spans = Vec::new();
        for row in 0..3 {
            let y = 100.0 - row as f32 * 20.0;
            for col in 0..6 {
                let x = 20.0 + col as f32 * 20.0;
                spans.push(create_test_span("9", x, y, 8.0, 10.0));
            }
        }
        let config = TableDetectionConfig::default();
        let columns = detect_columns(
            &spans,
            config.column_tolerance,
            config.column_merge_threshold,
            Some(&config),
        );
        assert_eq!(
            columns.len(),
            6,
            "a 20pt-pitch dense table must keep all 6 columns distinct \
             despite the 25pt fixed merge_threshold, got {}",
            columns.len()
        );
    }

    #[test]
    fn test_detect_columns_order_independent() {
        let spans_ordered = vec![
            create_test_span("A", 50.0, 100.0, 30.0, 10.0),
            create_test_span("B", 200.0, 100.0, 30.0, 10.0),
            create_test_span("C", 400.0, 100.0, 30.0, 10.0),
            create_test_span("D", 50.0, 80.0, 30.0, 10.0),
            create_test_span("E", 200.0, 80.0, 30.0, 10.0),
            create_test_span("F", 400.0, 80.0, 30.0, 10.0),
        ];
        // Same spans but in reverse order
        let spans_reversed = vec![
            create_test_span("F", 400.0, 80.0, 30.0, 10.0),
            create_test_span("E", 200.0, 80.0, 30.0, 10.0),
            create_test_span("D", 50.0, 80.0, 30.0, 10.0),
            create_test_span("C", 400.0, 100.0, 30.0, 10.0),
            create_test_span("B", 200.0, 100.0, 30.0, 10.0),
            create_test_span("A", 50.0, 100.0, 30.0, 10.0),
        ];
        let config = TableDetectionConfig::default();
        let cols_ordered = detect_columns(
            &spans_ordered,
            config.column_tolerance,
            config.column_merge_threshold,
            Some(&config),
        );
        let cols_reversed = detect_columns(
            &spans_reversed,
            config.column_tolerance,
            config.column_merge_threshold,
            Some(&config),
        );
        assert_eq!(
            cols_ordered.len(),
            cols_reversed.len(),
            "Column count should be independent of span order"
        );
        // Centers should be in the same sorted order
        let centers_ordered: Vec<f32> = cols_ordered
            .iter()
            .map(|c| (c.x_center * 10.0).round())
            .collect();
        let centers_reversed: Vec<f32> = cols_reversed
            .iter()
            .map(|c| (c.x_center * 10.0).round())
            .collect();
        assert_eq!(
            centers_ordered, centers_reversed,
            "Column centers should match regardless of input order"
        );
    }

    #[test]
    fn test_detect_header_row_returns_none_when_no_heuristic_matches() {
        // All spans have same font size, none bold -- no header signal
        let spans = vec![
            create_test_span("A", 10.0, 100.0, 30.0, 10.0),
            create_test_span("B", 50.0, 100.0, 30.0, 10.0),
            create_test_span("C", 10.0, 80.0, 30.0, 10.0),
            create_test_span("D", 50.0, 80.0, 30.0, 10.0),
        ];
        let columns = detect_columns(&spans, 15.0, 25.0, Some(&TableDetectionConfig::default()));
        let rows = detect_rows(&spans, 2.8);
        let grid = assign_spans_to_cells(&spans, &columns, &rows);
        let header = detect_header_row(&grid, &spans);
        assert_eq!(header, None, "Should return None when no heuristic matches");
    }

    #[test]
    fn test_hierarchical_header_with_visual_heuristic() {
        let spans = vec![
            create_test_span("H1", 10.0, 115.0, 35.0, 10.0),
            create_test_span("H2", 55.0, 115.0, 35.0, 10.0),
            create_test_span("Col 1", 10.0, 95.0, 35.0, 10.0),
            create_test_span("Col 2", 55.0, 95.0, 35.0, 10.0),
            create_test_span("Data 1", 10.0, 75.0, 35.0, 10.0),
            create_test_span("Data 2", 55.0, 75.0, 35.0, 10.0),
        ];
        let lines = vec![
            make_line_path(10.0, 130.0, 90.0, 130.0),
            make_line_path(10.0, 110.0, 90.0, 110.0),
            make_line_path(10.0, 90.0, 90.0, 90.0),
            make_v_line(10.0, 70.0, 60.0),
            make_v_line(50.0, 70.0, 20.0),
            make_v_line(90.0, 70.0, 60.0),
        ];
        let config = TableDetectionConfig::default();
        let tables = detect_tables_with_lines(&spans, &lines, &config);
        assert_eq!(tables.len(), 1);
        assert!(tables[0].rows[0].is_header);
        assert!(tables[0].rows[1].is_header);
    }

    // ---------------------------------------------------------------
    // Intersection-based detection tests
    // ---------------------------------------------------------------

    #[test]
    fn test_intersection_basic_2x2_table() {
        // 3 H lines at y=100, y=200, y=300 spanning x=50..400
        // 3 V lines at x=50, x=200, x=400 spanning y=100..300
        // This creates a 2-row x 2-col grid.
        let lines = vec![
            make_h_line(50.0, 100.0, 350.0),  // y=100
            make_h_line(50.0, 200.0, 350.0),  // y=200
            make_h_line(50.0, 300.0, 350.0),  // y=300
            make_v_line(50.0, 100.0, 200.0),  // x=50
            make_v_line(200.0, 100.0, 200.0), // x=200
            make_v_line(400.0, 100.0, 200.0), // x=400
        ];
        // Place text spans in each cell (center of each cell).
        // Cell (row0, col0): x in [50,200], y in [100,200] -> center (125, 150)
        // Cell (row0, col1): x in [200,400], y in [100,200] -> center (300, 150)
        // Cell (row1, col0): x in [50,200], y in [200,300] -> center (125, 250)
        // Cell (row1, col1): x in [200,400], y in [200,300] -> center (300, 250)
        let spans = vec![
            create_test_span("A1", 120.0, 145.0, 20.0, 10.0),
            create_test_span("B1", 295.0, 145.0, 20.0, 10.0),
            create_test_span("A2", 120.0, 245.0, 20.0, 10.0),
            create_test_span("B2", 295.0, 245.0, 20.0, 10.0),
        ];
        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            min_table_cells: 4,
            min_table_columns: 2,
            ..TableDetectionConfig::default()
        };
        let tables = detect_tables_with_lines(&spans, &lines, &config);
        assert_eq!(tables.len(), 1, "Should detect exactly 1 table");
        let table = &tables[0];
        assert_eq!(table.rows.len(), 2, "Should have 2 rows");
        assert_eq!(table.col_count, 2, "Should have 2 columns");

        // Higher y = higher on page, so rows sorted descending by y.
        // Row at y=[200,300] (higher) comes first in display order.
        // Row at y=[100,200] (lower) comes second.
        let r0_texts: Vec<&str> = table.rows[0]
            .cells
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        let r1_texts: Vec<&str> = table.rows[1]
            .cells
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(r0_texts, vec!["A2", "B2"], "Top row (higher y) should be A2, B2");
        assert_eq!(r1_texts, vec!["A1", "B1"], "Bottom row (lower y) should be A1, B1");
    }

    #[test]
    fn test_intersection_snap_and_merge_edges() {
        // Two H edges at y=100 and y=101.5 (within SNAP_TOL=3) should snap.
        let mut edges = vec![
            Edge {
                coord: 100.0,
                start: 0.0,
                end: 50.0,
            },
            Edge {
                coord: 101.5,
                start: 0.0,
                end: 50.0,
            },
        ];
        snap_and_merge(&mut edges);
        assert_eq!(edges.len(), 1, "Snapped edges should merge into 1");
        assert!((edges[0].coord - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_intersection_join_collinear_segments() {
        // Two segments on same coord, gap of 2pt (within JOIN_TOL=3).
        let mut edges = vec![
            Edge {
                coord: 100.0,
                start: 0.0,
                end: 50.0,
            },
            Edge {
                coord: 100.0,
                start: 52.0,
                end: 100.0,
            },
        ];
        snap_and_merge(&mut edges);
        assert_eq!(edges.len(), 1, "Collinear segments within 3pt should join");
        assert!((edges[0].start - 0.0).abs() < 0.01);
        assert!((edges[0].end - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_intersection_discard_short_edges() {
        let mut edges = vec![
            Edge {
                coord: 100.0,
                start: 0.0,
                end: 4.0,
            }, // 4pt < MIN_EDGE_LEN
            Edge {
                coord: 200.0,
                start: 0.0,
                end: 50.0,
            },
        ];
        snap_and_merge(&mut edges);
        assert_eq!(edges.len(), 1, "Short edge should be discarded");
        assert!((edges[0].coord - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_intersection_find_intersections_basic() {
        let h = vec![
            Edge {
                coord: 100.0,
                start: 0.0,
                end: 200.0,
            },
            Edge {
                coord: 200.0,
                start: 0.0,
                end: 200.0,
            },
        ];
        let v = vec![
            Edge {
                coord: 50.0,
                start: 50.0,
                end: 250.0,
            },
            Edge {
                coord: 150.0,
                start: 50.0,
                end: 250.0,
            },
        ];
        let pts = find_intersections(&h, &v);
        assert_eq!(pts.len(), 4, "2 H x 2 V = 4 intersections");
    }

    #[test]
    fn test_intersection_no_crossing_means_no_intersection() {
        // H line at y=100 spanning x=[0,50], V line at x=100 (outside H range).
        let h = vec![Edge {
            coord: 100.0,
            start: 0.0,
            end: 50.0,
        }];
        let v = vec![Edge {
            coord: 100.0,
            start: 0.0,
            end: 200.0,
        }];
        let pts = find_intersections(&h, &v);
        assert!(pts.is_empty(), "Non-crossing edges should produce no intersection");
    }

    #[test]
    fn test_intersection_build_cells() {
        let pts = vec![
            Intersection { x: 0.0, y: 0.0 },
            Intersection { x: 100.0, y: 0.0 },
            Intersection { x: 0.0, y: 100.0 },
            Intersection { x: 100.0, y: 100.0 },
        ];
        let cells = build_cells_from_intersections(&pts);
        assert_eq!(cells.len(), 1, "4 corners should produce 1 cell");
    }

    #[test]
    fn test_intersection_group_adjacent_cells() {
        // Two horizontally adjacent cells.
        let cells = vec![
            IntersectionCell {
                x1: 0.0,
                y1: 0.0,
                x2: 100.0,
                y2: 100.0,
            },
            IntersectionCell {
                x1: 100.0,
                y1: 0.0,
                x2: 200.0,
                y2: 100.0,
            },
        ];
        let groups = group_cells_into_tables(&cells);
        assert_eq!(groups.len(), 1, "Adjacent cells should be in 1 group");
    }

    #[test]
    fn test_intersection_separate_tables() {
        // Two cells far apart - not sharing any edge.
        let cells = vec![
            IntersectionCell {
                x1: 0.0,
                y1: 0.0,
                x2: 100.0,
                y2: 100.0,
            },
            IntersectionCell {
                x1: 500.0,
                y1: 500.0,
                x2: 600.0,
                y2: 600.0,
            },
        ];
        let groups = group_cells_into_tables(&cells);
        assert_eq!(groups.len(), 2, "Distant cells should be in separate groups");
    }

    #[test]
    fn test_intersection_rect_decomposition() {
        // A rectangle should decompose into 4 edges.
        let lines = vec![crate::elements::PathContent::rect(10.0, 10.0, 100.0, 50.0)];
        let (h, v) = extract_edges(&lines);
        assert_eq!(h.len(), 2, "Rectangle should produce 2 horizontal edges");
        assert_eq!(v.len(), 2, "Rectangle should produce 2 vertical edges");
    }

    #[test]
    fn test_intersection_3x3_grid_produces_4_cells() {
        // 3x3 intersection grid = 2x2 = 4 cells.
        let pts = vec![
            Intersection { x: 0.0, y: 0.0 },
            Intersection { x: 50.0, y: 0.0 },
            Intersection { x: 100.0, y: 0.0 },
            Intersection { x: 0.0, y: 50.0 },
            Intersection { x: 50.0, y: 50.0 },
            Intersection { x: 100.0, y: 50.0 },
            Intersection { x: 0.0, y: 100.0 },
            Intersection { x: 50.0, y: 100.0 },
            Intersection { x: 100.0, y: 100.0 },
        ];
        let cells = build_cells_from_intersections(&pts);
        assert_eq!(cells.len(), 4, "3x3 grid should produce 4 cells");
        let groups = group_cells_into_tables(&cells);
        assert_eq!(groups.len(), 1, "All 4 cells should form 1 table");
    }

    #[test]
    fn test_dotted_line_reconstitution() {
        // 10 short H segments at y=300, each 3pt wide, spanning x=50..350
        // Each segment is below MIN_EDGE_LEN (5pt) so would normally be discarded.
        // The reconstitution pass should merge them into one edge from x=50 to x=350.
        let mut edges: Vec<Edge> = (0..10)
            .map(|i| Edge {
                coord: 300.0,
                start: 50.0 + i as f32 * 30.0,
                end: 53.0 + i as f32 * 30.0, // 3pt each
            })
            .collect();

        snap_and_merge(&mut edges);

        assert_eq!(edges.len(), 1, "Dotted segments should reconstitute into 1 edge");
        assert!((edges[0].coord - 300.0).abs() < 0.01, "Reconstituted edge should be at y=300");
        assert!((edges[0].start - 50.0).abs() < 0.01, "Reconstituted edge should start at x=50");
        // Last segment ends at 53 + 9*30 = 323
        assert!((edges[0].end - 323.0).abs() < 0.01, "Reconstituted edge should end at x=323");
    }

    #[test]
    fn test_dotted_line_too_few_segments_discarded() {
        // Only 2 short segments — below DOTTED_MIN_SEGMENTS threshold.
        // Should be discarded entirely (not reconstituted, not kept individually).
        let mut edges = vec![
            Edge {
                coord: 200.0,
                start: 10.0,
                end: 13.0,
            },
            Edge {
                coord: 200.0,
                start: 20.0,
                end: 23.0,
            },
        ];
        snap_and_merge(&mut edges);
        assert!(edges.is_empty(), "Two short segments should not be reconstituted or kept");
    }

    #[test]
    fn test_dotted_line_narrow_span_discarded() {
        // 5 short segments at same coord but total span < DOTTED_MIN_SPAN (50pt).
        // Gaps between segments are > JOIN_TOL (3pt) so they won't be joined.
        let mut edges: Vec<Edge> = (0..5)
            .map(|i| Edge {
                coord: 400.0,
                start: 10.0 + i as f32 * 8.0,
                end: 13.0 + i as f32 * 8.0,
            })
            .collect();
        // span = (13+4*8) - 10 = 45 - 10 = 35pt < 50pt
        snap_and_merge(&mut edges);
        assert!(edges.is_empty(), "Short segments with narrow total span should be discarded");
    }

    #[test]
    fn test_dotted_line_mixed_with_long_edges() {
        // One long edge + several dotted segments at a different coord.
        // Both should survive.
        let mut edges = vec![
            Edge {
                coord: 100.0,
                start: 0.0,
                end: 200.0,
            }, // long, survives normally
        ];
        // Add 10 short segments at y=300
        for i in 0..10 {
            edges.push(Edge {
                coord: 300.0,
                start: 50.0 + i as f32 * 30.0,
                end: 53.0 + i as f32 * 30.0,
            });
        }
        snap_and_merge(&mut edges);
        assert_eq!(edges.len(), 2, "Long edge + reconstituted dotted line = 2 edges");
    }

    #[test]
    fn test_join_chain_of_short_segments() {
        // 10 H segments at y=100, each 25pt wide, touching end-to-end
        // x: 0-25, 25-50, 50-75, ..., 225-250
        // Should join into 1 segment x=0..250
        let mut edges: Vec<Edge> = (0..10)
            .map(|i| Edge {
                coord: 100.0,
                start: i as f32 * 25.0,
                end: (i + 1) as f32 * 25.0,
            })
            .collect();

        snap_and_merge(&mut edges);

        assert_eq!(edges.len(), 1, "Chain of 10 touching H segments should join into 1");
        assert!((edges[0].start - 0.0).abs() < 0.01, "Joined edge should start at 0");
        assert!((edges[0].end - 250.0).abs() < 0.01, "Joined edge should end at 250");
    }

    #[test]
    fn test_join_tiny_vertical_segments() {
        // 10 V segments at x=50, each 6pt tall, touching
        // y: 0-6, 6-12, 12-18, ..., 54-60
        // Should join into 1 segment y=0..60
        let mut edges: Vec<Edge> = (0..10)
            .map(|i| Edge {
                coord: 50.0,
                start: i as f32 * 6.0,
                end: (i + 1) as f32 * 6.0,
            })
            .collect();

        snap_and_merge(&mut edges);

        assert_eq!(edges.len(), 1, "Chain of 10 touching V segments should join into 1");
        assert!((edges[0].start - 0.0).abs() < 0.01, "Joined edge should start at 0");
        assert!((edges[0].end - 60.0).abs() < 0.01, "Joined edge should end at 60");
    }

    #[test]
    fn test_join_segments_with_slightly_different_coords() {
        // Segments at very close but not identical coords (within SNAP_TOL)
        // should snap to the same coord and then join.
        let mut edges = vec![
            Edge {
                coord: 87.4,
                start: 36.0,
                end: 117.0,
            },
            Edge {
                coord: 87.41,
                start: 117.0,
                end: 143.0,
            },
            Edge {
                coord: 87.39,
                start: 143.0,
                end: 170.0,
            },
        ];

        snap_and_merge(&mut edges);

        assert_eq!(edges.len(), 1, "Segments at near-identical coords should snap and join");
        assert!((edges[0].start - 36.0).abs() < 0.01, "Joined edge should start at 36");
        assert!((edges[0].end - 170.0).abs() < 0.01, "Joined edge should end at 170");
    }

    #[test]
    fn rule_families_tied_on_first_coord_emit_in_input_order() {
        // Eight side-by-side families (disjoint x-ranges, so none unify),
        // every family's first rule at the same y: `a[0].coord` ties across
        // all of them, so only the root tiebreak orders the output. Under
        // the pre-fix HashMap grouping this order was per-process hash
        // order (8! arrangements), so this assertion fails on all but the
        // seed that happens to sort.
        let edges: Vec<Edge> = (0..8)
            .flat_map(|k| {
                let x0 = 200.0 * k as f32;
                [
                    Edge {
                        coord: 700.0,
                        start: x0,
                        end: x0 + 150.0,
                    },
                    Edge {
                        coord: 600.0,
                        start: x0,
                        end: x0 + 150.0,
                    },
                ]
            })
            .collect();
        let wide: Vec<&Edge> = edges.iter().collect();

        let families = x_coherent_rule_families(&wide);

        assert_eq!(families.len(), 8, "Disjoint x-ranges should stay separate families");
        for (k, family) in families.iter().enumerate() {
            assert_eq!(family.len(), 2);
            assert!((family[0].coord - 700.0).abs() < 0.01, "First rule should be the top rule");
            assert!(
                (family[0].start - 200.0 * k as f32).abs() < 0.01,
                "Tied families should emit in input order, family {k} out of place"
            );
        }
    }

    #[test]
    fn rule_families_with_distinct_first_coords_order_by_coord() {
        // Distinct `a[0].coord` values: the coord key alone decides, so the
        // input (and root) order must NOT leak through — the family given
        // second sorts first because its top rule has the smaller y.
        let edges = [
            Edge {
                coord: 500.0,
                start: 0.0,
                end: 150.0,
            },
            Edge {
                coord: 400.0,
                start: 0.0,
                end: 150.0,
            },
            Edge {
                coord: 300.0,
                start: 300.0,
                end: 450.0,
            },
            Edge {
                coord: 200.0,
                start: 300.0,
                end: 450.0,
            },
        ];
        let wide: Vec<&Edge> = edges.iter().collect();

        let families = x_coherent_rule_families(&wide);

        assert_eq!(families.len(), 2);
        assert!((families[0][0].coord - 300.0).abs() < 0.01);
        assert!((families[1][0].coord - 500.0).abs() < 0.01);
    }

    #[test]
    fn test_hybrid_line_cols_text_rows() {
        // V lines at x=50, 200, 400 (2 columns) spanning y=100..300
        // H lines at y=100 and y=300 only (top and bottom, NO middle rows)
        // This creates a single intersection-based row, but text lives at 3 Y positions.
        let lines = vec![
            make_h_line(50.0, 100.0, 350.0),  // y=100 (bottom)
            make_h_line(50.0, 300.0, 350.0),  // y=300 (top)
            make_v_line(50.0, 100.0, 200.0),  // x=50
            make_v_line(200.0, 100.0, 200.0), // x=200
            make_v_line(400.0, 100.0, 200.0), // x=400
        ];
        // Text spans at three distinct Y positions within the single row:
        //   Row 1 (y~270): "A" in col0, "B" in col1
        //   Row 2 (y~210): "C" in col0, "D" in col1
        //   Row 3 (y~150): "E" in col0, "F" in col1
        let spans = vec![
            create_test_span("A", 60.0, 265.0, 20.0, 10.0), // col0, y=270
            create_test_span("B", 210.0, 265.0, 20.0, 10.0), // col1, y=270
            create_test_span("C", 60.0, 205.0, 20.0, 10.0), // col0, y=210
            create_test_span("D", 210.0, 205.0, 20.0, 10.0), // col1, y=210
            create_test_span("E", 60.0, 145.0, 20.0, 10.0), // col0, y=150
            create_test_span("F", 210.0, 145.0, 20.0, 10.0), // col1, y=150
        ];
        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            min_table_cells: 4,
            min_table_columns: 2,
            ..TableDetectionConfig::default()
        };
        let tables = detect_tables_with_lines(&spans, &lines, &config);
        assert_eq!(tables.len(), 1, "Should detect exactly 1 table");
        let table = &tables[0];
        assert_eq!(
            table.rows.len(),
            3,
            "Should have 3 rows (split from text Y positions), got {}",
            table.rows.len()
        );
        assert_eq!(table.col_count, 2, "Should have 2 columns");

        // Rows sorted top-to-bottom (descending Y in PDF coords).
        let r0: Vec<&str> = table.rows[0]
            .cells
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        let r1: Vec<&str> = table.rows[1]
            .cells
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        let r2: Vec<&str> = table.rows[2]
            .cells
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(r0, vec!["A", "B"], "Top row should be A, B");
        assert_eq!(r1, vec!["C", "D"], "Middle row should be C, D");
        assert_eq!(r2, vec!["E", "F"], "Bottom row should be E, F");
    }

    #[test]
    fn test_strip_form_numbering_artifacts() {
        use crate::structure::table_extractor::{TableCell, TableRow};

        let make_cell = |text: &str| TableCell {
            text: text.to_string(),
            spans: Vec::new(),
            colspan: 1,
            rowspan: 1,
            mcids: Vec::new(),
            bbox: None,
            is_header: false,
        };

        let mut rows = vec![
            // Row 0: all single-digit -> should be removed entirely
            TableRow {
                cells: vec![make_cell("5"), make_cell(""), make_cell(""), make_cell("")],
                is_header: false,
            },
            // Row 1: digit prefix artifacts -> should be stripped
            TableRow {
                cells: vec![
                    make_cell("1 Apr 11, 2025"),
                    make_cell("1 12111 - Rinse-Fluoride Treatment"),
                    make_cell("1 $14.60"),
                    make_cell("1"),
                ],
                is_header: false,
            },
            // Row 2: no artifacts -> unchanged
            TableRow {
                cells: vec![
                    make_cell("Apr 11, 2025"),
                    make_cell("11101 - One unit of time"),
                    make_cell("$47.60"),
                    make_cell(""),
                ],
                is_header: false,
            },
            // Row 3: digit prefix but remainder starts with digit -> NOT stripped
            TableRow {
                cells: vec![
                    make_cell("3 items"),
                    make_cell(""),
                    make_cell(""),
                    make_cell(""),
                ],
                is_header: false,
            },
        ];

        strip_form_numbering_artifacts(&mut rows);

        // Row 0 (all single-digit) should have been removed.
        assert_eq!(rows.len(), 3, "Single-digit-only row should be removed");

        // Former row 1 is now row 0: digit prefixes stripped.
        let r0: Vec<&str> = rows[0].cells.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(r0[0], "Apr 11, 2025", "Leading '1 ' should be stripped");
        assert_eq!(
            r0[1], "12111 - Rinse-Fluoride Treatment",
            "Leading '1 ' stripped, rest starts with digit but contains '-'"
        );
        assert_eq!(r0[2], "$14.60", "Leading '1 ' stripped, rest starts with '$'");
        // "1" alone is cleared in Phase 3 because other cells in this row were stripped.
        assert_eq!(r0[3], "", "Lone '1' cleared when other cells in row were stripped");

        // Former row 2 is now row 1: unchanged.
        let r1: Vec<&str> = rows[1].cells.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(r1[0], "Apr 11, 2025");
        assert_eq!(r1[1], "11101 - One unit of time");
        assert_eq!(r1[2], "$47.60");

        // Former row 3 is now row 2: "3 items" should NOT be stripped because
        // the remainder ("items") is a plain word with no date/code indicators.
        let r2: Vec<&str> = rows[2].cells.iter().map(|c| c.text.as_str()).collect();
        assert_eq!(
            r2[0], "3 items",
            "'3 items' should NOT be stripped (plain word, no date/code/currency)"
        );
    }

    #[test]
    fn test_strip_dash_separator_cells() {
        // T5 summary table: "------" appears as a decorative line separator
        // in a cell. After stripping, that cell should be empty.
        use crate::structure::table_extractor::{TableCell, TableRow};

        let make_cell = |text: &str| TableCell {
            text: text.to_string(),
            spans: Vec::new(),
            colspan: 1,
            rowspan: 1,
            mcids: Vec::new(),
            bbox: None,
            is_header: false,
        };

        let mut rows = vec![
            // Row with a dash-only cell (decorative separator)
            TableRow {
                cells: vec![
                    make_cell("------"),
                    make_cell("Total"),
                    make_cell("$500.00"),
                ],
                is_header: false,
            },
            // Row with underscore-only cell
            TableRow {
                cells: vec![
                    make_cell("____"),
                    make_cell("Subtotal"),
                    make_cell("$200.00"),
                ],
                is_header: false,
            },
            // Row with mixed dashes and underscores
            TableRow {
                cells: vec![make_cell("--__--"), make_cell("Tax"), make_cell("$10.00")],
                is_header: false,
            },
            // Row where ALL cells are dashes -> should be removed entirely
            TableRow {
                cells: vec![make_cell("------"), make_cell("---"), make_cell("------")],
                is_header: false,
            },
            // Row with real data that happens to contain a dash
            TableRow {
                cells: vec![
                    make_cell("2025-01-01"),
                    make_cell("Payment"),
                    make_cell("$100.00"),
                ],
                is_header: false,
            },
        ];

        strip_form_numbering_artifacts(&mut rows);

        // Row 0: "------" cell should be cleared
        assert_eq!(rows[0].cells[0].text.trim(), "", "Dash-only cell should be cleared");
        assert_eq!(rows[0].cells[1].text, "Total");
        assert_eq!(rows[0].cells[2].text, "$500.00");

        // Row 1: "____" cell should be cleared
        assert_eq!(rows[1].cells[0].text.trim(), "", "Underscore-only cell should be cleared");

        // Row 2: "--__--" cell should be cleared
        assert_eq!(
            rows[2].cells[0].text.trim(),
            "",
            "Mixed dash/underscore cell should be cleared"
        );

        // Row with all-dash cells becomes all-empty (kept for downstream
        // empty-row splitting, which uses empty rows as table separators).
        assert_eq!(rows.len(), 5, "All-dash row kept as empty separator");
        assert!(
            rows[3].cells.iter().all(|c| c.text.trim().is_empty()),
            "All-dash row should now be all-empty"
        );

        // Real data with dash in date should be preserved
        assert_eq!(rows[4].cells[0].text, "2025-01-01");
    }

    #[test]
    fn test_separate_small_and_large_table_clusters() {
        // Simulate an invoice layout with header + main table:
        // Header table: H-lines at y=83,142; V-lines at x=409,534 spanning y=71-143
        // Main table:   H-lines at y=150,553; V-lines at x=22,589 spanning y=150-553
        // These should form 2 separate clusters, not 1.
        let lines = vec![
            // Header table horizontal lines (thin rects: large width, tiny height)
            make_rect_path(409.0, 83.0, 125.0, 0.5), // H-line at y=83
            make_rect_path(409.0, 142.0, 125.0, 0.5), // H-line at y=142
            // Header table vertical lines (thin rects: tiny width, spans y=71..143)
            make_rect_path(409.0, 71.0, 0.5, 72.0), // V-line at x=409
            make_rect_path(534.0, 71.0, 0.5, 72.0), // V-line at x=534
            // Main table horizontal lines
            make_rect_path(22.0, 150.0, 567.0, 0.5), // H-line at y=150
            make_rect_path(22.0, 553.0, 567.0, 0.5), // H-line at y=553
            // Main table vertical lines (spans y=150..553)
            make_rect_path(22.0, 150.0, 0.5, 403.0), // V-line at x=22
            make_rect_path(490.0, 150.0, 0.5, 403.0), // V-line at x=490
            make_rect_path(589.0, 150.0, 0.5, 403.0), // V-line at x=589
        ];

        let config = TableDetectionConfig::default();
        let clusters = group_lines_into_clusters(&lines, &config);
        assert!(
            clusters.len() >= 2,
            "Expected at least 2 clusters (header table + main table), got {}",
            clusters.len()
        );

        // Verify that no single cluster contains both the header V-lines (y=71..143)
        // and the main table V-lines (y=150..553).
        for cluster in &clusters {
            let mut has_header_vline = false;
            let mut has_main_vline = false;
            for &idx in &cluster.lines {
                let bbox = &lines[idx].bbox;
                // V-line: width < 2
                if bbox.width.abs() < 2.0 && bbox.height.abs() > 5.0 {
                    let y_max = bbox.y + bbox.height;
                    if y_max < 145.0 {
                        has_header_vline = true;
                    }
                    if bbox.y >= 149.0 {
                        has_main_vline = true;
                    }
                }
            }
            assert!(
                !(has_header_vline && has_main_vline),
                "A single cluster should not contain both header V-lines (y<145) and main V-lines (y>149)"
            );
        }
    }

    // -----------------------------------------------------------------------
    // WS0.3b (C) header-row inclusion
    // -----------------------------------------------------------------------

    /// (C) A ruled 3-column grid whose header text row sits just ABOVE the top
    /// ruling (unruled header). Its three labels align to the three detected
    /// columns and span the table width, so the row must be pulled in as the
    /// table's header (cells preserved, not merged, marked as header).
    #[test]
    fn test_ws03b_header_row_above_included() {
        let lines = vec![
            make_h_line(100.0, 560.0, 300.0), // top ruling
            make_h_line(100.0, 530.0, 300.0),
            make_h_line(100.0, 500.0, 300.0),
            make_v_line(100.0, 500.0, 60.0),
            make_v_line(200.0, 500.0, 60.0),
            make_v_line(300.0, 500.0, 60.0),
            make_v_line(400.0, 500.0, 60.0),
        ];
        let spans = vec![
            // Header row ABOVE the top ruling (y ~568..578), aligned to columns.
            create_test_span("Name", 140.0, 568.0, 20.0, 10.0),
            create_test_span("Age", 240.0, 568.0, 20.0, 10.0),
            create_test_span("City", 340.0, 568.0, 20.0, 10.0),
            // Body rows.
            create_test_span("Ann", 145.0, 545.0, 10.0, 10.0),
            create_test_span("30", 245.0, 545.0, 10.0, 10.0),
            create_test_span("NYC", 345.0, 545.0, 10.0, 10.0),
            create_test_span("Bob", 145.0, 515.0, 10.0, 10.0),
            create_test_span("41", 245.0, 515.0, 10.0, 10.0),
            create_test_span("LA", 345.0, 515.0, 10.0, 10.0),
        ];
        let config = TableDetectionConfig::default();
        let clusters = group_lines_into_clusters(&lines, &config);
        assert_eq!(clusters.len(), 1, "single ruled grid → one cluster");
        let tables = detect_tables_in_cluster(&spans, &lines, &clusters[0], &config);
        assert_eq!(tables.len(), 1, "one table expected");
        let table = &tables[0];
        assert_eq!(
            table.rows.len(),
            3,
            "header row above the top ruling must be included (2 body + 1 header)"
        );
        assert!(table.has_header, "table must be flagged as having a header");
        assert!(table.rows[0].is_header, "row 0 must be the header row");
        // Header cells are preserved (not colspan-merged) and hold the labels.
        let header_text: String = table.rows[0]
            .cells
            .iter()
            .map(|c| c.text.trim())
            .collect::<Vec<_>>()
            .join("|");
        for label in ["Name", "Age", "City"] {
            assert!(
                header_text.contains(label),
                "header row must contain {label:?}, got {header_text:?}"
            );
        }
    }

    /// (C-negative) Unrelated text above the same grid at an x OUTSIDE the column
    /// extent does NOT align to any column, so no header row is added and the
    /// stray text is left out of the table (table unchanged: 2 body rows only).
    #[test]
    fn test_ws03b_unaligned_text_above_not_header() {
        let lines = vec![
            make_h_line(100.0, 560.0, 300.0),
            make_h_line(100.0, 530.0, 300.0),
            make_h_line(100.0, 500.0, 300.0),
            make_v_line(100.0, 500.0, 60.0),
            make_v_line(200.0, 500.0, 60.0),
            make_v_line(300.0, 500.0, 60.0),
            make_v_line(400.0, 500.0, 60.0),
        ];
        let spans = vec![
            // Caption far to the RIGHT of the column extent (100..400) → unaligned.
            create_test_span("Table", 520.0, 568.0, 20.0, 10.0),
            create_test_span("caption", 560.0, 568.0, 40.0, 10.0),
            // Body rows.
            create_test_span("Ann", 145.0, 545.0, 10.0, 10.0),
            create_test_span("30", 245.0, 545.0, 10.0, 10.0),
            create_test_span("NYC", 345.0, 545.0, 10.0, 10.0),
            create_test_span("Bob", 145.0, 515.0, 10.0, 10.0),
            create_test_span("41", 245.0, 515.0, 10.0, 10.0),
            create_test_span("LA", 345.0, 515.0, 10.0, 10.0),
        ];
        let config = TableDetectionConfig::default();

        // The gate itself must reject the unaligned text.
        let mut row_ys = vec![560.0_f32, 530.0, 500.0];
        row_ys.sort_by(|a, b| crate::utils::safe_float_cmp(*b, *a));
        let col_xs = vec![100.0_f32, 200.0, 300.0, 400.0];
        assert!(
            detect_header_row_above(&spans, &row_ys, &col_xs).is_none(),
            "text outside the column extent must not be treated as a header row"
        );

        // End-to-end: table is unchanged (only the 2 ruled body rows) and the
        // caption text is not present in any cell.
        let clusters = group_lines_into_clusters(&lines, &config);
        let tables = detect_tables_in_cluster(&spans, &lines, &clusters[0], &config);
        assert_eq!(tables.len(), 1);
        assert_eq!(
            tables[0].rows.len(),
            2,
            "no header row must be added for unaligned text above the grid"
        );
        let all_text: String = tables[0]
            .rows
            .iter()
            .flat_map(|r| r.cells.iter())
            .map(|c| c.text.clone())
            .collect();
        assert!(
            !all_text.contains("caption"),
            "unaligned caption text must stay out of the table, got {all_text:?}"
        );
    }

    #[test]
    fn test_text_edge_columns_form_layout() {
        // Simulate a form layout: text aligns to specific X positions across
        // many rows, but each row may only use a subset of columns.
        //
        // Column 1 (employer info):  left edge ~48
        // Column 2 (box codes):      left edge ~210
        // Column 3 (values):         left edge ~382
        // Column 4 (values):         left edge ~516
        //
        // We place 5+ spans at each column's X across different Y rows.

        let mut spans = Vec::new();
        let col_xs = [48.0_f32, 210.0, 382.0, 516.0];
        let row_ys = [700.0_f32, 680.0, 660.0, 640.0, 620.0, 600.0];

        for &cx in &col_xs {
            for &ry in &row_ys {
                spans.push(create_test_span("val", cx, ry, 40.0, 10.0));
            }
        }

        // Add some "noise" spans that only appear in 1-2 rows (should NOT
        // create extra columns).
        spans.push(create_test_span("noise", 130.0, 700.0, 20.0, 10.0));
        spans.push(create_test_span("noise", 132.0, 680.0, 20.0, 10.0));

        let config = TableDetectionConfig::default();
        let columns = detect_text_edge_columns(&spans, &config);

        // We expect roughly 4 column clusters (one per alignment edge),
        // possibly a few more for right-edges that also recur, but definitely
        // not 8+ as greedy clustering would produce.
        assert!(
            columns.len() >= 3 && columns.len() <= 6,
            "Expected 3-6 text-edge columns, got {}",
            columns.len()
        );

        // Verify the centres are close to the known left-edge positions.
        let centres: Vec<f32> = columns.iter().map(|c| c.x_center).collect();
        for &expected_x in &col_xs {
            assert!(
                centres
                    .iter()
                    .any(|&cx| (cx - expected_x).abs() < config.column_tolerance
                        || (cx - (expected_x + 40.0)).abs() < config.column_tolerance),
                "Expected a column near x={expected_x} (or its right edge), centres={centres:?}"
            );
        }
    }

    #[test]
    fn test_text_edge_columns_noise_filtered() {
        // Spans that only appear in 1 row should not produce columns.
        let spans = vec![
            // Only 1 span at x=100 — below the min_row_count=3 threshold
            create_test_span("a", 100.0, 500.0, 30.0, 10.0),
            // 4 spans at x=300 — should survive
            create_test_span("c", 300.0, 500.0, 30.0, 10.0),
            create_test_span("d", 300.0, 480.0, 30.0, 10.0),
            create_test_span("e", 300.0, 460.0, 30.0, 10.0),
            create_test_span("f", 300.0, 440.0, 30.0, 10.0),
        ];

        let config = TableDetectionConfig::default();
        let columns = detect_text_edge_columns(&spans, &config);

        // x=100 has only 1 row, so its left edge should be filtered out.
        // x=300 has 4 rows so its left-edge (and possibly right-edge at 330)
        // should survive.  At most 2 columns.
        assert!(!columns.is_empty(), "Should produce at least one column from x=300");
        // Make sure we don't get a column centred near 100
        for c in &columns {
            assert!(
                (c.x_center - 100.0).abs() > 15.0,
                "x=100 edge should have been filtered (only 1 row), but got column at {}",
                c.x_center
            );
        }
    }

    #[test]
    fn test_text_edge_fallback_integration() {
        // When greedy detect_columns produces >6 columns, detect_tables_from_spans
        // should fall back to text-edge detection and produce fewer columns.
        //
        // Build a layout with 4 true alignment columns but noisy X offsets
        // that cause greedy clustering (tolerance=15) to split them.
        let mut spans = Vec::new();
        let true_cols = [50.0_f32, 200.0, 350.0, 500.0];
        let row_ys = [700.0_f32, 680.0, 660.0, 640.0, 620.0];

        for (ci, &cx) in true_cols.iter().enumerate() {
            for (ri, &ry) in row_ys.iter().enumerate() {
                // Add slight jitter that stays within snap_tolerance but could
                // push greedy clustering into creating extra columns when
                // combined with different-width spans.
                let jitter = ((ci + ri) % 3) as f32 * 2.0;
                spans.push(create_test_span("v", cx + jitter, ry, 30.0, 10.0));
            }
        }

        // Also add extra scattered spans at unique X positions (each only in
        // 1 row) to bloat the greedy column count past 6.
        for i in 0..10 {
            let x = 80.0 + i as f32 * 30.0;
            spans.push(create_test_span("x", x, 700.0, 15.0, 10.0));
        }

        let config = TableDetectionConfig {
            column_tolerance: 8.0, // tight tolerance to force many greedy columns
            ..TableDetectionConfig::default()
        };

        let greedy_cols = detect_columns(
            &spans,
            config.column_tolerance,
            config.column_merge_threshold,
            Some(&config),
        );
        // With tight tolerance + scattered spans, greedy should exceed 6.
        assert!(
            greedy_cols.len() > 6,
            "Precondition: greedy should produce >6 columns, got {}",
            greedy_cols.len()
        );

        let te_cols = detect_text_edge_columns(&spans, &config);
        assert!(
            te_cols.len() < greedy_cols.len(),
            "Text-edge should produce fewer columns ({}) than greedy ({})",
            te_cols.len(),
            greedy_cols.len()
        );
    }

    #[test]
    fn test_reject_table_with_too_many_empty_cells() {
        // Build a table with 12 columns but ~80% empty cells — should be rejected.
        use crate::structure::table_extractor::{Table, TableCell, TableRow};
        let col_count = 12;
        let mut rows = Vec::new();
        // Header row: only 3 of 12 cells have text
        let mut header = TableRow::new(true);
        for c in 0..col_count {
            header.cells.push(TableCell {
                text: if c < 3 {
                    format!("H{c}")
                } else {
                    String::new()
                },
                spans: Vec::new(),
                colspan: 1,
                rowspan: 1,
                mcids: vec![],
                bbox: None,
                is_header: true,
            });
        }
        rows.push(header);
        // 4 data rows: only 2 of 12 cells have text each
        for r in 0..4 {
            let mut row = TableRow::new(false);
            for c in 0..col_count {
                row.cells.push(TableCell {
                    text: if c < 2 {
                        format!("R{r}C{c}")
                    } else {
                        String::new()
                    },
                    spans: Vec::new(),
                    colspan: 1,
                    rowspan: 1,
                    mcids: vec![],
                    bbox: None,
                    is_header: false,
                });
            }
            rows.push(row);
        }
        let table = Table {
            rows,
            has_header: true,
            col_count,
            bbox: None,
        };
        // 5 rows * 12 cols = 60 total, 3 + 4*2 = 11 filled, 49 empty → 81.7% empty
        assert!(!is_valid_table(&table), "Table with >60% empty cells should be rejected");
    }

    #[test]
    fn test_valid_table_passes_validation() {
        use crate::structure::table_extractor::{Table, TableCell, TableRow};
        let col_count = 3;
        let mut rows = Vec::new();
        for r in 0..4 {
            let mut row = TableRow::new(r == 0);
            for c in 0..col_count {
                row.cells.push(TableCell {
                    text: format!("R{r}C{c}"),
                    spans: Vec::new(),
                    colspan: 1,
                    rowspan: 1,
                    mcids: vec![],
                    bbox: None,
                    is_header: r == 0,
                });
            }
            rows.push(row);
        }
        let table = Table {
            rows,
            has_header: true,
            col_count,
            bbox: None,
        };
        assert!(is_valid_table(&table), "Well-populated table should pass validation");
    }

    /// Product data sheets have label/value rows that look like 2-column
    /// tables to the spatial detector (key text on the left, value on
    /// the right, with faint cell backgrounds). When the right-hand
    /// value wraps, the detector emits a continuation row whose left
    /// cell is empty — the hallmark of this false positive. Such tables
    /// must be rejected so their rows remain in the flow text.
    #[test]
    fn test_narrow_shallow_table_rejected_as_false_positive() {
        use crate::structure::table_extractor::{Table, TableCell, TableRow};
        let col_count = 2;
        let rows_data: Vec<(&str, &str)> = vec![
            ("Temperature resistance", "adhered to aluminium, -56° C to +82° C"),
            (
                "Resistance to cleaning agents",
                "adhered to aluminium, 8 h in solution (0.5% household",
            ),
            // Wrapping continuation → empty left cell.
            ("", "cleaning agents) at room temperature and 65° C, no"),
        ];
        let mut rows = Vec::new();
        for (label, value) in &rows_data {
            let mut row = TableRow::new(false);
            row.cells.push(TableCell {
                text: label.to_string(),
                spans: Vec::new(),
                colspan: 1,
                rowspan: 1,
                mcids: vec![],
                bbox: None,
                is_header: false,
            });
            row.cells.push(TableCell {
                text: value.to_string(),
                spans: Vec::new(),
                colspan: 1,
                rowspan: 1,
                mcids: vec![],
                bbox: None,
                is_header: false,
            });
            rows.push(row);
        }
        let table = Table {
            rows,
            has_header: false,
            col_count,
            bbox: None,
        };
        assert!(
            !is_valid_table(&table),
            "Narrow 2-column 'table' with an empty continuation cell must \
             be rejected so its rows stay in the flow text"
        );
    }

    /// A 2-column data table with enough filled rows is a real table
    /// and must continue to pass validation. Pins the threshold so the
    /// narrow-table guard does not regress genuine two-column tables.
    #[test]
    fn test_narrow_deep_table_still_accepted() {
        use crate::structure::table_extractor::{Table, TableCell, TableRow};
        let col_count = 2;
        let mut rows = Vec::new();
        for i in 0..6 {
            let mut row = TableRow::new(i == 0);
            row.cells.push(TableCell {
                text: format!("Key {i}"),
                spans: Vec::new(),
                colspan: 1,
                rowspan: 1,
                mcids: vec![],
                bbox: None,
                is_header: i == 0,
            });
            row.cells.push(TableCell {
                text: format!("Value {i}"),
                spans: Vec::new(),
                colspan: 1,
                rowspan: 1,
                mcids: vec![],
                bbox: None,
                is_header: i == 0,
            });
            rows.push(row);
        }
        let table = Table {
            rows,
            has_header: true,
            col_count,
            bbox: None,
        };
        assert!(is_valid_table(&table), "A 2-col × 6-row data table should still be accepted");
    }

    /// A sparse 2-column table with a missing value on the right is a
    /// legitimate pattern (key/value lists, form layouts, "N/A" rows) and
    /// must NOT match the narrow-table false-positive signature, which
    /// targets empty-LEFT / filled-RIGHT continuation rows specifically.
    #[test]
    fn test_narrow_sparse_table_with_missing_right_value_accepted() {
        use crate::structure::table_extractor::{Table, TableCell, TableRow};
        let col_count = 2;
        let rows_data: Vec<(&str, &str)> = vec![
            ("Name", "ACME Corp"),
            ("Registration", "12345"),
            ("Fax", ""),
            ("Email", "info@example.com"),
        ];
        let mut rows = Vec::new();
        for (label, value) in &rows_data {
            let mut row = TableRow::new(false);
            row.cells.push(TableCell {
                text: label.to_string(),
                spans: Vec::new(),
                colspan: 1,
                rowspan: 1,
                mcids: vec![],
                bbox: None,
                is_header: false,
            });
            row.cells.push(TableCell {
                text: value.to_string(),
                spans: Vec::new(),
                colspan: 1,
                rowspan: 1,
                mcids: vec![],
                bbox: None,
                is_header: false,
            });
            rows.push(row);
        }
        let table = Table {
            rows,
            has_header: false,
            col_count,
            bbox: None,
        };
        assert!(
            is_valid_table(&table),
            "A 2-col table with a missing right-hand value but no empty-left \
             continuation row must still validate"
        );
    }

    #[test]
    fn test_text_only_tables_capped_at_max_columns() {
        // Build spans that form 8+ columns of text aligned across rows.
        // detect_tables_from_spans should reject when columns exceed max_table_columns.
        let mut spans = Vec::new();
        let col_xs = [50.0_f32, 100.0, 150.0, 200.0, 250.0, 300.0, 350.0, 400.0];
        let row_ys = [700.0_f32, 680.0, 660.0, 640.0, 620.0];

        for &cx in &col_xs {
            for &ry in &row_ys {
                spans.push(create_test_span("val", cx, ry, 30.0, 10.0));
            }
        }

        // Use tight tolerance so each X position becomes its own column,
        // and cap at 6 columns via config.
        let config = TableDetectionConfig {
            column_tolerance: 5.0,
            column_merge_threshold: 8.0,
            max_table_columns: 6,
            ..TableDetectionConfig::default()
        };

        let tables = detect_tables_from_spans(&spans, &config);
        assert!(
            tables.is_empty(),
            "Text-only table with 8 columns should be rejected (max_table_columns=6), got {} table(s)",
            tables.len()
        );
    }

    #[test]
    fn test_extended_grid_when_lines_dont_cross() {
        // H-lines at y=100 and y=50 spanning full width x=0..500
        // V-lines at y=300..350 at x=0, x=100, x=200
        // These lines don't physically cross but should produce
        // a 1-row x 2-col grid via extended intersections.
        let lines = vec![
            make_h_line(0.0, 100.0, 500.0),
            make_h_line(0.0, 50.0, 500.0),
            make_v_line(0.0, 300.0, 50.0),
            make_v_line(100.0, 300.0, 50.0),
            make_v_line(200.0, 300.0, 50.0),
        ];

        // Place text in the cells to satisfy min_table_cells.
        let spans = vec![
            create_test_span("A", 30.0, 70.0, 20.0, 10.0),
            create_test_span("B", 130.0, 70.0, 20.0, 10.0),
        ];

        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            min_table_cells: 2,
            min_table_columns: 2,
            ..TableDetectionConfig::default()
        };

        let tables = detect_tables_from_intersections(&spans, &lines, &config);
        assert!(
            !tables.is_empty(),
            "Extended grid should produce at least one table when H and V lines don't cross"
        );
        let table = &tables[0];
        assert!(
            table.col_count >= 2,
            "Extended grid table should have at least 2 columns, got {}",
            table.col_count
        );
    }

    #[test]
    fn test_merge_vertically_adjacent_tables() {
        // Two tables with 3 columns each, bboxes separated by 5pt (< ADJACENT_TABLE_MERGE_GAP).
        let table1 = Table {
            rows: vec![TableRow {
                cells: vec![
                    TableCell {
                        text: "A".into(),
                        spans: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        mcids: vec![],
                        bbox: None,
                        is_header: false,
                    },
                    TableCell {
                        text: "B".into(),
                        spans: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        mcids: vec![],
                        bbox: None,
                        is_header: false,
                    },
                    TableCell {
                        text: "C".into(),
                        spans: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        mcids: vec![],
                        bbox: None,
                        is_header: false,
                    },
                ],
                is_header: false,
            }],
            has_header: false,
            col_count: 3,
            bbox: Some(Rect::new(0.0, 100.0, 300.0, 50.0)),
        };
        let table2 = Table {
            rows: vec![TableRow {
                cells: vec![
                    TableCell {
                        text: "D".into(),
                        spans: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        mcids: vec![],
                        bbox: None,
                        is_header: false,
                    },
                    TableCell {
                        text: "E".into(),
                        spans: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        mcids: vec![],
                        bbox: None,
                        is_header: false,
                    },
                    TableCell {
                        text: "F".into(),
                        spans: Vec::new(),
                        colspan: 1,
                        rowspan: 1,
                        mcids: vec![],
                        bbox: None,
                        is_header: false,
                    },
                ],
                is_header: false,
            }],
            has_header: false,
            col_count: 3,
            // Top at 155, so gap = 155 - (100+50) = 5pt
            bbox: Some(Rect::new(0.0, 155.0, 300.0, 50.0)),
        };

        let mut tables = vec![table1, table2];
        merge_vertically_adjacent_tables(&mut tables);
        assert_eq!(tables.len(), 1, "Adjacent tables should be merged into one");
        assert_eq!(tables[0].rows.len(), 2, "Merged table should have 2 rows");
        assert_eq!(tables[0].col_count, 3);
    }

    #[test]
    fn test_no_merge_when_gap_too_large() {
        let table1 = Table {
            rows: vec![TableRow {
                cells: vec![TableCell {
                    text: "A".into(),
                    spans: Vec::new(),
                    colspan: 1,
                    rowspan: 1,
                    mcids: vec![],
                    bbox: None,
                    is_header: false,
                }],
                is_header: false,
            }],
            has_header: false,
            col_count: 1,
            bbox: Some(Rect::new(0.0, 100.0, 300.0, 50.0)),
        };
        let table2 = Table {
            rows: vec![TableRow {
                cells: vec![TableCell {
                    text: "B".into(),
                    spans: Vec::new(),
                    colspan: 1,
                    rowspan: 1,
                    mcids: vec![],
                    bbox: None,
                    is_header: false,
                }],
                is_header: false,
            }],
            has_header: false,
            col_count: 1,
            // Top at 200, gap = 200 - 150 = 50pt >> ADJACENT_TABLE_MERGE_GAP
            bbox: Some(Rect::new(0.0, 200.0, 300.0, 50.0)),
        };

        let mut tables = vec![table1, table2];
        merge_vertically_adjacent_tables(&mut tables);
        assert_eq!(tables.len(), 2, "Tables with large gap should NOT be merged");
    }

    // ---------------------------------------------------------------
    // Census / W-2 / 1099 table detection tests
    // ---------------------------------------------------------------

    #[test]
    fn test_census_h_and_v_in_different_regions() {
        // Census-style layout: H-lines at y=100, y=50 spanning full width (x=36..576)
        // V-lines at y=500..550 at positions x=36, 117, 197, 277, 357, 437, 517, 576
        // The H and V lines DON'T physically cross (different Y regions)
        // But they should produce a table via extended grid.
        let lines = vec![
            // H-lines in the lower Y region
            make_h_line(36.0, 100.0, 540.0), // y=100, x=36..576
            make_h_line(36.0, 50.0, 540.0),  // y=50, x=36..576
            // V-lines in a completely different (higher) Y region
            make_v_line(36.0, 500.0, 50.0),  // x=36, y=500..550
            make_v_line(117.0, 500.0, 50.0), // x=117
            make_v_line(197.0, 500.0, 50.0), // x=197
            make_v_line(277.0, 500.0, 50.0), // x=277
            make_v_line(357.0, 500.0, 50.0), // x=357
            make_v_line(437.0, 500.0, 50.0), // x=437
            make_v_line(517.0, 500.0, 50.0), // x=517
            make_v_line(576.0, 500.0, 50.0), // x=576
        ];

        // Place text spans in the cells (between the H-lines).
        let spans = vec![
            create_test_span("A", 60.0, 70.0, 20.0, 10.0),
            create_test_span("B", 140.0, 70.0, 20.0, 10.0),
            create_test_span("C", 220.0, 70.0, 20.0, 10.0),
            create_test_span("D", 300.0, 70.0, 20.0, 10.0),
            create_test_span("E", 380.0, 70.0, 20.0, 10.0),
            create_test_span("F", 460.0, 70.0, 20.0, 10.0),
            create_test_span("G", 540.0, 70.0, 20.0, 10.0),
        ];

        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            min_table_cells: 2,
            min_table_columns: 2,
            ..TableDetectionConfig::default()
        };

        let tables = detect_tables_with_lines(&spans, &lines, &config);
        assert!(
            !tables.is_empty(),
            "Census layout with H/V in different Y regions should produce at least 1 table"
        );
        let table = &tables[0];
        // Should have ~7 columns (8 V-lines = 7 column spans) and 1 row (2 H-lines = 1 row)
        assert!(
            table.col_count >= 5,
            "Census table should have at least 5 columns, got {}",
            table.col_count
        );
    }

    #[test]
    fn test_w2_grid_not_fragmented() {
        // W-2 style: V-lines at x=100,200,300 spanning y=100..700 (full form)
        // V-lines at x=350,450 spanning y=300..500 (sub-section only)
        // H-lines at y=100,200,300,400,500,600,700
        // Should produce 1 table (or a few that merge), not 5+ fragments.
        let lines = vec![
            // Full-height V-lines
            make_v_line(100.0, 100.0, 600.0), // x=100, y=100..700
            make_v_line(200.0, 100.0, 600.0), // x=200, y=100..700
            make_v_line(300.0, 100.0, 600.0), // x=300, y=100..700
            // Sub-section V-lines
            make_v_line(350.0, 300.0, 200.0), // x=350, y=300..500
            make_v_line(450.0, 300.0, 200.0), // x=450, y=300..500
            // H-lines across full width
            make_h_line(100.0, 100.0, 350.0),
            make_h_line(100.0, 200.0, 350.0),
            make_h_line(100.0, 300.0, 350.0),
            make_h_line(100.0, 400.0, 350.0),
            make_h_line(100.0, 500.0, 350.0),
            make_h_line(100.0, 600.0, 350.0),
            make_h_line(100.0, 700.0, 350.0),
        ];

        // Place text in cells
        let spans = vec![
            create_test_span("R1C1", 120.0, 150.0, 30.0, 10.0),
            create_test_span("R1C2", 220.0, 150.0, 30.0, 10.0),
            create_test_span("R2C1", 120.0, 250.0, 30.0, 10.0),
            create_test_span("R2C2", 220.0, 250.0, 30.0, 10.0),
            create_test_span("R3C1", 120.0, 350.0, 30.0, 10.0),
            create_test_span("R3C2", 220.0, 350.0, 30.0, 10.0),
            create_test_span("R4C1", 120.0, 450.0, 30.0, 10.0),
            create_test_span("R4C2", 220.0, 450.0, 30.0, 10.0),
            create_test_span("R5C1", 120.0, 550.0, 30.0, 10.0),
            create_test_span("R5C2", 220.0, 550.0, 30.0, 10.0),
            create_test_span("R6C1", 120.0, 650.0, 30.0, 10.0),
            create_test_span("R6C2", 220.0, 650.0, 30.0, 10.0),
        ];

        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            min_table_cells: 4,
            min_table_columns: 2,
            ..TableDetectionConfig::default()
        };

        let tables = detect_tables_with_lines(&spans, &lines, &config);
        assert!(
            tables.len() <= 2,
            "W-2 grid should produce at most 2 tables (not fragmented into {})",
            tables.len()
        );
        // The total row count across all tables should be reasonable (not duplicated).
        let total_filled: usize = tables
            .iter()
            .flat_map(|t| &t.rows)
            .flat_map(|r| &r.cells)
            .filter(|c| !c.text.is_empty())
            .count();
        assert!(
            total_filled >= 8,
            "W-2 tables should capture most text spans, got {}",
            total_filled
        );
    }

    #[test]
    fn test_invoice_still_separate_tables() {
        // Invoice layout with header + main table:
        // Header V-lines at x=410,535 spanning y=71..143
        // Main table V-lines at x=22,103,490,541,589 spanning y=150..553
        // H-lines shared near y=150 (header at y=83,142; main at y=150,553)
        // Should produce 2 separate tables (header + main), not 1 merged.
        let lines = vec![
            // Header table
            make_h_line(410.0, 83.0, 125.0),  // y=83
            make_h_line(410.0, 142.0, 125.0), // y=142
            make_v_line(410.0, 71.0, 72.0),   // x=410, y=71..143
            make_v_line(535.0, 71.0, 72.0),   // x=535, y=71..143
            // Main table
            make_h_line(22.0, 150.0, 567.0),  // y=150
            make_h_line(22.0, 553.0, 567.0),  // y=553
            make_v_line(22.0, 150.0, 403.0),  // x=22, y=150..553
            make_v_line(103.0, 150.0, 403.0), // x=103, y=150..553
            make_v_line(490.0, 150.0, 403.0), // x=490, y=150..553
            make_v_line(541.0, 150.0, 403.0), // x=541, y=150..553
            make_v_line(589.0, 150.0, 403.0), // x=589, y=150..553
        ];

        // Spans in header
        let mut spans = vec![
            create_test_span("Balance Due", 420.0, 100.0, 80.0, 10.0),
            create_test_span("$500.00", 420.0, 120.0, 60.0, 10.0),
        ];
        // Spans in main table
        for i in 0..6 {
            let y = 160.0 + i as f32 * 60.0;
            spans.push(create_test_span("Date", 30.0, y, 40.0, 10.0));
            spans.push(create_test_span("Code", 110.0, y, 40.0, 10.0));
            spans.push(create_test_span("Desc", 200.0, y, 200.0, 10.0));
            spans.push(create_test_span("$100", 500.0, y, 30.0, 10.0));
        }

        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            min_table_cells: 2,
            min_table_columns: 1,
            ..TableDetectionConfig::default()
        };

        let tables = detect_tables_with_lines(&spans, &lines, &config);
        assert!(
            tables.len() >= 2,
            "Invoice should produce at least 2 separate tables (header + main), got {}",
            tables.len()
        );
    }

    #[test]
    fn test_two_column_table_detection() {
        // Left column: paragraph text spans at x=50..280
        let mut spans = vec![
            create_test_span("Abstract", 50.0, 700.0, 60.0, 12.0),
            create_test_span("We present a novel approach to language", 50.0, 680.0, 230.0, 12.0),
            create_test_span("Results show improvements across all", 50.0, 660.0, 230.0, 12.0),
            create_test_span("benchmarks with significant gains on", 50.0, 640.0, 230.0, 12.0),
            create_test_span("standard evaluation metrics.", 50.0, 620.0, 180.0, 12.0),
        ];

        // Right column: a 3x3 table at x=320..550
        // Header row
        spans.push(create_test_span("Model", 320.0, 700.0, 40.0, 12.0));
        spans.push(create_test_span("F1", 420.0, 700.0, 15.0, 12.0));
        spans.push(create_test_span("Acc", 500.0, 700.0, 20.0, 12.0));
        // Data row 1
        spans.push(create_test_span("BERT", 320.0, 680.0, 30.0, 12.0));
        spans.push(create_test_span("92.4", 420.0, 680.0, 25.0, 12.0));
        spans.push(create_test_span("89.1", 500.0, 680.0, 25.0, 12.0));
        // Data row 2
        spans.push(create_test_span("GPT", 320.0, 660.0, 25.0, 12.0));
        spans.push(create_test_span("91.2", 420.0, 660.0, 25.0, 12.0));
        spans.push(create_test_span("88.3", 500.0, 660.0, 25.0, 12.0));

        let config = TableDetectionConfig::default();
        let tables = detect_tables_from_spans_column_aware(&spans, &config);

        // Should detect 1 table (the 3x3 in the right column)
        assert_eq!(
            tables.len(),
            1,
            "Should detect exactly 1 table in the right column, got {}",
            tables.len()
        );
        // The table should have 3 columns
        assert_eq!(
            tables[0].col_count, 3,
            "Table should have 3 columns, got {}",
            tables[0].col_count
        );
    }

    #[test]
    fn test_single_column_no_regression() {
        // Standard single-column page with no tables — just flowing paragraph text
        let spans = vec![
            create_test_span("Introduction", 50.0, 700.0, 80.0, 14.0),
            create_test_span(
                "This paper presents a comprehensive study of natural language",
                50.0,
                680.0,
                450.0,
                12.0,
            ),
            create_test_span(
                "processing techniques applied to large-scale document analysis.",
                50.0,
                660.0,
                430.0,
                12.0,
            ),
            create_test_span(
                "Our approach builds on recent advances in transformer architectures",
                50.0,
                640.0,
                460.0,
                12.0,
            ),
            create_test_span(
                "and demonstrates improvements across multiple benchmarks.",
                50.0,
                620.0,
                400.0,
                12.0,
            ),
            create_test_span(
                "We evaluate our method on standard datasets and report results.",
                50.0,
                600.0,
                420.0,
                12.0,
            ),
        ];

        let config = TableDetectionConfig::default();
        let tables = detect_tables_from_spans_column_aware(&spans, &config);
        assert!(
            tables.is_empty(),
            "Single-column paragraph text should not be detected as a table, got {} table(s)",
            tables.len()
        );
    }

    #[test]
    fn test_h_rule_bounded_text_table() {
        // Two H-lines spanning x=50..400 at y=750 and y=700 (no V-lines).
        let lines = vec![
            make_h_line(50.0, 750.0, 350.0), // top rule
            make_h_line(50.0, 700.0, 350.0), // bottom rule
        ];

        // Header row just below the top rule, then 3 data rows with aligned columns.
        let spans = vec![
            // Header row (y=740)
            create_test_span("Model", 60.0, 740.0, 50.0, 10.0),
            create_test_span("Acc", 180.0, 740.0, 30.0, 10.0),
            create_test_span("F1", 280.0, 740.0, 20.0, 10.0),
            // Data row 1 (y=728)
            create_test_span("BERT", 60.0, 728.0, 40.0, 10.0),
            create_test_span("84.6", 180.0, 728.0, 30.0, 10.0),
            create_test_span("83.4", 280.0, 728.0, 30.0, 10.0),
            // Data row 2 (y=716)
            create_test_span("GPT", 60.0, 716.0, 35.0, 10.0),
            create_test_span("82.1", 180.0, 716.0, 30.0, 10.0),
            create_test_span("81.0", 280.0, 716.0, 30.0, 10.0),
            // Data row 3 (y=704)
            create_test_span("XLNet", 60.0, 704.0, 45.0, 10.0),
            create_test_span("85.2", 180.0, 704.0, 30.0, 10.0),
            create_test_span("84.1", 280.0, 704.0, 30.0, 10.0),
        ];

        let config = TableDetectionConfig::default();
        let tables = detect_tables_with_lines(&spans, &lines, &config);
        assert!(
            !tables.is_empty(),
            "Should detect at least 1 table from text within H-line boundaries"
        );
        let table = &tables[0];
        assert!(table.col_count >= 3, "Expected at least 3 columns, got {}", table.col_count);
        assert!(table.rows.len() >= 3, "Expected at least 3 rows, got {}", table.rows.len());
    }

    #[test]
    fn test_split_table_at_section_dividers() {
        // Simulate a multi-section form with 3 sections separated by
        // full-width H-lines.  Each section has its OWN vertical grid lines
        // (they don't span across section boundaries).
        //
        // Section 1: y=10..40   (3 rows, H-lines at y=10,20,30,40)
        // Section 2: y=40..70   (3 rows, H-lines at y=40,50,60,70)
        // Section 3: y=70..100  (3 rows, H-lines at y=70,80,90,100)
        //
        // V-lines per section: x=10,40,70,100 but only within each section's
        // Y-range, so no V-edge crosses y=40 or y=70.

        let mut lines: Vec<crate::elements::PathContent> = Vec::new();

        // Full-width H-lines for every row boundary (y=10,20,...,100)
        for i in 0..=9 {
            let y = 10.0 + i as f32 * 10.0;
            lines.push(make_h_line(10.0, y, 90.0)); // x=10..100
        }

        // V-lines per section (NOT spanning across section dividers)
        // Section 1: y=10..40
        for &x in &[10.0, 40.0, 70.0, 100.0] {
            lines.push(make_v_line(x, 10.0, 30.0)); // y=10..40
        }
        // Section 2: y=40..70
        for &x in &[10.0, 40.0, 70.0, 100.0] {
            lines.push(make_v_line(x, 40.0, 30.0)); // y=40..70
        }
        // Section 3: y=70..100
        for &x in &[10.0, 40.0, 70.0, 100.0] {
            lines.push(make_v_line(x, 70.0, 30.0)); // y=70..100
        }

        // Place text spans in each cell (3 cols x 9 rows = 27 spans)
        let mut spans = Vec::new();
        for row in 0..9 {
            let y = 15.0 + row as f32 * 10.0;
            for col in 0..3 {
                let x = 15.0 + col as f32 * 30.0;
                let label = format!("S{}-R{}-C{}", row / 3 + 1, row % 3 + 1, col + 1);
                spans.push(create_test_span(&label, x, y, 20.0, 8.0));
            }
        }

        let config = TableDetectionConfig {
            horizontal_strategy: TableStrategy::Lines,
            vertical_strategy: TableStrategy::Lines,
            ..TableDetectionConfig::default()
        };

        let tables = detect_tables_with_lines(&spans, &lines, &config);

        // The full-width H-lines at y=40 and y=70 have no V-edges crossing
        // through them, so they should be detected as section dividers.
        // We expect 3 tables (one per section).
        assert!(
            tables.len() >= 3,
            "Expected at least 3 tables after section-divider splitting, got {}",
            tables.len()
        );
        // Each sub-table should have 3 columns.
        for (i, t) in tables.iter().enumerate() {
            assert_eq!(t.col_count, 3, "Table {} should have 3 columns, got {}", i, t.col_count);
        }
    }

    // -----------------------------------------------------------------
    // validate_table_structure_internal: split-column-group tests
    //
    // These exercise has_split_modal_column_groups, the structural
    // check that replaced the row-density gate. The detector rejects
    // grids whose modal rows partition into two or more disconnected
    // column-co-occurrence components. make_split_grid models the
    // false-positive shape (two prose flows mis-clustered into one
    // grid); make_grouped_grid models the sparse grouped-row-header
    // shape from the scientific-table regression class; the real
    // failure may also involve upstream column over-counting, while
    // this unit fixture pins the validator-level property that sparse
    // modal rows with connected populated columns are accepted.
    // -----------------------------------------------------------------

    /// Build a minimal GridStructure with `num_rows` rows and
    /// `num_cols` columns, where every row populates exactly
    /// `populated_per_row` cells (the first N columns). Numeric
    /// fields of `ColumnCluster` / `RowCluster` are arbitrary —
    /// `validate_table_structure_internal` reads only
    /// `grid.columns.len()` and the emptiness of each cell.
    fn make_uniform_grid(
        num_cols: usize,
        num_rows: usize,
        populated_per_row: usize,
    ) -> GridStructure {
        let columns = (0..num_cols)
            .map(|_| ColumnCluster {
                x_center: 0.0,
                x_min: 0.0,
                x_max: 0.0,
                span_indices: vec![],
            })
            .collect();
        let rows = (0..num_rows)
            .map(|_| RowCluster {
                y_center: 0.0,
                y_min: 0.0,
                y_max: 0.0,
                span_indices: vec![],
            })
            .collect();
        let cells = (0..num_rows)
            .map(|_| {
                (0..num_cols)
                    .map(|c| {
                        if c < populated_per_row {
                            vec![0usize]
                        } else {
                            vec![]
                        }
                    })
                    .collect()
            })
            .collect();
        GridStructure {
            columns,
            rows,
            cells,
        }
    }

    /// Build a GridStructure modelling two adjacent text flows
    /// mis-clustered into one candidate grid. The first `num_cols / 2`
    /// columns form the "left flow"; the remaining columns form the
    /// "right flow". Rows alternate between populating only the left
    /// flow and only the right flow. All rows have the same populated
    /// cardinality (`num_cols / 2`), so the regular-row-ratio gate
    /// passes at 1.00. `num_cols` must be even.
    fn make_split_grid(num_cols: usize, num_rows: usize) -> GridStructure {
        assert!(num_cols.is_multiple_of(2), "make_split_grid requires even num_cols");
        let half = num_cols / 2;
        let columns = (0..num_cols)
            .map(|_| ColumnCluster {
                x_center: 0.0,
                x_min: 0.0,
                x_max: 0.0,
                span_indices: vec![],
            })
            .collect();
        let rows = (0..num_rows)
            .map(|_| RowCluster {
                y_center: 0.0,
                y_min: 0.0,
                y_max: 0.0,
                span_indices: vec![],
            })
            .collect();
        let cells = (0..num_rows)
            .map(|r| {
                let left_row = r % 2 == 0;
                (0..num_cols)
                    .map(|c| {
                        let in_left_half = c < half;
                        if left_row == in_left_half {
                            vec![0usize]
                        } else {
                            vec![]
                        }
                    })
                    .collect()
            })
            .collect();
        GridStructure {
            columns,
            rows,
            cells,
        }
    }

    /// Build a GridStructure modelling a hierarchical scientific table.
    /// `total_cols` columns; the first `group_cols` are populated only
    /// in the first row of each group of `group_size` consecutive rows;
    /// the remaining columns are populated in every row. Models the
    /// failure shape from arxiv_2510.24670v2: grouped row-headers above
    /// dense data columns. The over-counting that the maintainer
    /// described occurs upstream of this fixture; here we model the
    /// post-clustering grid the validator actually sees. Numeric
    /// cluster fields are arbitrary, matching the convention used by
    /// `make_uniform_grid` and `make_split_grid`.
    fn make_grouped_grid(
        total_cols: usize,
        num_rows: usize,
        group_cols: usize,
        group_size: usize,
    ) -> GridStructure {
        assert!(group_cols < total_cols, "group_cols must be < total_cols");
        assert!(group_size > 0, "group_size must be positive");
        let columns = (0..total_cols)
            .map(|_| ColumnCluster {
                x_center: 0.0,
                x_min: 0.0,
                x_max: 0.0,
                span_indices: vec![],
            })
            .collect();
        let rows = (0..num_rows)
            .map(|_| RowCluster {
                y_center: 0.0,
                y_min: 0.0,
                y_max: 0.0,
                span_indices: vec![],
            })
            .collect();
        let cells = (0..num_rows)
            .map(|r| {
                let is_group_header = r % group_size == 0;
                (0..total_cols)
                    .map(|c| {
                        let populated = if c < group_cols {
                            is_group_header
                        } else {
                            true
                        };
                        if populated {
                            vec![0usize]
                        } else {
                            vec![]
                        }
                    })
                    .collect()
            })
            .collect();
        GridStructure {
            columns,
            rows,
            cells,
        }
    }

    /// 6 columns, 6 rows, modal rows alternate between {0,1,2} and
    /// {3,4,5}. Two disconnected components of 3 columns each, each
    /// with 3 modal rows of support. Default profile.
    #[test]
    fn validate_rejects_split_column_groups() {
        let grid = make_split_grid(6, 6);
        let config = TableDetectionConfig::default();
        assert!(!validate_table_structure_internal(&grid, &config));
    }

    /// Same fixture, strict profile. The strict profile's stronger
    /// regular_row_ratio (0.8) does not catch this; the split-column
    /// detector does.
    #[test]
    fn validate_rejects_split_column_groups_under_strict_profile() {
        let grid = make_split_grid(6, 6);
        let config = TableDetectionConfig::strict();
        assert!(!validate_table_structure_internal(&grid, &config));
    }

    /// 11 columns, 5 rows, every row populates every column. One
    /// component spanning all columns → accepted.
    #[test]
    fn validate_accepts_dense_table() {
        let grid = make_uniform_grid(11, 5, 11);
        let config = TableDetectionConfig::default();
        assert!(validate_table_structure_internal(&grid, &config));
    }

    /// 6 columns, 5 rows, every row populates the first 4 columns.
    /// The old density gate would have admitted this at the boundary
    /// (4/6 = 2/3). One connected component of 4 columns → accepted.
    #[test]
    fn validate_accepts_sparse_connected_table() {
        let grid = make_uniform_grid(6, 5, 4);
        let config = TableDetectionConfig::default();
        assert!(validate_table_structure_internal(&grid, &config));
    }

    /// 8 columns, 12 rows, grouped row-headers occupy the first 2
    /// columns and are populated only in the first row of each group
    /// of 4. Models arxiv_2510.24670v2's failure shape (post-
    /// clustering): 9 modal data rows populate columns 2..8, six data
    /// columns all connected, one component → accepted. The real
    /// failure may also involve upstream column over-counting; this
    /// fixture pins the validator-level property we care about:
    /// sparse modal rows whose populated columns form one connected
    /// component must be accepted.
    #[test]
    fn validate_accepts_hierarchical_grouped_table() {
        let grid = make_grouped_grid(8, 12, 2, 4);
        let config = TableDetectionConfig::default();
        assert!(validate_table_structure_internal(&grid, &config));
    }

    /// num_cols = 3 short-circuits has_split_modal_column_groups
    /// (num_cols < 4), so a small dense grid passes. Documents the
    /// boundary.
    #[test]
    fn validate_accepts_three_column_grid() {
        let grid = make_uniform_grid(3, 4, 3);
        let config = TableDetectionConfig::default();
        assert!(validate_table_structure_internal(&grid, &config));
    }

    // ========================================================================
    // consolidate_adjacent_table_fragments (#485 / #486 / #487 regression)
    // ========================================================================

    /// Build a minimal Table with a bbox and col_count for consolidation tests.
    fn make_fragment(x: f32, y: f32, width: f32, height: f32, cols: u32) -> Table {
        let mut t = Table::new();
        t.bbox = Some(Rect::new(x, y, width, height));
        t.col_count = cols as usize;
        // Push one empty row so consolidation has something to extend; the
        // row count grows as fragments get merged.
        t.rows.push(TableRow::new(false));
        t
    }

    /// Two vertically-adjacent fragments with identical column structure
    /// merge into a single multi-row table.  Models the issue-336 / nougat_018
    /// fragmented-table pattern: every horizontal ruling line produces a
    /// separate 1-row Table.
    #[test]
    fn consolidate_merges_adjacent_aligned_fragments() {
        let upper = make_fragment(90.0, 480.0, 420.0, 16.0, 8);
        let lower = make_fragment(90.0, 464.0, 420.0, 16.0, 8); // upper.bottom = 480, lower.top = 480
        let merged = consolidate_adjacent_table_fragments(vec![upper, lower]);
        assert_eq!(merged.len(), 1, "two aligned adjacent fragments must merge");
        assert_eq!(merged[0].rows.len(), 2, "merged rows are concatenated");
        let bb = merged[0].bbox.expect("merged bbox preserved");
        assert!((bb.x - 90.0).abs() < 0.1);
        assert!((bb.width - 420.0).abs() < 0.1);
        // Total height covers both fragments.
        assert!((bb.height - 32.0).abs() < 0.1);
    }

    /// Fragments separated by more than `Y_TOLERANCE = 3pt` must NOT merge —
    /// they represent two distinct tables (e.g. two unrelated grids on the
    /// same page).
    #[test]
    fn consolidate_does_not_merge_non_adjacent_fragments() {
        let upper = make_fragment(90.0, 600.0, 420.0, 16.0, 8);
        let lower = make_fragment(90.0, 400.0, 420.0, 16.0, 8); // 200pt gap
        let result = consolidate_adjacent_table_fragments(vec![upper, lower]);
        assert_eq!(result.len(), 2, "well-separated fragments stay distinct");
    }

    /// A table ruled between row *bands* (not every row) emits fragments
    /// separated by a full inter-row pitch, not abutting. The default 3pt
    /// tolerance leaves them split (dropping the short band to the >=3-row
    /// guard); the row-height-scaled tolerance rejoins them. Models the
    /// PMC8103272 Table-1 [3, 2] → [5] recovery.
    #[test]
    fn consolidate_with_tol_merges_row_pitch_separated_bands() {
        // Fragment with `extra` additional empty rows beyond make_fragment's one.
        let frag = |x, y, cols, rows: usize| {
            let mut t = make_fragment(x, y, 420.0, 16.0, cols);
            for _ in 1..rows {
                t.rows.push(TableRow::new(false));
            }
            t
        };
        // upper.bottom = 480; lower.top = 452 + 16 = 468 → 12pt gap (one pitch).
        let upper = frag(90.0, 480.0, 6, 3);
        let lower = frag(90.0, 452.0, 6, 2);
        // Default tolerance: bands stay split (this is the pre-fix behaviour).
        let split = consolidate_adjacent_table_fragments(vec![upper.clone(), lower.clone()]);
        assert_eq!(split.len(), 2, "3pt tolerance leaves row-pitch bands split");
        // Row-height-scaled tolerance (1.5 * ~16pt row height = 24 > 12pt gap): merge.
        let merged = consolidate_adjacent_table_fragments_with_tol(vec![upper, lower], 2.0, 24.0);
        assert_eq!(merged.len(), 1, "row-pitch bands rejoin under scaled tolerance");
        assert_eq!(merged[0].rows.len(), 5, "[3,2] bands rejoin into a 5-row table");
    }

    /// The scaled tolerance still refuses fragments that are farther apart
    /// than a plausible single row pitch — two genuinely distinct grids.
    #[test]
    fn consolidate_with_tol_still_rejects_far_bands() {
        let upper = make_fragment(90.0, 600.0, 420.0, 16.0, 6);
        let lower = make_fragment(90.0, 400.0, 420.0, 16.0, 6); // 184pt gap
        let merged = consolidate_adjacent_table_fragments_with_tol(vec![upper, lower], 2.0, 24.0);
        assert_eq!(merged.len(), 2, "a scaled tolerance is not an unbounded merge");
    }

    /// Fragments with different column counts must NOT merge even if
    /// they sit adjacent vertically — they aren't a single logical table.
    #[test]
    fn consolidate_does_not_merge_different_col_counts() {
        let upper = make_fragment(90.0, 480.0, 420.0, 16.0, 8);
        let lower = make_fragment(90.0, 464.0, 420.0, 16.0, 5); // different col_count
        let result = consolidate_adjacent_table_fragments(vec![upper, lower]);
        assert_eq!(result.len(), 2, "different col_count blocks merging");
    }

    /// Fragments at different x-positions must NOT merge — they're in
    /// different columns of the page even if their Y ranges are adjacent.
    #[test]
    fn consolidate_does_not_merge_misaligned_x() {
        let upper = make_fragment(90.0, 480.0, 420.0, 16.0, 8);
        let lower = make_fragment(50.0, 464.0, 420.0, 16.0, 8); // x offset 40pt
        let result = consolidate_adjacent_table_fragments(vec![upper, lower]);
        assert_eq!(result.len(), 2, "misaligned-x blocks merging");
    }

    /// A chain of three+ adjacent fragments should chain-merge into one
    /// multi-row table.  This is the issue-336 shape: 8 column-aligned
    /// fragments → one 18-row table after consolidation.
    #[test]
    fn consolidate_chains_multiple_adjacent_fragments() {
        let f1 = make_fragment(90.0, 480.0, 420.0, 16.0, 8);
        let f2 = make_fragment(90.0, 464.0, 420.0, 16.0, 8);
        let f3 = make_fragment(90.0, 448.0, 420.0, 16.0, 8);
        let f4 = make_fragment(90.0, 432.0, 420.0, 16.0, 8);
        let merged = consolidate_adjacent_table_fragments(vec![f1, f2, f3, f4]);
        assert_eq!(merged.len(), 1, "chain of 4 adjacent fragments → 1 table");
        assert_eq!(merged[0].rows.len(), 4, "all rows preserved in chain merge");
    }

    /// Small vertical overlap (line-detector quirk) up to half the
    /// smaller fragment's height is still considered adjacent.
    #[test]
    fn consolidate_tolerates_small_overlap() {
        // upper bottom = 480, lower top = 484 (4pt overlap, smaller_h = 16, half = 8)
        let upper = make_fragment(90.0, 480.0, 420.0, 16.0, 8);
        let lower = make_fragment(90.0, 468.0, 420.0, 16.0, 8);
        let merged = consolidate_adjacent_table_fragments(vec![upper, lower]);
        assert_eq!(merged.len(), 1, "small overlap (< ½ row) still merges");
    }

    /// Empty input passes through unchanged.
    #[test]
    fn consolidate_empty_input() {
        let merged = consolidate_adjacent_table_fragments(vec![]);
        assert!(merged.is_empty());
    }

    /// Single-table input passes through unchanged (nothing to merge).
    #[test]
    fn consolidate_single_table_passthrough() {
        let t = make_fragment(90.0, 480.0, 420.0, 16.0, 8);
        let merged = consolidate_adjacent_table_fragments(vec![t]);
        assert_eq!(merged.len(), 1);
    }

    // ========================================================================
    // cell_span_separator (#485 / #487 regression)
    // ========================================================================

    /// Helper to construct a TextSpan with just the fields the separator
    /// rule actually reads: bbox, font_size, and text.
    fn ts(text: &str, x: f32, y: f32, width: f32, fs: f32) -> TextSpan {
        TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: text.to_string(),
            bbox: Rect::new(x, y, width, fs),
            font_size: fs,
            font_name: String::new(),
            font_weight: crate::layout::text_block::FontWeight::Normal,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            split_boundary_before: false,
            offset_semantic: false,
            is_italic: false,
            is_monospace: false,
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

    /// Adjacent spans with a sub-em gap must NOT have a separator
    /// inserted.  Models issue-336's `60000` + `≤` + `Q` + `＜` + `80000`
    /// where the operator glyphs touch / slightly overlap the digit
    /// glyphs and must be rendered as a single compound token.
    #[test]
    fn cell_span_separator_no_space_for_tight_glyphs() {
        let prev = ts("60000≤Q", 100.0, 200.0, 39.8, 10.6);
        let curr = ts("＜", 139.8, 200.0, 10.6, 10.6); // gap = 0
        assert_eq!(cell_span_separator(&prev, &curr), "");
    }

    /// Adjacent spans with a real gap (clear word boundary) get a
    /// single space separator inserted.
    #[test]
    fn cell_span_separator_inserts_space_for_real_gap() {
        let prev = ts("Quarter", 100.0, 200.0, 30.0, 10.0); // ends at x=130
        let curr = ts("Total", 140.0, 200.0, 25.0, 10.0); // gap = 10pt = 1em
        assert_eq!(cell_span_separator(&prev, &curr), " ");
    }

    /// CJK ↔ fullwidth-operator boundary suppresses separator even when
    /// the geometric gap would otherwise warrant a space.
    #[test]
    fn cell_span_separator_suppresses_cjk_fullwidth_boundary() {
        // "中" (U+4E2D, CJK) followed by "＜" (U+FF1C, fullwidth less-than).
        let prev = ts("中", 100.0, 200.0, 12.0, 12.0); // ends at 112
        let curr = ts("＜", 115.0, 200.0, 12.0, 12.0); // gap = 3pt = 0.25 em
        assert_eq!(cell_span_separator(&prev, &curr), "");
    }

    /// Trailing whitespace on the preceding span suppresses separator
    /// (no double-space).
    #[test]
    fn cell_span_separator_suppresses_on_existing_trailing_space() {
        let prev = ts("Quarter ", 100.0, 200.0, 35.0, 10.0); // text ends with ' '
        let curr = ts("Total", 140.0, 200.0, 25.0, 10.0);
        assert_eq!(cell_span_separator(&prev, &curr), "");
    }
}
