//! 관계형·고정형 — 생략 속성 수식의 셀 포함과 후행 표·각주 쪽 소속을 검사한다.
//! 주석 갱신: 2026-11-07T06:50:29+09:00; 새 파일, 직전 파일 커밋 없음.
//! 2026-11-07 측정: 96 DPI, 색상 경계 보정 전 2px 내용 실루엣, Print·compat 2022.
//! report-628 전195쪽 최저 Native92.99% / fresh WASM96.06%, 독립 Print 전195쪽과 일치.
//! 18쪽99.72/99.73%,122쪽99.58/99.59%,123쪽99.77/99.77%,124쪽99.92/99.91%,125쪽99.81/99.79%.
//! 위 다섯 쪽의 양쪽 review·overlay에서 수식·표 경계·뒤 본문·각주 소속을 직접 대조했다.
//! 근거: mydocs/pr/assets/planet6897_20261106/pr8642/r25/의 whole-comparison.json,
//! report-628-{native,wasm}.tsv·PNG·producer.json 및 ../../equation-implicit-fields-plan.md.
//! 원본과 독립 PDF의 SHA·명령·producer는 같은 폴더의 producer/whole-comparison 기록을 따른다.
//! 실물 절대 px 좌표를 고정하지 않는다. 시스템 OTF 설치를 전제하거나 skip하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "mydocs/pr/assets/planet6897_20261106/pr8642/report-628-original.hwp";

fn append_text(node: &RenderNode, text: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        text.push_str(&run.text);
    }
    for child in &node.children {
        append_text(child, text);
    }
}

fn check_equation_cells(node: &RenderNode, cell: Option<&BoundingBox>, count: &mut usize) {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(&node.bbox)
    } else {
        cell
    };
    if matches!(node.node_type, RenderNodeType::Equation(_)) {
        let owner = cell.expect("수식은 원래 셀 안에 속한다");
        let equation = &node.bbox;
        assert!(
            equation.y >= owner.y - 1e-6
                && equation.y + equation.height <= owner.y + owner.height + 1e-6,
            "수식의 현재 줄 높이를 포함한 셀 프레임: 수식={equation:?}, 셀={owner:?}"
        );
        *count += 1;
    }
    for child in &node.children {
        check_equation_cells(child, cell, count);
    }
}

#[test]
fn implicit_equation_ink_stays_in_its_recomposed_cell_frame() {
    let source =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let core = DocumentCore::from_bytes(&source).expect("원본 보고서");
    for page in [17, 121] {
        let tree = core
            .build_page_render_tree(page)
            .expect("같은 의미의 수식이 있는 두 쪽");
        let mut count = 0;
        check_equation_cells(&tree.root, None, &mut count);
        assert!(count > 0, "수식을 통째로 누락한 출력은 통과할 수 없다");
    }
}

#[test]
fn recomposed_equation_tail_preserves_following_tables_and_notes_on_their_pages() {
    let source =
        std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).unwrap();
    let core = DocumentCore::from_bytes(&source).expect("원본 보고서");
    assert_eq!(
        core.page_count(),
        195,
        "독립 Print 전체 쪽수와 대응 내용을 함께 검사한다"
    );
    for (page, caption, body, note) in [
        (122, "<표4-12>", "총보험금", "1,006,268원"),
        (123, "<표4-13>", "재정방식별필요보험료", "부분적립방식"),
        (124, "<표4-14>", "재정전망", "대만의경우"),
    ] {
        let tree = core
            .build_page_render_tree(page)
            .expect("표와 각주 소유 쪽");
        let mut text = String::new();
        append_text(&tree.root, &mut text);
        let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
        for expected in [caption, body, note] {
            assert!(
                compact.contains(expected),
                "Print {}쪽의 대응 내용 {expected} 누락/다른 쪽 이월",
                page + 1
            );
        }
        assert_eq!(
            compact.matches(caption).count(),
            1,
            "같은 표 캡션을 중복 배치하지 않는다"
        );
    }
}
