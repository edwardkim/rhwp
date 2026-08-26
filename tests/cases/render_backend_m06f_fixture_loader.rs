//! M06-f 픽스처 매니페스트·파서 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::render_backend::{
    fixture_root, load_manifest, load_scene_fixtures, parse_fixture_json, FixtureScene,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn manifest_ids_match_files() {
    let manifest = load_manifest(&manifest_dir()).unwrap();
    assert!(manifest.scene_count >= 196);
    let files = load_scene_fixtures(&manifest_dir()).unwrap();
    assert_eq!(files.len(), manifest.scene_count);
    let file_ids: Vec<String> = files.iter().map(|(_, f)| f.scene.id.clone()).collect();
    assert_eq!(file_ids, manifest.ids);
    assert!(fixture_root(&manifest_dir()).join("scenes").is_dir());
}

#[test]
fn parse_roundtrip_first_scene() {
    let files = load_scene_fixtures(&manifest_dir()).unwrap();
    let (_, first) = &files[0];
    let json = first.to_json_value();
    let parsed = parse_fixture_json(&json).unwrap();
    assert_eq!(parsed.scene.id, first.scene.id);
    assert_eq!(parsed.scene.ops.len(), first.scene.ops.len());
    assert_eq!(parsed.schema, FixtureScene::SCHEMA);
}

#[test]
fn parse_rejects_bad_schema() {
    let err = parse_fixture_json(
        r#"{"schema":99,"id":"x","width":1.0,"height":1.0,"contract":"c","ops":[],"expectedKinds":[],"expectedTrace":null}"#,
    )
    .unwrap_err();
    assert!(err.contains("schema"), "{err}");
}
