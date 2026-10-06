#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7151] 실제 Bold face 가 없는 face 의 볼드를 `font-weight="bold"` 로만
//! 내보내 굵히는 양이 받는 래스터라이저 몫이 된다 — Chromium 은 한/글보다 굵다.
//!
//! 오라클은 한/글 2022 정본 PDF 다. 볼드 요청을 `2 Tr`(fill+stroke)로 내보내고
//! 선 굵기를 **글꼴·크기 불문 `0.02 em`** 으로 준다 —
//! `pdf/issue2470/36382471_masked-2022.pdf` 1쪽의 `Tr 2` run 전부에서 `w / Tf`
//! 가 0.0200 이다(굴림체 `1.66/83`, HY헤드라인M `3.50/175`, HY견명조 `4.34/217`).
//! 배율을 1x~8x 로 올려도 볼드/평문 잉크비가 그대로라 래스터 헤어라인이 아니다.
//!
//! 수정 전/후 잉크비(Chromium 794×1123 래스터 ↔ 정본 PDF 같은 크기, 커버리지 합):
//!
//! ```text
//!   구간                     한/글   수정 전   수정 후
//!   제목 HY헤드라인M 32.5px    7120     7576     6881
//!   중랑물재생센터 34.7px       4379     4744     4355
//!   2026. 6. HY견명조 28px    765      834      703
//!   문서번호 굴림체 13.3px       157      233      199
//!   |정본 대비 오차| 평균                0.313    0.176
//! ```

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 한/글 실측 — 합성 볼드 획은 글자 크기의 이 비율이다.
const STROKE_EM: f64 = 0.02;

fn page_svg(rel: &str, page: u32) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let core = DocumentCore::from_bytes(&std::fs::read(p).expect("표본 읽기")).expect("문서 로드");
    core.render_page_svg_native(page).expect("SVG 렌더")
}

/// `<text …>` 하나에서 (font-size, stroke-width, font-weight) 를 뽑는다.
fn text_elements(svg: &str) -> Vec<(f64, Option<f64>, Option<String>)> {
    let attr = |el: &str, name: &str| -> Option<String> {
        let key = format!("{name}=\"");
        let i = el.find(&key)? + key.len();
        let rest = &el[i..];
        Some(rest[..rest.find('"')?].to_string())
    };
    svg.split("<text ")
        .skip(1)
        .filter_map(|rest| {
            let el = &rest[..rest.find('>')?];
            Some((
                attr(el, "font-size")?.parse().ok()?,
                attr(el, "stroke-width").and_then(|v| v.parse().ok()),
                attr(el, "font-weight"),
            ))
        })
        .collect()
}

/// Bold face 가 없는 face 의 볼드는 `0.02 em` 획으로 나가고 `font-weight` 는 빠진다.
///
/// 수정 전에는 이 쪽의 `<text>` 55 개가 `font-weight="bold"` 였고 획은 없었다.
#[test]
fn issue_7151_faux_bold_becomes_a_two_percent_em_stroke() {
    let svg = page_svg("samples/issue2470/36382471_masked.hwpx", 0);
    let stroked: Vec<_> = text_elements(&svg)
        .into_iter()
        .filter(|(_, sw, _)| sw.is_some())
        .collect();

    // 굴림체 22 · HY헤드라인M 21 · HY견명조 7 · 굴림체 12px 5 — 정본 PDF 의
    // `Tr 2` 글자 수와 낱자 단위로 같다.
    assert_eq!(
        stroked.len(),
        55,
        "합성 볼드 글자 수가 정본 `Tr 2` 와 같아야 한다 — {} 개",
        stroked.len()
    );
    for (size, sw, weight) in &stroked {
        let want = size * STROKE_EM;
        assert!(
            (sw.unwrap() - want).abs() <= 0.001,
            "획은 글자 크기의 {STROKE_EM} 배여야 한다 — 크기 {size:.3}px, \
             기대 {want:.3}px, 실측 {:.3}px",
            sw.unwrap()
        );
        assert!(
            weight.is_none(),
            "획으로 굵히는 글자에는 font-weight 를 함께 주지 않는다 — {weight:?}"
        );
    }
    assert!(
        !svg.contains("font-weight=\"bold\""),
        "이 쪽의 볼드 face 는 모두 Bold 항목이 없다 — font-weight=\"bold\" 가 남으면 안 된다"
    );
}

/// ⚠ 관문 — 실제 Bold 메트릭이 있는 face 는 종전대로 그 face 를 요청한다.
///
/// 그 face 는 배치 advance 도 Bold 메트릭으로 잡았으므로 regular 를 그리면
/// 글리프 폭이 advance 와 어긋난다. `form-002` 는 맑은 고딕(Bold 항목 있음)
/// 볼드가 122 군데다.
#[test]
fn issue_7151_faces_with_a_real_bold_keep_font_weight() {
    let svg = page_svg("samples/hwpx/form-002.hwpx", 0);
    assert!(
        svg.contains("font-weight=\"bold\""),
        "Bold 메트릭이 있는 face 는 font-weight=\"bold\" 를 유지해야 한다"
    );
    for (size, sw, weight) in text_elements(&svg) {
        if weight.as_deref() == Some("bold") {
            assert!(
                sw.is_none(),
                "Bold face 요청에 합성 획을 겹쳐 주면 두 번 굵어진다 — \
                 크기 {size:.3}px, 획 {sw:?}"
            );
        }
    }
}
