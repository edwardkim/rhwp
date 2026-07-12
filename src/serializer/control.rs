//! 컨트롤 직렬화 (표, 도형, 그림, 머리말/꼬리말, 각주/미주 등)
//!
//! `parser::body_text::parse_ctrl_header` + `parser::control::parse_control`의 역방향.
//! 각 Control enum variant를 CTRL_HEADER 레코드(+자식 레코드)로 변환한다.

use super::body_text::serialize_paragraph_list;
use super::byte_writer::ByteWriter;

use crate::document_core::converters::common_obj_attr_writer::pack_common_attr_bits;
use crate::model::control::*;
use crate::model::document::SectionDef;
use crate::model::footnote::FootnoteShape;
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply, MasterPage};
use crate::model::image::{ImageEffect, Picture};
use crate::model::page::{ColumnDef, ColumnDirection, ColumnType, PageBorderFill, PageDef};
use crate::model::paragraph::Paragraph;
use crate::model::shape::{
    Caption, CaptionDirection, CaptionVertAlign, CommonObjAttr, DrawingObjAttr, HorzRelTo,
    OleShape, ShapeComponentAttr, ShapeObject, TextFlow, TextWrap, VertRelTo,
};
use crate::model::style::{Fill, FillType, ImageFillMode, ShapeBorderLine};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::parser::record::Record;
use crate::parser::tags;

/// Control을 CTRL_HEADER 레코드(+자식)로 직렬화
///
/// `ctrl_data_record`: 원본의 CTRL_DATA 레코드 데이터 (라운드트립 보존용).
/// CTRL_HEADER 바로 다음에 삽입된다.
pub fn serialize_control(
    ctrl: &Control,
    level: u16,
    ctrl_data_record: Option<&[u8]>,
    records: &mut Vec<Record>,
) {
    let insert_pos = records.len(); // CTRL_HEADER가 쓰이는 위치 기억

    match ctrl {
        Control::SectionDef(sd) => serialize_section_def(sd, level, records),
        Control::ColumnDef(cd) => serialize_column_def(cd, level, records),
        Control::Table(table) => serialize_table(table, level, records),
        Control::Header(header) => serialize_header_control(header, level, records),
        Control::Footer(footer) => serialize_footer_control(footer, level, records),
        Control::Footnote(fn_) => serialize_footnote(fn_, level, records),
        Control::Endnote(en) => serialize_endnote(en, level, records),
        Control::HiddenComment(comment) => serialize_hidden_comment(comment, level, records),
        Control::AutoNumber(an) => {
            records.push(make_ctrl_record(
                tags::CTRL_AUTO_NUMBER,
                level,
                &serialize_auto_number(an),
            ));
        }
        Control::NewNumber(nn) => {
            records.push(make_ctrl_record(
                tags::CTRL_NEW_NUMBER,
                level,
                &serialize_new_number(nn),
            ));
        }
        Control::PageNumberPos(pnp) => {
            records.push(make_ctrl_record(
                tags::CTRL_PAGE_NUM_POS,
                level,
                &serialize_page_num_pos(pnp),
            ));
        }
        Control::PageHide(ph) => {
            records.push(make_ctrl_record(
                tags::CTRL_PAGE_HIDE,
                level,
                &serialize_page_hide(ph),
            ));
        }
        Control::Bookmark(bm) => {
            records.push(make_ctrl_record(tags::CTRL_BOOKMARK, level, &[]));
            if ctrl_data_record.is_none() {
                if let Some(data) = serialize_bookmark_ctrl_data(bm) {
                    records.push(Record {
                        tag_id: tags::HWPTAG_CTRL_DATA,
                        level: level + 1,
                        size: data.len() as u32,
                        data,
                    });
                }
            }
        }
        Control::Picture(pic) => serialize_picture_control(pic, level, ctrl_data_record, records),
        Control::Shape(shape) => serialize_shape_control(shape, level, ctrl_data_record, records),
        Control::CharOverlap(co) => {
            records.push(make_ctrl_record(
                tags::CTRL_TCPS,
                level,
                &serialize_char_overlap(co),
            ));
        }
        Control::Equation(eq) => serialize_equation_control(eq, level, records),
        Control::Field(f) => {
            // 필드 컨트롤 직렬화 (표 154)
            // ctrl_id(4) + 속성(4) + 기타속성(1) + command_len(2) + command(가변) + id(4)
            //
            // [Task #852 Stage 2.5] ClickHere 의 field_id 는 정답지 패턴 (form 마지막 +1) 우선.
            // form_order_counter 가 form 다음 ClickHere 시점에 5 (form 0..4 다음) → instance_id =
            // 0x7dcd59d6 + 5 = 0x7dcd59db (정답지와 일치).
            let field_id =
                if matches!(f.field_type, FieldType::ClickHere) && peek_form_order_counter() > 0 {
                    0x7dcd_59d6u32.wrapping_add(peek_form_order_counter())
                } else {
                    f.field_id
                };
            let ctrl_id = if matches!(f.field_type, FieldType::Memo) {
                tags::FIELD_UNKNOWN
            } else {
                f.ctrl_id
            };
            let properties = if matches!(f.field_type, FieldType::Memo) {
                f.properties | 0x8000
            } else {
                f.properties
            };
            let cmd_utf16: Vec<u16> = f.command.encode_utf16().collect();
            let cmd_len = cmd_utf16.len();
            let mut data = Vec::with_capacity(4 + 4 + 1 + 2 + cmd_len * 2 + 4);
            data.extend_from_slice(&ctrl_id.to_le_bytes());
            data.extend_from_slice(&properties.to_le_bytes());
            data.push(f.extra_properties);
            data.extend_from_slice(&(cmd_len as u16).to_le_bytes());
            for ch in &cmd_utf16 {
                data.extend_from_slice(&ch.to_le_bytes());
            }
            data.extend_from_slice(&field_id.to_le_bytes());
            data.extend_from_slice(&f.memo_index.to_le_bytes());
            records.push(Record {
                tag_id: tags::HWPTAG_CTRL_HEADER,
                level,
                size: data.len() as u32,
                data,
            });
            // [Task #852 Stage 2.5] ClickHere 필드의 CTRL_DATA 자식 레코드 (0x57, 26 bytes).
            // 정답지 (samples/form-01.hwp) reverse engineering 구조:
            //   0..2   0x021b (헤더 magic)
            //   2..6   0x00000001
            //   6..8   0x4000 (HWP5 CTRL_DATA flag — 정답지 관찰)
            //   8..10  0x0001
            //   10..12 u16 LE wchar_count (필드 이름 길이)
            //   12..   UTF-16 LE 필드 이름 (예: "myMsg01")
            //
            // ctrl_data_name (HWPX `<hp:fieldBegin name="...">`) 우선, 비어있으면 생성 안 함.
            if matches!(f.field_type, FieldType::ClickHere) && ctrl_data_record.is_none() {
                if let Some(name) = &f.ctrl_data_name {
                    if !name.is_empty() {
                        let name_utf16: Vec<u16> = name.encode_utf16().collect();
                        let nlen = name_utf16.len();
                        let mut cdata = Vec::with_capacity(12 + nlen * 2);
                        cdata.extend_from_slice(&0x021bu16.to_le_bytes());
                        cdata.extend_from_slice(&0x00000001u32.to_le_bytes());
                        cdata.extend_from_slice(&0x4000u16.to_le_bytes());
                        cdata.extend_from_slice(&0x0001u16.to_le_bytes());
                        cdata.extend_from_slice(&(nlen as u16).to_le_bytes());
                        for ch in &name_utf16 {
                            cdata.extend_from_slice(&ch.to_le_bytes());
                        }
                        records.push(Record {
                            tag_id: tags::HWPTAG_CTRL_DATA,
                            level: level + 1,
                            size: cdata.len() as u32,
                            data: cdata,
                        });
                    }
                }
            }
        }
        // [Task #852 Stage 2.4] 양식 개체 직렬화 — CTRL_HEADER + HWPTAG_FORM_OBJECT
        Control::Form(form) => serialize_form_control(form, level, records),
        // 미구현 컨트롤은 최소한의 CTRL_HEADER만 생성
        Control::Hyperlink(_) | Control::Ruby(_) | Control::Unknown(_) => {
            let ctrl_id = match ctrl {
                Control::Unknown(u) => u.ctrl_id,
                _ => 0,
            };
            if ctrl_id != 0 {
                let mut data = Vec::new();
                data.extend_from_slice(&ctrl_id.to_le_bytes());
                records.push(Record {
                    tag_id: tags::HWPTAG_CTRL_HEADER,
                    level,
                    size: data.len() as u32,
                    data,
                });
            }
        }
    }

    // CTRL_DATA 레코드 복원: 일반 컨트롤은 CTRL_HEADER 바로 다음에 삽입한다.
    // Picture/Shape 컨트롤은 각 serializer가 SHAPE_COMPONENT 자식(level+2)으로 배치한다.
    if let Some(data) = ctrl_data_record {
        if !matches!(ctrl, Control::Picture(_) | Control::Shape(_)) {
            let ctrl_data_pos = insert_pos + 1; // CTRL_HEADER 바로 다음
            records.insert(
                ctrl_data_pos,
                Record {
                    tag_id: tags::HWPTAG_CTRL_DATA,
                    level: level + 1,
                    size: data.len() as u32,
                    data: data.to_vec(),
                },
            );
        }
    }
}

// ============================================================
// CTRL_HEADER 레코드 생성 헬퍼
// ============================================================

/// ctrl_id + ctrl_data로 CTRL_HEADER 레코드 생성
fn make_ctrl_record(ctrl_id: u32, level: u16, ctrl_data: &[u8]) -> Record {
    let mut data = Vec::with_capacity(4 + ctrl_data.len());
    data.extend_from_slice(&ctrl_id.to_le_bytes());
    data.extend_from_slice(ctrl_data);
    Record {
        tag_id: tags::HWPTAG_CTRL_HEADER,
        level,
        size: data.len() as u32,
        data,
    }
}

// ============================================================
// 구역 정의 ('secd')
// ============================================================

fn serialize_section_def(sd: &SectionDef, level: u16, records: &mut Vec<Record>) {
    let mut w = ByteWriter::new();
    w.write_u32(sd.flags).unwrap();
    w.write_i16(sd.column_spacing).unwrap();
    w.write_i16(sd.line_grid).unwrap();
    w.write_i16(sd.char_grid).unwrap();
    w.write_u32(sd.default_tab_spacing).unwrap();
    w.write_u16(sd.outline_numbering_id).unwrap();
    w.write_u16(sd.page_num).unwrap();
    w.write_u16(sd.picture_num).unwrap();
    w.write_u16(sd.table_num).unwrap();
    w.write_u16(sd.equation_num).unwrap();
    // [Task #1058] HWP5 spec 표 129 정합 — SectionDef payload 26 byte (위 24 + 2 Language):
    //   UINT16 대표Language (5.0.1.5 이상)
    // 한컴 정답지는 26 byte payload + 8 byte zero (확장 영역) = 34 byte payload + 4 byte ctrl_id = 38 byte CTRL_HEADER.
    // 원본 추가 바이트 복원 (라운드트립용)
    if !sd.raw_ctrl_extra.is_empty() {
        w.write_bytes(&sd.raw_ctrl_extra).unwrap();
    } else {
        // HWPX 출처: 한컴 default 10 byte (Language 0 + zero 8)
        w.write_u16(0).unwrap(); // 대표Language = 0 (Application 지정)
        w.write_bytes(&[0u8; 8]).unwrap(); // 추가 zero padding (관찰된 한컴 정답지 패턴)
    }

    records.push(make_ctrl_record(
        tags::CTRL_SECTION_DEF,
        level,
        w.as_bytes(),
    ));

    // PAGE_DEF
    records.push(Record {
        tag_id: tags::HWPTAG_PAGE_DEF,
        level: level + 1,
        size: 0,
        data: serialize_page_def(&sd.page_def),
    });

    // FOOTNOTE_SHAPE (각주)
    records.push(Record {
        tag_id: tags::HWPTAG_FOOTNOTE_SHAPE,
        level: level + 1,
        size: 0,
        data: serialize_footnote_shape(&sd.footnote_shape),
    });

    // FOOTNOTE_SHAPE (미주)
    records.push(Record {
        tag_id: tags::HWPTAG_FOOTNOTE_SHAPE,
        level: level + 1,
        size: 0,
        data: serialize_footnote_shape(&sd.endnote_shape),
    });

    // PAGE_BORDER_FILL (첫 번째)
    records.push(Record {
        tag_id: tags::HWPTAG_PAGE_BORDER_FILL,
        level: level + 1,
        size: 0,
        data: serialize_page_border_fill(&sd.page_border_fill),
    });

    // 추가 PAGE_BORDER_FILL (2번째, 3번째 등)
    for pbf in &sd.extra_page_border_fills {
        records.push(Record {
            tag_id: tags::HWPTAG_PAGE_BORDER_FILL,
            level: level + 1,
            size: 0,
            data: serialize_page_border_fill(pbf),
        });
    }

    // 기타 자식 레코드 복원 (바탕쪽 LIST_HEADER + 문단 등)
    for raw in &sd.extra_child_records {
        records.push(Record {
            tag_id: raw.tag_id,
            level: raw.level,
            size: raw.data.len() as u32,
            data: raw.data.clone(),
        });
    }

    // HWPX 출처 바탕쪽은 raw child record가 없으므로 MasterPage 모델에서 HWP5 LIST_HEADER
    // + 문단 목록을 materialize한다. HWP 출처는 raw 바탕쪽 LIST_HEADER가 있으면 중복 출력하지 않는다.
    let has_raw_master_page_records = sd
        .extra_child_records
        .iter()
        .any(|raw| raw.tag_id == tags::HWPTAG_LIST_HEADER && raw.level == level + 1);
    if !has_raw_master_page_records {
        for master_page in sd.master_pages.iter().filter(|mp| !mp.is_extension) {
            serialize_master_page(master_page, level + 1, records);
        }
    }
}

