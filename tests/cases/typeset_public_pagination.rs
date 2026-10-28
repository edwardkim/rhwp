//! 공개 조판 API의 빈 문서·문단 보존·강제 쪽 나누기 계약.

use rhwp::model::page::{ColumnDef, PageDef};
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::renderer::composer::ComposedParagraph;
use rhwp::renderer::pagination::PageItem;
use rhwp::renderer::style_resolver::ResolvedStyleSet;
use rhwp::renderer::typeset::TypesetEngine;

fn a4_page_def() -> PageDef {
    PageDef {
        width: 59528,
        height: 84188,
        margin_left: 8504,
        margin_right: 8504,
        margin_top: 5669,
        margin_bottom: 4252,
        margin_header: 4252,
        margin_footer: 4252,
        margin_gutter: 0,
        ..Default::default()
    }
}

fn make_paragraph_with_height(line_height: i32) -> Paragraph {
    Paragraph {
        line_segs: vec![LineSeg {
            line_height,
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn test_typeset_empty_paragraphs() {
    let engine = TypesetEngine::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let composed: Vec<ComposedParagraph> = Vec::new();

    let result = engine.typeset_section(
        &[],
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
        &[],
        false,
        &std::collections::HashSet::new(),
    );

    assert_eq!(result.pages.len(), 1, "빈 문서도 최소 1페이지");
}

#[test]
fn test_typeset_page_overflow() {
    let engine = TypesetEngine::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    // 빈 문단만으로는 마지막 blank-only page 제거 경로를 검증하게 된다. 이
    // 테스트의 계약은 overflow 중 가시 문단의 배치이므로, 실제 내용도 넣는다.
    let mut paras: Vec<Paragraph> = (0..100).map(|_| make_paragraph_with_height(2000)).collect();
    for (idx, para) in paras.iter_mut().enumerate() {
        para.text = format!("paragraph {idx}");
    }
    let composed: Vec<ComposedParagraph> = Vec::new();
    let page_def = a4_page_def();
    let col_def = ColumnDef::default();

    let new_result = engine.typeset_section(
        &paras,
        &composed,
        &styles,
        &page_def,
        &col_def,
        0,
        &[],
        false,
        &std::collections::HashSet::new(),
    );

    // `Paginator`의 옛 measured-height 근사는 독립적인 oracle이 아니다. 이
    // synthetic case는 가시 문단 100개의 전량 보존과 body-fit(4쪽)을 직접
    // 고정한다.
    assert_eq!(
        new_result.pages.len(),
        4,
        "100개 2000-HWPUNIT 문단은 A4 4쪽"
    );
    let placed: Vec<_> = new_result
        .pages
        .iter()
        .flat_map(|page| page.column_contents.iter())
        .flat_map(|column| column.items.iter())
        .map(PageItem::para_index)
        .collect();
    assert_eq!(
        placed,
        (0..100).collect::<Vec<_>>(),
        "문단 손실·중복 없이 배치"
    );
}

#[test]
fn test_typeset_force_break_before_hint() {
    let engine = TypesetEngine::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    // 한 페이지에 충분히 들어가는 3개 문단
    let mut paras = vec![
        make_paragraph_with_height(400),
        make_paragraph_with_height(400),
        make_paragraph_with_height(400),
    ];
    for (idx, para) in paras.iter_mut().enumerate() {
        para.text = format!("paragraph {idx}");
    }
    let composed: Vec<ComposedParagraph> = Vec::new();
    let page_def = a4_page_def();
    let col_def = ColumnDef::default();

    // hint 없음 → 1페이지
    let baseline = engine.typeset_section(
        &paras,
        &composed,
        &styles,
        &page_def,
        &col_def,
        0,
        &[],
        false,
        &std::collections::HashSet::new(),
    );
    assert_eq!(baseline.pages.len(), 1, "hint 없으면 3문단 모두 1페이지");

    // para_idx=1 에 force-break hint → para 1 이 2페이지에서 시작
    let mut hint = std::collections::HashSet::new();
    hint.insert(1usize);
    let reflowed = engine.typeset_section(
        &paras,
        &composed,
        &styles,
        &page_def,
        &col_def,
        0,
        &[],
        false,
        &hint,
    );
    assert_eq!(reflowed.pages.len(), 2, "para1 force-break 로 2페이지");
    let page0_paras: Vec<usize> = reflowed.pages[0]
        .column_contents
        .iter()
        .flat_map(|cc| cc.items.iter().map(|it| it.para_index()))
        .collect();
    let page1_paras: Vec<usize> = reflowed.pages[1]
        .column_contents
        .iter()
        .flat_map(|cc| cc.items.iter().map(|it| it.para_index()))
        .collect();
    assert_eq!(page0_paras, vec![0], "1페이지엔 para0 만");
    assert_eq!(page1_paras, vec![1, 2], "2페이지엔 para1,2");
}
