// Issue #7441 — 꼬리말 영역까지 뻗은 용지 기준 표의 경계 확인 probe (회귀 테스트 아님).
//
// 입력: samples/[2027] 온새미로 1 본교재.hwp (실제 저장본)
// 기준: pdf/[2027] 온새미로 1 본교재-hwp-2020.pdf (저장 제품 hancom-office-2020 → 2020 기준),
//       참고 pdf/[2027] 온새미로 1 본교재-2024.pdf (Hwp 2024 13.0.0.3622)
//
// 1쪽 표지 틀(s0 p3 c0: 2행×1열, 글 뒤로, 용지 기준 위 정렬, 용지 36mm, 높이 247mm → 283mm)은
// 본문 끝 267mm 를 넘어 꼬리말 영역까지 뻗는다. 한컴은 2행(빈 문단 64.2px)을 다음 쪽으로
// 넘기지 않는다: 2쪽 첫 줄 `MEMO` 상단이 85.35pt(2024)·84.93pt(2020) ≈ 본문 시작 113.4px.
//
// 관측(2026-09-27, base eb9142dd7): 이슈 제안대로 용지 좌표로 바로잡고 꼬리말을 넘는 표만
// 자르면 표지 틀 2행이 2쪽 잔여 조각으로 넘어가고, 잔여 행 자리 예약 때문에 2쪽 `MEMO` 가
// 64.2px 밀린다(177.6px). 컷을 문단 기준 표로 한정한 최종 수정은 수정 전과 같이 113.4px.
//
// 이 문서는 같은 원본의 Native/fresh WASM 실루엣 일치율이 1쪽 89.90%(표지 제목 글꼴 차이,
// 수정 전후 동일)라 정식 회귀 추가 조건(관련 쪽 최저 90%)을 충족하지 못해 probe 로 둔다.
//
// 출력: 1쪽·2쪽의 표지 틀 행 목록, 표지 틀 하단과 꼬리말 시작, 2쪽 `MEMO` 상단(stdout).
// 실행: 루트 Cargo.toml 에 등록된 example 이 아니다. 필요하면 임시 [[example]] 로 등록해
// `cargo run --release --example check_7441_footer_area_cover` 로 실행하고 등록은 되돌린다.
use std::collections::BTreeSet;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SRC: &str = "samples/[2027] 온새미로 1 본교재.hwp";
const COVER: (usize, usize) = (3, 0);

fn cover_rows(node: &RenderNode, rows: &mut BTreeSet<u16>, bottom: &mut f64, inside: bool) {
    let is_cover = matches!(&node.node_type, RenderNodeType::Table(t)
        if t.cell_context.is_none() && t.para_index == Some(COVER.0) && t.control_index == Some(COVER.1));
    if is_cover {
        *bottom = bottom.max(node.bbox.y + node.bbox.height);
    }
    let inside = inside || is_cover;
    if inside {
        if let RenderNodeType::TableCell(cell) = &node.node_type {
            rows.extend(cell.row..cell.row + cell.row_span.max(1));
            return;
        }
    }
    for child in &node.children {
        cover_rows(child, rows, bottom, inside);
    }
}

fn text_top(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(tr) = &node.node_type {
        if tr.text.contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| text_top(child, needle))
}

fn main() {
    let bytes = std::fs::read(SRC).expect("read source");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse source");
    let def: serde_json::Value =
        serde_json::from_str(&doc.get_page_def_native(0).expect("page def")).expect("json");
    let hu = |k: &str| def[k].as_i64().expect("page def field") as f64 * 96.0 / 7200.0;
    let paper_h = if def["landscape"].as_bool().unwrap_or(false) {
        hu("width")
    } else {
        hu("height")
    };
    println!(
        "body top {:.1}px, footer start {:.1}px",
        hu("marginHeader") + hu("marginTop"),
        paper_h - hu("marginFooter") - hu("marginBottom")
    );

    for page in 0..2 {
        let tree = doc.build_page_render_tree(page).expect("render tree");
        let mut rows = BTreeSet::new();
        let mut bottom = 0.0;
        cover_rows(&tree.root, &mut rows, &mut bottom, false);
        println!(
            "page {}: cover rows {:?}, cover bottom {:.1}px",
            page + 1,
            rows,
            bottom
        );
    }
    let tree = doc.build_page_render_tree(1).expect("render tree p2");
    println!(
        "page 2: MEMO top {:?}px",
        text_top(&tree.root, "MEMO").map(|y| (y * 10.0).round() / 10.0)
    );
}
