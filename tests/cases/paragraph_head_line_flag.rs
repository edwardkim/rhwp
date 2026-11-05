//! 검사 유형: 관계형·고정형(원본 문단 머리 플래그); 페이지 절대 px 좌표 핀 없음.
//! 주석 갱신: 2026-11-05T14:31:40+09:00; 직전 파일 변경: bd8c7677cd424d228094a0ea06912b0e5ffc7b80 2026-11-03T06:00:37+09:00.
//! 측정일 2026-11-05, 제품 source 6337fe4d10; 96 DPI·Print·Native Skia/fresh WASM Canvas·2px raw 실루엣.
//! paragraph-heads: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! paragraph-head-edited-hwp: 전1쪽 Native 최저 99.71% / fresh WASM 최저 99.69%; 누락 없음.
//! paragraph-head-edited-hwpx: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! paragraph-head-off-hwpx: 전1쪽 Native 최저 100.00% / fresh WASM 최저 100.00%; 누락 없음.
//! 증거: mydocs/pr/assets/semanticist21-20261105-next10/validation/pre8622/candidate/{native,wasm}/<위 key>.tsv, validation.md 및 같은 영역의 독립 Print 확대 판독.
//! 순수 API/합성 계약 검사와 실물 문서의 시각 측정은 구분한다. 계산한 원본 크기·관계는 유지한다.
//!
//! 한/글(12.30 Mac)은 문단 머리(글머리표·그림 글머리표·번호·개요)가 있는 문단의 첫 줄에만
//! 줄 플래그 bit 21(`TAG_PARAGRAPH_HEAD`)을 켠다. 둘째 줄 이후와 머리를 끈 문단은 끈다.
//! HWP `PARA_LINE_SEG`와 HWPX `flags`가 같다. 비트가 꺼진 글머리표 문단은 한/글이
//! 글머리표를 그리지 않는다.
//!
//! 고정 문서는 한/글에서 만든 문단을 차례로 담는다: 글머리표(4줄), 번호(4줄), 개요 1(4줄),
//! 글머리표를 켰다 끈 문단, 보통 문단, 그림 글머리표(2줄), 글머리표(2줄).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::wasm_api::HwpDocument;

const FIXTURE: &str = "tests/fixtures/paragraph_head/hancom-heads.hwpx";
const LONG: &str = "Alpha bullet paragraph with enough words to wrap across several lines of the page so that we can see how the line flags look on the second and third lines of one paragraph.";

fn open() -> HwpDocument {
    HwpDocument::from_bytes(&std::fs::read(FIXTURE).unwrap()).unwrap()
}

/// 줄마다 bit 21 이 켜졌는지.
fn heads(para: &Paragraph) -> Vec<bool> {
    para.line_segs
        .iter()
        .map(|line| line.tag & LineSeg::TAG_PARAGRAPH_HEAD != 0)
        .collect()
}

/// 첫 줄만 켜진 여러 줄 문단(`head`) 또는 모두 꺼진 문단.
fn assert_head_lines(para: &Paragraph, head: bool, label: &str) {
    let flags = heads(para);
    assert!(!flags.is_empty(), "{label}: 줄 없음");
    let expected: Vec<bool> = (0..flags.len()).map(|i| head && i == 0).collect();
    assert_eq!(flags, expected, "{label}");
}

/// 문서와 그 HWP·HWPX 저장본을 다시 연 문서를 함께 돌려준다.
fn saved(doc: &HwpDocument) -> Vec<(&'static str, DocumentCore)> {
    vec![
        (
            "hwp",
            DocumentCore::from_bytes(&doc.export_hwp_native().unwrap()).unwrap(),
        ),
        (
            "hwpx",
            DocumentCore::from_bytes(&doc.export_hwpx_native().unwrap()).unwrap(),
        ),
    ]
}

