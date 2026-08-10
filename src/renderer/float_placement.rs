//! Flow reservation helpers for non-inline floating objects.

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo};
use crate::model::table::{Table, TablePageBreak};
use crate::model::HwpUnit;

use super::hwpunit_to_px;
use super::page_layout::LayoutRect;

/// Interpret an HWPUNIT value that may have been stored through a signed field.
pub(crate) fn signed_hwpunit(value: HwpUnit) -> i32 {
    value as i32
}

/// A non-TAC `TopAndBottom` object positioned from its host paragraph.
pub(crate) fn is_para_topbottom_float(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && matches!(common.text_wrap, TextWrap::TopAndBottom)
        && matches!(common.vert_rel_to, VertRelTo::Para)
}

/// Stored host-line evidence for the narrow native-HWP RowBreak flow contract (#2439).
///
/// The returned value is the non-synthetic stored line advance in HWPUNIT.  Callers may combine
/// it with the positive object offset for pagination, or with the painted lane bottom and outer
/// bottom for layout.  Keeping the structural predicate here prevents typeset/full/partial layout
/// from drifting apart.  A broad empty-host outer-margin rule is disproven by #2097.
pub(crate) fn native_empty_host_rowbreak_line_advance_hu(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
) -> Option<i32> {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    if !native_hwp5_layout
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || has_non_whitespace_text(para)
        || !matches!(table.common.vert_rel_to, VertRelTo::Para)
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || signed_hwpunit(table.common.vertical_offset) <= 0
        || para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
        || para
            .controls
            .iter()
            .filter(|control| {
                matches!(control, Control::Table(candidate)
                    if is_para_topbottom_float(&candidate.common))
            })
            .count()
            != 1
        || !next_para.is_some_and(|next| has_non_whitespace_text(next) && next.controls.is_empty())
    {
        return None;
    }

    let host_seg = para
        .line_segs
        .iter()
        .find(|seg| seg.tag & 0x80000000 == 0 && seg.line_height > 0)?;
    let advance = host_seg.line_height + host_seg.line_spacing.max(0);
    if advance <= 0 {
        return None;
    }
    // [#2808] 저장 vpos ladder 로 한컴이 host 줄 advance 를 실제 흐름에 계상했는지
    // 검증한다. #2439 재현 문서(기계 반복 양식)는 ladder 가 표 높이를 접고
    // `next.vpos - host.vpos == advance` 로 저장되는 반면(= advance 가 실 흐름 증거),
    // 일반 물리 ladder 문서는 델타가 표 높이+offset 을 이미 포함하므로 advance 를
    // 다시 더하면 이중 계상되어 쪽 경계 한 줄이 +1 로 밀린다 (10k r19 회귀 4건).
    let next_vpos = next_para
        .and_then(|next| {
            next.line_segs
                .iter()
                .find(|seg| seg.tag & 0x80000000 == 0 && seg.line_height > 0)
        })
        .map(|seg| seg.vertical_pos)?;
    if (next_vpos - host_seg.vertical_pos - advance).abs() > 1 {
        return None;
    }
    Some(advance)
}

