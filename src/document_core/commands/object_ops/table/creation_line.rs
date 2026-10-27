//! 새 셀의 상속 서식과 저장 줄 메트릭을 같은 재조판 결과로 만든다.
use crate::document_core::DocumentCore;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::renderer::composer::{reflow_line_segs, ParagraphBox};

pub(super) fn for_cursor(
    core: &DocumentCore,
    section: usize,
    paragraph: usize,
    offset: usize,
) -> LineSeg {
    let source = &core.document.sections[section].paragraphs[paragraph];
    let mut empty = Paragraph::new_empty_like(source);
    empty.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: source.char_shape_id_at(offset).unwrap_or(0),
    }];
    empty.line_segs.clear();
    // 빈 줄에는 너비로 나눌 내용이 없다. 실제 셀 너비는 호출자가 발행하며,
    // 높이·기준선·줄간격은 보통 편집과 같은 줄 생산자가 계산한다.
    let width = source
        .line_segs
        .first()
        .map(|line| line.segment_width)
        .filter(|width| *width > 0)
        .unwrap_or(core.document.sections[section].section_def.page_def.width as i32)
        .max(1);
    reflow_line_segs(
        &mut empty,
        ParagraphBox::content(0..width),
        &core.styles,
        core.dpi,
    );
    empty.line_segs.into_iter().next().unwrap_or_default()
}
