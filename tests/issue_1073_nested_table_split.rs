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
//! pi=674 표가 걸치는 페이지(0-based global index): `SPLIT_FIRST_PAGE`(첫 조각),
//! 그 다음 쪽(연속).
//!
//! [#5910] 43쪽 병합 셀 선언 높이 보정으로 이 문서가 78쪽 → **77쪽**(한글 2022 정본과
//! 일치)이 되면서 뒤쪽 쪽 번호가 한 칸씩 당겨졌다. 계약(첫 조각에 표 제목 있음 /
//! 연속 쪽에 없음)은 그대로고 인덱스만 65·66 → **64·65** 로 옮겼다. 실측: 제목
//! `소프트웨어사업` 이 있는 쪽이 수정 전 [65, 67] → 수정 후 [64, 66] 이라 첫 조각
//! 다음 쪽에 제목이 없다는 계약이 양쪽에서 같은 형태로 성립한다.

use std::fs;
use std::path::Path;

/// 중첩 표가 처음 걸치는 쪽(0-based). 문서 쪽수가 바뀌면 여기만 고치면 된다.
const SPLIT_FIRST_PAGE: u32 = 64;
/// 그 표가 이어지는 다음 쪽.
const SPLIT_CONT_PAGE: u32 = SPLIT_FIRST_PAGE + 1;

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
    let svg = doc
        .render_page_svg_native(SPLIT_FIRST_PAGE)
        .expect("render 첫 조각 쪽");
    let (h, max_y) = (svg_height(&svg), max_text_y(&svg));
    assert!(
        max_y <= h,
        "kps-ai {SPLIT_FIRST_PAGE}쪽(첫 조각): text max_y={max_y:.1} > 페이지 높이 {h:.1} (중첩 표 미분할 회귀)"
    );
}

#[test]
fn kps_ai_nested_table_continuation_no_overflow() {
    let doc = load_doc("samples/kps-ai.hwp");
    let svg = doc
        .render_page_svg_native(SPLIT_CONT_PAGE)
        .expect("render 연속 쪽");
    let (h, max_y) = (svg_height(&svg), max_text_y(&svg));
    assert!(
        max_y <= h,
        "kps-ai {SPLIT_CONT_PAGE}쪽(연속): text max_y={max_y:.1} > 페이지 높이 {h:.1}"
    );
}

/// 분할이 실제로 일어나며(첫 조각에 표 제목 존재), 연속 페이지가 제목을 재렌더하지 않음
/// (전체 재렌더 중복 + rowspan 라벨 누수 회귀 차단).
#[test]
fn kps_ai_nested_table_split_no_title_duplication() {
    let doc = load_doc("samples/kps-ai.hwp");
    let first = svg_text(
        &doc.render_page_svg_native(SPLIT_FIRST_PAGE)
            .expect("첫 조각 쪽"),
    );
    let cont = svg_text(
        &doc.render_page_svg_native(SPLIT_CONT_PAGE)
            .expect("연속 쪽"),
    );
    const TITLE: &str = "소프트웨어사업";
    assert!(
        first.contains(TITLE),
        "첫 조각({SPLIT_FIRST_PAGE}쪽)에 표 제목 누락 — 분할 미발생 의심"
    );
    assert!(
        !cont.contains(TITLE),
        "연속({SPLIT_CONT_PAGE}쪽)에 표 제목 재렌더 — 전체 재렌더 중복/rowspan 라벨 누수 회귀"
    );
}
