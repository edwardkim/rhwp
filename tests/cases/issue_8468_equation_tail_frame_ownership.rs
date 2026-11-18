//! 관계형·고정형 — 생략 속성 수식의 셀 포함과 후행 표·각주 쪽 소속을 검사한다.
//! 주석 갱신: 2026-11-08T16:02:19+09:00.
//! 추가 전 파일 변경: 59d83dc9ccdf6ae52d7ec77e00f2e0e22dea2a10 (2026-11-07T07:46:00+09:00).
//! 추가 합성 계약은 선택 문자열 생략의 연속 배율·정체/기울임 전진 관계이며 설치 글꼴·실물 px를 사용하지 않는다.
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

fn implicit_formula(script: &str, font_size: u32) -> rhwp::renderer::equation::layout::LayoutBox {
    use rhwp::model::control::Control;
    fn find(node: &RenderNode) -> Option<rhwp::renderer::equation::layout::LayoutBox> {
        if let RenderNodeType::Equation(value) = &node.node_type {
            return Some(value.layout_box.clone());
        }
        node.children.iter().find_map(find)
    }
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_equation_native(0, 0, 0, script, font_size, 0)
        .unwrap();
    let mut doc = core.document().clone();
    let equation = doc.sections[0].paragraphs[0]
        .controls
        .iter_mut()
        .find_map(|control| {
            if let Control::Equation(value) = control {
                Some(value)
            } else {
                None
            }
        })
        .expect("실제 삽입 경로가 만든 수식");
    equation.implicit_legacy_font = true;
    equation.font_name.clear();
    equation.version_info.clear();
    equation.common.size_protect = false;
    core.set_document(doc);
    find(&core.build_page_render_tree(0).unwrap().root).expect("생략 출처를 가진 최종 수식 상자")
}

#[test]
fn implicit_font_keeps_continuous_advances_and_separate_italic_shear() {
    // 독립 10/20pt 생략 문자열 Print의 전진 소유다. 원 CHAR의 정수 장치 격자를
    // 이 호환 계약에 이식하거나 정체 전진을 별도 이탤릭 표로 바꾸지 않는다.
    for script in ["a", "x+y", "rm{123}"] {
        let small = implicit_formula(script, 1000);
        let large = implicit_formula(script, 2000);
        assert!(small.width > 0.0);
        assert!(
            (large.width - small.width * 2.0).abs() < 1e-8,
            "생략 표시의 두 배 크기는 같은 전진의 두 배다: {script}: {} -> {}",
            small.width,
            large.width
        );
    }
    let italic = implicit_formula("a", 1000);
    let roman = implicit_formula("rm{a}", 1000);
    assert!(
        (italic.width - roman.width).abs() < 1e-8,
        "생략 표시의 기울임은 같은 정체 전진 위의 shear다: {} / {}",
        italic.width,
        roman.width
    );
}
