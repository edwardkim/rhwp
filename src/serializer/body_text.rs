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
    // [Issue #1915] IR 계약 폴백: 첫 문단에 Control::SectionDef 가 없는 IR(HWP3 파서
    // 산출물, 외부 생성 IR)은 secd/PAGE_DEF 계열 레코드가 통째로 누락되어 재로드 시
    // 용지·여백이 0 이 된다 (hwpdocs 10k 서베이 41건, 전부 HWP3-origin).
    // hwpx_to_hwp 어댑터의 insert_section_def_control 보강과 동일 계약을 직렬화기
    // 진입에서 적용한다 — 첫 문단만 SectionDef 컨트롤을 삽입한 사본으로 직렬화.
    // 원본 스트림 경로(raw_stream)는 위에서 이미 반환되므로 영향 없음.
    // 실질 page_def(용지 크기 보유)가 있을 때만 보강한다 — 기본값(0×0) section_def
    // 를 가진 합성/부분 IR(유닛테스트 fixture 등)에 무의미한 secd 를 주입해 레코드
    // 시퀀스를 바꾸지 않기 위함.
    let has_real_page_def =
        section.section_def.page_def.width > 0 && section.section_def.page_def.height > 0;
    let first_para_with_secd = section.paragraphs.first().and_then(|p| {
        if !has_real_page_def
            || p.controls
                .iter()
                .any(|c| matches!(c, Control::SectionDef(_)))
        {
            None
        } else {
            let mut clone = p.clone();
            clone.controls.insert(
                0,
                Control::SectionDef(Box::new(section.section_def.clone())),
            );
            // [#4680] 정의 제어문자가 글자보다 앞에 놓이도록 자리를 비운다 —
            // 간격이 없으면 `serialize_para_text` 가 텍스트 뒤에 몰아 쓰고, 그런 문서는
            // 한글이 열다 멎는다. 어댑터와 같은 문단 좌표 계약을 적용한다.
            let leading_defs = clone
                .controls
                .iter()
                .take_while(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)))
                .count();
            clone.reserve_leading_extended_control_slots(leading_defs);
            Some(clone)
        }
    });
    for (i, para) in section.paragraphs.iter().enumerate() {
        let is_last = i == para_count - 1 && !has_memo_tail;
        let para_ref = if i == 0 {
            first_para_with_secd.as_ref().unwrap_or(para)
        } else {
            para
        };
        serialize_paragraph_with_msb(para_ref, 0, is_last, &mut records);
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
    //
    // [#4402] `serialize_para_text` 는 미기입 누름틀의 안내문 잔재(`Field.guide_residue`)를
    // 함께 되살리고, 그로 인해 벌어진 위치들을 `residue_shifts` 로 돌려준다 — 아래
    // PARA_CHAR_SHAPE 가 그 시프트를 반영해야 텍스트 버퍼와 서식 경계가 어긋나지 않는다.
    // 표시만 있는 문단도 PARA_TEXT 가 있어야 8유닛이 파일에 남는다.
    let has_content =
        !para.text.is_empty() || !para.controls.is_empty() || !para.title_marks.is_empty();
    let (text_data, residue_shifts): (Option<Vec<u8>>, Vec<GuideResidueShift>) =
        if has_content || (para.has_para_text && para.char_count > 1) {
            let result = serialize_para_text(para);
            (Some(result.bytes), result.residue_shifts)
        } else {
            (None, Vec::new())
        };

    // char_count 재계산: PARA_TEXT가 있으면 code unit 수.
    //
    // [#4677] PARA_TEXT 를 내보내지 않는 문단은 **파일에 글자가 0 개**다. 모델 값을 그대로
    // 쓰면 헤더만 N 을 주장하는 문단이 생긴다 — HWPX 파서는 다단락 필드의 고아
    // `<hp:fieldEnd>`(8 유닛)를 `char_count` 에 세지만 HWP5 저장기에는 그 자리를 쓸 방법이
    // 없어서, 텍스트 없는 문단이 `char_count=9` 로 나간다. 한글 2022 는 그 문단을 만나면
    // 본문 전체를 버리고 빈 1쪽 문서로 연다(rhwp 재파싱은 통과 — `--verify` 로는 안 잡힌다).
    // 빈 문단의 규정 값은 끝 마커 1 이다.
    let actual_char_count = if let Some(ref td) = text_data {
        (td.len() / 2) as u32
    } else {
        para.char_count.min(1)
    };

    // [#4677] 본문에 대응하지 않는 lineseg 는 파일에 내보내지 않는다 — 조판 전용 보강 줄과
    // PARA_TEXT 밖을 가리키는 줄 두 갈래다(판정은 `line_segs_within_text` 주석 참조).
    // 한글 2022 는 그런 문단을 만나면 본문 전체를 버리고 빈 1쪽 문서로 연다 — rhwp 재파싱만
    // 통과하는 함정이라 `--verify` 로는 잡히지 않는다 (10k 전수 스윕 x2h 소실군).
    let line_segs_in_range = line_segs_within_text(
        &para.line_segs,
        actual_char_count,
        para.layout_only_fill_lines,
    );

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
            line_segs_in_range.len(),
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
        let shifted_char_shapes;
        let char_shapes_for_record: &[CharShapeRef] = if residue_shifts.is_empty() {
            effective_char_shapes
        } else {
            shifted_char_shapes =
                shift_char_shapes_for_residues(effective_char_shapes, &residue_shifts);
            &shifted_char_shapes
        };
        let data = serialize_para_char_shape(char_shapes_for_record);
        records.push(Record {
            tag_id: tags::HWPTAG_PARA_CHAR_SHAPE,
            level: base_level + 1,
            size: data.len() as u32,
            data,
        });
    }

    // PARA_LINE_SEG
    if !line_segs_in_range.is_empty() {
        let data = serialize_para_line_seg(line_segs_in_range);
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
        // [#4424] CTRL_HEADER 를 만들지 않는 컨트롤은 PARA_TEXT 에도 문자를 내지
        // 않으므로 control_mask 비트도 세우지 않는다 — 셋이 항상 함께 움직여야 한다.
        if !emits_ctrl_header(ctrl) {
            continue;
        }
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
    // 묶음 빈칸 (0x001E, NBSP): serialize_para_text 가 U+00A0 마다 코드 0x1E 를 방출하므로
    // (#1793) control_mask 비트 30 도 세워 PARA_HEADER 를 PARA_TEXT 와 일치시킨다.
    if para.text.contains('\u{00A0}') {
        mask |= 1u32 << 0x001E;
    }
    // 하이픈 (0x0018): serialize_para_text 가 U+00AD 마다 코드 0x18 을 방출하므로
    // control_mask 비트 24 도 세워 PARA_HEADER 를 PARA_TEXT 와 일치시킨다.
    if para.text.contains('\u{00AD}') {
        mask |= 1u32 << 0x0018;
    }
    // 제목 차례 표시 (0x0008): serialize_para_text 가 title_marks 마다 코드 0x08 을
    // 방출하므로 control_mask 비트 8 도 세워 PARA_HEADER 를 PARA_TEXT 와 일치시킨다.
    // 한컴 원본 실측(07589·08288·06858): 표시가 든 문단 286/286 이 비트 8 을 세우고,
    // 나머지 9,285 문단은 하나도 세우지 않아 이 비트는 표시와 정확히 일대일이다.
    if !para.title_marks.is_empty() {
        mask |= 1u32 << 0x0008;
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
    num_line_segs: usize,
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
    w.write_u16(num_line_segs as u16).unwrap();

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
    serialize_para_text(para).bytes
}

/// [#4402] `serialize_para_text` 가 되살린 안내문 잔재 1건의 위치 정보.
///
/// `position` 은 삽입 **이전** 좌표계 — 즉 적재 시 삭제 수술이 남긴, 지금 `para.char_shapes`/
/// `para.char_offsets` 가 쓰는 collapsed 좌표 — 기준 UTF-16 code unit 위치다.
/// `PARA_CHAR_SHAPE` 를 되살린 텍스트 폭만큼 뒤로 미는 데 쓴다
/// (`shift_char_shapes_for_residues`).
struct GuideResidueShift {
    position: u32,
    width: u32,
    char_shape_id: u32,
}

struct ParaTextResult {
    bytes: Vec<u8>,
    /// 문단 안에서 되살린 안내문 잔재들 — 위치 오름차순.
    residue_shifts: Vec<GuideResidueShift>,
}

/// [#4402] HWP5 저장에서도 미기입 누름틀 안내문을 되살릴지 판정한다.
///
/// `#3545` 가 HWPX 저장(`emit_guide_residue`, `src/serializer/hwpx/section.rs`)에 붙인 것과
/// 동일한 게이트 — 값이 채워진 필드(start != end)와 사용자가 비운 필드(dirty bit 15)는
/// 건너뛴다. 중복 주입·사용자가 비운 값의 부활을 막는다.
fn guide_residue_for<'a>(
    para: &'a Paragraph,
    fr: &crate::model::paragraph::FieldRange,
) -> Option<&'a crate::model::control::GuideResidue> {
    if fr.start_char_idx != fr.end_char_idx {
        return None;
    }
    let Some(Control::Field(f)) = para.controls.get(fr.control_idx) else {
        return None;
    };
    if f.is_dirty() {
        return None;
    }
    let residue = f.guide_residue.as_ref()?;
    if residue.text.is_empty() {
        return None;
    }
    Some(residue)
}

