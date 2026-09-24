//! [#7288] 표 «쪽 경계에서» 값 1 «셀 단위로 나눔» — 행이 원자 단위다.
//!
//! # 무엇이 깨져 있었나
//!
//! 조판 엔진이 이 속성을 읽지 않아, 값 1 표도 값 2 «나눔» 과 똑같이 행 **안**까지 잘랐다.
//! 값 0 은 [#7346] 에서 바로잡았고 값 1 이 남아 있었다.
//!
//! # 기대값의 출처 — 한/글 정본 실측 둘
//!
//! **편람** `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf` 158→159쪽: 40행 2열
//! `pageBreak="TABLE"`(= HWP5 값 1) 표에서 158쪽 마지막 행이 아래 괘선까지 닫힌 채 끝나고
//! 그 아래로 빈 공간을 남긴다. 159쪽은 반복 제목 줄 뒤 **새 행**("국토교통부(35)")으로
//! 시작한다 — 다음 행이 남은 공간에 안 들어가자 행을 통째로 넘겼다.
//!
//! **IR 경진대회** `pdf/156482639_startup_ir_contest-2020.pdf` 5→6쪽: 12행 4열 값 1 표에서
//! 5쪽이 항목 7 을 끝내고, 6쪽이 반복 제목 줄 뒤 `8 안산 (주)오토노미아` 라는 **새 행**으로
//! 시작한다. 종전 rhwp 는 같은 자리를 `기술을 보유하고 있음` 이라는 **문장 한가운데**에서
//! 이었다. 같은 문서의 쪽별 글자 수는 이 수정 뒤 정본과 10쪽 전부 일치한다
//! (`issue_7288_issue6132_matches_hancom_oracle`).
//!
//! # 대조군
//!
//! 값 2 «나눔» 은 같은 입력에서 행 **안**을 자른다(편람 정본 315→316쪽이 문장 한가운데를
//! 자른다). 두 값이 같은 입력에서 갈리는 것이 이 계약의 핵심이라, 한쪽만 검사하지 않는다.
//!
//! # 입력
//!
//! 합성 문서다. 신고자의 재현(`HwpDocument.createEmpty()` + 앞 빈 문단 28개 + 2행 1열 표)
//! 을 네이티브 경로로 옮긴 것이며, 문단 줄 정보는 rhwp 가 만든 것이지 한/글 저장본이 아니다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const LEAD: usize = 28;
const CELL_PROPS: &str = r#"{"height":1875}"#;

fn build(page_break: u8, lines: usize) -> DocumentCore {
    let mut doc = DocumentCore::new_empty();
    {
        let mut section = rhwp::model::document::Section::default();
        section.section_def.page_def = rhwp::model::page::PageDef::a4_default();
        section
            .paragraphs
            .push(rhwp::model::paragraph::Paragraph::new_empty());
        let mut document = rhwp::model::document::Document::default();
        document.sections.push(section);
        doc.set_document(document);
    }
    for _ in 0..LEAD {
        doc.insert_paragraph_native(0, 0).expect("앞 빈 문단");
    }
    doc.create_table_native(0, LEAD, 0, 2, 1)
        .expect("2행 1열 표");
    for cell in 0..2 {
        doc.set_cell_properties_native(0, LEAD, 0, cell, CELL_PROPS)
            .expect("셀 선언 높이");
    }
    doc.set_table_properties_native(0, LEAD, 0, &format!(r#"{{"pageBreak":{page_break}}}"#))
        .expect("«쪽 경계에서» 설정");
    for i in (1..=lines).rev() {
        doc.insert_text_in_cell_native(
            0,
            LEAD,
            0,
            0,
            0,
            0,
            &format!("줄{i} 한글 본문 내용입니다 길게 씁니다.\r"),
        )
        .expect("첫 행 칸 텍스트");
    }
    doc
}

fn collect(node: &RenderNode, page: usize, out: &mut Vec<(usize, u16)>) {
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        out.push((page, tc.row));
    }
    for child in &node.children {
        collect(child, page, out);
    }
}

/// 행 `row` 가 놓인 쪽 번호 집합.
fn pages_of_row(doc: &DocumentCore, row: u16) -> Vec<usize> {
    let mut all = Vec::new();
    for page in 0..doc.page_count() as usize {
        if let Ok(tree) = doc.build_page_render_tree(page as u32) {
            collect(&tree.root, page, &mut all);
        }
    }
    let mut pages: Vec<usize> = all
        .into_iter()
        .filter(|(_, r)| *r == row)
        .map(|(p, _)| p)
        .collect();
    pages.sort_unstable();
    pages.dedup();
    pages
}

/// **값 1 «셀 단위로 나눔»** 은 행 안을 자르지 않고, **값 2 «나눔»** 은 자른다.
///
/// 같은 입력에서 두 값이 갈리는 것이 이 계약이다 — 한쪽만 보면 과적용도 미적용도 못 잡는다.
#[test]
fn cell_unit_keeps_the_row_whole_where_split_mode_cuts_it() {
    let cell_unit = pages_of_row(&build(1, 100), 0);
    assert_eq!(
        cell_unit.len(),
        1,
        "«셀 단위로 나눔» 표는 행 0 을 행 **안**에서 자르면 안 된다 — 행이 원자 단위다. \
         행 0 이 놓인 쪽: {cell_unit:?}"
    );

    let split_mode = pages_of_row(&build(2, 100), 0);
    assert!(
        split_mode.len() >= 2,
        "같은 입력에서 «나눔» 표는 행 0 을 잘라 두 쪽에 걸쳐야 한다 — 값 1 의 계약을 \
         값 2 로 넓히면 안 된다. 행 0 이 놓인 쪽: {split_mode:?}"
    );
}

/// 행 경계 분할 자체는 일어난다 — 값 1 은 «나누지 않음» 이 아니다.
#[test]
fn cell_unit_still_breaks_at_row_boundaries() {
    let doc = build(1, 100);
    let row0 = pages_of_row(&doc, 0);
    let row1 = pages_of_row(&doc, 1);
    assert!(
        !row0.is_empty() && !row1.is_empty(),
        "표 조각을 찾지 못했다 — 시험 설정 오류. row0={row0:?} row1={row1:?}"
    );
    assert_ne!(
        row0, row1,
        "«셀 단위로 나눔» 표는 행 경계에서는 나뉘어야 한다 — 값 0 의 계약을 여기까지 \
         넓히면 안 된다. row0={row0:?} row1={row1:?}"
    );
}
