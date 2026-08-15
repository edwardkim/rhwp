//! BodyText 섹션 파싱
//!
//! BodyText/Section{N} 스트림의 레코드를 파싱하여 Section(문단 목록)으로 변환.
//! 레코드의 level 필드로 부모-자식 관계를 결정한다.
//!
//! 레코드 트리 구조 예시:
//! ```text
//! PARA_HEADER (level 0)
//!   PARA_TEXT (level 1)
//!   PARA_CHAR_SHAPE (level 1)
//!   PARA_LINE_SEG (level 1)
//!   CTRL_HEADER (level 1)  ← secd, cold, tbl, 등
//!     PAGE_DEF (level 2)
//!     FOOTNOTE_SHAPE (level 2)
//!     ...
//! ```

use super::byte_reader::ByteReader;
use super::record::Record;
use super::tags;

use crate::model::control::{Control, UnknownControl};
use crate::model::document::{RawRecord, Section, SectionDef};
use crate::model::footnote::FootnoteShape;
use crate::model::header_footer::{HeaderFooterApply, MasterPage};
use crate::model::page::{
    BindingMethod, ColumnDef, ColumnDirection, ColumnType, PageBorderFill, PageDef,
};
use crate::model::paragraph::{
    CharShapeRef, ColumnBreakType, FieldRange, LineSeg, OrphanFieldEnd, Paragraph, RangeTag,
    TitleMark,
};

/// `PARA_TEXT` 한 레코드에서 뽑아낸 문단 본문 축 정보.
struct ParaTextParts {
    text: String,
    char_offsets: Vec<u32>,
    field_ranges: Vec<FieldRange>,
    tab_extended: Vec<[u16; 7]>,
    title_marks: Vec<TitleMark>,
    orphan_field_ends: Vec<OrphanFieldEnd>,
}

/// BodyText 파싱 에러
#[derive(Debug)]
pub enum BodyTextError {
    RecordError(String),
    ParseError(String),
}

impl std::fmt::Display for BodyTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BodyTextError::RecordError(e) => write!(f, "BodyText 레코드 오류: {}", e),
            BodyTextError::ParseError(e) => write!(f, "BodyText 파싱 오류: {}", e),
        }
    }
}

impl std::error::Error for BodyTextError {}

/// 섹션 레코드 데이터를 파싱하여 Section으로 변환
///
/// data: 압축 해제된(배포용은 복호화+해제된) 레코드 바이트 스트림
pub fn parse_body_text_section(data: &[u8]) -> Result<Section, BodyTextError> {
    let records = Record::read_all(data).map_err(|e| BodyTextError::RecordError(e.to_string()))?;

    let mut section = Section::default();
    let mut idx = 0;

    while idx < records.len() {
        if records[idx].tag_id == tags::HWPTAG_PARA_HEADER {
            let base_level = records[idx].level;
            let start = idx;
            idx += 1;

            // 자식 레코드 수집 (level > base_level)
            while idx < records.len() && records[idx].level > base_level {
                idx += 1;
            }

            let para_records = &records[start..idx];
            let paragraph = parse_paragraph(para_records)?;

            // 구역 정의 추출
            for ctrl in &paragraph.controls {
                if let Control::SectionDef(sd) = ctrl {
                    section.section_def = (**sd).clone();
                }
            }

            section.paragraphs.push(paragraph);
        } else {
            idx += 1;
        }
    }

    link_orphan_field_ends(&mut section.paragraphs);

    // 확장 바탕쪽 파싱: 마지막 문단 이후의 LIST_HEADER (level=1)
    // HWP 바이너리에서 확장 바탕쪽(마지막 쪽, 임의 쪽)은 Section 스트림 끝에 저장되지만,
    // level=1로 태그되어 마지막 문단의 자식으로 오인됨.
    // 전체 레코드를 재스캔하여 마지막 PARA_HEADER(level=0) 이후의 LIST_HEADER(level=1)를 추출.
    {
        let all_records = Record::read_all(data).unwrap_or_default();
        let last_para0_idx = all_records
            .iter()
            .rposition(|r| r.tag_id == tags::HWPTAG_PARA_HEADER && r.level == 0);
        if let Some(lp) = last_para0_idx {
            // 마지막 문단의 본래 자식 레코드 범위 결정 (PARA_TEXT, PARA_CHAR_SHAPE 등)
            // LIST_HEADER(level=1)가 나타나면 그 이후는 확장 바탕쪽
            let mut scan = lp + 1;
            while scan < all_records.len() {
                if all_records[scan].tag_id == tags::HWPTAG_LIST_HEADER
                    && all_records[scan].level == 1
                {
                    // 확장 바탕쪽 발견
                    let tail: Vec<RawRecord> = all_records[scan..]
                        .iter()
                        .map(|r| RawRecord {
                            tag_id: r.tag_id,
                            level: r.level,
                            data: r.data.clone(),
                        })
                        .collect();
                    let ext_mps = parse_master_pages_from_raw(&tail, 0);
                    section.section_def.master_pages.extend(ext_mps);
                    break;
                }
                scan += 1;
            }
        }
    }

    Ok(section)
}

