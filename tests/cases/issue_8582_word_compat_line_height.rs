//! 관계형·고정형 회귀 검사. #8582 원본과 메인테이너 경계 사례.
//! 주석 갱신 2026-11-26 11:26 KST. 직전 파일 변경 c929ec9de0 2026-11-25T23:45:58+09:00.
//! 독립 한컴 Print와 96 DPI의 2px raw 실루엣. 전쪽 TSV · 확대 판독: `mydocs/pr/assets/hyunmyungjanelee-20261126/final/README.md`.
//! `word_gulim_ko` 전 1쪽. Native 100.00%, fresh WASM 100.00% (쪽 순서).
//! `word_gulim_en` 전 1쪽. Native 100.00%, fresh WASM 100.00% (쪽 순서).
//! `word_space_controls` 전 1쪽. Native 100.00%, fresh WASM 100.00% (쪽 순서).
//! `word_height_times` 전 1쪽. Native 100.00%, fresh WASM 100.00% (쪽 순서).
//! `word_height_arial` 전 1쪽. Native 100.00%, fresh WASM 100.00% (쪽 순서).
//!
//! #8582: 호환 대상이 MS 워드인 문서의 저장 줄 없는 문단 줄 높이.
//!
//! 기준: 데스크톱 한컴 2024 인쇄 PDF(Hancom PDF 드라이버)의 글자 기준선 간격(600dpi, 12 HU 단위).
//! 재현물은 `samples/test-image.hwpx` 에서 `targetProgram` 과 글꼴·줄 간격만 바꾼 11pt 문단
//! 16개다(문단 0 `TOP`, 1~16 대상, 17 `END`). 줄 피치:
//! - `hwp_hcr_p100` (HWP201X, 함초롬바탕, 비율 100%): 1100 HU (글자 크기 그대로)
//! - `word_hcr_p100` (MS_WORD, 함초롬바탕, 비율 100%): 1860 HU (= 1100 × 1.691)
//! - `word_malgun_p160` (MS_WORD, 맑은 고딕, 비율 160%): 3043 HU (= 1100 × 1.729 × 1.6)
//! - `word_hcr_fixed1800` (MS_WORD, 고정 1800): 1800 HU (고정값 그대로)
//! - `word_hcr_empty` (MS_WORD, 빈 문단 16개): `TOP` 에서 `END` 까지 32460 HU
//! - `word_mixed_hangul_only` (한글 함초롬바탕 + 영문 맑은 고딕, 한글만): 2371 HU
//! - `word_mixed_with_space` (같은 글자 모양, 빈칸 포함): 1903 HU
//!
//! 수정 전 rhwp 는 호환 대상을 읽지 않아 워드 문서도 글자 크기 그대로(1100 HU) 줄을 세웠다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const HU_PER_PX: f64 = 75.0;

fn load(name: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582")
        .join(format!("{name}.hwpx"));
    let bytes = std::fs::read(&path).expect("#8582 재현물");
    DocumentCore::from_bytes(&bytes).unwrap()
}

fn line_top(node: &RenderNode, para: usize) -> Option<f64> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) && line.section_index == Some(0) {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|child| line_top(child, para))
}

fn tops(name: &str) -> (DocumentCore, Vec<f64>) {
    let core = load(name);
    assert_eq!(core.page_count(), 1, "{name}: 한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let tops = (0..=17)
        .map(|para| line_top(&tree.root, para).unwrap_or(f64::NAN))
        .collect();
    (core, tops)
}

/// 대상 문단 1~16 의 평균 줄 피치(HU).
fn pitch_hu(name: &str) -> f64 {
    let (_, tops) = tops(name);
    (tops[16] - tops[1]) / 15.0 * HU_PER_PX
}

fn assert_near(name: &str, actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{name}: 줄 피치 {actual:.0} HU, 데스크톱 한컴 {expected:.0} HU"
    );
}

#[test]
fn hangul_target_keeps_font_size_line_pitch() {
    assert_near("hwp_hcr_p100", pitch_hu("hwp_hcr_p100"), 1100.0, 8.0);
}

#[test]
fn word_target_scales_percent_line_pitch_by_font() {
    assert_near("word_hcr_p100", pitch_hu("word_hcr_p100"), 1860.0, 8.0);
    assert_near(
        "word_malgun_p160",
        pitch_hu("word_malgun_p160"),
        3043.0,
        10.0,
    );
}

#[test]
fn word_target_keeps_fixed_line_pitch() {
    assert_near(
        "word_hcr_fixed1800",
        pitch_hu("word_hcr_fixed1800"),
        1800.0,
        8.0,
    );
}

#[test]
fn word_target_empty_paragraphs_use_scaled_height() {
    let (_, tops) = tops("word_hcr_empty");
    let span = (tops[17] - tops[0]) * HU_PER_PX;
    assert!(
        (span - 32460.0).abs() <= 60.0,
        "빈 문단 16개를 지난 TOP→END {span:.0} HU, 데스크톱 한컴 32460 HU"
    );
}

