//! 관계형: 공개 웹폰트 변환과 Native 로컬 우선 공급 계약을 검사한다.
//! 주석 갱신: 2026-11-07 07:46:43 KST; 원본 HFT 원형 기호와 표시 프로그램 hmtx의 소유를 추가 검증.
//! 직전 파일 변경: 5551091085418c17d8321c0fee2626365911481b (2026-11-04 12:13:59 +09:00).
//! 합성 글꼴 입력 검사이며 실물 페이지의 일치율이나 고정 px를 주장하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::native_webfonts::{
    decode_font, prepare, NativeWebfontStyle, NativeWebfontSupply,
};
use rhwp::DocumentCore;

fn document_with_face(name: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for faces in &mut document.doc_info.font_faces {
        for face in faces {
            face.name = name.into();
            face.is_embedded = false;
            face.subst_font = None;
        }
    }
    core.set_document(document);
    core
}

#[test]
fn woff2_decodes_to_a_readable_program_without_losing_korean_glyph_advance() {
    let bytes = include_bytes!("../../assets/fonts/NotoSansKR-Regular.woff2");
    let native = decode_font(bytes).expect("공개 WOFF2를 Native SFNT로 변환");
    let face = ttf_parser::Face::parse(&native, 0).unwrap();
    let glyph = face.glyph_index('가').expect("한글 glyph 보존");
    assert!(face.glyph_hor_advance(glyph).is_some_and(|width| width > 0));
    assert!(decode_font(b"wOF2").is_err(), "잘린 입력은 거부");
    let mut oversized = vec![0u8; 48];
    oversized[..4].copy_from_slice(b"wOF2");
    oversized[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
    assert!(
        decode_font(&oversized).is_err(),
        "과도한 압축 해제 크기는 변환 전에 거부"
    );
}

#[test]
fn local_face_wins_and_generic_fallback_does_not_claim_to_supply_the_unknown_original() {
    let local = document_with_face("Noto Sans KR ExtraLight");
    let supplied_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ttfs/opensource/NotoSansKR-ExtraLight.ttf");
    let supply = prepare(&local.document().doc_info, &[supplied_path]).unwrap();
    assert!(supply.mappings.is_empty());
    assert!(
        supply.paths.is_empty(),
        "로컬 원본이 있으면 다운로드하지 않음"
    );
    let unknown = document_with_face("RHWP Missing Font 8533");
    let generic = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ttfs/opensource/NotoSansKR-Regular.ttf");
    let supply = prepare(&unknown.document().doc_info, &[generic]).unwrap();
    assert_eq!(
        supply
            .mappings
            .get("RHWP Missing Font 8533")
            .map(String::as_str),
        Some("NotoSansKR-Regular")
    );
    assert_eq!(supply.generic_fallbacks, ["RHWP Missing Font 8533"]);
    assert!(
        supply.paths.is_empty(),
        "확인된 로컬 Noto를 다시 다운로드하지 않음"
    );
    assert!(supply
        .unavailable
        .iter()
        .any(|name| name == "RHWP Missing Font 8533"));

    let mut declared = unknown.document().doc_info.clone();
    for face in declared.font_faces.iter_mut().flatten() {
        face.subst_font = Some(rhwp::model::style::SubstFont {
            face: "NotoSansKR-Regular".into(),
            font_type: 1,
            ..Default::default()
        });
    }
    let generic = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("ttfs/opensource/NotoSansKR-Regular.ttf");
    let supplied = prepare(&declared, &[generic]).unwrap();
    assert!(
        supplied.mappings.is_empty() && supplied.generic_fallbacks.is_empty(),
        "설치된 문서 선언 대체는 일반 웹 대체로 덮어쓰지 않음"
    );
}

#[test]
fn svg_font_reference_preserves_local_priority_and_escapes_markup() {
    let mut supply = NativeWebfontSupply::default();
    supply.web_fonts.insert(
        "RHWP <font> & \"name\"".into(),
        "https://cdn.jsdelivr.net/gh/example/fonts@fixed/test.woff2".into(),
    );
    supply.web_font_styles.insert(
        "RHWP <font> & \"name\"".into(),
        NativeWebfontStyle {
            weight: 700,
            italic: false,
            weight_range: None,
        },
    );
    let style = supply.svg_style();
    assert!(style.contains("src:local("));
    assert!(style.find("local(").unwrap() < style.find("url(").unwrap());
    assert!(
        !style.contains("<font>"),
        "글꼴 이름이 SVG 요소로 해석되지 않음"
    );
    assert!(style.contains("test.woff2"));
    assert!(
        style.contains("font-weight:700"),
        "실제 Bold를 Regular로 등록하지 않음"
    );
    assert!(NativeWebfontSupply::default().svg_style().is_empty());
}

#[test]
fn display_substitute_preserves_source_advance_while_selected_program_can_own_it() {
    use rhwp::renderer::font_environment::FontEnvironment;
    let font = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
    let advance =
        |original: &str, preserve: bool, select_program: bool| {
            let mut core = document_with_face(original);
            core.insert_text_native(0, 0, 0, "가나다").unwrap();
            let mut doc = core.document().clone();
            doc.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
            for face in doc.doc_info.font_faces.iter_mut().flatten() {
                face.alt_type = 1;
            }
            core.set_document(doc);
            core.set_font_environment(Some(FontEnvironment::from_json(&serde_json::json!({
            "id":"display-program-owner-8533", "preserveSourceMetrics":preserve,
                "substitutions":{(original):"NotoSansKR-Regular"},
            "trueTypeFaces":if select_program {vec!["NotoSansKR-Regular"]} else {vec![]},
        }).to_string()).unwrap())).unwrap();
            let trace = |core: &DocumentCore| {
                let value: serde_json::Value = serde_json::from_str(
                    &core
                        .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                        .unwrap(),
                )
                .unwrap();
                value["records"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["source"]["character"] == "가")
                    .unwrap()["layoutMetric"]["finalAdvanceHwpunit"]
                    .as_f64()
                    .unwrap()
            };
            let before = trace(&core);
            core.prepare_native_otf_fonts_with(|_| Some((font.to_vec(), 0)))
                .unwrap();
            (before, trace(&core))
        };
    // 독립 #8438 Print의 원본 ExpoM-HM은 전각이다. 표시 Noto의 0.92em을
    // 원본 전진으로 등록하면 가운데 문구가 수축한다. 합성 계약도 같은 경계를 지킨다.
    let (before, after) = advance("RHWP missing original 8533", true, false);
    assert!(
        (before - after).abs() < 1e-6,
        "표시용 공급이 원본 전진을 바꿈: {before} -> {after}"
    );
    for (source, preserve, selected) in [
        ("RHWP missing original 8533", false, false),
        ("RHWP missing original 8533", true, true),
        ("NotoSansKR-Regular", true, false),
    ] {
        let (before, after) = advance(source, preserve, selected);
        assert!(after<before, "선택한 실제 프로그램의 자연 전진은 유지: {source} / {preserve} / {selected}: {before} -> {after}");
    }

    // 같은 표시 family를 공유해도 원본 슬롯과 표시 대체 슬롯의 폭 소유는 다르다.
    // 다른 슬롯이 등록한 프로그램을 family 캐시에서 빌려 원본 전진을 바꾸지 않는다.
    let mut core = document_with_face("RHWP missing original 8533");
    core.insert_text_native(0, 0, 0, "가나").unwrap();
    let mut doc = core.document().clone();
    doc.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    let mut real_ids = [0; 7];
    for (lang, faces) in doc.doc_info.font_faces.iter_mut().enumerate() {
        for face in faces.iter_mut() {
            face.alt_type = 1;
        }
        real_ids[lang] = faces.len() as u16;
        let mut real = faces[0].clone();
        real.name = "NotoSansKR-Regular".into();
        faces.push(real);
    }
    for shape in &mut doc.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    let real_id = doc.doc_info.char_shapes.len() as u32;
    let mut real = doc.doc_info.char_shapes[0].clone();
    real.font_ids = real_ids;
    doc.doc_info.char_shapes.push(real);
    doc.sections[0].paragraphs[0].char_shapes = vec![rhwp::model::paragraph::CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    let paragraph = &mut doc.sections[0].paragraphs[0];
    let real_char = paragraph.text.chars().position(|c| c == '나').unwrap();
    // 문단의 숨은 제어가 차지한 UTF-16 위치를 공용 편집 경로로 환산한다.
    paragraph.apply_char_shape_range(real_char, real_char + 1, real_id);
    core.set_document(doc);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"shared-display-owner-8533","preserveSourceMetrics":true,"substitutions":{"RHWP missing original 8533":"NotoSansKR-Regular","NotoSansKR-Regular":"NotoSansKR-Regular"}}"#,
    ).unwrap())).unwrap();
    let traces = |core: &DocumentCore| {
        let value: serde_json::Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                .unwrap(),
        )
        .unwrap();
        ["가", "나"].map(|c| {
            let record = value["records"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["source"]["character"] == c)
                .unwrap();
            assert_eq!(
                record["source"]["charShapeId"],
                if c == "가" { 0 } else { real_id },
                "검사할 문자의 원본 슬롯 소속: {record}"
            );
            record["layoutMetric"]["finalAdvanceHwpunit"]
                .as_f64()
                .unwrap()
        })
    };
    let before = traces(&core);
    core.prepare_native_otf_fonts_with(|_| Some((font.to_vec(), 0)))
        .unwrap();
    let after = traces(&core);
    assert!(
        (before[0] - after[0]).abs() < 1e-6,
        "다른 원본 슬롯의 프로그램이 표시 대체 슬롯 전진에 침투: {before:?} -> {after:?}"
    );
    assert!(
        after[1] < before[1],
        "실제 원본 슬롯의 자연 전진은 유지: {before:?} -> {after:?}"
    );
}