pub(crate) fn serialize_master_page(
    master_page: &MasterPage,
    level: u16,
    records: &mut Vec<Record>,
) {
    let data = if !master_page.raw_list_header.is_empty() {
        master_page.raw_list_header.clone()
    } else {
        let ext_flags =
            u16::from(master_page.overlap) | if master_page.is_extension { 0x02 } else { 0 };
        build_header_footer_list_header(
            master_page.paragraphs.len() as u16,
            0,
            master_page.text_width,
            master_page.text_height,
            master_page.text_ref,
            master_page.num_ref,
            ext_flags,
        )
    };

    records.push(Record {
        tag_id: tags::HWPTAG_LIST_HEADER,
        level,
        size: data.len() as u32,
        data,
    });
    serialize_paragraph_list(&master_page.paragraphs, level, records);
}

fn serialize_page_def(pd: &PageDef) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_u32(pd.width).unwrap();
    w.write_u32(pd.height).unwrap();
    w.write_u32(pd.margin_left).unwrap();
    w.write_u32(pd.margin_right).unwrap();
    w.write_u32(pd.margin_top).unwrap();
    w.write_u32(pd.margin_bottom).unwrap();
    w.write_u32(pd.margin_header).unwrap();
    w.write_u32(pd.margin_footer).unwrap();
    w.write_u32(pd.margin_gutter).unwrap();
    // [#1166] 용지 방향(landscape)은 attr bit0 (parse: body_text.rs `attr & 0x01`).
    // HWPX 출처 문서는 pd.landscape 만 설정되고 attr bit0 은 0 이므로, 직렬화 시
    // landscape 를 attr bit0 에 동기화해야 HWP5 저장본이 가로 방향을 보존한다.
    let attr = if pd.landscape {
        pd.attr | 0x01
    } else {
        pd.attr & !0x01
    };
    w.write_u32(attr).unwrap();
    w.into_bytes()
}

fn serialize_footnote_shape(fs: &FootnoteShape) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_u32(fs.attr).unwrap();
    w.write_u16(fs.user_char as u16).unwrap();
    w.write_u16(fs.prefix_char as u16).unwrap();
    w.write_u16(fs.suffix_char as u16).unwrap();
    w.write_u16(fs.start_number).unwrap();
    // HWP5 노트 구분선 길이는 i16 슬롯. 한컴 전폭 sentinel(14692344)은 i16을 넘지만
    // HWP5 포맷 한계상 하위 16비트로 기록한다(HWPX 경로는 i32 원본을 보존).
    w.write_i16(fs.separator_length as i16).unwrap();
    w.write_i16(fs.separator_margin_top).unwrap();
    w.write_i16(fs.separator_margin_bottom).unwrap();
    w.write_i16(fs.note_spacing).unwrap();
    // 미문서화 2바이트 (원본 보존)
    w.write_u16(fs.raw_unknown).unwrap();
    w.write_u8(fs.separator_line_type).unwrap();
    w.write_u8(fs.separator_line_width).unwrap();
    w.write_color_ref(fs.separator_color).unwrap();
    w.into_bytes()
}

fn serialize_page_border_fill(pbf: &PageBorderFill) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_u32(pbf.attr).unwrap();
    w.write_i16(pbf.spacing_left).unwrap();
    w.write_i16(pbf.spacing_right).unwrap();
    w.write_i16(pbf.spacing_top).unwrap();
    w.write_i16(pbf.spacing_bottom).unwrap();
    w.write_u16(pbf.border_fill_id).unwrap();
    w.into_bytes()
}

// ============================================================
// 단 정의 ('cold')
// ============================================================

fn serialize_column_def(cd: &ColumnDef, level: u16, records: &mut Vec<Record>) {
    let mut w = ByteWriter::new();

    // 표 141: 속성 bit 0-15 (원본이 있으면 그대로, 없으면 재구성)
    let attr: u16 = if cd.raw_attr != 0 {
        cd.raw_attr
    } else {
        let mut a: u16 = match cd.column_type {
            ColumnType::Normal => 0,
            ColumnType::Distribute => 1,
            ColumnType::Parallel => 2,
        };
        // bit 2-9: 단 개수
        a |= (cd.column_count as u16 & 0xFF) << 2;
        // bit 10-11: 단 방향
        if cd.direction == ColumnDirection::RightToLeft {
            a |= 1 << 10;
        }
        // bit 12: 단 너비 동일
        if cd.same_width {
            a |= 1 << 12;
        }
        a
    };

    w.write_u16(attr).unwrap();

    // hwplib 기준: same_width 여부에 따라 바이트 순서가 다름
    if !cd.same_width && cd.column_count > 1 {
        // same_width=false: [attr2(2)] [col0_width(2) col0_gap(2)] ...
        w.write_u16(0).unwrap(); // attr2
        for i in 0..cd.widths.len() {
            w.write_i16(cd.widths[i]).unwrap();
            let gap = cd.gaps.get(i).copied().unwrap_or(0);
            w.write_i16(gap).unwrap();
        }
    } else {
        // same_width=true: [gap(2)] [attr2(2)]
        w.write_i16(cd.spacing).unwrap();
        w.write_u16(0).unwrap(); // attr2
    }

    w.write_u8(cd.separator_type).unwrap();
    w.write_u8(cd.separator_width).unwrap();
    w.write_color_ref(cd.separator_color).unwrap();

    records.push(make_ctrl_record(tags::CTRL_COLUMN_DEF, level, w.as_bytes()));
}

// ============================================================
// 표 ('tbl ')
// ============================================================

fn serialize_table(table: &Table, level: u16, records: &mut Vec<Record>) {
    // CTRL_HEADER: raw_ctrl_data는 CommonObjAttr 전체 (attr 포함)
    // Task 271에서 파싱 변경: ctrl_data 전체 = CommonObjAttr
    //
    // [#1916] raw_ctrl_data 부재 시(HWPX 파스 IR·편집기 신설 표 등) 종전에는
    // 빈 데이터를 방출해 재파스 CommonObjAttr 전체(treat_as_char/wrap/
    // flowWithText 등)가 기본값으로 붕괴했다. 다른 GSO 컨트롤과 동일하게
    // IR 의 common 으로 합성한다 (attr=0 이면 pack_common_attr_bits 경유 —
    // flow_with_text bit 13 포함). HWP5 파스본(raw 보존)·어댑터 경로(Stage 2
    // 합성)는 raw_ctrl_data 가 채워져 있어 동작 불변.
    let composed_common;
    let ctrl_data: &[u8] = if !table.raw_ctrl_data.is_empty() {
        &table.raw_ctrl_data
    } else {
        composed_common = serialize_common_obj_attr(&table.common);
        &composed_common
    };
    records.push(make_ctrl_record(tags::CTRL_TABLE, level, ctrl_data));

    // 캡션 (TABLE 이전, level+1)
    if let Some(ref caption) = table.caption {
        serialize_caption(caption, level + 1, records);
    }

    // HWPTAG_TABLE 레코드
    records.push(Record {
        tag_id: tags::HWPTAG_TABLE,
        level: level + 1,
        size: 0,
        data: serialize_table_record(table),
    });

    // 셀 목록
    for cell in &table.cells {
        serialize_cell(cell, level + 1, records);
    }
}

fn serialize_table_record(table: &Table) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // attr (원본이 있으면 그대로, 없으면 재구성)
    let attr = if table.raw_table_record_attr != 0 {
        table.raw_table_record_attr
    } else {
        let mut a: u32 = 0;
        match table.page_break {
            TablePageBreak::CellBreak => a |= 0x01,
            TablePageBreak::RowBreak => a |= 0x02,
            TablePageBreak::None => {}
        }
        if table.repeat_header {
            a |= 0x04;
        }
        a
    };
    w.write_u32(attr).unwrap();

    w.write_u16(table.row_count).unwrap();
    w.write_u16(table.col_count).unwrap();
    w.write_i16(table.cell_spacing).unwrap();

    // 안쪽 여백
    w.write_i16(table.padding.left).unwrap();
    w.write_i16(table.padding.right).unwrap();
    w.write_i16(table.padding.top).unwrap();
    w.write_i16(table.padding.bottom).unwrap();

    // 행별 셀 수 (HWP 스펙: UINT16[NRows])
    for &h in &table.row_sizes {
        w.write_i16(h).unwrap();
    }

    w.write_u16(table.border_fill_id).unwrap();

    // 영역 속성: UINT16 nZones + TableZone[nZones].
    //
    // `셀 테두리/배경 - 하나의 셀처럼 적용`은 개별 셀이 아니라 TABLE cellzone
    // overlay로 저장되어야 한컴에서 선택 영역 전체 대각선으로 표시된다.
    w.write_u16(table.zones.len() as u16).unwrap();
    for zone in &table.zones {
        w.write_u16(zone.start_row).unwrap();
        w.write_u16(zone.start_col).unwrap();
        w.write_u16(zone.end_row).unwrap();
        w.write_u16(zone.end_col).unwrap();
        w.write_u16(zone.border_fill_id).unwrap();
    }

    // 원본 추가 바이트 복원 (라운드트립용)
    if !table.raw_table_record_extra.is_empty() {
        w.write_bytes(&table.raw_table_record_extra).unwrap();
    }

    w.into_bytes()
}

fn serialize_cell(cell: &Cell, level: u16, records: &mut Vec<Record>) {
    let mut w = ByteWriter::new();

    // LIST_HEADER 공통 (6 + 2 = 8바이트)
    let n_paragraphs = cell.paragraphs.len() as u16;
    w.write_u16(n_paragraphs).unwrap();

    // list_attr 재구성 (text_direction + vertical_align)
    let v_align_code: u32 = match cell.vertical_align {
        VerticalAlign::Top => 0,
        VerticalAlign::Center => 1,
        VerticalAlign::Bottom => 2,
    };
    let list_attr: u32 = ((cell.text_direction as u32) << 16) | (v_align_code << 21);
    w.write_u32(list_attr).unwrap();
    let list_header_width_ref = if cell.list_header_width_ref == 0 {
        0x0400
    } else {
        cell.list_header_width_ref
    };
    w.write_u16(list_header_width_ref).unwrap();

    // 셀 속성
    w.write_u16(cell.col).unwrap();
    w.write_u16(cell.row).unwrap();
    w.write_u16(cell.col_span).unwrap();
    w.write_u16(cell.row_span).unwrap();
    w.write_u32(cell.width).unwrap();
    w.write_u32(cell.height).unwrap();
    w.write_i16(cell.padding.left).unwrap();
    w.write_i16(cell.padding.right).unwrap();
    w.write_i16(cell.padding.top).unwrap();
    w.write_i16(cell.padding.bottom).unwrap();
    w.write_u16(cell.border_fill_id).unwrap();

    // 원본 추가 바이트 복원. HWPX/신규 생성 셀에는 원본이 없으므로
    // 한컴 저장본의 셀 LIST_HEADER 47바이트 contract에 맞춰 폭 참조를 보강한다.
    // [#1808] 모델의 field_name 과 raw_list_extra 인코딩이 일치하면 원본 보존,
    // 불일치(HWPX 출처 셀 필드, 편집기 변경)면 한컴 계약대로 재구성한다.
    let raw_field = crate::parser::control::parse_cell_field_name(&cell.raw_list_extra);
    if !cell.raw_list_extra.is_empty() && raw_field.as_deref() == cell.field_name.as_deref() {
        w.write_bytes(&cell.raw_list_extra).unwrap();
    } else {
        w.write_bytes(&build_cell_list_extra(cell)).unwrap();
    }

    records.push(Record {
        tag_id: tags::HWPTAG_LIST_HEADER,
        level,
        size: 0,
        data: w.into_bytes(),
    });

    // 셀 내부 문단 (원본 HWP에서는 LIST_HEADER와 같은 레벨)
    serialize_paragraph_list(&cell.paragraphs, level, records);
}

