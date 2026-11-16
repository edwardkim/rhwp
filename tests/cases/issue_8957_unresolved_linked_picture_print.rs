//! 관계형·고정형: 미해결 연결 그림의 화면/인쇄 분리, 내용·배치 보존과 독립 Print의 정상 그림 수.
//! 주석 갱신: 2026-11-12T17:04:43+09:00. 신규 검사.
//! 측정: 2026-11-12, 96 DPI PDFium, 색상 경계 보정 전 2px silhouette_raw_match_percent.
//! cover-linked.hwpx / cover-linked-2024.pdf: 전체 1쪽, Native/fresh WASM 각각 100.00%, 전쪽 최저 100.00%.
//! cover-mixed-images.hwpx / cover-mixed-images-2024.pdf: 전체 1쪽, 두 경로 100.00%, 전쪽 최저 100.00%.
//! 독립 기준은 한글 2024 13.0.0.3901 Print이며 혼재본은 정상 Oracle 그림 하나와 빈 연결 그림 둘이다.
//! 수정 전에는 인쇄에도 대체 상자가 남았다. 저장 줄·그림 bbox를 바꾸지 않고 정상 그림만 출력한다.
//! 같은 쪽의 제목·본문 간격·기준선과 정상 그림·빈 영역을 Native/WASM 모두 확대 판독했다.
//! 정식 증거: mydocs/pr/assets/issue8957/linked-print/ (양쪽 TSV·review·overlay), pdf/issue8957/.
//! 문서 간 내용·배치와 프로필 차이는 관계형, 혼재본의 인쇄 그림 1개는 독립 Print의 고정형 계약이다.
//! 절대 페이지 px 좌표는 고정하지 않으며 사용자 설치 글꼴을 요구하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    paint::RenderProfile,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue8957")
        .join(name);
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn content_and_placement(node: &RenderNode, records: &mut Vec<String>) {
    match &node.node_type {
        RenderNodeType::TextRun(run) => records.push(format!(
            "text {:?} {:?} {:?}",
            run.para_index, run.text, node.bbox
        )),
        RenderNodeType::Image(image) => records.push(format!(
            "image {:?} {:?} {:?} {:?}",
            image.section_index, image.para_index, image.control_index, node.bbox
        )),
        _ => {}
    }
    for child in &node.children {
        content_and_placement(child, records);
    }
}

fn image_operations(core: &DocumentCore, profile: RenderProfile) -> usize {
    core.build_page_layer_tree_with_profile(0, profile)
        .unwrap()
        .to_json()
        .matches("\"type\":\"image\"")
        .count()
}

#[test]
fn unresolved_linked_pictures_keep_editor_display_and_omit_print_placeholders() {
    let core = load("cover-linked.hwpx");
    let screen = image_operations(&core, RenderProfile::Screen);
    assert!(screen > 0, "source contains unresolved linked pictures");
    assert_eq!(image_operations(&core, RenderProfile::Print), 0);
    let editor = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Screen)
        .unwrap();
    assert!(editor.contains("[외부:"));
    for svg in [
        core.render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
            .unwrap(),
        core.render_page_svg_legacy_native(0).unwrap(),
    ] {
        assert!(!svg.contains("[외부:"));
        assert!(!svg.contains("stroke-dasharray=\"4\""));
    }
    let mut before = vec![];
    content_and_placement(&core.build_page_render_tree(0).unwrap().root, &mut before);
    assert!(before.iter().any(|entry| entry.starts_with("text ")));
    let mixed = load("cover-mixed-images.hwpx");
    let mut after = vec![];
    content_and_placement(&mixed.build_page_render_tree(0).unwrap().root, &mut after);
    assert_eq!(core.page_count(), mixed.page_count());
    assert_eq!(
        before, after,
        "loading an image does not move or lose surrounding content"
    );
    assert_eq!(image_operations(&mixed, RenderProfile::Screen), screen);
    assert_eq!(
        image_operations(&mixed, RenderProfile::Print),
        1,
        "independent mixed Hancom Print contains the embedded Oracle logo only"
    );
    let print = mixed
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .unwrap();
    assert!(print.contains("data:image/"));
    assert!(!print.contains("[외부:"));
}
