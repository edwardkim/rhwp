use super::*;
use crate::model::paragraph::{FieldRange, OrphanFieldEnd, TitleMark};
use crate::parser::tags;

/// 테스트용 레코드 바이너리 생성
fn make_record_bytes(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
    let size = data.len() as u32;
    let header = (tag_id as u32) | ((level as u32) << 10) | (size << 20);
    let mut bytes = header.to_le_bytes().to_vec();
    bytes.extend_from_slice(data);
    bytes
}

/// PARA_HEADER 테스트 데이터 생성
fn make_para_header_data(char_count: u32, para_shape_id: u16, style_id: u8) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&char_count.to_le_bytes()); // nChars
    data.extend_from_slice(&0u32.to_le_bytes()); // controlMask
    data.extend_from_slice(&para_shape_id.to_le_bytes()); // paraShapeId
    data.push(style_id); // styleId
    data.push(0); // breakType
    data
}

/// UTF-16LE 텍스트 생성 (문단 끝 포함)
fn make_para_text_data(text: &str) -> Vec<u8> {
    let mut data = Vec::new();
    for ch in text.encode_utf16() {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    // 문단 끝 마커 (0x000D)
    data.extend_from_slice(&0x000Du16.to_le_bytes());
    data
}

#[test]
fn hancom_single_odd_master_flag_is_not_parsed_as_both() {
    assert_eq!(
        master_page_apply_to(0x8008_0000, 1, 0),
        Some(HeaderFooterApply::Odd)
    );
    assert_eq!(master_page_apply_to(0x2008_0000, 1, 0), None);
    assert_eq!(master_page_apply_to(0xc008_0000, 2, 0), None);
}

#[test]
fn test_parse_para_text_simple() {
    let ParaTextParts {
        text,
        char_offsets: offsets,
        ..
    } = parse_para_text(&make_para_text_data("Hello, World!"));
    assert_eq!(text, "Hello, World!");
    assert_eq!(offsets.len(), 13);
    assert_eq!(offsets[0], 0); // 'H' at position 0

    bulkbuild_identical_exhaustive_single_unit();
    bulkbuild_identical_run_length_sweep();
    bulkbuild_identical_surrogates();
    bulkbuild_identical_random_fuzz();
}

#[test]
fn test_parse_para_text_korean() {
    let ParaTextParts {
        text,
        char_offsets: offsets,
        ..
    } = parse_para_text(&make_para_text_data("한글 테스트입니다."));
    assert_eq!(text, "한글 테스트입니다.");
    assert_eq!(offsets.len(), text.chars().count());
}

#[test]
fn test_parse_para_text_with_tab() {
    let mut data = Vec::new();
    // "A" + tab(0x0009, inline 8 code units = 16바이트) + "B" + para break
    data.extend_from_slice(&0x0041u16.to_le_bytes()); // 'A'
                                                      // tab: 0x0009 + 7 dummy code units (inline control data)
    data.extend_from_slice(&0x0009u16.to_le_bytes());
    for _ in 0..7 {
        data.extend_from_slice(&0x0000u16.to_le_bytes());
    }
    data.extend_from_slice(&0x0042u16.to_le_bytes()); // 'B'
    data.extend_from_slice(&0x000Du16.to_le_bytes()); // para break
    let ParaTextParts {
        text,
        char_offsets: offsets,
        ..
    } = parse_para_text(&data);
    assert_eq!(text, "A\tB");
    // 'A' at code unit 0, tab takes 8 units (1-8), 'B' at code unit 9
    assert_eq!(offsets, vec![0, 1, 9]);
}

#[test]
fn test_parse_para_text_with_extended_ctrl() {
    let mut data = Vec::new();
    // "A" + extended ctrl(0x000B, 8 code units) + "B" + para break
    data.extend_from_slice(&0x0041u16.to_le_bytes()); // 'A'
                                                      // Extended control character: 0x000B + 7 dummy code units
    data.extend_from_slice(&0x000Bu16.to_le_bytes());
    for _ in 0..7 {
        data.extend_from_slice(&0x0000u16.to_le_bytes());
    }
    data.extend_from_slice(&0x0042u16.to_le_bytes()); // 'B'
    data.extend_from_slice(&0x000Du16.to_le_bytes()); // para break
    let ParaTextParts {
        text,
        char_offsets: offsets,
        ..
    } = parse_para_text(&data);
    assert_eq!(text, "AB");
    // 'A' at code unit 0, extended ctrl takes 8 units (1-8), 'B' at code unit 9
    assert_eq!(offsets, vec![0, 9]);
}

#[test]
fn test_parse_para_text_empty() {
    // 문단 끝만 있는 경우
    let data = 0x000Du16.to_le_bytes();
    let ParaTextParts {
        text,
        char_offsets: offsets,
        ..
    } = parse_para_text(&data);
    assert_eq!(text, "");
    assert!(offsets.is_empty());
}

#[test]
fn test_is_extended_ctrl_char() {
    // extended (8 code units): 1-3, 11-12, 14-18, 21-23
    assert!(is_extended_ctrl_char(0x0001)); // reserved
    assert!(is_extended_ctrl_char(0x0002)); // section/column def
    assert!(is_extended_ctrl_char(0x0003)); // field begin
    assert!(is_extended_ctrl_char(0x000B)); // drawing/table
    assert!(is_extended_ctrl_char(0x000C)); // reserved
    assert!(is_extended_ctrl_char(0x0011)); // footnote/endnote
    assert!(is_extended_ctrl_char(0x0015)); // page control
    assert!(is_extended_ctrl_char(0x0017)); // annotation/overlap

    // inline (8 code units): 4-8, 19-20
    // (탭 0x09는 호출 전에 별도 처리되므로 여기서는 true)
    assert!(is_extended_ctrl_char(0x0004)); // field end (inline, 16 bytes)
    assert!(is_extended_ctrl_char(0x0005)); // reserved (inline, 16 bytes)
    assert!(is_extended_ctrl_char(0x0008)); // title mark (inline, 16 bytes)

    // char (1 code unit): 0, 10, 13, 24-31
    assert!(!is_extended_ctrl_char(0x0000)); // null
    assert!(!is_extended_ctrl_char(0x000A)); // line break
    assert!(!is_extended_ctrl_char(0x000D)); // para break
    assert!(!is_extended_ctrl_char(0x0018)); // hyphen
    assert!(!is_extended_ctrl_char(0x0019)); // reserved
    assert!(!is_extended_ctrl_char(0x001A)); // reserved
    assert!(!is_extended_ctrl_char(0x001E)); // non-breaking space
    assert!(!is_extended_ctrl_char(0x001F)); // fixed-width space

    // 일반 문자
    assert!(!is_extended_ctrl_char(0x0020)); // space
    assert!(!is_extended_ctrl_char(0x0041)); // 'A'
}

#[test]
fn test_parse_para_char_shape() {
    let mut data = Vec::new();
    // 항목 1: pos=0, id=3
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&3u32.to_le_bytes());
    // 항목 2: pos=10, id=5
    data.extend_from_slice(&10u32.to_le_bytes());
    data.extend_from_slice(&5u32.to_le_bytes());

    let refs = parse_para_char_shape(&data);
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].start_pos, 0);
    assert_eq!(refs[0].char_shape_id, 3);
    assert_eq!(refs[1].start_pos, 10);
    assert_eq!(refs[1].char_shape_id, 5);
}

