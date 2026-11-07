//! [#8715] 관계형·고정형 회귀 — 글 문단이 품은 `CELL` 분할 중첩 표가 쪽 경계에서 행 안을
//! 나눌 때, 칸마다 자기 글줄 경계에서 끊고 이어받는 쪽은 앞 쪽이 그리지 않은 줄부터 그린다.
//!
//! 주석 갱신: 2026-11-07 (KST) — 측정값은 아래 표(측정 전 자리표시 아님, 제출 전 갱신)
//!
//! - 입력: `samples/issue8715/nested_row_cell_cuts.hwp` — 공개 규제영향분석서(농업기계화
//!   촉진법 시행규칙 일부개정령안, HWPX)에서 비교표 1×1 표가 든 문단만 남기고, 일본어 원문
//!   글자를 한글 음절로 바꾼 뒤 한/글 2022 로 다시 저장한 파생 입력(3쪽). 바깥 1×1 칸의
//!   문단 0 은 글(「- 도로운송차량법 시행규칙 …」)과 13×2 중첩 표(`CELL` 분할, 자리차지)를
//!   함께 담는다. 한/글은 재저장 때 행 10 의 두 칸 모두 셋째 줄에서 저장 vpos 를 0 으로
//!   되감아(쪽 나눔 자리) 저장했다.
//! - 독립 기준: 같은 입력의 한/글 2022 Print PDF `pdf/issue8715_nested_row_cell_cuts-2020.pdf`
//!   (3쪽). 2쪽 바닥에 행 10 의 두 줄, 3쪽 머리에 나머지 다섯 줄.
//! - 수정 전 rhwp(#8686 2단계 head 80cc647e56): 글 문단 호스트의 중첩 표를 16px scalar 조각으로
//!   예약해, 2쪽은 반쯤 보인 행 10 을 버리고(#5846) 3쪽은 그 띠를 이미 보인 것으로 건너뛰어(#3658)
//!   행 10 의 앞 두 줄이 어느 쪽에도 없었다. 3쪽 뒤 행의 칸 내용도 시작 행 오프셋만큼 위로
//!   올라가 칸 상자 위에 그려졌다.
//!
//! 측정(`scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣): 아래 표는 제출
//! head 에서 갱신한다.
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | 쪽 | Native | fresh WASM | 수정 전 Native |
//! |---|---|---|---|---|---|
//! | nested_row_cell_cuts | 3 | 1 | TBD | TBD | 94.31 |
//! | | | 2 | TBD | TBD | 93.78 |
//! | | | 3 | TBD | TBD | 39.58 |
//!
//! 증거: `mydocs/pr/assets/issue_8715/{native,native_base,wasm}/nested_row_cell_cuts_silhouette.tsv`.
//! 검사는 행 10 칸의 저장 되감김 위치(전제), 칸별 줄의 쪽 소속·순서·누락·중복(관계형),
//! 이어받는 쪽 뒤 행 글줄이 제 칸 상자 안에 있다는 포함 관계(관계형), 독립 PDF 전체
//! 쪽수(고정형)다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8715/nested_row_cell_cuts.hwp";
const SPLIT_ROW: u16 = 10;

fn node_text(node: &RenderNode) -> String {
    let mut out = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        out.push_str(&node_text(child));
    }
    out
}

fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 중첩 표(행 ≥ 1 이 있는 표)의 칸마다 (행, 열, 칸 상자, 글줄 [(글, 상자)]).
type CellLines = Vec<(u16, u16, BoundingBox, Vec<(String, BoundingBox)>)>;

fn collect_cells(node: &RenderNode, out: &mut CellLines) {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        if cell.row >= 1 {
            let mut lines = Vec::new();
            collect_lines(node, &mut lines);
            out.push((cell.row, cell.col, node.bbox.clone(), lines));
            return;
        }
    }
    for child in &node.children {
        collect_cells(child, out);
    }
}