/// Native HWP5가 빈 host의 저장 LINE_SEG 사다리에 표의 outer box 전체를 기록한
/// 경우만 paint origin에 outer-left/top을 복원한다.
///
/// 모든 empty-host 표에 outer margin을 더하는 규칙은 #2097 실물과 충돌한다. 이
/// helper는 표 높이와 위·아래 outer margin의 합이 다음 실제 저장 vpos와 정확히
/// 일치하는 단일 whole-table 형상만 식별한다. Pagination/flow는 이미 이 outer box를
/// 예약하므로 caller는 paint subtree만 이동해야 한다.
pub(crate) fn native_empty_host_physical_outer_box_paint_inset(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
) -> bool {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    let declared_height = signed_hwpunit(table.common.height);
    if !native_hwp5_layout
        || has_non_whitespace_text(para)
        || para.controls.len() != 1
        || !matches!(para.controls.first(), Some(Control::Table(_)))
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || !matches!(table.common.horz_rel_to, HorzRelTo::Column)
        || !matches!(table.common.horz_align, HorzAlign::Left)
        || signed_hwpunit(table.common.horizontal_offset) != 0
        || signed_hwpunit(table.common.vertical_offset) != 0
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || table.row_count <= 1
        || table.col_count != 1
        || table.cells.len() != usize::from(table.row_count)
        || !table.cells.iter().enumerate().all(|(row, cell)| {
            cell.row == row as u16
                && cell.col == 0
                && cell.row_span == 1
                && cell.col_span == 1
        })
        || signed_hwpunit(table.common.width) <= 0
        || declared_height <= 0
        || table.outer_margin_left <= 0
        || table.outer_margin_right <= 0
        || table.outer_margin_top <= 0
        || table.outer_margin_bottom <= 0
        // 저장 vpos 사다리는 세로 outer box만 직접 증명한다. p120처럼 네 방향
        // margin이 같은 경우에만 그 증거를 수평 paint inset까지 확장한다.
        || table.outer_margin_left != table.outer_margin_right
        || table.outer_margin_left != table.outer_margin_top
        || table.outer_margin_left != table.outer_margin_bottom
        || table.caption.is_some()
        || next_para.is_some_and(|next| has_non_whitespace_text(next) || !next.controls.is_empty())
    {
        return false;
    }

    fn stored_seg(paragraph: &Paragraph) -> Option<&crate::model::paragraph::LineSeg> {
        paragraph.line_segs.iter().find(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && seg.line_height > 0
        })
    }
    let Some(host_seg) = stored_seg(para) else {
        return false;
    };
    let Some(next_seg) = next_para.and_then(stored_seg) else {
        return false;
    };
    let stored_advance = i64::from(next_seg.vertical_pos) - i64::from(host_seg.vertical_pos);
    let physical_outer_height = i64::from(declared_height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    stored_advance > 0 && (stored_advance - physical_outer_height).abs() <= 1
}

/// Paint-only geometry for the narrow native-HWP5 stored-reset table fragment contract.
///
/// These 1x1 RowBreak tables store the first physical fragment height in
/// `CommonObjAttr::height`, then restart the cell LINE_SEG ladder at `vpos=0` in the next
/// paragraph.  The paginator deliberately keeps its composed trailing line spacing for flow
/// ownership, but the painted first-fragment frame must stop at the stored height.  Both physical
/// fragments also paint inside the equal four-way outer margin already reserved by flow.
///
/// Callers must use this result only to change the paint subtree.  It is not a pagination or flow
/// height contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NativeStoredResetFragmentPaintGeometry {
    pub(crate) outer_left_hu: i32,
    pub(crate) outer_top_hu: i32,
    /// `Some` only for the first fragment.  A successor receives the same origin inset but keeps
    /// its measured fragment height.
    pub(crate) first_fragment_height_hu: Option<i32>,
}

