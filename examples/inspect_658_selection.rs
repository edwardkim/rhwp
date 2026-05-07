use std::env;
use std::fs;

use rhwp::wasm_api::HwpDocument;

#[derive(Debug)]
struct Rect {
    page_index: u32,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

fn json_number(json: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !matches!(c, '-' | '+' | '.' | '0'..='9' | 'e' | 'E'))
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn json_u32(json: &str, key: &str) -> Option<u32> {
    json_number(json, key).map(|n| n as u32)
}

fn parse_rects(json: &str) -> Vec<Rect> {
    let trimmed = json.trim().trim_start_matches('[').trim_end_matches(']');
    if trimmed.is_empty() {
        return Vec::new();
    }

    trimmed
        .split("},{")
        .filter_map(|part| {
            let item = part.trim_start_matches('{').trim_end_matches('}');
            Some(Rect {
                page_index: json_u32(item, "pageIndex")?,
                x: json_number(item, "x")?,
                y: json_number(item, "y")?,
                width: json_number(item, "width")?,
                height: json_number(item, "height")?,
            })
        })
        .collect()
}

fn print_rects(label: &str, json: &str, page_width: f64) {
    let rects = parse_rects(json);
    let overflowing: Vec<_> = rects
        .iter()
        .filter(|r| r.x < -0.5 || r.x + r.width > page_width + 0.5)
        .collect();

    println!("--- {label} ---");
    println!("rect_count={}", rects.len());
    for (idx, rect) in rects.iter().enumerate() {
        let right = rect.x + rect.width;
        let mark = if right > page_width + 0.5 { " OVERFLOW" } else { "" };
        println!(
            "#{idx:02} p={} x={:.1} y={:.1} w={:.1} h={:.1} right={:.1}{mark}",
            rect.page_index, rect.x, rect.y, rect.width, rect.height, right
        );
    }
    println!("overflow_count={}", overflowing.len());
    println!();
}

fn main() {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "samples/exam_social.hwp".to_string());
    let bytes = fs::read(&path).expect("sample read failed");
    let doc = HwpDocument::from_bytes(&bytes).expect("document load failed");

    println!("document={path}");
    println!("page_count={}", doc.page_count());

    let page_info = doc.get_page_info(1).expect("page info failed");
    let page_width = json_number(&page_info, "width").expect("page width missing");
    println!("page_info={page_info}");
    println!();

    // 영상(2026-05-07 13:28:24)에서 드래그한 페이지 2 오른쪽 자료 박스 부근.
    // section=1, parent_para=16, control=0, cell=0 은 dump-pages/dump로 확인한 1x1 자료 표이다.
    let cases = [
        ("body question s1 p15 0..66", doc.get_selection_rects(1, 15, 0, 15, 66)),
        (
            "data table p16 c0 paragraph 0",
            doc.get_selection_rects_in_cell(1, 16, 0, 0, 0, 0, 0, 209),
        ),
        (
            "data table p16 c0 paragraphs 0..6",
            doc.get_selection_rects_in_cell(1, 16, 0, 0, 0, 0, 6, 469),
        ),
        (
            "data table p16 c0 lower dialog",
            doc.get_selection_rects_in_cell(1, 16, 0, 0, 5, 0, 6, 180),
        ),
    ];

    for (label, result) in cases {
        print_rects(label, &result.expect("selection rects failed"), page_width);
    }

    println!("--- hit-test samples ---");
    for (x, y) in [
        (560.0, 365.0),
        (650.0, 365.0),
        (820.0, 365.0),
        (900.0, 365.0),
        (610.0, 575.0),
        (900.0, 575.0),
    ] {
        match doc.hit_test(1, x, y) {
            Ok(hit) => println!("x={x:.1} y={y:.1} -> {hit}"),
            Err(err) => println!("x={x:.1} y={y:.1} -> error: {err:?}"),
        }
    }
}
