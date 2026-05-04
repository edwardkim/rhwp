use super::*;
use crate::model::paragraph::{Paragraph, LineSeg, CharShapeRef};
use crate::model::page::{PageDef, ColumnDef};
use crate::model::style::{Numbering, NumberingHead};
use crate::renderer::composer::compose_paragraph;
use crate::renderer::style_resolver::ResolvedStyleSet;
use super::super::pagination::{PageContent, ColumnContent, PageItem};
use super::super::page_layout::PageLayoutInfo;
use super::utils::{expand_numbering_format, numbering_format_to_number_format};
use super::text_measurement::estimate_text_width;
use crate::renderer::{TextStyle, TabStop};

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

#[test]
fn test_build_empty_page() {
    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(
        &a4_page_def(),
        &ColumnDef::default(),
    );
    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: Vec::new(),
        active_header: None,
        active_footer: None,
        page_number_pos: None, page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None, extra_master_pages: Vec::new(),
    };
    let styles = ResolvedStyleSet::default();
    let tree = engine.build_render_tree(&page_content, &[], &[], &[], &[], &styles, &FootnoteShape::default(), &[], None, &[], None, 0, &[]);
    // 페이지 노드 + 배경 + 머리말 + 본문 + 각주 + 꼬리말
    assert!(tree.root.children.len() >= 4);
}

