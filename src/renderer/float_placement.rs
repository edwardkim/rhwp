//! 부동 개체의 흐름 예약과 실제 배치가 함께 쓰는 프레임.

use std::ops::Range;

use crate::model::control::Control;
use crate::model::image::Picture;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, TextFlow, TextWrap, VertAlign, VertRelTo,
};
use crate::model::style::Alignment;
use crate::model::table::{Table, TablePageBreak};
use crate::model::HwpUnit;

use super::hwpunit_to_px;
use super::layout::picture_flow_frame_size_hu;
use super::layout_frame::{FrameExclusion, FrameExclusionPolicy, LayoutFrame};
use super::page_layout::LayoutRect;

/// Paragraph-relative floats whose signed vertical intervals overlap occupy
/// one band. Negative offsets do not remove the object's flow height: when
/// its owner moves to another fragment the paragraph origin moves with it.
pub(crate) fn parallel_cell_float_band_height(controls: &[Control], dpi: f64) -> Option<f64> {
    if controls.len() < 2 {
        return None;
    }
    let mut latest_start = f64::NEG_INFINITY;
    let mut earliest_end = f64::INFINITY;
    let mut bottom = 0.0f64;
    for control in controls {
        let common = match control {
            Control::Picture(picture) => &picture.common,
            Control::Shape(shape) => shape.common(),
            _ => return None,
        };
        if common.treat_as_char
            || !matches!(
                common.text_wrap,
                TextWrap::Square | TextWrap::Tight | TextWrap::Through
            )
            || common.vert_rel_to != VertRelTo::Para
        {
            return None;
        }
        let start = hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
        let height = hwpunit_to_px(signed_hwpunit(common.height), dpi)
            + hwpunit_to_px(i32::from(common.margin.top), dpi)
            + hwpunit_to_px(i32::from(common.margin.bottom), dpi);
        if height <= 0.5 {
            return None;
        }
        latest_start = latest_start.max(start);
        earliest_end = earliest_end.min(start + height);
        bottom = bottom.max(start.max(0.0) + height);
    }
    (latest_start + 0.5 < earliest_end).then_some(bottom)
}

/// A single whitespace host line and its parallel flow pictures share an
/// origin, but the line and the picture band have separate fragment owners.
/// Explicit newlines, visible text, inline objects and distinct bands keep
/// their own line/anchor contracts.
pub(crate) fn parallel_cell_picture_band_height(para: &Paragraph, dpi: f64) -> Option<f64> {
    if !para.text.trim().is_empty()
        || para.text.contains(['\r', '\n'])
        || para.line_segs.len() > 1
        || !para.controls.iter().all(|control| {
            matches!(control,
                Control::Picture(picture) if picture.common.flow_with_text
                    && picture.common.text_wrap == TextWrap::Square
                    && picture.common.vert_align == VertAlign::Top
            )
        })
    {
        return None;
    }
    parallel_cell_float_band_height(&para.controls, dpi)
}

/// A reflowed empty block-table host shares its line origin with its tables.
/// Column definitions configure that flow; they do not create an extra line.
/// A following empty paragraph remains a separate line, as do text, fields,
/// TAC objects and stored line boxes.
pub(crate) fn reflow_block_table_host_occupied_height(
    para: &Paragraph,
    line_height: f64,
    table_height: f64,
) -> Option<f64> {
    if !crate::renderer::para_has_no_stored_line_segs(para) || !para.text.is_empty() {
        return None;
    }
    let mut has_table = false;
    for control in &para.controls {
        match control {
            Control::Table(table)
                if !table.common.treat_as_char
                    && table.common.flow_with_text
                    && table.common.text_wrap == TextWrap::TopAndBottom
                    && table.common.vert_rel_to == VertRelTo::Para
                    && table.common.vert_align == VertAlign::Top
                    && signed_hwpunit(table.common.vertical_offset) == 0 =>
            {
                has_table = true;
            }
            Control::ColumnDef(_) => {}
            _ => return None,
        }
    }
    has_table.then_some(line_height.max(table_height))
}

/// On reflow, an adjustable multi-row table is constrained by its physical
/// row declarations and content, rather than by scaling every row to a cached
/// object bounding height. A single row can consume the object-height minimum
/// directly; protected/noAdjust and stored-line tables retain their frame.
pub(crate) fn reflow_table_uses_row_height_constraints(table: &Table) -> bool {
    !table.common.treat_as_char
        && !table.common.size_protect
        && table.raw_table_record_attr & 0x08 == 0
        && table.row_count > 1
        && !table.cells.is_empty()
        && table.cells.iter().all(|cell| {
            !cell.paragraphs.is_empty()
                && cell
                    .paragraphs
                    .iter()
                    .all(crate::renderer::para_has_no_stored_line_segs)
        })
}

/// 자리차지 개체가 흐름에 추가하는 문단 기준 앞 공간.
/// 음수 오프셋은 앞 공간을 만들지 않는다. 셀의 가운데 정렬도 이 점유 프레임을
/// 소비하므로 음수 저장값을 별도의 정렬 이동으로 다시 적용하지 않는다.
pub(crate) fn topbottom_flow_vertical_offset_hu(common: &CommonObjAttr) -> i32 {
    signed_hwpunit(common.vertical_offset).max(0)
}

/// 글자 없는 칸 문단을 기준으로 흐름을 미는 Square·Tight·Through 개체의 배치용 속성.
///
/// 그런 문단은 개체 하나가 내용의 전부라, 칸 측정(`cell_non_inline_control_flow_height`)은
/// 개체가 **문단 위에서 시작해 개체 높이만큼** 점유한다고 계상하고 칸 `valign` 도 그
/// 프레임으로 문단 위를 정한다. 음수 저장 오프셋은 그 프레임 위에 공간을 만들지 않으므로
/// 배치에서도 개체를 문단 위보다 끌어올리지 않는다 — `topbottom_flow_vertical_offset_hu`
/// 와 같은 계약이다. 한/글 PDF 근거:
/// - 2024, 36308670 2쪽: 가운데 정렬 칸 Square 그림 다섯(오프셋 −71·−663·0·0·−561HU)이
///   모두 칸 가운데에 0.2px 안에서 놓인다.
/// - 2020, pr7518 nested-split 3쪽: 공백 문단의 그림 셋(−992·−1078·0HU)이 같은 y.
/// - 2024, 36296324 2쪽 중첩 표: 같은 행의 −455HU 그림과 0HU 그림 위 차가 0.5px.
///
/// 글자가 있는 문단은 개체가 글줄을 밀어 낸 저장 흐름 안에서 오프셋을 쓴다(보도자료
/// 머리 표 로고 −80HU: 한/글 PDF 와 0.13px) — 그대로 둔다. 양수 오프셋·문단 기준이
/// 아닌 개체도 그대로다.
pub(crate) fn cell_square_flow_placement_common(
    para: &Paragraph,
    common: &CommonObjAttr,
) -> Option<CommonObjAttr> {
    (para.text.trim().is_empty()
        && !common.treat_as_char
        && common.flow_with_text
        && matches!(common.vert_rel_to, VertRelTo::Para)
        && matches!(
            common.text_wrap,
            TextWrap::Square | TextWrap::Tight | TextWrap::Through
        )
        && signed_hwpunit(common.vertical_offset) < 0)
        .then(|| {
            let mut placed = common.clone();
            placed.vertical_offset = 0;
            placed
        })
}

/// 원본 HWPX noAdjust 셀의 완전한 저장 줄 프레임이 점유하는 끝점.
/// 여백을 이미 계상하는 중첩/TAC의 relaxed-pad 경로에서 줄 사이 저장 공간을
/// 재사용한다. noAdjust 자체는 본문 표의 안 여백을 지울 근거가 아니다.
/// 빈 줄도 포함하며, 편집·재조판·개체·되감김은 이 계약의 입력으로 받지 않는다.
pub(crate) fn stored_hwpx_no_adjust_cell_content_end(
    cell: &crate::model::table::Cell,
    table: &Table,
    dpi: f64,
    hwpx_stored: bool,
    session_edited: bool,
    text_reflowed: bool,
    padding_in_stored_frame: bool,
) -> Option<f64> {
    if !padding_in_stored_frame
        || !hwpx_stored
        || session_edited
        || text_reflowed
        || table.raw_table_record_attr & 0x08 == 0
        || cell.text_direction != 0
        || cell.paragraphs.is_empty()
    {
        return None;
    }
    let mut previous = None;
    let mut end = 0i64;
    for (pi, para) in cell.paragraphs.iter().enumerate() {
        if !para.controls.is_empty() || para.line_segs.is_empty() {
            return None;
        }
        for (li, seg) in para.line_segs.iter().enumerate() {
            if seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || seg.vertical_pos < 0
                || seg.line_height <= 0
                || (previous.is_none() && seg.vertical_pos != 0)
                || (previous.is_some()
                    && (seg.is_first_line_of_page() || seg.is_first_line_of_column()))
                || previous.is_some_and(|pos| {
                    seg.vertical_pos < pos || (pi > 0 && li == 0 && seg.vertical_pos == pos)
                })
            {
                return None;
            }
            previous = Some(seg.vertical_pos);
            end = end.max(i64::from(seg.vertical_pos) + i64::from(seg.line_height));
        }
    }
    i32::try_from(end).ok().map(|hu| hwpunit_to_px(hu, dpi))
}

/// 그림 띠 뒤의 빈 마지막 줄을 담는 원본 셀 프레임.
/// 그림들은 같은 띠의 원점에서 움직이며 각 그림 높이로 원점을 따로 역산하지 않는다.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StoredEmptyPictureCellFrame {
    pub line_offset_hu: i32,
    pub picture_band_origin_hu: i32,
    pub content_height_hu: i32,
}

pub(crate) fn stored_empty_picture_cell_frame(
    cell: &crate::model::table::Cell,
    table: &Table,
) -> Option<StoredEmptyPictureCellFrame> {
    use crate::model::paragraph::LineSeg;
    use crate::model::table::VerticalAlign;
    if cell.row_span != 1
        || cell.text_direction != 0
        || cell.vertical_align != VerticalAlign::Bottom
    {
        return None;
    }
    let [para] = cell.paragraphs.as_slice() else {
        return None;
    };
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    if !para.text.trim().is_empty()
        || para.column_type != crate::model::paragraph::ColumnBreakType::None
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || para.controls.is_empty()
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos <= 0
        || line.line_height <= 0
    {
        return None;
    }
    let padding = cell.effective_padding(&table.padding);
    let tail_height = line
        .line_height
        .checked_add(i32::from(padding.top))?
        .checked_add(i32::from(padding.bottom))?;
    let row_height = table
        .cells
        .iter()
        .filter(|other| other.row == cell.row && other.row_span == 1)
        .map(|other| signed_hwpunit(other.height))
        .max()?;
    // 빈 셀의 선언 높이는 마지막 줄과 안 여백이며, 저장 줄 위치까지 더하면 행을 닫는다.
    if padding.top < 0
        || padding.bottom < 0
        || signed_hwpunit(cell.height) != tail_height
        || line.vertical_pos.checked_add(tail_height)? != row_height
    {
        return None;
    }
    let mut picture_band_end = 0;
    let mut has_flow_picture = false;
    for control in &para.controls {
        let Control::Picture(picture) = control else {
            return None;
        };
        let common = &picture.common;
        let offset = signed_hwpunit(common.vertical_offset);
        if common.treat_as_char
            || !common.flow_with_text
            || picture.caption.is_some()
            || common.vert_rel_to != VertRelTo::Para
            || common.vert_align != VertAlign::Top
            || !matches!(
                common.text_wrap,
                TextWrap::TopAndBottom | TextWrap::InFrontOfText
            )
            || offset < 0
            || signed_hwpunit(common.height) <= 0
            || common.margin.top != 0
            || common.margin.bottom != 0
        {
            return None;
        }
        has_flow_picture |= common.text_wrap == TextWrap::TopAndBottom;
        picture_band_end = picture_band_end.max(offset.checked_add(signed_hwpunit(common.height))?);
    }
    if !has_flow_picture || picture_band_end > line.vertical_pos {
        return None;
    }
    Some(StoredEmptyPictureCellFrame {
        line_offset_hu: line.vertical_pos,
        picture_band_origin_hu: line.vertical_pos - picture_band_end,
        content_height_hu: line.vertical_pos.checked_add(line.line_height)?,
    })
}

/// 내용 유닛을 소비하지 않는 시작 행의 물리 공간과 이어받는 행의 요구 높이.
#[derive(Debug, Clone, Copy)]
pub(crate) struct StoredEmptyOpeningRowFrame {
    pub opening_height: f64,
    pub continuation_height: f64,
}

/// 빈 문단의 원본 어울림 형제 표가 공유하는 저장 호스트 프레임.
/// 음수 세로 오프셋은 같은 저장 줄에 놓인 형제의 별도 상단이 아니다.
/// 한컴 단일 속성 대조군에서 이를 0으로 바꿔도 전체 텍스트/그림 배치는 같다.
pub(crate) fn stored_square_sibling_outer_frame(para: &Paragraph) -> bool {
    if !para.text.trim().is_empty()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || para.controls.len() < 2
        || !matches!(para.line_segs.as_slice(), [line]
            if line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                && line.vertical_pos > 0 && line.line_height > 0)
    {
        return false;
    }
    para.controls.iter().all(|control| {
        matches!(control, Control::Table(t)
        if !t.common.treat_as_char && t.common.flow_with_text
            && t.caption.is_none()
            && signed_hwpunit(t.common.width) > 0 && signed_hwpunit(t.common.height) > 0
            && t.common.text_wrap == TextWrap::Square
            && t.common.vert_rel_to == VertRelTo::Para
            && t.common.vert_align == VertAlign::Top
            && t.common.horz_rel_to == HorzRelTo::Column
            && t.common.horz_align == HorzAlign::Left
            && signed_hwpunit(t.common.horizontal_offset) >= 0
            && t.cells.iter().all(|cell| cell.paragraphs.iter().all(|p|
                !p.stored_text_partition_is_dirty() && !p.cell_format_vpos_dirty)))
    })
}

/// 자리차지하지 않는 표 하나만 든 폭0 저장 줄은 그 개체의 앵커다.
/// 이 문단의 줄 간격을 개체의 캡션 간격으로 사용하지 않는다.
pub(crate) fn object_only_saved_table_anchor(para: &Paragraph, table: &Table) -> bool {
    // A zero-origin source line is an object anchor even when frame settings
    // accompany it. A positive host origin reserves paragraph space of its own;
    // keep that spacing on the existing single-control contract.
    let allow_frame_settings = !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && matches!(para.line_segs.as_slice(), [line]
            if line.vertical_pos == 0
                && line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0);
    let mut objects = para.controls.iter().filter(|control| {
        !allow_frame_settings || !matches!(control, Control::SectionDef(_) | Control::ColumnDef(_))
    });
    let only_table = matches!(objects.next(), Some(Control::Table(_))) && objects.next().is_none();
    !table.common.treat_as_char
        && is_para_topbottom_float(&table.common)
        && para.text.is_empty()
        && only_table
        && matches!(para.line_segs.as_slice(), [line] if line.segment_width == 0 && line.tag & 0x8000_0000 == 0)
}

pub(crate) fn block_table_caption_host_spacing_px(
    para: &Paragraph,
    table: &Table,
    dpi: f64,
) -> f64 {
    if object_only_saved_table_anchor(para, table) {
        0.0
    } else {
        para.line_segs
            .first()
            .map_or(0.0, |line| hwpunit_to_px(line.line_spacing, dpi))
    }
}

