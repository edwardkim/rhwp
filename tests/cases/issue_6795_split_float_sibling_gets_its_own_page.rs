//! [Issue #6795] 쪽에 걸쳐 쪼개진 자리차지 표의 **형제 표**가 그 조각 위에 겹쳐 놓이던
//! 결함의 가드.
//!
//! ## 두 기계가 같은 착각을 한다 — 이어지는 쪽을 빈 쪽으로 본다
//!
//! 빈 host 문단에 매달린 자리차지(`wrap=위아래`, `vert=문단`) 표들은 세로로 쌓여야 한다.
//! 첫 표가 커서 쪽을 넘기면 그 조각은 `PageItem::PartialTable` 로 나가는데, **뒤 표를
//! 배치하는 두 경로가 모두 그 조각을 못 본다.**
//!
//! ```text
//!   ① try_typeset_empty_para_float_table
//!        raw_top 은 para_start_height 기준 → 이어지는 쪽에서는 0.0
//!        실측: cur_h=415.7 인데 raw_top=0.0, lane_bottom 696.0 ≤ available 710.6 → 배치
//!
//!   ② #2813 통짜-배치 구제 (typeset_block_table_inner)
//!        "저장 앵커 줄이 스택 아래이고 본문 안이면 한글이 통째로 이 쪽에 놨다"
//!        실측: bounds=(655.8, 669.8) — 그런데 이 좌표는 **문단이 시작한 쪽**의 것이다
//!        → 구제가 잘못 발동해 advance_column_or_new_page 를 막는다
//! ```
//!
//! ①만 고치면 ②가 받아 같은 자리에 놓고, ②만 고치면 ①이 먼저 놓는다. **둘 다** 고쳐야
//! 닫힌다(실측: `lane` 단독 겹침 1, `guard` 단독 겹침 1, 둘 다 0).
//!
//! ## 재현체 실측 — `1341000-201100013`(사이버대학 인가신청서) 31쪽
//!
//! ```text
//!   PartialTable pi=113 ci=0   y=143.6..560.1   27×10 의 마지막 조각
//!   Table        pi=113 ci=1   y=158.2..854.2   19×6  통짜
//!                              → 548.0 × 401.9px 겹침 (아래 표가 통째로 가려진다)
//! ```
//!
//! 한/글 2018 오라클(문서 `lastSavedWith = 6.7.6.1002`, 설치본 중 최근접)은 27쪽에 조각,
//! **28쪽에 `현장실사 … 위원회 심의결과` 표 단독**, 29쪽에 `XIV. 종합의견` 을 둔다.
//! 수정 후 rhwp 도 같은 순서 · 같은 45쪽이 된다(종전 44쪽).
//!
//! ## 지키는 계약
//!
//! 겹침 자체가 결함이다 — 쪽수는 버전 드리프트가 있는 문서라 판정에 쓰지 않고,
//! **같은 쪽의 자리차지 표 두 장이 겹치지 않는다** 와 **문서 순서가 보존된다** 만 잠근다.
//!
//! ⚠ 기각한 안 두 개를 남긴다. `is_deferred_coanchored_rowbreak_table` 의
//! `vertical_offset > 0` 을 `>= 0` 으로 넓히면 겹침은 사라지지만 표가 `pi=114` 뒤로 가
//! **쪽 순서가 뒤집힌다**. 문단 기준 높이(`para_start_height`)를 쪽 넘김 뒤 다시 잡는
//! 안은 ②가 막아 효과가 0이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

/// 재현물은 코퍼스 문서다.
///
/// `hwpdocs_10k_share/prism_downloads/교육부/1341000-201100013_D0150004-2-002_
/// 2011개교예정_사이버대학인가신청서-제2차보고서-최종.hwp`
///
/// ⚠ `samples/` 에 넣으면 `samples/` 전체를 스윕하는 다른 기준선까지 끌고 온다
/// (`#6599` 와 같은 이유). 코퍼스에서 찾고, 없으면 건너뛴다.
/// `RHWP_ISSUE6795_SAMPLE` 로 경로를 덮어쓸 수 있다.
fn sample() -> Option<Vec<u8>> {
    if let Ok(path) = std::env::var("RHWP_ISSUE6795_SAMPLE") {
        return std::fs::read(path).ok();
    }
    let roots = [
        concat!(
            r"C:\Users\planet\hwpdocs_10k_share",
            r"\prism_downloads\교육부"
        ),
        concat!(r"D:\hwpdocs_10k_share", r"\prism_downloads\교육부"),
    ];
    for base in roots {
        let Ok(entries) = std::fs::read_dir(base) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("1341000-201100013") && name.ends_with(".hwp") {
                return std::fs::read(entry.path()).ok();
            }
        }
    }
    None
}

