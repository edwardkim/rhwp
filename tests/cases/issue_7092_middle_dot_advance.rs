//! [#7092] 가운뎃점 `·`(U+00B7)의 전진폭을 글꼴 실제값 대신 0.3em 으로 덮어쓴다.
//!
//! ## 증상
//!
//! 한/글은 글꼴이 지닌 전진폭을 그대로 쓴다. HY신명조처럼 이 글자를 전각으로 지닌
//! 글꼴에서는 점 하나마다 뒤 글자가 0.70em 씩 왼쪽으로 밀리고 줄 끝이 그만큼 짧아진다.
//!
//! ## 근인
//!
//! `text_measurement.rs` 의 `is_b7_notdef_artifact` 가 "메트릭이 전각이면 `.notdef`
//! 상자" 로 보고 0.3em 을 씌웠다. 메트릭 DB 는 **결측 글리프를 0 으로** 적으므로 전각으로
//! 적힌 값은 대개 실제 전진폭이다 — `H2MJSM.TTF`(HY신명조)의 `·` 는 `.notdef`(gid 0)가
//! 아니라 `periodcentered`(gid 20313)이고 advance 가 `1024/1024` 다.
//!
//! ## 푸는 범위 — 정본이 입증한 곳만
//!
//! 표를 믿는 경우는 **TTF 로 선언되고 대체 규칙이 이름을 바꾸지 않은 글꼴**뿐이다.
//!
//! - 재현체(`hwp2024Convert` engine 2020): `·` 31회가 전부 `HY신명조`(TTF)이고 정본
//!   전진폭 **0.999em**. 종전 rhwp 0.299em.
//! - `jubo_20260104.hwp` 점선 리더는 HFT `신명 신신명조` 가 `HY신명조` 로 **대체된** run
//!   이다. 26점이 150.6px 칸에 들어가니(전각이면 381px) 좁은 값이 옳다.
//!
//! 대체 안 된 HFT 가 전각이라는 정본은 아직 없어 그 경우는 종전대로 좁힌다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::paint::RenderProfile;
use rhwp::DocumentCore;

const FIXTURE: &[u8] = include_bytes!("../fixtures/issue_7092/soil-quarry-permit-standard.hwp");
const JUBO: &str = "samples/issues/2809/jubo_20260104.hwp";

/// layer SVG 는 글자마다 `<text>` 하나를 낸다. `·` 는 합성 `<circle>` 로 그려지고
/// 위치용 `<text fill-opacity="0">` 이 따라오므로, 그 `x`/`y`/`font-size` 를 읽는다.
#[derive(Debug, Clone)]
struct Glyph {
    x: f64,
    y: f64,
    size: f64,
    text: String,
}

fn attr(tag: &str, key: &str) -> Option<f64> {
    let at = tag.find(&format!(" {key}=\""))? + key.len() + 3;
    let rest = &tag[at..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

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
        let text = &body[..close];
        if let (Some(x), Some(y), Some(size)) =
            (attr(tag, "x"), attr(tag, "y"), attr(tag, "font-size"))
        {
            if text.chars().count() == 1 && !text.trim().is_empty() {
                out.push(Glyph {
                    x,
                    y,
                    size,
                    text: text.to_string(),
                });
            }
        }
        rest = &body[close..];
    }
    out
}

/// 같은 줄에서 `·` 다음 글자까지의 전진폭을 em 단위로 모은다.
fn dot_advances_em(svg: &str) -> Vec<f64> {
    let mut g = glyphs(svg);
    g.sort_by(|a, b| {
        (a.y * 10.0)
            .round()
            .total_cmp(&(b.y * 10.0).round())
            .then(a.x.total_cmp(&b.x))
    });
    g.windows(2)
        .filter(|w| w[0].text == "\u{00B7}" && (w[0].y - w[1].y).abs() < 0.5 && w[0].size > 0.0)
        .map(|w| (w[1].x - w[0].x) / w[0].size)
        .filter(|adv| *adv > 0.0)
        .collect()
}

/// 같은 줄에서 `·` → `·` 간격(점선 리더의 피치)만 em 단위로 모은다.
///
/// 리더의 마지막 점 다음 글자는 배분 정렬로 옆 칸까지 떨어져 있어 전진폭이 아니다.
fn dot_pitches_em(svg: &str) -> Vec<f64> {
    let mut g = glyphs(svg);
    g.sort_by(|a, b| {
        (a.y * 10.0)
            .round()
            .total_cmp(&(b.y * 10.0).round())
            .then(a.x.total_cmp(&b.x))
    });
    g.windows(2)
        .filter(|w| {
            w[0].text == "\u{00B7}"
                && w[1].text == "\u{00B7}"
                && (w[0].y - w[1].y).abs() < 0.5
                && w[0].size > 0.0
        })
        .map(|w| (w[1].x - w[0].x) / w[0].size)
        .filter(|adv| *adv > 0.0)
        .collect()
}

fn page_svg(core: &DocumentCore, page: u32) -> String {
    core.render_page_svg_layer_with_profile_native(page, RenderProfile::Print)
        .expect("SVG")
}

fn open_sample(path: &str) -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(full).expect("read sample")).expect("open sample")
}

#[test]
fn middle_dot_keeps_the_font_declared_full_width_advance() {
    let core = DocumentCore::from_bytes(FIXTURE).expect("별표 문서를 열지 못했다");
    let advances = dot_advances_em(&page_svg(&core, 0));
    assert!(
        advances.len() >= 10,
        "1쪽에서 `·` 가 10회 이상 나와야 한다 — 재현체가 바뀌었다 ({}회)",
        advances.len()
    );
    for adv in &advances {
        assert!(
            (adv - 1.0).abs() < 0.05,
            "HY신명조의 `·` 전진폭은 글꼴 선언값 1.000em 이어야 한다 (정본 0.999em) — \
             {adv:.3}em. 0.3em 이면 `.notdef` 덮어쓰기가 되살아난 것이다. 전체: {advances:?}"
        );
    }
}

#[test]
fn substituted_font_middle_dot_keeps_the_narrow_advance() {
    // HFT `신명 신신명조` → `HY신명조` 대체 run. 26점이 150.6px 칸에 들어가야 한다.
    let core = open_sample(JUBO);
    let advances = dot_pitches_em(&page_svg(&core, 1));
    assert!(
        advances.len() >= 20,
        "2쪽 점선 리더의 `·`→`·` 간격이 20회 이상 나와야 한다 ({}회)",
        advances.len()
    );
    let wide = advances.iter().filter(|adv| **adv > 0.6).count();
    assert_eq!(
        wide, 0,
        "대체된 글꼴의 `·` 는 빌려 온 표의 전각을 쓰면 안 된다 — 0.6em 초과 {wide}회: {advances:?}"
    );
}