/// 선언 개체 높이와 바깥 위·아래 여백이 본문을 닫는 원본 표 프레임.
/// 폭0 앵커 줄은 표를 가리킬 뿐 위여백을 대신 점유하는 본문 글줄이 아니다.
/// 구역·단·쪽번호 설정은 이 앵커의 가시 내용을 늘리지 않는다.
pub(crate) fn stored_body_filling_rowbreak_frame(
    para: &Paragraph,
    table: &Table,
    body_height: f64,
    dpi: f64,
    hwpx_stored: bool,
    session_edited: bool,
) -> bool {
    if !hwpx_stored
        || session_edited
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || table.page_break != TablePageBreak::RowBreak
        || table.common.vert_rel_to != VertRelTo::Para
        || table.common.vert_align != VertAlign::Top
        || signed_hwpunit(table.common.vertical_offset) != 0
        || table.common.height == 0
        || table.common.height >= 0x8000_0000
        || table.outer_margin_top == 0
        || body_height <= 0.0
        || !para.text.is_empty()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
    {
        return false;
    }
    let [line] = para.line_segs.as_slice() else {
        return false;
    };
    if line.vertical_pos != 0
        || line.segment_width != 0
        || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || para
            .controls
            .iter()
            .filter(|ctrl| matches!(ctrl, Control::Table(_)))
            .count()
            != 1
        || para.controls.iter().any(|ctrl| {
            !matches!(
                ctrl,
                Control::Table(_)
                    | Control::SectionDef(_)
                    | Control::ColumnDef(_)
                    | Control::PageNumberPos(_)
            )
        })
    {
        return false;
    }
    let frame_hu = i64::from(table.common.height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    // 용지 정규화와 단위 변환의 끝자리 차이는 원본 정수 단위에서 비교한다.
    // px 변환 뒤 2HU를 다시 비교하면 경계의 부동소수 오차로 동일성이 깨진다.
    let body_hu = (body_height * 7200.0 / dpi).round() as i64;
    (frame_hu - body_hu).abs() <= 2
}

/// 바깥 여백은 셀 열 수와 무관하게 개체 프레임에 속한다.
/// 이미 결정된 배치 원점과 중첩 프레임은 호출자가 처리한다.
pub(crate) fn column_rowbreak_fragment_opens_outer_top(
    hwpx_stored: bool,
    native_host: Option<&Paragraph>,
    table: &Table,
    is_continuation: bool,
    start_row: usize,
    start_cut: &[usize],
    starts_at_column_top: bool,
) -> bool {
    // 빈 저장 앵커와 표제 아래에 열리는 표는 같은 개체 프레임이다.
    // 표제의 쪽나눔 유무는 Para 앵커가 소유하는 바깥 여백을 바꾸지 않는다.
    // 표제의 글줄 잉크와 표의 바깥 여백은 별도로 소유하며 캡션 간격도 별도로 계산한다.
    let native_object_frame = native_host.is_some_and(|para| {
        if object_only_saved_table_anchor(para, table) {
            return true;
        }
        // 각주·미주는 호스트 글줄에 속하며 표의 독립 바깥 프레임을 없애지 않는다.
        // 다른 개체가 함께 있으면 단일 표 프레임으로 판정하지 않는다.
        let table_control_index = para
            .controls
            .iter()
            .position(|control| matches!(control, Control::Table(_)));
        let single_table_with_notes = para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            == 1
            && para.controls.iter().all(|control| {
                matches!(
                    control,
                    Control::Table(_) | Control::Footnote(_) | Control::Endnote(_)
                )
            });
        // 여러 저장 글줄도 개체 앞에서 모두 끝나면 같은 바깥 프레임을 소유한다.
        // 앵커 계획과 같은 유효성 판정을 소비하므로 되감긴 원본 줄은 받아들이지 않는다.
        if single_table_with_notes
            && table_control_index.is_some_and(|index| {
                ParagraphFloatPlacement::text_head_control_position(para, index).is_some()
            })
            && table_control_index.is_some_and(|index| {
                ParagraphFloatPlacement::stored_host_lines_are_valid(para, table, index)
            })
            && para
                .line_segs
                .first()
                .is_some_and(|line| line.vertical_pos >= 0)
            && !para.cell_format_vpos_dirty
        {
            return true;
        }
        let [line] = para.line_segs.as_slice() else {
            return false;
        };
        para_has_non_whitespace_text(para)
            && single_table_with_notes
            && !para.stored_text_partition_is_dirty()
            && !para.cell_format_vpos_dirty
            && line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && line.vertical_pos >= 0
            && line.line_height > 0
            && table.common.vert_rel_to == VertRelTo::Para
            && matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
            && signed_hwpunit(table.common.vertical_offset) >= line.line_height
    });
    // 저장 앵커가 첫 글줄 안에서 시작하면 그 글줄이 표 위 여백을 이미 소유한다.
    // 독립된 본문 밴드 뒤에서 시작하는 앵커만 첫 조각과 다음 쪽에 여백을 연다.
    // [#5585] 이어받은 조각의 이 계약은 저장 HWPX 와 저장 쪽 나눔을 쓰는 HWP5 에 같다 —
    // 한글 2020 PDF 는 HWP5 문단 기준 표(148776468 pi=169·pi=121)의 이어받은 조각도 쪽
    // 머리에서 바깥 위 여백(140HU)만큼 내려 그리고, 다음 문단은 저장 vpos 그대로 그 아래에
    // 둔다. HWP5 의 첫 조각은 호스트 간격이 이미 위 여백을 소유하므로 넓히지 않는다
    // (넓히면 hwpspec 178→180쪽). 위 행에서 내려온 병합 칸 한가운데서 이어지는 조각은
    // 앞 조각의 칸 상자가 계속되는 것이라 여백을 다시 열지 않는다 — 한글 2020 PDF 는
    // 1371000-201200057 8·10·14·15쪽(지역 열 병합 칸 안에서 이어짐)의 조각을 본문 위에
    // 붙여 그린다.
    let para_anchor_below_first_line = native_host.is_some_and(|host| {
        object_only_saved_table_anchor(host, table)
            && host
                .line_segs
                .first()
                .is_some_and(|line| line.vertical_pos >= line.line_height)
    });
    (hwpx_stored || native_object_frame)
        && !table.common.treat_as_char
        && is_para_topbottom_float(&table.common)
        && (table.common.horz_rel_to == HorzRelTo::Column
            || ((hwpx_stored || (is_continuation && !rowspan_straddles_row(table, start_row)))
                && table.common.horz_rel_to == HorzRelTo::Para
                && table.common.vert_rel_to == VertRelTo::Para
                && para_anchor_below_first_line))
        && table.page_break == TablePageBreak::RowBreak
        && table.outer_margin_top > 0
        && ((!is_continuation && start_row == 0 && start_cut.is_empty())
            || (is_continuation
                && starts_at_column_top
                && (table.common.horz_rel_to == HorzRelTo::Column
                    || (native_host.is_some_and(|host| {
                        !host.stored_text_partition_is_dirty()
                            && !host.cell_format_vpos_dirty
                            && !host.line_segs.is_empty()
                    })))))
}

/// 위 행에서 시작한 병합 칸이 `row` 를 걸쳐 내려오는가 (`row` 가 그 칸의 첫 행이 아님).
fn rowspan_straddles_row(table: &Table, row: usize) -> bool {
    table.cells.iter().any(|cell| {
        let top = cell.row as usize;
        top < row && top + (cell.row_span as usize).max(1) > row
    })
}

/// A paragraph-following front overlay still owns a physical RowBreak frame.
/// Its text wrapping policy does not remove the frame margins when its source
/// cells continue on another page. Fixed overlays and behind-text backgrounds
/// keep their separate placement contract.
pub(crate) fn paragraph_following_overlay_rowbreak_frame(table: &Table) -> bool {
    !table.common.treat_as_char
        && table.common.flow_with_text
        && table.common.vert_rel_to == VertRelTo::Para
        && table.common.text_wrap == TextWrap::InFrontOfText
        && table.page_break == TablePageBreak::RowBreak
}

pub(crate) fn column_rowbreak_caption_outer_spacing_px(
    fragment_opens_outer_margin: bool,
    para: &Paragraph,
    table: &Table,
    dpi: f64,
) -> f64 {
    if fragment_opens_outer_margin
        && object_only_saved_table_anchor(para, table)
        && table.caption.as_ref().is_some_and(|caption| {
            matches!(
                caption.direction,
                crate::model::shape::CaptionDirection::Top
                    | crate::model::shape::CaptionDirection::Bottom
            )
        })
    {
        hwpunit_to_px(table.outer_margin_bottom as i32, dpi)
    } else {
        0.0
    }
}

/// 종료 표 조각 바로 뒤의 저장 문단은 조각의 물리 하단에서 시작할 수 있다.
/// 그 문단의 spacing_before는 이미 공유 경계에 반영되어 있으므로
/// 쪽 나눔과 실제 배치가 같은 값을 재사용해 중복 예약하지 않는다
/// (#7406, 39–40쪽; RowBreak 대조군 5쪽의 가시 본문도 같은 경계를 공유한다).
pub(crate) fn stored_after_partial_table_shared_spacing_px(
    stored_layout: bool,
    previous_is_partial_table: bool,
    para: &Paragraph,
    spacing_before: f64,
    flow_from_body_top: f64,
    dpi: f64,
) -> f64 {
    if !stored_layout
        || !previous_is_partial_table
        || spacing_before <= 0.0
        || !para.controls.is_empty()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
    {
        return 0.0;
    }
    let Some(first) = para.line_segs.first() else {
        return 0.0;
    };
    if first.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 {
        return 0.0;
    }
    let saved_top = hwpunit_to_px(first.vertical_pos, dpi);
    if saved_top > 0.0 && (saved_top - flow_from_body_top).abs() <= 1.0 {
        spacing_before
    } else {
        0.0
    }
}

/// 캡션이 있는 떠 있는 그림의 끝과 다음 저장 글줄의 시작이 일치할 수 있다.
/// 이 경우 다음 문단의 앞 간격은 그림·캡션 배치 경계에 이미 포함되어 있으므로
/// 다시 더하지 않는다.
pub(crate) fn hwpx_after_picture_caption_shared_spacing_px(
    hwpx_stored: bool,
    previous: Option<&Paragraph>,
    para: &Paragraph,
    spacing_before: f64,
    flow_from_body_top: f64,
    dpi: f64,
) -> f64 {
    if !hwpx_stored || spacing_before <= 0.0 {
        return 0.0;
    }
    let has_bottom_caption = previous.is_some_and(|previous| {
        previous.controls.iter().any(|control| {
            matches!(control,
                Control::Picture(picture)
                    if !picture.common.treat_as_char
                        && picture.common.text_wrap == TextWrap::TopAndBottom
                        && picture.caption.as_ref().is_some_and(|caption|
                            caption.direction == crate::model::shape::CaptionDirection::Bottom))
        })
    });
    if !has_bottom_caption {
        return 0.0;
    }
    let Some(first) = para.line_segs.first() else {
        return 0.0;
    };
    if first.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 {
        return 0.0;
    }
    let saved_top = hwpunit_to_px(first.vertical_pos, dpi);
    if saved_top > 0.0 && (saved_top - flow_from_body_top).abs() <= 1.0 {
        spacing_before
    } else {
        0.0
    }
}

/// 다시 시작하는 뒤 문단이 선언된 개체 바깥 상자를 정확히 닫는다.
/// 사이의 저장된 빈 줄 사다리는 이 프레임에 속하며 별도 문단 전진이 아니다.
/// 이 원본 관계는 새 단의 개체 원점도 결정한다.
/// 원래 문단 기준 오프셋은 이월 전에 이미 소비됐다.
#[derive(Debug, Clone)]
pub(crate) struct StoredTableFrameWithGuides {
    pub guide_range: Range<usize>,
    pub top_hu: i32,
    pub bottom_hu: i32,
}

pub(crate) fn stored_table_frame_with_guides(
    paragraphs: &[Paragraph],
    host_index: usize,
    table: &Table,
) -> Option<StoredTableFrameWithGuides> {
    use crate::model::paragraph::{ColumnBreakType, LineSeg};
    let host = paragraphs.get(host_index)?;
    if !host.text.is_empty()
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || host.column_type != ColumnBreakType::None
        || host.stored_text_partition_is_dirty()
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || table.common.vert_rel_to != VertRelTo::Para
        || table.common.horz_rel_to != HorzRelTo::Column
        || table.page_break != TablePageBreak::RowBreak
        || signed_hwpunit(table.common.vertical_offset) <= 0
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
    {
        return None;
    }
    let [anchor] = host.line_segs.as_slice() else {
        return None;
    };
    if anchor.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || anchor.vertical_pos <= 0 {
        return None;
    }
    let top_hu = i32::from(table.outer_margin_top);
    let bottom_hu = i32::try_from(
        i64::from(top_hu) + i64::from(table.common.height) + i64::from(table.outer_margin_bottom),
    )
    .ok()?;
    let mut end = host_index + 1;
    let mut previous = anchor;
    while let Some(guide) = paragraphs.get(end) {
        if guide.para_shape_id != host.para_shape_id
            || !guide.text.is_empty()
            || !guide.controls.is_empty()
            || guide.column_type != ColumnBreakType::None
            || guide.stored_text_partition_is_dirty()
        {
            break;
        }
        let [line] = guide.line_segs.as_slice() else {
            return None;
        };
        if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || previous.line_height.saturating_add(previous.line_spacing) <= 0
            || line.vertical_pos
                != previous
                    .vertical_pos
                    .checked_add(previous.line_height)?
                    .checked_add(previous.line_spacing)?
        {
            return None;
        }
        previous = line;
        end += 1;
    }
    if end == host_index + 1 {
        return None;
    }
    let successor = paragraphs.get(end)?;
    if successor.stored_text_partition_is_dirty() || successor.column_type != ColumnBreakType::None
    {
        return None;
    }
    let first = successor.line_segs.first()?;
    if first.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || first.vertical_pos != bottom_hu
        || first.vertical_pos >= anchor.vertical_pos
    {
        return None;
    }
    Some(StoredTableFrameWithGuides {
        guide_range: host_index + 1..end,
        top_hu,
        bottom_hu,
    })
}

/// 문단 내부 컨트롤은 그 원시 UTF-16 위치가 속한 저장 줄을 앵커로 사용한다.
/// 전체 표 높이와 바깥여백이 뒤 문단의 저장 원점을 정확히 닫는 경우만
/// 원점을 확정한다. 첫/끝 줄의 기존 배치와 편집·재조판은 호출자가 분리한다.
pub(crate) fn stored_interior_control_table_frame(
    host: &Paragraph,
    successor: &Paragraph,
    control_index: usize,
    table: &Table,
    measured_height: f64,
    frame_vpos: i32,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::paragraph::LineSeg;
    let valid = |para: &Paragraph| {
        !para.stored_text_partition_is_dirty()
            && !para.line_segs.is_empty()
            && para
                .line_segs
                .iter()
                .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
            && para.line_segs.windows(2).all(|pair| {
                pair[1].text_start > pair[0].text_start
                    && pair[1].vertical_pos > pair[0].vertical_pos
            })
    };
    if !valid(host)
        || !valid(successor)
        || host.text.trim().is_empty()
        || host.controls.len() != 1
        || !matches!(host.controls.first(), Some(Control::Table(_)))
        || !table.common.flow_with_text
        || !is_para_topbottom_float(&table.common)
        || table.common.horz_rel_to != HorzRelTo::Column
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || signed_hwpunit(table.common.vertical_offset) <= 0
        || table.page_break != TablePageBreak::RowBreak
        || table.caption.is_some()
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || (measured_height * 7200.0 / dpi).round() != f64::from(table.common.height)
    {
        return None;
    }
    let line_index = *stored_control_line_indices(host)?.get(control_index)?;
    if line_index == 0 || line_index + 1 >= host.line_segs.len() {
        return None;
    }
    let anchor = host.line_segs.get(line_index)?.vertical_pos;
    let top = i64::from(anchor)
        + i64::from(signed_hwpunit(table.common.vertical_offset))
        + i64::from(table.outer_margin_top);
    let bottom = top + i64::from(table.common.height) + i64::from(table.outer_margin_bottom);
    let host_end = host
        .line_segs
        .iter()
        .map(|line| i64::from(line.vertical_pos) + i64::from(line.line_height))
        .max()?;
    // 남은 호스트 글줄과 표가 겹치지 않는 독립 저장 프레임이어야 한다.
    if anchor < frame_vpos
        || top < host_end
        || bottom != i64::from(successor.line_segs.first()?.vertical_pos)
    {
        return None;
    }
    let px = |value: i64| (value - i64::from(frame_vpos)) as f64 * dpi / 7200.0;
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y: px(i64::from(anchor)),
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top: px(top),
        occupied_bottom: px(bottom),
    })
}

/// A saved TAC line can close the preceding float's complete outer frame
/// at the column origin inside the same control-only host. Its height belongs
/// to the TAC, while its start closes the zero-offset float's full outer box.
/// A float offset after the TAC is a different ordering, not this frame.
pub(crate) fn stored_float_frame_before_tac_line(
    host: &Paragraph,
    control_index: usize,
    table: &Table,
    measured_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::paragraph::LineSeg;
    host.empty_control_stream_position(control_index)?;
    let [line] = host.line_segs.as_slice() else {
        return None;
    };
    let [Control::Table(a), Control::Table(b)] = host.controls.as_slice() else {
        return None;
    };
    let tac = if std::ptr::eq(a.as_ref(), table) {
        b
    } else if std::ptr::eq(b.as_ref(), table) {
        a
    } else {
        return None;
    };
    let offset = signed_hwpunit(table.common.vertical_offset);
    if host.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || table.common.treat_as_char
        || !tac.common.treat_as_char
        || !table.common.flow_with_text
        || !is_para_topbottom_float(&table.common)
        || table.common.vert_rel_to != VertRelTo::Para
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || table.caption.is_some()
        || tac.caption.is_some()
        || offset != 0
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || !measured_height.is_finite()
        || !dpi.is_finite()
        || dpi <= 0.0
        || (measured_height * 7200.0 / dpi).round() != f64::from(table.common.height)
        || i64::from(line.line_height)
            != i64::from(tac.common.height)
                + i64::from(tac.outer_margin_top)
                + i64::from(tac.outer_margin_bottom)
    {
        return None;
    }
    let bottom = i64::from(ladder_vpos(host, 0, line.vertical_pos));
    let top = bottom - i64::from(table.common.height) - i64::from(table.outer_margin_bottom);
    let anchor = top - i64::from(offset) - i64::from(table.outer_margin_top);
    if anchor != 0 {
        return None;
    }
    let px = |hu: i64| hu as f64 * dpi / 7200.0;
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y: px(anchor),
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: None,
        table_top: px(top),
        occupied_bottom: px(bottom),
    })
}

/// 빈 호스트와 다음 저장 줄이 전체 개체 바깥 상자를 정확히 닫는다.
/// 실측 본체·수직 캡션이 전체 선언 프레임과 같을 때 오프셋·양쪽 여백을 한 번 소비한다.
/// 분할 조각의 선언 높이, 편집 뒤 캐시, 합성 줄은 이 원점의 증거가 아니다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stored_empty_control_table_frame(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    measured_height: f64,
    host_spacing_before: f64,
    frame_vpos: i32,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::paragraph::{ColumnBreakType, LineSeg};
    let [anchor] = host.line_segs.as_slice() else {
        return None;
    };
    let next = successor.line_segs.first()?;
    let offset = signed_hwpunit(table.common.vertical_offset);
    // 구역 재계산이 만든 누적 vpos와 원본 쪽 상대 프레임을 섞지 않는다.
    // 실제 저장본 스냅샷이 있으면 호스트와 후속 닫힘을 같은 원본 축으로 읽는다.
    let anchor_vpos = ladder_vpos(host, 0, anchor.vertical_pos);
    let next_vpos = ladder_vpos(successor, 0, next.vertical_pos);
    if !host.text.trim().is_empty()
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || host.stored_text_partition_is_dirty()
        || successor.stored_text_partition_is_dirty()
        || host.cell_format_vpos_dirty
        || successor.cell_format_vpos_dirty
        || !matches!(
            host.column_type,
            ColumnBreakType::None | ColumnBreakType::Page
        )
        || successor.column_type != ColumnBreakType::None
        || anchor.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || next.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || (anchor.segment_width != 0 && table.caption.is_none())
        || anchor.line_height <= 0
        || next.line_height <= 0
        || anchor_vpos < frame_vpos
        || !table.common.flow_with_text
        || !is_para_topbottom_float(&table.common)
        || !matches!(
            table.common.horz_rel_to,
            HorzRelTo::Column | HorzRelTo::Para
        )
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || offset < 0
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
        || !measured_height.is_finite()
        || !host_spacing_before.is_finite()
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return None;
    }
    // 위/아래 캡션은 본체와 같은 수직 개체 상자를 소유한다. 측정·배치가
    // 공유하는 캡션 높이와 저장 후속 줄이 모두 일치할 때만 원점을 재사용한다.
    let caption_extent = if let Some(caption) = &table.caption {
        use crate::model::shape::CaptionDirection;
        if !matches!(
            caption.direction,
            CaptionDirection::Top | CaptionDirection::Bottom
        ) || caption.spacing < 0
            || caption.paragraphs.is_empty()
            || caption.paragraphs.iter().any(|para| {
                para.stored_text_partition_is_dirty()
                    || para.cell_format_vpos_dirty
                    || !para.controls.is_empty()
                    || para.line_segs.is_empty()
                    || para.line_segs.iter().any(|line| {
                        line.line_height <= 0
                            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    })
            })
        {
            return None;
        }
        let height = super::composer::caption_height_px(&table.caption, dpi);
        if height <= 0.0 || !height.is_finite() {
            return None;
        }
        height * 7200.0 / dpi + f64::from(caption.spacing)
    } else {
        0.0
    };
    let measured_extent = (measured_height * 7200.0 / dpi).round();
    if measured_extent != (f64::from(table.common.height) + caption_extent).round() {
        return None;
    }
    let object_extent = i64::from(offset)
        + i64::from(table.outer_margin_top)
        + measured_extent as i64
        + i64::from(table.outer_margin_bottom);
    // An explicit page break can suppress the host paragraph's leading band.
    // Reuse that page origin only when the saved successor exactly closes the
    // complete object box (body, caption, offset, and both outer margins).
    // Otherwise retain the saved host origin; the break alone is no evidence
    // that its stored vertical position should be discarded.
    let leading_band = (host_spacing_before * 7200.0 / dpi).round() as i64;
    let anchor_origin =
        if host.column_type == ColumnBreakType::Page && object_extent == i64::from(next_vpos) {
            0
        } else if table.caption.is_some()
            && leading_band > 0
            && leading_band <= i64::from(anchor_vpos)
            && i64::from(anchor_vpos) - leading_band + object_extent == i64::from(next_vpos)
        {
            // The saved host line includes spacing-before, while the object anchor
            // precedes that band. Reuse it only if the complete measured outer box
            // exactly closes at the original successor line.
            i64::from(anchor_vpos) - leading_band
        } else {
            i64::from(anchor_vpos)
        };
    let top = anchor_origin + i64::from(offset) + i64::from(table.outer_margin_top);
    let bottom = top + measured_extent as i64 + i64::from(table.outer_margin_bottom);
    if bottom != i64::from(next_vpos) {
        return None;
    }
    let px = |value: i64| (value - i64::from(frame_vpos)) as f64 * dpi / 7200.0;
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y: px(anchor_origin),
        stored_host_origin: None,
        stored_successor_line_origin: Some(px(bottom)),
        table_left: None,
        table_top: px(top),
        occupied_bottom: px(bottom),
    })
}

/// 앞 글줄의 전진 끝에 있는 저장 표 앵커와 다음 쪽의 본문 재시작이다.
/// 표가 원래 줄 폭을 가진 빈 호스트에 저장되어도 개체 원점은 같은 바깥 상자다.
/// 앞 줄 사다리와 실측 본체 높이를 확인해 호스트 줄 높이를 별도로 더하지 않는다.
pub(crate) fn stored_adjacent_line_table_frame(
    previous: &Paragraph,
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    measured_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::paragraph::{ColumnBreakType, LineSeg};
    let [anchor] = host.line_segs.as_slice() else {
        return None;
    };
    let before = previous.line_segs.last()?;
    let after = successor.line_segs.first()?;
    let valid = |para: &Paragraph| {
        !para.stored_text_partition_is_dirty()
            && !para.cell_format_vpos_dirty
            && para.column_type == ColumnBreakType::None
            && !para.line_segs.is_empty()
            && para.line_segs.iter().all(|line| {
                line.line_height > 0 && line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
    };
    if !valid(previous)
        || !valid(host)
        || !valid(successor)
        || !previous.controls.is_empty()
        || !successor.controls.is_empty()
        || para_has_non_whitespace_text(host)
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || anchor.vertical_pos <= 0
        || before.vertical_pos <= 0
        || after.vertical_pos != 0
        || i64::from(anchor.vertical_pos)
            != i64::from(before.vertical_pos)
                + i64::from(before.line_height)
                + i64::from(before.line_spacing)
        || table.common.treat_as_char
        || !table.common.flow_with_text
        || !is_para_topbottom_float(&table.common)
        || table.common.horz_rel_to != HorzRelTo::Column
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || table.page_break != TablePageBreak::RowBreak
        || table.caption.is_some()
        || signed_hwpunit(table.common.vertical_offset) < 0
        || table.common.height == 0
        || !measured_height.is_finite()
        || dpi <= 0.0
        || (measured_height * 7200.0 / dpi).round() != f64::from(table.common.height)
    {
        return None;
    }
    let anchor_y = hwpunit_to_px(anchor.vertical_pos, dpi);
    let table_top = anchor_y
        + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi)
        + hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y,
        stored_host_origin: Some(anchor_y),
        stored_successor_line_origin: None,
        table_left: None,
        table_top,
        occupied_bottom: table_top
            + measured_height
            + hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi),
    })
}

