//! [#6761] 저장 사다리가 표의 한 행을 **첫 줄 뒤에서** 나눈 자리를 25px 고아 기준이 버린다.
//!
//! ## 무엇이 문제였나
//!
//! `1480000-201900042` 의 `<표 2-5> 어린이보호에 대한 경고문구 표시 사례`(RowBreak, 15행)는
//! 한 쪽 끝에서 r=3 `시아노아크릴레이트 함유 혼합물` 행을 만난다. 한컴 2010 저장본은 그 행의
//! 두 칸 모두 **첫 줄만 이 쪽에 두고 둘째 줄을 새 쪽 원점에서** 재개했다고 적는다.
//!
//! ```text
//!   셀[8] r=3,c=1  p[0] ls[0] vpos=0  ls[1] vpos=0
//!   셀[9] r=3,c=2  p[0] ls[0] vpos=0  ls[1] vpos=0
//!                  p[1] ls[0] vpos=1320 = 1100 + 220   ← 되감긴 줄에서 한 줄 전진
//! ```
//!
//! rhwp 의 컷도 정확히 그 자리(`end_cut=[1,1]`, 17.6px)에 떨어지지만 25px 고아 기준이 이를
//! 기각해 행 전체를 다음 쪽으로 넘겼다. 그 쪽의 조각이 한 줄만큼 길어져 마지막 문단
//! `하나 이상의 피부과민성물질…` 이 본문 바닥(1028.0px)을 13px 넘었다.
//!
//! ## 기대값의 독립 근거
//!
//! - 저장 사다리: 위의 셀 되감김(`rhwp dump -s 1 -p 199`)
//! - 한컴 정본 `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` 39쪽 마지막
//!   행이 `시아노아크릴레이트 | – 순식간에 피부 및 안구가 접착됨. 어` 이고 40쪽이
//!   `함유 혼합물 | 린이 손에 닿게 하지 마시오.` 로 시작한다
//!
//! ## 수동 변형의 기대값 재검토
//!
//! 후속 저장 줄을 80HU 옮기거나 뒤에 0 원점 문단을 추가한 두 변형도 독립
//! 한컴 2020 Print는 첫 줄을 표의 시작 쪽에 남긴다. 사다리 확인이 깨지면 행 전체를
//! 이월해야 한다는 종전 가정은 기각한다. 원본/변형 전쪽 Native/fresh WASM은
//! 최저 90.47526%이며 검사는 행 소유·셀 안 포함·원본 내용의 순서/완전 보존을 확인한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";
/// r=3 첫 칸의 글자. 행을 나누면 앞 조각에 `시아노아크릴레이트`, 뒤 조각에 `함유 혼합물` 이 간다.
const ROW_HEAD: &str = "시아노아크릴레이트";
/// 표가 시작하는 쪽에만 있는 캡션.
const CAPTION: &str = "어린이보호에대한경고문구";

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn first_page_with_all(core: &DocumentCore, needles: &[&str]) -> u32 {
    (0..core.page_count() as u32)
        .find(|page| {
            let text = flat(&core.extract_page_text_native(*page).unwrap_or_default());
            needles.iter().all(|needle| text.contains(needle))
        })
        .unwrap_or_else(|| panic!("{needles:?} 를 모두 담은 쪽이 없다"))
}

fn first_page_with(core: &DocumentCore, needle: &str) -> u32 {
    first_page_with_all(core, &[needle])
}

/// 표가 시작하는 쪽. 캡션은 앞쪽 표 차례에도 있으므로 첫 행 `EU CLP` 와 함께 찾는다.
fn table_start_page(core: &DocumentCore) -> u32 {
    first_page_with_all(core, &[CAPTION, "EUCLP"])
}

/// 본문(`Body`) 아래 노드의 최대 하단. 꼬리말 줄은 본문 밖이라 세지 않는다.
fn body_bottom(node: &RenderNode, in_body: bool) -> f64 {
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    let own = if in_body && matches!(node.node_type, RenderNodeType::TextLine(_)) {
        node.bbox.y + node.bbox.height
    } else {
        f64::MIN
    };
    node.children
        .iter()
        .map(|child| body_bottom(child, in_body))
        .fold(own, f64::max)
}

