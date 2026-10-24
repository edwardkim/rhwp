//! 원본 글꼴 프로그램·기호 소유는 공개 글꼴 결정 API로 확인한다.
//! 전쪽 Native/fresh WASM90%와 해당 쪽98%·직접 판독을 마친 실물 경계는
//! 내용 관계와 독립±2CSSpx로 함께 보호한다. 근거: stage5 C75/C76 보고서.
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
    let fallback = core_with_palatino_program();
    // 독립2020 Print에서 Batang은 선택되지 않은 대체 후보로 남아 Type3이고,
    // Palatino 대체는 실제 CID TrueType으로 선택된다. 후보 존재와 선택을 구분한다.
    for alternate in [None, Some("Batang"), Some("Palatino Linotype")] {
        let mut program = core("HCI Poppy", "A/A·", None);
        let mut source = program.document().clone();
        for fonts in &mut source.doc_info.font_faces {
            for font in fonts {
                font.subst_font = alternate.map(|face| rhwp::model::style::SubstFont {
                    face: face.into(),
                    font_type: 1,
                    ..Default::default()
                });
            }
        }
        program.set_document(source);
        let original = program.export_hwpx_native().unwrap();
        let slash = record(&program, "/");
        if alternate == Some("Palatino Linotype") {
            assert_eq!(advance(&slash), advance(&record(&fallback, "/")));
            assert_ne!(
                slash["layoutMetric"]["widthSource"],
                "measuredOriginalGlyph"
            );
        } else {
            assert_eq!(
                slash["layoutMetric"]["widthSource"],
                "measuredOriginalGlyph"
            );
            // 원본/저장2022 Print의 동일 CharProc SHA, Widths605/1000em.
            // 기존4HU 장치 격자의 반 격자만 허용하며 전체 HFT 표를 승격하지 않는다.
            assert!((advance(&slash) - 7500.0 * 0.605).abs() <= 2.0);
        }
        for character in ["A", "·"] {
            assert_eq!(
                advance(&record(&program, character)),
                advance(&record(&fallback, character)),
                "미측정 글자는 표시 프로그램에 남는다:{alternate:?}/{character}"
            );
        }
        assert_eq!(
            program.export_hwpx_native().unwrap(),
            original,
            "선택은 원본을 수정하지 않는다"
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

#[test]
fn original_hci_plus_keeps_its_independent_type3_advance() {
    // 일·생활균형 독립 Print 4/5쪽 T10: ToUnicode002B,
    // Widths605, FontMatrix.001. 표시 Palatino의 반각 폭을 원본 폭으로 쓰지 않는다.
    let program = core("HCI Poppy", "A+A", None);
    let plus = record(&program, "+");
    assert!(
        (advance(&plus) - 7500.0 * 0.605).abs() <= 2.0,
        "HCI 원본 +의 자연 전진: {}",
        advance(&plus)
    );
    assert_eq!(plus["layoutMetric"]["widthSource"], "measuredOriginalGlyph");
    assert_eq!(program.document().sections[0].paragraphs[0].text, "A+A");
}

#[test]
fn regular_hci_comparison_signs_keep_the_independent_program_and_ttf_boundary() {
    // 별도 ASCII95 ×6크기 한컴2020 Print: 일반 HCI Poppy의 <=>도
    // Type3 FontMatrix.001, CharProc d1/Widths605다. 굵은 표를 재인용하지 않는다.
    let mut program = core("HCI Poppy", "A<=>A", None);
    for ch in ["<", "=", ">"] {
        let decision = record(&program, ch);
        assert_eq!(
            decision["layoutMetric"]["widthSource"],
            "measuredOriginalGlyph"
        );
        assert!((advance(&decision) - 7500.0 * 0.605).abs() <= 2.0);
    }
    program.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"explicit-regular-hci-ttf","substitutions":{"HCI Poppy":"Palatino Linotype"},"trueTypeFaces":["Palatino Linotype"]}"#,
    ).unwrap())).unwrap();
    for ch in ["<", "=", ">"] {
        assert_ne!(
            record(&program, ch)["layoutMetric"]["widthSource"],
            "measuredOriginalGlyph"
        );
    }
}

