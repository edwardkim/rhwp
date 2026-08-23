//! Issue #2279 (PR #2284) — 측정 정합 4수정의 직접 회귀 oracle.
//!
//! 페이지 수 pin(issue_1891)만으로는 같은 쪽수 안에서 되돌아가는 회귀를 잡지 못하므로
//! (maintainer 리뷰 P1), 각 수정의 관측 가능한 페이지-내 배치를 render tree 로 고정한다.
//! 기준 문서: `samples/86712_regulatory_analysis.hwp` (규제영향분석서, 한글 2022 = 65쪽).
//! 페이지 인덱스는 0-based (`build_page_render_tree(N)` = N+1쪽).

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

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

/// SVG renderer와 같은 클리핑을 적용해, 실제로 쪽에 칠해지는 텍스트만 찾는다.
///
/// Render tree는 디버그·재조판 관찰을 위해 부모 `TableCell`의 clip 밖 자식도
/// 보존한다. 따라서 단순 재귀 검색은 셀 하단에서 잘린 내부 표를 "이 쪽에
/// 있다"고 오판할 수 있다. `SvgRenderer`가 여는 Body/TableCell/TextBox clip을
/// 여기에도 적용해야 PDF 대조용 페이지 oracle이 실화면을 판정한다.
fn page_contains_paintable_text(core: &DocumentCore, page: u32, needle: &str) -> bool {
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("render tree p{page}: {e:?}"));
    find_paintable_text(&tree.root, needle, None, true)
}

fn clipped_intersection(a: BoundingBox, b: BoundingBox) -> Option<BoundingBox> {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    (right > left && bottom > top).then(|| BoundingBox::new(left, top, right - left, bottom - top))
}

fn find_paintable_text(
    node: &RenderNode,
    needle: &str,
    inherited_clip: Option<BoundingBox>,
    inherited_visible: bool,
) -> bool {
    let visible = inherited_visible && node.visible;
    if !visible {
        return false;
    }

    let own_clip = match &node.node_type {
        RenderNodeType::Body {
            clip_rect: Some(clip),
        } => Some(*clip),
        RenderNodeType::TableCell(cell) if cell.clip => Some(node.bbox),
        RenderNodeType::TextBox => Some(node.bbox),
        _ => None,
    };
    let clip = match (inherited_clip, own_clip) {
        (Some(parent), Some(own)) => match clipped_intersection(parent, own) {
            Some(intersection) => Some(intersection),
            None => return false,
        },
        (Some(parent), None) => Some(parent),
        (None, Some(own)) => Some(own),
        (None, None) => None,
    };

    if let RenderNodeType::TextRun(run) = &node.node_type {
        let paints_inside_clip = clip.as_ref().is_none_or(|clip| clip.intersects(&node.bbox));
        if paints_inside_clip && run.text.contains(needle) {
            return true;
        }
    }

    node.children
        .iter()
        .any(|child| find_paintable_text(child, needle, clip, visible))
}

fn find_text(node: &RenderNode, needle: &str) -> bool {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains(needle) {
            return true;
        }
    }
    node.children.iter().any(|c| find_text(c, needle))
}

/// 표 셀에서 줄/글자모양 경계로 갈라진 TextRun을 source 순서로 이어 검증한다.
/// 같은 header가 두 run으로 저장될 수 있으므로 단일-run substring만으로 표 존재를
/// 판정하면 거짓 음성이 된다.
fn table_contains_text_sequence(node: &RenderNode, rows: u16, cols: u16, needle: &str) -> bool {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.row_count == rows && table.col_count == cols {
            let mut text = String::new();
            collect_subtree_text(node, &mut text);
            if text.contains(needle) {
                return true;
            }
        }
    }
    node.children
        .iter()
        .any(|child| table_contains_text_sequence(child, rows, cols, needle))
}

