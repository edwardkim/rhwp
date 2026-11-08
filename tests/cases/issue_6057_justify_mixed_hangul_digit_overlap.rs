//! #6057: 독립 Print의 책괄호는 흰 겹낫표·반각이고 뒤 한글은 전각이다.
//! pdf/issue6057-29494-2020.pdf의 Haansoft Batang /W512·UPEM1024와
//! 원본 A854/A855 윤곽을 대조했다. 전각 《》 기대는 잘못된 표시 투영이었다.
//! C327 Native/fresh WASM 전2쪽 최저99.81353/99.82029%, 확대 판독 완료.
//! 1쪽 양쪽99.95%의 독립 책괄호 원점을±2CSSpx로 보호한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use rhwp::renderer::composer::expand_pua_render_text;
use rhwp::renderer::layout::map_pua_bullet_char;

const SAMPLE: &str = "samples/issue6057/29494.hwp";

#[derive(Clone, Copy)]
struct Glyph {
    x: f64,
    y: f64,
    ch: char,
    font_size: f64,
    fit_advance: Option<f64>,
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
        let font_size = attr_f64(attrs, "font-size").expect("실제 glyph의 글자 크기");
        out.push(Glyph {
            x,
            y,
            ch,
            font_size,
            fit_advance: attr_f64(attrs, "textLength"),
        });
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

fn je_to_one_advances(svg: &str) -> Vec<(f64, f64, f64)> {
    let glyphs = svg_glyphs(svg);
    let mut pairs = Vec::new();
    for win in glyphs.windows(2) {
        if win[0].ch == '제' && win[1].ch == '1' && (win[0].y - win[1].y).abs() < 1.0 {
            pairs.push((win[0].y, win[1].x - win[0].x, win[0].font_size));
        }
    }
    pairs
}

#[test]
fn issue_6057_book_bracket_pua_projects_white_corner_brackets() {
    assert_eq!(map_pua_bullet_char('\u{F0854}'), '\u{300E}');
    assert_eq!(map_pua_bullet_char('\u{F0855}'), '\u{300F}');
    assert_eq!(map_pua_bullet_char('《'), '《', "리터럴 겹화살괄호 보존");
    assert_eq!(map_pua_bullet_char('『'), '『', "리터럴 흰 겹낫표 보존");
    assert_eq!(
        expand_pua_render_text("법\u{F0855} 제"),
        "법』 제",
        "독립 Print의 기호 정체와 뒤 본문 순서를 보존한다"
    );
}

#[test]
fn issue_6057_justified_je_keeps_fullwidth_advance_before_digits() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("open");
    let svg = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .expect("page 1 Print SVG");

    let pairs = je_to_one_advances(&svg);
    assert!(
        pairs.len() >= 2,
        "1쪽에 '제'+'1' 쌍이 두 곳(본문 양쪽정렬 줄·통제군) 있어야 한다: {pairs:?}"
    );

    for (y, adv, font_size) in &pairs {
        assert!(
            *adv >= font_size * 0.9,
            "y={y:.1} 한글 '제'의 전각 점유 전에 숫자가 겹칩니다: {adv}/{font_size}"
        );
    }

    let glyphs = svg_glyphs(&svg);
    let brackets: Vec<_> = glyphs
        .iter()
        .filter(|glyph| matches!(glyph.ch, '『' | '』'))
        .collect();
    assert_eq!(brackets.len(), 2, "원본 책괄호 한 쌍의 누락·중복 방지");
    assert_eq!(
        (brackets[0].ch, brackets[1].ch),
        ('『', '』'),
        "앞뒤 줄 소속 순서"
    );
    for (glyph, (x, y)) in brackets
        .iter()
        .zip([(503.08732, 370.95459), (234.11312, 404.51790)])
    {
        let advance = glyph.fit_advance.expect("원본 반각 전진을 SVG도 소비");
        assert!((advance - glyph.font_size * 0.5).abs() <= 96.0 / 7200.0);
        // 독립 PDF의 실제 text origin(pt×96/72). 출력에서 복사한 좌표가 아니다.
        assert!(
            (glyph.x - x).abs() <= 2.0 && (glyph.y - y).abs() <= 2.0,
            "98% 이상 완료 쪽의 독립 책괄호 원점: ({},{}) / ({x},{y})",
            glyph.x,
            glyph.y
        );
    }
    let closing = brackets[1];
    let following = glyphs
        .iter()
        .find(|glyph| glyph.ch == '제' && (glyph.y - closing.y).abs() < 1.0)
        .expect("닫는 책괄호 뒤 같은 줄의 법조문");
    assert!(
        following.x >= closing.x + closing.fit_advance.unwrap(),
        "책괄호 뒤 한글의 포함·겹침 보존"
    );
}