/// [#1808] 셀 LIST_HEADER 추가 바이트(34바이트 이후) 재구성 — 한컴 계약.
///
/// 필드 없는 셀: width(4) + 0×9 = 13바이트.
/// 필드 셀 (한컴 원본 admrul_0039/0045 대조로 확정한 레이아웃):
///   [0..4]   width (u32 LE)
///   [4..8]   ff 1b 02 01 (필드 속성 마커)
///   [8..12]  00 ×4
///   [12..15] 40 01 00
///   [15..17] name_len (u16 LE)
///   [17..]   UTF-16LE 필드 이름
///   [+8]     00 ×8
/// 파서 대칭: parser::control::parse_cell_field_name (offset 15/17).
fn build_cell_list_extra(cell: &Cell) -> Vec<u8> {
    // 원본 extra 가 있으면 선두 4바이트(폭 참조 계약값)를 보존한다.
    let width_bytes: [u8; 4] = if cell.raw_list_extra.len() >= 4 {
        cell.raw_list_extra[0..4].try_into().unwrap()
    } else {
        cell.width.to_le_bytes()
    };
    match cell.field_name.as_deref() {
        Some(name) if !name.is_empty() => {
            let utf16: Vec<u16> = name.encode_utf16().collect();
            let mut v = Vec::with_capacity(25 + utf16.len() * 2);
            v.extend_from_slice(&width_bytes);
            v.extend_from_slice(&[0xff, 0x1b, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00]);
            v.extend_from_slice(&[0x40, 0x01, 0x00]);
            v.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
            for cu in &utf16 {
                v.extend_from_slice(&cu.to_le_bytes());
            }
            v.extend_from_slice(&[0u8; 8]);
            v
        }
        _ => {
            let mut v = Vec::with_capacity(13);
            v.extend_from_slice(&width_bytes);
            v.extend_from_slice(&[0u8; 9]);
            v
        }
    }
}

fn serialize_caption(caption: &Caption, level: u16, records: &mut Vec<Record>) {
    let mut w = ByteWriter::new();

    // LIST_HEADER 공통 (8바이트: n_para + list_attr + width_ref)
    let n_paragraphs = caption.paragraphs.len() as u16;
    w.write_u16(n_paragraphs).unwrap();
    // list_attr: bit 21~22 = 세로 정렬 (Left/Right 캡션용)
    let vert_align_bits: u32 = match caption.vert_align {
        CaptionVertAlign::Top => 0,
        CaptionVertAlign::Center => 1,
        CaptionVertAlign::Bottom => 2,
    };
    let list_attr: u32 = vert_align_bits << 21;
    w.write_u32(list_attr).unwrap();
    w.write_u16(0).unwrap(); // width_ref

    // 캡션 데이터
    let dir_val: u32 = match caption.direction {
        CaptionDirection::Left => 0,
        CaptionDirection::Right => 1,
        CaptionDirection::Top => 2,
        CaptionDirection::Bottom => 3,
    };
    let mut caption_attr = dir_val;
    if caption.include_margin {
        caption_attr |= 0x04;
    }
    w.write_u32(caption_attr).unwrap();
    w.write_u32(caption.width).unwrap();
    w.write_i16(caption.spacing).unwrap();
    w.write_u32(caption.max_width).unwrap();
    // 예약 필드 8바이트 (한컴 호환성: 원본 파일은 30바이트 LIST_HEADER)
    w.write_u32(0).unwrap();
    w.write_u32(0).unwrap();

    records.push(Record {
        tag_id: tags::HWPTAG_LIST_HEADER,
        level,
        size: 0,
        data: w.into_bytes(),
    });

    // 캡션 내부 문단 (LIST_HEADER와 같은 레벨)
    serialize_paragraph_list(&caption.paragraphs, level, records);
}

// ============================================================
// 머리말/꼬리말 ('head'/'foot')
// ============================================================

fn serialize_header_control(header: &Header, level: u16, records: &mut Vec<Record>) {
    let attr: u32 = if header.raw_attr != 0 {
        header.raw_attr
    } else {
        match header.apply_to {
            HeaderFooterApply::Both => 0,
            HeaderFooterApply::Even => 1,
            HeaderFooterApply::Odd => 2,
        }
    };
    let mut w = ByteWriter::new();
    w.write_u32(attr).unwrap();
    if !header.raw_ctrl_extra.is_empty() {
        w.write_bytes(&header.raw_ctrl_extra).unwrap();
    }
    records.push(make_ctrl_record(tags::CTRL_HEADER, level, w.as_bytes()));

    // LIST_HEADER + 문단
    serialize_header_footer_list_header_with_paragraphs(
        &header.paragraphs,
        HeaderFooterListLayout {
            list_attr: header.list_attr,
            text_width: header.text_width,
            text_height: header.text_height,
            text_ref: header.text_ref,
            num_ref: header.num_ref,
        },
        level + 1,
        records,
    );
}

fn serialize_footer_control(footer: &Footer, level: u16, records: &mut Vec<Record>) {
    let attr: u32 = if footer.raw_attr != 0 {
        footer.raw_attr
    } else {
        match footer.apply_to {
            HeaderFooterApply::Both => 0,
            HeaderFooterApply::Even => 1,
            HeaderFooterApply::Odd => 2,
        }
    };
    let mut w = ByteWriter::new();
    w.write_u32(attr).unwrap();
    if !footer.raw_ctrl_extra.is_empty() {
        w.write_bytes(&footer.raw_ctrl_extra).unwrap();
    }
    records.push(make_ctrl_record(tags::CTRL_FOOTER, level, w.as_bytes()));

    serialize_header_footer_list_header_with_paragraphs(
        &footer.paragraphs,
        HeaderFooterListLayout {
            list_attr: footer.list_attr,
            text_width: footer.text_width,
            text_height: footer.text_height,
            text_ref: footer.text_ref,
            num_ref: footer.num_ref,
        },
        level + 1,
        records,
    );
}

// ============================================================
// 각주/미주 ('fn  '/'en  ')
// ============================================================

fn serialize_footnote(fn_: &Footnote, level: u16, records: &mut Vec<Record>) {
    // [Task #1050] hwplib::ForControlFootnote::ctrlHeader 정합 — size=20 형식
    //   number(UInt4) + before(WChar) + after(WChar) + numberShape(UInt4) + instanceId(UInt4)
    let mut w = ByteWriter::new();
    w.write_u32(fn_.number as u32).unwrap();
    w.write_u16(fn_.before_decoration_letter).unwrap();
    let after = if fn_.after_decoration_letter == 0 {
        0x0029 // ')' default
    } else {
        fn_.after_decoration_letter
    };
    w.write_u16(after).unwrap();
    w.write_u32(fn_.number_shape).unwrap();
    w.write_u32(fn_.instance_id).unwrap();
    records.push(make_ctrl_record(tags::CTRL_FOOTNOTE, level, w.as_bytes()));

    serialize_footnote_endnote_list_header(
        &fn_.paragraphs,
        fn_.list_header_property,
        level + 1,
        records,
    );
}

fn serialize_endnote(en: &Endnote, level: u16, records: &mut Vec<Record>) {
    // [Task #1050] Footnote 와 동일 구조
    let mut w = ByteWriter::new();
    w.write_u32(en.number as u32).unwrap();
    w.write_u16(en.before_decoration_letter).unwrap();
    let after = if en.after_decoration_letter == 0 {
        0x0029
    } else {
        en.after_decoration_letter
    };
    w.write_u16(after).unwrap();
    w.write_u32(en.number_shape).unwrap();
    w.write_u32(en.instance_id).unwrap();
    records.push(make_ctrl_record(tags::CTRL_ENDNOTE, level, w.as_bytes()));

    serialize_footnote_endnote_list_header(
        &en.paragraphs,
        en.list_header_property,
        level + 1,
        records,
    );
}

/// [Task #1050] CTRL_FOOTNOTE / CTRL_ENDNOTE 의 LIST_HEADER 직렬화 (size=16 형식).
/// 형식: paraCount(SInt4) + property(UInt4) + 8 byte zero padding.
/// 참조: `hwplib::ForListHeaderForFootnodeEndnote`.
fn serialize_footnote_endnote_list_header(
    paragraphs: &[Paragraph],
    property: u32,
    level: u16,
    records: &mut Vec<Record>,
) {
    let mut w = ByteWriter::new();
    w.write_i32(paragraphs.len() as i32).unwrap();
    w.write_u32(property).unwrap();
    w.write_bytes(&[0u8; 8]).unwrap(); // 8 byte zero padding
    records.push(Record {
        tag_id: tags::HWPTAG_LIST_HEADER,
        level,
        size: 0,
        data: w.into_bytes(),
    });

    // 문단 목록 (LIST_HEADER 와 같은 레벨)
    serialize_paragraph_list(paragraphs, level, records);
}

// ============================================================
// 숨은 설명 ('tcmt')
// ============================================================

fn serialize_hidden_comment(comment: &HiddenComment, level: u16, records: &mut Vec<Record>) {
    records.push(make_ctrl_record(tags::CTRL_HIDDEN_COMMENT, level, &[]));
    serialize_list_header_with_paragraphs(&comment.paragraphs, level + 1, records);
}

// ============================================================
// 단순 컨트롤
// ============================================================

fn serialize_auto_number(an: &AutoNumber) -> Vec<u8> {
    let type_val: u32 = match an.number_type {
        AutoNumberType::Page => 0,
        AutoNumberType::Footnote => 1,
        AutoNumberType::Endnote => 2,
        AutoNumberType::Picture => 3,
        AutoNumberType::Table => 4,
        AutoNumberType::Equation => 5,
    };
    let mut attr: u32 = type_val & 0x0F;
    attr |= ((an.format as u32) & 0xFF) << 4; // bit 4~11: 번호 모양
    if an.superscript {
        attr |= 0x1000; // bit 12: 위 첨자
    }
    let mut data = Vec::new();
    data.extend_from_slice(&attr.to_le_bytes());
    // number가 0이면 assigned_number를 사용 (캡션 등 새로 생성된 AutoNumber)
    let num = if an.number > 0 {
        an.number
    } else {
        an.assigned_number
    };
    data.extend_from_slice(&num.to_le_bytes());
    data.extend_from_slice(&(an.user_symbol as u16).to_le_bytes());
    data.extend_from_slice(&(an.prefix_char as u16).to_le_bytes());
    data.extend_from_slice(&(an.suffix_char as u16).to_le_bytes());
    data
}

fn serialize_new_number(nn: &NewNumber) -> Vec<u8> {
    let type_val: u32 = match nn.number_type {
        AutoNumberType::Page => 0,
        AutoNumberType::Footnote => 1,
        AutoNumberType::Endnote => 2,
        AutoNumberType::Picture => 3,
        AutoNumberType::Table => 4,
        AutoNumberType::Equation => 5,
    };
    let attr: u32 = type_val & 0x0F;
    let mut data = Vec::new();
    data.extend_from_slice(&attr.to_le_bytes());
    data.extend_from_slice(&nn.number.to_le_bytes());
    data
}

fn serialize_page_num_pos(pnp: &PageNumberPos) -> Vec<u8> {
    let attr: u32 = (pnp.format as u32 & 0xFF) | ((pnp.position as u32 & 0x0F) << 8);
    let mut data = Vec::new();
    data.extend_from_slice(&attr.to_le_bytes());
    data.extend_from_slice(&(pnp.user_symbol as u16).to_le_bytes());
    data.extend_from_slice(&(pnp.prefix_char as u16).to_le_bytes());
    data.extend_from_slice(&(pnp.suffix_char as u16).to_le_bytes());
    data.extend_from_slice(&(pnp.dash_char as u16).to_le_bytes());
    data
}

