//! M06-f 장면 픽스처 재생 3/5.
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
fn fixture_m_tableader_160x120() {
    assert_fixture("m-tabLeader-160x120");
}

#[test]
fn fixture_m_tableader_240x180() {
    assert_fixture("m-tabLeader-240x180");
}

#[test]
fn fixture_m_tableader_320x240() {
    assert_fixture("m-tabLeader-320x240");
}

#[test]
fn fixture_m_tableader_480x360() {
    assert_fixture("m-tabLeader-480x360");
}

#[test]
fn fixture_m_tableader_640x480() {
    assert_fixture("m-tabLeader-640x480");
}

#[test]
fn fixture_m_tableader_80x60() {
    assert_fixture("m-tabLeader-80x60");
}

#[test]
fn fixture_m_textcontrolmark_160x120() {
    assert_fixture("m-textControlMark-160x120");
}

#[test]
fn fixture_m_textcontrolmark_240x180() {
    assert_fixture("m-textControlMark-240x180");
}

#[test]
fn fixture_m_textcontrolmark_320x240() {
    assert_fixture("m-textControlMark-320x240");
}

#[test]
fn fixture_m_textcontrolmark_480x360() {
    assert_fixture("m-textControlMark-480x360");
}

#[test]
fn fixture_m_textcontrolmark_640x480() {
    assert_fixture("m-textControlMark-640x480");
}

#[test]
fn fixture_m_textcontrolmark_80x60() {
    assert_fixture("m-textControlMark-80x60");
}

#[test]
fn fixture_m_textdecoration_160x120() {
    assert_fixture("m-textDecoration-160x120");
}

#[test]
fn fixture_m_textdecoration_240x180() {
    assert_fixture("m-textDecoration-240x180");
}

#[test]
fn fixture_m_textdecoration_320x240() {
    assert_fixture("m-textDecoration-320x240");
}

#[test]
fn fixture_m_textdecoration_480x360() {
    assert_fixture("m-textDecoration-480x360");
}

#[test]
fn fixture_m_textdecoration_640x480() {
    assert_fixture("m-textDecoration-640x480");
}

#[test]
fn fixture_m_textdecoration_80x60() {
    assert_fixture("m-textDecoration-80x60");
}

#[test]
fn fixture_m_textrun_160x120() {
    assert_fixture("m-textRun-160x120");
}

#[test]
fn fixture_m_textrun_240x180() {
    assert_fixture("m-textRun-240x180");
}

#[test]
fn fixture_m_textrun_320x240() {
    assert_fixture("m-textRun-320x240");
}

#[test]
fn fixture_m_textrun_480x360() {
    assert_fixture("m-textRun-480x360");
}

#[test]
fn fixture_m_textrun_640x480() {
    assert_fixture("m-textRun-640x480");
}

#[test]
fn fixture_m_textrun_80x60() {
    assert_fixture("m-textRun-80x60");
}

#[test]
fn fixture_s000_empty() {
    assert_fixture("s000-empty");
}

#[test]
fn fixture_s001_background() {
    assert_fixture("s001-background");
}

#[test]
fn fixture_s002_rect() {
    assert_fixture("s002-rect");
}

#[test]
fn fixture_s003_line() {
    assert_fixture("s003-line");
}

#[test]
fn fixture_s004_reorder() {
    assert_fixture("s004-reorder");
}

#[test]
fn fixture_s005_text() {
    assert_fixture("s005-text");
}

#[test]
fn fixture_s006_gradient_rect() {
    assert_fixture("s006-gradient-rect");
}

#[test]
fn fixture_s007_image() {
    assert_fixture("s007-image");
}

#[test]
fn fixture_s100_pagebackground() {
    assert_fixture("s100-pageBackground");
}

#[test]
fn fixture_s101_textrun() {
    assert_fixture("s101-textRun");
}

#[test]
fn fixture_s102_charoverlap() {
    assert_fixture("s102-charOverlap");
}

#[test]
fn fixture_s103_textcontrolmark() {
    assert_fixture("s103-textControlMark");
}

#[test]
fn fixture_s104_tableader() {
    assert_fixture("s104-tabLeader");
}

#[test]
fn fixture_s105_textdecoration() {
    assert_fixture("s105-textDecoration");
}

#[test]
fn fixture_s106_footnotemarker() {
    assert_fixture("s106-footnoteMarker");
}

#[test]
fn fixture_s107_line() {
    assert_fixture("s107-line");
}
