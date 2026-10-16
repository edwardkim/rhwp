//! 원본의 부분 글리프 전진과 기호 슬롯은 공개 글꼴 결정 API로 검증한다.
//! 물리 페이지의 임시 배치를 고정하지 않는 프로그램 메트릭 계약이다.
//! 독립 Print 프로그램과 전쪽/중요 px 증적은 source-program-and-inline-spacing/validation.json.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::Value;

fn core(face: &str, text: &str, symbol_face: Option<&str>) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        for font in fonts {
            font.name = face.into();
            font.alt_type = 1;
        }
    }
    if let Some(symbol) = symbol_face {
        for font in &mut document.doc_info.font_faces[5] {
            font.name = symbol.into();
        }
    }
    for style in &mut document.doc_info.char_shapes {
        style.base_size = 7500;
        style.ratios = [100; 7];
        style.spacings = [0; 7];
        style.relative_sizes = [100; 7];
    }
    core.set_document(document);
    core
}

fn record(core: &DocumentCore, character: &str) -> Value {
    let value: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":100}"#)
            .unwrap(),
    )
    .unwrap();
    value["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["source"]["character"] == character)
        .expect("원본 글자 기록")
        .clone()
}

fn advance(record: &Value) -> f64 {
    record["layoutMetric"]["baseAdvanceHwpunit"]
        .as_f64()
        .unwrap()
}

#[test]
fn gulim_missing_point_publishes_the_measured_supplier_to_paint() {
    // 프로그램 cmap·독립 PDF 공급자 계약이며 법원 서식의 미달 배치를 고정하지 않는다.
    let mut program = core("굴림", "가․가", None);
    let original = program.export_hwp_native().unwrap();
    let dot = record(&program, "․");
    assert_eq!(dot["document"]["face"], "굴림");
    assert_eq!(
        dot["layoutName"]["cssFamilyChain"],
        serde_json::json!(["굴림", "Haansoft Dotum"]),
        "확인된 Gulim cmap 결측은 독립 Print의 측정 공급자에 연결"
    );
    let hangul = record(&program, "가");
    assert!(
        (advance(&dot) / advance(&hangul) - 178.0 / 1024.0).abs() <= 4.0 / advance(&hangul),
        "실제 HDOTUM 프로그램의 자연 전진을 유지"
    );
    program
        .set_font_environment(Some(
            FontEnvironment::from_json(
                r#"{"id":"caller-gulim-glyphs","substitutions":{},"glyphFallbacks":{"굴림":["HCR Dotum"]}}"#,
            )
            .unwrap(),
        ))
        .unwrap();
    assert_eq!(
        record(&program, "․")["layoutName"]["cssFamilyChain"],
        serde_json::json!(["굴림", "HCR Dotum"]),
        "호출자가 지정한 공급 체인은 기본 체인보다 우선"
    );
    assert_eq!(program.export_hwp_native().unwrap(), original);
}

#[test]
fn measured_original_glyph_survives_legacy_type1_display_fallback() {
    let core = core("HCI Poppy", "A/A·", None);
    let slash = record(&core, "/");
    assert_eq!(
        slash["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    // 독립2020/2024 Print의 동일 CharProc SHA와 Widths=605/1000em.
    // 기존4HU 장치 격자의 반 격자만 허용하며 전체 HFT 표를 승격하지 않는다.
    assert!((advance(&slash) - 7500.0 * 0.605).abs() <= 2.0);
    let fallback = core_with_palatino_program();
    for character in ["A", "·"] {
        assert_eq!(
            advance(&record(&core, character)),
            advance(&record(&fallback, character)),
            "부분 표에 없는 글자는 표시 프로그램의 폭을 유지함:{character}"
        );
    }
}

fn core_with_palatino_program() -> DocumentCore {
    core("Palatino Linotype", "A/A·", None)
}

#[test]
fn explicit_true_type_program_keeps_its_own_slash_advance() {
    let mut core = core("HCI Poppy", "A/A·", None);
    let original = core.export_hwp_native().unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"verified-palatino-program","substitutions":{"HCI Poppy":"Palatino Linotype"},"trueTypeFaces":["Palatino Linotype"]}"#,
    ).unwrap())).unwrap();
    let slash = record(&core, "/");
    assert_eq!(
        advance(&slash),
        advance(&record(&core_with_palatino_program(), "/"))
    );
    assert_ne!(
        slash["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        original,
        "선택 환경은 원본을 수정하지 않음"
    );
}

#[test]
fn onepoint_uses_its_symbol_program_before_glyph_resolution() {
    let core = core("휴먼명조", "가․나", Some("한양신명조"));
    let dot = record(&core, "․");
    assert_eq!(
        dot["document"]["face"], "한양신명조",
        "독립 Type3의 원본 기호 슬롯"
    );
    assert!(
        (advance(&dot) - 7500.0).abs() <= 2.0,
        "독립 전각 프로그램 전진"
    );
}

