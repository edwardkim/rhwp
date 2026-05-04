use super::*;
use crate::model::document::{Section, SectionDef};
use crate::model::page::PageDef;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::parser::body_text::parse_body_text_section;
use crate::serializer::body_text::serialize_section;

/// SectionDef 라운드트립
#[test]
fn test_roundtrip_section_def() {
    let sd = SectionDef {
        flags: 0,
        default_tab_spacing: 800,
        page_num: 1,
        page_def: PageDef {
            width: 59528,
            height: 84188,
            margin_left: 8504,
            margin_right: 8504,
            margin_top: 5669,
            margin_bottom: 4252,
            margin_header: 4252,
            margin_footer: 4252,
            ..Default::default()
        },
        ..Default::default()
    };

    let para = Paragraph {
        char_count: 3,
        text: "A".to_string(),
        char_offsets: vec![8], // 0~7 = secd 컨트롤
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            text_start: 0,
            ..Default::default()
        }],
        controls: vec![Control::SectionDef(Box::new(sd))],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    assert_eq!(parsed.section_def.default_tab_spacing, 800);
    assert_eq!(parsed.section_def.page_num, 1);
    assert_eq!(parsed.section_def.page_def.width, 59528);
    assert_eq!(parsed.section_def.page_def.height, 84188);
}

/// ColumnDef 라운드트립
#[test]
fn test_roundtrip_column_def() {
    let cd = ColumnDef {
        column_type: ColumnType::Normal,
        column_count: 2,
        same_width: true,
        spacing: 1000,
        ..Default::default()
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::ColumnDef(cd)],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    let has_cold = parsed.paragraphs[0]
        .controls
        .iter()
        .any(|c| matches!(c, Control::ColumnDef(_)));
    assert!(has_cold);

    if let Some(Control::ColumnDef(cd)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::ColumnDef(_)))
    {
        assert_eq!(cd.column_count, 2);
        assert!(cd.same_width);
        assert_eq!(cd.spacing, 1000);
    }
}

/// Table 라운드트립
#[test]
fn test_roundtrip_table() {
    let cell = Cell {
        col: 0,
        row: 0,
        col_span: 1,
        row_span: 1,
        width: 10000,
        height: 5000,
        border_fill_id: 1,
        paragraphs: vec![Paragraph {
            char_count: 5,
            text: "test".to_string(),
            char_offsets: vec![0, 1, 2, 3],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    let table = Table {
        row_count: 1,
        col_count: 1,
        cell_spacing: 0,
        row_sizes: vec![1], // 행별 셀 수
        border_fill_id: 1,
        cells: vec![cell],
        ..Default::default()
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    let has_table = parsed.paragraphs[0]
        .controls
        .iter()
        .any(|c| matches!(c, Control::Table(_)));
    assert!(has_table);

    if let Some(Control::Table(t)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::Table(_)))
    {
        assert_eq!(t.row_count, 1);
        assert_eq!(t.col_count, 1);
        assert_eq!(t.cells.len(), 1);
        assert_eq!(t.cells[0].width, 10000);
        assert_eq!(t.cells[0].paragraphs[0].text, "test");
    }
}

/// AutoNumber 라운드트립
#[test]
fn test_roundtrip_auto_number() {
    let an = AutoNumber {
        number_type: AutoNumberType::Table,
        format: 0,
        superscript: false,
        ..Default::default()
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::AutoNumber(an)],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    if let Some(Control::AutoNumber(an)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::AutoNumber(_)))
    {
        assert_eq!(an.number_type, AutoNumberType::Table);
    } else {
        panic!("Expected AutoNumber control");
    }
}

/// Bookmark 라운드트립
#[test]
fn test_roundtrip_bookmark() {
    let bm = Bookmark {
        name: "테스트".to_string(),
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::Bookmark(bm)],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    if let Some(Control::Bookmark(bm)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::Bookmark(_)))
    {
        assert_eq!(bm.name, "테스트");
    } else {
        panic!("Expected Bookmark control");
    }
}

/// PageHide 라운드트립
#[test]
fn test_roundtrip_page_hide() {
    let ph = PageHide {
        hide_header: true,
        hide_footer: true,
        hide_master_page: false,
        hide_border: false,
        hide_fill: false,
        hide_page_num: true,
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::PageHide(ph)],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    if let Some(Control::PageHide(ph)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::PageHide(_)))
    {
        assert!(ph.hide_header);
        assert!(ph.hide_footer);
        assert!(!ph.hide_master_page);
        assert!(ph.hide_page_num);
    } else {
        panic!("Expected PageHide control");
    }
}

