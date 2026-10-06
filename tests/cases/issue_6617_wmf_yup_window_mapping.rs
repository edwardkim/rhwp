//! [#6617] y-up 창 WMF 의 음수 높이 DIB 가 두 번 뒤집혀 viewBox 밖으로 나간다
//! (bitmap.hwp OLE 표현 메타파일 — 모든 표면에서 그림이 안 보임).
//!
//! WMF: `SetMapMode(8)`, `SetWindowExt(1152, -648)`, `SetWindowOrg(0, 0)`,
//! `DIBStretchBlt dest (x=0, y=0, w=1152, h=-648)`. 수정 전 산출은
//! `<svg viewBox="0 -648 1152 1296"><g transform="translate(0,1296) scale(1,-1)">
//! <image y="-648" … transform="translate(0,-648) scale(1,-1)"/></g></svg>` 로, 그림이
//! y ∈ [1296, 1944] 에 놓여 viewBox 밖이었다.
//!
//! 창 매핑(`Window::to_device`)이 y-up 축을 한 번 뒤집고, 블릿 목적 사각형은 두 모서리를
//! 장치 좌표로 옮겨 정규화한다(`DeviceContext::blit_dest_rect`). 그 결과 그림은 창 전체
//! `(0, 0, 1152, 648)` 을 뒤집기 없이 채운다. 한/글 2022 PDF 1쪽 잉크 bbox (322, 178)–(484, 281)
//! 과 rhwp SVG 래스터가 같은 자리다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use base64::Engine;
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/bitmap.hwp";
const PAGE: u32 = 0;

#[test]
fn issue_6617_yup_window_bitmap_fills_view_box_upright() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(PAGE).expect("page 1 svg");

    // 페이지 SVG 는 같은 OLE 표현 WMF 를 두 자리(본문 그림, 겹침 레이어)에 심는다 — 전부 같은 계약.
    let inner = embedded_svgs(&svg);
    assert!(
        !inner.is_empty(),
        "1쪽에 OLE 표현 WMF 하위 SVG 가 있어야 한다"
    );
    for doc in &inner {
        assert!(
            doc.contains("viewBox=\"0 0 1152 648\""),
            "viewBox 는 창 범위 (0, 0, |ext_x|, |ext_y|) 여야 한다: {}",
            root_tag(doc)
        );
        assert!(
            !doc.contains("<g transform=") && !doc.contains("scale(1,-1)"),
            "y-up 창의 뒤집기는 창 매핑이 끝내므로 뒤집기 그룹·transform 이 남으면 안 된다"
        );
        let images = image_elements(doc);
        assert_eq!(
            images.len(),
            1,
            "비트맵 <image> 는 하나여야 한다: {images:?}"
        );
        let image = &images[0];
        for attr in ["x=\"0\"", "y=\"0\"", "width=\"1152\"", "height=\"648\""] {
            assert!(
                image.contains(attr),
                "비트맵은 창 전체를 채워야 한다 — {attr} 없음: {image}"
            );
        }
        assert!(
            !image.contains("transform="),
            "y-up 창의 음수 높이 DIB 는 바로 선 그림이라 자체 뒤집기가 없어야 한다: {image}"
        );
    }
}

fn root_tag(doc: &str) -> &str {
    let start = doc.find("<svg").unwrap_or(0);
    let end = doc[start..]
        .find('>')
        .map(|e| start + e + 1)
        .unwrap_or(doc.len());
    &doc[start..end]
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
