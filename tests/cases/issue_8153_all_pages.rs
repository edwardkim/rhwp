//! 관계형·고정형: 두 편람 전쪽의 내용과 독립 Print 원점·그림 경계를 검사한다.
//! 주석 갱신: 2026-10-31T18:41:48+09:00; 직전 파일 커밋: 2026-10-30T19:50:11+09:00 (c417310ca).
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
/// run 하나가 원문과 같으면 종전 기준 그대로이고, 여러 run 을 이을 때는 원문이 조각의 끝
/// (뒤에 공백만 남음)까지 이어져야 한다 — 낱말마다 run 이 나뉜 다른 줄의 앞부분이 원문과
/// 같은 경우를 같은 글로 세지 않는다(241쪽). 공백뿐인 run 은 시작·끝으로 세지 않는다.
fn text_anchor_runs<'a>(
    nodes: &[&'a RenderNode],
    text: &str,
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
            let mut joined = String::new();
            for (end, (_, run)) in segment.iter().enumerate().skip(start) {
                joined.push_str(&run.text);
                let rest_blank = segment[end + 1..]
                    .iter()
                    .all(|(_, rest)| rest.text.trim().is_empty());
                if !run.text.trim().is_empty()
                    && joined.trim() == text
                    && (end == start || rest_blank)
                {
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
                let runs = text_anchor_runs(&nodes, text);
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
