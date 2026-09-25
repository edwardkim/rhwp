//! [#7288] 저장 프레임의 초과 허용은 **그 내용에 대한 증거**일 때만 준다.
//!
//! # 무엇이 깨져 있나
//!
//! 행 스캔의 `source_frame_whole_row_fits` 는 저장된 첫 조각 프레임이 있으면 그 프레임이
//! 허용하는 만큼 예산을 넘겨 행 하나를 통째로 받는다([#5057]). 그 프레임의 높이는
//! **선언 개체 높이**(`table.common.height`)에서 온다.
//!
//! 그런데 그 갈래에는 선언을 어디까지 믿을지의 상한이 없다. 선언이 실제 측정보다 크게
//! 작으면 그 프레임은 이 내용의 기록이 아닌데도 초과 수용의 근거가 된다.
//!
//! ```text
//!   선언 개체 높이   53.8px
//!   측정 첫 행       515.0px      ← 9.6배
//!   잔여 밴드        335.5px
//!   종전 결과        행을 통째로 얹어 y=670..1185 — 지면(1009px) 밖 176px
//! ```
//!
//! # 기대값의 출처
//!
//! 같은 「선언을 얼마나 믿을까」 질문에 이 파일이 아니라 **엔진이 이미 답을 갖고 있다** —
//! [#3236] 의 `SINGLE_ROW_DECLARED_TRUST_MAX_RATIO`(1.5배)다. 측정이 선언의 이 배율을
//! 넘으면 글꼴 대체 팽창이 아니라 내용이 진짜로 큰 것이므로 선언 특례를 적용하지 않고
//! 인트라-로우 분할에 맡긴다는 규칙이고, 이미 네 곳의 선언-신뢰 경로가 이 상한을 쓴다.
//! 저장 프레임 초과 허용만 빠져 있었다.
//!
//! 값 2 «나눔» 은 행 내부까지 나누는 값이다 — 한컴 정본 실측(`pdf/2025 행정업무운영
//! 편람(최종)-hwp-2020.pdf` 315→316쪽이 문장 한가운데를 자른다). 그러므로 잔여 밴드에
//! 안 들어가는 행은 **그 자리에서 잘라야** 하고, 통째로 얹어 지면 밖으로 넘기면 안 된다.
//!
//! # 입력
//!
//! 합성 문서다. 신고자의 재현(`HwpDocument.createEmpty()` + 앞 빈 문단 28개 + 2행 1열 표,
//! `pageBreak=2`, 첫 행에 60줄)을 네이티브 경로로 옮긴 것이다. 문단 줄 정보는 rhwp 가
//! 편집 중 만든 것이지 한/글 저장본이 아니다 — 이 시험이 잠그는 계약이 바로 "그런 줄
//! 정보가 준 프레임을 초과 수용의 근거로 쓰지 않는다"이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 표를 쪽 아래쪽에서 시작시키는 앞 빈 문단 수.
const LEAD: usize = 28;
/// 셀 선언 높이 25pt = 1875 HWPUNIT — 내용보다 크게 작은 값이다.
const CELL_PROPS: &str = r#"{"height":1875}"#;
/// 쪽 바닥 판정 허용치.
const BOTTOM_TOL_PX: f64 = 2.0;

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

fn body_bottom_px(doc: &DocumentCore) -> f64 {
    let pd = &doc.document().sections[0].section_def.page_def;
    f64::from(pd.height - pd.margin_bottom - pd.margin_footer) / 75.0
}

fn collect(node: &RenderNode, page: usize, out: &mut Vec<(usize, u16, f64, f64)>) {
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        out.push((page, tc.row, node.bbox.y, node.bbox.y + node.bbox.height));
    }
    for child in &node.children {
        collect(child, page, out);
    }
}

fn frags(doc: &DocumentCore) -> Vec<(usize, u16, f64, f64)> {
    let mut out = Vec::new();
    for page in 0..doc.page_count() as usize {
        if let Ok(tree) = doc.build_page_render_tree(page as u32) {
            collect(&tree.root, page, &mut out);
        }
    }
    out
}

fn describe(fs: &[(usize, u16, f64, f64)]) -> String {
    fs.iter()
        .map(|(p, r, t, b)| format!("r{r}/p{p} y={t:.0}..{b:.0}"))
        .collect::<Vec<_>>()
        .join(" | ")
}

/// 선언보다 훨씬 큰 행은 저장 프레임을 근거로 통째로 얹지 않는다 — 지면 안에서 자른다.
#[test]
fn an_oversized_row_is_cut_instead_of_riding_the_declared_frame() {
    let doc = build(2, 60);
    let fs = frags(&doc);
    assert!(!fs.is_empty(), "표 조각을 찾지 못했다 — 시험 설정 오류");

    let bottom = body_bottom_px(&doc);
    let over: Vec<(usize, u16, f64, f64)> = fs
        .iter()
        .copied()
        .filter(|(_, _, _, b)| *b > bottom + BOTTOM_TOL_PX)
        .collect();
    assert!(
        over.is_empty(),
        "선언 개체 높이(53.8px)가 측정(515.0px)의 1/9 인데 그 프레임을 믿어 행을 통째로 \
         얹었다 — 지면(바닥 {bottom:.0}px) 밖으로 넘쳤다. 넘친 조각: {} / 전체: {}",
        describe(&over),
        describe(&fs)
    );
}

/// 대조군 — 선언이 측정과 맞는 작은 표는 종전대로 한 쪽에 통째로 놓인다.
#[test]
fn a_table_whose_declared_height_matches_is_unchanged() {
    let doc = build(2, 2);
    let fs = frags(&doc);
    assert!(!fs.is_empty(), "표 조각을 찾지 못했다 — 시험 설정 오류");

    let mut pages: Vec<usize> = fs.iter().map(|(p, _, _, _)| *p).collect();
    pages.sort_unstable();
    pages.dedup();
    assert_eq!(
        pages.len(),
        1,
        "선언과 측정이 맞는 표는 쪼개지면 안 된다. 조각: {}",
        describe(&fs)
    );
}
