use super::*;
use crate::model::page::{ColumnDef, PageDef};
use crate::model::paragraph::{LineSeg, Paragraph};

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
fn page_bottom_text_box_fit_keeps_line_even_when_advance_overflows() {
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let page_def = a4_page_def();
    let body_height_hu = page_def
        .height
        .saturating_sub(page_def.margin_top)
        .saturating_sub(page_def.margin_bottom)
        .saturating_sub(page_def.margin_header)
        .saturating_sub(page_def.margin_footer) as i32;

    let lead_advance = body_height_hu - 580;
    let lead = Paragraph {
        line_segs: vec![LineSeg {
            vertical_pos: 0,
            line_height: lead_advance,
            text_height: body_height_hu - 2500,
            line_spacing: 0,
            ..Default::default()
        }],
        ..Default::default()
    };
    let bottom_line = Paragraph {
        line_segs: vec![LineSeg {
            vertical_pos: body_height_hu - 1200,
            line_height: 1200,
            text_height: 1200,
            line_spacing: 840,
            ..Default::default()
        }],
        ..Default::default()
    };
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &[lead, bottom_line],
        &composed,
        &styles,
        &page_def,
        &ColumnDef::default(),
        0,
    );

    assert_eq!(result.pages.len(), 1);
    let items = &result.pages[0].column_contents[0].items;
    assert!(matches!(
        items.as_slice(),
        [
            PageItem::FullParagraph { para_index: 0 },
            PageItem::FullParagraph { para_index: 1 }
        ]
    ));
}

#[test]
fn page_bottom_empty_paragraph_before_vpos_reset_does_not_create_blank_page() {
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let page_def = a4_page_def();
    let body_height_hu = page_def
        .height
        .saturating_sub(page_def.margin_top)
        .saturating_sub(page_def.margin_bottom)
        .saturating_sub(page_def.margin_header)
        .saturating_sub(page_def.margin_footer) as i32;

    let lead = Paragraph {
        text: "앞쪽 본문".to_string(),
        line_segs: vec![LineSeg {
            vertical_pos: 0,
            line_height: body_height_hu - 100,
            text_height: body_height_hu - 100,
            line_spacing: 0,
            ..Default::default()
        }],
        ..Default::default()
    };
    let bottom_empty = Paragraph {
        line_segs: vec![LineSeg {
            vertical_pos: body_height_hu - 1200,
            line_height: 1000,
            text_height: 1000,
            line_spacing: 400,
            ..Default::default()
        }],
        ..Default::default()
    };
    let reset_next = Paragraph {
        text: "다음 쪽 본문".to_string(),
        line_segs: vec![LineSeg {
            vertical_pos: 0,
            line_height: 1000,
            text_height: 1000,
            line_spacing: 400,
            ..Default::default()
        }],
        ..Default::default()
    };

    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &[lead, bottom_empty, reset_next],
        &composed,
        &styles,
        &page_def,
        &ColumnDef::default(),
        0,
    );

    assert_eq!(result.pages.len(), 2);
    assert!(result.hidden_empty_paras.contains(&1));
    assert!(matches!(
        result.pages[0].column_contents[0].items.as_slice(),
        [PageItem::FullParagraph { para_index: 0 }]
    ));
    assert!(matches!(
        result.pages[1].column_contents[0].items.as_slice(),
        [PageItem::FullParagraph { para_index: 2 }]
    ));
}

#[test]
fn test_empty_paragraphs() {
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &[],
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );
    // 빈 문서도 최소 1페이지
    assert_eq!(result.pages.len(), 1);
}

#[test]
fn test_single_paragraph() {
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let paras = vec![make_paragraph_with_height(400)];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );
    assert_eq!(result.pages.len(), 1);
}

#[test]
fn test_page_overflow() {
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    // 본문 영역 높이를 넘는 많은 문단 생성
    let paras: Vec<Paragraph> = (0..100)
        .map(|_| make_paragraph_with_height(2000)) // 각 문단 약 26.7px
        .collect();
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );
    // 여러 페이지로 분할되어야 함
    assert!(!result.pages.is_empty());
}

