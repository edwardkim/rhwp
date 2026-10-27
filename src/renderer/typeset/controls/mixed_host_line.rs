//! 그림 앵커와 TAC 표가 공유하는 한 줄의 수용·배제·쪽 소속 계획.
//!
//! 저장 줄의 표 높이는 보존하되, 현재 단 폭에 들어가지 않는 TAC를 같은
//! 줄에 강제로 되돌리지 않는다. 계산한 원점과 흐름 끝은 paint도 그대로 쓴다.

use crate::model::{
    control::Control,
    paragraph::Paragraph,
    shape::{TextWrap, VertAlign, VertRelTo},
};
use crate::renderer::{
    float_placement::{
        signed_hwpunit, InlineBoxPlacement, ParagraphFloatFlow, ParagraphFloatPlacement,
    },
    height_measurer::MeasuredTable,
    hwpunit_to_px,
    style_resolver::ResolvedStyleSet,
};

pub(in crate::renderer::typeset) struct MixedHostLine {
    pub picture_control: usize,
    pub picture_page: usize,
    pub picture: ParagraphFloatPlacement,
    pub table_control: usize,
    pub table_page: usize,
    pub table: InlineBoxPlacement,
    pub table_end: f64,
}

/// 한 줄의 온전한 표와 캡션 없는 그림만 소유한다. 분할 표·여러 밴드·다른
/// 글줄·가시 본문은 각각의 기존 소유자에게 남긴다.
pub(in crate::renderer::typeset) fn prepare(
    para: &Paragraph,
    para_index: usize,
    styles: &ResolvedStyleSet,
    tables: &[MeasuredTable],
    width: f64,
    origin: f64,
    available: f64,
    dpi: f64,
) -> Option<MixedHostLine> {
    if ![width, origin, available, dpi]
        .iter()
        .all(|v| v.is_finite())
        || dpi <= 0.0
    {
        return None;
    }
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    if para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || line.line_spacing < 0
        || para.text.contains(['\n', '\r', '\t'])
        || para.text.chars().any(|c| c > '\u{001f}' && c != '\u{fffc}')
    {
        return None;
    }
    let mut picture = None;
    let mut table = None;
    for (control, item) in para.controls.iter().enumerate() {
        match item {
            Control::Picture(p) if picture.is_none() => picture = Some((control, p)),
            Control::Table(t) if table.is_none() && t.common.treat_as_char => {
                table = Some((control, t))
            }
            Control::SectionDef(_)
            | Control::ColumnDef(_)
            | Control::PageNumberPos(_)
            | Control::PageNumCtrl(_) => {}
            _ => return None,
        }
    }
    let (picture_control, picture) = picture?;
    let (table_control, table) = table?;
    let common = &picture.common;
    if common.treat_as_char
        || !common.flow_with_text
        || common.allow_overlap
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || picture.caption.is_some()
        || table.caption.is_some()
        || table
            .cells
            .iter()
            .flat_map(|c| &c.paragraphs)
            .any(|p| !p.controls.is_empty())
        || crate::renderer::composer::control_slot_line_index(para, picture_control) != Some(0)
        || crate::renderer::composer::control_slot_line_index(para, table_control) != Some(0)
    {
        return None;
    }
    let measured = tables
        .iter()
        .find(|m| m.para_index == para_index && m.control_index == table_control)?;
    let table_height = measured.total_height
        + hwpunit_to_px(
            i32::from(table.outer_margin_top) + i32::from(table.outer_margin_bottom),
            dpi,
        );
    // 저장 줄의 단일 TAC 상자가 현재 측정 높이를 실제로 닫는지 확인한다.
    if (table_height - hwpunit_to_px(line.line_height, dpi)).abs() > dpi / 7200.0
        || table_height > available
    {
        return None;
    }
    let style = styles.para_styles.get(para.para_shape_id as usize)?;
    let font = para
        .char_shapes
        .first()
        .and_then(|r| styles.char_styles.get(r.char_shape_id as usize))?
        .font_size;
    let blank = crate::renderer::composer::frame_metrics_for_line(
        font,
        font,
        style.line_spacing_type,
        style.line_spacing,
        dpi,
    );
    let blank_height = hwpunit_to_px(blank.line_height, dpi);
    let blank_advance = hwpunit_to_px(blank.line_height + blank.line_spacing, dpi);
    let table_gap = hwpunit_to_px(line.line_spacing, dpi);
    let table_width = hwpunit_to_px(
        table.common.width as i32
            + i32::from(table.outer_margin_left)
            + i32::from(table.outer_margin_right),
        dpi,
    );
    let column_width = width - style.margin_left - style.margin_right;
    let first_indent = crate::renderer::equation_tac_flow::paragraph_line_indent(style.indent, 0);
    let picture_first = picture_control < table_control;
    // 폭 0인 앵커도 실제 줄을 소유한다. 뒤 TAC가 넘치면 다음 줄에 놓는다.
    let wrapped = picture_first && table_width > column_width - first_indent + dpi / 7200.0;
    let top = origin + style.spacing_before;
    let table_first_bottom = top + table_height + table_gap;
    let (_, picture_height_hu) = crate::renderer::layout::picture_flow_frame_size_hu(picture);
    let picture_height = hwpunit_to_px(
        picture_height_hu + i32::from(common.margin.top) + i32::from(common.margin.bottom),
        dpi,
    );
    let offset = hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi);
    let picture_origin = if picture_first {
        top
    } else {
        table_first_bottom
    };
    let picture_top = picture_origin + offset.max(0.0);
    let picture_page = usize::from(picture_top + picture_height > available);
    let picture_top = if picture_page == 0 { picture_top } else { 0.0 };
    let picture_bottom = picture_top + picture_height;
    let clear = |y: f64, height: f64| {
        if picture_page == 0 && y < picture_bottom && y + height > picture_top {
            picture_bottom
        } else {
            y
        }
    };
    let table_top = if !picture_first {
        top
    } else if wrapped {
        clear(clear(top, blank_height) + blank_advance, table_height)
    } else {
        clear(top, table_height)
    };
    let table_page = usize::from(table_top + table_height > available);
    let table_top = if table_page == 0 { table_top } else { 0.0 };
    // 첫 TAC도 현재 쪽에 들어가지 않는 경계는 기존 문단/표 분할 소유자가 처리한다.
    if !picture_first && table_page != 0 {
        return None;
    }
    let indent = crate::renderer::equation_tac_flow::paragraph_line_indent(
        style.indent,
        usize::from(wrapped),
    );
    let spare = (column_width - indent - table_width).max(0.0);
    let x = style.margin_left
        + indent
        + match style.alignment {
            crate::model::style::Alignment::Right => spare,
            crate::model::style::Alignment::Center => spare / 2.0,
            _ => 0.0,
        };
    let table_end = table_top + table_height + table_gap + style.spacing_after;
    // 표를 그림 위에 그리더라도, 같은 쪽의 마지막 항목은 그림의 물리
    // 하단을 되감지 않는다. 표의 줄 끝과 문단의 흐름 끝은 별개다.
    let table_flow_end = if picture_first && picture_page == table_page {
        table_end.max(picture_bottom)
    } else {
        table_end
    };
    Some(MixedHostLine {
        picture_control,
        picture_page,
        picture: ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::InlineHostPicture {
                next_flow_y: picture_bottom,
            },
            anchor_y: picture_top - offset,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top: picture_top,
            occupied_bottom: picture_bottom,
        },
        table_control,
        table_page,
        table: InlineBoxPlacement {
            x,
            y: table_top,
            clearance: 0.0,
            resolved_x: true,
            advance_end: Some(table_flow_end),
            stored_horizontal_pen: None,
            stored_column_frame: None,
        },
        table_end,
    })
}
