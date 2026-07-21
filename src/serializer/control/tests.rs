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
        // [Task #1050] CTRL_FOOTNOTE 한컴 default
        after_decoration_letter: 0x0029,
        ..Default::default()
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

#[test]
fn footnote_after_decoration_zero_is_not_forced_to_paren() {
    use crate::model::footnote::Footnote;
    // 닫는 장식이 없는(after_decoration_letter=0) 각주는 저장 후에도 0 이어야 한다.
    // 종전엔 serializer 가 0 을 ')'(0x0029)로 치환해 오염됐다.
    let fn_ = Footnote {
        number: 1,
        before_decoration_letter: 0,
        after_decoration_letter: 0,
        paragraphs: vec![Paragraph {
            char_count: 3,
            text: "주".to_string(),
            char_offsets: vec![0],
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
    let Some(Control::Footnote(fn_)) = parsed.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::Footnote(_)))
    else {
        panic!("Expected Footnote control");
    };
    assert_eq!(
        fn_.after_decoration_letter, 0,
        "닫는 장식 없음(0)이 ')'(0x0029)로 오염되면 안 됨"
    );
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
    use crate::model::shape::{CommonObjAttr, GroupShape, ShapeComponentAttr, ShapeObject};

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
                assert_eq!(
                    p.image_attr.bin_data_id, 7,
                    "bin_data_id should survive roundtrip"
                );
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

#[test]
fn issue1452_picture_transparency_updates_hwp_extra_byte() {
    let mut pic = Picture::default();
    pic.crop.right = 1000;
    pic.crop.bottom = 500;
    pic.image_attr.transparency = 50;

    let bytes = serialize_picture_data(&pic);
    assert_eq!(
        bytes.last().copied(),
        Some(127),
        "HWP 그림 추가 속성의 마지막 alpha byte는 50% 투명도에서 127이어야 한다"
    );

    pic.raw_picture_extra = vec![0; 18];
    pic.image_attr.transparency = 100;
    let bytes = serialize_picture_data(&pic);
    assert_eq!(
        bytes.last().copied(),
        Some(255),
        "원본 raw_picture_extra가 있어도 마지막 alpha byte는 현재 투명도와 동기화되어야 한다"
    );
}

#[test]
fn picture_border_attr_word_serialized_from_ir() {
    // 그림 테두리 속성 워드(선 종류/끝모양 비트)가 IR 에서 방출돼야 한다.
    // 레이아웃: border_color(4) + border_width(4) + border_attr(4).
    // 종전엔 이 워드를 0 으로 고정 방출해 스타일 테두리가 저장 시 유실됐다.
    let mut pic = Picture::default();
    pic.border_attr.attr = 0x0000_00A5;
    let bytes = serialize_picture_data(&pic);
    assert_eq!(
        &bytes[8..12],
        &0x0000_00A5u32.to_le_bytes(),
        "그림 테두리 속성 워드가 IR(border_attr.attr)에서 방출돼야 함"
    );
}

/// [#1808] 셀 field_name 이 raw_list_extra 한컴 계약 레이아웃으로 기록되고
/// 파서 추출(parse_cell_field_name)과 대칭인지 검증.
#[test]
fn test_cell_field_name_extra_roundtrip() {
    let cell = crate::model::table::Cell {
        width: 23984,
        field_name: Some("발신명의".to_string()),
        ..Default::default()
    };
    let extra = build_cell_list_extra(&cell);
    // 레이아웃: width(4) + 마커(8) + 40 01 00(3) + name_len(2) + UTF-16LE(2n) + 0×8
    let n = "발신명의".encode_utf16().count();
    assert_eq!(extra.len(), 25 + n * 2);
    assert_eq!(&extra[0..4], &23984u32.to_le_bytes());
    assert_eq!(&extra[4..8], &[0xff, 0x1b, 0x02, 0x01]);
    assert_eq!(
        crate::parser::control::parse_cell_field_name(&extra).as_deref(),
        Some("발신명의")
    );

    // 필드 없는 셀은 기존 13바이트 default 유지
    let plain = crate::model::table::Cell {
        width: 100,
        ..Default::default()
    };
    let extra = build_cell_list_extra(&plain);
    assert_eq!(extra.len(), 13);
    assert_eq!(crate::parser::control::parse_cell_field_name(&extra), None);
}

/// [#2696] OLE 의 SHAPE_COMPONENT 가 DrawingObjAttr 전체를 기록하는지.
///
/// 종전에는 base-only `serialize_shape_component` 를 호출해 테두리(13B) + 채우기 +
/// 그림자(16B) + inst_id/shadow_alpha(6B) 가 빠졌고, 재파싱 시
/// `parse_shape_component_full` 의 `remaining()` 가드가 전부 실패해 조용히 기본값이 됐다.
#[test]
fn issue2696_ole_shape_component_keeps_border_fill_shadow() {
    use crate::model::style::SolidFill;

    let drawing = DrawingObjAttr {
        shape_attr: ShapeComponentAttr {
            original_width: 7200,
            original_height: 7200,
            current_width: 7200,
            current_height: 7200,
            ..Default::default()
        },
        border_line: ShapeBorderLine {
            color: 0x123456,
            width: 300,
            attr: 0x5,
            outline_style: 1,
        },
        fill: Fill {
            fill_type: FillType::Solid,
            solid: Some(SolidFill {
                background_color: 0x00FF00,
                pattern_color: 0x0000FF,
                pattern_type: -1,
            }),
            ..Default::default()
        },
        shadow_type: 2,
        shadow_color: 0x808080,
        shadow_offset_x: 141,
        shadow_offset_y: 282,
        inst_id: 0x0000_ABCD,
        shadow_alpha: 0x80,
        ..Default::default()
    };

    let ole = OleShape {
        common: CommonObjAttr {
            width: 7200,
            height: 7200,
            ..Default::default()
        },
        drawing,
        extent_x: 7200,
        extent_y: 7200,
        bin_data_id: 1,
        ..Default::default()
    };

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::Shape(Box::new(ShapeObject::Ole(Box::new(ole))))],
        ..Default::default()
    };
    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    let Control::Shape(shape) = &parsed.paragraphs[0].controls[0] else {
        panic!("Shape 컨트롤이 나와야 함");
    };
    let ShapeObject::Ole(reparsed) = shape.as_ref() else {
        panic!("Ole 도형이 나와야 함, got {:?}", shape);
    };
    let d = &reparsed.drawing;

    assert_eq!(d.border_line.color, 0x123456, "테두리 색이 보존돼야 함");
    assert_eq!(d.border_line.width, 300, "테두리 굵기가 보존돼야 함");
    assert_eq!(d.border_line.attr, 0x5, "테두리 속성 워드가 보존돼야 함");
    assert_eq!(d.border_line.outline_style, 1, "아웃라인 스타일이 보존돼야 함");

    assert_eq!(d.fill.fill_type, FillType::Solid, "단색 채우기가 보존돼야 함");
    let solid = d.fill.solid.expect("단색 채우기 본문이 있어야 함");
    assert_eq!(solid.background_color, 0x00FF00, "채우기 배경색이 보존돼야 함");
    assert_eq!(solid.pattern_color, 0x0000FF, "채우기 무늬색이 보존돼야 함");
    assert_eq!(solid.pattern_type, -1, "채우기 무늬 종류가 보존돼야 함");

    assert_eq!(d.shadow_type, 2, "그림자 종류가 보존돼야 함");
    assert_eq!(d.shadow_color, 0x808080, "그림자 색이 보존돼야 함");
    assert_eq!(d.shadow_offset_x, 141, "그림자 가로 오프셋이 보존돼야 함");
    assert_eq!(d.shadow_offset_y, 282, "그림자 세로 오프셋이 보존돼야 함");

    assert_eq!(d.inst_id, 0x0000_ABCD, "inst_id 가 보존돼야 함");
    assert_eq!(d.shadow_alpha, 0x80, "그림자 투명도가 보존돼야 함");
}

