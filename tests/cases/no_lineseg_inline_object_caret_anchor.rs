//! 관계형·고정형: no_lineseg_inline_object_caret_anchor의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-04T00:23:31+09:00; 직전 파일 변경 커밋: 2026-11-03T23:22:54+09:00 c6ad3eea59d1b7701efdc4fb582e2d9a44b4f775.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. form-slot HWP/HWPX 각 1쪽 양쪽 100.00%는 정상 개체 경계 대조다. no-lineseg-empty/text 각 1쪽은 Native/fresh WASM 최저 100.00%; no-lineseg-wrap 1쪽은 92.90%/96.56%다. 독립 Print 확대 판독에서 그림의 같은 160..164px 밴드와 본문 순서를 확인했다. 절대 px는 고정하지 않는다. 논리 주소와 원본 글자 크기는 입력 계약, 캐럿은 live/재열기 관계다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/validation.md 및 전쪽 TSV·확대 PNG.
//! 저장 줄 없이 다시 연 문단도 개체 뒤 캐럿과 글자 위치를 저장 줄 경로와 같게 둔다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

const PNG: &[u8] = include_bytes!("../../assets/logo/logo-16.png");

fn runs(core: &DocumentCore) -> Vec<(String, u64, f64)> {
    let layout: Value =
        serde_json::from_str(&core.get_page_text_layout_native(0).unwrap()).unwrap();
    layout["runs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|run| run["paraIdx"] == 0)
        .map(|run| {
            (
                run["text"].as_str().unwrap().to_string(),
                run["charStart"].as_u64().unwrap(),
                run["fontSize"].as_f64().unwrap(),
            )
        })
        .collect()
}

fn document(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    if !text.is_empty() {
        core.insert_text_native(0, 0, 0, text).unwrap();
    }
    let made: Value = serde_json::from_str(
        &core
            .insert_picture_native(
                0,
                0,
                1.min(text.chars().count()),
                &[],
                PNG,
                1200,
                400,
                16,
                16,
                "png",
                "",
                None,
                None,
            )
            .unwrap(),
    )
    .unwrap();
    let control = made["controlIdx"].as_u64().unwrap() as usize;
    core.set_picture_properties_native(0, 0, control, r#"{"treatAsChar":true}"#)
        .unwrap();
    core.apply_char_format_native(0, 0, 0, text.chars().count(), r#"{"fontSize":1000}"#)
        .unwrap();
    core
}

#[test]
fn empty_inline_picture_paragraph_keeps_end_anchor_after_hwpx_without_line_cache() {
    let core = document("");
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    // 편집으로 계산한 줄 캐시는 HWPX에 쓰지 않으므로 다시 연 문단은 저장 줄이 없다.
    assert!(reopened.document().sections[0].paragraphs[0]
        .line_segs
        .is_empty());
    let expected = runs(&core);
    assert_eq!(expected, vec![(String::new(), 1, 13.3)]);
    assert_eq!(runs(&reopened), expected);
    assert_eq!(
        reopened.get_cursor_rect_native(0, 0, 1).unwrap(),
        core.get_cursor_rect_native(0, 0, 1).unwrap()
    );
}

#[test]
fn text_after_inline_picture_keeps_stored_line_char_start_without_line_cache() {
    let core = document("가나");
    let reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert!(reopened.document().sections[0].paragraphs[0]
        .line_segs
        .is_empty());
    let starts = |core: &DocumentCore| {
        runs(core)
            .into_iter()
            .filter(|(text, _, _)| !text.is_empty())
            .map(|(text, start, _)| (text, start))
            .collect::<Vec<_>>()
    };
    // 글 런은 인라인 그림 한 칸을 포함한 논리 축을 보존한다.
    // #8187 이전의 원문 축 기대값으로 현행 주소 계약을 되돌리지 않는다.
    assert_eq!(
        starts(&core),
        vec![("가".to_string(), 0), ("나".to_string(), 2)]
    );
    assert_eq!(starts(&reopened), starts(&core));
}

#[test]
fn wrapped_object_only_last_row_keeps_the_paragraph_end_style_and_anchor() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.set_hangul2024_compat(true);
    core.insert_text_native(0, 0, 0, "가나다라마바").unwrap();
    let made: Value = serde_json::from_str(
        &core
            .insert_picture_native(0, 0, 6, &[], PNG, 40000, 400, 16, 16, "png", "", None, None)
            .unwrap(),
    )
    .unwrap();
    core.set_picture_properties_native(
        0,
        0,
        made["controlIdx"].as_u64().unwrap() as usize,
        r#"{"treatAsChar":true}"#,
    )
    .unwrap();
    let mut reopened = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert!(reopened.document().sections[0].paragraphs[0]
        .line_segs
        .is_empty());
    reopened.set_hangul2024_compat(true);
    for doc in [&core, &reopened] {
        let start: Value =
            serde_json::from_str(&doc.get_cursor_rect_native(0, 0, 0).unwrap()).unwrap();
        let end: Value =
            serde_json::from_str(&doc.get_cursor_rect_native(0, 0, 7).unwrap()).unwrap();
        assert!(
            end["y"].as_f64().unwrap() > start["y"].as_f64().unwrap(),
            "그림은 다음 물리 줄: {start} / {end}"
        );
        assert_eq!(
            end["height"], start["height"],
            "같은 글자 모양의 문단 끝이 기본 12px로 바뀌지 않는다"
        );
        let end_run = runs(doc)
            .into_iter()
            .find(|(text, pos, _)| text.is_empty() && *pos == 7)
            .expect("마지막 그림 뒤의 논리 끝 앵커");
        assert_eq!(end_run.2, start["height"].as_f64().unwrap());
    }
}
