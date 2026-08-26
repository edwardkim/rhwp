//! Issue #3486: 한글 97 안내문의 한컴 PUA 머리말/Enter pictogram 두부 문자.
//!
//! HWP3, HWP5 및 HWPX의 동일 문서에는 공개 글꼴이 보유하지 않는 한컴 전용
//! PUA가 저장된다. IR 원문은 보존하고, paint 경로만 Hancom PDF 기준의 읽을 수
//! 있는 표준 문자로 치환한다.

use rhwp::renderer::composer::{expand_pua_render_text, pua_to_display_text};

#[test]
fn issue_3486_hancom_header_pua_projects_to_hancom_company_name() {
    let raw = "\u{F03EF}\u{F03F0}\u{F03F1}\u{F03F2}\u{F03F3}\u{F03F4}";
    assert_eq!(
        expand_pua_render_text(raw),
        "한글과컴퓨터",
        "Hancom PDF의 머리말과 달리 PUA 여섯 글자가 tofu로 남으면 안 됨",
    );
    assert_eq!(pua_to_display_text('\u{F03EF}').as_deref(), Some("한"));
    assert_eq!(pua_to_display_text('\u{F03F4}').as_deref(), Some("터"));
}

#[test]
fn issue_3486_hancom_enter_pictogram_never_reaches_paint_as_tofu() {
    assert_eq!(
        expand_pua_render_text("\u{F03A0}를 누르면"),
        "↵를 누르면",
        "한컴 Enter-key PUA는 공개 글꼴에 없는 두부 문자로 남으면 안 됨",
    );
    assert_eq!(pua_to_display_text('\u{F03A0}').as_deref(), Some("↵"));
}
