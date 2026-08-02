//! `explain` 명령의 전용 집계 — 각주/미주 개수.
//!
//! `info`·`export-structure`·`export-tables`·`fields` 는 이미 문서 요약에 필요한
//! 대부분을 돌려준다. 이 모듈은 그 네 조회가 채우지 못하는 마지막 구멍 하나,
//! 각주·미주 개수만 담당한다. 새 판정 로직이 아니라 `table_extract::collect_from_paragraph`
//! 와 같은 컨테이너 재귀(글상자·머리말·꼬리말·표 셀·각주·미주)를 그대로 따라간다 —
//! 최상위 문단의 controls 만 훑으면 글상자·표 셀 안에 놓인 각주/미주를 놓친다.

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;

/// 문서 안 각주·미주 개수.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoteCounts {
    pub footnote_count: usize,
    pub endnote_count: usize,
}

/// 병적으로 깊은 중첩(손상/악의적 문서)에서 스택이 터지지 않게 하는 상한.
/// `table_extract::MAX_NEST_DEPTH` 와 같은 값을 쓴다.
const MAX_NEST_DEPTH: usize = 8;

fn count_from_paragraph(para: &Paragraph, depth: usize, counts: &mut NoteCounts) {
    if depth >= MAX_NEST_DEPTH {
        return;
    }
    for control in &para.controls {
        match control {
            Control::Footnote(f) => {
                counts.footnote_count += 1;
                for p in &f.paragraphs {
                    count_from_paragraph(p, depth + 1, counts);
                }
            }
            Control::Endnote(e) => {
                counts.endnote_count += 1;
                for p in &e.paragraphs {
                    count_from_paragraph(p, depth + 1, counts);
                }
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    for p in &cell.paragraphs {
                        count_from_paragraph(p, depth + 1, counts);
                    }
                }
            }
            Control::Shape(shape) => {
                if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                    for p in &tb.paragraphs {
                        count_from_paragraph(p, depth + 1, counts);
                    }
                }
            }
            Control::Header(h) => {
                for p in &h.paragraphs {
                    count_from_paragraph(p, depth + 1, counts);
                }
            }
            Control::Footer(f) => {
                for p in &f.paragraphs {
                    count_from_paragraph(p, depth + 1, counts);
                }
            }
            _ => {}
        }
    }
}

/// 문서 전체(본문·글상자·머리말/꼬리말·표 셀 포함)의 각주/미주 개수를 센다.
pub fn count_notes(doc: &Document) -> NoteCounts {
    let mut counts = NoteCounts::default();
    for section in &doc.sections {
        for para in &section.paragraphs {
            count_from_paragraph(para, 0, &mut counts);
        }
    }
    counts
}
