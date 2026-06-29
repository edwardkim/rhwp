use super::*;
use crate::model::bin_data::{BinDataCompression, BinDataStatus};
use crate::model::style::{
    Alignment, BorderLine, CenterLine, DiagonalLine, Fill, ImageFill, ImageFillMode,
    LineSpacingType, NumberingHead, SolidFill,
};
use crate::parser::doc_info::parse_doc_info;
use crate::parser::record::Record;
use crate::parser::tags;

#[test]
fn test_serialize_document_properties() {
    let props = DocProperties {
        section_count: 2,
        page_start_num: 1,
        footnote_start_num: 3,
        endnote_start_num: 4,
        picture_start_num: 5,
        table_start_num: 6,
        equation_start_num: 7,
        raw_data: None,
        caret_list_id: 0,
        caret_para_id: 0,
        caret_char_pos: 0,
    };

    let data = serialize_document_properties(&props);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    assert_eq!(r.read_u16().unwrap(), 2);
    assert_eq!(r.read_u16().unwrap(), 1);
    assert_eq!(r.read_u16().unwrap(), 3);
    assert_eq!(r.read_u16().unwrap(), 4);
    assert_eq!(r.read_u16().unwrap(), 5);
    assert_eq!(r.read_u16().unwrap(), 6);
    assert_eq!(r.read_u16().unwrap(), 7);
}

#[test]
fn test_serialize_id_mappings_uses_modern_count_table_size() {
    let mut doc_info = DocInfo::default();
    doc_info.font_faces = vec![Vec::new(); 7];
    doc_info.memo_shape_count = 1;

    let data = serialize_id_mappings(&doc_info);
    assert_eq!(data.len(), 72);

    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    for _ in 0..15 {
        r.read_u32().unwrap();
    }
    assert_eq!(r.read_u32().unwrap(), 1);
    assert_eq!(r.read_u32().unwrap(), 0);
    assert_eq!(r.read_u32().unwrap(), 0);
}

#[test]
fn test_serialize_face_name_simple() {
    let font = Font {
        raw_data: None,
        name: "함초롬바탕".to_string(),
        alt_type: 0,
        alt_name: None,
        type_info: None,
        default_name: None,
        subst_font: None,
    };

    let data = serialize_face_name(&font);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    let attr = r.read_u8().unwrap();
    assert_eq!(attr & 0x80, 0); // alt_name 없음
    let name = r.read_hwp_string().unwrap();
    assert_eq!(name, "함초롬바탕");
}

#[test]
fn test_serialize_face_name_with_alt() {
    let font = Font {
        raw_data: None,
        name: "맑은 고딕".to_string(),
        alt_type: 1,
        alt_name: Some("Malgun Gothic".to_string()),
        type_info: None,
        default_name: None,
        subst_font: None,
    };

    let data = serialize_face_name(&font);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    let attr = r.read_u8().unwrap();
    assert_eq!(attr & 0x80, 0x80); // alt_name 있음
    assert_eq!(attr & 0x03, 1); // alt_type
    let name = r.read_hwp_string().unwrap();
    assert_eq!(name, "맑은 고딕");
    assert_eq!(r.read_u8().unwrap(), 1);
    let alt_name = r.read_hwp_string().unwrap();
    assert_eq!(alt_name, "Malgun Gothic");
}

#[test]
fn test_serialize_face_name_with_type_info_and_default_name() {
    let font = Font {
        raw_data: None,
        name: "굴림".to_string(),
        alt_type: 1,
        alt_name: None,
        type_info: Some([2, 11, 6, 0, 0, 1, 1, 1, 1, 1]),
        default_name: Some("Gulim".to_string()),
        subst_font: None,
    };

    let data = serialize_face_name(&font);

    assert_eq!(
        data,
        vec![
            0x61, 0x02, 0x00, 0x74, 0xad, 0xbc, 0xb9, 0x02, 0x0b, 0x06, 0x00, 0x00, 0x01, 0x01,
            0x01, 0x01, 0x01, 0x05, 0x00, 0x47, 0x00, 0x75, 0x00, 0x6c, 0x00, 0x69, 0x00, 0x6d,
            0x00,
        ]
    );
}

