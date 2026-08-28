//! [#5372] `layout-anomaly` text-overlap 판정·봉투 계약.
//!
//! 합성 렌더 트리로 글자끼리의 bbox 교차를 고정하고, 자기서술·`--json` 필드·
//! 기본 exit 0 도 닫는다. 레이아웃 엔진은 건드리지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions, DocAnomalies, PageAnomalies};
use rhwp::renderer::render_tree::{
    BoundingBox, RenderNode, RenderNodeType, TableCellNode, TableNode, TextLineNode, TextRunNode,
};

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwp3-sample.hwp")
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

#[test]
fn json_envelope_declares_text_overlap_count_and_stays_exit_zero() {
    let src = sample();
    let args = ["layout-anomaly", src.as_str(), "--json"];
    let out = run(&args);
    assert_eq!(
        out.status.code(),
        Some(0),
        "기본은 text-overlap 포함 이상 신호가 있어도 0 이다. stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(
        v.get("textOverlapCount").and_then(|x| x.as_u64()).is_some(),
        "textOverlapCount 누락: {v}"
    );
    assert!(v["pages"].is_array(), "{v}");
    for page in v["pages"].as_array().unwrap() {
        assert!(
            page.get("textOverlap").is_some(),
            "pages[].textOverlap 누락: {page}"
        );
    }
}

#[test]
fn capabilities_lists_text_overlap_count() {
    let v: serde_json::Value =
        serde_json::from_slice(&run(&["capabilities"]).stdout).expect("capabilities");
    let entry = v["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "layout-anomaly")
        .expect("layout-anomaly");
    let fields: Vec<&str> = entry["recordFields"]
        .as_array()
        .expect("recordFields")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(
        fields.contains(&"textOverlapCount"),
        "recordFields 에 textOverlapCount 없음: {entry}"
    );
    assert!(
        entry["summary"]
            .as_str()
            .unwrap_or("")
            .contains("text-overlap"),
        "요약에 text-overlap 안내가 없다: {entry}"
    );
}

#[test]
fn help_mentions_text_overlap_and_strict_choice() {
    let out = run(&["--help"]);
    let joined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        joined.contains("text-overlap"),
        "help 에 text-overlap 안내가 없습니다:\n{joined}"
    );
}

fn page_root(width: f64, height: f64, body: RenderNode) -> RenderNode {
    let mut root = RenderNode::new(
        0,
        RenderNodeType::Page(rhwp::renderer::render_tree::PageNode {
            page_index: 0,
            width,
            height,
            section_index: 0,
        }),
        BoundingBox::new(0.0, 0.0, width, height),
    );
    root.children.push(body);
    root
}

fn body_node(bbox: BoundingBox, children: Vec<RenderNode>) -> RenderNode {
    let mut n = RenderNode::new(1, RenderNodeType::Body { clip_rect: None }, bbox);
    n.children = children;
    n
}

fn text_line(x: f64, y: f64, w: f64, h: f64) -> RenderNode {
    RenderNode::new(
        99,
        RenderNodeType::TextLine(TextLineNode::new(h, h * 0.8)),
        BoundingBox::new(x, y, w, h),
    )
}

fn text_run_at(text: &str, x: f64, y: f64, w: f64, h: f64) -> RenderNode {
    RenderNode::new(
        100,
        RenderNodeType::TextRun(TextRunNode {
            text: text.to_string(),
            style: rhwp::renderer::TextStyle::default(),
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 0.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        }),
        BoundingBox::new(x, y, w, h),
    )
}

fn table(x: f64, y: f64, w: f64, h: f64, children: Vec<RenderNode>) -> RenderNode {
    let mut n = RenderNode::new(
        2,
        RenderNodeType::Table(TableNode {
            row_count: 1,
            col_count: 1,
            border_fill_id: 0,
            section_index: None,
            para_index: None,
            control_index: None,
            cell_context: None,
        }),
        BoundingBox::new(x, y, w, h),
    );
    n.children = children;
    n
}