/// 한글 함초롬바탕 + 영문 맑은 고딕 글자 모양. 문단 끝 표지(영문 슬롯, 맑은 고딕 한 줄 1902 HU)가
/// 한글만 있는 줄(함초롬바탕 한 줄 1860 HU)보다 크면 함초롬바탕 줄의 아래쪽(1/4, 465 HU)이
/// 더해진다(2371 HU). 같은 글자 모양이라도 빈칸(영문 슬롯)이 끼면 맑은 고딕 한 줄이다(1903 HU).
#[test]
fn word_target_paragraph_mark_font_extends_last_line() {
    assert_near(
        "word_mixed_hangul_only",
        pitch_hu("word_mixed_hangul_only"),
        2371.0,
        10.0,
    );
    assert_near(
        "word_mixed_with_space",
        pitch_hu("word_mixed_with_space"),
        1903.0,
        10.0,
    );
}

#[test]
fn word_gulim_localized_family_names_place_every_line_together() {
    let (_, korean) = tops("word_gulim_ko");
    let (_, english) = tops("word_gulim_en");
    for (para, (ko, en)) in korean.iter().zip(english).enumerate() {
        assert!(
            (ko - en).abs() < 0.01,
            "같은 TTC를 지정한 문단 {para}의 줄 위끝 {ko} / {en}"
        );
    }
}

#[test]
fn word_western_faces_use_the_printed_script_fallback_line_height() {
    let (_, serif) = tops("word_height_times");
    let (_, sans) = tops("word_height_arial");
    let step = |rows: &[f64], start: usize| (rows[start + 4] - rows[start]) / 4.0;
    let western = step(&serif, 1);
    assert!((western - step(&sans, 1)).abs() < 0.01);
    let hangul_serif = step(&serif, 6);
    let hangul_sans = step(&sans, 6);
    assert!(western < hangul_serif && hangul_serif < hangul_sans);
    assert!((step(&serif, 11) - hangul_serif).abs() < 0.01);
    assert!((step(&sans, 11) - hangul_sans).abs() < 0.01);
}

#[test]
fn word_compatibility_uses_the_xml_attribute_of_the_compatible_document() {
    let mut core = load("word_hcr_p100");
    let original = core.document().clone();
    let pitch = |core: &DocumentCore| {
        let page = core.build_page_render_tree(0).unwrap();
        line_top(&page.root, 2).unwrap() - line_top(&page.root, 1).unwrap()
    };
    let word_pitch = pitch(&core);
    let tail = original
        .doc_info
        .hwpx_head_tail
        .as_deref()
        .expect("호환 설정");
    let attr = "targetProgram=\"MS_WORD\"";
    assert!(tail.contains(attr));
    let mut quoted = original.clone();
    quoted.doc_info.hwpx_head_tail = Some(tail.replace(attr, "targetProgram='MS_WORD'"));
    core.set_document(quoted);
    let bytes = core.export_hwpx_native().unwrap();
    core = DocumentCore::from_bytes(&bytes).unwrap();
    assert!((pitch(&core) - word_pitch).abs() < 0.01);

    let mut hangul = original.clone();
    hangul.doc_info.hwpx_head_tail = Some(format!(
        "<!-- {attr} -->{}",
        tail.replace(attr, "targetProgram='HWP201X'")
    ));
    core.set_document(hangul);
    let bytes = core.export_hwpx_native().unwrap();
    core = DocumentCore::from_bytes(&bytes).unwrap();
    assert!(pitch(&core) < word_pitch);
}

#[test]
fn word_single_latin_space_differs_from_mixed_and_repeated_spaces() {
    fn space_widths(node: &RenderNode, widths: &mut [Option<f64>; 4]) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            let slot = match (run.para_index, run.text.as_str()) {
                (Some(1), " ") => Some(0),  // A A
                (Some(2), " ") => Some(1),  // 가 가
                (Some(3), " ") => Some(2),  // A 가
                (Some(5), "  ") => Some(3), // A  A
                _ => None,
            };
            if let Some(i) = slot {
                assert!(widths[i].replace(node.bbox.width).is_none());
            }
        }
        for child in &node.children {
            space_widths(child, widths);
        }
    }
    let mut core = load("word_space_controls");
    core.set_hangul2024_compat(true);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut widths = [None; 4];
    space_widths(&tree.root, &mut widths);
    let [latin, hangul, mixed, double] = widths.map(|w| w.expect("공백 run"));
    assert!(latin < hangul);
    assert!((mixed - hangul).abs() < 0.05);
    assert!((double - 2.0 * hangul).abs() < 0.05);
}

