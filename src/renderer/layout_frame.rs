//! Physical-row geometry for paragraph layout.

use std::ops::Range;

use crate::model::paragraph::LineSeg;

const SEGMENT_BOUNDARY_TAGS: u32 = LineSeg::TAG_FIRST_SEGMENT | LineSeg::TAG_LAST_SEGMENT;
const MINIMUM_USABLE_INTERVAL_HWP: i32 = 1_440;
/// The column solver's unconditional inline-extent quantum.
///
/// The recovered column-solver trace records this as a third quantization above
/// the paragraph grid: for equal-width columns the solver truncates the full width
/// with `÷4×4` before the paragraph builder sees it. It is not `snapToGrid` and
/// it does not ceil/floor the paragraph's post-margin edges. #1440 demonstrates
/// the ordering: column width `38268` stays `38268`, then 850-unit left/right
/// margins produce stored `850..37418`. Snapping those edges independently
/// produced the false `852..37416` cache mismatch.
///
/// `SectionDef::char_grid` and the paragraph `snapToGrid` gate remain the
/// separate, unimplemented §2.12 builder/fill pitches.
///
/// The value 4 stays measured rather than derived: over 455 corpus documents
/// and 160,069 complete single-segment stored rows, the difference between our
/// column box and HWP's stored right edge is exactly `body_width % 4` in
/// 160,052 of them — `(delta 0, bw%4 0): 106,674`, `(1,1): 423`,
/// `(2,2): 52,835`, `(3,3): 120`, 17 exceptions. That is a snap-down to a
/// 4-HWPUNIT grid, not a constant inset, which is why a flat `-2` fixed the
/// `bw%4==2` documents and broke the `bw%4==0` ones.
///
const COLUMN_WIDTH_QUANTUM_HWP: i32 = 4;

/// Snap a base edge down to `pitch`.
///
/// `pitch <= 0` is the identity, which is what §2.5's gated arm produces.
///
/// `rem_euclid` floors negative values (`-2001` becomes `-2004`). Current
/// callers provide non-negative column origins and margins; widening that input
/// domain requires an explicit rounding decision.
fn snap_base_right(edge: i32, pitch: i32) -> i32 {
    match edge.checked_rem_euclid(pitch) {
        Some(remainder) => edge - remainder,
        None => edge,
    }
}

/// Snap a base edge up to `pitch`.
///
/// Saturating, unlike the plain `+` this replaced: it was the only arithmetic
/// in this module that could overflow, two functions from a `checked_add`. See
/// [`snap_base_right`] for the zero-pitch and rounding notes.
fn snap_base_left(edge: i32, pitch: i32) -> i32 {
    match edge.checked_rem_euclid(pitch) {
        Some(0) | None => edge,
        Some(remainder) => edge.saturating_add(pitch - remainder),
    }
}

/// The horizontal box one paragraph flow is given, **and the coordinate system
/// it is in**.
///
/// A width alone cannot say this, and that is the whole reason this type
/// exists. The two constructors are not a formatting preference; they are two
/// different coordinate systems, and a caller that cannot name its own cannot
/// be handed a correct box:
///
/// - [`ParagraphBox::column`] carries an **origin**. HWP stores `column_start`
///   relative to the column, not to the paragraph, so a body paragraph's box is
///   `[margin_left, column_width - margin_right]` and its published record must
///   say so. [`ParagraphBox::body`] first applies the column solver's width
///   quantum, then applies these paragraph margins.
/// - [`ParagraphBox::content`] belongs to a flow that has no column of its own —
///   a table cell, a footnote, a header, a caption, a text box. Its origin is
///   that flow's own left edge. Its width was already resolved by that owner,
///   so the body column solver must not touch it.
///
/// Both `column_start` and `segment_width` of every published `LineSeg` come out
/// of [`ParagraphBox::effective`], so the record a reflow writes and the box the
/// frame carves can no longer disagree — before this type they were computed in
/// two places from two different quantities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphBox {
    horizontal: Range<i32>,
    origin_is_derivable: bool,
}

impl ParagraphBox {
    /// A column-relative box: `[margin_left, column_width - margin_right]`, the
    /// same range the render sites hand `LayoutFrame`.
    pub(crate) fn column(horizontal: Range<i32>) -> Self {
        crate::hot_call!(Self::column_hot_impl, horizontal)
    }

    fn column_hot_impl(horizontal: Range<i32>) -> Self {
        Self {
            horizontal,
            origin_is_derivable: true,
        }
    }

    /// The column box of a body paragraph — the column's own edges inset by the
    /// resolved paragraph margins.
    ///
    /// This is the one expression the edit path and the three render sites
    /// share. One paragraph must not get two different boxes depending on which
    /// route reached it, and having a single constructor is what makes "the same
    /// frame" structural rather than a comment kept true by hand.
    ///
    /// The origin is right in general: `column_start ==
    /// px_to_hwpunit(margin_left)` on **189,576 of 191,057** corpus rows
    /// carrying authentic stored `LINE_SEG`s (99.22%). See
    /// [`ParagraphBox::with_derivable_origin`] for the residual.
    pub(crate) fn body(
        column_width_px: f64,
        margin_left_px: f64,
        margin_right_px: f64,
        dpi: f64,
    ) -> Self {
        crate::hot_call!(
            Self::body_hot_impl,
            column_width_px,
            margin_left_px,
            margin_right_px,
            dpi
        )
    }

    fn body_hot_impl(
        column_width_px: f64,
        margin_left_px: f64,
        margin_right_px: f64,
        dpi: f64,
    ) -> Self {
        Self::body_hwp(
            crate::renderer::px_to_hwpunit(column_width_px, dpi),
            crate::renderer::px_to_hwpunit(margin_left_px, dpi),
            crate::renderer::px_to_hwpunit(margin_right_px, dpi),
        )
    }

    pub(crate) fn body_hwp(
        column_width_hwp: i32,
        margin_left_hwp: i32,
        margin_right_hwp: i32,
    ) -> Self {
        crate::hot_call!(
            Self::body_hwp_hot_impl,
            column_width_hwp,
            margin_left_hwp,
            margin_right_hwp,
        )
    }

    fn body_hwp_hot_impl(
        column_width_hwp: i32,
        margin_left_hwp: i32,
        margin_right_hwp: i32,
    ) -> Self {
        // The column solver quantizes the full inline extent before paragraph
        // margins are applied. Snapping post-margin edges would incorrectly
        // turn 850..37418 into 852..37416 (#1440).
        let width = snap_base_right(column_width_hwp, COLUMN_WIDTH_QUANTUM_HWP);
        Self::column(margin_left_hwp..width.saturating_sub(margin_right_hwp))
    }

    /// [`ParagraphBox::body`] for a paragraph whose resolved style is known.
    ///
    /// Reads `head_type` so the list blocker in
    /// [`ParagraphBox::with_derivable_origin`] is applied in one place instead of
    /// being remembered at each body call site.
    pub(crate) fn body_for_style(
        column_width_px: f64,
        style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
        dpi: f64,
    ) -> Self {
        crate::hot_call!(Self::body_for_style_hot_impl, column_width_px, style, dpi)
    }

    fn body_for_style_hot_impl(
        column_width_px: f64,
        style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
        dpi: f64,
    ) -> Self {
        Self::body_hwp_for_style(
            crate::renderer::px_to_hwpunit(column_width_px, dpi),
            style,
            dpi,
        )
    }

    pub(crate) fn body_hwp_for_style(
        column_width_hwp: i32,
        style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
        dpi: f64,
    ) -> Self {
        crate::hot_call!(
            Self::body_hwp_for_style_hot_impl,
            column_width_hwp,
            style,
            dpi,
        )
    }

