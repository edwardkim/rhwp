//! 다른 물리 소유자에서 가져온 저장 줄은 부모·자식이 함께 새 프레임을 만든다.

use crate::model::{
    control::Control,
    identity::walk::{walk, Node},
    paragraph::Paragraph,
    table::{Cell, Table},
};
use crate::renderer::{px_to_hwpunit, style_resolver::ResolvedStyleSet};

/// 정상 저장본에도 최소 줄 폭과 문단 앞 간격이 있다. 폭 하나 또는 양수 원점
/// 하나로 무효화하지 않는다. 가로 소유 폭과 설명되지 않는 첫 원점이 함께
/// 다를 때만 현재 셀의 저장 프레임으로 수용할 수 없다는 증거로 사용한다.
fn displaced_cell(cell: &Cell, table: &Table, styles: &ResolvedStyleSet, dpi: f64) -> bool {
    let Some(first) = cell.paragraphs.first() else {
        return false;
    };
    let Some(origin) = first.line_segs.first() else {
        return false;
    };
    if cell.text_direction != 0 || cell.width == 0 {
        return false;
    }
    let before = styles
        .para_styles
        .get(first.para_shape_id as usize)
        .map_or(0, |style| px_to_hwpunit(style.spacing_before, dpi));
    const ROUNDING_HU: i64 = 2;
    if i64::from(origin.vertical_pos) <= i64::from(before) + ROUNDING_HU {
        return false;
    }
    let padding = cell.effective_padding(&table.padding);
    let inner = i64::from(table.physical_cell_width_hu(cell))
        - i64::from(padding.left)
        - i64::from(padding.right);
    cell.paragraphs.iter().any(|paragraph| {
        paragraph
            .line_segs
            .iter()
            .any(|line| i64::from(line.segment_width) > inner + ROUNDING_HU)
    })
}

fn has_displaced_cell(paragraph: &Paragraph, styles: &ResolvedStyleSet, dpi: f64) -> bool {
    paragraph.controls.iter().any(|control| match control {
        Control::Table(table) => table.cells.iter().any(|cell| {
            displaced_cell(cell, table, styles, dpi)
                || cell
                    .paragraphs
                    .iter()
                    .any(|child| has_displaced_cell(child, styles, dpi))
        }),
        _ => false,
    })
}

fn discard(paragraphs: &mut [Paragraph], implicit_char_table: bool) {
    // 이미 있는 공통 소유 트리 순회로 표·글상자·캡션·각주도 같은 캐시 수명을 쓴다.
    walk(paragraphs, |node| {
        if let Node::Paragraph(paragraph) = node {
            paragraph.replace_line_segs_with_space_metrics(Vec::new(), Vec::new());
            if implicit_char_table && paragraph.char_shapes.is_empty() {
                paragraph
                    .char_shapes
                    .push(crate::model::paragraph::CharShapeRef {
                        start_pos: 0,
                        char_shape_id: 0,
                    });
            }
        }
        Ok(())
    })
    .expect("읽기 전용 렌더 사본의 캐시 폐기에는 실패 작업이 없다");
}

pub(crate) fn normalize(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    implicit_char_table: bool,
    cell_relative_frames: bool,
) -> Option<Vec<Paragraph>> {
    match normalize_with_storage(
        original,
        styles,
        dpi,
        implicit_char_table,
        cell_relative_frames,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
pub(crate) fn normalize_with_storage<S: crate::storage::Storage>(
    original: &[Paragraph],
    styles: &ResolvedStyleSet,
    dpi: f64,
    implicit_char_table: bool,
    cell_relative_frames: bool,
    storage: &S,
) -> Result<Option<Vec<Paragraph>>, S::Error> {
    let affected: Vec<_> = storage.collect(
        original
            .iter()
            .enumerate()
            .filter(|(_, paragraph)| {
                implicit_char_table
                    || (cell_relative_frames && has_displaced_cell(paragraph, styles, dpi))
            })
            .map(|(index, _)| index),
    )?;
    if affected.is_empty() {
        return Ok(None);
    }
    let mut projected = storage.try_collect(original.iter().map(|p| storage.clone_paragraph(p)))?;
    for index in affected {
        // 셀 캐시만 지우면 바깥 host의 옛 전진이 남는다. 의존하는 호스트와
        // 하위 소유 프레임을 함께 구성하되 편집·직렬화의 원본은 유지한다.
        discard(
            std::slice::from_mut(&mut projected[index]),
            implicit_char_table,
        );
    }
    Ok(Some(projected))
}
