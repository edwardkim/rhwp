//! 고정형: 독립 Print의 가사 문자 원점을 96DPI ±2px로 검사한다.
//! 주석 갱신: 2026-11-07 16:23:50 KST.
//! 수정 전 1쪽 raw는 Native 99.86093% / fresh WASM 99.83671%다.
//! 수정 후 1쪽: Native 99.96356% / fresh WASM 99.96269%; 전4쪽 최저 98.75606% / 98.62409%.
//! 명시적 Print 글꼴 환경의 가사·하이픈을 직접 확대 판독했다. 모델 검사는 시스템 글꼴 설치에 의존하지 않는다.
//! 근거: mydocs/pr/assets/issue1994/lyric-spacing-20261107/README.md.
//! 기준 PDF: pdf/semanticist21-20261008/basic-issue1994_behindtext_table_20200830-8e7a95cf5919-2020.pdf.
use rhwp::renderer::{
    font_environment::FontEnvironment,
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    render_tree::{RenderNode, RenderNodeType, TextRunNode},
};
use rhwp::DocumentCore;
fn find<'a>(node: &'a RenderNode, text: &str) -> Option<(&'a RenderNode, &'a TextRunNode)> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == text {
            return Some((node, run));
        }
    }
    node.children.iter().find_map(|child| find(child, text))
}
fn check(text: &str, origins_px: [f64; 4]) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input =
        std::fs::read(root.join("samples/basic/issue1994_behindtext_table_20200830.hwp")).unwrap();
    let mut core = DocumentCore::from_bytes(&input).unwrap();
    let env =
        std::fs::read_to_string(root.join(
            "mydocs/pr/assets/issue1994/print-frame-20261107/reference-font-environment.json",
        ))
        .unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(&env).unwrap()))
        .unwrap();
    let page = core.build_page_render_tree(0).unwrap();
    let (node, run) = find(&page.root, text).expect("가사·하이픈의 실제 출력 소유");
    let positions = EmbeddedTextMeasurer.compute_char_positions(&run.text, &run.style);
    for (index, expected) in origins_px.into_iter().enumerate() {
        let actual = node.bbox.x + positions[index];
        assert!(
            (actual - expected).abs() <= 2.0,
            "{text} char{index}: actual={actual}, independent={expected}"
        );
    }
}
#[test]
fn first_lyric_keeps_the_printed_character_origins_before_dash_leaders() {
    check(
        "놀라운일----------",
        [
            242.93450927734375,
            253.00323486328125,
            263.0718994140625,
            273.140625,
        ],
    );
}
#[test]
fn second_lyric_keeps_its_own_character_origins_before_dash_leaders() {
    check(
        "전파하리---------",
        [369.9956461588542, 380.704, 391.4124, 402.1208],
    );
}
