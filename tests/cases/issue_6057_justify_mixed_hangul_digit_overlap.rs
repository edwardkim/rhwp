//! [Issue #6057] 양쪽 정렬 줄에서 PUA 책괄호(U+F0855→》) 측정이 반각으로
//! 짧아 뒤 한글 '제' 글리프 위에 숫자 '10'이 겹친다.
//!
//! `samples/issue6057/29494.hwp` 1쪽 `《…특별조치법》 제10조의2의 규정에`
//! 줄: 레이아웃은 원문 PUA 를 0.5em 휴리스틱으로 재고, SVG/Skia 는
//! `map_pua_bullet_char` 로 전각 》 를 그린다. run bbox 가 ~0.5em 짧아
//! 다음 라틴 run `10` 이 '제'(1em) 오른쪽 절반 위에 놓인다.
//! 같은 쪽 통제군 `…특별조치법 제10조의2(` 는 PUA 가 없어 제→1 이 ~1em.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::composer::expand_pua_render_text;
use rhwp::renderer::layout::map_pua_bullet_char;

const SAMPLE: &str = "samples/issue6057/29494.hwp";

#[derive(Clone, Copy)]
struct Glyph {
    x: f64,
    y: f64,
    ch: char,
}

fn svg_glyphs(svg: &str) -> Vec<Glyph> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(idx) = rest.find("<text ") {
        rest = &rest[idx + 6..];
        let Some(tag_end) = rest.find('>') else {
            break;
        };
        let attrs = &rest[..tag_end];
        let after = &rest[tag_end + 1..];
        let Some(text_end) = after.find("</text>") else {
            break;
        };
        let text = &after[..text_end];
        rest = &after[text_end + 7..];
        if text.chars().count() != 1 {
            continue;
        }
        let Some(ch) = text.chars().next() else {
            continue;
        };
        let Some(x) = attr_f64(attrs, "x") else {
            continue;
        };
        let Some(y) = attr_f64(attrs, "y") else {
            continue;
        };
        out.push(Glyph { x, y, ch });
    }
    out.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });
    out
}

fn attr_f64(attrs: &str, name: &str) -> Option<f64> {
    let key = format!("{name}=\"");
    let start = attrs.find(&key)? + key.len();
    let end = attrs[start..].find('"')? + start;
    attrs[start..end].parse().ok()
}

fn je_to_one_advances(svg: &str) -> Vec<(f64, f64)> {
    let glyphs = svg_glyphs(svg);
    let mut pairs = Vec::new();
    for win in glyphs.windows(2) {
        if win[0].ch == '제' && win[1].ch == '1' && (win[0].y - win[1].y).abs() < 1.0 {
            pairs.push((win[0].y, win[1].x - win[0].x));
        }
    }
    pairs
}

#[test]
fn issue_6057_book_bracket_pua_expands_to_fullwidth_angle_brackets() {
    assert_eq!(map_pua_bullet_char('\u{F0854}'), '\u{300A}');
    assert_eq!(map_pua_bullet_char('\u{F0855}'), '\u{300B}');
    assert_eq!(
        expand_pua_render_text("법\u{F0855} 제"),
        "법》 제",
        "paint 경로가 U+F0855 를 전각 》 로 바꿔야 한다"
    );
}

#[test]
fn issue_6057_justified_je_keeps_fullwidth_advance_before_digits() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    let pairs = je_to_one_advances(&svg);
    assert!(
        pairs.len() >= 2,
        "1쪽에 '제'+'1' 쌍이 두 곳(본문 양쪽정렬 줄·통제군) 있어야 한다: {pairs:?}"
    );

    // 14pt @ 96dpi = 18.67px. 겹침 결함은 제→1 이 ~9.8px(0.5em).
    let min_fullwidth = 16.0;
    for (y, adv) in &pairs {
        assert!(
            *adv >= min_fullwidth,
            "y={y:.1} 제→1 전진 {adv:.2}px 가 전각(~18.7)보다 짧다 — \
             PUA 》 반각 측정으로 숫자가 '제' 위에 겹친다"
        );
    }
}
