//! [Issue #6646] 묶음 빈칸 전진폭이 글꼴마다 달라 문항 번호 뒤 본문이 당겨진다.
//!
//! 근인: 묶음 빈칸(HWP5 문자 컨트롤 30 → `U+00A0`)을 일반 글자처럼 글꼴 글리프
//! 폭으로 재고 있었다. 묶음 빈칸은 **줄바꿈만 막는 공백**이라 전진폭이 일반 공백과
//! 같아야 하는데, 글꼴 표의 두 값이 제각각이다.
//!
//! `exam_eng.hwp` 1쪽 실측(글꼴 15.33px): 일반 공백은 어느 글꼴이든 7.667px 인데
//! 묶음 빈칸은 `Times New Roman` 3.827 · `HY신명조` 5.093 · `한양신명조` 7.747 이다.
//! 글꼴 표 자체도 567개 중 149개에서 두 값이 다르고 50개는 0 이다.
//!
//! 수정: `measure_char_width_embedded_decision` 에서 묶음 빈칸을 일반 공백과 같은
//! 갈래로 넣는다.
//!
//! 실측(`'.'` → `'대'` origin 간격): 수정 전 12.27px / 수정 후 16.27px /
//! 한/글 2022 PDF 16.01px.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/exam_eng.hwp";
/// 7번 문항 줄을 고르는 표식. 번호와 본문이 한 줄에 있다.
const LINE_PREFIX: &str = "7.";
const LINE_CONTAINS: &str = "대화를";
/// `'.'` → `'대'` 간격의 허용 구간. 한/글 16.01px 이고, 묶음 빈칸을 글꼴 글리프
/// 폭으로 재면 12.27px 로 좁아진다.
const EXPECTED_GAP: std::ops::RangeInclusive<f64> = 15.0..=17.0;

/// SVG 의 한 글자짜리 `<text>` 를 baseline 으로 묶어 글줄을 만든다.
/// 대부분의 글자는 `transform="translate(x,y)"` 로, 일부는 `x`/`y` 속성으로 놓인다.
fn glyph_lines(svg: &str) -> Vec<Vec<(f64, char)>> {
    let mut rows: std::collections::BTreeMap<i64, Vec<(f64, char)>> = Default::default();
    for chunk in svg.split("<text ").skip(1) {
        let Some(head_end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..head_end];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        let translated = head.find("translate(").map(|at| {
            let tail = &head[at + 10..];
            let end = tail.find(')').unwrap_or(tail.len());
            let mut parts = tail[..end].split(',');
            (
                parts.next().and_then(|v| v.trim().parse::<f64>().ok()),
                parts.next().and_then(|v| v.trim().parse::<f64>().ok()),
            )
        });
        let (Some(x), Some(y)) = translated.unwrap_or((attr("x"), attr("y"))) else {
            continue;
        };
        let body = &chunk[head_end + 1..];
        let Some(end) = body.find("</text>") else {
            continue;
        };
        let mut chars = body[..end].chars();
        let (Some(c), None) = (chars.next(), chars.next()) else {
            continue;
        };
        if c.is_whitespace() {
            continue;
        }
        rows.entry((y * 10.0).round() as i64)
            .or_default()
            .push((x, c));
    }
    rows.into_values()
        .map(|mut v| {
            v.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite x"));
            v
        })
        .collect()
}

#[test]
fn issue_6646_nbsp_advance_matches_plain_space() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    let line = glyph_lines(&svg)
        .into_iter()
        .find(|v| {
            let text: String = v.iter().map(|(_, c)| *c).collect();
            text.starts_with(LINE_PREFIX) && text.contains(LINE_CONTAINS)
        })
        .unwrap_or_else(|| panic!("1쪽에서 `{LINE_PREFIX}…{LINE_CONTAINS}` 글줄을 찾아야 한다"));

    let dot_x = line[1].0;
    let first_hangul_x = line
        .iter()
        .find(|(_, c)| *c == '대')
        .map(|(x, _)| *x)
        .expect("그 글줄의 `대`");
    let gap = first_hangul_x - dot_x;

    assert!(
        EXPECTED_GAP.contains(&gap),
        "`'.'` → `'대'` 간격이 {gap:.2}px 이다 — 한/글 16.01px 기준 {EXPECTED_GAP:?} \
         안이어야 한다. 묶음 빈칸을 글꼴 글리프 폭으로 재면 12.27px 로 좁아진다 (#6646)."
    );
}
