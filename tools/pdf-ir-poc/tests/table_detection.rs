use pdf_oxide::{
    elements::{PathContent, PathOperation},
    geometry::Rect,
    layout::TextSpan,
};
use rhwp_pdf_oxide_poc::{inspect_page, table_detection::detect_ruled_tables};
use serde_json::Value;

fn line(x1: f32, y1: f32, x2: f32, y2: f32) -> PathContent {
    PathContent::from_operations(vec![
        PathOperation::MoveTo(x1, y1),
        PathOperation::LineTo(x2, y2),
    ])
}
fn grid() -> Vec<PathContent> {
    vec![
        line(10.0, 10.0, 70.0, 10.0),
        line(10.0, 40.0, 70.0, 40.0),
        line(10.0, 70.0, 70.0, 70.0),
        line(10.0, 10.0, 10.0, 70.0),
        line(40.0, 10.0, 40.0, 70.0),
        line(70.0, 10.0, 70.0, 70.0),
    ]
}
fn detect(paths: &[PathContent], spans: &[TextSpan]) -> Value {
    detect_ruled_tables(spans, paths, &[], (0.0, 0.0, 200.0, 200.0)).unwrap()
}

#[test]
fn duplicate_coverage_retains_every_source_contributor() {
    let source = grid();
    let repeated = [source.clone(), source].concat();
    let d = detect(&repeated, &[]);
    assert_eq!(d["edgeCount"], 12);
    assert_eq!(d["canonicalEdgeCount"], 6);
    assert_eq!(
        d["tables"][0]["sourcePathIndices"],
        serde_json::json!((0..12).collect::<Vec<_>>())
    );
    assert_eq!(
        d["tables"][0]["sourceBoundarySegments"]
            .as_array()
            .unwrap()
            .len(),
        12
    );
}

#[test]
fn border_join_overhang_is_owned_but_unrelated_extension_is_not() {
    let mut source = grid();
    source[3].stroke_width = 4.;
    source[5].stroke_width = 4.;
    source[0] = line(9., 10., 71., 10.);
    source.push(line(5., 70., 75., 70.));
    let d = detect(&source, &[]);
    assert_eq!(count(&d), 1);
    assert_eq!(
        d["tables"][0]["sourcePathIndices"],
        serde_json::json!((0..6).collect::<Vec<_>>())
    );
}

#[test]
fn geometric_union_does_not_discard_distinct_paint_evidence() {
    let mut source = grid();
    let mut changed = source[0].clone();
    changed.stroke_width = 2.;
    source.push(changed);
    let d = detect(&source, &[]);
    assert_eq!(d["canonicalEdgeCount"], 6);
    let contributors = d["tables"][0]["sourceBoundarySegments"].as_array().unwrap();
    assert_eq!(contributors.len(), 7);
    assert_eq!(
        contributors.iter().find(|c| c["path"] == 6).unwrap()["strokeWidth"],
        2.
    );
}

#[test]
fn linear_grids_require_a_verified_parent_cell() {
    let one_row = vec![
        line(15., 45., 35., 45.),
        line(15., 65., 35., 65.),
        line(15., 45., 15., 65.),
        line(25., 45., 25., 65.),
        line(35., 45., 35., 65.),
    ];
    assert_eq!(count(&detect(&one_row, &[])), 0);
    let d = detect(&[grid(), one_row].concat(), &[]);
    assert_eq!(count(&d), 2);
    assert_eq!(d["tables"][1]["rows"], 1);
    assert_eq!(d["tables"][1]["columns"], 2);
    assert_eq!(
        d["tables"][1]["hierarchy"]["parent"],
        serde_json::json!({"table":0,"cell":0})
    );
    assert!(!d["tables"][1]["searchRegion"].is_null());
}

#[test]
fn one_column_nested_grid_has_two_real_faces() {
    let one_column = vec![
        line(15., 45., 35., 45.),
        line(15., 55., 35., 55.),
        line(15., 65., 35., 65.),
        line(15., 45., 15., 65.),
        line(35., 45., 35., 65.),
    ];
    let d = detect(&[grid(), one_column].concat(), &[]);
    assert_eq!(count(&d), 2);
    assert_eq!(d["tables"][1]["columns"], 1);
    assert_eq!(d["tables"][1]["cells"].as_array().unwrap().len(), 2);
    assert_eq!(
        d["tables"][1]["topology"]["betti"],
        serde_json::json!([1, 0, 0])
    );
}

