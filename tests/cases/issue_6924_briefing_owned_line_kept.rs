//! [#6924] 조각이 소유한 브리핑 범례가 실제 SVG에 정확히 한 번 방출된다.
//! 문단 간격이나 한컴과의 페이지 전체 픽셀 일치는 이 PR의 계약이 아니다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

fn ordered_text(svg: &str) -> String {
    let mut glyphs = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(head_end) = chunk.find('>') else {
            continue;
        };
        let (head, rest) = chunk.split_at(head_end);
        let Some(body_end) = rest.find("</text>") else {
            continue;
        };
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let start = head.find(&key)? + key.len();
            let end = head[start..].find('"')? + start;
            head[start..end].parse().ok()
        };
        if let (Some(x), Some(y)) = (attr("x"), attr("y")) {
            glyphs.push(((y * 10.0) as i64, (x * 10.0) as i64, &rest[1..body_end]));
        }
    }
    glyphs.sort();
    glyphs
        .into_iter()
        .flat_map(|(_, _, text)| text.chars())
        .filter(|c| !c.is_whitespace())
        .collect()
}

#[test]
fn briefing_owned_bottom_legend_is_painted_once_on_its_page() {
    let input =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue6924/148751598-briefing.hwp");
    let bytes = std::fs::read(&input).expect("required briefing fixture");
    let document = DocumentCore::from_bytes(&bytes).expect("parse briefing");
    let legend = "장관참석행사는★로,차관참석행사는☆로표기";
    for page in 0..document.page_count() {
        let svg = document
            .render_page_svg_native(u32::try_from(page).expect("page index"))
            .expect("render actual SVG, not the pre-paint tree");
        let count = ordered_text(&svg).matches(legend).count();
        assert_eq!(
            count,
            usize::from(page == 0),
            "#6924: 범례는 첫 페이지에 한 번만 있어야 한다: page={}",
            page + 1
        );
    }
}
