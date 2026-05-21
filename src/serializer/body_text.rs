//! BodyText 섹션 직렬화
//!
//! `parser::body_text`의 역방향으로, Section/Paragraph를 레코드 스트림으로 변환한다.
//!
//! 레코드 구조:
//! ```text
//! PARA_HEADER (level 0)
//!   PARA_TEXT (level 1)
//!   PARA_CHAR_SHAPE (level 1)
//!   PARA_LINE_SEG (level 1)
//!   PARA_RANGE_TAG (level 1)
//!   CTRL_HEADER (level 1)
//!     ... (level 2+)
//! ```

use super::byte_writer::ByteWriter;
use super::record_writer::write_records;

use crate::model::control::Control;
use crate::model::document::Section;
use crate::model::paragraph::{CharShapeRef, ColumnBreakType, LineSeg, Paragraph, RangeTag};
use crate::parser::record::Record;
use crate::parser::tags;

/// Section을 레코드 바이너리 스트림으로 직렬화
pub fn serialize_section(section: &Section) -> Vec<u8> {
    // 원본 스트림이 있으면 그대로 반환 (완벽한 라운드트립)
    if let Some(ref raw) = section.raw_stream {
        return raw.clone();
    }

    // [Task #852 Stage 2.4] Form 컨트롤의 z-order/TabOrder 카운터 reset.
    // 한 섹션 내 Form 등장순으로 0..N-1 부여 → 정답지 패턴 재현.
    super::control::reset_form_order_counter();

    let mut records = Vec::new();
    let para_count = section.paragraphs.len();
    for (i, para) in section.paragraphs.iter().enumerate() {
        let is_last = i == para_count - 1;
        serialize_paragraph_with_msb(para, 0, is_last, &mut records);
    }
    write_records(&records)
}

/// 문단 목록을 레코드로 직렬화 (재귀용: 셀, 머리말/꼬리말, 각주/미주 내부)
pub fn serialize_paragraph_list(
    paragraphs: &[Paragraph],
    base_level: u16,
    records: &mut Vec<Record>,
) {
    let para_count = paragraphs.len();
    for (i, para) in paragraphs.iter().enumerate() {
        let is_last = i == para_count - 1;
        serialize_paragraph_with_msb(para, base_level, is_last, records);
    }
}

