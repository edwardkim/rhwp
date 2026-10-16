//! 원본 프로그램의 굵은 글자 전진 간격과 자연 글리프 폭을 분리한다.
//! 독립 issue265 Print12쪽 T2/Widths의 한글은1000/1000em, TJ는약0.05em 추가 전진.
//! Native99.09329/fresh WASM99.09641% 전16쪽 확인 및12쪽 직접 판독 후 추가.
//! 동일한 혼용 제목 후보12/13/15쪽의 양쪽 최솟값이 가장 높은12쪽(99.65791%)을 선정.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    paint::{LayerJsonOptions, RenderProfile},
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType, TextRunNode},
    },
    DocumentCore,
};
use serde_json::Value;

fn core() -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/issue_265.hwp")).expect("원본 HWP3"),
    )
    .unwrap();
    core.set_font_environment(Some(
        FontEnvironment::from_json(r#"{"id":"paired-empty-no-substitution","substitutions":{}}"#)
            .unwrap(),
    ))
    .unwrap();
    core
}

fn paragraph_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run)
        if run.section_index == Some(0) && run.para_index == Some(121))
    {
        out.push(node);
    }
    for child in &node.children {
        paragraph_runs(child, out);
    }
}

fn text(node: &RenderNode) -> &TextRunNode {
    let RenderNodeType::TextRun(run) = &node.node_type else {
        panic!("원문 글줄");
    };
    run
}

fn source_ops<'a>(value: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(object) => {
            if value["type"] == "textRun" && value["source"]["stableSourceKey"] == key {
                out.push(value);
            }
            for child in object.values() {
                source_ops(child, key, out);
            }
        }
        Value::Array(array) => {
            for child in array {
                source_ops(child, key, out);
            }
        }
        _ => {}
    }
}

#[test]
fn native_bold_title_keeps_independent_origins_and_source_order() {
    let core = core();
    let para = &core.document().sections[0].paragraphs[121];
    let source = para.text.clone();
    assert_eq!(source, "4. 작업 할당 방식(Scheduling Algorithms)");
    for reference in &para.char_shapes {
        let shape = &core.document().doc_info.char_shapes[reference.char_shape_id as usize];
        assert_eq!(shape.spacings, [0; 7], "원본 자간0을 고쳐 맞추지 않는다");
    }
    let fonts = serde_json::to_value(&core.document().doc_info.font_faces).unwrap();
    assert!(
        !fonts.to_string().contains("native_program"),
        "프로그램 출처는 원본 속성으로 직렬화하지 않는다"
    );
    assert_eq!(core.page_count(), 16);
    let tree = core.build_page_render_tree(11).unwrap();
    let mut nodes = Vec::new();
    paragraph_runs(&tree.root, &mut nodes);
    assert_eq!(
        nodes
            .iter()
            .map(|node| text(node).text.as_str())
            .collect::<String>(),
        source
    );
    let korean: Vec<_> = nodes
        .iter()
        .filter(|node| text(node).char_start == Some(3))
        .collect();
    let english: Vec<_> = nodes
        .iter()
        .filter(|node| text(node).char_start == Some(12))
        .collect();
    assert_eq!(korean.len(), 1);
    assert_eq!(english.len(), 1);
    let (korean, english) = (korean[0], english[0]);
    assert_eq!(text(korean).text, "작업 할당 방식(");
    assert_eq!(text(english).text, "Scheduling Algorithms)");
    assert!(text(korean).style.bold && text(english).style.bold);
    assert_eq!(
        text(korean).style.font_family,
        text(english).style.font_family
    );
    assert_eq!(text(korean).style.letter_spacing, 0.0);
    assert_eq!(text(english).style.letter_spacing, 0.0);
    assert!(korean.bbox.x < english.bbox.x);
    assert!((korean.bbox.x + korean.bbox.width - english.bbox.x).abs() < 0.1);
    assert!(
        (korean.bbox.y + text(korean).baseline - english.bbox.y - text(english).baseline).abs()
            < 0.1
    );
    // 독립 Print72pt→96CSSpx. 원문 문단·scalar로 식별하며 좌표 영역으로 고르지 않는다.
    for (actual, expected) in [
        (korean.bbox.x, 145.7599995931),
        (english.bbox.x, 331.6799926758),
        (english.bbox.y + text(english).baseline, 152.7999979655),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "독립 Print±2CSSpx: {actual} vs {expected}"
        );
    }
    let json: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                11,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let mut ops = Vec::new();
    source_ops(&json["root"], "section:0/para:121/char:12", &mut ops);
    assert_eq!(ops.len(), 1, "최종 paint의 영문 소유");
    let op = ops[0];
    let last = op["clusters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cluster| cluster["textRangeUtf16"]["start"] == 21)
        .expect("마지막 닫는 괄호의 원문 소속");
    let closing_x =
        op["placement"]["runToPage"]["e"].as_f64().unwrap() + last["origin"]["x"].as_f64().unwrap();
    assert!(
        (closing_x - 609.9208).abs() <= 2.0,
        "독립 Print의 끝 글자까지 누적 간격 보존"
    );
    assert_eq!(
        serde_json::to_value(&core.document().doc_info.font_faces).unwrap(),
        fonts
    );
    assert_eq!(core.document().sections[0].paragraphs[121].text, source);
}

#[cfg(feature = "native-skia")]
#[test]
fn native_bold_tracking_does_not_stretch_the_actual_glyph() {
    use rhwp::{
        paint::{LayerNode, PageLayerTree, PaintOp},
        renderer::{
            layer_renderer::RasterRenderOptions, render_tree::BoundingBox, skia::SkiaLayerRenderer,
        },
    };
    let core = core();
    let tree = core.build_page_render_tree(11).unwrap();
    let mut nodes = Vec::new();
    paragraph_runs(&tree.root, &mut nodes);
    let source = text(
        nodes
            .into_iter()
            .find(|node| text(node).text == "작업 할당 방식(")
            .unwrap(),
    );
    let render = |natural: bool| {
        let mut run = source.clone();
        run.text = "작".into();
        run.layout_positions = None;
        if natural {
            run.style.metric_font_family = None;
        }
        let layer = PageLayerTree::new(
            64.0,
            48.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 48.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(8.0, 8.0, 40.0, 32.0),
                    run,
                )],
            ),
        );
        SkiaLayerRenderer::new()
            .render_pixels_with_options(
                &layer,
                RasterRenderOptions {
                    scale: 4.0,
                    ..Default::default()
                },
            )
            .unwrap()
    };
    let natural = render(true);
    let tracked = render(false);
    assert!(natural.bytes.chunks_exact(4).any(|pixel| pixel[3] > 0));
    let changed = natural
        .bytes
        .chunks_exact(4)
        .zip(tracked.bytes.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        changed, 0,
        "추가 전진과 독립적으로 자연 글리프 모양을 유지한다"
    );
}
