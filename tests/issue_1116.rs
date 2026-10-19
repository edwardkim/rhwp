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

/// 점선 리더를 **모양으로** 가려낸다 — 값으로 가리면 안 된다.
///
/// 종전에는 `stroke-dasharray="0.1 3"` 리터럴을 찾았는데, 그 값은 글꼴 크기와 무관한
/// 고정값이었고 #5843 에서 폰트 비례(`0.100 <간격>`)로 바뀌었다. 값에 묶인 탐지기는
/// 리더를 0개로 세어 아래 `>= 20` 단정을 엉뚱하게 깨뜨린다 — 이 시험이 지키려는 계약은
/// 리더의 **끝 x** 이지 점 간격이 아니다.
///
/// 그래서 (a) 둥근 끝(`stroke-linecap="round"`) 과 (b) 첫 dash 가 점만큼 짧은
/// 2단 dasharray 로 판별한다. 파선·쇄선(첫 dash 가 길다)과 표 괘선(둥근 끝이 없다)은
/// 걸리지 않는다.
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
        if !attrs.contains("stroke-linecap=\"round\"") {
            continue;
        }
        let dashes: Vec<f64> = attrs
            .find("stroke-dasharray=\"")
            .map(|at| &attrs[at + 18..])
            .and_then(|tail| tail.find('"').map(|end| &tail[..end]))
            .map(|v| {
                v.split_whitespace()
                    .filter_map(|p| p.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        if dashes.len() != 2 || dashes[0] > 0.5 || dashes[1] <= 0.0 {
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

// HWP3 원본의 목차 번호는 고정 x 영역 대신 원문 문단과 끝 토큰으로 고르는
// verified_hwp3_original_print_boundaries의 새 관계 검사에서 보호한다.

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
fn sample16_hwp5_2022_page3_latin_font_matches_legacy_hancom_mapping() {
    assert_page3_latin_poppy_resolves_to_palatino("samples/hwp3-sample16-hwp5-2022.hwp");
}

#[test]
fn sample16_hwp3_page3_latin_font_matches_legacy_hancom_mapping() {
    assert_page3_latin_poppy_resolves_to_palatino("samples/hwp3-sample16.hwp");
}

// HWP3 원본 제목의 독립 Print 기준선과 내용 소유는
// verified_hwp3_original_print_boundaries에서 검증한다.

#[test]
fn sample16_hwp5_page3_dump_pages_reports_line_spacing_in_height() {
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
// tests/cases/verified_sample16_2024_print_boundaries.rs
// 2022 저장본의 꼬리·이어받기 배치는 새 이름의 관계 검사로 보호한다.
// tests/cases/issue_7642_sample16_2022_paragraph_relations.rs
