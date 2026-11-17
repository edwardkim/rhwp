//! 관계형: 원본 subset에 없는 글자는 설치된 공개 Serif 프로그램으로 그리고 원본 공급 우선순위를 보존한다.
//! 주석 갱신: 2026-11-18T02:50:40+09:00. 기존 테스트 파일이 없는 신규 API 계약 검사다.
//! 공개 입력은 저장소 Noto KR subset과 OFL RHWP Public Batang이며 시스템 설치 글꼴에 의존하지 않는다.
//! 현재 devel 22da8358의 issue8590 공개 HWP 1쪽에서 Serif-only + subset cache는 한자 여섯 글자 잉크 0,
//! 같은 Serif-only + 빈 cache는 글자별 39~60px였다. 이 검사는 합성 paint의 glyph 선택·잉크·우선순위만 보호한다.
//! Native/fresh WASM·독립 한컴 Print의 전쪽 조판·모양 일치는 사용자 후속 검증이며 이 assertion과 구별한다.

#![cfg(all(feature = "native-skia", target_os = "linux"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::skia::{NativeFontFallbackReport, SkiaLayerRenderer};
use rhwp::renderer::TextStyle;

const FAMILY: &str = "NotoSansKR-Regular";
const CASE: &str = "RHWP_TEST_SYSTEM_CHARACTER_FALLBACK_CASE";
const INPUTS: &str = "RHWP_TEST_SYSTEM_CHARACTER_FALLBACK_INPUTS";
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn xml_text(path: &Path) -> String {
    path.to_string_lossy()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn prepare() -> Scratch {
    let path = std::env::temp_dir().join(format!(
        "rhwp-system-character-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).expect("isolated font inputs");
    for dir in ["primary", "serif", "empty", "cache"] {
        std::fs::create_dir(path.join(dir)).unwrap();
    }
    let primary = rhwp::renderer::native_webfonts::decode_font(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/NotoSansKR-Regular.woff2"
    )))
    .unwrap();
    let serif = rhwp::renderer::native_webfonts::decode_font(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/fonts/RHWPPublicBatang-Regular.woff2"
    )))
    .unwrap();
    let first = ttf_parser::Face::parse(&primary, 0).unwrap();
    let fallback = ttf_parser::Face::parse(&serif, 0).unwrap();
    for ch in "金丙女".chars() {
        assert!(first.glyph_index(ch).is_none(), "subset precondition");
        assert!(fallback.glyph_index(ch).is_some(), "available Serif glyph");
    }
    std::fs::write(path.join("primary/primary.ttf"), primary).unwrap();
    std::fs::write(path.join("serif/serif.ttf"), serif).unwrap();
    for (name, dir) in [("serif", "serif"), ("empty", "empty")] {
        std::fs::write(
            path.join(format!("{name}.conf")),
            format!(
                "<?xml version=\"1.0\"?><!DOCTYPE fontconfig SYSTEM \"urn:fontconfig:fonts.dtd\"><fontconfig><dir>{}</dir><cachedir>{}</cachedir></fontconfig>",
                xml_text(&path.join(dir)),
                xml_text(&path.join("cache"))
            ),
        )
        .unwrap();
    }
    Scratch(path)
}

fn render(paths: &[PathBuf], family: &str, text: &str) -> (Vec<u8>, NativeFontFallbackReport) {
    let operations = text
        .chars()
        .enumerate()
        .map(|(index, ch)| {
            let run = TextRunNode {
                text: ch.to_string(),
                style: TextStyle {
                    font_family: family.into(),
                    font_size: 18.0,
                    color: 0,
                    ..Default::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 20.0,
                field_marker: Default::default(),
                layout_positions: None,
                display_text: None,
            };
            PaintOp::text_run(
                BoundingBox::new(4.0 + index as f64 * 20.0, 4.0, 20.0, 24.0),
                run,
            )
        })
        .collect();
    let tree = PageLayerTree::new(
        72.0,
        32.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 72.0, 32.0), None, operations),
    );
    let renderer = SkiaLayerRenderer::new()
        .with_font_paths(paths)
        .with_font_fallback_reporting(true);
    let pixels = renderer
        .render_pixels_with_options(&tree, RasterRenderOptions::default())
        .expect("actual native CPU replay");
    assert_eq!((pixels.width, pixels.height, pixels.stride), (72, 32, 288));
    (pixels.bytes, renderer.font_fallback_report().unwrap())
}

fn inks(pixels: &[u8]) -> Vec<usize> {
    (0..3)
        .map(|index| {
            (4..28)
                .flat_map(|y| (4 + index * 20..24 + index * 20).map(move |x| (x, y)))
                .filter(|(x, y)| {
                    let p = &pixels[(y * 72 + x) * 4..][..4];
                    p[3] != 0 && p[..3].iter().any(|v| *v < 160)
                })
                .count()
        })
        .collect()
}

fn probe(case: &str, root: &Path) {
    let primary = root.join("primary");
    if case == "missing" {
        let (pixels, report) = render(&[primary], FAMILY, "金丙女");
        assert_eq!(
            inks(&pixels),
            [0, 0, 0],
            "no invented glyph without a supplier"
        );
        assert!(report.missing_requests.iter().any(|name| name == FAMILY));
        return;
    }
    let (pixels, report) = render(std::slice::from_ref(&primary), FAMILY, "金丙女");
    assert!(
        inks(&pixels).iter().all(|count| *count > 0),
        "available installed Serif glyphs must paint: {:?}",
        inks(&pixels)
    );
    assert!(report.missing_requests.is_empty());
    assert!(report
        .records
        .iter()
        .any(|record| record.source == "system" && record.resolved == "RHWP Public Batang"));

    let (hangul, primary_report) = render(std::slice::from_ref(&primary), FAMILY, "가나다");
    assert!(inks(&hangul).iter().all(|count| *count > 0));
    assert!(
        primary_report.records.is_empty(),
        "covered primary glyphs retain their declared program"
    );
    let paths = [primary, root.join("serif")];
    let (declared, declared_report) =
        render(&paths, "NotoSansKR-Regular,RHWP Public Batang", "金丙女");
    assert_eq!(
        pixels, declared,
        "system fallback paints the actual supplied Serif program"
    );
    assert!(
        declared_report.records.is_empty(),
        "explicit fallback precedes the system resolver"
    );
    let (with_extra, _) = render(&paths, FAMILY, "가나다");
    assert_eq!(
        hangul, with_extra,
        "another available face cannot replace covered primary glyphs"
    );
}

#[test]
fn installed_serif_supplies_missing_subset_glyphs_without_overriding_declared_programs() {
    if let Ok(case) = std::env::var(CASE) {
        probe(&case, &PathBuf::from(std::env::var_os(INPUTS).unwrap()));
        println!("RHWP_SYSTEM_CHARACTER_PROBE_COMPLETE:{case}");
        return;
    }
    let scratch = prepare();
    let name = std::thread::current().name().unwrap().to_owned();
    for (case, config) in [("fallback", "serif"), ("missing", "empty")] {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--nocapture", "--test-threads=1"])
            .env(CASE, case)
            .env(INPUTS, &scratch.0)
            .env("FONTCONFIG_FILE", scratch.0.join(format!("{config}.conf")))
            .env("FONTCONFIG_PATH", &scratch.0)
            .env_remove("RHWP_FONT_PATH")
            .output()
            .unwrap();
        assert!(
            output.status.success()
                && String::from_utf8_lossy(&output.stdout)
                    .contains(&format!("RHWP_SYSTEM_CHARACTER_PROBE_COMPLETE:{case}")),
            "{case}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