    fn body_hwp_for_style_hot_impl(
        column_width_hwp: i32,
        style: Option<&crate::renderer::style_resolver::ResolvedParaStyle>,
        dpi: f64,
    ) -> Self {
        use crate::model::style::HeadType;
        let margin_left = style.map(|value| value.margin_left).unwrap_or(0.0);
        let margin_right = style.map(|value| value.margin_right).unwrap_or(0.0);
        let head_type = style.map(|value| value.head_type).unwrap_or(HeadType::None);
        Self::body_hwp(
            column_width_hwp,
            crate::renderer::px_to_hwpunit(margin_left, dpi),
            crate::renderer::px_to_hwpunit(margin_right, dpi),
        )
        .with_derivable_origin(matches!(head_type, HeadType::None | HeadType::Outline))
    }

    /// Declare whether this box's **origin** may be published, or only its width.
    ///
    /// `false` is a **named blocker**, and it is temporary. The residual 0.78% of
    /// the measurement on [`ParagraphBox::body`] is one class, not noise: a
    /// numbered or bulleted marker hangs *left* of `margin_left`, so HWP stores a
    /// smaller `column_start` (deltas `-2000`, `-2500`, `-2252` — 731 rows).
    /// `para-head-num-2.hwp` paragraph 0.1 has ParaShape `margin_left = 4000` and
    /// stored `cs = 2000`, with `cs + sw = 42520` exactly the column width.
    ///
    /// The reason to withhold the origin is **not** that it cannot be computed —
    /// the paragraph's own stored record has it. It is that the render side
    /// cannot yet consume it for an *empty* list paragraph. The
    /// `issue_1329_bullet_caret` fixture records the decisive contrast:
    ///
    /// | paragraph | stored record | offset-0 caret |
    /// | --- | --- | --- |
    /// | 0.1, pristine, has text | `cs=2000 sw=40520` | 164.1 |
    /// | 0.2, split, empty | `cs=2000 sw=40520` | **190.7** |
    ///
    /// Identical records, 26.6px apart. A non-empty list line starts at
    /// `column_start` (140.1px) and its body run lands at 164.1; the empty line's
    /// marker run is emitted at `col_area.x + margin_left` = 166.7px with width
    /// 24.0 (`paragraph_layout.rs`), so `cursor_rect.rs`'s `marker_end_x`
    /// fallback reports 190.7. Publishing a truthful origin makes those two
    /// disagree by exactly one marker advance, which is what
    /// `issue_1329_bullet_caret` pins.
    ///
    /// So a list paragraph publishes `column_start = 0` for now: the pre-existing
    /// value, wrong in the way it has always been wrong, and consistent between
    /// the record and the frame. **The width lane is not withheld** —
    /// `segment_width` still comes from the column box and still takes the
    /// geometry pitch, which is the exposure Task 3 measured. Fixing the
    /// empty-list marker placement is the prerequisite for removing this.
    pub(crate) fn with_derivable_origin(self, origin_is_derivable: bool) -> Self {
        crate::hot_call!(
            Self::with_derivable_origin_hot_impl,
            self,
            origin_is_derivable
        )
    }

    fn with_derivable_origin_hot_impl(self, origin_is_derivable: bool) -> Self {
        Self {
            origin_is_derivable: self.origin_is_derivable && origin_is_derivable,
            ..self
        }
    }

    /// A box in a nested flow's own coordinates. Pass the real inset when the
    /// caller has one; `0..width` when the flow's left edge *is* the origin.
    pub(crate) fn content(horizontal: Range<i32>) -> Self {
        crate::hot_call!(Self::content_hot_impl, horizontal)
    }

    fn content_hot_impl(horizontal: Range<i32>) -> Self {
        Self {
            horizontal,
            origin_is_derivable: true,
        }
    }

    /// A nested flow's content box given as a width in pixels.
    ///
    /// Public because the cell rebuild it feeds
    /// (`composer::recompose_cell_lines_in_frame`) is public, and a caller that
    /// cannot name its own coordinate system cannot use that entry at all.
    pub fn content_width_px(width_px: f64, dpi: f64) -> Self {
        crate::hot_call!(Self::content_width_px_hot_impl, width_px, dpi)
    }

    fn content_width_px_hot_impl(width_px: f64, dpi: f64) -> Self {
        Self::content(0..crate::renderer::px_to_hwpunit(width_px, dpi))
    }

    /// The box after the geometry pitch — the single source for both the
    /// published record and the carved frame.
    pub(crate) fn effective(&self) -> Range<i32> {
        crate::hot_call!(Self::effective_hot_impl, self)
    }

    fn effective_hot_impl(&self) -> Range<i32> {
        if self.origin_is_derivable {
            self.horizontal.clone()
        } else {
            0..self.horizontal.end.saturating_sub(self.horizontal.start)
        }
    }

    pub(crate) fn width_hwp(&self) -> i32 {
        let effective = self.effective();
        effective.end.saturating_sub(effective.start)
    }

    /// Whether this box can carry a row at all — **the one rule, for every
    /// route**.
    ///
    /// False when the resolved margins meet or cross: a paragraph whose
    /// `margin_left` reaches `column_width - margin_right` has no usable width,
    /// which is legal input (a large left indent inside a narrow column, or a
    /// multi-column layout whose spacing eats the body). Measured on
    /// `effective()`, so it also catches a box the geometry pitch itself
    /// inverted — `snap_base_left(1)..snap_base_right(3)` is `4..0`.
    ///
    /// **Refusal, not a floor, and the distinction is load-bearing.** The
    /// narrow-base fallback that would discard paragraph margins and create a
    /// minimum-width row is not implemented. Flooring here would invent a
    /// different third behavior and hide that missing owner.
    ///
    /// A floor is also what hid this: the retired
    /// `(col_width - margin_left - margin_right).max(1.0)` turned an impossible
    /// box into a 75-HWPUNIT one, which reflows to one token per line and looks
    /// like a valid record. Refusing keeps HWP's own record, which is the same
    /// rule the frame already follows for a row it cannot reproduce (#4779),
    /// and it leaves the narrow-base fallback a live decision rather than
    /// something that has to displace a fabricated number.
    pub(crate) fn is_usable(&self) -> bool {
        self.width_hwp() > 0
    }

    pub(crate) fn width_px(&self, dpi: f64) -> f64 {
        crate::renderer::hwpunit_to_px(self.width_hwp(), dpi)
    }

    /// The frame this box describes, carrying the caller's wrap geometry.
    ///
    /// Built from `effective()` so the frame and the published record cannot
    /// drift. The picture band is routed through the same box rather than
    /// deriving another horizontal range: two expressions for one quantity are
    /// what previously let the band and body disagree.
    pub(crate) fn frame_with(&self, top: i32, exclusions: Vec<FrameExclusion>) -> LayoutFrame {
        crate::hot_call!(Self::frame_with_hot_impl, self, top, exclusions)
    }

    fn frame_with_hot_impl(&self, top: i32, exclusions: Vec<FrameExclusion>) -> LayoutFrame {
        LayoutFrame::new(self.effective(), top, exclusions)
    }

    /// [`ParagraphBox::frame_with`] for a flow that models no wrap geometry.
    ///
    /// Every body call site is here today, which is why `models_exclusions()`
    /// is false on the stored-row route — see
    /// `line_breaking::stored_rows_reproduce_frame_expectation`.
    pub(crate) fn frame(&self, top: i32) -> LayoutFrame {
        self.frame_with(top, Vec::new())
    }
}
/// The vertical values shared by every horizontal segment in one physical row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FrameRowMetrics {
    pub(crate) vertical_pos: i32,
    pub(crate) line_height: i32,
    pub(crate) text_height: i32,
    pub(crate) baseline_distance: i32,
    pub(crate) line_spacing: i32,
}

