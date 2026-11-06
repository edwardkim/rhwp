//! 관계형: 실제 프로그램의 가족 이름 조회와 원본 슬롯 요청 자격을 검사한다.
//! 주석 갱신: 2026-11-06T20:48:46+09:00.
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
