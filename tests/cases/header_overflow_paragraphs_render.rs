//! 관계형: header_overflow_paragraphs_render의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-04T00:23:31+09:00; 직전 파일 변경 커밋: 2026-11-03T21:45:03+09:00 39b4f81f6aab4cc08c6145ed6911db2a8fe5125e.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. 한컴 header-overflow HWP/HWPX 각 1쪽 Native/fresh WASM 최저 100.00%. 넘친 머리말과 본문 첫 줄의 겹침을 독립 Print와 직접 판독하고, 두 편집 경로의 소유를 따로 검사한다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/validation.md 및 전쪽 TSV·확대 PNG.
//! 머리말 틀을 넘친 문단도 한컴처럼 본문 위에 겹쳐 그린다.
//! 한컴 2024 확인: 70자 두 줄·빈 문단 뒤의 "마지막"이 본문 첫 줄과 겹쳐 보이고 본문은 밀리지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn run_y(core: &DocumentCore, text: &str) -> Option<f64> {
    let layout: serde_json::Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|run| run["text"] == text)
        .and_then(|run| run["y"].as_f64())
}

#[test]
fn header_paragraph_past_the_header_area_is_drawn_over_the_body() {
    for bytes in [
        &include_bytes!("../fixtures/header_overflow/header-overflow.hwp")[..],
        &include_bytes!("../fixtures/header_overflow/header-overflow.hwpx")[..],
    ] {
        let core = DocumentCore::from_bytes(bytes).unwrap();
        let body = run_y(&core, "본문").expect("본문 첫 줄");
        let overflow = run_y(&core, "마지막").expect("넘친 머리말 문단도 그린다");
        // 본문을 밀지 않고 그 위에 겹친다.
        assert!(overflow >= body - 1.0, "{overflow} < {body}");
        assert!(core.render_page_svg_native(0).unwrap().contains(">마<"));
        let header = core.document().sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .find_map(|c| match c {
                rhwp::model::control::Control::Header(h) => Some(h),
                _ => None,
            })
            .unwrap();
        let hp = header
            .paragraphs
            .iter()
            .position(|p| p.text.contains("마지막"))
            .unwrap();
        let rect: serde_json::Value = serde_json::from_str(
            &core
                .get_cursor_rect_in_header_footer_native(0, true, header.apply_to as u8, hp, 0, 0)
                .unwrap(),
        )
        .unwrap();
        let hit: serde_json::Value = serde_json::from_str(
            &core
                .hit_test_in_header_footer_native(
                    0,
                    true,
                    rect["x"].as_f64().unwrap() + 0.1,
                    rect["y"].as_f64().unwrap() + rect["height"].as_f64().unwrap() / 2.0,
                )
                .unwrap(),
        )
        .unwrap();
        assert_eq!(hit["hit"], true, "{hit}");
        assert_eq!(hit["paraIndex"], hp, "{hit}");
        let bp = core.document().sections[0]
            .paragraphs
            .iter()
            .position(|p| p.text.contains("본문"))
            .unwrap();
        let rect: serde_json::Value =
            serde_json::from_str(&core.get_cursor_rect_native(0, bp, 0).unwrap()).unwrap();
        let hit: serde_json::Value = serde_json::from_str(
            &core
                .hit_test_native(
                    0,
                    rect["x"].as_f64().unwrap() + 0.1,
                    rect["y"].as_f64().unwrap() + rect["height"].as_f64().unwrap() / 2.0,
                )
                .unwrap(),
        )
        .unwrap();
        assert_eq!(hit["paragraphIndex"], bp, "본문 소유 보존: {hit}");
    }
}
