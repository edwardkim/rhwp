//! [Issue #6837] 중첩 표의 **행이 원자 유닛**이라 행 안에서 쪽을 못 나눈다.
//!
//! `17544911`(양잠 전문인력 양성기관 지정기준) — 쪽수 3과 총 글자 수 1,252 는 engine
//! 2020 정본과 같은데 **쪽별 배분이 달랐다.**
//!
//! ```text
//!   쪽별 공백·점 제거 글자 수
//!     2020 정본  [618, 614, 20]
//!     rhwp(전)   [528, 636, 88]     ← 1쪽이 90자 적다
//! ```
//!
//! 바깥 표 셀 `r=2,c=0` 의 문단 `p[4]` 에 **6행×2열 중첩 표**가 있고, 그 행들이 그대로
//! `cell_units` 의 유닛이 된다.
//!
//! ```text
//!   u10 135.0  중첩 행3 "3) 뽕밭 관리"          누적  789.7  ✔ 행 예산 920.5 안
//!   u11 279.0  중첩 행4 "4) 양잠산물의 산업화"  누적 1068.7  ✗ 통째로 다음 쪽
//!
//!   DIAG_SPLITSCAN consumed=874.4 avail=1005.4   ← 130.8px 낭비
//! ```
//!
//! **한/글은 그 행 안에서 끊는다** — 정본 1쪽이 `나) 홍잠[…] 생산 기술 및 건강` 에서
//! 끝나고 2쪽이 `기능 효과 다) 수번데기…` 로 같은 행을 이어받는다.
//!
//! ⭐⭐ **근인은 `row_is_auto_height` 가 `cell.height == 0` 만 인정한 것이다.**
//! 이 중첩 표는 모든 행이 `h=282`(**3.8px**)로 저장돼 있는데 실제로는 135~308px 로
//! 그려진다. 저장값이 자기 내용의 **한 줄(16.0px)보다도 작으면** 그것은 실제 높이가
//! 아니라 **껍데기**다 — 그 행의 높이는 내용이 정한다.
//!
//! ⚠ **비율로 가르지 않는다.** `issue3637` 의 4×3 중첩 표는 선언 28.4px 에 실제
//! 34.4px(83%)로 살짝 넘칠 뿐이라 선언이 진짜 높이다. 한 줄(13.3px)보다 크므로
//! 껍데기 판정에서 갈린다. 비율(1% vs 83%)로 자르면 문턱이 생기지만, "한 줄보다
//! 작은가"는 문서가 스스로 주는 눈금이다.
//!
//! ⚠⚠ 넓게 켠 판(`선언 < 실제` 만)은 **게이트 4건**을 깼다 — `#5585`(7→10쪽),
//! `text_overlap` 파티션 7·8, `overflow_cell` 파티션 15. 껍데기 조건이 그것을 닫는다.
//!
//! ⭐ 이 브랜치는 **PR #6792(`#6790`) 위에 쌓았다.** 그 수정이 없으면 저장 프레임 꼬리
//! 확장이 먼저 1쪽을 254px 넘기게 만들어 이 축이 관측되지 않는다.
//!
//! 결과: 쪽별 글자 수가 **2020 정본과 완전히 같아진다** — `[618, 614, 20]`.
//!
//! ⭐ **글자겹침 2건도 같이 닫힌다**(`text_overlap` 래칫 2 → **0**). 같은 뿌리다 —
//! 행 4 가 원자라 2쪽 이어받는 조각의 높이가 실제 페인트와 어긋났고, 그 회계로
//! 놓인 행 5 가 행 4 꼬리 **위로** 9.1px 올라갔다.
//!
//! ```text
//!   2쪽, 수정 전        행4 꼬리 `바) 뽕잎…` y=376.8   행5 `가) 누에 사육기술…` y=367.7
//!                       → 6.90px 겹침 (169.0px · 224.0px 폭 두 쌍)
//!   2쪽, 수정 후        행4 꼬리 `사) 오디…` 끝 244.1  행5 `가) …` 시작 246.0
//!                       → 겹침 없음
//!   `tests/fixtures/text_overlap_baseline.tsv` 를 2 → 0 으로 낮춰 잠갔다.
//!   (래칫은 감소를 통과시키므로 낮추지 않으면 재발이 안 잡힌다.)
//! ```

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// `#6790` 과 같은 원본이다(이 브랜치가 그 위에 쌓여 있다). 중복 등재하지 않는다.
const SAMPLE: &str = "samples/issue6790/17544911-sericulture-training-criteria.hwp";

fn sample() -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
        .expect("#6837 회귀 sample 읽기 (PR #6792 의 samples/issue6790/)")
}

fn find_body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(find_body)
}

/// 이 쪽의 글자를 문서 순서대로 이어 붙인다(공백 제거).
fn page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let body = find_body(&tree.root).expect("Body 노드");
    let mut out = String::new();
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(tr) = &node.node_type {
            out.extend(tr.text.chars().filter(|c| !c.is_whitespace()));
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(body, &mut out);
    out
}