/// `Column` 의 **직계** 표만 모은다 — 칸 안의 중첩 표는 세지 않는다.
fn column_tables(node: &RenderNode, in_column: bool, out: &mut Vec<(f64, f64, f64, f64)>) {
    if in_column {
        if let RenderNodeType::Table(_) = &node.node_type {
            out.push((
                node.bbox.x,
                node.bbox.x + node.bbox.width,
                node.bbox.y,
                node.bbox.y + node.bbox.height,
            ));
            return;
        }
    }
    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        column_tables(child, in_column, out);
    }
}

/// 괘선 두께·반올림을 넘는 실질 겹침만 센다.
const TOLERANCE_PX: f64 = 8.0;

#[test]
fn split_float_sibling_does_not_overlap_the_fragment() {
    let Some(bytes) = sample() else {
        return;
    };
    let document = HwpDocument::from_bytes(&bytes).expect("문서 로드");
    let page_count = document.page_count();
    assert!(
        page_count >= 40,
        "표본이 어긋났다 — 40쪽 이상이어야 한다. got {page_count}"
    );

    // 문제 문단(pi=113)은 31쪽 언저리다. 쪽수가 하나 늘어나므로 앞뒤로 넉넉히 본다.
    let last = page_count.saturating_sub(1).min(34);
    let mut worst: Option<(u32, f64, f64)> = None;
    for page in 28..=last {
        let Ok(tree) = document.build_page_render_tree(page) else {
            continue;
        };
        let mut tables = Vec::new();
        column_tables(&tree.root, false, &mut tables);
        for i in 0..tables.len() {
            for j in (i + 1)..tables.len() {
                let (ax0, ax1, ay0, ay1) = tables[i];
                let (bx0, bx1, by0, by1) = tables[j];
                let w = ax1.min(bx1) - ax0.max(bx0);
                let h = ay1.min(by1) - ay0.max(by0);
                if w > TOLERANCE_PX
                    && h > TOLERANCE_PX
                    && worst.is_none_or(|(_, pw, ph)| w * h > pw * ph)
                {
                    worst = Some((page, w, h));
                }
            }
        }
    }

    assert!(
        worst.is_none(),
        "같은 쪽의 자리차지 표 두 장이 겹쳤다 — #6795 회귀 {worst:?} \
         (회귀 시 31쪽에서 548.0 × 401.9px, 아래 표가 통째로 가려진다)"
    );
}

/// 겹침만 없애고 표를 뒤로 미루는 안(`>= 0` 완화)을 함께 막는다 — 문서 순서가
/// 한/글 오라클과 같아야 한다: 조각 → `위원회 심의결과`(ci=1) → `종합의견`(pi=114).
#[test]
fn split_float_sibling_keeps_document_order() {
    let Some(bytes) = sample() else {
        return;
    };
    let document = HwpDocument::from_bytes(&bytes).expect("문서 로드");

    // `ci=1` 표의 머리글은 이 문자열로만 나온다.
    const SIBLING_HEAD: &str = "위원회";
    // `pi=114` 표의 머리글.
    const NEXT_HEAD: &str = "종합의견";

    fn page_text(document: &HwpDocument, page: u32) -> String {
        let Ok(tree) = document.build_page_render_tree(page) else {
            return String::new();
        };
        let mut out = String::new();
        fn walk(node: &RenderNode, out: &mut String) {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                out.push_str(&run.text);
            }
            for child in &node.children {
                walk(child, out);
            }
        }
        walk(&tree.root, &mut out);
        out
    }

    let last = document.page_count().saturating_sub(1).min(36);
    let mut sibling_page = None;
    let mut next_page = None;
    for page in 28..=last {
        let text = page_text(&document, page);
        if sibling_page.is_none() && text.contains(SIBLING_HEAD) {
            sibling_page = Some(page);
        }
        if next_page.is_none() && text.contains(NEXT_HEAD) {
            next_page = Some(page);
        }
    }

    let (sibling_page, next_page) = match (sibling_page, next_page) {
        (Some(a), Some(b)) => (a, b),
        other => panic!("표본이 어긋났다 — 두 표를 못 찾았다 {other:?}"),
    };
    assert!(
        sibling_page < next_page,
        "`위원회 심의결과` 표({sibling_page}쪽)가 `종합의견`({next_page}쪽)보다 뒤에 있다 \
         — 쪽 순서 역전. 한/글 2018 오라클은 28쪽 → 29쪽 순서다."
    );
}
