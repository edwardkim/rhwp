//! 쪽 끝 행을 내용 컷으로 가를 때, 그 행에서 끝나는 걸침 칸의 줄은 쪽 닫는 선까지 남는다.
//!
//! 1342000 교육과정 지도 구역 10 표(`pi=1`)의 199쪽 끝 행 63 은 칸 c4 저장 둘째 줄 `특성` 이
//! vpos 0 에서 다시 시작해(한/글 쪽 컷 기록) 첫 줄 `고등학교 유형과` 에서 갈린다. 같은 행에서
//! 끝나는 걸침 칸 r=62 c8(행 62–63)의 세 문단은 저장 vpos 200·1700·3200 으로 되감김이 없고,
//! 한/글 2020 PDF(`current-source-hancom2020-print.pdf`)는 셋째 줄 `- 희망진학학교 준비활동
//! 보고서작성하기` 를 199쪽 닫는 선(716.0) 안 699.8px 에 둔다.
//!
//! 수정 전 rhwp 는 걸침 칸의 끝을 행 63 의 내용 컷(첫 줄 16px)으로 재 셋째 줄을 200쪽으로
//! 넘겼다. 200쪽 첫 행이 2.9px 높아져 행 75 가 쪽 끝에서 그만큼 줄었고, 행 57–92 걸침 몫
//! 원장이 그 부족분을 끝 행 92 에 더해 202쪽 행 92 가 100.0px(선언 97.1, PDF 97.25)가 됐다.
//! 그 결과 202쪽 마지막 행의 둘째 줄이 203쪽으로 밀렸다.
//!
//! 반례: 행 63 의 c4 는 저장 재시작을 따라 `특성` 을 200쪽에서 잇는다(PDF 도 같다).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_1342000);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse: {error:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 공백을 지운 글자 조각 `needle` 이 놓인 쪽(0-기준)들.
fn pages_with(core: &DocumentCore, needle: &str) -> Vec<u32> {
    (0..core.page_count())
        .filter(|&page| {
            let tree = core.build_page_render_tree(page).expect("render page");
            let mut nodes = Vec::new();
            walk(&tree.root, &mut nodes);
            nodes.iter().any(|node| match &node.node_type {
                RenderNodeType::TextRun(run) => run
                    .text
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect::<String>()
                    .contains(needle),
                _ => false,
            })
        })
        .collect()
}

#[test]
fn span_cell_lines_inside_the_closing_line_stay_on_the_page() {
    let core = core();
    let row_63_first = pages_with(&core, "고등학교유형과");
    let span_third = pages_with(&core, "희망진학학교준비활동보고서작성하기");
    let row_63_second = pages_with(&core, "특성");
    assert_eq!(
        span_third, row_63_first,
        "걸침 칸 r=62 c8 셋째 줄은 행 63 첫 줄과 같은 쪽(199쪽)에 남아야 한다"
    );
    let page = row_63_first[0];
    assert!(
        row_63_second.contains(&(page + 1)),
        "행 63 c4 둘째 줄 `특성` 은 저장 재시작대로 다음 쪽({}쪽)에서 잇는다: {row_63_second:?}",
        page + 2
    );
}

#[test]
fn span_end_row_keeps_its_declared_height() {
    // 행 92(선언 7283HU = 97.1px)는 행 57–92·67–92 두 걸침 칸이 함께 끝나는 행이다. 앞 쪽의
    // 줄 밀림이 걸침 몫 원장을 거쳐 이 행에 더해지면 안 된다(PDF 97.25px).
    let core = core();
    let page = *pages_with(&core, "연극관련다양한직업이해하기")
        .first()
        .expect("행 92 쪽");
    let tree = core.build_page_render_tree(page).expect("render page");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let height = nodes
        .iter()
        .find_map(|node| match &node.node_type {
            RenderNodeType::TableCell(cell) if cell.row == 92 && cell.col == 8 => {
                Some(node.bbox.height)
            }
            _ => None,
        })
        .expect("행 92 c8 칸");
    let declared = 7283.0 * 96.0 / 7200.0;
    assert!(
        (height - declared).abs() <= 1.0,
        "{}쪽 행 92: {height:.1}px, 선언 {declared:.1}px",
        page + 1
    );
}
