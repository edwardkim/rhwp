//! [Issue #6060] 전각 낫표 「(U+300C)의 블랭킷 반각 강제(`is_halfwidth_cjk_quote`)
//! 가 반증되는 문서군 — 한글 2020 은 휴먼명조 15.9pt 본문에서 15.2pt, HY헤드라인M
//! 30pt 제목에서 30.0pt 로 **전폭** 전진시키는데 rhwp 는 7.5/15.0pt 로 눌렀다
//! (`samples/issue6060/30307_local_service_reform.hwp`).
//!
//! 수정: 반각 강제를 실측 확인 폰트(바탕·함초롬 계열, #630 계보)로 한정하고
//! 그 외 폰트는 메트릭 DB 글리프 폭(전각)을 신뢰한다. Skia/web_canvas 의 0.5×
//! 글리프 축소도 측정이 실제 반각(<0.6em)일 때만 발동하도록 측정-종속으로 바꿨다.
//! 반각 낫표 ｢(U+FF62) 통제군(#6047)은 8.0pt 그대로다.
//!
//! 이 문서에는 별건 결함(#6087: 첫머리 빈 1쪽, 총 14쪽 vs 한글 13쪽)이 있어
//! 쪽 번호가 밀릴 수 있다 — 어서션은 쪽 스캔으로 행을 찾아 쪽수에 무관하다.
//!
//! 결함 상태에서는 제목 「 전진 20.0px(15pt)·본문 「 10.5px(7.9pt)로 두 밴드가
//! 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6060/30307_local_service_reform.hwp";

#[test]
fn issue_6060_fullwidth_corner_bracket_advances_match_hangul() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // HY헤드라인M 30pt 제목 '「지방현장 국민불편」' — 한글 30.0pt = 40.0px.
    let title = find_bracket_advance(&core, "지방현장국민불편")
        .expect("제목 '「지방현장 국민불편」' 행을 찾아야 한다");
    assert!(
        (38.0..=42.0).contains(&title),
        "HY헤드라인M 제목 「 전진이 한글(30.0pt=40px) 근방이어야 한다 (결함 시 20px): {title:.1}"
    );

    // 휴먼명조 15.9pt 본문 '…별지와 같이 「부패방지 및…' — 한글 15.2pt ≈ 20.3px.
    let body =
        find_bracket_advance(&core, "부패방지").expect("본문 '「부패방지 및…' 행을 찾아야 한다");
    assert!(
        (19.0..=22.5).contains(&body),
        "휴먼명조 본문 「 전진이 한글(15.2pt≈20.3px) 근방이어야 한다 (결함 시 10.5px): {body:.1}"
    );
}

/// 앞 6쪽을 스캔해 `「` 바로 뒤 문맥에 `context_needle` 이 이어지는 행을 찾아
/// 「 글리프의 실제 전진(px)을 돌려준다. 쪽 번호 밀림(#6087)과 무관하다.
fn find_bracket_advance(core: &DocumentCore, context_needle: &str) -> Option<f64> {
    for page in 0..6u32 {
        let Ok(svg) = core.render_page_svg_native(page) else {
            continue;
        };
        let mut glyphs: Vec<(f64, f64, String)> = Vec::new();
        for chunk in svg.split("<text").skip(1) {
            let Some(tag_end) = chunk.find('>') else {
                continue;
            };
            let head = &chunk[..tag_end];
            let (Some(x), Some(y)) = (attr(head, "x"), attr(head, "y")) else {
                continue;
            };
            let Some(close) = chunk[tag_end + 1..].find("</text>") else {
                continue;
            };
            glyphs.push((y, x, chunk[tag_end + 1..tag_end + 1 + close].to_string()));
        }
        // 같은 행(±1px)으로 묶고 x 정렬.
        glyphs.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).expect("finite"));
        let mut i = 0usize;
        while i < glyphs.len() {
            let row_y = glyphs[i].0;
            let mut row: Vec<(f64, &str)> = Vec::new();
            while i < glyphs.len() && (glyphs[i].0 - row_y).abs() <= 1.0 {
                row.push((glyphs[i].1, glyphs[i].2.as_str()));
                i += 1;
            }
            let joined: String = row.iter().map(|(_, t)| *t).collect();
            if !joined.contains('「') || !joined.contains(context_needle) {
                continue;
            }
            for (k, (x, t)) in row.iter().enumerate() {
                if t.contains('「') && k + 1 < row.len() {
                    return Some(row[k + 1].0 - x);
                }
            }
        }
    }
    None
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