#[test]
fn test_parse_para_line_seg() {
    let mut data = Vec::new();
    // LineSeg: 36바이트
    data.extend_from_slice(&0u32.to_le_bytes()); // text_start
    data.extend_from_slice(&100i32.to_le_bytes()); // vertical_pos
    data.extend_from_slice(&500i32.to_le_bytes()); // line_height
    data.extend_from_slice(&400i32.to_le_bytes()); // text_height
    data.extend_from_slice(&300i32.to_le_bytes()); // baseline_distance
    data.extend_from_slice(&200i32.to_le_bytes()); // line_spacing
    data.extend_from_slice(&0i32.to_le_bytes()); // column_start
    data.extend_from_slice(&42000i32.to_le_bytes()); // segment_width
    data.extend_from_slice(&0x01u32.to_le_bytes()); // tag (first line of page)

    let segs = parse_para_line_seg(&data);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].text_start, 0);
    assert_eq!(segs[0].line_height, 500);
    assert_eq!(segs[0].segment_width, 42000);
    assert!(segs[0].is_first_line_of_page());
}

#[test]
fn test_parse_para_range_tag() {
    let mut data = Vec::new();
    data.extend_from_slice(&5u32.to_le_bytes()); // start
    data.extend_from_slice(&15u32.to_le_bytes()); // end
    data.extend_from_slice(&0x01000003u32.to_le_bytes()); // tag

    let tags = parse_para_range_tag(&data);
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].start, 5);
    assert_eq!(tags[0].end, 15);
    assert_eq!(tags[0].tag, 0x01000003);
}

#[test]
fn test_parse_page_def() {
    let mut data = Vec::new();
    data.extend_from_slice(&59528u32.to_le_bytes()); // width (A4)
    data.extend_from_slice(&84188u32.to_le_bytes()); // height
    data.extend_from_slice(&8504u32.to_le_bytes()); // margin_left
    data.extend_from_slice(&8504u32.to_le_bytes()); // margin_right
    data.extend_from_slice(&5669u32.to_le_bytes()); // margin_top
    data.extend_from_slice(&4252u32.to_le_bytes()); // margin_bottom
    data.extend_from_slice(&4252u32.to_le_bytes()); // margin_header
    data.extend_from_slice(&4252u32.to_le_bytes()); // margin_footer
    data.extend_from_slice(&0u32.to_le_bytes()); // margin_gutter
    data.extend_from_slice(&0u32.to_le_bytes()); // attr (세로, 한쪽)

    let pd = parse_page_def(&data);
    assert_eq!(pd.width, 59528);
    assert_eq!(pd.height, 84188);
    assert!(!pd.landscape);
    assert_eq!(pd.binding, BindingMethod::SingleSided);
}

#[test]
fn test_parse_page_def_landscape() {
    let mut data = Vec::new();
    data.extend_from_slice(&84188u32.to_le_bytes()); // width
    data.extend_from_slice(&59528u32.to_le_bytes()); // height
    for _ in 0..7 {
        data.extend_from_slice(&0u32.to_le_bytes()); // margins
    }
    data.extend_from_slice(&0x01u32.to_le_bytes()); // attr: landscape

    let pd = parse_page_def(&data);
    assert!(pd.landscape);
}

#[test]
fn test_parse_section_simple() {
    // 최소 섹션: PARA_HEADER + PARA_TEXT
    let para_header_data = make_para_header_data(6, 0, 0);
    let para_text_data = make_para_text_data("Hello");

    let mut section_bytes = Vec::new();
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_PARA_HEADER,
        0,
        &para_header_data,
    ));
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_PARA_TEXT,
        1,
        &para_text_data,
    ));

    let section = parse_body_text_section(&section_bytes).unwrap();
    assert_eq!(section.paragraphs.len(), 1);
    assert_eq!(section.paragraphs[0].text, "Hello");
}

#[test]
fn test_parse_section_multiple_paragraphs() {
    let mut section_bytes = Vec::new();

    // 문단 1
    let ph1 = make_para_header_data(4, 0, 0);
    let pt1 = make_para_text_data("ABC");
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph1));
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_TEXT, 1, &pt1));

    // 문단 2
    let ph2 = make_para_header_data(4, 1, 0);
    let pt2 = make_para_text_data("DEF");
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph2));
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_TEXT, 1, &pt2));

    let section = parse_body_text_section(&section_bytes).unwrap();
    assert_eq!(section.paragraphs.len(), 2);
    assert_eq!(section.paragraphs[0].text, "ABC");
    assert_eq!(section.paragraphs[1].text, "DEF");
    assert_eq!(section.paragraphs[1].para_shape_id, 1);
}

#[test]
fn test_parse_section_with_section_def() {
    let mut section_bytes = Vec::new();

    // 문단 1 (구역 정의 포함)
    let ph = make_para_header_data(2, 0, 0);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph));

    // 텍스트
    let pt = make_para_text_data("A");
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_TEXT, 1, &pt));

    // CTRL_HEADER (secd)
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&tags::CTRL_SECTION_DEF.to_le_bytes()); // ctrl_id
    ctrl_data.extend_from_slice(&0u32.to_le_bytes()); // flags
    ctrl_data.extend_from_slice(&0i16.to_le_bytes()); // column_spacing
    ctrl_data.extend_from_slice(&1200i16.to_le_bytes()); // line_grid
    ctrl_data.extend_from_slice(&900i16.to_le_bytes()); // char_grid
    ctrl_data.extend_from_slice(&800u32.to_le_bytes()); // default_tab_spacing
    ctrl_data.extend_from_slice(&0u16.to_le_bytes()); // numbering_id
    ctrl_data.extend_from_slice(&1u16.to_le_bytes()); // page_num
    ctrl_data.extend_from_slice(&0u16.to_le_bytes()); // picture_num
    ctrl_data.extend_from_slice(&0u16.to_le_bytes()); // table_num
    ctrl_data.extend_from_slice(&0u16.to_le_bytes()); // equation_num
    section_bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_HEADER, 1, &ctrl_data));

    // PAGE_DEF (secd의 자식)
    let mut page_data = Vec::new();
    page_data.extend_from_slice(&59528u32.to_le_bytes()); // width
    page_data.extend_from_slice(&84188u32.to_le_bytes()); // height
    for _ in 0..7 {
        page_data.extend_from_slice(&0u32.to_le_bytes());
    }
    page_data.extend_from_slice(&0u32.to_le_bytes()); // attr
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PAGE_DEF, 2, &page_data));

    let section = parse_body_text_section(&section_bytes).unwrap();
    assert_eq!(section.section_def.default_tab_spacing, 800);
    assert_eq!(section.section_def.line_grid, 1200);
    assert_eq!(section.section_def.char_grid, 900);
    assert_eq!(section.section_def.page_num, 1);
    assert_eq!(section.section_def.page_def.width, 59528);
}

