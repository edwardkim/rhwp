use pdf_oxide::{
    elements::{PathContent, PathOperation},
    geometry::Rect,
    layout::TextSpan,
};
use rhwp_pdf_oxide_poc::{
    inspect_page,
    recognition::{recognize_page, recognize_region, Policy, Region},
    table_detection::detect_ruled_tables,
};
use serde_json::{json, Value};

const PAGE: (f32, f32, f32, f32) = (0.0, 0.0, 200.0, 200.0);
fn span(text: &str, x: f32, y: f32, w: f32) -> TextSpan {
    TextSpan {
        text: text.into(),
        bbox: Rect::new(x, y, w, 10.0),
        font_size: 10.0,
        ..Default::default()
    }
}
fn region(indices: Vec<usize>) -> Region {
    Region {
        context: vec!["page".into(), "region:0".into()],
        bbox: Rect::new(0.0, 0.0, 200.0, 200.0),
        span_indices: indices,
        obstacles: Vec::new(),
    }
}
fn run(spans: &[TextSpan]) -> Value {
    recognize_region(
        spans,
        &region((0..spans.len()).collect()),
        PAGE,
        Policy::default(),
    )
    .unwrap()
}
fn paragraph_count(v: &Value) -> usize {
    v["paragraphs"].as_array().unwrap().len()
}
fn line(x1: f32, y1: f32, x2: f32, y2: f32) -> PathContent {
    PathContent::from_operations(vec![
        PathOperation::MoveTo(x1, y1),
        PathOperation::LineTo(x2, y2),
    ])
}
fn grid(dx: f32) -> Vec<PathContent> {
    vec![
        line(10.0 + dx, 10.0, 70.0 + dx, 10.0),
        line(10.0 + dx, 40.0, 70.0 + dx, 40.0),
        line(10.0 + dx, 70.0, 70.0 + dx, 70.0),
        line(10.0 + dx, 10.0, 10.0 + dx, 70.0),
        line(40.0 + dx, 10.0, 40.0 + dx, 70.0),
        line(70.0 + dx, 10.0, 70.0 + dx, 70.0),
    ]
}
fn detection(paths: &[PathContent], spans: &[TextSpan]) -> Value {
    detect_ruled_tables(spans, paths, &[], PAGE).unwrap()
}
fn all_refs(region: &Value) -> Vec<u64> {
    region["paragraphs"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| {
            p["spanIndices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i.as_u64().unwrap())
        })
        .collect()
}

#[test]
fn line_continuation_is_not_sentence_termination() {
    let d = run(&[
        span("first.", 10.0, 150.0, 70.0),
        span("second", 10.0, 134.0, 60.0),
    ]);
    assert_eq!(paragraph_count(&d), 1);
    assert_eq!(d["paragraphs"][0]["lines"].as_array().unwrap().len(), 2);
    assert_eq!(d["paragraphs"][0]["status"], "Confirmed");
}
#[test]
fn large_gap_emits_boundary_with_evidence() {
    let d = run(&[
        span("a", 10.0, 150.0, 70.0),
        span("b", 10.0, 134.0, 70.0),
        span("c", 10.0, 90.0, 70.0),
    ]);
    assert_eq!(paragraph_count(&d), 2);
    assert_eq!(all_refs(&d), [0, 1, 2]);
    let t = d["transitions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["event"] == "AcceptBoundary")
        .unwrap();
    assert_eq!(t["reason"], "large_baseline_gap");
    assert_eq!(t["evidence"]["baselineGap"], 44.0);
}
#[test]
fn centered_short_last_line_continues() {
    let d = run(&[
        span("a", 10.0, 150.0, 100.0),
        span("b", 10.0, 134.0, 100.0),
        span("c", 40.0, 118.0, 40.0),
    ]);
    assert_eq!(paragraph_count(&d), 1);
    assert_eq!(d["paragraphs"][0]["status"], "Confirmed");
}
#[test]
fn first_line_indent_is_not_an_automatic_break() {
    let d = run(&[
        span("a", 30.0, 150.0, 70.0),
        span("b", 10.0, 134.0, 70.0),
        span("c", 10.0, 118.0, 70.0),
    ]);
    assert_eq!(paragraph_count(&d), 1);
    assert_eq!(d["paragraphs"][0]["status"], "Confirmed");
}
#[test]
fn mixed_style_on_one_line_does_not_make_another_paragraph() {
    let a = span("plain", 10.0, 150.0, 20.0);
    let mut b = span("bold", 35.0, 150.0, 20.0);
    b.font_weight = pdf_oxide::layout::FontWeight::Bold;
    let d = run(&[a, b, span("next", 10.0, 134.0, 40.0)]);
    assert_eq!(paragraph_count(&d), 1);
    assert_eq!(d["paragraphs"][0]["lines"].as_array().unwrap().len(), 2);
}
#[test]
fn reordered_extraction_has_same_geometric_preview() {
    let a = span("left", 10.0, 150.0, 20.0);
    let b = span("right", 35.0, 150.0, 20.0);
    let c = span("next", 10.0, 134.0, 40.0);
    let before = run(&[a.clone(), b.clone(), c.clone()]);
    let after = run(&[c, b, a]);
    let preview = |v: &Value| {
        v["paragraphs"][0]["lines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l["previewText"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(preview(&before), preview(&after));
    assert_eq!(preview(&after), ["leftright", "next"]);
}
#[test]
fn horizontal_discontinuity_defers_unknown_columns() {
    let d = run(&[
        span("left", 10.0, 150.0, 20.0),
        span("right", 100.0, 150.0, 20.0),
    ]);
    assert_eq!(d["paragraphs"][0]["status"], "Deferred");
    assert_eq!(all_refs(&d), [0, 1]);
}
#[test]
fn conflicting_alignment_retains_all_content_for_review() {
    let d = run(&[
        span("a", 10.0, 150.0, 50.0),
        span("b", 50.0, 134.0, 20.0),
        span("c", 50.0, 118.0, 20.0),
    ]);
    assert_eq!(d["paragraphs"][0]["status"], "Deferred");
    assert_eq!(all_refs(&d), [0, 1, 2]);
}
#[test]
fn font_size_conflict_is_not_forced_into_a_heading() {
    let a = span("a", 10.0, 150.0, 50.0);
    let mut b = span("b", 10.0, 134.0, 50.0);
    b.font_size = 14.0;
    let d = run(&[a, b]);
    assert_eq!(d["paragraphs"][0]["status"], "Deferred");
}
#[test]
fn image_between_lines_is_a_recorded_boundary() {
    let spans = [
        span("before", 10.0, 150.0, 60.0),
        span("after", 10.0, 90.0, 60.0),
    ];
    let mut r = region(vec![0, 1]);
    r.obstacles.push(Rect::new(10.0, 60.0, 60.0, 40.0));
    let d = recognize_region(&spans, &r, PAGE, Policy::default()).unwrap();
    assert_eq!(paragraph_count(&d), 2);
    assert!(d["transitions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["reason"] == "intervening_image"));
}
#[test]
fn separate_region_contexts_do_not_share_state() {
    let spans = [
        span("cell a", 10.0, 150.0, 20.0),
        span("cell b", 10.0, 134.0, 20.0),
    ];
    let a = region(vec![0]);
    let mut b = region(vec![1]);
    b.context = vec!["page".into(), "region:1".into()];
    let a = recognize_region(&spans, &a, PAGE, Policy::default()).unwrap();
    let b = recognize_region(&spans, &b, PAGE, Policy::default()).unwrap();
    assert_eq!(all_refs(&a), [0]);
    assert_eq!(all_refs(&b), [1]);
    assert_ne!(a["context"], b["context"]);
}
#[test]
fn depth_invalid_policy_and_duplicate_refs_fail_without_poisoning() {
    let spans = [span("a", 10.0, 150.0, 20.0)];
    let r = region(vec![0]);
    let policy = Policy {
        max_context_depth: 2,
        ..Default::default()
    };
    assert!(recognize_region(&spans, &r, PAGE, policy).is_err());
    let policy = Policy {
        paragraph_gap_em: f32::NAN,
        ..Default::default()
    };
    assert!(recognize_region(&spans, &r, PAGE, policy).is_err());
    assert!(recognize_region(&spans, &region(vec![0, 0]), PAGE, Policy::default()).is_err());
    assert_eq!(paragraph_count(&run(&spans)), 1);
}
#[test]
fn table_fsm_defers_bad_topology() {
    let mut d = detection(&grid(0.0), &[]);
    d["tables"][0]["cells"][1]["column"] = json!(0);
    let r = recognize_page(&[], &[], &d, PAGE, Policy::default()).unwrap();
    assert_eq!(r["tables"][0]["status"], "Deferred");
    assert_eq!(r["tables"][0]["transitions"][2]["event"], "Hold");
}
#[test]
fn geometry_guard_rejects_bbox_that_disagrees_with_grid() {
    let mut d = detection(&grid(0.0), &[]);
    d["tables"][0]["cells"][0]["bbox"]["width"] = json!(20);
    let r = recognize_page(&[], &[], &d, PAGE, Policy::default()).unwrap();
    assert_eq!(r["tables"][0]["status"], "Deferred");
    assert_eq!(
        r["tables"][0]["reason"],
        "cell geometry does not match grid slot"
    );
}
#[test]
fn ambiguous_second_table_does_not_defer_first_table() {
    let paths = [grid(0.0), grid(90.0)].concat();
    let spans = [span("cross", 125.0, 15.0, 10.0)];
    let d = detection(&paths, &spans);
    let r = recognize_page(&spans, &[], &d, PAGE, Policy::default()).unwrap();
    assert_eq!(r["tables"][0]["status"], "GeometryConfirmed");
    assert_eq!(r["tables"][1]["status"], "Deferred");
    assert!(r["bodyRegions"].as_array().unwrap().is_empty());
    assert_eq!(r["unresolvedContent"][0]["index"], 0);
}
#[test]
fn ownership_conflict_defers_both_candidates() {
    let spans = [span("a", 15.0, 15.0, 10.0)];
    let mut d = detection(&grid(0.0), &spans);
    let duplicate = d["tables"][0].clone();
    d["tables"].as_array_mut().unwrap().push(duplicate);
    let r = recognize_page(&spans, &[], &d, PAGE, Policy::default()).unwrap();
    assert!(r["tables"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["status"] == "Deferred"));
    assert!(r["bodyRegions"].as_array().unwrap().is_empty());
}
#[test]
fn source_pdf_cell_hierarchy_preserves_all_refs_and_trace() {
    let repo = std::env::var("RHWP_POC_REPO").unwrap();
    let bytes = std::fs::read(
        std::path::Path::new(&repo).join("pdf/issue5875/nested_table_text_caption-2020.pdf"),
    )
    .unwrap();
    let d = inspect_page(&bytes, 0).unwrap();
    let r = &d["recognition"];
    let table = &r["tables"][0];
    assert_eq!(table["status"], "GeometryConfirmed");
    assert_eq!(table["semanticStatus"], "unverified");
    assert_eq!(table["cells"].as_array().unwrap().len(), 15);
    let events: Vec<_> = table["transitions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["event"].as_str().unwrap())
        .collect();
    assert_eq!(
        events,
        [
            "CandidateFound",
            "ValidateGeometry",
            "GeometryAccepted",
            "BeginCells",
            "EndCells"
        ]
    );
    let mut refs = Vec::new();
    for c in table["cells"].as_array().unwrap() {
        refs.extend(all_refs(&c["recognition"]));
        assert_eq!(c["recognition"]["context"].as_array().unwrap().len(), 3);
    }
    for body in r["bodyRegions"].as_array().unwrap() {
        refs.extend(all_refs(body));
    }
    refs.sort();
    assert_eq!(
        refs,
        (0..d["spans"].as_array().unwrap().len() as u64).collect::<Vec<_>>()
    );
    let cell = table["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["row"] == 2 && c["column"] == 2)
        .unwrap();
    assert_eq!(cell["imageIndices"], json!([0, 1]));
    let transitions = cell["recognition"]["transitions"].as_array().unwrap();
    assert_eq!(
        transitions
            .iter()
            .filter(|t| t["event"] == "AcceptBoundary" && t["reason"] == "intervening_image")
            .count(),
        2
    );
}
