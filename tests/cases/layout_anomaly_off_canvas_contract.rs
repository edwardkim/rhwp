//! [#5380] `layout-anomaly` off-canvas 판정·봉투 계약.
//!
//! overflow(본문 여백)와 off-canvas(페이지 상자·y<0)를 가르고, JSON 건수·
//! `--strict` 확정 신호·자기서술을 고정한다. 레이아웃 엔진은 건드리지 않는다.
//! samples/ 에 issue4889 재현본이 없어 판정 자체는 합성 트리로 닫는다.
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
fn json_envelope_declares_off_canvas_count_and_stays_exit_zero() {
    let src = sample();
    let args = ["layout-anomaly", src.as_str(), "--json"];
    let out = run(&args);
    assert_eq!(
        out.status.code(),
        Some(0),
        "기본은 off-canvas 포함 이상 신호가 있어도 0 이다. stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(
        v.get("offCanvasCount").and_then(|x| x.as_u64()).is_some(),
        "offCanvasCount 누락: {v}"
    );
    assert!(v["pages"].is_array(), "{v}");
    for page in v["pages"].as_array().unwrap() {
        assert!(
            page.get("offCanvas").is_some(),
            "pages[].offCanvas 누락: {page}"
        );
    }
}

#[test]
fn capabilities_lists_off_canvas_count() {
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
        fields.contains(&"offCanvasCount"),
        "recordFields 에 offCanvasCount 없음: {entry}"
    );
    assert!(
        entry["summary"]
            .as_str()
            .unwrap_or("")
            .contains("off-canvas"),
        "요약에 off-canvas 안내가 없다: {entry}"
    );
}

#[test]
fn help_mentions_off_canvas_and_strict_choice() {
    let out = run(&["--help"]);
    let joined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        joined.contains("off-canvas"),
        "help 에 off-canvas 안내가 없습니다:\n{joined}"
    );
    assert!(
        joined.contains("페이지 상자") || joined.contains("y<0"),
        "help 에 페이지 상자/y<0 구분이 없습니다:\n{joined}"
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

fn text_run(text: &str) -> RenderNode {
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
            display_text: None,
        }),
        BoundingBox::new(0.0, 0.0, 10.0, 10.0),
    )
}

fn table(x: f64, y: f64, w: f64, h: f64) -> RenderNode {
    RenderNode::new(
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
    )
}

#[test]
fn body_overflow_inside_page_is_not_off_canvas() {
    let t = table(0.0, 10.0, 160.0, 40.0);
    let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 300.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.overflow.len(), 1);
    assert!(
        pa.off_canvas.is_empty(),
        "본문만 넘치고 쪽 안에 있으면 off-canvas 가 아니다: {:?}",
        pa.off_canvas
    );
}

