//! [#6621] 1×1 상자 표(안 여백 850HU)를 unwrap 할 때 안쪽 표와 테두리가 상자 셀의 안 여백을
//! 잃던 결함의 계약.
//!
//! `samples/exam_social.hwp` 1쪽 pi=15: 바깥 1×1 표(셀 pad 850HU=11.33px, bf=6 테두리) 안에
//! 6×3 대화체 표가 들어 있고 첫 열 셀마다 41.5px 정사각형 그림이 글자처럼 놓인다. 한/글 2022
//! PDF(4절→A3 균일 배율 0.9385, 왼쪽 위 기준) 실측: 상자 왼쪽 선 x=549.9, 위 선 y=325.0,
//! 첫 그림 (561.2, 343.3) = 상자 원점 + 여백 11.33 + 첫 행 5.1 + 셀 여백 1.9. 종전 rhwp 는
//! 안쪽 표를 상자 원점(549.9, 324.9)에 그려 그림 5장이 (−11.3, −11.5), 상자 높이가 22.7px
//! 짧았다(저장 줄 원장은 그림 줄 lh = 그림 높이라 줄 높이 문제가 아니다).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

fn page0_svg() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_social.hwp");
    let bytes = std::fs::read(&path).expect("read exam_social.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse exam_social.hwp");
    doc.render_page_svg(0).expect("render page 1")
}

/// 태그 조각에서 `name="…"` 숫자 속성. 첫 속성은 앞에 공백이 없고(`<image x="…`), 다른
/// 속성 이름의 꼬리(`x1=` 의 `1=` 등)에 걸리지 않게 앞 글자가 영숫자가 아닐 때만 받는다.
fn attr(tag: &str, name: &str) -> Option<f64> {
    let pat = format!("{name}=\"");
    let mut from = 0;
    while let Some(i) = tag[from..].find(&pat) {
        let at = from + i;
        if at == 0 || !tag.as_bytes()[at - 1].is_ascii_alphanumeric() {
            let s = at + pat.len();
            let e = s + tag[s..].find('"')?;
            return tag[s..e].parse().ok();
        }
        from = at + 1;
    }
    None
}

/// 41.5px 정사각형 `<image>` 들 (x, y) — 문서 순서.
fn small_square_images(svg: &str) -> Vec<(f64, f64)> {
    svg.split("<image ")
        .skip(1)
        .map(|t| &t[..t.find('>').expect("image 닫힘")])
        .filter(|t| {
            let (w, h) = (
                attr(t, "width").unwrap_or(0.0),
                attr(t, "height").unwrap_or(0.0),
            );
            (w - 41.5).abs() < 0.6 && (h - 41.5).abs() < 0.6
        })
        .filter_map(|t| Some((attr(t, "x")?, attr(t, "y")?)))
        .collect()
}

#[test]
fn nested_table_pictures_sit_inside_the_wrapper_cell_padding() {
    let svg = page0_svg();
    let imgs = small_square_images(&svg);
    assert!(imgs.len() >= 4, "41.5px 그림이 4장 이상: {imgs:?}");
    // 첫 열(x≈561.2) 과 셋째 열(x≈906.6) 두 자리에만 있고, 첫 그림 y 는 343.3
    for (x, y) in &imgs {
        assert!(
            (x - 561.2).abs() < 0.7 || (x - 906.6).abs() < 0.7,
            "그림 x 가 상자 안 여백(549.9+11.33) 기준이 아니다: ({x:.1}, {y:.1})"
        );
    }
    let (x0, y0) = imgs[0];
    assert!((x0 - 561.2).abs() < 0.7, "첫 그림 x 561.2: {x0:.1}");
    assert!((y0 - 343.3).abs() < 0.7, "첫 그림 y 343.3: {y0:.1}");
}

#[test]
fn wrapper_border_box_preserves_declared_height_after_padding() {
    let svg = page0_svg();
    // 상자 왼쪽 세로 테두리: x≈549.9 인 <line> 중 가장 긴 것. 안쪽 표+여백보다
    // 큰 1x1 host의 선언 높이 27774HU=370.3px를 계속 보존해야 한다.
    let mut best: Option<(f64, f64)> = None;
    for t in svg.split("<line ").skip(1) {
        let t = &t[..t.find("/>").expect("line 닫힘")];
        let (Some(x1), Some(x2), Some(y1), Some(y2)) =
            (attr(t, "x1"), attr(t, "x2"), attr(t, "y1"), attr(t, "y2"))
        else {
            continue;
        };
        if (x1 - x2).abs() < 0.01 && (x1 - 549.9).abs() < 0.7 {
            let (top, bottom) = (y1.min(y2), y1.max(y2));
            if best.is_none_or(|(a, b)| bottom - top > b - a) {
                best = Some((top, bottom));
            }
        }
    }
    let (top, bottom) = best.expect("상자 왼쪽 테두리");
    assert!((top - 325.0).abs() < 0.7, "상자 위 325.0: {top:.1}");
    assert!(
        (bottom - top - 370.3).abs() < 1.0,
        "상자 높이 = host 선언 높이 370.3: {:.1}",
        bottom - top
    );
}
