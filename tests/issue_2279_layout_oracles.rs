//! Issue #2279 (PR #2284) — 측정 정합 4수정의 직접 회귀 oracle.
//!
//! 페이지 수 pin(issue_1891)만으로는 같은 쪽수 안에서 되돌아가는 회귀를 잡지 못하므로
//! (maintainer 리뷰 P1), 각 수정의 관측 가능한 페이지-내 배치를 render tree 로 고정한다.
//! 기준 문서: `samples/86712_regulatory_analysis.hwp` (규제영향분석서, 한글 2022 = 65쪽).
//! 페이지 인덱스는 0-based (`build_page_render_tree(N)` = N+1쪽).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/86712_regulatory_analysis.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse 86712_regulatory_analysis.hwp")
}

fn page_contains(core: &DocumentCore, page: u32, needle: &str) -> bool {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render tree p{page}: {e:?}"));
    find_text(&tree.root, needle)
}

fn find_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| find_text(c, needle))
}

fn collect_text_ys(node: &RenderNode, x_min: f64, y_range: (f64, f64), out: &mut Vec<f64>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty()
            && node.bbox.x >= x_min
            && node.bbox.y >= y_range.0
            && node.bbox.y <= y_range.1
        {
            out.push(node.bbox.y);
        }
    }
    for c in &node.children {
        collect_text_ys(c, x_min, y_range, out);
    }
}

/// [수정 1] 1×1 래퍼 중첩 셀 유닛화 (`nested_table_mixed_fragment_heights`) —
/// r27 근거설명(25문단 + 3×12/5×4 내부표)의 프래그먼트가 2단계 중첩 표·빈 문단
/// 줄박스·셀 말미 줄간격을 포함해야 한다 (-448px 과소 회귀 검출).
///
/// 회귀 시그니처(수정 전): pi=172 분할이 rows=26..28 로 물러나 p29 에 산식 r26
/// ("2891017" = 편익산식입력9)이 다시 렌더되고, r27 콘텐츠 소비 유닛이 준다.
#[test]
fn issue_2279_nested_cell_units_split_r27_not_r26() {
    let core = core();
    // p28(0-based 27): r27 콘텐츠 첫 유닛들 진입 (rows=0..28, end_cut=[1,6]).
    assert!(
        page_contains(&core, 27, "편익 수혜자"),
        "p28에 r27 콘텐츠 첫 유닛 부재 — 1×1 중첩 셀 유닛화 회귀 (rows=0..27 로 후퇴)"
    );
    // p29(0-based 28): r27 continuation 만 — 산식 행(r26)이 다시 걸치면 회귀.
    // 주의: "2891017"(콤마 없음)은 산식 필드 전용 — r27 내부 5×4 표의 값
    // "2,891,017"(콤마)과 구분된다.
    assert!(
        !page_contains(&core, 28, "2891017"),
        "p29에 산식 r26(편익산식입력9) 재등장 — cut 유닛 과소(-448) 회귀"
    );
    assert!(
        page_contains(&core, 28, "편익 수혜자"),
        "p29에 r27 continuation 부재 — 분할 구조 변화"
    );
}

/// [수정 2] 본문 NO_LS 폴백의 글자모양 보존 + 전체-문단 재래핑 렌더
/// (`recompose_for_body_width` + 재래핑 후 end_line 확장) —
/// 혼합 크기 문단(pi22: "ㅇ "=15pt + 본문 14pt)의 마지막 줄이 렌더에서 소실되지
/// 않아야 한다 (측정 4줄 fit vs 렌더 3줄 발산 회귀 검출).
#[test]
fn issue_2279_body_rewrap_keeps_paragraph_tail() {
    let core = core();
    assert!(
        page_contains(&core, 9, "규정하려는 것임"),
        "p10에 pi22 마지막 줄 부재 — 재래핑 줄수/end_line 클램프 회귀 (렌더 꼬리 소실)"
    );
}

/// [수정 3] 재래핑 줄별 pitch (`recompose_for_cell_width` per-line lh/ls) —
/// p10 본문(내어쓰기 ㅇ-불릿 문단들, 한글 실측 pitch 29.9px)의 인접 줄 간격
/// 중앙값이 30px 근처여야 한다. 회귀(문단 최대 fs 상속) 시 32.0 으로 복귀.
#[test]
fn issue_2279_per_line_pitch_uses_line_max_font_size() {
    let core = core();
    let tree = core.build_page_render_tree(9).expect("render tree p10");
    // 본문 불릿 문단 영역 (표 제외: y 140~660, 들여쓰기 본문 x>=100)
    let mut ys = Vec::new();
    collect_text_ys(&tree.root, 100.0, (140.0, 660.0), &mut ys);
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 3.0);
    let mut gaps: Vec<f64> = ys.windows(2).map(|w| w[1] - w[0]).collect();
    // 문단 사이 간격(빈 줄 포함, > 34px)은 제외 — 줄 pitch 만.
    gaps.retain(|g| *g > 20.0 && *g < 34.0);
    assert!(
        gaps.len() >= 8,
        "pitch 표본 부족 ({}개) — 페이지 구성 변화 시 창 조정 필요: {ys:?}",
        gaps.len()
    );
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = gaps[gaps.len() / 2];
    assert!(
        (28.5..31.5).contains(&median),
        "본문 줄 pitch 중앙값 {median:.2}px — 줄별 pitch(≈29.9, 한글 실측) 회귀 (32.0 = 문단 최대 fs 상속)"
    );
}

/// [수정 4] RowBreak float 선언-이월의 문단 단위 증거 판정 (`saved_span`) —
/// pi30 표(4×3 RowBreak, host 저장 LS 없음, 측정 비적합)는 한글처럼 행 분할되어
/// 머리 행(r0~r1: 대안명/규제대안1)이 p10 에 남아야 한다. 회귀(구역 전역
/// has_stored_line_segs 판정) 시 표 전체가 p11 로 이월된다.
#[test]
fn issue_2279_rowbreak_float_splits_without_host_line_segs() {
    let core = core();
    assert!(
        page_contains(&core, 9, "대안명"),
        "p10에 pi30 표 머리 행 부재 — saved_span 판정 회귀 (통째 이월)"
    );
    assert!(
        page_contains(&core, 9, "주민대표단의 법적"),
        "p10에 규제대안1 내용 부재 — RowBreak float 분할 회귀"
    );
    assert!(
        page_contains(&core, 10, "준 준용"),
        "p11에 잔여 행(규제대안2: 기존 기준 준용) 부재 — 분할 구조 변화"
    );
}
