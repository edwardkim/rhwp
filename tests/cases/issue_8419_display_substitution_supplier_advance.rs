//! 관계형·고정형: 다음 본문 소속·이어짐과 독립 원본 프로그램의 전진 비율. 절대 px 좌표는 고정하지 않는다.
//! 주석 갱신: 2026-11-04T23:54:05+09:00; 새 파일이며 직전 파일 변경 커밋은 없다.
//! 측정일 2026-11-04, Native 제품 4b149b6cdf / fresh WASM 빌드 head 7bb9e66fd0(같은 제품 source).
//! 원본 samples/issue5699/20099369_yeongwol_forms.hwp, 독립 Print
//! pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf.
//! 96 DPI, Native Skia/fresh WASM Canvas, 색상 경계 보정 전 2px raw 실루엣.
//! 전 4쪽의 Native 최저 99.96664% / fresh WASM 최저 99.97763%, 양쪽/PDF 쪽수 일치.
//! 대상 1쪽 Native 99.98384% / fresh WASM 99.98228%; 원본 dot와 다음 본문을 직접 판독했다.
//! 전쪽 TSV·1/3쪽 review/overlay·프로그램 출처: mydocs/pr/assets/planet6897_20261103/original_supplier/README.md.
//! 시각 측정은 전체 시스템 글꼴과 승인된 기존 대체 환경이며, 검사는 설치 글꼴 없이 공개 표시 치환을 선언한다.
//! 기대값은 HBATANG hmtx 178/1024em와 원본 글자 크기다. 원본 1HU 양자화와 실제 다음 run의 관계를 검사한다.
//! 기존 package의 같은 공개 API는 1200HU 전각 추정, 보정 package는 208HU 공급자로 판별된다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
    },
    DocumentCore,
};
use serde_json::Value;
fn text_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        out.push(node);
    }
    for child in &node.children {
        text_runs(child, out);
    }
}
#[test]
fn display_only_substitution_cannot_replace_the_original_missing_glyph_supplier_advance() {
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/issue5699/20099369_yeongwol_forms.hwp"
        ))
        .unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    // 공개 저장소 글꼴을 표시 face로 선언한다. 설치된 private/OTF 프로그램은 사용하지 않는다.
    core.set_font_environment(Some(FontEnvironment::from_json(r#"{"id":"source-dot-supplier-tripwire","preserveSourceMetrics":true,"substitutions":{"바탕":"Noto Sans KR ExtraLight","새굴림":"Malgun Gothic"}}"#).unwrap())).unwrap();
    let mut checked = 0;
    for page in 0..core.page_count() {
        let trace: Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(page, r#"{"maxCharacters":4096}"#)
                .unwrap(),
        )
        .unwrap();
        let tree = core.build_page_render_tree(page).unwrap();
        let mut runs = Vec::new();
        text_runs(&tree.root, &mut runs);
        for dot in trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["source"]["character"] == "․" && r["document"]["face"] == "바탕")
        {
            let shape_id = dot["source"]["charShapeId"].as_u64().unwrap() as usize;
            let shape = &core.document().doc_info.char_shapes[shape_id];
            let source_em = f64::from(shape.base_size) * f64::from(shape.relative_sizes[5]) / 100.0;
            // 독립 HBATANG hmtx=178, unitsPerEm=1024. 관측한 후보의 2.77px를 기대값으로 쓰지 않는다.
            let expected = source_em * 178.0 / 1024.0;
            let advance = dot["layoutMetric"]["baseAdvanceHwpunit"].as_f64().unwrap();
            assert!(
                (advance - expected).abs() <= 1.0,
                "원본 1HU 반올림 이내: {advance} vs {expected}"
            );
            assert_eq!(dot["layoutMetric"]["widthSource"], "metricGlyphFallback");
            assert_eq!(dot["layoutMetric"]["aliasResolvedFace"], "Haansoft Batang");
            let matching:Vec<_>=runs.iter().enumerate().filter(|(_,node)|matches!(&node.node_type,RenderNodeType::TextRun(run) if run.text=="․"&&run.char_shape_id==Some(shape_id as u32))).collect();
            assert_eq!(matching.len(), 1, "원본 dot의 소속과 누락·중복");
            let (index, node) = matching[0];
            let RenderNodeType::TextRun(run) = &node.node_type else {
                unreachable!()
            };
            assert!(
                run.style.font_family.contains("Noto Sans KR ExtraLight"),
                "표시 치환 경로를 실제로 실행"
            );
            let expected_width = run.style.font_size * run.style.ratio * 178.0 / 1024.0;
            assert!(
                (node.bbox.width - expected_width).abs() <= 96.0 / 7200.0,
                "배치도 원본 1HU 이내의 공급 전진을 소비"
            );
            let next = runs.get(index + 1).expect("dot 뒤 같은 원본 본문");
            let RenderNodeType::TextRun(next_run) = &next.node_type else {
                unreachable!()
            };
            assert!(next_run.text.starts_with("형사상의 모든"));
            assert!(
                (next.bbox.x - (node.bbox.x + node.bbox.width)).abs() <= 96.0 / 7200.0,
                "뒤 본문은 dot의 점유 끝에서 이어진다"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 1, "원본의 바탕 기호 슬롯 dot 한 곳을 실제 검사");
}
