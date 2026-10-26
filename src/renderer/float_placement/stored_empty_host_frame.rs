//! 다음 저장 줄이 닫는 빈 호스트 표의 프레임을 측정·배치에서 공유한다.

use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
    shape::{CommonObjAttr, HorzRelTo, TextWrap, VertAlign, VertRelTo},
    table::Table,
};
use crate::renderer::{hwpunit_to_px, px_to_hwpunit};

use super::{is_para_topbottom_float, signed_hwpunit, ParagraphFloatFlow, ParagraphFloatPlacement};

/// 종이·쪽에 고정된 장식은 보이더라도 호스트의 글줄 흐름을 밀지 않는다.
fn detached_overlay(common: &CommonObjAttr) -> bool {
    !common.treat_as_char
        && !common.flow_with_text
        && !common.affect_line_spacing
        && matches!(
            common.text_wrap,
            TextWrap::InFrontOfText | TextWrap::BehindText
        )
        && matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
        && matches!(common.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page)
}

fn single_flow_table(para: &Paragraph) -> bool {
    para.controls
        .iter()
        .filter(|c| matches!(c, Control::Table(_)))
        .count()
        == 1
        && para.controls.iter().all(|control| match control {
            Control::Table(_) => true,
            Control::Shape(shape) => detached_overlay(shape.common()),
            Control::Picture(picture) => detached_overlay(&picture.common),
            _ => false,
        })
}

/// 표 아래 캡션이 바깥 상자에 더하는 세로 높이(HWPUNIT). 캡션이 없으면 0 이다.
///
/// 한/글은 아래 캡션을 표 본체와 바깥 아래 여백 사이에 `캡션 간격 + 캡션 줄` 로 둔다
/// (한글문서파일형식 39쪽 표 235: 저장 델타 10545 = 위 566 + 표 6193 + 간격 852 + 줄 950 +
/// 아래 1984). 저장 줄이 모두 한컴이 쓴 줄일 때만 그 높이를 읽는다. 위·왼쪽·오른쪽 캡션은
/// 표 본체 원점이나 폭을 바꾸는 다른 기하라 이 상자로 대체하지 않는다.
pub(crate) fn stored_bottom_caption_hu(table: &Table) -> Option<i64> {
    let Some(caption) = table.caption.as_ref() else {
        return Some(0);
    };
    if caption.direction != crate::model::shape::CaptionDirection::Bottom
        || caption.paragraphs.is_empty()
    {
        return None;
    }
    let mut height = 0i64;
    for para in &caption.paragraphs {
        let (first, last) = (para.line_segs.first()?, para.line_segs.last()?);
        if para
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
        {
            return None;
        }
        height = height.max(
            i64::from(last.vertical_pos) + i64::from(last.line_height)
                - i64::from(first.vertical_pos.min(0)),
        );
    }
    Some(i64::from(caption.spacing) + height)
}

fn stored_line(para: &Paragraph) -> Option<i32> {
    let line = para.line_segs.first()?;
    (line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty)
        .then_some(line.vertical_pos)
}