/// 저장 사다리가 적은 자리에서 행을 나눈다 — 첫 줄이 표의 첫 쪽에 남는다.
#[test]
fn stored_first_line_split_keeps_row_head_on_table_page() {
    let core = open();
    let table_page = table_start_page(&core);
    let head_page = first_page_with(&core, ROW_HEAD);
    assert_eq!(
        head_page,
        table_page,
        "`{ROW_HEAD}` 행 첫 줄이 {}쪽에 있다 — 저장 되감김(ls[1] vpos 0)과 정본 39쪽대로 표의 \
         첫 쪽({}쪽)에 남아야 한다. 25px 고아 기준이 17.6px 한 줄을 기각하면 행 전체가 넘어간다",
        head_page + 1,
        table_page + 1
    );
    let next = core
        .extract_page_text_native(table_page + 1)
        .unwrap_or_default();
    assert!(
        flat(&next).starts_with(&flat("국외 관련 제도"))
            && flat(&next).contains(&flat("함유 혼합물")),
        "다음 쪽이 반복 제목행 뒤 `함유 혼합물` 조각으로 이어져야 한다: {:?}",
        next.chars().take(80).collect::<String>()
    );
}

/// 이어받은 쪽의 마지막 문단이 본문 바닥 안에 든다.
#[test]
fn continuation_page_stays_inside_body() {
    let core = open();
    let page = table_start_page(&core) + 1;
    let tree = core.build_page_render_tree(page).expect("render tree");
    let bottom = body_bottom(&tree.root, false);
    fn find_body(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            return Some(node);
        }
        node.children.iter().find_map(find_body)
    }
    let body = find_body(&tree.root).expect("실제 Body 상자");
    let limit = body.bbox.y + body.bbox.height;
    assert!(
        bottom <= limit + 0.5,
        "{}쪽 본문 마지막 줄은 실제 Body 안에 있어야 한다: {bottom} > {limit}",
        page + 1
    );
}

/// 저장 전진 확인의 변경이 실제로 수용 가능한 행 첫 줄을 강제로 이월하지 않는다.
#[test]
fn changed_later_line_preserves_the_physically_fitting_row_head() {
    let mut core = open();
    let mut doc = core.document().clone();
    let mut touched = 0;
    for section in &mut doc.sections {
        for paragraph in &mut section.paragraphs {
            for control in &mut paragraph.controls {
                let Control::Table(table) = control else {
                    continue;
                };
                for cell in table.cells.iter_mut().filter(|cell| cell.row == 3) {
                    let is_response = cell
                        .paragraphs
                        .first()
                        .is_some_and(|p| flat(&p.text).starts_with(&flat("- 순식간에")));
                    if !is_response {
                        continue;
                    }
                    for seg in &mut cell.paragraphs[1].line_segs {
                        seg.vertical_pos += 80;
                        touched += 1;
                    }
                }
            }
        }
    }
    assert_eq!(touched, 2, "대상 셀[9] p[1] 의 저장 줄 2개를 찾지 못했다");
    core.set_document(doc);
    assert_response_content_and_fragment_owners(&core);
}

/// 뒤의 확인 문단 추가는 독립 Print와 같은 앞 조각을 유지하며 추가 내용도 전부 보존한다.
#[test]
fn appended_rewind_preserves_row_ownership_and_all_response_paragraphs() {
    let mut core = open();
    let mut doc = core.document().clone();
    let mut touched = 0;
    for section in &mut doc.sections {
        for paragraph in &mut section.paragraphs {
            for control in &mut paragraph.controls {
                let Control::Table(table) = control else {
                    continue;
                };
                for cell in table.cells.iter_mut().filter(|cell| cell.row == 3) {
                    let is_response = cell
                        .paragraphs
                        .first()
                        .is_some_and(|p| flat(&p.text).starts_with(&flat("- 순식간에")));
                    if !is_response {
                        continue;
                    }
                    for seg in &mut cell.paragraphs[1].line_segs {
                        seg.vertical_pos += 80;
                    }
                    let mut rewound = cell.paragraphs[0].clone();
                    for seg in &mut rewound.line_segs {
                        seg.vertical_pos = 0;
                    }
                    let mut next = rewound.clone();
                    next.line_segs[0].vertical_pos = 1320;
                    cell.paragraphs.push(rewound);
                    cell.paragraphs.push(next);
                    touched += 1;
                }
            }
        }
    }
    assert_eq!(touched, 1, "대상 응답 셀을 찾지 못했다");
    core.set_document(doc);
    assert_response_content_and_fragment_owners(&core);
}

