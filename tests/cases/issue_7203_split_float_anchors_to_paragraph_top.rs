//! [Issue #7203] 자리차지 표의 세로 원점이 경로마다 다르다 — 분할 첫 조각 갈래.
//!
//! 쪼개지는 빈-host 자리차지 표의 첫 조각을 그릴 때, 원점을 앵커 문단의 **저장 줄 vpos**
//! 로 잡았다. 그 좌표는 문단 간격(`spacing_before`)을 이미 지난 자리다. 자리차지 개체의
//! 세로 기준은 문단 상자 상단이고 그 간격은 개체가 아니라 뒤따르는 **줄**에 붙으므로,
//! 조각이 간격만큼 아래로 내려갔다.
//!
//! # 기대값의 출처
//!
//! 한/글 engine 2020 정본 `pdf/hwpctl_API_v2.4-hwp-2020.pdf` 의 자리차지 표 44곳을
//! "표 윗변 − 앞 문단 마지막 줄 바닥" 으로 재면 한 값으로 모인다.
//!
//! ```text
//!   before = 0   n=23   중앙값 +3.17
//!   before > 0   n=21   중앙값 +3.32     <- 앵커의 spacing_before 와 무관하다
//! ```
//!
//! `283 HU = 3.77px` 인 `outer_margin_top` 이며, PDF 괘선 stroke 기준선 차이를 보정하면
//! 일치한다. 같은 규칙을 그림 경로는 이미 쓴다(`float_placement.rs` 의
//! `anchor_y = host_y - spacing_before`).
//!
//! # 이 시험이 잠그는 것
//!
//! 분할 첫 조각의 윗변이 **앵커 문단의 선행 간격만큼 내려가지 않는다**. 간격이 0 인 문단은
//! 보정이 no-op 이므로 종전 좌표가 그대로 유지된다 — 그 불변도 함께 잠근다.
//!
//! 실측(`samples/hwpctl_API_v2.4.hwp`, 렌더 트리 `Table` 노드 `bbox.y` 대 정본 괘선):
//!
//! ```text
//!   쪽   문단    간격(HU)   수정 전    수정 후    정본
//!   52   1274    500        948.40    941.70    940.73
//!   26   528     500        717.30    710.60    709.94
//!   12   176       0        899.00    899.00    898.06   (불변)
//! ```
//!
//! # 잠그지 않는 것
//!
//! 비분할 경로(`fragment_outer_top_px = 0` 으로 바깥여백이 빠지는 갈래)와 쪽나눔(typeset)
//! 쪽 원점은 범위 밖이다 — `#7203` 에 남아 있다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpctl_API_v2.4.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn collect_tables(node: &RenderNode, out: &mut Vec<(f64, Option<usize>)>) {
    if let RenderNodeType::Table(meta) = &node.node_type {
        out.push((node.bbox.y, meta.para_index));
    }
    for child in &node.children {
        collect_tables(child, out);
    }
}

/// `page_index` 쪽에서 문단 `para_index` 가 앵커인 표의 윗변.
fn table_top(core: &DocumentCore, page_index: u32, para_index: usize) -> f64 {
    let page = core
        .build_page_render_tree(page_index)
        .unwrap_or_else(|_| panic!("{page_index}쪽 render tree"));
    let mut tables = Vec::new();
    collect_tables(&page.root, &mut tables);
    tables
        .into_iter()
        .find(|(_, pi)| *pi == Some(para_index))
        .unwrap_or_else(|| panic!("{page_index}쪽에서 문단 {para_index} 의 표를 찾지 못했다"))
        .0
}

/// 앵커에 문단 간격이 있는 조각은 그만큼 내려가지 않는다.
#[test]
fn split_fragment_ignores_the_anchor_leading_gap() {
    // 두 곳 모두 앵커 문단의 저장 선행 간격이 500 HU(6.67px)다.
    let core = core();
    for (page, para, oracle) in [(51u32, 1274usize, 940.73f64), (25, 528, 709.94)] {
        let top = table_top(&core, page, para);
        let delta = top - oracle;
        assert!(
            (0.0..=2.0).contains(&delta),
            "{page}쪽 문단 {para}: 표 윗변 {top:.2} — 정본 {oracle:.2} 대비 {delta:+.2}px. \
             앵커의 선행 간격(6.67px)만큼 내려가면 안 된다 (수정 전 +7.4)",
        );
    }
}

/// 간격이 0 인 앵커는 보정이 no-op — 이미 맞던 좌표를 건드리지 않는다.
#[test]
fn split_fragment_without_a_gap_is_unchanged() {
    let core = core();
    for (page, para, oracle) in [
        (11u32, 176usize, 898.06f64),
        (20, 412, 921.23),
        (22, 440, 704.67),
        (34, 797, 980.05),
    ] {
        let top = table_top(&core, page, para);
        let delta = top - oracle;
        assert!(
            (0.0..=2.0).contains(&delta),
            "{page}쪽 문단 {para}: 표 윗변 {top:.2} — 정본 {oracle:.2} 대비 {delta:+.2}px. \
             간격이 없는 앵커는 종전 좌표를 유지해야 한다",
        );
    }
}

/// 이 문서의 총쪽수는 정답지와 같다 — 보정이 쪽 경계를 움직이지 않는다.
#[test]
fn the_correction_does_not_move_the_page_count() {
    let core = core();
    assert_eq!(
        core.page_count(),
        105,
        "한/글 정답지 105쪽과 같아야 한다 — 이 보정은 쪽 경계를 움직이지 않는다"
    );
}
