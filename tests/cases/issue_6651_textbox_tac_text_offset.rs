//! [#6651] 글상자 문단에서 글자처럼 그림 뒤의 글자가 그림 폭만큼 두 번 밀리지 않는 계약.
//!
//! `samples/table-in-tbox.hwp` 2쪽 글상자 문단 p[7] "␣[그림 17.5×15]␣서비스 기간 : 2025년 …"
//! (왼쪽 정렬, 안쪽 왼끝 79.4). 한/글 2022 PDF 실측: 그림 x 89.3, '서' origin x 116.8
//! = 79.4 + 빈칸 10 + 그림 17.5 + 빈칸 10. 종전 rhwp 는 첫 run 이 79.4 + 17.4 에서 시작해
//! '서' 가 134.3 이었다(그림은 89.4 로 맞았음).
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

/// 글자 하나짜리 `<text>` 의 (x, y, 글자). 원점은 `x="…" y="…"` 또는 `translate(x,y)`.
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

/// 폭·높이가 (w, h)(±0.6) 인 `<image>` 자리의 (x, y) 목록.
fn images_sized(svg: &str, w: f64, h: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for t in svg.split("<image ").skip(1) {
        let t = &t[..t.find('>').expect("태그 닫힘")];
        if (attr(t, "width").unwrap_or(0.0) - w).abs() < 0.6
            && (attr(t, "height").unwrap_or(0.0) - h).abs() < 0.6
        {
            if let (Some(x), Some(y)) = (attr(t, "x"), attr(t, "y")) {
                out.push((x, y));
            }
        }
    }
    out
}

#[test]
fn text_after_an_inline_picture_in_a_textbox_advances_once() {
    let svg = page_svg("samples/table-in-tbox.hwp", 1);
    // 그림 4장(17.5×15.0)은 제자리(x 89.4)다.
    let pics = images_sized(&svg, 17.5, 15.0);
    assert!(pics.len() >= 4, "글상자 안 17.5×15 그림 4장: {pics:?}");
    for (x, y) in &pics {
        assert!(
            (x - 89.4).abs() < 0.7,
            "그림 x 89.4 (한/글 89.3): ({x:.1}, {y:.1})"
        );
    }
    // 그림 줄의 첫 글자 '서'(y≈510.8 기준선) — 79.4 + 빈칸 + 그림 + 빈칸 = 116.9.
    let seo: Vec<(f64, f64)> = glyphs(&svg)
        .into_iter()
        .filter(|(_, y, c)| *c == '서' && (y - 510.8).abs() < 1.5)
        .map(|(x, y, _)| (x, y))
        .collect();
    assert!(!seo.is_empty(), "'서비스 기간' 줄의 '서' 글리프");
    for (x, y) in &seo {
        assert!(
            (x - 116.9).abs() < 0.7,
            "'서' x = 79.4 + 10 + 17.5 + 10 = 116.9 (한/글 116.8, 종전 134.3): ({x:.1}, {y:.1})"
        );
    }
}
