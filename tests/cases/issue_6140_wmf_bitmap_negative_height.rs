//! [Issue #6140] bottom-up WMF 안의 비트맵이 상하로 뒤집혀 그려진다
//! (156462405 7쪽 "기조 강연자" 사진).
//!
//! 근인: WMF 의 목적 사각형이 음수 높이(`dest_height < 0`)로 오는데 그 값을 SVG
//! `height` 에 그대로 냈다. SVG 는 음수 `width`/`height` 를 **오류로 규정**하므로
//! 브라우저는 그 속성을 무시하고(=절대값·비반전) 그린다.
//!
//! [#6617] 이 WMF 는 y-up 창(`SetWindowExt` y<0)이다. GDI 에서 y-up 창의 음수 높이
//! 블릿은 두 부호가 상쇄돼 **바로 선** 그림이고, 창 매핑(`Window::to_device`)이 y 축을
//! 이미 뒤집으므로 SVG 에는 어떤 뒤집기 `transform` 도 남지 않아야 한다. 예전의
//! "y-flip 그룹 + 요소 자체 역변환" 조합은 창 바닥이 논리 0 일 때만 맞았고, 원점 0 인
//! 이 문서에서는 그림을 viewBox 밖으로 보냈다.
//!
//! 이 테스트는 그 계약을 SVG 산출로 고정한다 — CLI PNG 는 PyMuPDF 가 내장 SVG 를
//! 건너뛰어("ignoring external image") 이 결함을 못 잡는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use base64::Engine;
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6140/156462405_smart_expo.hwp";
/// 사진이 있는 쪽(0-based).
const PAGE: u32 = 6;

/// 사진 WMF: `SetWindowExt(281, -285)`, `SetWindowOrg(0, 0)`, `DIBStretchBlt dest (0, 0, 281, -285)`.
const PHOTO_VIEW_BOX: &str = "viewBox=\"0 0 281 285\"";

#[test]
fn issue_6140_wmf_bitmap_keeps_positive_extent_without_any_flip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(PAGE).expect("page 7 svg");

    let inner = embedded_svgs(&svg);
    assert!(!inner.is_empty(), "7쪽에 내장 WMF SVG 가 있어야 한다");

    let photo = inner
        .iter()
        .find(|doc| doc.contains(PHOTO_VIEW_BOX))
        .unwrap_or_else(|| panic!("y-up 창 사진 WMF({PHOTO_VIEW_BOX})를 찾지 못했다"));
    assert!(
        !photo.contains("scale(1,-1)") && !photo.contains("scale(-1,1)"),
        "y-up 창의 뒤집기는 창 매핑이 끝내므로 SVG 에 뒤집기 transform 이 남으면 안 된다: {}",
        image_elements(photo).join(" | ")
    );
    let images = image_elements(photo);
    assert_eq!(
        images.len(),
        1,
        "사진 비트맵 <image> 는 하나여야 한다: {images:?}"
    );
    let image = &images[0];
    assert!(
        !image.contains("height=\"-") && !image.contains("width=\"-"),
        "SVG 는 음수 width/height 를 오류로 규정한다: {image}"
    );
    for attr in ["x=\"0\"", "y=\"0\"", "width=\"281\"", "height=\"285\""] {
        assert!(
            image.contains(attr),
            "사진은 창 전체 (0, 0, 281, 285) 를 채워야 한다 — {attr} 없음: {image}"
        );
    }
}

/// 페이지 SVG 안에 data URI 로 박힌 WMF 산출 SVG 들.
fn embedded_svgs(svg: &str) -> Vec<String> {
    let marker = "data:image/svg+xml;base64,";
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(idx) = rest.find(marker) {
        let tail = &rest[idx + marker.len()..];
        let end = tail
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '+' || ch == '/' || ch == '='))
            .unwrap_or(tail.len());
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&tail[..end]) {
            if let Ok(text) = String::from_utf8(bytes) {
                out.push(text);
            }
        }
        rest = &tail[end..];
    }
    out
}

fn image_elements(doc: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = doc;
    while let Some(idx) = rest.find("<image") {
        let tail = &rest[idx..];
        let end = tail.find('>').map(|e| e + 1).unwrap_or(tail.len());
        // href 의 base64 본문은 어서션 메시지에서 잘라 낸다.
        let element = &tail[..end];
        let trimmed = match element.find("href=\"") {
            Some(h) => {
                let head = &element[..h];
                let after = &element[h..];
                let close = after[6..].find('"').map(|e| e + 7).unwrap_or(after.len());
                format!("{head}href=\"…\"{}", &after[close..])
            }
            None => element.to_string(),
        };
        out.push(trimmed);
        rest = &tail[end..];
    }
    out
}
