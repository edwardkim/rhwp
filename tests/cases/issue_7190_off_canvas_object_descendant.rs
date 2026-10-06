//! [#7190 잔여 축] off-canvas 가 컨테이너 자손의 **그리는 개체**를 안 봐서 용지 밖 그림·수식이
//! 어느 지표에도 안 잡힌다.
//!
//! ## 무엇이 문제였나
//!
//! `walk` 는 검사 대상 노드에서 off-canvas 를 한 번 재고 `next_off_canvas_suppress = true` 로
//! **자손 검사를 접는다**(컨테이너당 1회 보고). 그런데 그 한 번의 판정에 쓰는
//! `deep_vertical_union_bbox` 는 가로를 [#7051] 이후 **자손 `TextRun` 의 잉크 상자**로만
//! 합쳤다. 그래서 조상(`TextLine`·`Table`)이 먼저 검사되면 그 안의 그림·수식·도형·누름틀은
//! 자기 검사를 영영 못 받고, 가로 합집합에도 안 들어가 **쪽 밖 이탈이 전부 침묵**했다.
//!
//! `#7190` 본문이 신고한 3011411 1쪽이 그 모양이었다 — 글줄 참여 그림이 첫째 줄에 붙어
//! 우변 834.7px(용지 793.7 밖 41.0px)까지 나갔는데 `offCanvasCount = 0` 이었다.
//! 그 배치 결함 자체는 [#7220](https://github.com/edwardkim/rhwp/pull/7220) 으로 고쳐졌고,
//! 이슈에 남은 범위가 이 **검출 축**이다.
//!
//! ## 기대값의 출처
//!
//! 이 시험은 검출기가 계산한 값을 되풀이하지 않는다. **렌더 트리 좌표에서 직접**
//! `쪽 우변`과 `가시 개체 자손의 최대 우변`을 재서 기대 초과량을 만들고, 검출기가 그 값을
//! 보고하는지 본다(`expected_over_right`). 곧 판정식이 아니라 기하가 정답지다.
//!
//! ## 비범위
//!
//! 개체가 **왜** 그 자리에 놓였는가는 이 변경이 다루지 않는다(진단 전용 — `src/diagnostics/`
//! 밖을 건드리지 않는다). 그러니 새로 잡히는 것은 전부 이전부터 있었는데 안 보이던 결함이다.
//!
//! ## `samples` 전수 델타 (`off_canvas_baseline.tsv` 갱신 근거)
//!
//! 같은 워크트리에서 소스만 되돌렸다 되살려 쟀다(release-test, 문서 684종).
//! 총 331 → 349건, **새 신고 18건 · 사라진 신고 0건**이고 전부 렌더 트리에서
//! 쪽 밖 좌표를 직접 확인했다.
//!
//! ```text
//!   basic/request.hwp                         0 → 1   Form   우측 +97.1px  (1쪽)
//!   basic/issue1994_behindtext_table_2020…hwp 0 → 1   Image  우측 +124.0px (2쪽)
//!   issue2217/20200830.hwp                    0 → 1   Image  우측 +124.0px (2쪽)
//!   issue6202/156483689-turmeric-…hwp         0 → 1   Image  우측 +377.8px (1쪽)
//!   issue6551/113424_evaluation_guideline     0 → 1   Group  우측 +217.9px (35쪽)
//!   issue6787/16774617-electronic-ballot-form 0 → 2   Image  좌 −9.6 · 우 +178.7px
//!   task1725/text_footnote_tail_overpagination 15 → 17 · hwpx 16 → 18  Equation 우측 +604.1px
//!   3-11월_실전_통합 계열 7종                 각 +1   Equation 우측 +32.4px
//! ```
//!
//! 3-11월 계열 6종은 이 워크트리의 기존 건수가 committed baseline 보다 낮다(환경에 따라
//! 조판이 갈리는 문서 — `off_canvas_baseline.rs` 의 #6325 주석). 그 행은 실측 delta(+1)를
//! **committed 값에 더해** 적었고, 나머지 9행은 실측값 그대로다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions, OffCanvasAnomaly};
use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn core(rel: &str) -> DocumentCore {
    let bytes = std::fs::read(sample(rel)).expect("정식 원본");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 그리는 개체 노드인가(글자·컨테이너 제외).
fn is_object(node: &RenderNode) -> bool {
    matches!(
        node.node_type,
        RenderNodeType::Table(_)
            | RenderNodeType::Image(_)
            | RenderNodeType::TextBox
            | RenderNodeType::Equation(_)
            | RenderNodeType::Group(_)
            | RenderNodeType::FormObject(_)
            | RenderNodeType::Placeholder(_)
            | RenderNodeType::RawSvg(_)
            | RenderNodeType::Line(_)
            | RenderNodeType::Rectangle(_)
            | RenderNodeType::Ellipse(_)
            | RenderNodeType::Path(_)
    )
}

/// 렌더 트리에서 직접 재는 "가시 개체 자손의 최대 우변".
fn max_object_right(node: &RenderNode) -> f64 {
    if !node.visible || node.editor_only {
        return f64::MIN;
    }
    let own = if is_object(node) && node.bbox.width > 0.0 && node.bbox.height > 0.0 {
        node.bbox.x + node.bbox.width
    } else {
        f64::MIN
    };
    node.children
        .iter()
        .map(max_object_right)
        .fold(own, f64::max)
}

fn page_right(root: &RenderNode) -> f64 {
    root.bbox.x + root.bbox.width
}

fn off_canvas_of(rel: &str, page: u32) -> (Vec<OffCanvasAnomaly>, f64) {
    let core = core(rel);
    let tree = core
        .build_page_render_tree(page)
        .expect("쪽 render tree 생성");
    let expected_over_right = max_object_right(&tree.root) - page_right(&tree.root);
    let anomalies = scan_page(
        page,
        &tree.root,
        core.page_count(),
        &AnomalyOptions::default(),
    );
    (anomalies.off_canvas, expected_over_right)
}

/// 표 칸 안의 누름틀이 쪽 우변 밖에 놓인다 — `basic/request.hwp` 1쪽.
///
/// 렌더 트리 실측: 칸(`Cell`, 우변 510.0)의 줄은 501.4 에서 끝나는데 `Form` 은
/// `x=614.0 w=50.0` 에 놓인다. 쪽 우변은 566.9 이므로 **97.1px 이 종이 밖**이다.
/// 수정 전에는 이 문서의 off-canvas 가 0건이었다.
#[test]
fn off_canvas_sees_form_outside_the_page_inside_a_table_cell() {
    let (found, expected_over_right) = off_canvas_of("samples/basic/request.hwp", 0);
    assert!(
        expected_over_right > 90.0,
        "픽스처 전제가 깨졌다 — 쪽 밖 개체가 없다 (초과 {expected_over_right:.1}px)"
    );

    let right: Vec<&OffCanvasAnomaly> = found.iter().filter(|a| a.over_right > 1.0).collect();
    assert!(
        !right.is_empty(),
        "칸 안 개체의 가로 이탈이 하나도 안 잡혔다 — 수정 전 상태다. off-canvas {}건",
        found.len()
    );
    let worst = right.iter().map(|a| a.over_right).fold(0.0_f64, f64::max);
    assert!(
        (worst - expected_over_right).abs() < 0.05,
        "렌더 트리 기하가 말하는 초과({expected_over_right:.3}px)를 그대로 보고해야 한다. got {worst:.3}px"
    );
}

/// 표 칸 안의 그림이 쪽 우변 밖 377.8px 까지 나간다 —
/// `issue6202/156483689-turmeric-industry-standardization.hwp` 1쪽.
///
/// 렌더 트리 실측: `Table1/Cell2/Image1` 이 `x=1054.3 w=117.2`(우변 1171.5), 쪽 우변 793.7.
/// 앞 시험(누름틀)과 다른 개체 종류·다른 깊이에서 같은 침묵이 있었음을 고정한다.
#[test]
fn off_canvas_sees_image_outside_the_page_inside_a_table_cell() {
    let (found, expected_over_right) = off_canvas_of(
        "samples/issue6202/156483689-turmeric-industry-standardization.hwp",
        0,
    );
    assert!(
        expected_over_right > 370.0,
        "픽스처 전제가 깨졌다 — 쪽 밖 그림이 없다 (초과 {expected_over_right:.1}px)"
    );

    let worst = found.iter().map(|a| a.over_right).fold(0.0_f64, f64::max);
    assert!(
        (worst - expected_over_right).abs() < 0.05,
        "렌더 트리 기하가 말하는 초과({expected_over_right:.3}px)를 그대로 보고해야 한다. got {worst:.3}px"
    );
}

/// 대조군 — 쪽 안에 얌전히 들어간 문서는 이 축이 켜져도 조용하다.
///
/// `#7190` 의 원 신고 문서(3011411)는 [#7220] 이 그림을 저장된 둘째 줄로 되돌린 뒤
/// 쪽 안에 있다. 컨테이너 자손을 합치기 시작했다고 정상 배치가 신고되면 안 된다.
#[test]
fn fixed_placement_stays_silent() {
    let rel = "samples/issue7190/3011411_tac_picture_second_line.hwpx";
    let core = core(rel);
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render tree");
        let anomalies = scan_page(
            page,
            &tree.root,
            core.page_count(),
            &AnomalyOptions::default(),
        );
        assert!(
            anomalies.off_canvas.is_empty(),
            "{rel} {page}쪽에서 off-canvas 가 새로 잡혔다: {:?}",
            anomalies
                .off_canvas
                .iter()
                .map(|a| (a.path.clone(), a.node_type, a.max_over()))
                .collect::<Vec<_>>()
        );
    }
}
