//! 관계형 회귀 — 어울림(Square) 그림 문단의 글 줄은 저장 `cs`·`sw`를 그대로 쓴다.
//! 주석 갱신: 2026-11-07 09:00 KST(+09:00).
//! 측정일: 2026-11-07. raw = `silhouette_raw_match_percent`, 96 DPI, 독립 한컴 Print PDF.
//! 측정 값은 아래 표에 원본별로 적고, 정식 증거는 `mydocs/pr/assets/issue_square_margin/`에 둔다.
//!
//! ```text
//! 원본                                        쪽  수정 전 Native  수정 후 Native  fresh WASM  전쪽 최저 N/W
//! samples/hwp3-sample5.hwp (64쪽)             22  97.90           100.00          WASM_S5_22  WASM_S5_MIN
//! samples/hwp3-sample5-hwp5.hwp (64쪽)        22  92.37           97.05           WASM_H5_22  WASM_H5_MIN
//! samples/issue6204/square_picture_band_host   1  80.82           92.15           WASM_B6_1   WASM_B6_MIN
//! ```
//!
//! 저장 줄의 시작 위치는 그림 왼쪽 바깥 여백·폭·오른쪽 바깥 여백을 모두 지난 자리다. 한컴이
//! 저장한 HWP5·HWPX 코퍼스와 `hwp3-sample5.hwp`의 한컴 변환본이 이를 보여 준다. 그래서
//! 배치에서 여백을 다시 더하면 그림 옆 줄, 그림 아래 전폭 줄, 그림이 오른쪽에 있어 `cs=0`인
//! 줄이 모두 여백만큼 밀린다. HWP3 파서는 왼쪽 여백을 빼고 있었고, 샘플의 좌우 여백이 같아
//! 배치의 가산이 그 부족분을 우연히 메웠다. 그래서 파서 값을 변환본과 직접 대조한다.
//!
//! 기대값은 구현 계산이 아니라 저장 값과 같은 쪽의 단 왼쪽 끝에서 얻는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::TextWrap;
use serde_json::Value;
use std::fs;
use std::path::Path;

const TOLERANCE_PX: f64 = 0.5;
const HWP3: &str = "samples/hwp3-sample5.hwp";
const HANCOM_HWP5: &str = "samples/hwp3-sample5-hwp5.hwp";
const HANCOM_HWPX: &str = "samples/hwp3-sample5-hwpx.hwpx";
const RIGHT_PICTURE: &str = "samples/issue6204/square_picture_band_host.hwp";

fn bytes(sample: &str) -> Vec<u8> {
    fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .unwrap_or_else(|e| panic!("read {sample}: {e}"))
}

fn document(sample: &str) -> Document {
    rhwp::parser::parse_document(&bytes(sample)).unwrap_or_else(|e| panic!("parse {sample}: {e}"))
}

fn page_tree(sample: &str, page_index: u32) -> Value {
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes(sample))
        .unwrap_or_else(|e| panic!("open {sample}: {e:?}"));
    let json = document
        .get_page_render_tree(page_index)
        .unwrap_or_else(|e| panic!("{sample} {}쪽: {e:?}", page_index + 1));
    serde_json::from_str(&json).expect("render tree json")
}

fn collect<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        out.push(node);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect(child, kind, out);
    }
}

fn bbox(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().expect("bbox 좌표")
}

fn column_left(tree: &Value) -> f64 {
    let mut columns = Vec::new();
    collect(tree, "Column", &mut columns);
    assert!(!columns.is_empty(), "쪽에 단이 있어야 한다");
    bbox(columns[0], "x")
}