#[test]
fn test_paginator_dpi() {
    let paginator = Paginator::new(72.0);
    assert!((paginator.dpi - 72.0).abs() < 0.01);
}

#[test]
fn test_table_page_split() {
    // 표가 페이지를 초과할 때 PartialTable로 분할되는지 테스트
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    // 짧은 텍스트 문단 + 큰 표 (4행, 각 행 높이 30000 HWPUNIT = ~400px → 총 ~1600px)
    let text_para = make_paragraph_with_height(1000);
    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(Table {
        row_count: 4,
        col_count: 2,
        cells: vec![
            Cell {
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 0,
                col: 1,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 1,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 1,
                col: 1,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 2,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 2,
                col: 1,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 3,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 3,
                col: 1,
                row_span: 1,
                col_span: 1,
                height: 30000,
                width: 5000,
                ..Default::default()
            },
        ],
        ..Default::default()
    })));

    let paras = vec![text_para, table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 표가 1페이지에 안 맞으므로 2페이지 이상이어야 함
    assert!(
        result.pages.len() >= 2,
        "표가 페이지를 넘어 분할되어야 함, pages={}",
        result.pages.len()
    );

    // PartialTable 항목이 존재하는지 확인
    let mut has_partial_table = false;
    for page in &result.pages {
        for col in &page.column_contents {
            for item in &col.items {
                if matches!(item, PageItem::PartialTable { .. }) {
                    has_partial_table = true;
                }
            }
        }
    }
    assert!(has_partial_table, "PartialTable 항목이 존재해야 함");
}

#[test]
fn test_table_fits_single_page() {
    // 표가 페이지에 들어가면 Table로 배치 (분할 안 됨)
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(Table {
        row_count: 2,
        col_count: 2,
        cells: vec![
            Cell {
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 2000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 0,
                col: 1,
                row_span: 1,
                col_span: 1,
                height: 2000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 1,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 2000,
                width: 5000,
                ..Default::default()
            },
            Cell {
                row: 1,
                col: 1,
                row_span: 1,
                col_span: 1,
                height: 2000,
                width: 5000,
                ..Default::default()
            },
        ],
        ..Default::default()
    })));

    let paras = vec![table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 1페이지에 들어가야 함
    assert_eq!(result.pages.len(), 1);
    // Table 항목이어야 함 (PartialTable 아님)
    let items = &result.pages[0].column_contents[0].items;
    assert!(
        matches!(items[0], PageItem::Table { .. }),
        "작은 표는 Table로 배치되어야 함"
    );
}

#[test]
fn test_table_split_with_repeat_header() {
    // repeat_header=true인 표가 분할될 때 is_continuation 확인
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(Table {
        row_count: 4,
        col_count: 1,
        repeat_header: true,
        cells: vec![
            Cell {
                row: 0,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 5000,
                width: 10000,
                is_header: true,
                ..Default::default()
            },
            Cell {
                row: 1,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 40000,
                width: 10000,
                ..Default::default()
            },
            Cell {
                row: 2,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 40000,
                width: 10000,
                ..Default::default()
            },
            Cell {
                row: 3,
                col: 0,
                row_span: 1,
                col_span: 1,
                height: 40000,
                width: 10000,
                ..Default::default()
            },
        ],
        ..Default::default()
    })));

    let paras = vec![table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 2페이지 이상으로 분할
    assert!(result.pages.len() >= 2, "큰 표가 분할되어야 함");

    // 두 번째 페이지의 PartialTable에 is_continuation=true 확인
    let mut found_continuation = false;
    for page in result.pages.iter().skip(1) {
        for col in &page.column_contents {
            for item in &col.items {
                if let PageItem::PartialTable {
                    is_continuation, ..
                } = item
                {
                    if *is_continuation {
                        found_continuation = true;
                    }
                }
            }
        }
    }
    assert!(
        found_continuation,
        "연속 페이지에 is_continuation=true인 PartialTable이 있어야 함"
    );
}

/// 여러 줄로 구성된 문단 생성 (줄 수, 줄당 높이 HWPUNIT)
fn make_multiline_paragraph(line_count: usize, line_height: i32) -> Paragraph {
    let line_segs: Vec<LineSeg> = (0..line_count)
        .map(|_| LineSeg {
            line_height,
            ..Default::default()
        })
        .collect();
    Paragraph {
        line_segs,
        ..Default::default()
    }
}

