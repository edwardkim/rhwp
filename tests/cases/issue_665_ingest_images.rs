//! 관계형 및 고정형(독립 Print 전쪽 쪽수): ingest 그림·본문 소유·자원·저장 계약.
//! 주석 갱신: 2026-11-02T22:58:12+09:00; 직전 파일 커밋: 2026-11-02T22:46:40+09:00 (cba7849e30a7f3a60df7050e4e228be60f880a5e).
//! 측정: 2026-11-02, 96 DPI, 색상 경계 보정 전 2px silhouette_raw_match_percent.
//! above/between/below/inline/runs/mixed/image-first: 각 1쪽 Native/fresh WASM 100.00%.
//! shared-boxed: 1쪽 Native 91.04%, fresh WASM 91.00%; 두 경로 전쪽 최저도 각각 이 값.
//! runs HWP/HWPX 저장 왕복: 각각 1쪽, 양쪽 100.00%. 모든 쪽수는 독립 한컴 2024 Print와 일치.
//! source d78048a6f0, Native/fresh WASM의 실제 producer SHA는 증적 JSON에 기록.
//! 정식 증적: mydocs/tech/investigations/issue-665/README.md 및 all-pages.tsv, 대표 확대 PNG.
//! 입력: tests/fixtures/issue665, 독립 Print: pdf/issue665. 관계 검사에 실제 px 좌표 핀은 쓰지 않는다.
//! 합성 UTF-16/거부 입력은 구조/API 계약이며 설치 글꼴이나 이미지 픽셀의 시각 근거로 쓰지 않는다.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use rhwp::document_core::builders::exam_paper::{
    build_exam_paper_with_images, IngestImage, IngestImageLimits,
};
use rhwp::model::{control::Control, document::Document, image::Picture};
use rhwp::parser::ingest::{parse_ingest_str, IngestDocument};

