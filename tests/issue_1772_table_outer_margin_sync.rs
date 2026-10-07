//! Issue #1772: HWPX 파서는 표 outMargin 을 common.margin 에도 동기화해야 한다 (IR 계약).
//!
//! 레이아웃의 쪽 고정 자리차지 표 예약 하단(calc_shape_bottom_y)과 HWPX→HWP 어댑터
//! (materialize_table_outer_margin)는 `table.common.margin` 을 기준으로 동작한다.
//! 파서가 `table.outer_margin_*` 만 채우면 HWPX 직파스 문서에서만 표 바깥 여백이
//! 무시되어 본문 첫 줄이 저장 lineseg(한컴 위치)보다 11.36px(3mm) 위로 붙는다.
//!
//! 재현 문서(samples/task1772/table_outer_margin_common_sync.hwpx, 36381023):
//! - 헤더 표: vert=Page + TopAndBottom, outMargin bottom=852(3mm)
//! - 본문 첫 줄은 헤더 표의 하단과 원본 바깥여백 뒤에서 시작한다.
//!   기준 한컴 PDF 1쪽 Visual Sweep은 100%이며, 절대 쪽 좌표는 검사하지 않는다.

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
    // 원본 표의 아래 여백을 본문 시작과 헤더 표 하단 사이에 예약해야 한다.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let source_margin_hu = load_ir()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Table(table) if table.outer_margin_bottom > 0 => {
                Some(i32::from(table.outer_margin_bottom))
            }
            _ => None,
        })
        .expect("헤더 표의 양의 아래 바깥여백");
    let source_margin =
        rhwp::renderer::hwpunit_to_px(source_margin_hu, rhwp::renderer::DEFAULT_DPI);
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree: {:?}", e));
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("parse tree json");

    fn has_text(v: &serde_json::Value, expected: &str) -> bool {
        v["text"]
            .as_str()
            .is_some_and(|text| text.contains(expected))
            || v["children"]
                .as_array()
                .is_some_and(|children| children.iter().any(|child| has_text(child, expected)))
    }

    fn collect<'a>(v: &'a serde_json::Value, out: &mut Vec<&'a serde_json::Value>) {
        if let Some(o) = v.as_object() {
            out.push(v);
            for c in o.values() {
                collect(c, out);
            }
        } else if let Some(a) = v.as_array() {
            for c in a {
                collect(c, out);
            }
        }
    }
    let mut nodes = Vec::new();
    collect(&json, &mut nodes);
    let header_table = nodes
        .iter()
        .filter(|node| node["type"] == "Table")
        .min_by(|a, b| {
            a["bbox"]["y"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["bbox"]["y"].as_f64().unwrap())
        })
        .expect("쪽 위의 결재 헤더 표");
    let body_line = nodes
        .iter()
        .find(|node| node["type"] == "TextLine" && has_text(node, "관련: 총무과"))
        .expect("결재 헤더 표 뒤의 첫 본문 문단");
    let header_bottom =
        header_table["bbox"]["y"].as_f64().unwrap() + header_table["bbox"]["h"].as_f64().unwrap();
    let first_body_y = body_line["bbox"]["y"].as_f64().unwrap();
    let margin_ratio = (first_body_y - header_bottom) / source_margin;
    assert!(
        (0.75..=1.25).contains(&margin_ratio),
        "첫 본문 문단은 헤더 표 하단 뒤에 원본 바깥여백을 두어야 함: 실제/원본 비율 {margin_ratio:.3}"
    );
}