/// One horizontal part of a physical row before it is flattened into a
/// `LineSeg`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowSegment {
    pub(crate) text_range: Range<u32>,
    pub(crate) horizontal: Range<i32>,
    /// The source tag retains provenance and line properties. Projection owns
    /// the FIRST/LAST boundary bits because they describe this row's group.
    pub(crate) source_tag: u32,
}

impl RowSegment {
    pub(crate) fn new(text_range: Range<u32>, horizontal: Range<i32>, source_tag: u32) -> Self {
        Self {
            text_range,
            horizontal,
            source_tag: source_tag & !SEGMENT_BOUNDARY_TAGS,
        }
    }
}

/// A completed physical row. Its vertical values are recorded once, while its
/// horizontal segments retain their left-to-right order until projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PhysicalRow {
    metrics: FrameRowMetrics,
    segments: Vec<RowSegment>,
}

/// Whether a stored row is the row this frame expected.
///
/// Exact equality covers interval count, `column_start`, and `segment_width`.
///
/// Stored geometry is reusable only when the current Frame reproduces it. Row
/// metrics (`vertical_pos`, height, baseline, spacing, and overhangs) are a
/// separate lane and do not widen this cache key. There is no geometry
/// tolerance.
///
/// Two things this predicate must therefore **not** absorb, because a
/// difference here is a bug in our carve or a gap in our inputs, and hiding it
/// destroys the evidence:
///
/// - Glyph shaping and kerning line-boundary differences → #4439.
/// - Column-solver quantization belongs in `ParagraphBox::body`, before
///   paragraph margins. This predicate must not absorb it a second time.
fn stored_row_matches_frame_expectation(expected: &Range<i32>, stored: &LineSeg) -> bool {
    crate::hot_call!(
        stored_row_matches_frame_expectation_hot_impl,
        expected,
        stored
    )
}

fn stored_row_matches_frame_expectation_hot_impl(expected: &Range<i32>, stored: &LineSeg) -> bool {
    expected.start == stored.column_start
        && stored
            .column_start
            .checked_add(stored.segment_width)
            .is_some_and(|end| expected.end == end)
}

/// The side-wrap choices represented by this physical-row frame. This is
/// layout geometry, rather than a mirror of model `TextFlow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameExclusionPolicy {
    BothSides,
    LargestSide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FrameExclusion {
    pub(crate) horizontal: Range<i32>,
    pub(crate) vertical: Range<i32>,
    pub(crate) policy: FrameExclusionPolicy,
}

/// Mutable geometry of one layout flow.
///
/// A carve is tentative. It may move the candidate row to an exact geometry
/// event, but it does not commit text or advance past a completed row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayoutFrame {
    pub(crate) horizontal: Range<i32>,
    pub(crate) top: i32,
    pub(crate) exclusions: Vec<FrameExclusion>,
    pub(crate) current_intervals: Vec<Range<i32>>,
    pub(crate) next_geometry_event: Option<i32>,
    pub(crate) minimum_width: i32,
    /// Whether `horizontal` is a column edge pair.
    ///
    /// The geometry pitch snaps the column's edge pair. A table cell's content
    /// width and `reflow_line_segs`'
    /// width-only box are not column edges, so snapping them has no native
    /// basis — and doing so moves cell frame widths that the table owner
    /// already resolved.
    rows: Vec<PhysicalRow>,
}

impl LayoutFrame {
    /// Start a paragraph-local physical frame at a known horizontal extent.
    pub(crate) fn new(horizontal: Range<i32>, top: i32, exclusions: Vec<FrameExclusion>) -> Self {
        crate::hot_call!(Self::new_hot_impl, horizontal, top, exclusions)
    }

    fn new_hot_impl(horizontal: Range<i32>, top: i32, exclusions: Vec<FrameExclusion>) -> Self {
        Self {
            horizontal,
            top,
            exclusions,
            current_intervals: Vec::new(),
            next_geometry_event: None,
            minimum_width: MINIMUM_USABLE_INTERVAL_HWP,
            rows: Vec::new(),
        }
    }

    /// Discard an uncommitted row trial and restore its exact frame state.
    pub(crate) fn restore_checkpoint(&mut self, checkpoint: Self) {
        crate::hot_call!(Self::restore_checkpoint_hot_impl, self, checkpoint)
    }

    fn restore_checkpoint_hot_impl(&mut self, checkpoint: Self) {
        *self = checkpoint;
    }

    /// Carve the current physical row into ordered horizontal intervals.
    pub(crate) fn carve(&mut self, band_height: i32) -> &[Range<i32>] {
        crate::hot_call!(Self::carve_hot_impl, self, band_height);
        &self.current_intervals
    }

    fn carve_hot_impl(&mut self, band_height: i32) {
        let old_max = self.next_geometry_event.map(|_| {
            self.current_intervals
                .iter()
                .map(|interval| interval.end - interval.start)
                .max()
                .unwrap_or(0)
        });
        let mut candidate_top = self.top;

        loop {
            let bottom = candidate_top.saturating_add(band_height.max(0));
            let base = self.horizontal.clone();
            let mut intervals = vec![base.clone()];
            let mut next_geometry_event = None;
            let minimum_width = self.minimum_width.max(0);
            let mut adequate = base.end - base.start >= minimum_width;

            for exclusion in &self.exclusions {
                if exclusion.vertical.start >= bottom || candidate_top >= exclusion.vertical.end {
                    continue;
                }

                let before = intervals.clone();
                let left = exclusion.horizontal.start;
                let right = exclusion.horizontal.end;
                let mut carved = Vec::with_capacity(intervals.len() + 1);

                for interval in &intervals {
                    if interval.end <= left || right <= interval.start {
                        carved.push(interval.clone());
                    } else if interval.start < left && right < interval.end {
                        let left_interval = interval.start..left;
                        let right_interval = right..interval.end;
                        if exclusion.policy == FrameExclusionPolicy::LargestSide {
                            let left_width = left_interval.end - left_interval.start;
                            let right_width = right_interval.end - right_interval.start;
                            // HWP's `LargestOnly` choice normally keeps the
                            // widest side, ties left, and has one recovered
                            // narrow-lane exception: a 1,440-HWP left lane
                            // chooses the right side even when it is shorter.
                            if left_width == MINIMUM_USABLE_INTERVAL_HWP || left_width < right_width
                            {
                                carved.push(right_interval);
                            } else {
                                carved.push(left_interval);
                            }
                        } else {
                            carved.push(left_interval);
                            carved.push(right_interval);
                        }
                    } else if interval.start < left {
                        carved.push(interval.start..left);
                    } else if right < interval.end {
                        carved.push(right..interval.end);
                    } else {
                        carved.push(interval.start..interval.start);
                    }
                }

                let changed = before != carved;
                intervals = carved;
                if changed {
                    next_geometry_event = Some(
                        next_geometry_event.map_or(exclusion.vertical.end, |event: i32| {
                            event.min(exclusion.vertical.end)
                        }),
                    );
                }

                adequate = intervals
                    .iter()
                    .any(|interval| interval.end - interval.start >= minimum_width);
                if !adequate {
                    break;
                }
            }

            let mut index = 0;
            while intervals.len() > 1 && index < intervals.len() {
                if intervals[index].end - intervals[index].start < minimum_width {
                    intervals.remove(index);
                } else {
                    index += 1;
                }
            }

            let retry = if !adequate {
                next_geometry_event.filter(|event| *event > candidate_top)
            } else {
                let new_max = intervals
                    .iter()
                    .map(|interval| interval.end - interval.start)
                    .max()
                    .unwrap_or(0);
                old_max
                    .filter(|previous| *previous > new_max)
                    .and(next_geometry_event)
                    .filter(|event| *event > candidate_top)
            };

            if let Some(event) = retry {
                candidate_top = event;
                continue;
            }

            self.top = candidate_top;
            self.current_intervals = intervals;
            self.next_geometry_event = next_geometry_event;
            return;
        }
    }

