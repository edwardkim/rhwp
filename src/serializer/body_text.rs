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
    let memo_lists = collect_memo_lists(section);
    let has_memo_tail = !memo_lists.is_empty();
    let para_count = section.paragraphs.len();
    for (i, para) in section.paragraphs.iter().enumerate() {
        let is_last = i == para_count - 1 && !has_memo_tail;
        serialize_paragraph_with_msb(para, 0, is_last, &mut records);
    }
    if has_memo_tail {
        serialize_memo_tail(section, &memo_lists, &mut records);
    }
    serialize_master_page_tail(section, &mut records);
    write_records(&records)
}

fn serialize_master_page_tail(section: &Section, records: &mut Vec<Record>) {
    // HWPX LAST_PAGE master page is an extension master page. Hancom HWP5 files store
    // extension master pages after the body paragraph stream as level-1 LIST_HEADER
    // records, not inside the SectionDef child record group.
    if section
        .section_def
        .extra_child_records
        .iter()
        .any(|raw| raw.tag_id == tags::HWPTAG_LIST_HEADER && raw.level == 1)
    {
        return;
    }

    for master_page in section
        .section_def
        .master_pages
        .iter()
        .filter(|master_page| master_page.is_extension)
    {
        super::control::serialize_master_page(master_page, 1, records);
    }
}

fn collect_memo_lists(section: &Section) -> Vec<(u32, Vec<Paragraph>)> {
    let mut memo_lists = Vec::new();
    for para in &section.paragraphs {
        for ctrl in &para.controls {
            if let Control::Field(field) = ctrl {
                if field.field_type == crate::model::control::FieldType::Memo
                    && !field.memo_paragraphs.is_empty()
                {
                    memo_lists.push((field.memo_index, field.memo_paragraphs.clone()));
                }
            }
        }
    }
    memo_lists
}

