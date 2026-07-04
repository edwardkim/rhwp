//! Issue #1116: HWP3→HWP5 sample16 목차 leader 및 p3 문단 vpos 정합 가드.

use std::fs;
use std::path::Path;

fn render_svg(rel_path: &str, page_idx: u32) -> String {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"));
    doc.render_page_svg_native(page_idx)
        .unwrap_or_else(|e| panic!("render {rel_path} page {page_idx}: {e:?}"))
}

fn load_doc(rel_path: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel_path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel_path}: {e:?}"))
}

fn extract_dotted_horizontal_lines(svg: &str) -> Vec<(f64, f64, f64)> {
    let mut lines = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("<line ") {
        let start = search_from + rel;
        search_from = start + 6;
        let Some(close_rel) = svg[start..].find("/>") else {
            break;
        };
        let attrs = &svg[start..start + close_rel];
        if !attrs.contains("stroke-dasharray=\"0.1 3\"") {
            continue;
        }
        let Some(x1) = attr_f64(attrs, "x1") else {
            continue;
        };
        let Some(x2) = attr_f64(attrs, "x2") else {
            continue;
        };
        let Some(y1) = attr_f64(attrs, "y1") else {
            continue;
        };
        let Some(y2) = attr_f64(attrs, "y2") else {
            continue;
        };
        if (y1 - y2).abs() < 0.1 {
            lines.push((x1, x2, y1));
        }
    }
    lines
}

fn extract_text_positions(svg: &str, wanted: &str) -> Vec<(f64, f64)> {
    let mut positions = Vec::new();
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
        let text = &svg[content_start..content_start + end_rel];
        if text == wanted {
            if let (Some(x), Some(y)) = (attr_f64(attrs, "x"), attr_f64(attrs, "y")) {
                positions.push((x, y));
            }
        }
    }
    positions
}

fn toc_page_number_right_edges(svg: &str) -> Vec<f64> {
    let mut rows: std::collections::BTreeMap<i32, Vec<(f64, f64, String)>> =
        std::collections::BTreeMap::new();
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
        let text = &svg[content_start..content_start + end_rel];
        if text.chars().all(|ch| ch.is_ascii_digit()) {
            let Some(x) = attr_f64(attrs, "x") else {
                continue;
            };
            let Some(y) = attr_f64(attrs, "y") else {
                continue;
            };
            let text_length = attr_f64(attrs, "textLength").unwrap_or(0.0);
            if x > 600.0 {
                rows.entry((y * 10.0).round() as i32).or_default().push((
                    x,
                    x + text_length,
                    text.to_string(),
                ));
            }
        }
    }

    rows.into_values()
        .filter_map(|mut digits| {
            digits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            digits.last().map(|(_, right, _)| *right)
        })
        .collect()
}

fn extract_text_attrs(svg: &str, wanted: &str) -> Vec<String> {
    let mut attrs_list = Vec::new();
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
        let text = &svg[content_start..content_start + end_rel];
        if text == wanted {
            attrs_list.push(attrs.to_string());
        }
    }
    attrs_list
}

fn attr_f64(attrs: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = attrs.find(&needle)? + needle.len();
    let end = attrs[start..].find('"')?;
    attrs[start..start + end].parse().ok()
}

#[test]
fn sample16_hwp5_toc_leaders_stop_before_page_numbers() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    let svg = doc.render_page_svg_native(1).expect("render p2");
    let leaders = extract_dotted_horizontal_lines(&svg);
    assert!(
        leaders.len() >= 20,
        "sample16 p2 목차 leader 점선을 충분히 찾아야 함: {}",
        leaders.len()
    );

    let max_x2 = leaders
        .iter()
        .map(|(_, x2, _)| *x2)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        max_x2 < 650.0,
        "목차 leader가 페이지 번호 뒤로 과도하게 연장됨: max x2={max_x2:.1}. \
         정상은 페이지 번호 시작 직전(x≈610~631)에서 멈춰야 함"
    );
}

#[test]
fn sample16_hwp5_toc_page_numbers_share_right_edge() {
    let svg = render_svg("samples/hwp3-sample16-hwp5.hwp", 1);
    let edges = toc_page_number_right_edges(&svg);
    assert!(
        edges.len() >= 20,
        "목차 페이지 번호를 충분히 찾아야 함: {edges:?}"
    );
    let min = edges.iter().copied().fold(f64::INFINITY, f64::min);
    let max = edges.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max - min <= 1.5,
        "목차 페이지 번호 오른쪽 끝이 한컴처럼 같은 기준선에 정렬되어야 함: min={min:.2}, max={max:.2}, edges={edges:?}"
    );
}

