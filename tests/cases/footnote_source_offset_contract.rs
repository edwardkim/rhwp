//! 각주·미주 번호 표시가 숨긴 문자는 주석 커서의 원본 주소를 이동시키지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

/// 본문 2번 자리에 각주나 미주(`endnote`)를 넣고, 그 첫 문단의 입력 자리(2)에 `text`를 쓴다.
fn core_with_note(endnote: bool, text: &str) -> (DocumentCore, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "본문 내용").unwrap();
    let inserted = if endnote {
        core.insert_endnote_native(0, 0, 2)
    } else {
        core.insert_footnote_native(0, 0, 2)
    };
    let inserted: Value = serde_json::from_str(&inserted.unwrap()).unwrap();
    let control = inserted["controlIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_footnote_native(0, 0, control, 0, 2, text)
        .unwrap();
    (core, control)
}

/// 입력 자리(2)부터 `text` 끝까지 글자 위치마다의 캐럿 x
fn caret_xs(core: &DocumentCore, control: usize, text: &str) -> Vec<f64> {
    (2..=2 + text.chars().count())
        .map(|offset| {
            let rect: Value = serde_json::from_str(
                &core
                    .get_cursor_rect_in_note_native(0, 0, control, 0, offset)
                    .unwrap(),
            )
            .unwrap();
            rect["x"].as_f64().unwrap()
        })
        .collect()
}

#[test]
fn footnote_number_prefix_preserves_final_caret_offset() {
    for text in ["가나다", "첫째 각주🦦", "첫째 "] {
        let (core, control) = core_with_note(false, text);
        let xs = caret_xs(&core, control, text);
        assert!(
            xs.windows(2).all(|pair| pair[1] > pair[0]),
            "{text:?}: 각주 본문 글자 앞뒤 캐럿이 겹치면 안 됩니다: {xs:?}",
        );
    }
}

/// 미주 첫 문단은 번호 자리표시 공백을 지우고 `1) `을 붙여 그린다. 입력한 글자 앞뒤의
/// 캐럿은 그린 문단에서 그 글자의 경계에 놓여야 한다.
#[test]
fn endnote_caret_sits_on_drawn_character_boundaries() {
    for text in ["가나다", "첫째 각주🦦", "첫째 "] {
        let (core, control) = core_with_note(true, text);
        let info: Value =
            serde_json::from_str(&core.get_note_edit_info_native(0, 0, control).unwrap()).unwrap();
        let page = info["pageNum"].as_u64().unwrap() as u32;
        let para = info["virtualParaIndex"].as_u64();
        let layout: Value =
            serde_json::from_str(&core.get_page_text_layout_native(page).unwrap()).unwrap();

        // 그린 미주 문단의 글자 경계 x. 마지막 값은 문단 끝이다.
        let mut drawn_text = String::new();
        let mut boundaries = Vec::new();
        for run in layout["runs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|run| run["paraIdx"].as_u64() == para)
        {
            let x = run["x"].as_f64().unwrap();
            boundaries.truncate(drawn_text.chars().count());
            boundaries.extend(
                run["charX"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|char_x| x + char_x.as_f64().unwrap()),
            );
            drawn_text.push_str(run["text"].as_str().unwrap());
        }
        assert!(
            drawn_text.ends_with(text),
            "{text:?}: 그린 미주 문단 {drawn_text:?}"
        );
        let expected = &boundaries[drawn_text.chars().count() - text.chars().count()..];

        let xs = caret_xs(&core, control, text);
        assert!(
            xs.len() == expected.len()
                && xs.iter().zip(expected).all(|(x, e)| (x - e).abs() < 0.15),
            "{text:?}: 미주 캐럿 {xs:?}가 그린 {drawn_text:?}의 글자 경계 {expected:?}와 다릅니다",
        );
    }
}
