//! Issue #1073: 셀 안의 페이지보다 큰 중첩 표(nested table)가 페이지 경계에서 중첩 표
//! 행 단위로 분할되는지 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플): `samples/kps-ai.hwp` (HWP5).
//! 한컴 정답지: `pdf/kps-ai-2022.pdf` p62~63 — "소프트웨어사업 영향평가 결과서" 표를
//! 페이지에 걸쳐 행 단위로 분할.
//!
//! 결함 본질: 중첩 표가 is_row_splittable/cell_units/부분 렌더에서 atom 으로 취급되어
//! 외부 행이 통째 배치 → 758px overflow + 연속 페이지 전체 재렌더.
//! 정정: cell_units per-중첩행 유닛 분해 + NestedTableSplit 컷 구동 + 연속 페이지 rowspan
//! 라벨 공란화.
//!
//! pi=674 표가 걸치는 페이지(0-based global index): 65(첫 조각), 66(연속).

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn svg_height(svg: &str) -> f64 {
    let i = svg.find("height=\"").expect("svg height attr") + "height=\"".len();
    let rest = &svg[i..];
    let end = rest.find('"').expect("height close");
    rest[..end].parse().expect("height f64")
}

fn max_text_y(svg: &str) -> f64 {
    let mut max = 0.0_f64;
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        let tag_end = rest[open..]
            .find('>')
            .map(|g| open + g)
            .unwrap_or(rest.len());
        let tag = &rest[open..tag_end];
        if let Some(yi) = tag.find(" y=\"") {
            let yrest = &tag[yi + 4..];
            if let Some(ye) = yrest.find('"') {
                if let Ok(y) = yrest[..ye].parse::<f64>() {
                    max = max.max(y);
                }
            }
        }
        rest = &rest[tag_end..];
    }
    max
}

/// SVG 내 모든 `<text>` 내용을 이어붙인다(부분 문자열 검색용).
fn svg_text(svg: &str) -> String {
    let mut out = String::new();
    let mut rest = svg;
    while let Some(open) = rest.find("<text") {
        if let Some(gt) = rest[open..].find('>') {
            let after = &rest[open + gt + 1..];
            if let Some(close) = after.find("</text>") {
                out.push_str(&after[..close]);
                rest = &after[close + 7..];
                continue;
            }
        }
        break;
    }
    out
}

#[test]
fn kps_ai_nested_table_first_chunk_no_overflow() {
    let doc = load_doc("samples/kps-ai.hwp");
    let svg = doc.render_page_svg_native(65).expect("render page 65");
    let (h, max_y) = (svg_height(&svg), max_text_y(&svg));
    assert!(
        max_y <= h,
        "kps-ai page 65(첫 조각): text max_y={max_y:.1} > 페이지 높이 {h:.1} (중첩 표 미분할 회귀)"
    );
}

#[test]
fn kps_ai_nested_table_continuation_no_overflow() {
    let doc = load_doc("samples/kps-ai.hwp");
    let svg = doc.render_page_svg_native(66).expect("render page 66");
    let (h, max_y) = (svg_height(&svg), max_text_y(&svg));
    assert!(
        max_y <= h,
        "kps-ai page 66(연속): text max_y={max_y:.1} > 페이지 높이 {h:.1}"
    );
}

/// 분할이 실제로 일어나며(첫 조각에 표 제목 존재), 연속 페이지가 제목을 재렌더하지 않음
/// (전체 재렌더 중복 + rowspan 라벨 누수 회귀 차단).
#[test]
fn kps_ai_nested_table_split_no_title_duplication() {
    let doc = load_doc("samples/kps-ai.hwp");
    let first = svg_text(&doc.render_page_svg_native(65).expect("page 65"));
    let cont = svg_text(&doc.render_page_svg_native(66).expect("page 66"));
    const TITLE: &str = "소프트웨어사업";
    assert!(
        first.contains(TITLE),
        "첫 조각(page 65)에 표 제목 누락 — 분할 미발생 의심"
    );
    assert!(
        !cont.contains(TITLE),
        "연속(page 66)에 표 제목 재렌더 — 전체 재렌더 중복/rowspan 라벨 누수 회귀"
    );
}
