//! [#7265] 셀 문단의 **글자 크기**를 바꾸면 후속 문단 사다리도 다시 세운다.
//!
//! # 무엇이 깨져 있었나
//!
//! `apply_char_format_in_cell_native` 는 글자 크기처럼 글줄 흐름에 영향을 주는 변경에서
//! 그 문단을 **리플로우까지는 했다**. 그런데 #6639 가 문단모양 경로에 세운 사다리 갱신
//! 표시(`cell_format_vpos_dirty` / `pending_cell_format_vpos`)를 세우지 않아, 문단이
//! 한 줄에서 두 줄로 늘어도 **다음 문단은 옛 `vpos` 에 그대로 남았다.**
//!
//! ```text
//!   글자 크기 키우기 전  문단0 [0]           문단1 [1440]
//!   키운 뒤(수정 전)     문단0 [0, 5120]     문단1 [1440]   ← 문단0 둘째 줄 위에 얹힌다
//!   키운 뒤(수정 후)     문단0 [0, 5120]     문단1 [10240]
//! ```
//!
//! # 이 검사가 쓰는 불변식
//!
//! 구현이 계산한 수를 되읽지 않는다. **한 칸 안에서 문단 `i+1` 의 첫 줄은 문단 `i` 의
//! 마지막 줄 아래에서 시작해야 한다** — 저장 사다리가 지켜야 하는 독립 계약이다.
//! (칸 안 쪽 나눔이 만드는 되감김은 계약에서 제외한다. 이 검사의 합성 입력에는 없다.)
//!
//! 대조군으로 **글줄 흐름에 영향을 주지 않는 글자 서식**(글자 색)은 사다리를 전혀 움직이지
//! 않아야 한다 — 표시를 넓게 세우면 여기서 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

const PARA_COUNT: usize = 5;

/// 한 칸 셀에 문단 다섯을 담은 표.
fn cell_with_paragraphs() -> DocumentCore {
    let mut doc = DocumentCore::new_empty();
    {
        let mut section = rhwp::model::document::Section::default();
        section.section_def.page_def = rhwp::model::page::PageDef::a4_default();
        section
            .paragraphs
            .push(rhwp::model::paragraph::Paragraph::new_empty());
        let mut document = rhwp::model::document::Document::default();
        document.sections.push(section);
        doc.set_document(document);
    }
    doc.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    for i in 0..PARA_COUNT {
        let text = format!("문단{} 한글 본문 내용입니다 길게 씁니다.", i + 1);
        let len = text.chars().count();
        doc.insert_text_in_cell_native(0, 0, 0, 0, i, 0, &text)
            .expect("칸 문단 글 삽입");
        if i + 1 < PARA_COUNT {
            doc.split_paragraph_in_cell_native(0, 0, 0, 0, i, len, None)
                .expect("칸 문단 쪼개기");
        }
    }
    doc
}

/// 칸 문단별 `(첫 줄 vpos, 마지막 줄 바닥)`.
fn ladder(doc: &DocumentCore) -> Vec<(i32, i32)> {
    let Some(Control::Table(table)) = doc.document().sections[0].paragraphs[0].controls.first()
    else {
        panic!("표를 찾지 못했다 — 시험 설정 오류");
    };
    table
        .cells
        .first()
        .expect("칸")
        .paragraphs
        .iter()
        .filter_map(|p| {
            let first = p.line_segs.first()?;
            let last = p.line_segs.last()?;
            Some((first.vertical_pos, last.vertical_pos + last.line_height))
        })
        .collect()
}

fn char_count(doc: &DocumentCore, cell_para_idx: usize) -> usize {
    let Some(Control::Table(table)) = doc.document().sections[0].paragraphs[0].controls.first()
    else {
        panic!("표 없음");
    };
    table.cells[0].paragraphs[cell_para_idx]
        .text
        .chars()
        .count()
}

/// 칸 문단의 첫 글자 모양 id — 되돌리기 경로에 넘길 원래 값.
fn original_char_shape_id(doc: &DocumentCore, cell_para_idx: usize) -> u32 {
    let Some(Control::Table(table)) = doc.document().sections[0].paragraphs[0].controls.first()
    else {
        panic!("표 없음");
    };
    table.cells[0].paragraphs[cell_para_idx]
        .char_shapes
        .first()
        .map(|cs| cs.char_shape_id)
        .unwrap_or(0)
}

/// 문단 `i+1` 의 첫 줄이 문단 `i` 의 마지막 줄 바닥보다 위면 그 쌍을 돌려준다.
fn overlapping_pair(ladder: &[(i32, i32)]) -> Option<(usize, i32, i32)> {
    ladder
        .windows(2)
        .enumerate()
        .find(|(_, pair)| pair[1].0 < pair[0].1)
        .map(|(i, pair)| (i, pair[0].1, pair[1].0))
}

