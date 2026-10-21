//! #7445 Paper 표와 저장 본문 축의 소유 관계.
//! 같은 원본의 Native/fresh WASM 전140쪽은 모두90% 이상이며1쪽은양쪽100%다.
//! 1쪽review/overlay직접판독 후 독립 Print 좌표±2CSSpx와 내용·소유 관계를 보호한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    model::control::Control,
    model::shape::{TextWrap, VertRelTo},
    paint::{LayerJsonOptions, RenderProfile},
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use serde_json::Value;
fn normalized(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}
fn body_runs<'a>(node: &'a RenderNode, para: usize, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type,RenderNodeType::TextRun(r)if r.section_index==Some(0)&&r.para_index==Some(para)&&r.cell_context.is_none())
    {
        out.push(node);
    }
    for c in &node.children {
        body_runs(c, para, out);
    }
}
fn titles(core: &DocumentCore) -> [f64; 2] {
    assert_eq!(core.page_count(), 140, "독립 한컴2024 Print의 전체 쪽수");
    let page = core.build_page_render_tree(0).unwrap();
    let mut result = [0.0; 2];
    for (i, pi) in [5, 6].into_iter().enumerate() {
        let source = &core.document().sections[0].paragraphs[pi];
        let mut runs = vec![];
        body_runs(&page.root, pi, &mut runs);
        assert!(!runs.is_empty(), "두 원문 제목은 첫 쪽 본문에 있다");
        let text: String = runs
            .iter()
            .filter_map(|n| {
                if let RenderNodeType::TextRun(r) = &n.node_type {
                    Some(r.text.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            normalized(&text),
            normalized(&source.text),
            "두 제목의 원문·순서·누락·중복"
        );
        let first: Vec<_> = runs
            .iter()
            .filter_map(|n| {
                if let RenderNodeType::TextRun(r) = &n.node_type {
                    (r.char_start == Some(0)).then_some((n.bbox.y + r.baseline, r.text.as_str()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(first.len(), 1, "원문 첫 글자는 첫 쪽에 한 번만 배치된다");
        assert!(first[0]
            .1
            .starts_with(if pi == 5 { "HIV" } else { "Validation" }));
        result[i] = first[0].0;
    }
    assert!(result[0] < result[1], "원문 한국어/영어 제목 순서");
    result
}
fn painted<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
    if v["type"] == "textRun" && v["source"]["stableSourceKey"] == key {
        out.push(v);
    }
    match v {
        Value::Object(o) => {
            for x in o.values() {
                painted(x, key, out)
            }
        }
        Value::Array(a) => {
            for x in a {
                painted(x, key, out)
            }
        }
        _ => {}
    }
}
#[test]
fn paper_object_margins_do_not_translate_the_source_owned_cover_titles() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue2006/1790387_prep_final_report.hwpx"
    ))
    .unwrap();
    let mut original = DocumentCore::from_bytes(&bytes).unwrap();
    original.set_hangul2024_compat(true);
    let para = &original.document().sections[0].paragraphs[0];
    let table = para
        .controls
        .iter()
        .find_map(|c| {
            if let Control::Table(t) = c {
                Some(t)
            } else {
                None
            }
        })
        .unwrap();
    assert!(!table.common.treat_as_char);
    assert_eq!(table.common.vert_rel_to, VertRelTo::Paper);
    assert_eq!(table.common.text_wrap, TextWrap::TopAndBottom);
    assert!(para.text.is_empty());
    let host = para.line_segs.first().unwrap();
    let next = &original.document().sections[0].paragraphs[1].line_segs[0];
    assert_eq!(
        next.vertical_pos,
        host.vertical_pos + host.line_height + host.line_spacing,
        "원본 저장 줄 상자와 다음 원점의 증거"
    );
    assert!(table.outer_margin_top > 0 && table.outer_margin_bottom > 0);
    let baseline = titles(&original);
    // 원본 독립 Print PDF SHA9580bb33…의 text matrix를 CSS96으로 변환했다.
    // 한/영 제목의 첫 글리프 소유로 고르고 후보 bbox를 기대값으로 복사하지 않는다.
    let paint: Value = serde_json::from_str(
        &original
            .get_page_layer_tree_with_options_native(
                0,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    for (pi, expected_x, expected_y) in [
        (5, 89.87129333333333, 307.823592),
        (6, 90.03120666666666, 415.86551733333334),
    ] {
        let key = format!("section:0/para:{pi}/char:0");
        let mut out = vec![];
        painted(&paint, &key, &mut out);
        assert_eq!(out.len(), 1, "독립 제목 첫 글리프의 최종 Print 단일 소유");
        let matrix = &out[0]["placement"]["runToPage"];
        let x = matrix["e"].as_f64().unwrap();
        let y = matrix["f"].as_f64().unwrap();
        assert!(
            (x - expected_x).abs() <= 2.0,
            "독립 Print 제목 x: {x} vs {expected_x}"
        );
        assert!(
            (y - expected_y).abs() <= 2.0,
            "독립 Print 제목 기준선: {y} vs {expected_y}"
        );
    }
    for (top, bottom) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut doc = original.document().clone();
        let table = doc.sections[0].paragraphs[0]
            .controls
            .iter_mut()
            .find_map(|c| {
                if let Control::Table(t) = c {
                    Some(t)
                } else {
                    None
                }
            })
            .unwrap();
        if top {
            table.common.margin.top = 0;
            table.outer_margin_top = 0;
        }
        if bottom {
            table.common.margin.bottom = 0;
            table.outer_margin_bottom = 0;
        }
        let mut changed = DocumentCore::new_empty();
        changed.set_document(doc);
        changed.set_hangul2024_compat(true);
        let after = titles(&changed);
        for (a, b) in after.into_iter().zip(baseline) {
            assert!((a-b).abs()<0.05,"Paper 개체 여백은 저장 본문 축을 이동시키지 않는다: top0={top},bottom0={bottom}, {a} vs {b}");
        }
    }
}
