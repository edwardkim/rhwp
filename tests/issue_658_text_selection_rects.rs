use std::path::Path;

use rhwp::wasm_api::HwpDocument;

#[derive(Debug)]
struct Rect {
    page_index: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn json_number(json: &str, key: &str) -> f64 {
    let pattern = format!("\"{}\":", key);
    let start = json.find(&pattern).expect("json key not found") + pattern.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse::<f64>().expect("json number parse")
}

fn parse_rects(json: &str) -> Vec<Rect> {
    let trimmed = json.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed.is_empty() {
        return Vec::new();
    }

    trimmed
        .split("},{")
        .map(|part| {
            let item = part.trim_start_matches('{').trim_end_matches('}');
            Rect {
                page_index: json_number(item, "pageIndex") as u32,
                x: json_number(item, "x"),
                y: json_number(item, "y"),
                width: json_number(item, "width"),
                height: json_number(item, "height"),
            }
        })
        .collect()
}

fn load_exam_social() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_social.hwp");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    HwpDocument::from_bytes(&bytes).expect("parse exam_social.hwp")
}

fn assert_rects_inside_page(rects: &[Rect], page_width: f64) {
    assert!(!rects.is_empty(), "selection rects should not be empty");
    for rect in rects {
        assert!(rect.page_index < 4, "unexpected page index: {rect:?}");
        assert!(rect.x >= -0.5, "rect starts before page: {rect:?}");
        assert!(rect.width > 0.0, "rect width should be positive: {rect:?}");
        assert!(
            rect.height > 0.0,
            "rect height should be positive: {rect:?}"
        );
        assert!(
            rect.x + rect.width <= page_width + 0.5,
            "rect overflows page width {page_width}: {rect:?}"
        );
    }
}

#[test]
fn issue_658_exam_social_data_cell_selection_rects_do_not_overflow_page() {
    let doc = load_exam_social();
    let page_info = doc.get_page_info(1).expect("page info");
    let page_width = json_number(&page_info, "width");

    // 영상의 페이지 2 오른쪽 자료 박스: section=1, parent_para=16, control=0, cell=0.
    let rects_json = doc
        .get_selection_rects_in_cell(1, 16, 0, 0, 0, 0, 6, 469)
        .expect("cell selection rects");
    let rects = parse_rects(&rects_json);

    assert_eq!(rects.len(), 18, "rects json: {rects_json}");
    assert_rects_inside_page(&rects, page_width);

    let first_para_json = doc
        .get_selection_rects_in_cell(1, 16, 0, 0, 0, 0, 0, 209)
        .expect("first cell paragraph selection rects");
    let first_para_rects = parse_rects(&first_para_json);
    assert_eq!(first_para_rects.len(), 3, "rects json: {first_para_json}");
    assert!(
        first_para_rects[1].y > first_para_rects[0].y + 0.5
            && first_para_rects[2].y > first_para_rects[1].y + 0.5,
        "line-start offsets should use the next line run, not the previous line end: {first_para_rects:?}"
    );
    assert_rects_inside_page(&first_para_rects, page_width);
}

#[test]
fn issue_658_exam_social_body_multiline_selection_uses_next_line_start() {
    let doc = load_exam_social();
    let page_info = doc.get_page_info(1).expect("page info");
    let page_width = json_number(&page_info, "width");

    let rects_json = doc
        .get_selection_rects(1, 15, 0, 15, 66)
        .expect("body selection rects");
    let rects = parse_rects(&rects_json);

    assert_eq!(rects.len(), 2, "rects json: {rects_json}");
    assert!(
        rects[1].y > rects[0].y + 0.5,
        "second line should not reuse the first line end cursor: {rects:?}"
    );
    assert_rects_inside_page(&rects, page_width);
}