#[test]
fn sample16_hwp3_toc_page_numbers_share_right_edge() {
    let svg = render_svg("samples/hwp3-sample16.hwp", 1);
    let edges = toc_page_number_right_edges(&svg);
    assert!(
        edges.len() >= 25,
        "원본 HWP3 목차 페이지 번호를 충분히 찾아야 함: {edges:?}"
    );
    let min = edges.iter().copied().fold(f64::INFINITY, f64::min);
    let max = edges.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max - min <= 1.5,
        "원본 HWP3 목차 페이지 번호 오른쪽 끝이 한컴처럼 같은 기준선에 정렬되어야 함: min={min:.2}, max={max:.2}, edges={edges:?}"
    );
}

#[test]
fn sample16_hwp5_page3_svg_latin_glyphs_pin_browser_width() {
    let svg = render_svg("samples/hwp3-sample16-hwp5.hwp", 2);
    let latin_c_attrs = extract_text_attrs(&svg, "C");
    assert!(
        latin_c_attrs.iter().any(|attrs| {
            attrs.contains("textLength=\"") && attrs.contains("lengthAdjust=\"spacingAndGlyphs\"")
        }),
        "p3 라틴 글자는 브라우저 폰트 폭 차이로 겹치지 않도록 textLength로 advance를 고정해야 함: {latin_c_attrs:?}"
    );

    let digit_attrs = extract_text_attrs(&svg, "1");
    assert!(
        digit_attrs.iter().any(|attrs| {
            attrs.contains("textLength=\"") && attrs.contains("lengthAdjust=\"spacingAndGlyphs\"")
        }),
        "목차/본문 숫자는 페이지 번호 오른쪽 정렬이 흐트러지지 않도록 textLength로 advance를 고정해야 함: {digit_attrs:?}"
    );

    let korean_attrs = extract_text_attrs(&svg, "사");
    assert!(
        korean_attrs
            .iter()
            .all(|attrs| !attrs.contains("textLength=\"")),
        "한글 글자 폭은 기존 좌표/폰트 렌더링을 유지해야 함: {korean_attrs:?}"
    );
}

fn assert_page3_latin_poppy_resolves_to_palatino(rel_path: &str) {
    let svg = render_svg(rel_path, 2);
    let latin_c_attrs = extract_text_attrs(&svg, "C");
    assert!(
        latin_c_attrs
            .iter()
            .any(|attrs| attrs.contains("font-family=\"Palatino Linotype,")),
        "{rel_path} p3 Latin glyphs must resolve HCI Poppy to Palatino Linotype: {latin_c_attrs:?}"
    );
    assert!(
        latin_c_attrs
            .iter()
            .all(|attrs| !attrs.contains("font-family=\"HCI Poppy,")),
        "{rel_path} p3 Latin glyphs must not fall back through unresolved HCI Poppy: {latin_c_attrs:?}"
    );
}

#[test]
fn sample16_hwp5_2022_page3_latin_font_matches_legacy_hancom_mapping() {
    assert_page3_latin_poppy_resolves_to_palatino("samples/hwp3-sample16-hwp5-2022.hwp");
}

#[test]
fn sample16_hwp3_page3_latin_font_matches_legacy_hancom_mapping() {
    assert_page3_latin_poppy_resolves_to_palatino("samples/hwp3-sample16.hwp");
}

#[test]
fn sample16_hwp5_page3_heading_positions_follow_lineseg_vpos() {
    let svg = render_svg("samples/hwp3-sample16-hwp5.hwp", 2);
    let twos = extract_text_positions(&svg, "2");
    let threes = extract_text_positions(&svg, "3");

    let heading2 = twos
        .iter()
        .find(|(x, y)| (*x - 83.36).abs() < 1.0 && (*y - 337.2).abs() < 2.0)
        .copied();
    assert!(
        heading2.is_some(),
        "p3 `2. 추진방향` heading digit must match the 3mm-grid Hancom PDF y≈337.2: {twos:?}"
    );

    let heading3 = threes
        .iter()
        .find(|(x, y)| (*x - 83.36).abs() < 1.0 && (*y - 749.3).abs() < 2.0)
        .copied();
    assert!(
        heading3.is_some(),
        "p3 `3. 주요 추진내용` heading digit must match the 3mm-grid Hancom PDF y≈749.3: {threes:?}"
    );
}

#[test]
fn sample16_hwp3_page3_heading_positions_follow_hancom_grid() {
    let svg = render_svg("samples/hwp3-sample16.hwp", 2);
    let twos = extract_text_positions(&svg, "2");
    let threes = extract_text_positions(&svg, "3");

    let heading2 = twos.iter().find(|(_, y)| (*y - 337.2).abs() < 2.0).copied();
    assert!(
        heading2.is_some(),
        "HWP3 원본 p3 `2. 추진방향`도 한컴 3mm 격자 y≈337.2를 따라야 함: {twos:?}"
    );

    let heading3 = threes
        .iter()
        .find(|(_, y)| (*y - 749.2).abs() < 2.0)
        .copied();
    assert!(
        heading3.is_some(),
        "HWP3 원본 p3 `3. 주요 추진내용`도 한컴 3mm 격자 y≈749.2를 따라야 함: {threes:?}"
    );
}

