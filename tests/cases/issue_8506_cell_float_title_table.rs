//! [#8506] 관계형 회귀 — 칸의 빈 호스트 문단(`[단 정의][제목 표][빈 글자처럼 취급 그림]`)에
//! 문단 기준 자리차지로 놓인 제목 표는 저장 오프셋 프레임으로 칸 내용과 행 높이를 정한다.
//!
//! 주석 갱신: 2026-11-27 18:30 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8506/36410942_approval_cover_title.hwpx` — 공개 결재문서 36410942
//!   (한/글 2020 11.0.0.8227 저장 HWPX, 2쪽). 1쪽 바깥 4×1 글자처럼 취급 표의 r=1 칸(세로 가운데,
//!   선언 17771HU, 안 여백 141HU)의 첫 문단이 빈 호스트이고, 제목 1×1 표(높이 7296HU, 바깥 여백
//!   138HU)는 그 문단 기준 오프셋 10450HU 에 놓인다. 저장 호스트 줄(4125HU)은 표 위에서 끝난다.
//! - 독립 기준: `pdf/issue8506/36410942_approval_cover_title-2020.pdf`(원격 변환 서버 한/글 2020
//!   엔진 Print → Hancom PDF, 2쪽). 제목 표 위 빨간 괘선 381–385px, 아래 파란 괘선 478–482px.
//!   바깥 표 상자 `sz height` 74706HU = 선언 행 합 + 533HU — r=1 칸이 표 띠(오프셋 + 바깥 여백
//!   위·아래 + 본체 + 안 여백 = 18304HU)로 자란 만큼이다.
//!
//! 측정: 2026-11-27, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`
//! (`silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), base devel `3e97980785`.
//!
//! | 쪽 | Native base | Native head | fresh WASM base | fresh WASM head |
//! |---|---|---|---|---|
//! | 1 | 51.19 | 98.47 | 51.63 | 98.59 |
//! | 2 | 97.65 | 97.65 | 98.20 | 98.20 |
//!
//! 전체 2쪽 = PDF 2쪽. head 전쪽 최저 Native 97.65(2쪽)·fresh WASM 98.20(2쪽). Native 2쪽은 수정 전후
//! 같은 값이며 배치·줄바꿈 일치, 글꼴(`HY신명조`·`HY견고딕`·`HY중고딕`·`휴먼명조` 미공급) 차이만 남는다.
//! 증거: `mydocs/pr/assets/issue_8506/{native,wasm,base-native,base-wasm}/`.
//!
//! 검사(관계형): 제목 표 위끝 = 칸 위 + 안 위 여백 + 오프셋 + 바깥 위 여백, 제목 표 높이 = 선언,
//! 제목 표와 바깥 여백·안 아래 여백이 칸 안에 담김, 다음 칸은 r=1 칸 바로 뒤, 바깥 표 높이 = 저장
//! 표 상자 높이, 제목 두 줄이 제목 표 안에 차례로 놓임.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::Table;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8506/36410942_approval_cover_title.hwpx";
/// 한/글 2020 Print 쪽수.
const HANCOM_PAGES: u32 = 2;
const DPI: f64 = 96.0;
const TOLERANCE_PX: f64 = 1.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), HANCOM_PAGES, "한/글 2020 Print 와 같은 쪽수");
    core
}

fn find_table(node: &RenderNode, rows: u16, cols: u16, width: f64) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.row_count == rows
            && table.col_count == cols
            && (node.bbox.width - width).abs() <= 1.0
        {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, rows, cols, width))
}

fn cell_of<'a>(table: &'a RenderNode, row: u16) -> &'a RenderNode {
    table
        .children
        .iter()
        .find(|child| matches!(&child.node_type, RenderNodeType::TableCell(cell) if cell.row == row))
        .unwrap_or_else(|| panic!("바깥 표 r={row} 칸"))
}

fn text_runs<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            out.push((node, run.text.clone()));
        }
    }
    for child in &node.children {
        text_runs(child, out);
    }
}