/// 단일 문단을 레코드로 직렬화 (MSB를 위치 기반으로 강제 설정)
///
/// is_last: 이 문단이 현재 스코프(섹션/셀/텍스트박스 등)의 마지막 문단인지 여부
fn serialize_paragraph_with_msb(
    para: &Paragraph,
    base_level: u16,
    is_last: bool,
    records: &mut Vec<Record>,
) {
    // HWP는 모든 문단에 최소 1개의 PARA_CHAR_SHAPE 엔트리 필요
    // char_shapes가 비어있으면 기본 엔트리(위치 0, char_shape_id 0)를 사용
    let default_char_shape = [CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    let effective_char_shapes: &[CharShapeRef] = if para.char_shapes.is_empty() {
        &default_char_shape
    } else {
        &para.char_shapes
    };

    // control_mask 재계산: 실제 controls에서 비트 마스크를 산출한다.
    // 모델의 control_mask가 controls와 불일치하면 한컴이 파일 손상으로 판단하므로,
    // 직렬화 시점에 항상 재계산하여 일관성을 보장한다.
    let actual_control_mask = compute_control_mask(para);

    // PARA_TEXT를 먼저 직렬화하여 실제 char_count를 계산한다.
    // char_count가 PARA_TEXT code unit 수와 불일치하면 한컴이 파일 손상으로 판단한다.
    let has_content = !para.text.is_empty() || !para.controls.is_empty();
    let text_data = if has_content || (para.has_para_text && para.char_count > 1) {
        Some(serialize_para_text(para))
    } else {
        None
    };

    // char_count 재계산: PARA_TEXT가 있으면 code unit 수, 없으면 모델 값 사용
    let actual_char_count = if let Some(ref td) = text_data {
        (td.len() / 2) as u32
    } else {
        para.char_count
    };

    // PARA_HEADER (effective_char_shapes 길이 반영)
    // MSB는 모델 값이 아닌 위치 기반으로 결정: 마지막 문단만 MSB=true
    records.push(Record {
        tag_id: tags::HWPTAG_PARA_HEADER,
        level: base_level,
        size: 0,
        data: serialize_para_header_with_mask(
            para,
            effective_char_shapes.len(),
            is_last,
            actual_control_mask,
            actual_char_count,
        ),
    });

    // PARA_TEXT
    if let Some(text_data) = text_data {
        records.push(Record {
            tag_id: tags::HWPTAG_PARA_TEXT,
            level: base_level + 1,
            size: text_data.len() as u32,
            data: text_data,
        });
    }

    // PARA_CHAR_SHAPE (항상 출력 — HWP 필수)
    {
        let data = serialize_para_char_shape(effective_char_shapes);
        records.push(Record {
            tag_id: tags::HWPTAG_PARA_CHAR_SHAPE,
            level: base_level + 1,
            size: data.len() as u32,
            data,
        });
    }

    // PARA_LINE_SEG
    if !para.line_segs.is_empty() {
        let data = serialize_para_line_seg(&para.line_segs);
        records.push(Record {
            tag_id: tags::HWPTAG_PARA_LINE_SEG,
            level: base_level + 1,
            size: data.len() as u32,
            data,
        });
    }

    // PARA_RANGE_TAG
    if !para.range_tags.is_empty() {
        let data = serialize_para_range_tag(&para.range_tags);
        records.push(Record {
            tag_id: tags::HWPTAG_PARA_RANGE_TAG,
            level: base_level + 1,
            size: data.len() as u32,
            data,
        });
    }

    // CTRL_HEADER (컨트롤별) + CTRL_DATA (있으면)
    for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
        let ctrl_data_record = para
            .ctrl_data_records
            .get(ctrl_idx)
            .and_then(|opt| opt.as_ref())
            .map(|v| v.as_slice());
        super::control::serialize_control(ctrl, base_level + 1, ctrl_data_record, records);
    }
}

/// 문단의 control_mask 비트를 계산한다.
///
/// 각 컨트롤의 char_code(제어 문자 코드)가 비트 위치에 대응:
/// - 0x0002 (SectionDef, ColumnDef) → bit 2 = 0x04
/// - 0x0003 (FIELD_BEGIN) → bit 3 = 0x08
/// - 0x0004 (FIELD_END) → bit 4 = 0x10
/// - 0x0009 (TAB) → bit 9 = 0x200
/// - 0x000B (Table, Shape, Picture) → bit 11 = 0x800
/// - 0x0010 (Header, Footer) → bit 16 = 0x10000
/// - etc.
fn compute_control_mask(para: &Paragraph) -> u32 {
    let mut mask: u32 = 0;
    for ctrl in &para.controls {
        let (char_code, _) = control_char_code_and_id(ctrl);
        mask |= 1u32 << char_code;
    }
    // FIELD_END (0x0004): field_ranges가 있으면 비트 4 설정
    if !para.field_ranges.is_empty() {
        mask |= 1u32 << 0x0004;
    }
    // TAB (0x0009): text에 탭이 있으면 비트 9 설정
    if para.text.contains('\t') {
        mask |= 1u32 << 0x0009;
    }
    // LINE_BREAK (0x000A): text에 줄바꿈이 있으면 비트 10 설정
    if para.text.contains('\n') {
        mask |= 1u32 << 0x000A;
    }
    mask
}

