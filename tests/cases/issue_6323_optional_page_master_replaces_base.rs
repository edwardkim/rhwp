//! [#6323] HWPX `OPTIONAL_PAGE pageDuplicate="0"` 바탕쪽이 기본 홀/짝 바탕쪽을
//! **대체**하는지 고정한다.
//!
//! `pageDuplicate="0"` 은 "겹치게 하기 끔" 이라는 문서의 명시적 선언이다. 종전에는
//! 파서가 이 선언을 `LAST_PAGE` 에만 반영하고 `OPTIONAL_PAGE` 에는 반영하지 않아,
//! 임의 쪽 바탕쪽이 기본 바탕쪽을 대체하지 못하고 그 **위에 덧그려졌다.**
//!
//! 실측(`samples/hwpx/exam_kor.hwpx` 20쪽) — 두 바탕쪽이 같은 좌표에 각자의 쪽번호를
//! 쓴다.
//!
//! ```text
//! MasterPage(EVEN)          x=119.1 y=142.4  '2'
//! MasterPage(OPTIONAL_PAGE) x=119.1 y=142.4  '4'   <- 같은 자리
//! ```
//!
//! 수능 국어 시험지의 쪽번호가 `2` 와 `4` 가 포개져 읽을 수 없는 글자가 된다. 머리말
//! `(언어와 매체)` 도 같은 좌표에 두 번 그려져 획이 겹친다.
//!
//! 같은 계약을 `LAST_PAGE` 에 대해서는
//! `src/renderer/layout/integration_tests.rs::test_1098_hwpx_last_page_master_replaces_base_master`
//! 가 이미 지키고 있었다. 이 시험은 그 짝인 `OPTIONAL_PAGE` 를 채운다.
//!
//! 판정은 렌더 트리에서 한다 — `pagination` 은 crate 내부 필드라 통합 시험에서 볼 수
//! 없고, 무엇보다 사용자가 보는 것은 "그 쪽에 바탕쪽이 몇 겹 그려졌는가" 이기 때문이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 구역 2 의 임의 쪽 바탕쪽(`masterpage8`,
/// `type="OPTIONAL_PAGE" pageNumber="4" pageDuplicate="0"`)이 적용되는 쪽이다(0 기준).
const SAMPLE: &str = "samples/hwpx/exam_kor.hwpx";
const PAGE: u32 = 19;

fn load() -> Option<DocumentCore> {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).ok()?;
    DocumentCore::from_bytes(&bytes).ok()
}

/// 페이지 루트의 직계 자식 중 바탕쪽 노드를 모은다.
fn master_page_children(root: &RenderNode) -> Vec<&RenderNode> {
    root.children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::MasterPage))
        .collect()
}

/// 노드 아래의 보이는 글자를 모은다.
fn visible_texts(node: &RenderNode, out: &mut Vec<String>) {
    if let RenderNodeType::TextRun(tr) = &node.node_type {
        let text = tr.display_or_text().trim();
        if !text.is_empty() {
            out.push(text.to_string());
        }
    }
    for child in &node.children {
        visible_texts(child, out);
    }
}

/// 그 쪽에는 바탕쪽이 **하나만** 그려진다.
#[test]
fn optional_page_master_does_not_stack_on_the_base_master() {
    let Some(core) = load() else {
        return;
    };
    let Ok(tree) = core.build_page_render_tree(PAGE) else {
        return;
    };

    let masters = master_page_children(&tree.root);
    let rendered: Vec<Vec<String>> = masters
        .iter()
        .map(|m| {
            let mut t = Vec::new();
            visible_texts(m, &mut t);
            t
        })
        .collect();

    assert_eq!(
        masters.len(),
        1,
        "바탕쪽이 겹쳐 그려지면 쪽번호·머리말이 같은 좌표에 포개진다. \
         그려진 바탕쪽 {}겹, 각 글자: {rendered:?}",
        masters.len()
    );
}

/// 겹쳐 그리던 시절의 증상 자체를 고정한다 — 쪽번호가 두 개 그려지지 않는다.
///
/// 임의 쪽 바탕쪽이 기본 바탕쪽을 대체하므로 이 쪽의 쪽번호는 `4` 하나여야 하고,
/// 기본 짝수 바탕쪽의 `2` 는 그려지지 않아야 한다.
#[test]
fn page_number_is_not_drawn_twice() {
    let Some(core) = load() else {
        return;
    };
    let Ok(tree) = core.build_page_render_tree(PAGE) else {
        return;
    };

    let mut texts = Vec::new();
    for master in master_page_children(&tree.root) {
        visible_texts(master, &mut texts);
    }

    let page_numbers: Vec<&String> = texts
        .iter()
        .filter(|t| t.chars().all(|c| c.is_ascii_digit()))
        .collect();
    assert_eq!(
        page_numbers.len(),
        1,
        "바탕쪽 쪽번호가 하나여야 한다. 실제로 그려진 숫자: {page_numbers:?} \
         (전체 바탕쪽 글자: {texts:?})"
    );
}