/// Recognize one physical fragment of a native-HWP5 1x1 stored-reset RowBreak table.
///
/// The predicate intentionally contains no paragraph, table, or fixture identifier.  In
/// particular, the declared head height must be independently proven by the last stored line
/// before the cross-paragraph rewind plus the effective vertical cell padding.  The fragment cut
/// must then meet that exact rewind boundary.
pub(crate) fn native_hwp5_stored_reset_fragment_paint_geometry(
    native_hwp5_layout: bool,
    host_para: &Paragraph,
    table: &Table,
    is_continuation: bool,
    start_cut: &[usize],
    end_cut: &[usize],
) -> Option<NativeStoredResetFragmentPaintGeometry> {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    let cell = table.cells.first()?;
    let declared_height_hu = signed_hwpunit(table.common.height);
    if !native_hwp5_layout
        || has_non_whitespace_text(host_para)
        || host_para.controls.len() != 1
        || !matches!(host_para.controls.first(), Some(Control::Table(_)))
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.common.vert_align, VertAlign::Top)
        || !matches!(table.common.horz_rel_to, HorzRelTo::Column)
        || !matches!(table.common.horz_align, HorzAlign::Left)
        || signed_hwpunit(table.common.horizontal_offset) != 0
        || signed_hwpunit(table.common.vertical_offset) != 0
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || table.row_count != 1
        || table.col_count != 1
        || table.cells.len() != 1
        || cell.row != 0
        || cell.col != 0
        || cell.row_span != 1
        || cell.col_span != 1
        || signed_hwpunit(table.common.width) <= 0
        || declared_height_hu <= 0
        || table.caption.is_some()
        || table.outer_margin_left <= 0
        || table.outer_margin_right <= 0
        || table.outer_margin_top <= 0
        || table.outer_margin_bottom <= 0
        || table.outer_margin_left != table.outer_margin_right
        || table.outer_margin_left != table.outer_margin_top
        || table.outer_margin_left != table.outer_margin_bottom
    {
        return None;
    }

    let effective_padding = cell.effective_padding(&table.padding);
    if effective_padding.top < 0 || effective_padding.bottom < 0 {
        return None;
    }

    // Count only real stored text lines.  A composed atom/spacer makes the count diverge from the
    // RowCut and is therefore rejected by the exact fragment-boundary check below.
    let mut previous: Option<(usize, &crate::model::paragraph::LineSeg)> = None;
    let mut stored_lines_before = 0usize;
    let mut reset_witness = None;
    'paragraphs: for (para_index, paragraph) in cell.paragraphs.iter().enumerate() {
        for seg in paragraph.line_segs.iter().filter(|seg| {
            seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && seg.text_height > 0
        }) {
            if let Some((previous_para_index, previous_seg)) = previous {
                if previous_para_index != para_index
                    && previous_seg.vertical_pos > 0
                    && seg.vertical_pos == 0
                {
                    reset_witness = Some((stored_lines_before, previous_seg));
                    break 'paragraphs;
                }
            }
            stored_lines_before += 1;
            previous = Some((para_index, seg));
        }
    }
    let (reset_unit_end, previous_seg) = reset_witness?;
    let stored_head_height_hu = i64::from(previous_seg.vertical_pos)
        + i64::from(previous_seg.text_height)
        + i64::from(effective_padding.top)
        + i64::from(effective_padding.bottom);
    if (stored_head_height_hu - i64::from(declared_height_hu)).abs() > 1 {
        return None;
    }

    let is_first_fragment = !is_continuation
        && start_cut.is_empty()
        && end_cut.len() == 1
        && end_cut[0] == reset_unit_end;
    let is_final_successor = is_continuation
        && start_cut.len() == 1
        && start_cut[0] == reset_unit_end
        && end_cut.is_empty();
    if !is_first_fragment && !is_final_successor {
        return None;
    }

    Some(NativeStoredResetFragmentPaintGeometry {
        outer_left_hu: i32::from(table.outer_margin_left),
        outer_top_hu: i32::from(table.outer_margin_top),
        first_fragment_height_hu: is_first_fragment.then_some(declared_height_hu),
    })
}

/// [Task #1658 v3] 페이지 하단 고정(vert=쪽·valign=Bottom) 자리차지 개체 (결재/서명 틀).
/// 한글은 이를 본문 하단에 절대배치(겹침 허용)하고 본문 텍스트를 그 위까지만 흐르게
/// 한다(하단 배타 영역) — 문서순 flow 소비 대상이 아니다. #1653 RCA 패턴 B.
pub(crate) fn is_page_bottom_fixed_float(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && matches!(common.text_wrap, TextWrap::TopAndBottom)
        && matches!(common.vert_rel_to, VertRelTo::Page)
        && matches!(common.vert_align, VertAlign::Bottom)
}

