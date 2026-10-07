//! 각주 번호 표시가 숨긴 문자는 본문 커서의 원본 주소를 이동시키지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

#[test]
fn footnote_number_prefix_preserves_final_caret_offset() {
    for text in ["가나다", "첫째 각주🦦"] {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "본문 내용").unwrap();
        let inserted: Value =
            serde_json::from_str(&core.insert_footnote_native(0, 0, 2).unwrap()).unwrap();
        let control = inserted["controlIdx"].as_u64().unwrap() as usize;
        core.insert_text_in_footnote_native(0, 0, control, 0, 2, text)
            .unwrap();

        let end = 2 + text.chars().count();
        let mut xs = Vec::new();
        for offset in 2..=end {
            let rect: Value = serde_json::from_str(
                &core
                    .get_cursor_rect_in_note_native(0, 0, control, 0, offset)
                    .unwrap(),
            )
            .unwrap();
            xs.push(rect["x"].as_f64().unwrap());
        }
        assert!(
            xs.windows(2).all(|pair| pair[1] > pair[0]),
            "{text:?}: 각주 본문 글자 앞뒤 캐럿이 겹치면 안 됩니다: {xs:?}",
        );
    }
}
