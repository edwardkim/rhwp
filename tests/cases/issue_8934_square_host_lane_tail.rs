//! [#8934] 관계형·고정형 회귀 — 앞 글자가 있는 host 의 문단 기준 어울림 표 옆 차선 문단이 표 아래
//! 끝보다 아래까지 이어지면, 뒤 문단은 표 바닥이 아니라 마지막 차선 줄 끝에서 시작한다.
//!
//! 주석 갱신: 2026-11-11 14:10 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8934/square_host_lane_tail.hwp` — 서울시교육청 공개 문서
//!   「2018학년도 후기고등학교 입학전 전학 시행 계획(안)」 8쪽 범위만 남기고 한/글 2022
//!   (12.0.0.4547)로 다시 저장한 1쪽 문서. 문단 1 은 글자 + 문단 기준 어울림 표, 문단 2~4 는 표
//!   왼쪽·오른쪽 차선 두 세그먼트의 저장 줄을 가진 빈/공백 문단이다.
//! - 독립 기준: `pdf/issue8934/square_host_lane_tail-2020.pdf`(같은 저장본의 한/글 2022 Print, 1쪽).
//! - 수정 전 rhwp(devel `e118de8aba`)는 이 host 에 Square 띠를 등록하지 않아 차선의 저장 끝을
//!   버리고 문단 5 를 표 바닥에서 시작했다(마지막 차선 줄과 겹치고 뒤 본문 전체가 약 26px 위).
//!
//! 측정: 2026-11-11, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `e118de8aba` + 이 수정.
//! 1쪽 Native 98.43 / fresh WASM 98.50(수정 전 Native 60.32). 전체 1쪽 = PDF 1쪽,
//! 전쪽 최저 Native 98.43(1쪽)·fresh WASM 98.50(1쪽).
//! 증거: `mydocs/pr/assets/issue_8934/{native,wasm}/`(TSV)·review·overlay PNG.
//!
//! 검사: 원본 계약(host 글자 있음·문단 기준 어울림 표, 차선 문단 저장 줄이 두 세그먼트이고 마지막
//! 차선 줄 끝이 표 바닥보다 아래), 쪽수 1 = 한/글(고정형), 차선 줄 셋이 표 오른쪽에 놓이고 문단 5
//! 첫 줄은 표와 겹치지 않으며 마지막 차선 줄 끝 + 그 저장 줄간격 뒤에서 시작한다(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8934/square_host_lane_tail.hwp";
const HOST: usize = 1;
const LANES: [usize; 3] = [2, 3, 4];
const NEXT: usize = 5;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 본문 흐름(표 칸 안 제외)의 글줄·표 상자를 모은다.
#[derive(Default)]
struct Flow {
    tables: Vec<(usize, BoundingBox)>,
    lines: Vec<(Option<usize>, BoundingBox)>,
}

fn collect(node: &RenderNode, flow: &mut Flow) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index {
                flow.tables.push((para, node.bbox));
            }
            return;
        }
        RenderNodeType::TextLine(line) => flow.lines.push((line.para_index, node.bbox)),
        _ => {}
    }
    for child in &node.children {
        collect(child, flow);
    }
}

#[test]
fn next_paragraph_starts_after_square_lane_that_runs_below_table() {
    let core = load();
    let doc = core.document();
    let paragraphs = &doc.sections[0].paragraphs;

    // 원본 계약: host 는 글자 + 문단 기준 어울림(비 TAC) 표.
    let host = &paragraphs[HOST];
    assert!(!host.text.trim().is_empty(), "host 앞 글자");
    let table = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("host 표");
    assert!(!table.common.treat_as_char, "글자처럼 취급 아님");
    assert!(
        matches!(table.common.text_wrap, rhwp::model::shape::TextWrap::Square),
        "어울림 표"
    );
    assert!(matches!(
        table.common.vert_rel_to,
        rhwp::model::shape::VertRelTo::Para
    ));
    let host_vpos = host.line_segs[0].vertical_pos;
    let table_bottom_hu =
        host_vpos + table.common.vertical_offset as i32 + table.common.height as i32;
    // 차선 문단은 표 왼쪽·오른쪽 두 세그먼트의 저장 줄을 갖고, 마지막 차선 줄 끝이 표 바닥보다 아래다.
    for &index in &LANES {
        let segs = &paragraphs[index].line_segs;
        assert_eq!(segs.len(), 2, "문단 {index} 차선 두 세그먼트");
        assert!(segs[1].column_start > 0, "문단 {index} 오른쪽 차선");
    }
    let last = paragraphs[LANES[2]].line_segs.last().unwrap();
    let lane_end_hu = last.vertical_pos + last.line_height + last.line_spacing;
    assert!(
        lane_end_hu > table_bottom_hu,
        "차선 끝 {lane_end_hu} > 표 바닥 {table_bottom_hu}"
    );

    assert_eq!(core.page_count(), 1, "전체 쪽수 = 한/글 1쪽");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut flow = Flow::default();
    collect(&page.root, &mut flow);

    let table_box = flow
        .tables
        .iter()
        .find(|(owner, _)| *owner == HOST)
        .map(|(_, bbox)| *bbox)
        .expect("host 표 상자");
    let table_right = table_box.x + table_box.width;
    let table_bottom = table_box.y + table_box.height;

    // 차선 줄: 표 오른쪽에서 표 높이 안에서 시작하는 글줄 셋.
    let mut lane_lines: Vec<BoundingBox> = flow
        .lines
        .iter()
        .map(|(_, bbox)| *bbox)
        .filter(|bbox| bbox.x >= table_right - 0.5 && bbox.y >= table_box.y - 0.5)
        .collect();
    lane_lines.sort_by(|a, b| a.y.total_cmp(&b.y));
    assert_eq!(lane_lines.len(), LANES.len(), "표 오른쪽 차선 줄");
    assert!(
        lane_lines.iter().all(|line| line.y < table_bottom),
        "차선 줄은 표 옆에서 시작"
    );
    let lane_bottom = lane_lines.last().map(|line| line.y + line.height).unwrap();
    assert!(
        lane_bottom > table_bottom,
        "마지막 차선 줄이 표 바닥 아래까지"
    );

    let next = flow
        .lines
        .iter()
        .filter(|(owner, _)| *owner == Some(NEXT))
        .map(|(_, bbox)| *bbox)
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("문단 5 첫 줄");
    assert!(
        next.y >= table_bottom - 0.5,
        "문단 5 는 표와 겹치지 않는다: {next:?}"
    );
    // 수정 전에는 문단 5 가 표 바닥에서 시작해 마지막 차선 줄과 겹쳤다.
    assert!(
        next.y + 0.5 >= lane_bottom + hu_to_px(last.line_spacing),
        "문단 5 는 마지막 차선 줄 끝 + 줄간격 뒤에서 시작: 문단 5 y={}, 차선 끝 {lane_bottom}",
        next.y
    );
}
