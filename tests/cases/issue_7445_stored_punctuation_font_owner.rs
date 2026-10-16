//! 저장 줄의 ASCII 구두점은 폭 계산과 같은 영문 슬롯을 그린다.
//! 개인정보 전66쪽 Native/fresh WASM 최저93.77937%, 쪽수66/66 확인 후 추가.
//! 같은 하이픈 경계10/29/53쪽 모두 양쪽100%; 첫10쪽을 선정했다.
//! 독립 Hancom Print의 PalatinoLinotype-Roman15pt, x64.08pt/baseline106.32pt를
//! 96CSSpx로 환산한 위치를 ±2px로 확인하며 원문 소속·순서도 함께 검증한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};

fn paragraph_runs<'a>(node: &'a RenderNode, runs: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.section_index == Some(0) && run.para_index == Some(95))
    {
        runs.push(node);
    }
    for child in &node.children {
        paragraph_runs(child, runs);
    }
}

#[test]
fn stored_hyphen_uses_the_measured_latin_face_and_preserves_source_membership() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx"))
            .expect("정식 개인정보 원본"),
    )
    .unwrap();
    let source = core.document().sections[0].paragraphs[95].text.clone();
    assert_eq!(source, " - 고용친화적 규제설계");
    let environment = FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
    ))
    .unwrap();
    core.set_font_environment(Some(environment)).unwrap();
    assert_eq!(core.page_count(), 66);
    let tree = core.build_page_render_tree(9).unwrap();
    let mut nodes = Vec::new();
    paragraph_runs(&tree.root, &mut nodes);
    let runs: Vec<_> = nodes
        .iter()
        .map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => run,
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(
        runs.iter().map(|run| run.text.as_str()).collect::<String>(),
        source,
        "언어 슬롯 분할로 원문 누락·중복·순서 변경이 없어야 한다"
    );
    let hyphens: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == "-" && run.char_start == Some(1)))
        .collect();
    assert_eq!(hyphens.len(), 1, "원문 두 번째 scalar의 단일 paint 소유");
    let hyphen = hyphens[0];
    let RenderNodeType::TextRun(run) = &hyphen.node_type else {
        unreachable!();
    };
    assert_eq!(run.char_shape_id, Some(45));
    assert_eq!(run.style.font_family, "Palatino Linotype");
    assert_eq!(
        run.style
            .original_glyph_metric_family
            .as_deref()
            .map(String::as_str),
        Some("HCI Poppy")
    );
    assert_eq!(run.style.font_size, 20.0, "독립 Print 15pt");
    assert!((hyphen.bbox.x - 85.44).abs() <= 2.0, "독립 Print x±2CSSpx");
    assert!(
        (hyphen.bbox.y + run.baseline - 141.76).abs() <= 2.0,
        "독립 Print 기준선±2CSSpx"
    );
    let body = runs
        .iter()
        .find(|run| run.text.contains("고용친화적"))
        .unwrap();
    assert!(body.style.font_family.contains("휴먼명조"));
    assert_ne!(body.style.font_family, run.style.font_family);
    assert_eq!(core.document().sections[0].paragraphs[95].text, source);
}