/// 안내문 텍스트를 UTF-16 code unit 으로 인코딩해 `code_units` 에 밀어 넣고, 되돌린 폭과
/// 위치를 `residue_shifts` 에 기록한다 (`shift_char_shapes_for_residues` 가 소비한다).
///
/// `position` 은 삽입 시점 기준 아직 시프트를 반영하지 않은 현재 좌표.
fn push_guide_residue(
    code_units: &mut Vec<u16>,
    residue_shifts: &mut Vec<GuideResidueShift>,
    residue: &crate::model::control::GuideResidue,
    position: u32,
) {
    let start_len = code_units.len();
    for c in residue.text.chars() {
        let mut buf = [0u16; 2];
        for cu in c.encode_utf16(&mut buf) {
            code_units.push(*cu);
        }
    }
    let width = (code_units.len() - start_len) as u32;
    if width == 0 {
        return;
    }
    residue_shifts.push(GuideResidueShift {
        position,
        width,
        char_shape_id: residue.char_shape_id,
    });
}

fn serialize_para_text(para: &Paragraph) -> ParaTextResult {
    let mut code_units: Vec<u16> = Vec::new();
    let text_chars: Vec<char> = para.text.chars().collect();
    let mut ctrl_idx = 0;
    let mut prev_end: u32 = 0;
    let mut tab_idx: usize = 0; // TAB 확장 데이터 인덱스
    let mut residue_shifts: Vec<GuideResidueShift> = Vec::new();

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
    // [#4402] empty_field_ends/trailing_end_after_ctrl 과 같은 키로 안내문 잔재를 매핑 —
    // 자기 FIELD_END 직전에 되살린다. mismatch(orphan) 경로는 #3545 와 동일하게 제외한다
    // (슬롯 위치 추정이 이미 무너진 퇴화 경로라 주입이 개선이라 단정할 수 없다).
    let mut empty_field_residues: BTreeMap<usize, Vec<&crate::model::control::GuideResidue>> =
        BTreeMap::new();
    let mut trailing_residues: HashMap<usize, Vec<&crate::model::control::GuideResidue>> =
        HashMap::new();

    // [#4.6] 위치별 방출 순서를 명시하기 위한 두 갈래.
    //
    // 파서는 FIELD_BEGIN/END 를 **스택(LIFO)** 으로 짝짓는다(`parser/body_text.rs`). 그래서 한
    // 필드가 끝나는 자리에서 다음 필드가 시작하면 END 가 BEGIN 보다 먼저 나가야 하고, 빈
    // 필드(시작==끝)는 자기 BEGIN 뒤에 END 가 붙어야 한다. 두 경우를 한 통에 담으면 순서를
    // 가릴 수 없다.
    //
    // - `field_ends`       : 시작 < 끝 — 그 자리의 **모든 것보다 먼저** 나간다.
    // - `empty_field_ends` : 시작 == 끝 — 자기 BEGIN **직후에** 나간다.
    let mut empty_field_ends: BTreeMap<usize, Vec<FieldEndMarker>> = BTreeMap::new();
    // 컨트롤 → 그 컨트롤이 여는 필드의 시작 문자 위치. FIELD_BEGIN 은 이 위치보다 앞에
    // 나올 수 없다 — 갭 크기만 보고 밀어 넣으면 뒤 필드의 BEGIN 이 앞 갭으로 빨려 들어가
    // 위치 0 에 두 개가 겹쳐 방출된다.
    let mut field_begin_pos: HashMap<usize, usize> = HashMap::new();
    for fr in &para.field_ranges {
        field_begin_pos
            .entry(fr.control_idx)
            .and_modify(|p| *p = (*p).min(fr.start_char_idx))
            .or_insert(fr.start_char_idx);
    }

    for fr in &para.field_ranges {
        let marker = if let Some(control) = para.controls.get(fr.control_idx) {
            field_end_marker(control)
        } else {
            FieldEndMarker::default()
        };
        let residue = guide_residue_for(para, fr);
        if fr.end_char_idx < text_len && fr.start_char_idx == fr.end_char_idx {
            empty_field_ends
                .entry(fr.end_char_idx)
                .or_default()
                .push(marker);
            if let Some(residue) = residue {
                empty_field_residues
                    .entry(fr.end_char_idx)
                    .or_default()
                    .push(residue);
            }
        } else if fr.end_char_idx < text_len {
            field_ends.entry(fr.end_char_idx).or_default().push(marker);
        } else {
            // trailing FIELD_END: control_idx가 남은 컨트롤에 포함되는지 판별은
            // 메인 루프 후에 수행 (ctrl_idx 확정 후)
            trailing_end_after_ctrl
                .entry(fr.control_idx)
                .or_default()
                .push(marker);
            if let Some(residue) = residue {
                trailing_residues
                    .entry(fr.control_idx)
                    .or_default()
                    .push(residue);
            }
        }
    }

    for (i, ch) in text_chars.iter().enumerate() {
        let offset = if i < para.char_offsets.len() {
            para.char_offsets[i]
        } else {
            prev_end
        };

        // 다단락 필드의 종료 마커 — 짝 fieldBegin 이 앞 문단에 있어 `field_ranges` 가
        // 아니라 `orphan_field_ends` 로 온다. 내지 않으면 8유닛 슬롯이 사라져 이 문단의
        // lineseg 가 범위 밖이 되고 조판이 통째로 버려진다(01752 실측: 쪽수 1→2).
        // `begin_ctrl_id` 가 0 이면 필드 종류를 모른다는 뜻이라 내지 않는다 — 종류를
        // 지어내면 한글이 짝을 못 맞춘다.
        for ofe in para
            .orphan_field_ends
            .iter()
            .filter(|o| o.char_idx == i && o.begin_ctrl_id != 0)
        {
            push_extended_ctrl(&mut code_units, 0x0004, ofe.begin_ctrl_id);
            prev_end += 8;
        }

        // 제목 차례 표시 — CTRL_HEADER 없는 인라인 컨트롤이라 여기서 직접 낸다.
        // 이 자리의 다른 컨트롤보다 먼저 놓는다: 8유닛만 채우면 되므로 순서는 축에
        // 영향을 주지 않고, 실측 대다수(2,237 중 1,879)가 문단 선두다.
        for m in para.title_marks.iter().filter(|m| m.char_idx == i) {
            push_extended_ctrl(
                &mut code_units,
                0x0008,
                if m.ignore {
                    tags::CTRL_TITLE_MARK_IGNORE_ON
                } else {
                    tags::CTRL_TITLE_MARK_IGNORE_OFF
                },
            );
            prev_end += 8;
        }

        // [Task #1050] AutoNumber placeholder 검출:
        // char_offsets[i] == prev_end 이고 ch == ' ' 이고 다음 char_offset 이 prev_end + 8 +
        // (실제 char 폭)인 경우 = placeholder space (i char 한 자리 차지 + 다음 char 가 8 점프 후).
        // 이 경우 ' ' 대신 AUTO_NUMBER 컨트롤 8 cu 작성 + prev_end = offset + 8.
        let next_offset = if i + 1 < para.char_offsets.len() {
            Some(para.char_offsets[i + 1])
        } else {
            None
        };
        // [#2740] placeholder 가 문단의 **마지막 문자**면 next_offset 이 없어 위 판정이
        // 항상 실패했다. 그러면 공백을 리터럴로 쓰고 남은 컨트롤을 뒤에 다시 방출하므로,
        // 재파싱 때 placeholder 가 하나 더 생겨 저장할 때마다 공백이 1개씩 무한히 늘었다
        // (수렴하지 않음 — 쪽번호 자동번호가 든 머리말/꼬리말이 대표 사례).
        //
        // 마지막 문자를 placeholder 로 봐도 안전한 근거: 파서(parser/body_text.rs:334)는
        // 0x0012 를 만나면 **항상** text 에 공백 placeholder 를 push 한다. 따라서 남은
        // 컨트롤이 자동번호인데 공백이 마지막이면 그 공백이 곧 placeholder 다 — 진짜
        // 공백이었다면 그 뒤에 placeholder 가 하나 더 붙어 마지막이 아니게 된다.
        let is_last_text_char = i + 1 == text_chars.len();
        let is_autonum_placeholder = *ch == ' '
            && offset == prev_end
            && ctrl_idx < para.controls.len()
            // [#3495] 0x0012(자동번호)만 placeholder 공백을 만든다. 두 파서 모두
            // 그렇다 — parser/body_text.rs 는 ch == 0x0012 일 때만, HWPX section.rs 는
            // 0x0012 파트일 때만 text 에 공백을 push 한다. 각주·미주(0x0011)는
            // placeholder 를 만들지 않으므로, 여기 포함하면 미주 앞의 진짜 공백을
            // placeholder 로 오인해 컨트롤로 덮어쓴다 (SO-SUEOP.hwp 문단 238:
            // 공백 12개 -> 11개, 뒤 텍스트가 한 칸 당겨짐).
            && matches!(control_char_code_and_id(&para.controls[ctrl_idx]).0, 0x0012)
            && next_offset.map_or(is_last_text_char, |n| n >= offset + 8);
        if is_autonum_placeholder {
            let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
            push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);
            ctrl_idx += 1;
            prev_end = offset + 8;
            continue;
        }

        // 갭에 컨트롤 문자 배치 (각 컨트롤 = 8 code unit)
        // [#1795] 이 인덱스에 삽입될 FIELD_END(각 8 cu)의 공간을 먼저 예약한다.
        // 예약 없이 갭을 컨트롤로 채우면 FIELD_END 전용 갭(8 cu)을 다음 컨트롤이
        // 선점하여 이후 모든 char_offsets 가 시프트되고, 재파싱 시 lineseg
        // text_start 매핑이 어긋나 줄바꿈 위치가 이동한다 (seoul_0043 글상자).
        // ① 여기서 **끝나는** 필드의 FIELD_END — 이 자리의 무엇보다 먼저 닫는다.
        //    (그래야 같은 자리에서 시작하는 다음 필드의 BEGIN 과 뒤엉키지 않는다)
        if let Some(markers) = field_ends.get(&i) {
            for &marker in markers {
                push_field_end_ctrl(&mut code_units, marker);
                prev_end += 8;
            }
        }

        // ② 갭 채우기 — 빈 필드의 END 자리는 예약해 둔다.
        let pending_field_end_cus = empty_field_ends
            .get(&i)
            .map(|markers| markers.len() as u32 * 8)
            .unwrap_or(0);
        while prev_end + 8 + pending_field_end_cus <= offset
            && ctrl_idx < para.controls.len()
            // 필드를 여는 컨트롤은 자기 시작 위치 전에 방출하지 않는다.
            && field_begin_pos
                .get(&ctrl_idx)
                .is_none_or(|&start| start <= i)
        {
            if emits_ctrl_header(&para.controls[ctrl_idx]) {
                let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
                push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);
                prev_end += 8;
            }
            ctrl_idx += 1;
        }

        // ③ 이 자리에서 시작하는 필드의 FIELD_BEGIN 은 **갭 예산과 무관하게 강제 방출**한다.
        //    갭은 텍스트 편집 뒤 실제 구조보다 크거나 작게 남을 수 있어서, 예산만 믿으면
        //    필드가 통째로 사라진다(막기만 했을 때 실제로 165→164 로 줄었다).
        while ctrl_idx < para.controls.len()
            && field_begin_pos
                .get(&ctrl_idx)
                .is_some_and(|&start| start == i)
        {
            if emits_ctrl_header(&para.controls[ctrl_idx]) {
                let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
                push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);
                prev_end += 8;
            }
            ctrl_idx += 1;
        }

        // ④ 빈 필드(시작==끝)의 FIELD_END — 자기 BEGIN 직후.
        // [#4402] 안내문 잔재는 자기 FIELD_END 바로 앞에 되살린다 (HWPX
        // `emit_field_end_at` 과 동일 순서 — BEGIN 뒤, END 앞).
        if let Some(residues) = empty_field_residues.get(&i) {
            for &residue in residues {
                push_guide_residue(&mut code_units, &mut residue_shifts, residue, prev_end);
            }
        }
        if let Some(markers) = empty_field_ends.get(&i) {
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
                // 묶음 빈칸 (HWP 5.0 표 7: 코드 30). 코드 24(0x18)는 하이픈이므로
                // 여기 쓰면 안 된다 (#1793).
                code_units.push(0x001E);
                prev_end = offset + 1;
            }
            '\u{00AD}' => {
                // 소프트 하이픈 (HWP 5.0 표 7: 코드 24). 파서가 0x18 을 U+00AD 로
                // 받으므로 저장도 제어문자로 되돌린다 — 리터럴로 쓰면 다음 왕복에서
                // 실제 하이픈이 되어 단어가 갈라진다.
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
    //
    // [#4402] 이 구간은 본디 위치 예산(offset/prev_end 갭 채우기)을 추적하지 않지만,
    // `char_shapes` 시프트 계산은 절대 위치가 필요하다 — `prev_end` 를 그대로 이어 써서
    // (메인 루프가 끝난 시점 값에서 시작) 8 code unit 씩 전진시킨다.
    // 마지막 문자 뒤(또는 텍스트가 없는 문단)의 고아 종료 마커.
    for ofe in para
        .orphan_field_ends
        .iter()
        .filter(|o| o.char_idx >= text_chars.len() && o.begin_ctrl_id != 0)
    {
        push_extended_ctrl(&mut code_units, 0x0004, ofe.begin_ctrl_id);
        prev_end += 8;
    }

    // 마지막 문자 뒤(또는 텍스트가 없는 문단)의 제목 차례 표시.
    for m in para
        .title_marks
        .iter()
        .filter(|m| m.char_idx >= text_chars.len())
    {
        push_extended_ctrl(
            &mut code_units,
            0x0008,
            if m.ignore {
                tags::CTRL_TITLE_MARK_IGNORE_ON
            } else {
                tags::CTRL_TITLE_MARK_IGNORE_OFF
            },
        );
        prev_end += 8;
    }

    while ctrl_idx < para.controls.len() {
        if emits_ctrl_header(&para.controls[ctrl_idx]) {
            let (ctrl_code, ctrl_id) = control_char_code_and_id(&para.controls[ctrl_idx]);
            push_extended_ctrl(&mut code_units, ctrl_code, ctrl_id);
            prev_end += 8;
        }

        // 이 컨트롤(FIELD_BEGIN)에 대응하는 안내문 잔재 — 자기 FIELD_END 바로 앞.
        if let Some(residues) = trailing_residues.get(&ctrl_idx) {
            for &residue in residues {
                push_guide_residue(&mut code_units, &mut residue_shifts, residue, prev_end);
            }
        }

        // 이 컨트롤(FIELD_BEGIN)에 대응하는 trailing FIELD_END 삽입
        if let Some(end_markers) = trailing_end_after_ctrl.remove(&ctrl_idx) {
            for marker in end_markers {
                push_field_end_ctrl(&mut code_units, marker);
                prev_end += 8;
            }
        }

        ctrl_idx += 1;
    }

    // orphan trailing FIELD_END: FIELD_BEGIN이 본문 갭에서 이미 배치된 경우
    // (trailing_end_after_ctrl에 남아있는 항목 = ctrl_idx가 이미 소진된 컨트롤)
    //
    // [#4402] 이 경로의 안내문 잔재는 의도적으로 되살리지 않는다 — 슬롯 위치 추정이
    // 이미 무너진 mismatch/퇴화 경로라 주입이 개선이라 단정할 수 없다는 #3545 의
    // HWPX 축 판단(`emit_guide_residue` 는 이 경로를 다루지 않는다)을 그대로 따른다.
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
    residue_shifts.sort_by_key(|s| s.position);
    ParaTextResult {
        bytes,
        residue_shifts,
    }
}

