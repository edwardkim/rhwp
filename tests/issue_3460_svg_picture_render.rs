//! Issue #3460: 임베드 SVG picture(`<hp:pic>` + `image/svg` BinData)가 빈 공간으로 렌더.
//!
//! 재현 문서: `samples/issue3460/svg_picture_repro.hwpx`
//! - 1쪽 표지 배너(`binaryItemIDRef="BIN0001"`, 본문 인라인 그림)
//! - 1~3쪽 러닝 헤더 밴드(`binaryItemIDRef="BINHDR"`, 머리말 `applyPageType="BOTH"`)
//!
//! 두 결함이 겹쳐 있었다.
//!
//! 1. **MIME 오판** — 네이티브 판별기(`image_resolver::detect_image_mime_type`)에 SVG 분기가
//!    없어 data URI 가 `application/octet-stream` 으로 나갔다. 요소는 있는데 브라우저·rsvg 가
//!    그리지 않으니 빈 공간으로 보인다.
//! 2. **비숫자 `binaryItemIDRef` 유실** — 섹션 파서는 참조에서 숫자만 뽑아 `bin_data_id` 로
//!    쓰는데 `BINHDR` 은 숫자가 없어 0 이 되고, BinData(위치 기준 id) 매칭에 실패해 머리말
//!    그림이 통째로 사라졌다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue3460/svg_picture_repro.hwpx";

fn render_page_svg(page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse repro.hwpx");
    doc.render_page_svg(page).expect("render svg")
}

fn svg_image_data_uris(svg: &str) -> Vec<String> {
    let mut uris = Vec::new();
    let mut rest = svg;
    while let Some(pos) = rest.find("href=\"data:") {
        let tail = &rest[pos + "href=\"".len()..];
        let end = tail.find(';').unwrap_or(tail.len());
        uris.push(tail[..end].to_string());
        rest = tail;
    }
    uris
}

/// 표지 배너(본문 인라인 SVG 그림)는 `image/svg+xml` data URI 로 방출돼야 한다.
#[test]
fn cover_svg_picture_is_emitted_with_svg_mime() {
    let svg = render_page_svg(0);
    let uris = svg_image_data_uris(&svg);
    assert!(
        uris.iter().any(|uri| uri == "data:image/svg+xml"),
        "표지 SVG 배너가 image/svg+xml 로 방출되어야 함, got={:?}",
        uris
    );
}

/// 머리말 밴드는 `binaryItemIDRef="BINHDR"` 처럼 숫자가 없는 ID 를 쓴다. 모든 쪽에서
/// 그림이 살아 있어야 한다 (종전에는 bin_data_id=0 으로 떨어져 전 쪽에서 사라졌다).
#[test]
fn header_svg_band_survives_non_numeric_bin_item_id() {
    for page in 0..3u32 {
        let svg = render_page_svg(page);
        let uris = svg_image_data_uris(&svg);
        assert!(
            uris.iter().any(|uri| uri == "data:image/svg+xml"),
            "{}쪽에 SVG 그림이 없음 (머리말 밴드 유실), got={:?}",
            page + 1,
            uris
        );
    }
}
