//! 관계형·고정형: 독립 Print 날짜 원점과 셀 소유를 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 80쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 85쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 86쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 87쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 80쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 85쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 86쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 87쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 두 원본의 80·85~87쪽 Native/fresh WASM raw98% 이상, 96dpi, ±2px.
//! 독립 근거와 원본 SHA: issue_8153_stored_cell_width_20261024.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn find<'a>(
    node: &'a RenderNode,
    owner: Option<&'a RenderNode>,
    text: &str,
    found: &mut Vec<(&'a RenderNode, &'a RenderNode)>,
) {
    let owner = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        owner
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.display_or_text() == text {
            found.push((node, owner.expect("날짜의 소유 셀")));
        }
    }
    for child in &node.children {
        find(child, owner, text, found);
    }
}

fn check(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 쪽수");
    for (page, text, first_digit) in [
        (80, "2025. 9. 30. ", 482.08),
        (85, "2025. 9. 20. ", 465.44),
        (86, "2025. 9. 20. ", 480.96),
        (87, "2025. 9. 20. ", 464.32),
    ] {
        let tree = core.build_page_render_tree(page - 1).unwrap();
        let mut found = Vec::new();
        find(&tree.root, None, text, &mut found);
        assert_eq!(found.len(), 1, "{extension} {page}쪽 날짜 내용·수량 보존");
        let (node, cell) = found[0];
        assert!(
            (node.bbox.x - first_digit).abs() <= 2.0,
            "{extension} {page}쪽 독립 Print 날짜 원점 {first_digit}, 실제 {}",
            node.bbox.x
        );
        assert!(node.bbox.x >= cell.bbox.x);
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        let positions = run
            .layout_positions
            .clone()
            .unwrap_or_else(|| EmbeddedTextMeasurer.compute_char_positions(text, &run.style));
        let last_visible = text
            .chars()
            .enumerate()
            .filter(|(_, ch)| *ch != ' ')
            .map(|(index, _)| index)
            .last()
            .unwrap();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(
            node.bbox.x + positions[last_visible] < cell.bbox.x + cell.bbox.width,
            "말미 구분 공백과 실제 마지막 글자의 소속을 구별한다"
        );
        assert!(node.bbox.y >= cell.bbox.y);
        assert!(node.bbox.y + node.bbox.height <= cell.bbox.y + cell.bbox.height);
    }
}

#[test]
fn hwp_signature_dates_use_their_source_cell_frame() {
    check("hwp");
}

#[test]
fn hwpx_signature_dates_use_their_source_cell_frame() {
    check("hwpx");
}
