//! Markdown output converter.
//!
//! Converts ordered text spans to Markdown format.

use crate::error::Result;
use crate::layout::FontWeight;
use crate::pipeline::{OrderedTextSpan, StructRole, TextPipelineConfig};
use crate::structure::table_extractor::Table;
use crate::text::HyphenationHandler;
use regex::Regex;
use std::sync::LazyLock;

use super::OutputConverter;

static RE_URL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(https?://[^\s<>\[\]]*[^\s<>\[\].,!?;:])").unwrap());
static RE_EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})").unwrap());

/// The text with every whitespace character removed.
///
/// Used to compare what a table rendered against what a span carries when the
/// two sides disagree only about spacing.
fn squash_whitespace(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Put `out` at the start of a line before a block-level marker is written
/// into it.
///
/// A heading prefix only opens a heading at the start of a line. Written after
/// text it is three literal `#` characters welded to the preceding word, and
/// the heading it was meant to open is lost as well as the word boundary — a
/// form's reset-button label came out as `au Luxembourg### Réinitialiser`.
///
/// This is only ever additive: it inserts a newline that CommonMark requires
/// and never removes one, so a caller that already ended its line is
/// unaffected.
fn begin_block(out: &mut String) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

/// Detect markdown table separator rows like `|---|---|` or
/// `| :--- | ---: |`. A line qualifies if every `|`-delimited cell is
/// a sequence of `-` (with optional surrounding `:` for alignment) and
/// optional spaces. At least two cells required so single-pipe lines
/// (which are the very pattern we're trying to escape) do not match.
fn is_table_separator_line(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return false;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    let cells: Vec<&str> = inner.split('|').collect();
    if cells.len() < 2 {
        return false;
    }
    cells.iter().all(|cell| {
        let c = cell.trim();
        !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':')
    })
}

/// Issue #10 band-aid. Walk the rendered markdown line by line; for any
/// line that starts with `|` but is *not* part of a markdown table block
/// (defined as the line itself being a separator, or the next line being
/// a separator, or the previous line already classified as in-table),
/// escape the leading `|` as `\|`. Without this, stray header/footer
/// fragments leak into prose and downstream markdown parsers misread
/// them as malformed table rows, fragmenting subsequent text.
fn escape_stray_leading_pipes(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    let mut in_table = vec![false; lines.len()];

    // First pass: classify separator lines and the lines immediately
    // above (header) and below (data rows) that are clearly part of
    // the same table block.
    for (i, line) in lines.iter().enumerate() {
        if is_table_separator_line(line) {
            in_table[i] = true;
            if i > 0 && lines[i - 1].trim_start().starts_with('|') {
                in_table[i - 1] = true;
            }
            // Mark contiguous downstream data rows that also start with `|`.
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim_start().starts_with('|') {
                in_table[j] = true;
                j += 1;
            }
        }
    }

    let mut out = String::with_capacity(s.len());
    for (i, line) in lines.iter().enumerate() {
        if !in_table[i] {
            let leading_ws_len = line.len() - line.trim_start().len();
            let trimmed = &line[leading_ws_len..];
            if let Some(rest) = trimmed.strip_prefix('|') {
                out.push_str(&line[..leading_ws_len]);
                out.push_str("\\|");
                out.push_str(rest);
            } else {
                out.push_str(line);
            }
        } else {
            out.push_str(line);
        }
        if i + 1 < lines.len() {
            out.push('\n');
        }
    }
    out
}

/// Heuristic for the 2-fragment wrapped-heading case used by
/// `merge_consecutive_same_level_headings` (issue #4). Returns true
/// when the two heading fragments visually look like ONE heading split
/// across two lines (wrap), as opposed to two distinct same-level
/// sections.
///
/// Generic, script-agnostic signals (no English word lists):
///   1. First fragment does NOT end with a sentence-terminating
///      punctuation (`.`, `?`, `!`, and their CJK/Arabic equivalents
///      `。`, `？`, `！`, `؟`). Sentence-end is the strong split
///      signal across scripts.
///   2. AND one of:
///      a) first ends with continuation punctuation (`,`, `;`, `、`,
///         `；` — comma / semicolon variants), OR
///      b) second fragment opens with a Unicode-lowercase letter
///         (`\p{Ll}`). A wrapped heading's continuation is virtually
///         always lowercase (or non-cased in scripts that lack case)
///         while a distinct following heading typically begins with a
///         capitalized word.
fn looks_like_heading_wrap(first: &str, second: &str) -> bool {
    let first_trim = first.trim_end();
    if let Some(last) = first_trim.chars().last() {
        // Sentence terminators (Latin + CJK + Arabic).
        if matches!(last, '.' | '?' | '!' | '。' | '？' | '！' | '\u{061F}') {
            return false;
        }
        // Continuation punctuation (Latin comma/semicolon + CJK + middle dot).
        if matches!(last, ',' | ';' | '、' | '；' | '·') {
            return true;
        }
    }
    // Lowercase opener on the second fragment, Unicode-aware via
    // char.is_lowercase() (matches `\p{Ll}`).
    let second_first = second.trim_start().chars().next();
    if let Some(c) = second_first {
        if c.is_lowercase() {
            return true;
        }
    }
    false
}

/// Issue #2 fix. Drop consecutive duplicate paragraphs from the final
/// markdown. Duplicates surface in the reporter's corpus when the
/// extractor emits the same content twice (once via the structure
/// pipeline, once via the plaintext fallback). Exact-match only; we
/// will not touch near-duplicates because legitimate prose can repeat
/// a short phrase.
// RETIRED from the active pipeline (see render_spans). Removes legit
// repeated content (distinct form widgets with identical labels,
// repeated headings). Kept for reference + unit-test documentation.
#[allow(dead_code)]
fn dedup_consecutive_paragraphs(s: &str) -> String {
    let paras: Vec<&str> = s.split("\n\n").collect();
    let mut out: Vec<&str> = Vec::with_capacity(paras.len());
    let mut prev_norm: Option<String> = None;
    for p in paras {
        let norm: String = p
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if norm.is_empty() {
            out.push(p);
            prev_norm = None;
            continue;
        }
        if prev_norm.as_deref() == Some(norm.as_str()) {
            // Skip — identical to the immediately-previous content paragraph.
            continue;
        }
        prev_norm = Some(norm);
        out.push(p);
    }
    out.join("\n\n")
}

/// Issue #5 fix. Some spatial-grouping artifacts produce header rows
/// where every cell carries the same identifier (e.g. `| Q1'25 |
/// Q1'25 | Q1'25 | Q1'25 |`). Detect such all-identical header rows
/// (marker: the row's next line IS a markdown separator `|---|...|`)
/// and dedup so only the first cell carries the value. Conservative:
/// only fires when ALL non-empty cells are byte-identical AND there
/// are >= 3 cells (single duplicates are too ambiguous to touch).
// RETIRED from the active pipeline (see render_spans). Blanking
// "duplicate" header cells assumes the duplication is an artifact.
// Kept for reference + unit-test documentation.
#[allow(dead_code)]
fn dedup_identical_header_cells(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let next_is_sep = i + 1 < lines.len() && is_table_separator_line(lines[i + 1]);
        let trimmed = line.trim();
        let looks_like_header = trimmed.starts_with('|') && trimmed.ends_with('|');
        if !next_is_sep || !looks_like_header {
            out.push(line.to_string());
            i += 1;
            continue;
        }
        let inner = &trimmed[1..trimmed.len() - 1];
        let cells: Vec<&str> = inner.split('|').collect();
        let non_empty: Vec<&str> = cells
            .iter()
            .map(|c| c.trim())
            .filter(|c| !c.is_empty())
            .collect();
        if non_empty.len() < 3 {
            out.push(line.to_string());
            i += 1;
            continue;
        }
        let first = non_empty[0];
        let all_same = non_empty.iter().all(|c| *c == first);
        if !all_same {
            out.push(line.to_string());
            i += 1;
            continue;
        }
        // Rewrite: keep first cell, blank the rest. Preserve cell count.
        let mut new_cells: Vec<String> = Vec::with_capacity(cells.len());
        let mut wrote_first = false;
        for cell in &cells {
            if cell.trim().is_empty() {
                new_cells.push(String::new());
            } else if !wrote_first {
                new_cells.push(format!(" {} ", cell.trim()));
                wrote_first = true;
            } else {
                new_cells.push(String::from(" "));
            }
        }
        out.push(format!("|{}|", new_cells.join("|")));
        i += 1;
    }
    out.join("\n")
}

/// Issue #1 + #4 fix. Merge runs of consecutive same-level markdown
/// headings into a single heading when the run is unambiguously ONE
/// logical heading. See `looks_like_heading_wrap` for the 2-fragment
/// wrapped-heading rule; otherwise require 3+ fragments each <= 2
/// words (canonical PowerPoint word-per-heading pattern).
/// Is `line` a Markdown list-item marker line (`- `, `* `, `+ `, or an ordered
/// `N.`/`N)` marker)? Used to tighten lists.
fn is_md_list_item_line(line: &str) -> bool {
    let t = line.trim_start();
    if let Some(rest) = t
        .strip_prefix("- ")
        .or_else(|| t.strip_prefix("* "))
        .or_else(|| t.strip_prefix("+ "))
    {
        return !rest.trim().is_empty() || rest.is_empty();
    }
    // Ordered marker: leading ASCII digits then '.' or ')' then a space.
    let digits = t.bytes().take_while(|b| b.is_ascii_digit()).count();
    digits > 0
        && matches!(t.as_bytes().get(digits), Some(b'.') | Some(b')'))
        && t.as_bytes().get(digits + 1) == Some(&b' ')
}

/// Collapse blank lines that sit between two consecutive list-item marker lines
/// so the list renders tight (CommonMark §5.3) rather than loose. The span
/// flush in [`MarkdownOutputConverter::convert`] always appends a blank line
/// after each item, which would otherwise wrap every item in its own `<p>` and
/// fragment the document's list structure. Blank lines that separate a list
/// item from non-list content (the end of the list) are preserved.
fn tighten_list_items(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        out.push(lines[i].to_string());
        if is_md_list_item_line(lines[i]) {
            // Look past blank lines to the next content line.
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            // Only collapse when the next content line is also a list item AND
            // at least one blank line separated them (so we actually tighten).
            if j > i + 1 && j < lines.len() && is_md_list_item_line(lines[j]) {
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out.join("\n")
}

fn merge_consecutive_same_level_headings(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        // Capture leading `#`s, require space after.
        let level = trimmed.bytes().take_while(|&b| b == b'#').count();
        let is_heading =
            (1..=6).contains(&level) && trimmed.as_bytes().get(level).copied() == Some(b' ');
        if !is_heading {
            out.push(line.to_string());
            i += 1;
            continue;
        }

        // Accumulate consecutive same-level headings separated only by
        // blank lines. No word-count gate here — policy decision is
        // made AFTER collection so the wrapped-2-fragment case (which
        // tolerates longer fragments) is reachable.
        let mut texts: Vec<String> = vec![trimmed[level + 1..].trim().to_string()];
        let mut j = i + 1;
        loop {
            // Skip blank lines.
            while j < lines.len() && lines[j].trim().is_empty() {
                j += 1;
            }
            if j >= lines.len() {
                break;
            }
            let next_trim = lines[j].trim_start();
            let next_level = next_trim.bytes().take_while(|&b| b == b'#').count();
            let next_is_heading =
                next_level == level && next_trim.as_bytes().get(next_level).copied() == Some(b' ');
            if !next_is_heading {
                break;
            }
            let next_text = next_trim[next_level + 1..].trim().to_string();
            // Hard guard: refuse to even ATTEMPT merge if any single
            // fragment is implausibly long for a heading (> 15 words).
            // That cap is high enough that no real wrapped heading
            // exceeds it, while still preventing pathological fusion.
            if next_text.split_whitespace().count() > 15 {
                break;
            }
            texts.push(next_text);
            j += 1;
        }

        // Two policies that both prove the run is one logical heading:
        //   A) 3+ fragments AND each <= 2 words — canonical PowerPoint
        //      word-per-heading pattern.
        //   B) Exactly 2 fragments AND the FIRST ends with a
        //      continuation-strength punctuation (`,` or `;`) or no
        //      sentence-terminator (`.`, `?`, `!`, `:`). The second
        //      fragment must visually look like a continuation: start
        //      lowercase or with a connector word ("and"/"or"/"the"/
        //      "with"/"of"/...). This matches the reporter's wrapped-
        //      heading shape `## Despite seasonal slowdown,` +
        //      `## warehouse operations maintained...` while still
        //      keeping `# First Heading` / `# Second Heading` apart
        //      (no trailing comma, second word "Second" is capitalized
        //      and not a connector).
        let three_plus_short =
            texts.len() >= 3 && texts.iter().all(|t| t.split_whitespace().count() <= 2);
        let wrapped_two = texts.len() == 2 && looks_like_heading_wrap(&texts[0], &texts[1]);
        if three_plus_short || wrapped_two {
            let merged = texts.join(" ");
            let hashes = "#".repeat(level);
            out.push(format!("{} {}", hashes, merged));
            i = j;
        } else {
            out.push(line.to_string());
            i += 1;
        }
    }
    out.join("\n")
}

/// Issue #9 — DELIBERATELY NOT a post-process filter. Initial
/// implementation regex-matched "Page N" / "N of M" / "— 12 —" at
/// the markdown stage and dropped those lines from the output. That
/// was wrong: it discards legitimate text content. If a PDF actually
/// has "Page 1" in its content stream the correct behavior is to
/// extract it, not silently delete it.
///
/// The proper fix lives upstream and follows the PDF spec
/// (ISO 32000-1:2008 §14.8.2.2 "Artifacts"). Pagination, headers,
/// and footers are supposed to be marked as `/Artifact` marked-
/// content elements; extraction can/should skip artifacts when
/// producing the document's logical text stream. For untagged PDFs
/// without artifact metadata, geometric header/footer detection at
/// extraction time (consistent y-position across pages, repeated
/// content) is the correct heuristic — not a regex that pattern-
/// matches the rendered prose.
///
/// The function is retained as a no-op stub for backward source
/// compatibility (the post-process pipeline below no longer invokes
/// it). Future work: implement the upstream artifact-skip path.
#[allow(dead_code)]
fn filter_page_number_lines(s: &str) -> String {
    s.to_string()
}

/// Issue #13 — DELIBERATELY NOT a post-process replacement. The
/// reporter's examples (`•` → `❍`, unexpected `ī`, `Ƅ`, `ώ`) all
/// trace back to font-encoding / ToUnicode CMap misses in the
/// extractor (PARSER_WARNINGS report, 25,350 occurrences of
/// "ToUnicode CMap MISS"). Pattern-replacing codepoints at the
/// markdown layer would MODIFY the document's actual text — if a
/// PDF really uses `❍` deliberately, dropping it to `•` is content
/// corruption, not a fix.
///
/// The correct fix is upstream and follows PDF §9.10 (Extraction of
/// text content): when a Type0 font has no `/ToUnicode` CMap and no
/// recognizable Encoding, fall back to the `/CIDSystemInfo` or
/// glyph-name heuristics rather than emitting garbage codepoints.
/// The bullet symptom disappears for free once the CMap fallback
/// path is robust.
///
/// Function retained as a no-op for backward source compatibility.
#[allow(dead_code)]
fn normalize_bullet_glyphs(s: &str) -> String {
    s.to_string()
}

/// Issues #3 / #6 / partial #11 band-aid. Detect "degenerate" markdown
/// table blocks produced by the spatial-table heuristic firing on
/// multi-column prose, and replace them with a single flowing paragraph.
///
/// A table block is considered degenerate when:
///   - >= 5 columns (typical multi-column prose run width),
///   - >= 2 data rows after the header/separator,
///   - >= 60% of non-empty cells contain a single word.
///
/// Such blocks are almost never legitimate data tables — real tables in
/// the test corpus average 2-4 words per cell. The replacement is a
/// best-effort: concatenate every non-empty cell with a single space, in
/// row-major order.
// RETIRED from the active pipeline (see render_spans). Flattened a
// real country-data table in the 70-PDF regression sweep. A
// markdown-layer heuristic cannot reliably distinguish a spurious
// prose "table" from a real sparse one. Kept for reference +
// unit-test documentation.
#[allow(dead_code)]
fn simplify_degenerate_tables(s: &str) -> String {
    let lines: Vec<&str> = s.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        // Detect a candidate table: header row + separator + at least one data row.
        let header = lines[i];
        if !header.trim_start().starts_with('|')
            || i + 1 >= lines.len()
            || !is_table_separator_line(lines[i + 1])
        {
            out.push(header.to_string());
            i += 1;
            continue;
        }

        // Collect the full table block.
        let mut block_end = i + 2;
        while block_end < lines.len() && lines[block_end].trim_start().starts_with('|') {
            block_end += 1;
        }
        let block = &lines[i..block_end];

        // Split each row's cells (drop the outer empty cells from the
        // leading/trailing pipes).
        let parse_row = |row: &str| -> Vec<String> {
            row.trim()
                .trim_start_matches('|')
                .trim_end_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect()
        };

        let header_cells = parse_row(header);
        let data_rows: Vec<Vec<String>> = block.iter().skip(2).map(|r| parse_row(r)).collect();

        let cols = header_cells.len();
        let data_row_count = data_rows.len();

        if cols < 5 || data_row_count < 2 {
            out.extend(block.iter().map(|l| l.to_string()));
            i = block_end;
            continue;
        }

        // Compute single-word-cell ratio among non-empty cells.
        let mut non_empty = 0usize;
        let mut single_word = 0usize;
        for cell in header_cells.iter().chain(data_rows.iter().flatten()) {
            if cell.is_empty() {
                continue;
            }
            non_empty += 1;
            if cell.split_whitespace().count() == 1 {
                single_word += 1;
            }
        }
        if non_empty == 0 {
            // Pure empty block — drop entirely.
            i = block_end;
            continue;
        }
        let single_ratio = single_word as f32 / non_empty as f32;

        if single_ratio < 0.6 {
            out.extend(block.iter().map(|l| l.to_string()));
            i = block_end;
            continue;
        }

        // Degenerate: flatten to a single paragraph.
        let mut words: Vec<String> = Vec::new();
        for cell in header_cells.iter().chain(data_rows.iter().flatten()) {
            if !cell.is_empty() {
                words.push(cell.clone());
            }
        }
        out.push(words.join(" "));
        i = block_end;
    }
    out.join("\n")
}

/// Issue #11 (partial) band-aid. Detect runs of 2+ consecutive numeric-only
/// H1/H2 headings (e.g. `# 23,500`, `# 99.2%`, `# 87%`, `# 4.2 days`)
/// produced when a KPI dashboard's large numbers were spatially read as
/// stand-alone headings. Convert the run into a bulleted list so the
/// values render as data instead of as section titles. Conservative:
/// every heading in the run must match the numeric pattern; if any one
/// fails, the run is left alone.
fn collapse_numeric_heading_runs(s: &str) -> String {
    // Matches a heading line whose body is a short numeric/percentage/
    // currency/duration value. Allowed: digits, comma/period/colon/dash/
    // slash, `%`, `$`, `£`, `€`, optional letters for "K"/"M"/"B"/"days"/
    // "hrs"/"min"/"sec". Capped length keeps real numeric headings
    // (e.g. "# 2024 Annual Report") from matching by accident.
    static RE_NUMERIC_HEADING: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(#{1,2})\s+([\$£€]?\d[\d,.:\-/]*\s*(?:%|K|M|B|days|day|hrs|hr|min|sec)?)\s*$")
            .unwrap()
    });
    let lines: Vec<&str> = s.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        // Skip blank lines normally.
        if !RE_NUMERIC_HEADING.is_match(lines[i]) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }
        // Found one — look ahead for more numeric headings of the same
        // level, allowing blank-line separators.
        let level = lines[i]
            .trim_start()
            .bytes()
            .take_while(|&b| b == b'#')
            .count();
        let mut values: Vec<String> = Vec::new();
        let mut last_match_idx = i;
        let mut j = i;
        while j < lines.len() {
            if lines[j].trim().is_empty() {
                j += 1;
                continue;
            }
            let trim = lines[j].trim_start();
            let l = trim.bytes().take_while(|&b| b == b'#').count();
            if l != level {
                break;
            }
            if let Some(caps) = RE_NUMERIC_HEADING.captures(lines[j]) {
                let v = caps
                    .get(2)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();
                if v.chars().count() > 20 {
                    break;
                }
                values.push(v);
                last_match_idx = j;
                j += 1;
            } else {
                break;
            }
        }
        if values.len() < 2 {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }
        // Emit as a bulleted list.
        for v in &values {
            out.push(format!("- {}", v));
        }
        out.push(String::new()); // trailing blank line
        i = last_match_idx + 1;
    }
    out.join("\n")
}

/// Issue #12 (narrow) band-aid. Within a single bold block `**...**`,
/// detect the CamelCase fragmentation pattern produced when a word
/// rendered with mixed fonts (e.g. bold first letter, regular rest) is
/// emitted as space-separated fragments inside one bold span. The
/// canonical example from the reporter's corpus is `**S alesF orce**`
/// (intended: `**SalesForce**`).
///
/// Match criteria: a single uppercase ASCII letter followed by a space,
/// then a lowercase chunk that itself contains a later uppercase letter
/// (the CamelCase indicator), then a space and another lowercase chunk.
/// All three pieces must live inside the same `**...**` pair. Replacing
/// `**A bcD efg**` with `**AbcDefg**`.
///
/// Conservative on purpose: matching mid-prose "I am Bob" or "USB Type C"
/// would corrupt legitimate text, so the regex requires the CamelCase
/// signal to be unambiguous (lowercase+uppercase within a single inner
/// fragment).
fn coalesce_camelcase_bold_fragments(s: &str) -> String {
    // Unicode-aware (script-agnostic): `\p{Lu}` matches any
    // uppercase letter in Unicode, `\p{Ll}` matches any lowercase
    // letter. The CamelCase signal — a lowercase-letter run
    // containing a later uppercase letter inside one fragment — is
    // unambiguous across Latin, Cyrillic, Greek, Armenian, Coptic,
    // and other cased scripts. Non-cased scripts (CJK, Arabic,
    // Hebrew) lack CamelCase entirely so the pattern can never
    // match — that's correct behavior.
    //
    // Pass 1 — inline form: `**A bcD ef**` (closing `**` after the
    // lowercase tail). Three fragments inside one bold pair.
    static RE_CAMELCASE_BOLD_INLINE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\*\*(\p{Lu})\s+(\p{Ll}+\p{Lu}\p{Ll}*)\s+(\p{Ll}+)\*\*").unwrap()
    });
    // Pass 2 — bound form: `**A bcD** ef` (closing `**` mid-CamelCase,
    // lowercase tail outside the bold). Two fragments inside the bold
    // pair, tail immediately (or after one optional space) after.
    static RE_CAMELCASE_BOLD_BOUND: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\*\*(\p{Lu})\s+(\p{Ll}+\p{Lu}\p{Ll}*)\*\*\s*(\p{Ll}+)").unwrap()
    });
    let pass1 = RE_CAMELCASE_BOLD_INLINE
        .replace_all(s, |caps: &regex::Captures| {
            format!("**{}{}{}**", &caps[1], &caps[2], &caps[3])
        })
        .to_string();
    RE_CAMELCASE_BOLD_BOUND
        .replace_all(&pass1, |caps: &regex::Captures| {
            format!("**{}{}{}**", &caps[1], &caps[2], &caps[3])
        })
        .to_string()
}

/// Markdown output converter.
///
/// Converts ordered text spans to Markdown format with optional formatting:
/// - Bold text using `**text**` markers
/// - Italic text using `*text*` markers
/// - Heading detection based on font size (when enabled)
/// - Paragraph separation based on vertical gaps
/// - Table detection and formatting
/// - Layout preservation with whitespace
/// - URL/Email linkification
/// - Whitespace normalization
pub struct MarkdownOutputConverter {
    /// Line spacing threshold ratio for paragraph detection.
    paragraph_gap_ratio: f32,
}

impl MarkdownOutputConverter {
    /// Create a new Markdown converter with default settings.
    pub fn new() -> Self {
        Self {
            paragraph_gap_ratio: 1.5,
        }
    }

    /// Create a Markdown converter with custom paragraph gap ratio.
    pub fn with_paragraph_gap(ratio: f32) -> Self {
        Self {
            paragraph_gap_ratio: ratio,
        }
    }

    /// Check if a span should be rendered as bold.
    fn is_bold(&self, span: &OrderedTextSpan, config: &TextPipelineConfig) -> bool {
        use crate::pipeline::config::BoldMarkerBehavior;

        match span.span.font_weight {
            FontWeight::Bold | FontWeight::Black | FontWeight::ExtraBold | FontWeight::SemiBold => {
                match config.output.bold_marker_behavior {
                    BoldMarkerBehavior::Aggressive => true,
                    BoldMarkerBehavior::Conservative => {
                        // Only apply bold to content-bearing text
                        span.span.text.chars().any(|c| !c.is_whitespace())
                    },
                }
            },
            _ => false,
        }
    }

    /// Check if a span should be rendered as italic.
    fn is_italic(&self, span: &OrderedTextSpan) -> bool {
        span.span.is_italic && span.span.text.chars().any(|c| !c.is_whitespace())
    }

    /// Apply linkification to text (URLs and emails).
    fn linkify(&self, text: &str) -> String {
        // Quick pre-check: skip regex for spans that can't contain URLs or emails.
        // This avoids regex overhead for ~95% of regular text spans.
        let might_have_url = text.contains("://") || text.contains("www.");
        let might_have_email = text.contains('@');

        if !might_have_url && !might_have_email {
            return text.to_string();
        }

        let mut result = if might_have_url {
            RE_URL
                .replace_all(text, |caps: &regex::Captures| {
                    let url = &caps[0];
                    format!("[{}]({})", url, url)
                })
                .to_string()
        } else {
            text.to_string()
        };

        if might_have_email {
            result = RE_EMAIL
                .replace_all(&result, |caps: &regex::Captures| {
                    let email = &caps[0];
                    format!("[{}](mailto:{})", email, email)
                })
                .to_string();
        }

        result
    }

