//! 관계형·고정형 — `insert_picture_in_cell_native` 는 그림을 표 문단의 형제가 아니라 칸 문단 안에 넣고,
//! 글 옆 칸 그림은 행 나눔에서 한 번만 센다 (#8556). 고정형은 독립 한컴 인쇄의 전체 쪽수(5)뿐이다.
//! 주석 갱신: 2026-11-06T22:17:14+09:00 (Asia/Seoul; 메인테이너 경계 검사 추가)
//! 직전 파일 변경: c2f05a26f38fc314c012d0655067cd664be35bd1 2026-11-04T21:48:26+09:00 (Git 시각; 아래 측정일과 구분).
//! 추가 합성 검사는 실패 원자성·기본 인수·반환 소유 경로만 검사하며 새 배치를 고정하지 않는다.
//! 앞 두 검사는 합성 문서의 소속·위치 기준·오프셋만 본다(시각 측정 해당 없음).
//! 쪽 소속 검사: 2026-11-04 측정, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`
//! raw 실루엣(2px 이웃), 입력 `samples/issue8556/21868765_cell_picture_insert.hwp`(같은 편집),
//! 기준 `pdf/issue8556/21868765_cell_picture_insert-hwp-mac-12.30.pdf`(한글 2022 맥 12.30.0 인쇄, 5쪽).
//! Native 98.77·99.59·98.67·99.26·100.00, fresh WASM 98.77·99.59·98.68·98.75·100.00 (각 5쪽, 최저 98.67·98.68).
//! 증거: `mydocs/pr/assets/issue_8556_insert_picture_in_cell/`.
//! 기대값 근거: 한컴에서 칸 안 커서로 서명 그림을 넣고 저장한 문서의 구조 — 그림이 칸 문단의 컨트롤이고
//! 글자처럼 취급 아님, 세로 기준 문단(Para), 가로 기준 단(Column), 본문 영역 제한(attr bit 13) 켬이다.
//! 한글 2022(맥 12.31)에서 bit 13 을 끈 칸 그림을 인쇄하면 칸이 아니라 표 머리(바깥 문단 기준)에 그려졌다.
//! 저장 후 다시 열어도 같아야
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

/// 칸 문단 안 그림의 (가로 기준, 세로 기준, 가로·세로 오프셋, 글자처럼 취급, 본문 영역 제한)
/// — 표 문단 형제 그림이 있으면 실패.
fn cell_picture(doc: &Document, label: &str) -> (HorzRelTo, VertRelTo, i32, i32, bool, bool) {
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
        common.flow_with_text,
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

    let expected = (HorzRelTo::Column, VertRelTo::Para, 1200, -500, false, true);
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

/// 쪽마다 본문 표 칸에 그려진 글자열을 모은다.
fn page_texts(doc: &rhwp::wasm_api::HwpDocument) -> Vec<Vec<String>> {
    fn walk(node: &serde_json::Value, out: &mut Vec<String>) {
        if node["type"] == "TextRun" {
            if let Some(text) = node["text"].as_str().filter(|t| !t.trim().is_empty()) {
                out.push(text.trim().to_string());
            }
        }
        for child in node["children"].as_array().into_iter().flatten() {
            walk(child, out);
        }
    }
    (0..doc.page_count())
        .map(|page| {
            let tree: serde_json::Value =
                serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap();
            let mut out = Vec::new();
            walk(&tree, &mut out);
            out
        })
        .collect()
}

#[test]
fn cell_picture_beside_host_text_grows_the_row_once_for_pagination() {
    // 1쪽 3행(「3. 보건소간 업무조정…」) 내용 칸에 글 옆 그림(9.9mm)을 넣는다.
    // 한글 2022(맥 12.30) 인쇄: 행이 그림 높이만큼 한 번 커지고 1쪽은 「33.」 행까지, 2쪽은 「34.」부터,
    // 전체 5쪽. 그림을 호스트 줄 아래에 쌓아 세면 1쪽이 「32.」에서 끝난다.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/21868765_별표2_보건소_분장사무.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).expect("표본");
    core.insert_picture_in_cell_native(
        0,
        4,
        0,
        &[(0, 11, 0)],
        &PNG,
        2800,
        2800,
        1,
        1,
        "png",
        "그림",
        Some(22000),
        Some(0),
    )
    .expect("칸 그림");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&core.export_hwp_native().unwrap())
        .expect("저장본");
    let pages = page_texts(&doc);
    assert_eq!(pages.len(), 5, "한컴 인쇄와 같은 5쪽");
    let page_of = |label: &str| {
        pages
            .iter()
            .position(|texts| texts.iter().any(|t| t == label))
            .unwrap_or_else(|| panic!("{label} 행이 어느 쪽에도 없다"))
    };
    assert_eq!(page_of("33."), 0, "「33.」 행은 1쪽");
    assert_eq!(page_of("34."), 1, "「34.」 행은 2쪽");
}

fn blank_cell_core() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.create_table_native(0, 0, 0, 1, 1).unwrap();
    let (para, control) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls
                .iter()
                .position(|c| matches!(c, Control::Table(_)))
                .map(|ci| (pi, ci))
        })
        .unwrap();
    (core, para, control)
}

#[test]
fn invalid_cell_paragraph_does_not_register_image_data_or_change_document() {
    let (mut core, para, control) = blank_cell_core();
    for path in [[(control, 0, 999)], [(control, 999, 0)], [(999, 0, 0)]] {
        let before = core.export_hwp_native().unwrap();
        assert!(core
            .insert_picture_in_cell_native(
                0, para, 0, &path, &PNG, 5000, 3000, 1, 1, "png", "서명", None, None
            )
            .is_err());
        assert_eq!(
            core.export_hwp_native().unwrap(),
            before,
            "실패한 경로 {path:?}는 BinData·문단·저장 메타데이터를 바꾸지 않는다"
        );
    }
}

#[test]
fn each_cell_picture_offset_defaults_independently() {
    for (x, y) in [(Some(1200), None), (None, Some(-500)), (None, None)] {
        let (mut core, para, control) = blank_cell_core();
        core.insert_picture_in_cell_native(
            0,
            para,
            0,
            &[(control, 0, 0)],
            &PNG,
            5000,
            3000,
            1,
            1,
            "png",
            "서명",
            x,
            y,
        )
        .unwrap();
        assert_eq!(
            cell_picture(core.document(), "독립 오프셋"),
            (
                HorzRelTo::Column,
                VertRelTo::Para,
                x.unwrap_or(0),
                y.unwrap_or(0),
                false,
                true
            )
        );
    }
}
