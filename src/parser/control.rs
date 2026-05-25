//! 컨트롤 파싱 (표, 도형, 그림, 머리말/꼬리말, 각주/미주)
//!
//! CTRL_HEADER의 ctrl_id로 컨트롤 종류를 식별하여 파싱한다.
//! 셀, 머리말/꼬리말 등의 내부 문단은 body_text::parse_paragraph_list로 재귀 처리.

use super::body_text::parse_paragraph_list;
use super::byte_reader::ByteReader;
use super::record::Record;
use super::tags;

use std::collections::HashMap;

use crate::model::control::{
    AutoNumber, AutoNumberType, Bookmark, CharOverlap, Control, Equation, Field, FieldType,
    FormObject, FormType, HiddenComment, NewNumber, PageHide, PageNumberPos, UnknownControl,
};
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply};
use crate::model::image::{ImageEffect, Picture};
use crate::model::shape::{
    ArcShape, Caption, CaptionDirection, CaptionVertAlign, CommonObjAttr, CurveShape,
    DrawingObjAttr, EllipseShape, GroupShape, HorzAlign, HorzRelTo, LineShape, PolygonShape,
    RectangleShape, ShapeComponentAttr, ShapeObject, TextWrap, VertAlign, VertRelTo,
};
use crate::model::style::{Fill, ShapeBorderLine};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::model::Padding;
use crate::model::Point;

/// ctrl_id 기반으로 컨트롤 파싱
///
/// body_text::parse_ctrl_header에서 secd/cold 이외의 컨트롤을 위임받는다.
pub fn parse_control(ctrl_id: u32, ctrl_data: &[u8], child_records: &[Record]) -> Control {
    match ctrl_id {
        tags::CTRL_TABLE => parse_table_control(ctrl_data, child_records),
        tags::CTRL_GEN_SHAPE => parse_gso_control(ctrl_data, child_records),
        tags::CTRL_HEADER => parse_header_control(ctrl_data, child_records),
        tags::CTRL_FOOTER => parse_footer_control(ctrl_data, child_records),
        tags::CTRL_FOOTNOTE => parse_footnote_control(ctrl_data, child_records),
        tags::CTRL_ENDNOTE => parse_endnote_control(ctrl_data, child_records),
        tags::CTRL_HIDDEN_COMMENT => parse_hidden_comment_control(child_records),
        tags::CTRL_AUTO_NUMBER => parse_auto_number(ctrl_data),
        tags::CTRL_NEW_NUMBER => parse_new_number(ctrl_data),
        tags::CTRL_PAGE_NUM_POS => parse_page_num_pos(ctrl_data),
        tags::CTRL_PAGE_HIDE => parse_page_hide(ctrl_data),
        tags::CTRL_BOOKMARK => parse_bookmark(ctrl_data),
        tags::CTRL_TCPS => parse_char_overlap(ctrl_data),
        tags::CTRL_EQUATION => parse_equation_control(ctrl_data, child_records),
        tags::CTRL_FORM => parse_form_control(ctrl_data, child_records),
        id if tags::is_field_ctrl_id(id) => parse_field_control(id, ctrl_data),
        _ => Control::Unknown(UnknownControl { ctrl_id }),
    }
}

// ============================================================
// 필드 컨트롤 (%clk, %hlk 등)
// ============================================================

/// ctrl_id를 FieldType으로 매핑
fn ctrl_id_to_field_type(ctrl_id: u32) -> FieldType {
    match ctrl_id {
        tags::FIELD_CLICKHERE => FieldType::ClickHere,
        tags::FIELD_HYPERLINK => FieldType::Hyperlink,
        tags::FIELD_BOOKMARK => FieldType::Bookmark,
        tags::FIELD_DATE => FieldType::Date,
        tags::FIELD_DOCDATE => FieldType::DocDate,
        tags::FIELD_PATH => FieldType::Path,
        tags::FIELD_MAILMERGE => FieldType::MailMerge,
        tags::FIELD_CROSSREF => FieldType::CrossRef,
        tags::FIELD_FORMULA => FieldType::Formula,
        tags::FIELD_SUMMARY => FieldType::Summary,
        tags::FIELD_USERINFO => FieldType::UserInfo,
        tags::FIELD_MEMO => FieldType::Memo,
        tags::FIELD_PRIVATE_INFO => FieldType::PrivateInfoSecurity,
        tags::FIELD_TOC => FieldType::TableOfContents,
        _ => FieldType::Unknown,
    }
}

