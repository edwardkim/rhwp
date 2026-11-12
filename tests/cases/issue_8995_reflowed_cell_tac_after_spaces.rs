//! [#8995] 관계형·고정형 회귀 — 저장 줄 없는 칸 문단의 공백 뒤 TAC 표는 배치된 줄의 펜과 줄 위끝을 원점으로 쓴다.
//!
//! 주석 갱신: 2026-11-12 15:40 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8995/reflowed_cell_tac_after_spaces.hwpx` — 공개 결재 문서
//!   「(0617)디자인산업본부사업업무추진비지출(2건)」(HWPX, 1쪽). 쪽 기준 큰 표의 본문 칸(행 8, 열 0)
//!   넷째 문단(칸 문단 3)이 공백 2칸 + TAC 표(3행×5열, 바깥 여백 왼쪽 140·위 140HU)이며
//!   `linesegarray` 가 없다(재조판 문단).
//! - 독립 기준: `pdf/issue8995/reflowed_cell_tac_after_spaces-2020.pdf`(한/글 2022 12.0.0.4547 Print,
//!   1쪽). 표 왼쪽 괘선 = 공백 2칸 뒤 + 바깥 여백 왼쪽, 표 위 괘선 = 배치된 줄 위끝 + 바깥 여백 위.
//! - 수정 전 rhwp(devel `80b1c354c2`)는 공백(15.1px)을 그려 놓고도 표를 줄 맨 앞에 두고(15px 왼쪽),
//!   문단 앞 간격 전 커서를 원점으로 써 표가 줄 위끝보다 4.8px 위(PDF 보다 7px 위)에서 시작했다.
//!
//! 측정: 2026-11-12, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣). 전체 1쪽 = 한/글 1쪽.
//! 수정 후(devel `80b1c354c2` 기준) 1쪽 Native 99.99%, fresh WASM 은 `mydocs/pr/assets/issue_8995/wasm/`
//! TSV 참조(두 경로 전쪽 최저값 = 1쪽 값). 수정 전 Native 78.38%.
//! 증거: `mydocs/pr/assets/issue_8995/{native,wasm,base-native}/`.
//!
//! 검사: 원본 계약(칸 문단 3 은 저장 줄 없음, 공백 뒤 TAC 표), 쪽수 1 = 한/글(고정형),
//! 표 왼쪽 = 공백 글자 끝 + 바깥 여백 왼쪽, 표 위 = 그 줄 위끝 + 바깥 여백 위, 표가 앞 제목 줄 아래·
//! 칸 안(관계형).
//!
//! 반례(두 번째 검사): `samples/task2279/36358528_gyeoljae.hwpx` 본문 문단 5 — 저장 줄 없이 글자
//! `"    ************ \t  "` 뒤 TAC 표(폭 621px)가 와서 표가 합성 둘째 줄로 넘어간다. 조합기의 개체
//! 줄 소속(항상 0)을 그대로 쓰면 펜이 첫 줄 글자 폭만큼 밀려 표가 쪽 오른쪽 밖(x 288.7)으로 나간다.
//! 독립 기준 `pdf/issue8995/36358528_gyeoljae-2020.pdf`(한/글 2022 12.0.0.4547 Print, 1쪽, 로컬
//! 오라클 engine 2020). 같은 측정 방법 1쪽 Native: devel `80b1c354c2` 99.98%, 줄 소속을 고치지
//! 않은 중간 시안 78.64%, 최종 수정 99.98%. 검사: 쪽수 1(고정형), 표가 첫 줄 아래·첫 줄 글자 끝
//! 앞에서 시작하고 단 안에 머문다(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8995/reflowed_cell_tac_after_spaces.hwpx";
/// 쪽 기준 큰 표를 품은 본문 문단과 그 표 컨트롤.
const HOST: usize = 0;
/// 본문 칸.
const ROW: u16 = 8;
const COL: u16 = 0;
/// 공백 2칸 + TAC 표 문단.
const TAC_PARA: usize = 3;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn find_cell(node: &RenderNode, cell_index: u32) -> Option<&RenderNode> {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        if cell.model_cell_index == Some(cell_index) && cell.row == ROW && cell.col == COL {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_cell(child, cell_index))
}