/// 마지막 행이 이어지는 저장 표에서 전체 행 프레임과 첫 개체 프레임의
/// 차이가 뒤 문단의 저장 원점을 정확히 닫으면 종료 조각의 물리 높이이다.
/// 내용 유닛에 없는 빈 밴드도 보존하며, 편집·로컬 재시작은 증거로 쓰지 않는다.
pub(crate) fn stored_rowbreak_closing_frame_height(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    dpi: f64,
) -> Option<f64> {
    use crate::model::paragraph::LineSeg;
    let before = host.line_segs.first()?;
    let after = successor.line_segs.first()?;
    if host.stored_text_partition_is_dirty()
        || successor.stored_text_partition_is_dirty()
        || host
            .line_segs
            .iter()
            .chain(&successor.line_segs)
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
        || !successor.controls.is_empty()
        || after.vertical_pos <= 0
        || before.vertical_pos <= after.vertical_pos
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || table.page_break != TablePageBreak::RowBreak
        || table.row_count <= 1
        || table.common.height == 0
        || table.caption.is_some()
    {
        return None;
    }
    let rows = table.get_raw_row_heights();
    if rows.len() != usize::from(table.row_count) || rows.contains(&0) {
        return None;
    }
    let whole = rows.iter().map(|height| i64::from(*height)).sum::<i64>()
        + i64::from(table.cell_spacing) * (rows.len() - 1) as i64;
    let remaining = whole - i64::from(table.common.height);
    // 뒤 문단 원점은 표 상자와 바깥 여백을 함께 닫는다. 반환 높이는
    // 표 상자만 소유하고 여백은 기존 조각 배치가 별도로 예약한다.
    let margins = i64::from(table.outer_margin_top) + i64::from(table.outer_margin_bottom);
    (remaining > 0 && remaining + margins == i64::from(after.vertical_pos))
        .then(|| remaining as f64 * dpi / 7200.0)
}

/// 단일 셀의 전체 선언 높이와 첫 개체 프레임의 차이를 뒤 빈 문단이
/// 정확히 닫으면 종료 조각의 물리 높이이다. 내용 높이와 빈 밴드를 구분한다.
pub(crate) fn stored_single_cell_closing_frame_height(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    dpi: f64,
) -> Option<f64> {
    use crate::model::paragraph::LineSeg;
    let [cell] = table.cells.as_slice() else {
        return None;
    };
    let [next] = successor.line_segs.as_slice() else {
        return None;
    };
    if table.row_count != 1
        || table.col_count != 1
        || cell.row_span != 1
        || cell.col_span != 1
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || table.page_break != TablePageBreak::RowBreak
        || table.caption.is_some()
        || table.outer_margin_bottom != 0
        || table.common.height == 0
        || table.common.height >= cell.height
        || cell.height > i32::MAX as u32
        || !host.text.is_empty()
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || !successor.text.is_empty()
        || !successor.controls.is_empty()
        || host.stored_text_partition_is_dirty()
        || successor.stored_text_partition_is_dirty()
        || host.line_segs.is_empty()
        || host
            .line_segs
            .iter()
            .chain(&successor.line_segs)
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
    {
        return None;
    }
    let remaining = cell.height - table.common.height;
    (i64::from(next.vertical_pos) == i64::from(remaining))
        .then(|| hwpunit_to_px(next.vertical_pos, dpi))
}

/// 다음 빈 문단이 누적 저장 원점과 개체 프레임의 끝을 정확히 잇는 경우,
/// 선언 높이를 첫 RowBreak 조각의 물리 높이로 반환한다.
pub(crate) fn stored_cumulative_rowbreak_opening_frame_height(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    dpi: f64,
) -> Option<f64> {
    let first = host.line_segs.first()?;
    let next = successor.line_segs.first()?;
    if host.stored_text_partition_is_dirty()
        || successor.stored_text_partition_is_dirty()
        || !para_has_non_whitespace_text(host)
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || !successor.text.is_empty()
        || !successor.controls.is_empty()
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || table.page_break != TablePageBreak::RowBreak
        || table.row_count <= 1
        || table.common.height == 0
        || table.common.height > i32::MAX as u32
    {
        return None;
    }
    let offset = signed_hwpunit(table.common.vertical_offset);
    let expected =
        i64::from(first.vertical_pos) + i64::from(offset) + i64::from(table.common.height);
    (offset > 0 && expected == i64::from(next.vertical_pos))
        .then(|| hwpunit_to_px(table.common.height as i32, dpi))
}

/// 종료 조각의 실제 하단과 뒤 원본 줄 사이에 저장된 바깥 아래 여백이다.
/// 원본 줄의 시작점과 프레임의 종료식이 맞는 경우만 반환한다.
pub(crate) fn stored_terminal_rowbreak_outer_margin_px(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    fragment_bottom: f64,
    dpi: f64,
) -> Option<f64> {
    stored_terminal_rowbreak_outer_margin_with_consumed_host_px(
        host,
        successor,
        table,
        fragment_bottom,
        false,
        dpi,
    )
}

/// 앞 프레임에서 이미 소비한 제목·주석은 이어받기 프레임의 공간을 다시 차지하지 않는다.
/// 다른 개체가 없고 뒤 저장 줄이 종료 여백을 정확히 닫는 경우에만 이 계약을 확장한다.
pub(crate) fn stored_terminal_rowbreak_outer_margin_with_consumed_host_px(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    fragment_bottom: f64,
    host_consumed_in_previous_frame: bool,
    dpi: f64,
) -> Option<f64> {
    use crate::model::paragraph::LineSeg;
    let line = successor.line_segs.first()?;
    let consumed_host = host_consumed_in_previous_frame
        && host
            .controls
            .iter()
            .filter(|c| matches!(c, Control::Table(_)))
            .count()
            == 1
        && host.controls.iter().all(|c| {
            matches!(
                c,
                Control::Table(_) | Control::Footnote(_) | Control::Endnote(_)
            )
        })
        && successor.text.is_empty()
        && successor.controls.is_empty();
    if (!consumed_host
        && (para_has_non_whitespace_text(host)
            || !matches!(host.controls.as_slice(), [Control::Table(_)])))
        || host.stored_text_partition_is_dirty()
        || successor.stored_text_partition_is_dirty()
        || host
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos <= 0
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || table.page_break != TablePageBreak::RowBreak
        || table.outer_margin_bottom == 0
    {
        return None;
    }
    let margin = hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    ((fragment_bottom + margin - hwpunit_to_px(line.vertical_pos, dpi)).abs() <= 0.5)
        .then_some(margin)
}

/// 표 앞에서 선행 소비한 줄 다음의 저장 원점이 종료 조각의 바깥 상자를 닫는다.
/// 다음 쪽 첫 줄은 문단 첫 줄과 다르므로 실제 소유 컷을 받아 판정한다.
pub(crate) fn stored_terminal_rowbreak_outer_margin_after_prefix_px(
    host: &Paragraph,
    successor: &Paragraph,
    table: &Table,
    successor_start_line: usize,
    fragment_bottom: f64,
    dpi: f64,
) -> Option<f64> {
    let previous = successor
        .line_segs
        .get(successor_start_line.checked_sub(1)?)?;
    let line = successor.line_segs.get(successor_start_line)?;
    if !ParagraphFloatPlacement::stored_head_host_lines_are_valid(host, table, 0)
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || !successor.controls.is_empty()
        || successor.stored_text_partition_is_dirty()
        || successor.cell_format_vpos_dirty
        || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos <= 0
        || line.vertical_pos >= previous.vertical_pos
        || table.page_break != TablePageBreak::RowBreak
        || table.outer_margin_bottom == 0
    {
        return None;
    }
    let margin = hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    ((fragment_bottom + margin - hwpunit_to_px(line.vertical_pos, dpi)).abs() <= dpi / 7200.0)
        .then_some(margin)
}

/// 문단 상대 떠 있는 개체의 확정된 배치. 모든 값은 단 상대 px다.
/// 예약과 출력이 같은 결과를 사용하므로 renderer에서 원점을 다시 더하지 않는다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParagraphFloatPlacement {
    /// 기하만으로 문단 흐름을 소비하지 않는다. 떠 있는 배제 영역 위에
    /// 후속 텍스트가 들어갈 공간이 남을 수 있다.
    pub flow: ParagraphFloatFlow,
    pub anchor_y: f64,
    /// 텍스트 생성과 공유하는 유효 저장 줄 또는 현재 재조판 호스트 원점.
    /// None이면 이어받기를 포함한 기존 흐름 호스트 계약을 유지한다.
    pub stored_host_origin: Option<f64>,
    /// 전체 저장 프레임이 닫는 후속 첫 글줄 원점. 후속 문단의 앞 간격도 이 경계에 포함된다.
    pub stored_successor_line_origin: Option<f64>,
    /// 바깥여백을 이미 소비한 표 왼쪽 원점. None이면 기존 가로 배치 계약이다.
    pub table_left: Option<f64>,
    /// 표와 캡션을 함께 담는 배치 상자의 상단. 위 캡션은 이 상자 안에서 배치한다.
    pub table_top: f64,
    pub occupied_bottom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParagraphFloatFlow {
    Exclusion,
    /// The trailing object cannot fit in the host line's remaining inline space.
    NextLine,
    /// The saved successor line starts at the picture's occupied bottom. Its
    /// spacing-before is already inside that reservation, not an extra gap.
    StoredPicture {
        next_flow_y: f64,
    },
}

/// 확정 개체 프레임의 후속 줄은 이미 앞 간격을 포함한 저장 원점을 소유한다.
/// 실제 커서가 그 경계에 있을 때만 측정과 배치가 같은 앞 간격을 한 번 소비한다.
pub(crate) fn stored_frame_successor_shared_spacing_px(
    placements: &std::collections::HashMap<(usize, usize), ParagraphFloatPlacement>,
    para_index: usize,
    spacing_before: f64,
    current_y: f64,
) -> f64 {
    if spacing_before <= 0.0 {
        return 0.0;
    }
    placements
        .iter()
        .find_map(|(&(owner, _), placement)| {
            let origin = placement.stored_successor_line_origin?;
            (owner.checked_add(1) == Some(para_index) && (origin - current_y).abs() <= 0.5)
                .then_some(spacing_before)
        })
        .unwrap_or(0.0)
}

/// A two-frame cell's declared total and the successor line independently
/// close its opening box. Multi-page cell ladders are not this contract.
pub(crate) fn stored_two_frame_successor_origin_hu(
    table: &Table,
    successor: &Paragraph,
) -> Option<i32> {
    let [cell] = table.cells.as_slice() else {
        return None;
    };
    let [line] = successor.line_segs.as_slice() else {
        return None;
    };
    if cell.dirty_flag
        || table.row_count != 1
        || table.col_count != 1
        || table.common.height == 0
        || cell.height > i32::MAX as u32
        || successor.stored_text_partition_is_dirty()
        || successor.cell_format_vpos_dirty
        || !successor.controls.is_empty()
        || !successor.text.trim().is_empty()
        || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.vertical_pos <= 0
    {
        return None;
    }
    let mut previous = None;
    let mut resets = 0;
    for para in &cell.paragraphs {
        if para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.line_segs.is_empty()
        {
            return None;
        }
        for seg in &para.line_segs {
            if seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || seg.vertical_pos < 0
                || seg.line_height <= 0
            {
                return None;
            }
            if previous.is_some_and(|pos| seg.vertical_pos < pos) {
                if seg.vertical_pos != 0 {
                    return None;
                }
                resets += 1;
            }
            previous = Some(seg.vertical_pos);
        }
    }
    let extent = i64::from(table.common.height) + i64::from(line.vertical_pos)
        - i64::from(table.outer_margin_top)
        - i64::from(table.outer_margin_bottom);
    (resets == 1 && extent == i64::from(cell.height)).then_some(line.vertical_pos)
}

/// 실제 재조판에서 확정한 호스트 줄. 문자 위치는 `Paragraph.text`의 scalar 축,
/// 세로 위치는 spacing-before를 제외한 첫 텍스트 줄 상대 px다.
#[derive(Debug, Clone, Copy)]
pub struct ParagraphHostLine {
    pub char_start: usize,
    pub top: f64,
    pub height: f64,
}

/// 양수 앞 간격을 가진 빈 후속 줄의 기존 저장 프레임 계약.
/// 다음 문단/실제 앞 커서를 받지 않는 호출은 0앞간격으로 흐름을 되감지 않는다.
pub fn stored_picture_successor_placement(
    para: &Paragraph,
    successor: &Paragraph,
    spacing_before: f64,
    successor_spacing_before: f64,
    frame_vpos: i32,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    stored_picture_successor_with_following_placement(
        para,
        successor,
        None,
        spacing_before,
        successor_spacing_before,
        frame_vpos,
        f64::NAN,
        dpi,
    )
}

/// 빈 그림 호스트의 저장 프레임 끝과 후속 줄이 같은 흐름을 소유하는지 확인한다.
/// 양수 앞 간격이 없거나 글자를 가진 후속 줄은 다음 저장 줄까지의 전진(그 줄의 앞 간격
/// 포함)과 실제 앞 커서도 일치해야 한다. 그림과 줄이 기하적으로 닿는다는 사실만으로
/// 흐름을 되감지 않는다 — 글자 있는 후속 줄은 세 독립 기록(그림 하단 = 후속 저장 줄,
/// 후속 줄 끝 + 다음 앞 간격 = 다음 저장 줄, 스냅된 실제 커서 = 그림 앵커)이 모두 맞을 때만
/// 이 프레임을 따른다.
/// 호스트의 0폭 줄은 그림이 소유하며, 후속 빈 줄은 자신의 음수 간격도 보존한다.
/// 편집/합성·명시 경계는 호출자와 이 함수의 저장 계약에서 제외한다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stored_picture_successor_with_following_placement(
    para: &Paragraph,
    successor: &Paragraph,
    following: Option<(&Paragraph, f64)>,
    spacing_before: f64,
    successor_spacing_before: f64,
    frame_vpos: i32,
    actual_host_flow_y: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let [Control::Picture(picture)] = para.controls.as_slice() else {
        return None;
    };
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    let next = successor.line_segs.first()?;
    let common = &picture.common;
    if para.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
        || !successor.controls.is_empty()
        || successor.column_type != crate::model::paragraph::ColumnBreakType::None
        || host.tag & 0x8000_0000 != 0
        || next.tag & 0x8000_0000 != 0
        || host.segment_width != 0
        || host.line_height <= 0
        || next.line_height <= 0
        || common.treat_as_char
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || common.margin.top != 0
        || picture.caption.is_some()
        || !dpi.is_finite()
        || dpi <= 0.0
        || !spacing_before.is_finite()
        || !successor_spacing_before.is_finite()
        || successor_spacing_before < 0.0
    {
        return None;
    }
    let host_y = hwpunit_to_px(host.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let anchor_y = host_y - spacing_before;
    let top = anchor_y + hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
    let (_, height) = picture_flow_frame_size_hu(picture);
    let bottom =
        top + hwpunit_to_px(height, dpi) + hwpunit_to_px(i32::from(common.margin.bottom), dpi);
    let next_y = hwpunit_to_px(next.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let successor_has_text = !successor.text.is_empty();
    if successor_spacing_before == 0.0 || successor_has_text {
        let (following, following_spacing_before) = following?;
        let after = following.line_segs.first()?;
        let advance = next.line_height.checked_add(next.line_spacing)?;
        let advance_px = hwpunit_to_px(advance, dpi) + following_spacing_before;
        let after_px = hwpunit_to_px(after.vertical_pos.checked_sub(next.vertical_pos)?, dpi);
        if successor.line_segs.len() != 1
            || following.column_type != crate::model::paragraph::ColumnBreakType::None
            || after.tag & 0x8000_0000 != 0
            || advance <= 0
            || !following_spacing_before.is_finite()
            || following_spacing_before < 0.0
            || (successor_has_text && successor_spacing_before != 0.0)
            || (after_px - advance_px).abs() > dpi / 7200.0
            || !actual_host_flow_y.is_finite()
            || (actual_host_flow_y - anchor_y).abs() > dpi / 7200.0
        {
            return None;
        }
    }
    // 정수 HWPUNIT의 반올림 1단위만 허용하며 픽셀 여유를 더하지 않는다.
    if anchor_y < 0.0 || height <= 0 || (bottom - next_y).abs() > dpi / 7200.0 {
        return None;
    }
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::StoredPicture {
            next_flow_y: next_y - successor_spacing_before,
        },
        anchor_y,
        stored_host_origin: Some(host_y),
        stored_successor_line_origin: None,
        table_left: None,
        table_top: top,
        occupied_bottom: bottom,
    })
}

/// 본문 끝 그림의 소유 줄과 뒤 문단의 저장 어울림 경계가 같은 프레임을 증명한다.
/// 첫 줄 기준 추정 대신 컨트롤의 UTF-16 소유 줄을 배치와 예약에 함께 전달한다.
pub(crate) fn stored_tail_square_picture_placement(
    para: &Paragraph,
    successor: &Paragraph,
    control_index: usize,
    actual_host_flow_y: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::paragraph::LineSeg;
    let Control::Picture(picture) = para.controls.get(control_index)? else {
        return None;
    };
    let common = &picture.common;
    let valid = |para: &Paragraph| {
        !para.stored_text_partition_is_dirty()
            && !para.line_segs.is_empty()
            && para.line_segs.iter().all(|line| {
                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0 && line.line_height > 0
            })
            && para.line_segs.windows(2).all(|pair| {
                pair[1].vertical_pos >= pair[0].vertical_pos
                    && pair[1].text_start >= pair[0].text_start
            })
    };
    if !valid(para)
        || !valid(successor)
        || para.text.trim().is_empty()
        || common.treat_as_char
        || common.text_wrap != TextWrap::Square
        || common.vert_rel_to != VertRelTo::Para
        || common.horz_rel_to != HorzRelTo::Column
        || common.vert_align != VertAlign::Top
        || common.horz_align != HorzAlign::Left
        || !common.flow_with_text
        || !actual_host_flow_y.is_finite()
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return None;
    }
    let owner = *stored_control_line_indices(para)?.get(control_index)?;
    if owner == 0 || owner + 1 != para.line_segs.len() {
        return None;
    }
    let first = para.line_segs.first()?;
    let line = para.line_segs.get(owner)?;
    let top = i64::from(line.vertical_pos) + i64::from(signed_hwpunit(common.vertical_offset));
    let left = signed_hwpunit(common.horizontal_offset);
    let (width, height) = picture_flow_frame_size_hu(picture);
    let right = left.checked_add(width)?;
    // 마지막 본문 줄은 그림 위에서 완결되고, 뒤 저장 줄의 양쪽 경계는
    // 같은 그림의 좌우를 정확히 가리켜야 한다. 줄 위치만으로는 인정하지 않는다.
    if width <= 0
        || height <= 0
        || line.vertical_pos <= first.vertical_pos
        || top < i64::from(line.vertical_pos) + i64::from(line.line_height)
        || !successor.line_segs.iter().any(|before| {
            let end = i64::from(before.vertical_pos) + i64::from(before.line_height);
            i64::from(before.vertical_pos) <= top
                && top < end
                && before.column_start + before.segment_width as i32 == left
                && successor.line_segs.iter().any(|after| {
                    after.vertical_pos == before.vertical_pos && after.column_start == right
                })
        })
    {
        return None;
    }
    let anchor_y = actual_host_flow_y + hwpunit_to_px(line.vertical_pos - first.vertical_pos, dpi);
    let top = anchor_y + hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::Exclusion,
        anchor_y,
        stored_host_origin: Some(actual_host_flow_y),
        stored_successor_line_origin: None,
        table_left: None,
        table_top: top,
        occupied_bottom: top + hwpunit_to_px(height, dpi),
    })
}

