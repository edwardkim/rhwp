//! [#5459] layout-anomaly 판정 픽스처·성적표 고도화 (M02-f).
//!
//! `tests/fixtures/layout_anomaly_m02f/` 의 합성 트리·verdict 행렬·transcript 를
//! `scan_page` 실측과 대조한다. 레이아웃 엔진은 읽지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::{Path, PathBuf};

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::model::shape::TextWrap;
use rhwp::renderer::render_tree::{
    BoundingBox, EllipseNode, GroupNode, ImageNode, LineNode, PlaceholderNode, RawSvgNode,
    RectangleNode, RenderLayerInfo, RenderNode, RenderNodeType, TableCellNode, TableNode,
    TextLineNode, TextRunNode,
};
use rhwp::renderer::{LineStyle, ShapeStyle};

const FIXTURE_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/layout_anomaly_m02f"
);

fn fixture(rel: &str) -> PathBuf {
    Path::new(FIXTURE_ROOT).join(rel)
}

fn read_json(rel: &str) -> serde_json::Value {
    let raw = fs::read_to_string(fixture(rel))
        .unwrap_or_else(|e| panic!("{} 읽기 실패: {e}", fixture(rel).display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{} JSON 파싱 실패: {e}", fixture(rel).display()))
}

fn f64_field(v: &serde_json::Value, key: &str) -> f64 {
    v.get(key)
        .and_then(|x| x.as_f64())
        .unwrap_or_else(|| panic!("{key} 숫자 필요: {v}"))
}

fn intern_type(label: &str) -> &'static str {
    match label {
        "Table" => "Table",
        "Image" => "Image",
        "TextBox" => "TextBox",
        "Equation" => "Equation",
        "Group" => "Group",
        "Form" => "Form",
        "Placeholder" => "Placeholder",
        "RawSvg" => "RawSvg",
        "Line" => "Line",
        "Rect" => "Rect",
        "Ellipse" => "Ellipse",
        "Path" => "Path",
        "TextLine" => "TextLine",
        other => panic!("--types 라벨이 아닙니다: {other}"),
    }
}

fn parse_wrap(raw: Option<&str>) -> Option<TextWrap> {
    match raw {
        None => None,
        Some("Square") => Some(TextWrap::Square),
        Some("Tight") => Some(TextWrap::Tight),
        Some("Through") => Some(TextWrap::Through),
        Some("TopAndBottom") => Some(TextWrap::TopAndBottom),
        Some("BehindText") => Some(TextWrap::BehindText),
        Some("InFrontOfText") => Some(TextWrap::InFrontOfText),
        Some(other) => panic!("알 수 없는 wrap: {other}"),
    }
}

fn bbox_of(v: &serde_json::Value) -> BoundingBox {
    BoundingBox::new(
        f64_field(v, "x"),
        f64_field(v, "y"),
        f64_field(v, "w"),
        f64_field(v, "h"),
    )
}

fn text_run_node(n: &serde_json::Value, bbox: BoundingBox) -> RenderNode {
    RenderNode::new(
        100,
        RenderNodeType::TextRun(TextRunNode {
            text: n
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string(),
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
            char_overlap: if n.get("charOverlap").and_then(|x| x.as_bool()) == Some(true) {
                Some(rhwp::renderer::composer::CharOverlapInfo {
                    border_type: 1,
                    inner_char_size: 100,
                })
            } else {
                None
            },
            border_fill_id: 0,
            baseline: 0.0,
            field_marker: Default::default(),
            display_text: None,
        }),
        bbox,
    )
}

fn table_node(bbox: BoundingBox) -> RenderNode {
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
        bbox,
    )
}

fn cell_node(bbox: BoundingBox) -> RenderNode {
    RenderNode::new(
        5,
        RenderNodeType::TableCell(TableCellNode {
            col: 0,
            row: 0,
            col_span: 1,
            row_span: 1,
            border_fill_id: 0,
            text_direction: 0,
            clip: false,
            model_cell_index: None,
        }),
        bbox,
    )
}

