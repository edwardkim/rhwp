//! 컨트롤 직렬화 (표, 도형, 그림, 머리말/꼬리말, 각주/미주 등)
//!
//! `parser::body_text::parse_ctrl_header` + `parser::control::parse_control`의 역방향.
//! 각 Control enum variant를 CTRL_HEADER 레코드(+자식 레코드)로 변환한다.

use super::body_text::serialize_paragraph_list;
use super::byte_writer::{char_to_wchar, ByteWriter};

use crate::model::control::*;
use crate::model::document::SectionDef;
use crate::model::footnote::FootnoteShape;
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply, MasterPage};
use crate::model::image::{ImageEffect, Picture};
use crate::model::page::{ColumnDef, ColumnDirection, ColumnType, PageBorderFill, PageDef};
use crate::model::paragraph::Paragraph;
use crate::model::shape::{
    Caption, CaptionDirection, CaptionVertAlign, CommonObjAttr, DrawingObjAttr, HorzAlign,
    HorzRelTo, OleShape, ShapeComponentAttr, ShapeObject, SizeCriterion, TextFlow, TextWrap,
    VertAlign, VertRelTo,
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
        Control::SectionDef(sd) => serialize_section_def(sd, level, ctrl_data_record, records),
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
        // 쪽 번호 시작 쪽 — payload 는 u32 하나뿐이다(실측 102건 전부 8바이트).
        Control::PageNumCtrl(pnc) => {
            records.push(make_ctrl_record(
                tags::CTRL_PAGE_NUM_CTRL,
                level,
                &pnc.page_starts_on.to_hwp5().to_le_bytes(),
            ));
        }
        Control::IndexMark(im) => {
            records.push(make_ctrl_record(
                tags::CTRL_INDEX_MARK,
                level,
                &serialize_index_mark_payload(im),
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
            // [#4396] `f.parameters`(HWPX `<hp:parameters>` 트리, HWPX 파서가 채움)에
            // `command`/`memo_index` 외 항목이 있으면 HWP5 로는 옮길 자리가 없다 —
            // `pdf/hwpspec-2024.pdf` §4.2.8(HWPTAG_CTRL_DATA, 표 61: "필드 이름이나
            // 하이퍼링크 정보를 저장" — 파라미터 셋을 감싸기만 할 뿐 필드별 item ID
            // 스키마는 스펙에 없다), §4.2.10.11(책갈피는 "책갈피이름 밖에 없다"),
            // §4.2.10.15(필드 CTRL_HEADER 는 command/id 뿐)를 확인한 결과다. 스펙에
            // 없는 값을 임의로 써넣는 대신(#4396 리뷰에서 되돌림 — 실제로
            // `hwpx_to_hwp.rs`의 0x4000+idx 순차 할당과 충돌 가능성도 있었다) 조용히
            // 버리지 않고 경고만 낸다.
            if let Some(warning) = field_parameter_loss_warning(f) {
                eprintln!("[hwp5] {warning}");
            }
        }
        // [Task #852 Stage 2.4] 양식 개체 직렬화 — CTRL_HEADER + HWPTAG_FORM_OBJECT
        Control::Form(form) => serialize_form_control(form, level, records),
        // 미구현 컨트롤은 최소한의 CTRL_HEADER만 생성
        Control::Hyperlink(_) | Control::Ruby(_) | Control::Unknown(_) => {
            let ctrl_id = match ctrl {
                Control::Unknown(u) => u.ctrl_id,
                // [#4677] 덧말은 PARA_TEXT 에 `17 00 'tdut'` 제어문자가 나가므로 짝이 되는
                // CTRL_HEADER 도 반드시 있어야 한다. 내용(mainText/subText/속성)까지 옮기는
                // 것은 #4397 소관이고, 여기서는 **짝을 맞추는 것**이 목적이다 — 짝이 없으면
                // 한글 2022 가 그 문서의 본문을 통째로 버린다.
                Control::Ruby(_) => tags::CTRL_CHAR_OVERLAP,
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

/// [#4396] `field.parameters`(HWPX `<hp:parameters>` 트리) 중 HWP5 로 옮길 자리가 없는
/// 항목이 있으면 경고 문자열을 만든다. `command`(CTRL_HEADER 의 command)와
/// `Number`(memo_index)는 이미 HWP5 쪽 슬롯이 있으므로 손실이 아니다 — 그 둘을 뺀
/// 나머지가 하나라도 있으면 `Some`.
///
/// 순수 판정 함수로 분리해 단위 테스트가 실제 stderr 를 가로채지 않고도 이 조건을
/// 검증할 수 있게 한다. 호출부(`serialize_control`)가 실제 경고를 `eprintln!` 한다.
fn field_parameter_loss_warning(field: &Field) -> Option<String> {
    let lost: Vec<String> = field
        .parameters
        .items
        .iter()
        .filter(|p| !matches!(parameter_name(p), Some("Command") | Some("Number")))
        .map(|p| parameter_name(p).unwrap_or("<이름 없음>").to_string())
        .collect();
    if lost.is_empty() {
        return None;
    }
    Some(format!(
        "필드({:?}) 파라미터 {}개 손실 — HWP5 CTRL_DATA에 규정된 슬롯 없음(#4396): {}",
        field.field_type,
        lost.len(),
        lost.join(", ")
    ))
}

/// `Parameter` 5종 공통으로 `name` 속성을 꺼낸다(경고 메시지 조립용).
fn parameter_name(p: &Parameter) -> Option<&str> {
    match p {
        Parameter::Boolean { name, .. }
        | Parameter::Integer { name, .. }
        | Parameter::Float { name, .. }
        | Parameter::String { name, .. } => name.as_deref(),
        Parameter::List(list) => list.name.as_deref(),
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

fn serialize_section_def(
    sd: &SectionDef,
    level: u16,
    ctrl_data_record: Option<&[u8]>,
    records: &mut Vec<Record>,
) {
    let mut w = ByteWriter::new();
    // HWPX 파서(parse_start_num, pageStartsOn)는 page_num_type 필드만 세팅하고
    // flags bit 20-21은 동기화하지 않는다. HWP5는 page_num_type을 별도 필드로
    // 갖지 않고 flags bit 20-21로만 표현하므로, 여기서 반영하지 않으면 HWPX
    // 출처 문서를 HWP5로 저장할 때 홀/짝 쪽번호 시작 설정이 유실된다.
    let flags = (sd.flags & !0x0030_0000) | (((sd.page_num_type as u32) & 0x03) << 20);
    w.write_u32(flags).unwrap();
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
    let mut nested_ctrl_header_seen = false;
    for raw in &sd.extra_child_records {
        let is_exact_owned_ctrl_data = ctrl_data_record.is_some_and(|data| {
            !nested_ctrl_header_seen
                && raw.tag_id == tags::HWPTAG_CTRL_DATA
                && raw.level == level.saturating_add(1)
                && raw.data.as_slice() == data
        });
        if is_exact_owned_ctrl_data {
            // 과거 parser 또는 외부 생성 IR이 같은 직접 자식 CTRL_DATA를
            // Paragraph.ctrl_data_records와 extra_child_records에 동시에 보유해도,
            // 아래 공용 복원 경로가 CTRL_HEADER 직후에 한 번만 출력하도록 한다.
            continue;
        }
        if raw.tag_id == tags::HWPTAG_CTRL_HEADER {
            // 이 경계 뒤의 CTRL_DATA는 중첩 raw 구조의 일부일 수 있으므로
            // payload가 같아도 원래 위치에서 보존한다.
            nested_ctrl_header_seen = true;
        }
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
    w.write_u16(char_to_wchar(fs.user_char)).unwrap();
    w.write_u16(char_to_wchar(fs.prefix_char)).unwrap();
    w.write_u16(char_to_wchar(fs.suffix_char)).unwrap();
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
    // after_decoration_letter 를 IR 값 그대로 방출한다. 종전엔 0 을 ')'(0x0029)로
    // 치환해, 닫는 장식이 없는(0) HWP5 각주가 저장 시 ')' 로 오염됐다. 생성 경로
    // (insert_footnote_native, HWPX 파서)는 항상 0x0029/실제값을 명시 설정하므로
    // 이 치환은 불필요하고, before_decoration_letter 방출과도 비대칭이었다.
    w.write_u16(fn_.after_decoration_letter).unwrap();
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
    // after_decoration_letter 를 IR 값 그대로 방출(footnote 와 동일 근거).
    w.write_u16(en.after_decoration_letter).unwrap();
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
        AutoNumberType::TotalPage => 6,
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
    data.extend_from_slice(&char_to_wchar(an.user_symbol).to_le_bytes());
    data.extend_from_slice(&char_to_wchar(an.prefix_char).to_le_bytes());
    data.extend_from_slice(&char_to_wchar(an.suffix_char).to_le_bytes());
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
        AutoNumberType::TotalPage => 6,
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
    data.extend_from_slice(&char_to_wchar(pnp.user_symbol).to_le_bytes());
    data.extend_from_slice(&char_to_wchar(pnp.prefix_char).to_le_bytes());
    data.extend_from_slice(&char_to_wchar(pnp.suffix_char).to_le_bytes());
    data.extend_from_slice(&char_to_wchar(pnp.dash_char).to_le_bytes());
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

/// 찾아보기 표식 payload — 책갈피와 달리 키가 CTRL_HEADER 안에 들어간다.
///
/// 실측(06926): `ctrl_id` 뒤에 곧바로 `WORD+WCHAR[]` 두 벌과 예약 4바이트가 오고,
/// 뒤따르는 CTRL_DATA 레코드는 없다.
fn serialize_index_mark_payload(im: &IndexMark) -> Vec<u8> {
    let mut w = ByteWriter::new();
    for key in [&im.first_key, &im.second_key] {
        let utf16: Vec<u16> = key.encode_utf16().collect();
        w.write_u16(utf16.len() as u16).unwrap();
        for ch in utf16 {
            w.write_u16(ch).unwrap();
        }
    }
    // 예약 4바이트 — 실측 전부 0.
    w.write_u32(0).unwrap();
    w.into_bytes()
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
///
/// [#3143] 미검증 `as` 캐스팅 방어:
/// - 겹칠 글자는 `encode_utf16()` 으로 서로게이트 쌍을 포함해 인코딩하고,
///   카운트는 파서(parse_char_overlap)와 대칭인 UTF-16 코드유닛 수로 기록한다.
///   (기존 `ch as u16` 은 비BMP 문자를 하위 16비트로 절단했다.)
/// - 카운트 필드(u16/u8)는 타입 한계로 상한을 두고 기록 배열도 함께 절단해
///   카운트와 실제 데이터가 항상 일치하도록 보장한다.
fn serialize_char_overlap(co: &CharOverlap) -> Vec<u8> {
    let mut w = ByteWriter::new();

    // 겹칠 글자: WCHAR(UTF-16LE) 배열, 서로게이트 쌍 포함
    let mut utf16: Vec<u16> = Vec::with_capacity(co.chars.len());
    let mut buf = [0u16; 2];
    for &ch in &co.chars {
        utf16.extend_from_slice(ch.encode_utf16(&mut buf));
    }
    // u16 카운트 필드 wraparound 방지 (스펙상 최대 9글자라 실사용에선 도달하지 않음)
    utf16.truncate(u16::MAX as usize);
    // 절단 끝이 홀로 남은 상위 서로게이트면 함께 제거
    if matches!(utf16.last(), Some(&last) if (0xD800..=0xDBFF).contains(&last)) {
        utf16.pop();
    }
    w.write_u16(utf16.len() as u16).unwrap();
    for unit in utf16 {
        w.write_u16(unit).unwrap();
    }

    w.write_u8(co.border_type).unwrap();
    w.write_i8(co.inner_char_size).unwrap();
    w.write_u8(co.expansion).unwrap();

    // charshape 카운트: u8 wraparound 방지 — 카운트와 기록 개수를 함께 상한 절단
    let n_ids = co.char_shape_ids.len().min(u8::MAX as usize);
    w.write_u8(n_ids as u8).unwrap();
    for &id in &co.char_shape_ids[..n_ids] {
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
    // 테두리 속성 워드(선 종류/끝모양 등, 표 87). 파서는 pic.border_attr.attr 로
    // 읽지만 종전엔 0 을 고정 방출해 스타일 테두리가 저장 시 유실됐다.
    w.write_u32(pic.border_attr.attr).unwrap(); // border_attr

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

    // [#2715] 캡션(SHAPE_COMPONENT 앞, level+1) 방출 헬퍼.
    //
    // 파서는 `SHAPE_COMPONENT` **앞**의 LIST_HEADER 를 캡션으로 읽는데
    // (`parser/control/shape.rs:134-147`), 종전 도형 arm 은 어느 것도 방출하지
    // 않아 그리기 도형·묶음·차트 캡션이 HWP5 저장에서 전량 소실됐다 (표 `:492`·
    // 그림 `:998` 만 방출 중이었다). 한컴 저장본도 `$rec`(사각형)·`$con`(묶음)에
    // `$pic`(그림)과 **동일한 30B LIST_HEADER** 를 SHAPE_COMPONENT 앞에 쓴다
    // (`samples/3-09월_교육_통합_2023.hwp`, `samples/draw-group.hwp` 실측).
    //
    // 호출 위치는 반드시 `emit_top_level_synthesized_ctrl_data` **뒤**여야 한다 —
    // `parse_caption` 은 `records[1..]` 전체를 캡션 문단으로 넘기므로, 캡션과
    // SHAPE_COMPONENT 사이에 CTRL_DATA 가 끼면 문단 파싱이 오염된다.
    //
    // OLE arm 은 제외한다: `#1283` 의 196B base-only 계약 영역이고
    // (`tests/issue_1251_ole_chart_contents.rs` 가 고정), 캡션 보유 OLE 한컴
    // 실물 샘플을 확보하지 못해 방출 위치를 실측 검증할 수 없다.
    let emit_caption = |caption: &Option<Caption>, records: &mut Vec<Record>| {
        if let Some(caption) = caption {
            serialize_caption(caption, level + 1, records);
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
            emit_caption(&line.drawing.caption, records);
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
                // [#3570] 한컴은 제어점 뒤에 0 4바이트를 더 쓴다(전수 20건 모두
                // `00000000`). 종전에는 raw_trailing 이 비면 아무것도 쓰지 않아
                // 레코드가 4바이트 짧았다. 다각형(위)과 같은 관례로 채운다.
                if conn.raw_trailing.is_empty() {
                    w.write_u32(0).unwrap();
                } else {
                    w.write_bytes(&conn.raw_trailing).unwrap();
                }
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
            emit_caption(&rect.drawing.caption, records);
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
            emit_caption(&ellipse.drawing.caption, records);
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
            emit_caption(&poly.drawing.caption, records);
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
            emit_caption(&arc.drawing.caption, records);
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
            emit_caption(&curve.drawing.caption, records);
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
            emit_caption(&group.caption, records);
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
        ShapeObject::Picture(pic) => {
            // [#2696] 그룹 해제(object_ops/shape.rs 의 ungroup_shape_native)는 그림
            // 자식을 최상위 Control::Shape(ShapeObject::Picture) 로 삽입하면서
            // char_count += 8 을 함께 적용한다. 종전에는 이 arm 이 아무
            // 레코드도 방출하지 않아 그림이 사라질 뿐 아니라 PARA_TEXT 의 확장 컨트롤
            // 문자와 CTRL_HEADER 개수가 어긋나 이후 컨트롤이 잘못된 위치에 결합됐다.
            // Control::Picture 가 쓰는 검증된 경로에 그대로 위임한다.
            serialize_picture_control(pic, level, ctrl_data_record, records);
        }
        ShapeObject::Chart(chart) => {
            // Task #195 단계 2: raw_chart_data를 그대로 보존하여 라운드트립 유지
            records.push(make_ctrl_record(
                tags::CTRL_GEN_SHAPE,
                level,
                &serialize_common_obj_attr(&chart.common),
            ));
            let sc_ctrl_id = chart.drawing.shape_attr.ctrl_id;
            emit_caption(&chart.caption, records);
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
            // 캡션 (SHAPE_COMPONENT 앞, level+1)
            if let Some(ref caption) = ole.caption {
                serialize_caption(caption, level + 1, records);
            }
            let drawing = ole_drawing_with_shape_component_contract(ole, true);
            records.push(Record {
                tag_id: tags::HWPTAG_SHAPE_COMPONENT,
                level: level + 1,
                size: 0,
                // [#2696] OLE 의 SHAPE_COMPONENT 는 의도적으로 base-only 다.
                // 한컴 저장본(samples/143E433F503322BD33.hwp)의 `$ole` SHAPE_COMPONENT
                // 실측 크기가 196B(base) 이고, 테두리/채우기/그림자 꼬리를 붙인
                // 252B 는 같은 파일의 도형 레코드 쪽이다. #1283 이 한컴 읽기 오류를
                // 잡으며 확정한 계약이므로 Chart arm 과 달리 확장하지 않는다.
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
            // [#3565] 연결선 자식은 실제 레코드가 '$col' 로 나간다
            // (serialize_group_child 의 connector 분기). 목록도 같은 판정을
            // 써야 부모 선언과 자식 실체가 어긋나지 않는다.
            ShapeObject::Line(line) => {
                if line.connector.is_some() {
                    tags::SHAPE_CONNECTOR_ID
                } else {
                    tags::SHAPE_LINE_ID
                }
            }
            ShapeObject::Rectangle(_) => tags::SHAPE_RECT_ID,
            ShapeObject::Ellipse(_) => tags::SHAPE_ELLIPSE_ID,
            ShapeObject::Arc(_) => tags::SHAPE_ARC_ID,
            ShapeObject::Polygon(_) => tags::SHAPE_POLYGON_ID,
            ShapeObject::Curve(_) => tags::SHAPE_CURVE_ID,
            // [#3565] 중첩 그룹 자식의 실제 레코드는 '$con' 이다
            // (serialize_group_child 의 Group arm). 'gso ' 로 선언하면
            // 한컴이 자식 트리를 세우지 못해 문서를 열지 못한다.
            ShapeObject::Group(_) => tags::SHAPE_CONTAINER_ID,
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
                // [#3570] 한컴은 제어점 뒤에 0 4바이트를 더 쓴다(전수 20건 모두
                // `00000000`). 종전에는 raw_trailing 이 비면 아무것도 쓰지 않아
                // 레코드가 4바이트 짧았다. 다각형(위)과 같은 관례로 채운다.
                if conn.raw_trailing.is_empty() {
                    w.write_u32(0).unwrap();
                } else {
                    w.write_bytes(&conn.raw_trailing).unwrap();
                }
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
                // [#2696] 최상위 OLE arm 과 동일하게 base-only 를 유지한다(#1283 계약).
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
        // HWPX parser can materialize this compatibility bit semantically while
        // retaining a non-zero raw attr captured before that materialization.
        // Keep raw-first serialization, but do not drop an asserted bit 28.
        common.attr
            | if common.hwp5_gen_shape_attr_bit28 {
                1 << 28
            } else {
                0
            }
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

/// `CommonObjAttr` 의 enum 필드들로부터 attr u32 비트를 합성한다.
///
/// [#4400] `document_core/converters/common_obj_attr_writer.rs` 에서 이 파일로 이동했다
/// (`sync_anchor_bits`도 같은 이유로 함께 옮겼다 — 아래 참고). CTRL_HEADER attr 비트
/// 레이아웃을 아는 것은 본질적으로 직렬화 로직이고, 이 파일의 `serialize_common_obj_attr`
/// (및 `document_core::converters::common_obj_attr_writer::serialize_common_obj_attr`
/// 어댑터)가 유일한 소비 목적이다 — 직렬화기가 도리어 `document_core` 를 참조하는
/// 역방향 의존을 없앤다. `document_core` 쪽에서 필요한 호출(패스스루 무효화에 준하는
/// attr 동기화)은 이 pub(crate) 함수와 `sync_anchor_bits`를 그대로 가져다 쓴다.
///
/// 비트 레이아웃 (parser/control/shape.rs 의 역방향):
/// - bit 0: treat_as_char
/// - bit 2: affect_line_spacing (줄 간격에 영향 — [#2784], 스펙 표 70)
/// - bit 3-4: vert_rel_to (Paper=0, Page=1, Para=2)
/// - bit 5-7: vert_align
/// - bit 8-9: horz_rel_to
/// - bit 10-12: horz_align
/// - bit 13: flow_with_text (HWPX object contract)
/// - bit 14: allow_overlap (HWPX object contract)
/// - bit 15-17: width_criterion
/// - bit 18-19: height_criterion
/// - bit 21-23: text_wrap
/// - bit 24-25: text_flow
/// - bit 20: size protect when VertRelTo is Para
/// - bit 26: HWPX GenShape storage high bit 후보
/// - bit 28: HWPX GenShape numbering category high bit 후보
pub(crate) fn pack_common_attr_bits(common: &CommonObjAttr) -> u32 {
    let mut a: u32 = 0;
    if common.treat_as_char {
        a |= 0x01;
    }
    // [#2784] affectLSpacing — 개체 공통 속성 attr bit 2 (스펙 표 70).
    if common.affect_line_spacing {
        a |= 1 << 2;
    }
    a |= (vert_rel_to_to_bits(common.vert_rel_to) & 0x03) << 3;
    a |= (vert_align_to_bits(common.vert_align) & 0x07) << 5;
    a |= (horz_rel_to_to_bits(common.horz_rel_to) & 0x03) << 8;
    a |= (horz_align_to_bits(common.horz_align) & 0x07) << 10;
    if common.flow_with_text {
        a |= 1 << 13;
    }
    if common.allow_overlap {
        a |= 1 << 14;
    }
    if common.size_protect {
        a |= 1 << 20;
    }
    a |= (width_criterion_to_bits(common.width_criterion) & 0x07) << 15;
    a |= (height_criterion_to_bits(common.height_criterion) & 0x03) << 18;
    a |= (text_wrap_to_bits(common.text_wrap) & 0x07) << 21;
    a |= (text_flow_to_bits(common.text_flow) & 0x03) << 24;
    if common.hwp5_gen_shape_attr_bit26 {
        a |= 1 << 26;
    }
    if common.hwp5_gen_shape_attr_bit28 {
        a |= 1 << 28;
    }
    a
}

/// tac/rel_to 마이그레이션 후 stale packed `attr` 동기화.
///
/// [#4400] `document_core/converters/common_obj_attr_writer.rs` 에서 `pack_common_attr_bits`와
/// 함께 이 파일로 이동했다 — 같은 CTRL_HEADER attr 비트 지식(같은 마스크 상수, 같은
/// `vert_rel_to_to_bits`/`horz_rel_to_to_bits`)을 다루는 동종 로직을 한쪽만 옮기면
/// `document_core`에 직렬화 지식이 남고, 그걸 지탱하려고 헬퍼를 `pub(crate)`로 넓혀야
/// 했다(gestell 자기검증에서 지적된 "총체적 독립성 미달"). 이 함수까지 옮기면 그 넓힘이
/// 필요 없어진다 — 아래 두 헬퍼를 다시 private으로 좁힌 이유다.
///
/// 배경 (Issue #3781 실측): `insert_picture_native` 가 `attr` 비트를 seed 하고
/// `migrate_picture_floating_to_inline` 은 **enum 필드만** 갱신한다. 직렬화는
/// `attr != 0` 이면 packed 값을 우선하므로(라운드트립 보존 계약), 마이그레이션된
/// 그림이 HWP 바이너리 왕복에서 floating(Paper) 앵커로 되살아나
/// `treatAsChar=1 + vertRelTo=PAPER` 모순 산출물이 된다 (한글 렌더 깨짐).
/// 앵커 관련 비트(tac bit0 · vert_rel 3-4 · horz_rel 8-9)만 enum 에서 재기입하고,
/// criterion/wrap/flow 등 나머지 비트는 보존한다 (전체 재패킹은 enum 미동기
/// 필드의 정보 손실 위험).
pub(crate) fn sync_anchor_bits(common: &mut CommonObjAttr) {
    if common.attr == 0 {
        return; // 직렬화가 pack_common_attr_bits 로 전량 재패킹 — 손댈 것 없음.
    }
    let mut a = common.attr;
    a &= !(0x01 | (0x03 << 3) | (0x03 << 8));
    if common.treat_as_char {
        a |= 0x01;
    }
    a |= (vert_rel_to_to_bits(common.vert_rel_to) & 0x03) << 3;
    a |= (horz_rel_to_to_bits(common.horz_rel_to) & 0x03) << 8;
    common.attr = a;
}

/// 배치(`text_wrap`)를 packed `attr` 에 되쓴다 — `sync_anchor_bits` 와 같은 이유다.
///
/// 배치를 바꾸는 편집(`ShapeObjCtrlSendBehindText` 계열)이 enum 만 갱신하면 저장에서
/// **묻힌다** — `attr != 0` 이면 직렬화가 packed 값을 우선하기 때문이다. 한글 저장본과
/// 견줘 보고서야 드러났다(오라클은 `attr` 과 `text_wrap` 이 함께 움직이는데 이쪽은 둘 다
/// 그대로였다). 건드리는 것은 **비트 21‥23 뿐**이다.
pub(crate) fn sync_text_wrap_bits(common: &mut CommonObjAttr) {
    if common.attr == 0 {
        return; // 직렬화가 전량 재패킹한다.
    }
    let mut a = common.attr;
    a &= !(0x07 << 21);
    a |= (text_wrap_to_bits(common.text_wrap) & 0x07) << 21;
    common.attr = a;
}

fn vert_rel_to_to_bits(v: VertRelTo) -> u32 {
    match v {
        VertRelTo::Paper => 0,
        VertRelTo::Page => 1,
        VertRelTo::Para => 2,
    }
}

fn vert_align_to_bits(v: VertAlign) -> u32 {
    match v {
        VertAlign::Top => 0,
        VertAlign::Center => 1,
        VertAlign::Bottom => 2,
        VertAlign::Inside => 3,
        VertAlign::Outside => 4,
    }
}

fn horz_rel_to_to_bits(v: HorzRelTo) -> u32 {
    match v {
        HorzRelTo::Paper => 0,
        HorzRelTo::Page => 1,
        HorzRelTo::Column => 2,
        HorzRelTo::Para => 3,
    }
}

fn horz_align_to_bits(v: HorzAlign) -> u32 {
    match v {
        HorzAlign::Left => 0,
        HorzAlign::Center => 1,
        HorzAlign::Right => 2,
        HorzAlign::Inside => 3,
        HorzAlign::Outside => 4,
    }
}

fn width_criterion_to_bits(v: SizeCriterion) -> u32 {
    match v {
        SizeCriterion::Paper => 0,
        SizeCriterion::Page => 1,
        SizeCriterion::Column => 2,
        SizeCriterion::Para => 3,
        SizeCriterion::Absolute => 4,
    }
}

fn height_criterion_to_bits(v: SizeCriterion) -> u32 {
    match v {
        SizeCriterion::Paper => 0,
        SizeCriterion::Page => 1,
        // height 는 Absolute 만 의미 있음 (parser bit 18-19, 2비트만 사용)
        _ => 2,
    }
}

fn text_wrap_to_bits(v: TextWrap) -> u32 {
    // hwplib 기준: 0=어울림(Square), 1=자리차지(TopAndBottom), 2=글뒤로(BehindText), 3=글앞으로(InFrontOfText)
    // Tight/Through 는 HWP 5.0 에 직접 매핑이 없어 Square 로 폴백.
    match v {
        TextWrap::Square => 0,
        TextWrap::Tight => 0,
        TextWrap::Through => 0,
        TextWrap::TopAndBottom => 1,
        TextWrap::BehindText => 2,
        TextWrap::InFrontOfText => 3,
    }
}

fn text_flow_to_bits(v: TextFlow) -> u32 {
    match v {
        TextFlow::BothSides => 0,
        TextFlow::LeftOnly => 1,
        TextFlow::RightOnly => 2,
        TextFlow::LargestOnly => 3,
    }
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
    // [#2727] 종전 상수 0 하드코딩 → IR 보존값 방출. bit0(수식 범위, HWPX lineMode)
    // 이 저장마다 CHAR 로 초기화되던 손실 제거.
    w.write_u32(eq.attr).unwrap();
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
