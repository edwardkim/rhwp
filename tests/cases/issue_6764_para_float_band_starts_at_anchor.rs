//! [Issue #6764] 문단-기준 **자리차지(TopAndBottom)** 표가 남긴 배제 밴드를,
//! 뒤따르는 **블록 표**가 짚지 않고 그냥 통과했다.
//!
//! `apply_visible_float_exclusions` 는 흐름이 밴드 **안에서 시작할 때만**(`starts_in_zone`)
//! 바닥으로 끌어올린다. 겹침 프로브(`overlaps_zone`)는 HWPX 저장 프로파일 전용이라,
//! 네이티브 HWP5 에서 `vertical_offset` 이 앵커와 표 상단 사이를 벌려 놓으면 그 틈에서
//! 시작하는 후속 항목은 두 술어 어느 쪽도 못 맞춘다.
//!
//! `1613000-202200037` (항공교통관제사 CBTA 최종보고서) 182쪽 실측:
//!
//! ```text
//!   pi=1 cur_h= 94.0    ← 자리차지 표 배치, 밴드 184.2 .. 960.6
//!   pi=4 cur_h=150.0    ← 46x3 표가 틈 안에서 시작 → 밴드를 통과
//!   DIAG_SPLITSCAN pi=4 cursor=0 end_row=23 consumed=791.8 avail=817.6
//! ```
//!
//! 계상은 예산 817.6px 으로 23행을 같은 쪽에 잘라 넣었지만 페인트는 밴드 아래
//! `y = 1216 .. 2008` — 용지(1122.5px) 밖 **885.6px**. 그 23행이 인쇄·PDF 어디에도
//! 남지 않아 「과목 2: 인적 요소」 장 전체가 사라졌다.
//!
//! ⭐ 표는 한 덩어리라 어긋남이 **쪽 규모**로 드러난다. 그래서 수정은 문단 흐름이
//! 아니라 **블록 표 배치 직전**으로 좁혔다 — `apply_float_band_before_block_table`.
//! 같은 문단이 만든 밴드는 건드리지 않는다(co-anchored float 스택, `#1510`).
//!
//! ⚠ 문단 흐름 쪽은 남는다 — 같은 쪽 `pi=2 / pi=3` 두 줄은 여전히 밴드 위에 계상돼
//! 본문 하한 아래(`+61.5px`)에 그려진다. 밴드 상단을 앵커까지 내려 그 두 줄까지
//! 잡으려 했더니 `#5701`(후속 문단이 1쪽에 남아야 한다)·`#5941`(202쪽 핀)이 깨졌다.
//!
//! 실측(`layout-anomaly`): 용지밖 6 → 5건, **표 최대 초과 885.6 → 13.4px**,
//! 넘침 63 → 62, 쪽수 200 → **201** (한/글 2024 = 204쪽 — 방향이 맞다).
//!
//! ⚠ 재현물은 12.8MB 라 저장소에 넣지 않는다(`.hwp` 는 `ir_field_sweep_baseline`
//! 이 `samples/` 전체를 훑는다). 코퍼스에 없으면 건너뛴다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const CORPUS_REL: &str = concat!(
    "prism_downloads/국토교통부/",
    "1613000-202200037_D0150004-1-001_[최종보고서]항공교통관제사 CBTA 체계 도입 연구 방안 용역 Ver.6 [최종].hwp"
);

/// 코퍼스 문서 경로. `RHWP_ISSUE_6764_DOC` 로 덮어쓸 수 있다.
fn document_path() -> Option<PathBuf> {
    if let Ok(explicit) = std::env::var("RHWP_ISSUE_6764_DOC") {
        let path = PathBuf::from(explicit);
        return path.is_file().then_some(path);
    }
    let root = std::env::var("RHWP_CORPUS_ROOT")
        .unwrap_or_else(|_| "C:/Users/planet/hwpdocs_10k_share".to_string());
    let path = PathBuf::from(root).join(CORPUS_REL);
    path.is_file().then_some(path)
}

fn paper_bottom(node: &RenderNode) -> f64 {
    node.bbox.y + node.bbox.height
}

/// 이 쪽에서 용지 아래로 가장 많이 삐져나간 `Table` 의 초과폭.
fn worst_table_overhang(node: &RenderNode, paper_bottom: f64) -> f64 {
    let mut worst = 0.0f64;
    if matches!(node.node_type, RenderNodeType::Table { .. }) {
        worst = worst.max(node.bbox.y + node.bbox.height - paper_bottom);
    }
    for child in &node.children {
        worst = worst.max(worst_table_overhang(child, paper_bottom));
    }
    worst
}

/// 어떤 쪽에서도 표가 용지 아래로 **쪽 규모**로 나가면 안 된다.
///
/// 수정 전: 182쪽 46x3 표의 첫 조각이 `y = 1216 .. 2008` (용지 1122.5) — `+885.6px`.
/// 남아 있는 최대 초과는 다른 축(#6764 밖)의 `13.4px` 다.
#[test]
fn para_float_band_keeps_the_next_table_fragment_on_paper() {
    let Some(path) = document_path() else {
        eprintln!("[#6764] 코퍼스 문서 없음 — 건너뜀 ({CORPUS_REL})");
        return;
    };
    let bytes = std::fs::read(&path).expect("#6764 코퍼스 문서 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let mut worst = 0.0f64;
    let mut worst_page = 0u32;
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page as u32)
            .expect("render tree");
        let bottom = paper_bottom(&tree.root);
        let overhang = worst_table_overhang(&tree.root, bottom);
        if overhang > worst {
            worst = overhang;
            worst_page = page as u32 + 1;
        }
    }

    assert!(
        worst < 100.0,
        "표가 용지 밖으로 쪽 규모로 나가면 안 된다 — #6764 회귀          \
         (최대 초과 {worst:.1}px @ {worst_page}쪽; 수정 전 +885.6px @ 182쪽)"
    );
}