#[test]
fn negative_y_table_is_off_canvas() {
    let t = table(10.0, -80.0, 80.0, 120.0);
    let body = body_node(BoundingBox::new(10.0, 20.0, 180.0, 260.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.off_canvas.len(), 1);
    assert_eq!(pa.off_canvas[0].node_type, "Table");
    assert!(pa.off_canvas[0].bbox.y < 0.0);
    assert!(pa.has_signal());
}

#[test]
fn off_canvas_alone_is_strict_signal() {
    let t = table(10.0, -40.0, 80.0, 50.0);
    let body = body_node(BoundingBox::new(0.0, -50.0, 200.0, 350.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa: PageAnomalies = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(!pa.off_canvas.is_empty());
    assert!(pa.overflow.is_empty());
    assert!(pa.has_signal(), "off-canvas 단독도 --strict 확정 신호다");
    let doc = DocAnomalies {
        page_count: 1,
        pages: vec![pa],
    };
    assert_eq!(doc.off_canvas_count(), 1);
    assert!(doc.has_signal());
}

#[test]
fn clean_on_page_content_has_no_off_canvas() {
    let mut line = text_line(10.0, 10.0, 50.0, 12.0);
    line.children.push(text_run("hello"));
    let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![line]);
    let root = page_root(100.0, 200.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(pa.off_canvas.is_empty());
    assert!(!pa.has_signal());
}

#[test]
fn table_past_page_width_is_off_canvas() {
    let t = table(0.0, 10.0, 250.0, 40.0);
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.off_canvas.len(), 1);
    assert!((pa.off_canvas[0].over_right - 50.0).abs() < 1e-9);
}

#[test]
fn table_past_page_height_is_off_canvas() {
    let t = table(10.0, 250.0, 80.0, 80.0);
    let body = body_node(BoundingBox::new(10.0, 20.0, 180.0, 260.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.off_canvas.len(), 1);
    assert!((pa.off_canvas[0].over_bottom - 30.0).abs() < 1e-9);
}

/// [#5586] 컨테이너 자기 bbox 는 쪽 안이지만 중첩 내용이 쪽 밖으로 흘러나온
/// 형상(선언 높이 신뢰 배치의 1x1 래퍼 표)을 심층 범위로 잡는다. 종전에는
/// suppress 설계 때문에 자기 bbox 만 보고 전량 침묵했다(00365 실측).
#[test]
fn nested_content_past_page_box_is_off_canvas() {
    // 쪽 200 높이. 표 자체는 y=100 h=80 (쪽 안), 그 안의 줄이 y=230 (쪽 밖).
    let mut spilled = text_line(10.0, 230.0, 50.0, 12.0);
    spilled.children.push(text_run("넘친 줄"));
    let mut tbl = table(10.0, 100.0, 80.0, 80.0);
    tbl.children.push(spilled);
    let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 190.0), vec![tbl]);
    let root = page_root(100.0, 200.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.off_canvas.len(), 1, "심층 범위 off-canvas 1건: {pa:?}");
    assert_eq!(pa.off_canvas[0].node_type, "Table");
    // 보고 bbox 는 심층 범위(하단 242)를 담아야 근거가 된다.
    let b = &pa.off_canvas[0].bbox;
    assert!((b.y + b.height - 242.0).abs() < 0.01, "deep bottom: {b:?}");
}

/// 심층 범위가 전부 쪽 안이면 종전대로 무보고여야 한다.
#[test]
fn nested_content_inside_page_box_stays_clean() {
    let mut inner = text_line(10.0, 150.0, 50.0, 12.0);
    inner.children.push(text_run("정상"));
    let mut tbl = table(10.0, 100.0, 80.0, 80.0);
    tbl.children.push(inner);
    let body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 190.0), vec![tbl]);
    let root = page_root(100.0, 200.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(pa.off_canvas.is_empty(), "{pa:?}");
}

#[test]
fn off_canvas_within_tolerance_is_not_flagged() {
    let t = table(0.0, -0.5, 50.0, 40.0);
    let body = body_node(BoundingBox::new(0.0, 0.0, 200.0, 300.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert!(
        pa.off_canvas.is_empty(),
        "0.5px 음수 y 는 기본 허용치(1.0px) 이내: {:?}",
        pa.off_canvas
    );
}

#[test]
fn nested_lines_inside_off_canvas_table_are_not_double_reported() {
    let mut cell_line = text_line(-80.0, -80.0, 30.0, 10.0);
    cell_line.children.push(text_run("x"));
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
        BoundingBox::new(-80.0, -80.0, 30.0, 10.0),
    );
    cell.children.push(cell_line);
    let t = RenderNode::new(
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
        BoundingBox::new(-80.0, -80.0, 80.0, 120.0),
    );
    let mut t = t;
    t.children.push(cell);
    let body = body_node(BoundingBox::new(10.0, 20.0, 180.0, 260.0), vec![t]);
    let root = page_root(200.0, 300.0, body);
    let pa = scan_page(0, &root, 3, &AnomalyOptions::default());
    assert_eq!(pa.off_canvas.len(), 1);
    assert_eq!(pa.off_canvas[0].node_type, "Table");
}
