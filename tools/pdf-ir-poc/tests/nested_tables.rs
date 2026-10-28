use pdf_oxide::{
    elements::{PathContent, PathOperation},
    geometry::Rect,
    layout::TextSpan,
};
use rhwp_pdf_oxide_poc::{
    recognition::{recognize_page, Policy},
    table_detection::detect_ruled_tables,
    table_hierarchy::{compose, Embedding},
    table_topology,
};
use serde_json::{json, Value};

const PAGE: (f32, f32, f32, f32) = (0., 0., 200., 200.);
fn grid(x: f32, y: f32, size: f32) -> Vec<PathContent> {
    let mut lines = Vec::new();
    for d in [0., size / 2., size] {
        lines.push((x, y + d, x + size, y + d));
        lines.push((x + d, y, x + d, y + size));
    }
    lines
        .into_iter()
        .map(|(a, b, c, d)| {
            PathContent::from_operations(vec![
                PathOperation::MoveTo(a, b),
                PathOperation::LineTo(c, d),
            ])
        })
        .collect()
}
fn nested() -> Vec<PathContent> {
    [
        grid(10., 10., 180.),
        grid(20., 110., 60.),
        grid(25., 145., 20.),
    ]
    .concat()
}
fn span(x: f32, y: f32) -> TextSpan {
    TextSpan {
        text: "x".into(),
        bbox: Rect::new(x, y, 3., 3.),
        font_size: 3.,
        ..Default::default()
    }
}
fn detection(paths: &[PathContent], spans: &[TextSpan]) -> Value {
    detect_ruled_tables(spans, paths, &[], PAGE).unwrap()
}
fn table_with_depth(d: &Value, depth: usize) -> &Value {
    d["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["hierarchy"]["depth"] == depth)
        .unwrap()
}

#[test]
fn three_level_slot_tree_owns_each_span_only_at_the_deepest_cell() {
    let spans = [span(15., 175.), span(22., 135.), span(27., 157.)];
    let d = detection(&nested(), &spans);
    assert_eq!(d["tables"].as_array().unwrap().len(), 3);
    assert!(d["ambiguousContent"].as_array().unwrap().is_empty());
    for depth in 0..3 {
        let t = table_with_depth(&d, depth);
        assert_eq!(t["topology"]["betti"], json!([1, 0, 0]));
        let refs: Vec<_> = t["cells"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|c| c["spanIndices"].as_array().unwrap().iter().cloned())
            .collect();
        assert_eq!(refs, vec![json!(depth)]);
    }
    let r = recognize_page(&spans, &[], &d, PAGE, Policy::default()).unwrap();
    assert!(r["tables"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["status"] == "GeometryConfirmed"));
    for t in r["tables"].as_array().unwrap() {
        let depth = t["depth"].as_u64().unwrap() as usize;
        for c in t["cells"].as_array().unwrap() {
            assert_eq!(
                c["recognition"]["context"].as_array().unwrap().len(),
                3 + 2 * depth
            );
        }
    }
}

#[test]
fn path_order_and_duplicates_do_not_change_the_tree() {
    let a = detection(&nested(), &[]);
    let mut reversed = nested();
    reversed.reverse();
    reversed.extend(nested());
    let b = detection(&reversed, &[]);
    let semantic_tables = |d: &Value| {
        let mut tables = d["tables"].clone();
        for table in tables.as_array_mut().unwrap() {
            let table = table.as_object_mut().unwrap();
            // Source IDs follow the input ordering; geometry, cells and hierarchy do not.
            table.remove("sourceBoundarySegments");
            table.remove("sourcePathIndices");
        }
        tables
    };
    assert_eq!(semantic_tables(&a), semantic_tables(&b));
    assert_eq!(a["canonicalEdgeCount"], b["canonicalEdgeCount"]);
    for (a, b) in a["tables"]
        .as_array()
        .unwrap()
        .iter()
        .zip(b["tables"].as_array().unwrap())
    {
        assert_eq!(
            b["sourcePathIndices"].as_array().unwrap().len(),
            2 * a["sourcePathIndices"].as_array().unwrap().len()
        );
    }
}

#[test]
fn child_across_parent_separator_is_deferred_not_inserted_into_a_cell() {
    // Crosses x=100 and joins the parent ruling graph rather than forming an isolated component.
    let paths = [grid(10., 10., 180.), grid(80., 120., 40.)].concat();
    // This connected crossing cannot be separated by this POC and must not invent a nested tree.
    let d = detection(&paths, &[]);
    assert!(d["tables"].as_array().unwrap().is_empty());
    assert!(!d["rejectedComponents"].as_array().unwrap().is_empty());
    let parent = Rect::new(10., 10., 180., 180.);
    let cells = vec![
        Rect::new(10., 10., 90., 90.),
        Rect::new(100., 10., 90., 90.),
        Rect::new(10., 100., 90., 90.),
        Rect::new(100., 100., 90., 90.),
    ];
    let p = compose(&[(parent, cells), (Rect::new(80., 120., 40., 40.), vec![])]).unwrap();
    assert!(p.iter().all(|p| p.deferred.is_some()));
    assert!(p[1].parent.is_none());
}

#[test]
fn adjacent_tables_are_roots_and_an_inner_frame_is_not_a_child_table() {
    let paths = [grid(10., 10., 60.), grid(110., 10., 60.)].concat();
    let d = detection(&paths, &[]);
    assert!(d["tables"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["hierarchy"]["parent"].is_null()));
    let mut paths = grid(10., 10., 180.);
    paths.push(PathContent::from_operations(vec![
        PathOperation::Rectangle(20., 120., 30., 30.),
    ]));
    let d = detection(&paths, &[]);
    assert_eq!(d["tables"].as_array().unwrap().len(), 1);
    assert!(d["tables"][0]["hierarchy"]["parent"].is_null());
}

#[test]
fn siblings_in_one_slot_have_disjoint_content_ownership() {
    let paths = [
        grid(10., 10., 180.),
        grid(20., 120., 20.),
        grid(60., 120., 20.),
    ]
    .concat();
    let d = detection(&paths, &[span(22., 132.), span(62., 132.)]);
    assert_eq!(d["tables"].as_array().unwrap().len(), 3);
    let parent = table_with_depth(&d, 0);
    assert_eq!(
        parent["cells"][0]["childTableIndices"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert!(parent["cells"]
        .as_array()
        .unwrap()
        .iter()
        .all(|c| c["spanIndices"] == json!([])));
    assert!(d["ambiguousContent"].as_array().unwrap().is_empty());
}

#[test]
fn depth_limit_holds_the_family_without_losing_owned_content() {
    let spans = [span(27., 157.)];
    let d = detection(&nested(), &spans);
    let r = recognize_page(
        &spans,
        &[],
        &d,
        PAGE,
        Policy {
            max_context_depth: 6,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(r["tables"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["status"] == "Deferred"));
    assert!(r["bodyRegions"].as_array().unwrap().is_empty());
}

#[test]
fn forged_parent_is_not_a_substitute_for_geometric_composition() {
    let mut d = detection(&nested(), &[]);
    let child = d["tables"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|t| t["hierarchy"]["depth"] == 1)
        .unwrap();
    child["hierarchy"]["parent"] = json!({"table":999,"cell":0});
    let r = recognize_page(&[], &[], &d, PAGE, Policy::default()).unwrap();
    assert!(r["tables"]
        .as_array()
        .unwrap()
        .iter()
        .all(|t| t["status"] == "Deferred"));
}

#[test]
fn affine_substitution_has_identity_and_associative_composition() {
    let a = Embedding {
        x: 0.1,
        y: 0.2,
        width: 0.5,
        height: 0.6,
    };
    let b = Embedding {
        x: 0.2,
        y: 0.1,
        width: 0.6,
        height: 0.5,
    };
    let c = Embedding {
        x: 0.3,
        y: 0.2,
        width: 0.4,
        height: 0.3,
    };
    assert_eq!(a.compose(Embedding::IDENTITY), a);
    assert_eq!(Embedding::IDENTITY.compose(a), a);
    let left = a.compose(b).compose(c);
    let right = a.compose(b.compose(c));
    for (a, b) in [
        (left.x, right.x),
        (left.y, right.y),
        (left.width, right.width),
        (left.height, right.height),
    ] {
        assert!((a - b).abs() < 1e-6);
    }
}

#[test]
fn topology_detects_holes_disconnection_and_dependent_faces() {
    let full: Vec<_> = (0..3)
        .flat_map(|r| (0..3).map(move |c| (r, c, 1, 1)))
        .collect();
    let good = table_topology::evidence(3, 3, &full).unwrap();
    assert_eq!(good["skeletonCycleRank"], 9);
    assert_eq!(good["rankBoundary2"], 9);
    assert_eq!(good["boundarySquaredZero"], true);
    let ring: Vec<_> = full
        .iter()
        .copied()
        .filter(|(r, c, _, _)| (*r, *c) != (1, 1))
        .collect();
    assert!(table_topology::evidence(3, 3, &ring).is_err());
    assert!(table_topology::evidence(3, 3, &[(0, 0, 1, 1), (2, 2, 1, 1)]).is_err());
    assert!(table_topology::evidence(1, 1, &[(0, 0, 1, 1), (0, 0, 1, 1)]).is_err());
    assert!(table_topology::evidence(usize::MAX, 2, &[]).is_err());
}

#[test]
fn merged_cell_slots_remain_faces_in_the_chain_complex() {
    let v = table_topology::evidence(2, 2, &[(0, 0, 1, 2), (1, 0, 1, 1), (1, 1, 1, 1)]).unwrap();
    assert_eq!(v["faces"], 3);
    assert_eq!(v["betti"], json!([1, 0, 0]));
}

#[test]
fn nested_image_ownership_is_exclusive_and_crossing_content_stays_ambiguous() {
    use pdf_oxide::extractors::{ColorSpace, ImageData, PdfImage};
    let image = PdfImage::with_bbox(
        1,
        1,
        ColorSpace::DeviceRGB,
        8,
        ImageData::Raw {
            pixels: vec![0, 0, 0],
            format: pdf_oxide::extractors::PixelFormat::RGB,
        },
        Rect::new(27., 157., 3., 3.),
    );
    let d = detect_ruled_tables(&[span(33., 157.)], &nested(), &[image], PAGE).unwrap();
    let deepest = table_with_depth(&d, 2);
    assert_eq!(deepest["cells"][0]["imageIndices"], json!([0]));
    // A span crossing a child separator must not fall back into an ancestor cell.
    assert_eq!(d["ambiguousContent"].as_array().unwrap().len(), 1);
    assert!(d["tables"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|t| t["cells"].as_array().unwrap())
        .all(|c| c["spanIndices"] == json!([])));
}

#[test]
fn nested_incidence_and_composition_budgets_are_bounded() {
    assert!(table_topology::evidence(1, 4096, &vec![(0, 0, 1, 4096); 4096]).is_err());
    let invalid = Rect::new(f32::NAN, 0., 10., 10.);
    assert!(compose(&[(invalid, vec![])]).is_err());
    assert!(compose(&vec![(Rect::new(0., 0., 10., 10.), vec![]); 129]).is_err());
}