struct TempDir(std::path::PathBuf);
impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-665-images-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const PNG: &[u8] = include_bytes!("../fixtures/issue665/img/diagram.png");
fn input(name: &str) -> IngestDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue665")
        .join(format!("{name}.json"));
    parse_ingest_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn assets() -> HashMap<String, IngestImage> {
    HashMap::from([(
        "img/diagram.png".into(),
        IngestImage::from_bytes(PNG.to_vec(), IngestImageLimits::default()).unwrap(),
    )])
}
fn build(name: &str) -> Document {
    build_exam_paper_with_images(&input(name), &assets(), IngestImageLimits::default()).unwrap()
}
fn pictures(doc: &Document) -> Vec<&Picture> {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| {
            if let Control::Picture(p) = c {
                Some(p.as_ref())
            } else {
                None
            }
        })
        .collect()
}
fn check_resources(doc: &Document, count: usize) {
    assert_eq!(
        pictures(doc).len(),
        count,
        "one control per requested placement"
    );
    assert_eq!(
        doc.bin_data_content.len(),
        1,
        "repeated image bytes share one resource"
    );
    assert_eq!(doc.bin_data_content[0].data.load().as_slice(), PNG);
    for pic in pictures(doc) {
        assert!(
            pic.common.treat_as_char,
            "pictures own a position in the text flow"
        );
        assert_eq!(pic.image_attr.bin_data_id, 1);
        assert_eq!(pic.crop.right, pic.img_dim.0 as i32);
        assert_eq!(pic.crop.bottom, pic.img_dim.1 as i32);
        assert_eq!(pic.img_dim, (640 * 75, 160 * 75));
        // Aspect ratio is derived from the source pixels, not a screenshot coordinate.
        assert!(pic.common.width.abs_diff(pic.common.height * 4) <= 2);
    }
}
fn roundtrips(doc: &Document) -> Vec<Document> {
    let hwpx = rhwp::serializer::serialize_hwpx(doc).unwrap();
    assert!(rhwp::serializer::hwpx::package_check::check_package(&hwpx, doc).is_ok());
    let hwp = rhwp::serializer::serialize_hwp(doc).unwrap();
    vec![
        rhwp::parser::parse_document(&hwpx).unwrap(),
        rhwp::parser::parse_document(&hwp).unwrap(),
    ]
}
#[test]
fn issue665_placement_and_embedded_bytes_survive_both_formats() {
    for mode in ["above", "between", "below", "inline", "image-first"] {
        let doc = build(mode);
        for restored in std::iter::once(&doc).chain(roundtrips(&doc).iter()) {
            check_resources(restored, 1);
            let paragraphs = &restored.sections[0].paragraphs;
            let stem = paragraphs
                .iter()
                .position(|p| p.text == "1. Image placement test.")
                .unwrap();
            let first = paragraphs
                .iter()
                .position(|p| p.text == "A. First option.")
                .unwrap();
            let last = paragraphs
                .iter()
                .position(|p| p.text == "B. Second option.")
                .unwrap();
            let image = paragraphs
                .iter()
                .position(|p| p.controls.iter().any(|c| matches!(c, Control::Picture(_))))
                .unwrap();
            assert!(stem < first && first < last);
            match mode {
                "above" | "image-first" => assert!(image < stem),
                "between" => assert!(stem < image && image < first),
                "below" => assert!(last < image),
                "inline" => assert_eq!(image, stem),
                _ => unreachable!(),
            }
            assert_eq!(
                paragraphs
                    .iter()
                    .filter(|p| p.text.starts_with("1. "))
                    .count(),
                1
            );
            assert!(paragraphs.iter().all(|p| !p.text.contains("[이미지:")));
        }
    }
}
#[test]
fn issue665_runs_keep_text_image_boundaries_and_shared_resource() {
    let doc = build("runs");
    for restored in std::iter::once(&doc).chain(roundtrips(&doc).iter()) {
        check_resources(restored, 2);
        let p = restored.sections[0]
            .paragraphs
            .iter()
            .find(|p| p.text.starts_with("1. Before "))
            .unwrap();
        assert_eq!(p.text, "1. Before  after, then  final.");
        for boundary in [
            "1. Before ".chars().count(),
            "1. Before  after, then ".chars().count(),
        ] {
            assert_eq!(
                p.char_offsets[boundary] - p.char_offsets[boundary - 1],
                9,
                "one eight-unit image slot between adjacent ASCII chars"
            );
        }
    }
}
#[test]
fn issue665_shared_passage_and_boxed_paragraph_ownership() {
    let doc = build("shared-boxed");
    for restored in std::iter::once(&doc).chain(roundtrips(&doc).iter()) {
        check_resources(restored, 5);
        let p = &restored.sections[0].paragraphs;
        assert_eq!(p.iter().filter(|p| p.text == "Shared passage.").count(), 1);
        assert_eq!(p.iter().filter(|p| p.text == "Inside  box.").count(), 2);
        let boxed: Vec<_> = p.iter().filter(|p| p.text == "Inside  box.").collect();
        for para in boxed {
            assert!(para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Picture(_))));
            assert_ne!(
                restored.doc_info.para_shapes[para.para_shape_id as usize].border_fill_id,
                1
            );
        }
        let outside: Vec<_> = p
            .iter()
            .filter(|p| {
                p.text.is_empty() && p.controls.iter().any(|c| matches!(c, Control::Picture(_)))
            })
            .collect();
        assert_eq!(
            outside.len(),
            3,
            "passage image plus one root-owned image after each box"
        );
        for para in outside {
            assert_eq!(
                restored.doc_info.para_shapes[para.para_shape_id as usize].border_fill_id,
                1
            );
        }
        for n in [1, 2] {
            let q = p
                .iter()
                .position(|p| p.text == format!("{n}. Question {n}."))
                .unwrap();
            let answer = p
                .iter()
                .enumerate()
                .skip(q + 1)
                .find(|(_, p)| p.text == "A. Answer.")
                .map(|(i, _)| i)
                .unwrap();
            assert!(p[q + 1..answer].iter().any(|p| p.text == "Inside  box."));
        }
    }
}
#[test]
fn issue665_unicode_offsets_follow_utf16_and_control_slots() {
    let mut src = input("runs");
    src.questions[0].auto_number = false;
    src.questions[0].stem_blocks=serde_json::from_value(serde_json::json!([{"type":"paragraph","runs":[{"type":"text","text":"A😀"},{"type":"image","ref":"img/diagram.png"},{"type":"text","text":"B"}]}])).unwrap();
    let doc = build_exam_paper_with_images(&src, &assets(), IngestImageLimits::default()).unwrap();
    for restored in std::iter::once(&doc).chain(roundtrips(&doc).iter()) {
        let p = restored.sections[0]
            .paragraphs
            .iter()
            .find(|p| p.text == "A😀B")
            .unwrap();
        assert_eq!(p.char_offsets.len(), 3);
        assert_eq!(p.char_offsets[1] - p.char_offsets[0], 1);
        assert_eq!(p.char_offsets[2] - p.char_offsets[1], 2 + 8);
    }
}
#[test]
fn issue665_asset_budgets_and_invalid_input_fail_explicitly() {
    let limited = IngestImageLimits {
        max_image_pixels: 1,
        ..Default::default()
    };
    assert!(IngestImage::from_bytes(PNG.to_vec(), limited)
        .unwrap_err()
        .to_string()
        .contains("INGEST_IMAGE_LIMIT"));
    assert!(
        IngestImage::from_bytes(PNG[..PNG.len() / 2].to_vec(), IngestImageLimits::default())
            .is_err()
    );
    assert!(IngestImage::from_bytes(b"<svg/>".to_vec(), IngestImageLimits::default()).is_err());
    let src = input("runs");
    for limits in [
        IngestImageLimits {
            max_image_instances: 1,
            ..Default::default()
        },
        IngestImageLimits {
            max_total_image_bytes: PNG.len() - 1,
            ..Default::default()
        },
    ] {
        assert!(build_exam_paper_with_images(&src, &assets(), limits)
            .unwrap_err()
            .to_string()
            .contains("INGEST_IMAGE_LIMIT"));
    }
    assert!(
        build_exam_paper_with_images(&src, &HashMap::new(), IngestImageLimits::default())
            .unwrap_err()
            .to_string()
            .contains("INGEST_IMAGE_REFERENCE")
    );
    for width in [0.0, -1.0, 1000.0] {
        let mut src = src.clone();
        src.questions[0].media[0].target_w_mm = Some(width);
        assert!(
            build_exam_paper_with_images(&src, &assets(), IngestImageLimits::default())
                .unwrap_err()
                .to_string()
                .contains("INGEST_IMAGE_SIZE")
        );
    }
    let mut bad = src.clone();
    bad.questions[0].media[0].natural_w += 1;
    assert!(
        build_exam_paper_with_images(&bad, &assets(), IngestImageLimits::default())
            .unwrap_err()
            .to_string()
            .contains("INGEST_IMAGE_SIZE")
    );
}
#[test]
fn issue665_run_fields_are_strict() {
    for run in [
        serde_json::json!({"type":"image","ref":"x","placement":"inline"}),
        serde_json::json!({"type":"text","text":"x","ref":"x"}),
        serde_json::json!({"type":"paragraph","runs":[]}),
    ] {
        assert!(serde_json::from_value::<rhwp::parser::ingest::InlineRun>(run).is_err());
    }
}
#[test]
fn issue665_cli_resolves_relative_files_and_rejects_escape() {
    let dir = TempDir::new();
    std::fs::create_dir(dir.path().join("img")).unwrap();
    std::fs::write(dir.path().join("img/diagram.png"), PNG).unwrap();
    let json = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue665/runs.json");
    let local = dir.path().join("ingest.json");
    std::fs::copy(json, &local).unwrap();
    let output = dir.path().join("result.hwpx");
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let result = Command::new(&binary)
        .arg("build-from-ingest")
        .arg(&local)
        .args(["-o"])
        .arg(&output)
        .arg("--json")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let parsed = rhwp::parser::parse_document(&std::fs::read(output).unwrap()).unwrap();
    check_resources(&parsed, 2);
    let mut src: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&local).unwrap()).unwrap();
    src["questions"][0]["media"][0]["id"] = serde_json::json!("../outside.png");
    std::fs::write(&local, serde_json::to_vec(&src).unwrap()).unwrap();
    let denied = dir.path().join("denied.hwpx");
    let result = Command::new(&binary)
        .arg("build-from-ingest")
        .arg(local)
        .arg("-o")
        .arg(&denied)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(!denied.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("INGEST_IMAGE_PATH"));
}