#[test]
fn test_partial_paragraph_split() {
    // 10줄 문단 (줄당 ~133px) → A4 본문 영역(~826px)에 ~6줄만 들어감
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let paras = vec![make_multiline_paragraph(10, 10000)]; // 10줄 x 10000 HWPUNIT
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 2페이지 이상으로 분할되어야 함
    assert!(
        result.pages.len() >= 2,
        "긴 문단이 2페이지 이상으로 분할되어야 함, pages={}",
        result.pages.len()
    );

    // PartialParagraph 항목이 존재하는지 확인
    let mut has_partial = false;
    let mut partial_ranges: Vec<(usize, usize)> = Vec::new();
    for page in &result.pages {
        for col in &page.column_contents {
            for item in &col.items {
                if let PageItem::PartialParagraph {
                    start_line,
                    end_line,
                    ..
                } = item
                {
                    has_partial = true;
                    partial_ranges.push((*start_line, *end_line));
                }
            }
        }
    }
    assert!(has_partial, "PartialParagraph 항목이 존재해야 함");

    // 첫 파트는 start_line=0이어야 함
    assert_eq!(partial_ranges[0].0, 0, "첫 파트 start_line은 0이어야 함");

    // 파트가 연속적이어야 함 (이전 end_line == 다음 start_line)
    for i in 1..partial_ranges.len() {
        assert_eq!(
            partial_ranges[i - 1].1,
            partial_ranges[i].0,
            "파트 {}의 end_line({})이 파트 {}의 start_line({})과 일치해야 함",
            i - 1,
            partial_ranges[i - 1].1,
            i,
            partial_ranges[i].0,
        );
    }

    // 마지막 파트의 end_line은 전체 줄 수(10)여야 함
    assert_eq!(
        partial_ranges.last().unwrap().1,
        10,
        "마지막 파트 end_line은 전체 줄 수(10)여야 함"
    );
}

#[test]
fn test_short_paragraph_no_split() {
    // 1줄 짧은 문단은 FullParagraph로 유지
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let paras = vec![make_paragraph_with_height(400)];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    assert_eq!(result.pages.len(), 1);
    let items = &result.pages[0].column_contents[0].items;
    assert!(
        matches!(items[0], PageItem::FullParagraph { .. }),
        "짧은 문단은 FullParagraph여야 함"
    );
}

#[test]
fn test_partial_paragraph_multi_page_span() {
    // 30줄 문단이 3페이지 이상에 걸치는지 확인
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let paras = vec![make_multiline_paragraph(30, 10000)]; // 30줄 x ~133px
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    assert!(
        result.pages.len() >= 3,
        "30줄 문단이 3페이지 이상이어야 함, pages={}",
        result.pages.len()
    );
}

#[test]
fn test_partial_paragraph_after_content() {
    // 기존 콘텐츠 뒤에 긴 문단이 올 때 올바르게 분할
    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let paras = vec![
        make_multiline_paragraph(3, 10000), // 짧은 문단 (3줄 x ~133px = ~400px)
        make_multiline_paragraph(10, 10000), // 긴 문단 (10줄 x ~133px)
    ];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 2페이지 이상
    assert!(result.pages.len() >= 2, "긴 문단이 페이지를 넘어야 함");

    // 첫 페이지에 첫 문단 FullParagraph + 두번째 문단 PartialParagraph
    let page1_items = &result.pages[0].column_contents[0].items;
    assert!(
        matches!(page1_items[0], PageItem::FullParagraph { para_index: 0 }),
        "첫 문단은 FullParagraph여야 함"
    );

    let has_partial_on_page1 = page1_items
        .iter()
        .any(|item| matches!(item, PageItem::PartialParagraph { para_index: 1, .. }));
    assert!(
        has_partial_on_page1,
        "첫 페이지에 두번째 문단의 PartialParagraph가 있어야 함"
    );
}

