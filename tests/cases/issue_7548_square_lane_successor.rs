//! [#7548] 어울림(Square) 표 옆 차선에서 시작하는 다음 문단은 표 하단으로 밀리지 않는다.
//!
//! `samples/21_언어_기출_편집가능본.hwp` 14쪽 — `[A]` 꺾쇠 3x2 어울림 표(host pi=299,
//! 문단 기준) 다음 문단 pi=300 `최근에는 기존의 …`.
//!
//! 저장 LineSeg: pi=299 마지막 줄 vpos 66420, pi=300 첫 줄 vpos 68236(같은 줄 간격
//! 1816HU 로 이어짐). pi=300 첫 줄만 표 옆 차선 `cs=3455 sw=27581`(host 와 같음),
//! 둘째 줄부터 전폭 `cs=852 sw=30184`. 한/글 2022 정본(`pdf/21_언어_기출_편집가능본-2022.pdf`)
//! 도 첫 줄을 표 옆(글자 x 639.4), 둘째 줄을 전폭(x 591.5)에 그린다.
//!
//! 수정 전 rhwp 는 host 를 그린 뒤 흐름을 표 하단까지 보내 pi=300 을 표 아래에 그렸고
//! (+18.6px), wrap 줄 x 에 왼 여백을 cs 위에 한 번 더 얹어 11.4px 오른쪽에 그렸다.
//! 검사는 절대 좌표가 아니라 관계(저장 줄 간격·표 옆/전폭 차선)로 한다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/21_언어_기출_편집가능본.hwp";
const PAGE_INDEX: u32 = 13;
const HOST_PARA: usize = 299;
const SUCCESSOR_PARA: usize = 300;
/// 저장 LineSeg 의 pi=299 마지막 줄 → pi=300 첫 줄 vpos 간격(1816HU).
const STORED_PITCH_PX: f64 = 1816.0 / 75.0;

#[derive(Debug, Clone)]
struct Line {
    para: usize,
    line: u32,
    x: f64,
    y: f64,
}

fn collect(node: &RenderNode, lines: &mut Vec<Line>, tables: &mut Vec<(f64, f64, f64, f64)>) {
    match &node.node_type {
        RenderNodeType::TextLine(tl) => {
            if let (Some(para), Some(line)) = (tl.para_index, tl.line_index) {
                lines.push(Line {
                    para,
                    line,
                    x: node.bbox.x,
                    y: node.bbox.y,
                });
            }
        }
        RenderNodeType::Table(_) => {
            tables.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, lines, tables);
    }
}

fn page_lines() -> (Vec<Line>, (f64, f64, f64, f64)) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read 21_언어 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse 21_언어 fixture");
    let page = core
        .build_page_render_tree(PAGE_INDEX)
        .expect("render 14쪽");
    let mut lines = Vec::new();
    let mut tables = Vec::new();
    collect(&page.root, &mut lines, &mut tables);
    // 꺾쇠 표: 폭이 좁고(본문 줄보다 훨씬 좁음) host 줄들과 세로로 겹치는 유일한 표.
    let host_top = lines
        .iter()
        .filter(|l| l.para == HOST_PARA)
        .map(|l| l.y)
        .fold(f64::INFINITY, f64::min);
    let bracket = tables
        .iter()
        .copied()
        .filter(|(_, y, w, h)| *w < 40.0 && *y < host_top + 40.0 && y + h > host_top)
        .collect::<Vec<_>>();
    assert_eq!(bracket.len(), 1, "pi=299 의 [A] 꺾쇠 표 하나: {tables:?}");
    (lines, bracket[0])
}

fn line(lines: &[Line], para: usize, line: u32) -> &Line {
    lines
        .iter()
        .find(|l| l.para == para && l.line == line)
        .unwrap_or_else(|| panic!("pi={para} line={line} 이 14쪽에 있어야 한다"))
}

#[test]
fn issue_7548_successor_first_line_keeps_stored_pitch_beside_square_table() {
    let (lines, (_, table_y, _, table_h)) = page_lines();
    let host_last = lines
        .iter()
        .filter(|l| l.para == HOST_PARA)
        .max_by_key(|l| l.line)
        .expect("host 줄");
    let first = line(&lines, SUCCESSOR_PARA, 0);
    let gap = first.y - host_last.y;
    assert!(
        (gap - STORED_PITCH_PX).abs() < 0.5,
        "pi=300 첫 줄은 host 마지막 줄에서 저장 줄 간격({STORED_PITCH_PX:.1}px)만큼 아래다 \
         — 표 하단으로 밀리면 안 된다: gap={gap:.1}"
    );
    assert!(
        first.y < table_y + table_h,
        "pi=300 첫 줄은 표 띠 안(표 옆)에서 시작한다: line y={:.1}, table bottom={:.1}",
        first.y,
        table_y + table_h
    );
}

#[test]
fn issue_7548_successor_first_line_is_in_table_lane_and_second_line_is_full_width() {
    let (lines, (table_x, _, table_w, _)) = page_lines();
    let first = line(&lines, SUCCESSOR_PARA, 0);
    let second = line(&lines, SUCCESSOR_PARA, 1);
    assert!(
        first.x >= table_x + table_w,
        "pi=300 첫 줄은 표 오른쪽 차선에서 시작한다: x={:.1}, table right={:.1}",
        first.x,
        table_x + table_w
    );
    // 같은 단의 전폭 줄(pi=298 둘째 줄)과 pi=300 둘째 줄은 같은 시작 x 다.
    let full = line(&lines, 298, 1);
    assert!(
        (second.x - full.x).abs() < 0.5,
        "pi=300 둘째 줄은 전폭 줄과 같은 x: {:.1} vs {:.1}",
        second.x,
        full.x
    );
    assert!(
        second.x + table_w <= first.x,
        "전폭 줄은 표 차선보다 표 폭 이상 왼쪽에서 시작한다: {:.1} + {:.1} vs {:.1}",
        second.x,
        table_w,
        first.x
    );
}