#[test]
fn editing_a_hancom_bullet_paragraph_keeps_the_head_line_flag() {
    let mut doc = open();
    // 4줄 글머리표 문단은 가운데에, 2줄 글머리표 문단(6)은 맨 앞에 글을 넣어 다시 짠다.
    doc.insert_text_native(0, 0, 6, "edited ").unwrap();
    doc.insert_text_native(0, 6, 0, "x").unwrap();
    for para in [0, 6] {
        let lines = &doc.document().sections[0].paragraphs[para];
        assert!(lines.line_segs.len() > 1);
        assert_head_lines(lines, true, &format!("편집한 글머리표 문단 {para}"));
        for (format, core) in saved(&doc) {
            let lines = &core.document().sections[0].paragraphs[para];
            assert_head_lines(
                lines,
                true,
                &format!("{format} 저장본의 글머리표 문단 {para}"),
            );
        }
    }
}

#[test]
fn turning_a_head_off_clears_the_flag() {
    let mut doc = open();
    doc.apply_para_format_native(0, 0, r#"{"headType":"None"}"#)
        .unwrap();
    assert_head_lines(&doc.document().sections[0].paragraphs[0], false, "끈 문단");
    for (format, core) in saved(&doc) {
        let para = &core.document().sections[0].paragraphs[0];
        assert_head_lines(para, false, &format!("{format} 저장본의 끈 문단"));
    }
}

#[test]
fn applying_a_head_marks_only_the_first_line() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    let bullet = doc.ensure_default_bullet("●");
    let number = doc.ensure_default_numbering();
    // 본문 0: 글머리표, 1: 번호, 2: 머리 없음, 그 뒤 1×1 표의 칸 문단에 글머리표.
    doc.insert_text_native(0, 0, 0, LONG).unwrap();
    for para in 1..3 {
        doc.split_paragraph_native(0, para - 1, LONG.chars().count(), None)
            .unwrap();
        doc.insert_text_native(0, para, 0, LONG).unwrap();
    }
    doc.split_paragraph_native(0, 2, LONG.chars().count(), None)
        .unwrap();
    let created: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 3, 0, 1, 1).unwrap()).unwrap();
    let table_para = created["paraIdx"].as_u64().unwrap() as usize;
    let control = created["controlIdx"].as_u64().unwrap() as usize;
    doc.insert_text_in_cell_native(0, table_para, control, 0, 0, 0, LONG)
        .unwrap();
    let props = |head: &str, id: u16| {
        format!(r#"{{"headType":"{head}","paraLevel":0,"numberingId":{id}}}"#)
    };
    doc.apply_para_format_native(0, 0, &props("Bullet", bullet))
        .unwrap();
    doc.apply_para_format_native(0, 1, &props("Number", number))
        .unwrap();
    doc.apply_para_format_in_cell_native(0, table_para, control, 0, 0, &props("Bullet", bullet))
        .unwrap();

    let cell_of = |paragraphs: &[Paragraph]| -> Paragraph {
        paragraphs[table_para]
            .controls
            .iter()
            .find_map(|control| match control {
                rhwp::model::control::Control::Table(table) => {
                    Some(table.cells[0].paragraphs[0].clone())
                }
                _ => None,
            })
            .unwrap()
    };
    let check_body = |paragraphs: &[Paragraph], label: &str| {
        assert_head_lines(&paragraphs[0], true, &format!("{label} 글머리표"));
        assert_head_lines(&paragraphs[1], true, &format!("{label} 번호"));
        assert_head_lines(&paragraphs[2], false, &format!("{label} 머리 없음"));
    };
    let paragraphs = &doc.document().sections[0].paragraphs;
    check_body(paragraphs, "편집");
    assert!(paragraphs[0].line_segs.len() > 1);
    let cell = cell_of(paragraphs);
    assert_head_lines(&cell, true, "편집 칸 글머리표");
    let synthetic_cell = cell
        .line_segs
        .iter()
        .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0);
    for (format, core) in saved(&doc) {
        let paragraphs = &core.document().sections[0].paragraphs;
        check_body(paragraphs, format);
        let cell = cell_of(paragraphs);
        if format == "hwpx" && synthetic_cell {
            // HWPX는 rhwp가 만든 합성 줄(bit 31)만 가진 문단의 줄을 내지 않고 한/글이 다시 짜게 한다(#5847).
            assert!(cell.line_segs.is_empty(), "hwpx 칸 글머리표");
        } else {
            assert_head_lines(&cell, true, &format!("{format} 칸 글머리표"));
        }
    }
}
