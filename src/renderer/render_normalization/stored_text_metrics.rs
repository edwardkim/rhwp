//! 원문 글자 크기와 줄간격이 함께 입증하는 명목 줄 메트릭의 렌더 전용 투영.

use crate::model::{
    control::Control,
    paragraph::{LineSeg, Paragraph},
    style::{BorderLineType, HeadType, LineSpacingType},
};
use crate::renderer::{
    float_placement::is_para_topbottom_float,
    px_to_hwpunit,
    style_resolver::{detect_lang_category, ResolvedStyleSet},
};

fn floating_text_host(paragraph: &Paragraph) -> bool {
    let mut floating = false;
    for control in &paragraph.controls {
        match control {
            Control::Table(table) if is_para_topbottom_float(&table.common) => floating = true,
            Control::SectionDef(_) | Control::ColumnDef(_) => {}
            _ => return false,
        }
    }
    floating
}

fn nominal_height(paragraph: &Paragraph, styles: &ResolvedStyleSet, dpi: f64) -> Option<i32> {
    // 여러 글자모양의 경계는 완전한 문자 주소가 있을 때만 읽는다.
    // 주소가 없는 합성 입력에서 첫 글자모양을 전체 문단의 근거로 쓰지 않는다.
    if paragraph.char_shapes.is_empty()
        || paragraph
            .char_shapes
            .windows(2)
            .any(|pair| pair[0].start_pos > pair[1].start_pos)
        || (paragraph.char_shapes.len() > 1
            && (paragraph.char_offsets.len() != paragraph.text.chars().count()
                || paragraph
                    .char_offsets
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])))
    {
        return None;
    }
    let dpi = styles.resolved_dpi.unwrap_or(dpi);
    let mut height = 0;
    let mut visible = false;
    for (position, ch) in paragraph.text.chars().enumerate() {
        if ch.is_control() || ch == '\u{FFFC}' {
            return None;
        }
        visible |= !ch.is_whitespace();
        let style = styles
            .char_styles
            .get(paragraph.char_shape_id_at(position)? as usize)?;
        let language = detect_lang_category(ch);
        if style.superscript
            || style.subscript
            || style.font_position_for_lang(language) != 0
            || style.outline_type != 0
            || style.shadow_type != 0
            || style.emboss
            || style.engrave
            || style.emphasis_dot != 0
        {
            return None;
        }
        if let Some(border) = style
            .border_fill_id
            .checked_sub(1)
            .and_then(|index| styles.border_styles.get(usize::from(index)))
        {
            if border
                .borders
                .iter()
                .any(|edge| edge.line_type != BorderLineType::None)
            {
                return None;
            }
        }
        let size = style.font_size_for_lang(language);
        if !size.is_finite() || size <= 0.0 {
            return None;
        }
        height = height.max(px_to_hwpunit(size, dpi));
    }
    // 마지막 빈 run은 보이는 글자가 없어도 문단 끝 줄 상자의 글자모양이다.
    // 재조판과 같은 원본 UTF-16 종단 주소를 소비해야 정상 저장 높이를
    // 가시 글자의 크기만으로 줄이고 셀의 기준선을 이동시키지 않는다.
    let end_size = crate::renderer::composer::paragraph_end_font_size_px(paragraph, styles);
    if end_size.is_finite() && end_size > 0.0 {
        height = height.max(px_to_hwpunit(end_size, dpi));
    }
    (visible && height > 0).then_some(height)
}

fn projected_line(paragraph: &Paragraph, styles: &ResolvedStyleSet, dpi: f64) -> Option<LineSeg> {
    // 다른 문단의 편집으로 유효한 저장 줄까지 버리지 않는다.
    // 실제 편집·셀 서식 변경으로 무효화된 줄만 원래 재조판 경로에 맡긴다.
    if paragraph.stored_text_partition_is_dirty() || paragraph.cell_format_vpos_dirty {
        return None;
    }
    let [line] = paragraph.line_segs.as_slice() else {
        return None;
    };
    let style = styles.para_styles.get(paragraph.para_shape_id as usize)?;
    if !matches!(style.head_type, HeadType::None)
        || !matches!(style.line_spacing_type, LineSpacingType::Percent)
        || !style.line_spacing.is_finite()
        || style.line_spacing < 100.0
        || line.text_height <= 0
        || line.baseline_distance <= 0
        || line.baseline_distance > line.text_height
    {
        return None;
    }
    let height = nominal_height(paragraph, styles, dpi)?;
    // 원본 글자 크기와 원본 Percent 간격이 같은 높이를 증명해야 한다.
    // 1 HWPUNIT는 정수로 저장되는 줄간격의 반올림만 허용한다.
    let expected_spacing = (f64::from(height) * (style.line_spacing / 100.0 - 1.0)).round();
    if line.text_height <= height
        || line.line_height <= height
        || (f64::from(line.line_spacing) - expected_spacing).abs() > 1.0
    {
        return None;
    }
    let mut projected = line.clone();
    projected.line_height = height;
    projected.text_height = height;
    projected.baseline_distance = (i64::from(line.baseline_distance) * i64::from(height)
        / i64::from(line.text_height)) as i32;
    Some(projected)
}

fn project(
    paragraph: &Paragraph,
    cell: bool,
    hwpx_cells: bool,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<Paragraph> {
    let eligible =
        floating_text_host(paragraph) || (cell && hwpx_cells && paragraph.controls.is_empty());
    let mut projected = eligible
        .then(|| projected_line(paragraph, styles, dpi))
        .flatten()
        .map(|line| {
            let mut result = paragraph.clone();
            result.line_segs[0] = line;
            result
        });
    for (control_index, control) in paragraph.controls.iter().enumerate() {
        let Control::Table(table) = control else {
            continue;
        };
        for (cell_index, cell) in table.cells.iter().enumerate() {
            if cell.text_direction != 0 || cell.line_wrap != 0 {
                continue;
            }
            for (paragraph_index, child) in cell.paragraphs.iter().enumerate() {
                if let Some(child) = project(child, true, hwpx_cells, styles, dpi) {
                    let result = projected.get_or_insert_with(|| paragraph.clone());
                    let Control::Table(table) = &mut result.controls[control_index] else {
                        unreachable!()
                    };
                    table.cells[cell_index].paragraphs[paragraph_index] = child;
                }
            }
        }
    }
    projected
}

pub(crate) fn normalize(
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    hwpx_cells: bool,
) -> Option<Vec<Paragraph>> {
    let mut result = None;
    for (index, paragraph) in paragraphs.iter().enumerate() {
        if let Some(projected) = project(paragraph, false, hwpx_cells, styles, dpi) {
            result.get_or_insert_with(|| paragraphs.to_vec())[index] = projected;
        }
    }
    result
}
