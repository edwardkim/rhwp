//! [#6653] 중첩 표는 자기 줄 상자 자리에 그려진다 — 그 줄만큼 밀고 나서 그리지 않는다.
//!
//! 저장 사다리는 중첩 표를 품은 줄을 그 표 높이로 적어 둔다.
//! `samples/hwpx_sample2.hwp` 의 큰 셀 문단 `p[21]` 이 그렇다.
//!
//! ```text
//! p[21] ctrls=1 text_len=102
//!       ls[0] vpos=61628 lh=1100  ls=660   ← 8쪽: 글자 줄
//!       ls[1] vpos=0     lh=12080 ls=600   ← 9쪽: 표를 품는 줄 (161.07px)
//! ```
//!
//! 안쪽 표 높이가 157.3px 이므로 그 줄 상자가 곧 표 자리다. 글자와 표를 함께 가진 문단은
//! 텍스트 갈래를 타는데, 그 갈래가 표 전용 줄까지 지나 흐름을 밀고 나서 표를 그렸다 —
//! 같은 높이를 두 번 쓴 셈이다. 8쪽에는 글자 줄이 있어 드러나지 않지만, 표 줄만 넘어온
//! 9쪽 조각은 표가 161px 아래로 내려가 위에 빈 띠가 남았다.
//!
//! 한/글 2024 PDF(`pdf/hwpx_sample2-2024.pdf`) 9쪽 실측: 바깥 표 조각 37.72~568.98,
//! 안쪽 표 가로선 52.1 / 76.72 / 161.74 / 209.21.
//!
//! 이 계약은 표가 조각 위쪽(줄 상자 자리)에 오는 것을 고정한다. 한/글 52.1 과 남는
//! 10.6px 은 셀 `valign=Center` 몫으로 보이며 아직 확정하지 못했다 — 이 시험은 그 값을
//! 박지 않고, 표가 161px 아래로 내려가지 않는 것만 단언한다.
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

/// `<line>` 조각의 (x1, y1, x2, y2).
fn lines(svg: &str) -> Vec<(f64, f64, f64, f64)> {
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
            out.push((x1, y1, x2, y2));
        }
    }
    out
}

#[test]
fn nested_table_paints_at_its_own_host_line_on_a_continuation_page() {
    let svg = page_svg("samples/hwpx_sample2.hwp", 8);
    // 안쪽 표(구 분/조회방법)의 가로선은 폭 600px 대이고, 바깥 표 조각 테두리(718px)와
    // 구분된다. 조각 상단(37.8) 아래 첫 가로선이 안쪽 표 윗선이다.
    let mut inner: Vec<f64> = lines(&svg)
        .into_iter()
        .filter(|(x1, y1, x2, y2)| {
            (y1 - y2).abs() < 0.01 && (500.0..700.0).contains(&(x2 - x1).abs()) && *y1 > 38.0
        })
        .map(|(_, y, ..)| y)
        .collect();
    inner.sort_by(f64::total_cmp);
    let top = *inner
        .first()
        .unwrap_or_else(|| panic!("9쪽 안쪽 표 가로선: {inner:?}"));
    assert!(
        top < 60.0,
        "9쪽 안쪽 표 윗선은 조각 상단 가까이 있어야 한다 (한/글 52.1, 수정 전 210.6): {top}"
    );
    assert!(
        inner.len() >= 4,
        "안쪽 표 가로선 4개(윗선·행 경계 2개·아랫선): {inner:?}"
    );
}
