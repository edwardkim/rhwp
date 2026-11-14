//! #7727 저장 RIGHT 탭 전진의 소유를 확인하는 관계형 회귀 검사다.
//!
//! 주석 갱신: 2026-11-15 02:55 KST.
//! 검사 종류: 관계형. 입력의 저장 폭 변화와 다음 글자의 이동량을 비교하며,
//! 문서의 절대 px 좌표를 고정하지 않는다.
//! 보정 전 원본 전204쪽 최저: Native 98.11%, fresh WASM 98.08%.
//! 근거: 동일 원본의 BodyText와 ViewText에서 TAB 폭만 각각 +1500HU로 바꾼
//! 한컴2020 Print 대조군은 쪽번호를 +19.99996 CSSpx 이동했다.
//! MCP job c8e37f0f-e545-4880-b774-decf06def96e, 실제 제품 11.0.0.9136.
//! 원본 글꼴 설치에 의존하지 않고 공개 DocumentCore의 현재 문자 배치를 비교한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp";
const PARAGRAPH: usize = 23;

fn core_with_stored_tab_delta(delta_hu: u32, clear_stored_lines: bool) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("원본 파싱");
    core.set_hangul2024_compat(false);
    let mut document = core.document().clone();
    let paragraph = &mut document.sections[0].paragraphs[PARAGRAPH];
    assert!(paragraph.text.ends_with("분석\t10"));
    assert_eq!(paragraph.tab_extended.len(), 1);
    assert!(!paragraph.line_segs.is_empty(), "원본의 저장 줄을 유지한다");
    let tab = &mut paragraph.tab_extended[0];
    assert_eq!(tab[2] >> 8, 2, "저장 RIGHT 탭");
    let advance = (u32::from(tab[0]) | (u32::from(tab[1]) << 16)) + delta_hu;
    tab[0] = advance as u16;
    tab[1] = (advance >> 16) as u16;
    if clear_stored_lines {
        paragraph.line_segs.clear();
        paragraph.source_line_seg_vertical_pos = None;
    }
    core.set_document(document);
    core
}

fn page_number_origin(delta_hu: u32, clear_stored_lines: bool) -> f64 {
    let core = core_with_stored_tab_delta(delta_hu, clear_stored_lines);
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(1)
            .expect("목차 2쪽 문자 배치"),
    )
    .expect("문자 배치 JSON");
    let run = layout["runs"]
        .as_array()
        .expect("문자 런")
        .iter()
        .find(|run| {
            run["paraIdx"].as_u64() == Some(PARAGRAPH as u64)
                && run["text"].as_str() == Some("\t10")
        })
        .expect("원본 문단의 탭 뒤 쪽번호");
    run["x"].as_f64().expect("런 원점") + run["charX"][1].as_f64().expect("쪽번호 문자 원점")
}

#[test]
fn accepted_stored_right_tab_width_owns_the_following_number_origin() {
    let before = page_number_origin(0, false);
    let after = page_number_origin(1500, false);
    let source_delta = 1500.0 * 96.0 / 7200.0;
    assert!(
        ((after - before) - source_delta).abs() < 1e-6,
        "유효한 저장 탭의 폭 변화가 다음 글자에 한 번만 반영돼야 한다: {before} → {after}"
    );
}

#[test]
fn reflow_without_stored_lines_does_not_reuse_the_old_tab_advance() {
    let before = page_number_origin(0, true);
    let after = page_number_origin(1500, true);
    assert!(
        (after - before).abs() < 1e-6,
        "저장 줄이 없는 재조판에서는 현재 정지점이 배치를 소유한다: {before} → {after}"
    );
}
