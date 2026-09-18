//! [#7203] 자리차지 표의 윗변이 **앵커 문단의 저장 자리**에 놓인다 — 앞 문단 글자를 뚫지 않는다.
//!
//! `samples/hwpctl_API_v2.4.hwp` 28쪽(0-based 27)의 코드 상자 표는 빈 host 문단
//! `pi=574`(저장 `vpos=40693`)에 매달린 `wrap=자리차지 · vert=문단` 표다. 이 쪽의 글줄은
//! 전부 저장 사다리를 따른다 — 예로 `pi=573` 은 `vpos=38893` → `y=650.84`(±0.1px).
//! 같은 사다리로 환산하면 host 의 자리는 **674.8px** 이다.
//!
//! ```text
//!   수정 전  표 윗변 661.51  = 앵커 저장 자리 − 줄 높이(1000HU = 13.33px)
//!                            앞 문단 pi=573 줄 상자 650.8 .. 664.1 을 관통한다
//!   한/글    표 윗변 671.27  (pdf/hwpctl_API_v2.4-hwp-2020.pdf 가로 괘선 실측)
//!   수정 후  표 윗변 674.8   = 앵커 저장 자리 (정본과 바깥여백 283HU = 3.77px 안)
//! ```
//!
//! 막고 있던 것은 `stored_ladder_leaves_object_room` 의 필요 공간 산식이었다. 앵커 아래에
//! `높이 + 위여백 + 아래여백` 을 요구했는데, 저장 사다리의 간격은 `10948HU` 로 그보다
//! `501HU` 작아 저장 anchor 경로가 통째로 꺼졌다. 정본은 이 표의 윗변을 앵커보다 **위여백
//! 한 개 위**에 두므로 앵커 아래로 필요한 공간은 `높이 + 아래여백 − 위여백`(= 대칭 여백이면
//! 높이)이고, 그 값으로는 `10882 ≤ 10948` 로 들어간다.
//!
//! 이 검사는 "표가 앞 문단 글자를 뚫지 않는다"와 "정본과 바깥여백 안"이라는 두 계약만
//! 고정한다 — 남은 ±바깥여백 무리는 이슈에 미해결로 남겨 둔다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/hwpctl_API_v2.4.hwp";
/// 한/글 2020 PDF 실측 (가로 괘선, 폭 427.7px).
const ORACLE_TABLE_TOP: f64 = 671.27;
/// 수정 전 관측값 — 앵커 저장 자리에서 줄 높이만큼 위.
const BEFORE_FIX_TABLE_TOP: f64 = 661.51;

fn collect(node: &RenderNode, tables: &mut Vec<(f64, f64)>, host_line: &mut Option<(f64, f64)>) {
    match &node.node_type {
        RenderNodeType::Table(_) if (node.bbox.width - 427.7).abs() < 1.5 => {
            tables.push((node.bbox.y, node.bbox.height));
        }
        RenderNodeType::TextLine(line) => {
            if line.para_index == Some(573) && host_line.is_none() {
                *host_line = Some((node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, tables, host_line);
    }
}

#[test]
fn para_float_table_top_sits_at_the_stored_anchor_not_a_line_above() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {FIXTURE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {FIXTURE}: {e:?}"));
    let tree = doc
        .build_page_render_tree(27)
        .unwrap_or_else(|e| panic!("render tree 28쪽: {e:?}"));

    let mut tables = Vec::new();
    let mut host_line = None;
    collect(&tree.root, &mut tables, &mut host_line);

    let (line_top, line_bottom) = host_line.expect("앞 문단(pi=573) 줄을 찾지 못했다");
    let table_top = tables
        .iter()
        .map(|(y, _)| *y)
        .find(|y| *y > line_top)
        .expect("28쪽에서 코드 상자 표를 찾지 못했다");

    assert!(
        table_top >= line_bottom,
        "표 윗변 {table_top:.2} 가 앞 문단 줄 상자({line_top:.2}..{line_bottom:.2})를 뚫는다 \
         (수정 전 {BEFORE_FIX_TABLE_TOP:.2})"
    );
    // 한/글은 이 표의 윗변을 앵커보다 바깥여백(283HU = 3.77px) 한 개 위에 둔다.
    // 우리는 앵커 자리에 두므로 그 한 개 폭 안에서만 어긋난다.
    let delta = table_top - ORACLE_TABLE_TOP;
    assert!(
        delta.abs() <= 4.2,
        "표 윗변 {table_top:.2} 가 정본 {ORACLE_TABLE_TOP:.2} 에서 바깥여백 한 개를 넘어 \
         어긋난다 (차 {delta:+.2}px, 수정 전 {:+.2}px)",
        BEFORE_FIX_TABLE_TOP - ORACLE_TABLE_TOP
    );
}
