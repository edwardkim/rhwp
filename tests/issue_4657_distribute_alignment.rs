//! [#4657] 배분 정렬 문단의 오른쪽 끝 정렬 — HWPX/SVG 실측 회귀.
//!
//! 같은 폭의 배분 정렬 문단 5개("문서관리번호 :" ~ "적용분류 :")는 글자 수가
//! 달라도 시작 x 와 마지막 글자(콜론) x 가 모두 같아야 한다. 남는 폭을 글자 수
//! N 으로 나누면 마지막 글자가 줄마다 slack/N 만큼 안쪽으로 밀려 콜론 x 가
//! 문단마다 어긋난다(결함 시 최대 Δ≈28px). 판정은 문단 간 상대 비교라 폰트
//! 메트릭 환경에 강건하다(절대좌표 단언 금지 — #3458 의 교훈).
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

fn render_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {e:?}", rel));
    doc.render_page_svg_native(0)
        .unwrap_or_else(|e| panic!("render {}: {e:?}", rel))
}

fn attr_f64(tag: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    tag[start..end].parse().ok()
}

/// `<text x=".." y="..">글자</text>` 를 (y, x) 로 수집한다. 배분 정렬 줄은
/// 글자마다 별도 `<text>` 로 배치되므로 같은 y 의 min/max x 가 줄의 시작/끝이다.
fn collect_text_positions(svg: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let mut search_from = 0usize;
    while let Some(rel) = svg[search_from..].find("<text ") {
        let tag_start = search_from + rel;
        let Some(tag_close_rel) = svg[tag_start..].find('>') else {
            break;
        };
        let tag = &svg[tag_start..tag_start + tag_close_rel];
        if let (Some(x), Some(y)) = (attr_f64(tag, "x"), attr_f64(tag, "y")) {
            out.push((y, x));
        }
        search_from = tag_start + tag_close_rel + 1;
    }
    out
}

#[test]
fn distribute_alignment_lines_share_left_and_right_edges() {
    let svg = render_svg("samples/issue4657/distribute-alignment-sample.hwpx");
    let positions = collect_text_positions(&svg);

    // y(줄) 별 (min x, max x). 글자 수가 다른 문단끼리 비교해야 slack/N 결함이
    // 드러난다 (8자/8자/5자/5자/6자 × 5문단).
    let mut lines: Vec<(f64, f64, f64)> = Vec::new(); // (y, min_x, max_x)
    for (y, x) in positions {
        match lines.iter_mut().find(|(ly, _, _)| (*ly - y).abs() < 0.5) {
            Some((_, min_x, max_x)) => {
                *min_x = min_x.min(x);
                *max_x = max_x.max(x);
            }
            None => lines.push((y, x, x)),
        }
    }
    assert_eq!(lines.len(), 5, "배분 정렬 문단 5개가 1쪽에 있어야 한다");

    let (_, base_min, base_max) = lines[0];
    for &(y, min_x, max_x) in &lines[1..] {
        assert!(
            (min_x - base_min).abs() <= 2.0,
            "y={y:.1} 줄 시작 x={min_x:.1} 가 첫 줄 시작 x={base_min:.1} 와 어긋났다",
        );
        assert!(
            (max_x - base_max).abs() <= 2.0,
            "y={y:.1} 줄 마지막 글자 x={max_x:.1} 가 첫 줄 마지막 글자 x={base_max:.1} 와 \
             어긋났다 — 배분 정렬이 남는 폭을 글자 사이(N-1)가 아니라 글자 수(N)로 나눴다 \
             (결함 시 Δ≈28px)",
        );
    }

    // 배분 자체가 무시(자연 폭 유지)되면 콜론이 시작 쪽으로 붙는다 — 줄이
    // 문단 폭 대부분을 채우고 있어야 한다.
    assert!(
        base_max - base_min > 150.0,
        "배분 정렬이 적용되지 않았다 — 줄 폭 {:.1}px",
        base_max - base_min,
    );
}