#[test]
fn sample16_hwp5_page3_dump_pages_reports_line_spacing_in_height() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    let dump = doc.dump_page_items(Some(2));
    let p74 = dump
        .lines()
        .find(|line| line.contains("FullParagraph  pi=74"))
        .unwrap_or_else(|| panic!("p3 pi=74 dump line not found:\n{dump}"));

    assert!(p74.contains("h=88.2") && p74.contains("lines=80.6"),
        "p3 3줄 문단 높이는 lh=52.0px, ls=28.6px, HWP3-origin spacing_before=7.6px를 포함해 표시되어야 함: {p74}"
    );
    assert!(
        p74.contains("lh=52.0") && p74.contains("ls=28.6"),
        "p3 3줄 문단 진단은 lh/ls 분해를 함께 보여야 함: {p74}"
    );
}

#[test]
fn sample16_hwp5_page3_dump_pages_summary_uses_lineseg_spacing() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    let dump = doc.dump_page_items(Some(2));
    // #1648 수정으로 페이지 하단 빈 문단(pi=87)이 한컴처럼 page3 에 배치(items 19→20).
    // 단, pi=87은 가시 내용이 없는 trailing 빈 문단이므로 시각 사용 높이 used는
    // 2022 변환본(items=19)과 같은 경계를 유지해야 한다.
    let summary = dump
        .lines()
        .find(|line| line.contains("단 0 (items=20"))
        .unwrap_or_else(|| panic!("p3 단 요약을 찾을 수 없음:\n{dump}"));

    assert!(
        summary.contains("used=874.5px")
            && summary.contains("hwp_used≈841.6px")
            && summary.contains("diff=+32.9px"),
        "p3 단 요약은 trailing 빈 문단을 items에 유지하되 시각 사용 높이는 2022 기준과 같은 경계를 유지해야 함: {summary}"
    );
}

#[test]
fn sample16_hwp5_page3_bcp_tail_paragraph_stays_single_visual_line_for_pdf_oracle() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    let dump = doc.dump_page_items(Some(2));
    let p83 = dump
        .lines()
        .find(|line| line.contains("FullParagraph  pi=83"))
        .unwrap_or_else(|| panic!("p3 pi=83 dump line not found:\n{dump}"));

    assert!(
        p83.contains("h=31.5")
            && p83.contains("lines=27.7")
            && p83.contains("lh=17.3")
            && p83.contains("ls=10.4"),
        "p83 BCP 문단은 한컴 PDF 정답지처럼 단일 시각 줄로 유지되어야 함: {p83}"
    );
}

#[test]
fn sample16_hwp5_2022_page3_bcp_tail_paragraph_folds_orphan_lineseg() {
    let doc = load_doc("samples/hwp3-sample16-hwp5-2022.hwp");
    let dump = doc.dump_page_items(Some(2));
    let p83 = dump
        .lines()
        .find(|line| line.contains("FullParagraph  pi=83"))
        .unwrap_or_else(|| panic!("2022 p3 pi=83 dump line not found:\n{dump}"));
    let summary = dump
        .lines()
        .find(|line| line.contains("단 0 (items=19"))
        .unwrap_or_else(|| panic!("2022 p3 단 요약을 찾을 수 없음:\n{dump}"));

    assert!(
        p83.contains("h=31.5")
            && p83.contains("lines=27.7")
            && p83.contains("lh=17.3")
            && p83.contains("ls=10.4"),
        "2022 p83 BCP 문단의 마지막 LINE_SEG 꼬리는 한컴오피스처럼 앞 줄에 접혀야 함: {p83}"
    );
    assert!(
        summary.contains("used=874.5px")
            && summary.contains("hwp_used≈841.6px")
            && summary.contains("diff=+32.9px"),
        "2022 p3 단 요약은 p83 꼬리 LINE_SEG를 별도 시각 줄로 세지 않아야 함: {summary}"
    );
}

#[test]
fn sample16_hwp5_2022_page3_bcp_tail_glyph_stays_on_hancom_line() {
    let svg = render_svg("samples/hwp3-sample16-hwp5-2022.hwp", 2);
    let tail_glyphs = extract_text_positions(&svg, "립");

    let folded_tail = tail_glyphs
        .iter()
        .find(|(x, y)| *x > 620.0 && (*y - 881.35).abs() < 1.0)
        .copied();
    assert!(
        folded_tail.is_some(),
        "2022 p83 BCP `수립`의 `립`은 한컴오피스처럼 p83 본문 줄 y≈881.35에 있어야 함: {tail_glyphs:?}"
    );

    let orphan_tail = tail_glyphs
        .iter()
        .find(|(x, y)| (*x - 126.7).abs() < 2.0 && (*y - 909.1).abs() < 2.0)
        .copied();
    assert!(
        orphan_tail.is_none(),
        "2022 p83 BCP `립`이 다음 줄 머리에 단독 배치되면 p84 이하가 한컴오피스보다 내려감: {tail_glyphs:?}"
    );
}