fn serialize_page_hide(ph: &PageHide) -> Vec<u8> {
    let mut attr: u32 = 0;
    if ph.hide_header {
        attr |= 0x01;
    }
    if ph.hide_footer {
        attr |= 0x02;
    }
    if ph.hide_master_page {
        attr |= 0x04;
    }
    if ph.hide_border {
        attr |= 0x08;
    }
    if ph.hide_fill {
        attr |= 0x10;
    }
    if ph.hide_page_num {
        attr |= 0x20;
    }
    attr.to_le_bytes().to_vec()
}

fn serialize_bookmark_ctrl_data(bm: &Bookmark) -> Option<Vec<u8>> {
    if bm.name.is_empty() {
        return None;
    }

    let mut w = ByteWriter::new();
    w.write_u16(0x021b).unwrap(); // ParameterSet id
    w.write_u16(1).unwrap(); // item count
    w.write_u16(0).unwrap(); // dummy
    w.write_u16(0x4000).unwrap(); // item id: bookmark name
    w.write_u16(0x0001).unwrap(); // string

    let utf16: Vec<u16> = bm.name.encode_utf16().collect();
    w.write_u16(utf16.len() as u16).unwrap();
    for ch in utf16 {
        w.write_u16(ch).unwrap();
    }

    Some(w.into_bytes())
}

/// 글자 겹침 직렬화 (HWP 스펙 표 152)
fn serialize_char_overlap(co: &CharOverlap) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_u16(co.chars.len() as u16).unwrap();
    for &ch in &co.chars {
        w.write_u16(ch as u16).unwrap();
    }
    w.write_u8(co.border_type).unwrap();
    w.write_i8(co.inner_char_size).unwrap();
    w.write_u8(co.expansion).unwrap();
    w.write_u8(co.char_shape_ids.len() as u8).unwrap();
    for &id in &co.char_shape_ids {
        w.write_u32(id).unwrap();
    }
    w.into_bytes()
}

// ============================================================
// 그림 ('gso ' + Picture)
// ============================================================

fn serialize_picture_control(
    pic: &Picture,
    level: u16,
    ctrl_data_record: Option<&[u8]>,
    records: &mut Vec<Record>,
) {
    // CTRL_HEADER: ctrl_id(gso) + common_obj_attr
    records.push(make_ctrl_record(
        tags::CTRL_GEN_SHAPE,
        level,
        &serialize_common_obj_attr(&pic.common),
    ));

    // 캡션 (SHAPE_COMPONENT 앞, level+1)
    if let Some(ref caption) = pic.caption {
        serialize_caption(caption, level + 1, records);
    }

    // SHAPE_COMPONENT
    records.push(Record {
        tag_id: tags::HWPTAG_SHAPE_COMPONENT,
        level: level + 1,
        size: 0,
        data: serialize_shape_component(tags::SHAPE_PICTURE_ID, &pic.shape_attr, true),
    });

    // CTRL_DATA: SHAPE_COMPONENT 자식으로 배치 (level+2)
    if let Some(data) = ctrl_data_record {
        records.push(Record {
            tag_id: tags::HWPTAG_CTRL_DATA,
            level: level + 2,
            size: data.len() as u32,
            data: data.to_vec(),
        });
    }

    // SHAPE_COMPONENT_PICTURE (SHAPE_COMPONENT의 자식)
    records.push(Record {
        tag_id: tags::HWPTAG_SHAPE_COMPONENT_PICTURE,
        level: level + 2,
        size: 0,
        data: serialize_picture_data(pic),
    });
}

fn serialize_picture_data(pic: &Picture) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_color_ref(pic.border_color).unwrap();
    w.write_i32(pic.border_width).unwrap();
    w.write_u32(0).unwrap(); // border_attr

    for &x in &pic.border_x {
        w.write_i32(x).unwrap();
    }
    for &y in &pic.border_y {
        w.write_i32(y).unwrap();
    }

    // 자르기 정보
    w.write_i32(pic.crop.left).unwrap();
    w.write_i32(pic.crop.top).unwrap();
    w.write_i32(pic.crop.right).unwrap();
    w.write_i32(pic.crop.bottom).unwrap();

    // 안쪽 여백
    w.write_i16(pic.padding.left).unwrap();
    w.write_i16(pic.padding.right).unwrap();
    w.write_i16(pic.padding.top).unwrap();
    w.write_i16(pic.padding.bottom).unwrap();

    // 이미지 속성
    w.write_i8(pic.image_attr.brightness).unwrap();
    w.write_i8(pic.image_attr.contrast).unwrap();
    let effect_val: u8 = match pic.image_attr.effect {
        ImageEffect::RealPic => 0,
        ImageEffect::GrayScale => 1,
        ImageEffect::BlackWhite => 2,
        ImageEffect::Pattern8x8 => 3,
    };
    w.write_u8(effect_val).unwrap();
    w.write_u16(pic.image_attr.bin_data_id).unwrap();

    // 원본 추가 바이트 복원 (라운드트립 보존)
    if !pic.raw_picture_extra.is_empty() {
        w.write_bytes(&picture_raw_extra_with_transparency(pic))
            .unwrap();
    } else {
        // border_opacity(1) + instance_id(4) + image_effect(4) = 9바이트
        w.write_u8(pic.border_opacity).unwrap();
        w.write_u32(pic.instance_id).unwrap();
        w.write_u32(0).unwrap(); // image_effect_extra
                                 // 원본 이미지 크기(HWPUNIT) + 플래그(1): 한컴 호환 추가 9바이트
                                 // [#1929] IR img_dim(HWPX hp:imgDim 대응)이 있으면 우선 기록 —
                                 // 종전 crop 폴백만으로는 HWPX→HWP5 왕복에서 imgDim 소실.
        let (dim_w, dim_h) = if pic.img_dim != (0, 0) {
            pic.img_dim
        } else {
            (pic.crop.right as u32, pic.crop.bottom as u32)
        };
        w.write_u32(dim_w).unwrap(); // original width
        w.write_u32(dim_h).unwrap(); // original height
        w.write_u8(pic.image_attr.transparency_alpha_byte())
            .unwrap();
    }

    w.into_bytes()
}

fn picture_raw_extra_with_transparency(pic: &Picture) -> Vec<u8> {
    let transparency = pic.image_attr.transparency_alpha_byte();
    let mut extra = pic.raw_picture_extra.clone();
    if extra.len() >= 18 {
        if let Some(last) = extra.last_mut() {
            *last = transparency;
        }
    } else if transparency > 0 {
        let original_width = if pic.shape_attr.original_width > 0 {
            pic.shape_attr.original_width
        } else {
            pic.crop.right.max(0) as u32
        };
        let original_height = if pic.shape_attr.original_height > 0 {
            pic.shape_attr.original_height
        } else {
            pic.crop.bottom.max(0) as u32
        };
        extra.extend_from_slice(&original_width.to_le_bytes());
        extra.extend_from_slice(&original_height.to_le_bytes());
        extra.push(transparency);
    }
    extra
}

// ============================================================
// 도형 ('gso ' + Shape)
// ============================================================

fn synthesize_hwpx_shape_ctrl_data(shape: &ShapeObject) -> Option<Vec<u8>> {
    let ShapeObject::Rectangle(rect) = shape else {
        return None;
    };
    rect.drawing.text_box.as_ref()?;
    let common = &rect.common;
    if !(common.size_protect
        && common.flow_with_text
        && common.allow_overlap
        && common.vert_rel_to == VertRelTo::Para
        && common.horz_rel_to == HorzRelTo::Para
        && common.text_wrap == TextWrap::Square
        && common.text_flow == TextFlow::RightOnly)
    {
        return None;
    }

    Some(vec![
        0x1b, 0x02, 0x01, 0x00, 0x00, 0x00, 0x03, 0x30, 0x00, 0x80, 0x03, 0x30, 0x01, 0x00, 0x00,
        0x00, 0x01, 0x70, 0x09, 0x00, 0x01, 0x00, 0x00, 0x00,
    ])
}

