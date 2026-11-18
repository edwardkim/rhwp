//! 함수별 검사 유형은 각 함수 주석에 구분한다. 원본 HFT 전진과 paint 폭은 별개다.
//! 주석 갱신: 2026-11-10T06:44:00+09:00.
//! report-628 110쪽의 stage4 Native/fresh WASM raw는 93.35921%/93.37018%다.
//! 독립 Print의 글머리표 원점 893 → 본문 원점 975(600dpi)은 96dpi에서13.12px다.
//! paint의 Gulim U+2022 자연 폭0.375em과 원본 HFT의 전진1em은 별개다.
//! stage10 완료110쪽: Native99.79423% / fresh WASM99.79424%, 96dpi/PDFium/raw2px.
//! 전195쪽 최저94.83952%/94.84611%이며 다른 쪽에 px 핀을 확장하지 않는다.
//! 근거: mydocs/pr/assets/issue8856/stage10-hft-bullet-advance/validation.json.
//! 설치 글꼴에 의존하지 않는다. 아래 고정형만 직접 판독한110쪽 경계에±2CSSpx를 적용한다.

use rhwp::DocumentCore;
use serde_json::Value;

fn source_core() -> DocumentCore {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("mydocs/pr/assets/planet6897_20261106/pr8642/report-628-original.hwp"),
    )
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_font_environment(Some(rhwp::renderer::font_environment::FontEnvironment::from_json(
        r#"{"id":"hft-bullet-display-control","preserveSourceMetrics":true,"substitutions":{"한양신명조":"NotoSansKR-Regular"}}"#
    ).unwrap())).unwrap();
    core
}

/// 검사 유형: 관계형. 원본 글자 크기에 대한 전진과 소속 개수를 검사한다.
#[test]
fn original_hft_bullet_keeps_full_cell_before_the_following_text() {
    let core = source_core();
    let trace: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(109, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    let mut count = 0;
    for record in trace["records"].as_array().unwrap() {
        if record["source"]["character"] != "•" || record["document"]["face"] != "한양신명조"
        {
            continue;
        }
        let id = record["source"]["charShapeId"].as_u64().unwrap() as usize;
        let slot = record["document"]["languageSlot"].as_u64().unwrap() as usize;
        let shape = &core.document().doc_info.char_shapes[id];
        let size = f64::from(shape.base_size) * f64::from(shape.relative_sizes[slot]) / 100.0;
        let advance = record["layoutMetric"]["baseAdvanceHwpunit"]
            .as_f64()
            .unwrap();
        assert!(
            (advance / size - 1.0).abs() < 0.001,
            "원본 HFT 한 칸이 대체 glyph 폭이나 반각 휴리스틱으로 바뀜: {record}"
        );
        count += 1;
    }
    assert_eq!(
        count, 22,
        "좌우 셀의11개 문단 각각에 원본 bullet이 한 번씩 속한다"
    );
}

/// 검사 유형: 고정형. 독립 Print110쪽의 좌우 첫 본문 원점(96dpi)을 ±2CSSpx로 보호한다.
#[test]
fn completed_table_first_text_origins_keep_the_independent_print_positions() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(node.node_type, RenderNodeType::TextRun(_)) {
            out.push(node);
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let core = source_core();
    let tree = core.build_page_render_tree(109).unwrap();
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let mut count = 0;
    for (cell, text, x, bullet_x) in [(2, "강제적", 156.0, 142.88), (3, "자발적", 426.24, 413.12)]
    {
        let selected: Vec<_> = nodes
            .iter()
            .filter(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => {
                    run.text == text
                        && run.cell_context.as_ref().is_some_and(|ctx| {
                            ctx.parent_para_index == 587
                                && ctx.path.first().is_some_and(|entry| {
                                    entry.cell_index == cell && entry.cell_para_index == 0
                                })
                        })
                }
                _ => false,
            })
            .collect();
        assert_eq!(
            selected.len(),
            1,
            "같은 셀·문단의 본문이 한 번만 속한다: {text}"
        );
        let node = selected[0];
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        assert!(
            (node.bbox.x - x).abs() <= 2.0,
            "{text} x={} / 독립 Print={x}",
            node.bbox.x
        );
        assert!(
            (node.bbox.y + run.baseline - 268.64).abs() <= 2.0,
            "{text} 기준선"
        );
        let bullets: Vec<_> = nodes
            .iter()
            .filter(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => {
                    run.text == "•"
                        && run.cell_context.as_ref().is_some_and(|ctx| {
                            ctx.parent_para_index == 587
                                && ctx.path.first().is_some_and(|entry| {
                                    entry.cell_index == cell && entry.cell_para_index == 0
                                })
                        })
                }
                _ => false,
            })
            .collect();
        assert_eq!(bullets.len(), 1, "동일 문단의 원본 bullet 소속·중복");
        assert!((bullets[0].bbox.x - bullet_x).abs() <= 2.0, "bullet 원점");
        assert!(
            bullets[0].bbox.x < node.bbox.x,
            "bullet 뒤에 본문이 이어진다"
        );
        count += 1;
    }
    assert_eq!(count, 2, "좌우 셀의 첫 본문 원점을 모두 검사한다");
}