/// 그림 앞의 저장 줄 끝, 그림 프레임, 뒤쪽 호스트 줄과 후속 줄이 모두
/// 같은 저장 축에서 닫힐 때만 그림이 호스트 텍스트 앞의 공간을 소유한다.
/// 실제 앞 흐름도 그 경계에 있어야 하므로 편집되거나 누락된 공간을 되감지 않는다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stored_picture_before_host_placement(
    previous: &Paragraph,
    para: &Paragraph,
    successor: &Paragraph,
    spacing_after: f64,
    successor_spacing_before: f64,
    frame_vpos: i32,
    actual_host_flow_y: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let [Control::Picture(picture)] = para.controls.as_slice() else {
        return None;
    };
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    let prev = previous.line_segs.last()?;
    let next = successor.line_segs.first()?;
    let common = &picture.common;
    if !para.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
        || para.column_type != crate::model::paragraph::ColumnBreakType::None
        || successor.column_type != crate::model::paragraph::ColumnBreakType::None
        || [prev, host, next]
            .iter()
            .any(|line| line.tag & 0x8000_0000 != 0 || line.line_height <= 0)
        || common.treat_as_char
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || picture.caption.is_some()
        || !dpi.is_finite()
        || dpi <= 0.0
        || !actual_host_flow_y.is_finite()
        || !spacing_after.is_finite()
        || !successor_spacing_before.is_finite()
    {
        return None;
    }
    let prev_end = prev
        .vertical_pos
        .checked_add(prev.line_height)?
        .checked_add(prev.line_spacing)?;
    let anchor_y = hwpunit_to_px(prev_end.checked_sub(frame_vpos)?, dpi);
    let host_y = hwpunit_to_px(host.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let top = anchor_y + hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
    let (_, height) = picture_flow_frame_size_hu(picture);
    let bottom = top
        + hwpunit_to_px(height, dpi)
        + hwpunit_to_px(
            i32::from(common.margin.top) + i32::from(common.margin.bottom),
            dpi,
        );
    let host_end = host_y
        + hwpunit_to_px(host.line_height.checked_add(host.line_spacing)?, dpi)
        + spacing_after;
    let next_y = hwpunit_to_px(next.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let next_flow_y = next_y - successor_spacing_before;
    let unit = dpi / 7200.0;
    if anchor_y < 0.0
        || top < anchor_y
        || height <= 0
        || (actual_host_flow_y - anchor_y).abs() > unit
        || (bottom - host_y).abs() > unit
        || (host_end - next_flow_y).abs() > unit
    {
        return None;
    }
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::StoredPicture { next_flow_y },
        anchor_y,
        stored_host_origin: Some(host_y),
        stored_successor_line_origin: Some(next_y),
        table_left: None,
        table_top: top,
        occupied_bottom: bottom,
    })
}

/// 빈 개체 호스트 줄이 그림 프레임을 소유하고 후속 줄이 그 하단에서 시작하는
/// 완전한 저장 흐름. 앞 줄 끝과 현재 커서까지 일치해야 글줄 점유를 중복하지 않는다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn stored_picture_empty_host_placement(
    previous: &Paragraph,
    para: &Paragraph,
    successor: &Paragraph,
    successor_spacing_before: f64,
    frame_vpos: i32,
    actual_host_flow_y: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let [Control::Picture(picture)] = para.controls.as_slice() else {
        return None;
    };
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    let prev = previous.line_segs.last()?;
    let next = successor.line_segs.first()?;
    let common = &picture.common;
    if para.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
        || para.column_type != crate::model::paragraph::ColumnBreakType::None
        || successor.column_type != crate::model::paragraph::ColumnBreakType::None
        || [prev, host, next]
            .iter()
            .any(|line| line.tag & 0x8000_0000 != 0 || line.line_height <= 0)
        || common.treat_as_char
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || picture.caption.is_some()
        || !dpi.is_finite()
        || dpi <= 0.0
        || !actual_host_flow_y.is_finite()
        || !successor_spacing_before.is_finite()
    {
        return None;
    }
    let prev_end = prev
        .vertical_pos
        .checked_add(prev.line_height)?
        .checked_add(prev.line_spacing)?;
    if prev_end != host.vertical_pos {
        return None;
    }
    let host_y = hwpunit_to_px(host.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let top = host_y + hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
    let (_, height) = picture_flow_frame_size_hu(picture);
    let bottom = top
        + hwpunit_to_px(height, dpi)
        + hwpunit_to_px(
            i32::from(common.margin.top) + i32::from(common.margin.bottom),
            dpi,
        );
    let next_y = hwpunit_to_px(next.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let unit = dpi / 7200.0;
    if host_y < 0.0
        || top < host_y
        || height <= 0
        || (actual_host_flow_y - host_y).abs() > unit
        || (bottom - next_y).abs() > unit
    {
        return None;
    }
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::StoredPicture {
            next_flow_y: next_y - successor_spacing_before,
        },
        anchor_y: host_y,
        stored_host_origin: Some(host_y),
        stored_successor_line_origin: Some(next_y),
        table_left: None,
        table_top: top,
        occupied_bottom: bottom,
    })
}

