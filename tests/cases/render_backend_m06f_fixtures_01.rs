//! M06-f 장면 픽스처 재생 1/5.
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
fn fixture_c00_rectangle_ellipse_path() {
    assert_fixture("c00-rectangle-ellipse-path");
}

#[test]
fn fixture_c01_line_textrun_textdecoration() {
    assert_fixture("c01-line-textRun-textDecoration");
}

#[test]
fn fixture_c02_image_placeholder_rawsvg() {
    assert_fixture("c02-image-placeholder-rawSvg");
}

#[test]
fn fixture_c03_formobject_equation_footnotemarker() {
    assert_fixture("c03-formObject-equation-footnoteMarker");
}

#[test]
fn fixture_c04_charoverlap_tableader_textcontrolmark() {
    assert_fixture("c04-charOverlap-tabLeader-textControlMark");
}

#[test]
fn fixture_c05_rectangle_textrun_image() {
    assert_fixture("c05-rectangle-textRun-image");
}

#[test]
fn fixture_c06_ellipse_path_line() {
    assert_fixture("c06-ellipse-path-line");
}

#[test]
fn fixture_c07_placeholder_formobject_rawsvg() {
    assert_fixture("c07-placeholder-formObject-rawSvg");
}

#[test]
fn fixture_m_charoverlap_160x120() {
    assert_fixture("m-charOverlap-160x120");
}

#[test]
fn fixture_m_charoverlap_240x180() {
    assert_fixture("m-charOverlap-240x180");
}

#[test]
fn fixture_m_charoverlap_320x240() {
    assert_fixture("m-charOverlap-320x240");
}

#[test]
fn fixture_m_charoverlap_480x360() {
    assert_fixture("m-charOverlap-480x360");
}

#[test]
fn fixture_m_charoverlap_640x480() {
    assert_fixture("m-charOverlap-640x480");
}

#[test]
fn fixture_m_charoverlap_80x60() {
    assert_fixture("m-charOverlap-80x60");
}

#[test]
fn fixture_m_ellipse_160x120() {
    assert_fixture("m-ellipse-160x120");
}

#[test]
fn fixture_m_ellipse_240x180() {
    assert_fixture("m-ellipse-240x180");
}

#[test]
fn fixture_m_ellipse_320x240() {
    assert_fixture("m-ellipse-320x240");
}

#[test]
fn fixture_m_ellipse_480x360() {
    assert_fixture("m-ellipse-480x360");
}

#[test]
fn fixture_m_ellipse_640x480() {
    assert_fixture("m-ellipse-640x480");
}

#[test]
fn fixture_m_ellipse_80x60() {
    assert_fixture("m-ellipse-80x60");
}

#[test]
fn fixture_m_equation_160x120() {
    assert_fixture("m-equation-160x120");
}

#[test]
fn fixture_m_equation_240x180() {
    assert_fixture("m-equation-240x180");
}

#[test]
fn fixture_m_equation_320x240() {
    assert_fixture("m-equation-320x240");
}

#[test]
fn fixture_m_equation_480x360() {
    assert_fixture("m-equation-480x360");
}

#[test]
fn fixture_m_equation_640x480() {
    assert_fixture("m-equation-640x480");
}

#[test]
fn fixture_m_equation_80x60() {
    assert_fixture("m-equation-80x60");
}

#[test]
fn fixture_m_footnotemarker_160x120() {
    assert_fixture("m-footnoteMarker-160x120");
}

#[test]
fn fixture_m_footnotemarker_240x180() {
    assert_fixture("m-footnoteMarker-240x180");
}

#[test]
fn fixture_m_footnotemarker_320x240() {
    assert_fixture("m-footnoteMarker-320x240");
}

#[test]
fn fixture_m_footnotemarker_480x360() {
    assert_fixture("m-footnoteMarker-480x360");
}

#[test]
fn fixture_m_footnotemarker_640x480() {
    assert_fixture("m-footnoteMarker-640x480");
}

#[test]
fn fixture_m_footnotemarker_80x60() {
    assert_fixture("m-footnoteMarker-80x60");
}

#[test]
fn fixture_m_formobject_160x120() {
    assert_fixture("m-formObject-160x120");
}

#[test]
fn fixture_m_formobject_240x180() {
    assert_fixture("m-formObject-240x180");
}

#[test]
fn fixture_m_formobject_320x240() {
    assert_fixture("m-formObject-320x240");
}

#[test]
fn fixture_m_formobject_480x360() {
    assert_fixture("m-formObject-480x360");
}

#[test]
fn fixture_m_formobject_640x480() {
    assert_fixture("m-formObject-640x480");
}

#[test]
fn fixture_m_formobject_80x60() {
    assert_fixture("m-formObject-80x60");
}

#[test]
fn fixture_m_image_160x120() {
    assert_fixture("m-image-160x120");
}

#[test]
fn fixture_m_image_240x180() {
    assert_fixture("m-image-240x180");
}
