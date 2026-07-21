use super::*;
use crate::model::document::{Section, SectionDef};
use crate::model::page::PageDef;
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::model::shape::{GroupShape, RectangleShape};
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
        controls: vec![Control::Shape(Box::new(ShapeObject::Picture(Box::new(
            pic,
        ))))],
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

/// [#2696] OLE 의 `SHAPE_COMPONENT` 는 **의도적으로** base-only 다.
///
/// 최초 조사에서 "Chart 형제 arm 은 `serialize_drawing_shape_component` 로
/// DrawingObjAttr 전체를 기록하는데 OLE 만 base-only 이므로 비대칭 결함" 이라고
/// 판단했으나, 한컴 저장본 `samples/143E433F503322BD33.hwp` 를 실측한 결과
/// `$ole` ctrl id 를 가진 `SHAPE_COMPONENT` 는 **196B(base-only)** 였다. 같은
/// 파일에서 테두리/채우기/그림자 꼬리를 가진 252B 레코드는 도형 쪽이다.
/// 즉 base-only 가 한컴의 실제 OLE 포맷이며, `#1283` 이 한컴 읽기 오류를
/// 잡으면서 확정한 계약이다(`tests/issue_1251_ole_chart_contents.rs` 가
/// `shape_component.size == 196` 으로 고정).
///
/// 파서가 `parse_shape_component_full` 로 꼬리를 읽을 수 있는 것은 관대함이지
/// 직렬화 의무가 아니다. 같은 오판이 반복되지 않도록 계약을 여기에 못박는다.
#[test]
fn issue2696_ole_shape_component_stays_base_only() {
    let base = serialize_shape_component(tags::SHAPE_OLE_ID, &ShapeComponentAttr::default(), true);
    let full =
        serialize_drawing_shape_component(tags::SHAPE_OLE_ID, &DrawingObjAttr::default(), true);
    assert!(
        full.len() > base.len(),
        "전제 확인: 전체 직렬화가 base 보다 길어야 이 계약이 의미를 가짐"
    );
    assert_eq!(
        base.len(),
        196,
        "[#2696] OLE SHAPE_COMPONENT 는 한컴 실측치와 같은 196B(base-only)여야 한다 \
         — 꼬리를 붙이면 #1283 의 한컴 호환 계약이 깨진다"
    );
}

// ============================================================
// [#2715] 그리기 도형·묶음·차트 캡션 HWP5 직렬화
// ============================================================

