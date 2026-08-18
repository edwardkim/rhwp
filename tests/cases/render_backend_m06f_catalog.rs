//! M06-f 카탈로그·장면 빌더 계약.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::render_backend::{
    builtin_scenes, catalog_invariants_hold, materializable_kinds, materialize_scene_op,
    paint_op_kind, spec_for_kind, SceneOp, HONESTY_TEXT, PAINT_OP_KIND_COUNT, PAINT_OP_KIND_SPECS,
};

#[test]
fn catalog_invariants_and_count() {
    catalog_invariants_hold().unwrap();
    assert_eq!(PAINT_OP_KIND_SPECS.len(), 18);
    assert_eq!(PAINT_OP_KIND_COUNT, PAINT_OP_KIND_SPECS.len());
    let names: Vec<_> = PAINT_OP_KIND_SPECS.iter().map(|s| s.kind).collect();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), names.len());
}

#[test]
fn every_catalog_kind_has_spec() {
    for spec in PAINT_OP_KIND_SPECS {
        let found = spec_for_kind(spec.kind).unwrap();
        assert_eq!(found.kind, spec.kind);
        assert!(found.appears_in_trace);
        assert!(!found.summary_ko.is_empty());
        assert_eq!(found.plane_name(), spec.default_plane.as_str());
    }
    assert!(spec_for_kind("not-a-kind").is_none());
}

#[test]
fn materializable_kinds_roundtrip_paint_op_kind() {
    for kind in materializable_kinds() {
        let op = if *kind == "pageBackground" {
            SceneOp::new(*kind, 0.0, 0.0, 400.0, 300.0)
        } else if *kind == "textRun" {
            SceneOp::new(*kind, 10.0, 20.0, 80.0, 16.0).with_text(HONESTY_TEXT)
        } else if *kind == "image" {
            SceneOp::new(*kind, 0.0, 0.0, 8.0, 8.0).with_image()
        } else {
            SceneOp::new(*kind, 12.0, 24.0, 40.0, 18.0)
        };
        let paint = materialize_scene_op(&op);
        assert_eq!(paint_op_kind(&paint), *kind, "{kind}");
        let b = paint.bounds();
        assert_eq!(b.x, op.bounds.x);
        assert_eq!(b.y, op.bounds.y);
        assert_eq!(b.width, op.bounds.width);
        assert_eq!(b.height, op.bounds.height);
    }
}

#[test]
fn builtin_scenes_ids_are_unique_and_catalogued() {
    let scenes = builtin_scenes();
    let mut ids = std::collections::BTreeSet::new();
    for scene in &scenes {
        assert!(ids.insert(scene.id.clone()), "중복 {}", scene.id);
        assert!(!scene.contract.is_empty());
        assert!(scene.width > 0.0 && scene.height > 0.0);
        for op in &scene.ops {
            assert!(spec_for_kind(&op.kind).is_some(), "{}", op.kind);
        }
    }
    assert!(scenes.len() >= 20);
}

#[test]
fn catalog_row_pagebackground() {
    let spec = spec_for_kind("pageBackground").unwrap();
    assert_eq!(spec.kind, "pageBackground");
    assert_eq!(spec.plane_name(), "background");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_textrun() {
    let spec = spec_for_kind("textRun").unwrap();
    assert_eq!(spec.kind, "textRun");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "vectorText");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_glyphrun() {
    let spec = spec_for_kind("glyphRun").unwrap();
    assert_eq!(spec.kind, "glyphRun");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "vectorText");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_glyphoutline() {
    let spec = spec_for_kind("glyphOutline").unwrap();
    assert_eq!(spec.kind, "glyphOutline");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "vectorText");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_charoverlap() {
    let spec = spec_for_kind("charOverlap").unwrap();
    assert_eq!(spec.kind, "charOverlap");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "vectorText");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_textcontrolmark() {
    let spec = spec_for_kind("textControlMark").unwrap();
    assert_eq!(spec.kind, "textControlMark");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_tableader() {
    let spec = spec_for_kind("tabLeader").unwrap();
    assert_eq!(spec.kind, "tabLeader");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_textdecoration() {
    let spec = spec_for_kind("textDecoration").unwrap();
    assert_eq!(spec.kind, "textDecoration");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "vectorText");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_footnotemarker() {
    let spec = spec_for_kind("footnoteMarker").unwrap();
    assert_eq!(spec.kind, "footnoteMarker");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "vectorText");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_line() {
    let spec = spec_for_kind("line").unwrap();
    assert_eq!(spec.kind, "line");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_rectangle() {
    let spec = spec_for_kind("rectangle").unwrap();
    assert_eq!(spec.kind, "rectangle");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_ellipse() {
    let spec = spec_for_kind("ellipse").unwrap();
    assert_eq!(spec.kind, "ellipse");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_path() {
    let spec = spec_for_kind("path").unwrap();
    assert_eq!(spec.kind, "path");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_image() {
    let spec = spec_for_kind("image").unwrap();
    assert_eq!(spec.kind, "image");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "images");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_equation() {
    let spec = spec_for_kind("equation").unwrap();
    assert_eq!(spec.kind, "equation");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_formobject() {
    let spec = spec_for_kind("formObject").unwrap();
    assert_eq!(spec.kind, "formObject");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_placeholder() {
    let spec = spec_for_kind("placeholder").unwrap();
    assert_eq!(spec.kind, "placeholder");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}

#[test]
fn catalog_row_rawsvg() {
    let spec = spec_for_kind("rawSvg").unwrap();
    assert_eq!(spec.kind, "rawSvg");
    assert_eq!(spec.plane_name(), "flow");
    assert_eq!(spec.feature_name(), "none");
    assert_eq!(spec.survives_flatten, true);
    assert!(spec.appears_in_trace);
}
