//! [#6704] 앵커 문단이 앞 쪽에서 이어진 조각이면 떠 있는 그림은 그 쪽 본문 맨 위를 기준으로 놓는다.
//!
//! `vert=Para` 개체는 앵커 문단의 시작 y 에 저장 offset 을 더해 놓인다. 그런데 앵커
//! 문단이 앞 쪽에서 시작해 이 쪽으로 이어진 조각이면, `para_start_y` 에 담긴 값은 그
//! 문단의 시작이 아니라 이 쪽에서 흐름이 도달한 자리다. 개체는 흐름에서 빠져 있으므로
//! 그 자리를 쓰면 앞 항목이 흘린 만큼 그대로 내려간다.
//!
//! `samples/hwp3-sample.hwp` 문단 0.76(저장 `vpos=51520` = 686.9px, 6쪽)에 붙은
//! `bin_id=3` 그림(`tac=false`, `wrap=TopAndBottom`, `vert=Para(off=8400)`)은 7쪽에
//! 그려진다. 7쪽 본문 시작은 132.27px 인데 종전 rhwp 는 흐름이 도달한 217.60px 을 앵커로
//! 써서 그림이 85.6px 아래에 놓였다.
//!
//! 한/글 2022 PDF(`pdf/hwp3-sample-2022.pdf`, 16쪽/16쪽·A4 동일) 실측: 그 그림 y=255.4,
//! x=143.1. 가로는 종전에도 맞았다(dx −0.02).
//!
//! 같은 쪽 다른 그림들은 앵커가 이 쪽에서 시작하므로 규칙 밖이다 — 그 값이 함께 바뀌지
//! 않는지도 같이 고정한다.
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

/// SVG 안 `<image>` 의 (x, y, width) 를 모은다.
fn image_boxes(svg: &str) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<image").skip(1) {
        let head = &chunk[..chunk.find('>').unwrap_or(chunk.len())];
        let attr = |k: &str| -> Option<f64> {
            let pat = format!(" {k}=\"");
            let i = head.find(&pat)? + pat.len();
            let rest = &head[i..];
            rest[..rest.find('"')?].parse().ok()
        };
        if let (Some(x), Some(y), Some(w)) = (attr("x"), attr("y"), attr("width")) {
            out.push((x, y, w));
        }
    }
    out
}

#[test]
fn floating_picture_anchored_to_continued_paragraph_starts_at_body_top() {
    let svg = page_svg("samples/hwp3-sample.hwp", 6); // 0-based → 7쪽

    // 폭 519px(38940 HU) 인 그림이 문제의 개체다. 같은 쪽 바닥글 그림(80px)과 구분된다.
    let big: Vec<(f64, f64, f64)> = image_boxes(&svg)
        .into_iter()
        .filter(|(_, _, w)| (515.0..=525.0).contains(w))
        .collect();

    assert_eq!(big.len(), 1, "7쪽 519px 그림이 하나여야 한다: {big:?}");
    let (x, y, _) = big[0];

    // 한/글 실측 (255.4, 143.1). 오라클 추출 오차 ±3px.
    assert!(
        (252.0..=259.0).contains(&y),
        "그림 y 가 한/글(255.4) 근처여야 한다. 종전 값은 341.0 이었다. 실제: {y}"
    );
    assert!(
        (140.0..=146.0).contains(&x),
        "그림 x 는 종전에도 맞았다(한/글 143.1). 실제: {x}"
    );
}

#[test]
fn same_page_pictures_with_local_anchor_are_untouched() {
    let svg = page_svg("samples/hwp3-sample.hwp", 6);

    // 바닥글 그림(80px). 앵커가 이 쪽에서 시작하므로 이 규칙의 대상이 아니다.
    let footer: Vec<(f64, f64, f64)> = image_boxes(&svg)
        .into_iter()
        .filter(|(_, _, w)| (75.0..=85.0).contains(w))
        .collect();

    assert!(
        !footer.is_empty(),
        "7쪽 바닥글 그림을 찾지 못했다 — 표본이 바뀌었는지 확인할 것"
    );
    // 한/글 실측 y=1019.4 (dy +1.13 은 이 수정 전후로 같다).
    for (_, y, _) in &footer {
        assert!(
            (1016.0..=1024.0).contains(y),
            "바닥글 그림 y 가 움직이면 안 된다(한/글 1019.4). 실제: {y}"
        );
    }
}