#[test]
fn display_replacement_preserves_existing_tables_and_latin_slot_metrics() {
    use rhwp::renderer::font_environment::FontEnvironment;
    let trace = |core: &DocumentCore| {
        let value: serde_json::Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":32}"#)
                .unwrap(),
        )
        .unwrap();
        value["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| "가1.·A\u{2007}".contains(r["source"]["character"].as_str().unwrap()))
            .map(|r| {
                (
                    r["source"]["character"].as_str().unwrap().to_owned(),
                    r["layoutMetric"]["finalAdvanceHwpunit"].as_f64().unwrap(),
                )
            })
            .collect::<Vec<_>>()
    };
    for (source, kind) in [
        ("Haansoft Batang", 1),
        ("Gulim", 1),
        ("한컴 윤고딕 720", 2),
        ("RHWP unknown source 8533", 1),
    ] {
        let mut core = document_with_face(source);
        core.insert_text_native(0, 0, 0, "가1.·A\u{2007}").unwrap();
        let mut doc = core.document().clone();
        doc.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
        for face in doc.doc_info.font_faces.iter_mut().flatten() {
            face.alt_type = kind;
        }
        core.set_document(doc);
        let before = trace(&core);
        assert!(!before.is_empty());
        core.set_font_environment(Some(
            FontEnvironment::from_json(
                &serde_json::json!({
                    "id":"preserved-table-8533", "preserveSourceMetrics":true,
                    "substitutions":{(source):"NotoSansKR-Regular"},
                })
                .to_string(),
            )
            .unwrap(),
        ))
        .unwrap();
        assert_eq!(
            before,
            trace(&core),
            "표시 family가 원본 폭 표·영문 슬롯 전진을 바꿈: {source}"
        );
    }
}

