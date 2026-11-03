//! 관계형 — `insert_picture_in_cell_native` 는 그림을 표 문단의 형제가 아니라 칸 문단 안에 넣는다 (#8556).
//! 주석 갱신: 2026-11-04T03:20:00+09:00 (새 파일)
//! 시각 측정은 해당 없음: 합성 문서에서 그림의 소속(칸 문단)과 위치 기준·오프셋 보존만 검사한다.
//! 기대값 근거: 한컴에서 칸 안 커서로 서명 그림을 넣고 저장한 문서의 구조 — 그림이 칸 문단의 컨트롤이고
//! 글자처럼 취급 아님, 세로 기준 문단(Para), 가로 기준 단(Column)이다. 저장 후 다시 열어도 같아야
//! 행 높이 변화·표 이동·쪽 나눔에서 그림이 칸과 함께 움직인다. 기존 `insertPicture` 의 칸 분기(#1151,
//! 표 문단 형제·종이 기준)는 바꾸지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::{HorzRelTo, VertRelTo};

/// 1×1 PNG (투명 픽셀 하나).
const PNG: [u8; 65] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60,
    0x82,
];

/// 칸 문단 안 그림의 (가로 기준, 세로 기준, 가로·세로 오프셋, 글자처럼 취급) — 표 문단 형제 그림이 있으면 실패.
fn cell_picture(doc: &Document, label: &str) -> (HorzRelTo, VertRelTo, i32, i32, bool) {
    // 저장본은 구역 정의 등이 앞에 붙어 컨트롤 번호가 달라질 수 있어 소속으로 찾는다.
    let parent = doc.sections[0]
        .paragraphs
        .iter()
        .find(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .unwrap_or_else(|| panic!("{label}: 표 문단"));
    assert!(
        parent
            .controls
            .iter()
            .all(|c| !matches!(c, Control::Picture(_))),
        "{label}: 표 문단 형제로 그림이 생기면 안 된다"
    );
    let table = parent
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("표");
    let pics: Vec<_> = table.cells[0].paragraphs[0]
        .controls
        .iter()
        .filter_map(|c| match c {
            Control::Picture(p) => Some(p),
            _ => None,
        })
        .collect();
    let [pic] = pics.as_slice() else {
        panic!("{label}: 칸 문단 안 그림 하나: {}", pics.len());
    };
    let common = &pic.common;
    (
        common.horz_rel_to,
        common.vert_rel_to,
        common.horizontal_offset as i32,
        common.vertical_offset as i32,
        common.treat_as_char,
    )
}

#[test]
fn cell_anchored_picture_lives_in_the_cell_paragraph_and_survives_save() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("유효한 글자·문단 모양 정의");
    core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    let table_para = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .expect("표 문단");
    let table_ctrl = core.document().sections[0].paragraphs[table_para]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .unwrap();

    // 문단 위로 걸친 서명처럼 세로 오프셋은 음수도 받는다.
    core.insert_picture_in_cell_native(
        0,
        table_para,
        0,
        &[(table_ctrl, 0, 0)],
        &PNG,
        5000,
        3000,
        1,
        1,
        "png",
        "서명",
        Some(1200),
        Some(-500),
    )
    .expect("칸 안 그림 넣기");

    let expected = (HorzRelTo::Column, VertRelTo::Para, 1200, -500, false);
    assert_eq!(cell_picture(core.document(), "넣은 직후"), expected);
    let reopened =
        DocumentCore::from_bytes(&core.export_hwp_native().expect("저장")).expect("다시 열기");
    assert_eq!(
        cell_picture(reopened.document(), "저장 후 다시 열기"),
        expected
    );
}

#[test]
fn cell_insert_requires_a_cell_path() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("유효한 글자·문단 모양 정의");
    let before = core.document().sections[0].paragraphs[0].controls.len();
    let result = core.insert_picture_in_cell_native(
        0,
        0,
        0,
        &[],
        &PNG,
        5000,
        3000,
        1,
        1,
        "png",
        "서명",
        None,
        None,
    );
    assert!(result.is_err(), "칸 경로 없이는 거절한다");
    assert_eq!(
        core.document().sections[0].paragraphs[0].controls.len(),
        before
    );
}
