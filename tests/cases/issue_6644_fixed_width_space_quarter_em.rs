//! [#6644] HWP 고정폭 빈칸(제어 문자 31 → `\u{2007}`)의 전진폭 계약: 0.25em × 장평.
//!
//! 한/글 2022 PDF 글리프 origin 실측:
//! - `samples/k-water-rfp.hwp` 8쪽 "업체␣중 결격사유": 16.0px 글꼴, 장평 1.0. 업→체 16.0,
//!   체→중 20.0 → 고정폭 빈칸 4.0 = 0.25em. 종전 rhwp 는 0.5em(8.0) 이라 체→중 24.0.
//! - `samples/exam_kor.hwp` 5쪽 셀[5] "㉠␣[그림] 4칸 ㉡␣[그림] 4칸 ㉢␣[그림]": 그림 3장 왼쪽
//!   x 181.0 / 304.9 / 429.0 (4절→A3 균일 배율 0.9385 되돌림). 종전 rhwp 179.8 / 307.2 / 434.5
//!   — 묶음(4칸+㉡+고정폭)이 48.0 vs 한/글 44.6 이라 장당 +3.4 누적.
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

/// 태그 조각에서 `name="…"` 숫자 속성. 첫 속성은 앞에 공백이 없고, 다른 속성 이름의 꼬리에
/// 걸리지 않게 앞 글자가 영숫자가 아닐 때만 받는다.
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

/// 글자 하나짜리 `<text>` 의 (x, y, 글자) 목록. 원점은 `x="…" y="…"` 또는
/// `transform="translate(x,y) …"` (폭 보정 글꼴) 두 형식 중 하나로 온다.
fn glyphs(svg: &str) -> Vec<(f64, f64, char)> {
    let mut out = Vec::new();
    for t in svg.split("<text ").skip(1) {
        let Some(close) = t.find('>') else { continue };
        let (head, rest) = t.split_at(close);
        let body = &rest[1..rest.find("</text>").unwrap_or(1)];
        let mut cs = body.chars();
        let (Some(c), None) = (cs.next(), cs.next()) else {
            continue;
        };
        let origin = if let Some(tr) = head.find("translate(") {
            let nums = &head[tr + "translate(".len()..];
            nums.find(')').and_then(|end| {
                let mut it = nums[..end].split(',');
                let x = it.next()?.trim().parse().ok()?;
                let y = it.next()?.trim().parse().ok()?;
                Some((x, y))
            })
        } else {
            attr(head, "x").zip(attr(head, "y"))
        };
        if let Some((x, y)) = origin {
            out.push((x, y, c));
        }
    }
    out
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
fn fixed_width_space_advances_a_quarter_em() {
    let svg = page_svg("samples/k-water-rfp.hwp", 7);
    let g = glyphs(&svg);
    // 같은 줄(y 같음)에서 '체' 바로 오른쪽의 '중' 을 찾는다.
    let mut found = Vec::new();
    for &(x_che, y, c) in &g {
        if c != '체' {
            continue;
        }
        let next = g
            .iter()
            .filter(|(x, yy, cc)| (yy - y).abs() < 0.01 && *x > x_che && *cc == '중')
            .map(|(x, ..)| x - x_che)
            .fold(f64::INFINITY, f64::min);
        if next.is_finite() && next < 40.0 {
            found.push((y, next));
        }
    }
    assert!(
        found.iter().any(|(_, dx)| (dx - 20.0).abs() < 0.4),
        "'체'→'중' origin 간격 = 전진폭 16.0 + 고정폭 빈칸 4.0 (한/글 20.0, 종전 24.0): {found:?}"
    );
}

#[test]
fn pictures_after_fixed_width_spaces_follow_hancom() {
    let svg = page_svg("samples/exam_kor.hwp", 4);
    let mut imgs = images_with_width(&svg, 79.4);
    imgs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert_eq!(imgs.len(), 3, "㉠㉡㉢ 그림 3장: {imgs:?}");
    let expected = [180.8, 305.2, 429.5];
    for ((x, _), e) in imgs.iter().zip(expected) {
        assert!(
            (x - e).abs() < 0.7,
            "그림 x {e} (한/글 181.0 / 304.9 / 429.0, 종전 179.8 / 307.2 / 434.5): {imgs:?}"
        );
    }
}
