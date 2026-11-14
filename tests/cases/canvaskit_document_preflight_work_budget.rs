//! 관계형: 문서 사전 검사의 작업 예산 소비·중단과 후속 쪽의 소유를 검사한다.
//! 고정형: 공개 작업 상한 100,000은 자원 정책이며 실물 배치의 px 고정값이 아니다.
//! 주석 갱신: 2026-11-14T17:10:05+09:00.
//! 실물 일치율로 만든 배치 검사가 아닌 합성 작업 예산 계약이다. OS 글꼴에 의존하지 않는다.

use rhwp::paint::{LayerNode, PageLayerTree, RenderProfile};
use rhwp::renderer::canvaskit_policy::{
    analyze_canvaskit_document_preflight, CanvasKitDocumentPreflightBlockerCode,
    CanvasKitPreflightPageBuild, CanvasKitReplayMode,
};
use rhwp::renderer::render_tree::BoundingBox;

fn empty_page() -> Box<PageLayerTree> {
    Box::new(PageLayerTree::new(
        10.0,
        10.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, vec![]),
    ))
}

#[test]
fn a_page_above_the_previous_budget_completes_within_the_new_bound() {
    let report = analyze_canvaskit_document_preflight(
        1,
        CanvasKitReplayMode::Default,
        RenderProfile::Print,
        |_, remaining| {
            Ok::<_, &'static str>(if remaining >= 75_000 {
                CanvasKitPreflightPageBuild::Complete {
                    tree: empty_page(),
                    prelower_work_units: 75_000,
                }
            } else {
                CanvasKitPreflightPageBuild::WorkLimitExceeded
            })
        },
    );
    assert_eq!(report.limits.max_work_units, 100_000);
    assert!(report.complete && report.eligible);
    assert_eq!(report.scanned_pages, 1);
    assert_eq!(report.scanned_work_units, 75_000);
    assert!(report.blockers.is_empty());
}

#[test]
fn the_next_page_receives_only_the_remaining_document_budget() {
    let mut supplied_budgets = Vec::new();
    let report = analyze_canvaskit_document_preflight(
        3,
        CanvasKitReplayMode::Default,
        RenderProfile::Print,
        |page, remaining| {
            supplied_budgets.push((page, remaining));
            let cost = if page == 0 { 75_000 } else { 25_001 };
            Ok::<_, &'static str>(if cost <= remaining {
                CanvasKitPreflightPageBuild::Complete {
                    tree: empty_page(),
                    prelower_work_units: cost,
                }
            } else {
                CanvasKitPreflightPageBuild::WorkLimitExceeded
            })
        },
    );
    assert_eq!(supplied_budgets, [(0, 100_000), (1, 25_000)]);
    assert!(!report.complete && !report.eligible);
    assert_eq!(report.scanned_pages, 1);
    assert_eq!(report.scanned_work_units, 100_000);
    assert_eq!(report.blockers.len(), 1);
    assert_eq!(report.blockers[0].page_index, 1);
    assert_eq!(
        report.blockers[0].code,
        CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded
    );
}
