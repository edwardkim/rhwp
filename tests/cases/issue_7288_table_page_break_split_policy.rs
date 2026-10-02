//! [#7288] 표 «쪽 경계에서» 값 0 «나누지 않음» 을 조판이 지키게 한다.
//!
//! # 무엇이 깨져 있나
//!
//! 조판 엔진은 이 속성을 한 번도 읽지 않는다. 분할 가능 여부를 정하는 자리가
//! `typeset.rs` 의
//!
//! ```text
//! let can_intra_split = !mt.cells.is_empty();
//! ```
//!
//! 라서, 칸이 있기만 하면 **어떤 표든 행 내부까지** 자른다. 같은 함수에서 원자 단위
//! 높이를 정하는 `split_unit_h` 도 항상 첫 행이라, 「나누지 않음」 표에서 표 전체가
//! 단위여야 한다는 계약이 들어갈 자리가 없다.
//!
//! # 기대값의 출처 — 한/글 정본 실측
//!
//! **값 대응**: 한/글 13.0 이 같은 원문을 두 형식으로 저장한 쌍둥이
//! (`samples/2025 행정업무운영 편람(최종).hwp` / `.hwpx`) 에서 표별 분포가 양쪽 일치한다
//! (45 / 4 / 274) — HWP5 `HWPTAG_TABLE` bit 0~1 의 `0/1/2` 가 HWPX
//! `pageBreak="NONE"/"TABLE"/"CELL"` 에 대응한다.
//!
//! **값 0 의 뜻**: 저장소 정본 `pdf/text_footnote_tail_overpagination-2024.pdf` 의 62·63쪽이
//! 7×7 `pageBreak=NONE` 표(표 7.4-1 · 표 7.4-2)를 **각각 한 쪽에 통째로** 담는다. 62쪽은
//! 표 7.4-1 을 끝내고 표 7.4-2 의 캡션만 남긴 채 쪽이 끝난다 — 쪼개지 않고 넘긴다.
//! 정본 51문서 교차 확인에서도 본문 앵커로 쪽을 확정한 `NONE` 표 21건 전부 한 쪽 안이었고
//! 쪽을 넘긴 사례는 0건이다(같은 표본의 `CELL` 표는 33건이 쪽을 넘는다).
//!
//! 원자 단위 자체가 한 쪽보다 크면 나눈다. 그 경계는 여기서 검사하지 않는다.
//!
//! # 범위 — 무엇을 안 고쳤는지
//!
//! **값 1 «셀 단위로 나눔» 은 미해결로 남긴다.** 행 경계까지만 나누는 것이 맞다는 근거는
//! 있다(편람 정본 158→159쪽: 마지막 행이 괘선까지 닫히고 바닥에 빈 공간을 남긴 뒤, 다음
//! 쪽이 반복 제목 줄 뒤 새 행으로 시작). 그러나 그대로 적용하면
//! `samples/issue6132/156482639_startup_ir_contest.hwp` 가 10쪽 → 11쪽이 되는데, 그 문서의
//! 정본 PDF 가 없어 어느 쪽이 맞는지 가릴 수 없었다. 아래 `other_values_still_split` 이
//! 값 1·2 의 **종전 동작**을 잠가, 값 0 의 규칙이 그쪽으로 새지 않게 한다.
//!
//! **양수 세로 오프셋의 가시-host 자리차지 float 도 제외한다.** 그 갈래는 흐름을
//! 전진시키지 않고 배제 영역에만 밴드를 남기며 뒤 본문의 y 는 저장 사다리에서 온다.
//! 두 좌표계가 화해되지 않은 상태에서 표만 옮기면 글 위에 표가 그려진다
//! (`issue5941/1490000-201600081_roadmap_research.hwp` 149쪽 실측: 겹침 134 → 156).
//! `none_table_is_atomic_here` 의 주석과 같은 범위다.
//!
//! # 입력
//!
//! 합성 문서다. 신고자의 재현(`HwpDocument.createEmpty()` + 앞 빈 문단 28개 + 2행 1열 표)
//! 을 네이티브 경로로 옮긴 것이다. 문단 줄 정보는 rhwp 가 **합성**한 것이지 한/글 저장본이
//! 아니므로, 저장 LineSeg 재사용 경로의 계약은 여기서 검사하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 표를 쪽 아래쪽에서 시작시키는 앞 빈 문단 수.
const LEAD: usize = 28;
/// 셀 선언 높이 25pt = 1875 HWPUNIT.
const CELL_PROPS: &str = r#"{"height":1875}"#;