/// [#4402] `PARA_CHAR_SHAPE` 에 되살린 안내문 잔재 폭을 반영해 `start_pos` 를 민다.
///
/// HWP5 의 char_shape 는 (HWPX 의 런별 `charPrIDRef` 와 달리) 문단 텍스트 버퍼 안 절대
/// UTF-16 위치를 가리키는 표다. `serialize_para_text` 가 삽입한 잔재 폭만큼 그 뒤 항목을
/// 밀어주지 않으면 이후 모든 서식 경계가 어긋난다.
///
/// 적재 시 삭제 수술(`document_core::commands::document`)은 (a) 삭제 범위 **안**의 경계를
/// 삭제 시작점(`u_start`)으로 접고 (b) 삭제 범위 **뒤**의 경계는 `u_start` 로 감산해 옮긴다 —
/// 둘 다 결과적으로 `u_start` 에 위치가 겹칠 수 있다(zero-width). `residue.char_shape_id` 는
/// 삭제 **전** 좌표에서 `u_start` 이하 마지막 항목(= 잔재 자신의 서식)을 가리키므로, 같은
/// 위치에 묶인 항목들 중 그 id 를 가진 것까지는 그대로 두고(잔재가 시작하는 지점) 그 뒤부터
/// 미는 것이 되돌리기의 역연산이다. 그 id 를 못 찾으면(잔재 자신의 항목이 `u_start` 보다
/// 앞에 있던 축퇴 경로) 묶음 전체가 삭제 범위 뒤에서 온 것으로 보고 첫 항목부터 민다.
fn shift_char_shapes_for_residues(
    char_shapes: &[CharShapeRef],
    residue_shifts: &[GuideResidueShift],
) -> Vec<CharShapeRef> {
    if residue_shifts.is_empty() {
        return char_shapes.to_vec();
    }
    let mut result = Vec::with_capacity(char_shapes.len());
    let mut shift: u32 = 0;
    let mut next = 0usize;
    let mut i = 0usize;
    while i < char_shapes.len() {
        while next < residue_shifts.len()
            && residue_shifts[next].position < char_shapes[i].start_pos
        {
            shift += residue_shifts[next].width;
            next += 1;
        }
        if next < residue_shifts.len() && residue_shifts[next].position == char_shapes[i].start_pos
        {
            let pos = char_shapes[i].start_pos;
            let mut j = i;
            while j < char_shapes.len() && char_shapes[j].start_pos == pos {
                j += 1;
            }
            let residue = &residue_shifts[next];
            let anchor = (i..j).find(|&k| char_shapes[k].char_shape_id == residue.char_shape_id);
            let shift_from = anchor.map(|a| a + 1).unwrap_or(i);
            for (k, cs) in char_shapes.iter().enumerate().take(j).skip(i) {
                let extra = if k < shift_from { 0 } else { residue.width };
                result.push(CharShapeRef {
                    start_pos: pos + shift + extra,
                    char_shape_id: cs.char_shape_id,
                });
            }
            shift += residue.width;
            next += 1;
            i = j;
            continue;
        }
        result.push(CharShapeRef {
            start_pos: char_shapes[i].start_pos + shift,
            char_shape_id: char_shapes[i].char_shape_id,
        });
        i += 1;
    }
    result
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

/// [#4677] 파일에 실을 수 있는 lineseg 만 남긴 슬라이스.
///
/// 두 가지를 잘라 낸다. 둘 다 한글 2022 가 본문을 통째로 버리게 만드는 값이다 — rhwp 는
/// 자기가 쓴 파일을 그대로 다시 읽으므로 `--verify` 로는 잡히지 않는다.
///
/// 1. **조판 전용 보강 줄**(`Paragraph::layout_only_fill_lines`) — 셀 저장 높이를 채우려고
///    끝에 덧붙인, 본문에 없는 줄. 한컴 정답지는 그 셀 문단에 줄을 하나만 쓴다.
/// 2. **PARA_TEXT 밖을 가리키는 줄** — HWP5 가 규정하지 않는 컨트롤(색인 표시 `idxm` 등)을
///    저장에서 떨구면 문단이 8 유닛씩 짧아지는데, 원본 HWPX 의 lineseg 는 그 컨트롤을 센
///    `textpos` 를 그대로 들고 있다.
///
/// 경계값 `text_start == char_count` 는 **정상**이다 — 한컴 자신이 빈 문단(`char_count=1`)에
/// 끝 마커를 가리키는 둘째 세그먼트(`text_start=1`, EMPTY_SEGMENT)를 쓴다(`hwpctl_API_v2.4.hwp`
/// 14곳). 그래서 판정은 `>` 다. 실제로 문제가 된 값은 훨씬 멀리 나간다(문단 길이 5 에
/// `text_start=10`, 37 에 40).
///
/// 첫 줄(`text_start == 0`)은 어떤 문단에도 있어야 하므로 전부 범위를 벗어나면 그대로 둔다
/// (문단 자체가 비정상이라는 뜻이며, 줄을 0 개로 만들면 다른 손상이 된다). 유효한 줄이
/// 하나라도 있으면 **접두부만** 남긴다 — 줄은 순서대로 이어져야 한다.
///
/// 경계는 `text_start == char_count` 를 **포함하지 않는다**. 한글이 직접 쓴 문서에 그 값이
/// 흔하기 때문이다 — 빈 문단(`char_count=1`, 줄 `[0, 1]`)과 개체만 있는 셀 문단
/// (`char_count=9`, 줄 `[0, 9]`)이 그렇고, 저장소 샘플 5건에서 40개 문단이 이 형태다.
/// 한글은 그 문서를 정상 개방하므로 끝 위치를 가리키는 줄은 버릴 값이 아니다. 오라클로
/// 본문 폐기를 확정한 값은 모두 끝을 **넘어선다**(`char_count=5` 에 `10`, `37` 에 `40`).
fn line_segs_within_text(
    line_segs: &[LineSeg],
    char_count: u32,
    layout_only_fill_lines: usize,
) -> &[LineSeg] {
    let real = line_segs.len().saturating_sub(layout_only_fill_lines);
    let in_range = line_segs[..real]
        .iter()
        .position(|seg| seg.text_start > char_count)
        .unwrap_or(real);
    if in_range == 0 {
        line_segs
    } else {
        &line_segs[..in_range]
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
/// 이 컨트롤이 CTRL_HEADER 레코드를 만드는가.
///
/// [#4424] `serialize_control` 의 catch-all arm(`Hyperlink | Ruby | Unknown`)은
/// `ctrl_id == 0` 이면 CTRL_HEADER 를 아예 만들지 않는다. 그런데 PARA_TEXT 쪽은
/// 그것과 무관하게 확장 컨트롤 문자(0x000B)를 방출해 왔다.
///
/// HWP5 파서는 확장 컨트롤 문자를 셀 때마다 `controls[]` 인덱스를 하나 올리므로
/// (`parser/body_text.rs` 의 `is_extended_only_ctrl_char` 가 11 을 포함한다),
/// 짝 없는 문자 하나가 **뒤따르는 필드의 `field_ranges[].control_idx` 를 한 칸
/// 밀어 존재하지 않는 인덱스를 가리키게 만든다.**
///
/// 그래서 텍스트 쪽 방출 조건을 레코드 쪽과 정확히 같게 맞춘다. 문자를 내지 않으면
/// 컨트롤 자체는 잃지만(하이퍼링크는 HWP5 에 규정된 슬롯이 없다 — HWP5 파서는
/// `Control::Hyperlink` 를 만들지 않고 생성 지점이 HWP3 하나뿐이다), 짝짓기는
/// 깨지지 않는다. 슬롯을 발명하는 것보다 낫다.
///
/// `Control::Ruby` 는 같은 arm 이지만 규정된 ctrl_id(`tdut`)가 있어 방출을 막는 것이
/// 답이 아니다 — #4397 에서 따로 다룬다. 여기서는 건드리지 않는다.
fn emits_ctrl_header(ctrl: &Control) -> bool {
    // [#4677] 판정은 `Control::occupies_ctrl_char_slot` 하나에서만 나온다 — 합성 lineseg 의
    // `text_start` 를 계산하는 renderer 쪽이 같은 규칙을 봐야 오프셋이 어긋나지 않는다.
    ctrl.occupies_ctrl_char_slot()
}

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
        Control::PageNumCtrl(_) => (0x0015, tags::CTRL_PAGE_NUM_CTRL),
        Control::PageHide(_) => (0x0015, tags::CTRL_PAGE_HIDE),
        Control::Bookmark(_) => (0x0016, tags::CTRL_BOOKMARK),
        Control::IndexMark(_) => (0x0016, tags::CTRL_INDEX_MARK),
        Control::Hyperlink(_) => (0x000B, 0),
        // [#4677] 덧말은 한컴 원본에서 `17 00 74 75 64 74 …`(코드 0x0017 + 'tdut')로 나간다.
        // 종전엔 `(0x000B, 0)` — 개체 제어문자 자리에 **id 0** 을 써 놓고 짝이 되는
        // CTRL_HEADER 는 내지 않았다. 한글 2022 는 짝 없는 개체 제어문자를 만나면 본문을
        // 통째로 버린다(0자·1쪽). `CTRL_CHAR_OVERLAP` 상수는 이름과 달리 'tdut'(덧말)이다.
        Control::Ruby(_) => (0x0017, tags::CTRL_CHAR_OVERLAP),
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
    use crate::model::control::{
        AutoNumber, Bookmark, Field, FieldType, Hyperlink, IndexMark, NewNumber, PageNumCtrl,
        PageStartsOn,
    };
    use crate::model::document::{Section, SectionDef};
    use crate::model::paragraph::{
        CharShapeRef, FieldRange, LineSeg, Paragraph, RangeTag, TitleMark,
    };
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

    /// 쪽 번호 시작 쪽 라운드트립 — 세 값 모두 자기 자리로 돌아와야 한다.
    ///
    /// 열거 대응은 한글 2022 양방향 실측이다(06731 을 HWPX 로 저장 → 속성만 바꿔
    /// 다시 HWP5 로 저장, 각 17/17): 0=BOTH, 1=EVEN, 2=ODD.
    #[test]
    fn test_roundtrip_page_num_ctrl() {
        for want in [PageStartsOn::Both, PageStartsOn::Even, PageStartsOn::Odd] {
            let para = Paragraph {
                char_count: 9,
                controls: vec![Control::PageNumCtrl(PageNumCtrl {
                    page_starts_on: want,
                })],
                ..Default::default()
            };
            let section = Section {
                paragraphs: vec![para],
                raw_stream: None,
                ..Default::default()
            };
            let parsed = parse_body_text_section(&serialize_section(&section)).unwrap();
            match &parsed.paragraphs[0].controls[0] {
                Control::PageNumCtrl(pnc) => assert_eq!(pnc.page_starts_on, want),
                other => panic!("PageNumCtrl 이 아니다: {other:?}"),
            }
        }
    }

    /// 규정 밖 값은 기본값(BOTH)으로 떨어뜨린다 — 열거 밖 값을 그대로 쓰면 한글이
    /// 쓰레기값으로 읽는다(#4756 FieldType 계열과 같은 계약).
    #[test]
    fn page_starts_on_rejects_values_outside_the_enum() {
        assert_eq!(PageStartsOn::from_hwp5(3), PageStartsOn::Both);
        assert_eq!(PageStartsOn::from_hwp5(u32::MAX), PageStartsOn::Both);
        assert_eq!(PageStartsOn::from_hwpx("SOMETHING"), PageStartsOn::Both);
        for v in [PageStartsOn::Both, PageStartsOn::Even, PageStartsOn::Odd] {
            assert_eq!(PageStartsOn::from_hwp5(v.to_hwp5()), v);
            assert_eq!(PageStartsOn::from_hwpx(v.as_hwpx()), v);
        }
    }

    /// 찾아보기 표식 라운드트립 — 키와 8유닛 자리가 함께 살아야 한다.
    ///
    /// arm 이 없으면 `Control::Unknown` 이 되고, HWPX 저장기가 슬롯으로는 세어 놓고
    /// XML 은 내지 않아 문단 축이 8유닛 짧아진다. 한글은 범위를 넘는 `textpos` 를
    /// 만나면 파일을 아예 열지 못한다(06926·07833·08051 실측).
    #[test]
    fn test_roundtrip_index_mark() {
        let para = Paragraph {
            // 글자 2 + 표식 8 + 끝 마커 1
            char_count: 11,
            text: "가나".to_string(),
            char_offsets: vec![0, 1],
            controls: vec![Control::IndexMark(IndexMark {
                first_key: "위로보상금과".to_string(),
                second_key: String::new(),
            })],
            ..Default::default()
        };
        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };
        let parsed = parse_body_text_section(&serialize_section(&section)).unwrap();
        let out = &parsed.paragraphs[0];

        assert_eq!(out.text, "가나");
        assert_eq!(
            out.controls.len(),
            1,
            "표식이 컨트롤로 남아야 한다: {:?}",
            out.controls
        );
        match &out.controls[0] {
            Control::IndexMark(im) => {
                assert_eq!(im.first_key, "위로보상금과");
                assert_eq!(im.second_key, "");
            }
            other => panic!("IndexMark 가 아니다: {other:?}"),
        }
        assert_eq!(
            out.control_mask & (1 << 0x0016),
            1 << 0x0016,
            "찾아보기 표식은 컨트롤 문자 0x16 을 쓴다"
        );
    }

    /// 두 키가 모두 있는 표식도 그대로 왕복한다.
    #[test]
    fn test_roundtrip_index_mark_with_second_key() {
        let para = Paragraph {
            char_count: 9,
            controls: vec![Control::IndexMark(IndexMark {
                first_key: "첫키".to_string(),
                second_key: "둘째키".to_string(),
            })],
            ..Default::default()
        };
        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };
        let parsed = parse_body_text_section(&serialize_section(&section)).unwrap();
        match &parsed.paragraphs[0].controls[0] {
            Control::IndexMark(im) => {
                assert_eq!(
                    (im.first_key.as_str(), im.second_key.as_str()),
                    ("첫키", "둘째키")
                );
            }
            other => panic!("IndexMark 가 아니다: {other:?}"),
        }
    }

    /// 제목 차례 표시 라운드트립 — 위치·`ignore` 값·8유닛 폭이 모두 살아야 한다.
    ///
    /// 이 표시를 흘리면 문단 축이 8유닛 짧아지고, 한글은 축이 어긋난 lineseg 를
    /// 만나면 본문을 통째로 버린다(10k 스윕 F-절단군 77문서·2,237개).
    #[test]
    fn test_roundtrip_title_mark() {
        let para = Paragraph {
            // 표시 8 + 글자 2 + 끝 마커 1
            char_count: 11,
            text: "가나".to_string(),
            char_offsets: vec![8, 9],
            title_marks: vec![TitleMark {
                char_idx: 0,
                ignore: true,
            }],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            ..Default::default()
        };

        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };
        let parsed = parse_body_text_section(&serialize_section(&section)).unwrap();
        let out = &parsed.paragraphs[0];

        assert_eq!(out.text, "가나", "표시가 텍스트로 새면 안 된다");
        assert_eq!(
            out.title_marks,
            vec![TitleMark {
                char_idx: 0,
                ignore: true,
            }]
        );
        assert_eq!(out.char_offsets, vec![8, 9], "표시가 앞 8유닛을 점유한다");
        assert_eq!(out.char_count, 11);
        assert_eq!(
            out.control_mask & (1 << 0x0008),
            1 << 0x0008,
            "한컴 원본 실측(286/286)대로 control_mask 비트 8 을 세운다"
        );
    }

    /// `ignore="0"`(`Mign`)도 자기 ID 로 나가야 한다 — 두 fourcc 가 별개 컨트롤이다.
    #[test]
    fn test_roundtrip_title_mark_ignore_off() {
        let para = Paragraph {
            char_count: 10,
            text: "가".to_string(),
            char_offsets: vec![0],
            title_marks: vec![TitleMark {
                char_idx: 1,
                ignore: false,
            }],
            ..Default::default()
        };
        let section = Section {
            paragraphs: vec![para],
            raw_stream: None,
            ..Default::default()
        };
        let parsed = parse_body_text_section(&serialize_section(&section)).unwrap();
        assert_eq!(
            parsed.paragraphs[0].title_marks,
            vec![TitleMark {
                char_idx: 1,
                ignore: false,
            }]
        );
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

    /// [#4677] PARA_TEXT 밖을 가리키는 lineseg 는 파일에 나가지 않는다.
    ///
    /// 색인 표시(`idxm`)처럼 HWP5 저장에서 떨어지는 컨트롤이 있으면 문단이 8 유닛 짧아지는데,
    /// 원본 HWPX 의 lineseg 는 그 컨트롤을 센 `textpos` 를 그대로 들고 있다. 그 값을 그대로
    /// 쓰면 한글 2022 가 본문 전체를 버리고 빈 1쪽으로 연다.
    #[test]
    fn test_out_of_range_line_segs_are_not_serialized() {
        let para = Paragraph {
            char_count: 3,
            text: "AB".to_string(),
            char_offsets: vec![0, 1],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![
                LineSeg {
                    text_start: 0,
                    line_height: 500,
                    ..Default::default()
                },
                // 떨어져 나간 8 유닛 컨트롤을 센 잔재 — "AB" + 문단끝 = 3 유닛보다 멀리 나간다.
                // (경계값 `text_start == char_count` 는 한컴도 쓰는 정상 값이라 남긴다.)
                LineSeg {
                    text_start: 10,
                    line_height: 500,
                    ..Default::default()
                },
            ],
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
            parsed.paragraphs[0].line_segs.len(),
            1,
            "범위 밖 lineseg 는 레코드에서 제외된다"
        );
        assert_eq!(parsed.paragraphs[0].line_segs[0].text_start, 0);
    }

    /// [#4677] 끝 위치를 가리키는 lineseg 는 남는다 — 한글이 직접 쓰는 값이다.
    ///
    /// 빈 문단은 `char_count=1` 에 줄 `[0, 1]` 로 저장되는 일이 흔하다(저장소 샘플 5건에
    /// 40개 문단). 경계를 `>=` 로 잡으면 그 둘째 줄까지 잘려 평범한 문서를 편집·저장할
    /// 때마다 조판 정보가 사라진다 — 범위 밖 판정은 끝을 **넘어선** 값에만 걸어야 한다.
    #[test]
    fn test_line_seg_at_text_end_is_kept() {
        let para = Paragraph {
            char_count: 1,
            line_segs: vec![
                LineSeg {
                    text_start: 0,
                    line_height: 500,
                    ..Default::default()
                },
                // 끝 위치(= char_count)를 가리키는 줄 — 한글 원본에 실재한다.
                LineSeg {
                    text_start: 1,
                    line_height: 500,
                    ..Default::default()
                },
            ],
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
            parsed.paragraphs[0].line_segs.len(),
            2,
            "끝 위치를 가리키는 줄은 범위 밖이 아니다"
        );
    }

    /// [#4677] PARA_TEXT 를 내보내지 않는 문단의 `char_count` 는 끝 마커 1 이다.
    ///
    /// HWPX 파서는 다단락 필드의 고아 `<hp:fieldEnd>` 를 8 유닛으로 세지만 HWP5 저장기에는
    /// 그 자리를 쓸 방법이 없다. 헤더만 9 를 주장하고 글자는 하나도 없는 문단이 되면
    /// 한글 2022 는 본문 전체를 버린다.
    #[test]
    fn test_char_count_without_para_text_is_normalized() {
        let para = Paragraph {
            char_count: 9, // 고아 fieldEnd 8 유닛 + 끝 마커
            text: String::new(),
            controls: Vec::new(),
            has_para_text: false,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
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

        assert_eq!(
            parsed.paragraphs[0].char_count, 1,
            "PARA_TEXT 없는 문단은 끝 마커 1 만 센다"
        );
    }

    /// [#4677] 조판 전용 보강 줄은 파일에 나가지 않는다.
    ///
    /// HWPX RowBreak 표 셀의 저장 높이를 채우려고 덧붙인 줄은 본문에 없는 줄이다. 한컴은
    /// 그 셀 문단에 줄을 하나만 쓰고, 두 줄짜리로 저장하면 본문 전체를 버린다.
    #[test]
    fn test_layout_only_fill_lines_are_not_serialized() {
        let para = Paragraph {
            char_count: 3,
            text: "AB".to_string(),
            char_offsets: vec![0, 1],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![
                LineSeg {
                    text_start: 0,
                    line_height: 500,
                    ..Default::default()
                },
                // 셀 높이를 채우려고 덧붙인 줄 — 위치는 범위 안이지만 본문에 없는 줄이다.
                LineSeg {
                    text_start: 1,
                    line_height: 500,
                    ..Default::default()
                },
            ],
            layout_only_fill_lines: 1,
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
            parsed.paragraphs[0].line_segs.len(),
            1,
            "조판 전용 보강 줄은 레코드에서 제외된다"
        );
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

    /// [#1793] 묶음 빈칸(NBSP, U+00A0)은 코드 30(0x1E)으로 직렬화되어야 한다.
    /// 코드 24(0x18)는 하이픈이라 재파싱 시 '-' 로 손상된다.
    #[test]
    fn test_nbsp_serializes_as_code_30() {
        let para = Paragraph {
            char_count: 4,
            text: "가\u{00A0}나".to_string(),
            char_offsets: vec![0, 1, 2],
            ..Default::default()
        };

        let bytes = test_serialize_para_text(&para);

        assert_eq!(&bytes[2..4], &0x001E_u16.to_le_bytes());
    }

    /// [NBSP mask] U+00A0 은 PARA_TEXT 에 코드 0x1E 로 방출되므로 PARA_HEADER control_mask
    /// 비트 30 도 서야 한다(안 서면 한컴에서 헤더/텍스트 불일치).
    #[test]
    fn test_nbsp_sets_control_mask_bit_30() {
        let para = Paragraph {
            char_count: 4,
            text: "가\u{00A0}나".to_string(),
            char_offsets: vec![0, 1, 2],
            ..Default::default()
        };
        assert_ne!(
            compute_control_mask(&para) & (1u32 << 0x001E),
            0,
            "U+00A0 포함 시 control_mask 비트 30 이 서야 PARA_TEXT(0x1E)와 일치한다"
        );
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
                ..Default::default()
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

    /// [#1795] FIELD_END 전용 갭(8 cu)을 다음 컨트롤(FIELD_BEGIN)이 선점하면
    /// 이후 char_offsets 가 시프트되어 lineseg text_start 매핑이 어긋난다.
    /// 필드 2개 문단에서 스트림 배치와 offsets 가 라운드트립 보존되는지 검증.
    #[test]
    fn test_field_end_gap_not_stolen_by_next_control() {
        let make_field = || {
            Control::Field(Field {
                field_type: FieldType::Hyperlink,
                ctrl_id: tags::FIELD_HYPERLINK,
                ..Default::default()
            })
        };
        let para = Paragraph {
            char_count: 38,
            text: "ab cd".to_string(),
            // [FB0 0..8] a=8 b=9 [FE0 10..18] ' '=18 [FB1 19..27] c=27 d=28 [FE1] 0x0D
            char_offsets: vec![8, 9, 18, 27, 28],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            controls: vec![make_field(), make_field()],
            field_ranges: vec![
                crate::model::paragraph::FieldRange {
                    start_char_idx: 0,
                    end_char_idx: 2,
                    control_idx: 0,
                    ..Default::default()
                },
                crate::model::paragraph::FieldRange {
                    start_char_idx: 3,
                    end_char_idx: 5,
                    control_idx: 1,
                    ..Default::default()
                },
            ],
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
        let p = &parsed.paragraphs[0];
        assert_eq!(p.text, "ab cd");
        assert_eq!(
            p.char_offsets,
            vec![8, 9, 18, 27, 28],
            "FIELD_END 갭 선점으로 char_offsets 가 시프트되면 안 된다"
        );
        assert_eq!(p.field_ranges.len(), 2);
        assert_eq!(
            p.field_ranges[1].start_char_idx, 3,
            "두 번째 필드 범위가 앞으로 당겨지면 안 된다"
        );
        assert_eq!(p.field_ranges[1].end_char_idx, 5);
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

    #[test]
    fn issue4680_serializer_fallback_shifts_all_leading_control_coordinates() {
        use crate::model::page::PageDef;

        let para = Paragraph {
            text: "AB".to_string(),
            char_offsets: vec![0, 1],
            char_shapes: vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 1,
                },
                CharShapeRef {
                    start_pos: 1,
                    char_shape_id: 2,
                },
            ],
            range_tags: vec![RangeTag {
                start: 0,
                end: 1,
                tag: 0x0100_0003,
            }],
            line_segs: vec![
                LineSeg {
                    text_start: 0,
                    line_height: 500,
                    text_height: 400,
                    ..Default::default()
                },
                LineSeg {
                    text_start: 1,
                    line_height: 500,
                    text_height: 400,
                    ..Default::default()
                },
            ],
            controls: vec![Control::ColumnDef(Default::default())],
            ..Default::default()
        };
        let section = Section {
            section_def: SectionDef {
                page_def: PageDef {
                    width: 59528,
                    height: 84188,
                    ..Default::default()
                },
                ..Default::default()
            },
            paragraphs: vec![para],
            raw_stream: None,
        };

        let bytes = serialize_section(&section);
        let parsed = parse_body_text_section(&bytes).unwrap();
        let parsed_para = &parsed.paragraphs[0];

        assert_eq!(parsed_para.char_offsets, vec![16, 17]);
        assert_eq!(parsed_para.char_shapes[0].start_pos, 0);
        assert_eq!(parsed_para.char_shapes[1].start_pos, 17);
        assert_eq!(parsed_para.range_tags[0].start, 16);
        assert_eq!(parsed_para.range_tags[0].end, 17);
        assert_eq!(parsed_para.line_segs[0].text_start, 0);
        assert_eq!(parsed_para.line_segs[1].text_start, 17);
    }

    /// #4424: `Control::Hyperlink` 는 `control_char_code_and_id` 에서 `(0x000B, 0)` 이라
    /// PARA_TEXT 에 확장 컨트롤 문자 0x000B 만 남기고 CTRL_HEADER 레코드는 만들지 않는다.
    ///
    /// 파서(`parse_para_text`)는 확장 컨트롤 문자를 만날 때마다 `ctrl_idx` 를 올려
    /// `controls[]` 와 위치를 1:1로 맞춘다(`is_extended_only_ctrl_char` 가 11 을 포함).
    /// 짝 없는 0x000B 하나가 그 카운터를 한 칸 올려버리므로, **뒤따르는 필드의
    /// `field_ranges[].control_idx` 가 실제 `controls[]` 인덱스보다 하나 크게 잡힌다.**
    ///
    /// `controls` 벡터 자체는 CTRL_HEADER 레코드 순서로 따로 만들어지므로 멀쩡하다 —
    /// 어긋나는 것은 위치↔인덱스 매핑이고, 그 매핑을 `wasm_api` 의 필드 조회들이 쓴다.
    #[test]
    fn test_hyperlink_does_not_shift_following_field_control_idx() {
        let para = Paragraph {
            // 'A'(1) + Hyperlink(1) + FIELD_BEGIN(1) + 'B'(1) + FIELD_END(1) + 문단끝(1)
            char_count: 6,
            text: "AB".to_string(),
            char_offsets: vec![0, 10],
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                ..Default::default()
            }],
            controls: vec![
                Control::Hyperlink(Hyperlink {
                    url: String::new(),
                    text: String::new(),
                }),
                Control::Field(Field {
                    field_type: FieldType::ClickHere,
                    ctrl_id: crate::parser::tags::FIELD_CLICKHERE,
                    ..Default::default()
                }),
            ],
            field_ranges: vec![FieldRange {
                start_char_idx: 1,
                end_char_idx: 2,
                control_idx: 1,
                end_field_id: 0,
                inner_slot_count: 0,
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
        let p = &parsed.paragraphs[0];

        // 하이퍼링크는 레코드가 없으니 controls 에는 필드 하나만 남는다 (손실은 이 테스트의
        // 관심사가 아니다 — 소실 자체는 #4397/#4424 본문에서 따로 다룬다).

        assert_eq!(
            p.controls.len(),
            1,
            "하이퍼링크는 레코드가 없으니 controls 에는 필드 하나만 남는다"
        );
        assert!(
            matches!(p.controls[0], Control::Field(_)),
            "controls[0] 이 필드여야 한다: {:?}",
            p.controls[0]
        );

        // 핵심 단언: 필드를 가리키는 인덱스가 실제 위치(0)여야 한다.
        assert_eq!(
            p.field_ranges.len(),
            1,
            "필드 범위가 하나 잡혀야 한다: {:?}",
            p.field_ranges
        );
        assert_eq!(
            p.field_ranges[0].control_idx, 0,
            "짝 없는 0x000B 가 ctrl_idx 를 밀어 필드 인덱스가 어긋났다 \
             (controls[{}] 는 존재하지 않는다)",
            p.field_ranges[0].control_idx
        );
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
