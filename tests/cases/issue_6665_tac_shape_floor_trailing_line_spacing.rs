//! [Issue #6665] 도형 전용 줄(머리 상자) 다음 문단이 6.7px 위로 올라온다.
//!
//! 근인: `layout.rs` 의 TAC-Shape 높이 바닥값이 이겼을 때, 다음 문단을 놓는
//! 자리에 그 줄의 **꼬리 줄간격**이 빠졌다. 문단 앞 간격(`sb`)과 아래 간격
//! (`sa`)은 이미 더하고 있었으므로 항 하나만 빠진 상태였다. 일반 문단은
//! 마지막 줄을 `줄 상단 + lh + ls` 로 닫는다.
//!
//! 형제 결함: #5788 은 저장 lineseg 가 없는 문단에서 같은 `ls` 가 빠졌다.
//! 그쪽 주석이 "저장 lineseg 보유 문서는 seg.line_spacing 경로가 이미 싣는다"
//! 고 적었는데, 이 바닥값 경로에서는 싣지 않고 있었다.
//!
//! 실측(`3-09월_교육_통합_2024-구분선아래20.hwp` 4쪽 왼쪽 단): 본문 첫 글줄
//! baseline 이 137.0 — 한/글 PDF 는 143.8 이라 6.8px 차이. 수정 후 143.1.
//! 같은 쪽에서 14개 글줄이 +6.0~6.1px 내려가고 나머지는 그대로다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/3-09월_교육_통합_2024-구분선아래20.hwp";
/// 4쪽 왼쪽 단 x 대역. 오른쪽 단(x>390)을 뺀다.
const LEFT_COLUMN_X: std::ops::Range<f64> = 30.0..390.0;
/// 머리 상자 안의 글자. 이 줄이 도형 전용 줄이고, 그 다음 줄이 본문 첫 줄이다.
const SHAPE_LINE_TEXT: &str = "확률과통계";
/// 도형 줄 → 본문 첫 줄 간격의 허용 구간. 한/글 실측 34.5px 이고, 꼬리
/// 줄간격이 빠지면 28.4px 로 좁아진다.
const EXPECTED_GAP: std::ops::RangeInclusive<f64> = 33.0..=36.0;

/// SVG 의 한 글자짜리 `<text>` 를 baseline y 로 묶어 글줄을 만든다.
fn glyph_lines(svg: &str) -> BTreeMap<i64, (f64, String)> {
    let mut rows: BTreeMap<i64, Vec<(f64, char)>> = BTreeMap::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(head_end) = chunk.find('>') else {
            continue;
        };
        let (head, rest) = chunk.split_at(head_end);
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        let (Some(x), Some(y)) = (attr("x"), attr("y")) else {
            continue;
        };
        let body = rest.trim_start_matches('>');
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
            (key, (min_x, v.into_iter().map(|(_, c)| c).collect()))
        })
        .collect()
}

#[test]
fn issue_6665_paragraph_after_shape_only_line_keeps_trailing_line_spacing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(3).expect("page 4 svg");

    let column: Vec<(f64, String)> = glyph_lines(&svg)
        .into_iter()
        .map(|(key, (x, text))| (key as f64 / 10.0, x, text))
        .filter(|(_, x, _)| LEFT_COLUMN_X.contains(x))
        .map(|(y, _, text)| (y, text))
        .collect();

    let shape_idx = column
        .iter()
        .position(|(_, text)| text == SHAPE_LINE_TEXT)
        .expect("4쪽 왼쪽 단 머리 상자 글줄");
    let (shape_y, _) = column[shape_idx];
    let (body_y, body_text) = column
        .get(shape_idx + 1)
        .expect("머리 상자 다음 본문 글줄")
        .clone();
    let gap = body_y - shape_y;

    assert!(
        EXPECTED_GAP.contains(&gap),
        "머리 상자 줄({shape_y:.1}) → 본문 첫 줄({body_y:.1}) 간격이 {gap:.1}px 이다 \
         — 한/글 34.5px 기준 {EXPECTED_GAP:?} 안이어야 한다. 도형 전용 줄의 꼬리 \
         줄간격이 빠지면 28.4px 로 좁아진다 (#6665). 본문 글줄: {body_text}",
    );
}
