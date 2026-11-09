//! 관계형: 실제 프로그램의 가족 이름 조회와 원본 슬롯 요청 자격을 검사한다.
//! 주석 갱신: 2026-11-08T16:52:19+09:00.
//! 추가 전 파일 변경: 40c90ca187201527f5b7776f86f0fe6e493cef52 (2026-11-08T15:13:19+09:00).
//! 이동 직전 두 원본의 변경: 2026-11-06T20:11:29+09:00 6844f6ec0854b1c8150244d65cc1a099031287ac.
//! 기존 내부 검사 2건을 공개 FontEnvironment·DocumentCore 경로로 이동한다.
//! 합성 글꼴 선언·기존 OFL 글꼴 fixture의 프로그램 신원을 비교하며 설치 글꼴에 의존하지 않는다.
//! 조판 좌표·실문서 출력 검사가 아니므로 Native/fresh WASM raw·PDF·TSV는 비해당이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::document::DocInfo;
use rhwp::model::style::{CharShape, Font};
use rhwp::renderer::font_environment::FontEnvironment;
use serde_json::{json, Value};
use std::path::Path;

#[test]
fn declared_family_aliases_select_one_program_without_promoting_full_names() {
    let names = [
        "검사용 표기 가족",
        "Rhwp Family Contract",
        "검사용 가족 이름",
        "검사용 WWS 가족",
        "검사용 전체 얼굴 이름",
        "RhwpFamilyContract-Regular",
    ];
    let info = DocInfo {
        font_faces: vec![names
            .iter()
            .map(|name| Font {
                name: (*name).into(),
                alt_type: 2,
                ..Default::default()
            })
            .collect()],
        ..Default::default()
    };
    let font = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native_font_family_names/family-contract.ttf");
    let environment = FontEnvironment::from_native_document_substitutions(&info, &[font])
        .unwrap()
        .expect("명시적으로 공급한 fixture 프로그램");
    let value = serde_json::to_value(environment).unwrap();
    let programs = &value["selectedPrograms"];
    let original = programs[names[0]]["programId"]
        .as_str()
        .expect("원래 가족 이름의 프로그램 신원");
    for name in &names[..4] {
        assert_eq!(
            programs[*name]["programId"], original,
            "실제 가족 이름 {name}"
        );
        assert_eq!(programs[*name]["trueType"], true);
    }
    assert_ne!(
        programs[names[4]]["programId"], original,
        "Full Name을 가족 이름으로 승격하지 않는다"
    );
    assert_eq!(
        programs[names[5]]["family"], "Rhwp Family Contract",
        "명시적 PostScript 조회도 원래 가족을 보존하며 PostScript를 가족으로 승격하지 않는다"
    );
    assert_eq!(programs[names[5]]["programId"], original);
}

fn original_program_requested(source_font_id: u16) -> bool {
    let names = ["RHWP Original Program A", "RHWP Original Program B"];
    let mut core = DocumentCore::new_empty();
    {
        let info = &mut core.document_mut().doc_info;
        info.font_faces = (0..7)
            .map(|_| {
                names
                    .iter()
                    .map(|name| Font {
                        name: (*name).into(),
                        ..Default::default()
                    })
                    .collect()
            })
            .collect();
        info.char_shapes = vec![CharShape {
            font_ids: [source_font_id; 7],
            ..Default::default()
        }];
    }
    let environment: Value = json!({
        "id": "original-slot-contract",
        "preserveSourceMetrics": true,
        "substitutions": {(names[0]): names[0], (names[1]): names[0]}
    });
    core.set_font_environment(Some(
        FontEnvironment::from_json(&environment.to_string()).unwrap(),
    ))
    .unwrap();
    core.native_otf_font_requests()
        .iter()
        .any(|request| request.source_ttf)
}

#[test]
fn another_sources_display_family_does_not_authorize_original_truetype() {
    assert!(original_program_requested(0));
    assert!(
        !original_program_requested(1),
        "표시 family가 같아도 원본이 다른 슬롯의 TrueType 요청 자격을 빌리지 않는다"
    );
}

fn alias_metric_style(
    source: &str,
    alt_type: u8,
    query: &str,
    family: &str,
) -> rhwp::renderer::TextStyle {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn text_style(node: &RenderNode) -> Option<rhwp::renderer::TextStyle> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains('·') {
                return Some(run.style.clone());
            }
        }
        node.children.iter().find_map(text_style)
    }
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    doc.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.name = source.into();
        font.alt_type = alt_type;
        font.subst_font = None;
        font.is_embedded = false;
    }
    for shape in &mut doc.doc_info.char_shapes {
        shape.bold = false;
        shape.italic = false;
        shape.base_size = 1500;
    }
    let environment = FontEnvironment::from_available_programs(
        &doc.doc_info,
        &json!([{
            "query": query, "family": family, "programId": "explicit-alias-contract",
            "trueType": true, "openTypeCff": false
        }])
        .to_string(),
    )
    .unwrap()
    .expect("명시적으로 공급한 원본 프로그램의 인증 결과");
    core.set_document(doc);
    core.insert_text_native(0, 0, 0, "가·나").unwrap();
    core.set_font_environment(Some(environment)).unwrap();
    text_style(&core.build_page_render_tree(0).unwrap().root).expect("문장부호를 소유한 실제 run")
}

#[test]
fn verified_original_truetype_alias_keeps_its_metric_and_punctuation_advance() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    // 원 #7092의 검증된 주 TTF 폭 계약이다. 설치 글꼴의 윤곽을 읽지 않는다.
    for (source, family) in [("HY헤드라인M", "HYHeadLine M"), ("HY견고딕", "HYgtrE")] {
        let original = alias_metric_style(source, 1, source, source);
        let alias = alias_metric_style(source, 1, source, family);
        assert!(original.font_metric_trusted);
        assert!(
            alias.font_metric_trusted,
            "같은 원본 쿼리와 같은 전진의 영문 family 별칭: {source} / {family}"
        );
        let measurer = EmbeddedTextMeasurer;
        assert_eq!(
            measurer.compute_char_positions("가·나", &alias),
            measurer.compute_char_positions("가·나", &original),
            "표기 인증만으로 가운데점과 뒤 글자의 전진을 줄이지 않는다: {source} / {family}"
        );
    }
}

#[test]
fn unrelated_or_legacy_program_does_not_inherit_verified_truetype_metrics() {
    let unrelated = alias_metric_style("HY헤드라인M", 1, "HY헤드라인M", "Arial");
    assert!(
        !unrelated.font_metric_trusted,
        "다른 메트릭을 원래 검증된 표로 승격하지 않는다"
    );
    let legacy = alias_metric_style("HY헤드라인M", 2, "HY헤드라인M", "HYHeadLine M");
    assert!(
        !legacy.font_metric_trusted,
        "원본 HFT에는 주 TTF의 신뢰를 새로 부여하지 않는다"
    );
    let different_query = alias_metric_style("HY헤드라인M", 1, "HYHeadLine-Medium", "HYHeadLine M");
    assert!(
        !different_query.font_metric_trusted,
        "다른 조회 이름의 선택으로 원본 TTF의 인증을 대신하지 않는다"
    );
}