/// 필드 컨트롤 파싱 (표 154: ctrl_id + 속성 + 기타속성 + command + id)
///
/// ctrl_data는 CTRL_HEADER에서 ctrl_id(4바이트)를 제외한 나머지 데이터이다.
/// 그러나 HWP 파일에서 필드의 CTRL_HEADER 포맷은 일반 컨트롤과 다르다:
/// - 일반 컨트롤: CTRL_HEADER = [ctrl_id(4)] + [ctrl_data]
/// - 필드 컨트롤: CTRL_HEADER = [ctrl_id(4)] + [속성(4)] + [기타속성(1)] + [command_len(2)] + [command] + [id(4)]
fn parse_field_control(ctrl_id: u32, ctrl_data: &[u8]) -> Control {
    let field_type = ctrl_id_to_field_type(ctrl_id);

    // ctrl_data: 속성(4) + 기타속성(1) + command_len(2) + command(가변) + id(4)
    if ctrl_data.len() < 7 {
        // 데이터가 부족하면 최소한의 Field 반환
        return Control::Field(Field {
            field_type,
            ctrl_id,
            ..Default::default()
        });
    }

    let mut reader = ByteReader::new(ctrl_data);
    let properties = reader.read_u32().unwrap_or(0);
    let extra_properties = reader.read_u8().unwrap_or(0);
    let command_len = reader.read_u16().unwrap_or(0) as usize;

    let command = if command_len > 0 && reader.remaining() >= command_len * 2 {
        let mut chars = Vec::with_capacity(command_len);
        for _ in 0..command_len {
            chars.push(reader.read_u16().unwrap_or(0));
        }
        String::from_utf16_lossy(&chars)
    } else {
        String::new()
    };

    let field_id = if reader.remaining() >= 4 {
        reader.read_u32().unwrap_or(0)
    } else {
        0
    };

    let memo_index = if reader.remaining() >= 4 {
        reader.read_u32().unwrap_or(0)
    } else {
        0
    };

    Control::Field(Field {
        field_type,
        command,
        properties,
        extra_properties,
        field_id,
        ctrl_id,
        ctrl_data_name: None,
        memo_index,
        memo_paragraphs: Vec::new(),
    })
}

// ============================================================
// 표 ('tbl ')
// ============================================================

/// 표 컨트롤 파싱
fn parse_table_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let mut table = Table::default();

    // ctrl_data = CommonObjAttr (Shape/GSO와 동일 구조, hwplib: ForCtrlHeaderGso)
    // 표의 CTRL_HEADER는 Shape와 동일한 CommonObjAttr로 시작
    if !ctrl_data.is_empty() {
        table.common = super::control::shape::parse_common_obj_attr(ctrl_data);
        // CommonObjAttr.attr → table.attr 동기화 (기존 코드 호환)
        table.attr = table.common.attr;
        // 라운드트립 보존용 원본 데이터
        table.raw_ctrl_data = ctrl_data.to_vec();
        // 바깥 여백 파싱 (CommonObjAttr 내 offset 24..32)
        if ctrl_data.len() >= 32 {
            table.outer_margin_left = i16::from_le_bytes([ctrl_data[24], ctrl_data[25]]);
            table.outer_margin_right = i16::from_le_bytes([ctrl_data[26], ctrl_data[27]]);
            table.outer_margin_top = i16::from_le_bytes([ctrl_data[28], ctrl_data[29]]);
            table.outer_margin_bottom = i16::from_le_bytes([ctrl_data[30], ctrl_data[31]]);
        }
    }

    // HWPTAG_TABLE 레코드 위치 찾기
    let table_record_idx = child_records
        .iter()
        .position(|r| r.tag_id == tags::HWPTAG_TABLE);

    // HWPTAG_TABLE 이전에 LIST_HEADER가 있으면 캡션
    if let Some(table_idx) = table_record_idx {
        // 캡션 LIST_HEADER 찾기 (TABLE 이전)
        let caption_start = child_records[..table_idx]
            .iter()
            .position(|r| r.tag_id == tags::HWPTAG_LIST_HEADER);

        if let Some(start) = caption_start {
            // 캡션 레코드 범위 수집 (TABLE 레코드 이전까지)
            let caption_records: Vec<Record> =
                child_records[start..table_idx].iter().cloned().collect();
            if !caption_records.is_empty() {
                table.caption = Some(parse_caption(&caption_records));
            }
        }
    }

    // 자식 레코드 순회: TABLE, LIST_HEADER(셀)
    let mut idx = 0;
    let mut table_record_seen = false;

    while idx < child_records.len() {
        match child_records[idx].tag_id {
            tags::HWPTAG_TABLE => {
                parse_table_record(&child_records[idx].data, &mut table);
                table_record_seen = true;
                idx += 1;
            }
            tags::HWPTAG_LIST_HEADER => {
                // TABLE 레코드 이전의 LIST_HEADER는 캡션 (이미 처리됨)
                if !table_record_seen {
                    // 캡션 레코드 건너뛰기
                    let base_level = child_records[idx].level;
                    idx += 1;
                    while idx < child_records.len() {
                        if child_records[idx].level <= base_level {
                            if child_records[idx].tag_id == tags::HWPTAG_TABLE
                                || child_records[idx].tag_id == tags::HWPTAG_LIST_HEADER
                            {
                                break;
                            }
                        }
                        idx += 1;
                    }
                    continue;
                }

                // TABLE 레코드 이후의 LIST_HEADER는 셀
                let base_level = child_records[idx].level;
                let start = idx;
                idx += 1;
                // 다음 셀(LIST_HEADER) 또는 다른 레코드까지 수집
                while idx < child_records.len() {
                    if child_records[idx].level < base_level {
                        break;
                    }
                    // 같은 레벨의 LIST_HEADER/TABLE은 다음 셀이므로 중단
                    if child_records[idx].level == base_level {
                        match child_records[idx].tag_id {
                            tags::HWPTAG_LIST_HEADER | tags::HWPTAG_TABLE => break,
                            _ => {}
                        }
                    }
                    idx += 1;
                }
                let cell = parse_cell(&child_records[start..idx]);
                table.cells.push(cell);
            }
            _ => {
                idx += 1;
            }
        }
    }

    table.rebuild_grid();
    Control::Table(Box::new(table))
}

