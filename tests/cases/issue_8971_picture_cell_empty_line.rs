//! 관계형·고정형 — [#8971] 저장 줄 없는 그림 칸의 빈 문단 자리 줄은 그림 행 높이에 더해지지 않는다.
//! 주석 갱신: MEASURE_PENDING
//!
//! 입력: 강동구 「2026 강동 환경 한마당 행사 결과보고」(한/글 2020 저장, 6쪽)의 그림 25개를 같은
//! 비율의 회색 단색 PNG 로만 바꾼 축소본 `samples/issue8971/36400701-picture-cell-empty-line.hwpx`.
//! XML 은 원본과 바이트가 같고 rhwp 배치 좌표도 6쪽 모두 원본과 같다.
//! 6쪽 「행사 사진」 표(4행 3열, TAC, 저장 줄 높이 23393HU)의 0·2행 칸은 글자 없는 문단 하나에
//! 위아래 배치(문단 기준) 그림 하나만 있고 저장 줄이 없다. 문단 글자 모양은 10pt·11pt 다.
//! 한/글은 그림 행을 칸 위 여백 + 세로 오프셋 + 그림 + 칸 아래 여백으로 잡는다(0행: 141 + 225 +
//! 9637 + 141 HU). 수정 전 rhwp 는 빈 문단 자리 줄(글자 크기 em)에서 400HU 만 빼 남은 몫
//! (8.0px·9.3px)만큼 그림 행을 늘려 표가 329.2px 이 되고 아래 내용이 밀렸다.
//! - 고정형: 전체 쪽수 6(= PDF).
//! - 관계형: 6쪽 사진 표 높이 = 표 선언 높이(23393HU, 저장 줄 높이와 같음), 0행 높이 = 칸 위·아래
//!   여백 + 세로 오프셋 + 그림 높이(±1px). 원본 저장값에서 정한 값이다.
//!
//! MEASURE_PENDING
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue8971/36400701-picture-cell-empty-line.hwpx";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 6;
/// 사진 표가 있는 쪽(0 기반).
const PHOTO_PAGE: u32 = 5;
/// 사진 표 선언 높이 = 표 문단의 저장 줄 높이.
const TABLE_HEIGHT_HU: i32 = 23393;
/// 0행: 칸 위 여백 141 + 세로 오프셋 225 + 그림 9637 + 칸 아래 여백 141.
const PICTURE_ROW_HU: i32 = 141 + 225 + 9637 + 141;

fn px(hu: i32) -> f64 {
    f64::from(hu) / 75.0
}

fn tables(node: &RenderNode, out: &mut Vec<RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node.clone());
        return;
    }
    for child in &node.children {
        tables(child, out);
    }
}

#[test]
fn empty_picture_cell_line_does_not_grow_picture_row() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    let tree = core
        .build_page_render_tree(PHOTO_PAGE)
        .expect("render tree");
    let mut found = Vec::new();
    tables(&tree.root, &mut found);
    // 사진 표는 이 쪽에서 그림을 담은 유일한 표다.
    let photo: Vec<&RenderNode> = found
        .iter()
        .filter(|t| {
            fn has_image(n: &RenderNode) -> bool {
                matches!(n.node_type, RenderNodeType::Image(_)) || n.children.iter().any(has_image)
            }
            has_image(t)
        })
        .collect();
    assert_eq!(photo.len(), 1, "6쪽 사진 표는 하나: {}", found.len());
    let table = photo[0];
    assert!(
        (table.bbox.height - px(TABLE_HEIGHT_HU)).abs() <= 1.0,
        "사진 표 높이는 선언(저장 줄) 높이 {:.2}px 여야 한다: {:.2}",
        px(TABLE_HEIGHT_HU),
        table.bbox.height
    );

    let first_row: Vec<&RenderNode> = table
        .children
        .iter()
        .filter(|c| {
            matches!(c.node_type, RenderNodeType::TableCell(_))
                && (c.bbox.y - table.bbox.y).abs() <= 0.5
        })
        .collect();
    assert_eq!(first_row.len(), 3, "0행 칸 3개");
    for cell in first_row {
        assert!(
            (cell.bbox.height - px(PICTURE_ROW_HU)).abs() <= 1.0,
            "0행 그림 칸 높이는 여백 + 오프셋 + 그림 {:.2}px 여야 한다: {:.2}",
            px(PICTURE_ROW_HU),
            cell.bbox.height
        );
    }
}