/// 다단락 필드의 종료 마커에 짝 `fieldBegin` 의 id 를 채운다.
///
/// PARA_TEXT 의 종료 마커에는 짝 id 가 없어서(`04 00 6b 6c 63 09 01 00 …` — ctrl_id
/// 자리에 필드 종류만) 문단 단위 파싱만으로는 알 수 없다. 섹션을 순서대로 훑으며
/// 아직 닫히지 않은 필드를 쌓아 두고 연결한다.
///
/// 매달린 참조(`beginIDRef="0"`)를 그대로 내보내면 **한글이 파일을 열지 못한다**
/// (01752 실측). 짝을 못 찾은 종료 마커는 8유닛 슬롯만 지키고 id 는 0 으로 남긴다.
fn link_orphan_field_ends(paragraphs: &mut [Paragraph]) {
    // (필드 인스턴스 id, HWP5 ctrl_id)
    let mut open_fields: Vec<(u32, u32)> = Vec::new();

    for para in paragraphs.iter_mut() {
        // 이 문단의 종료 마커는 **앞서 열린** 필드를 닫는다.
        for ofe in para.orphan_field_ends.iter_mut() {
            if ofe.begin_id_ref == 0 {
                if let Some((id, ctrl_id)) = open_fields.pop() {
                    ofe.begin_id_ref = id;
                    ofe.begin_ctrl_id = ctrl_id;
                }
            }
        }

        // 이 문단에서 열리고 여기서 닫히지 않은 필드를 쌓는다.
        for (i, ctrl) in para.controls.iter().enumerate() {
            let Control::Field(field) = ctrl else {
                continue;
            };
            let closed_here = para.field_ranges.iter().any(|fr| fr.control_idx == i);
            if !closed_here && field.field_id != 0 {
                open_fields.push((field.field_id, field.ctrl_id));
            }
        }
    }
}

/// [#4827] 문단↔표↔셀 상호재귀 깊이 상한.
///
/// 셀 안의 문단이 다시 표를 품는 사이클(`parse_paragraph`→`parse_ctrl_header`→
/// `parse_control`→`parse_table_control`→`parse_cell`→`parse_paragraph_list`→
/// `parse_paragraph`)에 상한이 없으면, 손상 문서가 스택을 고갈시켜 SIGSEGV(패닉과 달리
/// `catch_unwind` 로 못 잡음) 를 낸다. 레코드 레벨은 10비트(≤1023)라 표 중첩이 최대 ~341겹까지
/// 파일로 도달 가능하고, 그 깊이가 스레드 기본 스택 한계 근처라 크래시/완주가 비결정적으로 갈린다
/// (#4822 §2). 이 재귀 계열은 머리말/꼬리말·각주/미주·글상자·캡션까지 **전부 `parse_paragraph` 를
/// 경유**하므로, 그 진입 깊이를 스레드-로컬로 세어 한 곳에서 전 경로를 막는다(파라미터를 여러
/// 호출부에 관통시키지 않는다). HWPX `MAX_HWPX_SECTION_DEPTH`(#4759)·HWP3(#4285)·HWP5 묶음
/// 개체(#4761)·HML 의 형제 가드와 같은 취지·같은 값이다. 실문서의 표 중첩은 이에 한참 못 미친다.
pub(crate) const MAX_HWP5_SECTION_DEPTH: u32 = 64;

