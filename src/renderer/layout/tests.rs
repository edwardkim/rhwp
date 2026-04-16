use super::super::page_layout::PageLayoutInfo;
use super::super::pagination::{ColumnContent, PageContent, PageItem};
use super::text_measurement::estimate_text_width;
use super::utils::{expand_numbering_format, numbering_format_to_number_format};
use super::*;
use crate::model::page::{ColumnDef, PageDef};
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::model::style::{Numbering, NumberingHead};
use crate::renderer::composer::compose_paragraph;
use crate::renderer::style_resolver::ResolvedStyleSet;

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
    let layout = PageLayoutInfo::from_page_def_default(&a4_page_def(), &ColumnDef::default());
    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: Vec::new(),
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };
    let styles = ResolvedStyleSet::default();
    let tree = engine.build_render_tree(
        &page_content,
        &[],
        &[],
        &[],
        &[],
        &styles,
        &FootnoteShape::default(),
        &[],
        None,
        &[],
        None,
        0,
        &[],
    );
    // 페이지 노드 + 배경 + 머리말 + 본문 + 각주 + 꼬리말
    assert!(tree.root.children.len() >= 4);
}

#[test]
fn test_build_page_with_paragraph() {
    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(&a4_page_def(), &ColumnDef::default());

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
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(
        &page_content,
        &paragraphs,
        &paragraphs,
        &paragraphs,
        &composed,
        &styles,
        &FootnoteShape::default(),
        &[],
        None,
        &[],
        None,
        0,
        &[],
    );
    assert!(tree.needs_render());

    // Body 노드 찾기
    let body = tree
        .root
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }));
    assert!(body.is_some());
    let body = body.unwrap();
    // Column 노드가 있어야 함
    assert!(!body.children.is_empty());
}