#[test]
fn test_section_def_direct_ctrl_data_has_single_owner_and_nested_records_are_preserved() {
    let mut section_bytes = Vec::new();

    let ph = make_para_header_data(2, 0, 0);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph));

    let mut ctrl_header = Vec::new();
    ctrl_header.extend_from_slice(&tags::CTRL_SECTION_DEF.to_le_bytes());
    ctrl_header.extend_from_slice(&[0u8; 24]);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_HEADER, 1, &ctrl_header));

    let canonical_ctrl_data = vec![0x10, 0x20, 0x30, 0x40];
    let additional_direct_ctrl_data = vec![0x50, 0x60, 0x70, 0x80];
    let nested_ctrl_data = vec![0x90, 0xA0, 0xB0, 0xC0];
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_CTRL_DATA,
        2,
        &canonical_ctrl_data,
    ));
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_CTRL_DATA,
        2,
        &additional_direct_ctrl_data,
    ));

    let nested_ctrl_id = 0x7473_6574u32; // 'test'
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_CTRL_HEADER,
        2,
        &nested_ctrl_id.to_le_bytes(),
    ));
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_CTRL_DATA,
        3,
        &nested_ctrl_data,
    ));

    let section = parse_body_text_section(&section_bytes).unwrap();
    let para = &section.paragraphs[0];

    assert_eq!(
        para.ctrl_data_records,
        vec![Some(canonical_ctrl_data.clone())],
        "SectionDef의 첫 직접 자식 CTRL_DATA는 문단 control 슬롯이 소유해야 한다"
    );

    let section_def = match &para.controls[0] {
        Control::SectionDef(section_def) => section_def,
        other => panic!("SectionDef를 기대했지만 {other:?}"),
    };
    assert!(
        !section_def.extra_child_records.iter().any(|raw| {
            raw.tag_id == tags::HWPTAG_CTRL_DATA
                && raw.level == 2
                && raw.data == canonical_ctrl_data
        }),
        "문단이 소유한 첫 직접 자식 CTRL_DATA를 SectionDef extra에도 중복 보존하면 안 된다"
    );
    assert!(
        section_def.extra_child_records.iter().any(|raw| {
            raw.tag_id == tags::HWPTAG_CTRL_DATA
                && raw.level == 2
                && raw.data == additional_direct_ctrl_data
        }),
        "추가 직접 자식 CTRL_DATA는 원본 보존 대상이다"
    );
    assert!(
        section_def.extra_child_records.iter().any(|raw| {
            raw.tag_id == tags::HWPTAG_CTRL_DATA && raw.level == 3 && raw.data == nested_ctrl_data
        }),
        "중첩 control의 CTRL_DATA는 SectionDef extra에서 보존돼야 한다"
    );
}

#[test]
fn test_section_def_ctrl_data_after_nested_header_stays_in_raw_children() {
    let mut section_bytes = Vec::new();

    let ph = make_para_header_data(2, 0, 0);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph));

    let mut ctrl_header = Vec::new();
    ctrl_header.extend_from_slice(&tags::CTRL_SECTION_DEF.to_le_bytes());
    ctrl_header.extend_from_slice(&[0u8; 24]);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_HEADER, 1, &ctrl_header));

    let nested_ctrl_id = 0x7473_6574u32; // 'test'
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_CTRL_HEADER,
        2,
        &nested_ctrl_id.to_le_bytes(),
    ));
    section_bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_DATA, 3, &[0x10, 0x20]));

    let late_direct_ctrl_data = vec![0x30, 0x40, 0x50, 0x60];
    section_bytes.extend(make_record_bytes(
        tags::HWPTAG_CTRL_DATA,
        2,
        &late_direct_ctrl_data,
    ));

    let section = parse_body_text_section(&section_bytes).unwrap();
    let para = &section.paragraphs[0];
    assert_eq!(
        para.ctrl_data_records,
        vec![None],
        "중첩 CTRL_HEADER 뒤의 직접 자식 CTRL_DATA를 문단 슬롯으로 이동하면 안 된다"
    );

    let section_def = match &para.controls[0] {
        Control::SectionDef(section_def) => section_def,
        other => panic!("SectionDef를 기대했지만 {other:?}"),
    };
    assert!(
        section_def.extra_child_records.iter().any(|raw| {
            raw.tag_id == tags::HWPTAG_CTRL_DATA
                && raw.level == 2
                && raw.data == late_direct_ctrl_data
        }),
        "중첩 CTRL_HEADER 뒤의 직접 자식 CTRL_DATA는 raw 자식의 원래 위치에 남아야 한다"
    );
}

#[test]
fn test_parse_section_with_column_def() {
    let mut section_bytes = Vec::new();

    // 문단
    let ph = make_para_header_data(2, 0, 0);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph));

    // CTRL_HEADER (cold) - 2단, 같은 너비, 간격 1000
    // 표 141: bit 0-1=종류(0), bit 2-9=단수(2), bit 12=동일너비(1)
    let attr: u16 = (2 << 2) | (1 << 12); // 0x1008
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&tags::CTRL_COLUMN_DEF.to_le_bytes());
    ctrl_data.extend_from_slice(&attr.to_le_bytes()); // attr (bits 0-15)
    ctrl_data.extend_from_slice(&1000i16.to_le_bytes()); // spacing
    ctrl_data.extend_from_slice(&0u16.to_le_bytes()); // attr2 (bits 16-32)
    section_bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_HEADER, 1, &ctrl_data));

    let section = parse_body_text_section(&section_bytes).unwrap();
    assert_eq!(section.paragraphs.len(), 1);

    let has_column_def = section.paragraphs[0]
        .controls
        .iter()
        .any(|c| matches!(c, Control::ColumnDef(_)));
    assert!(has_column_def);

    if let Some(Control::ColumnDef(cd)) = section.paragraphs[0]
        .controls
        .iter()
        .find(|c| matches!(c, Control::ColumnDef(_)))
    {
        assert_eq!(cd.column_count, 2);
        assert!(cd.same_width);
        assert_eq!(cd.spacing, 1000);
    }
}

#[test]
fn test_parse_table_control_delegation() {
    let mut section_bytes = Vec::new();

    let ph = make_para_header_data(2, 0, 0);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph));

    // 표 컨트롤 → control.rs로 위임되어 Table로 파싱
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&tags::CTRL_TABLE.to_le_bytes());
    ctrl_data.extend_from_slice(&[0u8; 20]); // dummy data
    section_bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_HEADER, 1, &ctrl_data));

    let section = parse_body_text_section(&section_bytes).unwrap();
    let has_table = section.paragraphs[0]
        .controls
        .iter()
        .any(|c| matches!(c, Control::Table(_)));
    assert!(has_table);
}

#[test]
fn test_parse_unknown_control() {
    let mut section_bytes = Vec::new();

    let ph = make_para_header_data(2, 0, 0);
    section_bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, 0, &ph));

    // 등록되지 않은 임의의 컨트롤 ID → Unknown
    let unknown_ctrl_id: u32 = 0x78797A77; // 'wxyz' (미등록)
    let mut ctrl_data = Vec::new();
    ctrl_data.extend_from_slice(&unknown_ctrl_id.to_le_bytes());
    ctrl_data.extend_from_slice(&[0u8; 20]); // dummy data
    section_bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_HEADER, 1, &ctrl_data));

    let section = parse_body_text_section(&section_bytes).unwrap();
    let has_unknown = section.paragraphs[0]
        .controls
        .iter()
        .any(|c| matches!(c, Control::Unknown(u) if u.ctrl_id == unknown_ctrl_id));
    assert!(has_unknown);
}

#[test]
fn test_parse_para_header_fields() {
    let data = make_para_header_data(42, 5, 2);
    let para = parse_para_header(&data);
    assert_eq!(para.char_count, 42);
    assert_eq!(para.para_shape_id, 5);
    assert_eq!(para.style_id, 2);
}

#[test]
fn test_parse_page_border_fill() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x01u32.to_le_bytes()); // attr
    data.extend_from_slice(&100i16.to_le_bytes()); // spacing_left
    data.extend_from_slice(&200i16.to_le_bytes()); // spacing_right
    data.extend_from_slice(&300i16.to_le_bytes()); // spacing_top
    data.extend_from_slice(&400i16.to_le_bytes()); // spacing_bottom
    data.extend_from_slice(&7u16.to_le_bytes()); // border_fill_id

    let pbf = parse_page_border_fill(&data);
    assert_eq!(pbf.attr, 0x01);
    assert_eq!(pbf.spacing_left, 100);
    assert_eq!(pbf.border_fill_id, 7);
    assert_eq!(pbf.basis, crate::model::page::PageBorderBasis::BodyBased);
    assert_eq!(pbf.ui_basis, crate::model::page::PageBorderUiBasis::Page);

    data[0..4].copy_from_slice(&0x00u32.to_le_bytes());
    let pbf = parse_page_border_fill(&data);
    assert_eq!(pbf.attr, 0x00);
    assert_eq!(pbf.basis, crate::model::page::PageBorderBasis::PaperBased);
    assert_eq!(pbf.ui_basis, crate::model::page::PageBorderUiBasis::Paper);
}

