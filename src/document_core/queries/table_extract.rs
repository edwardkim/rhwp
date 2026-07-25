//! 표 격자 구조화 추출 — 표 데이터의 기계 소비용.
//!
//! `Table.cells` 는 **앵커 셀만** 담고 각 셀이 `row/col/row_span/col_span` 을 가지므로
//! (`src/model/table.rs`), 격자를 펼치지 않고 그대로 직역한다. 병합은 span 으로 보존되고,
//! 셀 안의 표는 `nested` 로 재귀 표현한다.
//!
//! 파서/렌더 무변경의 읽기 전용 질의(추가 기능). 자기 라운드트립·시각 충실도와 무관.

use serde::Serialize;

use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::Paragraph;
use crate::model::table::{Cell, Table};

/// 격자 셀 하나. 병합 셀은 **앵커 위치에 한 번만** 나오고 덮인 칸은 출력하지 않는다.
#[derive(Debug, Clone, Serialize)]
pub struct GridCell {
    /// 행 주소 (0부터).
    pub row: u16,
    /// 열 주소 (0부터).
    pub col: u16,
    /// 세로 병합 개수 (병합 없으면 1).
    #[serde(rename = "rowSpan")]
    pub row_span: u16,
    /// 가로 병합 개수 (병합 없으면 1).
    #[serde(rename = "colSpan")]
    pub col_span: u16,
    /// 제목 셀 여부 (LIST_HEADER bit 18).
    #[serde(rename = "isHeader")]
    pub is_header: bool,
    /// 셀 문단 텍스트를 줄바꿈으로 결합한 값.
    pub text: String,
    /// 셀 안의 표(중첩). 없으면 생략된다.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nested: Vec<TableGrid>,
}

/// 본문 문단에서 표까지 내려가는 컨테이너 경로 한 단계.
///
/// `section`·`paragraph`만으로는 같은 문단에 놓인 여러 글상자·머리말·표 셀을
/// 구분할 수 없으므로, 컨트롤과 내부 문단(필요 시 셀)을 함께 남긴다.
#[derive(Debug, Clone, Serialize)]
pub struct TableContainerRef {
    /// 컨테이너 종류: textbox, header, footer, footnote, endnote, tableCell.
    pub kind: &'static str,
    /// 부모 문단 안의 컨트롤 인덱스.
    pub control: usize,
    /// 컨테이너 안의 문단 인덱스.
    pub paragraph: usize,
    /// tableCell 경로일 때만 셀 인덱스.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<usize>,
}

/// 표 하나의 격자 표현.
#[derive(Debug, Clone, Serialize)]
pub struct TableGrid {
    /// 문서 내 표 순번 (0부터, 중첩 표는 부모의 `nested` 안에서 다시 0부터).
    pub index: usize,
    /// 표가 놓인 구역 인덱스.
    pub section: usize,
    /// 표를 담은 문단 인덱스 — 인용·역참조용 주소.
    pub paragraph: usize,
    /// 표 자체의 부모 문단 내 컨트롤 인덱스.
    pub control: usize,
    /// 글상자·머리말·각주·표 셀 등 중첩 컨테이너 경로.
    #[serde(rename = "containerPath", skip_serializing_if = "Vec::is_empty")]
    pub container_path: Vec<TableContainerRef>,
    /// 행 수.
    pub rows: u16,
    /// 열 수.
    pub cols: u16,
    /// 앵커 셀 개수 (= `cells` 길이).
    #[serde(rename = "cellCount")]
    pub cell_count: usize,
    /// 표 캡션 텍스트. 없으면 생략된다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// 앵커 셀 목록 (행 우선).
    pub cells: Vec<GridCell>,
}

