//! #1105: K-water RFP의 쪽 나눔·표·꼬리말 회귀. sample16 검사는 #8160으로 통합.

use std::fs;
use std::path::Path;

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn svg_text_rows(svg: &str) -> Vec<(f64, String)> {
    let mut rows: Vec<(f64, String)> = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("<text ") {
        let tag_start = search_from + rel;
        search_from = tag_start + 6;
        let Some(close_rel) = svg[tag_start..].find('>') else {
            break;
        };
        let attrs = &svg[tag_start..tag_start + close_rel];
        let content_start = tag_start + close_rel + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else {
            break;
        };
        let Some(y) = attr_f64(attrs, "y") else {
            continue;
        };
        let text = decode_svg_text(&svg[content_start..content_start + end_rel]);
        let key = (y * 10.0).round() / 10.0;
        if let Some((_, row_text)) = rows
            .iter_mut()
            .find(|(row_y, _)| (*row_y - key).abs() < 0.01)
        {
            row_text.push_str(&text);
        } else {
            rows.push((key, text));
        }
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    rows
}

fn attr_f64(attrs: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = attrs.find(&needle)? + needle.len();
    let end = attrs[start..].find('"')?;
    attrs[start..start + end].parse().ok()
}

fn decode_svg_text(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
}

#[test]
fn task1105_k_water_rfp_2024_page_count_matches_hancom_pdf() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");
    assert_eq!(doc.page_count(), 27);
}

#[test]
fn task1105_k_water_rfp_2024_first_rowspan_table_keeps_line_reset_split() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");

    let page5 = doc.dump_page_items(Some(4));
    assert!(
        page5.contains("PartialTable   pi=52 ci=0  rows=0..4"),
        "the first large rowspan table must split inside its last row:\n{page5}"
    );
    assert!(
        page5.contains("end_cut=[3, 4, 2, 4, 4, 2, 20]"),
        "the first large rowspan table must cut before the orphan `유의사항` line on page 5:\n{page5}"
    );

    let svg = doc
        .render_page_svg_native(4)
        .expect("render k-water-rfp-2024 page 5");
    let rows = svg_text_rows(&svg);
    assert!(
        rows.iter()
            .any(|(_, row)| row.contains("규격") && row.contains("A4") && row.contains("가로방향")),
        "page 5 must still end near the `A4 가로방향` row:\n{rows:?}"
    );
    assert!(
        rows.iter().all(|(_, row)| !row.contains("유의사항")),
        "page 5 must not include the next `유의사항` row from the same reset paragraph:\n{rows:?}"
    );
}

#[test]
fn task1105_k_water_rfp_2024_cover_hides_first_page_footer() {
    let doc = load_doc("samples/k-water-rfp-2024.hwp");
    let svg = doc
        .render_page_svg_native(0)
        .expect("render k-water-rfp-2024 page 1");

    assert!(
        !svg.contains(
            r##"<line x1="80" y1="1034.8666666666668" x2="713.96" y2="1034.8666666666668" stroke="#787878" stroke-width="1.5"/>"##
        ),
        "first page footer table line must be hidden by SectionDef first-page footer hide"
    );
}