/// 앞 쪽의 호스트는 남고 그림만 다음 쪽 상단을 소유하는 저장 경계.
/// 현재 프레임의 물리 초과와 다음 본문 원점의 전체 그림 높이가 함께 이를 입증한다.
pub(crate) fn stored_picture_next_page_placement(
    para: &Paragraph,
    following: &[Paragraph],
    frame_vpos: i32,
    actual_host_flow_y: f64,
    available_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let [Control::Picture(picture)] = para.controls.as_slice() else {
        return None;
    };
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    let common = &picture.common;
    if para.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
        || host.tag & 0x8000_0000 != 0
        || host.line_height <= 0
        || common.treat_as_char
        || common.allow_overlap
        || !common.flow_with_text
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || picture.caption.is_some()
        || !dpi.is_finite()
        || dpi <= 0.0
        || !available_height.is_finite()
        || !actual_host_flow_y.is_finite()
    {
        return None;
    }
    let host_y = hwpunit_to_px(host.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let offset = hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
    let (_, height) = picture_flow_frame_size_hu(picture);
    let frame_height = height
        .checked_add(i32::from(common.margin.top))?
        .checked_add(i32::from(common.margin.bottom))?;
    let bottom = hwpunit_to_px(frame_height, dpi);
    if host_y < 0.0
        || offset < 0.0
        || frame_height <= 0
        || bottom > available_height
        || (actual_host_flow_y - host_y).abs() > dpi / 7200.0
        || host_y + offset + bottom <= available_height
    {
        return None;
    }
    let mut previous_end = host.vertical_pos;
    for next in following {
        if !next.controls.is_empty() {
            return None;
        }
        let first = next.line_segs.first()?;
        if next
            .line_segs
            .iter()
            .any(|line| line.tag & 0x8000_0000 != 0 || line.line_height <= 0)
        {
            return None;
        }
        if first.vertical_pos < previous_end {
            if first.vertical_pos != frame_height
                || !next.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
            {
                return None;
            }
            return Some(ParagraphFloatPlacement {
                flow: ParagraphFloatFlow::StoredPicture {
                    next_flow_y: bottom,
                },
                anchor_y: -offset,
                stored_host_origin: None,
                stored_successor_line_origin: None,
                table_left: None,
                table_top: 0.0,
                occupied_bottom: bottom,
            });
        }
        previous_end = next.line_segs.last()?.vertical_pos;
    }
    None
}

/// 앞쪽에서 프레임이 넘친 배경 그림은 다음 원점의 겹침 허용 레이블과 같은
/// 쪽을 소유한다. 배경은 보이지만 글줄을 전진시키지 않으므로 예약 높이는 0이다.
pub(crate) fn stored_background_picture_next_page_placement(
    para: &Paragraph,
    following: &[Paragraph],
    frame_vpos: i32,
    actual_host_flow_y: f64,
    available_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::paragraph::LineSeg;
    let [Control::Picture(picture)] = para.controls.as_slice() else {
        return None;
    };
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    let common = &picture.common;
    if para_has_non_whitespace_text(para)
        || para.stored_text_partition_is_dirty()
        || host.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || host.line_height <= 0
        || common.treat_as_char
        || !common.allow_overlap
        || !common.flow_with_text
        || common.text_wrap != TextWrap::BehindText
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || picture.caption.is_some()
        || !dpi.is_finite()
        || dpi <= 0.0
        || !actual_host_flow_y.is_finite()
        || !available_height.is_finite()
    {
        return None;
    }
    let host_y = hwpunit_to_px(host.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let offset = hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
    let (_, height) = picture_flow_frame_size_hu(picture);
    let picture_height = hwpunit_to_px(height, dpi);
    if host_y < 0.0
        || offset < 0.0
        || height <= 0
        || offset + picture_height > available_height
        || (actual_host_flow_y - host_y).abs() > dpi / 7200.0
        || host_y + offset + picture_height <= available_height
    {
        return None;
    }
    let mut previous = host;
    for next in following {
        let [line] = next.line_segs.as_slice() else {
            return None;
        };
        if next.stored_text_partition_is_dirty()
            || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || line.line_height <= 0
            || para_has_non_whitespace_text(next)
        {
            return None;
        }
        if line.vertical_pos < previous.vertical_pos {
            if line.vertical_pos != 0
                || next.controls.is_empty()
                || !next.controls.iter().all(|control| {
                    matches!(control, Control::Table(table)
                    if !table.common.treat_as_char && table.common.allow_overlap
                        && table.common.vert_rel_to == VertRelTo::Para
                        && table.common.horz_rel_to == HorzRelTo::Column
                        && signed_hwpunit(table.common.vertical_offset) >= 0
                        && i64::from(signed_hwpunit(table.common.vertical_offset))
                            + i64::from(table.common.height) <= i64::from(height))
                })
            {
                return None;
            }
            return Some(ParagraphFloatPlacement {
                flow: ParagraphFloatFlow::StoredPicture { next_flow_y: 0.0 },
                // 이월된 프레임은 새 쪽 원점에서 시작한다. 출력에서 원본
                // 호스트 오프셋을 다시 더하므로 같은 계획에 역변환을 기록한다.
                anchor_y: -offset,
                stored_host_origin: None,
                stored_successor_line_origin: None,
                table_left: None,
                table_top: 0.0,
                occupied_bottom: picture_height,
            });
        }
        if i64::from(line.vertical_pos)
            != i64::from(previous.vertical_pos)
                + i64::from(previous.line_height)
                + i64::from(previous.line_spacing)
        {
            return None;
        }
        previous = line;
    }
    None
}

/// 나누지 않는 표의 전체 바깥 상자와 뒤 본문의 저장 재시작 원점이 일치하면
/// 표만 다음 쪽을 소유한다. 재시작 전 본문/빈 줄은 원래 쪽에 보존한다.
pub(crate) fn stored_table_next_page_placement(
    para: &Paragraph,
    following: &[Paragraph],
    table: &Table,
    styles: &super::style_resolver::ResolvedStyleSet,
    measured_height: f64,
    frame_vpos: i32,
    actual_host_flow_y: f64,
    available_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::paragraph::LineSeg;
    let [host] = para.line_segs.as_slice() else {
        return None;
    };
    let visible_host = para_has_non_whitespace_text(para);
    if !matches!(para.controls.as_slice(), [Control::Table(_)])
        || para.stored_text_partition_is_dirty()
        || host.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || host.line_height <= 0
        || table.page_break != TablePageBreak::None
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !table.common.flow_with_text
        || table.common.vert_align != VertAlign::Top
        || !matches!(
            table.common.horz_rel_to,
            HorzRelTo::Column | HorzRelTo::Para
        )
        || table.caption.is_some()
        || !measured_height.is_finite()
        || !dpi.is_finite()
        || dpi <= 0.0
        || !available_height.is_finite()
        || !actual_host_flow_y.is_finite()
        || (measured_height * 7200.0 / dpi).round() != f64::from(table.common.height)
    {
        return None;
    }
    let top = hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let frame_height_hu = i64::from(table.common.height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    let bottom = frame_height_hu as f64 * dpi / 7200.0;
    let host_y = hwpunit_to_px(host.vertical_pos.checked_sub(frame_vpos)?, dpi);
    let offset = hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
    if host_y <= 0.0
        || offset < 0.0
        || bottom > available_height
        || (!visible_host && (actual_host_flow_y - host_y).abs() > dpi / 7200.0)
        || (visible_host && actual_host_flow_y + offset + bottom <= available_height)
        || host_y + offset + bottom <= available_height
    {
        return None;
    }
    let mut previous = host;
    let mut previous_para = para;
    for (next_index, next) in following.iter().enumerate() {
        if next.stored_text_partition_is_dirty() || next.line_segs.is_empty() {
            return None;
        }
        for (line_index, line) in next.line_segs.iter().enumerate() {
            if line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0 || line.line_height <= 0 {
                return None;
            }
            if line.vertical_pos < previous.vertical_pos {
                // 문서 끝의 빈 줄도 표 아래에서 재시작하는 저장 프레임이다.
                // 뒤에 다른 개체나 본문이 있으면 그 본문의 직접 재시작만 수용한다.
                let terminal_blank_tail = following[next_index..].iter().all(|p| {
                    !para_has_non_whitespace_text(p)
                        && p.controls.is_empty()
                        && !p.stored_text_partition_is_dirty()
                        && !p.line_segs.is_empty()
                        && p.line_segs.iter().all(|s| {
                            s.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0 && s.line_height > 0
                        })
                });
                if i64::from(line.vertical_pos) != frame_height_hu
                    || !(para_has_non_whitespace_text(next) || terminal_blank_tail)
                {
                    return None;
                }
                return Some(ParagraphFloatPlacement {
                    flow: ParagraphFloatFlow::NextLine,
                    anchor_y: 0.0,
                    stored_host_origin: None,
                    stored_successor_line_origin: None,
                    table_left: None,
                    table_top: top,
                    occupied_bottom: bottom,
                });
            }
            // 문단 경계의 저장 사다리는 줄간격 외에 앞뒤 문단 간격도 소비한다.
            // 조판과 같은 해석된 스타일을 사용하며 문단 안쪽 줄에는 더하지 않는다.
            let paragraph_gap_hu = if line_index == 0 {
                let after = styles
                    .para_styles
                    .get(previous_para.para_shape_id as usize)?
                    .spacing_after;
                let before = styles
                    .para_styles
                    .get(next.para_shape_id as usize)?
                    .spacing_before;
                i64::from(super::px_to_hwpunit(after + before, dpi))
            } else {
                0
            };
            if i64::from(line.vertical_pos)
                != i64::from(previous.vertical_pos)
                    + i64::from(previous.line_height)
                    + i64::from(previous.line_spacing)
                    + paragraph_gap_hu
            {
                return None;
            }
            previous = line;
        }
        previous_para = next;
    }
    None
}

/// A saved CellBreak table can store its first fragment height in common.height.
/// Accept that boundary only when BOTH fragments close in source units: a whole
/// row prefix equals the frame, and the remaining rows plus repeated header end
/// at the following paragraph's reset LINESEG. This is not a page-height clamp.
pub fn stored_cellbreak_fragment_row_end(
    table: &Table,
    host: &Paragraph,
    successor: &Paragraph,
) -> Option<usize> {
    let [line] = host.line_segs.as_slice() else {
        return None;
    };
    let next = successor.line_segs.first()?;
    if table.page_break != TablePageBreak::CellBreak
        || table.common.treat_as_char
        || table.common.text_wrap != TextWrap::TopAndBottom
        || table.common.vert_rel_to != VertRelTo::Para
        || table.caption.is_some()
        || table.cell_spacing != 0
        || table.outer_margin_top != 0
        || table.outer_margin_bottom != 0
        || host.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
        || line.segment_width != 0
        || line.tag & 0x8000_0000 != 0
        || next.tag & 0x8000_0000 != 0
        || next.vertical_pos <= 0
        || next.vertical_pos >= line.vertical_pos
        || !successor.controls.is_empty()
    {
        return None;
    }
    let heights = (0..table.row_count)
        .map(|row| {
            table
                .cells
                .iter()
                .filter(|cell| cell.row == row && cell.row_span == 1)
                .map(|cell| i64::from(cell.height))
                .max()
                .filter(|h| *h > 0 && *h <= i64::from(i32::MAX))
        })
        .collect::<Option<Vec<_>>>()?;
    let mut prefix = 0_i64;
    let end = heights.iter().position(|height| {
        prefix += height;
        prefix == i64::from(table.common.height)
    })? + 1;
    if end >= heights.len()
        || table.cells.iter().any(|cell| {
            usize::from(cell.row) < end && usize::from(cell.row) + usize::from(cell.row_span) > end
        })
    {
        return None;
    }
    let header_height: i64 = if table.repeat_header {
        table
            .leading_header_rows()
            .iter()
            .map(|&row| heights[row])
            .sum()
    } else {
        0
    };
    let tail: i64 = heights[end..].iter().sum::<i64>() + header_height;
    (tail == i64::from(next.vertical_pos)).then_some(end)
}

impl ParagraphFloatPlacement {
    /// Close a paragraph only after its text and logically trailing objects have
    /// been placed. The object reservation already includes its outer margins;
    /// paragraph spacing-after belongs after the resulting occupied line box.
    /// Repeated consumers must not add that reservation a second time.
    pub fn paragraph_end(self, text_flow_end: f64, spacing_after: f64) -> f64 {
        match self.flow {
            ParagraphFloatFlow::Exclusion => text_flow_end,
            ParagraphFloatFlow::NextLine => text_flow_end.max(self.occupied_bottom + spacing_after),
            ParagraphFloatFlow::StoredPicture { next_flow_y } => next_flow_y,
        }
    }

    /// Only a measured inline-space miss may turn a geometric reservation into
    /// paragraph flow. Unknown host widths keep the existing exclusion behavior.
    pub fn with_tail_line_space(
        mut self,
        remaining_width: Option<f64>,
        table: &Table,
        dpi: f64,
    ) -> Self {
        let width = hwpunit_to_px(table.common.width as i32, dpi)
            + hwpunit_to_px(i32::from(table.outer_margin_left), dpi)
            + hwpunit_to_px(i32::from(table.outer_margin_right), dpi);
        self.flow = if remaining_width.is_some_and(|remaining| {
            dpi.is_finite()
                && dpi > 0.0
                && remaining.is_finite()
                && width.is_finite()
                && width > 0.0
                && width > remaining
        }) {
            ParagraphFloatFlow::NextLine
        } else {
            ParagraphFloatFlow::Exclusion
        };
        self
    }

    /// First fragments use the paragraph reference before spacing-before, not
    /// the whole-object outer-margin box. Keep the text anchor unchanged, and
    /// place the border (or top caption) below the actual last host line.
    /// Call before resolving preceding-object exclusions: converting an already
    /// constrained box could move it back into an occupied band.
    pub fn for_first_fragment(
        mut self,
        table: &Table,
        applied_spacing_before: f64,
        host_line_height: f64,
        dpi: f64,
    ) -> Self {
        // 부분 조각은 전체 저장 프레임의 후속 줄 원점을 증명하지 않는다.
        self.stored_successor_line_origin = None;
        let box_tail = self.occupied_bottom - self.table_top;
        let paragraph_anchor = self.anchor_y - applied_spacing_before;
        let positioned_top =
            paragraph_anchor + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
        self.table_top = positioned_top.max(self.anchor_y + host_line_height);
        self.occupied_bottom = self.table_top + box_tail;
        self
    }

    /// Recover a stored host only when the next source row accounts for exactly
    /// this measured band. A raw vpos by itself is not a page-layout oracle.
    /// The caller supplies a continuous, unedited column source frame; no painted
    /// node or successor's rendered position participates in this decision.
    pub fn with_stored_band_origin(
        mut self,
        para: &Paragraph,
        next: &Paragraph,
        frame_vpos: i32,
        frame_origin: f64,
        dpi: f64,
    ) -> Self {
        use crate::model::paragraph::LineSeg;
        let ([host], Some(successor)) = (para.line_segs.as_slice(), next.line_segs.first()) else {
            return self;
        };
        if para.controls.len() != 1
            || para.stored_text_partition_is_dirty()
            || next.stored_text_partition_is_dirty()
            || [host, successor]
                .iter()
                .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
            || successor.vertical_pos <= host.vertical_pos
            || host.vertical_pos < frame_vpos
            || !next.controls.is_empty()
            || !frame_origin.is_finite()
            || !dpi.is_finite()
            || dpi <= 0.0
        {
            return self;
        }
        let source_advance =
            (i64::from(successor.vertical_pos) - i64::from(host.vertical_pos)) as f64 * dpi
                / 7200.0;
        let measured_advance = self.occupied_bottom - self.anchor_y;
        // One HWPUNIT permits integer source rounding, not a visual tolerance.
        if (source_advance - measured_advance).abs() > dpi / 7200.0 {
            return self;
        }
        let origin = frame_origin
            + (i64::from(host.vertical_pos) - i64::from(frame_vpos)) as f64 * dpi / 7200.0;
        if !origin.is_finite() || origin < self.anchor_y {
            return self; // Never rewind already consumed flow into an earlier band.
        }
        let shift = origin - self.anchor_y;
        self.anchor_y = origin;
        self.stored_host_origin = Some(origin);
        self.table_top += shift;
        self.occupied_bottom += shift;
        self
    }

    /// This result owns a control following text on the paragraph's last logical
    /// line, not every paragraph-relative object drawn below some stored rows.
    /// Paragraph boundaries already live in the IR; an internal hard break ends
    /// the preceding text line even when the control has the same scalar offset.
    /// Missing character mapping is not evidence of a tail attachment.
    /// 표 제어문자가 글 맨 앞(첫 줄 시작)에 있고 뒤에 보이는 글이 있을 때의 위치(0).
    fn text_head_control_position(para: &Paragraph, control_index: usize) -> Option<usize> {
        let text_len = para.text.chars().count();
        if text_len == 0 || para.char_offsets.len() != text_len {
            return None;
        }
        let position = *para.control_text_positions().get(control_index)?;
        (position == 0
            && !para.text.contains('\n')
            && para
                .text
                .chars()
                .any(|ch| !ch.is_whitespace() && !ch.is_control() && ch != '\u{FFFC}'))
        .then_some(position)
    }

    fn text_tail_control_position(para: &Paragraph, control_index: usize) -> Option<usize> {
        let text_len = para.text.chars().count();
        if text_len == 0 || para.char_offsets.len() != text_len {
            return None;
        }
        let position = *para.control_text_positions().get(control_index)?;
        (position == text_len
            && para.text.rsplit('\n').next().is_some_and(|line| {
                line.chars()
                    .any(|ch| !ch.is_whitespace() && !ch.is_control() && ch != '\u{FFFC}')
            }))
        .then_some(position)
    }

    /// Preserve the same formatted before/body/after box in whole fit and paint.
    /// No text line is inferred from an empty control-only host.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_empty_reflow_host(
        para: &Paragraph,
        table: &Table,
        origin: f64,
        stored_origin: Option<f64>,
        table_height: f64,
        before: f64,
        after: f64,
        dpi: f64,
    ) -> Option<Self> {
        // 동일 단의 실제 원점0·연속 저장 사다리를 확인한 경우만 저장 원점을 쓴다.
        // 폭0 자체는 현재 흐름과 저장/paint 원점이 같다는 증거가 아니다.
        let stored_origin = stored_origin.filter(|_| saved_whole_empty_table_anchor(para, table));
        let origin = stored_origin.unwrap_or(origin);
        if !(empty_table_host_uses_formatted_box(para, table) || stored_origin.is_some())
            || ![origin, table_height, before, after]
                .iter()
                .all(|value| value.is_finite())
            || table_height < 0.0
        {
            return None;
        }
        // A positive paragraph offset is physical space before the flow
        // object even though it has no content unit or saved host line.
        let table_top =
            origin + before + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
        Some(Self {
            flow: ParagraphFloatFlow::Exclusion,
            anchor_y: origin,
            stored_host_origin: stored_origin,
            stored_successor_line_origin: None,
            table_left: None,
            table_top,
            occupied_bottom: table_top + table_height + after,
        })
    }

    /// 저장 LineSeg 대신 현재 frame에서 계산된 줄로 앵커를 결정한다.
    /// 모든 호스트 줄이 표보다 앞서는 계약만 소유하며, 혼합 배치를 임의로
    /// 본문 뒤 배치로 바꾸지 않는다. source의 UTF-16 위치/높이는 읽지 않는다.
    pub fn from_computed_host(
        para: &Paragraph,
        table: &Table,
        control_index: usize,
        text_origin: f64,
        lines: &[ParagraphHostLine],
        table_height: f64,
        dpi: f64,
    ) -> Option<Self> {
        let char_pos = Self::text_tail_control_position(para, control_index)?;
        Self::from_computed_host_at(char_pos, para, table, text_origin, lines, table_height, dpi)
    }

    /// [#7418] 표 제어문자가 글 **앞**에 있는 host 의 계산 줄 배치.
    ///
    /// 글 끝 앵커 모델(`from_computed_host`, #6950 이 논리 순서를 지킨다)과 따로 둔다. 같은
    /// 기하 검사 — «모든 host 줄이 문단 기준 오프셋 위끝 전에 끝난다» — 를 통과할 때만 글이
    /// 표 위에 선다(한/글 자리차지: 오프셋 안에 자리가 있으면 글을 위에 둔다). 70833 pi=83
    /// (`- 규제 차등화…`, 줄 20px ≤ 오프셋 23.8)·21298295 pi=4(16 ≤ 20.2)·156403546 pi=22
    /// (20 ≤ 26.4)가 정본에서 표 위 제목이다. 오프셋이 줄보다 작으면(pr-1674: 0) None — 글은
    /// 표 아래로 간다.
    pub fn from_computed_head_host(
        para: &Paragraph,
        table: &Table,
        control_index: usize,
        text_origin: f64,
        lines: &[ParagraphHostLine],
        table_height: f64,
        dpi: f64,
    ) -> Option<Self> {
        let char_pos = Self::text_head_control_position(para, control_index)?;
        Self::from_computed_host_at(char_pos, para, table, text_origin, lines, table_height, dpi)
    }

    fn from_computed_host_at(
        char_pos: usize,
        para: &Paragraph,
        table: &Table,
        text_origin: f64,
        lines: &[ParagraphHostLine],
        table_height: f64,
        dpi: f64,
    ) -> Option<Self> {
        if !dpi.is_finite()
            || dpi <= 0.0
            || !table_height.is_finite()
            || table_height < 0.0
            || !is_para_topbottom_float(&table.common)
            || !matches!(table.common.vert_align, VertAlign::Top)
            || signed_hwpunit(table.common.vertical_offset) <= 0
            || lines
                .first()
                .is_none_or(|line| line.char_start != 0 || line.top != 0.0)
        {
            return None;
        }
        let text_len = para.text.chars().count();
        if lines.iter().any(|line| {
            line.char_start > text_len
                || !line.top.is_finite()
                || !line.height.is_finite()
                || line.height < 0.0
        }) || lines
            .windows(2)
            .any(|pair| pair[1].char_start < pair[0].char_start || pair[1].top < pair[0].top)
        {
            return None;
        }
        let anchor = lines
            .iter()
            .rev()
            .find(|line| line.char_start <= char_pos)?;
        let offset_top =
            anchor.top + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
        if lines.iter().any(|line| line.top + line.height > offset_top) {
            return None;
        }
        let anchor_y = text_origin + anchor.top;
        let table_top =
            text_origin + offset_top + hwpunit_to_px(table.outer_margin_top as i32, dpi);
        let occupied_bottom =
            table_top + table_height + hwpunit_to_px(table.outer_margin_bottom as i32, dpi);
        [anchor_y, table_top, occupied_bottom]
            .iter()
            .all(|v| v.is_finite())
            .then_some(Self {
                flow: ParagraphFloatFlow::Exclusion,
                anchor_y,
                // 재조판 글줄도 표 예약에 쓴 원점에서 생성한다. 출력 단계가
                // 저장 vpos로 다시 원점을 추측하면 호스트와 표가 서로 갈라진다.
                stored_host_origin: Some(text_origin),
                stored_successor_line_origin: None,
                table_left: None,
                table_top,
                occupied_bottom,
            })
    }

    /// 선행 개체의 점유 구간을 지나도록 표 상자만 전진시킨다.
    /// 앵커 줄은 움직이지 않으며, 정렬된 구간을 한 번씩만 방문한다.
    pub fn clear_occupied_bands(mut self, bands: impl IntoIterator<Item = Range<f64>>) -> Self {
        let mut bands: Vec<_> = bands
            .into_iter()
            .filter(|band| band.start.is_finite() && band.end.is_finite() && band.start < band.end)
            .collect();
        bands.sort_by(|a, b| a.start.total_cmp(&b.start));
        for band in bands {
            if self.table_top < band.end && self.occupied_bottom > band.start {
                let shift = band.end - self.table_top;
                self.table_top = band.end;
                self.occupied_bottom += shift;
            }
        }
        self
    }

    /// 저장 줄이 호스트 텍스트 전부를 표 앞에 배치하는 계약일 때 사용한다.
    /// `text_origin`은 spacing-before가 반영된 첫 텍스트 줄의 단 상대 원점이다.
    /// 원본이 아닌 합성/재조판 줄은 저장 좌표의 증거로 사용하지 않는다.
    pub fn from_stored_host(
        para: &Paragraph,
        table: &Table,
        control_index: usize,
        text_origin: f64,
        table_height: f64,
        dpi: f64,
    ) -> Option<Self> {
        Self::text_tail_control_position(para, control_index)?;
        Self::from_stored_host_at(para, table, control_index, text_origin, table_height, dpi)
    }

    /// 글 앞 개체도 저장 호스트 줄이 양수 오프셋 안에서 모두 끝나면
    /// 같은 앵커 상자를 예약한다. 계산 줄과 저장 줄의 제어문자 소유를 구분한다.
    /// 원점은 글줄 앞 간격을 더하기 전의 문단 시작이다.
    pub fn from_stored_head_host(
        para: &Paragraph,
        table: &Table,
        control_index: usize,
        text_origin: f64,
        table_height: f64,
        dpi: f64,
    ) -> Option<Self> {
        Self::text_head_control_position(para, control_index)?;
        Self::from_stored_host_at(para, table, control_index, text_origin, table_height, dpi)
    }

    /// 저장 글 앞 제어문자와 선행 호스트 줄이 동일한 앵커 계약에 속하는지 확인한다.
    pub(crate) fn stored_head_host_lines_are_valid(
        para: &Paragraph,
        table: &Table,
        control_index: usize,
    ) -> bool {
        Self::text_head_control_position(para, control_index).is_some()
            && Self::stored_host_lines_are_valid(para, table, control_index)
    }

    fn from_stored_host_at(
        para: &Paragraph,
        table: &Table,
        control_index: usize,
        text_origin: f64,
        table_height: f64,
        dpi: f64,
    ) -> Option<Self> {
        if !dpi.is_finite()
            || dpi <= 0.0
            || !table_height.is_finite()
            || table_height < 0.0
            || !is_para_topbottom_float(&table.common)
            || !matches!(table.common.vert_align, VertAlign::Top)
            || !Self::stored_host_lines_are_valid(para, table, control_index)
        {
            return None;
        }
        let anchor_y = text_origin
            + super::layout::stored_float_anchor_offset_px(para, table, control_index, dpi);
        let table_top = anchor_y
            + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi)
            + hwpunit_to_px(table.outer_margin_top as i32, dpi);
        let occupied_bottom =
            table_top + table_height + hwpunit_to_px(table.outer_margin_bottom as i32, dpi);
        [anchor_y, table_top, occupied_bottom]
            .iter()
            .all(|v| v.is_finite())
            .then_some(Self {
                flow: ParagraphFloatFlow::Exclusion,
                anchor_y,
                stored_host_origin: None,
                stored_successor_line_origin: None,
                table_left: None,
                table_top,
                occupied_bottom,
            })
    }

    /// 저장 앵커의 줄 소유를 판정하는 공통 근거. 개체 바깥여백도 이 판정을 소비한다.
    fn stored_host_lines_are_valid(para: &Paragraph, table: &Table, control_index: usize) -> bool {
        is_para_topbottom_float(&table.common)
            && table.common.vert_align == VertAlign::Top
            && super::layout::stored_host_lines_precede_float(para, table, control_index)
            && !para.stored_text_partition_is_dirty()
            && !para.line_segs.is_empty()
            && para.line_segs.iter().all(|line| {
                line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
            && para.line_segs.windows(2).all(|pair| {
                pair[1].vertical_pos >= pair[0].vertical_pos
                    && pair[1].text_start >= pair[0].text_start
            })
    }
}

/// 개체 배치에 쓰이는 실제 좌표계. Paper와 Page(본문 영역)를 구분하며,
/// 셀/문단의 container를 종이나 단으로 대체하지 않는다.
///
/// #6812: 점유 영역도 paint와 같은 원점·정렬 해석을 사용할 수 있도록
/// 렌더 노드 생성과 무관한 계산으로 분리한다. 크기는 호출자가 캡션과
/// 바깥 여백까지 포함해 해석한 개체 상자의 크기다.
pub(crate) struct ObjectPlacementFrame<'a> {
    pub(crate) container: &'a LayoutRect,
    pub(crate) column: &'a LayoutRect,
    pub(crate) body: &'a LayoutRect,
    pub(crate) paper: &'a LayoutRect,
    pub(crate) paragraph_y: f64,
    pub(crate) alignment: Alignment,
    pub(crate) dpi: f64,
}

impl ObjectPlacementFrame<'_> {
    /// 기준 영역 → 정렬 → signed offset. 저장 LineSeg나 paint 목록은 읽지 않는다.
    pub(crate) fn position(&self, common: &CommonObjAttr, width: f64, height: f64) -> (f64, f64) {
        let h_offset = hwpunit_to_px(signed_hwpunit(common.horizontal_offset), self.dpi);
        let v_offset = hwpunit_to_px(signed_hwpunit(common.vertical_offset), self.dpi);
        let x = if common.treat_as_char {
            match self.alignment {
                Alignment::Center | Alignment::Distribute => {
                    self.container.x + (self.container.width - width).max(0.0) / 2.0
                }
                Alignment::Right => self.container.x + (self.container.width - width).max(0.0),
                _ => self.container.x,
            }
        } else {
            let reference = match common.horz_rel_to {
                HorzRelTo::Paper => self.paper,
                HorzRelTo::Page => self.body,
                HorzRelTo::Column => self.column,
                HorzRelTo::Para => self.container,
            };
            match common.horz_align {
                HorzAlign::Left | HorzAlign::Inside => reference.x + h_offset,
                HorzAlign::Center => reference.x + (reference.width - width) / 2.0 + h_offset,
                HorzAlign::Right | HorzAlign::Outside => {
                    reference.x + reference.width - width - h_offset
                }
            }
        };
        let y = if common.treat_as_char {
            self.paragraph_y
        } else {
            let (reference_y, reference_height) = match common.vert_rel_to {
                VertRelTo::Paper => (self.paper.y, self.paper.height),
                VertRelTo::Page => (self.body.y, self.body.height),
                VertRelTo::Para => (self.paragraph_y, self.container.height),
            };
            match common.vert_align {
                VertAlign::Top | VertAlign::Inside => reference_y + v_offset,
                VertAlign::Center => reference_y + (reference_height - height) / 2.0 + v_offset,
                VertAlign::Bottom | VertAlign::Outside => {
                    reference_y + reference_height - height - v_offset
                }
            }
        };
        (x, y)
    }

    /// 실제 출력과 같은 여백·캡션 포함 상자. 흐름에 영향을 주는 Square 그림만 등록한다.
    /// `allow_overlap`은 floating 개체 간 허용이며 TAC 줄의 어울림을 취소하지 않는다.
    pub(crate) fn picture_exclusion(&self, picture: &Picture) -> Option<FrameExclusion> {
        let common = &picture.common;
        if common.treat_as_char || common.text_wrap != TextWrap::Square {
            return None;
        }
        let (width, height) = picture_flow_frame_size_hu(picture);
        if width <= 0 || height <= 0 {
            return None;
        }
        let mut width = hwpunit_to_px(width, self.dpi);
        let mut height = hwpunit_to_px(height, self.dpi);
        if let Some(caption) = &picture.caption {
            use crate::model::shape::CaptionDirection;
            let spacing = hwpunit_to_px(caption.spacing as i32, self.dpi);
            match caption.direction {
                CaptionDirection::Top | CaptionDirection::Bottom => {
                    height +=
                        super::composer::caption_height_px(&picture.caption, self.dpi) + spacing;
                }
                CaptionDirection::Left | CaptionDirection::Right => {
                    width += hwpunit_to_px(caption.width as i32, self.dpi) + spacing;
                }
            }
        }
        width += hwpunit_to_px(
            i32::from(common.margin.left) + i32::from(common.margin.right),
            self.dpi,
        );
        height += hwpunit_to_px(
            i32::from(common.margin.top) + i32::from(common.margin.bottom),
            self.dpi,
        );
        let (x, mut y) = self.position(common, width, height);
        if common.flow_with_text && common.vert_rel_to == VertRelTo::Para {
            y = y.min((self.column.y + self.column.height - height).max(self.column.y));
        }
        if width <= 0.0
            || height <= 0.0
            || ![x, y, width, height].iter().all(|value| value.is_finite())
        {
            return None;
        }
        let hu = |px| super::px_to_hwpunit(px, self.dpi);
        Some(FrameExclusion {
            horizontal: hu(x)..hu(x + width),
            vertical: hu(y)..hu(y + height),
            policy: match common.text_flow {
                TextFlow::BothSides => FrameExclusionPolicy::BothSides,
                TextFlow::LargestOnly => FrameExclusionPolicy::LargestSide,
                TextFlow::LeftOnly => FrameExclusionPolicy::LeftSide,
                TextFlow::RightOnly => FrameExclusionPolicy::RightSide,
            },
        })
    }
}

/// 분할기가 확정한 TAC 줄의 배치. x/y는 해당 단 원점 기준(px), 여백 포함 pen 좌표다.
/// 렌더는 이 결과를 소비하며 별도의 그림 회피 판정을 반복하지 않는다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InlineBoxPlacement {
    pub x: f64,
    pub y: f64,
    pub clearance: f64,
    /// 저장 TAC 줄이 확정한 다음 흐름 위치(단 상대). 이 경우 x는 기존 표 정렬이 담당한다.
    pub advance_end: Option<f64>,
}

/// 가로 구간에서 원자적 inline 상자가 들어갈 첫 줄을 찾는다.
/// 그림 경계에서만 이동하는 LayoutFrame을 사용하며 저장 vpos는 입력받지 않는다.
/// 원래 프레임보다 넓은 상자는 축소하지 않고, 배제 영역이 없는 전폭 줄까지 기다린다.
pub(crate) fn place_inline_box(
    horizontal: Range<f64>,
    top: f64,
    width: f64,
    height: f64,
    alignment: Alignment,
    exclusions: &[FrameExclusion],
    dpi: f64,
) -> Option<InlineBoxPlacement> {
    if exclusions.is_empty()
        || ![horizontal.start, horizontal.end, top, width, height, dpi]
            .iter()
            .all(|v| v.is_finite())
        || horizontal.end <= horizontal.start
        || width <= 0.0
        || height <= 0.0
        || dpi <= 0.0
    {
        return None;
    }
    let hu = |px| super::px_to_hwpunit(px, dpi);
    let base = hu(horizontal.start)..hu(horizontal.end);
    let base_width = base
        .end
        .checked_sub(base.start)
        .filter(|width| *width > 0)?;
    let start = hu(top);
    let band_height = hu(height).max(1);
    // 다른 세로/가로 영역의 그림 때문에 기존 정렬·leading 경로를 대체하지 않는다.
    if !exclusions.iter().any(|e| {
        e.horizontal.start < base.end
            && base.start < e.horizontal.end
            && e.vertical.start < start.saturating_add(band_height)
            && start < e.vertical.end
    }) {
        return None;
    }
    let mut frame = LayoutFrame::new(base.clone(), start, exclusions.to_vec());
    frame.minimum_width = hu(width).max(1).min(base_width);
    let intervals = frame.carve(band_height);
    let lane = match alignment {
        Alignment::Right => intervals.last(),
        _ => intervals.first(),
    }?;
    let left = hwpunit_to_px(lane.start, dpi);
    let right = hwpunit_to_px(lane.end, dpi);
    let x = match alignment {
        Alignment::Right => (right - width).max(left),
        Alignment::Center => (left + (right - left - width) / 2.0).max(left),
        _ => left,
    };
    let y = hwpunit_to_px(frame.top, dpi).max(top);
    Some(InlineBoxPlacement {
        x,
        y,
        clearance: y - top,
        advance_end: None,
    })
}