/// PARA_HEADER 직렬화 (control_mask를 외부에서 전달)
///
/// 레이아웃: char_count(u32) + control_mask(u32) + para_shape_id(u16) + style_id(u8) + break_type(u8)
/// + numCharShapes(u16) + numRangeTags(u16) + numLineSegs(u16) + instanceId(u32) + [추가 바이트]
fn serialize_para_header_with_mask(
    para: &Paragraph,
    num_char_shapes: usize,
    is_last: bool,
    control_mask: u32,
    char_count: u32,
) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // MSB는 위치 기반으로 결정: 현재 스코프의 마지막 문단만 MSB=1
    let char_count_raw = char_count | if is_last { 0x80000000 } else { 0 };
    w.write_u32(char_count_raw).unwrap();
    w.write_u32(control_mask).unwrap();
    w.write_u16(para.para_shape_id).unwrap();
    w.write_u8(para.style_id).unwrap();

    let break_val: u8 = if para.raw_break_type != 0 {
        para.raw_break_type
    } else {
        match para.column_type {
            ColumnBreakType::Section => 0x01,
            ColumnBreakType::MultiColumn => 0x02,
            ColumnBreakType::Page => 0x04,
            ColumnBreakType::Column => 0x08,
            ColumnBreakType::None => 0x00,
        }
    };
    w.write_u8(break_val).unwrap();

    // count 필드는 실제 데이터 기반으로 항상 재생성 (편집 후 불일치 방지)
    w.write_u16(num_char_shapes as u16).unwrap();
    w.write_u16(para.range_tags.len() as u16).unwrap();
    w.write_u16(para.line_segs.len() as u16).unwrap();

    // instanceId + 추가 바이트: raw_header_extra에서 복원
    // raw_header_extra[0..5] = numCharShapes(2) + numRangeTags(2) + numLineSegs(2) → 건너뜀
    // raw_header_extra[6..] = instanceId(4) + 나머지
    if para.raw_header_extra.len() >= 10 {
        let extra = &para.raw_header_extra[6..];
        w.write_bytes(extra).unwrap();
    } else {
        // 새 문단 (raw_header_extra 없음): instanceId(4)만 기록
        w.write_u32(0).unwrap();
    }

    w.into_bytes()
}

/// 확장 컨트롤 문자 8 code unit을 code_units에 추가
///
/// 구조 (16바이트 = 8 code units):
///   code_unit[0]: 제어 문자 코드 (0x0002, 0x000B 등)
///   code_unit[1-2]: ctrl_id (u32 LE → 2 code units)
///   code_unit[3-6]: 0 (예약)
///   code_unit[7]: 제어 문자 코드 반복 (HWP 관례)
fn push_extended_ctrl(code_units: &mut Vec<u16>, ctrl_code: u16, ctrl_id: u32) {
    code_units.push(ctrl_code);
    // ctrl_id를 2개의 u16 code units로 변환 (LE)
    let id_bytes = ctrl_id.to_le_bytes();
    code_units.push(u16::from_le_bytes([id_bytes[0], id_bytes[1]]));
    code_units.push(u16::from_le_bytes([id_bytes[2], id_bytes[3]]));
    // 예약 (4 code units)
    for _ in 0..4 {
        code_units.push(0);
    }
    // 마지막 code unit: 제어 문자 코드 반복
    code_units.push(ctrl_code);
}

/// PARA_TEXT 직렬화
///
/// 텍스트 + 컨트롤 문자를 UTF-16LE로 변환한다.
/// char_offsets를 사용하여 각 문자의 원본 UTF-16 위치를 결정하고,
/// 위치 간 갭(8 code unit)에 컨트롤 문자를 배치한다.
/// 테스트용 public wrapper
#[cfg(test)]
pub fn test_serialize_para_text(para: &Paragraph) -> Vec<u8> {
    serialize_para_text(para)
}