#[test]
fn test_parse_empty_section() {
    let section = parse_body_text_section(&[]).unwrap();
    assert!(section.paragraphs.is_empty());
}

/// 진단용 테스트: hancom-webgian.hwp의 LineSeg 데이터를 분석하여
/// vertical_pos, line_height, line_spacing 간 관계를 검증한다.
#[test]
fn test_lineseg_field_semantics() {
    let path = std::path::Path::new("samples/hancom-webgian.hwp");
    if !path.exists() {
        eprintln!("samples/hancom-webgian.hwp 없음 — 건너뜀");
        return;
    }
    let data = std::fs::read(path).unwrap();
    let doc = crate::parser::parse_hwp(&data).expect("parse");

    eprintln!("\n=== LineSeg 필드 의미 분석 (hancom-webgian.hwp) ===\n");

    // 1. 모든 문단의 LineSeg 출력 (첫 20개 + lh > 2000인 문단)
    for (sec_idx, section) in doc.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            let text_preview: String = para.text.chars().take(30).collect();
            let has_large_font = para.line_segs.iter().any(|s| s.line_height > 2000);
            if para_idx < 20 || has_large_font {
                eprintln!(
                    "Para{}: text=\"{}\" psid={} segs={}",
                    para_idx,
                    text_preview,
                    para.para_shape_id,
                    para.line_segs.len()
                );
                for (i, seg) in para.line_segs.iter().enumerate() {
                    eprintln!(
                        "  L{}: vpos={} lh={} th={} bd={} ls={} tag={:#010x}",
                        i,
                        seg.vertical_pos,
                        seg.line_height,
                        seg.text_height,
                        seg.baseline_distance,
                        seg.line_spacing,
                        seg.tag
                    );
                }
            }
        }
    }

    // 2. 줄 내 관계 검증 (multi-line paragraphs)
    let mut match_ls_count = 0;
    let mut match_lh_ls_count = 0;
    let mut total_pairs = 0;

    for (_sec_idx, section) in doc.sections.iter().enumerate() {
        for (_para_idx, para) in section.paragraphs.iter().enumerate() {
            if para.line_segs.len() < 2 {
                continue;
            }
            for i in 0..para.line_segs.len() - 1 {
                let curr = &para.line_segs[i];
                let next = &para.line_segs[i + 1];
                let vpos_diff = next.vertical_pos - curr.vertical_pos;
                total_pairs += 1;
                if vpos_diff == curr.line_spacing {
                    match_ls_count += 1;
                }
                if vpos_diff == curr.line_height + curr.line_spacing {
                    match_lh_ls_count += 1;
                }
            }
        }
    }

    eprintln!("\n=== 결과 요약 ===");
    eprintln!("총 줄 쌍: {}", total_pairs);
    eprintln!(
        "vpos_diff == line_spacing: {} ({}%)",
        match_ls_count,
        if total_pairs > 0 {
            match_ls_count * 100 / total_pairs
        } else {
            0
        }
    );
    eprintln!(
        "vpos_diff == line_height + line_spacing: {} ({}%)",
        match_lh_ls_count,
        if total_pairs > 0 {
            match_lh_ls_count * 100 / total_pairs
        } else {
            0
        }
    );

    // 2. 문단 간 vpos 관계 분석
    eprintln!("\n=== 문단 간 관계 분석 ===");
    for (sec_idx, section) in doc.sections.iter().enumerate() {
        for i in 0..section.paragraphs.len().saturating_sub(1) {
            let curr_para = &section.paragraphs[i];
            let next_para = &section.paragraphs[i + 1];

            if curr_para.line_segs.is_empty() || next_para.line_segs.is_empty() {
                continue;
            }

            let last_seg = curr_para.line_segs.last().unwrap();
            let next_first = &next_para.line_segs[0];

            // 현재 문단의 마지막 줄 끝 위치 (다양한 해석)
            let end_with_lh = last_seg.vertical_pos + last_seg.line_height;
            let end_with_lh_ls =
                last_seg.vertical_pos + last_seg.line_height + last_seg.line_spacing;
            let gap_from_lh = next_first.vertical_pos - end_with_lh;
            let gap_from_lh_ls = next_first.vertical_pos - end_with_lh_ls;

            // 같은 페이지 내에서만 분석 (vpos가 감소하면 새 페이지)
            if next_first.vertical_pos < last_seg.vertical_pos {
                continue;
            }

            if i < 5 || gap_from_lh_ls != 0 {
                eprintln!(
                    "  Para{}→{}: last_vpos={} last_lh={} last_ls={} next_vpos={} gap(lh)={} gap(lh+ls)={}",
                    i, i + 1, last_seg.vertical_pos, last_seg.line_height, last_seg.line_spacing,
                    next_first.vertical_pos, gap_from_lh, gap_from_lh_ls
                );
            }
        }
    }

    // 3. 전체 문단 수 및 줄 수 통계
    let mut total_paras = 0;
    let mut total_lines = 0;
    let mut unique_lh_ls: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    for section in &doc.sections {
        total_paras += section.paragraphs.len();
        for para in &section.paragraphs {
            total_lines += para.line_segs.len();
            for seg in &para.line_segs {
                unique_lh_ls.insert((seg.line_height, seg.line_spacing));
            }
        }
    }
    eprintln!("\n=== 통계 ===");
    eprintln!("문단 수: {}, 줄 수: {}", total_paras, total_lines);
    eprintln!("고유 (line_height, line_spacing) 쌍:");
    let mut pairs: Vec<_> = unique_lh_ls.iter().collect();
    pairs.sort();
    for (lh, ls) in pairs {
        eprintln!("  lh={} ls={} total={}", lh, ls, lh + ls);
    }

    assert_eq!(
        match_lh_ls_count, total_pairs,
        "모든 줄 쌍이 vpos_diff == line_height + line_spacing 이어야 함"
    );
}