/// 셀 내용이 포함된 CellBreak 표 생성 헬퍼
fn make_cellbreak_table(
    row_count: u16,
    col_count: u16,
    cell_height: u32,
) -> crate::model::table::Table {
    use crate::model::paragraph::LineSeg;
    use crate::model::table::{Cell, Table, TablePageBreak};

    let mut cells = Vec::new();
    for r in 0..row_count {
        for c in 0..col_count {
            // 각 셀에 여러 줄의 문단을 넣어 높이를 키움
            let line_count = (cell_height / 1000) as usize;
            let line_segs: Vec<LineSeg> = (0..line_count.max(1))
                .map(|_| LineSeg {
                    line_height: 1000,
                    ..Default::default()
                })
                .collect();
            let para = Paragraph {
                line_segs,
                ..Default::default()
            };
            cells.push(Cell {
                row: r,
                col: c,
                row_span: 1,
                col_span: 1,
                height: cell_height,
                width: 5000,
                paragraphs: vec![para],
                ..Default::default()
            });
        }
    }
    Table {
        row_count,
        col_count,
        cells,
        page_break: TablePageBreak::CellBreak,
        ..Default::default()
    }
}

#[test]
#[ignore = "Task #993: 레거시 Paginator(engine.rs)는 분할 표 컷을 생산하지 않음 — TypesetEngine 컷 모델로 대체. advance_row_cut 단위 테스트(table_layout.rs row_cut_tests)가 인트라-로우 분할을 검증."]
fn test_table_cell_break_intra_row_split() {
    // CellBreak 표: 행이 페이지보다 크면 인트라-로우 분할 발생
    use crate::model::control::Control;

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    // 1행 2열, 셀 높이 80000 HWPUNIT (>> A4 본문 ~60000 HWPUNIT)
    let table = make_cellbreak_table(1, 2, 80000);
    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(table)));

    let paras = vec![table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 2페이지 이상이어야 함
    assert!(
        result.pages.len() >= 2,
        "CellBreak 큰 행이 분할되어야 함, pages={}",
        result.pages.len()
    );

    // split_start_content_offset 또는 split_end_content_limit > 0인 PartialTable 존재 확인
    let mut has_intra_split = false;
    for page in &result.pages {
        for col in &page.column_contents {
            for item in &col.items {
                if let PageItem::PartialTable {
                    start_cut, end_cut, ..
                } = item
                {
                    if !start_cut.is_empty() || !end_cut.is_empty() {
                        has_intra_split = true;
                    }
                }
            }
        }
    }
    assert!(
        has_intra_split,
        "CellBreak 표에 인트라-로우 분할이 발생해야 함"
    );
}

#[test]
#[ignore = "Task #993: 레거시 Paginator(engine.rs)는 분할 표 컷을 생산하지 않음 — TypesetEngine 컷 모델로 대체. advance_row_cut 단위 테스트(table_layout.rs row_cut_tests)가 인트라-로우 분할을 검증."]
fn test_table_none_also_intra_row_split() {
    // page_break=None 표도 인트라-로우 분할 적용 (모든 표에 적용)
    use crate::model::control::Control;
    use crate::model::table::TablePageBreak;

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    // 1행 2열, 셀 높이 80000 HWPUNIT (>> A4 본문)
    let mut table = make_cellbreak_table(1, 2, 80000);
    table.page_break = TablePageBreak::None;

    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(table)));

    let paras = vec![table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 2페이지 이상이어야 함
    assert!(
        result.pages.len() >= 2,
        "None 표도 큰 행이 분할되어야 함, pages={}",
        result.pages.len()
    );

    // 인트라-로우 분할이 발생해야 함
    let mut has_intra_split = false;
    for page in &result.pages {
        for col in &page.column_contents {
            for item in &col.items {
                if let PageItem::PartialTable {
                    start_cut, end_cut, ..
                } = item
                {
                    if !start_cut.is_empty() || !end_cut.is_empty() {
                        has_intra_split = true;
                    }
                }
            }
        }
    }
    assert!(
        has_intra_split,
        "None 표에도 인트라-로우 분할이 발생해야 함"
    );
}