/// 모델: 바깥 4×1 표와 r=1 칸의 제목 표.
fn model(core: &DocumentCore) -> (&Table, &Table) {
    let doc = core.document();
    let outer = doc.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.row_count == 4 && table.col_count == 1 => Some(table),
            _ => None,
        })
        .expect("바깥 4×1 표");
    let cell = outer.cells.iter().find(|cell| cell.row == 1).expect("r=1 칸");
    let host = &cell.paragraphs[0];
    let title = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("제목 표");
    // 입력 조건: 빈 호스트에 단 정의·제목 표·글자처럼 취급 그림이 함께 있다.
    assert!(host.text.chars().all(|ch| ch.is_whitespace() || ch.is_control()));
    assert!(host.controls.iter().any(|c| matches!(c, Control::ColumnDef(_))));
    assert!(host
        .controls
        .iter()
        .any(|c| matches!(c, Control::Picture(p) if p.common.treat_as_char)));
    assert!(!title.common.treat_as_char, "제목 표는 문단 기준 자리차지");
    (outer, title)
}

#[test]
fn cover_title_table_sits_at_its_paragraph_offset_and_grows_the_row() {
    let core = load();
    let (outer, title) = model(&core);
    let cell_model = outer.cells.iter().find(|cell| cell.row == 1).unwrap();
    let padding = cell_model.effective_padding(&outer.padding);

    let page = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let outer_node = find_table(&page.root, 4, 1, px(outer.common.width as i32)).expect("바깥 표");
    let title_node =
        find_table(&page.root, 1, 1, px(title.common.width as i32)).expect("제목 1×1 표");
    let cell1 = cell_of(outer_node, 1);
    let cell2 = cell_of(outer_node, 2);

    // 제목 표 위끝 = 칸 위 + 안 위 여백 + 문단 오프셋 + 바깥 위 여백(칸 세로 가운데로 밀리지 않음).
    let expected_top = cell1.bbox.y
        + px(i32::from(padding.top))
        + px(title.common.vertical_offset as i32)
        + px(i32::from(title.outer_margin_top));
    assert!(
        (title_node.bbox.y - expected_top).abs() <= TOLERANCE_PX,
        "제목 표 위끝 {:.1} = 칸 위 {:.1} + 안 여백 + 오프셋 + 바깥 여백 = {expected_top:.1}",
        title_node.bbox.y,
        cell1.bbox.y
    );
    // 제목 표 높이는 선언(잘리지 않음).
    let declared = px(title.common.height as i32);
    assert!(
        (title_node.bbox.height - declared).abs() <= TOLERANCE_PX,
        "제목 표 높이 {:.1} = 선언 {declared:.1}",
        title_node.bbox.height
    );
    // 표 띠(바깥 아래 여백 + 안 아래 여백까지)가 r=1 칸 안에 담기고, 다음 칸은 그 바로 뒤다.
    let band_bottom = title_node.bbox.y
        + title_node.bbox.height
        + px(i32::from(title.outer_margin_bottom))
        + px(i32::from(padding.bottom));
    let cell1_bottom = cell1.bbox.y + cell1.bbox.height;
    assert!(
        band_bottom <= cell1_bottom + TOLERANCE_PX,
        "제목 표 띠 끝 {band_bottom:.1} ≤ r=1 칸 끝 {cell1_bottom:.1}"
    );
    assert!(
        (cell2.bbox.y - cell1_bottom).abs() <= 0.5,
        "r=2 칸 {:.1} 은 r=1 칸 끝 {cell1_bottom:.1} 바로 뒤",
        cell2.bbox.y
    );
    // 바깥 표 높이 = 한/글이 저장한 표 상자 높이(자란 r=1 행을 포함한 합).
    let stored_box = px(outer.common.height as i32);
    assert!(
        (outer_node.bbox.height - stored_box).abs() <= TOLERANCE_PX,
        "바깥 표 높이 {:.1} = 저장 상자 {stored_box:.1}",
        outer_node.bbox.height
    );

    // 제목 두 줄은 제목 표 안에 위아래로 놓인다.
    let mut runs = Vec::new();
    text_runs(title_node, &mut runs);
    let first = runs
        .iter()
        .find(|(_, text)| text.contains("의무운행기간"))
        .expect("제목 첫 줄");
    let second = runs
        .iter()
        .find(|(_, text)| text.contains("검토보고"))
        .expect("제목 둘째 줄");
    for (node, text) in [first, second] {
        assert!(
            node.bbox.y >= title_node.bbox.y - 0.5
                && node.bbox.y + node.bbox.height <= title_node.bbox.y + title_node.bbox.height + 0.5,
            "제목 줄 {text:?} 은 제목 표 안"
        );
    }
    assert!(
        second.0.bbox.y >= first.0.bbox.y + first.0.bbox.height - 0.5,
        "둘째 줄은 첫 줄 아래"
    );
}
