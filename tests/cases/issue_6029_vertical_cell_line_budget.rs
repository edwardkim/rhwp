//! [#6029] 표 셀 세로쓰기의 세로줄 예산이 칸 높이가 아니라 칸 너비에서 나와
//! 직함 글자가 소실되던 결함 가드.
//!
//! 3200477(ICAO 처리 절차, 14KB HWPX) 1쪽 "담당" 열 세로쓰기 칸 3곳 — 저장
//! lineseg 는 세로줄 하나(extent 를 horzsize=칸 높이 축에 인코딩)인데, 셀
//! 공통의 칸-너비 재분할(Task #671 recompose)이 이를 "가로로 넘친 한 줄"로
//! 오인해 열을 2~3자마다 쪼갰다. 넘친 열이 왼쪽으로 밀리다 칸 밖 분량이
//! 소실돼 "(항공자격국제협력팀장)" 등 직함 27자 중 18자가 사라졌다(한글
//! 2020: 칸 높이 ~113pt 한 열에 11자). 수정 = 세로쓰기 배치 진입 시 원문으로
//! 다시 compose(저장 열 구조 보존)하고 칸 **높이** measure 로만 재분할.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6029/3200477_icao_procedure.hwpx";

fn collect_runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            out.push((node.bbox.x, node.bbox.y, run.text.clone()));
        }
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

#[test]
fn issue_6029_vertical_cell_title_stays_in_one_column() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse #6029 fixture");

    let p1 = core.build_page_render_tree(0).expect("render p1");
    let mut runs = Vec::new();
    collect_runs(&p1.root, &mut runs);
    let all: String = runs.iter().map(|(_, _, t)| t.as_str()).collect();

    // 소실됐던 직함 3건이 전부 방출돼야 한다.
    for probe in ["항공자격국제협력팀장", "담당부서장", "항공안전정책과장"] {
        assert!(
            all.contains(probe),
            "직함 {probe:?} 이 렌더에서 소실됨 (칸-너비 재분할 회귀)",
        );
    }

    // 세로줄 구조: "(항공자격국제협력팀장)" 13자(괄호 포함)는 하나의 열 —
    // 글자들의 x 가 한 값이고 y 가 단조 증가해야 한다. 칸-너비 분할이면
    // 열이 2~3자마다 끊겨 서로 다른 x 로 흩어진다.
    // 열 기준점: 상단 셀 ① 의 '항' 단일 글자 run.
    let x_ref = runs
        .iter()
        .find(|(_, y, t)| t == "항" && *y < 420.0)
        .map(|(x, _, _)| *x)
        .expect("셀 ① 세로쓰기 '항' run");
    let title_chars = "항공자격국제협력팀장";
    for ch in title_chars.chars() {
        let in_column = runs.iter().any(|(x, y, t)| {
            t.chars().count() == 1
                && t.chars().next() == Some(ch)
                && *y < 420.0
                && (*x - x_ref).abs() < 1.0
        });
        assert!(
            in_column,
            "직함 글자 {ch:?} 가 '항' 열(x={x_ref:.1}) 밖 — 열이 쪼개졌거나 소실됨",
        );
    }
}
