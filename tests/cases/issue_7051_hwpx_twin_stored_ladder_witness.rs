//! [Issue #7051] HWP3 계보 신호가 없는 HWPX 쌍둥이도 HFT 한글 face 의 ASCII 를 반각으로 잰다.
//!
//! `#7313` 은 HWP3 변환 HWP5(`hwp3-sample10-hwp5.hwp`)의 HFT 한글 전용 face ASCII 를 반각으로
//! 고쳤다. 그 경계는 `HwpSummaryInformation` 의 HWP3 시대 연도였고, 같은 문서를 한컴이 HWPX 로
//! 저장한 `hwp3-sample10-hwpx.hwpx` 에는 그 신호가 없어 off-canvas 49건이 남았다
//! (`content.hpf` 메타데이터는 자리표시자, 호환 설정은 일반 HWPX 452개와 같다).
//!
//! 이 문서의 저장 줄 사다리가 반각 조판을 증언한다 — 저장 줄의 글자를 비례 폭으로 재면 줄폭을
//! 넘고 반각으로 재면 들어가는 줄이 1283개, 거꾸로 비례 폭만 설명하는 줄이 38개다
//! (`renderer::hft_ascii_evidence`). samples 1150문서 중 이 증언이 서는 문서는 이 쌍둥이 둘뿐이고,
//! 같은 HFT 이름을 쓰는 진짜 HWP5 `exam_kor` 의 HWPX 판은 증인이 0이다(아래 반례).
//!
//! 정본: `pdf/pr7268/hwp3-sample10-hwp5-p301-600-2024.pdf`(같은 문서의 한컴 2024 출력) 489쪽.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const HWPX_TWIN: &str = "samples/hwp3-sample10-hwpx.hwpx";
const HWP5_TWIN: &str = "samples/hwp3-sample10-hwp5.hwp";
/// 0-기반 쪽 번호 — 정본 489쪽.
const PAGE: u32 = 488;
const RUN_HEAD: &str = " TABLESPACE(ROLLBACK_DATA),";
/// 정본 실측 ASCII 전진폭(px) — 9pt(em 12px)의 절반.
const ORACLE_ASCII_ADVANCE_PX: f64 = 6.02;
/// 저장 LineSeg 줄폭 42520HU 의 96dpi px.
const STORED_LINE_WIDTH_PX: f64 = 42520.0 / 7200.0 * 96.0;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn open(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open")
}

fn tablespace_run(core: &DocumentCore) -> (f64, String) {
    let page = core.build_page_render_tree(PAGE).expect("page 489");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text.starts_with(RUN_HEAD) => {
                Some((n.bbox.width, r.text.clone()))
            }
            _ => None,
        })
        .expect("489쪽의 TABLESPACE 런")
}

/// HWPX 쌍둥이의 같은 런도 저장 줄폭 안에 들고 자당 전진폭이 정본(em/2)이다.
///
/// 수정 전: HWPX 쌍둥이만 비례 폭(자당 0.737em, 런 폭 804.7px)으로 남았다.
#[test]
fn hwpx_twin_hft_ascii_run_fits_stored_line_and_matches_oracle_advance() {
    let (width, text) = tablespace_run(&open(HWPX_TWIN));
    assert!(text.is_ascii(), "표본 런이 ASCII 전용이 아니다: {text:?}");
    assert!(
        width <= STORED_LINE_WIDTH_PX + 0.5,
        "HWPX 쌍둥이 런 폭 {width:.1}px 가 저장 줄폭 {STORED_LINE_WIDTH_PX:.1}px 를 넘는다"
    );
    let per_char = width / text.chars().count() as f64;
    assert!(
        (per_char - ORACLE_ASCII_ADVANCE_PX).abs() <= 0.15,
        "ASCII 자당 전진폭이 정본({ORACLE_ASCII_ADVANCE_PX:.2}px = em/2)과 다르다: {per_char:.3}px"
    );
}

/// 같은 문서의 두 저장본은 같은 폭으로 잰다.
#[test]
fn hwpx_twin_measures_like_the_hwp5_twin() {
    let (hwpx, _) = tablespace_run(&open(HWPX_TWIN));
    let (hwp5, _) = tablespace_run(&open(HWP5_TWIN));
    assert!(
        (hwpx - hwp5).abs() <= 0.5,
        "같은 문서의 HWPX({hwpx:.1}px)·HWP5({hwp5:.1}px) 런 폭이 갈린다"
    );
}

/// 반례 — 같은 legacy HFT 이름을 쓰는 진짜 HWP5 의 HWPX 판은 저장 줄이 반각을 증언하지 않는다.
///
/// 정본 `pdf/exam_kor-2022.pdf` 6쪽의 큰 숫자 `6` 은 26.93px(0.645em)이고 반각이면 20.87px 다.
#[test]
fn modern_hwpx_with_legacy_hft_names_keeps_proportional_ascii() {
    let core = open("samples/hwpx/exam_kor.hwpx");
    let page = core.build_page_render_tree(5).expect("page 6");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let six = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.display_or_text() == "6" && n.bbox.width > 15.0 => {
                Some(n.bbox)
            }
            _ => None,
        })
        .max_by(|a, b| a.width.total_cmp(&b.width))
        .expect("6쪽의 큰 '6' 런");
    assert!(
        six.width > 24.0,
        "저장 줄 증언이 없는 문서에 반각 규칙이 발화했다 — '6' 폭 {:.2}px (정본 26.93px)",
        six.width,
    );
}