    /// Admit stored rows only where this frame's own carve already expects
    /// them, so the strict reflow can be skipped without ceding ownership.
    ///
    /// The order is frame first. `carve()` computes what the frame expects the
    /// row to be — its vertical band and its left-to-right intervals — and by
    /// #4755 §2 it neither fills text nor produces a `LineSeg`. That is the
    /// cheap half. The stored row is then compared against that expectation on
    /// equality.
    ///
    /// **What the answer does, exactly.** It is the production cache-admission
    /// gate. A match leaves the frame's recomputed rows committed and permits
    /// the stored text partition to stand. A mismatch restores the exact entry
    /// checkpoint and the caller attempts strict reflow. Text staleness remains
    /// an independent invalidator even when this geometry key matches.
    ///
    /// #4755 §1 bounds what the stored row may contribute: its horizontal
    /// extent is only ever compared, never adopted — the committed segment is
    /// the carved interval.
    ///
    /// Row metrics come from `metrics_for`, not from the stored record.
    /// §1.4.1 recomputes `vertical_pos`, `line_height`, `text_height`,
    /// `baseline_distance` and `line_spacing` on both arms, so the frame must
    /// never read them. The provider is given one FIRST..LAST group and returns
    /// that row's metrics; the resulting `line_height` is what `carve()` is
    /// asked to band.
    ///
    /// **Known divergence from §1.4.1 — the accept-arm write-back is not
    /// implemented, and this is deliberate.** §1.4.1 does not stop at
    /// recomputing: on accept it writes the recomputed metrics back over every
    /// segment in the physical row. We compute the metrics into
    /// `commit_carved_row` and leave them there; on the admitted arm
    /// `recompose_stored_lines_in_frame` returns `None`, so
    /// `project_line_segs()` never runs and the frame's metrics have **no
    /// outward path at all**. What flows on instead is the stored record, via
    /// `compose_lines`, which copies `line_height`, `baseline_distance` and
    /// `line_spacing` straight off the `LineSeg`.
    ///
    /// The reader does see a different value. On `samples/field-01.hwp` page 0
    /// (`export-render-tree`), 25 admissions hit `project_line_segs` 0 times,
    /// and 3 of the 25 rows computed `line_spacing = 450` against a stored
    /// `452`; `TypesetEngine::format_paragraph` then read `line_spacing = 452`
    /// into `corrected_line_metrics_for_source`. §2.14's metricCtx settlement
    /// is the residual — the stored value is Hancom's, ours is the
    /// approximation §1.4.2 warns about ("the real bug is … an inexact
    /// COMPUTE").
    ///
    /// Implementing it was measured, not assumed. Publishing the projection on
    /// the admitted arm exactly as the reflowed arm already does took the suite
    /// from 5983 passed / 2 failed to **5979 passed / 6 failed**: two
    /// Hancom-PDF oracle pins (`issue_1116`'s `lh=17.3 ls=10.4`,
    /// `used=874.5px`), one SVG golden (`svg_snapshot::issue_157_page_1`), and
    /// `issue_4576`'s negative control, which stopped seeing the composition
    /// corruption it deliberately injects because the recompute healed it.
    /// This cache-admission change activates §5 item 5c. It deliberately does
    /// not activate the separate accept-arm metric write-back.
    ///
    /// Note also that §1.4.1's prose and its own pseudocode disagree about
    /// which lanes the accept branch writes: the prose names `vertical_pos`,
    /// `line_height`, `text_height`, `baseline_distance`, `line_spacing` and
    /// the overhangs, while the pseudocode writes `+0x04` and `+0x4c…+0x58` —
    /// i.e. `vertical_pos` and the overhang block, but **not** the `+0x08…+0x14`
    /// row-metric lanes that §1.3 maps to `line_height`/`text_height`/
    /// `baseline`/`line_spacing`. The lanes rhwp's render path can actually
    /// write (`ComposedLine` carries `line_height`, `baseline_distance`,
    /// `line_spacing`) are the ones the pseudocode omits, and the two it does
    /// list are unreachable from here — `vertical_pos` and `text_height` are
    /// read off `para.line_segs`, which the render path holds by shared
    /// reference. Treat the `file:line` and offset citations per §1.5 item 3 /
    /// §6.C24.
    ///
    /// A mismatch restores the exact entry state, so the frame is left exactly
    /// as the caller handed it in and a later pass may reuse it.
    pub(crate) fn try_admit_stored_rows(
        &mut self,
        line_segs: &[LineSeg],
        metrics_for: impl Fn(&[LineSeg]) -> Option<FrameRowMetrics>,
    ) -> bool {
        crate::hot_call!(
            Self::try_admit_stored_rows_hot_impl,
            self,
            line_segs,
            &metrics_for,
        )
    }

    fn try_admit_stored_rows_hot_impl(
        &mut self,
        line_segs: &[LineSeg],
        metrics_for: &dyn Fn(&[LineSeg]) -> Option<FrameRowMetrics>,
    ) -> bool {
        let checkpoint = self.clone();
        let admitted = self.admit_stored_rows(line_segs, metrics_for).is_some();
        if !admitted {
            self.restore_checkpoint(checkpoint);
        }
        admitted
    }

    fn admit_stored_rows(
        &mut self,
        line_segs: &[LineSeg],
        metrics_for: impl Fn(&[LineSeg]) -> Option<FrameRowMetrics>,
    ) -> Option<()> {
        // Text order is a structural precondition: the rows are consumed in
        // sequence and a rewind means the records do not describe one flow.
        //
        // `vertical_pos` is deliberately **not** here. §1.3/§1.4.1: the native
        // validator never compares it — it is recomputed every pass and written
        // back over the stored record — so it cannot take part in the decision.
        if line_segs.is_empty()
            || line_segs
                .windows(2)
                .any(|pair| pair[0].text_start > pair[1].text_start)
        {
            return None;
        }

        let mut consumed = 0usize;
        while consumed < line_segs.len() {
            if !line_segs[consumed].is_first_segment() {
                return None;
            }
            // One physical row is FIRST..LAST, however many slots that spans.
            let count = line_segs[consumed..]
                .iter()
                .position(LineSeg::is_last_segment)?
                + 1;
            let stored_row = &line_segs[consumed..consumed + count];

            let metrics = metrics_for(stored_row)?;
            if metrics.line_height <= 0 {
                return None;
            }

            let candidate_top = self.top;
            let intervals = self.carve(metrics.line_height).to_vec();
            if self.top != candidate_top {
                // The carve moved to a geometry event, so the frame does not
                // expect a row here at all.
                return None;
            }

            // §1.4.1's three quantities: interval COUNT, then horzpos and
            // horzsize per slot, all by exact equality.
            if intervals.len() != count
                || intervals.iter().zip(stored_row).any(|(expected, stored)| {
                    !stored_row_matches_frame_expectation(expected, stored)
                })
            {
                return None;
            }

            // The committed geometry is the frame's own carve, never the stored
            // extent — they are equal here, and equality is the point.
            let segments = intervals
                .iter()
                .zip(stored_row)
                .map(|(expected, stored)| {
                    RowSegment::new(
                        stored.text_start..stored.text_start,
                        expected.clone(),
                        stored.tag,
                    )
                })
                .collect();
            self.commit_carved_row(metrics, segments)?;
            consumed += count;
        }
        Some(())
    }