#[test]
#[ignore = "Task #993: 레거시 Paginator(engine.rs)는 분할 표 컷을 생산하지 않음 — TypesetEngine 컷 모델로 대체. advance_row_cut 단위 테스트(table_layout.rs row_cut_tests)가 인트라-로우 분할을 검증."]
fn test_table_cell_break_multi_page_row() {
    // CellBreak: 하나의 행이 3페이지 이상에 걸치는 경우
    use crate::model::control::Control;

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    // 1행 1열, 셀 높이 200000 HWPUNIT (~3페이지 분량)
    let table = make_cellbreak_table(1, 1, 200000);
    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(table)));

    let paras = vec![table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 3페이지 이상
    assert!(
        result.pages.len() >= 3,
        "200000 HWPUNIT 행이 3+페이지에 걸쳐야 함, pages={}",
        result.pages.len()
    );

    // [Task #993] 시작 컷이 단조 전진하는지 확인 — 컷 합(셀별 소비 유닛 수의 합)
    // 을 진행도 지표로 사용. 첫 페이지=0(빈 컷), 이후 증가.
    let mut progress: Vec<usize> = Vec::new();
    for page in &result.pages {
        for col in &page.column_contents {
            for item in &col.items {
                if let PageItem::PartialTable { start_cut, .. } = item {
                    progress.push(start_cut.iter().sum());
                }
            }
        }
    }

    // 첫 페이지: 컷 합=0, 이후 페이지: 컷 합 > 0 증가
    if progress.len() >= 2 {
        assert_eq!(progress[0], 0, "첫 페이지 시작 컷은 비어 있어야 함");
        for i in 1..progress.len() {
            assert!(
                progress[i] > 0,
                "{}번째 페이지 시작 컷은 0보다 커야 함: {}",
                i + 1,
                progress[i]
            );
            if i >= 2 {
                assert!(
                    progress[i] > progress[i - 1],
                    "시작 컷이 증가해야 함: {} > {}",
                    progress[i],
                    progress[i - 1]
                );
            }
        }
    }
}

// ─── 타스크 198: 표 페이지 경계 분할 검증 테스트 ───

/// 10행 표가 페이지 하단에서 시작 → 행 단위 분리 검증 (S1)
#[test]
fn test_table_split_10rows_at_page_bottom() {
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    // 본문 영역 절반 이상을 채우는 텍스트 문단 (본문 ~826px, 500px 점유)
    let filler = make_multiline_paragraph(5, 7500); // 5줄 x ~100px = ~500px

    // 10행 표 (각 행 6000 HWPUNIT ≈ 80px → 총 ~800px, 남은 ~326px에 안 맞음)
    let mut cells = Vec::new();
    for r in 0..10u16 {
        for c in 0..2u16 {
            cells.push(Cell {
                row: r,
                col: c,
                row_span: 1,
                col_span: 1,
                height: 6000,
                width: 5000,
                ..Default::default()
            });
        }
    }
    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(Table {
        row_count: 10,
        col_count: 2,
        cells,
        ..Default::default()
    })));

    let paras = vec![filler, table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 2페이지 이상으로 분할되어야 함
    assert!(
        result.pages.len() >= 2,
        "10행 표가 페이지 하단에서 분할되어야 함, pages={}",
        result.pages.len()
    );

    // PartialTable들의 행 범위를 수집
    let mut partials: Vec<(usize, usize)> = Vec::new();
    for page in &result.pages {
        for col in &page.column_contents {
            for item in &col.items {
                if let PageItem::PartialTable {
                    start_row, end_row, ..
                } = item
                {
                    partials.push((*start_row, *end_row));
                }
            }
        }
    }
    assert!(!partials.is_empty(), "PartialTable이 존재해야 함");

    // 첫 파트 start_row=0
    assert_eq!(partials[0].0, 0, "첫 파트 start_row은 0이어야 함");

    // 파트가 연속적이어야 함 (이전 end_row == 다음 start_row)
    for i in 1..partials.len() {
        assert_eq!(
            partials[i - 1].1,
            partials[i].0,
            "행 범위가 연속적이어야 함: 파트{} end_row={} ≠ 파트{} start_row={}",
            i - 1,
            partials[i - 1].1,
            i,
            partials[i].0
        );
    }

    // 마지막 파트 end_row=10
    assert_eq!(
        partials.last().unwrap().1,
        10,
        "마지막 파트 end_row은 전체 행 수(10)여야 함"
    );
}