/// 한 쪽에 놓인 표 칸 조각.
#[derive(Debug, Clone, Copy)]
struct Frag {
    page: usize,
    row: u16,
    top: f64,
    bottom: f64,
}

/// 신고자의 재현 문서를 네이티브로 만든다.
///
/// `page_break` 는 `setTableProperties` 와 같은 정수 계약(0/1/2)이고,
/// `lines` 는 첫 행 칸에 넣을 문단 수다.
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

fn collect(node: &RenderNode, page: usize, out: &mut Vec<Frag>) {
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        out.push(Frag {
            page,
            row: tc.row,
            top: node.bbox.y,
            bottom: node.bbox.y + node.bbox.height,
        });
    }
    for child in &node.children {
        collect(child, page, out);
    }
}

fn frags(doc: &DocumentCore) -> Vec<Frag> {
    let mut out = Vec::new();
    for page in 0..doc.page_count() as usize {
        if let Ok(tree) = doc.build_page_render_tree(page as u32) {
            collect(&tree.root, page, &mut out);
        }
    }
    out
}

fn describe(fs: &[Frag]) -> String {
    fs.iter()
        .map(|f| format!("r{}/p{} y={:.0}..{:.0}", f.row, f.page, f.top, f.bottom))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// 표 조각이 놓인 쪽 번호 집합.
fn pages_of(fs: &[Frag], row: Option<u16>) -> Vec<usize> {
    let mut pages: Vec<usize> = fs
        .iter()
        .filter(|f| row.is_none_or(|r| f.row == r))
        .map(|f| f.page)
        .collect();
    pages.sort_unstable();
    pages.dedup();
    pages
}

/// **값 0 «나누지 않음»** — 남은 공간에 안 들어가면 표 전체가 다음 쪽으로 간다.
///
/// 60줄·100줄 첫 행 모두 앞 빈 문단 28개 뒤 잔여 공간에는 안 들어간다. 이때 표는
/// 쪼개지지 않고 통째로 이월돼야 한다 — 정본 62·63쪽의 7×7 `NONE` 표와 같은 계약이다.
#[test]
fn none_moves_the_whole_table_instead_of_splitting() {
    for lines in [60usize, 100] {
        let doc = build(0, lines);
        let fs = frags(&doc);
        assert!(
            !fs.is_empty(),
            "lines={lines}: 표 조각을 찾지 못했다 — 시험 설정 오류"
        );

        let pages = pages_of(&fs, None);
        assert_eq!(
            pages.len(),
            1,
            "lines={lines}: «나누지 않음» 표가 쪼개졌다 — 표 전체가 원자 단위여야 한다. \
             조각: {}",
            describe(&fs)
        );
    }
}

/// 값 0 의 규칙이 다른 값으로 새지 않는다 — 값 1·2 는 종전대로 나뉜다.
///
/// 값 1 «셀 단위로 나눔» 의 행 경계 계약은 이번 범위 밖이다(모듈 주석 참조). 여기서는
/// 두 값이 **여전히 쪼개진다**는 것만 잠가, 원자 규칙의 과적용을 막는다.
#[test]
fn other_values_still_split() {
    for page_break in [1u8, 2] {
        let doc = build(page_break, 100);
        let fs = frags(&doc);
        assert!(
            !fs.is_empty(),
            "pageBreak={page_break}: 표 조각을 찾지 못했다 — 시험 설정 오류"
        );
        let pages = pages_of(&fs, None);
        assert!(
            pages.len() >= 2,
            "pageBreak={page_break}: 이 값은 «나누지 않음» 이 아니다 — 쪽 경계에서 나뉘어야 \
             한다. 조각: {}",
            describe(&fs)
        );
    }
}

/// 대조군 — 한 쪽에 들어가는 표는 어떤 값에서도 쪼개지지 않는다.
#[test]
fn a_table_that_fits_is_never_split() {
    for page_break in [0u8, 1, 2] {
        let doc = build(page_break, 2);
        let fs = frags(&doc);
        assert!(
            !fs.is_empty(),
            "pageBreak={page_break}: 표 조각을 찾지 못했다 — 시험 설정 오류"
        );
        for row in 0..2u16 {
            let pages = pages_of(&fs, Some(row));
            assert!(
                pages.len() <= 1,
                "pageBreak={page_break}: 한 쪽에 들어가는 표의 행 {row} 이 {pages:?} 쪽에 \
                 걸쳤다. 조각: {}",
                describe(&fs)
            );
        }
    }
}