/// HWPTAG_TABLE 레코드 데이터 파싱
fn parse_table_record(data: &[u8], table: &mut Table) {
    let mut r = ByteReader::new(data);

    let attr = r.read_u32().unwrap_or(0);
    table.page_break = match attr & 0x03 {
        1 | 3 => TablePageBreak::CellBreak,
        2 => TablePageBreak::RowBreak,
        _ => TablePageBreak::None,
    };
    table.repeat_header = attr & 0x04 != 0;
    // 원본 attr 전체 보존 (라운드트립용)
    table.raw_table_record_attr = attr;

    table.row_count = r.read_u16().unwrap_or(0);
    table.col_count = r.read_u16().unwrap_or(0);
    table.cell_spacing = r.read_i16().unwrap_or(0);

    // 안쪽 여백
    table.padding = Padding {
        left: r.read_i16().unwrap_or(0),
        right: r.read_i16().unwrap_or(0),
        top: r.read_i16().unwrap_or(0),
        bottom: r.read_i16().unwrap_or(0),
    };

    // 행별 셀 수 (HWP 스펙: UINT16[NRows])
    for _ in 0..table.row_count {
        if let Ok(h) = r.read_i16() {
            table.row_sizes.push(h);
        }
    }

    table.border_fill_id = r.read_u16().unwrap_or(0);

    // 영역 속성 (zones): UINT16 nZones + TableZone[nZones]
    if r.remaining() >= 2 {
        let n_zones = r.read_u16().unwrap_or(0) as usize;
        for _ in 0..n_zones {
            if r.remaining() >= 10 {
                let start_row = r.read_u16().unwrap_or(0);
                let start_col = r.read_u16().unwrap_or(0);
                let end_row = r.read_u16().unwrap_or(0);
                let end_col = r.read_u16().unwrap_or(0);
                let bf_id = r.read_u16().unwrap_or(0);
                table.zones.push(crate::model::table::TableZone {
                    start_col,
                    start_row,
                    end_col,
                    end_row,
                    border_fill_id: bf_id,
                });
            }
        }
    }

    // 나머지 추가 데이터 보존 (라운드트립용)
    if r.remaining() > 0 {
        table.raw_table_record_extra = r.read_bytes(r.remaining()).unwrap_or_default();
    }
}