fn serialize_memo_tail(
    section: &Section,
    memo_lists: &[(u32, Vec<Paragraph>)],
    records: &mut Vec<Record>,
) {
    if memo_lists.is_empty() {
        return;
    }

    // HWP5 spec: 메모 관련 정보는 마지막 구역 끝에 문단 리스트 형태로 저장된다.
    // 한컴 저장본은 마지막 본문 문단의 조판 속성을 복제한 빈 root 문단 아래에
    // MEMO_LIST, LIST_HEADER, 메모 본문 문단을 순서대로 둔다.
    let last_para = section.paragraphs.last();
    let mut root = Paragraph {
        char_count: 1,
        para_shape_id: last_para.map_or(0, |p| p.para_shape_id),
        style_id: last_para.map_or(0, |p| p.style_id),
        char_shapes: last_para
            .and_then(|p| p.char_shapes.first().cloned())
            .map(|mut cs| {
                cs.start_pos = 0;
                vec![cs]
            })
            .unwrap_or_else(|| {
                vec![CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                }]
            }),
        line_segs: last_para
            .map(|p| p.line_segs.clone())
            .filter(|segs| !segs.is_empty())
            .unwrap_or_else(|| Paragraph::new_empty().line_segs),
        raw_header_extra: vec![0; 12],
        ..Default::default()
    };
    for seg in &mut root.line_segs {
        seg.vertical_pos = seg
            .vertical_pos
            .saturating_add(seg.line_height)
            .saturating_add(seg.line_spacing);
    }
    root.has_para_text = false;
    serialize_paragraph_with_msb(&root, 0, true, records);

    for (memo_index, paragraphs) in memo_lists {
        records.push(Record {
            tag_id: tags::HWPTAG_MEMO_LIST,
            level: 1,
            size: 4,
            data: memo_index.to_le_bytes().to_vec(),
        });

        let mut list_header = Vec::with_capacity(16);
        list_header.extend_from_slice(&(paragraphs.len() as u32).to_le_bytes());
        list_header.extend_from_slice(&[0; 12]);
        records.push(Record {
            tag_id: tags::HWPTAG_LIST_HEADER,
            level: 1,
            size: list_header.len() as u32,
            data: list_header,
        });

        let mut memo_paragraphs = paragraphs.clone();
        for para in &mut memo_paragraphs {
            if para.raw_header_extra.len() < 12 {
                para.raw_header_extra = vec![0; 12];
            }
            // Hancom writes memo body paragraphs under MEMO_LIST without
            // PARA_LINE_SEG records. HWPX subList parsing may synthesize a
            // default line segment, but keeping it here breaks the HWP5 memo
            // container contract.
            para.line_segs.clear();
        }
        serialize_paragraph_list(&memo_paragraphs, 1, records);
    }
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
    // FIXED_WIDTH_SPACE (0x001F): HWPX에서 들어온 일부 문맥은 U+2007을
    // literal code point가 아니라 HWP5 fixed blank control로 저장해야 한다.
    if should_serialize_figure_space_as_hwp_fixed_blank(para) {
        mask |= 1u32 << 0x001F;
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
    // raw_header_extra[0..6] = numCharShapes(2) + numRangeTags(2) + numLineSegs(2) → 건너뜀
    // raw_header_extra[6..] = instanceId(4) + (옵션) 변경추적 UINT16 (2, 5.0.3.2 이상)
    if para.raw_header_extra.len() >= 10 {
        let extra = &para.raw_header_extra[6..];
        w.write_bytes(extra).unwrap();
    } else {
        // 새 문단 (HWPX 출처, raw_header_extra 없음): instanceId(4)만 기록.
        // 한컴 정답지 footnote-01.hwp 의 PARA_HEADER size=22 = 18 (heading) + 4 (instanceId).
        // 변경추적 UINT16 (size=24 형식) 은 한컴 정답지에 미사용.
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
    let mut field_ends: BTreeMap<usize, Vec<FieldEndMarker>> = BTreeMap::new();
    // trailing FIELD_END: control_idx → marker 매핑 (FIELD_BEGIN 직후에 삽입)
    let mut trailing_end_after_ctrl: HashMap<usize, Vec<FieldEndMarker>> = HashMap::new();
    // trailing FIELD_END 중 FIELD_BEGIN이 이미 본문에 배치된 경우 (orphan)
    let mut trailing_orphan_ends: Vec<u32> = Vec::new();

    for fr in &para.field_ranges {
        let marker = if let Some(control) = para.controls.get(fr.control_idx) {
            field_end_marker(control)
        } else {
            FieldEndMarker::default()
        };
        if fr.end_char_idx < text_len {
            field_ends.entry(fr.end_char_idx).or_default().push(marker);
        } else {
            // trailing FIELD_END: control_idx가 남은 컨트롤에 포함되는지 판별은
            // 메인 루프 후에 수행 (ctrl_idx 확정 후)
            trailing_end_after_ctrl
                .entry(fr.control_idx)
                .or_default()
                .push(marker);
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
        if let Some(markers) = field_ends.get(&i) {
            for &marker in markers {
                push_field_end_ctrl(&mut code_units, marker);
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
                    // tab_extended 없을 때: ext[6]=0x0009 마커 필수, 나머지 0
                    for cu in [0u16, 0, 0, 0, 0, 0, 0x0009] {
                        code_units.push(cu);
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
            '\u{2007}' => {
                if should_serialize_figure_space_as_hwp_fixed_blank(para) {
                    code_units.push(0x001F);
                } else {
                    code_units.push(0x2007);
                }
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
        if let Some(end_markers) = trailing_end_after_ctrl.remove(&ctrl_idx) {
            for marker in end_markers {
                push_field_end_ctrl(&mut code_units, marker);
            }
        }

        ctrl_idx += 1;
    }

    // orphan trailing FIELD_END: FIELD_BEGIN이 본문 갭에서 이미 배치된 경우
    // (trailing_end_after_ctrl에 남아있는 항목 = ctrl_idx가 이미 소진된 컨트롤)
    for end_markers in trailing_end_after_ctrl.values() {
        for &marker in end_markers {
            push_field_end_ctrl(&mut code_units, marker);
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

#[derive(Debug, Clone, Copy, Default)]
struct FieldEndMarker {
    ctrl_id: u32,
    memo_index: u32,
}

fn field_end_marker(ctrl: &Control) -> FieldEndMarker {
    match ctrl {
        Control::Field(field)
            if field.field_type == crate::model::control::FieldType::Memo
                || field.command.starts_with("MEMO/") =>
        {
            FieldEndMarker {
                ctrl_id: tags::FIELD_MEMO,
                memo_index: memo_field_index(field),
            }
        }
        Control::Field(field) => FieldEndMarker {
            ctrl_id: field.ctrl_id,
            memo_index: 0,
        },
        _ => FieldEndMarker::default(),
    }
}

fn memo_field_index(field: &crate::model::control::Field) -> u32 {
    if field.memo_index != 0 {
        return field.memo_index;
    }
    parse_memo_index_from_command(&field.command).unwrap_or(0)
}

fn parse_memo_index_from_command(command: &str) -> Option<u32> {
    command.split('/').nth(2)?.parse().ok()
}

fn push_field_end_ctrl(code_units: &mut Vec<u16>, marker: FieldEndMarker) {
    if marker.ctrl_id == tags::FIELD_MEMO {
        // Hancom writes MEMO field end with a distinct 8-code-unit marker:
        //   04 00 65 6d 25 00 01 ff ff 00 01 00 00 00 04 00
        // The sixth code unit is the memo index. Hard-coding `1` only
        // makes the first memo look correct and breaks later memo anchors.
        // The begin marker is still `%%me`; reusing that begin marker for
        // FIELD_END makes Hancom open the file but leaves memo visual styling
        // unapplied.
        code_units.extend_from_slice(&[
            0x0004,
            0x6d65,
            0x0025,
            0xff01,
            0x00ff,
            marker.memo_index as u16,
            0x0000,
            0x0004,
        ]);
    } else {
        push_extended_ctrl(code_units, 0x0004, marker.ctrl_id);
    }
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

fn should_serialize_figure_space_as_hwp_fixed_blank(para: &Paragraph) -> bool {
    const HWP5_AUTONUM_FWSPACE_TRAILING_TAG: u32 = 0x0100_0023;
    const HWP5_FIXED_WIDTH_SPACE_MASK: u32 = 1u32 << 0x001f;

    if para.control_mask & HWP5_FIXED_WIDTH_SPACE_MASK != 0 && para.text.contains('\u{2007}') {
        return true;
    }

    para.text.starts_with(" \u{2007}")
        && para
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, Control::AutoNumber(_)))
        && para
            .range_tags
            .iter()
            .any(|range_tag| range_tag.tag == HWP5_AUTONUM_FWSPACE_TRAILING_TAG)
}

/// 컨트롤에 대응하는 PARA_TEXT 내 제어 문자 코드와 ctrl_id를 반환
///
/// HWP 5.0 제어 문자 분류 (표 6):
///   0x0002: 구역/단 정의 (secd, cold)
///   0x000B: 표/그림/도형 (tbl, gso)
///   0x000F: 숨은 설명 (tcmt)
///   0x0010: 머리말/꼬리말 (head, foot)
///   0x0011: 각주/미주 (fn, en)
///   0x0012: 자동번호 (atno)
///   0x0015: 페이지 컨트롤/새 번호 (pgnp, pghi, nwno)
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
        // Hancom HWP5 oracle files store `nwno` in the 0x0015 page-control
        // family. Serializing it as 0x0012 makes Hancom 2020 treat the first
        // section paragraph as damaged/modified around the page control chain.
        Control::NewNumber(_) => (0x0015, tags::CTRL_NEW_NUMBER),
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
    use crate::model::control::{AutoNumber, Field, FieldType, NewNumber};
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

    #[test]
    fn test_plain_fixed_width_space_keeps_unicode_code_point() {
        let para = Paragraph {
            char_count: 2,
            text: "\u{2007}".to_string(),
            char_offsets: vec![0],
            ..Default::default()
        };

        let bytes = test_serialize_para_text(&para);

        assert_eq!(&bytes[0..2], &0x2007_u16.to_le_bytes());
    }

    #[test]
    fn test_autonum_range_tagged_fixed_width_space_serializes_as_hwp_control_code() {
        let para = Paragraph {
            char_count: 17,
            text: " \u{2007}(사회·문화)".to_string(),
            char_offsets: vec![0, 8, 9, 10, 11, 12, 13, 14, 15],
            controls: vec![Control::AutoNumber(AutoNumber::default())],
            range_tags: vec![RangeTag {
                start: 15,
                end: 16,
                tag: 0x0100_0023,
            }],
            ..Default::default()
        };

        let bytes = test_serialize_para_text(&para);

        assert_eq!(&bytes[16..18], &0x001F_u16.to_le_bytes());
    }

    #[test]
    fn test_control_mask_fixed_width_space_serializes_as_hwp_control_code() {
        let para = Paragraph {
            char_count: 10,
            control_mask: 1u32 << 0x001f,
            text: "사회탐구\u{2007}영역".to_string(),
            char_offsets: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            ..Default::default()
        };

        let bytes = test_serialize_para_text(&para);

        assert_eq!(&bytes[8..10], &0x001F_u16.to_le_bytes());
        assert_ne!(compute_control_mask(&para) & (1u32 << 0x001f), 0);
    }

    /// 컨트롤 문자 코드 매핑 테스트
    #[test]
    fn test_control_char_code() {
        assert_eq!(
            control_char_code_and_id(&Control::SectionDef(Box::default())).0,
            0x0002
        );
        assert_eq!(
            control_char_code_and_id(&Control::AutoNumber(AutoNumber::default())).0,
            0x0012
        );
        assert_eq!(
            control_char_code_and_id(&Control::NewNumber(NewNumber::default())).0,
            0x0015
        );
    }

    #[test]
    fn test_memo_field_end_uses_hancom_marker_tail() {
        let para = Paragraph {
            char_count: 2,
            text: "A".to_string(),
            char_offsets: vec![8],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            controls: vec![Control::Field(Field {
                field_type: FieldType::Memo,
                ctrl_id: tags::FIELD_MEMO,
                command: "MEMO/65535/2/1517431184/31247371/user/\\;;".to_string(),
                memo_index: 2,
                ..Default::default()
            })],
            field_ranges: vec![crate::model::paragraph::FieldRange {
                start_char_idx: 0,
                end_char_idx: 1,
                control_idx: 0,
            }],
            ..Default::default()
        };

        let bytes = test_serialize_para_text(&para);
        let expected_field_end = [
            0x04, 0x00, 0x65, 0x6d, 0x25, 0x00, 0x01, 0xff, 0xff, 0x00, 0x02, 0x00, 0x00, 0x00,
            0x04, 0x00,
        ];

        assert!(bytes
            .windows(expected_field_end.len())
            .any(|window| window == expected_field_end));
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
