//! #8979 문서 수식에서 바깥 괄호의 짝 `)` 가 중괄호 안에 놓이면
//! (`(x - bar{x) ^{2}}`) 그룹 루프가 괄호 경계(term_limit)에서 토큰을 소비하지
//! 못한 채 Empty 를 무한히 쌓아 수십 GB 를 할당하고 프로세스가 죽었다.
//!
//! 계약: 문서 항은 괄호 경계를 입력 끝으로 보고 열린 중괄호 그룹을 그 자리에서
//! 닫는다. 그래서 짝이 어긋난 식은 짝을 맞춘 식과 같은 조판 트리·상자로 그려진다.
//! 수식은 `rhwp edit insert-equation` 으로 저장소의 `samples/field-01.hwp` 에 넣어
//! 실제 문서 수식 경로(`document_script::parse`)를 지난다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn with_equation(tag: &str, script: &str) -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/field-01.hwp");
    let out = std::env::temp_dir().join(format!(
        "rhwp-8979-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let output = Command::new(rhwp_bin())
        .args(["edit", "insert-equation"])
        .arg(&src)
        .args(["--script", script, "--offset", "0", "-o"])
        .arg(&out)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    out
}

fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        descendants(child, out);
    }
}

/// 첫 쪽에 그린 그 수식의 (상자, 조판 트리) 를 돌려준다.
fn drawn_equation(tag: &str, script: &str) -> (String, String) {
    let path = with_equation(tag, script);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let _ = std::fs::remove_file(&path);
    let page = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    descendants(&page.root, &mut nodes);
    let found: Vec<_> = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Equation(eq) if eq.script == script => {
                Some((format!("{:?}", n.bbox), format!("{:?}", eq.layout_box)))
            }
            _ => None,
        })
        .collect();
    assert_eq!(found.len(), 1, "{tag}: 넣은 수식 하나가 그려져야 한다");
    found.into_iter().next().unwrap()
}

fn assert_same_as_balanced(tag: &str, unbalanced: &str, balanced: &str) {
    let (bad_box, bad_layout) = drawn_equation(&format!("{tag}-bad"), unbalanced);
    let (good_box, good_layout) = drawn_equation(&format!("{tag}-good"), balanced);
    assert_eq!(bad_layout, good_layout, "{tag}: 조판 트리");
    assert_eq!(bad_box, good_box, "{tag}: 수식 상자");
}

#[test]
fn bar_group_closes_at_outer_paren_boundary() {
    assert_same_as_balanced("min", "(x - bar{x) ^{2}}", "(x - bar{x}) ^{2}");
}

#[test]
fn variance_formula_from_report_renders_like_balanced_form() {
    // 원 문서(정부 연구보고서 40쪽 표본분산) 의 저장 스크립트 그대로.
    assert_same_as_balanced(
        "variance",
        "s  ^{2} = {편차의`제곱합} over {n-1} = {1} over {n-1} = sum _{} ^{} (x  _{i} - bar{x)  ^{2}}",
        "s  ^{2} = {편차의`제곱합} over {n-1} = {1} over {n-1} = sum _{} ^{} (x  _{i} - bar{x})  ^{2}",
    );
}