thread_local! {
    static HWP5_SECTION_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// `parse_paragraph` 진입 시 재귀 깊이를 +1 하고 이탈(Drop, 오류 전파·조기 반환 포함) 시
/// 되돌리는 RAII 가드. 상한 초과면 스택을 고갈시키기 전에 오류로 거부한다.
struct SectionDepthGuard;

impl SectionDepthGuard {
    fn enter() -> Result<SectionDepthGuard, BodyTextError> {
        HWP5_SECTION_DEPTH.with(|d| {
            if d.get() >= MAX_HWP5_SECTION_DEPTH {
                return Err(BodyTextError::ParseError(format!(
                    "문단 중첩이 {MAX_HWP5_SECTION_DEPTH} 단계를 초과했습니다(표·셀 상호재귀 상한)"
                )));
            }
            d.set(d.get() + 1);
            Ok(SectionDepthGuard)
        })
    }
}

impl Drop for SectionDepthGuard {
    fn drop(&mut self) {
        HWP5_SECTION_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

/// 문단 레코드 그룹에서 Paragraph 구성
///
/// records[0] = PARA_HEADER, records[1..] = 자식 레코드
pub fn parse_paragraph(records: &[Record]) -> Result<Paragraph, BodyTextError> {
    // [#4827] 문단↔표↔셀 상호재귀 깊이 상한 — 위 `SectionDepthGuard` 참고. 진입 즉시 +1,
    // 반환(오류·조기 반환 포함) 시 -1. 상한 초과 시 `parse_paragraph_list` 의 `if let Ok(..)`
    // 가 해당 하위 트리만 절단하고 나머지는 정상 파싱한다.
    let _depth_guard = SectionDepthGuard::enter()?;

    if records.is_empty() || records[0].tag_id != tags::HWPTAG_PARA_HEADER {
        return Err(BodyTextError::ParseError("PARA_HEADER 레코드 없음".into()));
    }

    let mut para = parse_para_header(&records[0].data);
    let base_level = records[0].level;

    let mut i = 1;
    while i < records.len() {
        let record = &records[i];

        // 직접 자식만 처리 (level == base_level + 1)
        if record.level != base_level + 1 {
            i += 1;
            continue;
        }

        match record.tag_id {
            tags::HWPTAG_PARA_TEXT => {
                let parts = parse_para_text(&record.data);
                para.text = parts.text;
                para.char_offsets = parts.char_offsets;
                para.field_ranges = parts.field_ranges;
                para.tab_extended = parts.tab_extended;
                para.title_marks = parts.title_marks;
                para.orphan_field_ends = parts.orphan_field_ends;
                para.has_para_text = true;
            }
            tags::HWPTAG_PARA_CHAR_SHAPE => {
                para.char_shapes = parse_para_char_shape(&record.data);
            }
            tags::HWPTAG_PARA_LINE_SEG => {
                para.line_segs = parse_para_line_seg(&record.data);
            }
            tags::HWPTAG_PARA_RANGE_TAG => {
                para.range_tags = parse_para_range_tag(&record.data);
            }
            tags::HWPTAG_CTRL_HEADER => {
                // 컨트롤의 자식 레코드 범위 수집
                let ctrl_start = i;
                i += 1;
                while i < records.len() && records[i].level > base_level + 1 {
                    i += 1;
                }
                let ctrl_records = &records[ctrl_start..i];
                let mut control = parse_ctrl_header(ctrl_records);

                // CTRL_DATA 레코드 추출 (라운드트립 보존용)
                // 중첩 CTRL_HEADER 이전까지만 검색하여 내부 컨트롤의 CTRL_DATA 혼입 방지
                //
                // SectionDef는 바탕쪽 같은 중첩 raw record를 extra_child_records에도 보존하므로,
                // 다음 중첩 CTRL_HEADER 전의 직접 자식만 문단 control 슬롯의 canonical owner로
                // 삼는다. 다른 control은 Picture/Shape처럼 CTRL_DATA가 더 깊은 level에 올 수
                // 있어 기존 탐색 계약을 유지한다.
                let ctrl_data_record = if matches!(control, Control::SectionDef(_)) {
                    section_def_ctrl_data_index(&ctrl_records[1..], ctrl_records[0].level)
                        .map(|index| &ctrl_records[index + 1])
                } else {
                    ctrl_records[1..]
                        .iter()
                        .take_while(|r| r.tag_id != tags::HWPTAG_CTRL_HEADER)
                        .find(|r| r.tag_id == tags::HWPTAG_CTRL_DATA)
                };
                let ctrl_data = ctrl_data_record.map(|r| r.data.clone());

                // CTRL_DATA에서 필드 이름 추출 → Field.ctrl_data_name에 설정
                if let Control::Field(ref mut field) = control {
                    if let Some(ref cd) = ctrl_data {
                        field.ctrl_data_name = parse_ctrl_data_field_name(cd);
                    }
                }

                // CTRL_DATA에서 책갈피 이름 추출 (HWP 스펙: 책갈피 이름은 HWPTAG_CTRL_DATA의 ParameterSet에 저장)
                if let Control::Bookmark(ref mut bm) = control {
                    if let Some(ref cd) = ctrl_data {
                        if let Some(name) = parse_ctrl_data_field_name(cd) {
                            bm.name = name;
                        }
                    }
                }

                para.controls.push(control);
                para.ctrl_data_records.push(ctrl_data);
                continue; // i는 이미 전진됨
            }
            _ => {}
        }

        i += 1;
    }

    Ok(para)
}

/// PARA_HEADER 바이너리 데이터 파싱
///
/// 레이아웃 (최소 12바이트, 실제로 22~24바이트):
/// - u32: nChars (bit 31은 플래그)
/// - u32: controlMask
/// - u16: paraShapeId
/// - u8:  styleId
/// - u8:  breakType (bits 0-2)
/// - [이후 10~12바이트: numCharShapes, numRangeTags, numLineSegs, instanceId 등]
fn parse_para_header(data: &[u8]) -> Paragraph {
    let mut r = ByteReader::new(data);
    let mut para = Paragraph::default();

    let n_chars_raw = r.read_u32().unwrap_or(0);
    para.char_count = n_chars_raw & 0x7FFFFFFF;
    para.char_count_msb = n_chars_raw & 0x80000000 != 0;

    para.control_mask = r.read_u32().unwrap_or(0);
    para.para_shape_id = r.read_u16().unwrap_or(0);
    para.style_id = r.read_u8().unwrap_or(0);

    // 단 나누기 종류 (표 61: 비트 플래그)
    // 0x01 = 구역 나누기, 0x02 = 다단 나누기, 0x04 = 쪽 나누기, 0x08 = 단 나누기
    let break_val = r.read_u8().unwrap_or(0);
    para.raw_break_type = break_val;
    para.column_type = if break_val & 0x04 != 0 {
        ColumnBreakType::Page
    } else if break_val & 0x08 != 0 {
        ColumnBreakType::Column
    } else if break_val & 0x01 != 0 {
        ColumnBreakType::Section
    } else if break_val & 0x02 != 0 {
        ColumnBreakType::MultiColumn
    } else {
        ColumnBreakType::None
    };

    // 12바이트 이후 추가 데이터 보존 (라운드트립용)
    if data.len() > 12 {
        para.raw_header_extra = data[12..].to_vec();
    }

    para
}

/// PARA_TEXT 바이너리 데이터에서 텍스트 추출
///
/// HWP의 텍스트는 UTF-16LE로 저장되며, 0x0000~0x001F 범위는 컨트롤 문자.
/// - 확장 컨트롤 문자: 8 code unit (16바이트) 차지
/// - 인라인 컨트롤 문자: 1 code unit (2바이트) 차지
fn parse_para_text(data: &[u8]) -> ParaTextParts {
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
fn is_extended_only_ctrl_char(ch: u16) -> bool {
    matches!(ch, 1..=3 | 11..=12 | 14..=18 | 21..=23)
}

/// 16바이트 컨트롤 문자 여부 (8 code unit 차지)
///
/// HWP 5.0 제어 문자 분류 (표 6):
///   char (1 code unit = 2바이트): 0, 10, 13, 24-31
///   inline (8 code unit = 16바이트): 4-9, 19-20
///   extended (8 code unit = 16바이트): 1-3, 11-12, 14-18, 21-23
///
/// 탭(9), 줄 끝(10), 문단 끝(13)은 호출 전에 별도 처리된다.
fn is_extended_ctrl_char(ch: u16) -> bool {
    matches!(ch, 1..=8 | 11..=12 | 14..=23)
}

/// PARA_CHAR_SHAPE 바이너리 데이터 파싱
///
/// 각 항목: [u32 start_pos] + [u32 char_shape_id] (8바이트)
fn parse_para_char_shape(data: &[u8]) -> Vec<CharShapeRef> {
    let mut refs = Vec::new();
    let mut r = ByteReader::new(data);

    while r.remaining() >= 8 {
        let start_pos = r.read_u32().unwrap_or(0);
        let char_shape_id = r.read_u32().unwrap_or(0);
        refs.push(CharShapeRef {
            start_pos,
            char_shape_id,
        });
    }

    refs
}

/// PARA_LINE_SEG 바이너리 데이터 파싱
///
/// 각 항목: 36바이트 (u32 + i32×7 + u32)
fn parse_para_line_seg(data: &[u8]) -> Vec<LineSeg> {
    let mut segs = Vec::new();
    let mut r = ByteReader::new(data);

    while r.remaining() >= 36 {
        segs.push(LineSeg {
            text_start: r.read_u32().unwrap_or(0),
            vertical_pos: r.read_i32().unwrap_or(0),
            line_height: r.read_i32().unwrap_or(0),
            text_height: r.read_i32().unwrap_or(0),
            baseline_distance: r.read_i32().unwrap_or(0),
            line_spacing: r.read_i32().unwrap_or(0),
            column_start: r.read_i32().unwrap_or(0),
            segment_width: r.read_i32().unwrap_or(0),
            tag: r.read_u32().unwrap_or(0),
        });
    }

    // [#2070] 전부 0 높이(lh=0, th=0)인 PARA_LINE_SEG 는 부재로 정규화한다.
    // 생성계 문서(80168 등 규제영향분석서)는 lineseg 를 0 으로 채워 저장하는데,
    // 0 높이 lineseg 는 배치 권위가 없고(한글은 열 때 재계산) 실저장 취급 시
    // NO_LS 성장 경로가 죽어 셀/문단 높이가 선언값으로 붕괴한다
    // (hwpx section.rs parse_paragraph 와 동일 규칙).
    if !segs.is_empty()
        && segs
            .iter()
            .all(|s| s.line_height == 0 && s.text_height == 0)
    {
        return Vec::new();
    }

    segs
}

/// PARA_RANGE_TAG 바이너리 데이터 파싱
///
/// 각 항목: 12바이트 (u32 × 3)
fn parse_para_range_tag(data: &[u8]) -> Vec<RangeTag> {
    let mut result = Vec::new();
    let mut r = ByteReader::new(data);

    while r.remaining() >= 12 {
        result.push(RangeTag {
            start: r.read_u32().unwrap_or(0),
            end: r.read_u32().unwrap_or(0),
            tag: r.read_u32().unwrap_or(0),
        });
    }

    result
}

/// 레코드 목록에서 문단 리스트 추출 (재귀 파싱용)
///
/// TABLE 셀, 머리말/꼬리말, 각주/미주 등에서 문단 목록을 파싱할 때 사용.
pub fn parse_paragraph_list(records: &[Record]) -> Vec<Paragraph> {
    let mut paragraphs = Vec::new();
    let mut idx = 0;

    while idx < records.len() {
        if records[idx].tag_id == tags::HWPTAG_PARA_HEADER {
            let base_level = records[idx].level;
            let start = idx;
            idx += 1;
            while idx < records.len() && records[idx].level > base_level {
                idx += 1;
            }
            if let Ok(para) = parse_paragraph(&records[start..idx]) {
                paragraphs.push(para);
            }
        } else {
            idx += 1;
        }
    }

    // 셀·각주 같은 중첩 문단 목록도 자기 안에서 필드가 여러 문단에 걸칠 수 있다.
    // 최상위에서만 연결하면 그 안의 종료 마커가 짝을 못 찾아 저장에서 빠진다.
    link_orphan_field_ends(&mut paragraphs);

    paragraphs
}

/// CTRL_HEADER 레코드 그룹 파싱
///
/// records[0] = CTRL_HEADER, records[1..] = 자식 레코드
/// ctrl_id(처음 4바이트)로 컨트롤 종류를 식별한다.
fn parse_ctrl_header(records: &[Record]) -> Control {
    if records.is_empty() || records[0].tag_id != tags::HWPTAG_CTRL_HEADER {
        return Control::Unknown(UnknownControl { ctrl_id: 0 });
    }

    let data = &records[0].data;
    if data.len() < 4 {
        return Control::Unknown(UnknownControl { ctrl_id: 0 });
    }

    let ctrl_id = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let ctrl_data = &data[4..];
    let child_records = &records[1..];

    match ctrl_id {
        tags::CTRL_SECTION_DEF => {
            let section_def = parse_section_def(ctrl_data, child_records, records[0].level);
            Control::SectionDef(Box::new(section_def))
        }
        tags::CTRL_COLUMN_DEF => {
            let column_def = parse_column_def_ctrl(ctrl_data);
            Control::ColumnDef(column_def)
        }
        _ => {
            // 표, 도형, 그림, 머리말/꼬리말 등은 control.rs에서 처리
            super::control::parse_control(ctrl_id, ctrl_data, child_records)
        }
    }
}

/// SectionDef가 문단 control 슬롯으로 소유할 CTRL_DATA의 자식 배열 인덱스.
///
/// 기존 control 파싱 계약처럼 첫 중첩 CTRL_HEADER까지만 검색한다. 그 뒤의
/// 직접 자식 CTRL_DATA는 바탕쪽 등 raw 자식 레코드의 일부일 수 있으므로
/// 원래 위치를 유지해야 한다.
fn section_def_ctrl_data_index(child_records: &[Record], ctrl_level: u16) -> Option<usize> {
    let direct_child_level = ctrl_level.saturating_add(1);
    child_records
        .iter()
        .enumerate()
        .take_while(|(_, record)| record.tag_id != tags::HWPTAG_CTRL_HEADER)
        .find(|(_, record)| {
            record.tag_id == tags::HWPTAG_CTRL_DATA && record.level == direct_child_level
        })
        .map(|(index, _)| index)
}

/// 구역 정의 파싱 ('secd' 컨트롤)
///
/// ctrl_data: CTRL_HEADER의 ctrl_id 이후 데이터
/// child_records: 자식 레코드 (CTRL_DATA, PAGE_DEF, FOOTNOTE_SHAPE, PAGE_BORDER_FILL, raw 중첩)
/// ctrl_level: SectionDef CTRL_HEADER의 record level
fn parse_section_def(ctrl_data: &[u8], child_records: &[Record], ctrl_level: u16) -> SectionDef {
    let mut sd = SectionDef::default();
    let mut r = ByteReader::new(ctrl_data);

    sd.flags = r.read_u32().unwrap_or(0);
    sd.column_spacing = r.read_i16().unwrap_or(0);
    sd.line_grid = r.read_i16().unwrap_or(0);
    sd.char_grid = r.read_i16().unwrap_or(0);
    sd.default_tab_spacing = r.read_u32().unwrap_or(0);
    sd.outline_numbering_id = r.read_u16().unwrap_or(0);
    sd.page_num = r.read_u16().unwrap_or(0);
    sd.picture_num = r.read_u16().unwrap_or(0);
    sd.table_num = r.read_u16().unwrap_or(0);
    sd.equation_num = r.read_u16().unwrap_or(0);

    // 파싱된 필드 이후 추가 바이트 보존 (라운드트립용)
    let consumed = 4 + 2 + 2 + 2 + 4 + 2 + 2 + 2 + 2 + 2; // = 24 bytes
    if ctrl_data.len() > consumed {
        sd.raw_ctrl_extra = ctrl_data[consumed..].to_vec();
    }

    // 숨기기 플래그 (flags에서 추출)
    // HWP 5.0 spec table 119: bit 0..5 are first-page hide flags.
    sd.hide_header = sd.flags & 0x0001 != 0;
    sd.hide_footer = sd.flags & 0x0002 != 0;
    sd.hide_master_page = sd.flags & 0x0004 != 0; // bit 2 (HWP5 스펙, 첫쪽 바탕쪽 감춤)
    sd.hide_border = sd.flags & 0x0008 != 0;
    sd.hide_fill = sd.flags & 0x0010 != 0;
    sd.hide_empty_line = sd.flags & 0x00080000 != 0; // bit 19: 빈 줄 감추기
    sd.page_num_type = ((sd.flags >> 20) & 0x03) as u8; // bit 20-21: 쪽 번호 종류 (0=이어서, 1=홀수, 2=짝수)

    // 자식 레코드에서 PAGE_DEF, FOOTNOTE_SHAPE, PAGE_BORDER_FILL 파싱
    let mut footnote_count = 0u32;
    let mut border_fill_count = 0u32;
    let owned_ctrl_data_index = section_def_ctrl_data_index(child_records, ctrl_level);
    for (index, record) in child_records.iter().enumerate() {
        if Some(index) == owned_ctrl_data_index {
            // 첫 중첩 CTRL_HEADER 전의 직접 자식 CTRL_DATA는
            // Paragraph.ctrl_data_records가 canonical owner다.
            continue;
        }

        match record.tag_id {
            tags::HWPTAG_PAGE_DEF => {
                sd.page_def = parse_page_def(&record.data);
            }
            tags::HWPTAG_FOOTNOTE_SHAPE => {
                let fs = parse_footnote_shape_record(&record.data);
                if footnote_count == 0 {
                    sd.footnote_shape = fs;
                } else {
                    sd.endnote_shape = fs;
                }
                footnote_count += 1;
            }
            tags::HWPTAG_PAGE_BORDER_FILL => {
                let pbf = parse_page_border_fill(&record.data);
                if border_fill_count == 0 {
                    sd.page_border_fill = pbf;
                } else {
                    sd.extra_page_border_fills.push(pbf);
                }
                border_fill_count += 1;
            }
            _ => {
                // 인식하지 못한 자식 레코드 보존 (바탕쪽 LIST_HEADER, 문단 등)
                sd.extra_child_records
                    .push(crate::model::document::RawRecord {
                        tag_id: record.tag_id,
                        level: record.level,
                        data: record.data.clone(),
                    });
            }
        }
    }

    // extra_child_records에서 바탕쪽 (LIST_HEADER) 파싱
    sd.master_pages = parse_master_pages_from_raw(&sd.extra_child_records, sd.flags);

    sd
}

/// extra_child_records에서 바탕쪽 LIST_HEADER를 파싱한다.
///
/// LIST_HEADER(tag 66)가 나타나면 바탕쪽으로 파싱.
/// 기본 순서: 1번째=양쪽(Both), 2번째=홀수(Odd), 3번째=짝수(Even).
/// 단, 한컴 2020이 HWPX의 희소 Odd 바탕쪽을 HWP5로 저장할 때는 LIST_HEADER 하나와
/// SECTION_DEF 상위 플래그 `0x80000000`을 쓴다. 이 조합은 앞 구역의 짝수 쪽을
/// 상속하고 현재 구역의 홀수 쪽만 바꾸므로 첫 목록을 Both로 해석하면 안 된다.
fn parse_master_pages_from_raw(raw_records: &[RawRecord], section_flags: u32) -> Vec<MasterPage> {
    let mut master_pages = Vec::new();

    // RawRecord를 Record로 변환
    let records: Vec<Record> = raw_records
        .iter()
        .map(|r| Record {
            tag_id: r.tag_id,
            level: r.level,
            size: r.data.len() as u32,
            data: r.data.clone(),
        })
        .collect();

    // 바탕쪽 LIST_HEADER 위치 수집 (level 2만 — 하위 레벨은 도형 내부 텍스트박스)
    let top_level = records
        .iter()
        .filter(|r| r.tag_id == tags::HWPTAG_LIST_HEADER)
        .map(|r| r.level)
        .min()
        .unwrap_or(0);
    let list_header_positions: Vec<usize> = records
        .iter()
        .enumerate()
        .filter(|(_, r)| r.tag_id == tags::HWPTAG_LIST_HEADER && r.level == top_level)
        .map(|(i, _)| i)
        .collect();

    if list_header_positions.is_empty() {
        return master_pages;
    }

    let apply_order = [
        HeaderFooterApply::Both,
        HeaderFooterApply::Odd,
        HeaderFooterApply::Even,
    ];

    for (mp_idx, &start) in list_header_positions.iter().enumerate() {
        let apply_to = master_page_apply_to(section_flags, list_header_positions.len(), mp_idx)
            .or_else(|| apply_order.get(mp_idx).copied())
            .unwrap_or(HeaderFooterApply::Both);

        // LIST_HEADER 데이터 파싱
        let list_data = &records[start].data;
        let raw_list_header = list_data.to_vec();
        let mut r = ByteReader::new(list_data);

        // 표준 LIST_HEADER 프리픽스: para_count(2) + attr(4) + width_ref(2) = 8바이트
        let _para_count = r.read_u16().unwrap_or(0);
        let _list_attr = r.read_u32().unwrap_or(0);
        let _width_ref = r.read_u16().unwrap_or(0);

        // 바탕쪽 정보 (표 139, 10바이트)
        let text_width = r.read_u32().unwrap_or(0);
        let text_height = r.read_u32().unwrap_or(0);
        let text_ref = r.read_u8().unwrap_or(0);
        let num_ref = r.read_u8().unwrap_or(0);

        // 영역 0×0 LIST_HEADER는 MEMO/주석 컨트롤의 텍스트 박스가 오분류된 것.
        // 실제 바탕쪽은 반드시 text_width > 0 || text_height > 0.
        if text_width == 0 && text_height == 0 {
            continue;
        }

        // 확장 플래그 (byte 18-19, 표 139 이후)
        let ext_flags = r.read_u16().unwrap_or(0);

        // Task #347: ext_flags 비트로 확장 여부 판별 (bit 1) — 휴리스틱(같은 apply_to 중복)
        // 단독으로는 ext_flags=0x03 같은 케이스(첫 등록 + 확장 표시)를 놓침.
        // 비트 + 휴리스틱 OR 조합으로 보강.
        let overlap = ext_flags & 0x01 != 0;
        let is_extension = (ext_flags & 0x02 != 0)
            || master_pages
                .iter()
                .any(|m: &MasterPage| m.apply_to == apply_to);

        // 이 LIST_HEADER에 속하는 문단 레코드 범위 결정
        let end = if mp_idx + 1 < list_header_positions.len() {
            list_header_positions[mp_idx + 1]
        } else {
            records.len()
        };

        // LIST_HEADER 다음 레코드부터 문단 파싱
        let para_records = &records[start + 1..end];
        let paragraphs = parse_paragraph_list(para_records);

        master_pages.push(MasterPage {
            apply_to,
            is_extension,
            overlap,
            replace_base: false,
            ext_flags,
            page_front: false, // HWP5 바이너리 바탕쪽엔 pageFront 개념 없음
            text_direction: 0, // HWP5 바이너리 바탕쪽엔 textDirection 개념 없음
            paragraphs,
            text_width,
            text_height,
            text_ref,
            num_ref,
            hwpx_page_number: None,
            raw_list_header,
        });
    }

    master_pages
}

fn master_page_apply_to(
    section_flags: u32,
    list_header_count: usize,
    master_page_index: usize,
) -> Option<HeaderFooterApply> {
    const MASTER_PAGE_FLAGS_MASK: u32 = 0xe000_0000;
    const HANCOM_SINGLE_ODD_MASTER_PAGE_FLAGS: u32 = 0x8000_0000;

    (list_header_count == 1
        && master_page_index == 0
        && section_flags & MASTER_PAGE_FLAGS_MASK == HANCOM_SINGLE_ODD_MASTER_PAGE_FLAGS)
        .then_some(HeaderFooterApply::Odd)
}

/// 단 정의 파싱 ('cold' 컨트롤)
///
/// ctrl_data: CTRL_HEADER의 ctrl_id 이후 데이터
fn parse_column_def_ctrl(ctrl_data: &[u8]) -> ColumnDef {
    let mut cd = ColumnDef::default();
    let mut r = ByteReader::new(ctrl_data);

    // 표 140: UINT16 속성 (표 141 참조)
    let attr = r.read_u16().unwrap_or(0);
    cd.raw_attr = attr;
    // bit 0-1: 단 종류
    cd.column_type = match attr & 0x03 {
        1 => ColumnType::Distribute,
        2 => ColumnType::Parallel,
        _ => ColumnType::Normal,
    };
    // bit 2-9: 단 개수 (1-255)
    cd.column_count = ((attr >> 2) & 0xFF) as u16;
    // bit 10-11: 단 방향
    cd.direction = match (attr >> 10) & 0x03 {
        1 => ColumnDirection::RightToLeft,
        _ => ColumnDirection::LeftToRight,
    };
    // bit 12: 단 너비 동일 여부
    cd.same_width = attr & (1 << 12) != 0;

    // hwplib 기준: same_width 여부에 따라 바이트 순서가 다름
    if !cd.same_width && cd.column_count > 1 {
        // same_width=false: [attr2(2)] [col0_width(2) col0_gap(2)] [col1_width(2) col1_gap(2)] ...
        // 너비/간격 값은 비례값 (합계=32768), 절대 HWPUNIT이 아님
        let _attr2 = r.read_u16().unwrap_or(0);
        for _ in 0..cd.column_count {
            let w = r.read_i16().unwrap_or(0);
            let g = r.read_i16().unwrap_or(0);
            cd.widths.push(w);
            cd.gaps.push(g);
        }
        cd.proportional_widths = true;
    } else {
        // same_width=true: [gap(2)] [attr2(2)]
        cd.spacing = r.read_i16().unwrap_or(0);
        let _attr2 = r.read_u16().unwrap_or(0);
    }

    // 표 140: 단 구분선
    cd.separator_type = r.read_u8().unwrap_or(0);
    cd.separator_width = r.read_u8().unwrap_or(0);
    cd.separator_color = r.read_color_ref().unwrap_or(0);

    cd
}

/// 용지 설정 파싱 (HWPTAG_PAGE_DEF)
///
/// 레이아웃: u32 × 9 (크기+여백) + u32 attr
fn parse_page_def(data: &[u8]) -> PageDef {
    let mut pd = PageDef::default();
    let mut r = ByteReader::new(data);

    pd.width = r.read_u32().unwrap_or(59528);
    pd.height = r.read_u32().unwrap_or(84188);
    pd.margin_left = r.read_u32().unwrap_or(8504);
    pd.margin_right = r.read_u32().unwrap_or(8504);
    pd.margin_top = r.read_u32().unwrap_or(5669);
    pd.margin_bottom = r.read_u32().unwrap_or(4252);
    pd.margin_header = r.read_u32().unwrap_or(4252);
    pd.margin_footer = r.read_u32().unwrap_or(4252);
    pd.margin_gutter = r.read_u32().unwrap_or(0);
    pd.attr = r.read_u32().unwrap_or(0);

    pd.landscape = pd.attr & 0x01 != 0;
    pd.binding = match (pd.attr >> 1) & 0x03 {
        1 => BindingMethod::DuplexSided,
        2 => BindingMethod::TopFlip,
        _ => BindingMethod::SingleSided,
    };

    pd
}

/// 각주/미주 모양 파싱 (HWPTAG_FOOTNOTE_SHAPE)
///
/// 스펙 문서는 26바이트로 기술하지만, 실제 레코드는 28바이트.
/// note_spacing과 separator_line_type 사이에 미문서화된 2바이트 필드가 있음.
fn parse_footnote_shape_record(data: &[u8]) -> FootnoteShape {
    let mut fs = FootnoteShape::default();
    let mut r = ByteReader::new(data);

    fs.attr = r.read_u32().unwrap_or(0);

    // 표 134: bit 8~9는 위치, bit 10~11은 번호 매기기이다.
    fs.apply_attr_fields_from_raw();

    fs.user_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
    fs.prefix_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
    fs.suffix_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
    fs.start_number = r.read_u16().unwrap_or(1);
    fs.separator_length = r.read_i16().unwrap_or(0) as i32;
    fs.separator_margin_top = r.read_i16().unwrap_or(0);
    // HWP5 실파일에서는 이 슬롯이 한컴 UI "구분선 위" 값으로 쓰이는 사례가 있다.
    // HWPX aboveLine 은 separator_margin_top 에 들어오므로 정규화 접근자에서 합친다.
    fs.separator_margin_bottom = r.read_i16().unwrap_or(0);
    fs.note_spacing = r.read_i16().unwrap_or(0);

    // 미문서화 2바이트 (스펙에는 없지만 실제 데이터에 존재)
    fs.raw_unknown = r.read_u16().unwrap_or(0);

    fs.separator_line_type = r.read_u8().unwrap_or(0);
    fs.separator_line_width = r.read_u8().unwrap_or(0);
    fs.separator_color = r.read_color_ref().unwrap_or(0);

    fs
}

/// 쪽 테두리/배경 파싱 (HWPTAG_PAGE_BORDER_FILL)
fn parse_page_border_fill(data: &[u8]) -> PageBorderFill {
    let mut pbf = PageBorderFill::default();
    let mut r = ByteReader::new(data);

    pbf.attr = r.read_u32().unwrap_or(0);
    pbf.spacing_left = r.read_i16().unwrap_or(0);
    pbf.spacing_right = r.read_i16().unwrap_or(0);
    pbf.spacing_top = r.read_i16().unwrap_or(0);
    pbf.spacing_bottom = r.read_i16().unwrap_or(0);
    pbf.border_fill_id = r.read_u16().unwrap_or(0);
    // HWP5 PAGE_BORDER_FILL attr bit0 is the stored textBorder value.
    // Hancom Office dialog shows bit0=0 as paper basis and bit0=1 as page basis.
    // Task #1129 Stage 28: on initial load, a Hancom page-basis setting must
    // render from the page/body area edge, not the paper edge.
    pbf.ui_basis = if (pbf.attr & 0x01) != 0 {
        pbf.basis = crate::model::page::PageBorderBasis::BodyBased;
        crate::model::page::PageBorderUiBasis::Page
    } else {
        pbf.basis = crate::model::page::PageBorderBasis::PaperBased;
        crate::model::page::PageBorderUiBasis::Paper
    };

    pbf
}

/// CTRL_DATA에서 필드 이름을 추출한다.
///
/// CTRL_DATA 레이아웃 (누름틀 필드):
///   바이트 0~9: 헤더 (paramset 등)
///   바이트 10~11: WORD - 필드 이름 길이 (글자 수)
///   바이트 12~: WCHAR[len] - 필드 이름 (UTF-16LE)
fn parse_ctrl_data_field_name(data: &[u8]) -> Option<String> {
    if data.len() < 12 {
        return None;
    }
    let name_len = u16::from_le_bytes([data[10], data[11]]) as usize;
    if name_len == 0 {
        return None;
    }
    let name_bytes = &data[12..];
    if name_bytes.len() < name_len * 2 {
        return None;
    }
    let wchars: Vec<u16> = name_bytes[..name_len * 2]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let name = String::from_utf16_lossy(&wchars);
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests;
