//! Issue #4698: 쪽 경계에 걸친 병합(rowspan) 셀의 문단이 조각별로 나뉘어 소유되는지 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플): `samples/kps-ai.hwp` (HWP5).
//! 한컴 정답지: `pdf/kps-ai-2022.pdf` — `- 62 -` 쪽 하단 라벨은 "3. 민간 / 소프트웨어" 까지,
//! 이어지는 `- 63 -` 쪽 상단이 "시장침해 / 가능성" 을 받는다.
//! (0-based 페이지 인덱스로는 각각 65, 66)
//!
//! 결함 본질: 셀[39](r=18, rs=6)은 기하만 두 조각으로 나뉘고 **내용은 나뉘지 않았다**.
//! 앞 조각이 문단 5 개 전부를 받아 조각 하단(1033.8px)을 넘긴 줄이 Cell clip 으로 사라지고,
//! 뒤 조각은 [#1073] "라벨은 앞 쪽에 이미 렌더됨" 가정으로 통째 공란화되어 `시장침해` 4 자가
//! 어느 쪽에도 남지 않았다.
//! 정정: 저장 `LINE_SEG.vpos` 재시작(= 한컴 자신의 조각 경계)대로 문단을 조각에 배분한다.
//!
//! SVG 는 글리프마다 `<text>` 를 방출하므로 문자열 검색만으로는 같은 쪽의 다른 셀("민간
//! 소프트웨어 시장 침해 가능성 없음")과 구분되지 않는다. 그래서 좌표로 판정한다 — 이 라벨
//! 열은 x < 175px 이고, 조각 경계는 앞 쪽 셀 하단 1033.8px / 뒤 쪽 셀 상단 134.2px 이다.

use std::fs;
use std::path::Path;

/// 라벨 열 우측 경계(px) — 이 안쪽 글리프만 병합 라벨 셀 소속이다.
const LABEL_COLUMN_RIGHT: f64 = 175.0;
/// 앞 조각 라벨 셀 하단(px).
const FIRST_FRAGMENT_CELL_BOTTOM: f64 = 1033.8;
/// 뒤 조각 라벨 셀 상단 + 한 줄 여유(px).
const CONTINUATION_LABEL_TOP_BAND: f64 = 200.0;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

/// `<text ... translate(x,y) ...>내용</text>` 을 (x, y, 내용) 으로 훑는다.
fn glyphs(svg: &str) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let Some(gt) = rest[open..].find('>') else {
            break;
        };
        let tag = &rest[open..open + gt];
        let after = &rest[open + gt + 1..];
        let Some(close) = after.find("</text>") else {
            break;
        };
        let text = after[..close].to_string();
        rest = &after[close + "</text>".len()..];
        let Some(tr) = tag.find("translate(") else {
            continue;
        };
        let inner = &tag[tr + "translate(".len()..];
        let Some(end) = inner.find(')') else { continue };
        let mut parts = inner[..end].split(',');
        let (Some(x), Some(y)) = (parts.next(), parts.next()) else {
            continue;
        };
        if let (Ok(x), Ok(y)) = (x.trim().parse::<f64>(), y.trim().parse::<f64>()) {
            out.push((x, y, text));
        }
    }
    out
}

/// 라벨 열(x < 175) 안에서 주어진 y 구간에 있는 글자들.
fn label_column_chars(svg: &str, y_min: f64, y_max: f64) -> String {
    glyphs(svg)
        .into_iter()
        .filter(|(x, y, _)| *x < LABEL_COLUMN_RIGHT && *y >= y_min && *y <= y_max)
        .map(|(_, _, t)| t)
        .collect::<String>()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// 앞 조각은 자기 몫(`3. 민간 / 소프트웨어`)만 받고, 셀 하단을 넘겨 그리지 않는다.
#[test]
fn first_fragment_keeps_only_its_own_paragraphs() {
    let doc = load_doc("samples/kps-ai.hwp");
    let svg = doc.render_page_svg_native(65).expect("render page 65");

    let own = label_column_chars(&svg, 900.0, FIRST_FRAGMENT_CELL_BOTTOM);
    assert!(
        own.contains("민간") && own.contains("소프트웨어"),
        "앞 조각(page 65) 라벨 셀에 `3. 민간 / 소프트웨어` 누락 — 조각 배분 회귀 (실측: {own:?})"
    );

    let overflow = label_column_chars(&svg, FIRST_FRAGMENT_CELL_BOTTOM, f64::MAX);
    assert!(
        overflow.is_empty(),
        "앞 조각(page 65)이 다음 쪽 소유 문단을 셀 하단 아래로 흘려보냈다 (실측: {overflow:?})"
    );
}

/// 뒤 조각은 공란이 아니라 `시장침해 / 가능성` 을 상단에서 받는다.
#[test]
fn continuation_fragment_receives_the_remaining_paragraphs() {
    let doc = load_doc("samples/kps-ai.hwp");
    let svg = doc.render_page_svg_native(66).expect("render page 66");

    let top = label_column_chars(&svg, 0.0, CONTINUATION_LABEL_TOP_BAND);
    assert!(
        top.contains("시장침해"),
        "연속(page 66) 라벨 셀 상단이 `시장침해` 를 받지 못했다 — \
         병합 라벨 공란화(#1073) 회귀로 4 자가 어느 쪽에도 남지 않는다 (실측: {top:?})"
    );
    assert!(
        top.contains("가능성"),
        "연속(page 66) 라벨 셀 상단에 `가능성` 누락 (실측: {top:?})"
    );
}