#[test]
fn test_serialize_char_shape_roundtrip() {
    let cs = CharShape {
        raw_data: None,
        font_ids: [0, 1, 2, 0, 0, 0, 0],
        ratios: [100, 80, 100, 100, 100, 100, 100],
        spacings: [0, -5, 0, 0, 0, 0, 0],
        relative_sizes: [100; 7],
        char_offsets: [0; 7],
        base_size: 1000,
        attr: 0x03, // bold + italic
        italic: true,
        bold: true,
        underline_type: crate::model::style::UnderlineType::None,
        outline_type: 0,
        shadow_type: 0,
        shadow_offset_x: 0,
        shadow_offset_y: 0,
        text_color: 0x000000FF,
        underline_color: 0,
        shade_color: 0x00FFFFFF,
        shadow_color: 0x00B2B2B2,
        border_fill_id: 0,
        strike_color: 0,
        strikethrough: false,
        subscript: false,
        superscript: false,
        emboss: false,
        engrave: false,
        emphasis_dot: 0,
        underline_shape: 0,
        strike_shape: 0,
        kerning: false,
        use_font_space: false,
    };

    let data = serialize_char_shape(&cs);
    // 레코드로 감싸서 파서로 읽기
    let record_bytes = write_record(tags::HWPTAG_CHAR_SHAPE, 0, &data);
    let records = Record::read_all(&record_bytes).unwrap();
    assert_eq!(records.len(), 1);

    // ByteReader로 직접 검증
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    // font_ids
    for i in 0..7 {
        assert_eq!(r.read_u16().unwrap(), cs.font_ids[i]);
    }
    // ratios
    for i in 0..7 {
        assert_eq!(r.read_u8().unwrap(), cs.ratios[i]);
    }
    // spacings
    for i in 0..7 {
        assert_eq!(r.read_i8().unwrap(), cs.spacings[i]);
    }
    // relative_sizes
    for i in 0..7 {
        assert_eq!(r.read_u8().unwrap(), cs.relative_sizes[i]);
    }
    // char_offsets
    for i in 0..7 {
        assert_eq!(r.read_i8().unwrap(), cs.char_offsets[i]);
    }
    assert_eq!(r.read_i32().unwrap(), 1000);
    assert_eq!(r.read_u32().unwrap(), 0x03);
}

#[test]
fn test_serialize_char_shape_use_font_space_bit() {
    let mut cs = CharShape {
        base_size: 1000,
        ratios: [100; 7],
        relative_sizes: [100; 7],
        shade_color: 0x00FFFFFF,
        shadow_color: 0x00B2B2B2,
        use_font_space: true,
        ..Default::default()
    };

    let data = serialize_char_shape(&cs);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    r.skip(14 + 7 + 7 + 7 + 7 + 4).unwrap();
    let attr = r.read_u32().unwrap();
    assert_ne!(attr & (1 << 25), 0);

    cs.attr = 1 << 25;
    cs.use_font_space = false;
    let data = serialize_char_shape(&cs);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    r.skip(14 + 7 + 7 + 7 + 7 + 4).unwrap();
    let attr = r.read_u32().unwrap();
    assert_eq!(attr & (1 << 25), 0);
}

