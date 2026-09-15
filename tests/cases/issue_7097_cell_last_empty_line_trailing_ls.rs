#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7097] 칸의 **마지막 줄이 빈 줄**일 때 그 줄의 trailing 줄간격이 칸 높이에
//! 들어가 행이 8px 과대해지던 결함의 가드.
//!
//! ## 계약
//!
//! `#5923` 이 비-TAC 표에서는 칸 마지막 줄의 trailing 을 문단 수와 무관하게 제외했고,
//! TAC 표의 다문단 칸만 보존 핀(`Task #874/#1086`)을 위해 포함 회계를 남겼다. `#6681` 이
//! 그 예외에서 「개체만 있는 마지막 줄」을 뺐다 — 근거는 "그 줄 뒤에 붙일 줄이 없다" 였다.
//! 그 근거는 **글자가 아예 없는 빈 마지막 줄**에 그대로 적용된다.
//!
//! ## 실측 — `samples/issue2470/36382471_masked.hwpx` 1쪽
//!
//! 바깥 표의 2행 칸은 마지막 문단이 글자도 개체도 없는 빈 문단(`lh=1000 sp=600`)이다.
//!
//! ```text
//!   표 선언 총높이        68562 HU = 914.16px
//!   2행 = 저장 사다리 끝(25372 HU = 338.29) + 칸 여백 282 HU(3.76) = 342.05px
//!         → 행 합 201.09 + 221.85 + 342.05 + 149.16 = 914.15 로 표 선언과 일치
//!
//!                       수정 전    수정 후    한/글
//!     2행 높이           350.10    342.10    342.05
//!     3행 상단           890.10    882.10    882.09  (위 합에서 유도)
//!     「중랑물재생센터」     920.30    912.60    912.26  (정본 PDF 직접 측정)
//! ```
//!
//! 정본은 `pdf/issue2470/36382471_masked-2022.pdf`(`Hwp 2022 12.0.0.4547`,
//! `Hancom PDF 1.3.0.550`)다. 2행 칸이 `vertAlign="CENTER"` 라 과대분의 **절반**이 그
//! 칸 안 글자에, **전량**이 3행 아래로 나타났다.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{PageRenderTree, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue2470/36382471_masked.hwpx";

/// 정본과의 남은 차(0.4px 안)는 담고 수정 전(8.0px)은 분명히 거르는 폭.
const TOL: f64 = 1.5;

fn page0() -> PageRenderTree {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#7097 공개 fixture 읽기 {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes)
        .expect("문서 로드")
        .build_page_render_tree(0)
        .expect("1쪽 render tree")
}

fn collect(node: &RenderNode, out: &mut Vec<(String, f64, f64, f64)>) {
    let label = match &node.node_type {
        RenderNodeType::TableCell(_) => Some("Cell".to_string()),
        RenderNodeType::TextRun(run) => Some(format!("TextRun:{}", run.text)),
        _ => None,
    };
    if let Some(label) = label {
        out.push((label, node.bbox.y, node.bbox.height, node.bbox.width));
    }
    for child in &node.children {
        collect(child, out);
    }
}

/// 바깥 표(폭 653.9)의 행들을 위에서부터 — 안쪽 띠 표(623.2)와 폭으로 갈린다.
fn outer_rows(all: &[(String, f64, f64, f64)]) -> Vec<(f64, f64)> {
    let mut rows: Vec<(f64, f64)> = all
        .iter()
        .filter(|(label, _, _, w)| label == "Cell" && *w > 640.0)
        .map(|(_, y, h, _)| (*y, *h))
        .collect();
    rows.sort_by(|a, b| a.partial_cmp(b).expect("정렬"));
    rows
}

/// 빈 마지막 줄의 trailing 줄간격이 행을 부풀리면 안 된다.
#[test]
fn cell_with_empty_last_line_does_not_reserve_trailing_spacing() {
    let tree = page0();
    let mut all = Vec::new();
    collect(&tree.root, &mut all);
    let rows = outer_rows(&all);
    assert_eq!(rows.len(), 4, "바깥 표는 4행이어야 한다 — {rows:?}");

    // `rowAddr=2` — 마지막 문단이 빈 문단인 칸이다.
    let (_, inflated_h) = rows[2];
    assert!(
        (inflated_h - 342.05).abs() < TOL,
        "rowAddr=2 행 높이가 {inflated_h} — 한/글 342.05, 수정 전 350.10"
    );
    let (below_y, _) = rows[3];
    assert!(
        (below_y - 882.09).abs() < TOL,
        "rowAddr=3 행 상단이 {below_y} — 한/글 882.09, 수정 전 890.10"
    );
}

/// 그 아래 흐름이 통째로 밀리지 않아야 한다 — 정본 PDF 에서 직접 잰 좌표다.
#[test]
fn content_below_the_inflated_row_returns_to_the_oracle() {
    let tree = page0();
    let mut all = Vec::new();
    collect(&tree.root, &mut all);
    let (_, y, _, _) = all
        .iter()
        .find(|(label, _, _, _)| label.starts_with("TextRun:중랑물재생센터"))
        .unwrap_or_else(|| panic!("「중랑물재생센터」 글줄을 찾지 못했다"));
    assert!(
        (y - 912.26).abs() < TOL,
        "「중랑물재생센터」가 {y} — 한/글 912.26, 수정 전 920.30"
    );
}
