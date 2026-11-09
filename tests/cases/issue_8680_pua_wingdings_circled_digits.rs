//! MS 워드에서 온 Wingdings 기호(HWP 기본 PUA `U+F000 + Wingdings 코드`)의 표시와 전진폭.
//!
//! 한컴 웹한글기안기 인쇄 PDF(`pdf/issue8680/wingdings_pua_symbols-webhwp-print.pdf`)는
//! 이 글자를 문서 글꼴과 상관없이 `Haan Wings` 글꼴의 같은 코드(0x81 등)로 그린다.
//! - 0x80~0x95 는 ⓪ ①..⑩ ⓿ ❶..❿ (Wingdings→Unicode 공개 표와 PDF 글리프가 일치)
//! - 0xA1 은 ○ 크기의 흰 원, 0xFE 는 ☑
//! - 내장 Haan Wings 폭: 0x80~0x95·0xA1 = 891/1000, 0xFE = 892/1000
//!
//! 수정 전 rhwp 는 0x80~0x95 의 표시 대체가 없어 빈칸(반각 전진)으로 그렸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue8680/wingdings_pua_symbols.hwpx";

struct Glyph {
    text: String,
    x: f64,
    y: f64,
    size: f64,
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let key = format!(" {name}=\"");
    let start = head.find(&key)? + key.len();
    let end = head[start..].find('"')?;
    head[start..start + end].parse().ok()
}

fn glyphs(svg: &str) -> Vec<Glyph> {
    let mut out = Vec::new();
    for cap in svg.split("<text").skip(1) {
        let Some(gt) = cap.find('>') else { continue };
        let head = &cap[..gt];
        let body = &cap[gt + 1..cap.find("</text>").unwrap_or(cap.len())];
        let (Some(x), Some(y), Some(size)) =
            (attr(head, "x"), attr(head, "y"), attr(head, "font-size"))
        else {
            continue;
        };
        out.push(Glyph {
            text: body.to_string(),
            x,
            y,
            size,
        });
    }
    out
}

fn page_svg() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("재현물")).expect("문서 열기");
    core.render_page_svg_native(0).expect("1쪽 SVG")
}

#[test]
fn wingdings_pua_circled_digits_are_painted_with_visible_symbols() {
    let svg = page_svg();
    let all = glyphs(&svg);
    let count = |s: &str| all.iter().filter(|g| g.text == s).count();
    for raw in [
        '\u{F080}', '\u{F081}', '\u{F082}', '\u{F083}', '\u{F08A}', '\u{F08B}', '\u{F08C}',
        '\u{F095}', '\u{F0A1}',
    ] {
        assert!(
            !all.iter().any(|g| g.text.contains(raw)),
            "원 PUA U+{:04X} 가 그대로 그려지면 공개 글꼴에서 빈칸이 된다",
            raw as u32
        );
    }
    // 첫 줄의 PUA ①②③ + 둘째 줄의 일반 ①②③
    assert_eq!(count("①"), 2, "PUA 0x81 과 일반 ① 이 모두 ① 로 보인다");
    assert_eq!(count("②"), 2);
    assert_eq!(count("③"), 2);
    for s in ["⓪", "⑩", "⓿", "❶", "❿"] {
        assert_eq!(count(s), 1, "Wingdings 0x80~0x95 대역 {s}");
    }
    // 0xA1 세 개 + 일반 ○ 세 개
    assert_eq!(count("○"), 6, "Wingdings 0xA1 은 ○ 크기의 흰 원이다");
}

#[test]
fn wingdings_pua_symbols_advance_like_haan_wings() {
    let svg = page_svg();
    let all = glyphs(&svg);
    // "영:" 줄의 연속 기호 다섯 개: 서로 같은 줄, 일정한 전진.
    let run: Vec<&Glyph> = ["⓪", "⑩", "⓿", "❶", "❿"]
        .iter()
        .map(|s| all.iter().find(|g| g.text == *s).expect("기호"))
        .collect();
    let em = run[0].size;
    for pair in run.windows(2) {
        assert!((pair[0].y - pair[1].y).abs() < 0.5, "한 줄에 놓인다");
        let advance = pair[1].x - pair[0].x;
        assert!(
            (advance / em - 0.891).abs() < 0.03,
            "Haan Wings 전진폭(0.891em)을 따른다: {:.3}em",
            advance / em
        );
    }
    // 첫 줄 PUA ① 은 둘째 줄 일반 ① 보다 좁게 전진한다(한컴 PDF: 다음 '가' 가 약 1pt 앞).
    let mut circled: Vec<&Glyph> = all.iter().filter(|g| g.text == "①").collect();
    circled.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
    let next_ga = |g: &Glyph| {
        all.iter()
            .filter(|h| (h.y - g.y).abs() < 0.5 && h.x > g.x + 0.1)
            .map(|h| h.x)
            .fold(f64::INFINITY, f64::min)
    };
    let pua_adv = next_ga(circled[0]) - circled[0].x;
    let lit_adv = next_ga(circled[1]) - circled[1].x;
    assert!(
        (pua_adv / em - 0.891).abs() < 0.03,
        "PUA ① 다음 글자까지 0.891em: {:.3}em",
        pua_adv / em
    );
    assert!(pua_adv < lit_adv, "PUA ① 은 일반 ① 보다 좁다");
}