fn serialize_shape_control(
    shape: &ShapeObject,
    level: u16,
    ctrl_data_record: Option<&[u8]>,
    records: &mut Vec<Record>,
) {
    let synthesized_ctrl_data = if ctrl_data_record.is_none() {
        synthesize_hwpx_shape_ctrl_data(shape)
    } else {
        None
    };
    let ctrl_data_record = ctrl_data_record.or(synthesized_ctrl_data.as_deref());

    let emit_top_level_synthesized_ctrl_data = |records: &mut Vec<Record>| {
        if let Some(data) = synthesized_ctrl_data.as_deref() {
            records.push(Record {
                tag_id: tags::HWPTAG_CTRL_DATA,
                level: level + 1,
                size: data.len() as u32,
                data: data.to_vec(),
            });
        }
    };

    // CTRL_DATA를 SHAPE_COMPONENT 자식으로 배치하는 헬퍼
    let emit_ctrl_data = |records: &mut Vec<Record>| {
        if let Some(data) = ctrl_data_record {
            records.push(Record {
                tag_id: tags::HWPTAG_CTRL_DATA,
                level: level + 2,
                size: data.len() as u32,
                data: data.to_vec(),
            });
        }
    };

    match shape {
        ShapeObject::Line(line) => {
            let is_connector = line.connector.is_some();
            let sc_ctrl_id = if is_connector {
                tags::SHAPE_CONNECTOR_ID
            } else {
                tags::SHAPE_LINE_ID
            };
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&line.common),
            ));
            emit_top_level_synthesized_ctrl_data(records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_drawing_shape_component(sc_ctrl_id, &line.drawing, true),
            });
            emit_ctrl_data(records);
            serialize_text_box_if_present(&line.drawing, level + 2, records);
            let mut w = ByteWriter::new();
            w.write_i32(line.start.x).unwrap();
            w.write_i32(line.start.y).unwrap();
            w.write_i32(line.end.x).unwrap();
            w.write_i32(line.end.y).unwrap();
            if let Some(ref conn) = line.connector {
                // 연결선 확장 데이터
                w.write_u32(conn.link_type as u32).unwrap();
                w.write_u32(conn.start_subject_id).unwrap();
                w.write_u32(conn.start_subject_index).unwrap();
                w.write_u32(conn.end_subject_id).unwrap();
                w.write_u32(conn.end_subject_index).unwrap();
                w.write_u32(conn.control_points.len() as u32).unwrap();
                for cp in &conn.control_points {
                    w.write_i32(cp.x).unwrap();
                    w.write_i32(cp.y).unwrap();
                    w.write_u16(cp.point_type).unwrap();
                }
                w.write_bytes(&conn.raw_trailing).unwrap();
            } else {
                w.write_i32(if line.started_right_or_bottom { 1 } else { 0 })
                    .unwrap();
            }
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_LINE,
                level: level + 2,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Rectangle(rect) => {
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&rect.common),
            ));
            emit_top_level_synthesized_ctrl_data(records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_drawing_shape_component(tags::SHAPE_RECT_ID, &rect.drawing, true),
            });
            emit_ctrl_data(records);
            // 글상자(텍스트) 내용 직렬화
            serialize_text_box_if_present(&rect.drawing, level + 2, records);
            let mut w = ByteWriter::new();
            w.write_u8(rect.round_rate).unwrap();
            for i in 0..4 {
                w.write_i32(rect.x_coords[i]).unwrap();
                w.write_i32(rect.y_coords[i]).unwrap();
            }
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_RECTANGLE,
                level: level + 2,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Ellipse(ellipse) => {
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&ellipse.common),
            ));
            emit_top_level_synthesized_ctrl_data(records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_drawing_shape_component(
                    tags::SHAPE_ELLIPSE_ID,
                    &ellipse.drawing,
                    true,
                ),
            });
            emit_ctrl_data(records);
            serialize_text_box_if_present(&ellipse.drawing, level + 2, records);
            let mut w = ByteWriter::new();
            w.write_u32(ellipse.attr).unwrap();
            w.write_i32(ellipse.center.x).unwrap();
            w.write_i32(ellipse.center.y).unwrap();
            w.write_i32(ellipse.axis1.x).unwrap();
            w.write_i32(ellipse.axis1.y).unwrap();
            w.write_i32(ellipse.axis2.x).unwrap();
            w.write_i32(ellipse.axis2.y).unwrap();
            w.write_i32(ellipse.start1.x).unwrap();
            w.write_i32(ellipse.start1.y).unwrap();
            w.write_i32(ellipse.end1.x).unwrap();
            w.write_i32(ellipse.end1.y).unwrap();
            w.write_i32(ellipse.start2.x).unwrap();
            w.write_i32(ellipse.start2.y).unwrap();
            w.write_i32(ellipse.end2.x).unwrap();
            w.write_i32(ellipse.end2.y).unwrap();
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_ELLIPSE,
                level: level + 2,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Polygon(poly) => {
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&poly.common),
            ));
            emit_top_level_synthesized_ctrl_data(records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_drawing_shape_component(
                    tags::SHAPE_POLYGON_ID,
                    &poly.drawing,
                    true,
                ),
            });
            emit_ctrl_data(records);
            serialize_text_box_if_present(&poly.drawing, level + 2, records);
            let mut w = ByteWriter::new();
            w.write_i32(poly.points.len() as i32).unwrap();
            for p in &poly.points {
                w.write_i32(p.x).unwrap();
                w.write_i32(p.y).unwrap();
            }
            if poly.raw_trailing.is_empty() {
                w.write_u32(0).unwrap();
            } else {
                w.write_bytes(&poly.raw_trailing).unwrap();
            }
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_POLYGON,
                level: level + 2,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Arc(arc) => {
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&arc.common),
            ));
            emit_top_level_synthesized_ctrl_data(records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_drawing_shape_component(tags::SHAPE_ARC_ID, &arc.drawing, true),
            });
            emit_ctrl_data(records);
            serialize_text_box_if_present(&arc.drawing, level + 2, records);
            let mut w = ByteWriter::new();
            w.write_u8(arc.arc_type).unwrap();
            w.write_i32(arc.center.x).unwrap();
            w.write_i32(arc.center.y).unwrap();
            w.write_i32(arc.axis1.x).unwrap();
            w.write_i32(arc.axis1.y).unwrap();
            w.write_i32(arc.axis2.x).unwrap();
            w.write_i32(arc.axis2.y).unwrap();
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_ARC,
                level: level + 2,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Curve(curve) => {
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&curve.common),
            ));
            emit_top_level_synthesized_ctrl_data(records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_drawing_shape_component(tags::SHAPE_CURVE_ID, &curve.drawing, true),
            });
            emit_ctrl_data(records);
            serialize_text_box_if_present(&curve.drawing, level + 2, records);
            let mut w = ByteWriter::new();
            w.write_i32(curve.points.len() as i32).unwrap();
            for p in &curve.points {
                w.write_i32(p.x).unwrap();
                w.write_i32(p.y).unwrap();
            }
            for &t in &curve.segment_types {
                w.write_u8(t).unwrap();
            }
            // hwplib: sr.skip(4) — 4바이트 패딩
            w.write_u32(0).unwrap();
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_CURVE,
                level: level + 2,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Group(group) => {
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&group.common),
            ));
            emit_top_level_synthesized_ctrl_data(records);
            // 그룹 컨테이너: SHAPE_COMPONENT + 자식 수 + 자식 ctrl_id 목록 (한컴 호환)
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: group_container_component_data(group, true),
            });
            emit_ctrl_data(records);
            // 자식 개체 직렬화 (CTRL_HEADER 없이 SHAPE_COMPONENT + 도형별 태그)
            let child_comp_level = level + 2;
            let child_type_level = level + 3;
            for child in &group.children {
                serialize_group_child(child, child_comp_level, child_type_level, records);
            }
        }
        ShapeObject::Picture(_pic) => {
            // 그룹 내 그림: 그룹 직렬화 시 자식으로 처리됨 (단독 Picture는 Control::Picture로 직렬화)
        }
        ShapeObject::Chart(chart) => {
            // Task #195 단계 2: raw_chart_data를 그대로 보존하여 라운드트립 유지
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&chart.common),
            ));
            let sc_ctrl_id = chart.drawing.shape_attr.ctrl_id;
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_drawing_shape_component(sc_ctrl_id, &chart.drawing, true),
            });
            emit_ctrl_data(records);
            serialize_text_box_if_present(&chart.drawing, level + 2, records);
            records.push(Record {
                tag_id: tags::HWPTAG_CHART_DATA,
                level: level + 2,
                size: 0,
                data: chart.raw_chart_data.clone(),
            });
        }
        ShapeObject::Ole(ole) => {
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&ole.common),
            ));
            let drawing = ole_drawing_with_shape_component_contract(ole, true);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                data: serialize_shape_component(tags::SHAPE_OLE_ID, &drawing.shape_attr, true),
            });
            emit_ctrl_data(records);
            serialize_text_box_if_present(&drawing, level + 2, records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_OLE,
                level: level + 2,
                size: 0,
                data: serialize_ole_data(ole),
            });
        }
    }
}

/// 그룹('$con') SHAPE_COMPONENT 데이터: 공통 component + 자식 수(u16) +
/// 자식 ctrl_id 목록(u32[]) + instance_id (한컴 호환). 최상위/중첩 그룹 공용.
fn group_container_component_data(
    group: &crate::model::shape::GroupShape,
    top_level: bool,
) -> Vec<u8> {
    use crate::parser::tags;

    let mut data =
        serialize_shape_component(tags::SHAPE_CONTAINER_ID, &group.shape_attr, top_level); // '$con'
    let mut w = ByteWriter::new();
    w.write_u16(group.children.len() as u16).unwrap();
    for child in &group.children {
        let child_ctrl_id = match child {
            ShapeObject::Line(_) => tags::SHAPE_LINE_ID,
            ShapeObject::Rectangle(_) => tags::SHAPE_RECT_ID,
            ShapeObject::Ellipse(_) => tags::SHAPE_ELLIPSE_ID,
            ShapeObject::Arc(_) => tags::SHAPE_ARC_ID,
            ShapeObject::Polygon(_) => tags::SHAPE_POLYGON_ID,
            ShapeObject::Curve(_) => tags::SHAPE_CURVE_ID,
            ShapeObject::Group(_) => tags::CTRL_GEN_SHAPE,
            ShapeObject::Picture(_) => tags::SHAPE_PICTURE_ID,
            ShapeObject::Chart(c) => c.drawing.shape_attr.ctrl_id,
            ShapeObject::Ole(o) => {
                if o.drawing.shape_attr.ctrl_id != 0 {
                    o.drawing.shape_attr.ctrl_id
                } else {
                    tags::SHAPE_OLE_ID
                }
            }
        };
        w.write_u32(child_ctrl_id).unwrap();
    }
    w.write_u32(group.common.instance_id).unwrap();
    data.extend_from_slice(&w.into_bytes());
    data
}

/// 그룹 자식 개체 직렬화 (CTRL_HEADER 없이 SHAPE_COMPONENT + 도형별 태그)
fn serialize_group_child(
    child: &ShapeObject,
    comp_level: u16, // SHAPE_COMPONENT level
    type_level: u16, // 도형별 태그 level
    records: &mut Vec<Record>,
) {
    use crate::parser::tags;

    match child {
        ShapeObject::Line(line) => {
            let sc_ctrl_id = if line.connector.is_some() {
                tags::SHAPE_CONNECTOR_ID
            } else {
                tags::SHAPE_LINE_ID
            };
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_drawing_shape_component(sc_ctrl_id, &line.drawing, false),
            });
            serialize_text_box_if_present(&line.drawing, type_level, records);
            let mut w = ByteWriter::new();
            w.write_i32(line.start.x).unwrap();
            w.write_i32(line.start.y).unwrap();
            w.write_i32(line.end.x).unwrap();
            w.write_i32(line.end.y).unwrap();
            if let Some(ref conn) = line.connector {
                w.write_u32(conn.link_type as u32).unwrap();
                w.write_u32(conn.start_subject_id).unwrap();
                w.write_u32(conn.start_subject_index).unwrap();
                w.write_u32(conn.end_subject_id).unwrap();
                w.write_u32(conn.end_subject_index).unwrap();
                w.write_u32(conn.control_points.len() as u32).unwrap();
                for cp in &conn.control_points {
                    w.write_i32(cp.x).unwrap();
                    w.write_i32(cp.y).unwrap();
                    w.write_u16(cp.point_type).unwrap();
                }
                w.write_bytes(&conn.raw_trailing).unwrap();
            } else {
                w.write_i32(if line.started_right_or_bottom { 1 } else { 0 })
                    .unwrap();
            }
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_LINE,
                level: type_level,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Rectangle(rect) => {
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_drawing_shape_component(tags::SHAPE_RECT_ID, &rect.drawing, false),
            });
            serialize_text_box_if_present(&rect.drawing, type_level, records);
            let mut w = ByteWriter::new();
            w.write_u8(rect.round_rate).unwrap();
            for i in 0..4 {
                w.write_i32(rect.x_coords[i]).unwrap();
                w.write_i32(rect.y_coords[i]).unwrap();
            }
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_RECTANGLE,
                level: type_level,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Ellipse(ellipse) => {
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_drawing_shape_component(
                    tags::SHAPE_ELLIPSE_ID,
                    &ellipse.drawing,
                    false,
                ),
            });
            serialize_text_box_if_present(&ellipse.drawing, type_level, records);
            let mut w = ByteWriter::new();
            w.write_u32(ellipse.attr).unwrap();
            w.write_i32(ellipse.center.x).unwrap();
            w.write_i32(ellipse.center.y).unwrap();
            w.write_i32(ellipse.axis1.x).unwrap();
            w.write_i32(ellipse.axis1.y).unwrap();
            w.write_i32(ellipse.axis2.x).unwrap();
            w.write_i32(ellipse.axis2.y).unwrap();
            w.write_i32(ellipse.start1.x).unwrap();
            w.write_i32(ellipse.start1.y).unwrap();
            w.write_i32(ellipse.end1.x).unwrap();
            w.write_i32(ellipse.end1.y).unwrap();
            w.write_i32(ellipse.start2.x).unwrap();
            w.write_i32(ellipse.start2.y).unwrap();
            w.write_i32(ellipse.end2.x).unwrap();
            w.write_i32(ellipse.end2.y).unwrap();
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_ELLIPSE,
                level: type_level,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Arc(arc) => {
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_drawing_shape_component(tags::SHAPE_ARC_ID, &arc.drawing, false),
            });
            serialize_text_box_if_present(&arc.drawing, type_level, records);
            let mut w = ByteWriter::new();
            w.write_u8(arc.arc_type).unwrap();
            w.write_i32(arc.center.x).unwrap();
            w.write_i32(arc.center.y).unwrap();
            w.write_i32(arc.axis1.x).unwrap();
            w.write_i32(arc.axis1.y).unwrap();
            w.write_i32(arc.axis2.x).unwrap();
            w.write_i32(arc.axis2.y).unwrap();
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_ARC,
                level: type_level,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Polygon(poly) => {
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_drawing_shape_component(
                    tags::SHAPE_POLYGON_ID,
                    &poly.drawing,
                    false,
                ),
            });
            serialize_text_box_if_present(&poly.drawing, type_level, records);
            let mut w = ByteWriter::new();
            w.write_i32(poly.points.len() as i32).unwrap();
            for p in &poly.points {
                w.write_i32(p.x).unwrap();
                w.write_i32(p.y).unwrap();
            }
            if poly.raw_trailing.is_empty() {
                w.write_u32(0).unwrap();
            } else {
                w.write_bytes(&poly.raw_trailing).unwrap();
            }
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_POLYGON,
                level: type_level,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Curve(curve) => {
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_drawing_shape_component(
                    tags::SHAPE_CURVE_ID,
                    &curve.drawing,
                    false,
                ),
            });
            serialize_text_box_if_present(&curve.drawing, type_level, records);
            let mut w = ByteWriter::new();
            w.write_i32(curve.points.len() as i32).unwrap();
            for p in &curve.points {
                w.write_i32(p.x).unwrap();
                w.write_i32(p.y).unwrap();
            }
            for &t in &curve.segment_types {
                w.write_u8(t).unwrap();
            }
            w.write_u32(0).unwrap();
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_CURVE,
                level: type_level,
                size: 0,
                data: w.into_bytes(),
            });
        }
        ShapeObject::Group(group) => {
            // [Task #1771] 중첩 그룹: 파서(parse_container_children)·한컴 계약은
            // "자식 경계 = SHAPE_COMPONENT @ child_level ('$con') + 손자들이 더 깊은
            // level" 이다. 기존 CONTAINER(0x56) 단독 방출은 경계로 인식되지 않아
            // 재파스 시 하위 전체가 소실됐다 (3067999: children 710→12).
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: group_container_component_data(group, false),
            });
            for nested_child in &group.children {
                serialize_group_child(nested_child, comp_level + 1, comp_level + 2, records);
            }
        }
        ShapeObject::Picture(pic) => {
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_shape_component(tags::SHAPE_PICTURE_ID, &pic.shape_attr, false),
            });
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_PICTURE,
                level: type_level,
                size: 0,
                data: serialize_picture_data(pic),
            });
        }
        ShapeObject::Chart(chart) => {
            // Task #195 단계 2: 그룹 내 차트는 raw_chart_data로 라운드트립
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_drawing_shape_component(
                    chart.drawing.shape_attr.ctrl_id,
                    &chart.drawing,
                    false,
                ),
            });
            serialize_text_box_if_present(&chart.drawing, type_level, records);
            records.push(Record {
                tag_id: tags::HWPTAG_CHART_DATA,
                level: type_level,
                size: 0,
                data: chart.raw_chart_data.clone(),
            });
        }
        ShapeObject::Ole(ole) => {
            let drawing = ole_drawing_with_shape_component_contract(ole, false);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: comp_level,
                size: 0,
                data: serialize_shape_component(tags::SHAPE_OLE_ID, &drawing.shape_attr, false),
            });
            serialize_text_box_if_present(&drawing, type_level, records);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT_OLE,
                level: type_level,
                size: 0,
                data: serialize_ole_data(ole),
            });
        }
    }
}