/// 캡션 1개짜리 테스트 픽스처.
fn caption_fixture(text: &str) -> Caption {
    Caption {
        direction: CaptionDirection::Top,
        vert_align: CaptionVertAlign::Center,
        width: 4321,
        spacing: 850,
        max_width: 30000,
        include_margin: true,
        paragraphs: vec![Paragraph {
            char_count: (text.chars().count() + 1) as u32,
            text: text.to_string(),
            char_offsets: (0..text.chars().count() as u32).collect(),
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
    }
}

/// 컨트롤 1개를 담은 섹션을 직렬화 → 재파싱한다.
fn roundtrip_single_control(ctrl: Control) -> Control {
    let section = Section {
        paragraphs: vec![Paragraph {
            char_count: 2,
            text: String::new(),
            char_offsets: vec![],
            controls: vec![ctrl],
            ..Default::default()
        }],
        raw_stream: None,
        ..Default::default()
    };
    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).unwrap();
    parsed.paragraphs[0]
        .controls
        .first()
        .expect("컨트롤이 왕복돼야 함")
        .clone()
}

fn shape_of(ctrl: &Control) -> &ShapeObject {
    match ctrl {
        Control::Shape(s) => s.as_ref(),
        other => panic!("도형 컨트롤이 나와야 함, got {other:?}"),
    }
}

/// [#2715] 사각형 도형의 캡션이 HWP5 왕복에서 보존돼야 한다.
///
/// 종전에는 `serialize_shape_control` 의 어떤 arm 도 `serialize_caption` 을
/// 호출하지 않아 `drawing.caption` 이 통째로 사라졌다 — 한컴 저장본
/// `samples/3-09월_교육_통합_2023.hwp` 는 `$rec` 도형에 그림($pic)과 동일한
/// 30B LIST_HEADER 캡션을 쓰므로 포맷 제약이 아니라 미구현이었다.
#[test]
fn issue2715_rectangle_caption_roundtrips() {
    let rect = RectangleShape {
        drawing: DrawingObjAttr {
            caption: Some(caption_fixture("사각형 캡션")),
            ..Default::default()
        },
        ..Default::default()
    };
    let out = roundtrip_single_control(Control::Shape(Box::new(ShapeObject::Rectangle(rect))));

    let ShapeObject::Rectangle(r) = shape_of(&out) else {
        panic!("사각형이 나와야 함, got {:?}", shape_of(&out));
    };
    let cap = r
        .drawing
        .caption
        .as_ref()
        .expect("[#2715] 사각형 캡션이 왕복 보존돼야 함");
    assert_eq!(cap.paragraphs[0].text, "사각형 캡션", "캡션 텍스트 보존");
    assert_eq!(cap.direction, CaptionDirection::Top, "캡션 방향 보존");
    assert_eq!(
        cap.vert_align,
        CaptionVertAlign::Center,
        "캡션 세로 정렬 보존"
    );
    assert_eq!(cap.width, 4321, "캡션 폭 보존");
    assert_eq!(cap.spacing, 850, "캡션-틀 간격 보존");
    assert_eq!(cap.max_width, 30000, "캡션 최대 폭 보존");
    assert!(cap.include_margin, "include_margin 보존");
}

/// [#2715] 묶음(`$con`) 캡션 왕복 — 한컴 `samples/draw-group.hwp` 실측 대응.
#[test]
fn issue2715_group_caption_roundtrips() {
    let group = GroupShape {
        caption: Some(caption_fixture("묶음 캡션")),
        ..Default::default()
    };
    let out = roundtrip_single_control(Control::Shape(Box::new(ShapeObject::Group(group))));

    let ShapeObject::Group(g) = shape_of(&out) else {
        panic!("묶음이 나와야 함, got {:?}", shape_of(&out));
    };
    assert_eq!(
        g.caption
            .as_ref()
            .expect("[#2715] 묶음 캡션이 왕복 보존돼야 함")
            .paragraphs[0]
            .text,
        "묶음 캡션"
    );
}

/// [#2715] 캡션은 `SHAPE_COMPONENT` **앞**, 글상자 LIST_HEADER 는 **뒤**로
/// 분리 방출돼야 한다 (파서가 위치로 둘을 구분하므로 순서가 곧 계약이다).
/// LIST_HEADER 크기 30B 는 한컴 저장본 실측치와 같다.
#[test]
fn issue2715_caption_precedes_shape_component_and_textbox_follows() {
    let rect = RectangleShape {
        drawing: DrawingObjAttr {
            caption: Some(caption_fixture("캡션")),
            text_box: Some(crate::model::shape::TextBox {
                paragraphs: vec![Paragraph {
                    char_count: 4,
                    text: "글상자".to_string(),
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
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    let ctrl = Control::Shape(Box::new(ShapeObject::Rectangle(rect)));

    let mut records: Vec<Record> = Vec::new();
    serialize_control(&ctrl, 1, None, &mut records);

    let comp_idx = records
        .iter()
        .position(|r| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT)
        .expect("SHAPE_COMPONENT 가 있어야 함");
    let caption_idx = records
        .iter()
        .position(|r| r.tag_id == tags::HWPTAG_LIST_HEADER)
        .expect("[#2715] 캡션 LIST_HEADER 가 방출돼야 함");
    assert!(
        caption_idx < comp_idx,
        "캡션 LIST_HEADER 는 SHAPE_COMPONENT 앞이어야 함 (caption={caption_idx}, comp={comp_idx})"
    );
    assert_eq!(
        records[caption_idx].level, 2,
        "캡션 LIST_HEADER 는 level+1 이어야 함"
    );
    assert_eq!(
        records[caption_idx].data.len(),
        30,
        "캡션 LIST_HEADER 는 한컴 실측치와 같은 30B 여야 함"
    );

    let textbox_idx = records
        .iter()
        .enumerate()
        .skip(comp_idx)
        .find(|(_, r)| r.tag_id == tags::HWPTAG_LIST_HEADER)
        .map(|(i, _)| i)
        .expect("글상자 LIST_HEADER 가 방출돼야 함");
    assert!(
        textbox_idx > comp_idx,
        "글상자 LIST_HEADER 는 SHAPE_COMPONENT 뒤여야 함"
    );

    // 재파싱 시 캡션과 글상자가 각각 제자리에 복원되는지
    let out = roundtrip_single_control(ctrl);
    let ShapeObject::Rectangle(r) = shape_of(&out) else {
        panic!("사각형이 나와야 함");
    };
    assert_eq!(
        r.drawing.caption.as_ref().expect("캡션 보존").paragraphs[0].text,
        "캡션"
    );
    assert_eq!(
        r.drawing.text_box.as_ref().expect("글상자 보존").paragraphs[0].text,
        "글상자",
        "캡션 추가가 글상자 경로를 침범하면 안 됨"
    );
}

/// [#2715] 캡션이 없는 도형은 LIST_HEADER 를 방출하지 않아야 한다
/// (불필요한 레코드 추가로 기존 레코드 시퀀스를 흔들지 않기 위함).
#[test]
fn issue2715_shape_without_caption_emits_no_list_header() {
    let ctrl = Control::Shape(Box::new(ShapeObject::Rectangle(RectangleShape::default())));
    let mut records: Vec<Record> = Vec::new();
    serialize_control(&ctrl, 1, None, &mut records);
    assert!(
        !records.iter().any(|r| r.tag_id == tags::HWPTAG_LIST_HEADER),
        "캡션 없는 도형은 LIST_HEADER 를 방출하면 안 됨"
    );
}
