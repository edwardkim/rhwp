//! [#7287] 빈 host 어울림(Square) 자리차지 표의 윗변이 위쪽 바깥여백만큼 위였다.
//!
//! ## 무엇이 문제였나
//!
//! 어울림 자리차지 표(`wrap=SQUARE` · `treatAsChar=0` · `vertRelTo=PARA` ·
//! `vertAlign=TOP`)에는 `layout.rs` 의 `table_y_start` 체인에 **전용 갈래가 없어**
//! 마지막 폴백(`y_offset` = 흐름 위치)까지 떨어졌다. 그래서 위쪽 바깥여백을 아무도
//! 내지 않았다. 저장 앵커 경로(`native_empty_single_topbottom_table_saved_top` ·
//! 빈 host lane 의 `fragment_outer_top_px`)는 모두 `is_para_topbottom_float` 를
//! 요구하고, `compute_table_y_position` 의 절대 배치 분기도
//! `TopAndBottom | BehindText | InFrontOfText` 만 받는다.
//!
//! 가로는 `#6887` 이 이미 세웠다(`para_relative_left_aligned_outer_margin_left_hu`).
//! 이 변경이 그 세로판이다.
//!
//! ## 기대값의 출처 — 한/글 출력 PDF 의 괘선
//!
//! `pdf/hwpctl_API_v2.4-hwp-2020.pdf`(engine 2020)를 PyMuPDF `get_drawings()` 로 재고
//! 96/72 로 환산했다. 같은 y 근처에 **칸 안쪽 괘선**이 함께 있으므로 표 폭과 일치하는
//! 가로 괘선만 쓴다(`#7203` 에서 핀 하나가 실제로 안쪽 괘선에 맞아 있었다).
//!
//! ```text
//!   쪽    pi      수정 전     정본      Δ전     수정 후     Δ후
//!   64  1555     838.60    841.48   +2.88     842.30   −0.82
//!   93  2404     558.70    561.95   +3.25     562.40   −0.45
//!   94  2426     173.90    177.57   +3.67     177.60   −0.03
//!   95  2469     406.30    409.79   +3.49     410.10   −0.31
//!   96  2509     438.50    441.92   +3.42     442.30   −0.38
//! ```
//!
//! 다섯 건의 어긋남이 선언 `outMargin.top` 283HU(3.77px)에서 괘선 stroke/2(0.32px)를
//! 뺀 값과 같다.
//!
//! ## 비범위 — 가시 host
//!
//! host 에 글자가 있는 어울림 자리차지 표는 host 줄의 흐름이 이미 자리를 정한다.
//! `samples/hwp_table_test-m.hwp` 1쪽 표가 그 대조군으로, 정본 250.77 에 대해 251.0 으로
//! **이미 맞다**. 여백을 더하면 벗어나므로 이 변경은 빈 host 로 한정한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 정본 괘선 두께(0.64px)의 절반과 rhwp 격자 반올림을 덮는 여유.
const TOLERANCE_PX: f64 = 1.2;

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn core(rel: &str) -> DocumentCore {
    let bytes = std::fs::read(sample(rel)).expect("정식 원본");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 해당 쪽에서 `para_index` 가 소유한 최상위 표의 `(y, x, w)`.
fn table_of(core: &DocumentCore, page: u32, para_index: usize) -> Option<(f64, f64, f64)> {
    fn walk(node: &RenderNode, depth: usize, para_index: usize, out: &mut Option<(f64, f64, f64)>) {
        if out.is_none() && depth <= 4 {
            if let RenderNodeType::Table(meta) = &node.node_type {
                if meta.para_index == Some(para_index) && meta.cell_context.is_none() {
                    *out = Some((node.bbox.y, node.bbox.x, node.bbox.width));
                }
            }
        }
        for child in &node.children {
            walk(child, depth + 1, para_index, out);
        }
    }
    let tree = core.build_page_render_tree(page).expect("쪽 render tree");
    let mut out = None;
    walk(&tree.root, 0, para_index, &mut out);
    out
}

/// 빈 host 어울림 자리차지 표가 자기 위쪽 바깥여백만큼 안으로 들어간다.
#[test]
fn empty_host_square_float_top_includes_its_outer_margin() {
    /// `(0-based 쪽, 문단, 정본 윗변, 수정 전 윗변)` — 정본은 위 문서 주석의 실측이다.
    const CASES: &[(u32, usize, f64, f64)] = &[
        (63, 1555, 841.48, 838.60),
        (92, 2404, 561.95, 558.70),
        (93, 2426, 177.57, 173.90),
        (94, 2469, 409.79, 406.30),
        (95, 2509, 441.92, 438.50),
    ];

    let core = core("samples/hwpctl_API_v2.4.hwp");
    for &(page, para_index, oracle_top, before_top) in CASES {
        let (top, _, _) = table_of(&core, page, para_index)
            .unwrap_or_else(|| panic!("{}쪽 문단 {para_index} 의 표를 찾지 못했다", page + 1));
        assert!(
            (top - before_top).abs() > TOLERANCE_PX,
            "{}쪽 pi={para_index} 가 수정 전 자리 {before_top:.2} 에 그대로 있다",
            page + 1
        );
        let diff = (top - oracle_top).abs();
        assert!(
            diff <= TOLERANCE_PX,
            "{}쪽 pi={para_index} 윗변이 한/글 정본과 {diff:.2}px 어긋난다 \
             (rhwp {top:.2} vs 정본 {oracle_top:.2}). 어울림 자리차지 표가 자기 \
             outMargin.top 만큼 안으로 들어갔는지 확인하라.",
            page + 1
        );
    }
}

/// 대조군 — **가시 host** 어울림 자리차지 표는 움직이지 않는다.
///
/// `samples/hwp_table_test-m.hwp` 1쪽 표(pi=3)는 host 에 글자가 있고 `vertOffset > 0` 이라
/// `#5566` 갈래가 자리를 정한다. 정본 `pdf/hwp_table_test-m-hwp-2020.pdf` 1쪽의 표 외곽
/// 가로 괘선은 250.77(x=117.06 w=555.54)이고 rhwp 는 251.0 으로 이미 맞다.
#[test]
fn visible_host_square_float_is_untouched() {
    const ORACLE_TOP: f64 = 250.77;

    let core = core("samples/hwp_table_test-m.hwp");
    let (top, x, width) = table_of(&core, 0, 3).expect("1쪽 문단 3 의 표");
    assert!(
        (width - 555.54).abs() < 1.0 && (x - 117.06).abs() < 1.0,
        "대조군 표가 아니다 (x={x:.2} w={width:.2}) — 픽스처 전제가 깨졌다"
    );
    let diff = (top - ORACLE_TOP).abs();
    assert!(
        diff <= TOLERANCE_PX,
        "가시 host 어울림 표가 움직였다 — 정본과 {diff:.2}px 어긋난다 \
         (rhwp {top:.2} vs 정본 {ORACLE_TOP:.2})"
    );
}
