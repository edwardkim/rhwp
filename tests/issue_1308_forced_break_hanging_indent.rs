//! Issue #1308: Shift+Enter 강제 줄바꿈 뒤 TAC 수식 줄에 내어쓰기 적용.
//!
//! 재현 문서: `eq-002` 문단 0.4. 텍스트는 `"\t\n"` 뿐이고 TAC 수식 4개가
//! 붙어 있다. 한컴은 `\n` 뒤 수식 줄을 같은 문단의 두 번째 visual line 으로
//! 해석하므로, ParaShape `indent < 0` 의 내어쓰기 폭이 적용되어야 한다.
//!
//! 회귀: 빈 runs + TAC-only 렌더 경로가 일반 TextLine 과 달리
//! `effective_margin_left` 를 더하지 않아 두 번째 수식 줄이 본문 좌측에서 시작했다.

use std::fs;
use std::path::Path;

use serde_json::Value;

fn parse_document(rel: &str) -> rhwp::model::document::Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::parser::parse_document(&bytes).unwrap_or_else(|e| panic!("parse {}: {e:?}", rel))
}

fn load_wasm_document(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {e:?}", rel))
}

fn render_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {e:?}", rel));
    doc.render_page_svg_native(0)
        .unwrap_or_else(|e| panic!("render {}: {e:?}", rel))
}

fn cursor_rect(doc: &rhwp::wasm_api::HwpDocument, para: u32, offset: u32) -> Value {
    let json = doc
        .get_cursor_rect(0, para, offset)
        .unwrap_or_else(|e| panic!("cursor rect para={para} offset={offset}: {e:?}"));
    serde_json::from_str(&json.to_string())
        .unwrap_or_else(|e| panic!("parse cursor rect para={para} offset={offset}: {e}"))
}

fn assert_forced_break_cursor_enters_second_tac_line(rel: &str) {
    let doc = load_wasm_document(rel);

    let next = doc.navigate_next_editable_wasm(0, 4, 3, 1, "[]");
    let next: Value = serde_json::from_str(&next).expect("parse navigate_next_editable result");
    assert_eq!(
        next["charOffset"], 4,
        "첫 visual line 마지막 수식 뒤 오른쪽 이동은 두 번째 visual line 첫 수식 앞으로 가야 함: {rel}"
    );

    let first_line_end = cursor_rect(&doc, 4, 3);
    let second_line_start = cursor_rect(&doc, 4, 4);
    let first_y = first_line_end["y"].as_f64().expect("first y");
    let second_x = second_line_start["x"].as_f64().expect("second x");
    let second_y = second_line_start["y"].as_f64().expect("second y");

    assert!(
        second_y > first_y + 10.0,
        "강제 줄바꿈 뒤 첫 TAC 수식 앞 커서는 첫 줄이 아니라 두 번째 줄 y에 있어야 함: first={first_y:.1}, second={second_y:.1}, {rel}"
    );
    assert!(
        (165.0..170.0).contains(&second_x),
        "강제 줄바꿈 뒤 첫 TAC 수식 앞 커서 x도 내어쓰기 적용 위치여야 함: actual {second_x:.1}, {rel}"
    );
}

fn assert_paragraph_boundary_enters_before_leading_tac(rel: &str) {
    let doc = load_wasm_document(rel);
    let para0_len = doc
        .get_paragraph_length(0, 0)
        .unwrap_or_else(|e| panic!("paragraph length: {e:?}"));
    let next = doc.navigate_next_editable_wasm(0, 0, para0_len, 1, "[]");
    let next: Value = serde_json::from_str(&next).expect("parse navigate_next_editable result");

    assert_eq!(
        next["para"], 1,
        "문단 0.0 끝에서 오른쪽 이동하면 다음 문단 0.1로 진입해야 함: {rel}, {next}"
    );
    assert_eq!(
        next["charOffset"], 0,
        "수식으로 시작하는 다음 문단에 진입할 때 첫 수식 뒤가 아니라 수식 앞 offset=0에 멈춰야 함: {rel}, {next}"
    );
}

fn parse_translate(attrs: &str) -> Option<(f64, f64)> {
    let marker = "transform=\"translate(";
    let start = attrs.find(marker)? + marker.len();
    let rest = &attrs[start..];
    let end = rest.find(')')?;
    let inner = &rest[..end];
    let mut parts = inner
        .split(|ch: char| ch == ',' || ch.is_whitespace())
        .filter(|s| !s.is_empty());
    let x = parts.next()?.parse::<f64>().ok()?;
    let y = parts.next()?.parse::<f64>().ok()?;
    Some((x, y))
}