fn serialize_para_text(para: &Paragraph) -> Vec<u8> {
    let mut code_units: Vec<u16> = Vec::new();
    let text_chars: Vec<char> = para.text.chars().collect();
    let mut ctrl_idx = 0;
    let mut prev_end: u32 = 0;
    let mut tab_idx: usize = 0; // TAB 확장 데이터 인덱스

    // field_ranges에서 FIELD_END 삽입 정보를 수집
    // 두 종류로 분류:
    // 1. mid-text: end_char_idx < text_chars.len() → 해당 텍스트 문자 앞 갭에 삽입
    // 2. trailing: end_char_idx == text_chars.len() → 남은 컨트롤과 인터리빙
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    let text_len = para.text.chars().count();
    let mut field_ends: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    // trailing FIELD_END: control_idx → ctrl_id 매핑 (FIELD_BEGIN 직후에 삽입)
    let mut trailing_end_after_ctrl: HashMap<usize, Vec<u32>> = HashMap::new();
    // trailing FIELD_END 중 FIELD_BEGIN이 이미 본문에 배치된 경우 (orphan)
    let mut trailing_orphan_ends: Vec<u32> = Vec::new();

    for fr in &para.field_ranges {
        let ctrl_id = if let Some(crate::model::control::Control::Field(f)) =
            para.controls.get(fr.control_idx)
        {
            f.ctrl_id
        } else {
            0
        };
        if fr.end_char_idx < text_len {
            field_ends.entry(fr.end_char_idx).or_default().push(ctrl_id);
        } else {
            // trailing FIELD_END: control_idx가 남은 컨트롤에 포함되는지 판별은
            // 메인 루프 후에 수행 (ctrl_idx 확정 후)
            trailing_end_after_ctrl
                .entry(fr.control_idx)
                .or_default()
                .push(ctrl_id);
        }
    }

    for (i, ch) in text_chars.iter().enumerate() {
        let offset = if i < para.char_offsets.len() {
            para.char_offsets[i]
        } else {
            prev_end
        };

        // [Task #1050] AutoNumber placeholder 검출:
        // char_offsets[i] == prev_end 이고 ch == ' ' 이고 다음 char_offset 이 prev_end + 8 +
        // (실제 char 폭)인 경우 = placeholder space (i char 한 자리 차지 + 다음 char 가 8 점프 후).
        // 이 경우 ' ' 대신 AUTO_NUMBER 컨트롤 8 cu 작성 + prev_end = offset + 8.
        let next_offset = if i + 1 < para.char_offsets.len() {
            Some(para.char_offsets[i + 1])
        } else {
            None
        };
        let is_autonum_placeholder = *ch == ' '
            && offset == prev_end
            && ctrl_idx < para.controls.len()
            && matches!(
                control_char_code_and_id(&para.controls[ctrl_idx]).0,
                0x0011 | 0x0012
            )
            && next_offset.map_or(false, |n| n >= offset + 8);
        if is_autonum_placeholder {
            let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
            push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);
            ctrl_idx += 1;
            prev_end = offset + 8;
            continue;
        }

        // 갭에 컨트롤 문자 배치 (각 컨트롤 = 8 code unit)
        while prev_end + 8 <= offset && ctrl_idx < para.controls.len() {
            let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
            push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);
            ctrl_idx += 1;
            prev_end += 8;
        }

        // FIELD_END 삽입: 컨트롤(FIELD_BEGIN) 뒤, 텍스트 문자 앞
        if let Some(ids) = field_ends.get(&i) {
            for &ctrl_id in ids {
                push_extended_ctrl(&mut code_units, 0x0004, ctrl_id);
                prev_end += 8;
            }
        }

        // 텍스트 문자 쓰기
        match *ch {
            '\t' => {
                code_units.push(0x0009);
                // TAB 확장 데이터 복원 (탭 너비, 종류 등)
                if tab_idx < para.tab_extended.len() {
                    for &cu in &para.tab_extended[tab_idx] {
                        code_units.push(cu);
                    }
                } else {
                    for _ in 0..7 {
                        code_units.push(0);
                    }
                }
                tab_idx += 1;
                prev_end = offset + 8;
            }
            '\n' => {
                code_units.push(0x000A);
                prev_end = offset + 1;
            }
            '\u{00A0}' => {
                code_units.push(0x0018);
                prev_end = offset + 1;
            }
            c => {
                let mut buf = [0u16; 2];
                let encoded = c.encode_utf16(&mut buf);
                for cu in encoded.iter() {
                    code_units.push(*cu);
                }
                prev_end = offset + encoded.len() as u32;
            }
        }
    }

    // 남은 컨트롤 배치 + trailing FIELD_END 인터리빙
    // FIELD_BEGIN 컨트롤 직후에 대응하는 FIELD_END를 삽입하여 올바른 순서를 보장한다.
    while ctrl_idx < para.controls.len() {
        let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
        push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);

        // 이 컨트롤(FIELD_BEGIN)에 대응하는 trailing FIELD_END 삽입
        if let Some(end_ids) = trailing_end_after_ctrl.remove(&ctrl_idx) {
            for eid in end_ids {
                push_extended_ctrl(&mut code_units, 0x0004, eid);
            }
        }

        ctrl_idx += 1;
    }

    // orphan trailing FIELD_END: FIELD_BEGIN이 본문 갭에서 이미 배치된 경우
    // (trailing_end_after_ctrl에 남아있는 항목 = ctrl_idx가 이미 소진된 컨트롤)
    for end_ids in trailing_end_after_ctrl.values() {
        for &eid in end_ids {
            push_extended_ctrl(&mut code_units, 0x0004, eid);
        }
    }

    // 문단 끝 마커
    code_units.push(0x000D);

    // UTF-16LE 바이트로 변환
    let mut bytes = Vec::with_capacity(code_units.len() * 2);
    for cu in &code_units {
        bytes.extend_from_slice(&cu.to_le_bytes());
    }
    bytes
}

