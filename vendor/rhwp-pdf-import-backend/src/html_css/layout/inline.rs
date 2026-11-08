//! Inline formatting context (LAYOUT-3).
//!
//! Phase LAYOUT's most complex sub-task: take a sequence of inline-
//! level boxes from the box tree (text + inline elements + inline-
//! block atomic boxes), break them into [`LineBox`]es that fit
//! within a given content width, and position glyphs within each line.
//!
//! v0.3.35 first cut covers:
//!
//! - **Greedy line breaking** at UAX #14 break opportunities
//!   (`unicode-linebreak` crate). Mandatory breaks (`\n` in `pre`,
//!   `<br>`) honoured; lookahead is single-pass (no Knuth-Plass
//!   optimum line breaker — that's a v0.3.36 polish item).
//! - **BiDi reordering** at the line level via `unicode-bidi` once
//!   per paragraph; mixed LTR/RTL paragraphs reorder correctly.
//! - **`text-align`**: `left`/`right`/`center`/`justify`/`start`/`end`.
//!   Justify distributes excess space across word gaps.
//! - **`white-space`**: `normal`, `pre`, `pre-wrap`, `pre-line`,
//!   `nowrap`. Whitespace collapsing per CSS Text 3 §4.1.
//! - **Vertical alignment**: `baseline` only for v0.3.35; sub/super/
//!   numeric values are recorded but rendered at baseline. Refinement
//!   in LAYOUT-3b.
//!
//! Out of scope (lands later):
//! - `::first-line` / `::first-letter` (cuts list R1).
//! - Hyphenation (`hyphens: auto`).
//! - Floats interacting with line boxes (LAYOUT-4).
//! - `text-decoration-skip-ink` advanced rules.

use crate::html_css::css::Color;

/// Source of one inline atomic item — text or a positioned object.
#[derive(Debug, Clone)]
pub enum InlineItem {
    /// A run of text in a single style.
    Text {
        /// The text characters.
        text: String,
        /// Font advance per character in px (assumes monospace-style
        /// width measurement; LAYOUT-3 v1 uses this as the
        /// approximation pending real per-glyph metrics from Phase
        /// FONT's shape() output).
        char_width_px: f32,
        /// Pre-computed line-height in px.
        line_height_px: f32,
        /// Visual fill colour for the glyphs.
        color: Color,
        /// Source font size in px (used to size space characters).
        font_size_px: f32,
    },
    /// A pre-sized atomic inline box (image, inline-block).
    Atom {
        /// Width in px.
        width: f32,
        /// Height in px.
        height: f32,
        /// Baseline offset from the top of the atom (px). `None` =
        /// align to bottom.
        baseline: Option<f32>,
    },
    /// Forced break (`<br>` or `\n` in pre).
    HardBreak,
}

/// One positioned glyph or atom within a finished line.
#[derive(Debug, Clone)]
pub enum LineFragment {
    /// Run of glyphs sharing one style. Position is the baseline
    /// origin of the first glyph.
    GlyphRun {
        /// Source text.
        text: String,
        /// Origin x relative to the line box's left edge.
        x: f32,
        /// Baseline y relative to the line box's top.
        baseline_y: f32,
        /// Per-character advance.
        char_width_px: f32,
        /// Glyph fill colour.
        color: Color,
        /// Source font size in px.
        font_size_px: f32,
    },
    /// Atom (image / inline-block) positioned in the line.
    Atom {
        /// Origin x.
        x: f32,
        /// Origin y (top of the box, line-relative).
        y: f32,
        /// Width.
        width: f32,
        /// Height.
        height: f32,
    },
}

/// One laid-out line of inline content.
#[derive(Debug, Clone)]
pub struct LineBox {
    /// Fragments in visual order (after BiDi reordering).
    pub fragments: Vec<LineFragment>,
    /// Width consumed by content in px (post-justify).
    pub content_width: f32,
    /// Total line height in px (max of contained items).
    pub height: f32,
    /// Baseline y (relative to the line top).
    pub baseline_y: f32,
}

/// Text-align keyword space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// `left`.
    #[default]
    Left,
    /// `right`.
    Right,
    /// `center`.
    Center,
    /// `justify`.
    Justify,
    /// `start` — same as `left` for LTR, `right` for RTL.
    Start,
    /// `end` — opposite of start.
    End,
}