#[test]
fn issue665_distinct_images_and_header_footer_keep_their_owners() {
    let source = input("mixed");
    let other = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/issue665/img/second.png"),
    )
    .unwrap();
    let mut images = assets();
    images.insert(
        "img/second.png".into(),
        IngestImage::from_bytes(other.clone(), IngestImageLimits::default()).unwrap(),
    );
    let doc = build_exam_paper_with_images(&source, &images, IngestImageLimits::default()).unwrap();
    for restored in std::iter::once(&doc).chain(roundtrips(&doc).iter()) {
        let pics = pictures(restored);
        assert_eq!(pics.len(), 3);
        assert_eq!(restored.bin_data_content.len(), 2);
        for (i, expected) in [PNG, other.as_slice(), PNG].iter().enumerate() {
            let storage_id = restored.doc_info.bin_data_list
                [usize::from(pics[i].image_attr.bin_data_id) - 1]
                .storage_id;
            let data = restored
                .bin_data_content
                .iter()
                .find(|data| data.id == storage_id)
                .expect("picture resolves its resource");
            assert_eq!(data.data.load().as_slice(), *expected);
        }
        let controls = &restored.sections[0].paragraphs[0].controls;
        let headers: Vec<_> = controls
            .iter()
            .filter_map(|c| {
                if let Control::Header(h) = c {
                    Some(h.paragraphs[0].text.as_str())
                } else {
                    None
                }
            })
            .collect();
        let footers: Vec<_> = controls
            .iter()
            .filter_map(|c| {
                if let Control::Footer(h) = c {
                    Some(h.paragraphs[0].text.as_str())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(headers, ["Type A Exam images"]);
        assert_eq!(footers, ["1/1"]);
    }
}

#[test]
fn issue665_generated_pages_match_independent_print_without_content_loss() {
    for (name, count) in [
        ("above", 1),
        ("between", 1),
        ("below", 1),
        ("inline", 1),
        ("image-first", 1),
        ("runs", 2),
        ("shared-boxed", 5),
    ] {
        let doc = build(name);
        let bytes = rhwp::serializer::serialize_hwpx(&doc).unwrap();
        let core = rhwp::document_core::DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            core.page_count(),
            1,
            "{name}: independent full-document Print PDF has one page"
        );
        check_resources(core.document(), count);
        assert_eq!(
            core.document().sections[0]
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>(),
            doc.sections[0]
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>(),
            "page count must not hide missing/duplicated text"
        );
    }
}
