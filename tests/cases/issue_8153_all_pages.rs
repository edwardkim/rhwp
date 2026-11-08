//! 관계형·고정형(독립 Print 원점·그림 경계 및 쪽수): PR #8409의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T20:25:43+09:00 (`1c55958acf99196c1cdecc324b89382083fc0563`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/2025 행정업무운영 편람(최종).hwp`; 독립 Print `pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`.
//! 전체 383쪽, 전쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 입력 `samples/2025 행정업무운영 편람(최종).hwpx`; 독립 Print `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`.
//! 전체 383쪽, 전쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8409_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03T20:20:24+09:00; 직전 파일 커밋: 2026-11-03T19:46:04+09:00 (8e4f05c63a).
//! 원문 기준점은 run 경계와 무관하게 같은 기준선에서 맞닿은 연속 run 의 글로 대조한다.
//! 2026-10-30 측정 raw(96dpi, PDFium, 2px): HWP·HWPX 1–129·131–383쪽 Native·fresh WASM 각 100.00%,
//! 130쪽 각 99.96%; 두 원본 전 383쪽 최저 Native·fresh WASM 각 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/rebase-ad4-validation-20261030/README.md.
//! 독립 PDF의 원점과 표지 그림 경계는 130쪽 ±2px, 그 밖의 쪽은 ±1px다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TextRunNode};
use rhwp::DocumentCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Deserialize)]
struct Fixture {
    input: String,
    input_sha256: String,
    pdf: String,
    pdf_sha256: String,
    pages: Vec<Page>,
}

