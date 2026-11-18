//! [#8755 #8756] 관계형 회귀 — 병합 칸만 덮는 행은 걸친 칸이 요구하는 만큼만 차지하고(요구가
//! 없으면 0), 칸 문단의 자리차지 표가 자기 앵커 줄을 민 저장 형상이면 표는 문단 시작에 놓이고
//! 칸 세로 정렬은 저장 내용 묶음 전체에 적용된다.
//!
//! 주석 갱신: 2026-11-10 23:10 KST(+09:00). 직전 파일 변경 커밋 `f736da3d46`(2026-11-08 12:38 +09:00)을
//! devel `5ea1227505` 위 번호순 사슬(#8758 head `6159174e75` 다음)로 옮기고 다시 측정했다.
//!
//! - 입력: `samples/issue8755/46551_road_traffic_rule_amendment.hwp`(경찰청 입법예고안, HWP5, 30쪽).
//! - 독립 기준: `pdf/issue8755/46551_road_traffic_rule_amendment-2020.pdf`(한/글 2022 Print, 30쪽).
//! - 28쪽 32×17 표(구역 2 문단 13): rs=1 칸이 없는 행 16·17·19·24·26 을 한/글은 0.0~0.3px 로 그린다
//!   — 오른쪽 열 행 18·20·25·27 괘선 376.7/398.9/463.5/505.0px 이 왼쪽 열 행 16·19·24·26 괘선
//!   376.7/398.6/463.3/504.9px 과 같은 높이다. 수정 전 rhwp(devel `5077b216cb`)는 이 행들에
//!   10~20px 를 넣어 오른쪽 열이 최대 20.6px 아래(행 27 525.1px), 표 끝이 842.5px(한/글 821.9px).
//! - 13쪽 20×8 표 r17 c3 칸: 3×8 자리차지 표(문단 기준, 오프셋 0, 높이 5749HU, 바깥 여백 141HU)가
//!   둘째 문단의 첫 줄을 밀어 저장했다(첫 줄 vpos 20269 = 문단 시작 14238 + 표 틀 6031). 한/글은 표를
//!   문단 시작에 두고(「위 금액을 수납 의뢰합니다」 글자 위끝 787.9px), 칸 세로 가운데 정렬을 저장 끝
//!   21269HU 에 적용한다(10×10 표 위끝 593.9px). 수정 전 rhwp 는 3×8 표를 밀린 줄(849.6px)에 놓아
//!   칸 끝에서 잘렸고(높이 40.1px, 「은행(우체국)」 행 소실) 10×10 표는 칸 내용 위끝(581.2px)에 붙었다.
//!
//! 측정: 2026-11-10, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 문서 face 만 모은 글꼴 폴더 공급.
//! 소스는 devel `5ea1227505` + 앞 번호 PR 사슬 + 이 수정(코드 커밋 기준). 13쪽 Native 99.79 / fresh
//! WASM 99.78, 28쪽 Native 99.77 / fresh WASM 99.75(수정 전 devel `5ea1227505` Native 87.77 / 70.20).
//! 전체 30쪽 = PDF 30쪽, 전쪽 최저 Native 96.88(1쪽)·fresh WASM 99.74(7·29쪽).
//! 증거: `mydocs/pr/assets/issue_8755/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사는 행 경계의 열 사이 일치·걸친 칸 높이와 rs=1 선언 합의 관계(관계형), 3×8 표와 10×10 표의
//! 간격이 저장 문단 시작의 차와 같음·3×8 표가 잘리지 않음·칸 위아래 여백이 같음(관계형)이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8755/46551_road_traffic_rule_amendment.hwp";
const DPI: f64 = 96.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn find_table(node: &RenderNode, rows: u16, cols: u16) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.row_count == rows && table.col_count == cols {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, rows, cols))
}

fn cell(table: &RenderNode, row: u16, col: u16) -> &RenderNode {
    table
        .children
        .iter()
        .find(|child| {
            matches!(&child.node_type, RenderNodeType::TableCell(c) if c.row == row && c.col == col)
        })
        .unwrap_or_else(|| panic!("칸 r{row} c{col}"))
}

fn bottom(b: &BoundingBox) -> f64 {
    b.y + b.height
}

