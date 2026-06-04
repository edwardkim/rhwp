//! Issue #1156: 2단 문서에서 차트(OLE) 컨트롤의 단 이동 + 자리차지 텍스트
//! back-fill + 페이지 배경 워터마크 효과 회귀 가드.
//!
//! 재현 문서: `samples/hwpx/143E433F503322BD33.hwpx` (2단 + 차트 OLE + 표 + 배경 워터마크).
//!
//! 검증 항목:
//! 1. 차트(80mm, 비-TAC TopAndBottom)가 단0 끝을 넘어 단1 상단으로 이동.
//! 2. 자리차지(TopAndBottom) 속성이므로 단1 후속 텍스트가 차트 영역과 겹치지 않음
//!    (텍스트 첫 줄 y >= 차트 bottom).
//! 3. 페이지 배경 이미지(워터마크)는 반투명 합성 — SVG `<g opacity=...>` 적용
//!    (PR #1019 RealPic 톤 프리셋 사각지대 회귀 가드).

use std::fs;
use std::path::Path;

fn render_page_svg(rel: &str, page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    doc.render_page_svg(page).expect("render svg")
}

/// SVG `<text x=... y=...>` 중 x 가 [x_min, x_max), y >= y_min 인 노드들의 최소 y.
/// y_min 으로 머리말(body_area 위) 텍스트를 제외한다.
fn min_text_y_in_x_range(svg: &str, x_min: f64, x_max: f64, y_min: f64) -> Option<f64> {
    let mut min_y: Option<f64> = None;
    for chunk in svg.split("<text ").skip(1) {
        let tag_end = match chunk.find('>') {
            Some(p) => p,
            None => continue,
        };
        let tag = &chunk[..tag_end];
        // 차트/OLE placeholder 라벨 (fill #707070) 은 본문 텍스트가 아니므로 제외.
        if tag.contains("707070") {
            continue;
        }
        let text_body = &chunk[tag_end + 1..];
        if text_body.starts_with("OLE 개체") || text_body.starts_with("차트") {
            continue;
        }
        let x = attr_f64(tag, "x");
        let y = attr_f64(tag, "y");
        if let (Some(x), Some(y)) = (x, y) {
            if x >= x_min && x < x_max && y >= y_min {
                min_y = Some(min_y.map_or(y, |m: f64| m.min(y)));
            }
        }
    }
    min_y
}

fn attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", name);
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn attr_f64(tag: &str, name: &str) -> Option<f64> {
    attr_value(tag, name)?.parse().ok()
}

fn translate_y(tag: &str) -> Option<f64> {
    let transform = attr_value(tag, "transform")?;
    let inner = transform.strip_prefix("translate(")?.strip_suffix(')')?;
    inner
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .nth(1)?
        .parse()
        .ok()
}

/// 차트 placeholder (회색 점선 rect, fill #f0f0f0) 의 (y, height).
fn chart_placeholder_y_h(svg: &str) -> Option<(f64, f64)> {
    for chunk in svg.split("<rect ").skip(1) {
        let tag_end = chunk.find("/>").or_else(|| chunk.find('>'))?;
        let tag = &chunk[..tag_end];
        if tag.contains("f0f0f0") {
            let y = attr_f64(tag, "y")?;
            let h = attr_f64(tag, "height")?;
            return Some((y, h));
        }
    }
    None
}

/// 실제 OLE chart SVG `<g transform="translate(x y)">` 의 (y, height).
fn rendered_ole_chart_y_h(svg: &str) -> Option<(f64, f64)> {
    for chunk in svg.split("<g ").skip(1) {
        let tag_end = chunk.find('>')?;
        let tag = &chunk[..tag_end];
        if !tag.contains("hwp-ole-chart") {
            continue;
        }
        let y = translate_y(tag)?;
        let body = &chunk[tag_end + 1..];
        for rect_chunk in body.split("<rect ").skip(1) {
            let rect_tag_end = rect_chunk.find("/>").or_else(|| rect_chunk.find('>'))?;
            let rect_tag = &rect_chunk[..rect_tag_end];
            let h = attr_f64(rect_tag, "height")?;
            return Some((y, h));
        }
    }
    None
}

fn chart_y_h(svg: &str) -> Option<(f64, f64)> {
    rendered_ole_chart_y_h(svg).or_else(|| chart_placeholder_y_h(svg))
}

#[test]
fn chart_moves_to_second_column_and_text_does_not_overlap() {
    let svg = render_page_svg("samples/hwpx/143E433F503322BD33.hwpx", 0);

    // 차트 OLE 렌더 결과(또는 구버전 placeholder)의 단1 위치.
    let (chart_y, chart_h) = chart_y_h(&svg).expect("chart OLE bbox");
    let chart_bottom = chart_y + chart_h;

    // 차트가 단1 상단 근처 (body_area 상단 ~ 130px 이내) 에 배치.
    assert!(
        chart_y < 130.0,
        "차트가 단 상단에 배치되지 않음 (y={:.1})",
        chart_y
    );

    // 단1 (x >= 396) 영역 + body_area(y>=113) 텍스트 첫 줄이 차트 bottom 아래여야
    // (자리차지 겹침 없음). 머리말(y<113)은 제외.
    let col1_first_text_y =
        min_text_y_in_x_range(&svg, 396.0, 720.0, 113.0).expect("단1 영역 텍스트");
    assert!(
        col1_first_text_y >= chart_bottom - 1.0,
        "단1 텍스트(y={:.1})가 차트 영역(bottom={:.1})과 겹침 — 자리차지 미적용",
        col1_first_text_y,
        chart_bottom
    );
}

#[test]
fn page_background_watermark_has_opacity() {
    let svg = render_page_svg("samples/hwpx/143E433F503322BD33.hwpx", 0);
    // 페이지 배경 워터마크는 반투명 합성 — <g opacity="..."> 그룹으로 감쌈.
    // PR #1019(#975) RealPic 톤 프리셋 사각지대로 opacity 가 빠졌던 회귀 가드.
    assert!(
        svg.contains("<g opacity=\""),
        "페이지 배경 워터마크 opacity 그룹이 없음 (워터마크 효과 회귀)"
    );
}
