//! 각주·머리말 문단의 누름틀 입력: 같은 자리의 빈 누름틀은 첫 것 하나만, 활성 누름틀은 그 누름틀이 친 글을 품는다.
//! devel은 경로로 각주에 누름틀을 넣지 못해 머리말로 본다. 각주 입력도 같은 함수를 쓴다.
//!
//! 본문·표 셀 입력은 활성 누름틀과 누름틀 슬롯 규칙을 따르지만, 각주·머리말 입력은 일반 삽입을 써서 같은 자리의
//! 빈 누름틀이 모두 친 글을 품고(겹친 필드로 저장된다) 활성 누름틀도 보지 않았다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

fn blank(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    core
}

fn json(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap()
}

/// 각주·머리말 첫 문단의 누름틀 범위: (컨트롤 번호, 시작, 끝).
fn ranges(core: &DocumentCore, para: usize, control: usize) -> Vec<(usize, usize, usize)> {
    let paragraphs = match &core.document().sections[0].paragraphs[para].controls[control] {
        Control::Footnote(note) => &note.paragraphs,
        Control::Header(header) => &header.paragraphs,
        _ => panic!("각주·머리말이 아니다"),
    };
    paragraphs[0]
        .field_ranges
        .iter()
        .map(|range| (range.control_idx, range.start_char_idx, range.end_char_idx))
        .collect()
}

/// 저장한 HWP·HWPX를 다시 열어도 같은 범위인지 본다.
fn assert_saved(core: &DocumentCore, para: usize, control: usize) {
    let expected = ranges(core, para, control);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(ranges(&reopened, para, control), expected);
    }
}

#[test]
fn header_typing_keeps_the_active_field_and_fills_one_empty_field() {
    let mut core = blank("본문");
    let header = json(&core.create_header_footer_native(0, true, 0).unwrap());
    let (hp, hc) = (
        header["paraIndex"].as_u64().unwrap() as usize,
        header["controlIndex"].as_u64().unwrap() as usize,
    );
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "머리 끝")
        .unwrap();
    for name in ["다", "라"] {
        core.insert_click_here_field_at_by_path(0, hp, &[(hc, 0, 0)], 3, "안내", "", name, true)
            .unwrap();
    }
    let [(c, _, _), (d, _, _)] = ranges(&core, hp, hc)[..] else {
        panic!("누름틀 둘이 있어야 한다");
    };
    // 같은 자리의 빈 누름틀 둘 가운데 첫 것만 친 글을 품는다.
    core.insert_text_in_header_footer_native(0, true, 0, 0, 3, "값")
        .unwrap();
    assert_eq!(ranges(&core, hp, hc), vec![(c, 3, 4), (d, 4, 4)]);

    // 활성으로 둔 다의 끝에서 친 글은 그 자리의 빈 라가 아니라 다에 든다.
    assert!(core.set_active_field_by_path(0, hp, &[(hc, 0, 0)], 3));
    core.insert_text_in_header_footer_native(0, true, 0, 0, 4, "더")
        .unwrap();
    assert_eq!(ranges(&core, hp, hc), vec![(c, 3, 5), (d, 5, 5)]);
    assert_saved(&core, hp, hc);
}
