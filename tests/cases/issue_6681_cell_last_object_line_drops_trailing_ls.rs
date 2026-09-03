//! [Issue #6681] 칸을 닫고 다음 요소로 가는 전진량이 6px 과다하다.
//!
//! 근인: `#5923` 이 "칸 마지막 줄의 꼬리 줄간격은 제외" 를 세우면서, 글자처럼
//! 취급 표의 다문단 칸만 예외로 남겼다. 그 예외는 원리가 아니라 당시 보존 핀
//! (KTX TOC 등)에 맞춘 조건이다.
//!
//! `exam_science.hwp` 4쪽 `자료` 칸의 마지막 문단은 **글자 없이 안쪽 표만 담은
//! 줄**이다.
//!
//! ```text
//! p[15] ctrls=1 text_len=0  ls[0] vpos=29580 lh=3037 ls=460
//! p[15] 내부표: 2행×3열   (칸 h=1424 + 1613 = 3037HU)
//! ```
//!
//! `lh` 가 안쪽 표 높이와 같고 `ls=460`(6.1px)이 그 예외로 칸 높이에 들어가,
//! 그 아래 흐름이 통째로 6px 밀렸다.
//!
//! 실측(왼쪽 단 글줄을 한/글과 짝지은 편차):
//!   수정 전  644.3 (+0.6) → 704.4 (**−5.4**) → 736.3 (−5.3) → 903.4 (−4.4)
//!   수정 후  644.3 (+0.6) → 698.3 (**+0.7**) → 730.2 (+0.8) → 897.3 (+1.7)
//!
//! 그런 줄의 높이는 개체가 차지한 자리이고 뒤에 붙일 줄이 없다. 보존 핀의
//! 마지막 문단은 글자가 있어 종전 회계를 그대로 쓴다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/exam_science.hwp";
/// 4쪽 왼쪽 단 x 대역. 단 경계는 렌더 트리 기준 70.7~493.3 이다.
const LEFT_COLUMN_X: std::ops::Range<f64> = 68.0..495.0;
/// 문제의 표 아래 첫 본문 글줄에 들어 있는 글자들. 이 줄이 6px 밀렸다.
const ANCHOR_TEXT: &str = "일정하고";
/// 그 글줄 baseline 의 허용 구간. 한/글 699.0px, 수정 전에는 704.4px 였다.
const EXPECTED_BASELINE: std::ops::RangeInclusive<f64> = 696.0..=701.0;

/// SVG 의 한 글자짜리 `<text>` 를 baseline 으로 묶어 글줄을 만든다.
fn glyph_lines(svg: &str) -> Vec<(f64, f64, String)> {
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
        // 대부분의 글자는 `transform="translate(x,y) …"` 로 놓인다. `x`/`y` 속성을
        // 쓰는 글자도 섞여 있어 둘 다 받는다.
        let translated = head.find("translate(").map(|at| {
            let tail = &head[at + 10..];
            let end = tail.find(')').unwrap_or(tail.len());
            let mut parts = tail[..end].split(',');
            (
                parts.next().and_then(|v| v.trim().parse::<f64>().ok()),
                parts.next().and_then(|v| v.trim().parse::<f64>().ok()),
            )
        });
        let (Some(x), Some(y)) = translated
            .map(|(a, b)| (a, b))
            .unwrap_or((attr("x"), attr("y")))
        else {
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
    rows.into_iter()
        .map(|(key, mut v)| {
            v.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite x"));
            let min_x = v[0].0;
            (
                key as f64 / 10.0,
                min_x,
                v.into_iter().map(|(_, c)| c).collect(),
            )
        })
        .collect()
}

#[test]
fn issue_6681_object_only_last_line_does_not_add_trailing_line_spacing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(3).expect("page 4 svg");

    let line = glyph_lines(&svg)
        .into_iter()
        .find(|(_, x, text)| LEFT_COLUMN_X.contains(x) && text.contains(ANCHOR_TEXT))
        .unwrap_or_else(|| panic!("4쪽 왼쪽 단에서 `{ANCHOR_TEXT}` 글줄을 찾아야 한다"));

    assert!(
        EXPECTED_BASELINE.contains(&line.0),
        "표 아래 첫 본문 글줄 baseline 이 {:.1} 이다 — 한/글 699.0px 기준 \
         {EXPECTED_BASELINE:?} 안이어야 한다. 글자 없이 개체만 담은 칸 마지막 줄의 \
         꼬리 줄간격(6.1px)을 칸 높이에 넣으면 704.4 로 밀린다 (#6681). 글줄: {}",
        line.0,
        line.2,
    );
}