/// 진단용 테스트: hancom-webgian.hwp에서 표를 포함하는 문단의
/// line_seg, table 속성, para_shape(spacing_before/after) 정보를 출력한다.
/// 표 페이지네이션 overflow 원인 분석용.
#[test]
fn test_table_paragraph_diagnostics() {
    let path = std::path::Path::new("samples/hancom-webgian.hwp");
    if !path.exists() {
        eprintln!("samples/hancom-webgian.hwp 없음 — 건너뜀");
        return;
    }
    let data = std::fs::read(path).unwrap();
    let doc = crate::parser::parse_hwp(&data).expect("parse");

    eprintln!("\n=== 표 포함 문단 진단 (hancom-webgian.hwp) ===\n");

    for (sec_idx, section) in doc.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            // 표 컨트롤이 있는 문단만 처리
            let tables: Vec<&crate::model::table::Table> = para
                .controls
                .iter()
                .filter_map(|c| {
                    if let Control::Table(t) = c {
                        Some(t.as_ref())
                    } else {
                        None
                    }
                })
                .collect();

            if tables.is_empty() {
                continue;
            }

            // 텍스트 미리보기 (첫 20자)
            let text_preview: String = para.text.chars().take(20).collect();

            eprintln!("--- Section {} / Para {} ---", sec_idx, para_idx);
            eprintln!("  para_shape_id: {}", para.para_shape_id);
            eprintln!("  text_preview: \"{}\"", text_preview);
            eprintln!("  line_segs count: {}", para.line_segs.len());

            // 첫 번째 line_seg 정보
            if let Some(seg) = para.line_segs.first() {
                eprintln!(
                    "  first line_seg: vertical_pos={} line_height={} text_height={} line_spacing={} baseline_dist={} tag={:#010x}",
                    seg.vertical_pos,
                    seg.line_height,
                    seg.text_height,
                    seg.line_spacing,
                    seg.baseline_distance,
                    seg.tag,
                );
            }

            // 모든 line_seg 출력 (2개 이상인 경우)
            if para.line_segs.len() > 1 {
                for (i, seg) in para.line_segs.iter().enumerate() {
                    eprintln!(
                        "  line_seg[{}]: vpos={} lh={} th={} ls={} bd={} tag={:#010x}",
                        i,
                        seg.vertical_pos,
                        seg.line_height,
                        seg.text_height,
                        seg.line_spacing,
                        seg.baseline_distance,
                        seg.tag,
                    );
                }
            }

            // ParaShape 조회
            let ps_id = para.para_shape_id as usize;
            if ps_id < doc.doc_info.para_shapes.len() {
                let ps = &doc.doc_info.para_shapes[ps_id];
                eprintln!(
                    "  para_shape: spacing_before={} spacing_after={} line_spacing={} line_spacing_type={:?} line_spacing_v2={}",
                    ps.spacing_before,
                    ps.spacing_after,
                    ps.line_spacing,
                    ps.line_spacing_type,
                    ps.line_spacing_v2,
                );
                // host_spacing 계산 (진단 목적)
                let host_spacing = ps.spacing_before + ps.spacing_after;
                eprintln!("  host_spacing (before+after): {}", host_spacing);
            } else {
                eprintln!(
                    "  para_shape: id {} out of range (max {})",
                    ps_id,
                    doc.doc_info.para_shapes.len()
                );
            }

            // 각 표 정보 출력
            for (t_idx, table) in tables.iter().enumerate() {
                let treat_as_char = (table.attr & 1) != 0;
                eprintln!(
                    "  table[{}]: row_count={} col_count={} attr={:#010x} treat_as_char={} page_break={:?} repeat_header={}",
                    t_idx,
                    table.row_count,
                    table.col_count,
                    table.attr,
                    treat_as_char,
                    table.page_break,
                    table.repeat_header,
                );
                eprintln!(
                    "  table[{}]: cell_spacing={} cells_count={} caption={:?}",
                    t_idx,
                    table.cell_spacing,
                    table.cells.len(),
                    table.caption.as_ref().map(|c| format!(
                        "dir={:?} paras={}",
                        c.direction,
                        c.paragraphs.len()
                    )),
                );

                // 행별 셀 높이 합산을 위해 각 셀의 크기 출력
                for (c_idx, cell) in table.cells.iter().enumerate() {
                    eprintln!(
                        "    cell[{}]: row={} col={} row_span={} col_span={} width={} height={}",
                        c_idx,
                        cell.row,
                        cell.col,
                        cell.row_span,
                        cell.col_span,
                        cell.width,
                        cell.height,
                    );
                }
            }

            eprintln!();
        }
    }

    eprintln!("=== 진단 완료 ===\n");
}

