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
/// 0-기반 쪽 번호 — 정본 489쪽.
const PAGE: u32 = 488;
const RUN_HEAD: &str = " TABLESPACE(ROLLBACK_DATA),";
/// 독립 PDF가 증명하는 ASCII 반각 비율. 절대 픽셀 위치를 고정하지 않는다.
const ORACLE_ASCII_ADVANCE_EM: f64 = 0.5;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn open(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(path).expect("검증 원본 읽기")).expect("문서 열기")
}

fn tablespace_run(core: &DocumentCore) -> (f64, String, f64, f64) {
    let page = core.build_page_render_tree(PAGE).expect("489쪽 렌더 트리");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    nodes
        .iter()
        .find_map(|line| {
            if !matches!(line.node_type, RenderNodeType::TextLine(_)) {
                return None;
            }
            line.children.iter().find_map(|node| match &node.node_type {
                RenderNodeType::TextRun(run) if run.text.starts_with(RUN_HEAD) => Some((
                    node.bbox.width,
                    run.text.clone(),
                    run.style.font_size * run.style.ratio,
                    line.bbox.width,
                )),
                _ => None,
            })
        })
        .expect("489쪽 TABLESPACE 런과 소유 글줄")
}

/// HWPX 쌍둥이의 같은 런도 저장 줄폭 안에 들고 자당 전진폭이 정본(em/2)이다.
///
/// 수정 전: HWPX 쌍둥이만 비례 폭(자당 0.737em, 런 폭 804.7px)으로 남았다.
#[test]
fn hwpx_twin_hft_ascii_run_fits_stored_line_and_matches_oracle_advance() {
    let (width, text, em, line_width) = tablespace_run(&open(HWPX_TWIN));
    assert!(text.is_ascii(), "표본 런이 ASCII 전용이 아니다: {text:?}");
    assert!(
        width <= line_width + rhwp::renderer::hwpunit_to_px(4, 96.0),
        "HWPX 쌍둥이 런 폭 {width}가 소유 글줄 폭 {line_width}를 넘는다"
    );
    let per_char_em = width / text.chars().count() as f64 / em;
    assert!(
        (per_char_em - ORACLE_ASCII_ADVANCE_EM).abs() <= 0.02,
        "ASCII 자당 전진폭이 독립 정본의 반각 비율과 다르다: {per_char_em}em"
    );
}

// HWP5 쌍둥이의 전진폭 동등 검사는 #7445로 분리했다. 직접 비교에서 해당 쪽이
// Native/WASM 모두 37.61557%였으므로, 이 원본을 새 시각 회귀의 정상 대조군으로 쓰지 않는다.
// 문서 자체와 독립 PDF는 보존한다.

/// 반례 — 같은 legacy HFT 이름을 쓰는 진짜 HWP5 의 HWPX 판은 저장 줄이 반각을 증언하지 않는다.
///
/// 정본 `pdf/exam_kor-2022.pdf` 6쪽의 큰 숫자 `6`은 장평 반영 em 대비 약 0.645이다.
/// 원본 장평을 함께 반영하여 비례 전진폭이 반각으로 바뀌지 않는지 확인한다.
#[test]
fn modern_hwpx_with_legacy_hft_names_keeps_proportional_ascii() {
    let core = open("samples/hwpx/exam_kor.hwpx");
    let page = core.build_page_render_tree(5).expect("page 6");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let six = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.display_or_text() == "6" => {
                Some((n.bbox.width, r.style.font_size * r.style.ratio))
            }
            _ => None,
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("6쪽의 큰 '6' 런");
    assert!(
        six.0 / six.1 > 0.60,
        "저장 줄 증언이 없는 문서에 반각 규칙이 발화했다 — '6' 전진폭 {}em (독립 정본 약 0.645em)",
        six.0 / six.1,
    );
}
