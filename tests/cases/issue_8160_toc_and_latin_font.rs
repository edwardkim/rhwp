//! #8160: sample16 회귀 검사 통합. 이전 등록: tests/issue_1116.rs
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

mod svg_tab_leaders {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/common/svg_tab_leaders.rs"
    ));
}

fn extract_dotted_horizontal_lines(svg: &str) -> Vec<(f64, f64, f64)> {
    svg_tab_leaders::dot_leaders(svg)
        .into_iter()
        .map(|leader| (leader.start, leader.end, leader.y))
        .collect()
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
fn issue_8160_sample16_hwp5_toc_leaders_stop_before_page_numbers() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    let svg = doc.render_page_svg_native(1).expect("render p2");
    let leaders = extract_dotted_horizontal_lines(&svg);
    assert!(
        leaders.len() >= 20,
        "sample16 p2 목차 leader 점선을 충분히 찾아야 함: {}",
        leaders.len()
    );

    // 같은 원본의 독립 Print와 새 Native/fresh WASM 전64쪽을 먼저 확인했다.
    // 2쪽은 Native raw98% 미만이므로 과거 x650 경계로 배치를 고정하지 않는다.
    // SVG가 내보내는 실제 숫자 glyph와 리더의 같은 글줄·앞뒤 관계를 보호한다.
    let numbers: Vec<_> = ('0'..='9')
        .flat_map(|digit| extract_text_attrs(&svg, &digit.to_string()))
        .filter_map(|attrs| {
            Some((
                attr_f64(&attrs, "x")?,
                attr_f64(&attrs, "y")?,
                attr_f64(&attrs, "font-size")?,
            ))
        })
        .collect();
    for (start, end, leader_y) in leaders {
        let number_left = numbers
            .iter()
            .filter(|(x, baseline, em)| {
                *x >= start && *baseline >= leader_y && *baseline - *em <= leader_y
            })
            .map(|(x, _, _)| *x)
            .min_by(f64::total_cmp)
            .expect("각 목차 리더 뒤 같은 글줄에 쪽번호가 있어야 함");
        assert!(
            end <= number_left,
            "리더 끝은 같은 줄 쪽번호의 첫 glyph를 침범하지 않아야 함: {end} > {number_left}"
        );
    }
}

#[test]
fn issue_8160_sample16_hwp5_toc_page_numbers_share_right_edge() {
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

// HWP3 원본의 목차 번호는 고정 x 영역 대신 원문 문단과 끝 토큰으로 고르는
// verified_hwp3_original_print_boundaries의 새 관계 검사에서 보호한다.

#[test]
fn issue_8160_sample16_hwp5_page3_svg_latin_glyphs_pin_browser_width() {
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
        latin_c_attrs.iter().any(|attrs| {
            // SVG attribute serialization must XML-escape the CSS family-name
            // quotes. Assert the semantic family list rather than a single
            // serializer spelling (plain `'` vs `&apos;`).
            attrs
                .replace("&apos;", "'")
                .contains("font-family=\"'Palatino Linotype',")
        }),
        "{rel_path} p3 Latin glyphs must resolve HCI Poppy to Palatino Linotype: {latin_c_attrs:?}"
    );
    assert!(
        latin_c_attrs.iter().all(|attrs| {
            !attrs
                .replace("&apos;", "'")
                .contains("font-family=\"'HCI Poppy',")
        }),
        "{rel_path} p3 Latin glyphs must not fall back through unresolved HCI Poppy: {latin_c_attrs:?}"
    );
}

#[test]
fn issue_8160_sample16_hwp5_2022_page3_latin_font_matches_legacy_hancom_mapping() {
    assert_page3_latin_poppy_resolves_to_palatino("samples/hwp3-sample16-hwp5-2022.hwp");
}

#[test]
fn issue_8160_sample16_hwp3_page3_latin_font_matches_legacy_hancom_mapping() {
    assert_page3_latin_poppy_resolves_to_palatino("samples/hwp3-sample16.hwp");
}

// HWP3 원본 제목의 독립 Print 기준선과 내용 소유는
// verified_hwp3_original_print_boundaries에서 검증한다.

#[test]
fn issue_8160_sample16_hwp5_page3_dump_pages_reports_line_spacing_in_height() {
    let doc = load_doc("samples/hwp3-sample16-hwp5.hwp");
    let dump = doc.dump_page_items(Some(2));
    let p74 = dump
        .lines()
        .find(|line| line.contains("FullParagraph  pi=74"))
        .unwrap_or_else(|| panic!("p3 pi=74 dump line not found:\n{dump}"));

    // #4628 dump-pages 는 HeightMeasurer 합(h=88.2)이 아니라 프로덕션
    // format_paragraph total 을 말한다. lh/ls 분해와 줄 간격 포함은 유지한다.
    assert!(
        p74.contains("lines=80.6"),
        "p3 3줄 문단은 줄 간격이 포함된 lines=80.6 을 유지해야 함: {p74}"
    );
    assert!(
        p74.contains("lh=52.0") && p74.contains("ls=28.6"),
        "p3 3줄 문단 진단은 lh/ls 분해를 함께 보여야 함: {p74}"
    );
}

// 2024 저장본의 제목·BCP 꼬리·후행 빈 문단은 전쪽 검증과 독립 Print 좌표를
// 확인한 새 이름의 검사로 보호한다. 다른 원본의 검증 근거로 대신하지 않는다.
// tests/cases/issue_8160_hwp2024_print_boundaries.rs
// 2022 저장본의 꼬리·이어받기 배치는 새 이름의 관계 검사로 보호한다.
// tests/cases/issue_8160_hwp2022_paragraph_relations.rs