/// 글자 크기를 키우면 후속 문단이 그만큼 내려간다 — 포개지지 않는다.
#[test]
fn enlarging_cell_text_moves_the_following_paragraphs_down() {
    let mut doc = cell_with_paragraphs();
    let before = ladder(&doc);
    assert!(
        overlapping_pair(&before).is_none(),
        "시험 설정 오류 — 편집 전부터 문단이 포개져 있다: {before:?}"
    );

    for cell_para in 0..PARA_COUNT {
        let end = char_count(&doc, cell_para);
        doc.apply_char_format_in_cell_native(0, 0, 0, 0, cell_para, 0, end, r#"{"fontSize":3200}"#)
            .expect("글자 크기 적용");
    }
    let after = ladder(&doc);

    // 이 검사가 비어 있지 않음을 먼저 보인다 — 이 변경은 실제로 줄을 늘린다.
    assert!(
        after
            .iter()
            .zip(&before)
            .any(|(a, b)| a.1 - a.0 > b.1 - b.0),
        "글자 크기를 키웠는데 어떤 문단도 높아지지 않았다 — 검사가 대상 경로를 타지 못한다. \
         전={before:?} 후={after:?}"
    );

    assert_eq!(
        overlapping_pair(&after),
        None,
        "글자 크기를 키운 뒤 문단이 포개졌다 — 리플로우는 했는데 후속 문단 사다리를 \
         다시 세우지 않았다는 뜻이다. 사다리={after:?}"
    );
}

/// 대조군 — 글줄 흐름에 영향이 없는 글자 색 변경은 사다리를 건드리지 않는다.
#[test]
fn a_paint_only_char_format_leaves_the_ladder_alone() {
    let mut doc = cell_with_paragraphs();
    let before = ladder(&doc);
    for cell_para in 0..PARA_COUNT {
        let end = char_count(&doc, cell_para);
        doc.apply_char_format_in_cell_native(0, 0, 0, 0, cell_para, 0, end, r#"{"textColor":255}"#)
            .expect("글자 색 적용");
    }
    assert_eq!(
        ladder(&doc),
        before,
        "글자 색만 바꿨는데 사다리가 움직였다 — 갱신 표시를 너무 넓게 세웠다."
    );
}

/// 되돌리기 경로(`setCharShapeId`)도 같은 계약을 지킨다 — do/undo 대칭.
#[test]
fn restoring_the_original_char_shape_restores_the_ladder() {
    let mut doc = cell_with_paragraphs();
    let before = ladder(&doc);
    let original = original_char_shape_id(&doc, 0);

    for cell_para in 0..PARA_COUNT {
        let end = char_count(&doc, cell_para);
        doc.apply_char_format_in_cell_native(0, 0, 0, 0, cell_para, 0, end, r#"{"fontSize":3200}"#)
            .expect("글자 크기 적용");
    }
    let enlarged = ladder(&doc);
    assert_ne!(
        enlarged, before,
        "시험 설정 오류 — 글자 크기가 사다리를 안 움직였다"
    );

    for cell_para in 0..PARA_COUNT {
        let end = char_count(&doc, cell_para);
        doc.set_char_shape_id_in_cell_native(0, 0, 0, 0, cell_para, 0, end, original)
            .expect("원래 글자 모양 복원");
    }
    let restored = ladder(&doc);
    assert_eq!(
        overlapping_pair(&restored),
        None,
        "되돌린 뒤 문단이 포개졌다 — undo 경로가 사다리를 다시 세우지 않았다. 사다리={restored:?}"
    );
    // 원래 사다리와의 **완전 일치**는 요구하지 않는다. 이 합성 입력의 최초 사다리는
    // 글 삽입 경로가 쌓은 것이라 문단 모양과 아직 어긋나 있고, 서식 경로를 한 번 지나면
    // 그 불일치가 정리되어 값이 달라진다. 계약은 «포개지지 않는다» 쪽이다.
    assert!(
        restored.len() == before.len(),
        "되돌린 뒤 문단 수가 달라졌다 — 되돌리기가 내용을 건드렸다. 전={before:?} 후={restored:?}"
    );
}

/// Studio 가 셀에 쓰는 `by_path` 입구(깊이 1)도 같은 계약을 지킨다.
///
/// 깊이 1 은 flat 형제에 위임하므로 이 검사는 그 위임이 살아 있는지를 함께 잠근다.
#[test]
fn the_by_path_entry_point_keeps_the_ladder_sane() {
    let mut doc = cell_with_paragraphs();
    let before = ladder(&doc);
    for cell_para in 0..PARA_COUNT {
        let end = char_count(&doc, cell_para);
        doc.apply_char_format_in_cell_by_path(
            0,
            0,
            &[(0, 0, cell_para)],
            0,
            end,
            r#"{"fontSize":3200}"#,
        )
        .expect("by_path 글자 크기 적용");
    }
    let after = ladder(&doc);
    assert_ne!(
        after, before,
        "시험 설정 오류 — by_path 변경이 사다리를 안 움직였다"
    );
    assert_eq!(
        overlapping_pair(&after),
        None,
        "by_path 로 글자 크기를 키운 뒤 문단이 포개졌다. 사다리={after:?}"
    );
}
