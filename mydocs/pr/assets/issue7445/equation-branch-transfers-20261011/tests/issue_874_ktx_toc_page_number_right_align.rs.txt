//! Task #874 후속 (PR #1021): KTX.hwp 목차 페이지번호 — 단일-run RIGHT + leader
//! 인라인 탭의 cell right inner 정렬 회귀 가드.
//!
//! 재현 문서: `samples/KTX.hwp` page 2 (0-indexed page=1) — 목차.
//! 동일 페이지를 `svg_snapshot::issue_267_ktx_toc_page` 가 golden 으로 검사하나,
//! 본 가드는 페이지번호 x 좌표를 명시적으로 단언해 회귀를 더 분명히 드러낸다.
//!
//! 본질: PR #1021 이전에는 장제목 페이지번호("8" 등)의 오른쪽 끝이 x≈699.76 으로,
//! cell right inner(x≈689.76)보다 10px 우측으로 이탈했다. PR #1021 의
//! `text_measurement.rs` 단일-run RIGHT+leader 분기(`(2, _) if fill_low != 0`)가
//! 이를 x≈689.76 으로 정렬했다.
//!
//! 회귀 가드 등록 사유: PR #1026(좁은 구두점 폭 fix) 검토 중, stale 브랜치를
//! 통째-파일 교체로 통합하면서 PR #1021 코드가 유실되어 이 +10px 이탈이
//! 재현된 사례가 있었다. golden 비교만으로는 원인이 모호하므로, 좌표 단언으로
//! 영구 회귀 가드를 둔다.
//!
//! 좌표 측정: `render_page_svg_native` SVG 의 페이지번호 digit `<text>` x (96 DPI).

use std::fs;
use std::path::Path;

/// 렌더된 SVG 에서 우측 페이지번호 열(x > 600)에 있는 한 자리 숫자 글리프의
/// x 좌표를 모두 추출한다. 목차 leader 점선은 `<line>` 으로 emit 되므로
/// 텍스트로 잡히지 않는다 — x > 600 영역의 `<text>` 는 페이지번호 숫자뿐이다.
fn page_number_digit_xs(svg: &str) -> Vec<f64> {
    let mut xs = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = svg[search_from..].find("<text ") {
        let tag_start = search_from + rel;
        search_from = tag_start + 6;

        let after = &svg[tag_start + 6..];
        let Some(close) = after.find('>') else { break };
        let attrs = &after[..close];
        let content_start = tag_start + 6 + close + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else {
            break;
        };
        let content = &svg[content_start..content_start + end_rel];

        let x = {
            let Some(p) = attrs.find("x=\"") else {
                continue;
            };
            let s = p + 3;
            let Some(e_rel) = attrs[s..].find('"') else {
                continue;
            };
            match attrs[s..s + e_rel].parse::<f64>() {
                Ok(v) => v,
                Err(_) => continue,
            }
        };

        if x > 600.0 && content.chars().count() == 1 && content.chars().all(|c| c.is_ascii_digit())
        {
            xs.push(x);
        }
    }
    xs
}

#[test]
fn ktx_toc_page_numbers_aligned_to_cell_right_inner() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/KTX.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse KTX.hwp");

    // 페이지 2 (0-indexed page=1) — 목차. svg_snapshot::issue_267_ktx_toc_page 와 동일.
    let svg = doc
        .render_page_svg_native(1)
        .expect("render KTX.hwp page 2");

    let xs = page_number_digit_xs(&svg);
    assert!(
        xs.len() >= 8,
        "KTX 목차 페이지번호 숫자를 8개 이상 찾아야 함 \
         (장제목 3/8/16/20/24 = 숫자 8개 이상), got {}",
        xs.len()
    );

    // PR #1021 정합: 페이지번호 오른쪽 끝 ≈ x=689.76 (cell right inner).
    //   - 장제목(font-size 20) 최댓값 ≈ 689.76, 소제목(font-size 18.67) ≈ 690.76.
    // PR #1021 회귀 시: 장제목이 ≈ x=699.76 (+10px 우측 이탈).
    // 임계 695.0 — 정상(≤690.8)과 회귀(≈699.76)를 ±4px 여유로 결정적 판별.
    let max_x = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max_x < 695.0,
        "KTX 목차 페이지번호가 cell right inner(x≈689.76)에 정렬되어야 함.\n  \
         페이지번호 digit 최대 x={:.2} (정상 ≤690.8, 회귀 시 ≈699.76)\n  \
         x≥695 = PR #1021(단일-run RIGHT+leader cell right inner 정렬) 회귀 — \
         text_measurement.rs 의 `(2, _) if fill_low != 0` 분기를 확인할 것.",
        max_x
    );
}