impl ParagraphFloatPlacement {
    /// 표 이어받기 옆의 빈 저장 줄이 새 쪽의 앵커 축을 닫는다.
    /// 이전 쪽의 마지막 줄간격은 새 쪽 호스트의 원점에 재가산하지 않는다.
    pub(crate) fn from_stored_overlay_guide_chain(
        paragraphs: &[Paragraph],
        anchor: usize,
        host: usize,
        flow: f64,
        body_height: f64,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        dpi: f64,
    ) -> Option<Self> {
        let source = paragraphs.get(anchor)?;
        let para = paragraphs.get(host)?;
        let [Control::Table(table)] = para.controls.as_slice() else {
            return None;
        };
        let [Control::Table(owner)] = source.controls.as_slice() else {
            return None;
        };
        let overlay = |table: &Table| {
            !table.common.treat_as_char
                && matches!(
                    table.common.text_wrap,
                    TextWrap::InFrontOfText | TextWrap::BehindText
                )
                && table.common.vert_rel_to == VertRelTo::Para
                && table.common.vert_align == VertAlign::Top
        };
        if anchor >= host
            || !overlay(table)
            || !overlay(owner)
            || !source.text.is_empty()
            || source.line_segs.len() != 1
            || !para.text.is_empty()
            || para.line_segs.len() != 1
            || !flow.is_finite()
            || !body_height.is_finite()
        {
            return None;
        }
        stored_line(source)?;
        let mut previous = source.line_segs.first()?;
        if previous.line_height <= 0 || previous.line_spacing < 0 {
            return None;
        }
        let body = px_to_hwpunit(body_height, dpi);
        let mut reset = false;
        for index in anchor + 1..=host {
            let paragraph = paragraphs.get(index)?;
            if !paragraph.text.is_empty()
                || paragraph.line_segs.len() != 1
                || paragraph.column_type != ColumnBreakType::None
                || (index < host && !paragraph.controls.is_empty())
            {
                return None;
            }
            stored_line(paragraph)?;
            let style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
            // 줄 상자와 후행 간격으로 닫힌 축이다. 별도 문단 간격을
            // 같은 높이로 추정하거나 새 호스트의 앞 간격에 중복 반영하지 않는다.
            if style.spacing_before != 0.0 || style.spacing_after != 0.0 {
                return None;
            }
            let line = &paragraph.line_segs[0];
            if line.line_height <= 0 || line.line_spacing < 0 {
                return None;
            }
            let ink_end = previous.vertical_pos.checked_add(previous.line_height)?;
            let end = ink_end.checked_add(previous.line_spacing)?;
            if line.vertical_pos != end {
                // 줄의 내용은 앞 쪽에 들어가고 후행 간격만 쪽 끝에서 닫힌다.
                if reset || line.vertical_pos != 0 || ink_end > body || end <= body {
                    return None;
                }
                reset = true;
            }
            previous = line;
        }
        let origin = hwpunit_to_px(previous.vertical_pos, dpi);
        if !reset
            || origin < flow
            || previous.vertical_pos.checked_add(previous.line_height)? > body
        {
            return None;
        }
        Some(Self {
            flow: ParagraphFloatFlow::NextLine,
            anchor_y: origin,
            stored_host_origin: Some(origin),
            stored_successor_line_origin: None,
            table_left: None,
            table_top: origin
                + hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi)
                + hwpunit_to_px(i32::from(table.outer_margin_top), dpi),
            // 장식 개체는 뒤 본문의 줄 흐름을 점유하지 않는다.
            occupied_bottom: origin,
        })
    }

    /// 원본 빈 겹침 호스트의 앞 간격은 글줄 앵커가 소유한다.
    /// 앞 가시 줄의 저장 끝과 현재 흐름이 같을 때 개체는 그 원점의 바깥 위 여백에서 열린다.
    pub(crate) fn from_stored_empty_overlay_host(
        previous: &Paragraph,
        para: &Paragraph,
        table: &Table,
        origin: f64,
        spacing_before: f64,
        page_base_hu: i32,
        dpi: f64,
    ) -> Option<Self> {
        let visible = |p: &Paragraph| {
            p.text
                .chars()
                .any(|c| !c.is_whitespace() && !c.is_control() && c != '\u{FFFC}')
        };
        if !single_flow_table(para)
            || visible(para)
            || !visible(previous)
            || !previous.controls.is_empty()
            || table.common.treat_as_char
            || !table.common.flow_with_text
            || !matches!(
                table.common.text_wrap,
                TextWrap::InFrontOfText | TextWrap::BehindText
            )
            || table.common.vert_rel_to != VertRelTo::Para
            || table.common.vert_align != VertAlign::Top
            || signed_hwpunit(table.common.vertical_offset) < 0
            || table.caption.is_some()
            || para.line_segs.len() != 1
            || previous.stored_text_partition_is_dirty()
            || previous.cell_format_vpos_dirty
            || !origin.is_finite()
            || !spacing_before.is_finite()
            || spacing_before < 0.0
        {
            return None;
        }
        let host = stored_line(para)?;
        stored_line(previous)?;
        let last = previous.line_segs.last()?;
        if last.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
            || last.line_height <= 0
            || last.line_spacing < 0
        {
            return None;
        }
        let previous_end = i64::from(last.vertical_pos)
            + i64::from(last.line_height)
            + i64::from(last.line_spacing);
        let before_hu = i64::from(px_to_hwpunit(spacing_before, dpi));
        let relative_end = previous_end - i64::from(page_base_hu);
        if previous_end + before_hu != i64::from(host)
            || (origin * 7200.0 / dpi - relative_end as f64).abs() > 2.0
        {
            return None;
        }
        let top = origin
            + (i64::from(signed_hwpunit(table.common.vertical_offset))
                + i64::from(table.outer_margin_top)) as f64
                * dpi
                / 7200.0;
        Some(Self {
            flow: ParagraphFloatFlow::NextLine,
            anchor_y: origin,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: top,
            occupied_bottom: top,
        })
    }
}

/// 저장 호스트 앞 간격을 제외한 원점부터 다음 줄까지가 선언 본체·오프셋·
/// 바깥 여백을 정확히 포함할 때만 그 상자를 수용한다. 선언 높이가 같다는
/// 사실만으로 문단 뒤 간격이나 호스트 줄 높이를 추가하지 않는다.
pub(super) fn from_stored_ladder(
    para: &Paragraph,
    next: &Paragraph,
    table: &Table,
    origin: f64,
    spacing_before: f64,
    measured_table_height: f64,
    origin_closes_float_frame: bool,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    from_ladder(
        para,
        next,
        table,
        origin,
        spacing_before,
        measured_table_height,
        origin_closes_float_frame,
        dpi,
        false,
    )
}

