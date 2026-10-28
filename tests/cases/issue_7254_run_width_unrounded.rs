//! [#7254] 배치가 run 폭을 정수 px 로 반올림해 줄 나눔과 다른 값을 쓴다.
//!
//! ## 무엇이 문제였나
//!
//! 줄 나눔(`renderer/composer/line_breaking.rs`)은 반올림하지 않은 폭으로 줄을 짜는데,
//! 배치(`renderer/layout/paragraph_layout.rs`)는 `estimate_text_width` 의 **정수 폭**을
//! run bbox 와 다음 run 의 원점에 썼다. 곧 같은 줄을 측정과 배치가 다른 폭으로 소비했다
//! (`AGENTS.md` 의 "측정과 배치의 공통 결과"). run 이 한 글자면 그 글자의 전진폭 자체가
//! 반올림 대상이라 run 경계마다 최대 ±0.5px 가 붙고 뒤 run 들이 그만큼 밀린다.
//!
//! 같은 파일의 field run 예외가 이미 같은 사유를 적어 두고 그 갈래만 고쳐 뒀다
//! (`#3216`·`#1100`: *"정수 반올림을 하면 뒤의 fwSpace/텍스트 앵커가 SVG 실제 glyph
//! advance 보다 앞선다"*).
//!
//! ## 기대값의 출처 — 한/글 PDF 의 글자 전진폭
//!
//! `pdf/table_scattered_header_rowbreak-2024.pdf`(같은 원본의 한/글 2024 출력) 1쪽 첫 줄
//! `【별표 2】` 를 PyMuPDF `rawdict` 의 `chars[].origin` 으로 재고 96/72 로 환산했다.
//! 내장 글꼴은 `Haansoft Batang`, 크기 9.9519pt(= 13.269px)다.
//!
//! ```text
//!   글자   origin(px)   전진(px)
//!   【     75.479       13.273
//!   별     88.752       13.432
//!   표    102.184       13.273
//!   ␠     115.457        6.717
//!   2     122.174        7.676
//!   】    129.850          —
//! ```
//!
//! 곧 한/글은 `【` 를 **1.0003 em** 으로 전진시킨다 — 정수 px(13.0 = 0.980 em)가 아니다.
//! run 단위 기대 폭은 그 전진폭의 합이다: `【` 13.273 · `별표␠` 33.422 · `2` 7.676.
//!
//! rhwp 는 이 문단을 10pt(13.333px)로 조판하고 PDF 는 9.9519pt(13.269px)로 적었다
//! (0.48% 차). 그래서 허용치는 그 척도 차가 33.4px 짜리 run 에서 만드는 0.16px 보다 작게
//! 잡지 않고 **0.15px** 로 둔다. 수정 전 오차는 0.273 / 0.422 / 0.324px 로 이 허용치를
//! 넘고, 수정 후에는 0.060 / 0.089 / 0.097px 다.
//!
//! ## 비범위
//!
//! metric 표 값 자체가 실물과 다른 축(`#6389`), 특정 글자의 전진폭 결정(`#7092`·`#7080`),
//! 그리고 반증된 `#7192` 의 글꼴 별칭은 이 시험이 다루지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/table_scattered_header_rowbreak.hwp";

/// 한/글 PDF 글자 origin 차로 만든 run 단위 기대 전진폭(px).
const ORACLE_RUN_ADVANCE: &[(&str, f64)] = &[("【", 13.273), ("별표 ", 33.422), ("2", 7.676)];

/// 한/글 PDF 와 rhwp 의 글자 크기 표기 차(9.9519pt vs 10pt)가 33.4px run 에서 만드는 0.16px
/// 보다 작게 잡지 않는다.
const TOLERANCE_PX: f64 = 0.15;

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn core(rel: &str) -> DocumentCore {
    let bytes = std::fs::read(sample(rel)).expect("정식 원본");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 첫 `TextLine` 의 글자 든 `TextRun` 들을 (텍스트, x, 폭) 으로 모은다.
fn first_line_runs(root: &RenderNode) -> Vec<(String, f64, f64)> {
    fn first_line<'a>(node: &'a RenderNode) -> Option<&'a RenderNode> {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            return Some(node);
        }
        node.children.iter().find_map(first_line)
    }
    fn runs(node: &RenderNode, out: &mut Vec<(String, f64, f64)>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            let text = run.display_or_text().to_string();
            if !text.trim().is_empty() {
                out.push((text, node.bbox.x, node.bbox.width));
            }
        }
        for child in &node.children {
            runs(child, out);
        }
    }
    let line = first_line(root).expect("첫 TextLine");
    let mut out = Vec::new();
    runs(line, &mut out);
    out
}

/// 배치가 소비하는 run 폭이 한/글의 실제 전진폭과 같다 — 정수 px 가 아니다.
#[test]
fn run_width_matches_the_hancom_advance_not_an_integer_px() {
    let core = core(SAMPLE);
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let runs = first_line_runs(&tree.root);

    let joined: String = runs.iter().map(|(t, _, _)| t.as_str()).collect();
    assert!(
        joined.starts_with("【별표 2】"),
        "픽스처 전제가 깨졌다 — 1쪽 첫 줄이 `【별표 2】` 가 아니다: {joined:?}"
    );

    for (idx, (expected_text, expected_w)) in ORACLE_RUN_ADVANCE.iter().enumerate() {
        let (text, _, width) = &runs[idx];
        assert_eq!(
            text, expected_text,
            "run {idx} 의 텍스트가 기대와 다르다 — 줄 구성이 바뀌었다"
        );
        let diff = (width - expected_w).abs();
        assert!(
            diff <= TOLERANCE_PX,
            "run {idx} ({text:?}) 폭이 한/글 전진폭과 {diff:.3}px 어긋난다 \
             (rhwp {width:.3} vs 한/글 {expected_w:.3}). 정수 반올림이면 이 값이 나온다."
        );
    }
}

/// 다음 run 의 원점은 앞 run 의 폭만큼만 전진한다 — 폭과 원점이 같은 값에서 나온다.
///
/// 이 시험은 **수정 전에도 통과한다**(수정 전에는 둘 다 반올림한 폭이었다). 결함 검출이
/// 아니라, 폭만 비반올림으로 바꾸고 원점은 그대로 두는 식의 회귀를 막는 가드다.
#[test]
fn next_run_origin_advances_by_the_measured_width() {
    let core = core(SAMPLE);
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let runs = first_line_runs(&tree.root);
    assert!(runs.len() >= 4, "첫 줄 run 이 4개 미만이다: {runs:?}");

    for idx in 0..3 {
        let (text, x, width) = &runs[idx];
        let (next_text, next_x, _) = &runs[idx + 1];
        let drift = (x + width - next_x).abs();
        assert!(
            drift <= 0.01,
            "run {idx} ({text:?}) 끝과 다음 run ({next_text:?}) 원점이 {drift:.3}px 어긋난다 \
             — 폭과 원점이 다른 값에서 나온다"
        );
    }
}
