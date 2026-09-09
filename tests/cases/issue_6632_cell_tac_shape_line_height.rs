//! [#6632] 셀 안에서 글자와 글자처럼 도형이 한 줄인 문단의 줄 높이 계약.
//!
//! `samples/exam_kor.hwp` 5쪽 pi=145 셀[5]: p1 "(가) ⇨ [A 단계] ⇨ (나)" 는 글자와 글자처럼
//! 글상자(높이 26.5px)가 한 줄이고 저장 줄 원장 lh=1984HU(26.5px). 다음 문단 p2(그림 3장,
//! 79.4px)의 저장 vpos=11024HU(147.0px) = p1 시작 104.7 + 26.5 + 줄간격 9.2 + 문단 뒤 여백 6.7.
//! 종전 rhwp 는 글자+도형 줄을 글꼴 높이(18.4)로 접어 p2 가 138.9(8.1px 위)였다. 본문은
//! `layout_column_item` 바닥값이 되돌리지만 셀엔 그 바닥값이 없다. 한/글 2022 PDF(4절→A3
//! 균일 배율 0.9385, 왼쪽 위 기준) 실측: 그림 상단 451.2 (셀 내용 상단 304.3 + 147.0).
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

/// 글자 `ch` 하나짜리 `<text>` 의 (x, 기준선 y) 목록.
fn glyphs(svg: &str, ch: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for t in svg.split("<text ").skip(1) {
        let Some(close) = t.find('>') else { continue };
        let (head, rest) = t.split_at(close);
        let body = &rest[1..rest.find("</text>").unwrap_or(1)];
        if body.trim() != ch {
            continue;
        }
        if let (Some(x), Some(y)) = (attr(head, "x"), attr(head, "y")) {
            out.push((x, y));
        }
    }
    out
}

/// 같은 결함의 글자 판: `samples/hwpspec.hwp` 106쪽 셀 안 그림 라벨 "(x1, y1)" 줄은 글자와
/// 글자처럼 곡선 도형(저장 lh 152.9px) 한 줄. 종전엔 16.0 으로 접혀 다음 문단 "(x2, y2)",
/// "(x3, y3)" 가 도형 위에 겹쳤다(기준선 422.1·446.1). 한/글 2024 PDF 실측 글리프 상자
/// 아래 543.7·585.9 (기준선 ≈ 541.0·583.2), 수정 후 540.7·582.9.
#[test]
fn text_after_a_text_and_shape_line_in_a_cell_follows_the_stored_line_height() {
    let svg = page_svg("samples/hwpspec.hwp", 105);
    let two: Vec<_> = glyphs(&svg, "2")
        .into_iter()
        .filter(|(x, _)| (x - 505.2).abs() < 0.7)
        .collect();
    let three: Vec<_> = glyphs(&svg, "3")
        .into_iter()
        .filter(|(x, _)| (x - 261.5).abs() < 0.7)
        .collect();
    assert!(
        two.iter().any(|(_, y)| (y - 540.7).abs() < 0.7),
        "(x2, y2) 의 '2' 기준선 540.7 (종전 422.1): {two:?}"
    );
    assert!(
        three.iter().any(|(_, y)| (y - 582.9).abs() < 0.7),
        "(x3, y3) 의 '3' 기준선 582.9 (종전 446.1): {three:?}"
    );
}

#[test]
fn pictures_after_a_text_and_shape_line_follow_the_stored_line_height() {
    let svg = page_svg("samples/exam_kor.hwp", 4);
    let imgs = images_with_width(&svg, 79.4);
    assert_eq!(imgs.len(), 3, "㉠㉡㉢ 그림 3장: {imgs:?}");
    for (x, y) in &imgs {
        assert!(
            (y - 451.3).abs() < 0.7,
            "그림 y = 셀 내용 상단 304.3 + 저장 vpos 147.0 (종전 443.2): ({x:.1}, {y:.1})"
        );
    }
}
