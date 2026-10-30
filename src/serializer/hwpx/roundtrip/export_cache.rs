//! 편집 세션의 조판용 캐시와 HWPX에 저장되는 원본 줄 캐시를 구분한다.

use super::{diff_documents, IrDiff};
use crate::model::{
    control::Control,
    document::Document,
    paragraph::{LineSeg, Paragraph},
    shape::{Caption, ShapeObject},
};

/// HWPX 자기검증만 저장기의 줄 캐시 계약을 적용한다.
/// 원본 IR 보존 검사의 `diff_documents`는 그대로 엄격하게 비교한다.
/// 문서·저장 바이트를 바꾸지 않으며 다른 속성의 차이는 제거하지 않는다.
pub fn diff_exported_hwpx_documents(expected: &Document, actual: &Document) -> IrDiff {
    let mut expected = expected.clone();
    let mut actual = actual.clone();
    for document in [&mut expected, &mut actual] {
        for section in &mut document.sections {
            project_paragraphs(&mut section.paragraphs);
            for master in &mut section.section_def.master_pages {
                project_paragraphs(&mut master.paragraphs);
            }
        }
    }
    diff_documents(&expected, &actual)
}

/// [#8310] HWP 저장 자기검증 — 다시 연 저장본이 원본 HWPX 의 저장 줄 계약을 지닌 변환본
/// (`Document::hwpx_origin_keeps_source_lines`)일 때만 HWPX 와 같은 줄 캐시 계약으로 비교한다.
/// 그 밖의 HWP 저장은 종전대로 `diff_documents` 로 엄격하게 비교한다.
pub fn diff_exported_hwp_documents(expected: &Document, actual: &Document) -> IrDiff {
    if actual.hwpx_origin_keeps_source_lines() {
        diff_exported_hwpx_documents(expected, actual)
    } else {
        diff_documents(expected, actual)
    }
}

/// 저장기가 파일에 싣는 줄 캐시로 문단을 투영한다 — 임시 재조판 줄은 비우고, 원본 줄은
/// 원본 좌표 스냅샷(#5847)으로 되돌린다. HWPX 저장기와 원본 저장 줄 계약 HWP 저장(#8310)이
/// 같은 계약을 쓴다.
pub(crate) fn project_paragraphs(paragraphs: &mut [Paragraph]) {
    for para in paragraphs {
        // #5847: 임시 재조판 줄은 파일에 싣지 않는다. 정상 원본 줄은 유지한다.
        if !crate::serializer::line_cache::can_serialize_line_partition(para)
            || para
                .line_segs
                .iter()
                .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
        {
            para.line_segs.clear();
        } else if let Some(positions) =
            crate::serializer::line_cache::source_vertical_positions(para).map(<[i32]>::to_vec)
        {
            for (line, source) in para.line_segs.iter_mut().zip(positions) {
                line.vertical_pos = source;
            }
        }
        for control in &mut para.controls {
            match control {
                Control::Table(table) => {
                    for cell in &mut table.cells {
                        project_paragraphs(&mut cell.paragraphs);
                    }
                    project_caption(&mut table.caption);
                }
                Control::Picture(picture) => project_caption(&mut picture.caption),
                Control::Shape(shape) => project_shape(shape),
                Control::Header(header) => project_paragraphs(&mut header.paragraphs),
                Control::Footer(footer) => project_paragraphs(&mut footer.paragraphs),
                Control::Footnote(note) => project_paragraphs(&mut note.paragraphs),
                Control::Endnote(note) => project_paragraphs(&mut note.paragraphs),
                Control::HiddenComment(comment) => project_paragraphs(&mut comment.paragraphs),
                Control::Field(field) => project_paragraphs(&mut field.memo_paragraphs),
                _ => {}
            }
        }
    }
}

fn project_caption(caption: &mut Option<Caption>) {
    if let Some(caption) = caption {
        project_paragraphs(&mut caption.paragraphs);
    }
}

fn project_shape(shape: &mut ShapeObject) {
    if let Some(drawing) = shape.drawing_mut() {
        if let Some(text_box) = &mut drawing.text_box {
            project_paragraphs(&mut text_box.paragraphs);
        }
    }
    let caption = match shape {
        ShapeObject::Line(shape) => &mut shape.drawing.caption,
        ShapeObject::Rectangle(shape) => &mut shape.drawing.caption,
        ShapeObject::Ellipse(shape) => &mut shape.drawing.caption,
        ShapeObject::Arc(shape) => &mut shape.drawing.caption,
        ShapeObject::Polygon(shape) => &mut shape.drawing.caption,
        ShapeObject::Curve(shape) => &mut shape.drawing.caption,
        ShapeObject::Chart(shape) => &mut shape.caption,
        ShapeObject::Ole(shape) => &mut shape.caption,
        ShapeObject::Group(shape) => &mut shape.caption,
        ShapeObject::Picture(shape) => &mut shape.caption,
    };
    project_caption(caption);
    if let ShapeObject::Group(group) = shape {
        for child in &mut group.children {
            project_shape(child);
        }
    }
}