/// 50행 대형 표 → 여러 페이지 분할 검증 (S2)
#[test]
fn test_table_split_50rows_multi_page() {
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    // 50행 표 (각 행 4000 HWPUNIT ≈ 53px → 총 ~2667px, 약 3~4페이지 필요)
    let mut cells = Vec::new();
    for r in 0..50u16 {
        cells.push(Cell {
            row: r,
            col: 0,
            row_span: 1,
            col_span: 1,
            height: 4000,
            width: 10000,
            ..Default::default()
        });
    }
    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(Table {
        row_count: 50,
        col_count: 1,
        cells,
        ..Default::default()
    })));

    let paras = vec![table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 3페이지 이상
    assert!(
        result.pages.len() >= 3,
        "50행 대형 표가 3+페이지에 걸쳐야 함, pages={}",
        result.pages.len()
    );

    // 모든 PartialTable의 행 범위 수집
    let mut partials: Vec<(usize, usize)> = Vec::new();
    for page in &result.pages {
        for col in &page.column_contents {
            for item in &col.items {
                if let PageItem::PartialTable {
                    start_row, end_row, ..
                } = item
                {
                    partials.push((*start_row, *end_row));
                }
            }
        }
    }

    // 전체 50행이 빠짐없이 커버되어야 함
    assert_eq!(partials[0].0, 0, "첫 파트 start_row=0");
    for i in 1..partials.len() {
        assert_eq!(
            partials[i - 1].1,
            partials[i].0,
            "행 범위 연속: 파트{} end={}  ≠ 파트{} start={}",
            i - 1,
            partials[i - 1].1,
            i,
            partials[i].0
        );
    }
    assert_eq!(partials.last().unwrap().1, 50, "마지막 파트 end_row=50");

    // 각 파트의 행 범위가 비어있지 않아야 함
    for (i, (s, e)) in partials.iter().enumerate() {
        assert!(e > s, "파트{}: start_row={} >= end_row={}", i, s, e);
    }
}