/// 표 셀 파싱 (LIST_HEADER + 내부 문단)
///
/// records[0] = LIST_HEADER, records[1..] = 셀 내부 레코드
fn parse_cell(records: &[Record]) -> Cell {
    let mut cell = Cell::default();

    if records.is_empty() {
        return cell;
    }

    let data = &records[0].data;
    let mut r = ByteReader::new(data);

    // LIST_HEADER 공통 필드
    let _n_paragraphs = r.read_u16().unwrap_or(0);
    let list_attr = r.read_u32().unwrap_or(0);
    cell.list_header_width_ref = r.read_u16().unwrap_or(0);

    // list_attr 비트필드에서 텍스트 방향, 세로 정렬 추출 (표 67)
    // 스펙 문서는 bit 0~6으로 기술하지만, 실제로는 상위 16비트(bit 16~22)에 위치
    // bit 16~18: 텍스트 방향 (0=가로, 1=세로)
    // bit 19~20: 줄바꿈 방식
    // bit 21~22: 세로 정렬 (0=top, 1=center, 2=bottom)
    cell.text_direction = ((list_attr >> 16) & 0x07) as u8;
    let v_align = ((list_attr >> 21) & 0x03) as u8;
    cell.vertical_align = match v_align {
        1 => VerticalAlign::Center,
        2 => VerticalAlign::Bottom,
        _ => VerticalAlign::Top,
    };

    // list_header_width_ref (bytes 6-7)에는 셀 확장 속성이 포함됨
    // hwplib ListHeaderPropertyForCell 기준:
    //   bit 0 (=property bit 16): 안 여백 지정
    //   bit 1 (=property bit 17): 셀 보호
    //   bit 2 (=property bit 18): 제목 셀
    //   bit 3 (=property bit 19): 양식모드 편집 가능
    // 현재 IR은 미해석 확장 bit를 분해하지 않고 list_header_width_ref 원값으로 보존한다.
    cell.is_header = (cell.list_header_width_ref & 0x04) != 0;

    // 셀 속성 (표 82: 26바이트)
    cell.col = r.read_u16().unwrap_or(0);
    cell.row = r.read_u16().unwrap_or(0);
    cell.col_span = r.read_u16().unwrap_or(1);
    cell.row_span = r.read_u16().unwrap_or(1);
    cell.width = r.read_u32().unwrap_or(0);
    cell.height = r.read_u32().unwrap_or(0);

    cell.padding = Padding {
        left: r.read_i16().unwrap_or(0),
        right: r.read_i16().unwrap_or(0),
        top: r.read_i16().unwrap_or(0),
        bottom: r.read_i16().unwrap_or(0),
    };

    cell.border_fill_id = r.read_u16().unwrap_or(0);

    // "안 여백 지정" (LIST_HEADER bytes 6-7 bit 0, hwplib: isApplyInnerMargin) 미설정이면
    // 셀 패딩을 무시하고 테이블 기본 패딩을 사용해야 함
    // HWP는 이 비트가 0이어도 패딩 필드에 값을 저장하지만 렌더링에서 무시
    // "안 여백 지정" (width_ref bit 0): 셀 고유 여백 vs 표 기본 여백 선택
    // bit 0=1: 셀 고유 여백 사용 (파싱한 패딩값 그대로)
    // bit 0=0: 표 기본 여백 사용 — 단, 레이아웃 시 표 기본 패딩으로 대체
    // → 파싱 단계에서는 원본값을 보존하고, 레이아웃에서 처리
    cell.apply_inner_margin = (cell.list_header_width_ref & 0x0001) != 0;

    // 34바이트 이후 추가 데이터 보존 (라운드트립용)
    if r.remaining() > 0 {
        cell.raw_list_extra = r.read_bytes(r.remaining()).unwrap_or_default();
        // 셀 필드명 추출: raw_list_extra offset 14-15(name_len) + 16~(UTF-16LE)
        cell.field_name = parse_cell_field_name(&cell.raw_list_extra);
    }

    // 셀 내부 문단 파싱
    cell.paragraphs = parse_paragraph_list(&records[1..]);

    cell
}

/// 셀의 raw_list_extra에서 필드 이름을 추출한다.
/// 구조: raw_list_extra[14..16] = name_len (u16), [16..16+name_len*2] = UTF-16LE 문자열
fn parse_cell_field_name(extra: &[u8]) -> Option<String> {
    if extra.len() < 18 {
        return None;
    }
    let name_len = u16::from_le_bytes([extra[15], extra[16]]) as usize;
    if name_len == 0 || extra.len() < 17 + name_len * 2 {
        return None;
    }
    let wchars: Vec<u16> = extra[17..17 + name_len * 2]
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

/// 캡션 파싱 (LIST_HEADER + 캡션 데이터 + 내부 문단)
///
/// records[0] = LIST_HEADER (캡션 데이터 포함), records[1..] = 캡션 내부 문단
/// HWP 스펙: 표 73 (캡션 리스트), 표 74 (캡션), 표 75 (캡션 속성)
pub(crate) fn parse_caption(records: &[Record]) -> Caption {
    let mut caption = Caption::default();

    if records.is_empty() {
        return caption;
    }

    let data = &records[0].data;
    let mut r = ByteReader::new(data);

    // LIST_HEADER 공통 필드 (8바이트: n_para + list_attr + width_ref)
    let _n_paragraphs = r.read_u16().unwrap_or(0);
    let _list_attr = r.read_u32().unwrap_or(0);
    let _width_ref = r.read_u16().unwrap_or(0);

    // 캡션 데이터 (14바이트)
    // 속성 (4바이트): bit 0~1 = 방향, bit 2 = include_margin
    let caption_attr = r.read_u32().unwrap_or(0);
    caption.direction = match caption_attr & 0x03 {
        0 => CaptionDirection::Left,
        1 => CaptionDirection::Right,
        2 => CaptionDirection::Top,
        _ => CaptionDirection::Bottom,
    };
    caption.include_margin = (caption_attr >> 2) & 0x01 != 0;
    // list_attr bit 21~22: Left/Right 캡션의 세로 정렬 (0=Top, 1=Center, 2=Bottom)
    caption.vert_align = match (_list_attr >> 21) & 0x03 {
        1 => CaptionVertAlign::Center,
        2 => CaptionVertAlign::Bottom,
        _ => CaptionVertAlign::Top,
    };

    // 캡션 폭 (세로 방향일 때 사용)
    caption.width = r.read_u32().unwrap_or(0);

    // 캡션-틀 간격
    caption.spacing = r.read_i16().unwrap_or(0);

    // 텍스트 최대 길이
    caption.max_width = r.read_u32().unwrap_or(0);

    // 캡션 내부 문단 파싱
    caption.paragraphs = parse_paragraph_list(&records[1..]);

    caption
}

mod shape;
pub(crate) use shape::{parse_common_obj_attr, parse_gso_control};

// ============================================================
// 머리말/꼬리말 ('head'/'foot')
// ============================================================

/// 머리말 컨트롤 파싱
fn parse_header_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let mut header = Header::default();

    if !ctrl_data.is_empty() {
        let mut r = ByteReader::new(ctrl_data);
        let attr = r.read_u32().unwrap_or(0);
        header.raw_attr = attr;
        header.apply_to = match attr & 0x03 {
            1 => HeaderFooterApply::Even,
            2 => HeaderFooterApply::Odd,
            _ => HeaderFooterApply::Both,
        };
        // 4바이트 이후 추가 데이터 보존 (라운드트립용)
        if ctrl_data.len() > 4 {
            header.raw_ctrl_extra = ctrl_data[4..].to_vec();
        }
    }

    header.paragraphs = find_list_header_paragraphs(child_records);

    Control::Header(Box::new(header))
}

