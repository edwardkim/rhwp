//! M06-f 장면 픽스처 재생 4/5.
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
fn fixture_s108_rectangle() {
    assert_fixture("s108-rectangle");
}

#[test]
fn fixture_s109_ellipse() {
    assert_fixture("s109-ellipse");
}

#[test]
fn fixture_s110_path() {
    assert_fixture("s110-path");
}

#[test]
fn fixture_s111_image() {
    assert_fixture("s111-image");
}

#[test]
fn fixture_s112_equation() {
    assert_fixture("s112-equation");
}

#[test]
fn fixture_s113_formobject() {
    assert_fixture("s113-formObject");
}

#[test]
fn fixture_s114_placeholder() {
    assert_fixture("s114-placeholder");
}

#[test]
fn fixture_s115_rawsvg() {
    assert_fixture("s115-rawSvg");
}

#[test]
fn fixture_s200_size_1x1() {
    assert_fixture("s200-size-1x1");
}

#[test]
fn fixture_s201_size_10x10() {
    assert_fixture("s201-size-10x10");
}

#[test]
fn fixture_s202_size_40x30() {
    assert_fixture("s202-size-40x30");
}

#[test]
fn fixture_s203_size_96x96() {
    assert_fixture("s203-size-96x96");
}

#[test]
fn fixture_s204_size_200x150() {
    assert_fixture("s204-size-200x150");
}

#[test]
fn fixture_s205_size_400x300() {
    assert_fixture("s205-size-400x300");
}

#[test]
fn fixture_s206_size_595x842() {
    assert_fixture("s206-size-595x842");
}

#[test]
fn fixture_s207_size_800x600() {
    assert_fixture("s207-size-800x600");
}

#[test]
fn fixture_s208_size_1024x768() {
    assert_fixture("s208-size-1024x768");
}

#[test]
fn fixture_s209_size_1280x720() {
    assert_fixture("s209-size-1280x720");
}

#[test]
fn fixture_s300_rect_grid() {
    assert_fixture("s300-rect-grid");
}

#[test]
fn fixture_s301_text_then_bg() {
    assert_fixture("s301-text-then-bg");
}

#[test]
fn fixture_s302_all_materializable() {
    assert_fixture("s302-all-materializable");
}

#[test]
fn fixture_s400_empty_50x50() {
    assert_fixture("s400-empty-50x50");
}

#[test]
fn fixture_s401_empty_100x200() {
    assert_fixture("s401-empty-100x200");
}

#[test]
fn fixture_s402_empty_300x100() {
    assert_fixture("s402-empty-300x100");
}

#[test]
fn fixture_s403_empty_777x333() {
    assert_fixture("s403-empty-777x333");
}

#[test]
fn fixture_s500_line_stack() {
    assert_fixture("s500-line-stack");
}

#[test]
fn fixture_s501_text_ladder() {
    assert_fixture("s501-text-ladder");
}

#[test]
fn fixture_s502_decorations() {
    assert_fixture("s502-decorations");
}

#[test]
fn fixture_s503_chrome() {
    assert_fixture("s503-chrome");
}

#[test]
fn fixture_s504_shapes() {
    assert_fixture("s504-shapes");
}

#[test]
fn fixture_s505_offset() {
    assert_fixture("s505-offset");
}

#[test]
fn fixture_s506_zero_height_line() {
    assert_fixture("s506-zero-height-line");
}

#[test]
fn fixture_s507_tiny_60() {
    assert_fixture("s507-tiny-60");
}

#[test]
fn fixture_s508_a4_zones() {
    assert_fixture("s508-a4-zones");
}

#[test]
fn fixture_s509_landscape() {
    assert_fixture("s509-landscape");
}

#[test]
fn fixture_s510_overlap_pair() {
    assert_fixture("s510-overlap-pair");
}

#[test]
fn fixture_s600_honesty_text() {
    assert_fixture("s600-honesty-text");
}

#[test]
fn fixture_s601_honesty_gradient() {
    assert_fixture("s601-honesty-gradient");
}

#[test]
fn fixture_s602_honesty_image() {
    assert_fixture("s602-honesty-image");
}

#[test]
fn fixture_s700_rect_x_0() {
    assert_fixture("s700-rect-x-0");
}