    /// Whether the intervals from the last `carve()` can carry a row.
    ///
    /// **The one validity rule, for both arms.** A carve can return an interval
    /// that no row can occupy: an exclusion covering a whole interval leaves
    /// `start..start`, and a base rect narrower than the geometry pitch comes
    /// back inverted (`snap_base_left(1)..snap_base_right(3)` is `4..0`). The
    /// minimum-width prune does not remove either, because it never deletes the
    /// last interval, so the final validity check must reject zero/inverted
    /// geometry explicitly.
    ///
    /// Both arms reach this through [`LayoutFrame::commit_carved_row`], which
    /// every committed row passes. `layout_paragraph_in_frame` also asks
    /// *before* filling text, but only as an optimization — it is the same
    /// question, not a second rule. The admitted arm asked nothing at all
    /// before this, so a degenerate carve could commit a row and
    /// `project_line_segs` would emit it: `segment_width` comes from
    /// `end.saturating_sub(start)`, which does not clamp at zero, so an
    /// inverted interval published a negative width — the same corrupt record
    /// `ParagraphBox::is_usable` refuses upstream.
    pub(crate) fn carved_row_is_usable(&self) -> bool {
        crate::hot_call!(Self::carved_row_is_usable_hot_impl, self)
    }

    fn carved_row_is_usable_hot_impl(&self) -> bool {
        !self.current_intervals.is_empty()
            && self
                .current_intervals
                .iter()
                .all(|interval| interval.start < interval.end)
    }

    /// Commit the row carved at the current frame position.
    ///
    /// The caller supplies exactly one text result for every interval returned
    /// by `carve`. The frame gives all of them one vertical position, retains
    /// them as one physical row, then advances exactly once.
    pub(crate) fn commit_carved_row(
        &mut self,
        metrics: FrameRowMetrics,
        segments: Vec<RowSegment>,
    ) -> Option<usize> {
        crate::hot_call!(Self::commit_carved_row_hot_impl, self, metrics, segments)
    }

    fn commit_carved_row_hot_impl(
        &mut self,
        mut metrics: FrameRowMetrics,
        segments: Vec<RowSegment>,
    ) -> Option<usize> {
        if !self.carved_row_is_usable()
            || segments.len() != self.current_intervals.len()
            || segments
                .iter()
                .zip(&self.current_intervals)
                .any(|(segment, interval)| segment.horizontal != *interval)
        {
            return None;
        }

        metrics.vertical_pos = self.top;
        let row_index = self.rows.len();
        self.rows.push(PhysicalRow { metrics, segments });
        self.top = self
            .top
            .saturating_add(metrics.line_height)
            .saturating_add(metrics.line_spacing);
        // A carved interval belongs only to the row just committed. Requiring
        // a new carve prevents a caller from advancing this frame twice with
        // stale geometry.
        self.current_intervals.clear();
        Some(row_index)
    }

    /// Whether this frame models any wrap geometry at all.
    ///
    /// The body call site constructs its frame with an empty exclusion list:
    /// a float anchored in another paragraph never reaches `para.controls`,
    /// and neither consumer can name the float set today (the render path has
    /// a `WrapAnchorRef` and does not forward it; the measurement path loses
    /// it at the `format_paragraph` boundary). A frame in that state can carve
    /// exactly one full-width interval, so it has nothing to say about rows
    /// that some geometry it cannot see has split.
    pub(crate) fn models_exclusions(&self) -> bool {
        !self.exclusions.is_empty()
    }

    pub(crate) fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Flatten retained physical rows at the document boundary.
    pub(crate) fn project_line_segs(&self) -> Vec<LineSeg> {
        crate::hot_call!(Self::project_line_segs_hot_impl, self)
    }

    fn project_line_segs_hot_impl(&self) -> Vec<LineSeg> {
        let mut projected = Vec::new();

        for row in &self.rows {
            let last_segment = row.segments.len().saturating_sub(1);
            for (segment_index, segment) in row.segments.iter().enumerate() {
                let mut tag = segment.source_tag & !SEGMENT_BOUNDARY_TAGS;
                if segment_index == 0 {
                    tag |= LineSeg::TAG_FIRST_SEGMENT;
                }
                if segment_index == last_segment {
                    tag |= LineSeg::TAG_LAST_SEGMENT;
                }
                projected.push(LineSeg {
                    text_start: segment.text_range.start,
                    vertical_pos: row.metrics.vertical_pos,
                    line_height: row.metrics.line_height,
                    text_height: row.metrics.text_height,
                    baseline_distance: row.metrics.baseline_distance,
                    line_spacing: row.metrics.line_spacing,
                    column_start: segment.horizontal.start,
                    segment_width: segment
                        .horizontal
                        .end
                        .saturating_sub(segment.horizontal.start),
                    tag,
                });
            }
        }

        projected
    }

    /// Flatten only rows appended after a paragraph-local checkpoint.
    pub(crate) fn project_line_segs_since(&self, first_row: usize) -> Vec<LineSeg> {
        crate::hot_call!(Self::project_line_segs_since_hot_impl, self, first_row)
    }