/// A paper/page-anchored side-wrap float that can explain a stored body row's
/// missing right-side width.
///
/// Width alone is deliberately insufficient: a same-width object on another
/// vertical band must not make an unrelated paragraph keep stale narrow rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FloatCarveEvidence {
    pub(crate) width: i32,
    pub(crate) vertical: Range<i32>,
}

impl FloatCarveEvidence {
    /// A stored row may use this evidence only when its missing width matches
    /// and at least one of the paragraph's stored vertical bands intersects the
    /// float. A same-width object elsewhere is not an exclusion provenance.
    pub(crate) fn matches_stored_rows(
        &self,
        missing_width: i32,
        rows: &[crate::model::paragraph::LineSeg],
        tolerance: i32,
    ) -> bool {
        (missing_width - self.width).abs() <= tolerance
            && rows.iter().any(|segment| {
                let top = segment.vertical_pos;
                let bottom = top.saturating_add(segment.line_height.max(1));
                top < self.vertical.end && self.vertical.start < bottom
            })
    }
}

/// Interpret an HWPUNIT value that may have been stored through a signed field.
pub(crate) fn signed_hwpunit(value: HwpUnit) -> i32 {
    value as i32
}

/// [#7008] 문단의 중첩 표가 **어느 줄에 속하는가** — `para.controls` 와 같은 길이로,
/// 표가 아닌 컨트롤 자리는 `None`.
///
/// 줄 소속을 추측하지 않는다. 저장 `LINE_SEG` 사다리가 있으면 **그 소속을 보존**하고
/// (컨트롤 시작과 줄 시작을 같은 원시 UTF-16 축으로 정규화한다),
/// 사다리가 없는 문단은 `None` 을 돌려 호출자가 재조판 경로로 가게 한다.
///
/// 이 매핑이 있으면 문단 높이는 "줄별 점유 높이의 합" 으로 계산된다 — 같은 줄의 표를
/// 세로로 합산하지 않고, 여러 줄이면 각 줄이 제 몫을 낸다.
pub(crate) fn para_nested_table_line_indices(para: &Paragraph) -> Option<Vec<Option<usize>>> {
    let lines = stored_control_line_indices(para)?;
    Some(
        para.controls
            .iter()
            .zip(lines)
            .map(|(ctrl, line)| matches!(ctrl, Control::Table(_)).then_some(line))
            .collect(),
    )
}

/// 저장 줄과 컨트롤은 모두 PARA_TEXT UTF-16 축에서 비교한다.
/// visible character 위치로 투영하면 한 갭 안의 서로 다른 컨트롤 시작이 소실된다.
pub(crate) fn stored_control_line_indices(para: &Paragraph) -> Option<Vec<usize>> {
    if para.line_segs.is_empty() || crate::renderer::para_has_no_stored_line_segs(para) {
        return None;
    }
    Some(
        para.control_utf16_positions()
            .into_iter()
            .map(|pos| {
                (0..para.line_segs.len())
                    .rfind(|&li| para.line_seg_text_start(li) <= pos)
                    .unwrap_or(0)
            })
            .collect(),
    )
}

/// 측정과 셀 배치가 함께 소비하는 중첩 표의 저장 줄 그룹.
/// 저장 줄이 없으면 TAC의 같은 줄 소속을 가정하지 않고 종전 적층 폴백을 유지한다.
pub(crate) struct NestedTableGroup {
    pub(crate) line: Option<usize>,
    pub(crate) controls: Vec<usize>,
    pub(crate) side_by_side: bool,
}

/// 저장 HWPX의 글 뒤로 표는 문단 앵커를 공유하는 배경이며 흐름을 전진하지 않는다.
/// 가로 기준이 단/문단인 경우 모두 같은 역할이다. TAC와 HWP5 기존 경로는 보존한다.
pub(crate) fn nested_table_is_hwpx_overlay(table: &Table, hwpx_stored: bool) -> bool {
    hwpx_stored
        && !table.common.treat_as_char
        && table.common.text_wrap == TextWrap::BehindText
        && table.common.flow_with_text
        && table.common.vert_rel_to == VertRelTo::Para
        && matches!(
            table.common.horz_rel_to,
            HorzRelTo::Column | HorzRelTo::Para
        )
}

/// 흐름 표는 나란함/적층을 따르지만 배경의 시각 높이는 같은 앵커에서 최댓값으로 합친다.
/// 측정과 셀 정렬이 같은 결과를 소비하며 배경을 뒤에 놓인 새 흐름으로 더하지 않는다.
pub(crate) fn nested_group_occupied_height(
    para: &Paragraph,
    group: &NestedTableGroup,
    heights: &[f64],
    hwpx_stored: bool,
) -> f64 {
    let mut flow_height = 0.0_f64;
    let mut overlay_height = 0.0_f64;
    for &ci in &group.controls {
        let Control::Table(table) = &para.controls[ci] else {
            continue;
        };
        if nested_table_is_hwpx_overlay(table, hwpx_stored) {
            overlay_height = overlay_height.max(heights[ci]);
        } else if group.side_by_side {
            flow_height = flow_height.max(heights[ci]);
        } else {
            flow_height += heights[ci];
        }
    }
    flow_height.max(overlay_height)
}

pub(crate) fn nested_table_groups(para: &Paragraph, hwpx_stored: bool) -> Vec<NestedTableGroup> {
    let lines = para_nested_table_line_indices(para);
    let mut groups: Vec<NestedTableGroup> = Vec::new();
    for (ci, ctrl) in para.controls.iter().enumerate() {
        if !matches!(ctrl, Control::Table(_)) {
            continue;
        }
        let line = lines.as_ref().and_then(|v| v[ci]);
        if let Some(group) = groups.iter_mut().find(|g| g.line == line) {
            group.controls.push(ci);
        } else {
            groups.push(NestedTableGroup {
                line,
                controls: vec![ci],
                side_by_side: false,
            });
        }
    }
    for group in &mut groups {
        let tables: Vec<&Table> = group
            .controls
            .iter()
            .filter_map(|&ci| match &para.controls[ci] {
                Control::Table(t) if !nested_table_is_hwpx_overlay(t, hwpx_stored) => {
                    Some(t.as_ref())
                }
                _ => None,
            })
            .collect();
        group.side_by_side = if group.line.is_some() {
            line_nested_table_group_is_side_by_side(&tables)
        } else {
            para_float_group_is_side_by_side(para)
        };
    }
    groups
}

/// [#6787] 한 **줄**에 놓인 중첩 표들이 나란히 놓이는 무리인가 — 즉 그 줄이 예약해야 할
/// 높이가 합이 아니라 **최댓값**인가.
///
/// 나란히 놓이는 길은 둘이다.
///
/// - **글줄 안 순서**(`#7008`) — 글자처럼 취급되는 표는 글줄 흐름에 참여하므로, 같은
///   줄에 있다는 것 자체가 나란함이다. 오프셋을 쓰지 않는다.
/// - **가로 오프셋**(`#6787`) — 문단-기준(`VertRelTo::Para`) 자리차지(`TopAndBottom`)
///   비-TAC 표들이 각자 `horzOffset` 을 갖고 **가로로 겹치지 않으면** 한/글은 같은 y 에
///   놓는다(`#6494` 의 칸 안 짝). 하나라도 조건을 벗어나면 세로 적층으로 본다.
///
/// ⭐ **측정(`height_measurer`)과 배치(`table_layout`)가 이 하나의 판정을 함께 쓴다** —
/// 두 축이 문자 그대로 같은 함수를 부르므로 발동 조건이 갈리는 일이 구조적으로 없다.
/// (그 비대칭이 `#6787` 의 실제 결함이었다.)
pub(crate) fn line_nested_table_group_is_side_by_side(tables: &[&Table]) -> bool {
    if tables.len() < 2 {
        return false;
    }
    // 글자처럼 취급되는 표들은 같은 줄에 있다는 사실만으로 나란하다.
    if tables.iter().all(|table| table.common.treat_as_char) {
        return true;
    }
    let mut spans: Vec<(i32, i32)> = Vec::new();
    for table in tables {
        if !para_float_group_member_is_eligible(table) {
            return false;
        }
        let start = signed_hwpunit(table.common.horizontal_offset);
        let width = table.common.width.min(i32::MAX as u32) as i32;
        spans.push((start, start.saturating_add(width)));
    }
    spans.sort_unstable();
    // 가로 구간이 하나라도 겹치면 나란히 놓을 수 없다.
    spans.windows(2).all(|w| w[0].1 <= w[1].0 + 1)
}

/// 저장 사다리가 없어 줄 소속을 모르는 문단의 폴백 — 문단 전체를 한 무리로 본다.
pub(crate) fn para_float_group_is_side_by_side(para: &Paragraph) -> bool {
    let tables: Vec<&Table> = para
        .controls
        .iter()
        .filter_map(|ctrl| match ctrl {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .collect();
    tables.iter().all(|t| !t.common.treat_as_char)
        && line_nested_table_group_is_side_by_side(&tables)
}

/// 나란히 무리의 자격 — 무리 판정과 레인 배치가 **같은 술어**를 쓴다.
pub(crate) fn para_float_group_member_is_eligible(table: &Table) -> bool {
    !table.common.treat_as_char
        && matches!(table.common.text_wrap, TextWrap::TopAndBottom)
        && matches!(table.common.vert_rel_to, VertRelTo::Para)
        && matches!(table.common.horz_rel_to, HorzRelTo::Para)
        && signed_hwpunit(table.common.horizontal_offset) >= 0
}

/// Resolve the deliberately small Picture/Square side-wrap subset used by a
/// caller-owned `LayoutFrame`.
///
/// The caller owns both the paragraph-relative anchor and the exclusion's
/// lifetime. This function intentionally has no fallback policy: only an
/// uncaptained, non-TAC Picture with `Square` and the two recovered side-wrap
/// flows has a physical-row representation. Every other object shape remains
/// with its existing owner.
/// [#6175] 용지/쪽 기준 어울림 개체의 흐름 폭과 세로 band(HWPUNIT, 바깥 여백 포함).
///
/// `stored_rows_require_external_geometry` 가 저장 행의 결손 폭과 같은 **세로 band**의
/// 이 값을 함께 대조해, 균일하게 좁은 저장 행의 좁음이 문단 자신의 테두리 inset에서
/// 온 것인지 외부 개체에서 온 것인지 가른다. 셀에서는 #5818 이 같은 혼동을 같은 셀의
/// Square float 실재로 갈랐고, 이것은 그 계약의 본문 판이다.
///
/// 문단 기준 개체는 `resolve_picture_exclusion`의 caller-owned frame이 직접 소유한다.
/// 여기서는 그 frame이 지원하지 않는 용지/쪽 기준 개체만 수집한다.
pub(crate) fn paper_or_page_float_carve_evidence(
    paragraphs: &[crate::model::paragraph::Paragraph],
) -> Vec<FloatCarveEvidence> {
    use crate::model::control::Control;
    let mut evidence = Vec::new();
    for para in paragraphs {
        for control in &para.controls {
            let common = match control {
                Control::Picture(picture) => &picture.common,
                Control::Shape(shape) => shape.common(),
                Control::Table(table) => &table.common,
                _ => continue,
            };
            if common.treat_as_char
                || !matches!(
                    common.text_wrap,
                    TextWrap::Square | TextWrap::Tight | TextWrap::Through
                )
            {
                continue;
            }
            if !matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
                || !matches!(common.vert_align, VertAlign::Top | VertAlign::Inside)
            {
                continue;
            }
            let width = (common.width as i32)
                .saturating_add(i32::from(common.margin.left))
                .saturating_add(i32::from(common.margin.right));
            let height = (common.height as i32)
                .saturating_add(i32::from(common.margin.top))
                .saturating_add(i32::from(common.margin.bottom));
            let top = signed_hwpunit(common.vertical_offset);
            let vertical = top..top.saturating_add(height);
            let candidate = FloatCarveEvidence { width, vertical };
            if width > 0 && height > 0 && !evidence.contains(&candidate) {
                evidence.push(candidate);
            }
        }
    }
    evidence
}

/// [#6202] 용지 기준(`Paper`/`Page`) 어울림 개체를 재는 데 필요한 쪽 원점.
///
/// `Column`/`Para` 기준은 문단 밴드 안에서 끝나지만, 용지 기준은 본문 상자가 용지 안
/// 어디에 있는지를 알아야 한다. 그 값 **두 개**면 공식은 같다 — 기준점만 바뀐다.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PaperOrigin {
    /// 본문 상자의 용지 기준 왼쪽 (HWPUNIT).
    pub(crate) body_left: i32,
    /// 본문 상자의 용지 기준 위쪽 (HWPUNIT).
    pub(crate) body_top: i32,
    /// 이 밴드가 시작하는 문단의 **본문 절대** 세로 위치 (HWPUNIT).
    ///
    /// `Para` 기준은 `paragraph_top`(밴드-로컬)에 오프셋을 더하면 되지만, 용지 기준은
    /// 절대 좌표로 나오므로 밴드-로컬로 옮겨야 한다. 두 좌표계를 섞으면 밴드 끝이
    /// 어긋나 배제 루프가 밴드 밖 문단까지 훑는다(#6202 실측: pi=8 까지 가서 bail).
    pub(crate) band_abs_top: i32,
}

pub(crate) fn resolve_picture_exclusion(
    picture: &Picture,
    column_horizontal: Range<i32>,
    paragraph_horizontal: Range<i32>,
    paragraph_top: i32,
    paper: PaperOrigin,
) -> Option<FrameExclusion> {
    let common = &picture.common;
    // [#6202] 용지 기준 개체도 잰다. 코퍼스 1,997건 표본에서 Square float 개체를
    // 막는 관문은 사실상 `horz_rel`/`vert_rel` 의 `Paper|Page` 뿐이었다(각 57건).
    //
    // 156483689 실측 — 계산이 저장 사다리를 **1 HU 오차로 재현**한다:
    //
    // ```text
    //   그림  w=12482 h=9366  vert=Paper(36844) horz=Paper(42333)
    //   본문  왼쪽 5670 HU · 폭 48188 HU
    //   가로  깎인 줄 오른끝 5670+36664 = 42334   vs  그림 왼쪽 42333   (1 HU)
    //   세로  밴드 36844−5670 .. +9366 = 31174..40540
    //         pi=5 29631..32031 깎임 · pi=6 34431 깎임 · pi=7 에서 40540 지나 전폭 복귀
    // ```
    let paper_relative_horz = matches!(common.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page);
    let paper_relative_vert = matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page);
    if common.treat_as_char
        // [#6202] 캡션이 붙은 개체도 잰다 — 156483689 실측에서 밴드 높이는 캡션을 뺀
        // 그림 높이(9366 HU) 그대로이고, 저장 사다리의 깎임 끝(40540 HU)과 일치한다.
        || common.text_wrap != TextWrap::Square
        || !matches!(
            common.horz_rel_to,
            HorzRelTo::Column | HorzRelTo::Para | HorzRelTo::Paper | HorzRelTo::Page
        )
        || !matches!(
            common.vert_rel_to,
            VertRelTo::Para | VertRelTo::Paper | VertRelTo::Page
        )
        || !matches!(common.vert_align, VertAlign::Top | VertAlign::Inside)
    {
        return None;
    }
    let policy = match common.text_flow {
        TextFlow::BothSides => FrameExclusionPolicy::BothSides,
        TextFlow::LargestOnly => FrameExclusionPolicy::LargestSide,
        TextFlow::LeftOnly | TextFlow::RightOnly => return None,
    };

    let (width, height) = picture_flow_frame_size_hu(picture);
    if column_horizontal.is_empty() || paragraph_horizontal.is_empty() || width <= 0 || height <= 0
    {
        return None;
    }

    let horizontal_offset = signed_hwpunit(common.horizontal_offset);
    let column_end = column_horizontal.end;
    let reference = match common.horz_rel_to {
        HorzRelTo::Column => column_horizontal,
        HorzRelTo::Para => paragraph_horizontal,
        // 용지 기준은 본문 왼쪽을 빼 컬럼 좌표로 옮긴다 — 이후 공식은 동일하다.
        HorzRelTo::Paper | HorzRelTo::Page => {
            (-paper.body_left)..(column_end.saturating_sub(paper.body_left))
        }
    };
    let reference_width = reference.end.saturating_sub(reference.start);
    let visible_left = match common.horz_align {
        HorzAlign::Left | HorzAlign::Inside => reference.start.saturating_add(horizontal_offset),
        HorzAlign::Center => reference
            .start
            .saturating_add(
                reference_width
                    .saturating_sub(width)
                    .max(0)
                    .saturating_div(2),
            )
            .saturating_add(horizontal_offset),
        HorzAlign::Right | HorzAlign::Outside => reference
            .end
            .saturating_sub(width)
            .saturating_sub(horizontal_offset),
    };
    let visible_top = if paper_relative_vert {
        // 용지 기준 세로 = (용지 오프셋 − 본문 위쪽) − 밴드 시작 문단의 절대 위치.
        signed_hwpunit(common.vertical_offset)
            .saturating_sub(paper.body_top)
            .saturating_sub(paper.band_abs_top)
    } else {
        paragraph_top.saturating_add(signed_hwpunit(common.vertical_offset))
    };
    let _ = paper_relative_horz;
    let horizontal = visible_left.saturating_sub(i32::from(common.margin.left))
        ..visible_left
            .saturating_add(width)
            .saturating_add(i32::from(common.margin.right));
    let vertical = visible_top.saturating_sub(i32::from(common.margin.top))
        ..visible_top
            .saturating_add(height)
            .saturating_add(i32::from(common.margin.bottom));

    (!horizontal.is_empty() && !vertical.is_empty()).then_some(FrameExclusion {
        horizontal,
        vertical,
        policy,
    })
}

/// A non-TAC `TopAndBottom` object positioned from its host paragraph.
pub(crate) fn is_para_topbottom_float(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && matches!(common.text_wrap, TextWrap::TopAndBottom)
        && matches!(common.vert_rel_to, VertRelTo::Para)
}

/// With no line box, an empty block host places its sole flow table
/// inside the before/after space already reserved by table formatting. Text and
/// whitespace hosts retain their own line boxes; negative/absolute positions
/// and saved frames have different origins.
pub(crate) fn reflow_empty_table_host(para: &Paragraph, table: &Table) -> bool {
    para.text.is_empty()
        // An implementation-tagged LineSeg still describes a computed line.
        // Lack of a trustworthy saved anchor does not erase that line box.
        && para.line_segs.is_empty()
        && matches!(para.controls.as_slice(), [Control::Table(_)])
        && is_para_topbottom_float(&table.common)
        && matches!(table.common.vert_align, VertAlign::Top)
        && signed_hwpunit(table.common.vertical_offset) >= 0
}

