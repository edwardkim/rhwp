//! Issue #1105: HWP3를 HWP5로 변환한 문서가 sample16 21쪽 주변의 한컴 쪽 나눔을 보존한다.

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
fn task1105_hwp3_sample16_page23_square_bullet_matches_hancom() {
    let doc = load_doc("samples/hwp3-sample16.hwp");
    assert_eq!(doc.page_count(), 64);

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must start page 23 in the HWP3 source:\n{page23}"
    );
    assert!(
        page23.contains("\u{F03C5} 계약상대자는"),
        "HWP3 private bullet 0x3366 must be preserved as U+F03C5 before rendering:\n{page23}"
    );
    assert!(
        page23.contains("PartialParagraph  pi=460  lines=0..3"),
        "HWP3 source must honor the line-level page break inside paragraph 460:\n{page23}"
    );
    assert!(
        !page23.contains("○ 계약상대자는"),
        "HWP3 private bullet 0x3366 must not be lowered to a white circle:\n{page23}"
    );

    let svg = doc
        .render_page_svg_native(22)
        .expect("render hwp3-sample16 page 23");
    assert!(
        svg.contains("□"),
        "rendered HWP3 page 23 must display the Hancom square bullet"
    );
    assert!(
        !svg.contains("○ 계약상대자는"),
        "rendered HWP3 page 23 must not display the old white-circle bullet"
    );
}

fn assert_sample16_hwp5_business_selection_starts_next_page(rel_path: &str) {
    let doc = load_doc(rel_path);
    assert_eq!(doc.page_count(), 64, "{rel_path}");

    let page4 = doc.dump_page_items(Some(3));
    assert!(
        page4.contains("FullParagraph  pi=118"),
        "WAN line must stay on page 4 for {rel_path}:\n{page4}"
    );

    let page5 = doc.dump_page_items(Some(4));
    assert!(
        !page5.contains("pi=118"),
        "page 5 must start at the information-system importance paragraph, not the WAN line, for {rel_path}:\n{page5}"
    );
    assert!(
        page5.contains("FullParagraph  pi=119"),
        "information-system importance paragraph must start page 5 for {rel_path}:\n{page5}"
    );
    assert!(
        page5.contains("FullParagraph  pi=140"),
        "joint supply paragraph must close page 5 for {rel_path}:\n{page5}"
    );
    assert!(
        !page5.contains("pi=141"),
        "business selection heading must not remain on page 5 for {rel_path}:\n{page5}"
    );

    let page6 = doc.dump_page_items(Some(5));
    assert!(
        page6.contains("FullParagraph  pi=141"),
        "business selection heading must start page 6 for {rel_path}:\n{page6}"
    );
    assert!(
        page6.contains("FullParagraph  pi=142"),
        "business selection body must follow the heading for {rel_path}:\n{page6}"
    );
    assert!(
        page6.contains("FullParagraph  pi=144"),
        "successful bidder heading must remain on page 6 for {rel_path}:\n{page6}"
    );
}

fn assert_sample16_hwp5_server_requirements_page_matches_hancom(rel_path: &str) {
    let doc = load_doc(rel_path);
    assert_eq!(doc.page_count(), 64, "{rel_path}");

    let page22 = doc.dump_page_items(Some(21));
    assert!(
        page22.contains("FullParagraph  pi=449"),
        "RDBMS paragraph must close page 22 for {rel_path}:\n{page22}"
    );
    assert!(
        !page22.contains("pi=450"),
        "firewall paragraph must not leak into page 22 for {rel_path}:\n{page22}"
    );

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must start page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        page23.contains("FullParagraph  pi=451"),
        "hardware/software paragraph must follow on page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        page23.contains("PartialParagraph  pi=460  lines=0..3"),
        "page 23 must contain the first three lines of the integrated DB cluster paragraph for {rel_path}:\n{page23}"
    );
    assert!(
        !page23.contains("FullParagraph  pi=460"),
        "integrated DB cluster paragraph must not remain whole on page 23 for {rel_path}:\n{page23}"
    );
    assert!(
        !page23.contains("pi=461"),
        "next target-system paragraph must not remain on page 23 for {rel_path}:\n{page23}"
    );
}

#[test]
fn task1105_sample16_hwp5_2018_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2018.hwp");
}

#[test]
fn task1105_sample16_hwp5_2018_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2018.hwp",
    );
}

#[test]
fn task1105_sample16_hwp5_2024_business_selection_break_matches_hancom() {
    assert_sample16_hwp5_business_selection_starts_next_page("samples/hwp3-sample16-hwp5-2024.hwp");
}

#[test]
fn task1105_sample16_hwp5_2024_server_requirements_page_matches_hancom() {
    assert_sample16_hwp5_server_requirements_page_matches_hancom(
        "samples/hwp3-sample16-hwp5-2024.hwp",
    );
}

#[test]
fn task1105_sample16_hwp5_2024_clean_linesegs_reflow_is_noop() {
    let mut doc = load_doc("samples/hwp3-sample16-hwp5-2024.hwp");
    assert_eq!(doc.page_count(), 64);
    assert!(
        doc.get_validation_warnings()
            .contains(r#""count":0,"summary":{}"#),
        "2024 fixture must not request lineseg reflow"
    );

    let reflowed = doc.reflow_linesegs();
    assert_eq!(
        reflowed, 0,
        "clean HWP5 linesegs must not be regenerated by on-demand reflow"
    );
    assert_eq!(doc.page_count(), 64);

    let page23 = doc.dump_page_items(Some(22));
    assert!(
        page23.contains("FullParagraph  pi=450"),
        "firewall paragraph must still start page 23 after no-op reflow:\n{page23}"
    );
    assert!(
        page23.contains("PartialParagraph  pi=460  lines=0..3"),
        "integrated DB cluster paragraph split must survive no-op reflow:\n{page23}"
    );
    assert!(
        !page23.contains("pi=461"),
        "target-system paragraph must not move into page 23 after no-op reflow:\n{page23}"
    );
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
