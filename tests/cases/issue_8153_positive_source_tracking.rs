//! 관계형·고정형: 원본 자간은 숫자 윤곽을 늘리지 않고 문단 끝 밑줄에 남지 않는다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 111쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 111쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 Print 111쪽의 숫자 메트릭과 밑줄을 대조한다. 전체 쪽수·소유는 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::{layer_renderer::LayerRenderer, svg_layer::SvgLayerRenderer};
use rhwp::DocumentCore;

fn selected_number(text: &str) -> bool {
    matches!(text, "1311334" | "135")
}

fn select_numbers(node: &mut LayerNode, found: &mut Vec<(String, bool)>) {
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                select_numbers(child, found);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            select_numbers(child, found);
        }
        LayerNodeKind::Leaf { ops } => {
            ops.retain(|op| match op {
                PaintOp::TextRun { run, source, .. } if selected_number(&run.text) => {
                    assert!(source.is_some(), "숫자의 원본 소유를 유지한다");
                    assert!((run.style.letter_spacing / run.style.font_size - 0.5).abs() < 1e-6);
                    assert!((run.style.ratio - 0.98).abs() < 1e-6);
                    found.push((run.text.clone(), run.is_para_end));
                    true
                }
                PaintOp::TextDecoration { run, .. } => selected_number(&run.text),
                _ => false,
            });
        }
    }
}

fn numbers(extension: &str) -> PageLayerTree {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383);
    let mut tree = core
        .build_page_layer_tree_with_profile(110, RenderProfile::Print)
        .unwrap();
    let mut found = Vec::new();
    select_numbers(&mut tree.root, &mut found);
    tree.text_sources
        .entries
        .retain(|entry| selected_number(&entry.text));
    assert_eq!(
        found,
        [
            ("1311334".into(), false),
            ("135".into(), true),
            ("135".into(), true)
        ],
        "문단 중간·끝의 숫자 내용·순서·누락·중복을 확인한다"
    );
    tree
}

fn verify_glyphs(extension: &str) {
    let tree = numbers(extension);
    let mut svg = SvgLayerRenderer::new();
    svg.render_page(&tree).unwrap();
    let xml = roxmltree::Document::parse(svg.output()).unwrap();
    let glyphs: Vec<_> = xml
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter(|node| {
            node.text()
                .is_some_and(|s| s.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();
    assert_eq!(glyphs.len(), 13, "세 숫자 런의 13개 숫자 윤곽을 보존한다");
    for glyph in glyphs {
        let width: f64 = glyph.attribute("textLength").unwrap().parse().unwrap();
        // 독립 Print의 숫자와 동일한 원본 hmtx: 573/1000 em.
        // SVG local textLength에는 바깥 장평 .98을 중복 적용하지 않는다.
        assert!(
            (width - 573.0 / 1000.0 * (10.0 * 96.0 / 72.0)).abs() < 0.1,
            "원본 자간은 숫자 glyph 폭에 포함하지 않는다: {width}"
        );
    }
}

fn decorations(value: &serde_json::Value, found: &mut Vec<(u64, f64)>) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some("textDecoration") {
                let positions = value["decoration"]["positions"].as_array().unwrap();
                found.push((
                    value["source"]["id"].as_u64().unwrap(),
                    positions.last().unwrap().as_f64().unwrap(),
                ));
            }
            for child in map.values() {
                decorations(child, found);
            }
        }
        serde_json::Value::Array(array) => {
            for child in array {
                decorations(child, found);
            }
        }
        _ => {}
    }
}

fn verify_underline(extension: &str) {
    let tree = numbers(extension);
    let json: serde_json::Value =
        serde_json::from_str(&tree.to_json_with_options(rhwp::paint::LayerJsonOptions {
            omit_image_bytes: true,
            omit_font_bytes: true,
        }))
        .unwrap();
    let mut found = Vec::new();
    decorations(&json, &mut found);
    assert_eq!(found.len(), 3);
    // 독립 Print의 세 밑줄 길이(pt)다. 문단 중간의 간격은 유지하고,
    // 두 문단 끝은 마지막 가시 glyph 뒤의 자간을 제외한다.
    for ((_, width), points) in found.iter().zip([58.856, 22.536, 22.415]) {
        assert!(
            (width - points * 96.0 / 72.0).abs() <= 2.0,
            "원본 소유를 유지한 밑줄의 독립 길이: 실제 {width}, Print {points}pt"
        );
    }
}

#[test]
fn hwp_positive_source_tracking_preserves_intrinsic_digit_widths() {
    verify_glyphs("hwp");
}

#[test]
fn hwpx_positive_source_tracking_preserves_intrinsic_digit_widths() {
    verify_glyphs("hwpx");
}

#[test]
fn hwp_terminal_underline_ends_after_the_last_digit() {
    verify_underline("hwp");
}

#[test]
fn hwpx_terminal_underline_ends_after_the_last_digit() {
    verify_underline("hwpx");
}