/// 문단의 렌더 줄 시작을 왼쪽부터 중복 없이 돌려준다.
fn line_starts(tree: &Value, para_index: u64) -> Vec<f64> {
    let mut lines = Vec::new();
    collect(tree, "TextLine", &mut lines);
    let mut starts: Vec<f64> = lines
        .iter()
        .filter(|line| line["pi"].as_u64() == Some(para_index))
        .map(|line| bbox(line, "x"))
        .collect();
    starts.sort_by(f64::total_cmp);
    starts.dedup_by(|a, b| (*a - *b).abs() < TOLERANCE_PX);
    starts
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn square_picture_paragraphs(paragraphs: &[Paragraph]) -> Vec<usize> {
    paragraphs
        .iter()
        .enumerate()
        .filter(|(_, para)| {
            para.controls.iter().any(|control| {
                matches!(control, Control::Picture(picture)
                    if !picture.common.treat_as_char
                        && picture.common.text_wrap == TextWrap::Square
                        && picture.common.margin.right > 0)
            })
        })
        .map(|(index, _)| index)
        .collect()
}

fn widest_start(para: &Paragraph) -> (i32, i32) {
    para.line_segs
        .iter()
        .map(|seg| (seg.column_start, seg.segment_width))
        .max_by_key(|(start, _)| *start)
        .expect("저장 줄")
}

/// HWP3 파서의 어울림 줄은 같은 원본의 한컴 변환본과 같아야 한다. 좌우 여백이 같은
/// 샘플이라 배치의 가산이 파서의 부족분을 가리던 자리를 저장 값으로 직접 본다.
#[test]
fn hwp3_square_line_box_matches_the_hancom_conversion() {
    let hwp3 = document(HWP3);
    for converted_sample in [HANCOM_HWP5, HANCOM_HWPX] {
        let converted = document(converted_sample);
        let targets = square_picture_paragraphs(&converted.sections[0].paragraphs);
        let compared: Vec<usize> = targets
            .into_iter()
            .filter(|&pi| widest_start(&converted.sections[0].paragraphs[pi]).0 > 0)
            .collect();
        assert!(
            compared.len() >= 5,
            "{converted_sample}: 왼쪽 그림 어울림 문단이 비교 대상이어야 한다: {compared:?}"
        );
        for pi in compared {
            assert_eq!(
                widest_start(&hwp3.sections[0].paragraphs[pi]),
                widest_start(&converted.sections[0].paragraphs[pi]),
                "pi={pi}: HWP3 파서 (cs, sw) 는 {converted_sample} 저장 값과 같아야 한다"
            );
        }
    }
}

/// 왼쪽 그림: 좁은 줄은 단 왼쪽 + 저장 `cs`, 그림 아래 전폭 줄은 단 왼쪽에서 시작한다.
#[test]
fn left_picture_lines_start_at_their_stored_positions() {
    let converted = document(HANCOM_HWP5);
    for (sample, page_index, para_index) in [
        (HANCOM_HWP5, 3, 75_usize),
        (HANCOM_HWP5, 21, 605),
        (HWP3, 3, 75),
        (HWP3, 21, 605),
    ] {
        let tree = page_tree(sample, page_index);
        let left = column_left(&tree);
        let (stored_cs, _) = widest_start(&converted.sections[0].paragraphs[para_index]);
        assert!(stored_cs > 0, "pi={para_index}: 좁은 저장 줄이 있어야 한다");
        let starts = line_starts(&tree, para_index as u64);
        assert_eq!(
            starts.len(),
            2,
            "{sample} pi={para_index}: 전폭 줄과 좁은 줄이 함께 있어야 한다: {starts:?}"
        );
        assert!(
            (starts[0] - left).abs() < TOLERANCE_PX,
            "{sample} pi={para_index}: 전폭 줄 {} 은 단 왼쪽 {left} 이어야 한다",
            starts[0]
        );
        assert!(
            (starts[1] - (left + hu_to_px(stored_cs))).abs() < TOLERANCE_PX,
            "{sample} pi={para_index}: 좁은 줄 {} 은 단 왼쪽 + cs {stored_cs} 이어야 한다",
            starts[1]
        );
    }
}

/// 반례 — 그림이 오른쪽에 있으면 저장 `cs=0`이라 왼쪽 시작은 그림과 무관하다.
/// 줄 끝은 저장 `sw` 그대로 그림 왼쪽에서 끝난다.
#[test]
fn right_picture_keeps_the_left_edge_and_stored_line_end() {
    let source = document(RIGHT_PICTURE);
    let seg = &source.sections[0].paragraphs[1].line_segs[0];
    assert_eq!(seg.column_start, 0, "반례는 cs=0 줄이어야 한다");
    let tree = page_tree(RIGHT_PICTURE, 0);
    let left = column_left(&tree);
    let mut lines = Vec::new();
    collect(&tree, "TextLine", &mut lines);
    let first = lines
        .iter()
        .find(|line| line["pi"].as_u64() == Some(1))
        .expect("pi=1 첫 줄");
    assert!(
        (bbox(first, "x") - left).abs() < TOLERANCE_PX,
        "그림 옆 왼쪽 시작 {} 은 단 왼쪽 {left} 이어야 한다",
        bbox(first, "x")
    );
    let right = bbox(first, "x") + bbox(first, "w");
    assert!(
        (right - (left + hu_to_px(seg.segment_width))).abs() < TOLERANCE_PX,
        "줄 끝 {right} 은 단 왼쪽 + sw {} 이어야 한다",
        seg.segment_width
    );
}