/// Horizontal reference data used by float placement and table layout.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FloatPlacementContext {
    pub col_area: LayoutRect,
    pub body_area: Option<LayoutRect>,
    pub paper_width: Option<f64>,
    pub host_margin_left: f64,
    pub host_margin_right: f64,
}

impl FloatPlacementContext {
    pub(crate) fn new(col_area: LayoutRect) -> Self {
        Self {
            col_area,
            body_area: None,
            paper_width: None,
            host_margin_left: 0.0,
            host_margin_right: 0.0,
        }
    }

    pub(crate) fn with_body_area(mut self, body_area: LayoutRect) -> Self {
        self.body_area = Some(body_area);
        self
    }

    pub(crate) fn with_paper_width(mut self, paper_width: f64) -> Self {
        self.paper_width = Some(paper_width);
        self
    }

    pub(crate) fn with_host_margins(mut self, left: f64, right: f64) -> Self {
        self.host_margin_left = left;
        self.host_margin_right = right;
        self
    }
}

/// Compute the same depth-0 horizontal range used by table layout.
pub(crate) fn horizontal_range(
    common: &CommonObjAttr,
    width_px: f64,
    ctx: FloatPlacementContext,
    dpi: f64,
) -> (f64, f64) {
    let h_offset = hwpunit_to_px(signed_hwpunit(common.horizontal_offset), dpi);
    let col_area = ctx.col_area;
    let (ref_x, ref_w) = match common.horz_rel_to {
        HorzRelTo::Paper => {
            let fallback_paper_w = if width_px > col_area.width {
                col_area.x * 2.0 + width_px
            } else {
                col_area.x * 2.0 + col_area.width
            };
            let paper_w = ctx.paper_width.unwrap_or(fallback_paper_w);
            (0.0, paper_w)
        }
        HorzRelTo::Page => ctx
            .body_area
            .filter(|body| body.width > 0.0)
            .map(|body| (body.x, body.width))
            .unwrap_or((col_area.x, col_area.width)),
        HorzRelTo::Para => (
            col_area.x + ctx.host_margin_left,
            col_area.width - ctx.host_margin_left,
        ),
        HorzRelTo::Column => (col_area.x, col_area.width),
    };

    let x = match common.horz_align {
        HorzAlign::Left | HorzAlign::Inside => ref_x + h_offset,
        HorzAlign::Center => ref_x + (ref_w - width_px).max(0.0) / 2.0 + h_offset,
        HorzAlign::Right | HorzAlign::Outside => ref_x + (ref_w - width_px).max(0.0) - h_offset,
    };
    (x, x + width_px.max(0.0))
}

/// A placed float lane in page/column-relative coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FloatLane {
    pub x_start: f64,
    pub x_end: f64,
    pub bottom: f64,
}

impl FloatLane {
    fn overlaps_x(&self, x_start: f64, x_end: f64) -> bool {
        ranges_overlap(self.x_start, self.x_end, x_start, x_end)
    }
}

/// Tracks bottom reservations for horizontally independent float lanes.
#[derive(Debug, Default, Clone)]
pub(crate) struct FloatLaneSet {
    lanes: Vec<FloatLane>,
}

impl FloatLaneSet {
    pub(crate) fn new() -> Self {
        Self { lanes: Vec::new() }
    }

    pub(crate) fn clear(&mut self) {
        self.lanes.clear();
    }

    pub(crate) fn lanes(&self) -> &[FloatLane] {
        &self.lanes
    }

    pub(crate) fn pushed_top(&self, x_start: f64, x_end: f64, raw_top: f64) -> f64 {
        self.lanes
            .iter()
            .filter(|lane| lane.overlaps_x(x_start, x_end))
            .fold(raw_top, |top, lane| top.max(lane.bottom))
    }

    pub(crate) fn place(
        &mut self,
        x_start: f64,
        x_end: f64,
        raw_top: f64,
        height: f64,
    ) -> FloatLane {
        let top = self.pushed_top(x_start, x_end, raw_top);
        let lane = FloatLane {
            x_start,
            x_end,
            bottom: top + height.max(0.0),
        };
        self.lanes.push(lane);
        lane
    }

