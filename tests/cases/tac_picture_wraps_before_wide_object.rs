//! 글자처럼 취급한 그림 앞에 글자를 쳐서 글자와 그림이 한 줄에 다 들어가지 않으면
//! 그림 앞에서 줄을 나눈다. 그림이 문단 끝에 있어도 마찬가지다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::paragraph::Paragraph;

const TEXT: &str = "가나다라마바사아자차카타파하";

/// 글자처럼 취급한 그림만 있는 문단에서 그림 앞에 `TEXT` 를 친다.
fn typed_before_tac_picture(width: u32, height: u32) -> Paragraph {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let png = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/logo/logo-16.png"
    ))
    .unwrap();
    let result = core
        .insert_picture_native(
            0,
            0,
            0,
            &[],
            &png,
            width,
            height,
            16,
            16,
            "png",
            "그림",
            None,
            None,
        )
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    let control = result["controlIdx"].as_u64().unwrap() as usize;
    core.set_picture_properties_native(0, 0, control, r#"{"treatAsChar":true}"#)
        .unwrap();
    core.insert_text_native(0, 0, 0, TEXT).unwrap();
    core.document().sections[0].paragraphs[0].clone()
}

#[test]
fn text_typed_before_wide_tac_picture_breaks_before_picture() {
    // 단 폭(42520)의 약 90% 인 그림.
    let para = typed_before_tac_picture(38400, 15360);
    let starts: Vec<u32> = para.line_segs.iter().map(|seg| seg.text_start).collect();
    // 그림은 마지막 글자 바로 뒤에 있다. 첫 줄은 글자만, 둘째 줄은 그림 자리에서 시작한다.
    let picture_at = para.char_offsets.last().unwrap() + 1;
    assert_eq!(starts, vec![0, picture_at], "{:?}", para.line_segs);
    assert!(
        para.line_segs[0].line_height < 15360,
        "글자 줄이 그림 높이를 가지면 안 된다: {:?}",
        para.line_segs
    );
    assert!(
        para.line_segs[1].line_height >= 15360,
        "{:?}",
        para.line_segs
    );
}

#[test]
fn text_typed_before_narrow_tac_picture_keeps_one_line() {
    let para = typed_before_tac_picture(4000, 4000);
    assert_eq!(para.line_segs.len(), 1, "{:?}", para.line_segs);
    assert!(
        para.line_segs[0].line_height >= 4000,
        "{:?}",
        para.line_segs
    );
}
