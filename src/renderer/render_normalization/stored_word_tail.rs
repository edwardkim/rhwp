//! 줄 끝 재결합과 같은 원본 물리 프레임의 후속 원점을 함께 발행한다.
use crate::model::{
    control::Control,
    page::{ColumnDef, PageDef},
    paragraph::{ColumnBreakType, LineSeg, Paragraph},
};
use crate::renderer::{
    composer::{project_stored_soft_wraps, projects_current_marker_geometry, ParagraphBox},
    page_layout::PageLayoutInfo,
    px_to_hwpunit,
    style_resolver::ResolvedStyleSet,
};

fn uniform_column_width(page: &PageDef, columns: &ColumnDef, dpi: f64) -> Option<f64> {
    let layout = PageLayoutInfo::from_page_def(page, columns, dpi);
    let width = layout.column_areas.first()?.width;
    // 너비가 다른 단은 현재 문단의 실제 소유 단 없이 첫 단으로 추측하지 않는다.
    layout
        .column_areas
        .iter()
        .all(|column| px_to_hwpunit(column.width, dpi) == px_to_hwpunit(width, dpi))
        .then_some(width)
}

fn frame_advance(lines: &[LineSeg]) -> Option<i32> {
    let first = lines.first()?;
    let last = lines.last()?;
    last.vertical_pos
        .checked_add(last.line_height)?
        .checked_add(last.line_spacing)?
        .checked_sub(first.vertical_pos)
}

pub(crate) fn normalize(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
    page: &PageDef,
    initial_columns: &ColumnDef,
    dpi: f64,
) -> Option<Vec<Paragraph>> {
    let mut width = uniform_column_width(page, initial_columns, dpi);
    let mut shift = 0_i32;
    let mut previous_source_vpos = None;
    let mut result = Vec::with_capacity(original.len());
    let mut changed = false;
    for source in original {
        if source.column_type != ColumnBreakType::None
            || styles
                .para_styles
                .get(source.para_shape_id as usize)
                .is_some_and(|style| style.page_break_before)
        {
            shift = 0;
        }
        for control in &source.controls {
            if let Control::ColumnDef(columns) = control {
                width = uniform_column_width(page, columns, dpi);
                shift = 0;
            }
        }
        let mut paragraph = source.clone();
        let mut origin_changed = false;
        for line in &mut paragraph.line_segs {
            let source_vpos = line.vertical_pos;
            if line.is_first_line_of_page()
                || line.is_first_line_of_column()
                || previous_source_vpos.is_some_and(|previous| source_vpos < previous)
            {
                shift = 0;
            }
            previous_source_vpos = Some(source_vpos);
            line.vertical_pos = source_vpos.checked_add(shift)?;
            origin_changed |= shift != 0;
        }
        if origin_changed {
            // 원문 문자 축은 그대로지만 물리 원점은 새 프레임 소유다.
            // 이전 원점 snapshot을 남겨 같은 파생 줄을 서로 다른 축으로
            // 판정하지 않는다. 원본 document의 snapshot은 보존된다.
            paragraph.source_line_seg_vertical_pos = None;
        }
        if let Some(width) = width {
            let style = styles.para_styles.get(paragraph.para_shape_id as usize);
            let box_ = ParagraphBox::body_for_style(width, style, dpi);
            // 현재 측정으로 더 적은 줄에 들어간다는 사실만으로 정상 저장 줄을
            // 무효화하지 않는다. 원문 문자에 없는 목록 마커는 문서 순서에서
            // 해소한 실제 기하를 갱신해야 하며, 같은 결과가 채움·높이에 참여한다.
            if projects_current_marker_geometry(&paragraph, &box_, styles) {
                let first = paragraph.line_segs[0].vertical_pos;
                let frame = box_.frame(first);
                if let Some((lines, spaces)) =
                    project_stored_soft_wraps(&paragraph, &frame, styles, dpi, false)
                {
                    let old_advance = frame_advance(&paragraph.line_segs)?;
                    let new_advance = frame_advance(&lines)?;
                    shift = shift.checked_add(new_advance.checked_sub(old_advance)?)?;
                    paragraph.replace_line_segs_with_space_metrics(lines, spaces);
                    changed = true;
                }
            }
        }
        result.push(paragraph);
    }
    changed.then_some(result)
}
