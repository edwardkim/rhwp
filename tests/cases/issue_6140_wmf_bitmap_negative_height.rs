//! [Issue #6140] bottom-up WMF 안의 비트맵이 상하로 뒤집혀 그려진다
//! (156462405 7쪽 "기조 강연자" 사진).
//!
//! 근인: WMF 의 목적 사각형이 음수 높이(`dest_height < 0`)로 오는데 그 값을 SVG
//! `height` 에 그대로 냈다. SVG 는 음수 `width`/`height` 를 **오류로 규정**하므로
//! 브라우저는 그 속성을 무시하고(=절대값·비반전) 그린다. 이 WMF 는 이미 bottom-up
//! y-flip 그룹(`translate(0,H) scale(1,-1)`) 안에 있어, 비트맵 자신의 되돌림이
//! 사라지면 그룹의 flip 만 남아 사진이 뒤집힌다.
//!
//! 수정: 원점·크기를 양수로 정규화하고 뒤집힘은 요소 자신의 `transform` 으로
//! 표현한다. 이 테스트는 그 계약을 SVG 산출로 고정한다 — CLI PNG 는 PyMuPDF 가
//! 내장 SVG 를 건너뛰어("ignoring external image") 이 결함을 못 잡는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use base64::Engine;
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6140/156462405_smart_expo.hwp";
/// 사진이 있는 쪽(0-based).
const PAGE: u32 = 6;

#[test]
fn issue_6140_wmf_bitmap_keeps_positive_extent_and_own_flip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(PAGE).expect("page 7 svg");

    let inner = embedded_svgs(&svg);
    assert!(!inner.is_empty(), "7쪽에 내장 WMF SVG 가 있어야 한다");

    let mut checked = 0usize;
    for doc in &inner {
        // bottom-up WMF 만 대상 — y-flip 그룹이 있는 산출.
        if !doc.contains("scale(1,-1)") {
            continue;
        }
        for image in image_elements(doc) {
            assert!(
                !image.contains("height=\"-") && !image.contains("width=\"-"),
                "SVG 는 음수 width/height 를 오류로 규정한다 — 브라우저가 무시해 비트맵이 \
                 뒤집힌다: {image}"
            );
            // 목적 사각형이 뒤집힌 비트맵은 자기 transform 으로 되돌아와야 한다.
            assert!(
                image.contains("transform=\"") && image.contains("scale(1,-1)"),
                "y-flip 그룹 안 비트맵은 자체 역변환을 가져야 한다: {image}"
            );
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "bottom-up WMF 의 비트맵 <image> 를 찾지 못했다"
    );
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