/// 문단 목록의 텍스트를 줄바꿈으로 결합한다.
fn paragraphs_text(paragraphs: &[Paragraph]) -> String {
    paragraphs
        .iter()
        .map(|p| p.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// 셀 문단 안에 들어 있는 표(중첩)를 모은다.
///
/// 깊이 상한은 `dump` 명령(`dump_table_deep`)과 같은 이유로 둔다 — 악의적/손상 문서의
/// 병적 중첩에서 스택이 터지지 않게 한다.
const MAX_NEST_DEPTH: usize = 8;

fn nested_tables(
    cell: &Cell,
    section: usize,
    root_paragraph: usize,
    table_control: usize,
    cell_index: usize,
    container_path: &[TableContainerRef],
    depth: usize,
) -> Vec<TableGrid> {
    if depth >= MAX_NEST_DEPTH {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (para_idx, para) in cell.paragraphs.iter().enumerate() {
        let mut cell_path = container_path.to_vec();
        cell_path.push(TableContainerRef {
            kind: "tableCell",
            control: table_control,
            paragraph: para_idx,
            cell: Some(cell_index),
        });
        collect_from_paragraph(
            para,
            section,
            root_paragraph,
            &cell_path,
            depth + 1,
            &mut out,
        );
    }
    out
}

fn build_grid(
    table: &Table,
    index: usize,
    section: usize,
    paragraph: usize,
    control: usize,
    container_path: &[TableContainerRef],
    depth: usize,
) -> TableGrid {
    let rows = table.row_count;
    let cols = table.col_count;
    let cells: Vec<GridCell> = table
        .cells
        .iter()
        // 손상 문서의 범위 밖 앵커는 격자를 깨뜨리므로 버린다 —
        // doclang 변환(`convert_table`)이 clamp 로 하는 방어와 같은 취지다.
        .filter(|c| c.row < rows.max(1) && c.col < cols.max(1))
        .enumerate()
        .map(|(cell_index, c)| GridCell {
            row: c.row,
            col: c.col,
            // 파서가 0을 남기는 경우가 있어 최소 1로 정규화한다 — 소비자가
            // 면적을 계산할 때 0 이면 격자가 무너진다.
            row_span: c.row_span.max(1),
            col_span: c.col_span.max(1),
            is_header: c.is_header,
            text: paragraphs_text(&c.paragraphs),
            nested: nested_tables(
                c,
                section,
                paragraph,
                control,
                cell_index,
                container_path,
                depth,
            ),
        })
        .collect();

    TableGrid {
        index,
        section,
        paragraph,
        control,
        container_path: container_path.to_vec(),
        rows: table.row_count,
        cols: table.col_count,
        cell_count: cells.len(),
        caption: table
            .caption
            .as_ref()
            .map(|cap| paragraphs_text(&cap.paragraphs))
            .filter(|s| !s.is_empty()),
        cells,
    }
}

/// 한 문단이 품은 표를 모은다 — 본문 직속 표뿐 아니라 **컨테이너 안의 표**까지.
///
/// 공문서는 표를 글상자·머리말·각주 안에 두는 배치가 흔하다. 최상위 `controls` 만
/// 훑으면(현행 `info` 의 표 열거가 그렇다) 그 표들이 통째로 누락된다.
fn collect_from_paragraph(
    para: &Paragraph,
    section: usize,
    root_paragraph: usize,
    container_path: &[TableContainerRef],
    depth: usize,
    out: &mut Vec<TableGrid>,
) {
    if depth >= MAX_NEST_DEPTH {
        return;
    }
    for (control_index, control) in para.controls.iter().enumerate() {
        match control {
            Control::Table(table) => {
                let index = out.len();
                out.push(build_grid(
                    table,
                    index,
                    section,
                    root_paragraph,
                    control_index,
                    container_path,
                    depth,
                ));
            }
            // 컨테이너 컨트롤 — 내부 문단을 재귀한다.
            Control::Shape(shape) => {
                if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                    for (paragraph, p) in tb.paragraphs.iter().enumerate() {
                        let mut path = container_path.to_vec();
                        path.push(TableContainerRef {
                            kind: "textbox",
                            control: control_index,
                            paragraph,
                            cell: None,
                        });
                        collect_from_paragraph(p, section, root_paragraph, &path, depth + 1, out);
                    }
                }
            }
            Control::Header(h) => {
                for (paragraph, p) in h.paragraphs.iter().enumerate() {
                    let mut path = container_path.to_vec();
                    path.push(TableContainerRef {
                        kind: "header",
                        control: control_index,
                        paragraph,
                        cell: None,
                    });
                    collect_from_paragraph(p, section, root_paragraph, &path, depth + 1, out);
                }
            }
            Control::Footer(f) => {
                for (paragraph, p) in f.paragraphs.iter().enumerate() {
                    let mut path = container_path.to_vec();
                    path.push(TableContainerRef {
                        kind: "footer",
                        control: control_index,
                        paragraph,
                        cell: None,
                    });
                    collect_from_paragraph(p, section, root_paragraph, &path, depth + 1, out);
                }
            }
            Control::Footnote(f) => {
                for (paragraph, p) in f.paragraphs.iter().enumerate() {
                    let mut path = container_path.to_vec();
                    path.push(TableContainerRef {
                        kind: "footnote",
                        control: control_index,
                        paragraph,
                        cell: None,
                    });
                    collect_from_paragraph(p, section, root_paragraph, &path, depth + 1, out);
                }
            }
            Control::Endnote(e) => {
                for (paragraph, p) in e.paragraphs.iter().enumerate() {
                    let mut path = container_path.to_vec();
                    path.push(TableContainerRef {
                        kind: "endnote",
                        control: control_index,
                        paragraph,
                        cell: None,
                    });
                    collect_from_paragraph(p, section, root_paragraph, &path, depth + 1, out);
                }
            }
            _ => {}
        }
    }
}

/// 문서의 모든 표를 격자로 추출한다.
///
/// 본문·글상자·머리말/꼬리말·각주/미주를 재귀하며, 표 셀 안의 표는 부모 셀의
/// `nested` 안에 담긴다(최상위 목록에 중복되지 않는다).
pub fn extract_tables(doc: &Document) -> Vec<TableGrid> {
    let mut out = Vec::new();
    for (sec_idx, section) in doc.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            collect_from_paragraph(para, sec_idx, para_idx, &[], 0, &mut out);
        }
    }
    // 컨테이너 재귀로 수집 순서가 흔들릴 수 있으므로 최상위 index 를 다시 매긴다.
    for (i, t) in out.iter_mut().enumerate() {
        t.index = i;
    }
    out
}