/// PARA_CHAR_SHAPE 직렬화
///
/// 각 항목: start_pos(u32) + char_shape_id(u32) = 8바이트
fn serialize_para_char_shape(char_shapes: &[CharShapeRef]) -> Vec<u8> {
    let mut w = ByteWriter::new();
    for cs in char_shapes {
        w.write_u32(cs.start_pos).unwrap();
        w.write_u32(cs.char_shape_id).unwrap();
    }
    w.into_bytes()
}

/// PARA_LINE_SEG 직렬화
///
/// 각 항목: 36바이트 (u32 + i32×7 + u32)
fn serialize_para_line_seg(line_segs: &[LineSeg]) -> Vec<u8> {
    let mut w = ByteWriter::new();
    for seg in line_segs {
        w.write_u32(seg.text_start).unwrap();
        w.write_i32(seg.vertical_pos).unwrap();
        w.write_i32(seg.line_height).unwrap();
        w.write_i32(seg.text_height).unwrap();
        w.write_i32(seg.baseline_distance).unwrap();
        w.write_i32(seg.line_spacing).unwrap();
        w.write_i32(seg.column_start).unwrap();
        w.write_i32(seg.segment_width).unwrap();
        w.write_u32(seg.tag).unwrap();
    }
    w.into_bytes()
}

/// PARA_RANGE_TAG 직렬화
///
/// 각 항목: 12바이트 (u32 × 3)
fn serialize_para_range_tag(range_tags: &[RangeTag]) -> Vec<u8> {
    let mut w = ByteWriter::new();
    for rt in range_tags {
        w.write_u32(rt.start).unwrap();
        w.write_u32(rt.end).unwrap();
        w.write_u32(rt.tag).unwrap();
    }
    w.into_bytes()
}

