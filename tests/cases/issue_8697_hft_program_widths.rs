//! 관계형: HFT 프로그램(한양중고딕·신명 중고딕)이 직접 그리는 글자의 전진폭이 독립 한컴 Print 의 Type3 `/Widths` 와 같은 em 비율인지, 곧 같은 줄 앞뒤 글자 사이 간격이 글자 크기 × 폭/1000 인지 검사한다.
//! 주석 갱신: 2026-11-07T10:13:21+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-07 KST, base = #8685 head 34368de8cf, head = base + 이 PR 의 src 변경. 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw.
//! (1) 한양중고딕 탐침 4쪽=4쪽: 수정 전 Native 99.95·100.00·90.26·93.16 / fresh WASM 99.95·99.98·93.50·94.92 → 수정 후 Native 99.95·100.00·94.21·93.32 / fresh WASM 99.95·99.98·94.47·94.60(전쪽 최저 Native 93.32·WASM 94.47).
//! (2) 신명 중고딕 탐침 4쪽=4쪽: 수정 전 Native 86.93·87.02·79.32·83.71 / fresh WASM 89.24·89.30·84.22·86.52 → 수정 후 Native 99.84·99.95·90.67·93.21 / fresh WASM 99.85·99.94·93.32·94.80(전쪽 최저 Native 90.67·WASM 93.32).
//! 정식 TSV·PNG: `mydocs/pr/assets/issue_8697/{base,head}/{native,wasm}/hft_*_junggothic_probe_silhouette.tsv`, `png/probe-*`.
//! 같은 assertion 이 수정 전 FAIL(신명 중고딕 `-` 11.09px ≠ 6.67px, 한양중고딕 `‧` 4.00px ≠ 6.67px) / 수정 후 PASS.
//! 독립 기대값: `pdf/issue8697/hft-{hanyang,sinmyeong}_junggothic-probe-2020.pdf`(한컴 2020 11.0.0.9136, Print method 0)의 Type3 `/Widths`·ToUnicode. 입력은 `samples/lang_slot/gen_lang_slot_probe.py`(FTYPE=HFT, 일곱 슬롯 같은 글꼴)로 만든 `samples/issue8697/hft-*-probe.hwpx`.
//! 한/글이 이 프로그램 대신 TrueType 으로 대체해 그리는 글자는 검사하지 않는다(별도 축). `·` 는 layer SVG 가 원으로 합성해 전진을 읽을 수 없어 #7092 검사에 맡긴다.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다.
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

fn check(path: &str, expected: &[(char, u16)]) {
    let measured = advances(path);
    for &(ch, width) in expected {
        let hits: Vec<_> = measured.iter().filter(|(c, _, _)| *c == ch).collect();
        assert!(!hits.is_empty(), "{path}: {ch:?} 탐침 줄이 없다");
        for (_, advance, size) in hits {
            let want = size * f64::from(width) / 1000.0;
            assert!(
                (advance - want).abs() <= 0.05,
                "{path}: {ch:?} 전진 {advance:.3}px, Type3 {width}/1000 → {want:.3}px"
            );
        }
    }
}

#[test]
fn hanyang_junggothic_hft_draws_cjk_brackets_full_width() {
    check(
        "samples/issue8697/hft-hanyang_junggothic-probe.hwpx",
        &[
            ('〈', 1000),
            ('〉', 1000),
            ('《', 1000),
            ('「', 1000),
            ('】', 1000),
            ('〔', 1000),
            ('\u{3000}', 1000),
            ('…', 1000),
            ('※', 1000),
            ('‧', 500),
            ('‘', 225),
            ('“', 375),
            ('-', 518),
            ('A', 632),
        ],
    );
}

#[test]
fn sinmyeong_junggothic_hft_uses_its_own_latin_widths() {
    check(
        "samples/issue8697/hft-sinmyeong_junggothic-probe.hwpx",
        &[
            ('-', 500),
            ('4', 680),
            ('0', 680),
            ('A', 816),
            ('W', 1149),
            ('i', 323),
            ('m', 1050),
            ('~', 1000),
            ('‘', 200),
            ('“', 300),
            ('々', 1000),
        ],
    );
}