/// 독립 Print에서 행 첫 줄은 표의 시작 쪽에 남고, 다음 쪽이 같은 셀을 이어받는다.
/// 저장 사다리 후보의 변경과 실제 공간에 들어가는 내용의 소유는 구분한다.
fn assert_response_content_and_fragment_owners(core: &DocumentCore) {
    let mut owners = Vec::new();
    for (section_index, section) in core.document().sections.iter().enumerate() {
        for (para_index, paragraph) in section.paragraphs.iter().enumerate() {
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                let Control::Table(table) = control else {
                    continue;
                };
                for (cell_index, cell) in table.cells.iter().enumerate() {
                    if cell.row == 3
                        && cell
                            .paragraphs
                            .first()
                            .is_some_and(|p| flat(&p.text).starts_with(&flat("- 순식간에")))
                    {
                        owners.push((
                            section_index,
                            para_index,
                            control_index,
                            cell_index,
                            flat(
                                &cell
                                    .paragraphs
                                    .iter()
                                    .map(|p| p.text.as_str())
                                    .collect::<String>(),
                            ),
                        ));
                    }
                }
            }
        }
    }
    assert_eq!(owners.len(), 1, "검증할 원본 응답 셀은 하나다");
    let (section, para, control, cell_index, expected) = &owners[0];
    let table_page = table_start_page(core);
    assert_eq!(
        first_page_with(core, ROW_HEAD),
        table_page,
        "독립 Print와 같이 행 첫 줄은 표의 시작 쪽에 남아야 한다"
    );
    let mut actual = String::new();
    let mut cell_pages = Vec::new();
    fn text(node: &RenderNode, cell: &RenderNode, out: &mut String) {
        if !node.visible || node.editor_only {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            assert!(
                node.bbox.y + 0.5 >= cell.bbox.y
                    && node.bbox.y + node.bbox.height <= cell.bbox.y + cell.bbox.height + 0.5,
                "분할한 응답 줄은 배정된 셀 안에 있어야 한다"
            );
        }
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            text(child, cell, out);
        }
    }
    fn visit(
        node: &RenderNode,
        owner: (usize, usize, usize, usize),
        page: u32,
        out: &mut String,
        pages: &mut Vec<u32>,
    ) {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.section_index == Some(owner.0)
                && table.para_index == Some(owner.1)
                && table.control_index == Some(owner.2)
                && table.cell_context.is_none()
            {
                for child in &node.children {
                    if matches!(&child.node_type, RenderNodeType::TableCell(cell)
                        if cell.model_cell_index == Some(u32::try_from(owner.3).expect("모델 cell index")))
                    {
                        pages.push(page);
                        text(child, child, out);
                    }
                }
                return;
            }
        }
        for child in &node.children {
            visit(child, owner, page, out, pages);
        }
    }
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .expect("검증할 실제 render tree");
        visit(
            &tree.root,
            (*section, *para, *control, *cell_index),
            page,
            &mut actual,
            &mut cell_pages,
        );
    }
    assert_eq!(
        cell_pages,
        vec![table_page, table_page + 1],
        "응답 셀은 앞 조각과 이어받기 조각에 한 번씩 배정돼야 한다"
    );
    assert_eq!(
        flat(&actual),
        *expected,
        "원본 셀의 모든 문단을 순서대로 보존하며 누락·중복하지 않아야 한다"
    );
    assert!(
        flat(&core.extract_page_text_native(table_page + 1).unwrap())
            .contains(&flat("함유 혼합물")),
        "다음 쪽은 왼쪽 셀의 남은 내용도 보존해야 한다"
    );
}
