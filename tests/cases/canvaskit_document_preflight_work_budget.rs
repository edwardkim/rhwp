//! 관계형: 문서 사전 검사의 작업 예산 소비·중단과 후속 쪽의 소유를 검사한다.
//! 공개 작업 상한 200,000은 자원 정책값이며 실물 배치의 px 고정 검사가 아니다.
//! 주석 갱신: 2026-11-15T20:55:03+09:00.
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
    for cost in [75_000, 150_000, 200_000] {
        let report = analyze_canvaskit_document_preflight(
            1,
            CanvasKitReplayMode::Default,
            RenderProfile::Print,
            |_, remaining| {
                Ok::<_, &'static str>(if remaining >= cost {
                    CanvasKitPreflightPageBuild::Complete {
                        tree: empty_page(),
                        prelower_work_units: cost,
                    }
                } else {
                    CanvasKitPreflightPageBuild::WorkLimitExceeded
                })
            },
        );
        assert_eq!(report.limits.max_work_units, 200_000);
        assert!(report.complete && report.eligible);
        assert_eq!(report.scanned_pages, 1);
        assert_eq!(report.scanned_work_units, cost);
        assert!(report.blockers.is_empty());
    }
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
            let cost = if page == 0 { 150_000 } else { 50_001 };
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
    assert_eq!(supplied_budgets, [(0, 200_000), (1, 50_000)]);
    assert!(!report.complete && !report.eligible);
    assert_eq!(report.scanned_pages, 1);
    assert_eq!(report.scanned_work_units, 200_000);
    assert_eq!(report.blockers.len(), 1);
    assert_eq!(report.blockers[0].page_index, 1);
    assert_eq!(
        report.blockers[0].code,
        CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded
    );
}

#[test]
fn a_page_above_the_new_bound_stops_before_later_pages_are_built() {
    let mut requested_pages = Vec::new();
    let report = analyze_canvaskit_document_preflight(
        2,
        CanvasKitReplayMode::Default,
        RenderProfile::Print,
        |page, remaining| {
            requested_pages.push(page);
            assert_eq!(remaining, 200_000);
            Ok::<_, &'static str>(if 200_001 <= remaining {
                CanvasKitPreflightPageBuild::Complete {
                    tree: empty_page(),
                    prelower_work_units: 200_001,
                }
            } else {
                CanvasKitPreflightPageBuild::WorkLimitExceeded
            })
        },
    );
    assert_eq!(requested_pages, [0]);
    assert!(!report.complete && !report.eligible);
    assert_eq!(report.scanned_pages, 0);
    assert_eq!(report.scanned_work_units, 200_000);
    assert_eq!(report.blockers.len(), 1);
    assert_eq!(report.blockers[0].page_index, 0);
    assert_eq!(
        report.blockers[0].code,
        CanvasKitDocumentPreflightBlockerCode::WorkLimitExceeded
    );
}
