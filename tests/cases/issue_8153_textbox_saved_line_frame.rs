//! 관계형·고정형: 그림 글상자의 저장 줄 소유와 독립 Print 원점을 검사한다.
//! 주석 갱신: 2026-10-30T07:25:07+09:00.
//! 2026-10-30 측정 raw(96dpi, PDFium, 2px): HWP·HWPX 205쪽 Native·fresh WASM 각 100.00%.
//! 두 원본 전 383쪽 최저 Native·fresh WASM 각각 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/rebase-ad4-validation-20261030/README.md.
//! 독립 한컴 Print의 문자 원점은 PDF pt를 96dpi CSSpx로 환산해 ±1px로 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::path::Path;

fn labels<'a>(node: &'a RenderNode, in_textbox: bool, out: &mut Vec<&'a RenderNode>) {
    let in_textbox = in_textbox || matches!(node.node_type, RenderNodeType::TextBox);
    if in_textbox
        && matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == "Y" || run.text == "N")
    {
        out.push(node);
    }
    for child in &node.children {
        labels(child, in_textbox, out);
    }
}

fn verify(extension: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 편람")).expect("원본 문서 해석");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print의 전체 쪽수");
    let tree = core.build_page_render_tree(204).expect("205쪽");
    let mut found = Vec::new();
    labels(&tree.root, false, &mut found);
    let expected = [
        ("Y", 212.87999, 403.31998),
        ("N", 208.2, 425.16),
        ("N", 210.72, 468.72),
        ("Y", 203.39999, 496.2),
        ("Y", 420.47998, 341.88),
        ("N", 411.35999, 370.44),
    ];
    assert_eq!(
        found.len(),
        expected.len(),
        "글상자 안 글자의 누락·중복 없음"
    );
    for (node, (text, x_pt, baseline_pt)) in found.into_iter().zip(expected) {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        assert_eq!(run.text, text, "원본 글자의 순서와 글상자 소유");
        let x = x_pt * 96.0 / 72.0;
        let baseline = baseline_pt * 96.0 / 72.0;
        assert!(
            (node.bbox.x - x).abs() <= 1.0,
            "독립 Print 원점: {} / {x}",
            node.bbox.x
        );
        assert!(
            (node.bbox.y + run.baseline - baseline).abs() <= 1.0,
            "독립 Print 기준선: {} / {baseline}",
            node.bbox.y + run.baseline
        );
    }
}

#[test]
fn hwp_stored_textbox_uses_its_original_line_frame() {
    verify("hwp");
}

#[test]
fn hwpx_stored_textbox_uses_its_original_line_frame() {
    verify("hwpx");
}
