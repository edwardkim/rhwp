//! [#6655] 인라인으로 재분류된 부동 그림 스택은 각자의 가로 오프셋을 유지한다.
//!
//! `[#2004]` 정규화는 같은 자리에 겹친 전면급 부동 그림 더미를 그림 1장짜리 인라인
//! 문단 N개로 쪼갠다. 한글이 이런 더미를 쪽당 1장씩 놓는 것을 재현하는 변환이라
//! 되돌릴 수 없다 — 끄면 그 그림들이 아예 렌더되지 않는다. 다만 변환이 개체의
//! `horzOffset` 을 배치에 넘기지 않아, 종전에는 5장이 모두 셀 왼끝 82.4 에 겹쳤다.
//!
//! `samples/issue2004_cell_image_stack.hwpx` 문단 42 의 1×1 표 셀에 그림 5장이
//! 있고, 전부 `treatAsChar=0 / wrap=Square / horzRelTo=Para / horzAlign=Left` 로
//! `horzOffset` 만 다르다(0 / 1580 / 2092 / 1550 / 1432 HWPUNIT).
//!
//! 한글 2022 PDF(`pdf/issue2004_cell_image_stack-2022.pdf`) 실측 x 는 각각
//! 86.0 / 107.1 / 113.9 / 106.7 / 105.1 이다.
//!
//! [#7063] 종전에는 이 다섯 값에서 공통으로 3.6px 이 빠졌고(표 자체의 바깥 여백
//! 283HWPUNIT), `#6643` 의 몫이라 계약에서 뺐다. 자리차지 표가 자기 `outMargin.left`
//! 만큼 안으로 들어가면서 그 몫이 채워져, 이제 기대값을 **한/글 실측 그대로** 쓴다
//! (남은 차이는 다섯 장 모두 0.16px).
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

/// 태그 조각에서 `name="…"` 숫자 속성. 첫 속성은 앞에 공백이 없고, 다른 속성 이름의
/// 꼬리에 걸리지 않게 앞 글자가 영숫자가 아닐 때만 받는다.
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

/// 폭이 `w`(±0.6) 인 그림 자리의 x. 자른 그림은 `<svg>` 래퍼로 나오므로 둘 다 본다.
fn image_x_with_width(svg: &str, w: f64) -> Option<f64> {
    for open in ["<image ", "<svg "] {
        for t in svg.split(open).skip(1) {
            let t = &t[..t.find('>').expect("태그 닫힘")];
            if (attr(t, "width").unwrap_or(0.0) - w).abs() < 0.6 {
                if let Some(x) = attr(t, "x") {
                    return Some(x);
                }
            }
        }
    }
    None
}

#[test]
fn reclassified_stack_pictures_keep_their_paragraph_horizontal_offset() {
    // (0-기준 쪽, 그림 폭 px, 기대 x px, 한글 x px)
    let cases = [
        (3u32, 601.5, 86.0, 86.0),
        (4, 579.3, 107.1, 107.1),
        (5, 592.1, 113.9, 113.9),
        (6, 580.1, 106.7, 106.7),
        (7, 604.9, 105.1, 105.1),
    ];
    let mut seen = Vec::new();
    for (page, width, expected_x, hancom_x) in cases {
        let svg = page_svg("samples/issue2004_cell_image_stack.hwpx", page);
        let x = image_x_with_width(&svg, width)
            .unwrap_or_else(|| panic!("{}쪽 폭 {width}px 그림", page + 1));
        assert!(
            (x - expected_x).abs() < 0.5,
            "{}쪽 그림 x {expected_x} (한글 {hancom_x}): {x}",
            page + 1
        );
        seen.push(x);
    }
    // 종전에는 5장이 모두 셀 왼끝 82.4 에 겹쳤다. 오프셋이 살아 있으면 자리가 갈린다.
    assert!(
        seen.iter().any(|x| (x - seen[0]).abs() > 1.0),
        "그림 5장이 서로 다른 x 에 놓여야 한다: {seen:?}"
    );
}
