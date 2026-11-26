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
