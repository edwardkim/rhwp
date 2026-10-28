//! [Issue #3386] 저장 `cellSz height` 는 **여백 포함 행 높이**인데 측정기가 내용
//! 높이로 보고 셀 상하 안 여백을 덧붙여, 행마다 같은 상수가 붙고 표가 누적으로
//! 부풀던 결함의 가드.
//!
//! 실측 — `samples/issue1663_coanchored_float_orphan.hwpx` 2쪽 표(21행 2열):
//!
//! ```text
//! 선언  <hp:cellSz height="2800"/>            2800HU = 37.33px
//!       <hp:cellMargin top="141" bottom="141"/>  141+141 = 282HU = 3.76px
//!
//! 한/글 행 간격  37.3 / 37.2 / 37.4 …   (2020 · 2024 · 재생성 세 판 소수점까지 동일)
//! 종전 rhwp      41.09 균일 = 2800 + 282
//! ```
//!
//! 교정은 `trust_declared_row_heights` 의 **합 보존 가드**를 "전 행이 정확히
//! `선언 + 안 여백`" 인 서명일 때만 푸는 것이다. 그 서명이 아니면 종전대로 둔다.
//!
//! ⚠ **조각(fragment) 렌더 경로에서는 끈다.** 조각은 자식 내용을 감싸려고 노드를
//! `content_bottom` 까지 키우는데(`table_partial.rs`), 행을 줄이면 칸 내용이 정확히
//! `pad_top`(1.88px) 만큼 삐져나와 뒤 문단이 조각 바닥 위로 올라온다
//! (`issue_2439` 핀 실측). 분할되지 않은 표에는 그 성장 경로가 없고 실제로도 넘치지
//! 않는다 — 이 문서의 글자 baseline 은 행 바닥보다 9px 위다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue1663_coanchored_float_orphan.hwpx";
/// `cellSz height=2800HU`.
const DECLARED_ROW_PX: f64 = 2800.0 / 75.0;
/// `cellMargin top+bottom = 141+141 HU`.
const CELL_PAD_PX: f64 = 282.0 / 75.0;

fn collect_h_rules(node: &RenderNode, out: &mut Vec<f64>) {
    if let RenderNodeType::Line(l) = &node.node_type {
        if (l.y1 - l.y2).abs() < 0.5 {
            out.push(l.y1);
        }
    }
    for child in &node.children {
        collect_h_rules(child, out);
    }
}

/// 21행 표의 행 간격은 **선언 높이 그대로**여야 한다 — 안 여백을 덧붙이면 안 된다.
#[test]
fn declared_cell_height_already_includes_cell_padding() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core.build_page_render_tree(1).expect("2쪽 render tree");

    let mut ys = Vec::new();
    collect_h_rules(&page.root, &mut ys);
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    assert!(
        ys.len() >= 8,
        "21행 표의 가로 괘선이 있어야 한다 — 시험 설정 오류. ys={ys:?}"
    );

    let gaps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    // 표 앞뒤 여백 등 큰 간격은 빼고 행 간격만 본다.
    let row_gaps: Vec<f64> = gaps
        .into_iter()
        .filter(|g| *g > 20.0 && *g < 60.0)
        .collect();
    assert!(
        row_gaps.len() >= 6,
        "행 간격 표본이 모자라다 — 시험 설정 오류. row_gaps={row_gaps:?}"
    );

    let inflated = DECLARED_ROW_PX + CELL_PAD_PX;
    for g in &row_gaps {
        assert!(
            (g - DECLARED_ROW_PX).abs() <= 0.6,
            "행 간격은 **선언 높이**({DECLARED_ROW_PX:.2}px)여야 한다 — #3386 회귀. \
             got {g:.2} (안 여백을 덧붙이면 {inflated:.2}) row_gaps={row_gaps:?}"
        );
    }
}