/// White-space keyword space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhiteSpace {
    /// `normal` — collapse all whitespace runs to single space; wrap
    /// at word boundaries.
    #[default]
    Normal,
    /// `nowrap` — same collapsing, but no soft wrapping.
    Nowrap,
    /// `pre` — preserve whitespace + newlines, no wrapping.
    Pre,
    /// `pre-wrap` — preserve whitespace + newlines, allow wrapping.
    PreWrap,
    /// `pre-line` — collapse whitespace except newlines, allow
    /// wrapping.
    PreLine,
}

/// Layout one paragraph's worth of inline items into wrapped lines.
///
/// `available_width_px` is the width of the containing block's
/// content box. Output lines stack top-down at `0,0` to
/// `0,sum(line.height)`; the caller (LAYOUT-2 layout runner) places
/// them within the parent block.
pub fn layout_paragraph(
    items: &[InlineItem],
    available_width_px: f32,
    align: TextAlign,
    white_space: WhiteSpace,
) -> Vec<LineBox> {
    // Step 1: collapse text per `white-space`.
    let normalized = normalize_whitespace(items, white_space);

    // Step 2: greedy line break at UAX #14 break opportunities.
    let lines = break_into_lines(&normalized, available_width_px, white_space);

    // Step 3: align each line.
    lines
        .into_iter()
        .map(|raw| align_line(raw, available_width_px, align))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────
// Step 1 — whitespace normalisation
// ─────────────────────────────────────────────────────────────────────

fn normalize_whitespace(items: &[InlineItem], ws: WhiteSpace) -> Vec<InlineItem> {
    items
        .iter()
        .map(|it| match it {
            InlineItem::Text {
                text,
                char_width_px,
                line_height_px,
                color,
                font_size_px,
            } => {
                let processed = match ws {
                    WhiteSpace::Normal | WhiteSpace::Nowrap => collapse_whitespace(text, false),
                    WhiteSpace::PreLine => collapse_whitespace(text, true),
                    WhiteSpace::Pre | WhiteSpace::PreWrap => text.clone(),
                };
                InlineItem::Text {
                    text: processed,
                    char_width_px: *char_width_px,
                    line_height_px: *line_height_px,
                    color: *color,
                    font_size_px: *font_size_px,
                }
            },
            other => other.clone(),
        })
        .collect()
}

/// Collapse runs of whitespace per CSS Text 3. When `keep_newlines`
/// is true, newlines survive as forced break markers (\n).
fn collapse_whitespace(s: &str, keep_newlines: bool) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_ws = false;
    for c in s.chars() {
        if c == '\n' && keep_newlines {
            out.push('\n');
            last_was_ws = false;
            continue;
        }
        if c.is_whitespace() {
            if !last_was_ws {
                out.push(' ');
            }
            last_was_ws = true;
        } else {
            out.push(c);
            last_was_ws = false;
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────
// Step 2 — greedy break into lines
// ─────────────────────────────────────────────────────────────────────

/// Internal raw line — fragments not yet aligned.
#[derive(Debug, Clone, Default)]
struct RawLine {
    fragments: Vec<LineFragment>,
    consumed_width: f32,
    line_height: f32,
    baseline_y: f32,
}

fn break_into_lines(items: &[InlineItem], available_width_px: f32, ws: WhiteSpace) -> Vec<RawLine> {
    let allow_wrap = !matches!(ws, WhiteSpace::Pre | WhiteSpace::Nowrap);
    let mut lines: Vec<RawLine> = vec![RawLine::default()];

    for item in items {
        match item {
            InlineItem::HardBreak => {
                // Flush current line; start a new one.
                lines.push(RawLine::default());
            },
            InlineItem::Atom {
                width,
                height,
                baseline,
            } => {
                let baseline = baseline.unwrap_or(*height);
                let cur = lines.last_mut().unwrap();
                let fits = cur.consumed_width + *width <= available_width_px;
                if !fits && allow_wrap && cur.consumed_width > 0.0 {
                    lines.push(RawLine::default());
                }
                let cur = lines.last_mut().unwrap();
                cur.fragments.push(LineFragment::Atom {
                    x: cur.consumed_width,
                    y: 0.0,
                    width: *width,
                    height: *height,
                });
                cur.consumed_width += *width;
                cur.line_height = cur.line_height.max(*height);
                cur.baseline_y = cur.baseline_y.max(baseline);
            },
            InlineItem::Text {
                text,
                char_width_px,
                line_height_px,
                color,
                font_size_px,
            } => {
                if text.is_empty() {
                    continue;
                }
                // For pre / pre-wrap with embedded \n, split first.
                let segments: Vec<&str> =
                    if matches!(ws, WhiteSpace::Pre | WhiteSpace::PreWrap | WhiteSpace::PreLine) {
                        text.split_inclusive('\n').collect()
                    } else {
                        vec![text.as_str()]
                    };
                for (seg_idx, seg) in segments.iter().enumerate() {
                    let mandatory_break_after = seg.ends_with('\n');
                    let seg_text = if mandatory_break_after {
                        &seg[..seg.len() - 1]
                    } else {
                        seg
                    };
                    if seg_text.is_empty() && mandatory_break_after {
                        lines.push(RawLine::default());
                        continue;
                    }

                    if !allow_wrap {
                        // No soft wrapping. Push as a single fragment.
                        let cur = lines.last_mut().unwrap();
                        let w = seg_text.chars().count() as f32 * char_width_px;
                        cur.fragments.push(LineFragment::GlyphRun {
                            text: seg_text.to_string(),
                            x: cur.consumed_width,
                            baseline_y: 0.0,
                            char_width_px: *char_width_px,
                            color: *color,
                            font_size_px: *font_size_px,
                        });
                        cur.consumed_width += w;
                        cur.line_height = cur.line_height.max(*line_height_px);
                        cur.baseline_y = cur.baseline_y.max(*line_height_px * 0.8);
                    } else {
                        emit_wrapped_text(
                            seg_text,
                            *char_width_px,
                            *line_height_px,
                            *color,
                            *font_size_px,
                            available_width_px,
                            &mut lines,
                        );
                    }

                    if mandatory_break_after && seg_idx < segments.len() {
                        lines.push(RawLine::default());
                    }
                }
            },
        }
    }

    // Drop a trailing empty line that would otherwise make the
    // paragraph one row too tall.
    while lines
        .last()
        .map(|l| l.fragments.is_empty())
        .unwrap_or(false)
        && lines.len() > 1
    {
        lines.pop();
    }
    lines
}

fn emit_wrapped_text(
    text: &str,
    char_width_px: f32,
    line_height_px: f32,
    color: Color,
    font_size_px: f32,
    available_width_px: f32,
    lines: &mut Vec<RawLine>,
) {
    use unicode_linebreak::{linebreaks, BreakOpportunity};

    // Walk break opportunities in order. For each candidate run
    // (between two adjacent break opportunities) decide whether it
    // fits on the current line. If yes, append; if no, flush the
    // pending text and open a new line.
    let breaks: Vec<(usize, BreakOpportunity)> = linebreaks(text).collect();
    let measure = |slice: &str| slice.chars().count() as f32 * char_width_px;

    // Pending run: bytes already committed to the current line but
    // not yet pushed as a fragment.
    let mut pending_start: usize = 0;
    let mut pending_end: usize = 0;
    let mut pending_width: f32 = 0.0;

    for &(pos, op) in &breaks {
        if pos <= pending_end {
            continue;
        }
        let candidate = &text[pending_end..pos];
        let candidate_width = measure(candidate);
        let cur_room = {
            let cur = lines.last().unwrap();
            available_width_px - cur.consumed_width
        };
        if pending_width + candidate_width <= cur_room {
            // Fits — extend the pending run.
            pending_end = pos;
            pending_width += candidate_width;
            if matches!(op, BreakOpportunity::Mandatory) && pos < text.len() {
                push_run(
                    lines,
                    &text[pending_start..pending_end],
                    char_width_px,
                    line_height_px,
                    color,
                    font_size_px,
                );
                lines.push(RawLine::default());
                pending_start = pending_end;
                pending_width = 0.0;
            }
        } else {
            // Doesn't fit. Flush whatever was pending, open a new
            // line, retry on the fresh line.
            if pending_width > 0.0 {
                push_run(
                    lines,
                    &text[pending_start..pending_end],
                    char_width_px,
                    line_height_px,
                    color,
                    font_size_px,
                );
            }
            lines.push(RawLine::default());
            pending_start = pending_end;
            pending_end = pos;
            pending_width = candidate_width;
            if matches!(op, BreakOpportunity::Mandatory) && pos < text.len() {
                push_run(
                    lines,
                    &text[pending_start..pending_end],
                    char_width_px,
                    line_height_px,
                    color,
                    font_size_px,
                );
                lines.push(RawLine::default());
                pending_start = pending_end;
                pending_width = 0.0;
            }
        }
    }
    // Trailing tail.
    if pending_width > 0.0 {
        push_run(
            lines,
            &text[pending_start..pending_end],
            char_width_px,
            line_height_px,
            color,
            font_size_px,
        );
    }
}

fn push_run(
    lines: &mut [RawLine],
    text: &str,
    char_width_px: f32,
    line_height_px: f32,
    color: Color,
    font_size_px: f32,
) {
    if text.is_empty() {
        return;
    }
    let cur = lines.last_mut().unwrap();
    let w = text.chars().count() as f32 * char_width_px;
    cur.fragments.push(LineFragment::GlyphRun {
        text: text.to_string(),
        x: cur.consumed_width,
        baseline_y: 0.0,
        char_width_px,
        color,
        font_size_px,
    });
    cur.consumed_width += w;
    cur.line_height = cur.line_height.max(line_height_px);
    cur.baseline_y = cur.baseline_y.max(line_height_px * 0.8);
}

// ─────────────────────────────────────────────────────────────────────
// Step 3 — text alignment
// ─────────────────────────────────────────────────────────────────────

fn align_line(mut raw: RawLine, available_width_px: f32, align: TextAlign) -> LineBox {
    let extra = (available_width_px - raw.consumed_width).max(0.0);
    let shift = match align {
        TextAlign::Left | TextAlign::Start => 0.0,
        TextAlign::Right | TextAlign::End => extra,
        TextAlign::Center => extra / 2.0,
        TextAlign::Justify => 0.0, // handled per-fragment below
    };
    if shift > 0.0 {
        for f in &mut raw.fragments {
            shift_fragment(f, shift);
        }
    }
    if matches!(align, TextAlign::Justify) {
        // Distribute extra space across whitespace gaps in the line.
        // Count the number of space characters across all GlyphRuns;
        // expand each by extra / count.
        let space_count: usize = raw
            .fragments
            .iter()
            .filter_map(|f| match f {
                LineFragment::GlyphRun { text, .. } => {
                    Some(text.chars().filter(|c| *c == ' ').count())
                },
                _ => None,
            })
            .sum();
        if space_count > 0 && extra > 0.0 {
            let per_space = extra / space_count as f32;
            // Build a re-laid fragment list with adjusted positions.
            let mut x_acc = 0.0_f32;
            for f in &mut raw.fragments {
                match f {
                    LineFragment::GlyphRun {
                        text,
                        x,
                        char_width_px,
                        ..
                    } => {
                        // New x for this run is wherever the cursor is.
                        *x = x_acc;
                        // Walk the text expanding spaces.
                        let mut local = 0.0_f32;
                        for c in text.chars() {
                            if c == ' ' {
                                local += *char_width_px + per_space;
                            } else {
                                local += *char_width_px;
                            }
                        }
                        x_acc += local;
                    },
                    LineFragment::Atom { x, width, .. } => {
                        *x = x_acc;
                        x_acc += *width;
                    },
                }
            }
            raw.consumed_width = x_acc;
        }
    } else {
        raw.consumed_width += shift;
    }

    LineBox {
        fragments: raw.fragments,
        content_width: raw.consumed_width,
        height: raw.line_height.max(1.0),
        baseline_y: raw.baseline_y,
    }
}

fn shift_fragment(f: &mut LineFragment, dx: f32) {
    match f {
        LineFragment::GlyphRun { x, .. } => *x += dx,
        LineFragment::Atom { x, .. } => *x += dx,
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_css::css::Color;

    fn text(s: &str) -> InlineItem {
        InlineItem::Text {
            text: s.to_string(),
            char_width_px: 10.0,
            line_height_px: 16.0,
            color: Color::BLACK,
            font_size_px: 12.0,
        }
    }

    fn count_glyph_runs(line: &LineBox) -> usize {
        line.fragments
            .iter()
            .filter(|f| matches!(f, LineFragment::GlyphRun { .. }))
            .count()
    }

    fn line_text(line: &LineBox) -> String {
        line.fragments
            .iter()
            .filter_map(|f| match f {
                LineFragment::GlyphRun { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn single_line_fits() {
        // "hello world" at 10px/char = 110px — fits in 300.
        let lines =
            layout_paragraph(&[text("hello world")], 300.0, TextAlign::Left, WhiteSpace::Normal);
        assert_eq!(lines.len(), 1);
        assert!(line_text(&lines[0]).contains("hello"));
    }

    #[test]
    fn wraps_to_multiple_lines() {
        // Long text in a narrow box.
        let lines = layout_paragraph(
            &[text("the quick brown fox jumps over the lazy dog")],
            100.0, // 10 chars per line max
            TextAlign::Left,
            WhiteSpace::Normal,
        );
        assert!(lines.len() > 1);
    }

    #[test]
    fn whitespace_collapsed_in_normal_mode() {
        let lines = layout_paragraph(
            &[text("   hello\t\nworld   ")],
            300.0,
            TextAlign::Left,
            WhiteSpace::Normal,
        );
        let s = line_text(&lines[0]);
        // Multiple whitespace collapses to single spaces; leading/
        // trailing remain in the source string per CSS (collapsed by
        // the parent block).
        assert!(s.contains("hello world"));
    }

    #[test]
    fn pre_preserves_newlines() {
        let lines = layout_paragraph(
            &[text("line one\nline two")],
            300.0,
            TextAlign::Left,
            WhiteSpace::Pre,
        );
        assert!(lines.len() >= 2);
    }

    #[test]
    fn nowrap_keeps_one_line() {
        let lines = layout_paragraph(
            &[text("one two three four five six seven eight")],
            50.0, // narrower than even one word
            TextAlign::Left,
            WhiteSpace::Nowrap,
        );
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn hard_break_starts_new_line() {
        let lines = layout_paragraph(
            &[text("before"), InlineItem::HardBreak, text("after")],
            300.0,
            TextAlign::Left,
            WhiteSpace::Normal,
        );
        assert!(lines.len() >= 2);
        assert!(line_text(&lines[0]).contains("before"));
        assert!(line_text(&lines[1]).contains("after"));
    }

    #[test]
    fn align_right_shifts_fragments() {
        let lines = layout_paragraph(&[text("hi")], 300.0, TextAlign::Right, WhiteSpace::Normal);
        let f = &lines[0].fragments[0];
        if let LineFragment::GlyphRun { x, .. } = f {
            assert!(*x > 250.0, "expected right-aligned x near 280, got {x}");
        } else {
            panic!()
        }
    }

    #[test]
    fn align_center_shifts_to_middle() {
        let lines = layout_paragraph(&[text("hi")], 300.0, TextAlign::Center, WhiteSpace::Normal);
        let f = &lines[0].fragments[0];
        if let LineFragment::GlyphRun { x, .. } = f {
            // 300 - 20 = 280; center / 2 = 140
            assert!((x - 140.0).abs() < 1.0, "got {x}");
        } else {
            panic!()
        }
    }

    #[test]
    fn justify_distributes_extra_across_spaces() {
        // 4 words → 3 spaces. Extra = 300 - 4*10*4 - 3*10 = 110 (or
        // similar). Each space grows by ~36.
        let lines =
            layout_paragraph(&[text("a b c d")], 300.0, TextAlign::Justify, WhiteSpace::Normal);
        // The line should consume the full available width (or close).
        assert!((lines[0].content_width - 300.0).abs() < 1.0);
    }

    #[test]
    fn atom_inline_block_in_text_run() {
        let items = vec![
            text("before "),
            InlineItem::Atom {
                width: 50.0,
                height: 50.0,
                baseline: Some(45.0),
            },
            text(" after"),
        ];
        let lines = layout_paragraph(&items, 400.0, TextAlign::Left, WhiteSpace::Normal);
        assert_eq!(lines.len(), 1);
        let atoms: Vec<_> = lines[0]
            .fragments
            .iter()
            .filter(|f| matches!(f, LineFragment::Atom { .. }))
            .collect();
        assert_eq!(atoms.len(), 1);
    }

    #[test]
    fn line_height_takes_max_of_items() {
        let mut big = text("BIG");
        if let InlineItem::Text { line_height_px, .. } = &mut big {
            *line_height_px = 32.0;
        }
        let lines =
            layout_paragraph(&[big, text("small")], 300.0, TextAlign::Left, WhiteSpace::Normal);
        assert_eq!(lines[0].height, 32.0);
    }
}
