//! [Issue #6569] 세로 가운데 정렬 칸의 첫 문단 위 여백을 **정렬 공간에서 두 번 빼던**
//! 결함의 가드.
//!
//! `#6630` 이 "첫 문단 위 여백(`first_para_lead`)을 정렬 공간에서 뺀다" 는 계약을
//! 세웠는데, 그 계약은 **재조판 스택이 그 여백을 아직 안 품었을 때만** 옳다. 정렬은
//! 결국 *그려지는* 범위를 가운데에 두는 일이기 때문이다.
//!
//! 판별자는 저장 사다리가 준다 — `stored_flow_extent` 는 셀 내용 상단부터 마지막 줄
//! 바닥까지라 **첫 줄의 `vpos` 를 이미 포함**한다.
//!
//! ```text
//! 156678235 1쪽 제목 칸   ext 78.13 == content 78.13          → 스택이 품었다 → 빼면 안 됨
//! #6630 exam_eng 머리 칸  ext 45.37 vs content 37.80 (차=lead) → 스택이 뺐다   → 빼야 함
//! ```
//!
//! 한/글 2024 실측(제목 표): 표 상단 170.57pt, 제목 1행 글자 상단 188.25pt.
//! 종전 rhwp 186.03(−2.22pt = lead 6.67px 의 절반), 교정 후 188.53(+0.28).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6542/156678235_mid_para_vpos_rewind.hwp";

fn collect(node: &RenderNode, runs: &mut Vec<(f64, String)>, hlines: &mut Vec<f64>) {
    match &node.node_type {
        RenderNodeType::TextRun(r) => runs.push((node.bbox.y, r.text.clone())),
        RenderNodeType::Line(l) if (l.y1 - l.y2).abs() < 0.5 => hlines.push(l.y1),
        _ => {}
    }
    for c in &node.children {
        collect(c, runs, hlines);
    }
}

/// 제목 칸의 첫 줄은 표 상단에서 **23.6px**(한/글) 아래여야 한다 — `lead` 를 정렬
/// 공간에서 빼면 그 절반(3.3px)만큼 위로 쏠린다.
#[test]
fn centered_cell_does_not_subtract_lead_the_stack_already_holds() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let page = core.build_page_render_tree(0).expect("1쪽 render tree");

    let mut runs = Vec::new();
    let mut hlines = Vec::new();
    collect(&page.root, &mut runs, &mut hlines);

    // 제목 표는 1쪽 세 번째 표다 — 상단 괘선 227.8px(=170.85pt) 부근.
    hlines.sort_by(|a, b| a.partial_cmp(b).unwrap());
    hlines.dedup_by(|a, b| (*a - *b).abs() < 0.05);
    let table_top = hlines
        .iter()
        .copied()
        .find(|y| (*y - 227.8).abs() < 3.0)
        .unwrap_or_else(|| panic!("제목 표 상단 괘선을 못 찾았다 — 시험 설정 오류. {hlines:?}"));

    let title_top = runs
        .iter()
        .filter(|(_, t)| t.contains("사후소득"))
        .map(|(y, _)| *y)
        .fold(f64::INFINITY, f64::min);
    assert!(
        title_top.is_finite(),
        "제목 글줄을 못 찾았다 — 시험 설정 오류"
    );

    // 한/글 2024: 23.57px(글자 상단). 렌더 트리 TextRun 상단으로는 종전 19.77px.
    let gap = title_top - table_top;
    assert!(
        (gap - 23.57).abs() <= 1.2,
        "제목 칸 첫 줄이 표 상단에서 23.6px 아래여야 한다(한/글 2024) — #6569 회귀. \
         got {gap:.2}px (lead 를 정렬 공간에서 빼면 19.77)"
    );
}

/// `#6630` 의 반대 갈래는 그대로 유지된다 — 스택이 `lead` 를 안 품은 칸에서는 계속 뺀다.
/// `exam_eng` 2쪽 바탕쪽 머리 표의 제목 그림이 그 계약이다.
#[test]
fn issue_6630_contract_still_holds_when_stack_excludes_lead() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_eng.hwp");
    let Ok(bytes) = std::fs::read(&path) else {
        return;
    };
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_eng");
    let svg = doc.render_page_svg(1).expect("exam_eng 2쪽 SVG");

    // 148.7px 폭 제목 그림의 y — 한/글 149.2(셀 상단 132.3 + 16.9). 종전 결함은 145.5.
    let mut ys = Vec::new();
    for open in ["<image ", "<svg "] {
        for t in svg.split(open).skip(1) {
            let t = &t[..t.find('>').expect("태그 닫힘")];
            let num = |n: &str| -> Option<f64> {
                t.split(&format!("{n}=\""))
                    .nth(1)
                    .and_then(|r| r.split('"').next())
                    .and_then(|v| v.parse().ok())
            };
            if num("width").is_some_and(|w| (w - 148.7).abs() < 0.6) {
                if let Some(y) = num("y") {
                    ys.push(y);
                }
            }
        }
    }
    let Some(y) = ys.into_iter().reduce(f64::min) else {
        return;
    };
    assert!(
        (y - 149.2).abs() < 2.0,
        "#6630 계약(스택이 lead 를 안 품은 칸)이 깨졌다 — 제목 그림 y={y:.2} (한/글 149.2)"
    );
}