#[test]
fn test_build_page_with_paragraph() {
    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(
        &a4_page_def(),
        &ColumnDef::default(),
    );

    let paragraphs = vec![Paragraph {
        text: "안녕하세요".to_string(),
        line_segs: vec![LineSeg {
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        ..Default::default()
    }];

    let composed: Vec<_> = paragraphs.iter().map(|p| compose_paragraph(p)).collect();
    let styles = ResolvedStyleSet::default();

    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: vec![ColumnContent {
            column_index: 0,
            items: vec![PageItem::FullParagraph { para_index: 0 }],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None, page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None, extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(&page_content, &paragraphs, &paragraphs, &paragraphs, &composed, &styles, &FootnoteShape::default(), &[], None, &[], None, 0, &[]);
    assert!(tree.needs_render());

    // Body 노드 찾기
    let body = tree.root.children.iter().find(|n| matches!(n.node_type, RenderNodeType::Body { .. }));
    assert!(body.is_some());
    let body = body.unwrap();
    // Column 노드가 있어야 함
    assert!(!body.children.is_empty());
}

#[test]
fn test_layout_with_composed_styles() {
    use crate::renderer::style_resolver::ResolvedCharStyle;

    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(
        &a4_page_def(),
        &ColumnDef::default(),
    );

    let paragraphs = vec![Paragraph {
        text: "AAABBB".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5],
        char_count: 7,
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 0 },
            CharShapeRef { start_pos: 3, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg {
            line_height: 800,
            baseline_distance: 640,
            ..Default::default()
        }],
        ..Default::default()
    }];

    let composed: Vec<_> = paragraphs.iter().map(|p| compose_paragraph(p)).collect();

    let styles = ResolvedStyleSet {
        char_styles: vec![
            ResolvedCharStyle {
                font_family: "함초롬돋움".to_string(),
                font_size: 16.0,
                bold: true,
                ..Default::default()
            },
            ResolvedCharStyle {
                font_family: "함초롬바탕".to_string(),
                font_size: 12.0,
                italic: true,
                text_color: 0x00FF0000,
                ..Default::default()
            },
        ],
        para_styles: Vec::new(),
        border_styles: Vec::new(),
        numberings: Vec::new(),
        bullets: Vec::new(),
    };

    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: vec![ColumnContent {
            column_index: 0,
            items: vec![PageItem::FullParagraph { para_index: 0 }],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None, page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None, extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(&page_content, &paragraphs, &paragraphs, &paragraphs, &composed, &styles, &FootnoteShape::default(), &[], None, &[], None, 0, &[]);

    // Body > Column > TextLine 찾기
    let body = tree.root.children.iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let col = &body.children[0];
    let line = &col.children[0];

    // TextLine 내에 2개의 TextRun이 있어야 함
    assert_eq!(line.children.len(), 2);

    // 첫 번째 TextRun: "AAA", bold, 함초롬돋움
    match &line.children[0].node_type {
        RenderNodeType::TextRun(run) => {
            assert_eq!(run.text, "AAA");
            assert_eq!(run.style.font_family, "함초롬돋움");
            assert!(run.style.bold);
            assert!(!run.style.italic);
            assert!((run.style.font_size - 16.0).abs() < 0.01);
        }
        _ => panic!("Expected TextRun"),
    }

    // 두 번째 TextRun: "BBB", italic, 함초롬바탕
    match &line.children[1].node_type {
        RenderNodeType::TextRun(run) => {
            assert_eq!(run.text, "BBB");
            assert_eq!(run.style.font_family, "함초롬바탕");
            assert!(!run.style.bold);
            assert!(run.style.italic);
            assert_eq!(run.style.color, 0x00FF0000);
        }
        _ => panic!("Expected TextRun"),
    }
}

#[test]
fn test_layout_multi_run_x_position() {
    use crate::renderer::style_resolver::ResolvedCharStyle;

    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(
        &a4_page_def(),
        &ColumnDef::default(),
    );

    let paragraphs = vec![Paragraph {
        text: "AB가나".to_string(),
        char_offsets: vec![0, 1, 2, 3],
        char_count: 5,
        char_shapes: vec![
            CharShapeRef { start_pos: 0, char_shape_id: 0 },
            CharShapeRef { start_pos: 2, char_shape_id: 1 },
        ],
        line_segs: vec![LineSeg {
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        ..Default::default()
    }];

    let composed: Vec<_> = paragraphs.iter().map(|p| compose_paragraph(p)).collect();
    let styles = ResolvedStyleSet {
        char_styles: vec![
            ResolvedCharStyle { font_size: 16.0, ..Default::default() },
            ResolvedCharStyle { font_size: 16.0, ..Default::default() },
        ],
        para_styles: Vec::new(),
        border_styles: Vec::new(),
        numberings: Vec::new(),
        bullets: Vec::new(),
    };

    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: vec![ColumnContent {
            column_index: 0,
            items: vec![PageItem::FullParagraph { para_index: 0 }],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None, page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None, extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(&page_content, &paragraphs, &paragraphs, &paragraphs, &composed, &styles, &FootnoteShape::default(), &[], None, &[], None, 0, &[]);

    let body = tree.root.children.iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let col = &body.children[0];
    let line = &col.children[0];

    assert_eq!(line.children.len(), 2);

    // 두 번째 TextRun의 x 좌표가 첫 번째 TextRun 끝 이후여야 함
    let run1_x = line.children[0].bbox.x;
    let run1_w = line.children[0].bbox.width;
    let run2_x = line.children[1].bbox.x;
    assert!((run2_x - (run1_x + run1_w)).abs() < 0.01);
}

#[test]
fn test_resolved_to_text_style() {
    use crate::renderer::style_resolver::ResolvedCharStyle;
    use crate::model::style::UnderlineType;

    let styles = ResolvedStyleSet {
        char_styles: vec![ResolvedCharStyle {
            font_family: "나눔고딕".to_string(),
            font_size: 14.0,
            bold: true,
            italic: false,
            text_color: 0x000000FF,
            underline: UnderlineType::Bottom,
            letter_spacing: 1.5,
            ..Default::default()
        }],
        para_styles: Vec::new(),
        border_styles: Vec::new(),
        numberings: Vec::new(),
        bullets: Vec::new(),
    };

    let ts = resolved_to_text_style(&styles, 0, 0);
    assert_eq!(ts.font_family, "나눔고딕");
    assert!((ts.font_size - 14.0).abs() < 0.01);
    assert!(ts.bold);
    assert!(!ts.italic);
    assert!(matches!(ts.underline, UnderlineType::Bottom));
    assert_eq!(ts.color, 0x000000FF);
    assert!((ts.letter_spacing - 1.5).abs() < 0.01);
    assert!((ts.ratio - 1.0).abs() < 0.01); // 기본 장평 100%
}

#[test]
fn test_resolved_to_text_style_with_ratio() {
    use crate::renderer::style_resolver::ResolvedCharStyle;

    let styles = ResolvedStyleSet {
        char_styles: vec![ResolvedCharStyle {
            font_family: "함초롬돋움".to_string(),
            font_size: 16.0,
            ratio: 0.8,
            ..Default::default()
        }],
        para_styles: Vec::new(),
        border_styles: Vec::new(),
        numberings: Vec::new(),
        bullets: Vec::new(),
    };

    let ts = resolved_to_text_style(&styles, 0, 0);
    assert!((ts.ratio - 0.8).abs() < 0.01);
}

#[test]
fn test_resolved_to_text_style_missing_id() {
    let styles = ResolvedStyleSet::default();
    let ts = resolved_to_text_style(&styles, 999, 0);
    assert!(ts.font_family.is_empty());
    assert!((ts.font_size - 0.0).abs() < 0.01);
    assert!((ts.ratio - 1.0).abs() < 0.01); // 기본값 1.0
}

#[test]
fn test_estimate_text_width() {
    let style = TextStyle { font_size: 16.0, ..Default::default() };

    // Latin characters: 0.5 * font_size each
    let w = estimate_text_width("AB", &style);
    assert!((w - 16.0).abs() < 0.01); // 2 * 8.0

    // CJK characters: 1.0 * font_size each
    let w = estimate_text_width("가나", &style);
    assert!((w - 32.0).abs() < 0.01); // 2 * 16.0

    // Mixed
    let w = estimate_text_width("A가", &style);
    assert!((w - 24.0).abs() < 0.01); // 8.0 + 16.0
}

#[test]
fn test_estimate_text_width_with_ratio() {
    // 장평 80%: 기본 폭의 80%
    let style = TextStyle { font_size: 16.0, ratio: 0.8, ..Default::default() };
    let w = estimate_text_width("가나", &style);
    // base: 2 * 16.0 = 32.0, * 0.8 = 25.6 → round = 26.0
    assert!((w - 26.0).abs() < 0.01);

    // 장평 150%
    let style = TextStyle { font_size: 16.0, ratio: 1.5, ..Default::default() };
    let w = estimate_text_width("AB", &style);
    // base: 2 * 8.0 = 16.0, * 1.5 = 24.0
    assert!((w - 24.0).abs() < 0.01);

    // 장평 100%: 기존과 동일
    let style = TextStyle { font_size: 16.0, ratio: 1.0, ..Default::default() };
    let w = estimate_text_width("가나", &style);
    assert!((w - 32.0).abs() < 0.01);
}

#[test]
fn test_compute_char_positions_extra_word_spacing() {
    // extra_word_spacing은 공백 문자에만 추가 간격 적용
    let style = TextStyle {
        font_size: 16.0,
        extra_word_spacing: 10.0,
        ..Default::default()
    };
    let positions = compute_char_positions("A B", &style);
    // A: 8.0, ' ': 8.0 + 10.0 = 18.0, B: 8.0
    assert_eq!(positions.len(), 4); // 3문자 + 1
    assert!((positions[0] - 0.0).abs() < 0.01);
    assert!((positions[1] - 8.0).abs() < 0.01); // A
    assert!((positions[2] - 26.0).abs() < 0.01); // A + space(8+10)
    assert!((positions[3] - 34.0).abs() < 0.01); // A + space + B
}

#[test]
fn test_compute_char_positions_extra_char_spacing() {
    // extra_char_spacing은 모든 문자에 추가 간격 적용
    let style = TextStyle {
        font_size: 16.0,
        extra_char_spacing: 5.0,
        ..Default::default()
    };
    let positions = compute_char_positions("AB", &style);
    // A: 8.0 + 5.0 = 13.0, B: 8.0 + 5.0 = 13.0
    assert_eq!(positions.len(), 3);
    assert!((positions[0] - 0.0).abs() < 0.01);
    assert!((positions[1] - 13.0).abs() < 0.01);
    assert!((positions[2] - 26.0).abs() < 0.01);
}

#[test]
fn test_estimate_text_width_with_extra_spacing() {
    // extra_word_spacing + extra_char_spacing 동시 적용
    let style = TextStyle {
        font_size: 16.0,
        extra_word_spacing: 10.0,
        extra_char_spacing: 2.0,
        ..Default::default()
    };
    // "A B": A(8+2) + space(8+2+10) + B(8+2) = 10 + 20 + 10 = 40
    let w = estimate_text_width("A B", &style);
    assert!((w - 40.0).abs() < 0.01);
}

#[test]
fn test_extra_spacing_zero_default() {
    // 기본값(0.0)에서는 기존 동작과 동일
    let style = TextStyle { font_size: 16.0, ..Default::default() };
    let w_no_extra = estimate_text_width("가나다", &style);
    let positions_no_extra = compute_char_positions("가나다", &style);

    let style_explicit = TextStyle {
        font_size: 16.0,
        extra_word_spacing: 0.0,
        extra_char_spacing: 0.0,
        ..Default::default()
    };
    let w_explicit = estimate_text_width("가나다", &style_explicit);
    let positions_explicit = compute_char_positions("가나다", &style_explicit);

    assert!((w_no_extra - w_explicit).abs() < 0.01);
    for (a, b) in positions_no_extra.iter().zip(positions_explicit.iter()) {
        assert!((a - b).abs() < 0.01);
    }
}

#[test]
fn test_extra_word_spacing_no_effect_on_non_space() {
    // 공백 없는 텍스트에서 extra_word_spacing은 영향 없음
    let style_base = TextStyle { font_size: 16.0, ..Default::default() };
    let style_extra = TextStyle {
        font_size: 16.0,
        extra_word_spacing: 100.0,
        ..Default::default()
    };
    let w_base = estimate_text_width("가나다", &style_base);
    let w_extra = estimate_text_width("가나다", &style_extra);
    assert!((w_base - w_extra).abs() < 0.01);
}

#[test]
fn test_tab_not_affected_by_extra_spacing() {
    // 탭 문자는 extra_char_spacing/extra_word_spacing에 영향받지 않음
    let style = TextStyle {
        font_size: 16.0,
        extra_char_spacing: 100.0,
        extra_word_spacing: 100.0,
        ..Default::default()
    };
    let positions = compute_char_positions("\t", &style);
    assert_eq!(positions.len(), 2);
    // 탭은 tab_w로 스냅 (font_size * 4 = 64)
    assert!((positions[1] - 64.0).abs() < 0.01);
}

#[test]
fn test_layout_table_basic() {
    use crate::model::table::{Table, Cell};
    use crate::model::control::Control;
    use crate::renderer::style_resolver::ResolvedBorderStyle;

    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(
        &a4_page_def(),
        &ColumnDef::default(),
    );

    // 2x2 표가 있는 문단 (각 셀에 border_fill_id=1 설정)
    let table = Table {
        row_count: 2,
        col_count: 2,
        row_sizes: vec![2, 2], // 행별 셀 수
        cells: vec![
            Cell {
                col: 0, row: 0, col_span: 1, row_span: 1,
                width: 3000, height: 1200, border_fill_id: 1,
                paragraphs: vec![Paragraph { text: "A".to_string(), ..Default::default() }],
                ..Default::default()
            },
            Cell {
                col: 1, row: 0, col_span: 1, row_span: 1,
                width: 3000, height: 1200, border_fill_id: 1,
                paragraphs: vec![Paragraph { text: "B".to_string(), ..Default::default() }],
                ..Default::default()
            },
            Cell {
                col: 0, row: 1, col_span: 1, row_span: 1,
                width: 3000, height: 1200, border_fill_id: 1,
                paragraphs: vec![Paragraph { text: "C".to_string(), ..Default::default() }],
                ..Default::default()
            },
            Cell {
                col: 1, row: 1, col_span: 1, row_span: 1,
                width: 3000, height: 1200, border_fill_id: 1,
                paragraphs: vec![Paragraph { text: "D".to_string(), ..Default::default() }],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let paragraphs = vec![Paragraph {
        text: String::new(),
        controls: vec![Control::Table(Box::new(table))],
        line_segs: vec![LineSeg { line_height: 400, ..Default::default() }],
        ..Default::default()
    }];

    let composed: Vec<_> = paragraphs.iter().map(|p| compose_paragraph(p)).collect();
    // border_fill_id=1은 styles.border_styles[0]을 참조 (1-indexed)
    let styles = ResolvedStyleSet {
        border_styles: vec![ResolvedBorderStyle::default()],
        ..Default::default()
    };

    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: vec![ColumnContent {
            column_index: 0,
            items: vec![
                PageItem::FullParagraph { para_index: 0 },
                PageItem::Table { para_index: 0, control_index: 0 },
            ],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None, page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None, extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(&page_content, &paragraphs, &paragraphs, &paragraphs, &composed, &styles, &FootnoteShape::default(), &[], None, &[], None, 0, &[]);

    // Body > Column 내에 Table 노드가 있어야 함
    let body = tree.root.children.iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let col = &body.children[0];

    let table_node = col.children.iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table(_)))
        .expect("Table node should exist");

    // 4개 셀 + 엣지 기반 테두리 Line 노드들
    let cell_count = table_node.children.iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::TableCell(_)))
        .count();
    assert_eq!(cell_count, 4);

    // 엣지 기반 테두리: 표 노드의 직접 자식으로 Line 노드가 있어야 함
    // 2x2 표: 수평 3줄 + 수직 3줄 = 6개 이상의 Line 노드
    // (기본 Solid 테두리이므로 이중선/삼중선이 아니면 각 엣지당 1개)
    let table_line_count = table_node.children.iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::Line(_)))
        .count();
    assert!(table_line_count >= 6, "표에 6개 이상의 엣지 테두리가 있어야 함 (실제: {})", table_line_count);
}

#[test]
fn test_layout_table_cell_positions() {
    use crate::model::table::{Table, Cell};
    use crate::model::control::Control;

    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(
        &a4_page_def(),
        &ColumnDef::default(),
    );

    let table = Table {
        row_count: 2,
        col_count: 2,
        row_sizes: vec![2, 2], // 행별 셀 수
        cells: vec![
            Cell { col: 0, row: 0, col_span: 1, row_span: 1, width: 3600, height: 720, ..Default::default() },
            Cell { col: 1, row: 0, col_span: 1, row_span: 1, width: 3600, height: 720, ..Default::default() },
            Cell { col: 0, row: 1, col_span: 1, row_span: 1, width: 3600, height: 720, ..Default::default() },
            Cell { col: 1, row: 1, col_span: 1, row_span: 1, width: 3600, height: 720, ..Default::default() },
        ],
        ..Default::default()
    };

    let paragraphs = vec![Paragraph {
        text: String::new(),
        controls: vec![Control::Table(Box::new(table))],
        line_segs: vec![LineSeg { line_height: 400, ..Default::default() }],
        ..Default::default()
    }];

    let composed: Vec<_> = paragraphs.iter().map(|p| compose_paragraph(p)).collect();
    let styles = ResolvedStyleSet::default();

    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: vec![ColumnContent {
            column_index: 0,
            items: vec![
                PageItem::FullParagraph { para_index: 0 },
                PageItem::Table { para_index: 0, control_index: 0 },
            ],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None, page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None, extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(&page_content, &paragraphs, &paragraphs, &paragraphs, &composed, &styles, &FootnoteShape::default(), &[], None, &[], None, 0, &[]);

    let body = tree.root.children.iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let col = &body.children[0];
    let table_node = col.children.iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table(_)))
        .unwrap();

    // 셀 (1,0)의 x좌표는 셀 (0,0)의 x + width 이후
    let cell_00 = &table_node.children[0];
    let cell_10 = &table_node.children[1];
    let cell_01 = &table_node.children[2];

    // 3600 HWPUNIT @ 96dpi = 48.0 px
    let cell_width = 3600.0 * 96.0 / 7200.0;
    assert!((cell_10.bbox.x - cell_00.bbox.x - cell_width).abs() < 0.1);

    // 셀 (0,1)의 y좌표는 셀 (0,0)의 y + row_height 이후
    let row_height = 720.0 * 96.0 / 7200.0;
    assert!((cell_01.bbox.y - cell_00.bbox.y - row_height).abs() < 0.1);
}

#[test]
fn test_layout_rect_to_bbox() {
    let rect = LayoutRect {
        x: 10.0,
        y: 20.0,
        width: 100.0,
        height: 200.0,
    };
    let bbox = layout_rect_to_bbox(&rect);
    assert!((bbox.x - 10.0).abs() < 0.01);
    assert!((bbox.width - 100.0).abs() < 0.01);
}

#[test]
fn test_numbering_state_advance() {
    let mut state = NumberingState::default();

    // 첫 번째 수준 0 → counter[0] = 1
    let c = state.advance(0, 0, None);
    assert_eq!(c[0], 1);

    // 수준 1 → counter[1] = 1
    let c = state.advance(0, 1, None);
    assert_eq!(c[0], 1);
    assert_eq!(c[1], 1);

    // 수준 1 반복 → counter[1] = 2
    let c = state.advance(0, 1, None);
    assert_eq!(c[1], 2);

    // 수준 0으로 복귀 → counter[0] = 2, counter[1] 리셋
    let c = state.advance(0, 0, None);
    assert_eq!(c[0], 2);
    assert_eq!(c[1], 0);

    // 다른 numbering_id → 히스토리 없으면 리셋
    let c = state.advance(1, 0, None);
    assert_eq!(c[0], 1);
}

#[test]
fn test_expand_numbering_format_digit() {
    let numbering = Numbering {
        raw_data: None,
        heads: [NumberingHead { number_format: 0, ..Default::default() }; 7],
        level_formats: [
            "^1.".to_string(), "^2.".to_string(), "^3)".to_string(),
            String::new(), String::new(), String::new(), String::new(),
        ],
        start_number: 0,
        level_start_numbers: [1, 1, 1, 1, 1, 1, 1],
    };
    let counters = [3, 2, 1, 0, 0, 0, 0];
    let result = expand_numbering_format("^1.", &counters, &numbering, &numbering.level_start_numbers);
    assert_eq!(result, "3.");

    let result = expand_numbering_format("^2.", &counters, &numbering, &numbering.level_start_numbers);
    assert_eq!(result, "2.");

    let result = expand_numbering_format("(^3)", &counters, &numbering, &numbering.level_start_numbers);
    assert_eq!(result, "(1)");
}

#[test]
fn test_expand_numbering_format_hangul() {
    let mut heads = [NumberingHead::default(); 7];
    heads[1].number_format = 8; // HangulGaNaDa
    let numbering = Numbering {
        raw_data: None,
        heads,
        level_formats: [
            String::new(), "^2.".to_string(), String::new(),
            String::new(), String::new(), String::new(), String::new(),
        ],
        start_number: 0,
        level_start_numbers: [1, 1, 1, 1, 1, 1, 1],
    };
    let counters = [1, 3, 0, 0, 0, 0, 0];
    let result = expand_numbering_format("^2.", &counters, &numbering, &numbering.level_start_numbers);
    assert_eq!(result, "다.");
}

#[test]
fn test_numbering_format_to_number_format() {
    assert!(matches!(numbering_format_to_number_format(0), NumFmt::Digit));
    assert!(matches!(numbering_format_to_number_format(1), NumFmt::CircledDigit));
    assert!(matches!(numbering_format_to_number_format(2), NumFmt::RomanUpper));
    assert!(matches!(numbering_format_to_number_format(8), NumFmt::HangulGaNaDa));
    assert!(matches!(numbering_format_to_number_format(255), NumFmt::Digit));
}

// =====================================================================
// NumberingState 카운터 재계산 테스트
// =====================================================================

#[test]
fn test_numbering_state_level_change_recalculation() {
    // 시나리오: 가, 나, 다 → 나를 한 단계 내리면 → 가, 1), 나
    let mut state = NumberingState::default();

    // 같은 numbering_id=1로 3개 문단 모두 level 0
    let c1 = state.advance(1, 0, None); // "가"
    assert_eq!(c1[0], 1);

    let c2 = state.advance(1, 0, None); // "나"
    assert_eq!(c2[0], 2);

    let c3 = state.advance(1, 0, None); // "다"
    assert_eq!(c3[0], 3);

    // 이제 나를 level 1로 변경 후 처음부터 재계산
    state.reset();

    let c1 = state.advance(1, 0, None); // "가" (level 0, counter[0]=1)
    assert_eq!(c1[0], 1);

    let c2 = state.advance(1, 1, None); // level 1, counter[1]=1 → "1)"
    assert_eq!(c2[0], 1); // level 0 카운터 유지
    assert_eq!(c2[1], 1); // level 1 카운터 = 1

    let c3 = state.advance(1, 0, None); // 다 → "나" (level 0, counter[0]=2)
    assert_eq!(c3[0], 2); // level 0 = 2, 즉 "나"
    assert_eq!(c3[1], 0); // 하위 수준 리셋
}

#[test]
fn test_numbering_state_promote_recalculation() {
    // 시나리오: 한 단계 올리기
    // 1), 2), 3) → 2)를 한 단계 올리면 → 1), 가, 1)
    let mut state = NumberingState::default();

    // 모두 level 1
    let c1 = state.advance(1, 1, None);
    assert_eq!(c1[1], 1); // 1)

    let c2 = state.advance(1, 1, None);
    assert_eq!(c2[1], 2); // 2)

    let c3 = state.advance(1, 1, None);
    assert_eq!(c3[1], 3); // 3)

    // 2)를 level 0으로 올린 후 재계산
    state.reset();

    let c1 = state.advance(1, 1, None);
    assert_eq!(c1[1], 1); // 1)

    let c2 = state.advance(1, 0, None); // 한 단계 올림 → level 0
    assert_eq!(c2[0], 1); // "가"
    assert_eq!(c2[1], 0); // 하위 수준 리셋

    let c3 = state.advance(1, 1, None);
    assert_eq!(c3[0], 1); // level 0 유지
    assert_eq!(c3[1], 1); // level 1 = 1 → "1)" (리셋되었으므로)
}

#[test]
fn test_numbering_state_different_numbering_id_resets() {
    use crate::model::paragraph::NumberingRestart;
    // para-head-num-2.hwp 패턴 재현:
    // id=3: 가(1), 나(2) → id=2: 가(1, 리셋) → id=3: 다(3, 복원) → id=4: 1(1) → id=4: 2(2)
    let mut state = NumberingState::default();

    // id=3: 가, 나
    let c1 = state.advance(3, 1, None);
    assert_eq!(c1[1], 1); // "가"
    let c2 = state.advance(3, 1, None);
    assert_eq!(c2[1], 2); // "나"

    // id=2: 새 번호 시작 (히스토리 없음 → 리셋)
    let c3 = state.advance(2, 1, None);
    assert_eq!(c3[1], 1); // "가" (리셋)

    // id=3: 이전 번호 이어 (히스토리 복원 → 2에서 이어서 3)
    let c4 = state.advance(3, 1, None);
    assert_eq!(c4[1], 3); // "다"

    // id=4: 새 번호 시작 (히스토리 없음 → 리셋)
    let c5 = state.advance(4, 1, None);
    assert_eq!(c5[1], 1); // "1" (format이 다르지만 counter=1)

    // id=4: 앞 번호 이어
    let c6 = state.advance(4, 1, None);
    assert_eq!(c6[1], 2); // "2"
}

#[test]
fn test_geometric_shapes_treated_as_fullwidth() {
    // Task #146: Geometric Shapes (U+25A0-U+25FF) 는 HWP 문서의 섹션 머리
    // 기호 (□ 1. / ■ 가. / ○ ㅇ 등) 로 널리 쓰이므로 전각(font_size) 폭
    // 으로 측정되어야 한다.
    let style = TextStyle { font_size: 20.0, ..Default::default() };
    for c in ['□', '■', '▲', '▼', '◆', '○', '●', '◇'] {
        let text = c.to_string();
        let positions = compute_char_positions(&text, &style);
        assert!(
            (positions[1] - 20.0).abs() < 0.01,
            "'{}' (U+{:04X}) expected full-width advance 20.0, got {}",
            c, c as u32, positions[1]
        );
    }
}

#[test]
fn test_square_bullet_with_space_preserves_layout() {
    // Task #146 회귀 방지: "□ 가" 제목 패턴에서 □ 가 반각으로 측정되면
    // 후속 글자 x 좌표가 em 단위만큼 좌측으로 붕괴한다.
    // 자간 -8% 는 text-align.hwp 제목 CharShape 와 동일.
    let style = TextStyle {
        font_size: 20.0,
        letter_spacing: -1.6, // -8% of 20
        ..Default::default()
    };
    let positions = compute_char_positions("□ 가", &style);
    assert_eq!(positions.len(), 4);
    // □: 전각(20) + 자간(-1.6) = advance 18.4
    assert!((positions[1] - 18.4).abs() < 0.01, "positions[1] expected 18.4, got {}", positions[1]);
    // 공백: 반각(10) + 자간(-1.6) = advance 8.4 (min_clamp 5.0 미작동)
    assert!((positions[2] - 26.8).abs() < 0.01, "positions[2] expected 26.8, got {}", positions[2]);
    // 가: 전각(20) + 자간(-1.6) = advance 18.4
    assert!((positions[3] - 45.2).abs() < 0.01, "positions[3] expected 45.2, got {}", positions[3]);
}

#[test]
fn test_tac_leading_width_block_table_full_line() {
    // Task #146 v3: block 취급 TAC 표(너비 ≥ 90% seg_width)에서
    // composed.tac_controls 가 비어있을 때, 선행 텍스트는 line 0 전체로
    // 간주해 모든 run 폭을 합산해야 한다. text-align.hwp 문단 0.2 시나리오.
    use super::super::composer::{ComposedParagraph, ComposedLine, ComposedTextRun};
    use crate::renderer::style_resolver::{ResolvedStyleSet, ResolvedCharStyle};

    let line = ComposedLine {
        runs: vec![ComposedTextRun {
            text: "    ".to_string(),
            char_style_id: 0, lang_index: 0,
            ..Default::default()
        }],
        line_height: 400, baseline_distance: 320,
        segment_width: 48188, column_start: 0,
        line_spacing: 0, has_line_break: false, char_start: 0,
    };
    let composed = ComposedParagraph {
        lines: vec![line],
        para_style_id: 0,
        inline_controls: Vec::new(),
        numbering_text: None,
        tac_controls: Vec::new(), // block 취급이라 비어있음
        footnote_positions: Vec::new(),
        tab_extended: Vec::new(),
    };
    let styles = ResolvedStyleSet {
        char_styles: vec![ResolvedCharStyle {
            font_size: 20.0, letter_spacing: -1.6,
            ..Default::default()
        }],
        ..Default::default()
    };
    let width = super::compute_tac_leading_width(&composed, 0, &styles);
    // 4 spaces × (10 base - 1.6 lspc) = 33.6 (min_clamp 5.0 미작동)
    assert!((width - 33.6).abs() < 0.5, "expected ~33.6, got {}", width);
}

#[test]
fn test_is_heavy_display_face_matches_known_heavy_faces() {
    // Task #146 v4: HY헤드라인M 등 heavy display face 는 CharShape.bold=false
    // 여도 본래 heavy 이므로 SVG 에서 font-weight="bold" 강제 대상이어야 한다.
    //
    // Task #574: HY견명조 는 한컴 일반 두께 명조 — heavy 가 아님. 제거.
    // HY견명조B 는 명시 Bold variant — 보존.
    use crate::renderer::style_resolver::is_heavy_display_face;
    for face in ["HY헤드라인M", "HYHeadLine M", "HYHeadLine Medium",
                  "HY견고딕", "HY견명조B", "HY그래픽", "HY그래픽M"] {
        assert!(is_heavy_display_face(face), "{} should be heavy", face);
    }
    // 일반 face 는 false (HY견명조 는 Task #574 에서 heavy 제거)
    for face in ["Malgun Gothic", "맑은 고딕", "함초롬바탕", "함초롬돋움",
                  "바탕", "돋움", "HY신명조", "HY중고딕", "HY견명조"] {
        assert!(!is_heavy_display_face(face), "{} should NOT be heavy", face);
    }
}

#[test]
fn test_is_heavy_display_face_with_family_chain() {
    // font-family 체인에서 primary face(첫 항목) 기준 판정.
    use crate::renderer::style_resolver::is_heavy_display_face;
    assert!(is_heavy_display_face("HY헤드라인M,'Malgun Gothic',sans-serif"));
    assert!(is_heavy_display_face("HY견고딕, 돋움"));
    // 따옴표 포함
    assert!(is_heavy_display_face("'HY헤드라인M',Malgun Gothic"));
    assert!(is_heavy_display_face("\"HY그래픽\",바탕"));
    // primary 가 heavy 가 아니면 false (HY헤드라인M 이 두번째여도 false)
    assert!(!is_heavy_display_face("Malgun Gothic,HY헤드라인M"));
}

#[test]
fn test_tac_leading_width_inline_table_partial() {
    // inline 취급 TAC 표: tac_controls 에 위치 기록. 해당 위치까지만 합산.
    use super::super::composer::{ComposedParagraph, ComposedLine, ComposedTextRun};
    use crate::renderer::style_resolver::{ResolvedStyleSet, ResolvedCharStyle};

    let line = ComposedLine {
        runs: vec![ComposedTextRun {
            text: "ab가나".to_string(),
            char_style_id: 0, lang_index: 0,
            ..Default::default()
        }],
        line_height: 400, baseline_distance: 320,
        segment_width: 48188, column_start: 0,
        line_spacing: 0, has_line_break: false, char_start: 0,
    };
    let composed = ComposedParagraph {
        lines: vec![line],
        para_style_id: 0,
        inline_controls: Vec::new(),
        numbering_text: None,
        tac_controls: vec![(2, 1000, 0)], // pos=2 (ab 뒤), control_index=0
        footnote_positions: Vec::new(),
        tab_extended: Vec::new(),
    };
    let styles = ResolvedStyleSet {
        char_styles: vec![ResolvedCharStyle {
            font_size: 20.0, ..Default::default()
        }],
        ..Default::default()
    };
    let width = super::compute_tac_leading_width(&composed, 0, &styles);
    // "ab" 2 chars, 반각 × font_size/2 = 20*0.5*2 = 20
    assert!((width - 20.0).abs() < 0.5, "expected ~20.0, got {}", width);
}

// ────────────────────────────────────────────────────────────
// Task #290: resolve_last_tab_pending — cross-run 탭 감지 헬퍼
// ────────────────────────────────────────────────────────────

/// ext[2] 생성 편의: high=tab_type_enum+1, low=fill_type
fn mk_ext(width_hu: u16, tab_kind_hi: u8, fill_lo: u8) -> [u16; 7] {
    let tab_type = ((tab_kind_hi as u16) << 8) | (fill_lo as u16);
    [width_hu, 0, tab_type, 0, 0, 0, 9]
}

fn mk_text_style() -> TextStyle {
    TextStyle {
        font_size: 12.0,
        font_family: String::new(),
        line_x_offset: 0.0,
        ..Default::default()
    }
}

#[test]
fn task290_inline_left_returns_none() {
    // inline 이 LEFT (ext[2] high=1) 이면 pending 없음 — 본 수정의 핵심
    let ext = vec![mk_ext(100, 1, 0)]; // LEFT, fill=none
    let ts = mk_text_style();
    let tab_stops = vec![TabStop { position: 22.0, tab_type: 0, fill_type: 0 }];
    let result = super::paragraph_layout::resolve_last_tab_pending(
        "abc\t", 0, &ext, &ts, &tab_stops, 48.0, true, 420.0,
    );
    assert_eq!(result, None, "LEFT inline 은 pending 없음");
}

#[test]
fn task290_inline_right_uses_tabdef() {
    // inline 이 RIGHT (ext[2] high=2) 면 TabDef find_next_tab_stop 경로로 폴스루
    let ext = vec![mk_ext(200, 2, 3)]; // RIGHT, fill=dot
    let ts = mk_text_style();
    let tab_stops = vec![TabStop { position: 300.0, tab_type: 1, fill_type: 3 }];
    let result = super::paragraph_layout::resolve_last_tab_pending(
        "abc\t", 0, &ext, &ts, &tab_stops, 48.0, false, 420.0,
    );
    assert_eq!(result, Some((300.0, 1, 3)), "RIGHT inline → TabDef 기반 위치, fill=dot");
}

#[test]
fn task290_inline_center_uses_tabdef() {
    // inline 이 CENTER (ext[2] high=3) 면 TabDef 기반 위치
    let ext = vec![mk_ext(150, 3, 0)]; // CENTER
    let ts = mk_text_style();
    let tab_stops = vec![TabStop { position: 200.0, tab_type: 2, fill_type: 0 }];
    let result = super::paragraph_layout::resolve_last_tab_pending(
        "abc\t", 0, &ext, &ts, &tab_stops, 48.0, false, 420.0,
    );
    assert_eq!(result, Some((200.0, 2, 0)), "CENTER inline → TabDef 기반 위치, fill 없음");
}

#[test]
fn task290_no_inline_fallback_to_tabdef() {
    // inline_tabs 가 비었으면 TabDef 폴백 — 기존 동작 유지
    let ext: Vec<[u16; 7]> = vec![];
    let ts = mk_text_style();
    let tab_stops = vec![TabStop { position: 250.0, tab_type: 1, fill_type: 0 }];
    let result = super::paragraph_layout::resolve_last_tab_pending(
        "abc\t", 0, &ext, &ts, &tab_stops, 48.0, false, 420.0,
    );
    assert_eq!(result, Some((250.0, 1, 0)), "inline 없음 → TabDef RIGHT stop 사용, fill 없음");
}

#[test]
fn task290_no_inline_auto_tab_right_fallthrough() {
    // inline 없음 + TabDef stop 소진 + auto_tab_right=true → 우측 끝 RIGHT (기존 동작 유지)
    let ext: Vec<[u16; 7]> = vec![];
    let ts = mk_text_style();
    let tab_stops = vec![TabStop { position: 10.0, tab_type: 0, fill_type: 0 }]; // 이미 지나친 stop
    let result = super::paragraph_layout::resolve_last_tab_pending(
        "abcdef\t", 0, &ext, &ts, &tab_stops, 48.0, true, 420.0,
    );
    assert!(result.is_some(), "auto_tab_right 폴스루 → Some");
    let (tp, tt, _ft) = result.unwrap();
    assert_eq!(tt, 1, "auto_tab_right 은 RIGHT(1)");
    assert!((tp - 420.0).abs() < 0.1, "tab_pos 는 available_width 에 고정");
}

// [Task #296] inline_tab_type 헬퍼 단위 테스트
// HWP tab_extended 의 ext[2] 포맷: high byte = 탭 종류 enum+1, low byte = fill_type

#[test]
fn task296_inline_tab_type_left() {
    // ext[2] = 0x0100 (256) → high=1 = LEFT (exam_math #18 실측 케이스)
    let ext = [132u16, 0, 0x0100, 0, 0, 0, 9];
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 1);
}

#[test]
fn task296_inline_tab_type_right() {
    // ext[2] = 0x0203 (515) → high=2 = RIGHT, low=3 = fill=dot
    //         (hwp-3.0-HWPML 저작권\t1 실측 케이스, PR #292 트러블슈팅 기록)
    let ext = [200u16, 0, 0x0203, 0, 0, 0, 9];
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 2);
}

#[test]
fn task296_inline_tab_type_center() {
    // ext[2] = 0x0300 → high=3 = CENTER
    let ext = [150u16, 0, 0x0300, 0, 0, 0, 9];
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 3);
}

#[test]
fn task296_inline_tab_type_decimal() {
    // ext[2] = 0x0400 → high=4 = DECIMAL
    let ext = [100u16, 0, 0x0400, 0, 0, 0, 9];
    assert_eq!(super::text_measurement::inline_tab_type(&ext), 4);
}
