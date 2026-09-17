//! [#7235] 그림 채우기 유형 15(NONE)는 원래 픽셀 크기로 놓지 않고 종횡비를 지켜 영역에 맞춘다.
//!
//! `samples/issue7235/156086935_none_image_fill.hwp` 는 NONE 그림 채우기 세 곳을 가진다.
//!
//! | 자리 | 그림(px) | 채우기 영역(px) | 한/글 2020 PDF 그림 상자(px) |
//! | --- | --- | --- | --- |
//! | 1쪽 도형 머리띠 | 1500×166 | 642.5×74.1 | x 75.5~720.6, y 21.4~93.4 |
//! | 1쪽 로고 셀(bf=9) | 454×152 | 228.7×71.1 | x 495.3~713.7, y 100.1~173.3 |
//! | 9쪽 로고 셀(bf=8) | 1009×383 | 137.2×65.8 | x 81.7~222.9, y 952.2~1005.9 |
//!
//! 한/글은 세 곳 모두 종횡비를 지키고 영역 가운데에 둔다(PDF `get_image_info` 실측,
//! `-2020.pdf`). 수정 전 SVG 는 셀 채우기를 원래 픽셀 크기(454×152 등)로 셀 왼쪽 위에 놓아
//! 셀 clip 에 잘렸고, 도형 채우기는 176×57 로 왼쪽 위에 작게 놓았다.
//!
//! 한/글 상자는 영역에 딱 맞춘 크기보다 0.4~3% 크다 — 그 차이의 규칙은 확정하지 못해
//! 이 시험은 크기 5%, 중심 3px 허용으로 "맞춤 + 가운데" 만 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const FIXTURE: &str = "samples/issue7235/156086935_none_image_fill.hwp";

fn page_svg(page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {FIXTURE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {FIXTURE}: {e:?}"));
    doc.render_page_svg(page)
        .unwrap_or_else(|e| panic!("render {FIXTURE} p{}: {e:?}", page + 1))
}

/// SVG `<image>` 가 실제로 칠하는 상자 (x, y, w, h). `intrinsic` 은 그림의 픽셀 크기.
fn painted_boxes(svg: &str, intrinsic: (f64, f64)) -> Vec<(f64, f64, f64, f64)> {
    let mut out = Vec::new();
    for seg in svg.split("<image ").skip(1) {
        let head = &seg[..seg.find(" href=").unwrap_or(seg.len())];
        let get = |k: &str| -> Option<String> {
            let p = head.find(&format!("{k}=\""))? + k.len() + 2;
            let rest = &head[p..];
            Some(rest[..rest.find('"')?].to_string())
        };
        let num = |k: &str| get(k).and_then(|v| v.parse::<f64>().ok());
        let (Some(x), Some(y), Some(w), Some(h)) =
            (num("x"), num("y"), num("width"), num("height"))
        else {
            continue;
        };
        if get("preserveAspectRatio").as_deref() == Some("xMidYMid meet") {
            let scale = (w / intrinsic.0).min(h / intrinsic.1);
            let (fw, fh) = (intrinsic.0 * scale, intrinsic.1 * scale);
            out.push((x + (w - fw) / 2.0, y + (h - fh) / 2.0, fw, fh));
        } else {
            out.push((x, y, w, h));
        }
    }
    out
}

/// 한/글 상자 `(x0, y0, x1, y1)` 와 중심이 가장 가까운 칠한 상자가 크기·중심 허용 안에 있는지.
fn assert_matches_hancom(
    page: u32,
    intrinsic: (f64, f64),
    hancom: (f64, f64, f64, f64),
    what: &str,
) {
    let svg = page_svg(page);
    let (hx, hy) = ((hancom.0 + hancom.2) / 2.0, (hancom.1 + hancom.3) / 2.0);
    let hw = hancom.2 - hancom.0;
    let best = painted_boxes(&svg, intrinsic)
        .into_iter()
        .map(|(x, y, w, h)| {
            let d = ((x + w / 2.0 - hx).powi(2) + (y + h / 2.0 - hy).powi(2)).sqrt();
            (d, (x, y, w, h))
        })
        .filter(|(_, (_, _, w, h))| (w / h - intrinsic.0 / intrinsic.1).abs() < 0.05)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap_or_else(|| panic!("{what}: 종횡비가 맞는 그림이 없다"));
    let (d, (x, y, w, h)) = best;
    assert!(
        d <= 3.0 && (w / hw - 1.0).abs() <= 0.05,
        "{what}: 칠한 상자 x={x:.1} y={y:.1} w={w:.1} h={h:.1} (중심 거리 {d:.1}px) \
         — 한/글 x {:.1}~{:.1}, y {:.1}~{:.1} (w {hw:.1})",
        hancom.0,
        hancom.2,
        hancom.1,
        hancom.3,
    );
}

#[test]
fn none_fill_in_table_cell_fits_cell_keeping_aspect() {
    assert_matches_hancom(
        0,
        (454.0, 152.0),
        (495.3, 100.1, 713.7, 173.3),
        "1쪽 로고 셀",
    );
    assert_matches_hancom(
        8,
        (1009.0, 383.0),
        (81.7, 952.2, 222.9, 1005.9),
        "9쪽 로고 셀",
    );
}

#[test]
fn none_fill_in_shape_fits_shape_keeping_aspect() {
    assert_matches_hancom(
        0,
        (1500.0, 166.0),
        (75.5, 21.4, 720.6, 93.4),
        "1쪽 도형 머리띠",
    );
}