#[test]
fn asset_caption_comparison_program_reaches_the_following_source_text() {
    fn collect<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
        if node["type"] == "textRun"
            && node["source"]["stableSourceKey"]
                .as_str()
                .is_some_and(|key| {
                    key.starts_with("section:2/para:22/char:") && !key.contains("/cell:")
                })
        {
            out.push(node);
        }
        match node {
            Value::Array(children) => {
                for child in children {
                    collect(child, out);
                }
            }
            Value::Object(children) => {
                for child in children.values() {
                    collect(child, out);
                }
            }
            _ => {}
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6031/3249937_asset_management_rules.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let source = &core.document().sections[2].paragraphs[22].text;
    let tree: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(55, rhwp::paint::RenderProfile::Print)
            .unwrap(),
    )
    .unwrap();
    let mut nodes = Vec::new();
    collect(&tree["root"], &mut nodes);
    let rebuilt: String = nodes
        .iter()
        .map(|node| node["text"].as_str().unwrap())
        .collect();
    assert_eq!(&rebuilt, source, "같은 쪽의 원문 순서·누락·중복 보존");
    let mut found = 0;
    for node in &nodes {
        let clusters = node["clusters"].as_array().expect("최종 replay 클러스터");
        let text = node["text"].as_str().unwrap();
        assert_eq!(clusters.len(), text.chars().count());
        for (i, ch) in text.chars().enumerate() {
            if matches!(ch, '<' | '>') {
                found += 1;
                let em = clusters[i]["advance"]["dx"].as_f64().unwrap()
                    / node["style"]["fontSize"].as_f64().unwrap();
                assert!(
                    (em - 0.605).abs() < 0.005,
                    "원 프로그램의 비교 기호 전진: {ch} {em}"
                );
            }
        }
    }
    assert_eq!(found, 2, "같은 원문의 두 비교 기호만 검사");
}

#[test]
fn worklife_repeated_plus_keeps_the_following_value_on_its_independent_origin() {
    // C94: 전6쪽 Native/fresh WASM90% 이상, devel 대비0.01%p 하락0.
    // 같은 물리 줄의 반복 '+' 누적 경계는4쪽(99.00620/98.99872%)에 있다.
    // 독립 Print의 마지막 전남(+1.0...)에서 '1'/'점' 앞 소수점 원점을 보호한다.
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
        .find(|(_, p)| p.text.contains("충북(+1.2") && p.text.contains("전남(+1.0"))
        .map(|(i, p)| (i, p.text.clone()))
        .unwrap();
    let start = text.find("전남(+1.0").unwrap();
    let digit = text[..start].chars().count() + "전남(+".chars().count();
    let tree = core.build_page_render_tree(3).unwrap();
    fn collect<'a>(
        n: &'a rhwp::renderer::render_tree::RenderNode,
        pi: usize,
        out: &mut Vec<&'a rhwp::renderer::render_tree::TextRunNode>,
    ) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(r) = &n.node_type {
            if r.section_index == Some(0) && r.para_index == Some(pi) {
                out.push(r)
            }
        }
        for c in &n.children {
            collect(c, pi, out)
        }
    }
    let mut runs = vec![];
    collect(&tree.root, pi, &mut runs);
    assert_eq!(
        runs.iter().map(|r| r.text.as_str()).collect::<String>(),
        text
    );
    let owner: Vec<_> = runs
        .iter()
        .filter(|r| {
            r.char_start
                .is_some_and(|i| i <= digit && digit < i + r.text.chars().count())
        })
        .collect();
    assert_eq!(owner.len(), 1);
    let source_start = owner[0].char_start.unwrap();
    let local = digit - source_start;
    assert_eq!(owner[0].text.chars().nth(local), Some('1'));
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                3,
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
        &format!("section:0/para:{pi}/char:{source_start}"),
        &mut ops,
    );
    assert_eq!(ops.len(), 1);
    let op = ops[0];
    let x = |offset: usize| {
        let cs: Vec<_> = op["clusters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["textRangeUtf16"]["start"] == offset)
            .collect();
        assert_eq!(cs.len(), 1);
        op["placement"]["runToPage"]["e"].as_f64().unwrap() + cs[0]["origin"]["x"].as_f64().unwrap()
    };
    assert!(local > 0);
    assert!(x(local - 1) < x(local) && x(local) < x(local + 1));
    // PDF 260.8585205/267.095398pt →96CSSpx. 192/96DPI를 혼용하지 않는다.
    assert!(
        (x(local) - 347.8113606770833).abs() <= 2.0,
        "반복 '+' 뒤 숫자: {}",
        x(local)
    );
    assert!((x(local + 1) - 356.127197265625).abs() <= 2.0);
    assert!((op["placement"]["runToPage"]["f"].as_f64().unwrap() - 997.9493815104166).abs() <= 2.0);
    assert_eq!(core.document().sections[0].paragraphs[pi].text, text);
}