fn word_record(core: &DocumentCore, tag: u16) -> Vec<u32> {
    let mut matches = core
        .document()
        .doc_info
        .extra_records
        .iter()
        .filter(|r| r.tag_id == tag);
    let record = matches.next().expect("필요한 호환 레코드");
    assert!(matches.next().is_none(), "레코드가 중복되지 않아야 한다");
    record
        .data
        .chunks_exact(4)
        .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
        .collect()
}

#[test]
fn hancom_word_target_and_layout_survive_repeated_saves_in_both_directions() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue8582");
    // 한컴에서 같은 문서를 HWP로 저장해 표 55·56 레코드를 직접 읽은 값이다.
    let preset = [0x09e3, 0x00cd_f8ff, 0x4f, 0x07ec, 0];
    for (name, initial_hwpx) in [
        ("word_gulim_ko.hwpx", true),
        ("word_gulim_ko_hancom.hwp", false),
    ] {
        let mut core = DocumentCore::from_bytes(&std::fs::read(dir.join(name)).unwrap()).unwrap();
        for cycle in 0..4 {
            let save_hwp = (cycle % 2 == 0) == initial_hwpx;
            let bytes = if save_hwp {
                core.export_hwp_with_adapter().unwrap()
            } else {
                core.export_hwpx_native().unwrap()
            };
            core = DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(word_record(&core, 30), [2], "{name} 변환 {}", cycle + 1);
            assert_eq!(word_record(&core, 31), preset, "{name} 변환 {}", cycle + 1);
        }
    }
}

#[test]
fn hwp_custom_paragraph_flags_survive_hwpx_and_hwp_without_becoming_word_defaults() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue8582");
    for (name, bits) in [
        ("word_hcr_p100_hwp5_lcp_b3.hwp", 8),
        ("word_hcr_p100_hwp5_lcp_b4.hwp", 16),
        ("word_hcr_p100_hwp5c.hwp", 0),
    ] {
        let source = DocumentCore::from_bytes(&std::fs::read(dir.join(name)).unwrap()).unwrap();
        // 개별 설정에서 한컴이 발행하는 실제 단 좌표로 연속 본문을 복원한다.
        let export = source.hwpx_export_expected_document();
        let body = &export.sections[0].paragraphs;
        let first = &body[1].line_segs[0];
        let next = &body[2].line_segs[0];
        // 한컴이 같은 문단을 HWPX로 저장한 lineSeg의 textheight는 실제 글줄 높이와 같다.
        assert_eq!(first.text_height, first.line_height, "{name}");
        assert!(
            (next.vertical_pos - first.vertical_pos - first.line_height - first.line_spacing).abs()
                <= 1,
            "{name}"
        );
        assert!(
            first.line_height > 0 && first.baseline_distance > 0,
            "{name}"
        );
        assert!(
            next.vertical_pos >= first.vertical_pos + first.line_height,
            "{name}"
        );
        let package = source.export_hwpx_native().unwrap();
        let xml = DocumentCore::from_bytes(&package).unwrap();
        assert_eq!(word_record(&xml, 30), [2], "{name}");
        assert_eq!(word_record(&xml, 31), [0, bits, 0, 0, 0], "{name}");
        let binary = DocumentCore::from_bytes(&xml.export_hwp_with_adapter().unwrap()).unwrap();
        assert_eq!(word_record(&binary, 31), [0, bits, 0, 0, 0], "{name}");
    }
}

#[test]
fn edited_hwpx_header_or_truncated_compatibility_data_uses_the_public_word_settings() {
    use std::io::{Cursor, Read, Write};

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582/word_hcr_p100_hwp5_lcp_b3.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let custom = core.export_hwpx_native().unwrap();
    let original = DocumentCore::from_bytes(&custom).unwrap();
    assert_eq!(word_record(&original, 31), [0, 8, 0, 0, 0]);

    for changed in ["Contents/header.xml", "META-INF/rhwp-layout-compatibility"] {
        let mut source = zip::ZipArchive::new(Cursor::new(&custom)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..source.len() {
            let mut entry = source.by_index(index).unwrap();
            if entry.name().unwrap() != changed {
                output.raw_copy_file(entry).unwrap();
                continue;
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            if changed == "Contents/header.xml" {
                assert!(std::str::from_utf8(&bytes)
                    .unwrap()
                    .contains("hh:baseCharUnitOfIndentOnFirstChar"));
                bytes.extend_from_slice(b"<!-- updated header -->");
            } else {
                bytes.truncate(35);
            }
            let options =
                zip::write::SimpleFileOptions::default().compression_method(entry.compression());
            output.start_file(changed, options).unwrap();
            output.write_all(&bytes).unwrap();
        }
        let changed_package = output.finish().unwrap().into_inner();
        let reopened = DocumentCore::from_bytes(&changed_package).unwrap();
        assert_eq!(word_record(&reopened, 30), [2]);
        assert_eq!(
            word_record(&reopened, 31),
            [0x09e3, 0x00cd_f8ff, 0x4f, 0x07ec, 0],
            "{changed}"
        );
    }
}
