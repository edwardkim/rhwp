//! Issue #1079: 비-TAC TopAndBottom(vert=Para) 그림에서 파일 vpos 가 이미 그림 공간을
//! 반영하는데 typeset/렌더가 그림 높이를 추가로 더해(이중 계상) 그림이 본문 하단을 초과 +
//! 페이지가 분리되는 결함 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플): `samples/pr-149.hwp` (그림 3개 + 텍스트).
//! 한컴 정답지: `pdf/pr-149-2022.pdf` — 1페이지에 그림 3개(원본/회색조/흑백) + "입니다." 수용.
//!
//! 정정: gap_before(그림 para 줄 앞 빈 공간) ≥ 그림 높이이면 파일 vpos 가 이미 그림을 반영한
//! 것 → typeset pushdown 생략 + 렌더는 그림을 그 gap 안에 그리고 추가 진행 생략.

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn svg_height(svg: &str) -> f64 {
    let i = svg.find("height=\"").expect("svg height") + "height=\"".len();
    let rest = &svg[i..];
    rest[..rest.find('"').unwrap()].parse().expect("h")
}

/// 태그(<text>/<image>) 의 최대 (y + height|0) 를 반환.
fn max_bottom(svg: &str, tag: &str) -> f64 {
    let mut max = 0.0_f64;
    let mut rest = svg;
    let open = format!("<{tag}");
    while let Some(o) = rest.find(&open) {
        let end = rest[o..].find('>').map(|g| o + g).unwrap_or(rest.len());
        let t = &rest[o..end];
        let y = t
            .find(" y=\"")
            .and_then(|i| {
                t[i + 4..]
                    .find('"')
                    .map(|e| t[i + 4..i + 4 + e].parse::<f64>().ok())
            })
            .flatten();
        let h = t
            .find(" height=\"")
            .and_then(|i| {
                t[i + 9..]
                    .find('"')
                    .map(|e| t[i + 9..i + 9 + e].parse::<f64>().ok())
            })
            .flatten()
            .unwrap_or(0.0);
        if let Some(y) = y {
            max = max.max(y + h);
        }
        rest = &rest[end..];
    }
    max
}

fn svg_text(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(o) = rest.find("<text") {
        if let Some(gt) = rest[o..].find('>') {
            let after = &rest[o + gt + 1..];
            if let Some(c) = after.find("</text>") {
                out.push_str(&after[..c]);
                rest = &after[c + 7..];
                continue;
            }
        }
        break;
    }
    out
}

/// pr-149 는 한컴 PDF 와 동일하게 1페이지에 수용되어야 한다.
#[test]
fn pr149_single_page() {
    let doc = load_doc("samples/pr-149.hwp");
    assert_eq!(
        doc.page_count(),
        1,
        "pr-149: 그림 pushdown 이중 계상으로 2페이지 분리 회귀"
    );
}

/// 1페이지 안에서 텍스트·그림이 본문(페이지 높이) 을 넘지 않고, 본문 마지막 텍스트("입니다.")
/// 가 누락 없이 렌더된다.
#[test]
fn pr149_content_within_page_and_complete() {
    let doc = load_doc("samples/pr-149.hwp");
    let svg = doc.render_page_svg_native(0).expect("render page 0");
    let h = svg_height(&svg);
    let text_bottom = max_bottom(&svg, "text");
    let image_bottom = max_bottom(&svg, "image");
    assert!(
        text_bottom <= h,
        "pr-149: 텍스트 하단 {text_bottom:.1} > 페이지 {h:.1} (그림 pushdown 회귀)"
    );
    assert!(
        image_bottom <= h,
        "pr-149: 그림 하단 {image_bottom:.1} > 페이지 {h:.1}"
    );
    assert!(
        svg_text(&svg).contains("입니다"),
        "pr-149: 본문 마지막 텍스트 '입니다.' 누락"
    );
}
