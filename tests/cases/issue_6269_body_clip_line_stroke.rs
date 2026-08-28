//! [Issue #6269] 본문 좌단에 붙은 세로 테두리선의 획 절반이 body clip 에 잘려 거의
//! 안 보인다 (156739836 2·3쪽).
//!
//! 선의 **잉크는 bbox 밖으로 나간다.** bbox 는 `[경로, 경로+획]` 인데 백엔드는
//! `line.x1/y1` 을 경로로 삼아 획을 **중심 정렬**로 칠하므로 실제 잉크는
//! `[경로-획/2, 경로+획/2]` 다. 경계에 붙은 선은 그 바깥 절반이 잘린다.
//!
//! ```text
//! body clip      x = 75.5867            (본문 좌단)
//! 왼쪽 세로선    x = 75.5867  획 1.5    → 잉크 74.84..76.34, 왼쪽 0.75px 가 clip 밖
//! 오른쪽 세로선  x = 703.03   획 1.5    → clip 안쪽이라 온전
//! ```
//!
//! 헤드리스 Chrome 잉크 실측(scale 2): 왼쪽 **102** vs 오른쪽 204 — 같은 굵기인데
//! 정확히 절반이다. 수정 후 왼쪽 **188** 로, 이슈가 실측한 rhwp PDF 값(94×2=188)과
//! 같아진다.
//!
//! **`Body::clip_rect` 자체는 건드리지 않는다** — 그 값은 여러 잠금 테스트가 좌표
//! 기준점으로 쓴다(`issue_3820_*`, `issue_2007_*`). 완화는 clip 을 **방출하는
//! 지점**에서만 하므로, 이 잠금도 방출된 SVG 의 `clipPath` 를 본다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue6269/156739836_public_sector_jobs_stats.hwpx";
/// 결함이 나타나는 쪽(0-based). 3쪽도 같은 틀을 쓴다.
const PAGE: u32 = 1;

#[test]
fn issue_6269_painted_body_clip_contains_line_ink() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).expect("read sample");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse sample");
    let svg = doc.render_page_svg_native(PAGE).expect("render page 2 svg");

    let clip = body_clip_rect(&svg).expect("body-clip rect");
    let lines = vertical_hairlines(&svg);
    assert!(!lines.is_empty(), "2쪽에 세로 테두리 선이 있어야 한다");

    for (x, stroke) in lines {
        let half = stroke / 2.0;
        assert!(
            x - half >= clip.0 - 0.01,
            "세로선 잉크 왼끝({:.3})이 방출된 body clip({:.3}) 밖이라 획이 잘린다",
            x - half,
            clip.0,
        );
        assert!(
            x + half <= clip.1 + 0.01,
            "세로선 잉크 오른끝({:.3})이 방출된 body clip({:.3}) 밖이라 획이 잘린다",
            x + half,
            clip.1,
        );
    }
}

/// 방출된 `body-clip-*` 사각형의 (좌단, 우단).
fn body_clip_rect(svg: &str) -> Option<(f64, f64)> {
    let at = svg.find("<clipPath id=\"body-clip-")?;
    let rest = &svg[at..];
    let end = rest.find("/></clipPath>")?;
    let rect = &rest[..end];
    let x = attr(rect, " x=\"")?;
    let w = attr(rect, " width=\"")?;
    Some((x, x + w))
}

/// 세로 hairline 들의 (x, 획 두께).
fn vertical_hairlines(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find("/>") else {
            continue;
        };
        let tag = &chunk[..end];
        let (Some(x1), Some(x2), Some(w)) = (
            attr(tag, "x1=\""),
            attr(tag, "x2=\""),
            attr(tag, " stroke-width=\""),
        ) else {
            continue;
        };
        // 세로선만 — 가로선은 이 잠금의 대상이 아니다(같은 규칙이 y 축에도 걸린다).
        if (x1 - x2).abs() <= 0.01 && w > 0.0 {
            out.push((x1, w));
        }
    }
    out
}

fn attr(tag: &str, key: &str) -> Option<f64> {
    let at = tag.find(key)? + key.len();
    let rest = &tag[at..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
