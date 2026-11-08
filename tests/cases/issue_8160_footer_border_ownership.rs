//! #8160: sample16 전용 회귀. 이전 등록: tests/issue_1139_inline_picture_duplicate.rs
use rhwp::wasm_api::HwpDocument;

fn svg_attr_f64(tag: &str, name: &str) -> Option<f64> {
    let pattern = format!("{name}=\"");
    let start = tag.find(&pattern)? + pattern.len();
    let end = tag[start..].find('"')?;
    tag[start..start + end].parse().ok()
}

fn sample16_page3_bottom_border_and_page_number(svg: &str) -> (f64, f64) {
    let mut bottom_lines = Vec::new();
    for tag in svg.match_indices("<line ").filter_map(|(start, _)| {
        let end = svg[start..].find('>')?;
        Some(&svg[start..start + end + 1])
    }) {
        let x1 = svg_attr_f64(tag, "x1").unwrap_or_default();
        let y1 = svg_attr_f64(tag, "y1").unwrap_or_default();
        let x2 = svg_attr_f64(tag, "x2").unwrap_or_default();
        let y2 = svg_attr_f64(tag, "y2").unwrap_or_default();
        if (y1 - y2).abs() < 0.1 && y1 > 1000.0 && (x2 - x1).abs() > 700.0 {
            bottom_lines.push(y1);
        }
    }

    let mut page_number_y = Vec::new();
    for (start, _) in svg.match_indices("<text ") {
        let Some(tag_end) = svg[start..].find('>') else {
            continue;
        };
        let tag = &svg[start..start + tag_end + 1];
        let text_start = start + tag_end + 1;
        let Some(text_end) = svg[text_start..].find("</text>") else {
            continue;
        };
        let text = &svg[text_start..text_start + text_end];
        let is_page_number = text == "-" || text.chars().all(|ch| ch.is_ascii_digit());
        if is_page_number {
            let y = svg_attr_f64(tag, "y").unwrap_or_default();
            if y > 1050.0 {
                page_number_y.push(y);
            }
        }
    }

    bottom_lines.sort_by(|a, b| a.partial_cmp(b).unwrap());
    page_number_y.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (
        bottom_lines
            .last()
            .copied()
            .expect("page bottom border line"),
        page_number_y
            .last()
            .copied()
            .expect("bottom page number baseline"),
    )
}

#[test]
fn issue_8160_sample16_page3_page_number_stays_below_bottom_border() {
    let bytes = std::fs::read("samples/hwp3-sample16-hwp5.hwp").expect("sample16");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse sample16");
    let svg = doc.render_page_svg_native(2).expect("page 3 svg");
    let (bottom_line, page_number_y) = sample16_page3_bottom_border_and_page_number(&svg);

    let gap = page_number_y - bottom_line;
    assert!(
        (bottom_line - 1066.86).abs() < 0.5,
        "sample16 page-basis bottom border should not include extra double-line outset: {bottom_line}"
    );
    assert!(
        gap > 10.0,
        "page number should stay visibly below the bottom border: gap={gap}, border={bottom_line}, page_number={page_number_y}"
    );
}