/// 원본 사다리가 증명한 간격 소유는 보존하되 본체는 현재 재조판 높이를 쓴다.
pub(super) fn from_reflowed_ladder(
    para: &Paragraph,
    next: &Paragraph,
    table: &Table,
    origin: f64,
    spacing_before: f64,
    measured_table_height: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    from_ladder(
        para,
        next,
        table,
        origin,
        spacing_before,
        measured_table_height,
        false,
        dpi,
        true,
    )
}

fn from_ladder(
    para: &Paragraph,
    next: &Paragraph,
    table: &Table,
    origin: f64,
    spacing_before: f64,
    measured_table_height: f64,
    origin_closes_float_frame: bool,
    dpi: f64,
    reflowed: bool,
) -> Option<ParagraphFloatPlacement> {
    let paragraph_square = !table.common.treat_as_char
        && table.common.text_wrap == TextWrap::Square
        && table.common.vert_rel_to == VertRelTo::Para;
    if !single_flow_table(para)
        || para
            .text
            .chars()
            .any(|c| !c.is_whitespace() && !c.is_control() && c != '\u{FFFC}')
        || !(is_para_topbottom_float(&table.common) || paragraph_square)
        || next.column_type != ColumnBreakType::None
        || table.common.vert_align != VertAlign::Top
        || signed_hwpunit(table.common.vertical_offset) < 0
        || !origin.is_finite()
        || !spacing_before.is_finite()
        || spacing_before < 0.0
        || !measured_table_height.is_finite()
        || measured_table_height < 0.0
    {
        return None;
    }
    let caption_hu = stored_bottom_caption_hu(table)?;
    let declared = hwpunit_to_px(
        (i64::from(table.common.height.min(i32::MAX as u32)) + caption_hu) as i32,
        dpi,
    );
    if !reflowed && (measured_table_height - declared).abs() > 1.0 {
        return None;
    }
    let host_vpos = stored_line(para)?;
    let next_vpos = stored_line(next)?;
    let physical_hu = i64::from(signed_hwpunit(table.common.vertical_offset))
        + i64::from(table.outer_margin_top)
        + i64::from(table.common.height.min(i32::MAX as u32))
        + caption_hu
        + i64::from(table.outer_margin_bottom);
    let spacing_before_hu = px_to_hwpunit(spacing_before, dpi) as i64;
    let delta = i64::from(next_vpos) - i64::from(host_vpos);
    if delta <= 0 {
        return None;
    }
    // 원점은 문단 위끝이다. host 앞 간격이 적용되면 저장 줄은 그만큼 아래라 `델타 = 상자 −
    // 앞 간격` 이다. [#8149] 바로 앞 항목이 빈 host 자리차지 표 상자로 끝나 원점이 그 상자
    // 끝이면, 한/글은 이 host 의 앞 간격을 두지 않아 문단 위끝이 곧 저장 줄이고 `델타 = 상자`
    // 다(한글문서파일형식 39쪽 표 235: 앞은 표 234 조각, 정본 첫 행 255.4 = 줄 + 위 여백 566HU
    // + 칸 여백). 원점이 그 상자 끝이 아니면(글 host 의 밴드 뒤 — 156160455 7쪽 pi=71) 커서가
    // 저장 줄에 있다는 근거가 없으므로 이 등식으로 받지 않는다. 두 경우 모두 다음 저장 줄이
    // 상자 바닥이다.
    let applied = (delta - (physical_hu - spacing_before_hu)).abs() <= 2;
    let absorbed = origin_closes_float_frame && (delta - physical_hu).abs() <= 2;
    if !applied && !absorbed {
        return None;
    }
    // 점유 하단은 그림 원점이 아니라 다음 저장 줄이 닫는 전체 프레임이다.
    // 닫힌 등식과 동일한 저장 출처를 사용해 실제 본체 윗변을 정한다.
    // HWP5 앵커의 원점을 HWPX object frame의 위 여백으로 바꾸지 않는다.
    // 저장 등식이 없는 기존 앵커는 이 경로에 들어오지 않는다.
    let table_top = origin
        + (i64::from(signed_hwpunit(table.common.vertical_offset))
            + crate::renderer::stored_float_anchor::stored_outer_box_table_top_hu(table))
            as f64
            * dpi
            / 7200.0;
    let occupied_bottom = origin
        + physical_hu as f64 * dpi / 7200.0
        + if reflowed {
            measured_table_height - declared
        } else {
            0.0
        };
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::Exclusion,
        anchor_y: origin,
        stored_host_origin: None,
        stored_successor_line_origin: Some(occupied_bottom),
        table_left: None,
        table_top,
        occupied_bottom,
    })
}