#[test]
fn overhanging_join_does_not_create_a_partial_border_in_a_merged_cell() {
    let mut source = grid();
    for path in &mut source {
        path.stroke_width = 1.2;
    }
    source[1] = line(9.4, 40., 40.6, 40.);
    source[1].stroke_width = 1.2;
    let d = detect(&source, &[]);
    assert_eq!(count(&d), 1);
    assert_eq!(d["tables"][0]["cells"].as_array().unwrap().len(), 3);
    assert!(d["tables"][0]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["rowspan"] == 2));
    assert!(d["tables"][0]["sourcePathIndices"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!(1)));
}
fn count(v: &Value) -> usize {
    v["tables"].as_array().unwrap().len()
}
fn span(text: &str, x: f32, y: f32, width: f32) -> TextSpan {
    TextSpan {
        text: text.into(),
        bbox: Rect::new(x, y, width, 5.0),
        font_size: 5.0,
        ..Default::default()
    }
}

#[test]
fn empty_grid_cells_exist_without_text_density() {
    let d = detect(&grid(), &[]);
    assert_eq!(count(&d), 1);
    assert_eq!(d["tables"][0]["rows"], 2);
    assert_eq!(d["tables"][0]["columns"], 2);
    assert_eq!(d["tables"][0]["cells"].as_array().unwrap().len(), 4);
}
#[test]
fn repeated_rules_do_not_create_cells() {
    let mut paths = grid();
    paths.extend(grid());
    assert_eq!(
        detect(&paths, &[])["tables"][0]["cells"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}
#[test]
fn quantization_sized_gaps_are_joined() {
    let mut paths = grid();
    paths.remove(0);
    paths.push(line(10.0, 10.0, 39.8, 10.0));
    paths.push(line(40.2, 10.0, 70.0, 10.0));
    assert_eq!(count(&detect(&paths, &[])), 1);
}
#[test]
fn multiline_content_does_not_create_structural_rows() {
    let spans = vec![
        span("a", 15.0, 45.0, 5.0),
        span("b", 15.0, 52.0, 5.0),
        span("c", 15.0, 59.0, 5.0),
    ];
    let d = detect(&grid(), &spans);
    assert_eq!(d["tables"][0]["rows"], 2);
    assert_eq!(
        d["tables"][0]["cells"][0]["spanIndices"],
        serde_json::json!([0, 1, 2])
    );
}
#[test]
fn absent_vertical_boundary_yields_colspan() {
    let mut paths = grid();
    paths[4] = line(40.0, 10.0, 40.0, 40.0);
    let d = detect(&paths, &[]);
    assert_eq!(d["tables"][0]["cells"].as_array().unwrap().len(), 3);
    assert_eq!(d["tables"][0]["cells"][0]["colspan"], 2);
}
#[test]
fn absent_horizontal_boundary_yields_rowspan() {
    let mut paths = grid();
    paths[1] = line(40.0, 40.0, 70.0, 40.0);
    let d = detect(&paths, &[]);
    assert_eq!(d["tables"][0]["cells"].as_array().unwrap().len(), 3);
    assert_eq!(d["tables"][0]["cells"][0]["rowspan"], 2);
}
#[test]
fn nonrectangular_merge_is_not_invented() {
    let mut paths = grid();
    paths[1] = line(40.0, 40.0, 70.0, 40.0);
    paths[4] = line(40.0, 10.0, 40.0, 40.0);
    let d = detect(&paths, &[]);
    assert_eq!(count(&d), 0);
    assert_eq!(
        d["rejectedComponents"][0]["reason"],
        "non-rectangular cell topology"
    );
}
#[test]
fn partial_boundary_is_not_silently_completed() {
    let mut paths = grid();
    paths[4] = line(40.0, 10.0, 40.0, 55.0);
    assert_eq!(count(&detect(&paths, &[])), 0);
}
#[test]
fn frame_divider_and_underlines_are_not_tables() {
    let mut paths = grid();
    paths.remove(1);
    assert_eq!(count(&detect(&paths, &[])), 0);
    assert_eq!(count(&detect(&[line(10.0, 10.0, 70.0, 10.0)], &[])), 0);
}
#[test]
fn disconnected_grids_remain_separate() {
    let mut paths = grid();
    paths.extend([
        line(100.0, 10.0, 160.0, 10.0),
        line(100.0, 40.0, 160.0, 40.0),
        line(100.0, 70.0, 160.0, 70.0),
        line(100.0, 10.0, 100.0, 70.0),
        line(130.0, 10.0, 130.0, 70.0),
        line(160.0, 10.0, 160.0, 70.0),
    ]);
    assert_eq!(count(&detect(&paths, &[])), 2);
}
#[test]
fn crossing_content_has_no_duplicate_cell_owner() {
    let d = detect(&grid(), &[span("crossing", 35.0, 15.0, 10.0)]);
    assert_eq!(d["ambiguousContent"].as_array().unwrap().len(), 1);
    assert!(d["tables"][0]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["spanIndices"].as_array().unwrap().is_empty()));
}
#[test]
fn invalid_geometry_and_edge_budget_fail_explicitly() {
    assert!(detect_ruled_tables(&[], &grid(), &[], (0.0, 0.0, f32::NAN, 200.0)).is_err());
    let mut paths = grid();
    paths.push(line(f32::NAN, 0.0, 10.0, 0.0));
    assert!(detect_ruled_tables(&[], &paths, &[], (0.0, 0.0, 200.0, 200.0)).is_err());
    let paths: Vec<_> = (0..513).map(|_| line(10.0, 10.0, 70.0, 10.0)).collect();
    assert!(detect_ruled_tables(&[], &paths, &[], (0.0, 0.0, 200.0, 200.0)).is_err());
}
#[test]
fn source_pdf_grid_owns_all_table_content_and_both_images() {
    let repo = std::env::var("RHWP_POC_REPO").unwrap();
    let bytes = std::fs::read(
        std::path::Path::new(&repo).join("pdf/issue5875/nested_table_text_caption-2020.pdf"),
    )
    .unwrap();
    let snapshot = inspect_page(&bytes, 0).unwrap();
    let d = &snapshot["tableDetection"];
    assert_eq!(count(d), 1);
    assert_eq!(d["tables"][0]["rows"], 5);
    assert_eq!(d["tables"][0]["columns"], 3);
    assert_eq!(d["tables"][0]["cells"].as_array().unwrap().len(), 15);
    let cells = d["tables"][0]["cells"].as_array().unwrap();
    let image_cell = cells
        .iter()
        .find(|c| c["row"] == 2 && c["column"] == 2)
        .unwrap();
    assert_eq!(image_cell["imageIndices"], serde_json::json!([0, 1]));
    let owned: Vec<_> = cells
        .iter()
        .flat_map(|c| {
            c["spanIndices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i.as_u64().unwrap())
        })
        .collect();
    assert_eq!(owned.len(), snapshot["spans"].as_array().unwrap().len() - 1);
    let unique: std::collections::BTreeSet<_> = owned.iter().collect();
    assert_eq!(unique.len(), owned.len());
    assert!(!owned.contains(&0));
    assert_eq!(
        d["unassignedContent"],
        serde_json::json!([{"kind":"span","index":0}])
    );
    assert!(d["ambiguousContent"].as_array().unwrap().is_empty());
}

#[test]
fn significant_rule_gaps_are_not_filled() {
    let mut paths = grid();
    paths.remove(0);
    paths.push(line(10.0, 10.0, 39.0, 10.0));
    paths.push(line(41.0, 10.0, 70.0, 10.0));
    let d = detect(&paths, &[]);
    assert_eq!(count(&d), 0);
    assert_eq!(d["rejectedComponents"][0]["reason"], "open outer boundary");
}

#[test]
fn nonzero_media_box_origin_is_normalized() {
    let paths: Vec<_> = grid()
        .into_iter()
        .map(|p| {
            let ops = p
                .operations
                .iter()
                .map(|op| match *op {
                    PathOperation::MoveTo(x, y) => PathOperation::MoveTo(x + 10.0, y + 20.0),
                    PathOperation::LineTo(x, y) => PathOperation::LineTo(x + 10.0, y + 20.0),
                    _ => panic!("unexpected fixture path"),
                })
                .collect();
            PathContent::from_operations(ops)
        })
        .collect();
    let translated = detect_ruled_tables(&[], &paths, &[], (10.0, 20.0, 210.0, 220.0)).unwrap();
    assert_eq!(translated, detect(&grid(), &[]));
}