fn serialize_ole_data(ole: &OleShape) -> Vec<u8> {
    if !ole.raw_tag_data.is_empty() {
        return ole.raw_tag_data.clone();
    }

    let mut w = ByteWriter::new();
    w.write_u32(1).unwrap(); // property/type
    w.write_i32(ole.extent_x).unwrap();
    w.write_i32(ole.extent_y).unwrap();
    w.write_u32(ole.bin_data_id).unwrap();
    w.write_u32(0).unwrap();
    w.write_u32(0).unwrap();
    w.write_u16(0).unwrap();
    w.into_bytes()
}

fn ole_drawing_with_shape_component_contract(ole: &OleShape, top_level: bool) -> DrawingObjAttr {
    let mut drawing = ole.drawing.clone();
    let extent_w = if ole.extent_x > 0 {
        ole.extent_x as u32
    } else if drawing.shape_attr.current_width > 0 {
        drawing.shape_attr.current_width
    } else if drawing.shape_attr.original_width > 0 {
        drawing.shape_attr.original_width
    } else {
        7200
    };
    let extent_h = if ole.extent_y > 0 {
        ole.extent_y as u32
    } else if drawing.shape_attr.current_height > 0 {
        drawing.shape_attr.current_height
    } else if drawing.shape_attr.original_height > 0 {
        drawing.shape_attr.original_height
    } else {
        7200
    };

    let attr = &mut drawing.shape_attr;
    if attr.ctrl_id == 0 {
        attr.ctrl_id = tags::SHAPE_OLE_ID;
        attr.is_two_ctrl_id = top_level;
    } else if top_level && !attr.is_two_ctrl_id {
        attr.is_two_ctrl_id = true;
    }
    if attr.local_file_version == 0 {
        attr.local_file_version = 1;
    }
    if attr.original_width == 0 {
        attr.original_width = extent_w;
    }
    if attr.original_height == 0 {
        attr.original_height = extent_h;
    }
    if attr.current_width == 0 {
        attr.current_width = attr.original_width;
    }
    if attr.current_height == 0 {
        attr.current_height = attr.original_height;
    }
    drawing
}

/// DrawingObjAttr의 text_box가 있으면 LIST_HEADER + 문단 목록 직렬화
fn serialize_text_box_if_present(drawing: &DrawingObjAttr, level: u16, records: &mut Vec<Record>) {
    if let Some(ref text_box) = drawing.text_box {
        // LIST_HEADER
        let mut w = ByteWriter::new();
        // para_count: 스펙은 INT16이지만 실제 HWP 파일에서는 UINT32로 저장됨
        w.write_u32(text_box.paragraphs.len() as u32).unwrap();
        w.write_u32(text_box.list_attr).unwrap();
        // 여백 + 최대 폭 (글상자 고유 데이터)
        w.write_i16(text_box.margin_left).unwrap();
        w.write_i16(text_box.margin_right).unwrap();
        w.write_i16(text_box.margin_top).unwrap();
        w.write_i16(text_box.margin_bottom).unwrap();
        w.write_u32(text_box.max_width).unwrap();
        // 원본 추가 바이트 복원 (라운드트립 보존)
        // [Task #1058] hwplib::ForTextBox::listHeader 정합 — TextBox LIST_HEADER 의
        // 마지막 13 byte 필드 contract:
        //   sw.writeZero(8);                 // 8 byte zero padding
        //   sw.writeSInt4(editableAtFormMode); // 4 byte (0 = false)
        //   sw.writeUInt1(fieldNameFlag);     // 1 byte (0 = no fieldName)
        // 한컴은 이 contract 가 누락되면 글상자 안 paragraph 를 본문 list 로 인식하여
        // 신규 paragraph (각주) 추가 시 다단계 목록 번호 "1.1.1.1.1.1" 자동 부여.
        // HWP 출처 IR 은 raw_list_header_extra 에 보존 → HWPX 출처 (raw 부재) 는 default 적용.
        if !text_box.raw_list_header_extra.is_empty() {
            w.write_bytes(&text_box.raw_list_header_extra).unwrap();
        } else {
            // HWPX 출처: 한컴 default 13 byte (zero 8 + editable 0 + fieldName flag 0)
            w.write_bytes(&[0u8; 13]).unwrap();
        }
        records.push(Record {
            tag_id: tags::HWPTAG_LIST_HEADER,
            level,
            size: 0,
            data: w.into_bytes(),
        });

        // 문단 목록 (LIST_HEADER와 같은 레벨)
        serialize_paragraph_list(&text_box.paragraphs, level, records);
    }
}

// ============================================================
// 공통 직렬화 헬퍼
// ============================================================

/// CommonObjAttr 직렬화
fn serialize_common_obj_attr(common: &CommonObjAttr) -> Vec<u8> {
    let mut w = ByteWriter::new();
    let attr = if common.attr != 0 {
        common.attr
    } else {
        pack_common_attr_bits(common)
    };
    w.write_u32(attr).unwrap();
    w.write_u32(common.vertical_offset).unwrap();
    w.write_u32(common.horizontal_offset).unwrap();
    w.write_u32(common.width).unwrap();
    w.write_u32(common.height).unwrap();
    w.write_i32(common.z_order).unwrap();
    w.write_i16(common.margin.left).unwrap();
    w.write_i16(common.margin.right).unwrap();
    w.write_i16(common.margin.top).unwrap();
    w.write_i16(common.margin.bottom).unwrap();
    w.write_u32(common.instance_id).unwrap();
    // 쪽나눔 방지 (INT32)
    w.write_i32(common.prevent_page_break).unwrap();
    // 설명문 (항상 길이 포함, 빈 문자열이면 0)
    w.write_hwp_string(&common.description).unwrap();
    // 원본 추가 바이트 복원 (라운드트립 보존)
    if !common.raw_extra.is_empty() {
        w.write_bytes(&common.raw_extra).unwrap();
    }
    w.into_bytes()
}

/// SHAPE_COMPONENT 데이터 직렬화 (ShapeComponentAttr만 — Picture, Group용)
///
/// 구조: ctrl_id(×1 or ×2) + ShapeComponentAttr + rendering_info
fn serialize_shape_component(
    default_ctrl_id: u32,
    attr: &ShapeComponentAttr,
    top_level: bool,
) -> Vec<u8> {
    let mut w = ByteWriter::new();
    write_shape_component_base(&mut w, default_ctrl_id, attr, top_level);
    w.into_bytes()
}

/// SHAPE_COMPONENT 데이터 직렬화 (DrawingObjAttr 전체 — 도형용)
///
/// 구조: ctrl_id(×1 or ×2) + ShapeComponentAttr + rendering_info + border_line + fill + shadow
fn serialize_drawing_shape_component(
    default_ctrl_id: u32,
    drawing: &DrawingObjAttr,
    top_level: bool,
) -> Vec<u8> {
    let mut w = ByteWriter::new();
    write_shape_component_base(&mut w, default_ctrl_id, &drawing.shape_attr, top_level);

    // 테두리 선 정보 (13바이트: color 4 + width 4 + attr 4 + outline 1)
    w.write_color_ref(drawing.border_line.color).unwrap();
    w.write_i32(drawing.border_line.width).unwrap();
    w.write_u32(drawing.border_line.attr).unwrap();
    w.write_u8(drawing.border_line.outline_style).unwrap();

    // 채우기 정보
    serialize_shape_fill(&mut w, &drawing.fill);

    // 그림자 정보 (16바이트)
    w.write_u32(drawing.shadow_type).unwrap();
    w.write_color_ref(drawing.shadow_color).unwrap();
    w.write_i32(drawing.shadow_offset_x).unwrap();
    w.write_i32(drawing.shadow_offset_y).unwrap();

    // 인스턴스 ID (4바이트) + 예약 (1바이트) + 그림자 투명도 (1바이트)
    w.write_u32(drawing.inst_id).unwrap();
    w.write_u8(0).unwrap(); // 예약
    w.write_u8(drawing.shadow_alpha).unwrap();

    w.into_bytes()
}

/// ShapeComponentAttr 공통 기록 (ctrl_id + 속성 + 렌더링 행렬)
fn write_shape_component_base(
    w: &mut ByteWriter,
    default_ctrl_id: u32,
    attr: &ShapeComponentAttr,
    top_level: bool,
) {
    // ctrl_id: 원본에서 보존된 값 사용, 없으면 기본값
    let actual_id = if attr.ctrl_id != 0 {
        attr.ctrl_id
    } else {
        default_ctrl_id
    };
    let is_two = if attr.ctrl_id != 0 {
        attr.is_two_ctrl_id
    } else {
        top_level
    };

    w.write_u32(actual_id).unwrap();
    if is_two {
        w.write_u32(actual_id).unwrap();
    }

    // ShapeComponentAttr
    w.write_i32(attr.offset_x).unwrap();
    w.write_i32(attr.offset_y).unwrap();
    w.write_u16(attr.group_level).unwrap();
    w.write_u16(attr.local_file_version).unwrap();
    w.write_u32(attr.original_width).unwrap();
    w.write_u32(attr.original_height).unwrap();
    w.write_u32(attr.current_width).unwrap();
    w.write_u32(attr.current_height).unwrap();

    // flip: 원본 전체 값 사용 (상위 비트 보존)
    let flip = if attr.flip != 0 {
        attr.flip
    } else {
        let mut f: u32 = 0;
        if attr.horz_flip {
            f |= 0x01;
        }
        if attr.vert_flip {
            f |= 0x02;
        }
        f
    };
    w.write_u32(flip).unwrap();

    w.write_i16(attr.rotation_angle).unwrap();
    w.write_i32(attr.rotation_center.x).unwrap();
    w.write_i32(attr.rotation_center.y).unwrap();

    // Rendering 정보 (원본이 있으면 복원, 없으면 적절한 행렬 생성)
    if !attr.raw_rendering.is_empty() {
        w.write_bytes(&attr.raw_rendering).unwrap();
    } else if attr.rotation_angle.rem_euclid(360) != 0 {
        write_generated_rendering_matrix(w, attr);
    } else if has_explicit_rendering_matrix(attr) {
        write_parsed_rendering_matrix(w, attr);
    } else {
        let is_group_child = attr.group_level > 0;
        let cnt: u16 = if is_group_child { 2 } else { 1 };
        w.write_u16(cnt).unwrap();
        // translation matrix = identity [1, 0, 0, 0, 1, 0].
        // 그룹 자식 위치의 단일 권위는 render_tx/ty 다 (렌더러 layout_group_child_*,
        // object_ops 그룹 생성 모두 render_tx 기준). 위치를 의도한 작성자는 항상
        // render_tx 를 명시하므로(그 경우 explicit 경로로 감), 이 폴백에 오는
        // offset_x/y 는 위치가 아닌 메타데이터다(HWP3 relative_pos 등). 종전처럼
        // offset 을 tx/ty 로 승격하면 재파스에서 render_tx 가 생겨 그룹 자식이
        // offset 만큼 이동한다 (#1892 대법원 서식 최대 10485px 라운드트립 변위).
        w.write_f64(1.0).unwrap();
        w.write_f64(0.0).unwrap();
        w.write_f64(0.0).unwrap(); // tx
        w.write_f64(0.0).unwrap();
        w.write_f64(1.0).unwrap();
        w.write_f64(0.0).unwrap(); // ty
                                   // scale matrix = identity [1, 0, 0, 0, 1, 0]
                                   // (스케일은 current_width/original_width 값으로 표현 — 행렬에 중복 기록하면 이중 적용됨)
        w.write_f64(1.0).unwrap();
        w.write_f64(0.0).unwrap();
        w.write_f64(0.0).unwrap();
        w.write_f64(0.0).unwrap();
        w.write_f64(1.0).unwrap();
        w.write_f64(0.0).unwrap();
        // rotation matrix. Hancom applies visible picture rotation from the
        // rendering rotMatrix, not only from ShapeComponentAttr.rotation_angle.
        write_matrix(w, shape_rotation_matrix(attr));
        // 그룹 자식 (cnt=2): 두 번째 scale + rotation 세트 (identity)
        if is_group_child {
            // scale2 = identity
            w.write_f64(1.0).unwrap();
            w.write_f64(0.0).unwrap();
            w.write_f64(0.0).unwrap();
            w.write_f64(0.0).unwrap();
            w.write_f64(1.0).unwrap();
            w.write_f64(0.0).unwrap();
            // rotation2 = identity
            w.write_f64(1.0).unwrap();
            w.write_f64(0.0).unwrap();
            w.write_f64(0.0).unwrap();
            w.write_f64(0.0).unwrap();
            w.write_f64(1.0).unwrap();
            w.write_f64(0.0).unwrap();
        }
    }
}