fn build_node(n: &serde_json::Value) -> RenderNode {
    let t = n["t"].as_str().expect("node.t");
    let bbox = bbox_of(n);
    let mut node = match t {
        "Table" => table_node(bbox),
        "Cell" => cell_node(bbox),
        "TextLine" => RenderNode::new(
            99,
            RenderNodeType::TextLine(TextLineNode::new(bbox.height, bbox.height * 0.8)),
            bbox,
        ),
        "TextRun" => text_run_node(n, bbox),
        "TextBox" => RenderNode::new(3, RenderNodeType::TextBox, bbox),
        "Image" => RenderNode::new(4, RenderNodeType::Image(ImageNode::new(0, None)), bbox),
        "Group" => RenderNode::new(
            6,
            RenderNodeType::Group(GroupNode {
                section_index: None,
                para_index: None,
                control_index: None,
            }),
            bbox,
        ),
        "Rect" => RenderNode::new(
            7,
            RenderNodeType::Rectangle(RectangleNode::new(0.0, ShapeStyle::default(), None)),
            bbox,
        ),
        "Line" => RenderNode::new(
            8,
            RenderNodeType::Line(LineNode::new(
                bbox.x,
                bbox.y,
                bbox.x + bbox.width,
                bbox.y,
                LineStyle::default(),
            )),
            bbox,
        ),
        "Ellipse" => RenderNode::new(
            9,
            RenderNodeType::Ellipse(EllipseNode::new(ShapeStyle::default(), None)),
            bbox,
        ),
        "Placeholder" => RenderNode::new(
            10,
            RenderNodeType::Placeholder(PlaceholderNode::new(0, 0, "p".into())),
            bbox,
        ),
        "RawSvg" => RenderNode::new(
            11,
            RenderNodeType::RawSvg(RawSvgNode::new("<g/>".into())),
            bbox,
        ),
        "Column" => {
            let col = n.get("column").and_then(|c| c.as_u64()).unwrap_or(0) as u16;
            RenderNode::new(12, RenderNodeType::Column(col), bbox)
        }
        other => panic!("합성 트리에 없는 노드 타입: {other}"),
    };

    if let Some(wrap) = parse_wrap(n.get("wrap").and_then(|w| w.as_str())) {
        let mut layer = RenderLayerInfo::new(Some(wrap), 0, 0);
        if n.get("master").and_then(|m| m.as_bool()) == Some(true) {
            layer = layer.for_master_page();
        }
        node = node.with_layer(layer);
    } else if n.get("master").and_then(|m| m.as_bool()) == Some(true) {
        node = node.with_layer(RenderLayerInfo::new(None, 0, 0).for_master_page());
    }

    if n.get("visible").and_then(|v| v.as_bool()) == Some(false) {
        node.visible = false;
    }
    if n.get("editorOnly").and_then(|v| v.as_bool()) == Some(true) {
        node.editor_only = true;
    }
    if let Some(children) = n.get("children").and_then(|c| c.as_array()) {
        node.children = children.iter().map(build_node).collect();
    }
    node
}

fn page_root(page: &serde_json::Value, body: RenderNode) -> RenderNode {
    let w = f64_field(page, "w");
    let h = f64_field(page, "h");
    let x = page.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let y = page.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let mut root = RenderNode::new(
        0,
        RenderNodeType::Page(rhwp::renderer::render_tree::PageNode {
            page_index: 0,
            width: w,
            height: h,
            section_index: 0,
        }),
        BoundingBox::new(x, y, w, h),
    );
    root.children.push(body);
    root
}

fn options_of(opts: &serde_json::Value) -> AnomalyOptions {
    let types = opts.get("types").and_then(|t| t.as_array()).map(|arr| {
        arr.iter()
            .map(|v| intern_type(v.as_str().expect("types 항목")))
            .collect()
    });
    AnomalyOptions {
        overflow_tolerance_px: f64_field(opts, "overflowTol"),
        overlap_tolerance_px: f64_field(opts, "overlapTol"),
        type_filter: types,
    }
}

fn load_cases() -> Vec<serde_json::Value> {
    let dir = fixture("trees");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let raw = fs::read_to_string(&path).expect("tree json");
        let arr: Vec<serde_json::Value> =
            serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        out.extend(arr);
    }
    out
}