#[test]
fn reflowed_cell_tac_table_starts_after_leading_spaces_on_laid_line() {
    let core = load();
    assert_eq!(core.page_count(), 1, "한/글 Print 와 같은 1쪽");

    let host = &core.document().sections[0].paragraphs[HOST];
    let table = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if !table.common.treat_as_char => Some(table),
            _ => None,
        })
        .expect("쪽 기준 큰 표");
    let cell_index = table
        .cells
        .iter()
        .position(|cell| cell.row == ROW && cell.col == COL)
        .expect("본문 칸");
    let para = &table.cells[cell_index].paragraphs[TAC_PARA];

    // 원본 계약: 저장 줄이 없고(파서 합성 줄만), 공백 뒤에 TAC 표 하나가 온다.
    assert!(
        para.line_segs.iter().all(|seg| seg.tag & 0x8000_0000 != 0),
        "칸 문단 {TAC_PARA} 은 저장 줄 없는 재조판 문단"
    );
    assert!(
        !para.text.is_empty() && para.text.chars().all(|c| c == ' '),
        "표 앞 글자는 공백뿐: {:?}",
        para.text
    );
    let nested = para
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.common.treat_as_char => Some(table),
            _ => None,
        })
        .expect("공백 뒤 TAC 표");
    let om_left = hu_to_px(i32::from(nested.outer_margin_left));
    let om_top = hu_to_px(i32::from(nested.outer_margin_top));
    assert!(om_left > 0.0 && om_top > 0.0, "바깥 여백 왼쪽·위가 있다");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let cell_node = find_cell(&page.root, cell_index as u32).expect("본문 칸 상자");

    let line_of = |para_index: usize| {
        cell_node
            .children
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::TextLine(line)
                    if line.para_index == Some(para_index) && line.line_index == Some(0))
            })
            .unwrap_or_else(|| panic!("칸 문단 {para_index} 첫 줄"))
    };
    let tac_line = line_of(TAC_PARA);
    let spaces_end = tac_line
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextRun(_)))
        .map(|node| node.bbox.x + node.bbox.width)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        spaces_end > tac_line.bbox.x + 1.0,
        "공백 글자가 줄 안에 폭을 갖는다"
    );

    let table_node = cell_node
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Table(_)))
        .expect("중첩 표 상자");

    assert!(
        (table_node.bbox.x - (spaces_end + om_left)).abs() <= 0.5,
        "표 왼쪽 {:.1} = 공백 끝 {spaces_end:.1} + 바깥 여백 {om_left:.1}",
        table_node.bbox.x
    );
    assert!(
        (table_node.bbox.y - (tac_line.bbox.y + om_top)).abs() <= 0.5,
        "표 위 {:.1} = 줄 위끝 {:.1} + 바깥 여백 {om_top:.1}",
        table_node.bbox.y,
        tac_line.bbox.y
    );

    let heading = line_of(TAC_PARA - 1);
    assert!(
        table_node.bbox.y >= heading.bbox.y + heading.bbox.height,
        "표 위 {:.1} 는 앞 제목 줄 아래끝 {:.1} 아래",
        table_node.bbox.y,
        heading.bbox.y + heading.bbox.height
    );
    let cell_bottom = cell_node.bbox.y + cell_node.bbox.height;
    assert!(
        table_node.bbox.y + table_node.bbox.height <= cell_bottom + 0.5,
        "표 아래끝 {:.1} 은 칸 아래끝 {cell_bottom:.1} 안",
        table_node.bbox.y + table_node.bbox.height
    );
}

/// 반례: 저장 줄 없는 본문 문단에서 글자 뒤 TAC 표가 줄에 들어가지 않아 둘째 줄로 넘어간 경우.
/// 개체 줄 소속을 첫 줄로 보면 펜이 첫 줄 글자 폭만큼 밀려 표가 쪽 밖으로 나간다.
const COUNTER_SAMPLE: &str = "samples/task2279/36358528_gyeoljae.hwpx";
const COUNTER_PARA: usize = 5;

fn find_node<'a>(
    node: &'a RenderNode,
    pred: &dyn Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, pred))
}

#[test]
fn reflowed_body_tac_table_on_second_line_keeps_line_start_pen() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(COUNTER_SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), 1, "한/글 Print 와 같은 1쪽");

    // 원본 계약: 저장 줄이 없고, 글자 뒤 TAC 표 하나가 문단 끝에 있으며 표 폭이 첫 줄 글자 뒤
    // 남은 폭보다 넓다(합성 둘째 줄이 표에서 시작).
    let para = &core.document().sections[0].paragraphs[COUNTER_PARA];
    assert!(
        para.line_segs.iter().all(|seg| seg.tag & 0x8000_0000 != 0),
        "본문 문단 {COUNTER_PARA} 은 저장 줄 없는 재조판 문단"
    );
    assert!(
        !para.text.trim().is_empty(),
        "표 앞에 글자가 있다: {:?}",
        para.text
    );
    let [Control::Table(table)] = para.controls.as_slice() else {
        panic!("문단 {COUNTER_PARA} 의 개체는 TAC 표 하나");
    };
    assert!(table.common.treat_as_char);

    let page = core.build_page_render_tree(0).expect("1쪽");
    let column = find_node(&page.root, &|node| {
        matches!(node.node_type, RenderNodeType::Column(0))
    })
    .expect("본문 단");
    let first_line = find_node(column, &|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(COUNTER_PARA) && line.line_index == Some(0))
    })
    .expect("문단 첫 줄");
    let text_end = first_line
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextRun(_)))
        .map(|node| node.bbox.x + node.bbox.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let table_node = find_node(column, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(COUNTER_PARA) && t.control_index == Some(0))
    })
    .expect("본문 TAC 표 상자");

    // 표는 첫 줄 글자 뒤가 아니라 둘째 줄에서 시작하고 단 안에 머문다.
    assert!(
        table_node.bbox.y >= first_line.bbox.y + first_line.bbox.height - 0.5,
        "표 위 {:.1} 는 첫 줄 아래끝 {:.1} 아래",
        table_node.bbox.y,
        first_line.bbox.y + first_line.bbox.height
    );
    assert!(
        table_node.bbox.x < text_end,
        "표 왼쪽 {:.1} 는 첫 줄 글자 끝 {text_end:.1} 앞(둘째 줄 펜)",
        table_node.bbox.x
    );
    let column_right = column.bbox.x + column.bbox.width;
    assert!(
        table_node.bbox.x >= column.bbox.x - 0.5
            && table_node.bbox.x + table_node.bbox.width <= column_right + 0.5,
        "표 {:.1}~{:.1} 는 단 {:.1}~{column_right:.1} 안",
        table_node.bbox.x,
        table_node.bbox.x + table_node.bbox.width,
        column.bbox.x
    );
}