#[test]
fn overlapping_text_runs_inside_table_are_text_overlap_not_generic_overlap() {
    let mut line_a = text_line(10.0, 10.0, 40.0, 12.0);
    line_a
        .children
        .push(text_run_at("갑", 10.0, 10.0, 40.0, 12.0));
    let mut line_b = text_line(20.0, 12.0, 40.0, 12.0);
    line_b
        .children
        .push(text_run_at("을", 20.0, 12.0, 40.0, 12.0));
    let mut cell = RenderNode::new(
        5,
        RenderNodeType::TableCell(TableCellNode {
            col: 0,
            row: 0,
            col_span: 1,
            row_span: 1,
            border_fill_id: 0,
            text_direction: 0,
            clip: false,
            page_fragment: false,
            model_cell_index: None,
        }),
        BoundingBox::new(10.0, 10.0, 80.0, 30.0),
    );
    cell.children.extend([line_a, line_b]);
    let t = table(10.0, 10.0, 80.0, 30.0, vec![cell]);
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(
        pa.overlap.is_empty(),
        "표 하나로는 일반 overlap 이 나면 안 된다: {:?}",
        pa.overlap
    );
    assert_eq!(pa.text_overlap.len(), 1);
    assert_eq!(pa.text_overlap[0].type_a, "TextRun");
    assert_eq!(pa.text_overlap[0].type_b, "TextRun");
    assert!(pa.has_signal());
}

#[test]
fn two_tables_overlapping_are_not_text_overlap() {
    let a = table(10.0, 10.0, 40.0, 40.0, vec![]);
    let b = table(20.0, 20.0, 40.0, 40.0, vec![]);
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![a, b]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.overlap.len(), 1);
    assert!(pa.text_overlap.is_empty());
}

#[test]
fn adjacent_text_runs_on_same_line_are_clean() {
    let mut line = text_line(10.0, 10.0, 80.0, 12.0);
    line.children
        .push(text_run_at("왼", 10.0, 10.0, 30.0, 12.0));
    line.children
        .push(text_run_at("오", 40.0, 10.0, 30.0, 12.0));
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![line]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(
        pa.text_overlap.is_empty(),
        "같은 줄에서 맞닿은 런은 교차가 아니다: {:?}",
        pa.text_overlap
    );
}

#[test]
fn overlapping_text_runs_on_same_line_are_flagged() {
    let mut line = text_line(10.0, 10.0, 80.0, 12.0);
    line.children
        .push(text_run_at("왼", 10.0, 10.0, 40.0, 12.0));
    line.children
        .push(text_run_at("오", 20.0, 10.0, 40.0, 12.0));
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![line]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.text_overlap.len(), 1);
    assert!(
        pa.overlap.is_empty(),
        "한 줄 안 런 교차는 일반 overlap(줄 단위)이 아니다: {:?}",
        pa.overlap
    );
    assert!(pa.has_signal());
}

#[test]
fn intentional_char_overlap_runs_are_not_flagged() {
    let mut stacked = text_run_at("가", 10.0, 10.0, 20.0, 12.0);
    if let RenderNodeType::TextRun(tr) = &mut stacked.node_type {
        tr.char_overlap = Some(rhwp::renderer::composer::CharOverlapInfo {
            border_type: 1,
            inner_char_size: 100,
        });
    }
    let other = text_run_at("나", 12.0, 10.0, 20.0, 12.0);
    let mut line = text_line(10.0, 10.0, 40.0, 12.0);
    line.children.extend([stacked, other]);
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![line]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(
        pa.text_overlap.is_empty(),
        "글자겹침 컨트롤 런은 의도된 겹침이다: {:?}",
        pa.text_overlap
    );
}

#[test]
fn text_overlap_alone_is_confirmed_signal() {
    let mut line = text_line(10.0, 10.0, 80.0, 12.0);
    line.children
        .push(text_run_at("왼", 10.0, 10.0, 40.0, 12.0));
    line.children
        .push(text_run_at("오", 20.0, 10.0, 40.0, 12.0));
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![line]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(!pa.text_overlap.is_empty());
    assert!(pa.overflow.is_empty());
    assert!(pa.overlap.is_empty());
    assert!(pa.has_signal(), "text-overlap 단독도 --strict 확정 신호다");
    let doc = DocAnomalies {
        page_count: 1,
        pages: vec![pa],
    };
    assert_eq!(doc.text_overlap_count(), 1);
    assert!(doc.has_signal());
}

#[test]
fn clean_page_has_empty_text_overlap() {
    let mut line = text_line(10.0, 10.0, 50.0, 12.0);
    line.children
        .push(text_run_at("hello", 10.0, 10.0, 50.0, 12.0));
    let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![line]);
    let root = page_root(100.0, 200.0, body);
    let pa: PageAnomalies = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(pa.text_overlap.is_empty());
    assert!(!pa.has_signal());
}
