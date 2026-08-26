//! M06-f 장면 픽스처 재생 5/5.
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
fn fixture_s701_rect_x_1() {
    assert_fixture("s701-rect-x-1");
}

#[test]
fn fixture_s702_rect_x_7() {
    assert_fixture("s702-rect-x-7");
}

#[test]
fn fixture_s703_rect_x_13() {
    assert_fixture("s703-rect-x-13");
}

#[test]
fn fixture_s704_rect_x_50() {
    assert_fixture("s704-rect-x-50");
}

#[test]
fn fixture_s705_rect_x_99() {
    assert_fixture("s705-rect-x-99");
}

#[test]
fn fixture_s706_rect_x_150() {
    assert_fixture("s706-rect-x-150");
}

#[test]
fn fixture_s707_rect_x_200() {
    assert_fixture("s707-rect-x-200");
}

#[test]
fn fixture_s708_rect_x_250() {
    assert_fixture("s708-rect-x-250");
}

#[test]
fn fixture_s709_rect_x_300() {
    assert_fixture("s709-rect-x-300");
}

#[test]
fn fixture_s710_rect_x_350() {
    assert_fixture("s710-rect-x-350");
}

#[test]
fn fixture_s711_rect_x_389() {
    assert_fixture("s711-rect-x-389");
}

#[test]
fn fixture_s800_rect_y_0() {
    assert_fixture("s800-rect-y-0");
}

#[test]
fn fixture_s801_rect_y_1() {
    assert_fixture("s801-rect-y-1");
}

#[test]
fn fixture_s802_rect_y_7() {
    assert_fixture("s802-rect-y-7");
}

#[test]
fn fixture_s803_rect_y_13() {
    assert_fixture("s803-rect-y-13");
}

#[test]
fn fixture_s804_rect_y_50() {
    assert_fixture("s804-rect-y-50");
}

#[test]
fn fixture_s805_rect_y_99() {
    assert_fixture("s805-rect-y-99");
}

#[test]
fn fixture_s806_rect_y_150() {
    assert_fixture("s806-rect-y-150");
}

#[test]
fn fixture_s807_rect_y_200() {
    assert_fixture("s807-rect-y-200");
}

#[test]
fn fixture_s808_rect_y_250() {
    assert_fixture("s808-rect-y-250");
}

#[test]
fn fixture_s809_rect_y_289() {
    assert_fixture("s809-rect-y-289");
}

#[test]
fn fixture_s900_pair_rectangle_line() {
    assert_fixture("s900-pair-rectangle-line");
}

#[test]
fn fixture_s901_pair_line_ellipse() {
    assert_fixture("s901-pair-line-ellipse");
}

#[test]
fn fixture_s902_pair_ellipse_path() {
    assert_fixture("s902-pair-ellipse-path");
}

#[test]
fn fixture_s903_pair_path_textrun() {
    assert_fixture("s903-pair-path-textRun");
}

#[test]
fn fixture_s904_pair_textrun_image() {
    assert_fixture("s904-pair-textRun-image");
}

#[test]
fn fixture_s905_pair_image_equation() {
    assert_fixture("s905-pair-image-equation");
}

#[test]
fn fixture_s906_pair_equation_formobject() {
    assert_fixture("s906-pair-equation-formObject");
}

#[test]
fn fixture_s907_pair_formobject_placeholder() {
    assert_fixture("s907-pair-formObject-placeholder");
}

#[test]
fn fixture_s908_pair_placeholder_rawsvg() {
    assert_fixture("s908-pair-placeholder-rawSvg");
}

#[test]
fn fixture_s909_pair_rawsvg_footnotemarker() {
    assert_fixture("s909-pair-rawSvg-footnoteMarker");
}

#[test]
fn fixture_s910_pair_footnotemarker_tableader() {
    assert_fixture("s910-pair-footnoteMarker-tabLeader");
}

#[test]
fn fixture_s911_pair_tableader_textdecoration() {
    assert_fixture("s911-pair-tabLeader-textDecoration");
}

#[test]
fn fixture_s912_pair_textdecoration_charoverlap() {
    assert_fixture("s912-pair-textDecoration-charOverlap");
}

#[test]
fn fixture_s913_pair_charoverlap_textcontrolmark() {
    assert_fixture("s913-pair-charOverlap-textControlMark");
}

#[test]
fn fixture_s914_pair_textcontrolmark_rectangle() {
    assert_fixture("s914-pair-textControlMark-rectangle");
}