#[test]
fn test_serialize_para_shape_roundtrip() {
    let ps = ParaShape {
        raw_data: None,
        attr1: 0x04, // alignment = Left (1 << 2)
        margin_left: 1000,
        margin_right: 500,
        indent: 200,
        spacing_before: 100,
        spacing_after: 50,
        line_spacing: 160,
        alignment: Alignment::Left,
        line_spacing_type: LineSpacingType::Percent,
        tab_def_id: 1,
        numbering_id: 2,
        border_fill_id: 3,
        border_spacing: [10, 20, 30, 40],
        attr2: 0,
        attr3: 0,
        line_spacing_v2: 0,
        head_type: crate::model::style::HeadType::None,
        para_level: 0,
    };

    let data = serialize_para_shape(&ps);
    assert_eq!(data.len(), 58);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    assert_eq!(r.read_u32().unwrap(), 0x04);
    assert_eq!(r.read_i32().unwrap(), 1000);
    assert_eq!(r.read_i32().unwrap(), 500);
    assert_eq!(r.read_i32().unwrap(), 200);
    assert_eq!(r.read_i32().unwrap(), 100);
    assert_eq!(r.read_i32().unwrap(), 50);
    assert_eq!(r.read_i32().unwrap(), 160);
    assert_eq!(r.read_u16().unwrap(), 1);
    assert_eq!(r.read_u16().unwrap(), 2);
    assert_eq!(r.read_u16().unwrap(), 3);
    assert_eq!(&data[54..58], &[0, 0, 0, 0]);
}

#[test]
fn test_serialize_style_roundtrip() {
    let style = Style {
        raw_data: None,
        local_name: "바탕글".to_string(),
        english_name: "Normal".to_string(),
        style_type: 0,
        next_style_id: 0,
        lang_id: 1042,
        para_shape_id: 1,
        char_shape_id: 2,
    };

    let data = serialize_style(&style);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    assert_eq!(r.read_hwp_string().unwrap(), "바탕글");
    assert_eq!(r.read_hwp_string().unwrap(), "Normal");
    assert_eq!(r.read_u8().unwrap(), 0);
    assert_eq!(r.read_u8().unwrap(), 0);
    // [Task #1058 후속] lang_id (INT16) — HWP5 spec 표 47
    assert_eq!(r.read_i16().unwrap(), 1042);
    assert_eq!(r.read_u16().unwrap(), 1);
    assert_eq!(r.read_u16().unwrap(), 2);
    // trailing 2 byte zero
    assert_eq!(r.read_u16().unwrap(), 0);
}

#[test]
fn test_serialize_bin_data_embedding() {
    let bd = BinData {
        raw_data: None,
        attr: 0x0101, // Embedding, Default, Success
        data_type: BinDataType::Embedding,
        compression: BinDataCompression::Default,
        status: BinDataStatus::Success,
        abs_path: None,
        rel_path: None,
        storage_id: 5,
        extension: Some("jpg".to_string()),
    };

    let data = serialize_bin_data(&bd);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    assert_eq!(r.read_u16().unwrap(), 0x0101);
    assert_eq!(r.read_u16().unwrap(), 5); // storage_id
    assert_eq!(r.read_hwp_string().unwrap(), "jpg");
}

#[test]
fn test_serialize_border_fill_solid() {
    let bf = BorderFill {
        raw_data: None,
        attr: 0,
        borders: [
            BorderLine {
                line_type: BorderLineType::Solid,
                width: 3,
                color: 0x000000FF,
            },
            BorderLine {
                line_type: BorderLineType::Dash,
                width: 2,
                color: 0x0000FF00,
            },
            BorderLine {
                line_type: BorderLineType::Dot,
                width: 1,
                color: 0x00FF0000,
            },
            BorderLine {
                line_type: BorderLineType::None,
                width: 0,
                color: 0,
            },
        ],
        diagonal: DiagonalLine::default(),
        center_line: CenterLine::None,
        fill: Fill {
            fill_type: FillType::Solid,
            solid: Some(SolidFill {
                background_color: 0x00FFFFFF,
                pattern_color: 0,
                pattern_type: -1,
            }),
            gradient: None,
            image: None,
            alpha: 0,
        },
    };

    let data = serialize_border_fill(&bf);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);

    // attr
    assert_eq!(r.read_u16().unwrap(), 0);
    // 좌: Solid(1), 굵기 3, 빨강
    assert_eq!(r.read_u8().unwrap(), 1);
    assert_eq!(r.read_u8().unwrap(), 3);
    assert_eq!(r.read_color_ref().unwrap(), 0x000000FF);
    // 우: Dash(2), 굵기 2, 초록
    assert_eq!(r.read_u8().unwrap(), 2);
    assert_eq!(r.read_u8().unwrap(), 2);
    assert_eq!(r.read_color_ref().unwrap(), 0x0000FF00);
}

