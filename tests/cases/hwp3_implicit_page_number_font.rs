//! 검사 유형: 고정형(기본 글꼴·독립 Print의 시작점/폭/기준선 ±2 CSSpx), 관계형(명시 스타일 우선·번호 유일성).
//! 주석 갱신: 2026-11-09T05:01:27+09:00; 직전 파일 변경: 2db97cee036c398232b377c293cf2521fed8ae92 (2026-11-09T04:38:00+09:00).
//! 2026-11-09 canonical raw 2px, 96 DPI, compat2022, 독립 한컴2020 Print의 Linux PDFium PNG를 SHA 검증 후 재사용.
//! hwp3-sample10.hwp 전 763쪽 Native 최저 99.41%, fresh WASM 최저 99.38%.
//! 고정하는 629쪽은 Native/fresh WASM 모두 100.00%; 같은 쪽 review·overlay와 번호 원점을 직접 판독했다.
//! 명시 스타일 HWPX 대조군 전 2쪽은 두 경로 모두 100.00%; 기존 Native PNG와도 동일하다.
//! 보호 TSV의 Native 하락은 0쪽이며 WASM의 미복원 8쪽은 별도 추적한다. 전체 통합 검증 완료 증거가 아니다.
//! 검사 실행은 설치 글꼴에 의존하지 않으며 개인 글꼴·CoreText 서비스 접근을 차단한 Mac에서도 3 PASS.
//! 근거: mydocs/pr/assets/planet6897_20261106/hwp3-default-page-number/validation.json.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
fn numbers(node: &RenderNode, out: &mut Vec<String>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.char_shape_id.is_none()
            && run.para_index.is_none()
            && run.text.chars().any(|c| c.is_ascii_digit())
            && run
                .text
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_whitespace() || c == '-')
        {
            out.push(run.style.font_family.clone());
        }
    }
    for c in &node.children {
        numbers(c, out)
    }
}
fn number_fonts(core: &DocumentCore, page: u32) -> Vec<String> {
    let page = core.build_page_render_tree(page).unwrap();
    let mut out = Vec::new();
    numbers(&page.root, &mut out);
    out
}
#[test]
fn hwp3_implicit_numbers_keep_the_independent_print_default_program_owner() {
    let mut core =
        DocumentCore::from_bytes(&std::fs::read("samples/hwp3-sample10.hwp").unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(
        core.document().provenance.format,
        rhwp::model::provenance::SourceFormat::Hwp3
    );
    assert!(core
        .document()
        .doc_info
        .styles
        .iter()
        .any(|s| s.local_name == "쪽번호"));
    assert!(core
        .document()
        .doc_info
        .styles
        .iter()
        .all(|s| s.local_name != "쪽 번호" && !s.english_name.eq_ignore_ascii_case("Page Number")));
    assert_eq!(
        number_fonts(&core, 628),
        ["Haansoft Batang"],
        "독립 한컴2020 Print 629쪽의 글꼴 없는 HWP3 자동 번호"
    );
}
#[test]
fn explicit_page_number_style_keeps_its_declared_program_owner() {
    let mut core = DocumentCore::from_bytes(
        &std::fs::read("tests/fixtures/page_number_position/hancom-page-number-top.hwpx").unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(false);

    for page in [0, 1] {
        let names = number_fonts(&core, page);
        assert_eq!(names.len(), 1);
        assert_eq!(
            names[0], "함초롬돋움",
            "명시한 쪽번호 글꼴을 기본값으로 덮지 않는다"
        );
    }
}

#[test]
fn default_number_keeps_independent_print_origin_and_extent() {
    fn collect(n: &RenderNode, out: &mut Vec<(rhwp::renderer::render_tree::BoundingBox, f64)>) {
        if let RenderNodeType::TextRun(run) = &n.node_type {
            if run.char_shape_id.is_none() && run.para_index.is_none() && run.text == "- 623 -" {
                out.push((n.bbox, n.bbox.y + run.baseline));
            }
        }
        for c in &n.children {
            collect(c, out)
        }
    }
    let mut core =
        DocumentCore::from_bytes(&std::fs::read("samples/hwp3-sample10.hwp").unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    let tree = core.build_page_render_tree(628).unwrap();
    let mut numbers = Vec::new();
    collect(&tree.root, &mut numbers);
    assert_eq!(numbers.len(), 1, "꼬리말 숫자의 소유·누락·중복");
    let (bbox, baseline) = numbers[0];
    assert!(
        (bbox.x - 367.3209228515625).abs() <= 2.0,
        "독립 한컴 Print의 숫자 시작점: {}",
        bbox.x
    );
    assert!(
        (bbox.width - 58.71040852864583).abs() <= 2.0,
        "독립 한컴 Print의 숫자 폭: {}",
        bbox.width
    );
    assert!(
        (baseline - 1024.9598795572917).abs() <= 2.0,
        "독립 한컴 Print의 숫자 기준선: {baseline}"
    );
}
