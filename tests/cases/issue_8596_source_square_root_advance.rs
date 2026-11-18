//! 관계형: 표시 family 선택 전후 원 HFT 기호의 전진·원문 소유·저장 바이트 보존.
//! 주석 갱신: 2026-11-07T16:28:27+09:00 (새 파일).
//! 측정: 2026-11-07, 96 DPI, 독립 Hancom Print, Native Skia/fresh WASM Canvas,
//! `visual_sweep.py --silhouette-only --resume`의 보정 전 2px raw.
//! 원본 `samples/issue8596/policy_committee_agenda.hwp`, 독립 Print
//! `pdf/issue8596_policy_committee_agenda-2020.pdf`, 전체 14쪽 = PDF 14쪽.
//! 보정 전 전쪽 최저 Native 96.52% / fresh WASM 98.44%, 11쪽 97.59% / 99.96%.
//! 증거: `mydocs/pr/assets/planet6897_20261106/pr8604/source-square-root-r37/`.
//! 한양신명조 U+221A의 독립 PDF T6/HFT285 전진 1000/1000 em을 원본 글자 크기와 비교한다.
//! 절대 px 좌표·영역은 고정하지 않으며 시스템 OTF/TTF 설치나 외부 글꼴 바이트에 의존하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::{json, Value};

fn marks(core: &DocumentCore) -> Vec<Value> {
    let trace: Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(10, r#"{"maxCharacters":4096}"#)
            .expect("원본 11쪽의 공개 글꼴 결정"),
    )
    .unwrap();
    trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|record| {
            record["source"]["character"] == "√" && record["document"]["face"] == "한양신명조"
        })
        .cloned()
        .collect()
}

#[test]
fn selected_cff_display_name_preserves_the_source_square_root_advance() {
    let bytes = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue8596/policy_committee_agenda.hwp"
    ))
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(false);
    let original_saved = core.export_hwpx_native().unwrap();
    let original_pages = core.page_count();
    let original = marks(&core);
    assert_eq!(original.len(), 2, "원본 예산·연구 방식 행의 체크 두 개");

    for family in ["HY Sinmyeongjo", "Caller CFF Display Family"] {
        // 실제 공급자의 메타데이터만 선언한다. 특정 글꼴 설치·로컬 파일은 검사 조건이 아니다.
        let environment = json!({
            "id": "source-hft-square-root-program",
            "preserveSourceMetrics": true,
            "substitutions": {"한양신명조": family},
            "selectedPrograms": {"한양신명조": {
                "query": "한양신명조", "family": family,
                "programId": "caller-verified-cff-program",
                "trueType": false, "openTypeCff": true
            }}
        });
        core.set_font_environment(Some(
            FontEnvironment::from_json(&environment.to_string()).unwrap(),
        ))
        .unwrap();
        let selected = marks(&core);
        assert_eq!(selected.len(), original.len(), "체크 누락·중복 없음");
        for (before, after) in original.iter().zip(&selected) {
            assert_eq!(after["source"], before["source"], "원문·셀 소유 보존");
            assert_eq!(
                after["layoutName"]["cssFamilyChain"][0], family,
                "실제로 표시 family가 바뀐 경로를 검사"
            );
            let shape = after["source"]["charShapeId"].as_u64().unwrap() as usize;
            let source_size = core.document().doc_info.char_shapes[shape].base_size as f64;
            // 독립 PDF의 d1 폭은 1000, em은 1000이다. 후보 bbox에서 정답을 만들지 않는다.
            let expected = source_size * 1000.0 / 1000.0;
            assert_eq!(
                after["layoutMetric"]["baseAdvanceHwpunit"].as_f64(),
                Some(expected),
                "표시용 이름 변경이 원 HFT의 1em 전진을 반각 추정으로 바꾸지 않음"
            );
            assert_eq!(
                after["layoutMetric"]["baseAdvanceHwpunit"],
                before["layoutMetric"]["baseAdvanceHwpunit"],
                "동일 원본의 기호 전진 보존"
            );
            assert_ne!(
                after["layoutMetric"]["widthSource"], "heuristicHalfwidth",
                "독립적으로 확인된 원본 폭을 소비"
            );
        }
        assert_eq!(core.page_count(), original_pages, "출력 쪽 소속 보존");
        assert_eq!(core.export_hwpx_native().unwrap(), original_saved);
    }
}