/// 빈 개체 앵커는 글줄을 점유하지 않고 표 포맷의 앞/뒤 간격을 소비한다.
/// 폭 0 저장 줄의 vpos는 이전 흐름 위치일 수 있으므로 새 단의 원점을 덮지 않는다.
/// 이 전환은 명시적 쪽·단 나누기로 새 프레임을 여는 통째 표에 적용한다.
/// 같은 프레임 안의 저장 표는 폭 0이어도 원본 줄 원점을 계속 소유한다.
/// 분할 표의 저장 앵커는 원본 조각의 컷·높이·단을 소유하므로 통째 표 상자로
/// 치환하지 않는다. 일반 저장 글줄·음수 오프셋·절대 좌표도 기존 계약을 유지한다.
pub(crate) fn empty_table_host_uses_formatted_box(para: &Paragraph, table: &Table) -> bool {
    reflow_empty_table_host(para, table)
        || (table.page_break == TablePageBreak::None
            && matches!(
                para.column_type,
                crate::model::paragraph::ColumnBreakType::Page
                    | crate::model::paragraph::ColumnBreakType::Column
            )
            && object_only_saved_table_anchor(para, table)
            && !para.stored_text_partition_is_dirty()
            && !para.cell_format_vpos_dirty
            && para.line_segs.iter().all(|line| {
                line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
            && table.common.vert_align == VertAlign::Top
            && signed_hwpunit(table.common.vertical_offset) >= 0)
}

/// 저장 원점이 별도로 입증되어야 하는 통째 빈 개체 앵커의 속성 계약.
fn saved_whole_empty_table_anchor(para: &Paragraph, table: &Table) -> bool {
    table.page_break == TablePageBreak::None
        && object_only_saved_table_anchor(para, table)
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
        && para.line_segs.iter().all(|line| {
            line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        })
        && table.common.vert_align == VertAlign::Top
        && signed_hwpunit(table.common.vertical_offset) >= 0
}

/// 실제 예약에서 선택한 저장 원점 증거를 첫 조각도 그대로 소비한다.
pub(crate) fn empty_table_host_uses_shared_formatted_box(
    para: &Paragraph,
    table: &Table,
    placement: Option<&ParagraphFloatPlacement>,
) -> bool {
    empty_table_host_uses_formatted_box(para, table)
        || (saved_whole_empty_table_anchor(para, table)
            && placement.is_some_and(|plan| {
                plan.flow == ParagraphFloatFlow::Exclusion && plan.stored_host_origin.is_some()
            }))
}

/// 쪽·종이 기준 표의 외곽 여백을 포함한 가시 원점과 흐름 하단.
/// 예약과 paint가 같은 가장자리 정렬 결과를 소비한다.
pub(crate) fn absolute_table_vertical_geometry(
    table: &Table,
    ref_y: f64,
    ref_h: f64,
    height: f64,
    dpi: f64,
) -> (f64, f64) {
    let offset = hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
    let top_margin = hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let bottom_margin = hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let top = match table.common.vert_align {
        VertAlign::Top | VertAlign::Inside => ref_y + offset + top_margin,
        VertAlign::Center => ref_y + (ref_h - height) / 2.0 + offset,
        VertAlign::Bottom | VertAlign::Outside => ref_y + ref_h - height - offset - bottom_margin,
    };
    (top, top + height + bottom_margin)
}

/// A positive-offset empty host float whose next, generated body paragraph has
/// no stored line-segment anchor. Hancom consumes the empty host's physical row,
/// lays that body paragraph in the remaining gap above the float, then resumes
/// ordinary flow below the float.
///
/// Returns the stored host row's `(vertical_pos, line_height + line_spacing)`.
/// The first coordinate anchors the float; their sum anchors the generated text.
pub(crate) fn empty_offset_float_deferred_text_ladder_hu(
    host: &Paragraph,
    table: &Table,
    following: &Paragraph,
) -> Option<(i32, i32)> {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| !ch.is_whitespace() && ch != '\u{FFFC}')
    };

    let qualifies = is_para_topbottom_float(&table.common)
        && table.common.flow_with_text
        && signed_hwpunit(table.common.vertical_offset) > 0
        && host.controls.len() == 1
        && !has_non_whitespace_text(host)
        && has_non_whitespace_text(following)
        // HWPX 원본에 linesegarray가 없어도 parser가 계산 segment를 보충한다.
        // 저장 segment가 하나라도 있으면 그 위치를 우선해야 하므로 제외한다.
        && following.line_segs.iter().all(|segment| {
            segment.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        });
    qualifies.then(|| {
        host.line_segs
            .iter()
            .find(|segment| {
                segment.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
            .map(|segment| {
                (
                    segment.vertical_pos,
                    segment
                        .line_height
                        .saturating_add(segment.line_spacing)
                        .max(0),
                )
            })
    })?
}

/// [#6366] 원본 HWPX 문단 기준 글앞으로 다행·다열 표가 `flowWithText` 이면
/// 데코레이션 Shape 단축에서 빼 쪽 분할에 참여한다.
///
/// 모든 `flowWithText` 글앞으로/글뒤로 표에 열면 #5918 쪽수가 늘고
/// text-overlap 기준선이 커진다. 한글 6쪽 정합 픽스처
/// (`2700727_animal_facility_standards.hwpx` pi=9)만 연다: 원본 HWPX,
/// 비-TAC, IN_FRONT_OF_TEXT, vert=문단, horz=문단, 40행 이상 6열 이상.
/// #5918 의 4×5·31×7 글앞으로 표는 데코레이션으로 남긴다.
pub(crate) fn original_hwpx_infront_para_flow_paginates(
    original_hwpx: bool,
    table: &Table,
) -> bool {
    original_hwpx
        && !table.common.treat_as_char
        && table.common.flow_with_text
        && matches!(table.common.text_wrap, TextWrap::InFrontOfText)
        && matches!(table.common.vert_rel_to, VertRelTo::Para)
        && matches!(table.common.horz_rel_to, HorzRelTo::Para)
        && table.row_count >= 40
        && table.col_count >= 6
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
    // [#7198] 저장 사다리는 줄간격을 **부호 그대로** 담는다. 음수 줄간격을 0 으로 깎으면
    // `next - host == lh + ls` 인 사다리가 등식에서 떨어져, host 줄 전진과 양수 offset 이
    // 흐름에서 빠진다(156467175 1쪽: 448 != 1500, 제목이 한글보다 11.7px 위).
    let advance = host_seg.line_height + host_seg.line_spacing;
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

/// [#6147] 저장 사다리가 "빈 앵커 문단의 줄 하나"만 증언하는 자리차지 밴드의 host 줄 계약.
///
/// 한글은 자리차지(TopAndBottom) 개체를 매단 **빈 앵커 문단**도 개체 아래에 자기 줄
/// 상자(`lh + ls`)를 차지한다. rhwp 는 #1147 이래 이 줄을 일괄 억제해 왔고(빈 앵커 vpos 가
/// 이미 갭을 인코딩한다는 전제), 그래서 밴드 바로 아래 첫 본문 문단이 개체에 딱 붙는다.
///
/// 억제가 옳은 문단과 아닌 문단은 **저장 사다리가 가른다** — `next.vpos - host.vpos` 가
/// 정확히 `lh + ls`(줄간격 부호 그대로, #7198) 이면 한글이 개체 높이를 접고 host 줄 advance 만 흐름에
/// 계상했다는 뜻이라, 그 줄은 별도로 더해야 할 실 흐름이다(= #1147 의 "vpos 가 이미
/// 갭을 인코딩" 전제가 성립하지 않는 문단). 델타가 개체 높이를 품은 일반 물리 사다리는
/// 등식이 깨져 자연 배제된다 — #2439 의 [#2808] 판별자와 같은 축이다.
///
/// #2439(`native_empty_host_rowbreak_line_advance_hu`)는 이 계약의 **단일 표·양수 offset·
/// RowBreak·native HWP5** 특수형이고, 이 함수는 같은 증거를 HWPX 저장 레이아웃과 다중
/// 자리차지 개체(보도자료 서식의 머리표 2~3개)로 넓힌다. 개체가 여럿이면 마지막 자리차지
/// 개체에서만 계상해 밴드마다 중복 가산되지 않게 한다.
pub(crate) fn stored_empty_anchor_band_host_line_advance_hu(
    stored_layout: bool,
    para: &Paragraph,
    control_index: usize,
    next_para: Option<&Paragraph>,
) -> Option<i32> {
    // host 글자가 **한 자도 없어야** 한다. 공백 한 칸이라도 있으면 #1147 억제가
    // 애초에 걸리지 않아 조판이 이미 host 줄을 계상하고 있고(156272593 pi=44:
    // `text=" "` → `PartialParagraph` 항목 존재), 여기서 또 더하면 이중 계상이라
    // 쪽이 하나 늘어난다(코퍼스 4,000 표본 유일 회귀). 판정을 #1147 의 조판 술어
    // (`para.text.is_empty()`)와 정확히 같은 축에 둔다.
    if !stored_layout || !para.text.is_empty() {
        return None;
    }
    // 편집으로 무효화된 저장 시작점은 문단 종료 진행량의 증거가 아니다.
    // 그런 문단은 현재 재조판 흐름을 사용하고 원본 사다리를 다시 더하지 않는다.
    if para.stored_text_partition_is_dirty()
        || next_para.is_some_and(Paragraph::stored_text_partition_is_dirty)
    {
        return None;
    }
    // #6950: 문단 종료는 현재 문단의 마지막 개체에서 한 번 소비한다.
    // 다음 문단에 표가 있더라도 그 문단의 시작과 현재 문단의 종료 계약은
    // 사라지지 않는다. 다음 저장 시작점은 진행량의 증거로만 사용하며,
    // 표 개수나 다음 문단의 controls.is_empty()로 종료량을 결정하지 않는다.
    stored_anchor_band_host_line_from_ladder(
        para,
        control_index,
        next_text_paragraph_vpos(next_para),
    )
}

/// [#6312] 글이 있는 자리차지 host 문단의 저장 사다리가 host 줄만 증언하면
/// 표 밴드 아래에 그 줄 상자(`lh + ls`)를 계상한다.
///
/// #6147 은 빈 앵커(`text.is_empty()`)에만 발동한다. 글이 있으면 #1147 억제가
/// 꺼져 조판이 host 줄을 이미 계상한다고 가정했는데, `is_current_visible_para_float`
/// 경로는 표 뒤에 줄 높이/줄간격을 건너뛰어 다음 문단이 표에 붙는다
/// (156721992 1쪽: 한글 27.0pt 자리, rhwp 0). 같은 사다리 등식
/// `next.vpos - host.vpos == lh+ls` 가 표 높이를 접었다는 뜻이라 표 높이를
/// 다시 더하지 않는다(#4090 이중 계상 차단과 같은 축).
pub(crate) fn stored_visible_anchor_band_host_line_advance_hu(
    stored_layout: bool,
    para: &Paragraph,
    control_index: usize,
    next_para: Option<&Paragraph>,
) -> Option<i32> {
    stored_visible_anchor_band_host_line_advance_from_vpos(
        stored_layout,
        para,
        control_index,
        next_plain_text_vpos(next_para),
    )
}

pub(crate) fn stored_visible_anchor_band_host_line_advance_from_vpos(
    stored_layout: bool,
    para: &Paragraph,
    control_index: usize,
    next_plain_text_vpos: Option<i32>,
) -> Option<i32> {
    if !stored_layout || !para_has_non_whitespace_text(para) {
        return None;
    }
    stored_anchor_band_host_line_from_ladder(para, control_index, next_plain_text_vpos)
}

fn next_plain_text_vpos(next_para: Option<&Paragraph>) -> Option<i32> {
    let next = next_para?;
    if !next.controls.is_empty() {
        return None;
    }
    next_text_paragraph_vpos(Some(next))
}

fn next_text_paragraph_vpos(next_para: Option<&Paragraph>) -> Option<i32> {
    let next = next_para?;
    if !para_has_non_whitespace_text(next) {
        return None;
    }
    next.line_segs
        .iter()
        .enumerate()
        .find(|(_, seg)| seg.tag & 0x8000_0000 == 0 && seg.line_height > 0)
        .map(|(index, seg)| ladder_vpos(next, index, seg.vertical_pos))
}

fn ladder_vpos(paragraph: &Paragraph, index: usize, fallback: i32) -> i32 {
    paragraph
        .source_line_seg_vertical_pos
        .as_ref()
        .and_then(|source| source.get(index).copied())
        .unwrap_or(fallback)
}

fn stored_anchor_band_host_line_from_ladder(
    para: &Paragraph,
    control_index: usize,
    next_plain_text_vpos: Option<i32>,
) -> Option<i32> {
    // 문단의 가시 개체가 전부 비-TAC 자리차지(vert=문단) float 이어야 한다 — 인라인
    // 내용이 섞이면 host 줄이 그 내용의 줄이지 앵커 줄이 아니다.
    let mut last_float = None;
    for (index, control) in para.controls.iter().enumerate() {
        let common = match control {
            Control::Table(table) => &table.common,
            Control::Picture(picture) => &picture.common,
            Control::Shape(shape) => shape.common(),
            _ => continue,
        };
        if !is_para_topbottom_float(common) {
            return None;
        }
        last_float = Some(index);
    }
    if last_float != Some(control_index) {
        return None;
    }
    // 다음 본문 문단의 저장 시작점과 현재 문단의 종료 진행량을 대조한다.
    // 앵커 스택(다음도 빈 앵커)의 줄간격은 #1133 이 보존하므로 호출부에서 제외한다.
    let next_vpos = next_plain_text_vpos?;

    // [#6312] 사다리 등식은 재조판 좌표가 아니라 원본 저장 vpos 로 본다.
    let (host_index, host_seg) = para
        .line_segs
        .iter()
        .enumerate()
        .find(|(_, seg)| seg.tag & 0x8000_0000 == 0 && seg.line_height > 0)?;
    // [#7198] 줄간격은 부호 그대로 — `native_empty_host_rowbreak_line_advance_hu` 와 같은 축.
    let advance = host_seg.line_height + host_seg.line_spacing;
    if advance <= 0 {
        return None;
    }
    let host_vpos = ladder_vpos(para, host_index, host_seg.vertical_pos);
    ((next_vpos - host_vpos - advance).abs() <= 1).then_some(advance)
}

/// 문단에 공백·개체 마커가 아닌 실제 글자가 있는가.
fn para_has_non_whitespace_text(para: &Paragraph) -> bool {
    para.text
        .chars()
        .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
}

/// [#5922] native HWP5 CellBreak 자리차지 표의 연속 조각 바깥 여백 재개방 계약.
///
/// 한글은 다쪽으로 이어지는 CellBreak 조각을 쪽마다 표 바깥 여백(상·하)을 다시
/// 열어 그린다(화성시 별표2 실측: 본문 상단 42.52pt + 0.5mm 여백 = 괘선 44pt).
/// RowBreak 의 #2439 계약과 달리 저장 ladder 증거를 요구할 수 없다 — 거대 표의
/// 저장 vpos ladder 는 표 높이를 접기 때문이다. 대신 구조를 좁힌다: native HWP5,
/// 비-TAC TopAndBottom(vert=문단), 쪽나눔=CellBreak, 빈 host 문단(표 전용).
/// 바깥 여백이 0 이면 더해질 값이 없어 no-op다.
pub(crate) fn native_empty_host_cellbreak_fragment_repeats_outer_margin(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
) -> bool {
    let has_non_whitespace_text = |paragraph: &Paragraph| {
        paragraph
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
    };
    native_hwp5_layout
        && !table.common.treat_as_char
        && is_para_topbottom_float(&table.common)
        && matches!(table.page_break, TablePageBreak::CellBreak)
        && !has_non_whitespace_text(para)
}

/// [#7095] 본문을 통째로 담은 1×1 `RowBreak` 쪽 조각의 비끝 상자 아래 안쪽 거리 (HU).
///
/// 한/글 2020 정본에서 비끝 조각 상자 아래 = 본문 아래 − `outer_margin_bottom` − 이 값.
/// 정본 PDF 는 A4 쪽을 595×841pt 로 내 내용이 0.99895 배 축소되므로 그 척도를 걷은 뒤 쟀다:
/// 156060125 104HU · 30269 103HU(쪽 rect 척도), 위 가장자리 적합 척도로는 101HU.
/// 156060125 한 필드 돌연변이 7종(바깥 아래 여백 · 표 안 여백 · 칸 테두리 · 쪽 위/아래 여백
/// ±30000HU)에서 흔들리지 않는 상수라 이름 붙은 필드가 아니다.
pub(crate) const SINGLE_CELL_PAGE_FRAGMENT_BOTTOM_INSET_HU: i32 = 100;

/// [#7095] 본문을 통째로 담은 1×1 `RowBreak` 표의 쪽 조각인가.
///
/// 이 형상에서 한/글은 조각마다 표 바깥 여백(위·아래)을 다시 연다. 저장된 쪽 프레임의
/// 비끝 조각 상자는 본문 아래에서 [`SINGLE_CELL_PAGE_FRAGMENT_BOTTOM_INSET_HU`] 만큼
/// 더 안쪽에 고정한다. 형상만으로 저장 프레임을 보장하지는 않는다. runtime 파생 줄의
/// 내용 컷은 paint에서 출처를 확인해 내용 높이를 보존한다.
/// 페이지네이터 예산(`typeset.rs`)과 렌더러 상자(`table_partial.rs`)가 같은 술어를 써야
/// 컷과 그림이 어긋나지 않는다.
///
/// 근거는 native HWP5 저장본(156060125 · 30269)이다. HWPX 계보는 조각 기하 계약이 따로
/// 있고(`hwpx_stored_layout` 계열), 넓히면 `rowbreak-problem-pages.hwpx` 16쪽에서 칸 안
/// 글상자가 꼬리말과 겹친다(PR #7098 실측).
pub(crate) fn native_single_cell_rowbreak_page_fragment(
    native_hwp5_layout: bool,
    table: &Table,
) -> bool {
    native_hwp5_layout
        && table.row_count == 1
        && table.col_count == 1
        && !table.common.treat_as_char
        && matches!(table.page_break, TablePageBreak::RowBreak)
}

/// 저장된 native RowBreak 표는 새 쪽에서 잘린 셀을 마칠 수 있다. 호스트의 양수
/// 개체 오프셋이 없어도 한컴은 바깥 위 여백을 다시 연다(86712 p28: 141HU,
/// PDF 첫 괘선 77.5px, 본문 상단 75.6px).
/// #2097에서 반증한 광범위한 빈 호스트 여백 규칙과 구분한다. 확인된 계약은
/// 다열 표의 마지막 행 내부 컷이다. 한 열의 거대 셀(#2214)은 이 여백을
/// 다시 열지 않아야 PDF와 맞는다.
/// [#7418] 두 열 표도 여백을 다시 연다. 76076 p34의 첫 괘선은 77.3px(HWP
/// 2020) / 77.5px(2024), 78494 p20·p21은 77.5px이다. 모두 본문 상단
/// 75.6px에 141HU를 더한 값이다. 이전 `col_count > 2` 조건은 76076 p34가
/// 여백 없이 정렬된 것으로 잘못 판단했다.
pub(crate) fn native_terminal_multirow_rowbreak_reopens_outer_top(
    native_hwp5_layout: bool,
    table: &Table,
    is_continuation: bool,
    start_row: usize,
    start_cut: &[usize],
) -> bool {
    native_hwp5_layout
        && is_continuation
        && table.row_count > 1
        && table.col_count > 1
        && start_row + 1 == table.row_count as usize
        && !start_cut.is_empty()
        && table.outer_margin_top > 0
        && is_para_topbottom_float(&table.common)
        && matches!(table.page_break, TablePageBreak::RowBreak)
        && signed_hwpunit(table.common.vertical_offset) == 0
}

/// Physical bottom of a nonterminal single-cell page fragment. Callers use
/// the same column-relative boundary for row fitting and page-relative painting.
pub(crate) fn single_cell_page_fragment_bottom(table: &Table, body_bottom: f64, dpi: f64) -> f64 {
    body_bottom
        - hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi)
        - hwpunit_to_px(SINGLE_CELL_PAGE_FRAGMENT_BOTTOM_INSET_HU, dpi)
}

/// [#6887] 문단 기준 왼쪽 정렬 어울림 표가 원점에 싣는 바깥 왼쪽 여백 (HU).
///
/// 한글은 `horzRelTo="PARA"` + `horzAlign="LEFT"` 인 float 의 저장 `horzOffset` 을
/// **바깥 여백 상자의 왼끝** 기준으로 읽는다 — 표 자신의 왼끝은 거기서
/// `outMargin.left` 만큼 더 안쪽이다. 같은 좌표를 다루는 개체 경로
/// (`shape_layout::form_object_origin`)는 이미 `ref_x + m_left + h_offset` 로
/// 이 여백을 싣고 있고, 표 경로만 빠져 있었다 (156492236: 선언 outMargin.left
/// 1417HU=18.89px 만큼 그림이 17쪽 전부에서 왼쪽으로 치우침).
///
/// `#6378` 이 연 `HorzRelTo::Column` 형상과 겹치지 않게 `Para` 기준으로만 열고,
/// 오른쪽·가운데 정렬은 `ref_w` 산식이 달라 건드리지 않는다. 여백이 0 이면
/// 더해질 값이 없어 no-op 다.
pub(crate) fn para_relative_left_aligned_outer_margin_left_hu(table: &Table) -> Option<i32> {
    if table.common.treat_as_char
        || !matches!(table.common.horz_rel_to, HorzRelTo::Para)
        || !matches!(table.common.horz_align, HorzAlign::Left | HorzAlign::Inside)
        || table.outer_margin_left <= 0
    {
        return None;
    }
    Some(i32::from(table.outer_margin_left))
}

/// [#7063] 왼쪽 정렬 **자리차지(TopAndBottom)** 표의 왼쪽 바깥여백 (HU).
///
/// 위 [`para_relative_left_aligned_outer_margin_left_hu`] 와 같은 규칙이지만 **다른
/// 갈래**다. 그쪽은 어울림(Square) 표의 `#6887` 계약이고 `layout.rs` 의 Square 전용
/// 분기 하나만 소비한다. 그 술어를 넓히면 Square 표가 이 조건에서 탈락해
/// `#6887` 이 깨지므로(`left_and_inside_apply_the_declared_margin_once`), 자리차지
/// 갈래는 여기서 따로 판정한다.
///
/// 정본 실측(`hwpx_sample2` engine 2020, 자리차지 표 20여 개): 좌단 차가 선언
/// `outMargin.left` 와 같다 — 141HU→1.86px · 283HU→3.78px · 0HU→0. 같은 쪽에서
/// 글줄 참여 표는 이미 이 여백을 받아 형제끼리 갈렸다(19쪽 표1 39.70 / 표2 37.80,
/// 정본은 둘 다 39.66).
///
/// 범위를 자리차지로 묶는 이유: 흐름 표(block)의 바깥여백은 `HostSpacing`
/// (`spacing_before`)이 이미 흐름에 싣고 있어 여기서 또 실으면 두 번 든다
/// (`byeolpyo1` 의 이미 맞던 표가 1.9px 내려간다 — 실측).
/// 오른쪽·가운데 정렬은 `ref_w` 산식이 달라 열지 않는다.
/// [#7287] **어울림(Square) 자리차지** 표의 위쪽 바깥여백 (HU).
///
/// 가로 [`para_relative_left_aligned_outer_margin_left_hu`](`#6887`) 의 세로판이다.
///
/// 이 갈래는 종전에 `layout.rs` 의 `table_y_start` 체인에서 **전용 분기가 없어**
/// 마지막 폴백(`y_offset` = 흐름 위치)으로 떨어졌다. 저장 앵커 경로(`stored_top` ·
/// 빈 host lane)는 모두 `is_para_topbottom_float` 를 요구해 어울림은 타지 않고,
/// `compute_table_y_position` 의 절대 배치 분기도 `TopAndBottom | BehindText |
/// InFrontOfText` 만 받는다. 그래서 위쪽 바깥여백을 아무도 내지 않았다.
///
/// 정본 실측(`pdf/hwpctl_API_v2.4-hwp-2020.pdf`): 이 형상 5건이 전부 정본보다
/// `+2.88 .. +3.67px` 위였고, 선언 `outMargin.top` 283HU(3.77px)에서 괘선
/// stroke/2(0.32px)를 뺀 값과 같다.
///
/// 오른쪽·가운데 정렬과 글줄 참여(`treat_as_char`) 표는 실측 근거가 없어 열지 않는다.
pub(crate) fn square_float_outer_margin_top_hu(table: &Table) -> Option<i32> {
    if !matches!(table.common.text_wrap, TextWrap::Square)
        || table.common.treat_as_char
        || !matches!(table.common.vert_rel_to, VertRelTo::Para)
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || !matches!(table.common.horz_align, HorzAlign::Left | HorzAlign::Inside)
        || table.outer_margin_top <= 0
    {
        return None;
    }
    Some(i32::from(table.outer_margin_top))
}

pub(crate) fn topbottom_float_outer_margin_left_hu(table: &Table) -> Option<i32> {
    if !matches!(table.common.text_wrap, TextWrap::TopAndBottom)
        || table.common.treat_as_char
        || !matches!(
            table.common.horz_rel_to,
            HorzRelTo::Para | HorzRelTo::Column
        )
        || !matches!(table.common.horz_align, HorzAlign::Left | HorzAlign::Inside)
        || table.outer_margin_left <= 0
    {
        return None;
    }
    Some(i32::from(table.outer_margin_left))
}

/// [#6378] 원본 HWPX 단 기준 RowBreak 자리차지 표의 사방 균등 outMargin (HU).
///
/// `hwp5_stored_pagination_layout` 이 꺼진 원본 HWPX 는 native HWP5 빈-host
/// RowBreak helper(`native_empty_host_physical_outer_box_paint_inset`)가 표
/// 원점에 싣는 바깥 여백을 주지 않는다. 같은 문서 HWP 경로(`tac-img-02`)는
/// 1mm(283HU) 안쪽에 둔다. HWPX XML `pageBreak="CELL"` 도 이 픽스처에서는
/// IR `RowBreak` 로 들어온다. 모든 원본 HWPX 표·연속 block 표(#1133)에
/// 더하면 간격이 3.8px 줄고 글자 겹침 기준선이 커지므로, native helper 와
/// 같은 형상만 연다: 비-TAC TopAndBottom(vert=문단), 단·왼쪽, RowBreak,
/// 하나 이상의 행·1열, 사방 균등 양의 outMargin, 오프셋 0. 단일 셀도
/// 같은 바깥 상자를 쓰므로 행 수는 상단 여백을 없앨 근거가 아니다.
pub(crate) fn original_hwpx_column_rowbreak_equal_outer_margin_hu(
    original_hwpx: bool,
    table: &Table,
) -> Option<i32> {
    let declared_height = signed_hwpunit(table.common.height);
    if !original_hwpx
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.common.vert_align, VertAlign::Top | VertAlign::Inside)
        || !matches!(table.common.horz_rel_to, HorzRelTo::Column)
        || !matches!(table.common.horz_align, HorzAlign::Left | HorzAlign::Inside)
        || signed_hwpunit(table.common.horizontal_offset) != 0
        || signed_hwpunit(table.common.vertical_offset) != 0
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || table.row_count == 0
        || table.col_count != 1
        || table.cells.len() != usize::from(table.row_count)
        || !table.cells.iter().enumerate().all(|(row, cell)| {
            cell.row == row as u16 && cell.col == 0 && cell.row_span == 1 && cell.col_span == 1
        })
        || signed_hwpunit(table.common.width) <= 0
        || declared_height <= 0
        || table.outer_margin_left <= 0
        || table.outer_margin_right <= 0
        || table.outer_margin_top <= 0
        || table.outer_margin_bottom <= 0
        || table.outer_margin_left != table.outer_margin_right
        || table.outer_margin_left != table.outer_margin_top
        || table.outer_margin_left != table.outer_margin_bottom
        || table.caption.is_some()
    {
        return None;
    }
    Some(i32::from(table.outer_margin_left))
}

/// [#5870] 빈 host 자리차지 float 의 저장 사다리가 **물리 공식과 정확히 일치**하는지 —
/// `next.vpos - host.vpos == v_off + outer_top + 선언높이 + outer_bottom` (±2HU) — 를
/// 검증하고, 일치하면 흐름에 더 계상해야 할 여분(`v_off + outer_top + outer_bottom`, HU)을
/// 돌려준다. 한글은 이 형상의 흐름을 그 합만큼 전진시키는데(10645 [별지 제11호서식]
/// 40쪽: 저장 델타 8413 = 1840+140+6293+140 정확 일치), rhwp 의 일반 빈-앵커 계약은
/// 표 높이만 전진시켜 다음 float 가 위로 올라와 겹쳤다(19.7px). 광역 "빈 host 에
/// v_off+outer 가산"은 #2097 이 반증했으므로(82802 pi75 는 outer 를 한 번만 담아 저장 —
/// 이 등식에 걸리지 않는다) 문단 단위 저장 증거를 게이트로 쓴다.
///
/// 추가 조임 두 겹 — ① RowBreak 표 제외: 빈-host RowBreak 는 별도 저장 계약군
/// (#2439·#3931·#3820 rowbreak tail 등)이 흐름·조각을 이미 다뤄 이중 계상이 된다
/// (r 게이트 실측: 3931×3·3930·3820·5699·5801·3565 회귀). ② 호출부는 **다음 문단도
/// 빈 float 표 앵커**일 때만 발동한다 — 이 결함의 실증상이 float 뒤 float 겹침이고,
/// 후속이 텍스트면 저장 vpos 재고정으로 어차피 무결하다. 나머지 구조 조건(빈 host·
/// 단일 표·비합성 lineseg·프로파일)도 호출부가 확인한다.
pub(crate) fn empty_host_physical_ladder_extras_hu(
    table: &Table,
    host_vpos: i32,
    next_vpos: i32,
) -> Option<i64> {
    if matches!(table.page_break, TablePageBreak::RowBreak) {
        return None;
    }
    let extras = i64::from(signed_hwpunit(table.common.vertical_offset).max(0))
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    if extras <= 0 {
        return None;
    }
    let stored_delta = i64::from(next_vpos) - i64::from(host_vpos);
    let physical_delta = extras + i64::from(table.common.height.min(i32::MAX as u32));
    ((stored_delta - physical_delta).abs() <= 2).then_some(extras)
}

/// [#3931] native HWP5 다행 RowBreak 표가 cell 내부 저장 page reset을 갖고,
/// 후속 source 문단도 host anchor 위로 되감기는 빈-host 형상인지 판별한다.
///
/// 반환값은 host의 저장 line advance다. 첫 fragment를 앞선 표 바로 뒤에 그릴 때
/// layout cursor에 남은 이전 host trailing margin이 이 advance 이내면 회수할 수
/// 있다는 구조 증거로 사용한다.
pub(crate) fn native_multirow_internal_reset_rowbreak_anchor_advance_hu(
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
        || table.row_count <= 1
        || table.common.treat_as_char
        || !is_para_topbottom_float(&table.common)
        || !matches!(table.page_break, TablePageBreak::RowBreak)
        || has_non_whitespace_text(para)
        || para
            .controls
            .iter()
            .filter(|control| matches!(control, Control::Table(_)))
            .count()
            != 1
    {
        return None;
    }

    let host_seg = para.line_segs.iter().find(|seg| {
        seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            && seg.line_height > 0
    })?;
    let next_seg = next_para?
        .line_segs
        .iter()
        .find(|seg| seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)?;
    if next_seg.vertical_pos >= host_seg.vertical_pos {
        return None;
    }

    let has_internal_reset = table.cells.iter().any(|cell| {
        let mut previous_vpos = None;
        for cell_para in &cell.paragraphs {
            for seg in cell_para.line_segs.iter().filter(|seg| {
                seg.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            }) {
                if previous_vpos.is_some_and(|previous| previous > 0 && seg.vertical_pos <= 0) {
                    return true;
                }
                previous_vpos = Some(seg.vertical_pos);
            }
        }
        false
    });
    if !has_internal_reset {
        return None;
    }

    let advance = host_seg.line_height + host_seg.line_spacing.max(0);
    (advance > 0).then_some(advance)
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
    /// Control identity within the paragraph owning this lane set.
    pub control_index: Option<usize>,
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
        control_index: Option<usize>,
        x_start: f64,
        x_end: f64,
        raw_top: f64,
        height: f64,
    ) -> FloatLane {
        let top = self.pushed_top(x_start, x_end, raw_top);
        let lane = FloatLane {
            control_index,
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

/// [#6143] 문단 기준 양수 오프셋이 **쪽 경계에서 이미 소진**됐는지 판정한다.
///
/// 오프셋의 기준점은 앵커 문단이 놓인 자리다. 저장 사다리가 준 앵커 자리에 오프셋을
/// 얹었을 때 표 상단이 이 쪽 바닥에서 최소 조각(`MIN_FRAGMENT_KEEP_PX`)도 남기지 못하는
/// 자리에 떨어진다면, 그 자리는 **앞 쪽 바닥**이고 오프셋은 거기서 이미 쓰였다. 그런
/// 조각을 이 쪽 최상단에서 다시 오프셋만큼 밀면 쪽 상단에 빈 띠가 생기고, 그만큼 조각이
/// 짧아져 표가 한 쪽 더 갈라진다(156555538 9쪽: 앵커 vpos=32514(433.5px) + off=41592
/// (554.6px) = 988.1px 로 가용 990.3px 의 바닥. 한글 17쪽 ↔ rhwp 18쪽).
///
/// 반대로 앵커 자리 + 오프셋이 쪽 안에 여유 있게 들어가면 그 오프셋은 이 쪽에서 유효한
/// 통상적인 미세 이동이므로 그대로 둔다(1342000 교육부 맵 p25: 200.0 + 10.0 ≪ 585.9).
pub(crate) fn para_offset_consumed_by_page_break(
    para: &Paragraph,
    common: &CommonObjAttr,
    available_height: f64,
    dpi: f64,
) -> bool {
    /// 오프셋을 적용한 자리에 이만큼도 안 남으면 그 자리에서는 조각이 시작될 수 없다.
    const MIN_FRAGMENT_KEEP_PX: f64 = 25.0;

    if common.treat_as_char || !matches!(common.vert_rel_to, VertRelTo::Para) {
        return false;
    }
    let offset = signed_hwpunit(common.vertical_offset);
    if offset <= 0 {
        return false;
    }
    if !available_height.is_finite() || available_height <= 0.0 {
        return false;
    }
    let anchor_top = para
        .line_segs
        .first()
        .map(|seg| hwpunit_to_px(seg.vertical_pos, dpi))
        .unwrap_or(0.0);
    anchor_top + hwpunit_to_px(offset, dpi) + MIN_FRAGMENT_KEEP_PX >= available_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::paragraph::LineSeg;
    use crate::model::shape::{HorzAlign, HorzRelTo, VertAlign};
    use crate::model::table::Cell;
    use crate::renderer::layout_frame::FrameExclusionPolicy;

    fn base_common() -> CommonObjAttr {
        CommonObjAttr {
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Column,
            horz_align: HorzAlign::Left,
            ..Default::default()
        }
    }

    fn supported_picture(flow: TextFlow) -> Picture {
        Picture {
            common: CommonObjAttr {
                width: 300,
                height: 200,
                text_wrap: TextWrap::Square,
                text_flow: flow,
                vert_rel_to: VertRelTo::Para,
                vert_align: VertAlign::Top,
                horz_rel_to: HorzRelTo::Column,
                horz_align: HorzAlign::Left,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn picture_exclusion_maps_only_recovered_side_wrap_flows() {
        let both_sides = resolve_picture_exclusion(
            &supported_picture(TextFlow::BothSides),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .expect("BothSides is represented by the physical row frame");
        assert_eq!(both_sides.policy, FrameExclusionPolicy::BothSides);

        let largest_only = resolve_picture_exclusion(
            &supported_picture(TextFlow::LargestOnly),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .expect("LargestOnly has a recovered frame policy");
        assert_eq!(largest_only.policy, FrameExclusionPolicy::LargestSide);

        assert!(resolve_picture_exclusion(
            &supported_picture(TextFlow::LeftOnly),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .is_none());
        assert!(resolve_picture_exclusion(
            &supported_picture(TextFlow::RightOnly),
            0..1_000,
            0..1_000,
            50,
            PaperOrigin::default(),
        )
        .is_none());
    }

    #[test]
    fn picture_exclusion_uses_flow_frame_for_oversized_current_square_float() {
        let mut picture = supported_picture(TextFlow::BothSides);
        picture.shape_attr.current_width = 900;
        picture.shape_attr.current_height = 800;

        let exclusion =
            resolve_picture_exclusion(&picture, 0..1_000, 0..1_000, 50, PaperOrigin::default())
                .expect("Square side-wrap float has a physical row frame");

        assert_eq!(exclusion.horizontal, 0..300);
        assert_eq!(exclusion.vertical, 50..250);
    }

    #[test]
    fn picture_exclusion_rejects_non_square_or_inline_hosts() {
        let mut picture = supported_picture(TextFlow::BothSides);
        picture.common.treat_as_char = true;
        assert!(
            resolve_picture_exclusion(&picture, 0..1_000, 0..1_000, 0, PaperOrigin::default())
                .is_none()
        );

        picture.common.treat_as_char = false;
        picture.common.text_wrap = TextWrap::TopAndBottom;
        assert!(
            resolve_picture_exclusion(&picture, 0..1_000, 0..1_000, 0, PaperOrigin::default())
                .is_none()
        );
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
        let first = lanes.place(None, 0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(None, 120.0, 200.0, 10.0, 20.0);

        assert_eq!(first.bottom, 50.0);
        assert_eq!(second.bottom, 30.0);
        assert_eq!(lanes.max_bottom(), 50.0);
    }

    #[test]
    fn lane_set_pushes_overlapping_ranges() {
        let mut lanes = FloatLaneSet::new();
        lanes.place(None, 0.0, 100.0, 10.0, 40.0);
        let second = lanes.place(None, 90.0, 160.0, 10.0, 20.0);

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
                outer_top_hu: 283,
                first_fragment_height_hu: Some(2_282),
            })
        );
        assert_eq!(
            native_hwp5_stored_reset_fragment_paint_geometry(true, &host, &table, true, &[2], &[],),
            Some(NativeStoredResetFragmentPaintGeometry {
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

/// [#7548] 다음 문단 저장 첫 줄의 차선 증거 — 측정(typeset)과 배치(layout)가 같은 값을 쓴다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StoredLineLaneProbe {
    pub(crate) vertical_pos: i32,
    pub(crate) column_start: i32,
    pub(crate) segment_width: i32,
    /// 같은 문단 원본 저장 줄 중 가장 넓은 폭(전폭 줄 상자).
    pub(crate) full_width: i32,
}

/// 원본 저장 줄(구현 속성 bit31 이 아닌 줄)에서 첫 줄 차선 증거를 뽑는다.
pub(crate) fn stored_line_lane_probe(para: &Paragraph) -> Option<StoredLineLaneProbe> {
    use crate::model::paragraph::LineSeg;
    let original = |seg: &&LineSeg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0;
    let first = para.line_segs.first().filter(original)?;
    let full_width = para
        .line_segs
        .iter()
        .filter(original)
        .map(|seg| seg.segment_width as i32)
        .max()?;
    Some(StoredLineLaneProbe {
        vertical_pos: first.vertical_pos,
        column_start: first.column_start,
        segment_width: first.segment_width as i32,
        full_width,
    })
}

/// [#7548] 어울림(Square) 표 host 다음 문단의 저장 첫 줄이 표 옆 차선에서 시작하는가.
///
/// 한/글은 어울림 표로 흐름을 밀지 않는다. 표 띠와 겹치는 줄은 옆 공간이 있으면
/// 좁혀서 그 자리에 두고, 없으면 띠 아래로 넘긴다. 저장 LineSeg 가 그 결과다.
/// host 의 저장 첫 줄이 표 옆 차선(전폭보다 좁은 cs/sw)을 증언하고, 다음 문단의
/// 첫 줄이 그 차선 안에 있으면 그 줄은 표 옆에 놓인 것이다(21_언어 14쪽 pi=300:
/// host·다음 첫 줄 모두 cs=3455 sw=27581, 둘째 줄부터 전폭 cs=852 sw=30184).
/// host 차선이 없거나(빈 host sw=0) 다음 줄이 표와 가로로 겹치면 표 하단에서 잇는다.
pub(crate) fn square_successor_starts_beside_table(
    host: &Paragraph,
    next: Option<StoredLineLaneProbe>,
    table: &Table,
) -> bool {
    use crate::model::paragraph::LineSeg;
    if table.common.treat_as_char || !matches!(table.common.text_wrap, TextWrap::Square) {
        return false;
    }
    let (Some(host_first), Some(next)) = (
        host.line_segs
            .first()
            .filter(|seg| seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0),
        next,
    ) else {
        return false;
    };
    let host_start = host_first.column_start;
    let host_end = host_start + host_first.segment_width as i32;
    host_first.segment_width > 0
        && (host_first.segment_width as i32) < next.full_width
        && next.vertical_pos >= host_first.vertical_pos
        && next.segment_width > 0
        && next.column_start >= host_start
        && next.column_start + next.segment_width <= host_end
}