fn scan_case(case: &serde_json::Value) -> rhwp::diagnostics::layout_anomaly::PageAnomalies {
    let body_box = bbox_of(&case["body"]);
    let children: Vec<RenderNode> = case["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .map(build_node)
        .collect();
    let mut body = RenderNode::new(1, RenderNodeType::Body { clip_rect: None }, body_box);
    body.children = children;
    let root = page_root(&case["page"], body);
    let page = case["pageIndex"].as_u64().expect("pageIndex") as u32;
    let page_count = case["pageCount"].as_u64().expect("pageCount") as u32;
    scan_page(page, &root, page_count, &options_of(&case["opts"]))
}

fn report_lines(page: u32, pa: &rhwp::diagnostics::layout_anomaly::PageAnomalies) -> Vec<String> {
    let mut lines = Vec::new();
    for o in &pa.overflow {
        lines.push(format!(
            "  [OVERFLOW] page {page:>3}  {:>7.2}px  {} ({})",
            o.max_over(),
            o.path,
            o.node_type
        ));
    }
    for o in &pa.off_canvas {
        lines.push(format!(
            "  [OFF-CANVAS] page {page:>3}  {:>7.2}px  {} ({})",
            o.max_over(),
            o.path,
            o.node_type
        ));
    }
    for o in &pa.overlap {
        lines.push(format!(
            "  [OVERLAP]  page {page:>3}  {:.2}x{:.2}px  {} ({}) x {} ({})",
            o.overlap_w, o.overlap_h, o.path_a, o.type_a, o.path_b, o.type_b
        ));
    }
    for o in &pa.text_overlap {
        lines.push(format!(
            "  [TEXT-OVERLAP] page {page:>3}  {:.2}x{:.2}px  {} ({}) x {} ({})",
            o.overlap_w, o.overlap_h, o.path_a, o.type_a, o.path_b, o.type_b
        ));
    }
    if pa.empty_page.is_some() {
        lines.push(format!(
            "  [EMPTY_PAGE?] page {page:>3}  콘텐츠 없음 (가능성 신호 — 의도된 빈 쪽일 수 있음)"
        ));
    }
    lines
}

fn assert_case(case: &serde_json::Value) {
    let id = case["id"].as_str().expect("id");
    let expect = &case["expect"];
    let pa = scan_case(case);
    assert_eq!(
        pa.overflow.len(),
        expect["overflow"].as_u64().unwrap() as usize,
        "{id} overflow"
    );
    assert_eq!(
        pa.off_canvas.len(),
        expect["offCanvas"].as_u64().unwrap() as usize,
        "{id} offCanvas"
    );
    assert_eq!(
        pa.overlap.len(),
        expect["overlap"].as_u64().unwrap() as usize,
        "{id} overlap"
    );
    assert_eq!(
        pa.text_overlap.len(),
        expect["textOverlap"].as_u64().unwrap() as usize,
        "{id} textOverlap"
    );
    assert_eq!(
        pa.empty_page.is_some(),
        expect["empty"].as_bool().unwrap(),
        "{id} empty"
    );
    assert_eq!(
        pa.has_signal(),
        expect["signal"].as_bool().unwrap(),
        "{id} signal"
    );

    let got_types: Vec<&str> = pa.overflow.iter().map(|o| o.node_type).collect();
    let want_types: Vec<&str> = expect["overflowTypes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(got_types, want_types, "{id} overflowTypes");

    let got_off: Vec<&str> = pa.off_canvas.iter().map(|o| o.node_type).collect();
    let want_off: Vec<&str> = expect["offCanvasTypes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(got_off, want_off, "{id} offCanvasTypes");

    let got_paths: Vec<&str> = pa.overflow.iter().map(|o| o.path.as_str()).collect();
    let want_paths: Vec<&str> = expect["overflowPaths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(got_paths, want_paths, "{id} overflowPaths");

    if let Some(first) = pa.overflow.first() {
        assert!(
            (first.over_left - expect["overLeft"].as_f64().unwrap()).abs() < 1e-9,
            "{id} overLeft"
        );
        assert!(
            (first.over_top - expect["overTop"].as_f64().unwrap()).abs() < 1e-9,
            "{id} overTop"
        );
        assert!(
            (first.over_right - expect["overRight"].as_f64().unwrap()).abs() < 1e-9,
            "{id} overRight"
        );
        assert!(
            (first.over_bottom - expect["overBottom"].as_f64().unwrap()).abs() < 1e-9,
            "{id} overBottom"
        );
        assert!(
            (first.max_over() - expect["maxOver"].as_f64().unwrap()).abs() < 1e-9,
            "{id} maxOver"
        );
    }

    let page = case["pageIndex"].as_u64().unwrap() as u32;
    let got_report = report_lines(page, &pa);
    let want_report: Vec<&str> = expect["reportLines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(got_report, want_report, "{id} reportLines");

    let want_status = expect["status"].as_str().unwrap();
    let got_status = if pa.has_signal() { "ANOMALY" } else { "CLEAN" };
    assert_eq!(got_status, want_status, "{id} status");
}

#[test]
fn catalog_declares_case_count_and_families() {
    let catalog = read_json("catalog.json");
    assert_eq!(catalog["issue"], 5459);
    let cases = load_cases();
    assert_eq!(
        cases.len(),
        catalog["caseCount"].as_u64().unwrap() as usize,
        "catalog.caseCount 와 trees/ 건수가 어긋난다"
    );
    assert!(
        cases.len() >= 200,
        "M02-f 카탈로그가 너무 얇다: {}",
        cases.len()
    );
    for family in [
        "overflow",
        "overlap",
        "text-overlap",
        "off-canvas",
        "empty-page",
        "types",
        "combined",
        "suppress",
        "visibility",
        "tolerance",
    ] {
        assert!(
            cases.iter().any(|c| c["family"] == family),
            "가족 {family} 가 비어 있다"
        );
    }
}

#[test]
fn every_synthetic_tree_matches_scan_page() {
    let cases = load_cases();
    assert!(!cases.is_empty());
    let mut ids = Vec::new();
    for case in &cases {
        let id = case["id"].as_str().unwrap().to_string();
        assert_case(case);
        ids.push(id);
    }
    ids.sort();
    let mut uniq = ids.clone();
    uniq.dedup();
    assert_eq!(ids.len(), uniq.len(), "픽스처 id 가 중복이다");
}

#[test]
fn verdict_matrix_rows_match_tree_expects() {
    let cases = load_cases();
    let by_id: std::collections::BTreeMap<&str, &serde_json::Value> = cases
        .iter()
        .map(|c| (c["id"].as_str().unwrap(), c))
        .collect();
    let tsv = fs::read_to_string(fixture("matrices/all_verdicts.tsv")).expect("all_verdicts");
    let mut lines = tsv.lines();
    let header = lines.next().expect("header");
    assert!(header.starts_with("id\tfamily\t"), "{header}");
    let mut seen = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 17, "열 부족: {line}");
        let case = by_id
            .get(cols[0])
            .unwrap_or_else(|| panic!("행렬 id 가 트리에 없다: {}", cols[0]));
        let e = &case["expect"];
        assert_eq!(cols[1], case["family"].as_str().unwrap(), "{}", cols[0]);
        assert_eq!(
            cols[7].parse::<u64>().unwrap(),
            e["overflow"].as_u64().unwrap(),
            "{} overflow",
            cols[0]
        );
        assert_eq!(
            cols[8].parse::<u64>().unwrap(),
            e["offCanvas"].as_u64().unwrap(),
            "{} offCanvas",
            cols[0]
        );
        assert_eq!(
            cols[9].parse::<u64>().unwrap(),
            e["overlap"].as_u64().unwrap(),
            "{} overlap",
            cols[0]
        );
        assert_eq!(
            cols[10].parse::<u64>().unwrap(),
            e["textOverlap"].as_u64().unwrap(),
            "{} textOverlap",
            cols[0]
        );
        assert_eq!(
            cols[11].parse::<u64>().unwrap() == 1,
            e["empty"].as_bool().unwrap(),
            "{} empty",
            cols[0]
        );
        assert_eq!(
            cols[12].parse::<u64>().unwrap() == 1,
            e["signal"].as_bool().unwrap(),
            "{} signal",
            cols[0]
        );
        assert_eq!(
            cols[13],
            e["status"].as_str().unwrap(),
            "{} status",
            cols[0]
        );
        seen += 1;
    }
    assert_eq!(seen, cases.len(), "행렬 행 수와 트리 건수가 다르다");
}

#[test]
fn family_matrices_are_partitions_of_all_verdicts() {
    let all = fs::read_to_string(fixture("matrices/all_verdicts.tsv")).unwrap();
    let all_ids: Vec<&str> = all
        .lines()
        .skip(1)
        .filter(|l| !l.is_empty())
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    let mut parts = Vec::new();
    for name in [
        "overflow_simple",
        "overflow_nested",
        "overlap",
        "text_overlap",
        "off_canvas",
        "empty_page",
        "visibility",
        "combined",
        "tolerance",
    ] {
        let text = fs::read_to_string(fixture(&format!("matrices/{name}.tsv"))).unwrap();
        for line in text.lines().skip(1).filter(|l| !l.is_empty()) {
            parts.push(line.split('\t').next().unwrap().to_string());
        }
    }
    parts.sort();
    let mut all_sorted: Vec<&str> = all_ids.clone();
    all_sorted.sort();
    assert_eq!(parts, all_sorted);
}

#[test]
fn transcripts_match_catalog_and_cli_shape() {
    let cases = load_cases();
    let by_id: std::collections::BTreeMap<&str, &serde_json::Value> = cases
        .iter()
        .map(|c| (c["id"].as_str().unwrap(), c))
        .collect();
    let dir = fixture("transcripts");
    let mut humans = 0usize;
    let mut envelopes = 0usize;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        if let Some(id) = name.strip_suffix(".human.txt") {
            if id.starts_with("batch_") {
                continue;
            }
            humans += 1;
            let case = by_id.get(id).unwrap_or_else(|| panic!("transcript {id}"));
            let text = fs::read_to_string(&path).unwrap();
            assert!(text.contains(&format!("# transcript {id}")), "{name}");
            assert!(text.contains("쪽 수:"), "{name}");
            assert!(
                text.contains(&format!(
                    "status: {}",
                    case["expect"]["status"].as_str().unwrap()
                )),
                "{name}"
            );
            for line in case["expect"]["reportLines"].as_array().unwrap() {
                assert!(
                    text.contains(line.as_str().unwrap()),
                    "{name} 성적표에 {} 없음",
                    line
                );
            }
        } else if let Some(id) = name.strip_suffix(".envelope.json") {
            envelopes += 1;
            let case = by_id.get(id).unwrap_or_else(|| panic!("envelope {id}"));
            let env = read_json(&format!("transcripts/{name}"));
            assert_eq!(env["schemaVersion"], "1.0", "{name}");
            assert_eq!(env["mode"], "single", "{name}");
            assert_eq!(env["fixtureId"], id, "{name}");
            assert_eq!(
                env["hasSignal"], case["expect"]["signal"],
                "{name} hasSignal"
            );
            assert_eq!(
                env["overflowCount"].as_u64().unwrap() as usize,
                case["expect"]["overflow"].as_u64().unwrap() as usize,
                "{name} overflowCount"
            );
            assert_eq!(
                env["offCanvasCount"].as_u64().unwrap() as usize,
                case["expect"]["offCanvas"].as_u64().unwrap() as usize,
                "{name} offCanvasCount"
            );
            assert_eq!(
                env["textOverlapCount"].as_u64().unwrap() as usize,
                case["expect"]["textOverlap"].as_u64().unwrap() as usize,
                "{name} textOverlapCount"
            );
            assert!(
                env.get("pages").and_then(|p| p.as_array()).is_some(),
                "{name}"
            );
        }
    }
    assert!(humans >= 15, "사람 성적표가 너무 적다: {humans}");
    assert_eq!(humans, envelopes, "human/envelope 쌍이 안 맞는다");
}

#[test]
fn batch_transcript_is_order_stable_and_summarized() {
    let ndjson = fs::read_to_string(fixture("transcripts/batch_catalog.ndjson")).unwrap();
    let mut sources = Vec::new();
    let mut clean = 0u64;
    let mut anomaly = 0u64;
    for line in ndjson.lines().filter(|l| !l.is_empty()) {
        let rec: serde_json::Value = serde_json::from_str(line).unwrap();
        assert_eq!(rec["mode"], "batch");
        assert_eq!(rec["schemaVersion"], "1.0");
        assert!(rec.get("overflowCount").is_some());
        assert!(rec.get("offCanvasCount").is_some());
        assert!(rec.get("textOverlapCount").is_some());
        let src = rec["source"].as_str().unwrap().to_string();
        if rec["hasSignal"].as_bool().unwrap() {
            anomaly += 1;
            assert_eq!(rec["status"], "ANOMALY");
        } else {
            clean += 1;
            assert_eq!(rec["status"], "CLEAN");
        }
        sources.push(src);
    }
    let mut sorted = sources.clone();
    sorted.sort();
    assert_eq!(sources, sorted, "배치 NDJSON 은 source 정렬이어야 한다");
    assert!(!sources.is_empty());

    let human = fs::read_to_string(fixture("transcripts/batch_catalog.human.txt")).unwrap();
    assert!(human.contains("=== layout-anomaly 요약 ==="));
    assert!(human.contains(&format!("CLEAN           : {clean}")));
    assert!(human.contains(&format!("ANOMALY         : {anomaly}")));
    assert!(human.contains("LOAD_FAIL       : 0"));
    for src in &sources {
        assert!(human.contains(src), "배치 사람 성적표에 {src} 없음");
    }
}

#[test]
fn exit_contract_transcript_covers_strict_and_batch() {
    let text = fs::read_to_string(fixture("transcripts/exit_contract.tsv")).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    for line in text.lines().skip(2).filter(|l| !l.is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 3, "{line}");
        let exit: i32 = cols[1].parse().unwrap();
        assert!(matches!(exit, 0 | 1 | 2 | 3), "exit 계약 밖: {line}");
        seen.insert(cols[0].to_string());
    }
    for want in [
        "single-signal-default",
        "single-signal-strict",
        "single-empty-strict",
        "batch-load-fail",
        "batch-signal-strict",
        "off-canvas-strict",
        "text-overlap-strict",
    ] {
        assert!(seen.contains(want), "{want} 누락");
    }
}

#[test]
fn representative_families_have_clean_and_signal_rows() {
    let cases = load_cases();
    for family in [
        "overflow",
        "overlap",
        "text-overlap",
        "off-canvas",
        "empty-page",
    ] {
        let rows: Vec<_> = cases.iter().filter(|c| c["family"] == family).collect();
        let key = match family {
            "overflow" => "overflow",
            "overlap" => "overlap",
            "text-overlap" => "textOverlap",
            "off-canvas" => "offCanvas",
            "empty-page" => "empty",
            _ => unreachable!(),
        };
        if family == "empty-page" {
            assert!(
                rows.iter().any(|c| c["expect"]["empty"] == true),
                "empty-page 에 가능성 신호가 없다"
            );
            assert!(
                rows.iter()
                    .all(|c| c["expect"]["empty"] == false || c["expect"]["signal"] == false),
                "empty_page 단독은 has_signal 이 되면 안 된다"
            );
        } else {
            assert!(
                rows.iter().any(|c| c["expect"][key] == 0),
                "{family} 에 {key}=0 행이 없다"
            );
            assert!(
                rows.iter().any(|c| c["expect"][key] != 0),
                "{family} 에 {key}>0 행이 없다"
            );
        }
    }
}

#[test]
fn types_filter_cases_do_not_change_off_canvas_rule() {
    let cases = load_cases();
    let filtered: Vec<_> = cases.iter().filter(|c| c["family"] == "types").collect();
    assert!(!filtered.is_empty());
    for case in filtered {
        let pa = scan_case(case);
        let unfiltered = {
            let mut clone = case.clone();
            clone["opts"]["types"] = serde_json::Value::Null;
            scan_case(&clone)
        };
        assert_eq!(
            pa.off_canvas.len(),
            unfiltered.off_canvas.len(),
            "{} --types 가 off-canvas 를 걸러서는 안 된다",
            case["id"]
        );
    }
}