/// 꼬리말 컨트롤 파싱
fn parse_footer_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let mut footer = Footer::default();

    if !ctrl_data.is_empty() {
        let mut r = ByteReader::new(ctrl_data);
        let attr = r.read_u32().unwrap_or(0);
        footer.raw_attr = attr;
        footer.apply_to = match attr & 0x03 {
            1 => HeaderFooterApply::Even,
            2 => HeaderFooterApply::Odd,
            _ => HeaderFooterApply::Both,
        };
        // 4바이트 이후 추가 데이터 보존 (라운드트립용)
        if ctrl_data.len() > 4 {
            footer.raw_ctrl_extra = ctrl_data[4..].to_vec();
        }
    }

    footer.paragraphs = find_list_header_paragraphs(child_records);

    Control::Footer(Box::new(footer))
}

// ============================================================
// 각주/미주 ('fn  '/'en  ')
// ============================================================

/// 각주 컨트롤 파싱
fn parse_footnote_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let mut footnote = Footnote::default();
    // [Task #1050] hwplib CtrlHeaderFootnote 정합:
    //   number(UInt4) + before(WChar) + after(WChar) + numberShape(UInt4) + instanceId(UInt4, optional)
    if ctrl_data.len() >= 4 {
        let mut r = ByteReader::new(ctrl_data);
        footnote.number = r.read_u32().unwrap_or(0) as u16;
        if ctrl_data.len() >= 8 {
            footnote.before_decoration_letter = r.read_u16().unwrap_or(0);
            footnote.after_decoration_letter = r.read_u16().unwrap_or(0);
        }
        if ctrl_data.len() >= 12 {
            footnote.number_shape = r.read_u32().unwrap_or(0);
        }
        if ctrl_data.len() >= 16 {
            footnote.instance_id = r.read_u32().unwrap_or(0);
        }
    }

    footnote.paragraphs = find_list_header_paragraphs(child_records);
    footnote.list_header_property = find_list_header_property_for_footnote_endnote(child_records);

    Control::Footnote(Box::new(footnote))
}

/// 미주 컨트롤 파싱
fn parse_endnote_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let mut endnote = Endnote::default();
    // [Task #1050] CTRL_FOOTNOTE 와 동일 구조
    if ctrl_data.len() >= 4 {
        let mut r = ByteReader::new(ctrl_data);
        endnote.number = r.read_u32().unwrap_or(0) as u16;
        if ctrl_data.len() >= 8 {
            endnote.before_decoration_letter = r.read_u16().unwrap_or(0);
            endnote.after_decoration_letter = r.read_u16().unwrap_or(0);
        }
        if ctrl_data.len() >= 12 {
            endnote.number_shape = r.read_u32().unwrap_or(0);
        }
        if ctrl_data.len() >= 16 {
            endnote.instance_id = r.read_u32().unwrap_or(0);
        }
    }

    endnote.paragraphs = find_list_header_paragraphs(child_records);
    endnote.list_header_property = find_list_header_property_for_footnote_endnote(child_records);

    Control::Endnote(Box::new(endnote))
}

/// [Task #1050] CTRL_FOOTNOTE / CTRL_ENDNOTE 의 직속 LIST_HEADER property 읽기.
/// 형식: paraCount(SInt4) + property(UInt4) + 8 byte zero padding.
fn find_list_header_property_for_footnote_endnote(child_records: &[Record]) -> u32 {
    for record in child_records {
        if record.tag_id == crate::parser::tags::HWPTAG_LIST_HEADER && record.data.len() >= 8 {
            let mut r = ByteReader::new(&record.data);
            let _para_count = r.read_i32().unwrap_or(0);
            return r.read_u32().unwrap_or(0);
        }
    }
    0
}

// ============================================================
// 숨은 설명 ('tcmt')
// ============================================================

/// 숨은 설명 컨트롤 파싱
fn parse_hidden_comment_control(child_records: &[Record]) -> Control {
    let mut comment = HiddenComment::default();
    comment.paragraphs = find_list_header_paragraphs(child_records);
    Control::HiddenComment(Box::new(comment))
}

// ============================================================
// 단순 컨트롤 (AutoNumber, Bookmark, etc.)
// ============================================================

