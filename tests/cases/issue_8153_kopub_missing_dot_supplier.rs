//! 관계형·고정형: 결측 U+2024의 실제 공급자 메트릭과 원문·공백·기존 glyph hit를 보존한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 368쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 368쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 Print 368쪽의 점 2.07935px·공백 6.56px와 HBATANG 178/1024em이 근거다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};

fn find_cell<'a>(
    node: &'a RenderNode,
    owner: Option<&'a RenderNode>,
    out: &mut Vec<&'a RenderNode>,
) {
    let owner = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        owner
    };
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.display_or_text() == "․" {
            out.push(owner.expect("점의 실제 셀 소유"));
        }
    }
    for child in &node.children {
        find_cell(child, owner, out);
    }
}
fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a TextRunNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push(run);
    }
    for child in &node.children {
        collect(child, out);
    }
}
fn positions(run: &TextRunNode) -> Vec<f64> {
    run.layout_positions.clone().unwrap_or_else(|| {
        EmbeddedTextMeasurer.compute_char_positions(run.display_or_text(), &run.style)
    })
}
fn near_print(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print의 {label}: 기대 {expected}, 실제 {actual}"
    );
}
fn check(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(367).unwrap();
    let mut owners = Vec::new();
    find_cell(&tree.root, None, &mut owners);
    assert_eq!(owners.len(), 3, "원래 세 점의 수량");
    assert!(
        owners.iter().all(|n| std::ptr::eq(*n, owners[0])),
        "같은 원문 셀 소유"
    );
    let mut runs = Vec::new();
    collect(owners[0], &mut runs);
    let text: String = runs.iter().map(|run| run.display_or_text()).collect();
    assert_eq!(
        text, "(     ․     ․      ․      ：   )",
        "원문 순서·내용 보존"
    );
    let mut spaces = 0;
    for run in &runs {
        let text = run.display_or_text();
        let pos = positions(run);
        if text == "․" {
            near_print(pos[1] - pos[0], 2.07935, "실제 공급자의 점 전진");
        }
        for (index, ch) in text.chars().enumerate() {
            if ch == ' ' {
                near_print(pos[index + 1] - pos[index], 6.56, "같은 줄의 공백 전진");
                spaces += 1;
            }
        }
    }
    assert_eq!(spaces, 25, "원문 공백 수량");
    let dot = runs
        .iter()
        .find(|run| run.display_or_text() == "․")
        .unwrap();
    // U+2027은 원 KoPub의 cmap/hmtx 900/1000em이다. 결측 점과 혼동하지 않는다.
    let primary = EmbeddedTextMeasurer.compute_char_positions("‧", &dot.style);
    assert!(
        (primary[1] - primary[0] - 11.04).abs() <= 96.0 / 7200.0,
        "KoPub 자체의 유효한 기호 전진"
    );
}
#[test]
fn hwp_missing_kopub_dot_uses_declared_supplier_and_keeps_spaces() {
    check("hwp");
}
#[test]
fn hwpx_missing_kopub_dot_uses_declared_supplier_and_keeps_spaces() {
    check("hwpx");
}