/// 1쪽이 중첩 표 **행 4 안까지** 담는다 — 행 경계에서 멈추지 않는다.
///
/// 수정 전: 1쪽이 행 3(`라) 뽕나무(오디) 농약 안전사용 방법`)에서 끝났다.
/// 정본·수정 후: 행 4 의 `가) 누에분말…` 과 `나) 홍잠[…]` 까지 담는다.
#[test]
fn the_first_page_splits_inside_a_nested_row() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 3, "쪽수 핀 — engine 2020 정본도 3쪽");

    let first = page_text(&core, 0);

    assert!(
        first.contains("양잠산물의산업화"),
        "1쪽이 중첩 행4 의 머리(`4) 양잠산물의 산업화`)까지 담아야 한다 — #6837 회귀 \
         (1쪽 {}자; 수정 전에는 행3 에서 끝났다)",
        first.chars().count()
    );
    assert!(
        first.contains("누에분말"),
        "1쪽이 행4 의 첫 항목(`가) 누에분말…`)까지 담아야 한다 — #6837 회귀"
    );
}

/// 그 행의 **뒷부분은 2쪽이 이어받는다** — 잃지도 겹치지도 않는다.
#[test]
fn the_second_page_resumes_inside_the_same_nested_row() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let first = page_text(&core, 0);
    let second = page_text(&core, 1);

    // 행4 의 뒷 항목은 2쪽에만 있다.
    assert!(
        second.contains("수번데기") && !first.contains("수번데기"),
        "행4 의 뒷 항목(`다) 수번데기…`)은 2쪽에만 있어야 한다 — #6837 회귀 \
         (1쪽 포함={}, 2쪽 포함={})",
        first.contains("수번데기"),
        second.contains("수번데기")
    );
    // 앞 항목은 1쪽에만 있다(중복 없음).
    assert!(
        !second.contains("누에분말"),
        "행4 의 앞 항목이 두 쪽에 중복되면 안 된다 — #6837 회귀"
    );
}

/// 쪽 배분이 engine 2020 정본 형상과 같다.
///
/// ```text
///   렌더 트리 글자 수(공백 제거)
///     수정 전  [533, 645,  89]     1쪽 < 2쪽,  3쪽이 1쪽의 1/6
///     수정 후  [623, 623,  21]     1쪽 ≥ 2쪽,  3쪽이 한 줄 규모
///     2020 정본(PDF 추출)  [618, 614, 20]
/// ```
///
/// 두 지표가 **모두 뒤집힌다** — 어느 쪽으로 어긋나도 실패한다.
/// (PDF 추출값과 렌더 트리 값은 몇 자 차이가 있어 절대값이 아니라 형상을 계약한다.)
#[test]
fn the_page_distribution_matches_the_reference_engine() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 3, "쪽수 핀 — engine 2020 정본도 3쪽");

    let counts: Vec<usize> = (0..3)
        .map(|p| page_text(&core, p).chars().count())
        .collect();

    assert!(
        counts[0] >= counts[1],
        "정본은 1쪽이 2쪽보다 적지 않다 — #6837 회귀 (실측 {counts:?}; 수정 전 [533, 645, 89])"
    );
    assert!(
        counts[2] * 10 < counts[0],
        "정본 3쪽은 한 줄 규모다 — #6837 회귀 (실측 {counts:?}; 수정 전 3쪽 89자)"
    );
}

/// 2쪽에서 중첩 **행4 의 꼬리와 행5 가 겹치지 않는다**.
///
/// 행 4 가 원자였을 때는 2쪽 이어받는 조각의 높이가 실제 페인트와 어긋나, 그 회계로
/// 놓인 행 5 가 행 4 꼬리 **위로** 9.1px 올라가 6.90px 겹쳤다(169.0 · 224.0px 폭 두 쌍).
#[test]
fn the_continued_nested_row_does_not_overlap_the_next_row() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(1).expect("2쪽 render tree");
    let body = find_body(&tree.root).expect("Body 노드");

    // (텍스트, 상단, 하단, 좌, 우)
    let mut runs: Vec<(String, f64, f64, f64, f64)> = Vec::new();
    fn walk(node: &RenderNode, out: &mut Vec<(String, f64, f64, f64, f64)>) {
        if let RenderNodeType::TextRun(tr) = &node.node_type {
            if !tr.text.trim().is_empty() {
                out.push((
                    tr.text.clone(),
                    node.bbox.y,
                    node.bbox.y + node.bbox.height,
                    node.bbox.x,
                    node.bbox.x + node.bbox.width,
                ));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(body, &mut runs);

    let mut worst = 0.0f64;
    let mut worst_pair = String::new();
    for i in 0..runs.len() {
        for j in (i + 1)..runs.len() {
            let (a, b) = (&runs[i], &runs[j]);
            let ox = a.4.min(b.4) - a.3.max(b.3);
            let oy = a.2.min(b.2) - a.1.max(b.1);
            if ox > 1.0 && oy > 1.0 && oy > worst {
                worst = oy;
                worst_pair = format!("{:?} x {:?}", &a.0, &b.0);
            }
        }
    }

    assert!(
        worst <= 0.5,
        "2쪽 글줄이 겹치면 안 된다 — #6837 회귀 (세로 겹침 {worst:.2}px, {worst_pair}; \
         수정 전 6.90px)"
    );
}
