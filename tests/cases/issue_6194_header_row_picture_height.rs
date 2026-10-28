//! [Issue #6194] 머리 표 행을 선언 55.7px 대신 89.4px 로 키워 아래 표와 34px 겹치고
//! 괘선이 글자를 관통한다 (156494392 1쪽).
//!
//! 근인: 기관명 칸의 저장 사다리는 그림의 밀림을 **다음 문단의 vpos** 에 적어 뒀다.
//!
//! ```xml
//! <hp:p><hp:run>[PIC h=2906HU, TopAndBottom]</hp:run>
//!   <hp:lineseg vertpos="0"    vertsize="1200"/>
//! <hp:p><hp:t>국립농산물품질관리원</hp:t>
//!   <hp:lineseg vertpos="2906" vertsize="1000"/>     <!-- = 그림 높이 -->
//! ```
//!
//! `#2226` 의 `trust_stored` 는 "개체를 단 문단 **자신의** vpos > 0" 만 증인으로 봐서
//! 이 모양을 놓치고, 그림 높이를 줄높이 합에 다시 더했다(측정 content 85.7px 대
//! 저장 사다리 52.1px). 증인을 "개체 문단 뒤의 줄이 개체 바닥까지 내려와 있는가"로
//! 넓히면 사다리를 신뢰해 행이 선언 높이로 돌아온다.
//!
//! 한글 2020 실측: 머리 표 하단 165.9px, 높이 55.8px.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6194/156494392_agri_press_release.hwpx";
/// 결함이 나타나는 쪽(0-based).
const PAGE: u32 = 0;
/// 머리 표 3칸의 선언 높이 최대값 = 4178 HU.
const DECLARED_PX: f64 = 55.71;

#[test]
fn issue_6194_header_row_keeps_declared_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 1 render tree");
    let mut tables = Vec::new();
    collect_tables(&page.root, &mut tables);
    assert!(
        tables.len() >= 2,
        "1쪽에 표가 둘 이상 있어야 한다: {}",
        tables.len()
    );
    tables.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let (head_top, head_h) = tables[0];
    let (next_top, _) = tables[1];

    // 행이 선언 높이로 돌아온다 — 종전 89.40px.
    assert!(
        head_h < DECLARED_PX + 2.0,
        "머리 표 높이가 선언({DECLARED_PX:.1}px)보다 크게 부풀었다: {head_h:.1}px"
    );
    // 아래 표와 겹치지 않는다 — 종전 34.1px 겹침(괘선이 글자를 관통).
    assert!(
        head_top + head_h <= next_top + 0.5,
        "머리 표 하단({:.1})이 다음 표 상단({next_top:.1})을 덮는다",
        head_top + head_h,
    );
}

/// 최상위 `Table` 노드의 (top, height) 목록.
fn collect_tables(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push((node.bbox.y, node.bbox.height));
        return;
    }
    for child in &node.children {
        collect_tables(child, out);
    }
}
