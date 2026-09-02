//! M06-f 장면 픽스처 재생 2/5.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::render_backend::{
    load_scene_fixtures, parse_fixture_json, replay_page, FixtureScene, RenderBackend, TraceBackend,
};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_named(id: &str) -> FixtureScene {
    let all = load_scene_fixtures(&manifest_dir()).expect("fixtures");
    all.into_iter()
        .map(|(_, f)| f)
        .find(|f| f.scene.id == id)
        .unwrap_or_else(|| panic!("missing fixture {id}"))
}

fn assert_fixture(id: &str) {
    let fixture = load_named(id);
    assert_eq!(fixture.schema, FixtureScene::SCHEMA);
    let tree = fixture.scene.to_layer_tree();
    let mut backend = TraceBackend::new();
    replay_page(&mut backend, &tree).unwrap();
    let trace = backend.finish().unwrap();
    let kinds = fixture.scene.expected_replay_kinds();
    let got_kinds: Vec<&str> = kinds.iter().copied().collect();
    let expect_kinds: Vec<&str> = fixture.expected_kinds.iter().map(String::as_str).collect();
    assert_eq!(got_kinds, expect_kinds, "{id} kinds");
    if let Some(lines) = &fixture.expected_trace {
        let got: Vec<&str> = trace.lines().collect();
        assert_eq!(
            got,
            lines.iter().map(String::as_str).collect::<Vec<_>>(),
            "{id} trace"
        );
    }
}

#[test]
fn fixture_m_image_320x240() {
    assert_fixture("m-image-320x240");
}

#[test]
fn fixture_m_image_480x360() {
    assert_fixture("m-image-480x360");
}

#[test]
fn fixture_m_image_640x480() {
    assert_fixture("m-image-640x480");
}

#[test]
fn fixture_m_image_80x60() {
    assert_fixture("m-image-80x60");
}

#[test]
fn fixture_m_line_160x120() {
    assert_fixture("m-line-160x120");
}

#[test]
fn fixture_m_line_240x180() {
    assert_fixture("m-line-240x180");
}

#[test]
fn fixture_m_line_320x240() {
    assert_fixture("m-line-320x240");
}

#[test]
fn fixture_m_line_480x360() {
    assert_fixture("m-line-480x360");
}

#[test]
fn fixture_m_line_640x480() {
    assert_fixture("m-line-640x480");
}

#[test]
fn fixture_m_line_80x60() {
    assert_fixture("m-line-80x60");
}

#[test]
fn fixture_m_pagebackground_160x120() {
    assert_fixture("m-pageBackground-160x120");
}

#[test]
fn fixture_m_pagebackground_240x180() {
    assert_fixture("m-pageBackground-240x180");
}

#[test]
fn fixture_m_pagebackground_320x240() {
    assert_fixture("m-pageBackground-320x240");
}

#[test]
fn fixture_m_pagebackground_480x360() {
    assert_fixture("m-pageBackground-480x360");
}

#[test]
fn fixture_m_pagebackground_640x480() {
    assert_fixture("m-pageBackground-640x480");
}

#[test]
fn fixture_m_pagebackground_80x60() {
    assert_fixture("m-pageBackground-80x60");
}

#[test]
fn fixture_m_path_160x120() {
    assert_fixture("m-path-160x120");
}

#[test]
fn fixture_m_path_240x180() {
    assert_fixture("m-path-240x180");
}

#[test]
fn fixture_m_path_320x240() {
    assert_fixture("m-path-320x240");
}

#[test]
fn fixture_m_path_480x360() {
    assert_fixture("m-path-480x360");
}

#[test]
fn fixture_m_path_640x480() {
    assert_fixture("m-path-640x480");
}

#[test]
fn fixture_m_path_80x60() {
    assert_fixture("m-path-80x60");
}

#[test]
fn fixture_m_placeholder_160x120() {
    assert_fixture("m-placeholder-160x120");
}

#[test]
fn fixture_m_placeholder_240x180() {
    assert_fixture("m-placeholder-240x180");
}

#[test]
fn fixture_m_placeholder_320x240() {
    assert_fixture("m-placeholder-320x240");
}

#[test]
fn fixture_m_placeholder_480x360() {
    assert_fixture("m-placeholder-480x360");
}

#[test]
fn fixture_m_placeholder_640x480() {
    assert_fixture("m-placeholder-640x480");
}

#[test]
fn fixture_m_placeholder_80x60() {
    assert_fixture("m-placeholder-80x60");
}

#[test]
fn fixture_m_rawsvg_160x120() {
    assert_fixture("m-rawSvg-160x120");
}

#[test]
fn fixture_m_rawsvg_240x180() {
    assert_fixture("m-rawSvg-240x180");
}

#[test]
fn fixture_m_rawsvg_320x240() {
    assert_fixture("m-rawSvg-320x240");
}

#[test]
fn fixture_m_rawsvg_480x360() {
    assert_fixture("m-rawSvg-480x360");
}

#[test]
fn fixture_m_rawsvg_640x480() {
    assert_fixture("m-rawSvg-640x480");
}

#[test]
fn fixture_m_rawsvg_80x60() {
    assert_fixture("m-rawSvg-80x60");
}

#[test]
fn fixture_m_rectangle_160x120() {
    assert_fixture("m-rectangle-160x120");
}

#[test]
fn fixture_m_rectangle_240x180() {
    assert_fixture("m-rectangle-240x180");
}

#[test]
fn fixture_m_rectangle_320x240() {
    assert_fixture("m-rectangle-320x240");
}

#[test]
fn fixture_m_rectangle_480x360() {
    assert_fixture("m-rectangle-480x360");
}

#[test]
fn fixture_m_rectangle_640x480() {
    assert_fixture("m-rectangle-640x480");
}

#[test]
fn fixture_m_rectangle_80x60() {
    assert_fixture("m-rectangle-80x60");
}