/// 자동 번호 파싱 (HWP 스펙 표 144, 표 145)
fn parse_auto_number(ctrl_data: &[u8]) -> Control {
    let mut an = AutoNumber::default();
    if ctrl_data.len() >= 4 {
        let mut r = ByteReader::new(ctrl_data);
        let attr = r.read_u32().unwrap_or(0);
        an.number_type = match attr & 0x0F {
            0 => AutoNumberType::Page,
            1 => AutoNumberType::Footnote,
            2 => AutoNumberType::Endnote,
            3 => AutoNumberType::Picture,
            4 => AutoNumberType::Table,
            5 => AutoNumberType::Equation,
            _ => AutoNumberType::Page,
        };
        an.format = ((attr >> 4) & 0xFF) as u8; // bit 4~11: 번호 모양 (표 134)
        an.superscript = attr & 0x1000 != 0; // bit 12: 위 첨자
                                             // 표 144: UINT16 번호 + WCHAR 사용자기호 + WCHAR 앞장식 + WCHAR 뒤장식
        an.number = r.read_u16().unwrap_or(0);
        an.user_symbol = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
        an.prefix_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
        an.suffix_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
    }
    Control::AutoNumber(an)
}

/// 새 번호 지정 파싱
fn parse_new_number(ctrl_data: &[u8]) -> Control {
    let mut nn = NewNumber::default();
    if ctrl_data.len() >= 6 {
        let mut r = ByteReader::new(ctrl_data);
        let attr = r.read_u32().unwrap_or(0);
        nn.number_type = match attr & 0x0F {
            0 => AutoNumberType::Page,
            1 => AutoNumberType::Footnote,
            2 => AutoNumberType::Endnote,
            3 => AutoNumberType::Picture,
            4 => AutoNumberType::Table,
            5 => AutoNumberType::Equation,
            _ => AutoNumberType::Page,
        };
        nn.number = r.read_u16().unwrap_or(0);
    }
    Control::NewNumber(nn)
}

/// 쪽 번호 위치 파싱 (HWP 스펙 표 149, 표 150)
fn parse_page_num_pos(ctrl_data: &[u8]) -> Control {
    let mut pnp = PageNumberPos::default();
    if ctrl_data.len() >= 4 {
        let mut r = ByteReader::new(ctrl_data);
        let attr = r.read_u32().unwrap_or(0);
        pnp.format = (attr & 0xFF) as u8; // bit 0~7: 번호 모양 (표 134)
        pnp.position = ((attr >> 8) & 0x0F) as u8; // bit 8~11: 표시 위치 (표 150)
                                                   // 표 149: WCHAR 사용자기호 + 앞장식 + 뒤장식 + 대시
        pnp.user_symbol = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
        pnp.prefix_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
        pnp.suffix_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
        pnp.dash_char = char::from_u32(r.read_u16().unwrap_or(0) as u32).unwrap_or('\0');
    }
    Control::PageNumberPos(pnp)
}

/// 감추기 파싱
fn parse_page_hide(ctrl_data: &[u8]) -> Control {
    let mut ph = PageHide::default();
    if ctrl_data.len() >= 4 {
        let mut r = ByteReader::new(ctrl_data);
        let attr = r.read_u32().unwrap_or(0);
        ph.hide_header = attr & 0x01 != 0;
        ph.hide_footer = attr & 0x02 != 0;
        ph.hide_master_page = attr & 0x04 != 0;
        ph.hide_border = attr & 0x08 != 0;
        ph.hide_fill = attr & 0x10 != 0;
        ph.hide_page_num = attr & 0x20 != 0;
    }
    Control::PageHide(ph)
}

/// 책갈피 파싱
fn parse_bookmark(ctrl_data: &[u8]) -> Control {
    let mut bm = Bookmark::default();
    if ctrl_data.len() >= 2 {
        let mut r = ByteReader::new(ctrl_data);
        if let Ok(name) = r.read_hwp_string() {
            bm.name = name;
        }
    }
    Control::Bookmark(bm)
}

