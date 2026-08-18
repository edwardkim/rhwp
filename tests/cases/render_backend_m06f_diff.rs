//! M06-f 전어댑터 상호 diff — 같은 입력 추적 공유.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::render_backend::{
    all_families_share_trace, compare_shots, kind_set, shot_from_tree, svg_is_deterministic,
    BackendFamily, PairVerdict, SceneOp, SceneSpec,
};

fn sample() -> SceneSpec {
    SceneSpec::empty("diff-sample", 400.0, 300.0)
        .push(SceneOp::new("rectangle", 20.0, 20.0, 10.0, 10.0))
        .push(SceneOp::new("line", 0.0, 0.0, 50.0, 0.0))
        .push(SceneOp::new("pageBackground", 0.0, 0.0, 400.0, 300.0))
}

#[test]
fn families_share_the_same_trace() {
    let tree = sample().to_layer_tree();
    let trace = all_families_share_trace(&tree).unwrap();
    assert!(trace.starts_with("begin_page 400.00x300.00"), "{trace}");
    assert!(trace.contains("pageBackground"));
    assert!(trace.contains("rectangle"));
    assert!(trace.contains("line"));
}

#[test]
fn same_family_shots_match() {
    let tree = sample().to_layer_tree();
    let a = shot_from_tree(BackendFamily::Svg, &tree).unwrap();
    let b = shot_from_tree(BackendFamily::Svg, &tree).unwrap();
    assert_eq!(compare_shots(&a, &b), PairVerdict::Match);
}

#[test]
fn different_output_family_is_skipped() {
    let tree = sample().to_layer_tree();
    let svg = shot_from_tree(BackendFamily::Svg, &tree).unwrap();
    let png = shot_from_tree(BackendFamily::Png, &tree).unwrap();
    assert_eq!(
        compare_shots(&svg, &png),
        PairVerdict::SkippedDifferentFamily
    );
}

#[test]
fn svg_output_is_deterministic() {
    svg_is_deterministic(&sample().to_layer_tree()).unwrap();
}

#[test]
fn kind_set_counts_tree_not_replay_order() {
    let tree = sample().to_layer_tree();
    let set = kind_set(&tree);
    assert_eq!(set.get("rectangle").copied(), Some(1));
    assert_eq!(set.get("line").copied(), Some(1));
    assert_eq!(set.get("pageBackground").copied(), Some(1));
}

#[test]
fn shot_null_has_three_ops() {
    let tree = sample().to_layer_tree();
    let shot = shot_from_tree(BackendFamily::Null, &tree).unwrap();
    assert_eq!(shot.op_count, 3);
    assert_eq!(shot.family, BackendFamily::Null);
    assert_eq!(shot.caps.name, shot.family.as_str());
}

#[test]
fn shot_trace_has_three_ops() {
    let tree = sample().to_layer_tree();
    let shot = shot_from_tree(BackendFamily::Trace, &tree).unwrap();
    assert_eq!(shot.op_count, 3);
    assert_eq!(shot.family, BackendFamily::Trace);
    assert_eq!(shot.caps.name, shot.family.as_str());
}

#[test]
fn shot_svg_has_three_ops() {
    let tree = sample().to_layer_tree();
    let shot = shot_from_tree(BackendFamily::Svg, &tree).unwrap();
    assert_eq!(shot.op_count, 3);
    assert_eq!(shot.family, BackendFamily::Svg);
    assert_eq!(shot.caps.name, shot.family.as_str());
}

#[test]
fn shot_png_has_three_ops() {
    let tree = sample().to_layer_tree();
    let shot = shot_from_tree(BackendFamily::Png, &tree).unwrap();
    assert_eq!(shot.op_count, 3);
    assert_eq!(shot.family, BackendFamily::Png);
    assert_eq!(shot.caps.name, shot.family.as_str());
}

#[test]
fn shot_skia_has_three_ops() {
    let tree = sample().to_layer_tree();
    let shot = shot_from_tree(BackendFamily::Skia, &tree).unwrap();
    assert_eq!(shot.op_count, 3);
    assert_eq!(shot.family, BackendFamily::Skia);
    assert_eq!(shot.caps.name, shot.family.as_str());
}