/// 셀 내 중첩 표가 있는 행의 분할 검증 (S3)
#[test]
fn test_table_split_with_nested_table() {
    use crate::model::control::Control;
    use crate::model::paragraph::LineSeg;
    use crate::model::table::{Cell, Table};

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();

    // 중첩 표: 10행, 각 행 높이 8000 HWPUNIT → ~1067px (본문 영역 826px 초과)
    let mut nested_cells = Vec::new();
    for r in 0..10u16 {
        nested_cells.push(Cell {
            row: r,
            col: 0,
            row_span: 1,
            col_span: 1,
            height: 8000,
            width: 5000,
            paragraphs: vec![Paragraph {
                line_segs: vec![LineSeg {
                    line_height: 8000,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        });
    }
    let nested_table = Table {
        row_count: 10,
        col_count: 1,
        cells: nested_cells,
        ..Default::default()
    };

    // 외부 표: 2행, 첫 행에 중첩 표 포함
    // 셀 높이를 중첩 표 전체 높이(~1067px)로 설정 → 본문 영역 초과 → 분할 필수
    let nested_h: i32 = 8000 * 10; // 80000 HWPUNIT
    let outer_cell_0 = Cell {
        row: 0,
        col: 0,
        row_span: 1,
        col_span: 1,
        height: nested_h as u32,
        width: 10000,
        paragraphs: vec![Paragraph {
            line_segs: vec![LineSeg {
                line_height: nested_h,
                ..Default::default()
            }],
            controls: vec![Control::Table(Box::new(nested_table))],
            ..Default::default()
        }],
        ..Default::default()
    };
    let outer_cell_1 = Cell {
        row: 1,
        col: 0,
        row_span: 1,
        col_span: 1,
        height: 5000,
        width: 10000,
        paragraphs: vec![Paragraph {
            line_segs: vec![LineSeg {
                line_height: 5000,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut table_para = Paragraph::default();
    table_para.controls.push(Control::Table(Box::new(Table {
        row_count: 2,
        col_count: 1,
        cells: vec![outer_cell_0, outer_cell_1],
        ..Default::default()
    })));

    // 필러: 페이지의 절반을 채움
    let filler = make_multiline_paragraph(4, 7500);
    let paras = vec![filler, table_para];
    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, _measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &a4_page_def(),
        &ColumnDef::default(),
        0,
    );

    // 페이지가 분할되어야 함
    assert!(
        result.pages.len() >= 2,
        "중첩 표가 있는 외부 표가 분할되어야 함, pages={}",
        result.pages.len()
    );

    // PartialTable 존재 확인
    let has_partial = result.pages.iter().any(|p| {
        p.column_contents.iter().any(|c| {
            c.items
                .iter()
                .any(|i| matches!(i, PageItem::PartialTable { .. }))
        })
    });
    assert!(
        has_partial,
        "중첩 표 포함 외부 표에 PartialTable이 존재해야 함"
    );
}

/// B-011 재현: 표 높이가 body area를 초과하지 않는지 검증 (S4)
#[test]
fn test_table_height_within_body_area() {
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};

    let paginator = Paginator::with_default_dpi();
    let styles = ResolvedStyleSet::default();
    let page_def = a4_page_def();

    // A4 본문 영역 높이 계산 (HWPUNIT → px)
    let body_h_hwpunit = page_def.height as i32
        - page_def.margin_top as i32
        - page_def.margin_bottom as i32
        - page_def.margin_header as i32
        - page_def.margin_footer as i32;
    let body_h_px = crate::renderer::hwpunit_to_px(body_h_hwpunit, 96.0);

    // 여러 표를 순서대로 배치 (각 5행, 높이 10000 HWPUNIT)
    let mut paras = Vec::new();
    for _ in 0..5 {
        let mut cells = Vec::new();
        for r in 0..5u16 {
            for c in 0..2u16 {
                cells.push(Cell {
                    row: r,
                    col: c,
                    row_span: 1,
                    col_span: 1,
                    height: 10000,
                    width: 5000,
                    ..Default::default()
                });
            }
        }
        let mut table_para = Paragraph::default();
        table_para.controls.push(Control::Table(Box::new(Table {
            row_count: 5,
            col_count: 2,
            cells,
            ..Default::default()
        })));
        paras.push(table_para);
    }

    let composed: Vec<ComposedParagraph> = Vec::new();
    let (result, measured) = paginator.paginate(
        &paras,
        &composed,
        &styles,
        &page_def,
        &ColumnDef::default(),
        0,
    );

    // 각 페이지의 콘텐츠 높이 합이 body area를 초과하지 않는지 확인
    for (page_idx, page) in result.pages.iter().enumerate() {
        for col in &page.column_contents {
            let mut height_sum = 0.0_f64;
            for item in &col.items {
                match item {
                    PageItem::FullParagraph { para_index } => {
                        let mp = measured.get_measured_paragraph(*para_index);
                        if let Some(m) = mp {
                            height_sum += m.total_height;
                        }
                    }
                    PageItem::Table { para_index, .. } => {
                        let mt = measured.get_measured_table(*para_index, 0);
                        if let Some(m) = mt {
                            height_sum += m.total_height;
                        }
                    }
                    PageItem::PartialTable {
                        para_index,
                        start_row,
                        end_row,
                        ..
                    } => {
                        let mt = measured.get_measured_table(*para_index, 0);
                        if let Some(m) = mt {
                            // cumulative_heights로 부분 높이 계산
                            let start_h = if *start_row > 0 {
                                m.cumulative_heights
                                    .get(*start_row - 1)
                                    .copied()
                                    .unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            let end_h = m
                                .cumulative_heights
                                .get(*end_row - 1)
                                .copied()
                                .unwrap_or(m.total_height);
                            height_sum += end_h - start_h;
                        }
                    }
                    _ => {}
                }
            }
            // body area를 초과하면 안 됨 (약간의 여유 허용)
            assert!(
                height_sum <= body_h_px + 2.0,
                "page {} 콘텐츠 높이({:.1}px)가 body area({:.1}px)를 초과함",
                page_idx,
                height_sum,
                body_h_px
            );
        }
    }
}