    /// Normalize whitespace in text.
    fn normalize_whitespace(&self, text: &str) -> String {
        // Replace multiple spaces with single space
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Detect paragraph breaks between spans based on vertical spacing.
    ///
    /// Two break signals:
    /// 1. Vertical gap larger than `paragraph_gap_ratio × line_height`
    ///    (the classic geometric heuristic).
    /// 2. The current line begins with a list marker (bullet glyph or
    ///    ordered marker) while the previous line did not — list-items
    ///    must always start a fresh paragraph regardless of how tightly
    ///    they sit under the preceding paragraph (issue #377 D4: many
    ///    untagged docs use a sub-1.5× line gap before lists, which
    ///    glues the first item to the intro sentence).
    fn is_paragraph_break(&self, current: &OrderedTextSpan, previous: &OrderedTextSpan) -> bool {
        let line_height = current.span.font_size.max(previous.span.font_size);
        let gap = (previous.span.bbox.y - current.span.bbox.y).abs();
        if gap > line_height * self.paragraph_gap_ratio {
            return true;
        }
        // List-prefix transition guard. Bullet glyph or `1.` / `a)` /
        // `i.` ordered marker at the start of the current line, with
        // the previous line on a different baseline and not itself a
        // list item. The ordered-marker detection is conservative
        // (single digit/letter at line start) so figure captions
        // ("1.1 Foo") and years ("1986") are not promoted to lists.
        let line_changed =
            (previous.span.bbox.y - current.span.bbox.y).abs() > current.span.font_size * 0.5;
        if line_changed {
            let cur_text = current.span.text.trim_start();
            let cur_starts_list = Self::is_bullet_span(cur_text)
                || Self::starts_with_bullet(cur_text)
                || Self::is_ordered_list_marker(cur_text).is_some();
            let prev_text = previous.span.text.trim_start();
            let prev_starts_list = Self::is_bullet_span(prev_text)
                || Self::starts_with_bullet(prev_text)
                || Self::is_ordered_list_marker(prev_text).is_some();
            if cur_starts_list && !prev_starts_list {
                return true;
            }
        }
        false
    }

    /// Detect a markdown ordered-list marker at the start of `text`.
    /// Recognises `1.`, `12.`, `a.`, `iv.`, `1)`, `a)` followed by a
    /// space. Returns the (1-based) position number when known
    /// (Roman numerals coerced to position 1 for now), or `None`.
    ///
    /// Conservative on purpose — only single digit/letter tokens at
    /// the very start of the trimmed text qualify, so figure captions
    /// like "1.1 Foo" and years like "1986" are not falsely promoted
    /// to numbered lists. See issue #377 D3.
    fn is_ordered_list_marker(text: &str) -> Option<u32> {
        super::is_ordered_list_marker(text)
    }

    /// Check if a span consists of a single bullet character.
    ///
    /// Common bullet characters used in PDF documents:
    /// ► • ▪ ▸ ‣ ◦ ● ■ ◆ ○ □ ❍ ❖ ✓ ✔ ➢ ➤ 
    fn is_bullet_span(text: &str) -> bool {
        super::is_bullet_span(text)
    }

    /// Check if text starts with a bullet character (for inline bullets).
    fn starts_with_bullet(text: &str) -> bool {
        super::starts_with_bullet(text)
    }

    /// Validate that a string looks like a heading (not a paragraph or noise).
    ///
    /// Content-based guards only — no language/locale-specific keyword lists.
    fn is_valid_heading_text(text: &str) -> bool {
        let trimmed = text.trim();
        let text_len = trimmed.chars().count();
        // Headings must be non-trivial but also not full paragraphs.
        // 200 chars is ~35 words, which safely accommodates long wrapped titles
        // while excluding paragraph-length runs that share a larger font.
        if !(2..=200).contains(&text_len) {
            return false;
        }
        // Reject a bare ordinal suffix (`st`/`nd`/`rd`/`th`) left stranded when a
        // superscript ordinal is split from its number ("May 5th" → "May 5" +
        // superscript "th"). On its own it is never a heading; promoting it emits
        // a stray "#### th" that fragments the document outline.
        if matches!(trimmed.to_ascii_lowercase().as_str(), "st" | "nd" | "rd" | "th") {
            return false;
        }
        // Sentence-length guards: a heading rarely exceeds 20 words and
        // almost never contains a full stop followed by more text (that's
        // a paragraph, even if it happens to be set in a larger font).
        let word_count = trimmed.split_whitespace().count();
        if word_count > 20 {
            return false;
        }
        // Exclude runs with mid-sentence punctuation ("foo. Bar baz") —
        // real headings don't contain sentence boundaries.
        let bytes = trimmed.as_bytes();
        for i in 0..bytes.len().saturating_sub(2) {
            if bytes[i] == b'.' && bytes[i + 1] == b' ' {
                let next = bytes[i + 2];
                if next.is_ascii_alphabetic() {
                    return false;
                }
            }
        }

        // Reject if dominated by digits/punctuation (KPI numbers, page numbers,
        // "$100", "23.5K"). Require a minimum alphabetic ratio that scales:
        // very short strings need at least 2 letters; longer strings need
        // >=30% alphabetic characters.
        let alpha_count = trimmed.chars().filter(|c| c.is_alphabetic()).count();
        if text_len <= 8 {
            if alpha_count < 2 {
                return false;
            }
        } else if alpha_count * 10 < text_len * 3 {
            return false;
        }

        // Reject KPI-style values ("4.2 days", "+15% QoQ", "$1.2M Total"):
        // strings that LEAD with a number/sign/currency symbol are almost
        // always data values, not headings, even in a larger font. A real
        // heading leads with a word.
        let first = trimmed.chars().next().unwrap_or(' ');
        if first.is_ascii_digit() || matches!(first, '+' | '-' | '$' | '€' | '£' | '¥' | '%') {
            return false;
        }

        // Reject leader/rule/separator artifacts: a run of 4+ identical fill
        // characters (dot leaders, dashed rules, table separators) is TOC/table
        // furniture, never a heading — e.g. a font-listing row
        // "Embedded font--------------------Bavhavs" that a font-size tier would
        // otherwise promote (172 spurious `#` observed on one font-sample doc).
        {
            let mut run = 1usize;
            let mut prev = '\0';
            for c in trimmed.chars() {
                run = if c == prev && matches!(c, '-' | '_' | '.' | '·' | '—' | '=' | '*') {
                    run + 1
                } else {
                    1
                };
                if run >= 4 {
                    return false;
                }
                prev = c;
            }
        }

        // Reject a run that reads as a piece of a sentence rather than a
        // title — one ending on a function word, or opening lowercase and
        // closing on a full stop. See `reads_as_a_sentence_fragment`.
        if super::reads_as_a_sentence_fragment(trimmed) {
            return false;
        }

        // Reject a line ending in a hyphen: that is a mid-word line break
        // ("...three categories: com-"), i.e. a wrapped body line, not a heading.
        if trimmed.ends_with('-') {
            return false;
        }

        // Reject a lowercase-initial multi-word run: a real multi-word heading
        // leads with a capital; a lowercase start on 5+ words is a body
        // continuation wrongly caught by a font-size tier. (CJK/Arabic have no
        // case, so `is_lowercase()` is false there and those headings pass.)
        if word_count >= 5 {
            if let Some(f) = trimmed.chars().find(|c| c.is_alphabetic()) {
                if f.is_lowercase() {
                    return false;
                }
            }
        }

        // Title-case vs sentence-case: this is the line-shape signal that a
        // pure font-size heuristic (pymupdf4llm) lacks and an ML layout model
        // (marker) captures. A long line whose Latin-script words are mostly
        // lowercase is flowing prose — body text glued onto a promoted span —
        // not a heading, however large the font. Real multi-word headings are
        // title-cased (or short, <=8 words, which are exempt). Only cased-script
        // words are counted, so CJK/Arabic/numeric headings are unaffected.
        if word_count > 8 {
            let cased: Vec<char> = trimmed
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .filter(|c| c.is_uppercase() || c.is_lowercase())
                .collect();
            if !cased.is_empty() {
                let upper = cased.iter().filter(|c| c.is_uppercase()).count();
                // < 40% of cased words capitalised → sentence-case body.
                if upper * 5 < cased.len() * 2 {
                    return false;
                }
            }
        }

        true
    }

    /// Re-validate every emitted heading against its COMPLETE line, demoting
    /// (dropping the `#` prefix) any that no longer reads as a heading.
    ///
    /// Heading level is decided per span (before the line's spans are
    /// concatenated), so a short span that passed [`Self::is_valid_heading_text`]
    /// can accrete a body continuation into a long, sentence-case line. This
    /// applies the same gate at line granularity — the block-level view an ML
    /// layout model would take. Demotion-only: a genuine heading always passes,
    /// so this can never remove a real heading nor create a false one.
    fn demote_body_like_headings(md: &str) -> String {
        let mut out = String::with_capacity(md.len());
        for (i, line) in md.split('\n').enumerate() {
            if i > 0 {
                out.push('\n');
            }
            let hashes = line.bytes().take_while(|&b| b == b'#').count();
            if (1..=6).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ') {
                let text = line[hashes + 1..].trim();
                if !Self::is_valid_heading_text(text) {
                    // Demote to a plain paragraph line (keep the text verbatim).
                    out.push_str(line[hashes + 1..].trim_start());
                    continue;
                }
            }
            out.push_str(line);
        }
        out
    }

    /// Distinct left-edge x-positions of list-marker spans, ascending,
    /// collapsed to ~3pt buckets. A list item's nesting depth (WS2.5) is the
    /// number of these levels strictly to its left. A flat list yields one
    /// level, so every item is depth 0 and its markdown is byte-identical to
    /// the pre-nesting output — inert unless a document genuinely indents.
    fn list_marker_x_levels(spans: &[&OrderedTextSpan]) -> Vec<f32> {
        let mut xs: Vec<f32> = spans
            .iter()
            .filter(|s| {
                let t = s.span.text.trim_start();
                Self::is_bullet_span(&s.span.text)
                    || Self::starts_with_bullet(&s.span.text)
                    || Self::is_ordered_list_marker(t).is_some()
            })
            .map(|s| s.span.bbox.x)
            .collect();
        xs.sort_by(|a, b| crate::utils::safe_float_cmp(*a, *b));
        let mut levels: Vec<f32> = Vec::new();
        for x in xs {
            if levels.last().is_none_or(|&l| x - l > 3.0) {
                levels.push(x);
            }
        }
        levels
    }

    /// Two-space markdown indent for a list marker at `x` (two spaces per level
    /// to its left; empty at the leftmost level).
    fn list_indent(x: f32, levels: &[f32]) -> String {
        "  ".repeat(levels.iter().filter(|&&l| x > l + 3.0).count())
    }

    /// Append a `- ` list marker to `line`, prefixed with the WS2.5 nesting
    /// indent when `line` is empty (i.e. at the start of a fresh list line —
    /// mid-line bullets are left unindented). Flat lists yield an empty indent,
    /// keeping output byte-identical to before.
    fn push_list_marker(line: &mut String, x: f32, levels: &[f32]) {
        if line.is_empty() {
            line.push_str(&Self::list_indent(x, levels));
        }
        line.push_str("- ");
    }

    /// Flush-time trim that preserves a list item's leading indent. Identical
    /// to `str::trim()` for every non-list line, but keeps leading spaces when
    /// the trimmed content is a markdown list marker — so WS2.5 nesting
    /// survives the line flush. Inert on flat lists (their indent is empty).
    fn flush_line(s: &str) -> &str {
        let te = s.trim_end();
        if te.trim_start().starts_with("- ") {
            te
        } else {
            te.trim_start()
        }
    }

    /// Strip the leading bullet character from text, returning the rest.
    fn strip_bullet(text: &str) -> &str {
        let t = text.trim_start();
        // Bullet characters are single Unicode code points; skip first char
        if Self::starts_with_bullet(t) {
            let mut chars = t.chars();
            chars.next(); // skip bullet
            chars.as_str().trim_start()
        } else {
            text
        }
    }

    /// Detect heading level from the span's font size relative to the
    /// document's body size (caller-provided, typically the mode of
    /// observed sizes). Ratios: H1 >=1.8x, H2 >=1.4x, H3 >=1.2x, or
    /// H4 for bold at >=1.05x.
    ///
    /// The bold-threshold tier exists for documents whose section
    /// headings are set in the same family as body text but bumped by
    /// only a few percent of point size — common in corporate manuals
    /// (issue #377 D2: amt_handbook_sample, nougat_032, technical
    /// docs). Without the bold gate this would over-promote
    /// emphasised inline phrases.
    fn heading_level_ratio(&self, span: &OrderedTextSpan, base_font_size: f32) -> Option<u8> {
        if !Self::is_valid_heading_text(span.span.text.trim()) {
            return None;
        }
        if base_font_size <= 0.0 {
            return None;
        }
        let size_ratio = span.span.font_size / base_font_size;
        let is_bold = matches!(
            span.span.font_weight,
            FontWeight::Bold | FontWeight::Black | FontWeight::ExtraBold | FontWeight::SemiBold
        );
        if size_ratio >= 1.8 {
            Some(1)
        } else if size_ratio >= 1.4 {
            Some(2)
        } else if size_ratio >= 1.2 {
            Some(3)
        } else if is_bold && size_ratio >= 1.05 {
            // Bold text with even slight size increase is a heading signal.
            // H4 (was H3) since the weaker signal warrants a lower level.
            Some(4)
        } else {
            None
        }
    }

