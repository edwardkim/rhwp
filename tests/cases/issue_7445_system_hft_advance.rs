//! 시스템 HFT의 자연 폭·굵게 간격을 표시용 TTF의 폭으로 바꾸지 않는다.
//! 독립 한컴2020 Print의 전11172자/ASCII94자, 12·16pt 일반/굵게에서 확인한 em 계약이다.
//! 같은 원본 전2쪽 Native171 최저91.20154%, fresh WASM174 최저92.44329%를 먼저 확인했다.
//! 절대 화면 px 핀은 추가하지 않는다. stage8 C438의 전후 TSV·review/overlay를 참조한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use serde_json::Value;
fn open() -> DocumentCore {
    DocumentCore::from_bytes(
        &std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/issue6860/3067979-road-lighting-photometric-hancom2020.hwpx"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn run_for<'a>(n: &'a RenderNode, id: u32) -> Option<&'a RenderNode> {
    if let RenderNodeType::TextRun(t) = &n.node_type {
        if t.char_shape_id == Some(id) && t.text.starts_with("부록") {
            return Some(n);
        }
    }
    n.children.iter().find_map(|c| run_for(c, id))
}
fn decision(core: &DocumentCore, id: u32, ch: &str) -> Value {
    let v: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":512}"#)
            .unwrap(),
    )
    .unwrap();
    v["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["source"]["charShapeId"] == id && r["source"]["character"] == ch)
        .unwrap()
        .clone()
}
fn title_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type,RenderNodeType::TextRun(r)if r.para_index==Some(1)&&r.section_index==Some(0)&&r.char_shape_id==Some(8)&&r.cell_context.is_none())
    {
        out.push(node)
    }
    for c in &node.children {
        title_runs(c, out);
    }
}
fn original_program_advance(id: u32) {
    let core = open();
    assert_eq!(core.page_count(), 2);
    let tree = core.build_page_render_tree(0).unwrap();
    let node = run_for(&tree.root, id).unwrap();
    let RenderNodeType::TextRun(run) = &node.node_type else {
        unreachable!()
    };
    assert!(run.style.bold);
    let r = decision(&core, id, "부");
    assert_eq!(r["document"]["face"], "시스템");
    assert_eq!(r["document"]["altType"], 2);
    let source_em = f64::from(core.document().doc_info.char_shapes[id as usize].base_size);
    let actual = r["layoutMetric"]["baseAdvanceHwpunit"].as_f64().unwrap();
    assert!(
        (actual - source_em * 0.88).abs() <= 1.,
        "독립 Type3 자연0.88em: actual={actual},expected={}",
        source_em * 0.88
    );
    let final_advance = r["layoutMetric"]["finalAdvanceHwpunit"].as_f64().unwrap();
    assert!(
        (final_advance - source_em * 0.93).abs() <= 1.,
        "원본 자연폭과 굵게0.05em을 같은 전진에 반영: actual={final_advance}"
    );
    assert!(
        run.style.extra_char_spacing.abs() < 1e-6,
        "원본 폭으로 fit하는 줄에 표시용 TTF 폭의 사후 압축을 하지 않는다"
    );
    let mut title = Vec::new();
    title_runs(&tree.root, &mut title);
    let actual = title
        .iter()
        .map(|n| match &n.node_type {
            RenderNodeType::TextRun(t) => t.text.as_str(),
            _ => unreachable!(),
        })
        .collect::<String>();
    assert_eq!(
        actual,
        core.document().sections[0].paragraphs[1].text,
        "제목의 순서·누락·중복"
    );
    let body = tree
        .root
        .children
        .iter()
        .find(|n| matches!(&n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let mut last = body.bbox.x;
    for n in title {
        assert!(n.bbox.x + 1e-6 >= last, "제목 run의 순서·겹침");
        last = n.bbox.x + n.bbox.width;
        assert!(
            last <= body.bbox.x + body.bbox.width + 0.02,
            "제목의 본문 포함"
        );
    }
}
#[test]
fn title_uses_original_hft_natural_width_and_tracking() {
    original_program_advance(8)
}
#[test]
fn caption_uses_the_same_program_at_its_own_size() {
    let core = open();
    let tree = core.build_page_render_tree(0).unwrap();
    let node = run_for(&tree.root, 12).unwrap();
    let RenderNodeType::TextRun(run) = &node.node_type else {
        unreachable!()
    };
    assert_eq!(run.text, "부록-<표 ");
    assert!(run.style.bold);
    // 독립 Type3의 한글3자880, '-'406, '<'434와 비공백5자의 굵게50, 기존 반각 공백500.
    let expected_em = 3. * 0.88 + 0.406 + 0.434 + 5. * 0.05 + 0.5;
    assert!(
        (node.bbox.width / run.style.font_size - expected_em).abs() <= 0.01,
        "캡션의 같은 원본 프로그램 전진: actual={},independent={expected_em}",
        node.bbox.width / run.style.font_size
    );
}
#[test]
fn true_type_body_keeps_its_own_program() {
    let core = open();
    let r = decision(&core, 2, "도");
    assert_eq!(r["document"]["altType"], 1);
    let source_em = f64::from(core.document().doc_info.char_shapes[2].base_size);
    assert!(
        (r["layoutMetric"]["baseAdvanceHwpunit"].as_f64().unwrap() - source_em).abs() <= 1.,
        "바탕 TrueType의 전각을 HFT 시스템0.88em으로 바꾸지 않는다"
    );
}
