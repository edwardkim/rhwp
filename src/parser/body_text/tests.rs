use super::*;
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
fn test_parse_para_text_simple() {
    let (text, offsets, _, _) = parse_para_text(&make_para_text_data("Hello, World!"));
    assert_eq!(text, "Hello, World!");
    assert_eq!(offsets.len(), 13);
    assert_eq!(offsets[0], 0); // 'H' at position 0
}

#[test]
fn test_parse_para_text_korean() {
    let (text, offsets, _, _) = parse_para_text(&make_para_text_data("한글 테스트입니다."));
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
    let (text, offsets, _, _) = parse_para_text(&data);
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
    let (text, offsets, _, _) = parse_para_text(&data);
    assert_eq!(text, "AB");
    // 'A' at code unit 0, extended ctrl takes 8 units (1-8), 'B' at code unit 9
    assert_eq!(offsets, vec![0, 9]);
}

#[test]
fn test_parse_para_text_empty() {
    // 문단 끝만 있는 경우
    let data = 0x000Du16.to_le_bytes();
    let (text, offsets, _, _) = parse_para_text(&data);
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
    ctrl_data.extend_from_slice(&0u16.to_le_bytes()); // vertical_align
    ctrl_data.extend_from_slice(&0u16.to_le_bytes()); // horizontal_align
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
    assert_eq!(section.section_def.page_num, 1);
    assert_eq!(section.section_def.page_def.width, 59528);
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