    /// WS2.4: promote a multi-level numbered section title ("2.1.3 Results")
    /// to a heading whose level is its dot-depth (2.1.3 → H3), capped at H6.
    /// Only DOTTED patterns (≥1 dot) qualify — a bare "1. " leads an ordered
    /// list item, and a bare "2 Foo" is too ambiguous — so this never steals
    /// from the list path. Used only as a FALLBACK when the font-size heuristic
    /// finds no heading, so it is purely additive: it adds section headings a
    /// same-font-size numbered document would otherwise flatten, and never
    /// changes a size-derived level.
    fn numbered_heading_level(text: &str) -> Option<u8> {
        let t = text.trim_start();
        // Leading "N.N(.N)*" then whitespace then a non-digit title word.
        let mut chars = t.char_indices().peekable();
        let mut dots = 0u8;
        let mut saw_digit = false;
        let mut end = 0usize;
        while let Some(&(i, c)) = chars.peek() {
            if c.is_ascii_digit() {
                saw_digit = true;
                end = i + 1;
                chars.next();
            } else if c == '.' {
                // A trailing "1. " (dot then space) is an ordered-list marker,
                // not a section number — require another digit after the dot.
                let after = t[i + 1..].chars().next();
                if after.is_some_and(|n| n.is_ascii_digit()) {
                    dots += 1;
                    end = i + 1;
                    chars.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if !saw_digit || dots == 0 {
            return None;
        }
        // Require a real title after the number (whitespace then a letter).
        let rest = t[end..].trim_start();
        let starts_alpha = rest.chars().next().is_some_and(|c| c.is_alphabetic());
        if !starts_alpha || rest.len() < 2 || rest.len() > 120 {
            return None;
        }
        Some((dots + 1).min(6))
    }

    /// Render a Table as a markdown table string.
    ///
    /// Normalizes column counts so every row has the same number of pipe-delimited
    /// cells. Without this, markdown parsers silently drop trailing cells from
    /// short rows, which causes data loss (e.g. "CERTIFICATE NO.: 403852" missing
    /// from converted output).
    fn render_table_markdown(&self, table: &Table, config: &TextPipelineConfig) -> String {
        if table.rows.is_empty() {
            return String::new();
        }

        let mut output = String::new();

        // Determine header row index - use first row if has_header, or first is_header row
        let header_end = if table.has_header {
            table.rows.iter().position(|r| !r.is_header).unwrap_or(1)
        } else {
            // Treat first row as header for markdown (markdown requires a header row)
            1
        };

        // Find the maximum effective column count across all rows.
        // Each cell contributes `colspan` columns (default 1).
        let max_cols = table
            .rows
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|c| c.colspan.max(1) as usize)
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(0);

        for (row_idx, row) in table.rows.iter().enumerate() {
            output.push('|');
            let mut cols_written: usize = 0;
            for cell in &row.cells {
                output.push(' ');

                // Render bold/italic from span metadata when available;
                // fall back to plain text for cells without span info.
                let cell_text = if !cell.spans.is_empty() {
                    let mut cell_md = String::new();
                    let mut active_bold = false;
                    let mut active_italic = false;

                    // Order per-span emit: close-old-markers → inter-span
                    // space → open-new-markers → text. This keeps whitespace
                    // OUTSIDE emphasis delimiters, which CommonMark requires
                    // (`** text**` and `**text **` are both rejected as
                    // literal asterisks by strict renderers).
                    for (i, span) in cell.spans.iter().enumerate() {
                        // A Markdown header row is already rendered bold by
                        // readers (via the `|---|` separator beneath it), so
                        // explicit `**` in a header cell is redundant and
                        // diverges from the conventional rendering
                        // ("| Region |", not "| **Region** |"). Suppress bold
                        // in the header row ONLY when the table actually has
                        // data rows beneath it — a single-row table is all
                        // "header", so its emphasis is real content and must be
                        // kept. Data cells always keep their bold.
                        let has_data_rows = table.rows.len() > header_end;
                        let is_header = row_idx < header_end && has_data_rows;
                        let is_bold = !is_header && self.is_bold_raw(span, config);
                        let is_italic = span.is_italic;
                        let formatting_changed =
                            is_bold != active_bold || is_italic != active_italic;

                        if formatting_changed {
                            if active_italic {
                                cell_md.push('*');
                            }
                            if active_bold {
                                cell_md.push_str("**");
                            }
                        }

                        if i > 0 {
                            let prev = &cell.spans[i - 1];
                            // Insert an inter-span space when there is a visible
                            // gap OR when the word-boundary detector already
                            // determined this span begins a new word (splitting a
                            // fused run). The cell path previously used the gap
                            // alone and so glued tight-set word pairs like
                            // "Value"+"Aktien" -> "ValueAktien" that the main text
                            // path separates via that same boundary metadata.
                            let has_gap = super::has_horizontal_gap(prev, span)
                                || super::spans_are_stacked(prev, span)
                                || span.split_boundary_before;
                            let already_has_space =
                                cell_md.ends_with(' ') || span.text.starts_with(' ');
                            if has_gap && !already_has_space {
                                cell_md.push(' ');
                            }
                        }

                        if formatting_changed {
                            if is_bold {
                                cell_md.push_str("**");
                            }
                            if is_italic {
                                cell_md.push('*');
                            }
                            active_bold = is_bold;
                            active_italic = is_italic;
                        }

                        // Apply column-spanning-decimal split (issue 487
                        // nougat_018): sailing-score cells emitted as a
                        // single Tj "1.10" with sparse char_widths split
                        // into two tokens "1 10".
                        let mut processed_text = String::new();
                        crate::document::PdfDocument::push_span_text(&mut processed_text, span);
                        let mut text = processed_text.replace('|', "\\|").replace('\n', " ");
                        let just_opened = is_bold || is_italic;
                        if just_opened && (cell_md.ends_with("**") || cell_md.ends_with('*')) {
                            while text.starts_with(' ') {
                                text.remove(0);
                            }
                        }
                        cell_md.push_str(&text);
                    }

                    // Final close: CommonMark forbids whitespace adjacent
                    // to closing markers; strip it before the markers and
                    // re-append after.
                    if active_italic || active_bold {
                        let content_end = cell_md.trim_end().len();
                        let trailing = cell_md[content_end..].to_string();
                        cell_md.truncate(content_end);
                        if active_italic {
                            cell_md.push('*');
                        }
                        if active_bold {
                            cell_md.push_str("**");
                        }
                        cell_md.push_str(&trailing);
                    }

                    cell_md
                } else {
                    cell.text.replace('|', "\\|").replace('\n', " ")
                };

                output.push_str(cell_text.trim());
                output.push(' ');
                // Handle colspan by adding extra | separators
                let span = cell.colspan.max(1) as usize;
                for _ in 1..span {
                    output.push_str("| ");
                }
                output.push('|');
                cols_written += span;
            }
            // Pad short rows with empty cells so every row has `max_cols` columns.
            for _ in cols_written..max_cols {
                output.push_str(" |");
            }
            output.push('\n');

            // Add header separator after header rows
            if row_idx + 1 == header_end {
                output.push('|');
                // Separator must also match max_cols
                let header_cols: usize = row.cells.iter().map(|c| c.colspan.max(1) as usize).sum();
                for _ in 0..max_cols.max(header_cols) {
                    output.push_str("---|");
                }
                output.push('\n');
            }
        }

        output
    }

    /// Resolve bold emphasis for a raw TextSpan honoring config.
    fn is_bold_raw(&self, span: &crate::layout::TextSpan, config: &TextPipelineConfig) -> bool {
        use crate::pipeline::config::BoldMarkerBehavior;
        match span.font_weight {
            FontWeight::Bold | FontWeight::Black | FontWeight::ExtraBold | FontWeight::SemiBold => {
                match config.output.bold_marker_behavior {
                    BoldMarkerBehavior::Aggressive => true,
                    BoldMarkerBehavior::Conservative => {
                        span.text.chars().any(|c| !c.is_whitespace())
                    },
                }
            },
            _ => false,
        }
    }

    /// Core rendering logic shared between convert() and convert_with_tables().
    fn render_spans(
        &self,
        spans: &[OrderedTextSpan],
        tables: &[Table],
        config: &TextPipelineConfig,
    ) -> Result<String> {
        if spans.is_empty() && tables.is_empty() {
            return Ok(String::new());
        }

        // Sort by reading order
        let mut sorted: Vec<_> = spans.iter().collect();
        sorted.sort_by_key(|s| s.reading_order);

        // Body-font size for the heading-ratio reference. Span-count
        // mode bucketed to 0.5pt, with smaller-bucket tiebreak so body
        // text wins over headings when counts are close. Capped at 12pt
        // so that heading-only documents still produce sensible ratios.
        let base_font_size = super::base_heading_font_size(&sorted, config.output.detect_headings);

        // Footnote detection (WS2.7). Conservative, high-precision: an
        // inline superscript marker is only rewritten to `[^N]` when a
        // matching page-bottom definition block starting with the SAME
        // token is also present, and vice-versa. When either half is
        // missing the plan is empty and rendering is byte-identical to
        // before. See `detect_footnotes` for the heuristics.
        let footnote_plan = detect_footnotes(&sorted, base_font_size);

        // Per-block margins: the rightmost edge and the leftmost edge any span
        // sharing a paragraph `block_id` reaches. A *wrapped* prose line runs to
        // the column margin (the right edge for left-to-right text, the LEFT
        // edge for right-to-left text); a deliberately short line (a shell
        // command, a record row, a paragraph's last line) stops well short of
        // it. Used to gate the structure-authoritative paragraph reflow below.
        let mut block_right_max: std::collections::HashMap<u32, f32> =
            std::collections::HashMap::new();
        let mut block_left_min: std::collections::HashMap<u32, f32> =
            std::collections::HashMap::new();
        for s in &sorted {
            if let Some(b) = s.block_id {
                let right = s.span.bbox.x + s.span.bbox.width;
                let e = block_right_max.entry(b).or_insert(f32::MIN);
                if right > *e {
                    *e = right;
                }
                let l = block_left_min.entry(b).or_insert(f32::MAX);
                if s.span.bbox.x < *l {
                    *l = s.span.bbox.x;
                }
            }
        }

        // Track which tables have been rendered
        let mut tables_rendered = vec![false; tables.len()];
        // Pre-render table markdown so we can check for orphaned spans.
        let table_mds: Vec<String> = tables
            .iter()
            .map(|t| self.render_table_markdown(t, config))
            .collect();
        // Collect spans skipped because they fall inside a table region.
        let mut table_skipped_spans: Vec<Vec<&OrderedTextSpan>> = vec![Vec::new(); tables.len()];

        let mut result = String::new();
        let mut prev_span: Option<&OrderedTextSpan> = None;
        let mut current_line = String::new();
        // Track open inline formatting to consolidate adjacent bold/italic spans.
        // When consecutive same-line spans share the same bold or italic style,
        // we keep the markers open and only close them when the style changes or
        // the line is flushed, producing e.g. **ACME GLOBAL LTD.** instead
        // of **ACME** **GLOBAL** **LTD.**.
        let mut active_bold = false;
        let mut active_italic = false;
        let mut current_heading_level: Option<u8> = None;
        // Whether every non-blank span accumulated into `current_line` so far is
        // monospace (a fixed-pitch / code font). A plain paragraph that is all
        // monospace is a code line; consecutive ones are fused into a fenced
        // block by `fence_monospace_blocks` after rendering.
        let mut current_line_all_mono = true;

        /// Close any open bold/italic markers on `line`.
        ///
        /// CommonMark forbids whitespace adjacent to closing emphasis markers
        /// (e.g. `**bold **` is rendered as literal asterisks). Strip trailing
        /// whitespace before closing, then restore it after the markers.
        fn close_formatting(line: &mut String, bold: &mut bool, italic: &mut bool) {
            if !*bold && !*italic {
                return;
            }
            let content_end = line.trim_end().len();
            let trailing_ws = line[content_end..].to_string();
            line.truncate(content_end);
            // Close in reverse order of opening: italic first, then bold.
            if *italic {
                line.push('*');
                *italic = false;
            }
            if *bold {
                line.push_str("**");
                *bold = false;
            }
            line.push_str(&trailing_ws);
        }

        // Strip markdown emphasis markers (**bold**, *italic*) from a line.
        // Used when emitting heading lines, where the `#` prefix already
        // provides emphasis and nested markers (e.g. `# **Title**`) are
        // redundant and can confuse strict CommonMark renderers.
        fn strip_emphasis(s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            let chars: Vec<char> = s.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '*' {
                    // Skip one or two asterisks
                    i += 1;
                    if i < chars.len() && chars[i] == '*' {
                        i += 1;
                    }
                    continue;
                }
                out.push(chars[i]);
                i += 1;
            }
            out
        }

        // WS2.5: left-edge levels for list-item nesting depth (inert on flat lists).
        let list_x_levels = Self::list_marker_x_levels(&sorted);

        for (idx, span) in sorted.iter().enumerate() {
            // Skip artifacts (pagination, headers, footers) unless the
            // caller opted in via `config.output.include_artifacts`
            // (default true).
            if !config.output.include_artifacts && span.span.artifact_type.is_some() {
                continue;
            }

            // Suppress spans that make up a confirmed page-bottom footnote
            // definition block: they are re-emitted as `[^N]: …` lines at
            // the end of the document instead of appearing inline as body
            // text. (Empty set when no footnotes were confirmed.)
            if footnote_plan.definition_spans.contains(&idx) {
                continue;
            }

            // Whether this span is a confirmed inline footnote marker
            // (a raised superscript token with a matching bottom def).
            let footnote_marker_label = footnote_plan.inline_markers.get(&idx);

            // Skip "noise" spans: isolated single-character fragments that
            // are purely punctuation/symbol (e.g. a bare "|" or "—" on its
            // own baseline from a decorative PDF separator). These add no
            // semantic value but pollute output as lone-line paragraphs.
            // Bullet characters are excluded from this filter since they
            // are meaningful list markers handled downstream.
            //
            // EXCEPTION: a space-bearing punctuation span (" ," / " .") that is
            // FLANKED by right-to-left text on its own baseline is a
            // producer-drawn inter-word separator, not decoration. RTL scripts
            // emit the comma/period between two words as its own span, so
            // dropping it glues the words ("הטורפים, ממשפחת" → "הטורפיםממשפחת").
            // The RTL-context gate is deliberate: a lone decorative mark, or an
            // isolated punctuation fragment in a left-to-right data region,
            // has no RTL neighbour on its line and is still dropped (keeping it
            // there would fragment the line into one paragraph per mark).
            {
                let t = span.span.text.trim();
                let char_count = t.chars().count();
                if char_count > 0
                    && char_count <= 2
                    && !t.chars().any(|c| c.is_alphanumeric())
                    && !Self::is_bullet_span(t)
                    && !Self::starts_with_bullet(t)
                    && footnote_marker_label.is_none()
                {
                    let carries_space = span.span.text.chars().any(|c| c.is_whitespace());
                    let rtl_separator = carries_space && {
                        let y = span.span.bbox.y;
                        let tol = span.span.font_size.max(1.0) * 1.5;
                        sorted.iter().enumerate().any(|(j, other)| {
                            j != idx
                                && (other.span.bbox.y - y).abs() <= tol
                                && crate::text::bidi::looks_rtl(&other.span.text)
                        })
                    };
                    if !rtl_separator {
                        continue;
                    }
                }
            }

            // Check if this span belongs to a table region
            if !tables.is_empty() {
                if let Some(table_idx) = super::span_in_table(span, tables) {
                    if !tables_rendered[table_idx] {
                        // Flush current line
                        close_formatting(&mut current_line, &mut active_bold, &mut active_italic);
                        if !current_line.is_empty() {
                            push_plain_paragraph(
                                &mut result,
                                current_line.trim(),
                                current_line_all_mono,
                            );
                            current_line.clear();
                        }

                        // Render the table
                        result.push_str(&table_mds[table_idx]);
                        result.push('\n');
                        tables_rendered[table_idx] = true;
                        prev_span = None;
                    }
                    // Track span for orphan recovery
                    table_skipped_spans[table_idx].push(span);
                    // Skip this span (it's part of a table)
                    continue;
                }
            }

            // Heading level: structure-tree role takes precedence over
            // font-size heuristics when the source PDF is tagged. This
            // is the issue #377 D1 unlock — Word/Acrobat tagged PDFs
            // that set body and heading text in the same point size
            // would otherwise lose all heading hierarchy.
            let span_heading_level = match span.struct_role {
                Some(StructRole::Heading(level)) => Some(level.clamp(1, 6)),
                _ if config.output.detect_headings => {
                    // Font-size heuristic first; fall back to numbered-section
                    // promotion (WS2.4) so same-point-size numbered headings
                    // ("2.1 Method") are still recovered.
                    self.heading_level_ratio(span, base_font_size).or_else(|| {
                        if Self::is_valid_heading_text(span.span.text.trim()) {
                            Self::numbered_heading_level(&span.span.text)
                        } else {
                            None
                        }
                    })
                },
                _ => None,
            };

            // List-item role from the structure tree. When set, we emit
            // a markdown `- ` bullet at the start of the line for this
            // span (mirroring `is_bullet_span`/`starts_with_bullet`
            // detection used for untagged docs).
            let is_list_item_role = matches!(
                span.struct_role,
                Some(StructRole::ListItemBody)
                    | Some(StructRole::ListItem)
                    | Some(StructRole::ListItemLabel)
            );

            // Whether this span's own text already carries an ordered marker
            // (`1.`, `a)`). Ordered items must not also get a `- ` prefix.
            let span_is_ordered =
                Self::is_ordered_list_marker(span.span.text.trim_start()).is_some();

            // Force-flush an open heading before a list item begins, so a bullet
            // or ordered marker never glues onto the heading line.
            let span_starts_list = is_list_item_role
                || Self::is_bullet_span(&span.span.text)
                || Self::starts_with_bullet(&span.span.text)
                || span_is_ordered;
            if span_starts_list && !current_line.trim().is_empty() {
                if let Some(level) = current_heading_level {
                    close_formatting(&mut current_line, &mut active_bold, &mut active_italic);
                    begin_block(&mut result);
                    let prefix = "#".repeat(level as usize);
                    result.push_str(&format!(
                        "{} {}\n\n",
                        prefix,
                        strip_emphasis(current_line.trim())
                    ));
                    current_line.clear();
                    current_heading_level = None;
                }
            }

            // Check for paragraph break or line break
            let mut same_line = prev_span
                .map(|prev| (span.span.bbox.y - prev.span.bbox.y).abs() < span.span.font_size * 0.5)
                .unwrap_or(true);

            // Close a soft-hyphen wrap, using the same geometry the text
            // assembler uses. §14.8.2.2.3 makes U+00AD a break offered *inside*
            // a word, and §9.4.2 leaves the glyph positions as the only evidence
            // of where the line ended — evidence that is gone once the markdown
            // is a string, which is why the downstream pass cannot decide this.
            //
            // Two signals must coincide: the baseline drops about one line, and
            // the continuation returns left of where the previous run ended.
            // Over the corpus that accepts genuine wraps and rejects the false
            // joins a re-ordered scan offers, whose baseline "drop" is band
            // jitter rather than a line advance.
            let mut wrap_closed = false;
            if let Some(prev) = prev_span {
                let em = prev.span.font_size.max(span.span.font_size).max(6.0);
                let drop = prev.span.bbox.y - span.span.bbox.y;
                let seam_gap = span.span.bbox.x - (prev.span.bbox.x + prev.span.bbox.width);
                if current_line
                    .strip_suffix('\u{00AD}')
                    .is_some_and(|t| t.ends_with(char::is_alphabetic))
                    && span.span.text.starts_with(char::is_alphabetic)
                    && drop >= em * 0.6
                    && drop <= em * 1.6
                    && seam_gap < -em
                {
                    current_line.pop();
                    same_line = true;
                    wrap_closed = true;
                }
            }

            if let Some(prev) = prev_span {
                // Group boundary: when group_id changes, insert a paragraph break
                // to keep spatially partitioned regions (e.g. columns) contiguous.
                let group_changed = match (span.group_id, prev.group_id) {
                    (Some(a), Some(b)) => a != b,
                    _ => false,
                };

                let heading_changed = current_heading_level != span_heading_level;

                // A reading-order group change only forces a paragraph break
                // when the visual line also changes — this keeps horizontally
                // split elements (e.g. multi-span footer lines) together.
                let group_flush = group_changed && !same_line;

                let prev_was_list_item = matches!(
                    prev.struct_role,
                    Some(StructRole::ListItemBody)
                        | Some(StructRole::ListItem)
                        | Some(StructRole::ListItemLabel)
                );
                let list_item_changed = is_list_item_role != prev_was_list_item;

                // Tagged-PDF block boundary (issue #377 D5): adjacent
                // spans whose nearest paragraph-level structure ancestor
                // differs are explicitly separate paragraphs even when
                // the geometric gap is small (pdfa_049 has body-tight
                // inter-paragraph gaps that the gap heuristic never
                // catches).
                //
                // D5b refinement: gate this on `!same_line` so a tagged
                // form whose horizontal heading band is split into
                // multiple /P sub-elements on one line (irs_f1040 has
                // `Form` + `1040` + `U.S. Individual Income Tax Return`
                // as three sibling /P blocks at the same y) does not
                // become three separate `# Form` / `# 1040` / ... lines.
                //
                // D5c refinement: when same_line is true but a
                // multi-column gutter separates the spans (large
                // horizontal gap), restore the break. Newspapers
                // (IA_0047) and other multi-column tagged docs
                // otherwise produce concatenated tokens like
                // `andmight` from adjacent-column content sharing a
                // baseline.
                let column_gap = is_column_gap(prev, span);
                let line_truly_continuous = same_line && !column_gap;
                let block_changed = match (span.block_id, prev.block_id) {
                    (Some(a), Some(b)) => a != b,
                    _ => false,
                } && !line_truly_continuous;

                // For heading transitions: same logic — visual line
                // continuity wins over structure-tree fragmentation.
                // For list-item transitions: ALWAYS break because a
                // bullet `- ` needs its own markdown line regardless
                // of whether the source PDF rendered the marker
                // inline with a leading caption.
                let heading_changed_break = heading_changed && !line_truly_continuous;

                // Structure-authoritative paragraph reflow (ISO 32000-1 §14.8.3:
                // one `<P>` BLSE is a single paragraph that "can be split
                // between lines of text"). When two spans share a paragraph
                // block, the *geometric* gap heuristic must not split a wrapped
                // line — but only when this is genuinely a mid-sentence wrap.
                // The continuation signals (cheap, language-neutral) keep
                // intentional same-block line breaks intact: form fields and
                // record rows start with a capitalised label, code is tagged
                // preformatted, and a sentence end is a hard boundary.
                let same_block = matches!(
                    (span.block_id, prev.block_id),
                    (Some(a), Some(b)) if a == b
                );
                let plain_para = current_heading_level.is_none()
                    && span_heading_level.is_none()
                    && !is_list_item_role
                    && !prev_was_list_item
                    && !span.preformatted
                    && !prev.preformatted;
                // The next line continues the sentence when it starts lowercase
                // (`…downtown by\nnine…`), with a caseless-script letter
                // (Hebrew/Arabic/CJK/Indic have no capitalisation, so a letter
                // at the line start is always a continuation, never a new
                // sentence/field), or with a number that is not an ordered-list
                // marker (`…downtown by\n2028`). An ordered marker (`2.` / `3)`)
                // is a deliberate new item and must not merge.
                let next_trim = span.span.text.trim_start();
                let next_continues_lowercase = next_trim.chars().next().is_some_and(|c| {
                    c.is_lowercase()
                        || (c.is_alphabetic() && !c.is_uppercase())
                        || (c.is_ascii_digit()
                            && super::is_ordered_list_marker(next_trim).is_none())
                });
                let prev_trimmed = current_line.trim_end();
                let prev_unterminated = prev_trimmed
                    .chars()
                    .last()
                    .is_none_or(|c| !matches!(c, '.' | '!' | '?' | ':' | ';'));
                // A line broken mid-word at a hyphen (`…three sub-\nNeptune…`)
                // is always a wrap, even when the continuation is capitalised (a
                // proper noun) — the trailing hyphen is the continuation signal.
                let next_continues_lowercase =
                    next_continues_lowercase || prev_trimmed.ends_with('-');
                // The previous line must have run to the column margin — a
                // genuine wrap. A short ragged line (shell command, mis-tagged
                // code, a record value) is a deliberate break and is preserved.
                // Left-to-right lines fill toward the RIGHT margin; right-to-left
                // lines (Arabic/Hebrew) start at the right and fill toward the
                // LEFT margin, so the test is mirrored for them.
                let tol = prev.span.font_size.max(8.0) * 1.5;
                let prev_is_rtl = crate::text::bidi::looks_rtl(&prev.span.text);
                let prev_fills_column = prev.block_id.is_some_and(|b| {
                    if prev_is_rtl {
                        block_left_min
                            .get(&b)
                            .is_some_and(|&lo| prev.span.bbox.x <= lo + tol)
                    } else {
                        block_right_max
                            .get(&b)
                            .is_some_and(|&hi| prev.span.bbox.x + prev.span.bbox.width >= hi - tol)
                    }
                });
                let merge_wrapped_line = same_block
                    && plain_para
                    && next_continues_lowercase
                    && prev_unterminated
                    && prev_fills_column;
                // A genuine intra-paragraph wrap (same block, plain body text,
                // previous line filled the column margin, next line continues)
                // is one paragraph, so it suppresses every *soft* break — the
                // geometric gap, a reading-order group change, and a structure
                // block change (the same `<P>` re-entered). It does NOT override
                // a list-item transition or a real column gutter.
                let soft_break = group_flush
                    || self.is_paragraph_break(span, prev)
                    || heading_changed_break
                    || block_changed;
                if (soft_break && !merge_wrapped_line) || list_item_changed || column_gap {
                    close_formatting(&mut current_line, &mut active_bold, &mut active_italic);
                    if !current_line.is_empty() {
                        if let Some(level) = current_heading_level {
                            begin_block(&mut result);
                            let prefix = "#".repeat(level as usize);
                            result.push_str(&format!(
                                "{} {}\n\n",
                                prefix,
                                strip_emphasis(current_line.trim())
                            ));
                        } else {
                            push_plain_paragraph(
                                &mut result,
                                current_line.trim(),
                                current_line_all_mono,
                            );
                        }
                        current_line.clear();
                    }
                    current_heading_level = span_heading_level;
                    if is_list_item_role && !span_is_ordered {
                        Self::push_list_marker(&mut current_line, span.span.bbox.x, &list_x_levels);
                    }
                } else if !same_line {
                    // Different visual line but within paragraph spacing.
                    // Check if a bullet or ordered-marker item starts here
                    // — if so, start a new line. Issue #377 D3 guards
                    // numbered lists (`1. Foo` / `2. Bar` / `3. Baz`) at
                    // the same X across consecutive baselines: the items
                    // must not concatenate into one line of running text.
                    //
                    // Only fire on a list-item *transition* (the body of
                    // a wrapped LI keeps the same role across visual
                    // lines and must NOT emit a fresh bullet on each
                    // wrapped line).
                    let is_bullet = Self::is_bullet_span(&span.span.text)
                        || Self::starts_with_bullet(&span.span.text);
                    let is_ordered =
                        Self::is_ordered_list_marker(span.span.text.trim_start()).is_some();
                    // Tagged docs: each /LI gets its own `block_id`,
                    // so wrapped multi-line items share the same id
                    // and we should only fire on a TRANSITION
                    // (different block_id or list_item_changed).
                    // Untagged docs (block_id None on both): can't
                    // tell which body lines are wrapped vs which are
                    // new items, so fall back to "any list-role on a
                    // new baseline starts a new item".
                    let starts_new_list_item = if span.block_id.is_some() && prev.block_id.is_some()
                    {
                        is_list_item_role && (list_item_changed || block_changed)
                    } else {
                        is_list_item_role
                    };
                    if is_bullet || is_ordered || starts_new_list_item {
                        // Bullet on new line → flush current line and start list item
                        close_formatting(&mut current_line, &mut active_bold, &mut active_italic);
                        if !current_line.is_empty() {
                            if let Some(level) = current_heading_level {
                                begin_block(&mut result);
                                let prefix = "#".repeat(level as usize);
                                result.push_str(&format!(
                                    "{} {}\n\n",
                                    prefix,
                                    strip_emphasis(current_line.trim())
                                ));
                            } else {
                                result.push_str(Self::flush_line(&current_line));
                                result.push('\n');
                            }
                            current_line.clear();
                        }
                        current_heading_level = span_heading_level;
                        if starts_new_list_item && !span_is_ordered {
                            Self::push_list_marker(
                                &mut current_line,
                                span.span.bbox.x,
                                &list_x_levels,
                            );
                        }
                    } else {
                        // Different visual line within the same paragraph — close
                        // open formatting before the line-join space so that
                        // formatting is re-evaluated for the new line's spans.
                        close_formatting(&mut current_line, &mut active_bold, &mut active_italic);
                        if config.output.preserve_layout {
                            let spacing = (span.span.bbox.x - prev.span.bbox.x).max(0.0) as usize;
                            for _ in 0..spacing.min(20) {
                                current_line.push(' ');
                            }
                        } else if !current_line.trim_end().ends_with('-') {
                            // A line wrapped mid-word at a hyphen (`frozen-` /
                            // `thawed`) joins WITHOUT a space — `frozen- thawed`
                            // (a space after the hyphen) is never correct. Whether
                            // to also DROP the hyphen is genuinely ambiguous (a real
                            // compound `frozen-thawed` keeps it), so leave the hyphen
                            // and only suppress the spurious space.
                            current_line.push(' ');
                        }
                    }
                }
            } else {
                current_heading_level = span_heading_level;
                if is_list_item_role && !span_is_ordered {
                    Self::push_list_marker(&mut current_line, span.span.bbox.x, &list_x_levels);
                }
            }

            // Standalone bullet-glyph span → markdown list marker.
            if Self::is_bullet_span(&span.span.text) {
                if !current_line.ends_with("- ") {
                    if !current_line.is_empty() && !current_line.ends_with(' ') {
                        current_line.push(' ');
                    }
                    Self::push_list_marker(&mut current_line, span.span.bbox.x, &list_x_levels);
                }
                prev_span = Some(span);
                continue;
            }

            // Apply column-spanning-decimal / char_widths-boundary split
            // (issue 487 nougat_018).  Mirrors `push_span_text` in the text
            // extractor so sailing-score cells like "1.10" (sparse cw,
            // really `1` + `10` in adjacent columns) split into two tokens
            // for markdown output too.
            let mut text_str = String::new();
            crate::document::PdfDocument::push_span_text(&mut text_str, &span.span);

            // Confirmed inline footnote marker: replace the raw superscript
            // token (e.g. "1" / "*") with markdown footnote-reference
            // syntax `[^N]`. It then flows through the normal same-line
            // append path and glues onto the preceding body text.
            if let Some(label) = footnote_marker_label {
                text_str = label.clone();
            }

            // Normalize known mis-extracted bullet glyphs (DEL from Zapf
            // Dingbats mappings, ❍ from ligature remaps) to U+2022 so the
            // bullet-span logic above can recognize them uniformly.
            //
            // POSITION-AWARE (issue #13 / user-content-preservation
            // principle): only replace the FIRST occurrence when it
            // sits at the very start of the span (a bullet position).
            // Mid-prose `❍` / DEL must survive verbatim — if the
            // source PDF actually contains those codepoints in body
            // text, rewriting them is content corruption. Bullet
            // detection at line start is intact; arbitrary text-stream
            // codepoints are no longer mutated.
            let trim_start = text_str.trim_start();
            if let Some(first) = trim_start.chars().next() {
                if first == '\x7f' || first == '❍' {
                    let leading_ws_len = text_str.len() - trim_start.len();
                    // Replace just this leading char, leave any later
                    // occurrences inside the same span verbatim.
                    let bullet_byte_len = first.len_utf8();
                    text_str = format!(
                        "{}•{}",
                        &text_str[..leading_ws_len],
                        &text_str[leading_ws_len + bullet_byte_len..]
                    );
                }
            }

            // Pipe characters are only markdown-syntactic inside table
            // cells; in paragraph flow they are just text. Pipe escaping
            // for tables is handled in render_table_markdown. Leaving `|`
            // alone in flow avoids showing `&#124;` in user-visible prose.

            let mut text = text_str.as_str();

            // Handle inline bullets (text starts with bullet char)
            if Self::starts_with_bullet(text) {
                let stripped = Self::strip_bullet(text);
                if !current_line.ends_with("- ") {
                    if !current_line.is_empty() && !current_line.ends_with(' ') {
                        current_line.push(' ');
                    }
                    Self::push_list_marker(&mut current_line, span.span.bbox.x, &list_x_levels);
                }
                text = stripped;
            }

            let normalized;
            if !config.output.preserve_layout {
                // In PDFs, adjacent spans on the same line often have slightly
                // overlapping bboxes (negative horizontal gap) with the inter-span
                // whitespace encoded as leading/trailing spaces in the span text
                // itself.  normalize_whitespace collapses internal runs of spaces
                // but would also strip these boundary spaces, causing words from
                // neighbouring spans to merge (e.g. "visitwww.example.comto").
                // Preserve a leading space when a same-line predecessor exists and
                // a trailing space unconditionally so the next span can abut
                // correctly.  The plain-text converter avoids this problem by
                // skipping per-span normalization entirely.
                let had_leading_space =
                    same_line && prev_span.is_some() && text.starts_with(char::is_whitespace);
                let had_trailing_space = text.ends_with(char::is_whitespace);
                let mut norm = self.normalize_whitespace(text);
                if had_leading_space && !norm.starts_with(' ') {
                    norm.insert(0, ' ');
                }
                if had_trailing_space && !norm.ends_with(' ') && !norm.is_empty() {
                    norm.push(' ');
                }
                normalized = norm;
                text = &normalized;
            }

            // A span inside a /Link annotation becomes a markdown link
            // `[text](uri)` — but only for safe schemes (reject javascript:,
            // data:, … to avoid injecting active content). Otherwise fall back
            // to autolinking bare URLs in the text.
            let linkified = match span.link_uri.as_deref() {
                Some(uri) if super::is_safe_link_uri(uri) => {
                    format!("[{}]({})", text.trim(), uri.replace(' ', "%20").replace(')', "%29"))
                },
                _ => self.linkify(text),
            };

            let is_bold = self.is_bold(span, config);
            let is_italic = self.is_italic(span);

            // Issue #260: Detect horizontal gaps between same-line spans and
            // insert a space.  PDFs generated by PDFKit.NET (and similar) place
            // each word in its own BT/ET block with absolute positioning.  The
            // spans carry no leading/trailing whitespace so the PR #273
            // whitespace-preservation logic above cannot help.  We replicate the
            // same gap heuristic used by extract_text()'s should_insert_space():
            // gap > 15% of font size → space, but not if > 5× font size (column
            // boundary).
            if same_line && !current_line.is_empty() {
                if let Some(prev) = prev_span {
                    let no_existing_ws =
                        !current_line.ends_with(' ') && !linkified.starts_with(' ');
                    // Visual gap heuristic (issue #260).
                    let visual_gap = super::has_horizontal_gap(&prev.span, &span.span);
                    // Punctuation/case heuristic: when prev ends in a sentence
                    // boundary (`.`, `,`, `;`, `:`, `?`, `!`) and the next span
                    // begins with an uppercase letter or digit, it's overwhelmingly
                    // likely a missing space — even if the bbox gap is below the
                    // visual threshold (tightly typeset academic PDFs are common
                    // offenders, producing text like "methods.The financial...").
                    let punct_boundary = current_line
                        .chars()
                        .last()
                        .is_some_and(|c| matches!(c, '.' | ',' | ';' | ':' | '?' | '!'))
                        && linkified
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
                    // A closed wrap joins its halves directly — the marker was
                    // the hyphenation point, not a word boundary.
                    // A footnote marker sits at the base word's advance edge,
                    // so `has_horizontal_gap` declines it; the base being a
                    // prose word rather than a subscript host is what separates
                    // it from `H2O`.
                    let marker_boundary =
                        super::is_reference_marker_boundary(&prev.span, &span.span);
                    if !wrap_closed
                        && no_existing_ws
                        && (visual_gap || punct_boundary || marker_boundary)
                    {
                        current_line.push(' ');
                    }
                }
            }

            // Consolidate adjacent spans with the same formatting style into
            // a single bold/italic block instead of wrapping each span
            // individually (e.g. **ACME GLOBAL LTD.** not
            // **ACME** **GLOBAL** **LTD.**).
            //
            // When the formatting changes we close the old markers and open
            // new ones.  When it stays the same we just append the text.
            if is_bold != active_bold || is_italic != active_italic {
                // Close previous formatting markers (if any)
                close_formatting(&mut current_line, &mut active_bold, &mut active_italic);
                // Open new markers
                if is_bold {
                    current_line.push_str("**");
                    active_bold = true;
                }
                if is_italic {
                    current_line.push('*');
                    active_italic = true;
                }
            }

            // Track whether the line being built is entirely monospace. At this
            // point `current_line` reflects the post-flush state (a break above
            // cleared it), so an empty / bullet-only line re-arms the flag before
            // this span's content lands; each non-blank span then narrows it.
            if current_line.trim_start_matches("- ").trim().is_empty() {
                current_line_all_mono = true;
            }
            if span.span.text.chars().any(|c| !c.is_whitespace()) {
                current_line_all_mono &= span.span.is_monospace;
            }

            current_line.push_str(&linkified);

            prev_span = Some(span);
        }

        // Close any open formatting before final flushes
        close_formatting(&mut current_line, &mut active_bold, &mut active_italic);

        // Recover orphaned spans: spans inside a table region whose text does
        // not appear in the rendered table output.
        for (table_idx, skipped) in table_skipped_spans.iter().enumerate() {
            if !tables_rendered[table_idx] || skipped.is_empty() {
                continue;
            }
            let rendered = &table_mds[table_idx];
            // Compare on the glyph sequence, not on the spacing. The cell
            // builder joins its member spans with a space while the flow
            // assembler joins the same glyphs with none, so a span the table
            // already renders can fail a literal substring test purely on
            // whitespace and be re-emitted beside the table. Whitespace is a
            // rendering choice of each side; the glyphs are the content. The
            // row's `|` delimiters are not whitespace and so survive the
            // squash, which keeps a span that straddles two cells from
            // matching the concatenation of their texts.
            let rendered_glyphs = squash_whitespace(rendered);
            let mut orphans: Vec<&&OrderedTextSpan> = skipped
                .iter()
                .filter(|s| {
                    let trimmed = s.span.text.trim();
                    if trimmed.is_empty() {
                        return false;
                    }
                    let glyphs = squash_whitespace(trimmed);
                    !glyphs.is_empty() && !rendered_glyphs.contains(&glyphs)
                })
                .collect();
            if !orphans.is_empty() {
                orphans.sort_by_key(|s| s.reading_order);
                for orphan in orphans {
                    if !result.ends_with(' ') && !result.ends_with('\n') {
                        result.push(' ');
                    }
                    // Apply column-spanning-decimal / char_widths-boundary
                    // split (issue 487 nougat_018): orphan score spans
                    // emitted as "25.10" with sparse cw split into "25 10".
                    let mut processed = String::new();
                    crate::document::PdfDocument::push_span_text(&mut processed, &orphan.span);
                    result.push_str(&processed);
                }
            }
        }

        // Render any tables that weren't matched to spans (e.g., all spans were in tables)
        for (i, table) in tables.iter().enumerate() {
            if !tables_rendered[i] && !table.is_empty() {
                if !current_line.is_empty() {
                    if let Some(level) = current_heading_level {
                        begin_block(&mut result);
                        let prefix = "#".repeat(level as usize);
                        result.push_str(&format!(
                            "{} {}\n\n",
                            prefix,
                            strip_emphasis(current_line.trim())
                        ));
                    } else {
                        push_plain_paragraph(
                            &mut result,
                            current_line.trim(),
                            current_line_all_mono,
                        );
                    }
                    current_line.clear();
                }
                result.push_str(&table_mds[i]);
                result.push('\n');
            }
        }

        // Flush remaining content
        if !current_line.is_empty() {
            if let Some(level) = current_heading_level {
                begin_block(&mut result);
                let prefix = "#".repeat(level as usize);
                result.push_str(&format!("{} {}\n", prefix, strip_emphasis(current_line.trim())));
            } else if current_line_all_mono {
                // Trailing monospace paragraph — sentinel-wrap so the fence pass
                // fuses it with any preceding code lines.
                result.push(MONO_SENTINEL);
                result.push_str(Self::flush_line(&current_line));
                result.push(MONO_SENTINEL);
                result.push('\n');
            } else {
                result.push_str(Self::flush_line(&current_line));
                result.push('\n');
            }
        }

        // Final whitespace normalization
        let mut final_result = if config.output.preserve_layout {
            result
        } else {
            let cleaned = result
                .split("\n\n")
                .map(|para| para.trim())
                .filter(|para| !para.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");

            if result.ends_with('\n') && !cleaned.ends_with('\n') {
                format!("{}\n", cleaned)
            } else {
                cleaned
            }
        };

        // Merge key-value pairs that were split across lines due to column-based
        // reading order (e.g. "Grand Total\n$750.00" → "Grand Total $750.00").
        final_result = super::merge_key_value_pairs(&final_result);

        // Band-aid post-processing for known extraction-quality issues
        // reported against v0.3.51/v0.3.52 markdown output. The deeper
        // fixes (root-cause changes to the spatial-table detector,
        // heading-fragmentation prevention upstream, font-CMap recovery)
        // happen on follow-up branches; these post-process steps remove
        // the most damaging surface symptoms so downstream consumers
        // (LLM ingestion, RAG pipelines) get usable text now.
        //
        // Step order is deliberate:
        //   1. Pipe escape — clean up stray pipes BEFORE table-block
        //      detection runs again in subsequent steps.
        //   2. Degenerate-table simplification (#3, #6, partial #11).
        //   3. Heading merge (#1, #4) — only after degenerate tables
        //      have been collapsed so leftover heading fragments are
        //      contiguous and visible to the merger.
        //   4. Page-number filter (#9).
        //   5. Bullet glyph normalization (#13).
        //
        // SPEC-ALIGNMENT GATE (ISO 32000-1:2008 §14.8.4). When the
        // document carries an explicit structure tree — any span has a
        // resolved `struct_role` — the heading levels, table cells, and
        // block boundaries are AUTHORITATIVE per the spec
        // (§14.8.4.3.2: each H/H1-H6 is a distinct heading element).
        // In that case we must NOT apply the layout-recovery heuristics
        // that guess at structure, because they could override correct,
        // author-specified tagging (e.g. fuse three legitimately-
        // distinct H1 sections). The heuristic structure recovery is
        // ONLY valid for UNTAGGED documents, where the markdown
        // structure was itself derived heuristically (font-size ratios,
        // spatial grouping) and is therefore fair game to refine.
        let is_tagged = sorted.iter().any(|s| s.struct_role.is_some());

        // Always-safe steps (no semantic structure change): markdown
        // escaping, whitespace-only bold-fragment recovery, and
        // exact-duplicate paragraph dedup. These run for both tagged
        // and untagged documents.
        final_result = escape_stray_leading_pipes(&final_result);
        final_result = coalesce_camelcase_bold_fragments(&final_result);
        // Fuse consecutive monospace (fixed-pitch / code-font) paragraphs into a
        // fenced code block. A `Code` element renders its lines monospace even
        // when the producer left the block untagged.
        final_result = fence_monospace_blocks(&final_result);
        // Tight lists: drop blank lines between consecutive list-item markers.
        // The span flush always appends a blank line, which turns every list
        // into a Markdown "loose" list (each item wrapped in <p>); the golden
        // corpus and most renderers expect a "tight" list. CommonMark §5.3.
        final_result = tighten_list_items(&final_result);

        // Structure-recovery heuristics — UNTAGGED documents only.
        // For tagged PDFs the structure tree is authoritative (§14.8.4)
        // so these are skipped.
        if !is_tagged {
            final_result = collapse_numeric_heading_runs(&final_result);
            final_result = merge_consecutive_same_level_headings(&final_result);
            // Line-level heading re-validation: the level is decided per span,
            // before the line's spans are concatenated, so a short promoted span
            // can accrete a body continuation into a long, sentence-like line.
            // Re-apply the heading gate to each COMPLETE line and demote body
            // text back to a paragraph (marker-style block-level classification).
            // Tagged PDFs are skipped — their `/H` tags are authoritative.
            final_result = Self::demote_body_like_headings(&final_result);
        }
        // INTENTIONALLY NOT INVOKED — these would damage legitimate
        // content and were removed after a 70-PDF baseline-vs-HEAD
        // regression sweep proved real-world breakage:
        //
        //  * simplify_degenerate_tables — flattened a REAL country-
        //    data table (google_doc_document.pdf: countries × Continent
        //    / Capital / Currency / Population) into one prose line,
        //    because legitimate tables can be mostly single-word. A
        //    markdown-layer heuristic cannot reliably tell a spurious
        //    multi-column-prose "table" from a real sparse one. The
        //    correct fix is upstream: stop the spatial-table detector
        //    from firing on prose columns in the first place.
        //  * dedup_consecutive_paragraphs — removed DISTINCT form
        //    widgets that share a label (annotation-button-widget.pdf:
        //    several real radio buttons all labelled "Radio button,
        //    unselected") and collapsed legitimately-repeated headings
        //    (ArabicCIDTrueType.pdf). "Looks duplicated" != "is an
        //    extraction artifact". The correct fix is upstream: stop
        //    the structured + plaintext paths from double-emitting.
        //  * filter_page_number_lines — dropped real "Page N" text;
        //    correct fix is `/Artifact` handling (§14.8.2.2).
        //  * normalize_bullet_glyphs — rewrote codepoints; correct fix
        //    is ToUnicode-CMap fallback (§9.10).
        //
        // dedup_identical_header_cells is also retired from the active
        // path: blanking "duplicate" header cells assumes the
        // duplication is an artifact, which the same content-
        // preservation principle rejects without upstream certainty.

        // Apply hyphenation reconstruction if enabled
        if config.enable_hyphenation_reconstruction {
            let handler = HyphenationHandler::new();
            final_result = handler.process_text(&final_result);
        }

        // RTL emphasis cleanup (#377 D7-fix). The original D7 also
        // unconditionally re-ordered each RTL line via
        // `reorder_visual_to_logical`, on the assumption that PDF
        // content streams always emit RTL runs in *visual* order. In
        // practice some PDFs (notably the pdfium hebrew_mirrored.pdf
        // test fixture and Arabic CID-TrueType samples) already store
        // text in *logical* order and our blanket reorder reversed
        // them again, breaking previously-correct output (`בנימין` →
        // `ןימינב`, `# heading` → `heading #`). Without a reliable
        // way to detect which order the source uses we drop the
        // reorder step. The other half of D7 — stripping spurious
        // `**bold**` / `*italic*` markers that the font-weight
        // detector emits around Arabic contextual glyph forms — is
        // safe and stays.
        if crate::text::bidi::looks_rtl(&final_result) {
            // `str::lines()` strips trailing newlines, so `join("\n")` would
            // silently drop a terminal `\n` (or `\n\n`) that the whitespace
            // normalisation step above carefully preserved.  Restore the
            // suffix after reassembly so callers see a consistent document.
            let trailing_newlines: String = final_result
                .chars()
                .rev()
                .take_while(|&c| c == '\n')
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            final_result = final_result
                .lines()
                .map(|line| {
                    if crate::text::bidi::looks_rtl(line) {
                        strip_inline_emphasis_in_rtl(line)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !trailing_newlines.is_empty() {
                final_result.push_str(&trailing_newlines);
            }
        }

        // Bidi-isolation markers (UAX #9 §2.4 — #537 follow-up).
        //
        // The v0.3.54 #537 detector landed the geometric visual-vs-
        // logical RTL classifier in `text::bidi::detect_visual_order_run`
        // and the extractor now reverses content-stream visual-order
        // runs into logical order. That fixed the *codepoint sequence*
        // (Hebrew letters appear in correct reading order).
        //
        // What it did not fix: bidi-rendering contamination at run
        // boundaries. When a markdown viewer (Pandoc, GitHub, VS Code
        // preview, Obsidian) reads a paragraph with mixed LTR + RTL
        // content and applies the Unicode Bidirectional Algorithm, the
        // *neutral* characters at run boundaries (parens, commas,
        // periods, spaces) migrate visually across the boundary
        // because they inherit direction from surrounding strong
        // characters. UAX #9 §2.4 fixes this with explicit isolation
        // markers: U+2067 RLI / U+2069 PDI around an RTL run inside an
        // LTR paragraph, U+2066 LRI / U+2069 PDI around an LTR run
        // inside an RTL paragraph.
        //
        // Markdown ONLY — `extract_text` and `PlainTextConverter` skip
        // this step. Plain-text consumers do not honour UAX #9 and
        // would render the markers as literal garbage. Per the v0.3.55
        // plan `docs/releases/plans/v0.3.55/fix-537-followup-bidi-isolation-markers.md`.
        if crate::text::bidi::looks_rtl(&final_result) {
            final_result = wrap_bidi_isolates_per_line(&final_result);
        }

        // Append confirmed footnote definitions as a trailing block of
        // `[^N]: text` lines, separated from the body by a blank line.
        // Emitted after all body post-processing so the definition
        // markers are never mistaken for table/list syntax. Empty when
        // no footnotes were confirmed (byte-identical output).
        if !footnote_plan.definitions.is_empty() {
            let trailing_nl = final_result
                .chars()
                .rev()
                .take_while(|&c| c == '\n')
                .count();
            for _ in trailing_nl..2 {
                final_result.push('\n');
            }
            for def in &footnote_plan.definitions {
                final_result.push_str(def);
                final_result.push('\n');
            }
        }

        Ok(final_result)
    }
}

/// A footnote reference token: either a run of digits (`1`, `12`) or a
/// single footnote symbol (`*`, `†`, `‡`, `§`, `¶`). Markers and
/// definitions are matched by the equality of these tokens.
#[derive(Clone, PartialEq, Eq, Hash)]
enum FootnoteToken {
    /// Numeric marker; stored as the raw digit string so `[^12]` keeps
    /// the author's number.
    Number(String),
    /// Symbol marker (`*`/`†`/`‡`/`§`/`¶`).
    Symbol(char),
}

impl FootnoteToken {
    /// String used to match a marker against a definition (`"1"`, `"*"`).
    fn key(&self) -> String {
        match self {
            FootnoteToken::Number(s) => s.clone(),
            FootnoteToken::Symbol(c) => c.to_string(),
        }
    }
}

/// The result of footnote detection over one page's spans.
///
/// All index fields are positions into the `sorted` slice passed to
/// `detect_footnotes` (which is exactly the enumeration index used by
/// the render loop). Every collection is empty when no footnote was
/// confirmed, in which case rendering is byte-for-byte unchanged.
#[derive(Default)]
struct FootnotePlan {
    /// span index → inline replacement text (`"[^1]"`).
    inline_markers: std::collections::HashMap<usize, String>,
    /// span indices belonging to a confirmed definition block (suppressed
    /// from normal body rendering).
    definition_spans: std::collections::HashSet<usize>,
    /// assembled definition lines (`"[^1]: Smith et al. 2019"`), ordered.
    definitions: Vec<String>,
}

/// Parse a whole span's text as a footnote *marker* token. Accepts a run
/// of 1–3 digits or a single footnote symbol; anything else (letters, an
/// ordinal suffix like `th`, multi-symbol runs) yields `None`.
fn parse_footnote_marker_token(text: &str) -> Option<FootnoteToken> {
    let t = text.trim();
    if t.is_empty() {
        return None;
    }
    if t.chars().count() == 1 {
        let c = t.chars().next().unwrap();
        if matches!(c, '*' | '†' | '‡' | '§' | '¶') {
            return Some(FootnoteToken::Symbol(c));
        }
    }
    if t.len() <= 3 && t.chars().all(|c| c.is_ascii_digit()) {
        return Some(FootnoteToken::Number(t.to_string()));
    }
    None
}

/// Parse a page-bottom definition line: a leading footnote token, an
/// optional single `.`/`)`/`:` separator, then the definition text. The
/// marker must be followed by whitespace or a separator (so `1st` or
/// `12x` do not falsely parse), and the remaining definition must be
/// non-empty.
fn split_leading_footnote_def(line: &str) -> Option<(FootnoteToken, String)> {
    let t = line.trim_start();
    let first = t.chars().next()?;
    let (token, consumed) = if matches!(first, '*' | '†' | '‡' | '§' | '¶') {
        (FootnoteToken::Symbol(first), first.len_utf8())
    } else if first.is_ascii_digit() {
        let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() > 3 {
            return None;
        }
        let len = digits.len();
        (FootnoteToken::Number(digits), len)
    } else {
        return None;
    };
    let after = &t[consumed..];
    // Boundary: marker must be delimited from the definition text.
    if !(after.starts_with(char::is_whitespace) || after.starts_with(['.', ')', ':'])) {
        return None;
    }
    let rest = after.strip_prefix(['.', ')', ':']).unwrap_or(after);
    let def = rest.trim();
    if def.is_empty() {
        return None;
    }
    // A footnote definition is prose (a citation or note), not a stray numeric
    // run. Require a few letters so a math/table row like "2 2 1 2" — which
    // happens to lead with a digit token — is not mistaken for a definition.
    // Because a footnote is only emitted when a marker has a MATCHING
    // definition, rejecting the garbage def also suppresses the spurious inline
    // `[^n]` marker (the false-positive seen on math-heavy arXiv text).
    if def.chars().filter(|c| c.is_alphabetic()).count() < 3 {
        return None;
    }
    Some((token, def.to_string()))
}

/// Find the adjacent body span for a candidate superscript marker at
/// index `i`: the immediate predecessor or successor whose text carries a
/// letter (real prose, not another marker), preferring the larger font.
fn adjacent_body_span(spans: &[&OrderedTextSpan], i: usize) -> Option<usize> {
    let candidates = [i.checked_sub(1), (i + 1 < spans.len()).then_some(i + 1)];
    let mut best: Option<usize> = None;
    for c in candidates.into_iter().flatten() {
        if !spans[c].span.text.chars().any(|ch| ch.is_alphabetic()) {
            continue;
        }
        best = match best {
            Some(b) if spans[b].span.font_size >= spans[c].span.font_size => Some(b),
            _ => Some(c),
        };
    }
    best
}

/// Conservative, high-precision footnote detection (WS2.7).
///
/// A footnote is only emitted when BOTH halves are found and their tokens
/// match: (a) an inline superscript marker in body text — a small, raised
/// digit/symbol span — AND (b) a page-bottom definition line starting with
/// the same token in a smaller-than-body font. Missing either half leaves
/// the text untouched (a false footnote is worse than a missed one).
///
/// Thresholds (chosen for precision): a marker font must be < 0.75× its
/// adjacent body span and its baseline raised by > 0.1× that body font;
/// the definition band is the bottom 18% of the page's baseline range and
/// its leading span must be < 0.92× the body font.
fn detect_footnotes(spans: &[&OrderedTextSpan], base_font_size: f32) -> FootnotePlan {
    use std::collections::{HashMap, HashSet};
    let mut plan = FootnotePlan::default();
    if spans.len() < 2 || base_font_size <= 0.0 {
        return plan;
    }

    // Page baseline extent. Higher y = higher on the page.
    let (mut y_min, mut y_max) = (f32::MAX, f32::MIN);
    for s in spans {
        let y = s.span.bbox.y;
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    }
    let y_range = y_max - y_min;
    if y_range <= 0.0 {
        return plan;
    }
    let bottom_cut = y_min + 0.18 * y_range;

    // --- Candidate definitions from the bottom band. ---
    let mut bottom: Vec<usize> = (0..spans.len())
        .filter(|&i| spans[i].span.bbox.y <= bottom_cut)
        .collect();
    bottom.sort_by(|&a, &b| {
        crate::utils::safe_float_cmp(spans[b].span.bbox.y, spans[a].span.bbox.y)
            .then(crate::utils::safe_float_cmp(spans[a].span.bbox.x, spans[b].span.bbox.x))
    });
    // Group bottom-band spans into lines by baseline proximity.
    let mut def_lines: Vec<Vec<usize>> = Vec::new();
    for &i in &bottom {
        let y = spans[i].span.bbox.y;
        if let Some(last) = def_lines.last_mut() {
            let ly = spans[last[0]].span.bbox.y;
            let tol = spans[i].span.font_size.max(1.0) * 0.6;
            if (ly - y).abs() <= tol {
                last.push(i);
                continue;
            }
        }
        def_lines.push(vec![i]);
    }
    // token, member-span indices, definition text.
    let mut cand_defs: Vec<(FootnoteToken, Vec<usize>, String)> = Vec::new();
    for line in &def_lines {
        let mut ordered = line.clone();
        ordered.sort_by(|&a, &b| {
            crate::utils::safe_float_cmp(spans[a].span.bbox.x, spans[b].span.bbox.x)
        });
        // The leading (marker) span must be in a smaller-than-body font.
        if spans[ordered[0]].span.font_size >= base_font_size * 0.92 {
            continue;
        }
        let mut text = String::new();
        for &si in &ordered {
            let t = spans[si].span.text.trim();
            if t.is_empty() {
                continue;
            }
            if !text.is_empty() && !text.ends_with(' ') {
                text.push(' ');
            }
            text.push_str(t);
        }
        if let Some((token, def)) = split_leading_footnote_def(&text) {
            cand_defs.push((token, ordered, def));
        }
    }
    if cand_defs.is_empty() {
        return plan;
    }

    // --- Candidate inline markers in body text. ---
    let mut cand_markers: Vec<(usize, FootnoteToken)> = Vec::new();
    for i in 0..spans.len() {
        let s = spans[i];
        if s.span.bbox.y <= bottom_cut {
            continue; // markers live in body text, not the bottom band
        }
        let token = match parse_footnote_marker_token(&s.span.text) {
            Some(t) => t,
            None => continue,
        };
        let adj = match adjacent_body_span(spans, i) {
            Some(a) => a,
            None => continue,
        };
        let body = spans[adj];
        // Meaningfully smaller font.
        if s.span.font_size >= body.span.font_size * 0.75 {
            continue;
        }
        // Baseline raised relative to the body span (superscript).
        if s.span.bbox.y <= body.span.bbox.y + body.span.font_size * 0.1 {
            continue;
        }
        // Still on roughly the same line (inline, not a separate row).
        if (s.span.bbox.y - body.span.bbox.y).abs() > body.span.font_size * 1.2 {
            continue;
        }
        // A footnote reference attaches to the end of a WORD ("compressible¹").
        // A raised digit that immediately follows another digit (or a lone
        // variable) is a sub/superscript inside an equation, not a footnote —
        // the false positive in math-heavy academic text, where a math
        // subscript coincidentally matches a real footnote number. Suppress a
        // marker whose immediate predecessor on the same line ends in a digit.
        if let Some(prev) = i.checked_sub(1) {
            let p = spans[prev];
            let same_line =
                (p.span.bbox.y - s.span.bbox.y).abs() <= s.span.font_size.max(1.0) * 1.5;
            if same_line {
                if let Some(last) = p.span.text.trim_end().chars().next_back() {
                    if last.is_ascii_digit() {
                        continue;
                    }
                }
            }
        }
        cand_markers.push((i, token));
    }
    if cand_markers.is_empty() {
        return plan;
    }

    // --- Confirm tokens present in BOTH sets, assign labels. ---
    let def_tokens: HashSet<String> = cand_defs.iter().map(|(t, _, _)| t.key()).collect();
    // token key → label name used inside `[^ ]` (digits, or a sequential
    // id for symbols in first-appearance order).
    let mut token_name: HashMap<String, String> = HashMap::new();
    let mut symbol_counter: u32 = 0;
    for (i, token) in &cand_markers {
        let key = token.key();
        if !def_tokens.contains(&key) {
            continue;
        }
        let name = token_name
            .entry(key)
            .or_insert_with(|| match token {
                FootnoteToken::Number(s) => s.clone(),
                FootnoteToken::Symbol(_) => {
                    symbol_counter += 1;
                    symbol_counter.to_string()
                },
            })
            .clone();
        plan.inline_markers.insert(*i, format!("[^{name}]"));
    }
    if plan.inline_markers.is_empty() {
        return plan;
    }

    // --- Emit definitions for confirmed tokens (first line per token). ---
    let mut seen: HashSet<String> = HashSet::new();
    let mut ordered_defs: Vec<(u32, String)> = Vec::new();
    for (token, indices, def) in &cand_defs {
        let key = token.key();
        let name = match token_name.get(&key) {
            Some(n) => n.clone(),
            None => continue,
        };
        if !seen.insert(key) {
            continue;
        }
        for &si in indices {
            plan.definition_spans.insert(si);
        }
        let order = name.parse::<u32>().unwrap_or(u32::MAX);
        ordered_defs.push((order, format!("[^{name}]: {def}")));
    }
    ordered_defs.sort_by_key(|(o, _)| *o);
    plan.definitions = ordered_defs.into_iter().map(|(_, s)| s).collect();
    plan
}

/// Walk `text` line by line and wrap each line's RTL runs (or LTR
/// runs inside RTL-dominant lines) with Unicode bidi-isolation
/// markers per UAX #9 §2.4. Pure-LTR lines (no RTL chars) are
/// returned unchanged byte-for-byte.
///
/// Block direction is decided per *line* because markdown line
/// breaks (`\n`) implicitly start a new bidi paragraph in every
/// viewer that honours UAX #9. We use
/// [`crate::text::bidi::paragraph_is_rtl`] which follows §3.3.1
/// (first-strong-character rule).
///
/// Trailing newlines are preserved (`str::lines()` would otherwise
/// drop them) so the document-level newline shape stays intact.
fn wrap_bidi_isolates_per_line(text: &str) -> String {
    let trailing_newlines: String = text
        .chars()
        .rev()
        .take_while(|&c| c == '\n')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    let lines: Vec<&str> = text.lines().collect();
    let mut out = String::with_capacity(text.len() + 16);
    for (i, line) in lines.iter().enumerate() {
        if crate::text::bidi::looks_rtl(line) {
            let block_is_rtl = crate::text::bidi::paragraph_is_rtl(line);
            out.push_str(&crate::text::bidi::wrap_rtl_isolates(line, block_is_rtl));
        } else {
            out.push_str(line);
        }
        if i + 1 < lines.len() {
            out.push('\n');
        }
    }
    if !trailing_newlines.is_empty() {
        out.push_str(&trailing_newlines);
    }
    out
}

/// Remove markdown `**` and `*` emphasis pairs that surround RTL
/// (Arabic / Hebrew) tokens. Inserted by the bold/italic detector
/// when the source PDF reports a font-weight change between
/// contextual glyph forms (initial / medial / final shapes); they
/// fragment the line into spurious emphasis spans and break bidi
/// reordering. Keeps emphasis around purely LTR runs intact.
///
/// Implementation note: the byte-position search via
/// `find_matching` is safe even on multi-byte UTF-8 because we only
/// look for ASCII `*` (0x2A) which never appears as a continuation
/// byte; matched indices always fall on a UTF-8 boundary. We then
/// build the output by appending UTF-8 string slices between the
/// matched positions, never reinterpreting individual bytes as
/// chars. (Copilot review #3108056051: the previous implementation
/// emitted `bytes[i] as char` for non-marker bytes and corrupted
/// non-ASCII content like `בנימין * world` → `×<ctrl>×<ctrl>... * world`.)
fn strip_inline_emphasis_in_rtl(line: &str) -> String {
    // Cheap path: if there are no asterisks, nothing to do.
    if !line.contains('*') {
        return line.to_string();
    }
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    let mut last_copy = 0;
    while i < bytes.len() {
        // Try to match `**` first.
        if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'*' {
            if let Some(close) = find_matching(bytes, i + 2, b"**") {
                // Copy any text that came before this `**` token verbatim.
                if i > last_copy {
                    out.push_str(&line[last_copy..i]);
                }
                let inner = &line[i + 2..close];
                if crate::text::bidi::looks_rtl(inner) {
                    out.push_str(inner);
                } else {
                    out.push_str("**");
                    out.push_str(inner);
                    out.push_str("**");
                }
                i = close + 2;
                last_copy = i;
                continue;
            }
        }
        // Then `*` (italic).
        if bytes[i] == b'*' {
            if let Some(close) = find_matching(bytes, i + 1, b"*") {
                if i > last_copy {
                    out.push_str(&line[last_copy..i]);
                }
                let inner = &line[i + 1..close];
                if crate::text::bidi::looks_rtl(inner) {
                    out.push_str(inner);
                } else {
                    out.push('*');
                    out.push_str(inner);
                    out.push('*');
                }
                i = close + 1;
                last_copy = i;
                continue;
            }
        }
        i += 1;
    }
    if last_copy < bytes.len() {
        out.push_str(&line[last_copy..]);
    }
    out
}

fn find_matching(bytes: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    let mut i = from;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Detect a multi-column gutter between two spans on the same baseline.
///
/// Used by the markdown converter to refine its `same_line` gate: two
/// spans at the same y but separated by a large horizontal gap are
/// almost certainly in different columns (newspaper / two-column
/// academic paper). They must NOT be merged into one paragraph even
/// if their `block_id`s suggest a structural transition would be
/// suppressed by D5b.
///
/// Returns true in two distinct shapes (issue #377 D5d):
///
/// 1. **Forward column gap.** The horizontal gap from the right edge
///    of the previous span to the left edge of the current span
///    exceeds `max(3 × font_size, 30 pt)`. 3× font size catches
///    typical body-text columns (12pt body → 36pt gutter); the 30pt
///    floor catches small-font cases where a literal 36pt gap would
///    be too lenient.
///
/// 2. **Backward column wrap (x went backwards on the same baseline).**
///    LTR text on a single visual line always advances x forward; if
///    the current span starts to the left of the previous span by
///    more than `2 × font_size`, that is a column-major reading order
///    wrapping from the end of one column back to the top of the
///    next. The IA_0047 newspaper struct tree emits content this way:
///    `constitution` at x=976 ends a column, `Assailing` at x=192
///    starts the next, both at the same baseline. Without the
///    backward-wrap detection the converter joins them into the
///    nonsense token `constitutionAssailing`.
///
/// Marker wrapped around a fully-monospace plain paragraph at flush time so
/// `fence_monospace_blocks` can recognise and fuse consecutive ones. NUL never
/// occurs in extracted text, so it is unambiguous and is always removed.
const MONO_SENTINEL: char = '\u{0}';

/// Emit a plain (non-heading, non-list) paragraph, wrapping it in
/// [`MONO_SENTINEL`] markers when it is entirely monospace so the fence pass
/// can fuse a run of them into one code block.
fn push_plain_paragraph(result: &mut String, line: &str, mono: bool) {
    if mono && !line.is_empty() {
        result.push(MONO_SENTINEL);
        result.push_str(line);
        result.push(MONO_SENTINEL);
    } else {
        result.push_str(line);
    }
    result.push_str("\n\n");
}

/// Fuse consecutive monospace paragraphs (each flagged with [`MONO_SENTINEL`])
/// into a single fenced code block, preserving their internal line breaks.
/// A run of one or more sentinel-marked paragraphs becomes:
/// ```text
/// ```
/// line 1
/// line 2
/// ```
/// ```
/// Non-marked paragraphs pass through unchanged; the markers are always
/// stripped (so even a lone inline NUL, which never occurs anyway, is removed).
fn fence_monospace_blocks(s: &str) -> String {
    if !s.contains(MONO_SENTINEL) {
        return s.to_string();
    }
    let paras: Vec<&str> = s.split("\n\n").collect();
    let mut out: Vec<String> = Vec::with_capacity(paras.len());
    let mut code: Vec<String> = Vec::new();
    let flush_code = |code: &mut Vec<String>, out: &mut Vec<String>| {
        if !code.is_empty() {
            let body = code.join("\n");
            // A fence around whitespace-only content carries no information and,
            // on a scanned / blank page (whose only spans are stray whitespace),
            // it would make the page's Markdown non-empty — masking the empty
            // state and suppressing the `[OCR REQUIRED]` annotation. Drop it.
            if body.trim().is_empty() {
                code.clear();
                return;
            }
            out.push(format!("```\n{body}\n```"));
            code.clear();
        }
    };
    for p in paras {
        let trimmed = p.trim();
        if let Some(inner) = trimmed
            .strip_prefix(MONO_SENTINEL)
            .and_then(|t| t.strip_suffix(MONO_SENTINEL))
        {
            code.push(inner.to_string());
        } else {
            flush_code(&mut code, &mut out);
            // Defensive: strip any stray sentinel from a normal paragraph.
            if p.contains(MONO_SENTINEL) {
                out.push(p.replace(MONO_SENTINEL, ""));
            } else {
                out.push(p.to_string());
            }
        }
    }
    flush_code(&mut code, &mut out);
    out.join("\n\n")
}

fn is_column_gap(prev: &OrderedTextSpan, current: &OrderedTextSpan) -> bool {
    let prev_right = prev.span.bbox.x + prev.span.bbox.width;
    let cur_left = current.span.bbox.x;
    let font_size = current.span.font_size.max(prev.span.font_size).max(1.0);

    // Right-to-left text reads with *decreasing* X, so the backward-X "column
    // wrap" heuristic below (which assumes left-to-right flow) is inverted for
    // it: an RTL line's spans, ordered into logical reading order, step left on
    // every word. Treating that as a column boundary splits the line into one
    // paragraph/heading per word (Arabic/Hebrew titles and prose). The RTL
    // reading-order pass has already placed these spans correctly, so skip the
    // backward-X branch when both sides are RTL.
    // A whitespace / neutral span between two RTL words is part of the RTL
    // flow (it has no direction of its own), so it must not break the RTL
    // context — otherwise every word→space→word step inside a right-to-left
    // line reads as a same-baseline column gap and the line shatters into one
    // paragraph per word. This covers both a pure-space separator and a short
    // neutral-punctuation one (e.g. " ," / " ." between two words): per UAX #9
    // such neutrals inherit the surrounding right-to-left direction, so a span
    // of ≤2 non-alphanumeric chars is RTL-friendly too.
    let rtl_friendly = |t: &str| {
        let s = t.trim();
        crate::text::bidi::looks_rtl(t)
            || s.is_empty()
            || (s.chars().count() <= 2 && !s.chars().any(|c| c.is_alphanumeric()))
    };
    let both_rtl = rtl_friendly(&prev.span.text)
        && rtl_friendly(&current.span.text)
        && (crate::text::bidi::looks_rtl(&prev.span.text)
            || crate::text::bidi::looks_rtl(&current.span.text));

    // Backward wrap: x went meaningfully backwards. This is the signature of
    // BOTH a within-column line wrap (X resets to the column's left margin) and
    // a genuine column transition (X jumps to the next column). They are told
    // apart by Y: a normal line wrap steps DOWN by about one line, whereas a
    // column transition jumps back UP to the next column's top (Y increases) or
    // drops far more than a line. Only the latter is a column boundary —
    // treating a plain wrap as one splits every wrapped line in a narrow column
    // into its own paragraph (multi-column newspaper / journal bodies).
    if !both_rtl && cur_left + font_size * 2.0 < prev.span.bbox.x {
        // A within-column line wrap drops by about ONE line; a column
        // transition either jumps back UP (Y increases), drops far more than a
        // line, or stays on roughly the SAME baseline (a balanced column whose
        // next column resumes at the same height). Only a ~one-line downward
        // step is an ordinary wrap.
        let y_drop = prev.span.bbox.y - current.span.bbox.y;
        if y_drop > font_size * 0.5 && y_drop < font_size * 2.0 {
            return false; // ordinary line wrap, not a column boundary
        }
        return true;
    }

    // Forward gutter: gap exceeds typical inter-word spacing.
    let gap = cur_left - prev_right;
    if gap <= 0.0 {
        return false;
    }
    let threshold = (font_size * 3.0).max(30.0);
    gap > threshold
}

impl Default for MarkdownOutputConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputConverter for MarkdownOutputConverter {
    fn convert(&self, spans: &[OrderedTextSpan], config: &TextPipelineConfig) -> Result<String> {
        self.render_spans(spans, &[], config)
    }

    fn convert_with_tables(
        &self,
        spans: &[OrderedTextSpan],
        tables: &[Table],
        config: &TextPipelineConfig,
    ) -> Result<String> {
        self.render_spans(spans, tables, config)
    }

    fn name(&self) -> &'static str {
        "MarkdownOutputConverter"
    }

    fn mime_type(&self) -> &'static str {
        "text/markdown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;
    use crate::layout::{Color, TextSpan};
    use crate::pipeline::converters::span_in_table;
    use crate::pipeline::StructRole;
    use crate::structure::table_extractor::{TableCell, TableRow};

    #[test]
    fn test_fence_monospace_blocks() {
        let n = MONO_SENTINEL;
        // Consecutive monospace paragraphs fuse into one fenced block, keeping
        // their internal line breaks; surrounding prose is untouched.
        let input = format!("Intro\n\n{n}line one{n}\n\n{n}line two{n}\n\nOutro");
        assert_eq!(
            fence_monospace_blocks(&input),
            "Intro\n\n```\nline one\nline two\n```\n\nOutro"
        );
        // A lone monospace paragraph still fences.
        assert_eq!(fence_monospace_blocks(&format!("{n}only{n}")), "```\nonly\n```");
        // No sentinels → byte-identical.
        assert_eq!(fence_monospace_blocks("plain\n\ntext"), "plain\n\ntext");
        // A stray sentinel inside ordinary prose is stripped, not fenced.
        assert_eq!(fence_monospace_blocks(&format!("a{n}b")), "ab");
    }

    #[test]
    fn test_push_plain_paragraph_marks_monospace() {
        let n = MONO_SENTINEL;
        let mut out = String::new();
        push_plain_paragraph(&mut out, "code", true);
        assert_eq!(out, format!("{n}code{n}\n\n"));
        let mut out2 = String::new();
        push_plain_paragraph(&mut out2, "prose", false);
        assert_eq!(out2, "prose\n\n");
    }

    #[test]
    fn test_tighten_list_items_collapses_blank_lines_between_markers() {
        // Loose list (blank lines between items) → tight list. CommonMark §5.3.
        let loose = "- a\n\n- b\n\n- c\n\nNext paragraph.";
        let tight = tighten_list_items(loose);
        assert_eq!(tight, "- a\n- b\n- c\n\nNext paragraph.");

        // Ordered list likewise.
        let loose_ol = "1. one\n\n2. two\n\nDone.";
        assert_eq!(tighten_list_items(loose_ol), "1. one\n2. two\n\nDone.");

        // Blank line separating a list from following prose is preserved.
        let mixed = "- only item\n\nA paragraph after the list.";
        assert_eq!(tighten_list_items(mixed), mixed);

        // Non-list content is untouched.
        let prose = "# Title\n\nPara one.\n\nPara two.";
        assert_eq!(tighten_list_items(prose), prose);
    }

    #[test]
    fn test_is_md_list_item_line() {
        for yes in ["- x", "* y", "+ z", "1. a", "2) b", "  - indented"] {
            assert!(is_md_list_item_line(yes), "{yes:?} should be a list item");
        }
        for no in [
            "text",
            "-no space",
            "1.no space",
            "#- heading",
            "",
            "  plain",
        ] {
            assert!(!is_md_list_item_line(no), "{no:?} should NOT be a list item");
        }
    }

    /// The two fragments a garbled 1919 broadsheet had promoted to headings.
    /// Both clear every other test the predicate applies: the first leads with
    /// a capital and runs to five words, the second is two words, under the
    /// five-word floor the lowercase-initial rule uses.
    ///
    /// Asserted through `is_valid_heading_text` rather than the helper it calls,
    /// so that disabling the call site fails this test.
    #[test]
    fn test_mid_sentence_fragment_is_not_promoted_to_a_heading() {
        for fragment in ["Furthermore, one reads in the", "palaces league."] {
            assert!(
                !MarkdownOutputConverter::is_valid_heading_text(fragment),
                "{fragment:?} is a sentence fragment, not a heading"
            );
        }
    }

    /// Real headings must still promote, including ones that end on a word the
    /// guard could plausibly over-reach on.
    #[test]
    fn real_headings_still_promote() {
        for heading in [
            "Spring Equinox Gathering",
            "Introduction",
            "Contact Us",
            "Let It Be",
            "Doctor Who",
            "Materials and Methods",
            "Terms of Service",
        ] {
            assert!(
                MarkdownOutputConverter::is_valid_heading_text(heading),
                "{heading:?} is a heading and must still promote"
            );
        }
    }

    #[test]
    fn test_bare_ordinal_suffix_is_not_a_heading() {
        // A stranded superscript ordinal must never be promoted to a heading.
        for ord in ["st", "nd", "rd", "th", "ST", "Th", " th "] {
            assert!(
                !MarkdownOutputConverter::is_valid_heading_text(ord),
                "{ord:?} must not be a valid heading"
            );
        }
        // Real (word-leading) headings stay valid.
        assert!(MarkdownOutputConverter::is_valid_heading_text("Spring Equinox Gathering"));
        assert!(MarkdownOutputConverter::is_valid_heading_text("Eastern Apiary Update"));
    }

    /// D1 RED — when the structure tree carries an explicit heading role
    /// for a span (Word/Acrobat style: H1 → Span → MCR resolved by D8b),
    /// the markdown converter must emit `# title` regardless of font-size
    /// heuristics. Without this, every tagged Word document loses its
    /// heading hierarchy because body and heading text are often the
    /// same point size.
    #[test]
    fn test_struct_role_heading_emits_markdown_heading() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut title = make_span("Document Title", 0.0, 100.0, 12.0, FontWeight::Normal);
        title.struct_role = Some(StructRole::Heading(1));
        let body = make_span("Body paragraph one.", 0.0, 80.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[title, body], &config).unwrap();
        assert!(
            result.contains("# Document Title"),
            "expected '# Document Title' in output, got:\n{}",
            result
        );
        assert!(result.contains("Body paragraph one."));
    }

    /// D1 RED — heading role precedence: even on the same font size as
    /// body, Heading(2) must produce `## ...`. Mirrors the `nougat_011`
    /// failure pattern where per-section headers are body-sized.
    #[test]
    fn test_struct_role_h2_overrides_font_size_heuristic() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut h2 = make_span("Section Header", 0.0, 100.0, 11.0, FontWeight::Normal);
        h2.struct_role = Some(StructRole::Heading(2));
        let result = converter.convert(&[h2], &config).unwrap();
        assert!(result.starts_with("## "), "expected `## ` heading prefix, got:\n{}", result);
    }

    /// D3 unit — `is_ordered_list_marker` recognises common forms and
    /// rejects look-alikes that are NOT lists (figure captions, years).
    #[test]
    fn test_is_ordered_list_marker_recognition() {
        // Recognised forms.
        assert_eq!(MarkdownOutputConverter::is_ordered_list_marker("1. Foo"), Some(1));
        assert_eq!(MarkdownOutputConverter::is_ordered_list_marker("12. Foo"), Some(12));
        assert_eq!(MarkdownOutputConverter::is_ordered_list_marker("a) Foo"), Some(1));
        assert_eq!(MarkdownOutputConverter::is_ordered_list_marker("A. Foo"), Some(1));
        assert_eq!(MarkdownOutputConverter::is_ordered_list_marker("iv. Foo"), Some(1));
        // Conservative rejections so figure captions and years are not promoted.
        assert!(MarkdownOutputConverter::is_ordered_list_marker("1.1 Foo").is_none());
        assert!(MarkdownOutputConverter::is_ordered_list_marker("1986 was").is_none());
        assert!(MarkdownOutputConverter::is_ordered_list_marker("Item one").is_none());
    }

    /// D3 RED — three numbered items on consecutive lines must each
    /// land on their own markdown line. Reproduces the nougat_037
    /// "1. Treasurer ... 2. Safeguarding ... 3. Volunteering"
    /// collapse pattern (those three were on different baselines but
    /// joined by tight gap).
    #[test]
    fn test_numbered_list_consecutive_lines_separate() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let s1 = make_span("1. Treasurer", 0.0, 100.0, 12.0, FontWeight::Normal);
        let s2 = make_span("2. Safeguarding", 0.0, 88.0, 12.0, FontWeight::Normal);
        let s3 = make_span("3. Volunteering", 0.0, 76.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[s1, s2, s3], &config).unwrap();
        for marker in ["1. Treasurer", "2. Safeguarding", "3. Volunteering"] {
            assert!(
                result.lines().any(|l| l.trim_start().starts_with(marker)),
                "expected line starting with `{}`, got:\n{}",
                marker,
                result
            );
        }
    }

    /// D4 RED — when an untagged paragraph is followed by a bullet list
    /// with a small geometric gap, the list must still start on a new
    /// line preceded by a blank line. Reproduces the `Intro sentence.•
    /// First` glue pattern.
    #[test]
    fn test_bullet_after_paragraph_forces_break() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Tight gap: body 12pt, gap 4pt (well below 1.5×).
        let intro = make_span("Intro sentence.", 0.0, 100.0, 12.0, FontWeight::Normal);
        let b1 = make_span("• First item", 0.0, 88.0, 12.0, FontWeight::Normal);
        let b2 = make_span("• Second item", 0.0, 76.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[intro, b1, b2], &config).unwrap();
        assert!(
            result.contains("Intro sentence.\n\n- First item"),
            "expected blank line + bullet after intro, got:\n{}",
            result
        );
    }

    /// D1 coverage — every heading level H1..H6 from the structure tree
    /// emits the matching markdown prefix. Lock-in for #377 word /
    /// adobe-tagged docs whose body and heading text share a size.
    #[test]
    fn test_struct_role_emits_each_heading_level() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        for level in 1u8..=6 {
            let mut s =
                make_span(&format!("Title L{}", level), 0.0, 100.0, 12.0, FontWeight::Normal);
            s.struct_role = Some(StructRole::Heading(level));
            let body = make_span("body", 0.0, 80.0, 12.0, FontWeight::Normal);
            let result = converter.convert(&[s, body], &config).unwrap();
            let prefix = "#".repeat(level as usize);
            let expected = format!("{} Title L{}", prefix, level);
            assert!(result.contains(&expected), "expected `{}`, got:\n{}", expected, result);
        }
    }

