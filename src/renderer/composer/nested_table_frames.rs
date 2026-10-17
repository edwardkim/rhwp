//! 저장 줄 없는 셀 문단의 인라인 표 크기를 현재 줄 구성에 전달한다.

use super::{ComposedParagraph, ResolvedStyleSet, SingleLineOverflowCache};
use crate::model::{control::Control, paragraph::Paragraph, table::Table};

pub(crate) fn table_cells_have_no_stored_lines(table: &Table) -> bool {
    !table.cells.is_empty()
        && table.cells.iter().all(|cell| {
            cell.paragraphs
                .iter()
                .all(crate::renderer::para_has_no_stored_line_segs)
        })
}

/// 측정과 배치는 같은 폭·개체 크기로 기존 프레임 조판기를 호출한다.
/// 원본 모델은 유지하고, 파생 프레임에 실측 개체 높이와 줄 소속을 함께 보존한다.
#[allow(clippy::too_many_arguments)]
pub(crate) fn recompose_cell_lines_with_nested_tables(
    composed: &mut ComposedParagraph,
    source: &Paragraph,
    width: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    legacy_geometry: bool,
    native_hwp5: bool,
    cache: &SingleLineOverflowCache,
    mut nested_height: impl FnMut(&Table) -> f64,
) {
    let mut projected = None;
    let mut heights = Vec::new();
    let reflowed_table_host = !native_hwp5
        && !legacy_geometry
        && crate::renderer::para_has_no_stored_line_segs(source)
        && source.controls.iter().any(|control| {
            matches!(control, Control::Table(table)
                if table.common.treat_as_char && table_cells_have_no_stored_lines(table))
        });
    if reflowed_table_host {
        for (index, control) in source.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            if !table.common.treat_as_char {
                continue;
            }
            // 같은 줄 구성에 참여하는 저장 줄 자식도 프레임 소유자다. 일부만
            // 발행하면 셀 컷은 형제를 따로 예약하고 paint는 같은 줄에 겹쳐 그린다.
            // 자식의 내부 저장 줄은 재조판하지 않고 본체 실측만 전달한다.
            let measured = nested_height(table);
            let Ok(height) = u32::try_from(crate::renderer::px_to_hwpunit(measured, dpi)) else {
                continue;
            };
            let height = height.max(table.common.height);
            heights.push((index, height));
        }
        if !heights.is_empty() {
            let mut paragraph = source.clone();
            // 구현이 합성한 줄은 새 개체 크기를 위한 독립적인 저장 근거가 아니다.
            paragraph.line_segs.clear();
            for &(index, height) in &heights {
                if let Control::Table(table) = &mut paragraph.controls[index] {
                    table.common.height = height;
                }
            }
            projected = Some(paragraph);
        }
    }
    if let Some(paragraph) = &projected {
        // 원본에 글자가 없어도 인라인 개체 슬롯은 실제 줄을 만든다.
        // 빈 문자열을 점유 없음으로 바꾸지 않고 같은 프레임 소유자에게 전달한다.
        if let Some(rebuilt) = super::recompose_stored_lines_in_frame_with_known_square_band(
            composed,
            paragraph,
            super::ParagraphBox::content_for_style(
                width,
                styles.para_styles.get(paragraph.para_shape_id as usize),
                dpi,
            ),
            width,
            styles,
            dpi,
            legacy_geometry,
            super::line_breaking::StoredRowMissPolicy::Reflow,
            &[],
            false,
            false,
            false,
            &heights,
        ) {
            *composed = rebuilt;
        }
    } else {
        super::recompose_horizontal_cell_lines_for_width(
            composed,
            source,
            width,
            styles,
            dpi,
            legacy_geometry,
            native_hwp5,
            cache,
        );
    }
}