#[test]
fn worklife_declared_hft_quote_keeps_the_following_year_in_canvas_print() {
    use rhwp::document_core::CanvasMetricReply;
    use rhwp::renderer::supplemental_metrics::{MetricBackend, MetricContext};
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
        .find(|(_, p)| p.text.contains("’22년 기준 지수 세부 영역별 결과"))
        .map(|(i, p)| (i, p.text.clone()))
        .unwrap();
    let year = text[..text.find("22년").unwrap()].chars().count();
    let context = MetricContext {
        document_generation: 1,
        font_generation: 1,
        backend: MetricBackend::Canvas2d,
    };
    let batch = core.collect_canvas_metric_requests(context).unwrap();
    // The actual Studio fallback measured the source HFT quote as a full em.
    // Feed that real observation through the public preparation/selection path.
    let replies = batch
        .requests
        .iter()
        .map(|r| CanvasMetricReply {
            key: r.key.clone(),
            cluster: r.cluster.clone(),
            font: r.font.clone(),
            resolved_font: r.font.clone(),
            measured_advance_px: 20.0,
        })
        .collect();
    core.register_canvas_metric_replies(context, batch.revision, replies)
        .unwrap();
    core.select_canvas_metrics(true).unwrap();
    let layer: Value = serde_json::from_str(
        &core
            .get_canvas_page_layer_tree_with_options_native(
                3,
                rhwp::paint::RenderProfile::Print,
                rhwp::paint::LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    fn collect<'a>(value: &'a Value, prefix: &str, out: &mut Vec<&'a Value>) {
        match value {
            Value::Object(o) => {
                if value["type"] == "textRun"
                    && value["source"]["stableSourceKey"]
                        .as_str()
                        .is_some_and(|key| key.starts_with(prefix))
                {
                    out.push(value);
                }
                for child in o.values() {
                    collect(child, prefix, out);
                }
            }
            Value::Array(a) => {
                for child in a {
                    collect(child, prefix, out);
                }
            }
            _ => {}
        }
    }
    let mut ops = vec![];
    collect(&layer, &format!("section:0/para:{pi}/char:"), &mut ops);
    assert_eq!(
        ops.iter()
            .map(|r| r["text"].as_str().unwrap())
            .collect::<String>(),
        text,
        "The heading keeps every source character exactly once and in order"
    );
    let owners: Vec<_> = ops
        .iter()
        .filter(|r| r["source"]["stableSourceKey"] == format!("section:0/para:{pi}/char:{year}"))
        .collect();
    assert_eq!(owners.len(), 1);
    assert!(owners[0]["text"].as_str().unwrap().starts_with("22"));
    let x = owners[0]["placement"]["runToPage"]["e"].as_f64().unwrap();
    // C80: Native/fresh WASM all6 pages >=90%; p4=99.04483/99.03763%.
    // Independent2020 Print p4 H2hdrM year origin82.3953476pt ->96CSSpx.
    // T16 quote ToUnicode2019, Widths225, FontMatrix.001; no font hmtx inference.
    assert!(
        (x - 109.86046346028646).abs() <= 2.0,
        "year after HFT quote: {x}"
    );
    assert_eq!(core.document().sections[0].paragraphs[pi].text, text);
    assert_eq!(core.page_count(), 6);
}

#[test]
fn declared_hft_quote_preserves_ttf_and_unmeasured_style_boundaries() {
    use rhwp::renderer::supplemental_metrics::{MetricBackend, MetricContext};
    let context = MetricContext {
        document_generation: 1,
        font_generation: 1,
        backend: MetricBackend::Canvas2d,
    };
    let mut program = core("한양중고딕", "’A", None);
    let ttf_source = program.document().clone();
    assert_ne!(
        record(&program, "’")["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    assert!(program
        .collect_canvas_metric_requests(context)
        .unwrap()
        .requests
        .iter()
        .any(|r| r.cluster == "’"));
    let mut hft = ttf_source.clone();
    for face in hft.doc_info.font_faces.iter_mut().flatten() {
        face.alt_type = 2;
    }
    program.set_document(hft.clone());
    let quote = record(&program, "’");
    assert_eq!(
        quote["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    assert!((advance(&quote) - 7500.0 * 225.0 / 1000.0).abs() <= 2.0);
    assert!(!program
        .collect_canvas_metric_requests(context)
        .unwrap()
        .requests
        .iter()
        .any(|r| r.cluster == "’"));
    program.set_font_environment(Some(FontEnvironment::from_json(r#"{"id":"explicit-true-type-quote","substitutions":{},"trueTypeFaces":["한양중고딕"]}"#).unwrap())).unwrap();
    assert_ne!(
        record(&program, "’")["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    program.set_font_environment(None).unwrap();
    hft.doc_info
        .char_shapes
        .iter_mut()
        .for_each(|cs| cs.bold = true);
    program.set_document(hft);
    assert_ne!(
        record(&program, "’")["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
}

fn sample16_layer_ops<'a>(value: &'a Value, kind: &str, prefix: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(object) => {
            if value["type"] == kind
                && value["source"]["stableSourceKey"]
                    .as_str()
                    .is_some_and(|key| key.starts_with(prefix))
            {
                out.push(value);
            }
            for child in object.values() {
                sample16_layer_ops(child, kind, prefix, out);
            }
        }
        Value::Array(array) => {
            for child in array {
                sample16_layer_ops(child, kind, prefix, out);
            }
        }
        _ => {}
    }
}
fn sample16_assert_tab_owner(label: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/hwp3-sample16-hwp5.hwp")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    let (pi, text) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.contains(label) && p.text.contains('\t'))
        .map(|(i, p)| (i, p.text.clone()))
        .unwrap();
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                1,
                rhwp::paint::RenderProfile::Print,
                rhwp::paint::LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let prefix = format!("section:0/para:{pi}/char:");
    let mut texts = vec![];
    let mut leaders = vec![];
    sample16_layer_ops(&layer, "textRun", &prefix, &mut texts);
    sample16_layer_ops(&layer, "tabLeader", &prefix, &mut leaders);
    assert_eq!(
        texts
            .iter()
            .map(|op| op["text"].as_str().unwrap())
            .collect::<String>(),
        text,
        "원문 순서·누락·중복"
    );
    assert_eq!(
        leaders.len(),
        text.chars().filter(|&c| c == '\t').count(),
        "탭 하나의 리더가 도형 앞뒤에 중복되지 않음"
    );
    for leader in leaders {
        let owner: Vec<_> = texts
            .iter()
            .filter(|op| op["source"]["stableSourceKey"] == leader["source"]["stableSourceKey"])
            .collect();
        assert_eq!(owner.len(), 1);
        assert!(
            owner[0]["text"].as_str().unwrap().contains('\t'),
            "탭이 없는 앞 조각은 리더를 그리지 않음"
        );
        let source_start: usize = owner[0]["source"]["stableSourceKey"]
            .as_str()
            .unwrap()
            .strip_prefix(&prefix)
            .unwrap()
            .parse()
            .unwrap();
        let tab_local = owner[0]["text"]
            .as_str()
            .unwrap()
            .chars()
            .position(|c| c == '\t')
            .unwrap();
        assert_eq!(text.chars().nth(source_start + tab_local), Some('\t'));
        let number = texts.last().unwrap();
        for span in leader["leaders"].as_array().unwrap() {
            assert!(
                leader["bbox"]["x"].as_f64().unwrap() + span["endX"].as_f64().unwrap()
                    <= number["bbox"]["x"].as_f64().unwrap(),
                "리더는 뒤 쪽번호를 침범하지 않음"
            );
        }
    }
    assert_eq!(core.document().sections[0].paragraphs[pi].text, text);
}
#[test]
fn sample16_inline_object_fragments_keep_each_leader_with_its_source_tab() {
    sample16_assert_tab_owner("추진목적");
}
#[test]
fn sample16_ordinary_tab_keeps_its_single_source_owner() {
    sample16_assert_tab_owner("추진방향");
}

#[test]
fn sample16_hft_bold_heading_preserves_content_and_independent_completed_page_origin() {
    // 독립 Print p19 /T4의 기준선과 뒤 글자. 최종 양쪽 raw98% 자격은 stage5에 기록한다.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/hwp3-sample16-hwp5.hwp")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    let (pi, text) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.starts_with("3. 원격지 재해복구센터") && p.text.contains("Disaster"))
        .map(|(i, p)| (i, p.text.clone()))
        .unwrap();
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                18,
                rhwp::paint::RenderProfile::Print,
                rhwp::paint::LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let prefix = format!("section:0/para:{pi}/char:");
    let mut ops = vec![];
    sample16_layer_ops(&layer, "textRun", &prefix, &mut ops);
    assert_eq!(
        ops.iter()
            .map(|op| op["text"].as_str().unwrap())
            .collect::<String>(),
        text
    );
    let tail = text.chars().position(|c| c == '터').unwrap();
    let mut origins = vec![];
    for op in ops {
        let start: usize = op["source"]["stableSourceKey"]
            .as_str()
            .unwrap()
            .strip_prefix(&prefix)
            .unwrap()
            .parse()
            .unwrap();
        for cluster in op["clusters"].as_array().unwrap() {
            let local = cluster["textRangeUtf16"]["start"].as_u64().unwrap() as usize;
            if start + local == tail {
                assert_eq!(op["text"].as_str().unwrap().chars().nth(local), Some('터'));
                origins.push((
                    op["placement"]["runToPage"]["e"].as_f64().unwrap()
                        + cluster["origin"]["x"].as_f64().unwrap(),
                    op["placement"]["runToPage"]["f"].as_f64().unwrap()
                        + cluster["origin"]["y"].as_f64().unwrap(),
                ));
            }
        }
    }
    assert_eq!(origins.len(), 1);
    assert!(
        (origins[0].0 - 299.6766764322917).abs() <= 2.0,
        "원본 HFT 굵은 전진 뒤 '터': {:?}",
        origins[0]
    );
    assert!((origins[0].1 - 636.1599934895834).abs() <= 2.0);
    assert_eq!(core.document().sections[0].paragraphs[pi].text, text);
}
#[test]
fn hft_human_bold_tracking_preserves_regular_ttf_spaces_and_native_program_ownership() {
    let mut program = core("휴먼명조", "가 •", None);
    let mut source = program.document().clone();
    for font in source.doc_info.font_faces.iter_mut().flatten() {
        font.alt_type = 2;
    }
    program.set_document(source.clone());
    let plain = record(&program, "가");
    let space = record(&program, " ");
    let bullet = record(&program, "•");
    for shape in &mut source.doc_info.char_shapes {
        shape.bold = true;
    }
    program.set_document(source.clone());
    assert!(
        (advance(&record(&program, "가")) / advance(&plain) - 1.05).abs() <= 2.0 / advance(&plain)
    );
    assert_eq!(advance(&record(&program, " ")), advance(&space));
    assert_eq!(advance(&record(&program, "•")), advance(&bullet));
    program.set_font_environment(Some(FontEnvironment::from_json(r#"{"id":"selected-human-ttf-control","substitutions":{},"trueTypeFaces":["휴먼명조"]}"#).unwrap())).unwrap();
    assert_eq!(advance(&record(&program, "가")), advance(&plain));
    program.set_font_environment(None).unwrap();
    for font in source.doc_info.font_faces.iter_mut().flatten() {
        font.native_program = Some(rhwp::model::style::NativeFontProgram::HumanMyeongJo);
    }
    program.set_document(source);
    let final_advance = record(&program, "가")["layoutMetric"]["finalAdvanceHwpunit"]
        .as_f64()
        .unwrap();
    assert!(
        (final_advance - 7500.0 * 1.05).abs() <= 4.0,
        "원 프로그램의 bold 추적을 중복하지 않음:{final_advance}"
    );
}

#[test]
fn original_hci_bold_comparison_glyphs_preserve_style_and_ttf_boundaries() {
    let mut program = core("HCI Poppy", "A<=>A", None);
    let mut source = program.document().clone();
    for font in source.doc_info.font_faces.iter_mut().flatten() {
        font.alt_type = 2;
    }
    for shape in &mut source.doc_info.char_shapes {
        shape.bold = true;
    }
    program.set_document(source.clone());
    for c in ["<", ">", "="] {
        let decision = record(&program, c);
        assert_eq!(
            decision["layoutMetric"]["widthSource"],
            "measuredOriginalGlyph"
        );
        // 수동2024 Print T5: Widths605/FontMatrix.001, 각각 CharProc d1도605.
        assert!((advance(&decision) - 7500.0 * 0.605).abs() <= 2.0);
    }
    program.set_font_environment(Some(FontEnvironment::from_json(r#"{"id":"explicit-hci-bold-ttf","substitutions":{"HCI Poppy":"Palatino Linotype"},"trueTypeFaces":["Palatino Linotype"]}"#).unwrap())).unwrap();
    for c in ["<", ">", "="] {
        assert_ne!(
            record(&program, c)["layoutMetric"]["widthSource"],
            "measuredOriginalGlyph"
        );
    }
    program.set_font_environment(None).unwrap();
    for shape in &mut source.doc_info.char_shapes {
        shape.italic = true;
    }
    program.set_document(source);
    for c in ["<", ">", "="] {
        assert_ne!(
            record(&program, c)["layoutMetric"]["widthSource"],
            "measuredOriginalGlyph"
        );
    }
}

#[test]
fn declared_hft_gothic_keeps_its_monospace_ascii_punctuation() {
    // 별도 빈 HWPX의 전체 ASCII 표본을 한컴2020 Print: 95자 모두500/1000em.
    // sample11의 좌표나 잠정 배치를 기대값으로 사용하지 않는다.
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwp3-sample11.hwp");
    let original = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    let source_faces: Vec<_> = original
        .document()
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .filter(|face| face.name == "고딕")
        .collect();
    assert!(!source_faces.is_empty());
    assert!(
        source_faces.iter().all(|face| face.alt_type == 2),
        "공통 IR의 HFT 선언까지 전달되어야 실제 조판이 같은 폭을 소비한다"
    );
    let mut program = core("고딕", " .,:()!?A1", None);
    let mut source = program.document().clone();
    for font in source.doc_info.font_faces.iter_mut().flatten() {
        font.alt_type = 2;
    }
    for shape in &mut source.doc_info.char_shapes {
        shape.use_font_space = true;
    }
    program.set_document(source.clone());
    let space = record(&program, " ");
    assert!(
        (advance(&space) - 7500.0 * 0.5).abs() <= 2.0,
        "원본 고정폭 공백도 표시용 대체 글꼴의 폭을 소비하지 않음: {}",
        advance(&space)
    );
    assert_eq!(space["layoutMetric"]["widthSource"], "useFontSpace");
    for c in ['.', ',', ':', '(', ')', '!', '?', 'A', '1'] {
        let decision = record(&program, &c.to_string());
        assert!(
            (advance(&decision) - 7500.0 * 0.5).abs() <= 2.0,
            "원본 고정폭 ASCII {c}: {}",
            advance(&decision)
        );
        assert_eq!(
            decision["layoutMetric"]["widthSource"],
            "measuredOriginalGlyph"
        );
    }
    program
        .set_font_environment(Some(
            FontEnvironment::from_json(
                r#"{"id":"explicit-gothic-ttf","substitutions":{"고딕":"HCR Dotum"},"trueTypeFaces":["HCR Dotum"]}"#,
            )
            .unwrap(),
        ))
        .unwrap();
    assert_ne!(
        record(&program, ".")["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
    let mut ttf = core("HCR Dotum", " A", None);
    let mut ttf_source = ttf.document().clone();
    for shape in &mut ttf_source.doc_info.char_shapes {
        shape.use_font_space = true;
    }
    ttf.set_document(ttf_source);
    assert_eq!(
        advance(&record(&program, " ")),
        advance(&record(&ttf, " ")),
        "명시 TrueType 대조군은 선택한 TTF의 공백을 유지한다"
    );
    program.set_font_environment(None).unwrap();
    for shape in &mut source.doc_info.char_shapes {
        shape.italic = true;
    }
    program.set_document(source);
    assert_ne!(
        record(&program, ".")["layoutMetric"]["widthSource"],
        "measuredOriginalGlyph"
    );
}

fn pii_declared_program_core() -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx"))
            .expect("독립 Print와 같은 개인정보 원본"),
    )
    .unwrap();
    core.set_hangul2024_compat(false);
    core.set_font_environment(Some(FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
    )).unwrap())).unwrap();
    assert_eq!(core.page_count(), 66, "전쪽 독립 Print와 동일한 쪽 소속");
    core
}

#[test]
fn pii_declared_ttf_angles_keep_the_document_program_advance() {
    // 독립 PII18/F7·43/F4는 Palatino Type0/CIDFontType2, < >의 /W=500이다.
    // 별도2020 Print 8조건도 type1+명시 TTF 대체만500, implicit/type2는605다.
    // 큰 글자 합성본의 미검증 배치를 고정하지 않고 실제 완료 원문을 검사한다.
    let core = pii_declared_program_core();
    for (page, expected_count, bold) in [(18, 2, true), (43, 4, false)] {
        let trace: Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(page - 1, r#"{"maxCharacters":4096}"#)
                .unwrap(),
        )
        .unwrap();
        let angles: Vec<_> = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                matches!(r["source"]["character"].as_str(), Some("<" | ">"))
                    && r["document"]["face"] == "HCI Poppy"
            })
            .collect();
        assert_eq!(
            angles.len(),
            expected_count,
            "원문 제목의 꺾쇠 누락·중복 금지"
        );
        for decision in angles {
            let lang = decision["document"]["languageSlot"].as_u64().unwrap() as usize;
            let shape = &core.document().doc_info.char_shapes
                [decision["source"]["charShapeId"].as_u64().unwrap() as usize];
            let font = &core.document().doc_info.font_faces[lang][shape.font_ids[lang] as usize];
            assert_eq!(font.name, "HCI Poppy");
            assert_eq!(font.alt_type, 1);
            let substitute = font.subst_font.as_ref().expect("원본의 명시 대체 프로그램");
            assert_eq!(substitute.font_type, 1);
            assert_eq!(substitute.face, "Palatino Linotype");
            assert_eq!(shape.bold, bold, "원본의 일반/굵은 프로그램 보존");
            assert_eq!(
                decision["layoutMetric"]["aliasResolvedFace"],
                "Palatino Linotype"
            );
            let size = f64::from(shape.base_size) * f64::from(shape.relative_sizes[lang]) / 100.0;
            assert!(
                (advance(decision) - size * 0.5).abs() <= 2.0,
                "독립 CID /W500과 같은 TTF 자연 전진: {decision}"
            );
        }
    }
}

#[test]
fn pii_declared_ttf_closing_angles_keep_source_order_and_independent_ends() {
    use rhwp::{
        paint::{
            builder::LayerBuilder,
            layer_tree::{LayerNode, LayerNodeKind},
            paint_op::PaintOp,
            RenderProfile,
        },
        renderer::render_tree::{BoundingBox, TextRunNode},
    };
    fn collect<'a>(node: &'a LayerNode, out: &mut Vec<(&'a BoundingBox, &'a TextRunNode)>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    collect(child, out);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                collect(child, out)
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::TextRun { bbox, run, .. } = op {
                        out.push((bbox, run));
                    }
                }
            }
        }
    }
    fn owner(run: &TextRunNode, page: u32) -> bool {
        if run.section_index != Some(0) {
            return false;
        }
        if page == 18 {
            return run.para_index == Some(216) && run.cell_context.is_none();
        }
        run.cell_context.as_ref().is_some_and(|cell| {
            cell.parent_para_index == 403
                && cell.path.len() == 1
                && cell.path[0].control_index == 0
                && cell.path[0].cell_index == 2
                && matches!(cell.path[0].cell_para_index, 0 | 13)
        })
    }
    let core = pii_declared_program_core();
    // 전66쪽 Native/fresh WASM90% 및18/43쪽 양쪽100%·review/overlay 직접 판독 후 추가.
    // PDF CTM .12pt *96/72: p18 x1222,size125,W500 →205.52CSSpx;
    // p43 x1021,size117,W500 →172.72CSSpx. 평균 점수나 후보 좌표를 근거로 삼지 않는다.
    for (page, count, label, expected_end) in
        [(18, 1, " 규제 개요 ", 205.52), (43, 2, "신  설", 172.72)]
    {
        let tree = core.build_page_render_tree(page - 1).unwrap();
        let layers = LayerBuilder::new(RenderProfile::Print).build(&tree);
        let mut all = Vec::new();
        collect(&layers.root, &mut all);
        let headings: Vec<_> = all
            .iter()
            .copied()
            .filter(|(_, r)| owner(r, page))
            .collect();
        for text in ["<", label, ">"] {
            assert_eq!(
                headings.iter().filter(|(_, r)| r.text == text).count(),
                count,
                "원본 제목의 글자·문단·셀 소속과 누락·중복 보존"
            );
        }
        let starts: Vec<_> = headings.iter().filter(|(_, r)| r.text == "<").collect();
        let labels: Vec<_> = headings.iter().filter(|(_, r)| r.text == label).collect();
        let ends: Vec<_> = headings.iter().filter(|(_, r)| r.text == ">").collect();
        for ((start, middle), end) in starts.into_iter().zip(labels).zip(ends) {
            assert!(
                start.0.x + start.0.width <= middle.0.x + 0.1,
                "앞 꺾쇠와 뒤 본문의 순서·비겹침"
            );
            assert!(
                middle.0.x + middle.0.width <= end.0.x + 0.1,
                "본문과 닫는 꺾쇠의 순서·비겹침"
            );
            assert!(
                (end.0.x + end.0.width - expected_end).abs() <= 2.0,
                "독립 Print의 제목 끝 ±2CSSpx: page={page}, actual={}",
                end.0.x + end.0.width
            );
        }
    }
}
