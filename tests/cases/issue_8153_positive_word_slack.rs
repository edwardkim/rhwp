//! 관계형·고정형: 낱말을 넓히는 줄과 독립 Print의 단어 원점을 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 136쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 136쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 2024 Print 136쪽, 96dpi의 단어 원점과 저장 줄·셀의 내용 순서를 보호한다.
//! 양쪽 원본 Native/fresh DEV WASM raw 100%·미대응 0픽셀과 수정 전 FAIL/후 PASS는
//! `mydocs/plans/issue_8153_justify_float_residual_20261027.md`에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{BoundingBox, TextRunNode},
    },
    DocumentCore,
};

fn collect<'a>(node: &'a LayerNode, runs: &mut Vec<(&'a BoundingBox, &'a TextRunNode)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect(child, runs);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect(child, runs);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { bbox, run, .. } = op {
                    runs.push((bbox, run));
                }
            }
        }
    }
}

fn verify(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print의 전체 쪽수");
    let source = core.export_hwp_native().unwrap();
    let tree = core
        .build_page_layer_tree_with_profile(135, RenderProfile::Print)
        .unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    let matched: Vec<_> = runs
        .iter()
        .copied()
        .filter(|(_, run)| run.display_or_text().contains("카카오톡"))
        .collect();
    assert_eq!(matched.len(), 1, "원래 단어의 누락·중복 금지");
    let (bbox, run) = matched[0];
    assert!(run.cell_context.is_some(), "원본 표 셀 소유");
    assert!(run.style.extra_word_spacing > 0.0, "낱말을 넓히는 줄");
    assert_eq!(
        run.style.extra_char_spacing, 0.0,
        "양수 낱말 여백의 나눗셈 잔여는 글자 압축이 아니다"
    );
    let visible = run.display_or_text();
    let byte = visible.find("카카오톡").unwrap();
    let character = visible[..byte].chars().count();
    let positions = run
        .layout_positions
        .clone()
        .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(visible, &run.style));
    let x = bbox.x + positions[character];
    let baseline = bbox.y + run.baseline;
    // 독립 Print PDF의 물리 136쪽: 원점 585.59998px, 기준선 856.47998px.
    assert!((x - 585.59998).abs() <= 2.0, "독립 단어 원점: {x}");
    assert!(
        (baseline - 856.47998).abs() <= 2.0,
        "독립 기준선: {baseline}"
    );

    let mut owned: Vec<_> = runs
        .into_iter()
        .filter(|(_, other)| other.cell_context == run.cell_context)
        .collect();
    owned.sort_by(|a, b| a.0.y.total_cmp(&b.0.y).then(a.0.x.total_cmp(&b.0.x)));
    let text = owned
        .iter()
        .map(|(_, other)| other.display_or_text())
        .collect::<String>();
    assert!(
        text.contains("확인할 수 있도록 기존 카카오톡 알림"),
        "저장 줄 경계의 원문·구분 공백·다음 줄 소유 보존"
    );
    assert_eq!(
        source,
        core.export_hwp_native().unwrap(),
        "조판이 원본 저장 내용을 바꾸지 않는다"
    );
}

#[test]
fn hwpx_positive_word_slack_preserves_glyphs_and_next_line_content() {
    verify("hwpx");
}

#[test]
fn hwp_positive_word_slack_preserves_glyphs_and_next_line_content() {
    verify("hwp");
}