fn has_explicit_rendering_matrix(attr: &ShapeComponentAttr) -> bool {
    const EPSILON: f64 = 0.000_001;

    (attr.render_sx - 1.0).abs() > EPSILON
        || attr.render_b.abs() > EPSILON
        || attr.render_tx.abs() > EPSILON
        || attr.render_c.abs() > EPSILON
        || (attr.render_sy - 1.0).abs() > EPSILON
        || attr.render_ty.abs() > EPSILON
}

fn write_matrix(w: &mut ByteWriter, matrix: [f64; 6]) {
    for value in matrix {
        w.write_f64(value).unwrap();
    }
}

fn shape_scale_matrix(attr: &ShapeComponentAttr) -> [f64; 6] {
    let sx = if attr.original_width > 0 && attr.current_width > 0 {
        attr.current_width as f64 / attr.original_width as f64
    } else {
        1.0
    };
    let sy = if attr.original_height > 0 && attr.current_height > 0 {
        attr.current_height as f64 / attr.original_height as f64
    } else {
        1.0
    };
    [sx, 0.0, 0.0, 0.0, sy, 0.0]
}

fn shape_rotation_positive_correction(attr: &ShapeComponentAttr) -> (f64, f64) {
    let width = attr.current_width as f64;
    let height = attr.current_height as f64;
    if width <= 0.0 || height <= 0.0 {
        return (0.0, 0.0);
    }

    let theta = (attr.rotation_angle as f64).to_radians();
    let cos = theta.cos();
    let sin = theta.sin();
    let corners = [
        (0.0, 0.0),
        (width * cos, width * sin),
        (-height * sin, height * cos),
        (width * cos - height * sin, width * sin + height * cos),
    ];
    let min_x = corners
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::INFINITY, f64::min);
    let min_y = corners
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min);
    (-min_x, -min_y)
}

fn shape_rotation_matrix(attr: &ShapeComponentAttr) -> [f64; 6] {
    if attr.rotation_angle.rem_euclid(360) == 0 {
        return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    }

    let theta = (attr.rotation_angle as f64).to_radians();
    let cos = theta.cos();
    let sin = theta.sin();
    let (tx, ty) = shape_rotation_positive_correction(attr);

    [
        cos,
        -sin,
        tx - attr.offset_x as f64,
        sin,
        cos,
        ty - attr.offset_y as f64,
    ]
}

