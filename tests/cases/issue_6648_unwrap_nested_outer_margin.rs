//! [#6648] 1×1 상자 표를 풀어 그릴 때 안쪽 표의 바깥 여백(outer_margin)이 원점에 남는 계약.
//!
//! `samples/k-water-rfp.hwp` 17쪽 상자: 상자 셀 pad (510,510,141,141)HU, 안쪽 3×2 표 om 141HU
//! 네 방향. 한/글 2022 PDF(같은 쪽 크기) 실측: 상자 실선 x 82.0/717.2 · y 583.2/1001.4, 안쪽 표
//! 점선 x 90.6/712.0 · y 587.0/997.5 — 점선은 실선에서 (pad + om) = (8.7, 3.8) 안쪽이다.
//! om 이 빠지면 점선이 (88.7, 585.2) 로 (−1.9, −1.8) 어긋난다(#6621/#6645 직후 상태).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

fn page_svg(rel: &str, page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
    doc.render_page_svg(page)
        .unwrap_or_else(|e| panic!("render {rel} p{}: {e:?}", page + 1))
}

/// `<line>` 요소의 (x1, y1, x2, y2, 점선 여부).
fn lines(svg: &str) -> Vec<(f64, f64, f64, f64, bool)> {
    let mut out = Vec::new();
    for seg in svg.split("<line ").skip(1) {
        let head = &seg[..seg.find('>').unwrap_or(seg.len())];
        let get = |k: &str| -> Option<f64> {
            let p = head.find(&format!("{k}=\""))? + k.len() + 2;
            let rest = &head[p..];
            rest[..rest.find('"')?].parse().ok()
        };
        if let (Some(x1), Some(y1), Some(x2), Some(y2)) =
            (get("x1"), get("y1"), get("x2"), get("y2"))
        {
            out.push((x1, y1, x2, y2, head.contains("stroke-dasharray")));
        }
    }
    out
}

#[test]
fn unwrapped_nested_table_keeps_its_outer_margin_inside_the_cell_padding() {
    let svg = page_svg("samples/k-water-rfp.hwp", 16);
    let ls = lines(&svg);
    let wide = |x1: f64, x2: f64| (x2 - x1).abs() > 500.0;
    let tall = |y1: f64, y2: f64| (y2 - y1).abs() > 300.0;

    // 상자 실선(전폭 수평·세로) — 한/글 583.2/1001.4, 82.0/717.2.
    let solid_h: Vec<f64> = ls
        .iter()
        .filter(|(x1, y1, x2, y2, d)| !*d && (y1 - y2).abs() < 0.01 && wide(*x1, *x2))
        .map(|(_, y, ..)| *y)
        .collect();
    let solid_v: Vec<f64> = ls
        .iter()
        .filter(|(x1, y1, x2, y2, d)| {
            !*d && (x1 - x2).abs() < 0.01 && tall(*y1, *y2) && *y1 > 560.0
        })
        .map(|(x, ..)| *x)
        .collect();
    assert!(
        solid_h.iter().any(|y| (y - 583.3).abs() < 0.5),
        "상자 위 실선 583.3: {solid_h:?}"
    );
    assert!(
        solid_h.iter().any(|y| (y - 1001.4).abs() < 0.5),
        "상자 아래 실선 1001.4: {solid_h:?}"
    );
    assert!(
        solid_v.iter().any(|x| (x - 81.9).abs() < 0.5),
        "상자 왼쪽 실선 81.9: {solid_v:?}"
    );

    // 안쪽 표 점선(전폭 수평·세로) — 한/글 587.0/997.5, 90.6/712.0. 종전 585.2/995.8, 88.7/709.8.
    let dashed_h: Vec<f64> = ls
        .iter()
        .filter(|(x1, y1, x2, y2, d)| *d && (y1 - y2).abs() < 0.01 && wide(*x1, *x2))
        .map(|(_, y, ..)| *y)
        .collect();
    let dashed_v: Vec<f64> = ls
        .iter()
        .filter(|(x1, y1, x2, y2, d)| *d && (x1 - x2).abs() < 0.01 && tall(*y1, *y2))
        .map(|(x, ..)| *x)
        .collect();
    assert!(
        dashed_h.iter().any(|y| (y - 587.1).abs() < 0.4),
        "안쪽 표 위 점선 = 상자 583.3 + pad 1.9 + om 1.9 = 587.1 (한/글 587.0, 종전 585.2): {dashed_h:?}"
    );
    assert!(
        dashed_h.iter().any(|y| (y - 997.7).abs() < 0.4),
        "안쪽 표 아래 점선 997.7 (한/글 997.5, 종전 995.8): {dashed_h:?}"
    );
    assert!(
        dashed_v.iter().any(|x| (x - 90.6).abs() < 0.4),
        "안쪽 표 왼쪽 점선 = 상자 81.9 + pad 6.8 + om 1.9 = 90.6 (한/글 90.6, 종전 88.7): {dashed_v:?}"
    );
}
