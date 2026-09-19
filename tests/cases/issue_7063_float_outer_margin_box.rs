//! [#7063] 자리차지(T&B) 표의 저장 `horzOffset` 은 **바깥 여백 상자**의 왼끝이다.
//!
//! ## 무엇이 문제였나
//!
//! 표 자신의 왼끝은 저장 `horzOffset` 에서 `outMargin.left` 만큼 안쪽인데, 자리차지
//! 갈래만 그 여백을 싣지 않았다. 같은 쪽에서 글줄 참여 표는 받고 자리차지 표는 못 받아
//! 형제끼리 갈렸다(19쪽 표1 39.70 vs 표2 37.80, 정본은 둘 다 39.66).
//!
//! 저장소에는 같은 규칙의 **좁은 술어 셋**이 이미 있었다 — `#6378`(원본 HWPX·단 기준·
//! RowBreak·사방 균등), `#3820 Stage 120`(저장 되감김 조각), `native_empty_host_
//! physical_outer_box_paint_inset`(빈 host·1열 RowBreak). 셋 다 이 일반 규칙의
//! 부분집합이라, 일반 규칙을 켜면서 뒤 둘의 **가로** 몫은 제거했다(안 그러면 두 번 든다).
//!
//! ## 기대값의 출처 — 한/글 출력 PDF 의 괘선
//!
//! `pdf/hwpx_sample2-hwpx-2020.pdf`(engine 2020)를 PyMuPDF `get_drawings()` 로 재고
//! 96/72 로 환산했다. 표 상자의 왼끝은 **세로 괘선의 x** 다 — 가로 괘선은 stroke 0.64px
//! 만큼 양끝이 더 길어 그대로 쓰면 0.32px 어긋난다.
//!
//! 29쪽 전수에서 자리차지·왼쪽 정렬 표 28건을 뽑아 대조하면 좌단 차가 **예외 없이 선언
//! `outMargin.left`** 다(140HU→1.87 · 141HU→1.88 · 283HU→3.77 · 284HU→3.79px). 여백이
//! 0 인 표는 −0.06px(괘선 stroke/2)로 붙는다. 곧 형상 조건이 아니라 규칙이다.
//!
//! 이 시험은 그 28건에서 **서로 다른 갈래 둘**을 고정한다.
//!
//! ```text
//!   19쪽  pi=182  horzRelTo=PARA    horzOffset=0    outMargin.left=141HU  정본 좌단 39.66
//!   14쪽  pi=128  horzRelTo=COLUMN  horzOffset=158  outMargin.left=283HU  정본 좌단 43.66
//!          pi=129  (같은 값)                                               정본 좌단 43.66
//! ```
//!
//! 14쪽이 반례를 죽인다 — `horzOffset` 이 **0 이 아닌데도** 정본이 `outMargin.left` 를
//! 그대로 더한다(본문 37.80 + 158HU 2.11 + 283HU 3.77 = 43.68 ≈ 정본 43.66). 곧
//! "오프셋이 있으면 여백을 싣지 않는다" 는 가설은 기각이고, `horzRelTo` 도 `PARA`·
//! `COLUMN` 이 같은 규칙이다.
//!
//! ## 비범위
//!
//! **세로축은 이 시험이 다루지 않는다.** 저장 앵커의 세로 원점은 `#7203` 이
//! `src/renderer/stored_float_anchor.rs` 로 따로 정하며(저장 사다리가 자리를 비우는지,
//! `vertical_offset` 이 0 인지 등을 판별), 가로처럼 무조건 더할 수 없다. 19쪽 표2 의 상단은
//! 이 수정 뒤에도 정본 89.82 대비 88.13(−1.69px)이고 그 축은 `#7203`·`#6599` 의 몫이다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpx_sample2.hwpx";
/// 정본 괘선 두께(0.64px)의 절반과 rhwp 격자 반올림을 덮는 여유.
const TOLERANCE_PX: f64 = 0.3;

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn core() -> DocumentCore {
    let bytes = std::fs::read(sample(SAMPLE)).expect("정식 원본");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 쪽의 최상위 표들을 `(x, y, w)` 로, 위에서 아래 순서로 모은다.
fn top_level_tables(root: &RenderNode) -> Vec<(f64, f64, f64)> {
    fn walk(node: &RenderNode, depth: usize, out: &mut Vec<(f64, f64, f64)>) {
        if matches!(node.node_type, RenderNodeType::Table(_)) && depth <= 4 {
            out.push((node.bbox.x, node.bbox.y, node.bbox.width));
        }
        for child in &node.children {
            walk(child, depth + 1, out);
        }
    }
    let mut out = Vec::new();
    walk(root, 0, &mut out);
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    out
}

/// 문단 기준·오프셋 0 — 자리차지 표가 형제 글줄 참여 표와 같은 좌단에 선다 (19쪽).
#[test]
fn para_relative_float_table_left_includes_its_outer_margin() {
    /// 정본 19쪽 세로 괘선. 표1 = 글줄 참여, 표2 = 자리차지. 둘 다 같은 값이다.
    const ORACLE_LEFT: f64 = 39.66;

    let core = core();
    let tree = core.build_page_render_tree(18).expect("19쪽 render tree");
    let tables = top_level_tables(&tree.root);
    assert!(
        tables.len() >= 2,
        "19쪽 최상위 표가 2개 미만이다 — 픽스처 전제가 깨졌다: {tables:?}"
    );

    let (inline_x, _, _) = tables[0];
    let (float_x, _, float_w) = tables[1];
    assert!(
        float_w > 700.0,
        "둘째 표가 본문 폭 표가 아니다 (w={float_w:.1}) — 픽스처 전제가 깨졌다"
    );

    for (label, got) in [("표1(글줄 참여)", inline_x), ("표2(자리차지)", float_x)] {
        let diff = (got - ORACLE_LEFT).abs();
        assert!(
            diff <= TOLERANCE_PX,
            "{label} 좌단이 한/글 정본과 {diff:.2}px 어긋난다 (rhwp {got:.2} vs 정본 \
             {ORACLE_LEFT:.2}). 자리차지 표가 바깥여백만큼 안으로 들어갔는지 확인하라."
        );
    }

    // 두 표는 같은 좌단에 선다 — 한쪽만 여백을 받으면 이 불변식이 먼저 깨진다.
    assert!(
        (inline_x - float_x).abs() <= TOLERANCE_PX,
        "같은 쪽 형제 표의 좌단이 갈린다: 글줄 참여 {inline_x:.2} vs 자리차지 {float_x:.2}"
    );
}

/// 단 기준·오프셋 158HU — 오프셋이 0 이 아니어도 여백은 그대로 실린다 (14쪽).
///
/// "오프셋이 있으면 바깥여백을 싣지 않는다" 는 좁힘의 반례다. 그 가설대로라면 두 표가
/// 39.90(= 본문 37.80 + 158HU)에 서야 하는데 정본은 43.66 이다.
#[test]
fn column_relative_float_table_keeps_its_outer_margin_even_with_a_stored_offset() {
    /// 정본 14쪽 세로 괘선 — 같은 x 를 두 표가 공유한다.
    const ORACLE_LEFT: f64 = 43.66;
    /// 여백을 안 실었을 때의 값. 이 자리에 서면 수정 전 상태다.
    const WITHOUT_MARGIN: f64 = 39.90;

    let core = core();
    let tree = core.build_page_render_tree(13).expect("14쪽 render tree");
    // 선언 폭 52991HU = 706.5px. 같은 쪽의 다른 표(710.5px)와 갈라 고른다.
    const DECLARED_W: f64 = 706.5;
    let wide: Vec<(f64, f64, f64)> = top_level_tables(&tree.root)
        .into_iter()
        .filter(|&(_, _, w)| (w - DECLARED_W).abs() < 1.0)
        .collect();
    assert_eq!(
        wide.len(),
        2,
        "14쪽 선언 폭 {DECLARED_W}px 자리차지 표가 2개가 아니다 — 픽스처 전제가 깨졌다:          {wide:?}"
    );

    for (index, &(x, _, _)) in wide.iter().enumerate() {
        assert!(
            (x - WITHOUT_MARGIN).abs() > TOLERANCE_PX,
            "표{}가 바깥여백 없는 자리 {WITHOUT_MARGIN:.2} 에 그대로 있다 — 수정 전 상태다",
            index + 1
        );
        let diff = (x - ORACLE_LEFT).abs();
        assert!(
            diff <= TOLERANCE_PX,
            "표{} 좌단이 한/글 정본과 {diff:.2}px 어긋난다 (rhwp {x:.2} vs 정본 \
             {ORACLE_LEFT:.2}). 저장 오프셋 158HU 위에 outMargin 283HU 가 더 실려야 한다.",
            index + 1
        );
    }
}

/// #6643에서 이미 바로잡은 블록 wrapper의 좌단도 같은 여백을 한 번만 소비한다.
/// 독립 기준: pdf/80168_regulatory_analysis-2022.pdf 6쪽의 세로 괘선
/// x=79.317px, y=882.715..988.679px (PyMuPDF get_drawings, 96/72 환산).
/// 원점 공통화가 wrapper/안쪽 표의 기존 여백에 다시 더해지면 이 정상 대조군이 실패한다.
#[test]
fn block_wrapper_keeps_its_pdf_left_edge_when_margin_is_already_owned() {
    let bytes = std::fs::read(sample("samples/80168_regulatory_analysis.hwp"))
        .expect("tracked wrapper fixture");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("wrapper document");
    let svg = doc.render_page_svg(5).expect("wrapper page 6");
    let xml = roxmltree::Document::parse(&svg).expect("valid SVG");
    let left = xml
        .descendants()
        .filter(|n| n.has_tag_name("line"))
        .filter_map(|n| {
            let value = |key| n.attribute(key)?.parse::<f64>().ok();
            let (x1, y1, x2, y2) = (value("x1")?, value("y1")?, value("x2")?, value("y2")?);
            ((x1 - x2).abs() < 0.01 && y1.min(y2) > 840.0 && (y2 - y1).abs() > 80.0).then_some(x1)
        })
        .min_by(f64::total_cmp)
        .expect("page 6 bottom table vertical borders");
    assert!(
        (left - 79.317).abs() < 0.4,
        "wrapper left {left:.3}px differs from Hancom 79.317px; apply the outer margin once"
    );
}