/// Footnote 라운드트립
#[test]
fn test_roundtrip_footnote() {
    use crate::model::footnote::Footnote;

    let fn_ = Footnote {
        number: 3,
        paragraphs: vec![Paragraph {
            char_count: 3,
            text: "각주".to_string(),
            char_offsets: vec![0, 1],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::Footnote(Box::new(fn_))],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    if let Some(Control::Footnote(fn_)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::Footnote(_)))
    {
        assert_eq!(fn_.number, 3);
        assert_eq!(fn_.paragraphs.len(), 1);
        assert_eq!(fn_.paragraphs[0].text, "각주");
    } else {
        panic!("Expected Footnote control");
    }
}

/// Header 라운드트립
#[test]
fn test_roundtrip_header() {
    use crate::model::header_footer::Header;

    let header = Header {
        apply_to: HeaderFooterApply::Both,
        paragraphs: vec![Paragraph {
            char_count: 4,
            text: "머리말".to_string(),
            char_offsets: vec![0, 1, 2],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::Header(Box::new(header))],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    if let Some(Control::Header(h)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::Header(_)))
    {
        assert_eq!(h.apply_to, HeaderFooterApply::Both);
        assert_eq!(h.paragraphs.len(), 1);
        assert_eq!(h.paragraphs[0].text, "머리말");
    } else {
        panic!("Expected Header control");
    }
}

/// 그룹 내 Picture 자식 라운드트립 (#428 후속)
#[test]
fn test_roundtrip_group_picture_child() {
    use crate::model::image::Picture;
    use crate::model::shape::{
        CommonObjAttr, GroupShape, ShapeComponentAttr, ShapeObject,
    };

    let pic = Picture {
        common: CommonObjAttr::default(),
        shape_attr: ShapeComponentAttr {
            group_level: 1,
            original_width: 5000,
            original_height: 3000,
            current_width: 5000,
            current_height: 3000,
            ..Default::default()
        },
        image_attr: crate::model::image::ImageAttr {
            bin_data_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };

    let group = GroupShape {
        common: CommonObjAttr {
            width: 10000,
            height: 8000,
            ..Default::default()
        },
        shape_attr: ShapeComponentAttr {
            original_width: 10000,
            original_height: 8000,
            current_width: 10000,
            current_height: 8000,
            ..Default::default()
        },
        children: vec![ShapeObject::Picture(Box::new(pic))],
        caption: None,
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::Shape(Box::new(ShapeObject::Group(group)))],
        ..Default::default()
    };

    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    assert_eq!(parsed.paragraphs.len(), 1);
    let ctrl = &parsed.paragraphs[0].controls[0];
    if let Control::Shape(shape) = ctrl {
        if let ShapeObject::Group(g) = shape.as_ref() {
            assert_eq!(g.children.len(), 1, "Group should have 1 child");
            if let ShapeObject::Picture(p) = &g.children[0] {
                assert_eq!(p.image_attr.bin_data_id, 7, "bin_data_id should survive roundtrip");
                assert_eq!(p.shape_attr.original_width, 5000);
                assert_eq!(p.shape_attr.original_height, 3000);
            } else {
                panic!("Expected Picture child, got {:?}", g.children[0]);
            }
        } else {
            panic!("Expected Group shape");
        }
    } else {
        panic!("Expected Shape control");
    }
}