#[test]
fn test_layout_with_composed_styles() {
    use crate::renderer::style_resolver::ResolvedCharStyle;

    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(&a4_page_def(), &ColumnDef::default());

    let paragraphs = vec![Paragraph {
        text: "AAABBB".to_string(),
        char_offsets: vec![0, 1, 2, 3, 4, 5],
        char_count: 7,
        char_shapes: vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            },
            CharShapeRef {
                start_pos: 3,
                char_shape_id: 1,
            },
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
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(
        &page_content,
        &paragraphs,
        &paragraphs,
        &paragraphs,
        &composed,
        &styles,
        &FootnoteShape::default(),
        &[],
        None,
        &[],
        None,
        0,
        &[],
    );

    // Body > Column > TextLine 찾기
    let body = tree
        .root
        .children
        .iter()
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
    let layout = PageLayoutInfo::from_page_def_default(&a4_page_def(), &ColumnDef::default());

    let paragraphs = vec![Paragraph {
        text: "AB가나".to_string(),
        char_offsets: vec![0, 1, 2, 3],
        char_count: 5,
        char_shapes: vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            },
            CharShapeRef {
                start_pos: 2,
                char_shape_id: 1,
            },
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
            ResolvedCharStyle {
                font_size: 16.0,
                ..Default::default()
            },
            ResolvedCharStyle {
                font_size: 16.0,
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
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(
        &page_content,
        &paragraphs,
        &paragraphs,
        &paragraphs,
        &composed,
        &styles,
        &FootnoteShape::default(),
        &[],
        None,
        &[],
        None,
        0,
        &[],
    );

    let body = tree
        .root
        .children
        .iter()
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
    use crate::model::style::UnderlineType;
    use crate::renderer::style_resolver::ResolvedCharStyle;

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
    let style = TextStyle {
        font_size: 16.0,
        ..Default::default()
    };

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
    let style = TextStyle {
        font_size: 16.0,
        ratio: 0.8,
        ..Default::default()
    };
    let w = estimate_text_width("가나", &style);
    // base: 2 * 16.0 = 32.0, * 0.8 = 25.6 → round = 26.0
    assert!((w - 26.0).abs() < 0.01);

    // 장평 150%
    let style = TextStyle {
        font_size: 16.0,
        ratio: 1.5,
        ..Default::default()
    };
    let w = estimate_text_width("AB", &style);
    // base: 2 * 8.0 = 16.0, * 1.5 = 24.0
    assert!((w - 24.0).abs() < 0.01);

    // 장평 100%: 기존과 동일
    let style = TextStyle {
        font_size: 16.0,
        ratio: 1.0,
        ..Default::default()
    };
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
    let style = TextStyle {
        font_size: 16.0,
        ..Default::default()
    };
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
    let style_base = TextStyle {
        font_size: 16.0,
        ..Default::default()
    };
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
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};
    use crate::renderer::style_resolver::ResolvedBorderStyle;

    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(&a4_page_def(), &ColumnDef::default());

    // 2x2 표가 있는 문단 (각 셀에 border_fill_id=1 설정)
    let table = Table {
        row_count: 2,
        col_count: 2,
        row_sizes: vec![2, 2], // 행별 셀 수
        cells: vec![
            Cell {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                width: 3000,
                height: 1200,
                border_fill_id: 1,
                paragraphs: vec![Paragraph {
                    text: "A".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Cell {
                col: 1,
                row: 0,
                col_span: 1,
                row_span: 1,
                width: 3000,
                height: 1200,
                border_fill_id: 1,
                paragraphs: vec![Paragraph {
                    text: "B".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Cell {
                col: 0,
                row: 1,
                col_span: 1,
                row_span: 1,
                width: 3000,
                height: 1200,
                border_fill_id: 1,
                paragraphs: vec![Paragraph {
                    text: "C".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            Cell {
                col: 1,
                row: 1,
                col_span: 1,
                row_span: 1,
                width: 3000,
                height: 1200,
                border_fill_id: 1,
                paragraphs: vec![Paragraph {
                    text: "D".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let paragraphs = vec![Paragraph {
        text: String::new(),
        controls: vec![Control::Table(Box::new(table))],
        line_segs: vec![LineSeg {
            line_height: 400,
            ..Default::default()
        }],
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
                PageItem::Table {
                    para_index: 0,
                    control_index: 0,
                },
            ],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(
        &page_content,
        &paragraphs,
        &paragraphs,
        &paragraphs,
        &composed,
        &styles,
        &FootnoteShape::default(),
        &[],
        None,
        &[],
        None,
        0,
        &[],
    );

    // Body > Column 내에 Table 노드가 있어야 함
    let body = tree
        .root
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let col = &body.children[0];

    let table_node = col
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Table(_)))
        .expect("Table node should exist");

    // 4개 셀 + 엣지 기반 테두리 Line 노드들
    let cell_count = table_node
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::TableCell(_)))
        .count();
    assert_eq!(cell_count, 4);

    // 엣지 기반 테두리: 표 노드의 직접 자식으로 Line 노드가 있어야 함
    // 2x2 표: 수평 3줄 + 수직 3줄 = 6개 이상의 Line 노드
    // (기본 Solid 테두리이므로 이중선/삼중선이 아니면 각 엣지당 1개)
    let table_line_count = table_node
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::Line(_)))
        .count();
    assert!(
        table_line_count >= 6,
        "표에 6개 이상의 엣지 테두리가 있어야 함 (실제: {})",
        table_line_count
    );
}

#[test]
fn test_layout_table_cell_positions() {
    use crate::model::control::Control;
    use crate::model::table::{Cell, Table};

    let engine = LayoutEngine::with_default_dpi();
    let layout = PageLayoutInfo::from_page_def_default(&a4_page_def(), &ColumnDef::default());

    let table = Table {
        row_count: 2,
        col_count: 2,
        row_sizes: vec![2, 2], // 행별 셀 수
        cells: vec![
            Cell {
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
                width: 3600,
                height: 720,
                ..Default::default()
            },
            Cell {
                col: 1,
                row: 0,
                col_span: 1,
                row_span: 1,
                width: 3600,
                height: 720,
                ..Default::default()
            },
            Cell {
                col: 0,
                row: 1,
                col_span: 1,
                row_span: 1,
                width: 3600,
                height: 720,
                ..Default::default()
            },
            Cell {
                col: 1,
                row: 1,
                col_span: 1,
                row_span: 1,
                width: 3600,
                height: 720,
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let paragraphs = vec![Paragraph {
        text: String::new(),
        controls: vec![Control::Table(Box::new(table))],
        line_segs: vec![LineSeg {
            line_height: 400,
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
            items: vec![
                PageItem::FullParagraph { para_index: 0 },
                PageItem::Table {
                    para_index: 0,
                    control_index: 0,
                },
            ],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };

    let tree = engine.build_render_tree(
        &page_content,
        &paragraphs,
        &paragraphs,
        &paragraphs,
        &composed,
        &styles,
        &FootnoteShape::default(),
        &[],
        None,
        &[],
        None,
        0,
        &[],
    );

    let body = tree
        .root
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .unwrap();
    let col = &body.children[0];
    let table_node = col
        .children
        .iter()
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
        heads: [NumberingHead {
            number_format: 0,
            ..Default::default()
        }; 7],
        level_formats: [
            "^1.".to_string(),
            "^2.".to_string(),
            "^3)".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ],
        start_number: 0,
        level_start_numbers: [1, 1, 1, 1, 1, 1, 1],
    };
    let counters = [3, 2, 1, 0, 0, 0, 0];
    let result =
        expand_numbering_format("^1.", &counters, &numbering, &numbering.level_start_numbers);
    assert_eq!(result, "3.");

    let result =
        expand_numbering_format("^2.", &counters, &numbering, &numbering.level_start_numbers);
    assert_eq!(result, "2.");

    let result = expand_numbering_format(
        "(^3)",
        &counters,
        &numbering,
        &numbering.level_start_numbers,
    );
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
            String::new(),
            "^2.".to_string(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ],
        start_number: 0,
        level_start_numbers: [1, 1, 1, 1, 1, 1, 1],
    };
    let counters = [1, 3, 0, 0, 0, 0, 0];
    let result =
        expand_numbering_format("^2.", &counters, &numbering, &numbering.level_start_numbers);
    assert_eq!(result, "다.");
}

#[test]
fn test_numbering_format_to_number_format() {
    assert!(matches!(
        numbering_format_to_number_format(0),
        NumFmt::Digit
    ));
    assert!(matches!(
        numbering_format_to_number_format(1),
        NumFmt::CircledDigit
    ));
    assert!(matches!(
        numbering_format_to_number_format(2),
        NumFmt::RomanUpper
    ));
    assert!(matches!(
        numbering_format_to_number_format(8),
        NumFmt::HangulGaNaDa
    ));
    assert!(matches!(
        numbering_format_to_number_format(255),
        NumFmt::Digit
    ));
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