/// 컨트롤에 대응하는 PARA_TEXT 내 제어 문자 코드와 ctrl_id를 반환
///
/// HWP 5.0 제어 문자 분류 (표 6):
///   0x0002: 구역/단 정의 (secd, cold)
///   0x000B: 표/그림/도형 (tbl, gso)
///   0x000F: 숨은 설명 (tcmt)
///   0x0010: 머리말/꼬리말 (head, foot)
///   0x0011: 각주/미주 (fn, en)
///   0x0012: 자동번호/새번호 (atno, nwno)
///   0x0015: 페이지 컨트롤 (pgnp, pghi)
///   0x0016: 책갈피 (bokm)
fn control_char_code_and_id(ctrl: &Control) -> (u16, u32) {
    match ctrl {
        Control::SectionDef(_) => (0x0002, tags::CTRL_SECTION_DEF),
        Control::ColumnDef(_) => (0x0002, tags::CTRL_COLUMN_DEF),
        Control::Table(_) => (0x000B, tags::CTRL_TABLE),
        Control::Shape(_) => (0x000B, tags::CTRL_GEN_SHAPE),
        Control::Picture(_) => (0x000B, tags::CTRL_GEN_SHAPE),
        Control::HiddenComment(_) => (0x000F, tags::CTRL_HIDDEN_COMMENT),
        Control::Header(_) => (0x0010, tags::CTRL_HEADER),
        Control::Footer(_) => (0x0010, tags::CTRL_FOOTER),
        Control::Footnote(_) => (0x0011, tags::CTRL_FOOTNOTE),
        Control::Endnote(_) => (0x0011, tags::CTRL_ENDNOTE),
        Control::AutoNumber(_) => (0x0012, tags::CTRL_AUTO_NUMBER),
        Control::NewNumber(_) => (0x0012, tags::CTRL_NEW_NUMBER),
        Control::PageNumberPos(_) => (0x0015, tags::CTRL_PAGE_NUM_POS),
        Control::PageHide(_) => (0x0015, tags::CTRL_PAGE_HIDE),
        Control::Bookmark(_) => (0x0016, tags::CTRL_BOOKMARK),
        Control::Hyperlink(_) => (0x000B, 0),
        Control::Ruby(_) => (0x000B, 0),
        Control::CharOverlap(_) => (0x0017, tags::CTRL_TCPS),
        Control::Field(f) => (0x0003, f.ctrl_id),
        Control::Equation(_) => (0x000B, tags::CTRL_EQUATION),
        Control::Form(_) => (0x000B, tags::CTRL_FORM),
        Control::Unknown(u) => (0x000B, u.ctrl_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::AutoNumber;
    use crate::model::document::{Section, SectionDef};
    use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph, RangeTag};
    use crate::parser::body_text::parse_body_text_section;

    /// 간단한 텍스트 문단 라운드트립
    #[test]
    fn test_roundtrip_simple_text() {
        let para = Paragraph {
            char_count: 6,
            text: "Hello".to_string(),
            char_offsets: vec![0, 1, 2, 3, 4],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 400,
                text_height: 400,
                baseline_distance: 320,
                ..Default::default()
            }],
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
        assert_eq!(parsed.paragraphs[0].text, "Hello");
        assert_eq!(parsed.paragraphs[0].char_offsets, vec![0, 1, 2, 3, 4]);
    }

    /// 한글 텍스트 라운드트립
    #[test]
    fn test_roundtrip_korean_text() {
        let para = Paragraph {
            char_count: 10,
            text: "한글 테스트입니다.".to_string(),
            char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 1,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs[0].text, "한글 테스트입니다.");
    }

    /// 탭 문자 포함 라운드트립
    #[test]
    fn test_roundtrip_with_tab() {
        let para = Paragraph {
            char_count: 4,
            text: "A\tB".to_string(),
            char_offsets: vec![0, 1, 9],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs[0].text, "A\tB");
        assert_eq!(parsed.paragraphs[0].char_offsets, vec![0, 1, 9]);
    }

    /// 줄바꿈 포함 라운드트립
    #[test]
    fn test_roundtrip_with_linebreak() {
        let para = Paragraph {
            char_count: 4,
            text: "A\nB".to_string(),
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
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs[0].text, "A\nB");
    }

    /// 빈 문단 직렬화
    #[test]
    fn test_serialize_empty_paragraph() {
        let para = Paragraph {
            char_count: 0,
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
        assert!(parsed.paragraphs[0].text.is_empty());
    }

    /// 여러 문단 라운드트립
    #[test]
    fn test_roundtrip_multiple_paragraphs() {
        let para1 = Paragraph {
            char_count: 4,
            text: "ABC".to_string(),
            char_offsets: vec![0, 1, 2],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            para_shape_id: 0,
            style_id: 0,
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        let para2 = Paragraph {
            char_count: 4,
            text: "DEF".to_string(),
            char_offsets: vec![0, 1, 2],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 1,
            }],
            para_shape_id: 1,
            style_id: 0,
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para1, para2],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs.len(), 2);
        assert_eq!(parsed.paragraphs[0].text, "ABC");
        assert_eq!(parsed.paragraphs[1].text, "DEF");
        assert_eq!(parsed.paragraphs[1].para_shape_id, 1);
    }

    /// PARA_CHAR_SHAPE 라운드트립
    #[test]
    fn test_roundtrip_char_shapes() {
        let para = Paragraph {
            char_count: 5,
            text: "ABCD".to_string(),
            char_offsets: vec![0, 1, 2, 3],
            char_shapes: vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 1,
                },
                CharShapeRef {
                    start_pos: 2,
                    char_shape_id: 3,
                },
            ],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs[0].char_shapes.len(), 2);
        assert_eq!(parsed.paragraphs[0].char_shapes[0].start_pos, 0);
        assert_eq!(parsed.paragraphs[0].char_shapes[0].char_shape_id, 1);
        assert_eq!(parsed.paragraphs[0].char_shapes[1].start_pos, 2);
        assert_eq!(parsed.paragraphs[0].char_shapes[1].char_shape_id, 3);
    }

    /// PARA_LINE_SEG 라운드트립
    #[test]
    fn test_roundtrip_line_segs() {
        let para = Paragraph {
            char_count: 3,
            text: "AB".to_string(),
            char_offsets: vec![0, 1],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                vertical_pos: 100,
                line_height: 500,
                text_height: 400,
                baseline_distance: 300,
                line_spacing: 200,
                column_start: 0,
                segment_width: 42000,
                tag: 0x01,
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs[0].line_segs.len(), 1);
        let seg = &parsed.paragraphs[0].line_segs[0];
        assert_eq!(seg.vertical_pos, 100);
        assert_eq!(seg.line_height, 500);
        assert_eq!(seg.segment_width, 42000);
        assert!(seg.is_first_line_of_page());
    }

    /// PARA_RANGE_TAG 라운드트립
    #[test]
    fn test_roundtrip_range_tags() {
        let para = Paragraph {
            char_count: 20,
            text: "ABCDEFGHIJKLMNOPQRS".to_string(),
            char_offsets: (0..19).collect(),
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            range_tags: vec![RangeTag {
                start: 5,
                end: 15,
                tag: 0x01000003,
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs[0].range_tags.len(), 1);
        assert_eq!(parsed.paragraphs[0].range_tags[0].start, 5);
        assert_eq!(parsed.paragraphs[0].range_tags[0].end, 15);
        assert_eq!(parsed.paragraphs[0].range_tags[0].tag, 0x01000003);
    }

    /// 컨트롤 문자 코드 매핑 테스트
    #[test]
    fn test_control_char_code() {
        assert_eq!(
            control_char_code_and_id(&Control::SectionDef(Box::new(SectionDef::default()))).0,
            0x0002
        );
        assert_eq!(
            control_char_code_and_id(&Control::AutoNumber(AutoNumber::default())).0,
            0x0012
        );
    }

    /// 확장 컨트롤 포함 문단 라운드트립
    #[test]
    fn test_roundtrip_with_section_def_control() {
        let sd = SectionDef {
            flags: 0,
            default_tab_spacing: 800,
            page_num: 1,
            ..Default::default()
        };

        let para = Paragraph {
            char_count: 4,
            text: "AB".to_string(),
            char_offsets: vec![0, 9], // 0~7 = secd 컨트롤, 8~8 gap? 아니, 0=A, 1~8=secd, 9=B
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

        assert_eq!(parsed.paragraphs[0].text, "AB");
        // SectionDef 컨트롤이 파싱되어 section_def에 반영
        assert_eq!(parsed.section_def.default_tab_spacing, 800);
    }

    /// 단 나누기 종류 라운드트립
    #[test]
    fn test_roundtrip_break_type() {
        let para = Paragraph {
            char_count: 2,
            text: "A".to_string(),
            char_offsets: vec![0],
            column_type: ColumnBreakType::Page,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();

        assert_eq!(parsed.paragraphs[0].column_type, ColumnBreakType::Page);
    }
}
