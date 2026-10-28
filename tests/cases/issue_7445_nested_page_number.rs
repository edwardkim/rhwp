//! 중첩 셀의 구역 쪽번호 선언이 실제 꼬리말과 print paint까지 보존된다.
//! 같은 원본·독립 Print 전6쪽 최저97.61944/97.61696%,6쪽99.99225/99.99226%.
//! Native/fresh WASM review·overlay 직접 판독과 독립 원점 근거는 stage5 C77에 기록한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::{control::Control, paragraph::Paragraph},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use serde_json::Value;
fn declarations(ps: &[Paragraph], depth: usize, out: &mut Vec<(usize, u8)>) {
    for p in ps {
        for c in &p.controls {
            match c {
                Control::PageNumberPos(n) => out.push((depth, n.position)),
                Control::Table(t) => {
                    for cell in &t.cells {
                        declarations(&cell.paragraphs, depth + 1, out)
                    }
                }
                _ => {}
            }
        }
    }
}
fn footer_runs<'a>(n: &'a RenderNode, inside: bool, out: &mut Vec<&'a RenderNode>) {
    let inside = inside || matches!(n.node_type, RenderNodeType::Footer);
    if inside && matches!(n.node_type, RenderNodeType::TextRun(_)) {
        out.push(n)
    }
    for c in &n.children {
        footer_runs(c, inside, out)
    }
}
fn find_text<'a>(v: &'a Value, text: &str, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(o) => {
            if v["type"] == "textRun" && v["text"] == text {
                out.push(v)
            }
            for x in o.values() {
                find_text(x, text, out)
            }
        }
        Value::Array(a) => {
            for x in a {
                find_text(x, text, out)
            }
        }
        _ => {}
    }
}
#[test]
fn nested_page_number_reaches_every_footer_and_the_last_page_print_paint() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/issue6145/worklife_balance_index_156607916.hwpx"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(core.page_count(), 6);
    let original_text: Vec<_> = core.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| p.text.clone())
        .collect();
    let mut positions = vec![];
    declarations(&core.document().sections[0].paragraphs, 0, &mut positions);
    assert_eq!(
        positions,
        [(1, 5)],
        "쪽번호 선언은 원본 표 셀에 한 번 있고 위치는 BOTTOM_CENTER다"
    );
    for i in 0..6 {
        let tree = core.build_page_render_tree(i).unwrap();
        let mut runs = vec![];
        footer_runs(&tree.root, false, &mut runs);
        assert_eq!(
            runs.len(),
            1,
            "{}쪽 쪽번호는 실제 꼬리말에 한 번 있어야 한다",
            i + 1
        );
        let RenderNodeType::TextRun(run) = &runs[0].node_type else {
            unreachable!()
        };
        assert_eq!(run.text, format!("- {} -", i + 1));
        let footer = tree
            .root
            .children
            .iter()
            .find(|n| matches!(n.node_type, RenderNodeType::Footer))
            .unwrap();
        assert!(runs[0].bbox.y >= footer.bbox.y);
        assert!(runs[0].bbox.y + runs[0].bbox.height <= footer.bbox.y + footer.bbox.height);
    }
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                5,
                rhwp::paint::RenderProfile::Print,
                rhwp::paint::LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let mut ops = vec![];
    find_text(&layer, "- 6 -", &mut ops);
    assert_eq!(ops.len(), 1);
    let op = ops[0];
    assert_eq!(op["clusters"].as_array().unwrap().len(), 5);
    assert!(op["style"]["fontFamily"]
        .as_str()
        .unwrap()
        .split(',')
        .any(|f| matches!(f.trim(), "굴림" | "Gulim")));
    // 독립 Print p6 Gulim '- 6 -' 원점283.2864685/797.0089722pt →96CSSpx.
    // 전체6쪽90% 및 선택6쪽 양쪽98%·직접 review/overlay를 먼저 확인한다.
    assert!((op["placement"]["runToPage"]["e"].as_f64().unwrap() - 377.7152913411458).abs() <= 2.0);
    assert!(
        (op["placement"]["runToPage"]["f"].as_f64().unwrap() - 1062.6786295572917).abs() <= 2.0
    );
    assert_eq!(
        core.document().sections[0]
            .paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect::<Vec<_>>(),
        original_text
    );
}