fn equation_group_translates(svg: &str) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = svg[search_from..].find("<g ") {
        let tag_start = search_from + rel;
        search_from = tag_start + 3;

        let Some(tag_close_rel) = svg[tag_start..].find('>') else {
            break;
        };
        let tag_close = tag_start + tag_close_rel;
        let attrs = &svg[tag_start..tag_close];
        let Some((x, y)) = parse_translate(attrs) else {
            continue;
        };
        let content_start = tag_close + 1;
        let Some(end_rel) = svg[content_start..].find("</g>") else {
            continue;
        };
        let content = svg[content_start..content_start + end_rel].to_string();
        out.push((x, y, content));
    }
    out
}

fn forced_break_fraction_quarter_x(svg: &str) -> f64 {
    equation_group_translates(svg)
        .into_iter()
        .filter(|(_, y, content)| {
            // The target is the second visual line equation `=-3 ^{1/4} f(n)`.
            // It contains denominator 4 and is rendered around y=252px.
            (245.0..260.0).contains(y)
                && content.contains(">4</text>")
                && content.contains(">f</text>")
                && content.contains(">-</text>")
        })
        .map(|(x, _, _)| x)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("forced-break 1/4 equation group")
}

fn trailing_fraction_quarter_x(svg: &str) -> f64 {
    equation_group_translates(svg)
        .into_iter()
        .filter(|(_, y, content)| {
            // The final visual line starts with `{1} over {4} f(n)` around y=300px.
            // A marker-synthesis regression moved this leading equation after comma/tab text.
            (295.0..305.0).contains(y)
                && content.contains(">1</text>")
                && content.contains(">4</text>")
                && content.contains(">f</text>")
        })
        .map(|(x, _, _)| x)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("final-line 1/4 equation group")
}

fn assert_para6_tac_order(rel: &str) {
    let doc = parse_document(rel);
    let para = &doc.sections[0].paragraphs[6];
    let composed = rhwp::renderer::composer::compose_paragraph(para);
    let positions: Vec<usize> = composed
        .tac_controls
        .iter()
        .map(|(pos, _, _)| *pos)
        .collect();

    assert_eq!(
        positions,
        vec![0, 0, 2, 2, 4],
        "TAC 수식/쉼표/고정탭/일반 글자 순서를 원본 char_offsets 그대로 보존해야 함: {rel}"
    );
}

#[test]
fn hwp_forced_break_tac_equation_line_uses_hanging_indent() {
    let svg = render_svg("samples/eq-002.hwp");
    let x = forced_break_fraction_quarter_x(&svg);

    assert!(
        (165.0..170.0).contains(&x),
        "Shift+Enter 뒤 TAC 수식 줄은 내어쓰기 적용 후 x≈166.7px 에서 시작해야 함 (actual {x:.2}px)"
    );
}

#[test]
fn hwpx_forced_break_tac_equation_line_uses_hanging_indent() {
    let svg = render_svg("samples/hwpx/eq-002.hwpx");
    let x = forced_break_fraction_quarter_x(&svg);

    assert!(
        (165.0..170.0).contains(&x),
        "HWPX Shift+Enter 뒤 TAC 수식 줄도 HWP 와 동일하게 x≈166.7px 여야 함 (actual {x:.2}px)"
    );
}

#[test]
fn hwp_tac_equation_text_and_fixed_tabs_keep_editor_order() {
    assert_para6_tac_order("samples/eq-002.hwp");

    let svg = render_svg("samples/eq-002.hwp");
    let x = trailing_fraction_quarter_x(&svg);
    assert!(
        (112.0..116.0).contains(&x),
        "마지막 visual line 의 첫 수식은 쉼표/고정탭 뒤가 아니라 줄 시작 x≈113.4px 에 있어야 함 (actual {x:.2}px)"
    );
}

#[test]
fn hwpx_tac_equation_text_and_fixed_tabs_keep_editor_order() {
    assert_para6_tac_order("samples/hwpx/eq-002.hwpx");

    let svg = render_svg("samples/hwpx/eq-002.hwpx");
    let x = trailing_fraction_quarter_x(&svg);
    assert!(
        (112.0..116.0).contains(&x),
        "HWPX 마지막 visual line 의 첫 수식도 줄 시작 x≈113.4px 에 있어야 함 (actual {x:.2}px)"
    );
}

#[test]
fn hwp_cursor_navigation_enters_forced_break_tac_line() {
    assert_forced_break_cursor_enters_second_tac_line("samples/eq-002.hwp");
}

#[test]
fn hwpx_cursor_navigation_enters_forced_break_tac_line() {
    assert_forced_break_cursor_enters_second_tac_line("samples/hwpx/eq-002.hwpx");
}

#[test]
fn hwp_paragraph_boundary_enters_before_leading_tac_equation() {
    assert_paragraph_boundary_enters_before_leading_tac("samples/eq-002.hwp");
}

#[test]
fn hwpx_paragraph_boundary_enters_before_leading_tac_equation() {
    assert_paragraph_boundary_enters_before_leading_tac("samples/hwpx/eq-002.hwpx");
}
