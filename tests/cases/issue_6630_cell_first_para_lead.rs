//! [#6630] 세로 가운데/아래 정렬 셀의 첫 문단 위 여백 계약.
//!
//! `samples/exam_eng.hwp` 2쪽 바탕쪽 머리 표(1×3)의 가운데 셀(pad 141HU, valign=Center)에
//! 글자처럼 놓인 제목 그림(148.7×37.8px). 그림 문단(ps 1)의 위 여백은 1136HU(해석값 7.57px)이고
//! 저장 줄 원장 첫 줄 `vpos=568HU`(7.57px). 한/글 2022 PDF(4절→A3 균일 배율 0.9385, 왼쪽 위 기준)
//! 실측: 셀 (413.2, 132.3, 296.1×64.3), 그림 상단 = 셀 상단 + 16.9
//! = pad 1.9 + 가운데 정렬 (60.5 − (7.57 + 37.8)) / 2 + 7.57. 종전 rhwp 는 위 여백을 내용 높이에도
//! 첫 문단 y 에도 넣지 않아 13.2(3.7px 위)였다. `samples/exam_kor.hwp` 14쪽의 같은 구조(그림이
//! 글줄 안에 인라인으로 놓이는 경로)도 같은 규칙을 탄다.
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

/// 태그 조각에서 `name="…"` 숫자 속성. 첫 속성은 앞에 공백이 없고(`<image x="…`), 다른
/// 속성 이름의 꼬리에 걸리지 않게 앞 글자가 영숫자가 아닐 때만 받는다.
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

/// 폭이 `w`(±0.6) 인 그림 자리의 (x, y) 목록. 자른 그림은 `<svg x y width height viewBox>`
/// 래퍼 안에 원본 픽셀 크기의 `<image>` 로 들어가므로 래퍼도 본다.
fn images_with_width(svg: &str, w: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for open in ["<image ", "<svg "] {
        for t in svg.split(open).skip(1) {
            let t = &t[..t.find('>').expect("태그 닫힘")];
            if (attr(t, "width").unwrap_or(0.0) - w).abs() < 0.6 {
                if let (Some(x), Some(y)) = (attr(t, "x"), attr(t, "y")) {
                    out.push((x, y));
                }
            }
        }
    }
    out
}

#[test]
fn master_page_title_picture_sits_below_the_first_para_lead() {
    // exam_eng 2쪽: 셀 상단 132.3 + 16.9 = 149.2 (한/글). 종전 145.5.
    let svg = page_svg("samples/exam_eng.hwp", 1);
    let imgs = images_with_width(&svg, 148.7);
    assert_eq!(imgs.len(), 1, "바탕쪽 제목 그림 하나: {imgs:?}");
    let (x, y) = imgs[0];
    assert!((x - 486.9).abs() < 0.7, "x 486.9: {x:.1}");
    assert!(
        (y - 149.2).abs() < 0.7,
        "y = 셀 상단 + pad + 가운데 정렬 + 위 여백 7.57: {y:.1}"
    );
}

#[test]
fn inline_title_picture_in_centered_cell_follows_the_same_rule() {
    // exam_kor 14쪽(다른 바탕쪽, 그림이 글줄 안 인라인): 한/글 148.7 (종전 144.9).
    let svg = page_svg("samples/exam_kor.hwp", 13);
    let imgs = images_with_width(&svg, 150.3);
    assert_eq!(imgs.len(), 1, "머리 표 제목 그림 하나: {imgs:?}");
    let (_, y) = imgs[0];
    assert!((y - 148.7).abs() < 0.7, "y 148.7: {y:.1}");
}