/// 글자 겹침 파싱 (HWP 스펙 표 152)
///
/// ctrl_data 레이아웃 (ctrl_id 4바이트는 이미 제거된 상태):
///   WORD(2): 겹칠 글자 길이(len)
///   WCHAR[len](2×len): 겹칠 글자
///   UINT8(1): 테두리 타입
///   INT8(1): 내부 글자 크기
///   UINT8(1): 펼침
///   UINT8(1): charshape 아이디 수(cnt)
///   UINT[cnt](4×cnt): charshape_id 배열
fn parse_char_overlap(ctrl_data: &[u8]) -> Control {
    let mut co = CharOverlap::default();
    if ctrl_data.len() < 2 {
        return Control::CharOverlap(co);
    }

    let mut r = ByteReader::new(ctrl_data);

    let char_len = r.read_u16().unwrap_or(0) as usize;
    // WCHAR 배열 읽기 (UTF-16 서로게이트 쌍 처리)
    let mut wchars: Vec<u16> = Vec::with_capacity(char_len);
    for _ in 0..char_len {
        if let Ok(ch) = r.read_u16() {
            wchars.push(ch);
        }
    }
    // UTF-16 디코딩 (서로게이트 쌍 → Unicode 코드포인트)
    let mut i = 0;
    while i < wchars.len() {
        let w = wchars[i];
        if (0xD800..=0xDBFF).contains(&w) && i + 1 < wchars.len() {
            // 서로게이트 쌍
            let hi = w as u32;
            let lo = wchars[i + 1] as u32;
            if (0xDC00..=0xDFFF).contains(&(lo as u16)) {
                let code_point = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                if let Some(c) = char::from_u32(code_point) {
                    co.chars.push(c);
                }
                i += 2;
                continue;
            }
        }
        // 일반 BMP 문자
        if let Some(c) = char::from_u32(w as u32) {
            co.chars.push(c);
        }
        i += 1;
    }

    co.border_type = r.read_u8().unwrap_or(0);
    co.inner_char_size = r.read_i8().unwrap_or(0);
    co.expansion = r.read_u8().unwrap_or(0);

    let cs_count = r.read_u8().unwrap_or(0) as usize;
    for _ in 0..cs_count {
        if let Ok(id) = r.read_u32() {
            co.char_shape_ids.push(id);
        }
    }

    Control::CharOverlap(co)
}

// ============================================================
// 헬퍼 함수
// ============================================================

/// LIST_HEADER 이후의 문단 목록을 추출
///
/// 자식 레코드에서 LIST_HEADER를 찾고, 그 이후 레코드에서 문단 목록을 파싱.
/// LIST_HEADER와 PARA_HEADER가 동일 level인 경우가 있으므로 (표 셀 내 각주 등),
/// level 필터링 대신 LIST_HEADER 이후의 모든 레코드를 parse_paragraph_list에 위임.
fn find_list_header_paragraphs(
    child_records: &[Record],
) -> Vec<crate::model::paragraph::Paragraph> {
    let mut idx = 0;
    while idx < child_records.len() {
        if child_records[idx].tag_id == tags::HWPTAG_LIST_HEADER {
            return parse_paragraph_list(&child_records[idx + 1..]);
        }
        idx += 1;
    }
    Vec::new()
}

// ============================================================
// 수식 ('eqed')
// ============================================================

/// 수식 컨트롤 파싱
///
/// CTRL_HEADER(eqed)의 ctrl_data에서 CommonObjAttr를 읽고,
/// 자식 레코드 HWPTAG_EQEDIT에서 수식 스크립트 등을 추출한다.
fn parse_equation_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let common = parse_common_obj_attr(ctrl_data);

    let mut equation = Equation {
        common,
        raw_ctrl_data: ctrl_data.to_vec(),
        ..Default::default()
    };

    // HWPTAG_EQEDIT 자식 레코드 탐색
    if let Some(eq_rec) = child_records
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_EQEDIT)
    {
        let data = &eq_rec.data;
        let mut r = ByteReader::new(data);

        // attr: u32 (4바이트) — bit0: 스크립트 범위
        let _attr = r.read_u32().unwrap_or(0);

        // script: WCHAR 문자열 (길이 접두 UTF-16LE)
        if let Ok(script) = r.read_hwp_string() {
            equation.script = script;
        }

        // font_size: u32 (4바이트, HWPUNIT)
        equation.font_size = r.read_u32().unwrap_or(1000);

        // color: u32 (4바이트, COLORREF)
        equation.color = r.read_u32().unwrap_or(0);

        // baseline: i16 (2바이트)
        equation.baseline = r.read_i16().unwrap_or(0);

        // [Task #1061] unknown: u16 (2바이트) — HWP5 spec 표 105 누락 영역.
        // hwplib ForEQEdit.readUInt2() 정합. 한컴 실제 저장본에 baseline 과
        // version_info 사이 UINT16 zero 가 위치.
        equation.unknown = r.read_u16().unwrap_or(0);

        // version_info: WCHAR 문자열
        if let Ok(ver) = r.read_hwp_string() {
            equation.version_info = ver;
        }

        // font_name: WCHAR 문자열
        if let Ok(font) = r.read_hwp_string() {
            equation.font_name = font;
        }
    }

    Control::Equation(Box::new(equation))
}

// ============================================================
// 양식 개체 (form)
// ============================================================