    fn project_line_segs_since_hot_impl(&self, first_row: usize) -> Vec<LineSeg> {
        let first_segment = self.rows[..first_row.min(self.rows.len())]
            .iter()
            .map(|row| row.segments.len())
            .sum();
        self.project_line_segs()
            .into_iter()
            .skip(first_segment)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(line_height: i32, line_spacing: i32) -> FrameRowMetrics {
        FrameRowMetrics {
            vertical_pos: -1,
            line_height,
            text_height: line_height - 10,
            baseline_distance: line_height - 20,
            line_spacing,
        }
    }

    /// Test provider: the fixtures encode the row size they intend in the
    /// stored record, so echo it back. Production supplies computed metrics.
    fn echo_metrics(row: &[LineSeg]) -> Option<FrameRowMetrics> {
        let first = row.first()?;
        Some(FrameRowMetrics {
            vertical_pos: first.vertical_pos,
            line_height: first.line_height,
            text_height: first.text_height,
            baseline_distance: first.baseline_distance,
            line_spacing: first.line_spacing,
        })
    }

    fn frame(horizontal: Range<i32>, top: i32, exclusions: Vec<FrameExclusion>) -> LayoutFrame {
        LayoutFrame {
            horizontal,
            top,
            exclusions,
            current_intervals: Vec::new(),
            next_geometry_event: None,
            minimum_width: 1,
            rows: Vec::new(),
        }
    }

    /// **Every body route builds the same box for the same paragraph.**
    ///
    /// The origin blocker lives in `body_for_style`, so a route calling bare
    /// `body` skips it. For a list paragraph with `margin_left > 0` that is not
    /// a cosmetic difference: the edit route publishes `column_start = 0` while
    /// the render route's frame carves at `snap_base_left(margin_left)`, and
    /// `stored_row_matches_frame_expectation` compares by exact equality — so
    /// every row of an edited list paragraph fails admission, systematically.
    /// `with_derivable_origin`'s own doc claims the withheld origin stays
    /// "consistent between the record and the frame"; that was true on one
    /// route and false on four.
    ///
    /// The width lane is deliberately *not* withheld, and this pins that too:
    /// both boxes are the same width, so only the origin is at stake and no
    /// route can be wrapping to a different measure.
    fn every_body_route_builds_one_box_for_one_paragraph() {
        use crate::model::style::HeadType;
        use crate::renderer::style_resolver::ResolvedParaStyle;

        let dpi = crate::renderer::HWPUNIT_PER_INCH; // 1 px == 1 HWPUNIT
        let listed = |head_type| ResolvedParaStyle {
            margin_left: 2_000.0,
            margin_right: 1_000.0,
            head_type,
            ..Default::default()
        };

        for head_type in [
            HeadType::None,
            HeadType::Outline,
            HeadType::Number,
            HeadType::Bullet,
        ] {
            let style = listed(head_type);
            // The edit route's box, and what the render sites used to build.
            let edit = ParagraphBox::body_for_style(42_520.0, Some(&style), dpi);
            let render_before = ParagraphBox::body(42_520.0, 2_000.0, 1_000.0, dpi);

            // Width never diverged and must not start.
            assert_eq!(
                edit.width_hwp(),
                render_before.width_hwp(),
                "{head_type:?}: the width lane is not withheld"
            );

            match head_type {
                // No marker hangs left, so the origin is publishable and both
                // constructors already agreed.
                HeadType::None | HeadType::Outline => {
                    assert_eq!(edit.effective(), render_before.effective());
                    assert_eq!(edit.effective().start, 2_000);
                }
                // A marker hangs left, so `body_for_style` withholds the origin.
                // A route calling bare `body` would publish 2_000 against the
                // edit route's 0 — this is the divergence, named.
                HeadType::Number | HeadType::Bullet => {
                    assert_eq!(edit.effective().start, 0, "{head_type:?}");
                    assert_ne!(
                        edit.effective(),
                        render_before.effective(),
                        "{head_type:?}: bare `body` is not the same box"
                    );
                }
            }
        }
    }

    /// **The column solver quantizes before paragraph margins.**
    ///
    /// This is the scoping claim — the load-bearing one, and the one nothing
    /// checked. Every other pitch test here exercises the snap's *arithmetic*;
    /// this one exercises where the snap is allowed to reach, which is what
    /// fails if the pitch is ever applied to the wrong coordinate system.
    ///
    /// The contrast is one misaligned full column width with a one-unit left
    /// paragraph margin. Native first truncates the full width `1002 → 1000`,
    /// then applies the margin, producing `1..1000`. It does not independently
    /// snap those post-margin edges to `4..1000`. A content box is already
    /// resolved by its owner and is unchanged.
    fn the_column_solver_quantizes_before_paragraph_margins() {
        let misaligned = 1..1_002;
        let column = ParagraphBox::column(misaligned.clone());
        let content = ParagraphBox::content(misaligned.clone());

        // `column()` and `content()` both accept an already-resolved range.
        assert_eq!(column.effective(), misaligned);
        assert_eq!(
            content.effective(),
            1..1_002,
            "a content box is published exactly as the caller stated it"
        );

        // The frame boundary. Both lanes of a published record come from the
        // box, so the carve must agree with `effective()` or the record and the
        // frame have drifted again.
        let carved = |paragraph_box: &ParagraphBox| paragraph_box.frame(0).carve(100).to_vec();
        assert_eq!(carved(&column), vec![misaligned.clone()]);
        assert_eq!(carved(&content), vec![misaligned.clone()]);

        // The production body constructor owns the column-solver quantization.
        let body = ParagraphBox::body(1_002.0, 1.0, 0.0, crate::renderer::HWPUNIT_PER_INCH);
        assert_eq!(body.effective(), 1..1_000);
        assert_eq!(carved(&body), vec![1..1_000]);
        let cell = ParagraphBox::content_width_px(1_002.0, crate::renderer::HWPUNIT_PER_INCH);
        assert_eq!(
            cell.effective(),
            0..1_002,
            "content_width_px() is not, so a resolved cell width stays put"
        );

        // Withholding the origin still changes only the origin.
        let width_only = ParagraphBox::column(misaligned).with_derivable_origin(false);
        assert_eq!(width_only.effective(), 0..1_001);
    }

    /// The snap must be total over its whole domain, including the two points
    /// its previous form was not defined at: **pitch zero**, the value §2.5's
    /// gated arm produces and the one `rem_euclid` panicked on, and an edge at
    /// `i32::MAX`, where the ceil added without checking.
    ///
    /// Neither is reachable through today's call sites — the column quantum is
    /// positive and `px_to_hwpunit` saturates well below `i32::MAX`. They are
    /// pinned because pitch zero is the shape a future paragraph-grid gate
    /// takes, and
    /// because the `i32::MAX` case was a debug-build panic waiting on anyone
    /// widening the input range.
    fn the_snap_is_total_over_its_own_domain() {
        let up = |edge| snap_base_left(edge, COLUMN_WIDTH_QUANTUM_HWP);
        let down = |edge| snap_base_right(edge, COLUMN_WIDTH_QUANTUM_HWP);

        // Ceil and floor, on and off the grid, including negatives.
        assert_eq!((up(0), down(0)), (0, 0));
        assert_eq!((up(1), down(1)), (4, 0));
        assert_eq!((up(3), down(3)), (4, 0));
        assert_eq!((up(4), down(4)), (4, 4));
        assert_eq!((up(-1), down(-1)), (0, -4));

        // Pitch zero is the identity, not a division by zero.
        for edge in [-7, -1, 0, 1, 7, i32::MAX, i32::MIN] {
            assert_eq!(snap_base_left(edge, 0), edge, "pitch 0 ceil {edge}");
            assert_eq!(snap_base_right(edge, 0), edge, "pitch 0 floor {edge}");
        }

        // The ceil saturates rather than overflowing. `i32::MAX % 4 == 3`, so
        // this is the one input that used to wrap.
        assert_eq!(up(i32::MAX), i32::MAX);
        assert_eq!(down(i32::MIN), i32::MIN);

        // Every snapped edge lands on the grid, and the snap only ever moves
        // inward and by less than one pitch — the properties the carve depends
        // on. This fails if the pitch value changes without the carve's
        // consumers being revisited.
        for edge in -64..64 {
            let (left, right) = (up(edge), down(edge));
            assert_eq!(left.rem_euclid(COLUMN_WIDTH_QUANTUM_HWP), 0, "left {edge}");
            assert_eq!(
                right.rem_euclid(COLUMN_WIDTH_QUANTUM_HWP),
                0,
                "right {edge}"
            );
            assert!(left >= edge && right <= edge, "inward {edge}");
            assert!(left - edge < COLUMN_WIDTH_QUANTUM_HWP, "left bound {edge}");
            assert!(
                edge - right < COLUMN_WIDTH_QUANTUM_HWP,
                "right bound {edge}"
            );
        }
    }

    /// A carve that produces no occupiable interval must be refused on **both**
    /// arms, by the same rule.
    ///
    /// Two shapes reach `current_intervals` and survive the minimum-width
    /// prune, which never deletes the last interval: a zero-width base
    /// (`0..0`), and a base the geometry pitch inverted — `1..3` snaps to
    /// `4..0`, which is `snap_base_left(1)..snap_base_right(3)`.
    ///
    /// The admitted arm is the one this fixes. It asked nothing, and the
    /// stored-row predicate is satisfiable by a degenerate row: `0..0` matches
    /// a stored `cs=0 sw=0` exactly, and `4..0` matches `cs=4 sw=-4`. The
    /// retired `retain_preserved_single_segment_rows` refused those with a
    /// `segment_width > 0` precondition that was dropped when the comparison
    /// replaced it. Committing one publishes `segment_width` from
    /// `end.saturating_sub(start)`, which does not clamp at zero.
    fn a_carve_with_no_occupiable_interval_commits_on_neither_arm() {
        let degenerate = [
            (0..0, 0, 0, "zero-width base"),
            // A future grid pitch can invert a narrow positive-width base.
            (
                snap_base_left(1, COLUMN_WIDTH_QUANTUM_HWP)
                    ..snap_base_right(3, COLUMN_WIDTH_QUANTUM_HWP),
                4,
                -4,
                "pitch-inverted base",
            ),
        ];

        for (horizontal, column_start, segment_width, what) in degenerate {
            assert!(
                horizontal.start >= horizontal.end,
                "{what}: fixture must be degenerate"
            );

            // Direct commit: the frame refuses the row outright.
            let mut direct = frame(horizontal.clone(), 0, Vec::new());
            let carved = direct.carve(100).to_vec();
            assert_eq!(
                carved,
                vec![horizontal.clone()],
                "{what}: carve is unpruned"
            );
            assert!(!direct.carved_row_is_usable(), "{what}");
            assert_eq!(
                direct.commit_carved_row(
                    metrics(100, 10),
                    vec![RowSegment::new(
                        0..0,
                        horizontal.clone(),
                        LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    )],
                ),
                None,
                "{what}: a degenerate row must not commit"
            );
            assert_eq!(direct.row_count(), 0, "{what}");

            // Admitted arm: a stored row that matches the degenerate carve
            // exactly is still refused, and the frame is left untouched.
            let original = frame(horizontal.clone(), 0, Vec::new());
            let mut admitting = original.clone();
            let stored = [LineSeg {
                line_height: 100,
                column_start,
                segment_width,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }];
            assert!(
                stored_row_matches_frame_expectation(&horizontal, &stored[0]),
                "{what}: precondition — the predicate alone would admit this"
            );
            assert!(
                !admitting.try_admit_stored_rows(&stored, echo_metrics),
                "{what}: admission must refuse it anyway"
            );
            assert_eq!(admitting, original, "{what}: and roll back exactly");
        }
    }

    #[test]
    fn taller_candidate_recarves_before_the_row_is_committed() {
        every_body_route_builds_one_box_for_one_paragraph();
        the_column_solver_quantizes_before_paragraph_margins();
        the_snap_is_total_over_its_own_domain();
        a_carve_with_no_occupiable_interval_commits_on_neither_arm();
        a_row_split_from_first_to_last_is_compared_slot_by_slot();
        a_vertical_rewind_does_not_block_admission();
        any_horizontal_difference_falls_through_to_the_strict_reflow();
        a_later_rejected_row_rolls_back_partial_admission_on_a_shared_frame();
        let mut frame = frame(
            0..100,
            0,
            vec![FrameExclusion {
                horizontal: 0..60,
                vertical: 1_000..3_000,
                policy: FrameExclusionPolicy::BothSides,
            }],
        );

        let carved = frame.carve(900);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 0..100);
        assert_eq!(frame.top, 0);

        let carved = frame.carve(2_000);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 60..100);
        assert_eq!(frame.next_geometry_event, Some(3_000));

