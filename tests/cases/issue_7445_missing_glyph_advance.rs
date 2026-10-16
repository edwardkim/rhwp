//! C78: 원본66쪽 Native/fresh WASM 최저93.77937%,50쪽 직접 판독.
//! 동일 기호 제목7/26/50 중 양쪽 최솟값이 가장 높은50쪽99.99825% 선정.
//! 캡처192DPI, 독립 좌표96CSSpx±2 및 원문·단일 소유·순서 관계를 함께 보호.
//! 결측 기호를 실제로 그리는 선언 공급자와 측정 전진을 일치시킨다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerJsonOptions, RenderProfile},
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
use serde_json::Value;
fn runs<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&n.node_type, RenderNodeType::TextRun(r) if r.section_index==Some(0) && r.para_index==Some(458))
    {
        out.push(n)
    }
    for c in &n.children {
        runs(c, out)
    }
}
fn ops<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
    match v {
        Value::Object(o) => {
            if v["type"] == "textRun"
                && v["source"]["stableSourceKey"] == "section:0/para:458/char:3"
            {
                out.push(v)
            }
            for x in o.values() {
                ops(x, out)
            }
        }
        Value::Array(a) => {
            for x in a {
                ops(x, out)
            }
        }
        _ => {}
    }
}
#[test]
fn missing_hyphenation_point_keeps_its_supplier_advance_and_source_order() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    {
        let mut core = DocumentCore::from_bytes(
            &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx")).unwrap(),
        )
        .unwrap();
        {
            core.set_font_environment(Some(FontEnvironment::from_json(include_str!("../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json")).unwrap())).unwrap();
        }
        let source = core.document().sections[0].paragraphs[458].text.clone();
        assert_eq!(source, "1. 목적‧수단 간 비례적 타당성");
        assert_eq!(core.page_count(), 66);
        let t = core.build_page_render_tree(49).unwrap();
        let mut ns = vec![];
        runs(&t.root, &mut ns);
        assert_eq!(
            ns.iter()
                .map(|n| match &n.node_type {
                    RenderNodeType::TextRun(r) => r.text.as_str(),
                    _ => unreachable!(),
                })
                .collect::<String>(),
            source
        );
        let selected:Vec<_>=ns.iter().filter(|n|matches!(&n.node_type,RenderNodeType::TextRun(r)if r.char_start==Some(3)&&r.text=="목적‧수단 간 비례적 타당성")).collect();
        assert_eq!(selected.len(), 1);
        let RenderNodeType::TextRun(r) = &selected[0].node_type else {
            unreachable!()
        };
        assert!(r.style.font_family.contains("휴먼명조"));
        assert!(r.style.font_family.contains("Haansoft Batang"));
        assert!(r.style.bold);
        let j: Value = serde_json::from_str(
            &core
                .get_page_layer_tree_with_options_native(
                    49,
                    RenderProfile::Print,
                    LayerJsonOptions {
                        omit_image_bytes: true,
                        omit_font_bytes: true,
                    },
                )
                .unwrap(),
        )
        .unwrap();
        let mut found = vec![];
        ops(&j["root"], &mut found);
        assert_eq!(found.len(), 1);
        let op = found[0];
        let clusters = op["clusters"].as_array().unwrap();
        let origin = |offset: u64| {
            let c: Vec<_> = clusters
                .iter()
                .filter(|c| c["textRangeUtf16"]["start"] == offset)
                .collect();
            assert_eq!(c.len(), 1);
            op["placement"]["runToPage"]["e"].as_f64().unwrap()
                + c[0]["origin"]["x"].as_f64().unwrap()
        };
        let point = origin(2);
        let next = origin(3);
        assert!(next > point);
        // 독립 Print: 기호105.36pt, 다음 '수'108pt, 기준선109.56pt. 96CSSpx±2.
        assert!((point - 140.48).abs() <= 2.0);
        assert!((next - 144.0).abs() <= 2.0, "실제 공급자 다음 원점: {next}");
        assert!((op["placement"]["runToPage"]["f"].as_f64().unwrap() - 146.08).abs() <= 2.0);
        assert_eq!(core.document().sections[0].paragraphs[458].text, source);
    }
}