#[test]
fn explicit_onepoint_provider_uses_its_own_advance() {
    let mut core = core("휴먼명조", "가․나", None);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"selected-provider","substitutions":{},"trueTypeFaces":["휴먼명조"],"glyphFallbacks":{"휴먼명조":["Haansoft Batang"]}}"#,
    ).unwrap())).unwrap();
    let dot = record(&core, "․");
    assert!(
        (advance(&dot) - 7500.0 * 178.0 / 1024.0).abs() <= 2.0,
        "명시 공급자의 자연 전진"
    );
    assert_ne!(dot["layoutMetric"]["widthSource"], "heuristicFullwidth");
}

#[test]
fn hyphenation_point_selects_the_symbol_slot_before_glyph_fallback() {
    // 한컴2020 Print의 교차 슬롯 probe: 한글=한컴바탕, 영문=한컴돋움,
    // 기호=함초롬돋움일 때 U+2027은 HCRDotum이다. 한글/영문 앞 모두 같다.
    for text in ["가‧나", "A‧B"] {
        let core = core("한컴바탕", text, Some("함초롬돋움"));
        let point = record(&core, "‧");
        assert_eq!(point["document"]["face"], "함초롬돋움");
        assert_eq!(point["document"]["languageSlot"], 5);
        // 표시 폴백 전에 source 슬롯을 결정하며 모델 문자열은 보존한다.
        assert_eq!(core.document().sections[0].paragraphs[0].text, text);
    }
}

#[test]
fn worklife_symbol_point_preserves_source_and_independent_completed_page_position() {
    // C93 Native/fresh WASM 전6쪽90% 이상. 변경 경계인5쪽은 각각
    // raw99.93989/99.94147%,96DPI 직접 review/overlay 판독 완료.
    // 독립 Print T19의 원점292.641388/83.428825pt를96CSSpx로 환산한다.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/issue6145/worklife_balance_index_156607916.hwpx"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(core.page_count(), 6);
    let (pi, text) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.contains("모든 지역에서") && p.text.contains("국‧공립"))
        .map(|(i, p)| (i, p.text.clone()))
        .unwrap();
    let offset = text.chars().position(|c| c == '‧').unwrap();
    let tree = core.build_page_render_tree(4).unwrap();
    fn collect<'a>(
        n: &'a rhwp::renderer::render_tree::RenderNode,
        pi: usize,
        v: &mut Vec<&'a rhwp::renderer::render_tree::RenderNode>,
    ) {
        if matches!(&n.node_type, rhwp::renderer::render_tree::RenderNodeType::TextRun(r)
            if r.section_index==Some(0) && r.para_index==Some(pi))
        {
            v.push(n);
        }
        for c in &n.children {
            collect(c, pi, v);
        }
    }
    let mut nodes = vec![];
    collect(&tree.root, pi, &mut nodes);
    let rebuilt: String = nodes
        .iter()
        .map(|n| match &n.node_type {
            rhwp::renderer::render_tree::RenderNodeType::TextRun(r) => r.text.as_str(),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(rebuilt, text, "원문 순서·누락·중복 보호");
    let selected: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(&n.node_type,
        rhwp::renderer::render_tree::RenderNodeType::TextRun(r)
        if r.char_start==Some(offset) && r.text=="‧")
        })
        .collect();
    assert_eq!(selected.len(), 1);
    let next: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(&n.node_type,
        rhwp::renderer::render_tree::RenderNodeType::TextRun(r)
        if r.char_start==Some(offset+1) && r.text.starts_with('공'))
        })
        .collect();
    assert_eq!(next.len(), 1);
    assert!(
        selected[0].bbox.x + selected[0].bbox.width <= next[0].bbox.x + 0.01,
        "다음 본문의 앞에서 종료"
    );
    assert!((selected[0].bbox.x - 390.1885172526042).abs() <= 2.0);
    assert!((next[0].bbox.x - 401.2225341796875).abs() <= 2.0);
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                4,
                rhwp::paint::RenderProfile::Print,
                rhwp::paint::LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    fn find<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
        match v {
            Value::Object(o) => {
                if v["type"] == "textRun" && v["source"]["stableSourceKey"] == key {
                    out.push(v)
                }
                for x in o.values() {
                    find(x, key, out)
                }
            }
            Value::Array(a) => {
                for x in a {
                    find(x, key, out)
                }
            }
            _ => {}
        }
    }
    let mut ops = vec![];
    find(
        &layer,
        &format!("section:0/para:{pi}/char:{offset}"),
        &mut ops,
    );
    assert_eq!(ops.len(), 1);
    let baseline = ops[0]["placement"]["runToPage"]["f"].as_f64().unwrap();
    assert!((baseline - 111.23843383789062).abs() <= 2.0);
    assert_eq!(core.document().sections[0].paragraphs[pi].text, text);
}