#[test]
fn test_serialize_border_fill_cross_centerline_uses_hwp5_center_bits() {
    let bf = BorderFill {
        raw_data: None,
        attr: (0b010 << 2) | (0b010 << 5),
        borders: [BorderLine::default(); 4],
        diagonal: DiagonalLine::default(),
        center_line: CenterLine::Cross,
        fill: Fill::default(),
    };

    let data = serialize_border_fill(&bf);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);

    assert_eq!(
        r.read_u16().unwrap(),
        (1 << 13) | (0x03 << 8) | (1 << 10),
        "HWP5 바이너리 CROSS 중심선은 한컴 중심선 보조 비트를 함께 저장해야 함"
    );
}

#[test]
fn test_serialize_border_fill_image_fill_mode_uses_hwp5_values() {
    let cases = [
        (ImageFillMode::TileAll, 0),
        (ImageFillMode::TileHorzTop, 1),
        (ImageFillMode::TileHorzBottom, 2),
        (ImageFillMode::TileVertLeft, 3),
        (ImageFillMode::TileVertRight, 4),
        (ImageFillMode::FitToSize, 5),
        (ImageFillMode::Center, 6),
        (ImageFillMode::CenterTop, 7),
        (ImageFillMode::CenterBottom, 8),
        (ImageFillMode::LeftCenter, 9),
        (ImageFillMode::LeftTop, 10),
        (ImageFillMode::LeftBottom, 11),
        (ImageFillMode::RightCenter, 12),
        (ImageFillMode::RightTop, 13),
        (ImageFillMode::RightBottom, 14),
        (ImageFillMode::None, 15),
    ];

    for (mode, expected) in cases {
        let bf = BorderFill {
            raw_data: None,
            attr: 0,
            borders: [BorderLine::default(); 4],
            diagonal: DiagonalLine::default(),
            center_line: CenterLine::None,
            fill: Fill {
                fill_type: FillType::Image,
                solid: None,
                gradient: None,
                image: Some(ImageFill {
                    fill_mode: mode,
                    brightness: 0,
                    contrast: 0,
                    effect: 0,
                    bin_data_id: 7,
                }),
                alpha: 0,
            },
        };

        let data = serialize_border_fill(&bf);
        let mut r = crate::parser::byte_reader::ByteReader::new(&data[32..]);
        assert_eq!(r.read_u32().unwrap(), 2);
        assert_eq!(
            r.read_u8().unwrap(),
            expected,
            "{mode:?} must use HWP5 image fill mode {expected}"
        );
    }
}

#[test]
fn test_serialize_tab_def() {
    let td = TabDef {
        raw_data: None,
        attr: 0x03,
        tabs: vec![crate::model::style::TabItem {
            position: 7200,
            tab_type: 0,
            fill_type: 0,
        }],
        auto_tab_left: true,
        auto_tab_right: true,
    };

    let data = serialize_tab_def(&td);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);
    assert_eq!(r.read_u32().unwrap(), 0x03);
    assert_eq!(r.read_u32().unwrap(), 1); // tab_count
    assert_eq!(r.read_u32().unwrap(), 7200); // position
    assert_eq!(r.read_u8().unwrap(), 0); // tab_type
    assert_eq!(r.read_u8().unwrap(), 0); // fill_type
}

