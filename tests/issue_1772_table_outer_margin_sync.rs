//! Issue #1772: HWPX 파서는 표 outMargin 을 common.margin 에도 동기화해야 한다 (IR 계약).
//!
//! 레이아웃의 쪽 고정 자리차지 표 예약 하단(calc_shape_bottom_y)과 HWPX→HWP 어댑터
//! (materialize_table_outer_margin)는 `table.common.margin` 을 기준으로 동작한다.
//! 파서가 `table.outer_margin_*` 만 채우면 HWPX 직파스 문서에서만 표 바깥 여백이
//! 무시되어 본문 첫 줄이 저장 lineseg(한컴 위치)보다 11.36px(3mm) 위로 붙는다.
//!
//! Regression shape (samples/task1772/table_outer_margin_common_sync.hwpx, 36381023):
//! - 헤더 표: vert=Page + TopAndBottom, outMargin bottom=852(3mm)
//! - 수정 전: 본문 pi=0 첫 줄 y=295.4px / 수정 후(=HWP5 재파스본과 동일): 306.7px
//!   (저장 lineseg vpos=17478 → 75.6+233.0 ≈ 306.7 정합)

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;

const SAMPLE: &str = "samples/task1772/table_outer_margin_common_sync.hwpx";

fn load_ir() -> rhwp::model::document::Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::parser::parse_document(&bytes).unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_1772_hwpx_table_common_margin_synced_with_outer_margin() {
    let doc = load_ir();
    let mut checked = 0usize;
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    assert_eq!(
                        (
                            t.common.margin.left,
                            t.common.margin.right,
                            t.common.margin.top,
                            t.common.margin.bottom,
                        ),
                        (
                            t.outer_margin_left,
                            t.outer_margin_right,
                            t.outer_margin_top,
                            t.outer_margin_bottom,
                        ),
                        "표 common.margin 은 outer_margin_* 과 동기 상태여야 한다 (IR 계약)"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(
        checked >= 2,
        "표 컨트롤이 최소 2개 있어야 한다: {}",
        checked
    );
}

#[test]
fn issue_1772_body_first_line_respects_table_outer_margin_bottom() {
    // 렌더 레벨 회귀: 본문 첫 줄이 저장 lineseg 위치(≈306.7px)에 있어야 한다.
    // 수정 전에는 표 아래 여백 3mm 누락으로 295.4px 에 렌더되었다.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree: {:?}", e));
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("parse tree json");

    // 본문 좌측(x≈75.6) TextLine 의 최소 y — 결재 헤더 표 아래 본문 첫 줄.
    fn collect(v: &serde_json::Value, out: &mut Vec<(f64, f64)>) {
        if let Some(o) = v.as_object() {
            if o.get("type").and_then(|t| t.as_str()) == Some("TextLine") {
                if let Some(b) = o.get("bbox") {
                    let x = b.get("x").and_then(|x| x.as_f64()).unwrap_or(-1.0);
                    let y = b.get("y").and_then(|y| y.as_f64()).unwrap_or(-1.0);
                    out.push((x, y));
                }
            }
            for c in o.values() {
                collect(c, out);
            }
        } else if let Some(a) = v.as_array() {
            for c in a {
                collect(c, out);
            }
        }
    }
    let mut lines = Vec::new();
    collect(&json, &mut lines);
    let first_body_y = lines
        .iter()
        .filter(|(x, _)| (x - 75.6).abs() < 0.2)
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min);
    assert!(
        (first_body_y - 306.7).abs() < 1.0,
        "본문 첫 줄 y 는 저장 lineseg 위치(≈306.7px)여야 한다 (수정 전 결함값 295.4): {:.2}",
        first_body_y
    );
}