#[test]
fn rows_covered_only_by_span_cells_take_no_extra_height() {
    let core = load();
    let page = core.build_page_render_tree(27).expect("28쪽 렌더 트리");
    let table = find_table(&page.root, 32, 17).expect("28쪽 32×17 표");
    let top = |row, col| cell(table, row, col).bbox.y;

    // rs=1 칸이 없는 행은 걸친 칸이 요구하지 않으므로 두 열의 경계가 같은 높이에 놓인다.
    for (right, left, rows) in [
        ((18, 12), (16, 1), "16·17"),
        ((20, 13), (19, 3), "19"),
        ((25, 13), (24, 3), "24"),
        ((27, 13), (26, 1), "26"),
    ] {
        let (r, l) = (top(right.0, right.1), top(left.0, left.1));
        assert!(
            (r - l).abs() <= 1.0,
            "행 {rows} 는 높이가 없다: 오른쪽 r{} {r:.1} = 왼쪽 r{} {l:.1}",
            right.0,
            left.0
        );
    }

    // 행 27~30 은 rs=1 선언 그대로이고, 행 26~30 에 걸친 칸(선언 23738HU)은 그 합을 따른다.
    let doc = core.document();
    let model = doc.sections[2].paragraphs[13]
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("구역 2 문단 13 표");
    let declared = |row: u16| {
        model
            .cells
            .iter()
            .filter(|c| c.row == row && c.row_span == 1)
            .map(|c| c.height as i32)
            .max()
            .expect("rs=1 칸")
    };
    let rows_27_30: i32 = (27..=30).map(declared).sum();
    let span = cell(table, 26, 11).bbox.clone();
    let rows_box = bottom(&cell(table, 30, 12).bbox) - top(27, 13);
    assert!(
        (rows_box - px(rows_27_30)).abs() <= 1.0,
        "행 27~30 {rows_box:.1} = rs=1 선언 합 {:.1}",
        px(rows_27_30)
    );
    assert!(
        (span.height - rows_box).abs() <= 1.0,
        "행 26~30 걸친 칸 {:.1} = 행 27~30 합 {rows_box:.1}(행 26 은 0)",
        span.height
    );
}

#[test]
fn cell_table_pushing_its_own_anchor_line_sits_at_paragraph_start() {
    let core = load();
    let page = core.build_page_render_tree(12).expect("13쪽 렌더 트리");
    let outer = find_table(&page.root, 20, 8).expect("13쪽 20×8 표");
    let host = cell(outer, 17, 3);
    let upper = find_table(host, 10, 10).expect("10×10 표");
    let lower = find_table(host, 3, 8).expect("3×8 표");

    // 저장 근거: 문단 0 첫 줄의 TAC 10×10 표(바깥 여백 283HU)와 문단 1 시작(14238HU)의 3×8 표
    // (바깥 여백 141HU). 두 표 위끝의 차는 저장 문단 시작의 차와 같다.
    let doc = core.document();
    let model_cell = match &doc.sections[2].paragraphs[0].controls[2] {
        rhwp::model::control::Control::Table(t) => t
            .cells
            .iter()
            .find(|c| c.row == 17 && c.col == 3)
            .expect("r17 c3"),
        _ => panic!("구역 2 문단 0 의 셋째 개체는 표"),
    };
    let first = &model_cell.paragraphs[0].line_segs[0];
    let para1_start = first.vertical_pos + first.line_height + first.line_spacing;
    let expected_gap = px(para1_start + 141 - 283);
    let gap = lower.bbox.y - upper.bbox.y;
    assert!(
        (gap - expected_gap).abs() <= 1.0,
        "3×8 표는 문단 시작에 놓인다: 두 표 위끝 차 {gap:.1} = 저장 {expected_gap:.1}"
    );

    // 3×8 표는 잘리지 않고 칸 안에 있다.
    assert!(
        (lower.bbox.height - px(5749)).abs() <= 1.0,
        "3×8 표 높이 {:.1} = 선언 {:.1}",
        lower.bbox.height,
        px(5749)
    );
    assert!(bottom(&lower.bbox) <= bottom(&host.bbox) + 0.5);

    // 세로 가운데 정렬은 저장 내용 묶음(마지막 줄 끝)에 적용된다: 위아래 남는 공간이 같다.
    let last = model_cell
        .paragraphs
        .last()
        .unwrap()
        .line_segs
        .last()
        .unwrap();
    let pad = px(i32::from(model_cell.padding.top));
    let content_top = upper.bbox.y - px(283);
    let content_end = content_top + px(last.vertical_pos + last.line_height);
    let above = content_top - (host.bbox.y + pad);
    let below = bottom(&host.bbox) - pad - content_end;
    assert!(
        (above - below).abs() <= 1.0,
        "가운데 정렬: 위 여백 {above:.1} = 아래 여백 {below:.1}"
    );
}
