//! 관계형: 인쇄 세대와 무관한 연속 그림자 계획·본문 소유·저장 보존을 검사한다.
//! 주석 갱신: 2026-11-10T09:15:00+09:00 (Asia/Seoul).
//! MIT 합성 TTF 내장 계약으로 설치 글꼴·고정 px에 의존하지 않는다.
//! 관련 interview 전2쪽 stage2 최소 글꼴 최저 Native 97.02270%, fresh WASM 97.15145%.
//! 근거: mydocs/pr/assets/issue8856/stage2-table-circles/validation.json.
//! 독립 한컴 2020/2024의 NONE/DROP/CONTINUOUS 대조와 원본 분석은 stage3 작업 기록을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use super::legacy_shadow_fixture::{actual_run, fixture, host_request_document};
use rhwp::paint::RenderProfile;

#[test]
fn legacy_and_current_print_keep_the_same_continuous_samples() {
    let legacy = actual_run(&fixture(false, 2), RenderProfile::Print);
    let current = actual_run(&fixture(true, 2), RenderProfile::Print);
    let a = legacy
        .style
        .continuous_shadow
        .as_ref()
        .expect("2020 Print도 연속 그림자를 준비한다");
    let b = current
        .style
        .continuous_shadow
        .as_ref()
        .expect("2024 Print 계획 유지");
    assert_eq!(
        a.source_hash, b.source_hash,
        "같은 실제 내장 프로그램에서 해소한다"
    );
    let left = a.offsets().collect::<Vec<_>>();
    let right = b.offsets().collect::<Vec<_>>();
    assert!(left.len() > 1, "끝점 하나로 연속 그림자를 대신하지 않는다");
    assert_eq!(left, right, "독립 두 인쇄 세대와 같은 signed 중간 표본");
    assert_eq!(legacy.style.shadow_offset_x, current.style.shadow_offset_x);
    assert_eq!(legacy.style.shadow_offset_y, current.style.shadow_offset_y);
}

#[test]
fn discrete_and_absent_shadow_do_not_gain_continuous_samples() {
    for current in [false, true] {
        for kind in [0, 1] {
            let run = actual_run(&fixture(current, kind), RenderProfile::Print);
            assert_eq!(run.style.shadow_type, kind);
            assert!(
                run.style.continuous_shadow.is_none(),
                "NONE·DROP의 정상 계약 유지"
            );
        }
    }
}

#[test]
fn screen_keeps_its_own_shadow_for_both_generations() {
    for current in [false, true] {
        let run = actual_run(&fixture(current, 2), RenderProfile::Screen);
        assert_eq!(run.style.shadow_type, 2);
        assert!(
            run.style.continuous_shadow.is_none(),
            "화면 경로에 Print 장치 표본을 넣지 않는다"
        );
    }
}

#[test]
fn legacy_host_supply_requests_include_the_actual_shadow_text() {
    for current in [false, true] {
        let core = host_request_document(current, 2);
        let requests: Vec<serde_json::Value> =
            serde_json::from_str(&core.get_text_shadow_font_requests_native().unwrap()).unwrap();
        assert!(
            requests.iter().any(|r| {
                r["text"].as_str().is_some_and(|text| text.contains('A'))
                    && r["families"].as_array().is_some_and(|families| {
                        families
                            .iter()
                            .any(|family| family.as_str() == Some("RHWP Host Fixture"))
                    })
            }),
            "내장 글꼴이 없는 원본도 실제 host 프로그램을 요청한다: current={current}"
        );
    }
}

#[test]
fn legacy_tab_without_continuous_shadow_keeps_the_old_request_contract() {
    for kind in [0, 1] {
        let mut core = host_request_document(false, kind);
        core.insert_text_native(0, 0, 1, "\tA").unwrap();
        let requests: Vec<serde_json::Value> =
            serde_json::from_str(&core.get_text_shadow_font_requests_native().unwrap()).unwrap();
        assert!(
            requests.is_empty(),
            "2024 탭 리더의 공급 규칙을 이전 세대로 확장하지 않는다"
        );
    }
}

#[test]
fn legacy_page_supply_requests_include_only_the_owned_shadow_text() {
    for current in [false, true] {
        let mut core = host_request_document(current, 2);
        core.insert_page_break_native(0, 0, 1).unwrap();
        core.insert_text_native(0, 1, 0, "B").unwrap();
        assert_eq!(core.page_count(), 2);
        for (page, expected) in [(0, "A"), (1, "B")] {
            let requests: serde_json::Value =
                serde_json::from_str(&core.get_page_paint_font_requests_native(page).unwrap())
                    .unwrap();
            let glyphs = requests["textEffects"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|request| request["text"].as_str().unwrap().chars())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(
                glyphs,
                expected.chars().collect(),
                "페이지별 공급은 같은 쪽이 소유한 그림자 문자만 요청한다: current={current}, page={page}"
            );
        }
        assert!(core.get_page_paint_font_requests_native(2).is_err());
    }
}