fn collect_lines(node: &RenderNode, out: &mut Vec<(String, BoundingBox)>) {
    if let RenderNodeType::TextLine(_) = &node.node_type {
        let text = squeeze(&node_text(node));
        if !text.is_empty() {
            out.push((text, node.bbox.clone()));
        }
        return;
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

fn nested_table(core: &DocumentCore) -> &Table {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Table(outer) => outer.cells.iter().find_map(|cell| {
                cell.paragraphs.iter().find_map(|para| {
                    if para.text.trim().is_empty() {
                        return None;
                    }
                    para.controls.iter().find_map(|control| match control {
                        Control::Table(child) if child.row_count >= 2 => Some(child.as_ref()),
                        _ => None,
                    })
                })
            }),
            _ => None,
        })
        .expect("글 문단이 품은 중첩 표")
}

#[test]
fn text_hosted_nested_row_splits_per_cell_line_and_keeps_every_line() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    let page_count = core.page_count();
    assert_eq!(page_count, 3, "한컴 Print PDF 3쪽");

    // 전제: 글 문단 호스트의 13×2 `CELL` 분할 자리차지 표이고, 한/글이 행 10 의 두 칸을
    // 같은 줄에서 되감아 저장했다(그 줄이 다음 쪽의 첫 줄).
    let table = nested_table(&core);
    assert!(!table.common.treat_as_char, "자리차지 표");
    assert!(
        matches!(table.layout_page_break(), TablePageBreak::RowBreak),
        "셀 단위 분할"
    );
    let split_cells: Vec<_> = table
        .cells
        .iter()
        .filter(|cell| cell.row == SPLIT_ROW)
        .collect();
    assert_eq!(split_cells.len(), 2, "행 {SPLIT_ROW} 의 칸 둘");
    let mut expected = Vec::new();
    for cell in &split_cells {
        assert_eq!(cell.paragraphs.len(), 1, "행 {SPLIT_ROW} 칸은 문단 하나");
        let para = &cell.paragraphs[0];
        let reset = para
            .line_segs
            .windows(2)
            .position(|pair| pair[1].vertical_pos < pair[0].vertical_pos)
            .map(|index| index + 1)
            .expect("저장 되감김");
        assert!(reset >= 1 && reset < para.line_segs.len());
        expected.push((cell.col, reset, para.line_segs.len(), squeeze(&para.text)));
    }

    let pages: Vec<CellLines> = (0..page_count)
        .map(|page| {
            let tree = core
                .build_page_render_tree(page)
                .unwrap_or_else(|e| panic!("{}쪽 렌더 트리: {e:?}", page + 1));
            let mut cells = Vec::new();
            collect_cells(&tree.root, &mut cells);
            cells
        })
        .collect();

    // 관계형: 행 10 의 칸마다 2쪽은 되감김 앞 줄, 3쪽은 그 뒤 줄을 그리고, 두 쪽을 이으면
    // 원문 글이 빠짐·겹침 없이 그대로 나온다.
    for (col, reset, total, text) in &expected {
        let lines_on = |page: usize| -> Vec<String> {
            pages[page]
                .iter()
                .filter(|(row, c, _, _)| *row == SPLIT_ROW && c == col)
                .flat_map(|(_, _, _, lines)| lines.iter().map(|(text, _)| text.clone()))
                .collect()
        };
        let (first, second) = (lines_on(1), lines_on(2));
        assert_eq!(
            first.len(),
            *reset,
            "행 {SPLIT_ROW} 칸 {col}: 2쪽은 되감김 앞 {reset}줄 — {first:?}"
        );
        assert_eq!(
            second.len(),
            total - reset,
            "행 {SPLIT_ROW} 칸 {col}: 3쪽은 나머지 {}줄 — {second:?}",
            total - reset
        );
        assert_eq!(
            first.concat() + &second.concat(),
            *text,
            "행 {SPLIT_ROW} 칸 {col}: 두 쪽의 줄을 이으면 원문"
        );
    }

    // 관계형: 이어받는 3쪽에서 뒤 행 칸의 글줄은 제 칸 상자 안에 있다.
    for (row, col, cell_box, lines) in pages[2].iter().filter(|(row, ..)| *row > SPLIT_ROW) {
        for (text, line) in lines {
            assert!(
                line.y >= cell_box.y - 0.5
                    && line.y + line.height <= cell_box.y + cell_box.height + 0.5,
                "3쪽 행 {row} 칸 {col} 글줄 {text:?} ({:.1}..{:.1}) 은 칸 상자 {:.1}..{:.1} 안",
                line.y,
                line.y + line.height,
                cell_box.y,
                cell_box.y + cell_box.height
            );
        }
    }
}