/// 양식 개체 파싱
///
/// ctrl_data에서 width/height를 추출하고,
/// HWPTAG_FORM_OBJECT 자식 레코드에서 타입 ID와 속성 문자열을 파싱한다.
fn parse_form_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    let mut form = FormObject {
        enabled: true,
        ..Default::default()
    };

    // ctrl_data에서 width/height 추출 (bytes 12-19)
    if ctrl_data.len() >= 20 {
        let mut r = ByteReader::new(ctrl_data);
        let _attr = r.read_u32().unwrap_or(0);
        let _y_offset = r.read_i32().unwrap_or(0);
        let _x_offset = r.read_i32().unwrap_or(0);
        form.width = r.read_u32().unwrap_or(0);
        form.height = r.read_u32().unwrap_or(0);
    }

    // HWPTAG_FORM_OBJECT 자식 레코드에서 타입/속성 파싱
    if let Some(rec) = child_records
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_FORM_OBJECT)
    {
        let data = &rec.data;
        if data.len() >= 14 {
            // bytes 0-3: 타입 ID 문자열 (예: "tbp+", "tbc+", "boc+", "tbr+", "tde+")
            let type_id = &data[0..4];
            form.form_type = match type_id {
                b"tbp+" => FormType::PushButton,
                b"tbc+" => FormType::CheckBox,
                b"boc+" => FormType::ComboBox,
                b"tbr+" => FormType::RadioButton,
                b"tde+" => FormType::Edit,
                _ => FormType::PushButton,
            };

            // bytes 8-11: u32 전체 길이, bytes 12-13: u16 문자열 길이 (WCHAR 단위)
            let str_char_count = u16::from_le_bytes([data[12], data[13]]) as usize;
            let str_byte_start = 14;
            let str_byte_len = str_char_count * 2;

            if data.len() >= str_byte_start + str_byte_len {
                let str_bytes = &data[str_byte_start..str_byte_start + str_byte_len];
                let prop_str = decode_utf16le(str_bytes);
                parse_form_properties(&prop_str, &mut form);
            }
        }
    }

    Control::Form(Box::new(form))
}

/// UTF-16LE 바이트를 String으로 디코딩
fn decode_utf16le(data: &[u8]) -> String {
    let u16s: Vec<u16> = data
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&u16s)
}

/// 양식 개체 속성 문자열 파싱
///
/// 포맷: 속성간 공백 구분, 속성 내부 콜론 구분
/// - `Key:set:N:` → 컨테이너 (N=바이트 길이, 내용은 이어지는 속성들)
/// - `Key:wstring:N:VALUE` → N문자 길이 WCHAR 문자열
/// - `Key:int:VALUE` / `Key:bool:VALUE` → 정수/불린 (공백까지)
fn parse_form_properties(prop_str: &str, form: &mut FormObject) {
    let chars: Vec<char> = prop_str.chars().collect();
    let len = chars.len();
    let mut pos = 0;

    while pos < len {
        // 공백 건너뛰기
        while pos < len && chars[pos] == ' ' {
            pos += 1;
        }
        if pos >= len {
            break;
        }

        // Key 읽기 (':'까지)
        let key_start = pos;
        while pos < len && chars[pos] != ':' {
            pos += 1;
        }
        let key: String = chars[key_start..pos].iter().collect();
        if pos < len {
            pos += 1;
        } // ':' 건너뛰기

        // Type 읽기 (':'까지)
        let type_start = pos;
        while pos < len && chars[pos] != ':' {
            pos += 1;
        }
        let type_str: String = chars[type_start..pos].iter().collect();
        if pos < len {
            pos += 1;
        } // ':' 건너뛰기

        match type_str.as_str() {
            "set" => {
                // N(바이트 길이) 읽고 ':' 건너뛰기 — 내용은 이어지는 속성으로 처리됨
                while pos < len && chars[pos] != ':' {
                    pos += 1;
                }
                if pos < len {
                    pos += 1;
                }
            }
            "wstring" => {
                // N(문자 수) 읽기
                let n_start = pos;
                while pos < len && chars[pos] != ':' {
                    pos += 1;
                }
                let n: usize = chars[n_start..pos]
                    .iter()
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0);
                if pos < len {
                    pos += 1;
                } // ':' 건너뛰기
                  // 정확히 N문자 읽기
                let end = (pos + n).min(len);
                let value: String = chars[pos..end].iter().collect();
                pos = end;
                apply_form_property(&key, &value, form);
            }
            "int" | "bool" => {
                // 공백까지 값 읽기
                let v_start = pos;
                while pos < len && chars[pos] != ' ' {
                    pos += 1;
                }
                let value: String = chars[v_start..pos].iter().collect();
                apply_form_property(&key, &value, form);
            }
            _ => {
                // 알 수 없는 타입 — 공백까지 건너뛰기
                while pos < len && chars[pos] != ' ' {
                    pos += 1;
                }
            }
        }
    }
}

/// 파싱된 속성을 FormObject에 적용
fn apply_form_property(key: &str, value: &str, form: &mut FormObject) {
    match key {
        "Name" => form.name = value.to_string(),
        "Caption" => form.caption = value.to_string(),
        "Text" => form.text = value.to_string(),
        "ForeColor" => {
            form.fore_color = value.parse::<u32>().unwrap_or(0);
        }
        "BackColor" => {
            form.back_color = value.parse::<u32>().unwrap_or(0);
        }
        "Value" => {
            form.value = value.parse::<i32>().unwrap_or(0);
        }
        "Enabled" => {
            form.enabled = value != "0" && value.to_lowercase() != "false";
        }
        _ => {
            form.properties.insert(key.to_string(), value.to_string());
        }
    }
}

#[cfg(test)]
mod tests;
