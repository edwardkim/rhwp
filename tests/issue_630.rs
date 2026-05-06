//! Issue #630: aift.hwp 4페이지 목차 `(페이지 표기)` 우측 정렬 회귀
//!
//! 본질: `·` (U+00B7 MIDDLE DOT) 포함 라인의 `(페이지 표기)` 가 `·` 미포함
//! 라인 대비 정확히 8.67px (반각 1자) 좌측으로 이탈.
//!
//! 두 가지 본질 결함이 동시 작용:
//! 1. `is_halfwidth_punct` 가 U+00B7 강제 반각 처리 (한컴은 전각 측정)
//! 2. native `tab_type = ext[2]` raw u16 → enum 매치 실패 → LEFT fallback
//!
//! 정정 후 검증: aift.hwp page 4 의 모든 `(페이지 표기)` 시작 `(` x 좌표가
//! 단일 정렬 그룹 (±1.0px) 안에 들어와야 한다.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// SVG 텍스트에서 `(페이지 표기)` 의 시작 `(` x 좌표를 모두 추출한다.
///
/// SVG 의 각 글자는 `<text x="..." y="...">char</text>` 로 emit 된다.
/// y 좌표가 같은 글자들을 한 줄로 묶어 텍스트가 `페이지 ... 표기` 를 포함하는
/// 라인의 첫 `(` x 좌표를 수집.
fn extract_page_marker_paren_x_positions(svg: &str) -> Vec<f64> {
    let mut by_y: BTreeMap<i32, Vec<(f64, String)>> = BTreeMap::new();

    let mut i = 0;
    while i < svg.len() {
        let Some(rel) = svg[i..].find("<text ") else { break };
        let abs = i + rel;
        let after = &svg[abs + 6..];
        let Some(close) = after.find('>') else { i = abs + 6; continue };
        let attrs = &after[..close];
        let content_start = abs + 6 + close + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else { i = abs + 6; continue };
        let content = &svg[content_start..content_start + end_rel];

        let parse_attr = |key: &str| -> Option<f64> {
            let p = attrs.find(&format!("{}=\"", key))?;
            let s = p + key.len() + 2;
            let e = attrs[s..].find('"')? + s;
            attrs[s..e].parse::<f64>().ok()
        };

        if let (Some(x), Some(y)) = (parse_attr("x"), parse_attr("y")) {
            let y_key = (y * 10.0).round() as i32;
            by_y.entry(y_key).or_default().push((x, content.to_string()));
        }
        i = content_start + end_rel + 7;
    }

    let mut paren_xs = Vec::new();
    for (_y, mut chars) in by_y {
        chars.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let full: String = chars.iter().map(|(_, s)| s.as_str()).collect();
        // `페이지` + `표기` 포함 + 본문 영역 시작 (라인 첫 글자 x < 200) — wrap 두번째 줄 제외
        let starts_at_body = chars.first().map(|(x, _)| *x < 200.0).unwrap_or(false);
        if full.contains("페이지") && full.contains("표기") && starts_at_body {
            // "(페이지" 시퀀스 직전의 `(` x 를 찾는다 (6-2 의 "(협약…)" 등 다른 괄호 제외).
            // chars 는 x 순 정렬 — `페` 가 나오는 첫 인덱스 직전의 마지막 `(` 만 채택.
            let pe_idx = chars.iter().position(|(_, s)| s == "페");
            if let Some(pe_i) = pe_idx {
                if let Some((paren_x, _)) = chars[..pe_i].iter().rev().find(|(_, s)| s == "(") {
                    paren_xs.push(*paren_x);
                }
            }
        }
    }
    paren_xs
}

#[test]
fn test_630_aift_p4_toc_paren_alignment() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/aift.hwp");
    let bytes = fs::read(&hwp_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse aift.hwp");

    // 페이지 4 (0-indexed page=3) — 목차 페이지
    let svg = doc
        .render_page_svg_native(3)
        .expect("render aift.hwp page 4");

    let paren_xs = extract_page_marker_paren_x_positions(&svg);
    assert!(
        paren_xs.len() >= 20,
        "should find ≥ 20 `(페이지 표기)` lines on aift p4, got {}",
        paren_xs.len()
    );

    let min_x = paren_xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_x = paren_xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let spread = max_x - min_x;
    assert!(
        spread <= 1.0,
        "aift p4 목차 `(페이지 표기)` 시작 `(` 가 단일 그룹 (±1.0px) 안에 정렬되어야 함.\n  \
         lines={} min_x={:.2} max_x={:.2} spread={:.2}px (예상 ≤1.0)\n  \
         8.67px 이탈 = `·` 반각/전각 측정 차이 (Issue #630).",
        paren_xs.len(), min_x, max_x, spread
    );
}
