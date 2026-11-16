//! [#9087] 관계형·고정형 계약 — HFT 「신명 순명조」가 직접 그리는 글자의 전진폭은 독립 한컴 Print 의 Type3 `/Widths` 비율이고, 굵게는 글자마다 0.05em 더 전진한다.
//!
//! 주석 갱신: 2026-11-16 18:26 KST(+09:00).
//! 직전 파일 변경: 2026-11-16 17:39:14 KST, `709bd6962c3b46f81de15b35560e023eb2246908`.
//!
//! 유형: 같은 줄 앞뒤 글자 사이 간격이 글자 크기 × 폭/1000 인지 보는 관계형이다. 절대 px 좌표는
//! 고정하지 않는다. 추가 고정형 검사는 독립 Print 3쪽의 기호 뒤 「가」 x 좌표를 96 DPI에서 ±2px로
//! 보호한다. 모든 검사는 설치 글꼴을 가정하지 않는다.
//!
//! 대상: `samples/issue9087/hft-sinmyeong_sunmyeongjo-probe.hwpx`·`…-bold-probe.hwpx`. 일곱 언어 슬롯을
//! 모두 HFT 「신명 순명조」로 둔 합성 탐침(`samples/lang_slot/gen_lang_slot_probe.py`, FTYPE=HFT)이고
//! 굵게 탐침은 글자 모양에 굵게만 더했다. 문단마다 `가<글자>가` 를 적었다. 저장 줄은 없다.
//!
//! 독립 기대값: `pdf/issue9087/hft-sinmyeong_sunmyeongjo{,-bold}-probe-2022.pdf`(한컴오피스 2022
//! 12.0.0.4547 Print → Hancom PDF)의 Type3 `/Widths`·ToUnicode. 굵게 탐침의 `/Widths` 는 일반과 같고,
//! 글자 원점은 이 프로그램이 그린 글자(한글·ASCII)마다 약 0.05em 더 전진한다(`가<글자>` 쌍 평균
//! +0.105em, 16진 숫자 셋 +0.157em). 공백과 한컴바탕으로 대체된 글자는 더 전진하지 않는다.
//!
//! 시각 측정(2026-11-15, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`,
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/hwp`·`ttfs/windows`, 각 4쪽 = 한컴 4쪽):
//! - 일반 탐침: 수정 전 Native 98.93·99.65·95.82·94.54 → 수정 후 Native 99.85·99.82·90.38·93.64 / fresh WASM 99.93·99.95·94.42·93.90.
//! - 굵게 탐침: 수정 전 Native 95.22·95.35·92.83·93.70 → 수정 후 Native 99.86·99.92·91.57·93.90 / fresh WASM 99.93·99.95·95.39·94.51.
//!   전쪽 최저: 일반 Native 90.38·fresh WASM 93.90, 굵게 Native 91.57·fresh WASM 94.51.
//! - 위 수치는 원 PR의 기록이다. 통합 후보 `10d0d6ae`의 전각 기호 폭 보정 후 측정(2026-11-16,
//!   canonical PNG/96 DPI/compat 2022/2px, Windows 원본 HBATANG + Mac 사용자 글꼴 공급):
//!   일반 Native 100.00·100.00·99.66029·97.42349 / fresh WASM 100.00·100.00·99.66062·97.39462.
//!   굵게 Native 99.99536·99.99537·99.65994·97.59521 / fresh WASM 99.99536·99.99537·99.66001·97.50866.
//!   각 4쪽이며 전쪽 최저는 일반 Native 97.42349·WASM 97.39462, 굵게 Native 97.59521·WASM 97.50866.
//! - 추가 관계형 기대값: 독립 Print의 Haansoft Batang 기호 원점 간격/동일 줄 한글 전각 전진 = 1em.
//!   추가 고정형 기대값: 일반 Print 3쪽 기호 뒤 「가」 원점 105.54280pt × 96/72 = 140.72373px.
//!   3쪽 양쪽 98% 이상 및 같은 쪽 review/overlay 직접 판독 후 ±2px만 고정한다. 4쪽은 고정하지 않는다.
//! - 증거: `mydocs/pr/assets/planet6897_20261116/cjk-advance-before.json` 및
//!   `mydocs/pr/assets/planet6897_20261116/cjk-advance-validation.json`(독립 PDF·생산자·전쪽 결과).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::RenderProfile;
use rhwp::DocumentCore;

struct Glyph {
    x: f64,
    y: f64,
    size: f64,
    text: String,
}

fn attr<'a>(tag: &'a str, key: &str) -> Option<&'a str> {
    let at = tag.find(&format!(" {key}=\""))? + key.len() + 3;
    let rest = &tag[at..];
    Some(&rest[..rest.find('"')?])
}

/// `x`/`y` 속성 또는 `translate(x,y)` 로 놓인 한 글자 `<text>`.
fn glyphs(svg: &str) -> Vec<Glyph> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(at) = rest.find("<text") {
        rest = &rest[at..];
        let Some(gt) = rest.find('>') else { break };
        let tag = &rest[..=gt];
        let body = &rest[gt + 1..];
        let Some(close) = body.find("</text>") else {
            break;
        };
        let text = body[..close]
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .replace("&#183;", "\u{00B7}");
        let position = match (attr(tag, "x"), attr(tag, "y")) {
            (Some(x), Some(y)) => x.parse().ok().zip(y.parse().ok()),
            _ => attr(tag, "transform").and_then(|t| {
                let inner = t.strip_prefix("translate(")?;
                let inner = &inner[..inner.find(')')?];
                let (x, y) = inner.split_once(',')?;
                x.trim().parse().ok().zip(y.trim().parse().ok())
            }),
        };
        if let (Some((x, y)), Some(size)) = (
            position,
            attr(tag, "font-size").and_then(|v| v.parse::<f64>().ok()),
        ) {
            if text.chars().count() == 1 {
                out.push(Glyph { x, y, size, text });
            }
        }
        rest = &body[close..];
    }
    out
}

/// 탐침 문단 `가<글자>가` 에서 시험 글자의 전진폭(px)과 글자 크기를 모은다.
fn advances(path: &str) -> Vec<(char, f64, f64)> {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&full).expect("탐침 입력")).expect("탐침 열기");
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let svg = core
            .render_page_svg_layer_with_profile_native(page, RenderProfile::Print)
            .expect("SVG");
        let mut lines: Vec<Vec<Glyph>> = Vec::new();
        for glyph in glyphs(&svg) {
            match lines
                .iter_mut()
                .find(|line| (line[0].y - glyph.y).abs() < 0.5)
            {
                Some(line) => line.push(glyph),
                None => lines.push(vec![glyph]),
            }
        }
        for mut line in lines {
            line.sort_by(|a, b| a.x.total_cmp(&b.x));
            let ga: Vec<usize> = (0..line.len()).filter(|&i| line[i].text == "가").collect();
            if let [first, last] = ga[..] {
                if last == first + 2 {
                    let target = &line[first + 1];
                    let ch = target.text.chars().next().unwrap();
                    out.push((ch, line[last].x - target.x, target.size));
                }
            }
        }
    }
    out
}

/// 탐침의 시험 글자 전진이 글자 크기 × (폭/1000 + 굵기 전진 em) 인지 검사한다.
fn check(path: &str, bold_em: f64, expected: &[(char, u16)]) {
    let measured = advances(path);
    for &(ch, width) in expected {
        let hits: Vec<_> = measured.iter().filter(|(c, _, _)| *c == ch).collect();
        assert!(!hits.is_empty(), "{path}: {ch:?} 탐침 줄이 없다");
        for (_, advance, size) in hits {
            let want = size * (f64::from(width) / 1000.0 + bold_em);
            assert!(
                (advance - want).abs() <= 0.05,
                "{path}: {ch:?} 전진 {advance:.3}px, 기대 {want:.3}px(Type3 {width}/1000 + {bold_em}em)"
            );
        }
    }
}

/// Type3 `/Widths` 에서 고른 글자. 숫자 600, 구두점 400·500, 대소문자 고유 폭.
const WIDTHS: &[(char, u16)] = &[
    ('0', 600),
    ('4', 600),
    ('.', 400),
    (',', 400),
    ('(', 500),
    ('-', 500),
    ('A', 725),
    ('W', 969),
    ('i', 372),
    ('m', 1027),
    ('~', 1000),
    ('‘', 200),
    ('“', 400),
];

/// 일반 굵기: 「신명 순명조」 HFT 가 직접 그리는 글자는 Type3 폭으로 전진한다.
///
/// 수정 전에는 대체 글꼴 폭(숫자 0.5em 등)으로 쟀다.
#[test]
fn sinmyeong_sunmyeongjo_hft_uses_its_own_latin_widths() {
    check(
        "samples/issue9087/hft-sinmyeong_sunmyeongjo-probe.hwpx",
        0.0,
        WIDTHS,
    );
}

/// 굵게: 같은 글자가 Type3 폭에 0.05em 을 더해 전진한다.
///
/// 수정 전에는 폭 표가 없어 HFT 굵기 전진 계약에 연결되지 않았다.
#[test]
fn bold_sinmyeong_sunmyeongjo_hft_adds_bold_tracking_to_its_own_glyphs() {
    check(
        "samples/issue9087/hft-sinmyeong_sunmyeongjo-bold-probe.hwpx",
        0.05,
        WIDTHS,
    );
}

/// 독립 Print가 전각으로 그린 대체 기호도 뒤 한글을 반각만큼 앞당기지 않는다.
#[test]
fn cjk_fallback_symbols_preserve_independent_print_full_em_advance() {
    check(
        "samples/issue9087/hft-sinmyeong_sunmyeongjo-probe.hwpx",
        0.0,
        &[
            ('、', 1000),
            ('。', 1000),
            ('〄', 1000),
            ('々', 1000),
            ('〆', 1000),
            ('〇', 1000),
            ('〈', 1000),
            ('【', 1000),
            ('〤', 1000),
        ],
    );
}

/// 99.66%로 대조·판독한 3쪽에서 전각 기호 뒤 한글의 독립 Print x 원점을 보호한다.
#[test]
fn cjk_page_three_following_hangul_keeps_independent_print_x_origin() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue9087/hft-sinmyeong_sunmyeongjo-probe.hwpx");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("탐침 입력")).expect("탐침 열기");
    let svg = core
        .render_page_svg_layer_with_profile_native(2, RenderProfile::Print)
        .expect("독립 Print와 대조한 3쪽");
    let glyphs = glyphs(&svg);
    // PDFium의 독립 PDF 문자 원점을 pt→96 DPI px로 변환한 값이다.
    let pdf_x_px = 105.542_800_903_320_31 * 96.0 / 72.0;
    for symbol in ["、", "〄", "〈", "【", "〤"] {
        let target = glyphs.iter().find(|g| g.text == symbol).expect("기호 존재");
        let following = glyphs
            .iter()
            .filter(|g| g.text == "가" && (g.y - target.y).abs() < 0.5 && g.x > target.x)
            .min_by(|a, b| a.x.total_cmp(&b.x))
            .expect("같은 줄 뒤 한글 존재");
        assert!(
            (following.x - pdf_x_px).abs() <= 2.0,
            "{symbol:?} 뒤 한글 x={}px, 독립 Print {pdf_x_px}px ±2px",
            following.x
        );
    }
}