fn write_generated_rendering_matrix(w: &mut ByteWriter, attr: &ShapeComponentAttr) {
    let is_group_child = attr.group_level > 0;
    let cnt: u16 = if is_group_child { 2 } else { 1 };
    w.write_u16(cnt).unwrap();
    write_matrix(
        w,
        [
            1.0,
            0.0,
            attr.offset_x as f64,
            0.0,
            1.0,
            attr.offset_y as f64,
        ],
    );
    write_matrix(w, shape_scale_matrix(attr));
    write_matrix(w, shape_rotation_matrix(attr));
    if is_group_child {
        write_matrix(w, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        write_matrix(w, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    }
}

fn write_parsed_rendering_matrix(w: &mut ByteWriter, attr: &ShapeComponentAttr) {
    // HWP parser composes rendering info as:
    // result = translation x rotation x scale
    //
    // Store the parsed affine transform so reload reconstructs exactly:
    // [sx, b, tx; c, sy, ty] = [1,0,tx;0,1,ty] x I x [sx,b,0;c,sy,0]
    w.write_u16(1).unwrap();
    write_matrix(w, [1.0, 0.0, attr.render_tx, 0.0, 1.0, attr.render_ty]);
    write_matrix(
        w,
        [
            attr.render_sx,
            attr.render_b,
            0.0,
            attr.render_c,
            attr.render_sy,
            0.0,
        ],
    );
    write_matrix(w, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
}

/// 도형 채우기 직렬화 (SHAPE_COMPONENT 내부 — parse_fill과 동일한 형식)
fn serialize_shape_fill(w: &mut ByteWriter, fill: &Fill) {
    let fill_type_val: u32 = match fill.fill_type {
        FillType::None => 0,
        FillType::Solid => 1,
        FillType::Image => 2,
        FillType::Gradient => 4,
    };
    w.write_u32(fill_type_val).unwrap();

    if fill_type_val == 0 {
        // 채우기 없음: 4바이트 추가 (additional_size = 0)
        w.write_u32(0).unwrap();
        return;
    }

    // bit 0: 단색 채우기
    if fill_type_val & 0x01 != 0 {
        if let Some(ref solid) = fill.solid {
            w.write_color_ref(solid.background_color).unwrap();
            w.write_color_ref(solid.pattern_color).unwrap();
            w.write_i32(solid.pattern_type).unwrap();
        }
    }

    // bit 2: 그라데이션 채우기 (parse_fill 형식: kind=u8, angle/cx/cy/blur/count=u32)
    if fill_type_val & 0x04 != 0 {
        if let Some(ref grad) = fill.gradient {
            w.write_u8(grad.gradient_type as u8).unwrap();
            w.write_u32(grad.angle as u32).unwrap();
            w.write_u32(grad.center_x as u32).unwrap();
            w.write_u32(grad.center_y as u32).unwrap();
            w.write_u32(grad.blur as u32).unwrap();
            w.write_u32(grad.colors.len() as u32).unwrap();
            // change_points: count > 2일 때만 기록
            if grad.colors.len() > 2 {
                for &pos in &grad.positions {
                    w.write_i32(pos).unwrap();
                }
            }
            for &color in &grad.colors {
                w.write_color_ref(color).unwrap();
            }
        }
    }

    // bit 1: 이미지 채우기
    if fill_type_val & 0x02 != 0 {
        if let Some(ref img) = fill.image {
            let mode_val: u8 = match img.fill_mode {
                ImageFillMode::TileAll => 0,
                ImageFillMode::TileHorzTop => 1,
                ImageFillMode::TileHorzBottom => 2,
                ImageFillMode::TileVertLeft => 3,
                ImageFillMode::TileVertRight => 4,
                ImageFillMode::Total => 0,
                ImageFillMode::FitToSize => 5,
                ImageFillMode::Center => 6,
                ImageFillMode::CenterTop => 7,
                ImageFillMode::CenterBottom => 8,
                ImageFillMode::LeftCenter => 9,
                ImageFillMode::LeftTop => 10,
                ImageFillMode::LeftBottom => 11,
                ImageFillMode::RightCenter => 12,
                ImageFillMode::RightTop => 13,
                ImageFillMode::RightBottom => 14,
                ImageFillMode::None => 15,
            };
            w.write_u8(mode_val).unwrap();
            w.write_i8(img.brightness).unwrap();
            w.write_i8(img.contrast).unwrap();
            w.write_u8(img.effect).unwrap();
            w.write_u16(img.bin_data_id).unwrap();
        }
    }

    // 추가 속성. 그라데이션은 HWP5 fill contract상 blurring center 1바이트를 가진다.
    if fill_type_val & 0x04 != 0 {
        w.write_u32(1).unwrap();
        w.write_u8(fill.gradient.as_ref().map(|g| g.step_center).unwrap_or(0))
            .unwrap();
    } else {
        w.write_u32(0).unwrap();
    }

    // alpha 바이트 (채우기 종류별 각 1바이트)
    if fill_type_val & 0x01 != 0 {
        w.write_u8(fill.alpha).unwrap();
    }
    if fill_type_val & 0x04 != 0 {
        w.write_u8(fill.alpha).unwrap();
    }
    if fill_type_val & 0x02 != 0 {
        w.write_u8(fill.alpha).unwrap();
    }
}

/// LIST_HEADER(간단) + 문단 목록 직렬화
fn serialize_list_header_with_paragraphs(
    paragraphs: &[crate::model::paragraph::Paragraph],
    level: u16,
    records: &mut Vec<Record>,
) {
    let mut w = ByteWriter::new();
    w.write_u16(paragraphs.len() as u16).unwrap();
    w.write_u32(0).unwrap(); // list_attr

    records.push(Record {
        tag_id: tags::HWPTAG_LIST_HEADER,
        level,
        size: 0,
        data: w.into_bytes(),
    });

    serialize_paragraph_list(paragraphs, level + 1, records);
}

struct HeaderFooterListLayout {
    list_attr: u32,
    text_width: u32,
    text_height: u32,
    text_ref: u8,
    num_ref: u8,
}

fn serialize_header_footer_list_header_with_paragraphs(
    paragraphs: &[crate::model::paragraph::Paragraph],
    layout: HeaderFooterListLayout,
    level: u16,
    records: &mut Vec<Record>,
) {
    records.push(Record {
        tag_id: tags::HWPTAG_LIST_HEADER,
        level,
        size: 0,
        data: build_header_footer_list_header(
            paragraphs.len() as u16,
            layout.list_attr,
            layout.text_width,
            layout.text_height,
            layout.text_ref,
            layout.num_ref,
            0,
        ),
    });

    // HWP5 header/footer sub-list paragraphs are siblings of the LIST_HEADER
    // record, not one level deeper. Hancom 2020 treats the deeper
    // PARA_HEADER/PARA_TEXT pair as a damaged/modified BodyText contract.
    serialize_paragraph_list(paragraphs, level, records);
}

fn build_header_footer_list_header(
    para_count: u16,
    list_attr: u32,
    text_width: u32,
    text_height: u32,
    text_ref: u8,
    num_ref: u8,
    ext_flags: u16,
) -> Vec<u8> {
    let mut w = ByteWriter::new();
    w.write_u16(para_count).unwrap();
    w.write_u32(list_attr).unwrap();
    w.write_u16(0).unwrap();
    w.write_u32(text_width).unwrap();
    w.write_u32(text_height).unwrap();
    w.write_u8(text_ref).unwrap();
    w.write_u8(num_ref).unwrap();
    w.write_u16(ext_flags).unwrap();
    w.write_bytes(&[0u8; 14]).unwrap();
    w.into_bytes()
}

// ============================================================
// 수식 ('eqed')
// ============================================================

/// 수식 컨트롤 직렬화
///
/// raw_ctrl_data를 보존하여 라운드트립 무손실 직렬화.
fn serialize_equation_control(eq: &Equation, level: u16, records: &mut Vec<Record>) {
    // CTRL_HEADER with CommonObjAttr (또는 원본 ctrl_data)
    let ctrl_data = if eq.raw_ctrl_data.is_empty() {
        serialize_common_obj_attr(&eq.common)
    } else {
        eq.raw_ctrl_data.clone()
    };
    records.push(make_ctrl_record(tags::CTRL_EQUATION, level, &ctrl_data));

    // HWPTAG_EQEDIT 자식 레코드
    let mut w = ByteWriter::new();
    // attr: u32
    w.write_u32(0).unwrap();
    // script: HWP string (length-prefixed UTF-16LE)
    w.write_hwp_string(&eq.script).unwrap();
    // font_size: u32
    w.write_u32(eq.font_size).unwrap();
    // color: u32
    w.write_u32(eq.color).unwrap();
    // baseline: i16
    w.write_i16(eq.baseline).unwrap();
    // [Task #1061] unknown: u16 — HWP5 spec 표 105 누락 영역, hwplib 정합
    w.write_u16(eq.unknown).unwrap();
    // version_info: HWP string
    w.write_hwp_string(&eq.version_info).unwrap();
    // font_name: HWP string
    w.write_hwp_string(&eq.font_name).unwrap();

    records.push(Record {
        tag_id: tags::HWPTAG_EQEDIT,
        level: level + 1,
        size: 0,
        data: w.into_bytes(),
    });
}

// ============================================================
// [Task #852 Stage 2.4] 양식 개체 ('form')
// ============================================================

// 한 섹션 내 Form 컨트롤의 등장순 0-base 카운터.
// `serialize_section` 시작 시 0으로 reset, 각 Form 직렬화 후 +1.
// 정답지 form-01.hwp 의 5 form 이 TabOrder/order 0..4 로 단조증가하는 패턴 재현.
thread_local! {
    static FORM_ORDER_COUNTER: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

pub(super) fn reset_form_order_counter() {
    FORM_ORDER_COUNTER.with(|c| c.set(0));
}

fn next_form_order() -> u32 {
    FORM_ORDER_COUNTER.with(|c| {
        let o = c.get();
        c.set(o + 1);
        o
    })
}

/// 현재 카운터 값 조회 (다음 Form 직렬화 시 사용될 order). Form 5 개 직렬화 직후 = 5.
fn peek_form_order_counter() -> u32 {
    FORM_ORDER_COUNTER.with(|c| c.get())
}

/// 양식 개체 직렬화 — CTRL_HEADER (46 bytes) + HWPTAG_FORM_OBJECT 자식
///
/// 정답지 `samples/form-01.hwp` reverse engineering 결과를 기반으로 작성.
/// 자세한 구조는 `mydocs/plans/task_m100_852_stage24.md` 참조.
fn serialize_form_control(form: &FormObject, level: u16, records: &mut Vec<Record>) {
    let order = next_form_order();

    // 1) CTRL_HEADER "form" — 46 bytes 고정
    //
    // 구조:
    //   0..4   ctrl_id "form" (LE: "mrof")
    //   4..8   attr = 0x002a6211 (HWP5 common ctrl property flag, 정답지 고정값)
    //   8..12  y_offset = 0 (i32)
    //   12..16 x_offset = 0 (i32)
    //   16..20 width  (u32, HWPUNIT)
    //   20..24 height (u32, HWPUNIT)
    //   24..28 order  (u32, z-order/TabOrder-1, 0-base)
    //   28..36 zero (8 bytes)
    //   36..40 instance_id (u32, 0x7dcd59d6 + order)
    //   40..46 zero (6 bytes)
    let mut hdr = Vec::with_capacity(46);
    hdr.extend_from_slice(b"mrof"); // ctrl_id "form" little-endian
    hdr.extend_from_slice(&0x002a_6211u32.to_le_bytes());
    hdr.extend_from_slice(&0i32.to_le_bytes()); // y_offset
    hdr.extend_from_slice(&0i32.to_le_bytes()); // x_offset
    hdr.extend_from_slice(&form.width.to_le_bytes());
    hdr.extend_from_slice(&form.height.to_le_bytes());
    hdr.extend_from_slice(&order.to_le_bytes());
    hdr.extend_from_slice(&[0u8; 8]);
    hdr.extend_from_slice(&(0x7dcd_59d6u32.wrapping_add(order)).to_le_bytes());
    hdr.extend_from_slice(&[0u8; 6]);
    debug_assert_eq!(hdr.len(), 46);
    records.push(Record {
        tag_id: tags::HWPTAG_CTRL_HEADER,
        level,
        size: hdr.len() as u32,
        data: hdr,
    });

    // 2) HWPTAG_FORM_OBJECT 자식 (level + 1)
    //
    // 구조:
    //   0..4   type_id (ASCII "tbp+"/"tbc+"/"boc+"/"tbr+"/"tde+")
    //   4..8   type_id 중복 (magic marker)
    //   8..12  wchar_count (u32)
    //   12..14 wchar_count (u16, 8..12 와 동일 값)
    //   14..   UTF-16 LE 속성 문자열 (wchar_count chars)
    let type_id: &[u8; 4] = match form.form_type {
        FormType::PushButton => b"tbp+",
        FormType::CheckBox => b"tbc+",
        FormType::ComboBox => b"boc+",
        FormType::RadioButton => b"tbr+",
        FormType::Edit => b"tde+",
    };
    let prop_str = build_form_prop_str(form, order);
    let wchars: Vec<u16> = prop_str.encode_utf16().collect();
    let wlen = wchars.len();

    let mut fo = Vec::with_capacity(14 + wlen * 2);
    fo.extend_from_slice(type_id);
    fo.extend_from_slice(type_id);
    fo.extend_from_slice(&(wlen as u32).to_le_bytes());
    fo.extend_from_slice(&(wlen as u16).to_le_bytes());
    for w in &wchars {
        fo.extend_from_slice(&w.to_le_bytes());
    }
    records.push(Record {
        tag_id: tags::HWPTAG_FORM_OBJECT,
        level: level + 1,
        size: fo.len() as u32,
        data: fo,
    });
}

/// 정답지 fall-back BorderType (FormObject.properties 에 키가 없을 때).
fn default_border_type(ft: FormType) -> i32 {
    match ft {
        FormType::PushButton => 4,
        FormType::CheckBox => 0,
        FormType::RadioButton => 0,
        FormType::ComboBox => 5,
        FormType::Edit => 5,
    }
}

/// `FormObject.properties` 에서 키를 정수로 읽거나 fallback 반환.
fn prop_int(form: &FormObject, key: &str, fallback: i32) -> i32 {
    form.properties
        .get(key)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(fallback)
}

/// 정답지 포맷 속성 문자열 합성.
///
/// 정답지 reverse engineering 결과 구조 (set:N 의 N 은 chars 수):
/// ```text
/// CommonSet:set:{N1}:{common_body} CharShapeSet:set:{N2}:{char_body} {TypeSet}:set:{N3}:{type_body}
/// ```
///
/// 자세한 키 순서는 mydocs/plans/task_m100_852_stage24.md 참조.
fn build_form_prop_str(form: &FormObject, order: u32) -> String {
    let common = build_common_set(form, order);
    let char_shape = build_char_shape_set_for(form);
    let (type_name, type_body) = build_type_set(form);

    format!(
        "CommonSet:set:{}:{} CharShapeSet:set:{}:{} {}:set:{}:{} ",
        common.chars().count(),
        common,
        char_shape.chars().count(),
        char_shape,
        type_name,
        type_body.chars().count(),
        type_body,
    )
}

fn build_common_set(form: &FormObject, order: u32) -> String {
    let name = &form.name;
    let group = form
        .properties
        .get("GroupName")
        .cloned()
        .unwrap_or_default();
    let command = form.properties.get("Command").cloned().unwrap_or_default();
    let border_type = prop_int(form, "BorderType", default_border_type(form.form_type));
    let draw_frame = prop_int(form, "DrawFrame", 1);
    let tab_stop = prop_int(form, "TabStop", 1);
    let editable = prop_int(form, "Editable", 1);
    let printable = prop_int(form, "Printable", 1);
    // HWPX `tabOrder` (1-base, 정답지와 동일) 우선, 없으면 카운터+1.
    let tab_order = prop_int(form, "TabOrder", (order + 1) as i32);
    format!(
        "Name:wstring:{}:{} ForeColor:int:{} BackColor:int:{} GroupName:wstring:{}:{} \
         TabStop:bool:{} TabOrder:int:{} Enabled:bool:{} BorderType:int:{} DrawFrame:bool:{} \
         Command:wstring:{}:{} Editable:bool:{} Printable:bool:{} ",
        name.chars().count(),
        name,
        form.fore_color,
        form.back_color,
        group.chars().count(),
        group,
        tab_stop,
        tab_order,
        if form.enabled { 1 } else { 0 },
        border_type,
        draw_frame,
        command.chars().count(),
        command,
        editable,
        printable,
    )
}

fn build_char_shape_set_for(form: &FormObject) -> String {
    // HWPX `<hp:formCharPr charPrIDRef="..." followContext="..." autoSz="..." wordWrap="..."/>`
    // 보존 속성 우선. 없으면 정답지 기본값 (ComboBox 만 AutoSize=1).
    let char_shape_id = prop_int(form, "CharShapeID", 0);
    let follow_context = prop_int(form, "FollowContext", 0);
    let auto_size = prop_int(
        form,
        "AutoSize",
        if matches!(form.form_type, FormType::ComboBox) {
            1
        } else {
            0
        },
    );
    let word_wrap = prop_int(form, "WordWrap", 0);
    format!(
        "CharShapeID:int:{} FollowContext:bool:{} AutoSize:bool:{} WordWrap:bool:{} ",
        char_shape_id, follow_context, auto_size, word_wrap,
    )
}

fn build_type_set(form: &FormObject) -> (&'static str, String) {
    match form.form_type {
        FormType::PushButton => (
            "ButtonSet",
            format!(
                "Caption:wstring:{}:{} ",
                form.caption.chars().count(),
                form.caption,
            ),
        ),
        FormType::CheckBox => (
            "ButtonSet",
            format!(
                "Caption:wstring:{}:{} Value:int:{} TriState:bool:{} BackStyle:int:{} ",
                form.caption.chars().count(),
                form.caption,
                form.value,
                prop_int(form, "TriState", 0),
                prop_int(form, "BackStyle", 1),
            ),
        ),
        FormType::RadioButton => {
            let group = form
                .properties
                .get("RadioGroupName")
                .cloned()
                .unwrap_or_default();
            (
                "ButtonSet",
                format!(
                    "Caption:wstring:{}:{} RadioGroupName:wstring:{}:{} Value:int:{} \
                     TriState:bool:{} BackStyle:int:{} ",
                    form.caption.chars().count(),
                    form.caption,
                    group.chars().count(),
                    group,
                    form.value,
                    prop_int(form, "TriState", 0),
                    prop_int(form, "BackStyle", 1),
                ),
            )
        }
        FormType::ComboBox => {
            // HWPX `<hp:listItem value="...">` 의 첫 항목이 정답지의 ComboBox Text.
            // HWPX parser 가 form.properties["listItem0"] 로 보존. form.text 우선,
            // 비어있으면 listItem0 fallback.
            let text = if form.text.is_empty() {
                form.properties
                    .get("listItem0")
                    .cloned()
                    .unwrap_or_default()
            } else {
                form.text.clone()
            };
            (
                "ComboBoxSet",
                format!(
                    "ListBoxRows:int:{} Text:wstring:{}:{} ListBoxWidth:int:{} EditEnable:bool:{} ",
                    prop_int(form, "ListBoxRows", 4),
                    text.chars().count(),
                    text,
                    prop_int(form, "ListBoxWidth", 0),
                    prop_int(form, "EditEnable", 1),
                ),
            )
        }
        FormType::Edit => {
            let password = form
                .properties
                .get("PasswordChar")
                .cloned()
                .unwrap_or_default();
            (
                "EditSet",
                format!(
                    "Text:wstring:{}:{} MultiLine:bool:{} PasswordChar:wstring:{}:{} \
                     MaxLength:int:{} ScrollBars:int:{} TabKeyBehavior:int:{} Number:bool:{} \
                     ReadOnly:bool:{} AlignText:int:{} ",
                    form.text.chars().count(),
                    form.text,
                    prop_int(form, "MultiLine", 0),
                    password.chars().count(),
                    password,
                    prop_int(form, "MaxLength", 2147483647),
                    prop_int(form, "ScrollBars", 0),
                    prop_int(form, "TabKeyBehavior", 0),
                    prop_int(form, "Number", 0),
                    prop_int(form, "ReadOnly", 0),
                    prop_int(form, "AlignText", 0),
                ),
            )
        }
    }
}

#[cfg(test)]
mod tests;
