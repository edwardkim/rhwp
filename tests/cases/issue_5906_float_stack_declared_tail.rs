//! [Issue #5906] 빈 host 자리차지 표의 마지막 행이 저장 선언 초과분을 흡수하지
//! 않아 2쪽 문서가 3쪽으로 갈린다 (`samples/float-stack-defer.hwp`).
//!
//! 문단 1 은 빈 host 이고 자리차지(TopAndBottom)·문단 기준 float 표 두 개를
//! 소유한다. 두 번째 12행 표는 한글 2022 정본(`pdf/float-stack-defer-2022.pdf`)
//! 에서 2쪽 한 장에 통째로 들어간다 — 괘선 실측 90.86pt→770.64pt = 679.78pt 로
//! 저장 선언높이 hp:sz 68,051HU(680.51pt)와 같다.
//!
//! rhwp 는 앞 11행 경계를 정본과 ±1px 로 맞추면서 마지막 행만 저장
//! cellSz(5803HU=77.37px)를 그대로 썼다. 그 결과 표 측정 합 914.24px 가 선언
//! 907.35px 를 6.89px 넘겨 본문 바닥(1031.81px)을 1.6px 초과하고, 마지막 두 행이
//! 3쪽으로 밀렸다. 정본의 마지막 행은 70.48px — 정확히 초과분만큼 줄어 있다.
//!
//! 수정: 마지막 행이 저장 선언으로만 잡히고 저장 줄 내용 위로 여유가 남아 있으면
//! 그 행에서만 초과분을 회수한다(`fit_measured_table_declared_tail_to_declared_height`).
//! 페인트 경로가 이미 반대 방향(부족분 → 마지막 행)으로 하던 일의 대칭이다.
//!
//! 결함 상태에서는 쪽수 3, 2쪽 괘선 경계 11개(10행)로 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/float-stack-defer.hwp";

#[test]
fn issue_5906_second_float_table_keeps_all_rows_on_page_two() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(
        core.page_count(),
        2,
        "한글 2022 정본은 2쪽이다 (결함 시 마지막 두 행만 3쪽으로 밀린다)"
    );

    let svg = core.render_page_svg_native(1).expect("page 2 svg");
    let rules = horizontal_rule_ys(&svg);
    assert_eq!(
        rules.len(),
        13,
        "2쪽 표 괘선 경계는 12행 = 13개여야 한다 (결함 시 10행 = 11개): {rules:?}"
    );

    let tree = core.build_page_render_tree(1).expect("2쪽 렌더 트리");
    let mut nodes = Vec::new();
    collect_nodes(&tree.root, &mut nodes);
    let body = nodes
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문 영역");
    let body_bottom = body.bbox.y + body.bbox.height;
    let bottom = *rules.last().expect("마지막 괘선");
    assert!(bottom <= body_bottom, "모든 행이 본문 안에 있어야 한다");
    let last_row_top = rules[rules.len() - 2];
    let table = core.document().sections[0].paragraphs[1]
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .nth(1)
        .expect("두 번째 원본 표");
    let stored_last_height = table
        .cells
        .iter()
        .filter(|cell| cell.row == table.row_count - 1)
        .map(|cell| f64::from(cell.height) * 96.0 / 7200.0)
        .fold(0.0, f64::max);
    assert!(
        bottom > last_row_top && bottom - last_row_top < stored_last_height,
        "마지막 행은 저장 선언 초과분을 회수해야 한다"
    );
    let dots: Vec<_> = nodes
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if run.text == "․" => Some((*node, run)),
            _ => None,
        })
        .collect();
    assert!(!dots.is_empty(), "원본의 한점 리더가 유지돼야 한다");
    for (node, run) in dots {
        let expected = (run.style.font_size + run.style.letter_spacing) * run.style.ratio;
        assert!(
            (node.bbox.width - expected).abs() < 0.1,
            "HFT 한점 리더는 원본 장평·자간을 적용한 전각 전진이어야 한다: 실제={}, 기대={expected}, 원본 HFT={}", node.bbox.width, run.style.hft_fullwidth_dot
        );
    }
}

fn collect_nodes<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect_nodes(child, nodes);
    }
}

/// 표 괘선 중 가로선(`y1 == y2`, 길이 100px 초과)의 y 를 오름차순으로 모은다.
/// 제목행 아래 이중 괘선은 3px 이내로 묶어 한 경계로 센다 (가장 얇은 행이 41px 라
/// 실제 행 경계가 묶일 여지는 없다).
fn horizontal_rule_ys(svg: &str) -> Vec<f64> {
    let mut ys: Vec<f64> = Vec::new();
    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..end];
        let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
            attr(head, "x1"),
            attr(head, "y1"),
            attr(head, "x2"),
            attr(head, "y2"),
        ) else {
            continue;
        };
        if (y1 - y2).abs() > 0.01 || (x2 - x1).abs() < 100.0 {
            continue;
        }
        ys.push(y1);
    }
    ys.sort_by(|a, b| a.partial_cmp(b).expect("좌표는 유한값"));

    let mut merged: Vec<f64> = Vec::new();
    for y in ys {
        let is_new_boundary = match merged.last() {
            None => true,
            Some(last) => (y - last).abs() > 3.0,
        };
        if is_new_boundary {
            merged.push(y);
        }
    }
    merged
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