/// [#2696] 최상위 `ShapeObject::Picture` 가 실제로 직렬화되는지.
///
/// 그룹 해제(`ungroup_shape_native`)는 그림 자식을 최상위
/// `Control::Shape(ShapeObject::Picture)` 로 삽입한다. 종전에는 이 arm 이 아무 레코드도
/// 방출하지 않아 그림이 통째로 사라졌다.
#[test]
fn issue2696_top_level_shape_picture_is_serialized() {
    let pic = Picture {
        common: CommonObjAttr {
            width: 5000,
            height: 3000,
            ..Default::default()
        },
        shape_attr: ShapeComponentAttr {
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

    let para = Paragraph {
        char_count: 2,
        text: "".to_string(),
        char_offsets: vec![],
        controls: vec![Control::Shape(Box::new(ShapeObject::Picture(Box::new(pic))))],
        ..Default::default()
    };
    let section = Section {
        paragraphs: vec![para],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();

    assert_eq!(
        parsed.paragraphs[0].controls.len(),
        1,
        "최상위 ShapeObject::Picture 가 컨트롤 1개로 왕복돼야 함"
    );
    let bin_data_id = match &parsed.paragraphs[0].controls[0] {
        Control::Picture(p) => p.image_attr.bin_data_id,
        Control::Shape(s) => match s.as_ref() {
            ShapeObject::Picture(p) => p.image_attr.bin_data_id,
            other => panic!("그림 도형이 나와야 함, got {:?}", other),
        },
        _ => panic!("그림 컨트롤이 나와야 함"),
    };
    assert_eq!(bin_data_id, 7, "bin_data_id 가 왕복 보존돼야 함");
}

/// [#2696] 최상위 `ShapeObject::Picture` 가 CTRL_HEADER 를 정확히 1개 방출하는지.
///
/// 그룹 해제는 그림 1개당 `char_count += 8`(확장 컨트롤 문자)을 함께 적용한다
/// (`document_core/commands/object_ops/shape.rs:2317-2321`). CTRL_HEADER 가 0개면
/// PARA_TEXT 의 컨트롤 문자와 레코드 개수가 어긋나 **이후 컨트롤이 잘못된 문자 위치에
/// 결합**된다. 그림 유실보다 이 짝 어긋남이 더 위험하므로 개수를 별도로 고정한다.
#[test]
fn issue2696_top_level_shape_picture_emits_exactly_one_ctrl_header() {
    let pic = Picture {
        image_attr: crate::model::image::ImageAttr {
            bin_data_id: 7,
            ..Default::default()
        },
        ..Default::default()
    };
    let ctrl = Control::Shape(Box::new(ShapeObject::Picture(Box::new(pic))));

    let mut records: Vec<Record> = Vec::new();
    serialize_control(&ctrl, 1, None, &mut records);

    let ctrl_headers = records
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_CTRL_HEADER)
        .count();
    assert_eq!(
        ctrl_headers, 1,
        "최상위 그림 1개는 CTRL_HEADER 를 정확히 1개 방출해야 함 (char_count += 8 과 1:1)"
    );
    assert!(
        records
            .iter()
            .any(|r| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT_PICTURE),
        "SHAPE_COMPONENT_PICTURE 레코드가 함께 방출돼야 함"
    );
}