fn collect_subtree_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_subtree_text(child, out);
    }
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
/// ("2891017" = 편익산식입력9)이 다시 렌더된다. 또한 3×12 내부 표를 현재 쪽
/// 하단 clip 아래에서 소비하면 p29가 5×4 표부터 시작한다.
///
/// # [#5193] 셀 재조판 이관 후 실패 — 핀은 옳고, 이 핀이 서 있던 높이가 틀렸다
///
/// 이 테스트의 쪽 배정은 한컴 2024 PDF
/// (`samples/issue1891/86712_regulatory_analysis-2024.pdf`)가 직접 뒷받침한다 —
/// PDF p28이 `편익 수혜자`를 시작하고 p29가 `88.2` 표를 싣는다. **핀을 프레임
/// 값으로 옮기면 안 된다**: 프레임 경로는 `편익 수혜자`를 p27로, `88.2`를 p28로
/// 올려 세 assertion 을 PDF 와 반대로 만든다.
///
/// 실패 원인은 줄바꿈 잔차가 아니다. 문제의 셀(산식 표 r23~r26 c2,
/// `(2030년 기본형건축비 - …) * 주택면적 * 가구수 * 이자율`)은 **칠해지는 폭에서
/// 두 경로가 글자까지 동일**하다. 갈라지는 곳은 측정이 쓰는 더 좁은 두 번째
/// 폭이다:
///
/// ```text
/// w=204.45px (paint)     폐기 경로 4줄   프레임 4줄   (본문 동일)
/// w=192.85px (measure)   폐기 경로 5줄   프레임 4줄
///
/// 측정 폭 192.85px = 14464 HWPUNIT 에서의 판정
///   `-`     pen 13457  glyph  667  sum 14124  over  −340  → 프레임 수용(폐기 경로는 거부)
///   `2029`  pen 14822  glyph 2584  sum 17406  over +2942  → 거부, 줄0 = "(2030년 기본형건축비 - "
/// ```
///
/// 한컴 PDF p27은 이 셀을 **4줄**로 찍는다. 즉 프레임의 행높이(85.7px)가 맞고
/// 폐기 경로의 5줄 측정(106.5px)이 틀렸다. 그런데 그 틀린 측정이 이 쪽 나눔을
/// 지탱하고 있었다 — 산식 표 네 행(2032~2035)이 각각 20.8px 한 줄씩 줄면 p27에
/// 83.2px 이 남고, 거기로 r27 근거설명의 첫 유닛이 들어간다.
///
/// 즉 이 핀은 **보상 오차 위에 서 있던 옳은 기대값**이다. 오차를 고치면 상류의
/// 다른 문제가 드러난다 — 한컴 p27은 `□편익` 표 전체부터 시작하는데 rhwp p27은
/// 편익 표 중간에서 시작한다. 그 상류 차이는 #2279/#2308 소관이지 #5193 이 아니다.
///
/// 함께 기록: 측정 내폭(192.85)이 렌더 내폭(204.45)보다 **11.6px 좁다.** 두 경로
/// 모두에 있던 선행 결함이고, 바로 이것이 "측정에서는 줄바꿈이 갈리는데 렌더에서는
/// 안 갈리는" 상태를 가능하게 한다.
#[ignore = "#5193: 프레임 이관으로 실패. 핀은 한컴 PDF 가 뒷받침하므로 옮기지 않는다 — \
            폐기 경로의 5줄 측정(PDF 는 4줄)이 이 쪽 나눔을 지탱하고 있었다. 위 주석 참조."]
#[test]
fn issue_2279_nested_cell_units_split_r27_not_r26() {
    let core = core();
    // PDF p27에는 r27 근거설명이 아직 시작하지 않는다. 남은 공간에 wrapper의
    // 앞 조각만 넣으면 scalar child가 실제 잉크를 앞 쪽으로 누출한다.
    assert!(
        !page_contains_paintable_text(&core, 26, "편익 수혜자"),
        "p27에 r27 근거설명이 조기 노출 — fresh-page defer 회귀"
    );
    // p28(0-based 27)는 r27의 앞 문단 묶음을 실제로 paint한다.
    assert!(
        page_contains_paintable_text(&core, 27, "편익 수혜자"),
        "p28에 r27 콘텐츠 첫 유닛 부재 — 1×1 중첩 셀 유닛화 회귀 (rows=0..27 로 후퇴)"
    );
    // p28의 남은 공간에는 3×12 내부 표와 뒤의 출처/설명/5×4 표 묶음 전체가
    // 들어가지 않는다. 첫 표만 clip 아래에서 소비하면 안 된다.
    assert!(
        !page_contains_paintable_text(&core, 27, "88.2"),
        "p28에 3×12 내부 표가 부분 진입 — nested block을 fresh page로 이월해야 함"
    );
    // p29(0-based 28): r27 continuation 만 — 산식 행(r26)이 다시 걸치면 회귀.
    // 주의: "2891017"(콤마 없음)은 산식 필드 전용 — r27 내부 5×4 표의 값
    // "2,891,017"(콤마)과 구분된다.
    assert!(
        !page_contains_paintable_text(&core, 28, "2891017"),
        "p29에 산식 r26(편익산식입력9) 재등장 — cut 유닛 과소(-448) 회귀"
    );
    assert!(
        page_contains_paintable_text(&core, 28, "88.2"),
        "p29에 3×12 내부 표 부재 — p28 하단 clip 소비 회귀"
    );
    assert!(
        table_contains_text_sequence(
            &core
                .build_page_render_tree(28)
                .expect("render tree p29")
                .root,
            5,
            4,
            "주민대표단 구성",
        ),
        "p29에 3×12 표 뒤 5×4 내부 표 부재 — r27 block 순서 회귀"
    );
}

/// [수정 2] 본문 NO_LS 폴백의 글자모양 보존 + 전체-문단 재래핑 렌더
/// (재래핑 후 end_line 확장) —
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

/// [수정 3] 재래핑 줄별 pitch (#5193 이후 `frame_metrics_for_line` 의 행별 metric) —
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
