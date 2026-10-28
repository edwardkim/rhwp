//! 한컴에서 열 수 있는 기존 이동 저장본의 논리 캐럿·본문·차트 보존.
//! 손상 경고가 확인된 일반 HWP 두 개는 수집하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
#[path = "../support/inline_control_move_contract.rs"]
mod fixture_support;
use fixture_support::{body, caret, chart_content, source};

#[test]
fn valid_inline_move_fixtures_preserve_saved_carets_and_chart_data() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/inline-control-move");
    let expected_chart = chart_content(
        &DocumentCore::from_bytes(&std::fs::read(root.join("chart-before.hwpx")).unwrap()).unwrap(),
    );
    {
        let mut core =
            DocumentCore::from_bytes(&std::fs::read(root.join("before.hwpx")).unwrap()).unwrap();
        let after =
            DocumentCore::from_bytes(&std::fs::read(root.join("after.hwpx")).unwrap()).unwrap();
        let before_text: Vec<_> = core.document().sections[0]
            .paragraphs
            .iter()
            .map(|p| p.text.clone())
            .collect();
        for (offset, pi) in (1..=5).enumerate() {
            let ci = core.document().sections[0].paragraphs[pi]
                .controls
                .iter()
                .position(|c| c.is_treat_as_char_object())
                .unwrap();
            core.move_inline_control_native(&source(body(pi), ci), &caret(body(6), 7 + offset))
                .unwrap();
        }
        assert_eq!(
            core.document().sections[0]
                .paragraphs
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>(),
            before_text
        );
        assert_eq!(
            after.document().sections[0]
                .paragraphs
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>(),
            before_text
        );
        for p in &after.document().sections[0].paragraphs[1..6] {
            assert!(!p.controls.iter().any(|c| c.is_treat_as_char_object()));
        }
        for document in [&core, &after] {
            let p = &document.document().sections[0].paragraphs[6];
            assert_eq!(p.controls.len(), 5);
            assert_eq!(p.control_text_positions(), vec![7; 5]);
            assert_eq!(document.page_count(), 1);
        }
    }
    for ext in ["hwp", "hwpx"] {
        for name in ["chart-before", "chart-after"] {
            let chart = DocumentCore::from_bytes(
                &std::fs::read(root.join(format!("{name}.{ext}"))).unwrap(),
            )
            .unwrap();
            assert_eq!(chart.page_count(), 1);
            assert_eq!(chart_content(&chart), expected_chart);
            assert_eq!(
                chart.document().sections[0]
                    .paragraphs
                    .iter()
                    .flat_map(|p| &p.controls)
                    .filter(|c| c.is_treat_as_char_object())
                    .count(),
                1
            );
        }
    }
}