#[derive(Deserialize)]
struct Page {
    page: usize,
    #[serde(flatten)]
    anchor: Anchor,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Anchor {
    Text {
        text: String,
        x: f64,
        baseline: f64,
    },
    Image {
        x: f64,
        y: f64,
        right: f64,
        bottom: f64,
    },
    Blank,
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

/// 원문 기준점과 같은 글을 이루는 연속 TextRun 의 첫 run.
///
/// 같은 기준선에서 앞 run 의 끝과 뒤 run 의 시작이 맞닿으면 한 글줄 조각이다. run 경계는
/// 글자 슬롯 분류처럼 출력과 무관한 구현 세부로 바뀔 수 있으므로(`※ 표의…` 가 `※ ` 와
/// `표의…` 로 나뉘어도 위치는 같다) 조각 안의 연속 run 을 이어 원문과 대조한다.
/// 원문 뒤에 구두점이 같은 줄의 다음 run 으로 이어져도 원문 시작점은 변하지 않는다.
/// 독립 Print 원점·기준선에 있는 조각만 후보로 받아, 앞부분이 같은 다른 줄(241쪽)을
/// 중복으로 세지 않는다. 공백뿐인 run 은 시작·끝으로 세지 않는다.
fn text_anchor_runs<'a>(
    nodes: &[&'a RenderNode],
    text: &str,
    expected_x: f64,
    expected_baseline: f64,
    tolerance: f64,
) -> Vec<(&'a RenderNode, &'a TextRunNode)> {
    let mut segments: Vec<Vec<(&RenderNode, &TextRunNode)>> = Vec::new();
    for node in nodes {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            continue;
        };
        let joined = segments
            .last()
            .and_then(|segment| segment.last())
            .is_some_and(|(prev, prev_run)| {
                (prev.bbox.y + prev_run.baseline - (node.bbox.y + run.baseline)).abs() < 0.01
                    && (prev.bbox.x + prev.bbox.width - node.bbox.x).abs() < 0.05
            });
        if joined {
            segments.last_mut().unwrap().push((*node, run));
        } else {
            segments.push(vec![(*node, run)]);
        }
    }
    let mut found = Vec::new();
    for segment in &segments {
        for start in 0..segment.len() {
            if segment[start].1.text.trim().is_empty() {
                continue;
            }
            let (first_node, first_run) = segment[start];
            if (first_node.bbox.x - expected_x).abs() > tolerance
                || (first_node.bbox.y + first_run.baseline - expected_baseline).abs() > tolerance
            {
                continue;
            }
            let mut joined = String::new();
            for (_, run) in segment.iter().skip(start) {
                joined.push_str(&run.text);
                if !run.text.trim().is_empty() && joined.trim() == text {
                    found.push(segment[start]);
                    break;
                }
            }
        }
    }
    found
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn verify(data: &str) {
    let fixture: Fixture = serde_json::from_str(data).expect("독립 Print 전쪽 기준");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = std::fs::read(root.join(&fixture.input)).expect("원본 편람");
    let pdf = std::fs::read(root.join(&fixture.pdf)).expect("독립 한컴 Print PDF");
    assert_eq!(sha256(&input), fixture.input_sha256);
    assert_eq!(sha256(&pdf), fixture.pdf_sha256);
    let mut core = DocumentCore::from_bytes(&input).expect("편람 해석");
    core.set_hangul2024_compat(true);
    assert_eq!(fixture.pages.len(), 383);
    assert_eq!(core.page_count(), 383, "독립 Print 전체 쪽수");
    let mut errors = Vec::new();
    for (index, page) in fixture.pages.iter().enumerate() {
        assert_eq!(page.page, index + 1, "쪽 순서·누락·중복");
        let tree = core
            .build_page_render_tree(index as u32)
            .expect("전쪽 출력");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        let tolerance = if page.page == 130 { 2.0 } else { 1.0 };
        match &page.anchor {
            Anchor::Text { text, x, baseline } => {
                let runs = text_anchor_runs(&nodes, text, *x, *baseline, tolerance);
                if runs.len() != 1 {
                    errors.push(format!(
                        "{}쪽 원문 누락·중복({}): {text}",
                        page.page,
                        runs.len()
                    ));
                    continue;
                }
                let (node, run) = runs[0];
                if (node.bbox.x - *x).abs() > tolerance
                    || (node.bbox.y + run.baseline - *baseline).abs() > tolerance
                {
                    errors.push(format!("{}쪽 {text} 원점 {:.2}/{x:.2}, 기준선 {:.2}/{baseline:.2}, 허용 ±{tolerance}px", page.page, node.bbox.x, node.bbox.y + run.baseline));
                }
            }
            Anchor::Image {
                x,
                y,
                right,
                bottom,
            } => {
                let images: Vec<_> = nodes
                    .iter()
                    .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
                    .collect();
                if images.len() != 1 {
                    errors.push(format!(
                        "{}쪽 표지 그림 누락·중복({})",
                        page.page,
                        images.len()
                    ));
                    continue;
                }
                let b = images[0].bbox;
                for (actual, expected) in [
                    (b.x, *x),
                    (b.y, *y),
                    (b.x + b.width, *right),
                    (b.y + b.height, *bottom),
                ] {
                    if (actual - expected).abs() > tolerance {
                        errors.push(format!(
                            "{}쪽 그림 경계 {actual:.2}/{expected:.2}",
                            page.page
                        ));
                    }
                }
            }
            Anchor::Blank => {
                let blank = nodes.iter().all(|n| match &n.node_type {
                    RenderNodeType::TextRun(run) => run.text.trim().is_empty(),
                    RenderNodeType::Image(_)
                    | RenderNodeType::Line(_)
                    | RenderNodeType::Rectangle(_)
                    | RenderNodeType::Ellipse(_)
                    | RenderNodeType::Path(_)
                    | RenderNodeType::Equation(_) => false,
                    _ => true,
                });
                if !blank {
                    errors.push(format!("{}쪽 빈 쪽에 내용이 생겼다", page.page));
                }
            }
        }
    }
    assert!(
        errors.is_empty(),
        "전쪽 독립 Print 경계 실패:\n{}",
        errors.join("\n")
    );
}

#[test]
fn hwp_all_pages_keep_independent_print_anchors() {
    verify(include_str!("../fixtures/issue8153/all-pages-hwp.json"));
}

#[test]
fn hwpx_all_pages_keep_independent_print_anchors() {
    verify(include_str!("../fixtures/issue8153/all-pages-hwpx.json"));
}