#[test]
fn test_serialize_doc_info_roundtrip() {
    // 최소 DocInfo 구성
    let doc_props = DocProperties {
        section_count: 1,
        page_start_num: 1,
        footnote_start_num: 1,
        endnote_start_num: 1,
        picture_start_num: 1,
        table_start_num: 1,
        equation_start_num: 1,
        raw_data: None,
        caret_list_id: 0,
        caret_para_id: 0,
        caret_char_pos: 0,
    };

    let mut doc_info = DocInfo::default();
    doc_info.font_faces = vec![Vec::new(); 7];
    doc_info.font_faces[0].push(Font {
        raw_data: None,
        name: "함초롬바탕".to_string(),
        alt_type: 0,
        alt_name: None,
        type_info: None,
        default_name: None,
        subst_font: None,
    });
    doc_info.char_shapes.push(CharShape {
        raw_data: None,
        font_ids: [0; 7],
        ratios: [100; 7],
        spacings: [0; 7],
        relative_sizes: [100; 7],
        char_offsets: [0; 7],
        base_size: 1000,
        attr: 0,
        italic: false,
        bold: false,
        underline_type: crate::model::style::UnderlineType::None,
        outline_type: 0,
        shadow_type: 0,
        shadow_offset_x: 0,
        shadow_offset_y: 0,
        text_color: 0,
        underline_color: 0,
        shade_color: 0x00FFFFFF,
        shadow_color: 0x00B2B2B2,
        border_fill_id: 0,
        strike_color: 0,
        strikethrough: false,
        subscript: false,
        superscript: false,
        emboss: false,
        engrave: false,
        emphasis_dot: 0,
        underline_shape: 0,
        strike_shape: 0,
        kerning: false,
        use_font_space: false,
    });
    doc_info.para_shapes.push(ParaShape {
        raw_data: None,
        attr1: 0,
        margin_left: 0,
        margin_right: 0,
        indent: 0,
        spacing_before: 0,
        spacing_after: 0,
        line_spacing: 160,
        alignment: Alignment::Justify,
        line_spacing_type: LineSpacingType::Percent,
        tab_def_id: 0,
        numbering_id: 0,
        border_fill_id: 0,
        border_spacing: [0; 4],
        attr2: 0,
        attr3: 0,
        line_spacing_v2: 0,
        head_type: crate::model::style::HeadType::None,
        para_level: 0,
    });
    doc_info.styles.push(Style {
        raw_data: None,
        local_name: "바탕글".to_string(),
        english_name: "Normal".to_string(),
        style_type: 0,
        next_style_id: 0,
        lang_id: 1042,
        para_shape_id: 0,
        char_shape_id: 0,
    });

    // 직렬화 → 역직렬화
    let stream = serialize_doc_info(&doc_info, &doc_props);
    let (parsed_info, parsed_props) = parse_doc_info(&stream).unwrap();

    assert_eq!(parsed_props.section_count, 1);
    assert_eq!(parsed_info.font_faces[0].len(), 1);
    assert_eq!(parsed_info.font_faces[0][0].name, "함초롬바탕");
    assert_eq!(parsed_info.char_shapes.len(), 1);
    assert_eq!(parsed_info.char_shapes[0].base_size, 1000);
    assert_eq!(parsed_info.para_shapes.len(), 1);
    assert_eq!(parsed_info.para_shapes[0].line_spacing, 160);
    assert_eq!(parsed_info.styles.len(), 1);
    assert_eq!(parsed_info.styles[0].local_name, "바탕글");
}

#[test]
fn test_serialize_numbering_roundtrip() {
    let mut numbering = Numbering::default();
    numbering.heads[0] = NumberingHead {
        attr: 0x60, // number_format = 3 (bit 5~8)
        width_adjust: 100,
        text_distance: 200,
        char_shape_id: 1,
        number_format: 3,
    };
    numbering.level_formats[0] = "^1.".to_string();
    numbering.start_number = 1;
    numbering.level_start_numbers = [1; 7];

    let data = serialize_numbering(&numbering);
    let mut r = crate::parser::byte_reader::ByteReader::new(&data);

    // 첫 수준 머리 정보
    assert_eq!(r.read_u32().unwrap(), 0x60);
    assert_eq!(r.read_i16().unwrap(), 100);
    assert_eq!(r.read_i16().unwrap(), 200);
    assert_eq!(r.read_u32().unwrap(), 1);
    // 형식 문자열 "^1."
    let len = r.read_u16().unwrap();
    assert_eq!(len, 3);
}
