//! [#8501] 관계형 회귀 — 셀 단위로 나누는 표의 쪽 끝 행은, 셀 저장 줄이 새 원점(vpos 0)에서
//! 다시 시작하는 자리(한/글이 그 행을 나눈 기록)에서 나눈다. 셀 내용이 남은 예산에 겨우
//! 들어간다는 이유로 행 전체를 앞 쪽에 두지 않는다.
//!
//! - 입력: `samples/issue8501/child_product_safety_rule.hwp` — 산업통상자원부 «(법령안) 어린이제품
//!   안전 특별법 시행규칙 일부개정령(안)»(한/글 2020 저장). 1구역 20×5 표의 행 16 2열 셀
//!   「안전확인신고표시 | 사용금지 | 2개월」 은 셋째 문단 첫 줄이 vpos 0 으로 저장돼 있다.
//! - 독립 기준: 같은 HWP 의 한/글 2022 Print `pdf/issue8501_child_product_safety_rule-2020.pdf`
//!   — 19쪽. 6쪽은 행 16 을 3줄에서 끝내고, 7쪽 첫머리가 그 셀의 「2개월」 이다.
//! - 수정 전 rhwp 는 행 16 의 4줄을 모두 6쪽에 두어 7쪽이 행 17(「3) 국가기술표준원장이…」)로
//!   시작했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn collect_runs(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 쪽마다 그려진 글자(공백 제거)를 이어 붙인 문자열.
fn page_texts(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("렌더 트리");
            let mut text = String::new();
            collect_runs(&tree.root, &mut text);
            text.replace(' ', "")
        })
        .collect()
}

#[test]
fn page_end_row_splits_at_stored_paragraph_restart() {
    let core = core("samples/issue8501/child_product_safety_rule.hwp");
    let pages = page_texts(&core);
    assert_eq!(pages.len(), 19, "한/글 Print 쪽수");

    let row16 = "2)국가기술표준원장이고시";
    let row17 = "3)국가기술표준원장이고시";
    // 행 16 은 6쪽(0-based 5)에서 시작하고, 행 17 은 7쪽에 있다.
    assert!(pages[5].contains(row16), "행 16 은 6쪽에서 시작한다");
    assert!(!pages[5].contains(row17), "행 17 은 6쪽에 없다");
    assert!(pages[6].contains(row17), "행 17 은 7쪽에 있다");
    assert!(
        !pages[6].contains(row16),
        "행 16 의 첫 줄들은 7쪽에 다시 그리지 않는다"
    );

    // 행 16 의 저장 재시작 뒤 줄(「2개월」)은 7쪽 첫머리, 행 17 앞에 온다.
    let tail = pages[6].find("2개월").expect("7쪽에 행 16 꼬리 「2개월」");
    let next = pages[6].find(row17).unwrap();
    assert!(tail < next, "7쪽은 행 16 의 꼬리 줄로 시작한다");
    // 6쪽은 행 16 의 셋째 줄(「사용금지」)까지 담고, 그 뒤 꼬리 줄은 담지 않는다.
    let head = &pages[5][pages[5].find(row16).unwrap()..];
    assert!(
        head.contains("사용금지"),
        "6쪽에 행 16 의 저장 재시작 앞 줄"
    );
    assert!(
        !head.contains("2개월"),
        "저장 재시작 뒤 줄은 6쪽에 두지 않는다"
    );
}