    pub(crate) fn max_bottom(&self) -> f64 {
        self.lanes
            .iter()
            .map(|lane| lane.bottom)
            .fold(0.0, f64::max)
    }
}

pub(crate) fn ranges_overlap(a_start: f64, a_end: f64, b_start: f64, b_end: f64) -> bool {
    let a0 = a_start.min(a_end);
    let a1 = a_start.max(a_end);
    let b0 = b_start.min(b_end);
    let b1 = b_start.max(b_end);
    a0 < b1 && b0 < a1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::LineSeg;
    use crate::model::shape::{HorzAlign, HorzRelTo, VertAlign};
    use crate::model::table::Cell;

    fn base_common() -> CommonObjAttr {
        CommonObjAttr {
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Column,
            horz_align: HorzAlign::Left,
            ..Default::default()
        }
    }

    #[test]
    fn signed_hwpunit_preserves_negative_offsets() {
        assert_eq!(signed_hwpunit((-43892i32) as u32), -43892);
        assert_eq!(signed_hwpunit(51100), 51100);
    }

    #[test]
    fn para_topbottom_float_predicate_requires_non_tac_para_topbottom() {
        let mut common = base_common();
        assert!(is_para_topbottom_float(&common));

        common.treat_as_char = true;
        assert!(!is_para_topbottom_float(&common));

        common.treat_as_char = false;
        common.text_wrap = TextWrap::Square;
        assert!(!is_para_topbottom_float(&common));

        common.text_wrap = TextWrap::TopAndBottom;
        common.vert_rel_to = VertRelTo::Page;
        assert!(!is_para_topbottom_float(&common));
    }

    #[test]
    fn lane_set_does_not_push_non_overlapping_ranges() {
        let mut lanes = FloatLaneSet::new();
        let first = lanes.place(0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(120.0, 200.0, 10.0, 20.0);

        assert_eq!(first.bottom, 50.0);
        assert_eq!(second.bottom, 30.0);
        assert_eq!(lanes.max_bottom(), 50.0);
    }

    #[test]
    fn lane_set_pushes_overlapping_ranges() {
        let mut lanes = FloatLaneSet::new();
        lanes.place(0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(90.0, 160.0, 10.0, 20.0);

        assert_eq!(second.bottom, 70.0);
        assert_eq!(lanes.max_bottom(), 70.0);
    }

    #[test]
    fn horizontal_range_matches_column_right_offset_rule() {
        let mut common = base_common();
        common.horz_align = HorzAlign::Right;
        common.horizontal_offset = 10;

        let ctx = FloatPlacementContext::new(LayoutRect {
            x: 20.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        });
        let (x0, x1) = horizontal_range(&common, 50.0, ctx, 7200.0);

        assert_eq!(x0, 160.0);
        assert_eq!(x1, 210.0);
    }

    #[test]
    fn horizontal_range_uses_body_area_for_page_relative_objects() {
        let mut common = base_common();
        common.horz_rel_to = HorzRelTo::Page;
        common.horz_align = HorzAlign::Center;

        let ctx = FloatPlacementContext::new(LayoutRect {
            x: 20.0,
            y: 0.0,
            width: 200.0,
            height: 100.0,
        })
        .with_body_area(LayoutRect {
            x: 40.0,
            y: 0.0,
            width: 300.0,
            height: 100.0,
        });
        let (x0, x1) = horizontal_range(&common, 100.0, ctx, 7200.0);

        assert_eq!(x0, 140.0);
        assert_eq!(x1, 240.0);
    }

    fn stored_reset_fragment_candidate() -> (Paragraph, Table) {
        let mut cell = Cell::new_empty(0, 0, 41_954, 2_282, 1);
        cell.paragraphs = vec![
            Paragraph {
                line_segs: vec![
                    LineSeg {
                        vertical_pos: 0,
                        line_height: 2_000,
                        text_height: 1_000,
                        line_spacing: 1_000,
                        ..Default::default()
                    },
                    LineSeg {
                        vertical_pos: 1_000,
                        line_height: 2_000,
                        text_height: 1_000,
                        line_spacing: 1_000,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            Paragraph {
                line_segs: vec![LineSeg {
                    vertical_pos: 0,
                    line_height: 2_000,
                    text_height: 1_000,
                    line_spacing: 1_000,
                    ..Default::default()
                }],
                ..Default::default()
            },
        ];
        let table = Table {
            row_count: 1,
            col_count: 1,
            page_break: TablePageBreak::RowBreak,
            padding: crate::model::Padding {
                top: 141,
                bottom: 141,
                ..Default::default()
            },
            common: CommonObjAttr {
                width: 41_954,
                height: 2_282,
                treat_as_char: false,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                vert_align: VertAlign::Top,
                horz_rel_to: HorzRelTo::Column,
                horz_align: HorzAlign::Left,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            cells: vec![cell],
            ..Default::default()
        };
        let host = Paragraph {
            controls: vec![Control::Table(Box::new(table.clone()))],
            ..Default::default()
        };
        (host, table)
    }

    #[test]
    fn stored_reset_fragment_geometry_separates_first_paint_height_from_successor_origin() {
        let (host, table) = stored_reset_fragment_candidate();

        assert_eq!(
            native_hwp5_stored_reset_fragment_paint_geometry(true, &host, &table, false, &[], &[2],),
            Some(NativeStoredResetFragmentPaintGeometry {
                outer_left_hu: 283,
                outer_top_hu: 283,
                first_fragment_height_hu: Some(2_282),
            })
        );
        assert_eq!(
            native_hwp5_stored_reset_fragment_paint_geometry(true, &host, &table, true, &[2], &[],),
            Some(NativeStoredResetFragmentPaintGeometry {
                outer_left_hu: 283,
                outer_top_hu: 283,
                first_fragment_height_hu: None,
            })
        );

        // A neighboring cut is not the stored reset boundary.
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &table,
            false,
            &[],
            &[1],
        )
        .is_none());
    }

    #[test]
    fn stored_reset_fragment_geometry_rejects_unproven_neighboring_shapes() {
        let (host, table) = stored_reset_fragment_candidate();

        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            false,
            &host,
            &table,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut visible_host = host.clone();
        visible_host.text = "표 제목".to_string();
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &visible_host,
            &table,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut wrong_declared_height = table.clone();
        wrong_declared_height.common.height += 1_000;
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &wrong_declared_height,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut asymmetric_margin = table.clone();
        asymmetric_margin.outer_margin_right += 1;
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &asymmetric_margin,
            false,
            &[],
            &[2],
        )
        .is_none());

        let mut same_paragraph_rewind = table.clone();
        let reset = same_paragraph_rewind.cells[0].paragraphs.remove(1);
        same_paragraph_rewind.cells[0].paragraphs[0]
            .line_segs
            .extend(reset.line_segs);
        assert!(native_hwp5_stored_reset_fragment_paint_geometry(
            true,
            &host,
            &same_paragraph_rewind,
            false,
            &[],
            &[2],
        )
        .is_none());
    }

    fn physical_outer_box_candidate() -> (Paragraph, Table, Paragraph) {
        let table = Table {
            row_count: 6,
            col_count: 1,
            page_break: TablePageBreak::RowBreak,
            common: CommonObjAttr {
                width: 41_954,
                height: 23_790,
                treat_as_char: false,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                vert_align: VertAlign::Top,
                horz_rel_to: HorzRelTo::Column,
                horz_align: HorzAlign::Left,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            cells: (0..6)
                .map(|row| Cell::new_empty(0, row, 41_954, 3_965, 1))
                .collect(),
            ..Default::default()
        };
        let host = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 0,
                line_height: 1,
                ..Default::default()
            }],
            controls: vec![Control::Table(Box::new(table.clone()))],
            ..Default::default()
        };
        let next = Paragraph {
            line_segs: vec![LineSeg {
                vertical_pos: 24_356,
                line_height: 1_000,
                ..Default::default()
            }],
            ..Default::default()
        };
        (host, table, next)
    }

    #[test]
    fn physical_outer_box_paint_inset_requires_exact_native_stored_ladder() {
        let (host, table, next) = physical_outer_box_candidate();
        assert!(native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&next),
        ));
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            false,
            &host,
            &table,
            Some(&next),
        ));

        let mut short = next.clone();
        short.line_segs[0].vertical_pos = 23_790;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&short),
        ));

        let mut mismatched = next.clone();
        mismatched.line_segs[0].vertical_pos += 2;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&mismatched),
        ));

        let mut synthetic_host = host.clone();
        synthetic_host.line_segs[0].tag = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &synthetic_host,
            &table,
            Some(&next),
        ));

        let mut synthetic_next = next.clone();
        synthetic_next.line_segs[0].tag = LineSeg::TAG_IMPLEMENTATION_PROPERTY;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&synthetic_next),
        ));
    }

    #[test]
    fn physical_outer_box_paint_inset_rejects_neighboring_float_contracts() {
        let (host, table, next) = physical_outer_box_candidate();

        let mut positive_offset = table.clone();
        positive_offset.common.vertical_offset = 350;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &positive_offset,
            Some(&next),
        ));

        let mut horizontal_offset = table.clone();
        horizontal_offset.common.horizontal_offset = 350;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &horizontal_offset,
            Some(&next),
        ));

        let mut visible_host = host.clone();
        visible_host.text = "표 제목".to_string();
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &visible_host,
            &table,
            Some(&next),
        ));

        let mut two_tables = host.clone();
        two_tables
            .controls
            .push(Control::Table(Box::new(table.clone())));
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &two_tables,
            &table,
            Some(&next),
        ));

        let mut next_object_host = next.clone();
        next_object_host
            .controls
            .push(Control::Table(Box::new(table.clone())));
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&next_object_host),
        ));

        let mut next_visible = next.clone();
        next_visible.text = "다음 본문".to_string();
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &table,
            Some(&next_visible),
        ));

        let mut tac = table.clone();
        tac.common.treat_as_char = true;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &tac,
            Some(&next),
        ));

        let mut square = table.clone();
        square.common.text_wrap = TextWrap::Square;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &square,
            Some(&next),
        ));

        let mut page_relative = table.clone();
        page_relative.common.vert_rel_to = VertRelTo::Page;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &page_relative,
            Some(&next),
        ));

        let mut right_aligned = table.clone();
        right_aligned.common.horz_align = HorzAlign::Right;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &right_aligned,
            Some(&next),
        ));

        let mut missing_margin = table.clone();
        missing_margin.outer_margin_left = 0;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &missing_margin,
            Some(&next),
        ));

        let mut asymmetric_margin = table.clone();
        asymmetric_margin.outer_margin_right += 1;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &asymmetric_margin,
            Some(&next),
        ));

        let mut one_by_one = table.clone();
        one_by_one.row_count = 1;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &one_by_one,
            Some(&next),
        ));

        let mut two_columns = table.clone();
        two_columns.col_count = 2;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &two_columns,
            Some(&next),
        ));

        let mut missing_cells = table.clone();
        missing_cells.cells.clear();
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &missing_cells,
            Some(&next),
        ));

        let mut duplicate_row = table.clone();
        duplicate_row.cells[1].row = 0;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &duplicate_row,
            Some(&next),
        ));

        let mut spanning_row = table.clone();
        spanning_row.cells[0].row_span = 2;
        assert!(!native_empty_host_physical_outer_box_paint_inset(
            true,
            &host,
            &spanning_row,
            Some(&next),
        ));
    }
}
