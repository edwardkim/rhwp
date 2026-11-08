//! #2006: PrEP의 독립 한컴 2024 Print 140쪽과 내용 소속을 함께 검증한다.
//!
//! 정본은 같은 원본의 `pdf/issue2006/1790387_prep_final_report-2024.pdf`다.
//! 원본/PDF 출처는 `mydocs/pr/archives/pr_7406_review.md`, 기존 96dpi 전쪽
//! Native/fresh WASM 최저 91.93033% 증거는 완료 후보의 stored-anchor-repair
//! 요약에 연결한다. 쪽수만 맞는 잘못된 분할을 통과시키지 않는다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text(node: &RenderNode) -> String {
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    own + &node.children.iter().map(text).collect::<String>()
}

fn has_budget_table(node: &RenderNode) -> bool {
    matches!(&node.node_type, RenderNodeType::Table(data)
        if data.para_index == Some(336) && data.row_count == 20 && data.col_count == 6)
        || node.children.iter().any(has_budget_table)
}

#[test]
fn prep_1790387_page_count_and_boundary_content() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue2006/1790387_prep_final_report.hwpx");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("PrEP 정식 원본")).expect("문서 로드");
    assert_eq!(core.page_count(), 140, "독립 한컴 2024 Print의 전체 쪽수");
    let first = core.build_page_render_tree(0).expect("표지");
    let cover: String = text(&first.root)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert!(cover.contains("HIV노출전예방요법(PrEP)수요자타당도검증및질병부담연구"));
    let last = core.build_page_render_tree(139).expect("마지막 주의 사항");
    let final_text: String = text(&last.root)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    for needle in [
        "정책연구용역사업의최종결과보고서",
        "연구결과임을밝혀야합니다.",
        "발표또는공개하여서는아니됩니다.",
    ] {
        assert!(
            final_text.contains(needle),
            "마지막 주의 사항 보존: {needle}"
        );
    }
    let owners: Vec<_> = (0..core.page_count())
        .filter(|page| {
            has_budget_table(
                &core
                    .build_page_render_tree(*page)
                    .expect("전쪽 내용 소속")
                    .root,
            )
        })
        .collect();
    assert_eq!(
        owners,
        [104],
        "완결 연구비 표는 Print 105쪽에만 있다. 뒤쪽 빈 조각·중복으로 쪽수를 맞추지 않는다"
    );
}