        frame.top = 3_000;
        let carved = frame.carve(2_000);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 0..100);
        assert_eq!(frame.top, 3_000);
    }

    #[test]
    fn committed_row_projects_one_complete_lineseg_group_and_advances_once() {
        let mut frame = frame(10..110, 500, Vec::new());
        let carved = frame.carve(900);
        assert_eq!(carved.len(), 1);
        assert_eq!(carved[0], 10..110);

        assert_eq!(
            frame.commit_carved_row(
                metrics(900, 100),
                vec![RowSegment::new(
                    7..31,
                    10..110,
                    LineSeg::TAG_IMPLEMENTATION_PROPERTY | LineSeg::TAG_FIRST_SEGMENT,
                )],
            ),
            Some(0)
        );
        assert_eq!(frame.top, 1_500);
        assert_eq!(frame.row_count(), 1);
        assert_eq!(frame.commit_carved_row(metrics(900, 100), Vec::new()), None);
        assert_eq!(frame.top, 1_500);

        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].text_start, 7);
        assert_eq!(projected[0].vertical_pos, 500);
        assert_eq!(projected[0].column_start, 10);
        assert_eq!(projected[0].segment_width, 100);
        assert!(projected[0].is_first_segment());
        assert!(projected[0].is_last_segment());
        assert_ne!(projected[0].tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY, 0);
    }

    #[test]
    fn stored_rows_equal_to_the_frames_expectation_are_admitted() {
        let stored = [
            LineSeg {
                text_start: 0,
                vertical_pos: 100,
                line_height: 300,
                text_height: 280,
                baseline_distance: 250,
                line_spacing: 20,
                column_start: 10,
                segment_width: 100,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            },
            LineSeg {
                text_start: 12,
                vertical_pos: 420,
                line_height: 300,
                text_height: 280,
                baseline_distance: 250,
                line_spacing: 20,
                column_start: 10,
                segment_width: 100,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_IMPLEMENTATION_PROPERTY,
            },
        ];
        let mut frame = frame(10..110, 100, Vec::new());

        assert!(frame.try_admit_stored_rows(&stored, echo_metrics));
        assert_eq!(frame.row_count(), 2);
        assert_eq!(frame.top, 740);
        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[1].text_start, 12);
        assert_eq!(projected[1].vertical_pos, 420);
        // The committed geometry is the carve's, which equals the stored
        // extent — that equality is the admission test (#4755 §1).
        assert!(projected
            .iter()
            .all(|segment| segment.column_start == 10 && segment.segment_width == 100));
    }

    /// Pins a branch that **no production caller can reach today**, and that is
    /// deliberate rather than an oversight: `ParagraphBox::frame` builds every
    /// body frame with an empty exclusion list, so `models_exclusions()` is
    /// false at the only call site and `resolve_stored_line_segs_in_frame`
    /// turns split rows away before this comparison sees them. The multi-slot
    /// path becomes live the moment a float set reaches a body frame; until
    /// then this test is the only thing holding it correct.
    fn a_row_split_from_first_to_last_is_compared_slot_by_slot() {
        // §1.4.1 compares interval COUNT and then horzpos/horzsize per slot.
        // A FIRST..LAST row is one physical row spanning several slots, and it
        // is preserved when the carve reproduces every one of them — refusing
        // to compare it was our departure, not the native rule.
        let stored = [
            LineSeg {
                vertical_pos: 200,
                line_height: 400,
                segment_width: 35,
                tag: LineSeg::TAG_FIRST_SEGMENT,
                ..Default::default()
            },
            LineSeg {
                text_start: 4,
                vertical_pos: 200,
                line_height: 400,
                column_start: 65,
                segment_width: 35,
                tag: LineSeg::TAG_LAST_SEGMENT,
                ..Default::default()
            },
        ];
        let original = frame(
            0..100,
            200,
            vec![FrameExclusion {
                horizontal: 35..65,
                vertical: 0..1_000,
                policy: FrameExclusionPolicy::BothSides,
            }],
        );

        let mut frame = original.clone();
        assert!(frame.try_admit_stored_rows(&stored, echo_metrics));
        assert_eq!(frame.row_count(), 1, "two slots are one physical row");
        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 2);
        assert_eq!(
            (projected[0].column_start, projected[0].segment_width),
            (0, 35)
        );
        assert_eq!(
            (projected[1].column_start, projected[1].segment_width),
            (65, 35)
        );
    }

    fn a_vertical_rewind_does_not_block_admission() {
        // HWP restarts `vertical_pos` when a paragraph continues on the next
        // page. §1.3/§1.4.1: the native validator never compares that field —
        // it recomputes it every pass and writes it back — so it must not
        // take part in the decision. Text order still must not rewind.
        let original = frame(0..100, 200, Vec::new());
        let row = |text_start, vertical_pos| LineSeg {
            text_start,
            vertical_pos,
            line_height: 400,
            segment_width: 100,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        };

        let mut rewound = original.clone();
        assert!(rewound.try_admit_stored_rows(&[row(0, 200), row(4, 0)], echo_metrics));

        let mut backwards_text = original.clone();
        assert!(!backwards_text.try_admit_stored_rows(&[row(4, 200), row(0, 600)], echo_metrics));
        assert_eq!(backwards_text, original);
    }

    fn any_horizontal_difference_falls_through_to_the_strict_reflow() {
        // The predicate is exact equality today and is an open question
        // (`stored_row_matches_frame_expectation`). Both directions are
        // rejections: a stored row wider than the frame's expectation and one
        // narrower than it are equally "not the row the frame expected".
        let original = frame(0..100, 200, Vec::new());
        let row = |segment_width| {
            [LineSeg {
                vertical_pos: 200,
                line_height: 400,
                segment_width,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }]
        };

        let mut wider = original.clone();
        assert!(!wider.try_admit_stored_rows(&row(120), echo_metrics));
        assert_eq!(wider, original);

        let mut narrower = original.clone();
        assert!(!narrower.try_admit_stored_rows(&row(98), echo_metrics));
        assert_eq!(narrower, original);

        let mut exact = original.clone();
        assert!(exact.try_admit_stored_rows(&row(100), echo_metrics));
    }

    fn a_later_rejected_row_rolls_back_partial_admission_on_a_shared_frame() {
        let mut shared = frame(0..100, 50, Vec::new());
        let full_width = 0..100;
        assert_eq!(shared.carve(100), std::slice::from_ref(&full_width));
        assert_eq!(
            shared.commit_carved_row(
                metrics(100, 10),
                vec![RowSegment::new(
                    0..7,
                    0..100,
                    LineSeg::TAG_SINGLE_SEGMENT_LINE,
                )],
            ),
            Some(0)
        );
        let entry_checkpoint = shared.clone();
        let stored = [
            LineSeg {
                text_start: 7,
                vertical_pos: 160,
                line_height: 100,
                text_height: 90,
                baseline_distance: 80,
                line_spacing: 10,
                column_start: 0,
                segment_width: 100,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            },
            LineSeg {
                text_start: 20,
                vertical_pos: 270,
                line_height: 100,
                text_height: 90,
                baseline_distance: 80,
                line_spacing: 10,
                column_start: 0,
                segment_width: 130,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            },
        ];

        let mut first_row_probe = entry_checkpoint.clone();
        assert!(first_row_probe.try_admit_stored_rows(&stored[..1], echo_metrics));
        assert_eq!(
            first_row_probe.row_count(),
            entry_checkpoint.row_count() + 1
        );
        assert_eq!(first_row_probe.top, 270);

        assert!(!shared.try_admit_stored_rows(&stored, echo_metrics));
        assert_eq!(shared, entry_checkpoint);
    }

    #[test]
    fn one_physical_row_projects_each_carved_interval_with_shared_metrics() {
        let mut frame = frame(
            0..100,
            200,
            vec![FrameExclusion {
                horizontal: 35..65,
                vertical: 0..1_000,
                policy: FrameExclusionPolicy::BothSides,
            }],
        );
        assert_eq!(frame.carve(400), &[0..35, 65..100]);

        assert_eq!(
            frame.commit_carved_row(
                metrics(400, 50),
                vec![
                    RowSegment::new(0..4, 0..35, LineSeg::TAG_FIRST_SEGMENT),
                    RowSegment::new(
                        4..9,
                        65..100,
                        LineSeg::TAG_LAST_SEGMENT | LineSeg::TAG_IMPLEMENTATION_PROPERTY,
                    ),
                ],
            ),
            Some(0)
        );
        assert_eq!(frame.top, 650);

        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 2);
        assert_eq!(projected[0].vertical_pos, 200);
        assert_eq!(projected[1].vertical_pos, 200);
        assert_eq!(projected[0].line_height, projected[1].line_height);
        assert!(projected[0].is_first_segment());
        assert!(!projected[0].is_last_segment());
        assert!(!projected[1].is_first_segment());
        assert!(projected[1].is_last_segment());
        assert_eq!(projected[0].column_start, 0);
        assert_eq!(projected[0].segment_width, 35);
        assert_eq!(projected[1].column_start, 65);
        assert_eq!(projected[1].segment_width, 35);
    }

    #[test]
    fn both_sides_four_physical_rows_project_eight_segments() {
        // `pic2` has two floating Pictures and is intentionally rejected by
        // the one-Picture transaction. This independent frame proof keeps the
        // core's ordinary two-interval physical-row behavior explicit.
        let mut frame = frame(
            0..100,
            0,
            vec![FrameExclusion {
                horizontal: 35..65,
                vertical: 0..1_000,
                policy: FrameExclusionPolicy::BothSides,
            }],
        );

        for row in 0..4 {
            assert_eq!(frame.carve(100), &[0..35, 65..100]);
            assert_eq!(
                frame.commit_carved_row(
                    metrics(100, 0),
                    vec![
                        RowSegment::new(
                            (row * 2) as u32..(row * 2 + 1) as u32,
                            0..35,
                            LineSeg::TAG_FIRST_SEGMENT,
                        ),
                        RowSegment::new(
                            (row * 2 + 1) as u32..(row * 2 + 2) as u32,
                            65..100,
                            LineSeg::TAG_LAST_SEGMENT,
                        ),
                    ],
                ),
                Some(row)
            );
        }

        let projected = frame.project_line_segs();
        assert_eq!(projected.len(), 8);
        for (row, pair) in projected.chunks_exact(2).enumerate() {
            assert_eq!(pair[0].vertical_pos, (row * 100) as i32);
            assert_eq!(pair[1].vertical_pos, pair[0].vertical_pos);
            assert_eq!(
                (pair[0].column_start, pair[0].segment_width),
                (0, 35),
                "row {row} left interval"
            );
            assert_eq!(
                (pair[1].column_start, pair[1].segment_width),
                (65, 35),
                "row {row} right interval"
            );
            assert!(pair[0].is_first_segment());
            assert!(!pair[0].is_last_segment());
            assert!(!pair[1].is_first_segment());
            assert!(pair[1].is_last_segment());
        }
    }

    #[test]
    fn largest_side_carve_chooses_the_wider_right_lane() {
        let mut frame = frame(
            0..100,
            0,
            vec![FrameExclusion {
                horizontal: 20..60,
                vertical: 0..1_000,
                policy: FrameExclusionPolicy::LargestSide,
            }],
        );

        let expected = 60..100;
        assert_eq!(frame.carve(100), std::slice::from_ref(&expected));
    }

    #[test]
    fn largest_side_carve_breaks_a_tie_to_the_left_lane() {
        let mut frame = frame(
            0..100,
            0,
            vec![FrameExclusion {
                horizontal: 30..70,
                vertical: 0..1_000,
                policy: FrameExclusionPolicy::LargestSide,
            }],
        );

        let expected = 0..30;
        assert_eq!(frame.carve(100), std::slice::from_ref(&expected));
    }

    #[test]
    fn largest_side_carve_moves_a_1440_hwp_left_lane_to_the_right() {
        let mut frame = frame(
            0..10_000,
            0,
            vec![FrameExclusion {
                horizontal: 1_440..9_000,
                vertical: 0..1_000,
                policy: FrameExclusionPolicy::LargestSide,
            }],
        );

        let expected = 9_000..10_000;
        assert_eq!(frame.carve(100), std::slice::from_ref(&expected));
    }

    #[test]
    fn live_frame_retries_when_the_1440_exception_selects_an_unusable_right_lane() {
        let mut frame = LayoutFrame::new(
            0..10_000,
            0,
            vec![FrameExclusion {
                horizontal: 1_440..9_000,
                vertical: 0..1_000,
                policy: FrameExclusionPolicy::LargestSide,
            }],
        );

        let expected = 0..10_000;
        assert_eq!(frame.carve(100), std::slice::from_ref(&expected));
        assert_eq!(frame.top, 1_000);
        assert_eq!(frame.next_geometry_event, None);
    }
}