/// 제목 차례 표시(`Mtit`/`Mign`)를 통째로 흘리면 문단 축이 8유닛 짧아진다.
///
/// 한글은 축이 어긋난 `<hp:lineseg textpos>` 를 만나면 본문을 통째로 버리므로
/// (10k 스윕 F-절단군), 텍스트에 싣지 않되 위치는 반드시 남겨야 한다.
#[test]
fn title_mark_is_preserved_as_eight_unit_slot() {
    // [Mtit 16바이트][가][나]
    let mut data = Vec::new();
    data.extend_from_slice(&0x0008u16.to_le_bytes());
    data.extend_from_slice(&tags::CTRL_TITLE_MARK_IGNORE_ON.to_le_bytes());
    for _ in 0..4 {
        data.extend_from_slice(&0u16.to_le_bytes());
    }
    data.extend_from_slice(&0x0008u16.to_le_bytes());
    for ch in "가나".encode_utf16() {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    data.extend_from_slice(&0x000Du16.to_le_bytes());

    let parts = parse_para_text(&data);
    assert_eq!(parts.text, "가나", "표시는 텍스트가 아니다");
    assert_eq!(
        parts.char_offsets,
        vec![8, 9],
        "표시가 앞 8유닛을 점유하므로 첫 글자는 8 에서 시작한다"
    );
    assert_eq!(
        parts.title_marks,
        vec![TitleMark {
            char_idx: 0,
            ignore: true,
        }]
    );
}

/// `Mign` 은 같은 자리의 `ignore="0"` 짝이다 — 한글 2022 양방향 실측(06699).
#[test]
fn title_mark_ignore_off_variant_is_distinguished() {
    let mut data = Vec::new();
    for ch in "가".encode_utf16() {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    data.extend_from_slice(&0x0008u16.to_le_bytes());
    data.extend_from_slice(&tags::CTRL_TITLE_MARK_IGNORE_OFF.to_le_bytes());
    for _ in 0..4 {
        data.extend_from_slice(&0u16.to_le_bytes());
    }
    data.extend_from_slice(&0x0008u16.to_le_bytes());
    data.extend_from_slice(&0x000Du16.to_le_bytes());

    let parts = parse_para_text(&data);
    assert_eq!(parts.text, "가");
    assert_eq!(
        parts.title_marks,
        vec![TitleMark {
            char_idx: 1,
            ignore: false,
        }],
        "글자 뒤에 붙은 표시도 위치가 남아야 한다"
    );
}

/// 0x08 이라도 알려지지 않은 ctrl_id 는 표시로 오인하지 않는다.
#[test]
fn unknown_inline_ctrl_id_is_not_taken_for_a_title_mark() {
    let mut data = Vec::new();
    data.extend_from_slice(&0x0008u16.to_le_bytes());
    data.extend_from_slice(&u32::from_le_bytes(*b"zzzz").to_le_bytes());
    for _ in 0..4 {
        data.extend_from_slice(&0u16.to_le_bytes());
    }
    data.extend_from_slice(&0x0008u16.to_le_bytes());
    data.extend_from_slice(&0x000Du16.to_le_bytes());

    assert!(parse_para_text(&data).title_marks.is_empty());
}

/// 짝 FIELD_BEGIN 이 앞 문단에 있는 종료 마커도 8유닛 슬롯을 지켜야 한다.
///
/// 종전에는 스택이 비면 아무것도 남기지 않고 흘려보냈다. 그러면 축이 8유닛 짧아져
/// 그 문단의 lineseg 가 범위 밖이 되고 조판이 통째로 버려진다(01752 실측).
#[test]
fn orphan_field_end_is_preserved_as_a_slot() {
    let mut data = Vec::new();
    for ch in "가나".encode_utf16() {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    // FIELD_END 16바이트 (짝 BEGIN 없음)
    data.extend_from_slice(&0x0004u16.to_le_bytes());
    data.extend_from_slice(&u32::from_le_bytes(*b"klc\x09").to_le_bytes());
    for _ in 0..4 {
        data.extend_from_slice(&0u16.to_le_bytes());
    }
    data.extend_from_slice(&0x0004u16.to_le_bytes());
    data.extend_from_slice(&0x000Du16.to_le_bytes());

    let parts = parse_para_text(&data);
    assert_eq!(parts.text, "가나");
    assert_eq!(
        parts.field_ranges.len(),
        0,
        "짝이 없으니 범위는 만들지 않는다"
    );
    assert_eq!(parts.orphan_field_ends.len(), 1);
    assert_eq!(parts.orphan_field_ends[0].char_idx, 2, "글자 뒤에 놓인다");
}

/// 종료 마커는 앞 문단에서 열린 필드의 id 를 물려받아야 한다.
///
/// 매달린 참조(`beginIDRef="0"`)를 내보내면 한글이 **파일을 열지 못한다**(01752 실측).
#[test]
fn orphan_field_end_links_to_the_open_field_id() {
    use crate::model::control::{Field, FieldType};
    use crate::model::paragraph::OrphanFieldEnd;

    let mut opener = Paragraph {
        text: "가".to_string(),
        ..Default::default()
    };
    opener.controls.push(Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: 2031845287,
        ..Default::default()
    }));

    let closer = Paragraph {
        text: "나".to_string(),
        orphan_field_ends: vec![OrphanFieldEnd {
            char_idx: 1,
            begin_id_ref: 0,
            field_id: 0,
            begin_ctrl_id: 0,
        }],
        ..Default::default()
    };

    let mut paras = vec![opener, closer];
    link_orphan_field_ends(&mut paras);
    assert_eq!(paras[1].orphan_field_ends[0].begin_id_ref, 2031845287);
}

/// 같은 문단에서 닫힌 필드는 열린 채로 쌓지 않는다 — 뒤 문단의 마커가 엉뚱한 필드를
/// 가리키면 안 된다.
#[test]
fn field_closed_in_its_own_paragraph_is_not_left_open() {
    use crate::model::control::{Field, FieldType};
    use crate::model::paragraph::{FieldRange, OrphanFieldEnd};

    let mut closed = Paragraph {
        text: "가".to_string(),
        ..Default::default()
    };
    closed.controls.push(Control::Field(Field {
        field_type: FieldType::ClickHere,
        field_id: 111,
        ..Default::default()
    }));
    closed.field_ranges.push(FieldRange {
        start_char_idx: 0,
        end_char_idx: 1,
        control_idx: 0,
        end_field_id: 0,
        inner_slot_count: 0,
    });

    let stray = Paragraph {
        orphan_field_ends: vec![OrphanFieldEnd {
            char_idx: 0,
            begin_id_ref: 0,
            field_id: 0,
            begin_ctrl_id: 0,
        }],
        ..Default::default()
    };

    let mut paras = vec![closed, stray];
    link_orphan_field_ends(&mut paras);
    assert_eq!(
        paras[1].orphan_field_ends[0].begin_id_ref, 0,
        "짝을 못 찾으면 0 으로 남긴다 — 없는 id 를 지어내지 않는다"
    );
}

// [#4827] 문단↔표↔셀 상호재귀 깊이 상한 회귀 — 손상 문서 스택 오버플로 DoS 가드
// ==========================================================================
//
// #4860 벌크빌드 byte-identity 하네스
// ===================================================================
//
// parse_para_text 의 벌크빌드(reserve + 평문 런 일괄 extend) 최적화가 최적화 이전
// 스칼라 구현(문자별 캐스케이드 + 개별 push)과 완전히 동일한 출력을 내는지 대조한다.
// 아래 `parse_para_text_reference` 는 현재 devel 의 최적화 이전 함수를 그대로 복사한
// 독립 스칼라 레퍼런스다. text·offsets·field_ranges·tab_extended·title_marks·
// orphan_field_ends 산출물을 모두 비교한다.

fn parse_para_text_reference(data: &[u8]) -> ParaTextParts {
    let mut text = String::new();
    let mut char_offsets: Vec<u32> = Vec::new();
    let mut field_ranges: Vec<FieldRange> = Vec::new();
    let mut tab_extended: Vec<[u16; 7]> = Vec::new();
    let mut title_marks: Vec<TitleMark> = Vec::new();
    let mut orphan_field_ends: Vec<OrphanFieldEnd> = Vec::new();
    let mut pos = 0;
    // 확장 컨트롤(extended) 카운터 → controls[] 인덱스와 1:1 대응
    let mut ctrl_idx: usize = 0;
    // text 문자열 내 문자 수 (바이트가 아닌 char 카운트)
    let mut char_count: usize = 0;
    // 현재 열린 필드 범위 스택 (중첩 필드 지원)
    let mut field_stack: Vec<(usize, usize)> = Vec::new(); // (start_char_idx, control_idx)

    while pos + 1 < data.len() {
        let code_unit_pos = (pos / 2) as u32; // UTF-16 코드 유닛 인덱스
        let ch = u16::from_le_bytes([data[pos], data[pos + 1]]);

        if ch == 0 {
            pos += 2;
        } else if ch == 0x0009 {
            // 탭: inline 컨트롤 (8 code unit = 16바이트)
            char_offsets.push(code_unit_pos);
            text.push('\t');
            char_count += 1;
            // TAB 확장 데이터 보존 (code unit 1~7: 탭 너비, 종류 등)
            let mut ext = [0u16; 7];
            for k in 0..7 {
                let bp = pos + 2 + k * 2;
                if bp + 1 < data.len() {
                    ext[k] = u16::from_le_bytes([data[bp], data[bp + 1]]);
                }
            }
            // 직렬화기의 "데이터 없음" 마커([0,...,0,0x0009] — body_text.rs 탭 방출부)는
            // IR 에 싣지 않는다. 한컴 실측 탭 확장은 ext[2] 고바이트=종류 enum+1 이라
            // 전부 0 일 수 없고, 이 마커를 tab_extended 로 실으면 레이아웃이 ext[0]=0 을
            // 탭 결과 위치로 해석해 탭이 무폭이 된다 (#1892 — tab_extended 없던 HWP3
            // 문단이 라운드트립 후 탭 스톱을 잃는 렌더 분기).
            let is_null_ext = ext[..6].iter().all(|&v| v == 0) && ext[6] == 0x0009;
            if !is_null_ext {
                tab_extended.push(ext);
            }
            pos += 16;
        } else if ch == 0x000A {
            // 줄 끝: char 컨트롤 (1 code unit = 2바이트)
            char_offsets.push(code_unit_pos);
            text.push('\n');
            char_count += 1;
            pos += 2;
        } else if ch == 0x000D {
            // 문단 끝
            break;
        } else if is_extended_ctrl_char(ch) {
            // 확장/인라인 컨트롤 문자: 8 code unit = 16바이트
            if ch == 0x0003 {
                // FIELD_BEGIN: 확장 컨트롤 → controls[]에 대응
                field_stack.push((char_count, ctrl_idx));
                ctrl_idx += 1;
            } else if ch == 0x0004 {
                // FIELD_END: 인라인 컨트롤 → controls[]에 대응하지 않음
                if let Some((start_idx, field_ctrl_idx)) = field_stack.pop() {
                    // HWP5 는 인라인 개체도 char_count 를 전진시키므로(8유닛 슬롯)
                    // 텍스트 축 0길이가 곧 "안쪽이 비었다"를 뜻한다 — 별도 보정 불필요.
                    field_ranges.push(FieldRange {
                        start_char_idx: start_idx,
                        end_char_idx: char_count,
                        control_idx: field_ctrl_idx,
                        end_field_id: 0,
                        inner_slot_count: ctrl_idx.saturating_sub(field_ctrl_idx + 1),
                    });
                } else {
                    // 짝 FIELD_BEGIN 이 **앞 문단**에 있는 다단락 필드의 종료 마커.
                    //
                    // 종전에는 스택이 비면 아무것도 남기지 않고 흘려보냈다. 그러면 이
                    // 8유닛 슬롯이 IR 에서 사라져 문단 축이 그만큼 짧아지고, 원본
                    // lineseg 의 `textpos` 가 범위 밖을 가리켜 그 문단의 조판이 통째로
                    // 버려진다(01752 문단 13 실측: 한컴 lineseg 11 / rhwp 6, 쪽수 1→2).
                    //
                    // HWPX 파서는 이미 같은 것을 `orphan_field_ends` 로 보존한다
                    // (Task #1556). HWP5 쪽만 비어 있었다.
                    orphan_field_ends.push(OrphanFieldEnd {
                        char_idx: char_count,
                        // HWP5 PARA_TEXT 의 종료 마커는 짝 id 를 싣지 않는다
                        // (`04 00 6b 6c 63 09 01 00 …`). `link_orphan_field_ends` 가
                        // 섹션을 훑어 채운다.
                        begin_id_ref: 0,
                        field_id: 0,
                        begin_ctrl_id: 0,
                    });
                }
            } else if is_extended_only_ctrl_char(ch) {
                // extended 컨트롤 (CTRL_HEADER 있음) → ctrl_idx 증가
                ctrl_idx += 1;
            }
            // inline 컨트롤 (4-9, 19-20 중 0x04 제외): ctrl_idx 증가 없음
            //
            // 제목 차례 표시(0x08 + `Mtit`/`Mign`)는 CTRL_HEADER 가 없어 `controls[]` 에
            // 실을 자리가 없다. 그렇다고 그냥 흘려보내면 8유닛 슬롯이 IR 에서 사라져
            // 저장본의 문단 축이 그만큼 짧아지고, 한글은 어긋난 `textpos` 를 만나면
            // 본문을 통째로 버린다(10k 스윕 F-절단군). `title_marks` 로 위치만 보존한다.
            if ch == 0x0008 && pos + 5 < data.len() {
                let ctrl_id = u32::from_le_bytes([
                    data[pos + 2],
                    data[pos + 3],
                    data[pos + 4],
                    data[pos + 5],
                ]);
                match ctrl_id {
                    tags::CTRL_TITLE_MARK_IGNORE_ON => title_marks.push(TitleMark {
                        char_idx: char_count,
                        ignore: true,
                    }),
                    tags::CTRL_TITLE_MARK_IGNORE_OFF => title_marks.push(TitleMark {
                        char_idx: char_count,
                        ignore: false,
                    }),
                    _ => {}
                }
            }
            // 자동번호(0x12) / 새번호(0x12): 텍스트에 공백 placeholder 추가
            // → apply_auto_numbers_to_composed에서 "  " (연속 2공백)으로 번호 삽입
            if ch == 0x0012 {
                char_offsets.push(code_unit_pos);
                text.push(' ');
                char_count += 1;
            }
            pos += 16;
        } else if ch < 0x0020 {
            // 문자 컨트롤 (1 code unit = 2바이트)
            match ch {
                0x0018 => {
                    char_offsets.push(code_unit_pos);
                    // 하이픈 (HWP 5.0 표 7: 코드 24) — 줄바꿈 자리에서만 보이는
                    // **소프트 하이픈**이다. 한글은 텍스트 추출에 싣지 않는다.
                    // 종전처럼 '-'(U+002D)로 내리면 실제 하이픈과 구별할 수 없어
                    // HWPX 저장본이 `pertinent` 를 `per-tinent` 로 만든다
                    // (10k 스윕 G-순수증식). #4675 가 U+2007 을 `<hp:fwSpace/>` 로
                    // 옮긴 것과 같은 계열 — 고유 코드포인트로 받아 요소로 되돌린다.
                    text.push('\u{00AD}');
                    char_count += 1;
                }
                0x0019 => {
                    char_offsets.push(code_unit_pos);
                    text.push(' '); // 예약 (코드 25-29) — 호환성 위해 공백 유지
                    char_count += 1;
                }
                0x001E => {
                    char_offsets.push(code_unit_pos);
                    text.push('\u{00A0}'); // 묶음 빈칸 (HWP 5.0 표 7: 코드 30, NO-BREAK SPACE)
                    char_count += 1;
                }
                0x001F => {
                    char_offsets.push(code_unit_pos);
                    text.push('\u{2007}'); // 고정폭 빈칸 (HWP 5.0 표 7: 코드 31, FIGURE SPACE)
                    char_count += 1;
                }
                _ => {}
            }
            pos += 2;
        } else {
            // 일반 문자 (서로게이트 페어 처리)
            if (0xD800..=0xDBFF).contains(&ch) && pos + 3 < data.len() {
                let low = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
                if (0xDC00..=0xDFFF).contains(&low) {
                    let code_point = 0x10000 + ((ch as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
                    if let Some(c) = char::from_u32(code_point) {
                        char_offsets.push(code_unit_pos);
                        text.push(c);
                        char_count += 1;
                    }
                    pos += 4;
                    continue;
                }
            }
            if let Some(c) = char::from_u32(ch as u32) {
                char_offsets.push(code_unit_pos);
                text.push(c);
                char_count += 1;
            }
            pos += 2;
        }
    }

    ParaTextParts {
        text,
        char_offsets,
        field_ranges,
        tab_extended,
        title_marks,
        orphan_field_ends,
    }
}

/// extended 컨트롤 문자 여부 (CTRL_HEADER 레코드가 있는 컨트롤)
///
/// HWP 5.0 제어 문자 분류 (표 6):
///   extended: 1-3, 11-12, 14-18, 21-23
///   inline: 4-9, 19-20
fn title_mark_key(m: &TitleMark) -> (usize, bool) {
    (m.char_idx, m.ignore)
}

fn orphan_end_key(o: &OrphanFieldEnd) -> (usize, u32, u32, u32) {
    (o.char_idx, o.begin_id_ref, o.field_id, o.begin_ctrl_id)
}

/// FieldRange 는 PartialEq 를 파생하지 않으므로 비교용 튜플로 사영한다.
fn field_range_key(f: &FieldRange) -> (usize, usize, usize, u32, usize) {
    (
        f.start_char_idx,
        f.end_char_idx,
        f.control_idx,
        f.end_field_id,
        f.inner_slot_count,
    )
}

fn assert_decode_identical(data: &[u8]) {
    let new = parse_para_text(data);
    let reference = parse_para_text_reference(data);
    assert_eq!(new.text, reference.text, "text 불일치: {:02x?}", data);
    assert_eq!(
        new.char_offsets, reference.char_offsets,
        "offsets 불일치: {:02x?}",
        data
    );
    let fr_new: Vec<_> = new.field_ranges.iter().map(field_range_key).collect();
    let fr_ref: Vec<_> = reference.field_ranges.iter().map(field_range_key).collect();
    assert_eq!(fr_new, fr_ref, "field_ranges 불일치: {:02x?}", data);
    assert_eq!(
        new.tab_extended, reference.tab_extended,
        "tab_extended 불일치: {:02x?}",
        data
    );
    let tm_new: Vec<_> = new.title_marks.iter().map(title_mark_key).collect();
    let tm_ref: Vec<_> = reference.title_marks.iter().map(title_mark_key).collect();
    assert_eq!(tm_new, tm_ref, "title_marks 불일치: {:02x?}", data);
    let oe_new: Vec<_> = new.orphan_field_ends.iter().map(orphan_end_key).collect();
    let oe_ref: Vec<_> = reference
        .orphan_field_ends
        .iter()
        .map(orphan_end_key)
        .collect();
    assert_eq!(oe_new, oe_ref, "orphan_field_ends 불일치: {:02x?}", data);
    assert_eq!(
        new.text.chars().count(),
        new.char_offsets.len(),
        "char/offset 개수 불변식 위반: {:02x?}",
        data
    );
}

fn push_u16(buf: &mut Vec<u8>, v: u16) {
    buf.extend_from_slice(&v.to_le_bytes());
}

struct SplitMix64(u64);
impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

fn bulkbuild_identical_exhaustive_single_unit() {
    for v in 0..=u16::MAX {
        assert_decode_identical(&v.to_le_bytes());

        let mut embedded = Vec::new();
        push_u16(&mut embedded, 0x0041);
        push_u16(&mut embedded, v);
        push_u16(&mut embedded, 0x0042);
        assert_decode_identical(&embedded);

        let mut odd_tail = Vec::new();
        push_u16(&mut odd_tail, 0x0041);
        push_u16(&mut odd_tail, v);
        odd_tail.push(0x99);
        assert_decode_identical(&odd_tail);
    }
}

fn bulkbuild_identical_run_length_sweep() {
    let boundaries = [
        0x0000u16, 0x0009, 0x000A, 0x000B, 0x0012, 0x0018, 0x001F, 0xD800, 0xDC00,
    ];
    for len in 0..=80usize {
        for &boundary in &boundaries {
            let mut buf = Vec::new();
            for i in 0..len {
                push_u16(&mut buf, 0x0041 + (i % 26) as u16);
            }
            push_u16(&mut buf, boundary);
            for i in 0..len {
                push_u16(&mut buf, 0xAC00 + (i % 100) as u16);
            }
            assert_decode_identical(&buf);

            let mut odd = buf.clone();
            odd.push(0x77);
            assert_decode_identical(&odd);
        }
    }
}

fn bulkbuild_identical_surrogates() {
    let highs = [0xD800u16, 0xD83D, 0xDBFF];
    let lows = [0xDC00u16, 0xDE00, 0xDFFF];
    let others = [0x0000u16, 0x0041, 0x000D, 0xAC00, 0xDBFF, 0xDC00];

    for &h in &highs {
        for &l in &lows {
            let mut a = Vec::new();
            push_u16(&mut a, 0x0041);
            push_u16(&mut a, h);
            push_u16(&mut a, l);
            push_u16(&mut a, 0x0042);
            assert_decode_identical(&a);

            let mut split = Vec::new();
            push_u16(&mut split, h);
            push_u16(&mut split, 0x000A);
            push_u16(&mut split, l);
            assert_decode_identical(&split);
        }
        for &o in &others {
            let mut b = Vec::new();
            push_u16(&mut b, h);
            push_u16(&mut b, o);
            assert_decode_identical(&b);
        }
        assert_decode_identical(&h.to_le_bytes());
        let mut c = h.to_le_bytes().to_vec();
        c.push(0x00);
        assert_decode_identical(&c);
    }
    for &l in &lows {
        assert_decode_identical(&l.to_le_bytes());
    }
}

fn bulkbuild_identical_random_fuzz() {
    let mut rng = SplitMix64(0x0BAD_C0DE_CAFE_F00D);
    let mut buf = Vec::with_capacity(128);
    let iters = 320_000;
    for _ in 0..iters {
        buf.clear();
        let n_units = (rng.next_u64() % 41) as usize;
        for _ in 0..n_units {
            let r = rng.next_u64();
            let val = match r & 0x3 {
                0 | 1 => (r >> 8) as u16,
                2 => ((r >> 8) as u16) & 0x001F,
                _ => 0xD800 | (((r >> 8) as u16) & 0x07FF),
            };
            push_u16(&mut buf, val);
        }
        if rng.next_u64() & 0x3 == 0 {
            buf.push((rng.next_u64() >> 16) as u8);
        }
        assert_decode_identical(&buf);
    }
}

/// 표 `depth` 겹을 선형 중첩한 BodyText 레코드 바이트 스트림을 만든다.
///
/// 한 겹 = PARA_HEADER(L) → CTRL_HEADER(L+1, `tbl `) → HWPTAG_TABLE(L+2) →
/// LIST_HEADER(L+2, 셀). 셀 안의 다음 PARA_HEADER 는 L+3 — 즉 표 한 겹이 레코드 레벨을 3 판다.
/// 레벨 필드는 10비트(≤1023)라 이 방식으로 최대 ~341겹까지 만들 수 있다(실파일 도달 한계).
fn build_nested_table_stream(depth: u16) -> Vec<u8> {
    let para = make_para_header_data(0, 0, 0);
    let table_data = [0u8; 4];
    let cell_data = [0u8; 32];
    let ctrl = tags::CTRL_TABLE.to_le_bytes();

    let mut bytes = Vec::new();
    for k in 0..depth {
        let l = 3 * k;
        bytes.extend(make_record_bytes(tags::HWPTAG_PARA_HEADER, l, &para));
        bytes.extend(make_record_bytes(tags::HWPTAG_CTRL_HEADER, l + 1, &ctrl));
        bytes.extend(make_record_bytes(tags::HWPTAG_TABLE, l + 2, &table_data));
        bytes.extend(make_record_bytes(
            tags::HWPTAG_LIST_HEADER,
            l + 2,
            &cell_data,
        ));
    }
    bytes.extend(make_record_bytes(
        tags::HWPTAG_PARA_HEADER,
        3 * depth,
        &para,
    ));
    bytes
}

/// 파싱된 문단 트리에서 최대 표 중첩 깊이를 잰다(표→셀→문단 재귀).
fn max_table_nesting(paras: &[crate::model::paragraph::Paragraph]) -> usize {
    let mut best = 0;
    for p in paras {
        for c in &p.controls {
            if let Control::Table(t) = c {
                let mut deepest = 0;
                for cell in &t.cells {
                    deepest = deepest.max(max_table_nesting(&cell.paragraphs));
                }
                best = best.max(1 + deepest);
            }
        }
    }
    best
}

#[test]
fn nested_table_recursion_is_depth_capped() {
    // 상한(64)을 크게 넘는 표 중첩을 파싱해도 크래시 없이 완주하고, 결과 트리의 표 중첩
    // 깊이가 상한 이내로 절단돼야 한다. 가드가 없으면 이 입력은 341겹 근처에서 스택을
    // 고갈시켜 SIGSEGV 를 내거나(비결정적) 입력 깊이 그대로 내려간다. 넉넉한 스택 전용
    // 스레드에서 경계를 결정론적으로 시험한다(HWPX #4759 형제 테스트와 같은 방식).
    let input_depth = MAX_HWP5_SECTION_DEPTH + 40;
    let nesting = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let stream = build_nested_table_stream(input_depth as u16);
            let section = parse_body_text_section(&stream).expect("파싱은 성공(하위 트리만 절단)");
            max_table_nesting(&section.paragraphs)
        })
        .expect("파서 스레드 생성 실패")
        .join()
        .expect("파서 스레드 패닉");

    assert!(
        nesting <= MAX_HWP5_SECTION_DEPTH as usize,
        "표 중첩이 상한을 넘겨 절단되지 않았다 — 상호재귀 깊이 가드 회귀 (nesting={nesting})"
    );
    assert!(
        nesting >= 8,
        "가드가 얕은 깊이에서 과잉 차단했다 (nesting={nesting})"
    );
}

#[test]
fn shallow_table_nesting_is_preserved() {
    // 상한 안쪽의 정상적인 표 중첩은 깊이 그대로 보존돼야 한다(가드가 과잉 차단 안 함).
    let stream = build_nested_table_stream(5);
    let section = parse_body_text_section(&stream).expect("파싱 실패");
    assert_eq!(
        max_table_nesting(&section.paragraphs),
        5,
        "정상 깊이(5겹) 표 중첩이 보존되지 않았다 — 가드 과잉 차단"
    );
}