#[test]
fn declared_hft_substitute_keeps_its_own_punctuation_contract() {
    use rhwp::model::style::SubstFont;
    use rhwp::renderer::font_environment::FontEnvironment;
    let make = |name: &str, substitute: bool| {
        let mut core = document_with_face(name);
        let mut doc = core.document().clone();
        doc.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
        for face in doc.doc_info.font_faces.iter_mut().flatten() {
            face.alt_type = 2;
            face.subst_font = substitute.then(|| SubstFont {
                face: "명조".into(),
                font_type: 2,
                is_embedded: false,
                bin_item_id_ref: String::new(),
                resolved_bin_data_id: None,
            });
        }
        core.set_document(doc);
        core.insert_text_native(0, 0, 0, "가A1․·").unwrap();
        let environment = FontEnvironment::from_available_programs(&core.document().doc_info,
            r#"[{"query":"명조","family":"Myeongjo","programId":"synthetic-myeongjo-8533","openTypeCff":true}]"#
        ).unwrap();
        let proof = serde_json::to_value(environment.as_ref().unwrap()).unwrap();
        assert_eq!(
            proof["selectedPrograms"][name]["query"], "명조",
            "실제 조회 이름이 생산자에서 전달됨"
        );
        core.set_font_environment(environment).unwrap();
        let trace: serde_json::Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
                .unwrap(),
        )
        .unwrap();
        trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["source"]["character"].clone(),
                    r["layoutMetric"]["finalAdvanceHwpunit"].clone(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        make("#신명조", true),
        make("명조", false),
        "문서가 선언한 실제 HFT 대체는 임의 표시 치환과 구별해 같은 글리프 전진을 사용"
    );
}

