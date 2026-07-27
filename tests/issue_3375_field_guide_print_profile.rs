//! Issue #3375: 빈 누름틀 안내문이 인쇄 등가 프로필에서도 출력되던 문제.
//!
//! 한컴은 안내문(빨간 이탤릭)을 편집 화면에서만 보여 주고 인쇄·PDF 에는 내보내지 않는다.
//! 그림 미지정 placeholder(#2225/#2297)에는 이미 프로필 계약이 있었지만 누름틀 안내문 축에는
//! 적용돼 있지 않았다.
//!
//! 렌더 노드의 `editor_only` 표시는 paint LayerBuilder 가 이미 프로필로 걸러내지만, SVG
//! 렌더러는 렌더 트리를 직접 순회해 그 계약 밖에 있었다 — 두 곳을 함께 맞춘다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const SAMPLE: &str = "samples/field-01.hwp";
/// `samples/field-01.hwp` 의 빈 누름틀 안내문.
const GUIDE: &str = "여기에 입력";

fn svg_text(profile: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let svg = doc
        .render_page_svg_with_profile(0, profile)
        .unwrap_or_else(|e| panic!("render {profile}: {e:?}"));
    // `<text>` 안 내용만 이어 붙여 공백을 지운다 — SVG 는 글자를 여러 요소로 쪼갠다.
    let mut out = String::new();
    let mut rest = svg.as_str();
    while let Some(open) = rest.find("<text") {
        let after = &rest[open..];
        let Some(gt) = after.find('>') else {
            break;
        };
        let body = &after[gt + 1..];
        let Some(close) = body.find("</text>") else {
            break;
        };
        out.push_str(&body[..close]);
        rest = &body[close + "</text>".len()..];
    }
    out.chars().filter(|c| !c.is_whitespace()).collect()
}

fn guide_needle() -> String {
    GUIDE.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 편집 화면 프로필에서는 안내문이 보여야 한다(종전 동작 보존).
#[test]
fn screen_profile_keeps_field_guide() {
    let text = svg_text("screen");
    assert!(
        text.contains(&guide_needle()),
        "편집 화면에서는 안내문이 보여야 한다"
    );
}

/// 인쇄 등가 프로필에서는 안내문이 나가지 않아야 한다.
#[test]
fn print_profile_suppresses_field_guide() {
    let text = svg_text("print");
    assert!(
        !text.contains(&guide_needle()),
        "인쇄 프로필에 안내문이 남았다: {text}"
    );
}

/// 안내문 억제는 **표시**만 바꾼다 — 본문 텍스트는 두 프로필에서 같아야 한다.
/// (안내문은 별도 마커 노드라 흐름 폭에 영향이 없어 쪽수·줄바꿈이 갈리지 않는다.)
#[test]
fn suppression_only_removes_guide_text() {
    let screen = svg_text("screen");
    let print = svg_text("print");
    let needle = guide_needle();
    let screen_without_guides = screen.replace(&needle, "");
    assert_eq!(
        screen_without_guides, print,
        "안내문 외의 본문이 프로필에 따라 달라졌다"
    );
}