    /// D1 coverage — out-of-range Heading level values are clamped to
    /// the H1..H6 range. Defensive: a malformed structure tree
    /// reporting Heading(0) or Heading(99) should not produce 0 or
    /// 99 `#` characters.
    #[test]
    fn test_struct_role_heading_level_is_clamped() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        for raw_level in [0u8, 7, 99, 250] {
            let mut s = make_span("Edgy", 0.0, 100.0, 12.0, FontWeight::Normal);
            s.struct_role = Some(StructRole::Heading(raw_level));
            let result = converter.convert(&[s], &config).unwrap();
            // Find the prefix in the first line: count `#`s.
            let first_line = result.lines().next().unwrap_or("");
            let hash_count = first_line.chars().take_while(|c| *c == '#').count();
            assert!(
                (1..=6).contains(&hash_count),
                "raw_level {} produced {} `#`s in `{}`",
                raw_level,
                hash_count,
                first_line
            );
        }
    }

    /// A heading must not be glued to the first list item that follows it
    /// (`## Highlights - Revenue…`).
    #[test]
    fn test_heading_not_glued_to_following_list() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut h = make_span("Highlights", 0.0, 100.0, 12.0, FontWeight::Normal);
        h.struct_role = Some(StructRole::Heading(2));
        let mut a = make_span("Revenue grew steadily.", 0.0, 80.0, 12.0, FontWeight::Normal);
        a.struct_role = Some(StructRole::ListItemBody);
        a.block_id = Some(1);
        let mut b = make_span("Costs remained flat.", 0.0, 64.0, 12.0, FontWeight::Normal);
        b.struct_role = Some(StructRole::ListItemBody);
        b.block_id = Some(2);
        let out = converter.convert(&[h, a, b], &config).unwrap();
        eprintln!("--- heading/list output ---\n{out}\n---");
        assert!(!out.contains("Highlights - "), "heading glued to list:\n{out}");
        assert!(out.contains("- Revenue grew steadily."), "first item missing bullet:\n{out}");
    }

    /// D1 coverage — every list-role variant (LI / Lbl / LBody) on a
    /// span emits a `- ` bullet prefix. Lock-in against treating the
    /// three roles inconsistently, which was the original
    /// word365_structure regression.
    #[test]
    fn test_struct_role_all_list_variants_emit_bullets() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        for role in [
            StructRole::ListItem,
            StructRole::ListItemLabel,
            StructRole::ListItemBody,
        ] {
            let mut s = make_span("Item", 0.0, 100.0, 12.0, FontWeight::Normal);
            s.struct_role = Some(role);
            let result = converter.convert(&[s], &config).unwrap();
            assert!(
                result.lines().any(|l| l.starts_with("- ")),
                "role {:?} did not emit a bullet, got:\n{}",
                role,
                result
            );
        }
    }

    /// D1 coverage — heading immediately followed by a list-item must
    /// transition cleanly: heading flushes, list emits bullet on a
    /// fresh line. Cross-defect interaction guard.
    #[test]
    fn test_heading_then_list_item_transition() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut h = make_span("Section", 0.0, 100.0, 12.0, FontWeight::Normal);
        h.struct_role = Some(StructRole::Heading(2));
        let mut li = make_span("First", 0.0, 80.0, 12.0, FontWeight::Normal);
        li.struct_role = Some(StructRole::ListItemBody);
        let result = converter.convert(&[h, li], &config).unwrap();
        assert!(result.contains("## Section"));
        assert!(result.contains("- First"));
        // The heading line must not also carry the bullet.
        assert!(
            !result.contains("## - "),
            "heading prefix and bullet must not co-occur, got:\n{}",
            result
        );
    }

    /// D5 coverage — three sequential block_id transitions produce
    /// three paragraphs. Lock against off-by-one in the transition
    /// detector that would group two of three.
    #[test]
    fn test_block_id_three_paragraphs_three_breaks() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut spans = Vec::new();
        for (i, t) in ["alpha", "beta", "gamma"].iter().enumerate() {
            let mut s = make_span(t, 0.0, 100.0 - (i as f32 * 14.0), 12.0, FontWeight::Normal);
            s.block_id = Some((i + 1) as u32);
            spans.push(s);
        }
        let result = converter.convert(&spans, &config).unwrap();
        let paras: Vec<&str> = result
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        assert_eq!(
            paras,
            vec!["alpha", "beta", "gamma"],
            "expected 3 separate paragraphs, got {:?}",
            paras
        );
    }

    /// D5 coverage — when only one of two adjacent spans has a
    /// block_id (mixed tagged + untagged), no spurious break is
    /// emitted. Defends against the `(Some, None)` case being misread
    /// as a transition.
    #[test]
    fn test_partial_block_id_does_not_force_break() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let s1 = make_span("first", 0.0, 100.0, 12.0, FontWeight::Normal);
        let mut s2 = make_span("second", 0.0, 88.0, 12.0, FontWeight::Normal);
        s2.block_id = Some(1);
        let result = converter.convert(&[s1, s2], &config).unwrap();
        // Without explicit block transition, fall through to geometry —
        // 12pt gap below 1.5× threshold, so no double newline.
        assert!(
            !result.contains("\n\n"),
            "partial block_id must not introduce paragraph break, got:\n{}",
            result
        );
    }

    /// D3 coverage — extra whitelist + reject cases for ordered marker
    /// detection. Locks the conservative behaviour that distinguishes
    /// real lists from prose / numbers / captions.
    #[test]
    fn test_is_ordered_list_marker_extras() {
        // Recognised: trailing space required, both `.` and `)` close.
        assert_eq!(MarkdownOutputConverter::is_ordered_list_marker("99. Foo"), Some(99));
        assert_eq!(MarkdownOutputConverter::is_ordered_list_marker("z) Last"), Some(1));
        // Without the trailing space, not a list marker.
        assert!(MarkdownOutputConverter::is_ordered_list_marker("1.Foo").is_none());
        // Without a digit/letter prefix, not a list marker.
        assert!(MarkdownOutputConverter::is_ordered_list_marker(". Foo").is_none());
        assert!(MarkdownOutputConverter::is_ordered_list_marker(") Foo").is_none());
        // Empty / whitespace-only.
        assert!(MarkdownOutputConverter::is_ordered_list_marker("").is_none());
        assert!(MarkdownOutputConverter::is_ordered_list_marker("   ").is_none());
        // Looks like a list but is currency / unit / decimal.
        assert!(MarkdownOutputConverter::is_ordered_list_marker("$1. Total").is_none());
        assert!(MarkdownOutputConverter::is_ordered_list_marker("3.14 pi").is_none());
        // Long numeric (>3 digits) is not a marker (years, IDs).
        assert!(MarkdownOutputConverter::is_ordered_list_marker("2024. Year").is_none());
    }

    /// D6 coverage — superscript inline merging across multiple
    /// markers in the same line ("On the 1st, 2nd, and 3rd days").
    /// Each "st"/"nd"/"rd" must inline-merge with its preceding
    /// number.
    #[test]
    fn test_multiple_superscripts_one_line() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Body baseline at y=100 with 11pt; three superscripts raised
        // by 2.5pt with 7pt font.
        let parts: Vec<OrderedTextSpan> = vec![
            make_span("On the 1", 0.0, 100.0, 11.0, FontWeight::Normal),
            make_span("st", 25.0, 102.5, 7.0, FontWeight::Normal),
            make_span(", 2", 30.0, 100.0, 11.0, FontWeight::Normal),
            make_span("nd", 40.0, 102.5, 7.0, FontWeight::Normal),
            make_span(", and 3", 47.0, 100.0, 11.0, FontWeight::Normal),
            make_span("rd", 70.0, 102.5, 7.0, FontWeight::Normal),
            make_span(" days", 75.0, 100.0, 11.0, FontWeight::Normal),
        ];
        let result = converter.convert(&parts, &config).unwrap();
        // No bare superscript line.
        for sup in ["st", "nd", "rd"] {
            assert!(
                !result.lines().any(|l| l.trim() == sup),
                "bare `{}` line found in:\n{}",
                sup,
                result
            );
        }
        // Each composed token appears.
        for token in ["1st", "2nd", "3rd"] {
            assert!(result.contains(token), "expected `{}` in output, got:\n{}", token, result);
        }
    }

    /// D2 RED — bold text only slightly larger than body must still be
    /// detected as a heading. Many tagged-but-untyped corporate docs
    /// (amt_handbook_sample, manuals) use bold + 1.05–1.1× body for
    /// section headings without /H tags. Previous threshold was bold +
    /// 1.10×.
    #[test]
    fn test_bold_slight_size_bump_is_heading() {
        let converter = MarkdownOutputConverter::new();
        let mut config = TextPipelineConfig::default();
        config.output.detect_headings = true;
        // Body at 11pt, "section header" bold at 11.55pt (1.05× body).
        let body_a = make_span("First body sentence.", 0.0, 100.0, 11.0, FontWeight::Normal);
        let body_b = make_span("Second body sentence.", 0.0, 88.0, 11.0, FontWeight::Normal);
        let head = make_span("Section Header", 0.0, 76.0, 11.55, FontWeight::Bold);
        let body_c = make_span("After-heading body.", 0.0, 64.0, 11.0, FontWeight::Normal);
        let result = converter
            .convert(&[body_a, body_b, head, body_c], &config)
            .unwrap();
        assert!(
            result.contains("### Section Header") || result.contains("#### Section Header"),
            "expected heading prefix on bold +5% line, got:\n{}",
            result
        );
    }

    /// D5b RED — same-baseline spans with different `block_id`s
    /// from the structure tree (form-style PDFs that split a single
    /// horizontal heading into multiple /P sub-elements, e.g.
    /// `Form` + `1040` + `U.S. Individual Income Tax Return` rendered
    /// on one line) must NOT trigger a structure-tree paragraph break.
    /// Otherwise one heading becomes three `#` lines (irs_f1040
    /// regression observed in v0.3.36).
    #[test]
    fn test_same_baseline_blocks_do_not_split_heading() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Three pieces of one visual H1, same y=100, same font, but
        // each with its own structure-tree block_id (mimicking the
        // tagged form's three /P elements under one /H1 visually
        // joined in a horizontal heading band).
        let mk = |t: &str, x: f32, bid: u32| {
            let mut s = make_span(t, x, 100.0, 18.0, FontWeight::Bold);
            s.struct_role = Some(StructRole::Heading(1));
            s.block_id = Some(bid);
            s
        };
        let spans = vec![
            mk("Form", 0.0, 1),
            mk("1040", 50.0, 2),
            mk("U.S. Individual Income Tax Return", 100.0, 3),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        let heading_lines: Vec<&str> = result
            .lines()
            .filter(|l| l.trim_start().starts_with("# "))
            .collect();
        assert_eq!(
            heading_lines.len(),
            1,
            "expected one combined heading line, got {} in:\n{}",
            heading_lines.len(),
            result
        );
        assert!(
            heading_lines[0].contains("Form")
                && heading_lines[0].contains("1040")
                && heading_lines[0].contains("U.S. Individual Income Tax Return"),
            "all three pieces must be in the single heading line, got: {}",
            heading_lines[0]
        );
    }

    /// D5b coverage — same-baseline list-item segments don't fragment.
    /// Some forms wrap each item label in its own /LI struct elem but
    /// render the whole list horizontally on one line; the converter
    /// must keep them together when y matches.
    #[test]
    fn test_same_baseline_blocks_do_not_split_list_items() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, x: f32, bid: u32| {
            let mut s = make_span(t, x, 100.0, 12.0, FontWeight::Normal);
            s.struct_role = Some(StructRole::ListItemBody);
            s.block_id = Some(bid);
            s
        };
        let spans = vec![
            mk("Apple", 0.0, 1),
            mk("Banana", 60.0, 2),
            mk("Cherry", 120.0, 3),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        let bullet_lines: Vec<&str> = result.lines().filter(|l| l.starts_with("- ")).collect();
        assert_eq!(
            bullet_lines.len(),
            1,
            "horizontal list on one line must stay one bullet, got {} in:\n{}",
            bullet_lines.len(),
            result
        );
    }

    /// D5b coverage — different baselines must STILL fragment as
    /// before. Negative regression check on the D5 win: nougat_011
    /// went from 64 to 266 lines because each /P became its own
    /// paragraph; our same_line gate must not undo that for spans on
    /// different baselines.
    #[test]
    fn test_different_baseline_blocks_still_split() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut p1 = make_span("First.", 0.0, 100.0, 12.0, FontWeight::Normal);
        p1.block_id = Some(1);
        let mut p2 = make_span("Second.", 0.0, 70.0, 12.0, FontWeight::Normal);
        p2.block_id = Some(2);
        let mut p3 = make_span("Third.", 0.0, 40.0, 12.0, FontWeight::Normal);
        p3.block_id = Some(3);
        let result = converter.convert(&[p1, p2, p3], &config).unwrap();
        let paras: Vec<&str> = result
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        assert_eq!(
            paras,
            vec!["First.", "Second.", "Third."],
            "different baselines must still produce three paragraphs"
        );
    }

    /// `strip_inline_emphasis_in_rtl` must preserve non-ASCII (Arabic
    /// / Hebrew) characters in the non-emphasis portion of an RTL
    /// line. Earlier the function iterated the UTF-8 byte array and
    /// pushed each byte as a Latin-1 char, corrupting `בנימין * world`
    /// into `×<ctrl>×<ctrl>... * world`. The no-`*` short-circuit hid
    /// the bug from earlier RTL tests.
    #[test]
    fn test_strip_inline_emphasis_preserves_rtl_chars_around_lone_asterisk() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let span = make_span("בנימין * world", 0.0, 100.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[span], &config).unwrap();
        assert!(
            result.contains("בנימין"),
            "Hebrew letters lost — UTF-8 corruption: {:?}",
            result
        );
        assert!(
            !result
                .chars()
                .any(|c| (c as u32) == 0x91 || (c as u32) == 0xA0),
            "byte-as-char ghost characters present in: {:?}",
            result
        );
    }

    /// Arabic regression coverage — confirms `strip_inline_emphasis_in_rtl`
    /// preserves Arabic across the no-`*`, single-`*`, paired-`*`,
    /// and paired-`**` cases. Locks the Copilot-found UTF-8
    /// corruption out for good across realistic shapes.
    ///
    /// v0.3.55 (#537 follow-up): the markdown converter now also wraps
    /// LTR runs inside RTL-dominant paragraphs with U+2066/U+2069
    /// isolation markers. Substring assertions below strip those
    /// markers before matching so this test continues to cover what
    /// it was meant to cover — the emphasis-stripper's Arabic /
    /// Hebrew preservation contract — independently of the new
    /// bidi-isolation pass.
    #[test]
    fn test_arabic_strip_inline_emphasis_matrix() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Each tuple: (input span text, list of expected substrings).
        let cases: &[(&str, &[&str])] = &[
            // No `*` — short-circuit; must round-trip.
            ("اللغة العربية بسيطة", &["اللغة", "العربية", "بسيطة"]),
            // Hebrew with stray `*` (lone asterisk, no pair).
            ("בנימין * world", &["בנימין", "* world"]),
            // Arabic paragraph with `*emphasis*` around RTL token.
            ("مرحبا *عالم* اليوم", &["مرحبا", "عالم", "اليوم"]),
            // Arabic paragraph with `**bold**` around RTL token.
            ("مرحبا **عالم** اليوم", &["مرحبا", "عالم", "اليوم"]),
            // Mixed: emphasis around LTR (must keep markers) plus Arabic.
            ("مرحبا *Hello* اليوم", &["مرحبا", "*Hello*", "اليوم"]),
        ];
        for (input, expected_subs) in cases {
            let span = make_span(input, 0.0, 100.0, 12.0, FontWeight::Normal);
            let result = converter.convert(&[span], &config).unwrap();
            // Strip the v0.3.55 #537-follow-up bidi-isolation markers
            // (U+2066/U+2067/U+2068/U+2069) before substring checks —
            // they are correct, semantically additive, and orthogonal
            // to what this test exercises.
            let result_no_iso: String = result
                .chars()
                .filter(|c| !matches!(*c, '\u{2066}' | '\u{2067}' | '\u{2068}' | '\u{2069}'))
                .collect();
            for needle in *expected_subs {
                assert!(
                    result_no_iso.contains(needle),
                    "input {:?} → expected {:?} in output:\n{}",
                    input,
                    needle,
                    result_no_iso
                );
            }
            // Ghost-byte check: no Latin-1 control chars from
            // mis-cast UTF-8 should appear. Run against the raw
            // result so any new ghost-byte regression still fires.
            assert!(
                !result.chars().any(|c| {
                    let n = c as u32;
                    (0x80..=0x9F).contains(&n) || n == 0xA0
                }),
                "input {:?} produced Latin-1 ghost chars in: {:?}",
                input,
                result
            );
        }
    }

    /// Wrapped list-item body that spans multiple visual lines (same
    /// /LI struct elem, same block_id, same struct_role=ListItemBody)
    /// must NOT emit a fresh `- ` bullet on the second visual line.
    /// The break should fire on a list-item *transition*, not on the
    /// mere presence of a list role.
    #[test]
    fn test_wrapped_list_item_body_does_not_emit_extra_bullet() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut a = make_span("First half of an item", 0.0, 100.0, 12.0, FontWeight::Normal);
        a.struct_role = Some(StructRole::ListItemBody);
        a.block_id = Some(7);
        let mut b = make_span("that wraps to next line.", 0.0, 86.0, 12.0, FontWeight::Normal);
        b.struct_role = Some(StructRole::ListItemBody);
        b.block_id = Some(7);
        let result = converter.convert(&[a, b], &config).unwrap();
        let bullet_lines: Vec<&str> = result.lines().filter(|l| l.starts_with("- ")).collect();
        assert_eq!(
            bullet_lines.len(),
            1,
            "wrapped list item body must stay one bullet, got {} lines:\n{}",
            bullet_lines.len(),
            result
        );
    }

    /// D7-fix RED — Hebrew text already in logical Unicode order
    /// (pdfium hebrew_mirrored.pdf shape) must NOT be reversed by
    /// the markdown converter. Reproduces the v0.3.36 regression
    /// where `בנימין` (logical) became `ןימינב` (reversed) after
    /// the unconditional bidi reorder pass.
    #[test]
    fn test_logical_hebrew_passes_through_unchanged() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let span = make_span("בנימין", 0.0, 100.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[span], &config).unwrap();
        assert!(result.contains("בנימין"), "Hebrew must survive intact; got: {:?}", result);
        assert!(
            !result.contains("ןימינב"),
            "must NOT contain reversed Hebrew; got: {:?}",
            result
        );
    }

    /// D7-fix RED — Arabic heading line must keep `#` at the start
    /// after the converter runs. Reproduces the
    /// pdfs_pdfjs/ArabicCIDTrueType.pdf regression where `# ﺔﻴﺑﺮﻌﻟا`
    /// became `ﺔﻴﺑﺮﻌﻟا #` (hash moved to the end).
    #[test]
    fn test_arabic_heading_keeps_hash_at_start() {
        let converter = MarkdownOutputConverter::new();
        let mut config = TextPipelineConfig::default();
        config.output.detect_headings = true;
        let mut h = make_span("ﺔﻴﺑﺮﻌﻟا", 0.0, 100.0, 24.0, FontWeight::Bold);
        h.struct_role = Some(StructRole::Heading(1));
        let result = converter.convert(&[h], &config).unwrap();
        for line in result.lines() {
            if line.contains("ﺔﻴﺑﺮﻌﻟا") {
                assert!(
                    line.trim_start().starts_with('#'),
                    "heading line must start with `#`, got: {:?}",
                    line
                );
            }
        }
    }

    /// D5d RED — IA_0047 reproducer. The struct tree emits the last
    /// span of one column ("constitution" at x=976.7) immediately
    /// followed by the first span of the next column ("Assailing" at
    /// x=192.6) at the SAME baseline (y diff ≈ 1.5pt). A naive
    /// converter joins these into "constitutionAssailing".
    #[test]
    fn test_backward_x_wrap_at_same_baseline_splits_paragraph() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, x: f32, y: f32| make_span(t, x, y, 12.0, FontWeight::Normal);
        // Mirrors IA_0047 spans 1677 → 1678 (column wrap on same line).
        let prev = mk("constitution", 976.7, 1013.2);
        let cur = mk("Assailing", 192.6, 1011.7);
        let result = converter.convert(&[prev, cur], &config).unwrap();
        assert!(
            !result.contains("constitutionAssailing"),
            "column wrap created concatenation, got:\n{}",
            result
        );
        // Both words must be present, on different paragraphs.
        let paras: Vec<&str> = result
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        assert!(
            paras.len() >= 2,
            "expected ≥2 paragraphs from column wrap, got {} in:\n{}",
            paras.len(),
            result
        );
        assert!(result.contains("constitution"));
        assert!(result.contains("Assailing"));
    }

    /// D5d coverage — minor x backwards (≤ 2× font_size) is NOT a
    /// column wrap. Could happen with tight kerning, italic
    /// overhang, or the existing dedup code emitting near-duplicate
    /// glyphs. Must NOT be promoted to a paragraph break.
    #[test]
    fn test_minor_x_backwards_within_tolerance_does_not_split() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // 12pt font, x backs up by only 8pt (< 2 × 12 = 24pt).
        let prev = make_span("hello", 100.0, 100.0, 12.0, FontWeight::Normal);
        let cur = make_span("world", 92.0, 100.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[prev, cur], &config).unwrap();
        let paras: Vec<&str> = result
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        assert_eq!(paras.len(), 1, "minor backstep must stay on one paragraph: {:?}", result);
    }

    /// D5d coverage — the same backward-wrap detector fires when
    /// block_ids are present (IA_0047 tagged paths) AND when no
    /// block_ids are present (untagged multi-column docs).
    #[test]
    fn test_backward_x_wrap_works_with_or_without_block_id() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        for assign_block in [false, true] {
            let mut a = make_span("end of col1", 800.0, 100.0, 12.0, FontWeight::Normal);
            let mut b = make_span("Start of col2", 100.0, 100.0, 12.0, FontWeight::Normal);
            if assign_block {
                a.block_id = Some(1);
                b.block_id = Some(2);
            }
            let result = converter.convert(&[a, b], &config).unwrap();
            assert!(
                !result.contains("col1Start"),
                "block_id={}: column wrap concat in:\n{}",
                assign_block,
                result
            );
        }
    }

    /// D5d coverage — backward wrap on different baselines should
    /// also produce a paragraph break (defensive: even if same_line
    /// is false, a backwards x indicates layout boundary).
    #[test]
    fn test_backward_x_wrap_on_different_baseline() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Mimics column wrap with different baselines (column 1
        // bottom at y=200, column 2 top at y=600). is_paragraph_break
        // catches this via the gap heuristic, but we ensure the
        // backward-x detector does too as a safety net.
        let prev = make_span("col1 last", 800.0, 200.0, 12.0, FontWeight::Normal);
        let cur = make_span("Col2 first", 100.0, 600.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[prev, cur], &config).unwrap();
        assert!(!result.contains("lastCol2"));
    }

    /// D5d coverage — the exact pattern of all 5 regressions found in
    /// IA_0047_20200204: lowercase end + uppercase start, same y,
    /// negative x delta. Each must split into separate paragraphs.
    #[test]
    fn test_all_five_ia_0047_patterns_split() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Each tuple is (col1-end-word, col2-start-word, y, font_size).
        let patterns: &[(&str, &str, f32, f32)] = &[
            ("constitution", "Assailing", 1013.0, 12.0),
            ("harvesting", "Senator", 1162.0, 12.0),
            ("humoro", "Spartacus", 950.0, 11.0),
            ("posscssec", "France", 800.0, 12.0),
            ("should", "Satisfy", 600.0, 12.0),
        ];
        for (a, b, y, sz) in patterns {
            let prev = make_span(a, 800.0, *y, *sz, FontWeight::Normal);
            let cur = make_span(b, 150.0, *y - 1.0, *sz, FontWeight::Normal);
            let result = converter.convert(&[prev, cur], &config).unwrap();
            let joined = format!("{}{}", a, b);
            assert!(
                !result.contains(&joined),
                "pattern {:?}+{:?} created `{}` in:\n{}",
                a,
                b,
                joined,
                result
            );
        }
    }

    /// D5d coverage — column-wrap detector composes with D5b form
    /// fix. A form heading split into pieces on the same baseline
    /// (small forward gaps) still joins; only when the gap is
    /// genuinely a column boundary (large forward OR backward) does
    /// it split.
    #[test]
    fn test_column_wrap_does_not_break_form_heading_join() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, x: f32, bid: u32| {
            let mut s = make_span(t, x, 100.0, 18.0, FontWeight::Bold);
            s.struct_role = Some(StructRole::Heading(1));
            s.block_id = Some(bid);
            s
        };
        // All forward, small gaps.
        let spans = vec![
            mk("Form", 0.0, 1),
            mk("1040", 35.0, 2),
            mk("Title", 80.0, 3),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        let heading_lines: Vec<&str> = result.lines().filter(|l| l.starts_with("# ")).collect();
        assert_eq!(heading_lines.len(), 1, "form heading still joins: {}", result);
    }

    /// D5d unit — the helper itself. Property-style: matrix of
    /// gap/baseline/font shapes covering positive, zero, small
    /// negative, large negative, large positive.
    #[test]
    fn test_is_column_gap_matrix() {
        // (prev_x, prev_w, cur_x, font, expected)
        let cases: &[(f32, f32, f32, f32, bool)] = &[
            // Word gap inside a normal sentence: prev=Hello (50w) → cur="world".
            (100.0, 50.0, 154.0, 12.0, false),
            // Right at the 3× threshold: 36pt forward gap.
            (100.0, 50.0, 186.5, 12.0, true),
            // Far below threshold.
            (100.0, 50.0, 160.0, 12.0, false),
            // Backward 30pt at 12pt font (>2x = 24pt threshold).
            (200.0, 50.0, 100.0, 12.0, true),
            // Backward 8pt at 12pt font (under 24pt threshold).
            (100.0, 50.0, 92.0, 12.0, false),
            // Newspaper case: x=976→x=192 with 12pt font.
            (976.7, 37.8, 192.6, 12.0, true),
        ];
        for (px, pw, cx, font, expected) in cases {
            let prev = make_span("p", *px, 100.0, *font, FontWeight::Normal);
            let mut prev = prev;
            prev.span.bbox.width = *pw;
            let cur = make_span("c", *cx, 100.0, *font, FontWeight::Normal);
            let actual = is_column_gap(&prev, &cur);
            assert_eq!(
                actual, *expected,
                "(px={}, pw={}, cx={}, font={}) expected {} got {}",
                px, pw, cx, font, expected, actual
            );
        }
    }

    /// A backward-X transition is a column boundary ONLY when it is not an
    /// ordinary within-column line wrap. A wrap drops by ~one line; a column
    /// jump goes up, stays on the same baseline, or drops far more.
    #[test]
    fn test_is_column_gap_distinguishes_wrap_from_column_jump() {
        let font = 13.0;
        let mk = |x: f32, y: f32| {
            let mut s = make_span("w", x, y, font, FontWeight::Normal);
            s.span.bbox.width = 40.0;
            s
        };
        // Within-column wrap: X resets to the left margin, Y steps down one
        // line (~13.5pt). NOT a column boundary.
        assert!(!is_column_gap(&mk(285.0, 555.8), &mk(229.0, 542.2)));
        // Same-baseline column transition (end of col 1 → top-ish of col 2 at
        // the same Y, y_drop ≈ 0). IS a column boundary.
        assert!(is_column_gap(&mk(976.7, 1013.2), &mk(192.6, 1011.7)));
        // Column jump UP (next column resumes far above). IS a column boundary.
        assert!(is_column_gap(&mk(500.0, 100.0), &mk(72.0, 700.0)));
    }

    /// Right-to-left flow steps left on every word, including across the space
    /// spans between words — none of those steps is a column boundary, or an
    /// Arabic/Hebrew line shatters into one paragraph per word.
    #[test]
    fn test_is_column_gap_rtl_word_and_space_steps_are_not_columns() {
        let mk = |t: &str, x: f32| {
            let mut s = make_span(t, x, 700.0, 13.0, FontWeight::Normal);
            s.span.bbox.width = 30.0;
            s
        };
        // Hebrew word (right) → space → Hebrew word (further left), same line.
        let word_r = mk("\u{05D0}\u{05D1}", 300.0);
        let space = mk(" ", 290.0);
        let word_l = mk("\u{05D2}\u{05D3}", 250.0);
        assert!(!is_column_gap(&word_r, &space), "RTL word→space read as a column gap");
        assert!(!is_column_gap(&space, &word_l), "RTL space→word read as a column gap");
    }

    /// D5c RED — multi-column newspaper case. Two text spans on the
    /// same baseline but in different columns (large horizontal gap
    /// between the right edge of the previous span and the left edge
    /// of the current one), with different structure-tree block_ids.
    /// D5b would join them on one line and produce concatenated
    /// gibberish like `andmight`. The column-gap detector must split
    /// them into two paragraphs.
    #[test]
    fn test_column_gap_with_block_change_splits() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Column 1: "and" at x=0, width 30, baseline 100.
        // Column 2: "might" at x=180 (column gutter ≈ 150pt), baseline 100.
        // Body font 12pt, so the gap is well over 3× font_size.
        let mut col1 = make_span("and", 0.0, 100.0, 12.0, FontWeight::Normal);
        col1.block_id = Some(1);
        let mut col2 = make_span("might", 180.0, 100.0, 12.0, FontWeight::Normal);
        col2.block_id = Some(2);
        let result = converter.convert(&[col1, col2], &config).unwrap();
        // The two tokens must NOT be joined into `andmight`.
        assert!(
            !result.contains("andmight"),
            "column-gap join produced concatenated token, got:\n{}",
            result
        );
        // They must appear as separate words on separate lines or with
        // a paragraph break between them.
        assert!(result.contains("and"));
        assert!(result.contains("might"));
        // No `and might` glued onto one heading or paragraph either —
        // we want the two columns rendered as separate paragraphs.
        let paras: Vec<&str> = result
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        assert!(
            paras.len() >= 2,
            "expected ≥2 paragraphs separated by column gap, got {} in:\n{}",
            paras.len(),
            result
        );
    }

    /// D5c coverage — same-baseline pieces of a tagged form heading
    /// (small inline gap, different block_ids) must still JOIN even
    /// after the column-gap detector. Regression guard for D5b.
    #[test]
    fn test_form_heading_inline_gap_still_joins() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // `Form` ends at x≈30, `1040` starts at x≈40 — small inline
        // gap (≈10pt, well under 3× font_size = 54pt for 18pt heading).
        let mk = |t: &str, x: f32, bid: u32| {
            let mut s = make_span(t, x, 100.0, 18.0, FontWeight::Bold);
            s.struct_role = Some(StructRole::Heading(1));
            s.block_id = Some(bid);
            s
        };
        let spans = vec![
            mk("Form", 0.0, 1),
            mk("1040", 40.0, 2),
            mk("U.S.", 100.0, 3),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        let heading_lines: Vec<&str> = result.lines().filter(|l| l.starts_with("# ")).collect();
        assert_eq!(
            heading_lines.len(),
            1,
            "small-gap form pieces must stay on one heading line, got:\n{}",
            result
        );
    }

    /// D5c coverage — boundary case: a moderate gap (e.g. 2× font
    /// size, like a wide indent or cell separator) should NOT trigger
    /// column split. Only truly large gaps (multi-column gutter)
    /// trigger the break.
    #[test]
    fn test_moderate_gap_does_not_force_column_break() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Body 12pt, gap of 24pt (2× font_size) — wide indent but not
        // a column gutter.
        let mut a = make_span("First field", 0.0, 100.0, 12.0, FontWeight::Normal);
        a.block_id = Some(1);
        let mut b = make_span("Second field", 80.0, 100.0, 12.0, FontWeight::Normal);
        b.block_id = Some(2);
        // The gap from x=0+50 (text "First field" width=50 in make_span) to x=80 = 30pt = 2.5× font_size.
        // Just below the column-gap threshold (3× = 36pt).
        let result = converter.convert(&[a, b], &config).unwrap();
        let paras: Vec<&str> = result
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        assert_eq!(
            paras.len(),
            1,
            "moderate gap (≈2.5× font) must keep content on one paragraph, got:\n{}",
            result
        );
    }

    /// D5c coverage — three columns at the same baseline with large
    /// gaps must split into three paragraphs.
    #[test]
    fn test_three_column_layout_splits_into_three_paragraphs() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, x: f32, bid: u32| {
            let mut s = make_span(t, x, 100.0, 12.0, FontWeight::Normal);
            s.block_id = Some(bid);
            s
        };
        // Three 12pt-body columns at x=0, 200, 400 (gaps of ~150pt).
        let spans = vec![
            mk("col one", 0.0, 1),
            mk("col two", 200.0, 2),
            mk("col three", 400.0, 3),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        let paras: Vec<&str> = result
            .split("\n\n")
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .collect();
        assert_eq!(paras.len(), 3, "three columns must produce three paragraphs, got:\n{}", result);
    }

    /// D5c coverage — column-gap detector applies even when no
    /// block_id is set (untagged document with multi-column layout).
    /// Without this, untagged newspapers would also produce
    /// `andmight`-style joins.
    #[test]
    fn test_column_gap_without_block_id_still_splits() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // No block_id assigned (untagged).
        let a = make_span("left column.", 0.0, 100.0, 12.0, FontWeight::Normal);
        let b = make_span("right column.", 200.0, 100.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[a, b], &config).unwrap();
        // Pre-existing geometric heuristics should split too via the
        // group_id / has_horizontal_gap logic — verify the combined
        // result keeps the two columns as separate words at minimum.
        assert!(
            result.contains("left column") && result.contains("right column"),
            "both columns must surface, got:\n{}",
            result
        );
        // No concatenation across the gap.
        assert!(
            !result.contains("column.right"),
            "must not concatenate across column gap, got:\n{}",
            result
        );
    }

    /// D5b coverage — three-piece headings with a TINY (<1pt) y
    /// jitter still considered same-line. Forms often have minute
    /// baseline jitter due to font metric variation; the gate must be
    /// tolerant.
    #[test]
    fn test_minor_baseline_jitter_still_joins() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, x: f32, y: f32, bid: u32| {
            let mut s = make_span(t, x, y, 18.0, FontWeight::Bold);
            s.struct_role = Some(StructRole::Heading(1));
            s.block_id = Some(bid);
            s
        };
        // y values jitter within 0.5pt — well within the same_line
        // threshold (font_size * 0.5 = 9pt for an 18pt heading).
        let spans = vec![
            mk("A", 0.0, 100.0, 1),
            mk("B", 30.0, 100.3, 2),
            mk("C", 60.0, 99.7, 3),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        let heading_lines: Vec<&str> = result.lines().filter(|l| l.starts_with("# ")).collect();
        assert_eq!(heading_lines.len(), 1, "tiny jitter must not split heading, got:\n{}", result);
    }

    /// D5b coverage — large baseline drop (well past same_line) DOES
    /// split, even with same heading_level. Proves the gate isn't
    /// over-suppressing.
    #[test]
    fn test_large_baseline_drop_still_splits_heading() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, y: f32, bid: u32| {
            let mut s = make_span(t, 0.0, y, 18.0, FontWeight::Bold);
            s.struct_role = Some(StructRole::Heading(1));
            s.block_id = Some(bid);
            s
        };
        // 30pt drop between baselines — far beyond `font_size * 0.5`.
        let spans = vec![mk("First Heading", 100.0, 1), mk("Second Heading", 70.0, 2)];
        let result = converter.convert(&spans, &config).unwrap();
        let heading_lines: Vec<&str> = result.lines().filter(|l| l.starts_with("# ")).collect();
        assert_eq!(
            heading_lines.len(),
            2,
            "two visually-separated headings must both surface, got:\n{}",
            result
        );
    }

    /// D5 RED — when adjacent spans carry different `block_id` from
    /// the source PDF's structure tree, force a paragraph break even
    /// when the geometric gap is too small for the
    /// `paragraph_gap_ratio` heuristic. Reproduces the pdfa_049
    /// pattern where two body-sized paragraphs sit ~14pt apart on a
    /// 12pt body and our 1.5× heuristic (16.5pt threshold) merges
    /// them. Tagged structure tree gives us authoritative paragraph
    /// boundaries via `OrderedContent.block_id`.
    #[test]
    fn test_block_id_change_forces_paragraph_break() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Two paragraphs separated by 12pt (less than 1.5× line_height).
        let mut p1 = make_span("Paragraph one body text.", 0.0, 100.0, 12.0, FontWeight::Normal);
        p1.block_id = Some(1);
        let mut p2 = make_span("Paragraph two starts here.", 0.0, 88.0, 12.0, FontWeight::Normal);
        p2.block_id = Some(2);
        let result = converter.convert(&[p1, p2], &config).unwrap();
        assert!(
            result.contains("Paragraph one body text.\n\nParagraph two starts here."),
            "expected double newline between block_ids 1→2, got:\n{:?}",
            result
        );
    }

    /// D5 RED (negative) — same `block_id` keeps spans on the same
    /// logical paragraph, even on different baselines (line wrap
    /// inside one /P struct elem).
    #[test]
    fn test_same_block_id_keeps_paragraph_continuous() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut l1 = make_span("first line", 0.0, 100.0, 12.0, FontWeight::Normal);
        l1.block_id = Some(7);
        let mut l2 = make_span("second line", 0.0, 88.0, 12.0, FontWeight::Normal);
        l2.block_id = Some(7);
        let result = converter.convert(&[l1, l2], &config).unwrap();
        // No blank line between them.
        assert!(
            !result.contains("\n\n"),
            "same block_id must not introduce paragraph break, got:\n{:?}",
            result
        );
    }

    /// D6 RED — a small superscript span (≤4 chars, fontSize < 0.7× the
    /// preceding span) on a slightly raised baseline (PDF Ts/text-rise,
    /// spec §9.4.3) must merge into the same logical line as the body
    /// text instead of becoming its own paragraph. Reproduces the
    /// `21st → "21" + bare "st"` corruption visible in nougat_002 and
    /// the `23rd Street → "23" + "rd Street"` split visible in
    /// nougat_011 line 43.
    #[test]
    fn test_superscript_text_rise_does_not_split_line() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Body baseline at y=100 with 11pt body font.
        let pre = make_span("On June 21", 0.0, 100.0, 11.0, FontWeight::Normal);
        // Superscript "st" raised ~2.5pt with 7pt font (smaller than body).
        let sup = make_span("st", 35.0, 102.5, 7.0, FontWeight::Normal);
        let post = make_span(" they met.", 42.0, 100.0, 11.0, FontWeight::Normal);
        let result = converter.convert(&[pre, sup, post], &config).unwrap();
        assert!(
            result.contains("21st they met"),
            "expected '21st they met' inline, got:\n{}",
            result
        );
        assert!(
            !result.lines().any(|l| l.trim() == "st"),
            "no bare 'st' line allowed, got:\n{}",
            result
        );
    }

    /// D1 RED — list item body MCRs must emit a bullet on a new line.
    /// Reproduces the word365_structure / nougat_037 pattern where
    /// consecutive items collapse into a single line because the
    /// converter sees them as plain spans.
    #[test]
    fn test_struct_role_list_items_emit_bullets() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut items = Vec::new();
        for (i, t) in ["Apple", "Banana", "Cherry"].iter().enumerate() {
            let mut s = make_span(t, 0.0, 100.0 - (i as f32 * 14.0), 12.0, FontWeight::Normal);
            s.struct_role = Some(StructRole::ListItemBody);
            items.push(s);
        }
        let result = converter.convert(&items, &config).unwrap();
        for t in ["- Apple", "- Banana", "- Cherry"] {
            assert!(result.contains(t), "expected `{}` line in output, got:\n{}", t, result);
        }
    }

    fn make_span_w(
        text: &str,
        x: f32,
        y: f32,
        width: f32,
        font_size: f32,
        weight: FontWeight,
    ) -> OrderedTextSpan {
        OrderedTextSpan::new(
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: text.to_string(),
                bbox: Rect::new(x, y, width, font_size),
                font_name: "Test".to_string(),
                font_size,
                font_weight: weight,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                offset_semantic: false,
                split_boundary_before: false,
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
            },
            0,
        )
    }

    fn make_span(
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        weight: FontWeight,
    ) -> OrderedTextSpan {
        OrderedTextSpan::new(
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: text.to_string(),
                bbox: Rect::new(x, y, 50.0, font_size),
                font_name: "Test".to_string(),
                font_size,
                font_weight: weight,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                offset_semantic: false,
                split_boundary_before: false,
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
            },
            0,
        )
    }

    /// WS2.5: a list item whose bullet sits further right than the top-level
    /// markers gets a 2-space indent per level; a flat list is unindented
    /// (byte-identical to before).
    #[test]
    fn md_nested_list_indents_by_marker_x() {
        let config = TextPipelineConfig::default();
        let converter = MarkdownOutputConverter::new();

        let nested = converter
            .convert(
                &[
                    make_span("• Alpha", 50.0, 700.0, 12.0, FontWeight::Normal),
                    make_span("• Beta", 50.0, 686.0, 12.0, FontWeight::Normal),
                    make_span("• Gamma", 80.0, 672.0, 12.0, FontWeight::Normal),
                ],
                &config,
            )
            .unwrap();
        assert!(nested.contains("- Alpha"), "top-level item unindented: {nested:?}");
        assert!(nested.contains("  - Gamma"), "deeper-x item gets a 2-space indent: {nested:?}");

        let flat = converter
            .convert(
                &[
                    make_span("• One", 50.0, 700.0, 12.0, FontWeight::Normal),
                    make_span("• Two", 50.0, 686.0, 12.0, FontWeight::Normal),
                ],
                &config,
            )
            .unwrap();
        assert!(!flat.contains("  - "), "a flat list must not be indented: {flat:?}");
    }

    /// WS2.4: multi-level numbered section titles promote to headings at their
    /// dot-depth; list markers and non-headings do not.
    #[test]
    fn md_numbered_heading_promotion() {
        let f = MarkdownOutputConverter::numbered_heading_level;
        assert_eq!(f("2.1.3 Results and discussion"), Some(3));
        assert_eq!(f("2.1 Method"), Some(2));
        assert_eq!(f("4.2.1.5 Deep"), Some(4)); // 3 dots → level 4
                                                // Not headings:
        assert_eq!(f("1. First list item"), None, "single-dot+space is a list marker");
        assert_eq!(f("3 Discussion"), None, "no dot is too ambiguous");
        assert_eq!(f("Introduction"), None, "no number");
        assert_eq!(f("2.1.3"), None, "number with no title");
        assert_eq!(f("2020.05.13"), None, "date-like, no alpha title");
    }

    // ---- Heading size-ratio promotion: body text must not be over-promoted ----

    #[test]
    fn heading_ratio_does_not_promote_body_text() {
        // A document with a single clear heading (14pt) over 10pt body. The body
        // spans (10pt) must never be promoted to a heading — only the 14pt title,
        // which the size-ratio heuristic recovers.
        let converter = MarkdownOutputConverter::new();
        let mut config = TextPipelineConfig::default();
        config.output.detect_headings = true;
        let mut spans = vec![make_span_w(
            "Title Here",
            0.0,
            800.0,
            100.0,
            14.0,
            FontWeight::Normal,
        )];
        for i in 0..8 {
            spans.push(make_span_w(
                "plain body sentence that is clearly not a heading at all",
                0.0,
                700.0 - (i as f32) * 12.0,
                260.0,
                10.0,
                FontWeight::Normal,
            ));
        }
        let md = converter.convert(&spans, &config).unwrap();
        // No body line should have become a heading (no stray leading '#').
        for line in md.lines() {
            if line.starts_with('#') {
                assert!(
                    line.contains("Title Here"),
                    "body text wrongly promoted to heading: {line:?}"
                );
            }
        }
    }

    // ---- WS2.7 footnote detection ----

    #[test]
    fn footnote_marker_with_matching_bottom_def_emits_reference() {
        // Body text ending in a raised superscript "1" plus a page-bottom
        // definition line "1 Smith et al. 2019" → inline `[^1]` and a
        // trailing `[^1]: …` definition.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span_w("As shown", 0.0, 700.0, 60.0, 11.0, FontWeight::Normal),
            // raised (y 702 > 700) and small (7pt < 0.75×11)
            make_span_w("1", 60.0, 702.0, 5.0, 7.0, FontWeight::Normal),
            // page-bottom definition block (bottom 18% band)
            make_span_w("1", 0.0, 50.0, 5.0, 7.0, FontWeight::Normal),
            make_span_w("Smith et al. 2019", 10.0, 50.0, 120.0, 7.0, FontWeight::Normal),
        ];
        let md = converter.convert(&spans, &config).unwrap();
        assert!(md.contains("As shown[^1]"), "inline reference missing: {md:?}");
        assert!(md.contains("[^1]: Smith et al. 2019"), "definition missing: {md:?}");
    }

    #[test]
    fn footnote_symbol_marker_with_matching_def_emits_sequential_id() {
        // A symbol marker (`*`) is confirmed by a matching bottom def and
        // assigned the sequential id `1`. Also guards the noise-filter: a
        // lone `*` span must survive because it is a confirmed marker.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span_w("See note", 0.0, 700.0, 60.0, 11.0, FontWeight::Normal),
            make_span_w("*", 60.0, 702.0, 5.0, 7.0, FontWeight::Normal),
            make_span_w("*", 0.0, 50.0, 5.0, 7.0, FontWeight::Normal),
            make_span_w("Important caveat", 10.0, 50.0, 100.0, 7.0, FontWeight::Normal),
        ];
        let md = converter.convert(&spans, &config).unwrap();
        assert!(md.contains("See note[^1]"), "inline reference missing: {md:?}");
        assert!(md.contains("[^1]: Important caveat"), "definition missing: {md:?}");
    }

    #[test]
    fn footnote_superscript_without_matching_def_left_unchanged() {
        // A raised superscript "1" whose only bottom block starts with a
        // DIFFERENT token ("2") must not be rewritten — neither half is
        // confirmed, so no footnote syntax is emitted at all.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span_w("As shown", 0.0, 700.0, 60.0, 11.0, FontWeight::Normal),
            make_span_w("1", 60.0, 702.0, 5.0, 7.0, FontWeight::Normal),
            make_span_w("2", 0.0, 50.0, 5.0, 7.0, FontWeight::Normal),
            make_span_w("Unrelated note", 10.0, 50.0, 100.0, 7.0, FontWeight::Normal),
        ];
        let md = converter.convert(&spans, &config).unwrap();
        assert!(!md.contains("[^"), "unexpected footnote syntax: {md:?}");
        assert!(md.contains("As shown"), "body text lost: {md:?}");
    }

    #[test]
    fn footnote_ordinal_superscript_not_treated_as_footnote() {
        // "May 5th" with a superscript ordinal "th" must never be treated
        // as a footnote: the token is letters, not a digit/symbol, so it
        // is never even a candidate marker.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span_w("May", 0.0, 700.0, 30.0, 11.0, FontWeight::Normal),
            make_span_w("5", 30.0, 700.0, 8.0, 11.0, FontWeight::Normal),
            make_span_w("th", 38.0, 702.0, 8.0, 7.0, FontWeight::Normal),
            // a bottom block that would confirm a real footnote is present
            // but cannot match a letter-token "th".
            make_span_w("1", 0.0, 50.0, 5.0, 7.0, FontWeight::Normal),
            make_span_w("Footer note", 10.0, 50.0, 90.0, 7.0, FontWeight::Normal),
        ];
        let md = converter.convert(&spans, &config).unwrap();
        assert!(!md.contains("[^"), "ordinal wrongly footnoted: {md:?}");
    }

    #[test]
    fn md_wrapped_hyphen_line_joins_without_space() {
        // A line that wraps mid-word at a hyphen must join WITHOUT a space:
        // `frozen-` + `thawed` → `frozen-thawed`, never `frozen- thawed`.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span_w("frozen-", 0.0, 100.0, 40.0, 10.0, FontWeight::Normal),
            make_span_w("thawed", 0.0, 89.0, 40.0, 10.0, FontWeight::Normal),
        ];
        let md = converter.convert(&spans, &config).unwrap();
        assert!(md.contains("frozen-thawed"), "not joined: {md:?}");
        assert!(!md.contains("frozen- thawed"), "spurious space after hyphen: {md:?}");
    }

    #[test]
    fn md_wrapped_plain_line_keeps_space() {
        // Guard: a normal (non-hyphen) wrap still gets its joining space.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span_w("hello", 0.0, 100.0, 40.0, 10.0, FontWeight::Normal),
            make_span_w("world", 0.0, 89.0, 40.0, 10.0, FontWeight::Normal),
        ];
        let md = converter.convert(&spans, &config).unwrap();
        assert!(md.contains("hello world"), "lost joining space: {md:?}");
    }

    #[test]
    fn md_rtl_interword_punct_separator_is_kept_and_not_paragraph_broken() {
        // A right-to-left line emits the comma/period between two words as its
        // own space-bearing span (" ,"). It must (a) survive the noise filter
        // so the flanking words are NOT glued, and (b) not be treated as a
        // same-baseline column gap that shatters the line into one paragraph
        // per word. Mirrors the wiki-cat-he front-matter line
        // `…הטורפים, ממשפחת…`.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // RTL reading order steps left (decreasing x), same baseline.
        let spans = vec![
            make_span_w("הטורפים", 200.0, 100.0, 60.0, 13.0, FontWeight::Normal),
            make_span_w(" ,", 190.0, 100.0, 7.0, 13.0, FontWeight::Normal),
            make_span_w("ממשפחת", 130.0, 100.0, 55.0, 13.0, FontWeight::Normal),
        ];
        let md = converter.convert(&spans, &config).unwrap();
        assert!(md.contains("הטורפים"), "lost first word: {md:?}");
        assert!(md.contains("ממשפחת"), "lost second word: {md:?}");
        assert!(!md.contains("הטורפיםממשפחת"), "words glued — separator was dropped: {md:?}");
        let para_count = md
            .trim()
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count();
        assert_eq!(para_count, 1, "RTL line shattered into paragraphs: {md:?}");
    }

    #[test]
    fn test_empty_spans() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let result = converter.convert(&[], &config).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_single_span() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![make_span(
            "Hello world",
            0.0,
            100.0,
            12.0,
            FontWeight::Normal,
        )];
        let result = converter.convert(&spans, &config).unwrap();
        assert_eq!(result, "Hello world\n");
    }

    #[test]
    fn test_bold_text() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![make_span("Bold text", 0.0, 100.0, 12.0, FontWeight::Bold)];
        let result = converter.convert(&spans, &config).unwrap();
        assert_eq!(result, "**Bold text**\n");
    }

    #[test]
    fn test_whitespace_bold_conservative() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Whitespace-only bold should not have markers in conservative mode
        let spans = vec![make_span("   ", 0.0, 100.0, 12.0, FontWeight::Bold)];
        let result = converter.convert(&spans, &config).unwrap();
        // Should not contain bold markers
        assert!(!result.contains("**"));
    }

    #[test]
    fn test_convert_with_tables_renders_markdown_table() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));
        table.col_count = 2;
        table.has_header = true;

        let mut header = TableRow::new(true);
        header.add_cell(TableCell::new("Name".to_string(), true));
        header.add_cell(TableCell::new("Value".to_string(), true));
        table.add_row(header);

        let mut data = TableRow::new(false);
        data.add_cell(TableCell::new("A".to_string(), false));
        data.add_cell(TableCell::new("1".to_string(), false));
        table.add_row(data);

        let result = converter
            .convert_with_tables(&[], &[table], &config)
            .unwrap();

        assert!(result.contains("| Name |"));
        assert!(result.contains("| Value |"));
        assert!(result.contains("---|"));
        assert!(result.contains("| A |"));
        assert!(result.contains("| 1 |"));
    }

    // ============================================================================
    // render_table_markdown() tests
    // ============================================================================

    #[test]
    fn test_render_table_markdown_empty() {
        let table = Table::new();
        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());
        assert_eq!(result, "");
    }

    #[test]
    fn test_render_table_markdown_single_row_no_header() {
        let mut table = Table::new();
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("A".to_string(), false));
        row.add_cell(TableCell::new("B".to_string(), false));
        table.add_row(row);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());
        assert!(result.contains("| A |"));
        assert!(result.contains("| B |"));
        // First row treated as header by default in markdown
        assert!(result.contains("---|"));
    }

    #[test]
    fn test_render_table_markdown_with_colspan() {
        let mut table = Table::new();
        table.has_header = true;
        let mut header = TableRow::new(true);
        header.add_cell(TableCell::new("Wide".to_string(), true).with_colspan(2));
        table.add_row(header);

        let mut data = TableRow::new(false);
        data.add_cell(TableCell::new("Left".to_string(), false));
        data.add_cell(TableCell::new("Right".to_string(), false));
        table.add_row(data);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());
        // Colspan cell should produce extra | separators
        assert!(result.contains("| Wide |"));
        assert!(result.contains("---|---|"));
    }

    #[test]
    fn test_render_table_markdown_escapes_pipes() {
        let mut table = Table::new();
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("A|B".to_string(), false));
        table.add_row(row);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());
        assert!(result.contains("A\\|B"), "Pipes should be backslash-escaped: {}", result);
    }

    #[test]
    fn test_render_table_markdown_replaces_newlines() {
        let mut table = Table::new();
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("Line1\nLine2".to_string(), false));
        table.add_row(row);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());
        assert!(!result.contains("Line1\nLine2"), "Newlines in cells should be replaced");
        assert!(result.contains("Line1 Line2"));
    }

    #[test]
    fn test_render_table_markdown_trims_whitespace() {
        let mut table = Table::new();
        let mut row = TableRow::new(false);
        row.add_cell(TableCell::new("  padded  ".to_string(), false));
        table.add_row(row);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());
        assert!(result.contains("| padded |"));
    }

    #[test]
    fn test_render_table_markdown_multiple_header_rows() {
        let mut table = Table::new();
        table.has_header = true;

        let mut h1 = TableRow::new(true);
        h1.add_cell(TableCell::new("H1".to_string(), true));
        table.add_row(h1);

        let mut h2 = TableRow::new(true);
        h2.add_cell(TableCell::new("H2".to_string(), true));
        table.add_row(h2);

        let mut d1 = TableRow::new(false);
        d1.add_cell(TableCell::new("D1".to_string(), false));
        table.add_row(d1);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());
        // Separator should appear after last header row (row_idx == 1)
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4); // H1, H2, separator, D1
        assert!(lines[2].contains("---|"));
    }

    // ============================================================================
    // span_in_table() tests
    // ============================================================================

    #[test]
    fn test_span_in_table_match() {
        let span = make_span("text", 50.0, 70.0, 12.0, FontWeight::Normal);

        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));

        assert_eq!(span_in_table(&span, &[table]), Some(0));
    }

    #[test]
    fn test_span_in_table_no_match() {
        let span = make_span("text", 500.0, 500.0, 12.0, FontWeight::Normal);

        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));

        assert_eq!(span_in_table(&span, &[table]), None);
    }

    #[test]
    fn test_span_in_table_none_bbox() {
        let span = make_span("text", 50.0, 70.0, 12.0, FontWeight::Normal);

        let table = Table::new(); // No bbox
        assert_eq!(span_in_table(&span, &[table]), None);
    }

    #[test]
    fn test_span_in_table_tolerance() {
        // Span at bbox edge minus tolerance (2.0)
        let span = make_span("text", 8.5, 48.5, 12.0, FontWeight::Normal);

        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));

        assert_eq!(span_in_table(&span, &[table]), Some(0), "Should match within tolerance");
    }

    #[test]
    fn test_span_in_table_multiple_tables() {
        let span = make_span("text", 350.0, 70.0, 12.0, FontWeight::Normal);

        let mut t1 = Table::new();
        t1.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));

        let mut t2 = Table::new();
        t2.bbox = Some(Rect::new(300.0, 50.0, 200.0, 100.0));

        assert_eq!(span_in_table(&span, &[t1, t2]), Some(1));
    }

    // ============================================================================
    // convert_with_tables() integration tests
    // ============================================================================

    #[test]
    fn test_convert_with_tables_mixed_content() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        // Text before the table
        let mut span_before = make_span("Before table", 10.0, 200.0, 12.0, FontWeight::Normal);
        span_before.reading_order = 0;

        // Text after the table (lower Y = later in reading order)
        let mut span_after = make_span("After table", 10.0, 20.0, 12.0, FontWeight::Normal);
        span_after.reading_order = 2;

        // Text inside table region whose text matches table cell content
        // (not an orphan — absorbed by the table rendering).
        let mut span_in_table = make_span("Val", 50.0, 70.0, 12.0, FontWeight::Normal);
        span_in_table.reading_order = 1;

        let mut table = Table::new();
        table.bbox = Some(Rect::new(10.0, 50.0, 200.0, 100.0));
        table.has_header = true;
        let mut header = TableRow::new(true);
        header.add_cell(TableCell::new("Col".to_string(), true));
        table.add_row(header);
        let mut data = TableRow::new(false);
        data.add_cell(TableCell::new("Val".to_string(), false));
        table.add_row(data);

        let result = converter
            .convert_with_tables(&[span_before, span_in_table, span_after], &[table], &config)
            .unwrap();

        assert!(result.contains("Before table"), "Should contain text before table");
        assert!(result.contains("| Col |"), "Should contain table");
        assert!(result.contains("After table"), "Should contain text after table");
    }

    #[test]
    fn test_convert_with_tables_no_tables_is_same_as_convert() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![make_span("Hello", 0.0, 100.0, 12.0, FontWeight::Normal)];

        let result_convert = converter.convert(&spans, &config).unwrap();
        let result_with_tables = converter.convert_with_tables(&spans, &[], &config).unwrap();

        assert_eq!(result_convert, result_with_tables);
    }

    #[test]
    fn test_convert_with_tables_multiple_tables() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        let make_table = |x: f32, text: &str| -> Table {
            let mut t = Table::new();
            t.bbox = Some(Rect::new(x, 50.0, 100.0, 50.0));
            let mut row = TableRow::new(false);
            row.add_cell(TableCell::new(text.to_string(), false));
            t.add_row(row);
            t
        };

        let result = converter
            .convert_with_tables(&[], &[make_table(10.0, "T1"), make_table(200.0, "T2")], &config)
            .unwrap();

        assert!(result.contains("| T1 |"), "Should contain first table");
        assert!(result.contains("| T2 |"), "Should contain second table");
    }

    // ============================================================================
    // Issue #182: Bullet detection tests
    // ============================================================================

    #[test]
    fn test_is_bullet_span() {
        assert!(MarkdownOutputConverter::is_bullet_span("►"));
        assert!(MarkdownOutputConverter::is_bullet_span("•"));
        assert!(MarkdownOutputConverter::is_bullet_span("▪"));
        assert!(MarkdownOutputConverter::is_bullet_span(" ► "));
        assert!(!MarkdownOutputConverter::is_bullet_span("text"));
        assert!(!MarkdownOutputConverter::is_bullet_span("►text"));
        assert!(!MarkdownOutputConverter::is_bullet_span(""));
    }

    #[test]
    fn test_starts_with_bullet() {
        assert!(MarkdownOutputConverter::starts_with_bullet("►text"));
        assert!(MarkdownOutputConverter::starts_with_bullet("• item"));
        assert!(MarkdownOutputConverter::starts_with_bullet("  ► indented"));
        assert!(!MarkdownOutputConverter::starts_with_bullet("text"));
        assert!(!MarkdownOutputConverter::starts_with_bullet(""));
    }

    #[test]
    fn test_strip_bullet() {
        assert_eq!(MarkdownOutputConverter::strip_bullet("► text"), "text");
        assert_eq!(MarkdownOutputConverter::strip_bullet("•item"), "item");
        assert_eq!(MarkdownOutputConverter::strip_bullet("no bullet"), "no bullet");
    }

    #[test]
    fn test_bullet_spans_become_list_items() {
        // Simulates: ► (separate span) + "Analog input" (next span, same Y)
        // on a new line from previous content
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        let mut title = make_span("FEATURES", 50.0, 660.0, 11.0, FontWeight::Bold);
        title.reading_order = 0;

        let mut bullet = make_span("►", 50.0, 640.0, 8.8, FontWeight::Normal);
        bullet.reading_order = 1;

        let mut text = make_span("Analog input", 60.0, 640.0, 11.0, FontWeight::Normal);
        text.reading_order = 2;

        let mut bullet2 = make_span("►", 50.0, 626.0, 8.8, FontWeight::Normal);
        bullet2.reading_order = 3;

        let mut text2 = make_span("16-bit ADC", 60.0, 626.0, 11.0, FontWeight::Normal);
        text2.reading_order = 4;

        let spans = vec![title, bullet, text, bullet2, text2];
        let result = converter.convert(&spans, &config).unwrap();

        assert!(
            result.contains("- Analog input"),
            "Should convert bullet to list item: {}",
            result
        );
        assert!(result.contains("- 16-bit ADC"), "Should convert second bullet: {}", result);
        assert!(!result.contains("►"), "Should not contain raw bullet character: {}", result);
    }

    #[test]
    fn test_inline_bullet_becomes_list_item() {
        // Simulates: "► Analog input" as a single span (inline bullet)
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        let mut title = make_span("TITLE", 50.0, 660.0, 11.0, FontWeight::Bold);
        title.reading_order = 0;

        let mut bullet_text = make_span("► Analog input", 50.0, 640.0, 11.0, FontWeight::Normal);
        bullet_text.reading_order = 1;

        let spans = vec![title, bullet_text];
        let result = converter.convert(&spans, &config).unwrap();

        assert!(
            result.contains("- Analog input"),
            "Should convert inline bullet to list item: {}",
            result
        );
    }

    #[test]
    fn test_first_span_inline_bullet() {
        // First span on page starts with bullet — no prev_span exists.
        // Should still be converted to a markdown list item.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        let mut bullet_text = make_span("► First item", 50.0, 660.0, 11.0, FontWeight::Normal);
        bullet_text.reading_order = 0;

        let mut bullet_text2 = make_span("► Second item", 50.0, 646.0, 11.0, FontWeight::Normal);
        bullet_text2.reading_order = 1;

        let spans = vec![bullet_text, bullet_text2];
        let result = converter.convert(&spans, &config).unwrap();

        assert!(
            result.contains("- First item"),
            "First-span inline bullet should become list item: {}",
            result
        );
        assert!(
            result.contains("- Second item"),
            "Second inline bullet should become list item: {}",
            result
        );
    }

    // ============================================================================
    // Issue #182: Heading over-detection prevention
    // ============================================================================

    fn config_with_headings() -> TextPipelineConfig {
        let mut config = TextPipelineConfig::default();
        config.output.detect_headings = true;
        config
    }

    #[test]
    fn test_heading_base_font_excludes_small_spans() {
        // When page has many 8.8pt ► spans, the base font size should
        // still be ~11pt (excluding small spans), not 8.8pt
        let converter = MarkdownOutputConverter::new();
        let config = config_with_headings();

        let mut spans = Vec::new();
        let mut order = 0;

        // 10 bullet spans at 8.8pt (should be excluded from median)
        for i in 0..10 {
            let mut s = make_span("►", 50.0, 600.0 - (i as f32) * 14.0, 8.8, FontWeight::Normal);
            s.reading_order = order;
            order += 1;
            spans.push(s);
        }

        // 10 text spans at 11pt (should be the median)
        for i in 0..10 {
            let mut s = make_span(
                "body text content",
                60.0,
                600.0 - (i as f32) * 14.0,
                11.0,
                FontWeight::Bold,
            );
            s.reading_order = order;
            order += 1;
            spans.push(s);
        }

        let result = converter.convert(&spans, &config).unwrap();

        // "body text content" at 11pt should NOT be detected as heading
        // because base_font_size should be ~11pt (ratio 1.0)
        assert!(
            !result.contains("### body text content"),
            "11pt bold text should not be heading when base is 11pt: {}",
            result
        );
    }

    // ============================================================================
    // Issue #260: Single-word BT/ET blocks should have spaces between words
    // ============================================================================

    /// Helper to create a span with a specific width (for gap-detection tests).
    fn make_span_with_width(
        text: &str,
        x: f32,
        y: f32,
        width: f32,
        font_size: f32,
        weight: FontWeight,
        order: usize,
    ) -> OrderedTextSpan {
        let mut s = OrderedTextSpan::new(
            TextSpan {
                provenance: None,
                text_rise: 0.0,
                artifact_type: None,
                text: text.to_string(),
                bbox: Rect::new(x, y, width, font_size),
                font_name: "Test".to_string(),
                font_size,
                font_weight: weight,
                is_italic: false,
                is_monospace: false,
                color: Color::black(),
                mcid: None,
                mcid_scope: None,
                sequence: 0,
                offset_semantic: false,
                split_boundary_before: false,
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
            },
            order,
        );
        s.reading_order = order;
        s
    }

    #[test]
    fn test_issue_260_single_word_bt_et_blocks_get_spaces() {
        // PDFKit.NET places each word in its own BT/ET block with absolute positioning.
        // The markdown converter must detect the horizontal gap and insert a space.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        // Simulate: "The" at x=72 w=20, "quick" at x=96 w=30, "brown" at x=130 w=33
        // All same Y=500, font_size=12. Gaps: 96-92=4pt, 130-126=4pt.
        // 4pt gap > 0.15*12=1.8pt threshold → should insert space.
        let spans = vec![
            make_span_with_width("The", 72.0, 500.0, 20.0, 12.0, FontWeight::Normal, 0),
            make_span_with_width("quick", 96.0, 500.0, 30.0, 12.0, FontWeight::Normal, 1),
            make_span_with_width("brown", 130.0, 500.0, 33.0, 12.0, FontWeight::Normal, 2),
        ];

        let result = converter.convert(&spans, &config).unwrap();
        assert!(
            result.contains("The quick brown"),
            "Single-word BT/ET spans with gaps should have spaces inserted: got {:?}",
            result
        );
    }

    #[test]
    fn test_issue_260_no_space_for_tight_spans() {
        // When spans are tightly packed (no significant gap), no extra space should be added.
        // This covers ligature fragments or split characters.
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        // "Hel" at x=72 w=18, "lo" at x=90 w=12 — gap = 90-90 = 0pt, no space needed
        let spans = vec![
            make_span_with_width("Hel", 72.0, 500.0, 18.0, 12.0, FontWeight::Normal, 0),
            make_span_with_width("lo", 90.0, 500.0, 12.0, 12.0, FontWeight::Normal, 1),
        ];

        let result = converter.convert(&spans, &config).unwrap();
        assert!(
            result.contains("Hello"),
            "Tight spans should be merged without space: got {:?}",
            result
        );
    }

    #[test]
    fn test_heading_detection_still_works_for_large_fonts() {
        let converter = MarkdownOutputConverter::new();
        let config = config_with_headings();

        let mut heading = make_span("BIG HEADING", 50.0, 100.0, 24.0, FontWeight::Bold);
        heading.reading_order = 0;

        let mut body = make_span("Body text", 50.0, 70.0, 11.0, FontWeight::Normal);
        body.reading_order = 1;

        let spans = vec![heading, body];
        let result = converter.convert(&spans, &config).unwrap();

        assert!(result.contains("# BIG HEADING"), "24pt text should be H1: {}", result);
    }

    // ============================================================================
    // Bold consolidation tests
    // ============================================================================

    #[test]
    fn test_bold_consolidation_adjacent_bold_spans() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        // Three adjacent bold spans on the same line — each word is a separate span.
        // Use realistic bbox widths so that horizontal gap detection inserts spaces.
        let mut s1 = make_span_w("ACME", 72.0, 700.0, 55.0, 12.0, FontWeight::Bold);
        s1.reading_order = 0;

        let mut s2 = make_span_w("GLOBAL", 130.0, 700.0, 42.0, 12.0, FontWeight::Bold);
        s2.reading_order = 1;

        let mut s3 = make_span_w("LTD.", 175.0, 700.0, 24.0, 12.0, FontWeight::Bold);
        s3.reading_order = 2;

        let spans = vec![s1, s2, s3];
        let result = converter.convert(&spans, &config).unwrap();

        // Should consolidate into a single bold block
        assert!(
            result.contains("**ACME GLOBAL LTD.**"),
            "Adjacent bold spans should be consolidated into one bold block, got: {}",
            result
        );
        // Should NOT have per-word bold markers
        assert!(
            !result.contains("**ACME** **GLOBAL**"),
            "Should not wrap each word individually in bold markers, got: {}",
            result
        );
    }

    // ============================================================================
    // Issue: table cell dropping during markdown conversion
    // ============================================================================

    #[test]
    fn test_render_table_markdown_all_cells_present() {
        // Simulates a financial statement table:
        //   Row 1 (header): "Account No." | "Reference" | "Tax ID" | "Confirmation"
        //   Row 2 (data):   "20003035"    | "403852"    | "123 456 789" | "4351966"
        let mut table = Table::new();
        table.has_header = true;
        table.col_count = 4;

        let mut header = TableRow::new(true);
        header.add_cell(TableCell::new("Account No.".to_string(), true));
        header.add_cell(TableCell::new("Reference".to_string(), true));
        header.add_cell(TableCell::new("Tax ID".to_string(), true));
        header.add_cell(TableCell::new("Confirmation".to_string(), true));
        table.add_row(header);

        let mut data = TableRow::new(false);
        data.add_cell(TableCell::new("20003035".to_string(), false));
        data.add_cell(TableCell::new("403852".to_string(), false));
        data.add_cell(TableCell::new("123 456 789".to_string(), false));
        data.add_cell(TableCell::new("4351966".to_string(), false));
        table.add_row(data);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());

        // All cells must be present
        assert!(
            result.contains("403852"),
            "Reference value '403852' must be present in markdown table: {}",
            result
        );
        assert!(result.contains("20003035"), "Account No. value must be present: {}", result);
        assert!(result.contains("123 456 789"), "Tax ID value must be present: {}", result);
        assert!(result.contains("4351966"), "Confirmation value must be present: {}", result);
        assert!(result.contains("Reference"), "Header must be present: {}", result);

        // Must have pipe separators (markdown table format)
        assert!(result.contains("|"), "Must be markdown table format with pipe separators");
    }

    #[test]
    fn test_render_table_markdown_short_row_padded() {
        // When a data row has fewer cells than the header, the markdown table
        // must pad with empty cells so every row has the same column count.
        // Otherwise markdown parsers silently drop trailing columns.
        let mut table = Table::new();
        table.has_header = true;
        table.col_count = 4;

        let mut header = TableRow::new(true);
        header.add_cell(TableCell::new("A".to_string(), true));
        header.add_cell(TableCell::new("B".to_string(), true));
        header.add_cell(TableCell::new("C".to_string(), true));
        header.add_cell(TableCell::new("D".to_string(), true));
        table.add_row(header);

        // Data row with only 2 cells (e.g., merge detection removed 2 cells)
        let mut data = TableRow::new(false);
        data.add_cell(TableCell::new("1".to_string(), false));
        data.add_cell(TableCell::new("2".to_string(), false));
        table.add_row(data);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());

        // Count pipes in header vs data row — they must match
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 3, "Must have header, separator, and data row: {}", result);

        let header_pipes = lines[0].matches('|').count();
        let data_pipes = lines[2].matches('|').count();
        assert_eq!(
            header_pipes, data_pipes,
            "Header and data rows must have same number of pipe separators.\nHeader ({}): {}\nData   ({}): {}",
            header_pipes, lines[0], data_pipes, lines[2]
        );
    }

    #[test]
    fn test_render_table_markdown_short_header_padded() {
        // When the header has fewer cells than the widest data row, the header
        // must also be padded.
        let mut table = Table::new();
        table.has_header = true;
        table.col_count = 3;

        let mut header = TableRow::new(true);
        header.add_cell(TableCell::new("X".to_string(), true));
        header.add_cell(TableCell::new("Y".to_string(), true));
        table.add_row(header);

        let mut data = TableRow::new(false);
        data.add_cell(TableCell::new("1".to_string(), false));
        data.add_cell(TableCell::new("2".to_string(), false));
        data.add_cell(TableCell::new("3".to_string(), false));
        table.add_row(data);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &crate::pipeline::TextPipelineConfig::default());

        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() >= 3, "Must have header, separator, and data row: {}", result);

        let header_pipes = lines[0].matches('|').count();
        let data_pipes = lines[2].matches('|').count();
        assert_eq!(
            header_pipes, data_pipes,
            "Header and data rows must have same number of pipe separators.\nHeader ({}): {}\nData   ({}): {}",
            header_pipes, lines[0], data_pipes, lines[2]
        );

        // All data values must be present
        assert!(result.contains("| 3 |"), "Third cell in data row must be present: {}", result);
    }

    #[test]
    fn test_key_value_pair_merging_in_markdown() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();

        // Simulate a single label on one line followed by its value on the next.
        // This happens when spans from different groups produce separate lines.
        let mut s0 = make_span("Grand Total", 50.0, 200.0, 12.0, FontWeight::Normal);
        s0.reading_order = 0;
        s0.group_id = Some(0);

        // Value on a different line (different Y), next in reading order, different group
        let mut s1 = make_span("$750.00", 300.0, 185.0, 12.0, FontWeight::Normal);
        s1.reading_order = 1;
        s1.group_id = Some(1);

        let spans = vec![s0, s1];
        let result = converter.convert(&spans, &config).unwrap();

        assert!(
            result.contains("Grand Total $750.00"),
            "Should merge label with value on same line: {:?}",
            result,
        );
    }

    /// D7 — Arabic text with Bold font-weight must NOT produce `**` markers in
    /// the markdown output.  Reproduces the right_to_left_02 fixture where
    /// contextual glyph forms (initial/medial/final) triggered the bold
    /// detector, inserting spurious `**مرح**با` fragments.
    #[test]
    fn test_arabic_bold_span_no_spurious_bold_markers() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Even when the font is reported as Bold, Arabic text must NOT be
        // wrapped in `**…**` in the final markdown (the bold detector fires on
        // Latin-font-weight heuristics that are unreliable for Arabic glyphs).
        let span = make_span("مرحبا", 0.0, 100.0, 12.0, FontWeight::Bold);
        let result = converter.convert(&[span], &config).unwrap();
        assert!(
            !result.contains("**"),
            "spurious bold markers found in Arabic output: {:?}",
            result
        );
        assert!(result.contains("مرحبا"), "Arabic text lost in output: {:?}", result);
    }

    /// D7 — is_rtl_text / looks_rtl must return true for Arabic Unicode ranges
    /// and false for ASCII.  Pins the detector contract used by the converter.
    #[test]
    fn test_rtl_detection_arabic_and_ascii() {
        // Arabic main block
        assert!(crate::text::bidi::looks_rtl("مرحبا"), "Arabic U+0600-U+06FF must be RTL");
        // Arabic Presentation Forms-B (common in PDFs using contextual forms)
        assert!(
            crate::text::bidi::looks_rtl("\u{FE80}"),
            "Arabic Presentation Forms-B U+FE80 must be RTL"
        );
        // Hebrew
        assert!(crate::text::bidi::looks_rtl("שלום"), "Hebrew U+0590-U+05FF must be RTL");
        // Pure ASCII must not trigger the RTL path.
        assert!(!crate::text::bidi::looks_rtl("hello world"), "ASCII must not be RTL");
        assert!(!crate::text::bidi::looks_rtl(""), "empty string must not be RTL");
    }

    /// D7 — strip_inline_emphasis_in_rtl must remove `**…**` and `*…*`
    /// markers when the inner content is RTL (Arabic / Hebrew) and preserve
    /// them when the inner content is LTR.
    #[test]
    fn test_strip_inline_emphasis_removes_rtl_markers() {
        // `**bold**` around Arabic text → markers stripped
        let out = strip_inline_emphasis_in_rtl("**مرح**با");
        assert!(!out.contains("**"), "bold markers must be stripped from Arabic: {:?}", out);
        assert!(
            out.contains("مرح") && out.contains("با"),
            "Arabic chars must survive stripping: {:?}",
            out
        );

        // `*italic*` around Arabic text → markers stripped
        let out2 = strip_inline_emphasis_in_rtl("*مرحبا*");
        assert!(!out2.contains('*'), "italic markers must be stripped from Arabic: {:?}", out2);
        assert!(out2.contains("مرحبا"), "Arabic text lost: {:?}", out2);

        // Emphasis around LTR content must be preserved.
        let out3 = strip_inline_emphasis_in_rtl("*Hello*");
        assert_eq!(out3, "*Hello*", "LTR emphasis must be preserved: {:?}", out3);

        // No asterisks → identity.
        let out4 = strip_inline_emphasis_in_rtl("مرحبا");
        assert_eq!(out4, "مرحبا", "no-asterisk path must be identity: {:?}", out4);
    }

    /// D7 — the RTL emphasis cleanup block must preserve the trailing newline
    /// that the whitespace-normalisation pass added.  Previously `lines().join()`
    /// silently dropped the terminal `\n`, corrupting multi-paragraph documents.
    #[test]
    fn test_rtl_cleanup_preserves_trailing_newline() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        // Two Arabic paragraphs separated by `\n\n`.  The result must end with
        // the same suffix the normaliser emits (a single `\n` in default mode).
        let mut s1 = make_span("مرحبا", 0.0, 200.0, 12.0, FontWeight::Normal);
        s1.block_id = Some(1);
        let mut s2 = make_span("عالم", 0.0, 100.0, 12.0, FontWeight::Normal);
        s2.block_id = Some(2);
        let result = converter.convert(&[s1, s2], &config).unwrap();
        // Must contain both words.
        assert!(result.contains("مرحبا"), "first Arabic word lost: {:?}", result);
        assert!(result.contains("عالم"), "second Arabic word lost: {:?}", result);
        // Result must end with a newline (the document-level trailing `\n`).
        assert!(
            result.ends_with('\n'),
            "trailing newline was dropped by RTL cleanup: {:?}",
            result
        );
    }

    // ─────────────────────────────────────────────────────────────────
    // Regression suite for the v0.3.51/v0.3.52 markdown-extraction
    // quality issues (external reporter, 54-PDF corpus). Each test
    // exercises ONE issue with synthetic input — no external PDF
    // dependency — so the harness stays deterministic and survives
    // upstream re-extractor changes. Where a fix is post-process only,
    // the helper function is invoked directly; where the fix is
    // structural, a full `convert()` pass is used.
    // ─────────────────────────────────────────────────────────────────

    /// Issue #10 — stray leading `|` outside a table block must be
    /// escaped so downstream renderers do not misread it as a malformed
    /// table row.
    #[test]
    fn test_issue10_escape_stray_leading_pipes_basic() {
        let input = "| Finished Goods\n| Internal Use Only\nPage 1 of 12\n";
        let out = escape_stray_leading_pipes(input);
        assert!(out.contains("\\| Finished Goods"), "stray pipe must be escaped, got:\n{}", out);
        assert!(
            out.contains("\\| Internal Use Only"),
            "second stray pipe must be escaped, got:\n{}",
            out
        );
    }

    /// Issue #10 — a real markdown table block must NOT be escaped.
    /// Guards against over-eager pipe escaping that would corrupt
    /// legitimate tables.
    #[test]
    fn test_issue10_preserves_real_tables() {
        let input = "| Col A | Col B |\n|---|---|\n| 1 | 2 |\n";
        let out = escape_stray_leading_pipes(input);
        assert!(!out.contains("\\|"), "real table rows must not be escaped, got:\n{}", out);
    }

    /// REGRESSION GUARD (70-PDF sweep). A real markdown table with
    /// mostly single-word cells (e.g. countries × Continent/Capital/
    /// Currency) must NOT be flattened to prose by the pipeline. The
    /// simplify_degenerate_tables heuristic that did this is retired
    /// from the active path; this test pins the table survives a full
    /// convert_with_tables() pass.
    #[test]
    fn test_regression_real_sparse_table_not_flattened() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mut table = Table::new();
        let mut header = TableRow::new(true);
        for h in ["", "Indonesia", "Germany", "Austria", "France", "Vatican"] {
            header.add_cell(TableCell::new(h.to_string(), true));
        }
        table.add_row(header);
        for (label, vals) in [
            ("Continent", ["Asia", "", "Europe", "", ""]),
            ("Capital", ["Jakarta", "Berlin", "Vienna", "Paris", "Vatican City"]),
        ] {
            let mut row = TableRow::new(false);
            row.add_cell(TableCell::new(label.to_string(), false));
            for v in vals {
                row.add_cell(TableCell::new(v.to_string(), false));
            }
            table.add_row(row);
        }
        let result = converter
            .convert_with_tables(&[], &[table], &config)
            .unwrap();
        assert!(
            result.contains("|---|") || result.contains("| Indonesia |"),
            "real sparse table must survive as a table, got:\n{}",
            result
        );
    }

    /// REGRESSION GUARD (70-PDF sweep). Consecutive paragraphs with
    /// identical text (e.g. several distinct form widgets that share
    /// a label) must NOT be deduped away by the pipeline. The
    /// dedup_consecutive_paragraphs step that did this is retired.
    #[test]
    fn test_regression_repeated_identical_paragraphs_preserved() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span("Radio button, unselected", 0.0, 100.0, 12.0, FontWeight::Normal),
            make_span("Radio button, unselected", 0.0, 80.0, 12.0, FontWeight::Normal),
            make_span("Radio button, unselected", 0.0, 60.0, 12.0, FontWeight::Normal),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        let count = result.matches("Radio button, unselected").count();
        assert_eq!(
            count, 3,
            "three distinct identical-label widgets must all survive, got {}:\n{}",
            count, result
        );
    }

    /// SPEC-ALIGNMENT (§14.8.4.3.2). When the document is TAGGED —
    /// spans carry explicit `struct_role = Heading(_)` — three
    /// distinct short H1 elements are author-specified structure and
    /// MUST survive as three headings. The untagged word-per-heading
    /// merge heuristic must NOT override authoritative tagging.
    #[test]
    fn test_tagged_distinct_headings_are_not_merged() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, y: f32| {
            let mut s = make_span(t, 0.0, y, 18.0, FontWeight::Bold);
            s.struct_role = Some(StructRole::Heading(1));
            s
        };
        // Three short headings with large baseline drops → upstream
        // emits three `# ` lines; the gate must keep them at three.
        let spans = vec![mk("Alpha", 100.0), mk("Beta", 60.0), mk("Gamma", 20.0)];
        let result = converter.convert(&spans, &config).unwrap();
        let h1_count = result.lines().filter(|l| l.starts_with("# ")).count();
        assert_eq!(
            h1_count, 3,
            "tagged distinct H1 elements must NOT be merged (spec §14.8.4.3.2), got:\n{}",
            result
        );
    }

    /// Issue #1 — PowerPoint-exported word-per-heading runs must fuse
    /// into a single heading line.
    #[test]
    fn test_issue1_merge_word_per_heading_runs() {
        let input = "# Quarterly\n\n# Inventory\n\n# Review\n";
        let out = merge_consecutive_same_level_headings(input);
        assert_eq!(
            out.trim(),
            "# Quarterly Inventory Review",
            "three same-level short H1s must merge, got:\n{}",
            out
        );
    }

    /// Issue #4 — wrapped long-heading split across two lines must
    /// fuse when there is a continuation signal (trailing comma /
    /// semicolon on the first fragment, or a lowercase / connector-word
    /// opener on the second). See `looks_like_heading_wrap`.
    #[test]
    fn test_issue4_merge_wrapped_heading_trailing_comma() {
        let input = "## Despite seasonal slowdown,\n## warehouse maintained throughput\n";
        let out = merge_consecutive_same_level_headings(input);
        assert!(
            out.contains("## Despite seasonal slowdown, warehouse maintained throughput"),
            "wrapped heading with trailing comma must fuse, got:\n{}",
            out
        );
    }

    /// Issue #4 — alternative continuation signal: second fragment
    /// opens with a connector word ("and" / "with" / ...).
    #[test]
    fn test_issue4_merge_wrapped_heading_connector_opener() {
        let input = "# Architecture\n# and Implementation\n";
        let out = merge_consecutive_same_level_headings(input);
        assert!(
            out.contains("# Architecture and Implementation"),
            "wrapped heading with connector opener must fuse, got:\n{}",
            out
        );
    }

    /// Issue #4 — without ANY continuation signal (first ends without
    /// trailing comma; second is capitalized non-connector), the
    /// 2-fragment run must remain two separate headings. Guards the
    /// `test_large_baseline_drop_still_splits_heading` invariant.
    #[test]
    fn test_issue4_does_not_fuse_ambiguous_two_headings() {
        let input = "# First Heading\n# Second Heading\n";
        let out = merge_consecutive_same_level_headings(input);
        let h_lines = out.lines().filter(|l| l.starts_with("# ")).count();
        assert_eq!(
            h_lines, 2,
            "ambiguous 2-fragment same-level headings must NOT fuse, got:\n{}",
            out
        );
    }

    /// Issue #1/#4 — must NOT fuse two genuinely distinct headings
    /// when either side is long. Guards against over-eager merging.
    #[test]
    fn test_issue1_does_not_fuse_long_distinct_headings() {
        let h1 = "# Annual Sales Performance Across Every Region in Detail";
        let h2 = "# Q1 Highlights and Outlook for the Year";
        let input = format!("{}\n\n{}\n", h1, h2);
        let out = merge_consecutive_same_level_headings(&input);
        assert!(
            out.contains(h1) && out.contains(h2),
            "two long distinct headings must remain separate, got:\n{}",
            out
        );
    }

    /// Issue #3 — spatial-prose-as-table (>= 5 cols, >= 2 data rows,
    /// >= 60% single-word non-empty cells) collapses to a paragraph.
    #[test]
    fn test_issue3_degenerate_table_collapses_to_paragraph() {
        let input = "\
| Q1 | Warehouse | throughput | increased | 15% |
|---|---|---|---|---|
| quarter | over | quarter | to | 23,500 |
| units | per | day | strong | demand |
";
        let out = simplify_degenerate_tables(input);
        assert!(!out.contains("|---|"), "separator row should be gone, got:\n{}", out);
        assert!(
            out.contains("Q1 Warehouse throughput increased 15%"),
            "header words flattened to prose, got:\n{}",
            out
        );
    }

    /// Issue #3 — a normal table with multi-word cells must SURVIVE.
    /// Guards against over-eager flattening that would corrupt real
    /// tabular data.
    #[test]
    fn test_issue3_preserves_legitimate_multi_word_tables() {
        let input = "\
| Region | Revenue Q1 | Revenue Q2 | Revenue Q3 | Revenue Q4 |
|---|---|---|---|---|
| North America Sales | 1.2 M | 1.5 M | 1.7 M | 1.9 M |
| Europe Sales Total | 0.8 M | 0.9 M | 1.1 M | 1.3 M |
";
        let out = simplify_degenerate_tables(input);
        assert!(out.contains("|---|"), "real table must keep separator, got:\n{}", out);
        assert!(
            out.contains("| North America Sales |"),
            "real table cells must remain, got:\n{}",
            out
        );
    }

    /// Issue #9 — page-number-shaped lines (e.g. "Page 1 of 12",
    /// "— 5 —", "[12]") MUST be preserved in the markdown output if
    /// they appear in the prose stream. Dropping them at this layer
    /// would discard legitimate content — the proper fix is upstream
    /// artifact (`/Artifact` tag) handling per PDF §14.8.2.2. This
    /// test pins that contract: the post-process pipeline does not
    /// touch these lines.
    #[test]
    fn test_issue9_preserves_page_number_shaped_lines() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span("Some text.", 0.0, 100.0, 12.0, FontWeight::Normal),
            make_span("Page 1 of 12", 0.0, 80.0, 10.0, FontWeight::Normal),
            make_span("More text.", 0.0, 60.0, 12.0, FontWeight::Normal),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        assert!(result.contains("Page 1 of 12"), "page-N text must survive, got:\n{}", result);
        assert!(result.contains("Some text."), "prose must survive, got:\n{}", result);
        assert!(result.contains("More text."), "prose must survive, got:\n{}", result);
    }

    /// Issue #9 — in-prose "Page N" references must obviously also
    /// survive (this was the existing guard).
    #[test]
    fn test_issue9_preserves_page_in_prose() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![make_span(
            "See Page 3 for details about the change.",
            0.0,
            100.0,
            12.0,
            FontWeight::Normal,
        )];
        let result = converter.convert(&spans, &config).unwrap();
        assert!(
            result.contains("See Page 3 for details"),
            "in-prose 'Page N' must not be dropped, got:\n{}",
            result
        );
    }

    /// Issue #13 — wrong-glyph bullets (`❍`, `◦`, ...) at line start
    /// must NOT be silently dropped. The upstream renderer already
    /// recognizes these as bullet-glyph variants and emits them as
    /// idiomatic markdown `- ` bullets — that preserves the semantic
    /// list structure across all glyph variants. What this test
    /// pins is content preservation: the text content after the
    /// glyph (`First item`, `Second item`) must reach the output;
    /// the bullet symbol itself can be normalized to `-` because
    /// markdown's bullet semantics are the same.
    ///
    /// What is NOT acceptable (the bug we're guarding against): a
    /// post-process layer pattern-matching codepoints and rewriting
    /// them in arbitrary text. The pipeline does no such rewriting
    /// (see `normalize_bullet_glyphs` no-op doc).
    #[test]
    fn test_issue13_preserves_bullet_text_content() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span("\u{274D} First item", 0.0, 100.0, 12.0, FontWeight::Normal),
            make_span("\u{25E6} Second item", 0.0, 80.0, 12.0, FontWeight::Normal),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        assert!(result.contains("First item"), "list-item text must survive: {}", result);
        assert!(result.contains("Second item"), "list-item text must survive: {}", result);
    }

    /// Issue #13 (mid-prose codepoint preservation). A `❍` that
    /// appears in the MIDDLE of body text (not at line start) must
    /// be preserved verbatim — at that position the upstream does
    /// not treat it as a bullet, so any rewriting would be content
    /// corruption.
    #[test]
    fn test_issue13_preserves_mid_prose_bullet_codepoint() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![make_span(
            "The symbol \u{274D} indicates a shadow circle.",
            0.0,
            100.0,
            12.0,
            FontWeight::Normal,
        )];
        let result = converter.convert(&spans, &config).unwrap();
        assert!(
            result.contains("\u{274D}"),
            "mid-prose U+274D must survive verbatim, got:\n{}",
            result
        );
    }

    /// Issue #11 — KPI numeric-only H1 run collapses to bulleted list.
    #[test]
    fn test_issue11_collapses_numeric_heading_run() {
        let input = "# 23,500\n\n# 99.2%\n\n# 87%\n\n# 4.2 days\n";
        let out = collapse_numeric_heading_runs(input);
        for v in ["- 23,500", "- 99.2%", "- 87%", "- 4.2 days"] {
            assert!(out.contains(v), "expected `{}` in output, got:\n{}", v, out);
        }
        assert!(!out.contains("# 23,500"), "H1 form must be gone, got:\n{}", out);
    }

    /// Issue #11 — a numeric heading that LOOKS standalone (single
    /// occurrence) must NOT collapse. Two-or-more is the trigger.
    #[test]
    fn test_issue11_preserves_single_numeric_heading() {
        let input = "# 2024 Annual Report\n";
        let out = collapse_numeric_heading_runs(input);
        assert_eq!(out, input, "single non-numeric heading must be untouched: {}", out);
    }

    /// Issue #12 — `**S alesF orce**` CamelCase fragmentation inside a
    /// single bold pair coalesces to `**SalesForce**`.
    #[test]
    fn test_issue12_coalesces_inline_camelcase_bold() {
        let input = "**S alesF orce** is great.\n";
        let out = coalesce_camelcase_bold_fragments(input);
        assert!(
            out.contains("**SalesForce**"),
            "inline CamelCase bold must coalesce, got:\n{}",
            out
        );
    }

    /// Issue #12 — must NOT touch legitimate two-word bold like
    /// `**John Smith**` or `**USB Type C**`. The CamelCase signal
    /// (lowercase-then-uppercase inside one fragment) is required.
    #[test]
    fn test_issue12_preserves_normal_multi_word_bold() {
        let input = "**John Smith** wrote.\n**USB Type C** cable.\n";
        let out = coalesce_camelcase_bold_fragments(input);
        assert!(
            out.contains("**John Smith**"),
            "two-word person bold must not be merged, got:\n{}",
            out
        );
        assert!(
            out.contains("**USB Type C**"),
            "three-word product bold must not be merged, got:\n{}",
            out
        );
    }

    /// Issue #12 (BOUND case) — closing `**` lands mid-CamelCase:
    /// `**N orthW** ind` (intended `**N**orthWind` or `**NorthWind**`).
    /// This is the pattern not yet covered by the inline-bold regex.
    /// Marked `#[ignore]` until the bound coalescer lands.
    #[test]
    fn test_issue12_bound_camelcase_bold_coalesces() {
        let input = "**N orthW** ind";
        let out = coalesce_camelcase_bold_fragments(input);
        // Either of these post-coalesce forms is acceptable; both
        // recover the intended brand name.
        let acceptable = out.contains("**NorthWind**")
            || out.contains("**NorthW**ind")
            || out.contains("**N**orthWind");
        assert!(
            acceptable,
            "bound CamelCase bold (closing ** mid-word) should coalesce, got:\n{}",
            out
        );
    }

    /// Issue #8 — a table cell that carries bold spans must render the
    /// bold markers in the output. Reporter measured 73% bold-marker
    /// loss across 53/54 files; this asserts at least the simple case.
    #[test]
    fn test_issue8_table_cell_renders_bold_marker() {
        let bold_span = TextSpan {
            provenance: None,
            text_rise: 0.0,
            artifact_type: None,
            text: "Critical".to_string(),
            bbox: Rect::new(0.0, 0.0, 50.0, 12.0),
            font_name: "Test-Bold".to_string(),
            font_size: 12.0,
            font_weight: FontWeight::Bold,
            is_italic: false,
            is_monospace: false,
            color: Color::black(),
            mcid: None,
            mcid_scope: None,
            sequence: 0,
            offset_semantic: false,
            split_boundary_before: false,
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
        };
        let mut cell = TableCell::new("Critical".to_string(), false);
        cell.spans.push(bold_span.clone());
        let mut row = TableRow::new(false);
        row.add_cell(cell);
        let mut table = Table::new();
        table.add_row(row);

        let result = MarkdownOutputConverter::new()
            .render_table_markdown(&table, &TextPipelineConfig::default());
        assert!(
            result.contains("**Critical**"),
            "bold marker must appear in rendered cell, got:\n{}",
            result
        );
    }

    /// Issue #2 — consecutive duplicate paragraphs (structured +
    /// plaintext echo) must be deduped down to one.
    #[test]
    fn test_issue2_dedup_consecutive_duplicate_paragraphs() {
        let input = "Revenue grew by 15%.\n\nRevenue grew by 15%.\n\nNext paragraph here.\n";
        let out = dedup_consecutive_paragraphs(input);
        let occurrences = out.matches("Revenue grew by 15%.").count();
        assert_eq!(
            occurrences, 1,
            "exact-duplicate consecutive paragraph must collapse, got:\n{}",
            out
        );
        assert!(
            out.contains("Next paragraph here."),
            "subsequent paragraph must survive, got:\n{}",
            out
        );
    }

    /// Issue #2 — non-consecutive duplicates (separated by other
    /// content) must NOT be touched: legitimate prose can repeat a
    /// phrase later in the document.
    #[test]
    fn test_issue2_preserves_nonconsecutive_repeats() {
        let input = "Important note.\n\nOther content.\n\nImportant note.\n";
        let out = dedup_consecutive_paragraphs(input);
        let occurrences = out.matches("Important note.").count();
        assert_eq!(occurrences, 2, "non-consecutive repeat must survive, got:\n{}", out);
    }

    /// Issue #5 — all-identical header cells (spatial-grouping
    /// artifact) must be deduped to a single occurrence in the
    /// rendered output. Operates on the assembled markdown so it
    /// catches both render paths.
    #[test]
    fn test_issue5_dedups_identical_header_cells() {
        let input = "| Q1'25 | Q1'25 | Q1'25 | Q1'25 |\n|---|---|---|---|\n| Zone A |  |  |  |\n";
        let out = dedup_identical_header_cells(input);
        let q1_count = out.matches("Q1'25").count();
        assert_eq!(
            q1_count, 1,
            "all-identical header cells must dedup to one, got {} in:\n{}",
            q1_count, out
        );
        // Cell count preserved (still 4 pipes in the data row).
        assert!(out.contains("Zone A"), "data row must remain intact, got:\n{}", out);
    }

    /// Issue #5 — a legitimate header with distinct values must NOT
    /// be touched.
    #[test]
    fn test_issue5_preserves_real_distinct_headers() {
        let input = "| North | South | East | West |\n|---|---|---|---|\n| 1 | 2 | 3 | 4 |\n";
        let out = dedup_identical_header_cells(input);
        for col in ["North", "South", "East", "West"] {
            assert!(out.contains(col), "distinct header `{}` must survive: {}", col, out);
        }
    }

    /// Issue #7 — when side-by-side columns are present, text from
    /// column 2 must not interleave with column 1's text mid-paragraph.
    /// The existing `is_column_gap` heuristic (forward gutter > 3×
    /// font_size OR backward wrap) is what forces the paragraph break
    /// between columns; this test pins that behavior so future
    /// reading-order refactors don't silently regress it.
    #[test]
    fn test_issue7_no_column_interleaving() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let mk = |t: &str, x: f32, y: f32, bid: u32| {
            let mut s = make_span(t, x, y, 12.0, FontWeight::Normal);
            s.block_id = Some(bid);
            s
        };
        // Left column at x=0, right column at x=300; baselines stagger.
        let spans = vec![
            mk("Left A.", 0.0, 100.0, 1),
            mk("Right A.", 300.0, 100.0, 2),
            mk("Left B.", 0.0, 88.0, 1),
            mk("Right B.", 300.0, 88.0, 2),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        // Left column must surface as a contiguous run.
        assert!(
            result.contains("Left A.") && result.contains("Left B."),
            "left column must surface, got:\n{}",
            result
        );
        // No interleaving: "Left A. Right A." together would prove
        // interleaving (reading-order put right immediately after left
        // before left's continuation).
        assert!(
            !result.contains("Left A. Right A."),
            "columns must not interleave at the line level, got:\n{}",
            result
        );
    }

    // ==========================================================================
    // Bidi-isolation markers in markdown output (#537 follow-up — v0.3.55).
    // Acceptance tests from
    // docs/releases/plans/v0.3.55/fix-537-followup-bidi-isolation-markers.md.
    // ==========================================================================

    /// Hebrew run in an LTR-dominant line — must be wrapped with
    /// U+2067 (RLI) … U+2069 (PDI) so a UAX #9-aware markdown
    /// viewer does not let neutrals around the Hebrew bleed across
    /// the boundary. Pre-fix output had no markers; this test pins
    /// the post-fix behaviour.
    #[test]
    fn markdown_wraps_rtl_run_with_rli_pdi() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let span =
            make_span("The article שלום עולם is greetings.", 0.0, 100.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[span], &config).unwrap();
        assert!(
            result.contains('\u{2067}'),
            "expected U+2067 (RLI) in markdown output, got:\n{:?}",
            result
        );
        assert!(
            result.contains('\u{2069}'),
            "expected U+2069 (PDI) in markdown output, got:\n{:?}",
            result
        );
        // Block is LTR-dominant — LTR runs must NOT get LRI.
        assert!(
            !result.contains('\u{2066}'),
            "unexpected U+2066 (LRI) in LTR-block output:\n{:?}",
            result
        );
    }

    /// English brand-name embedded in a Hebrew (RTL-dominant) line
    /// — the English run must be wrapped with U+2066 (LRI) …
    /// U+2069 (PDI).
    #[test]
    fn markdown_wraps_ltr_run_inside_rtl_block_with_lri_pdi() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let span = make_span("הספר Microsoft חדש", 0.0, 100.0, 12.0, FontWeight::Normal);
        let result = converter.convert(&[span], &config).unwrap();
        assert!(
            result.contains('\u{2066}'),
            "expected U+2066 (LRI) wrapping the embedded LTR token, got:\n{:?}",
            result
        );
        assert!(
            result.contains('\u{2069}'),
            "expected U+2069 (PDI) closing the LRI, got:\n{:?}",
            result
        );
        // Block is RTL-dominant — RTL runs must NOT get RLI.
        assert!(
            !result.contains('\u{2067}'),
            "unexpected U+2067 (RLI) in RTL-block output:\n{:?}",
            result
        );
    }

    /// Regression guard: pure-LTR markdown output must contain
    /// ZERO bidi-isolation markers anywhere. This is the "no
    /// markers appear in pure-LTR documents" contract from the
    /// v0.3.55 plan's acceptance criteria. If this ever fails, the
    /// isolation pass leaked into LTR-only output.
    #[test]
    fn markdown_leaves_pure_ltr_unchanged() {
        let converter = MarkdownOutputConverter::new();
        let config = TextPipelineConfig::default();
        let spans = vec![
            make_span("The first paragraph.", 0.0, 100.0, 12.0, FontWeight::Normal),
            make_span("A second sentence.", 0.0, 84.0, 12.0, FontWeight::Normal),
            make_span("Numbers 123 and (parens) too.", 0.0, 68.0, 12.0, FontWeight::Normal),
        ];
        let result = converter.convert(&spans, &config).unwrap();
        for marker in ['\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'] {
            assert!(
                !result.contains(marker),
                "pure-LTR output must not contain U+{:04X}, got:\n{:?}",
                marker as u32,
                result
            );
        }
    }
}