#[test]
fn verified_hft_round_markers_keep_source_advance_after_display_program_supply() {
    // 기존 독립 Print: 평가결과서 U+2219와 로드맵 44쪽 U+2218은 1em.
    // 공개 Noto Sans Symbols 2의 실제 hmtx는 각각 0.270/0.276em이다.
    // 설치 글꼴 없이 표시 프로그램을 직접 공급해 후단 덮어쓰기를 재현한다.
    let bytes = include_bytes!("../../ttfs/opensource/NotoSansSymbols2-Regular.ttf");
    let public = ttf_parser::Face::parse(bytes, 0).unwrap();
    let trace = |core: &DocumentCore, character: char| {
        let value: serde_json::Value = serde_json::from_str(
            &core
                .get_font_decision_trace_native(0, r#"{"maxCharacters":32}"#)
                .unwrap(),
        )
        .unwrap();
        value["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["source"]["character"] == character.to_string())
            .unwrap()["layoutMetric"]
            .clone()
    };
    for character in ['\u{2218}', '\u{2219}'] {
        let public_advance = f64::from(
            public
                .glyph_hor_advance(public.glyph_index(character).unwrap())
                .unwrap(),
        ) / f64::from(public.units_per_em());
        assert!(public_advance < 0.5, "공개 프로그램은 전각과 다른 반례다");
        let mut core = document_with_face("한양신명조");
        core.insert_text_native(0, 0, 0, &format!("{character}가"))
            .unwrap();
        let mut document = core.document().clone();
        document.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
        for face in document.doc_info.font_faces.iter_mut().flatten() {
            face.alt_type = 2;
        }
        core.set_document(document);
        let before = trace(&core, character);
        core.prepare_native_otf_fonts_with(|_| Some((bytes.to_vec(), 0)))
            .unwrap();
        let after = trace(&core, character);
        assert_eq!(after["widthSource"], "measuredOriginalGlyph");
        assert_eq!(
            before["finalAdvanceHwpunit"], after["finalAdvanceHwpunit"],
            "표시 프로그램의 반각 hmtx가 독립 원본 전각을 덮어쓰면 안 된다: {character}"
        );

        // 실제 TrueType 선언은 HFT의 부분 표로 승격하지 않는다.
        let mut explicit = document_with_face("한양신명조");
        explicit
            .insert_text_native(0, 0, 0, &character.to_string())
            .unwrap();
        let mut document = explicit.document().clone();
        document.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
        for face in document.doc_info.font_faces.iter_mut().flatten() {
            face.alt_type = 1;
        }
        explicit.set_document(document);
        assert_ne!(
            trace(&explicit, character)["widthSource"],
            "measuredOriginalGlyph",
            "같은 이름의 명시 TrueType은 원본 HFT 프로그램과 구분한다"
        );
    }
}
