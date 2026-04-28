//! section*.xml 파싱 — HWPX 섹션 본문을 Section 모델로 변환
//!
//! 섹션 XML의 문단(<hp:p>), 텍스트 런(<hp:run>), 표(<hp:tbl>),
//! 이미지(<hp:pic>) 등을 기존 Document 모델로 변환한다.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::model::control::{
    Control, Equation, PageHide, PageNumberPos, AutoNumber, AutoNumberType,
    NewNumber, Bookmark, Field, FieldType, HiddenComment, Ruby, CharOverlap,
    FormObject, FormType,
};
use crate::model::header_footer::{Header, Footer, HeaderFooterApply};
use crate::model::footnote::{Footnote, Endnote};
use crate::model::document::{Section, SectionDef};
use crate::model::image::{ImageAttr, ImageEffect, CropInfo};
use crate::model::shape::{
    CommonObjAttr, ShapeComponentAttr, DrawingObjAttr, TextBox, ShapeObject,
    RectangleShape, EllipseShape, LineShape, ArcShape, PolygonShape, CurveShape, GroupShape,
    VertRelTo, HorzRelTo, VertAlign, HorzAlign, TextWrap,
};
use crate::model::style::{ShapeBorderLine, Fill};
use crate::model::page::{PageDef, ColumnDef, ColumnType, ColumnDirection};
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::model::HwpUnit16;

use super::HwpxError;
use super::utils::{local_name, attr_str, parse_u8, parse_i8, parse_u16, parse_i16, parse_u32, parse_i32, parse_color, parse_bool, skip_element};

/// section*.xml을 파싱하여 Section 모델로 변환한다.
pub fn parse_hwpx_section(xml: &str) -> Result<Section, HwpxError> {
    let mut section = Section::default();
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let ename = e.name(); let local = local_name(ename.as_ref());
                match local {
                    b"p" => {
                        // 최상위 문단
                        let (para, sec_def_opt) = parse_paragraph(e, &mut reader)?;
                        if let Some(sec_def) = sec_def_opt {
                            section.section_def = sec_def;
                        }
                        section.paragraphs.push(para);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("section: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(section)
}

// ─── SectionDef / PageDef ───

fn parse_section_def_start(e: &quick_xml::events::BytesStart, sec_def: &mut SectionDef) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"textDirection" => {
                let val = attr_str(&attr);
                sec_def.text_direction = if val == "VERTICAL" { 1 } else { 0 };
            }
            b"tabStop" => {
                sec_def.default_tab_spacing = parse_u32(&attr);
            }
            _ => {}
        }
    }
}

fn parse_page_pr(e: &quick_xml::events::BytesStart, page: &mut PageDef) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"width" => page.width = parse_u32(&attr),
            b"height" => page.height = parse_u32(&attr),
            // HWPX에서는 landscape 플래그를 false로 유지한다.
            // HWPX의 width/height는 이미 실제 용지 방향대로 저장되어 있어
            // 렌더러가 추가로 교환(swap)할 필요가 없다.
            // HWP 바이너리는 항상 짧은변=width, 긴변=height로 저장하고
            // landscape=true일 때 렌더러가 교환하지만, HWPX는 다른 규약을 따른다.
            b"landscape" => { /* 무시: landscape = false 유지 */ }
            _ => {}
        }
    }
}

fn parse_page_margin(e: &quick_xml::events::BytesStart, page: &mut PageDef) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"left" => page.margin_left = parse_u32(&attr),
            b"right" => page.margin_right = parse_u32(&attr),
            b"top" => page.margin_top = parse_u32(&attr),
            b"bottom" => page.margin_bottom = parse_u32(&attr),
            b"header" => page.margin_header = parse_u32(&attr),
            b"footer" => page.margin_footer = parse_u32(&attr),
            b"gutter" => page.margin_gutter = parse_u32(&attr),
            _ => {}
        }
    }
}

// ─── Paragraph ───

fn parse_paragraph(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<(Paragraph, Option<SectionDef>), HwpxError> {
    let mut para = Paragraph::default();
    let mut sec_def: Option<SectionDef> = None;

    // 문단 어트리뷰트
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"paraPrIDRef" => para.para_shape_id = parse_u16(&attr),
            b"styleIDRef" => para.style_id = parse_u8(&attr),
            b"columnBreak" => {
                if parse_u8(&attr) == 1 {
                    para.column_type = crate::model::paragraph::ColumnBreakType::Column;
                }
            }
            b"pageBreak" => {
                if parse_u8(&attr) == 1 {
                    para.column_type = crate::model::paragraph::ColumnBreakType::Page;
                }
            }
            _ => {}
        }
    }

    // 문단 내용 파싱
    let mut buf = Vec::new();
    let mut text_parts: Vec<String> = Vec::new();
    let mut current_char_shape_id: u32 = 0;
    let mut char_shape_changes: Vec<(u32, u32)> = Vec::new(); // (utf16_pos, char_shape_id)

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"run" => {
                        // 런 시작: charPrIDRef 읽기
                        for attr in ce.attributes().flatten() {
                            if attr.key.as_ref() == b"charPrIDRef" {
                                current_char_shape_id = parse_u32(&attr);
                            }
                        }
                        // 현재 UTF-16 위치에서 글자모양 변경 기록
                        let utf16_pos = calc_utf16_len_from_parts(&text_parts);
                        char_shape_changes.push((utf16_pos, current_char_shape_id));
                    }
                    b"t" => {
                        // 텍스트 읽기 (탭 확장 데이터 포함)
                        let (text, tab_exts) = read_text_content_with_tabs(reader)?;
                        text_parts.push(text);
                        para.tab_extended.extend(tab_exts);
                    }
                    b"tbl" => {
                        // 표 파싱
                        let table = parse_table(ce, reader)?;
                        // 표 위치에 제어 문자(0x0002) 삽입
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(Control::Table(Box::new(table)));
                    }
                    b"pic" => {
                        // 이미지 파싱
                        let pic = parse_picture(ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(pic);
                    }
                    b"switch" => {
                        // <hp:switch> — OOXML 차트 또는 OLE fallback
                        // 구조: <hp:switch>
                        //         <hp:case hp:required-namespace="...ooxmlchart">
                        //           <hp:chart chartIDRef="Chart/chartN.xml" .../>
                        //         </hp:case>
                        //         <hp:default><hp:ole .../></hp:default>
                        //       </hp:switch>
                        if let Some(ctrl) = parse_switch_chart_or_ole(reader)? {
                            text_parts.push("\u{0002}".to_string());
                            para.controls.push(ctrl);
                        }
                    }
                    b"chart" => {
                        // <hp:chart> 직접 출현 (switch 없이) — 아직 보지 못한 변형. 안전 경로.
                        if let Some(ctrl) = parse_hp_chart_element(ce, reader)? {
                            text_parts.push("\u{0002}".to_string());
                            para.controls.push(ctrl);
                        }
                    }
                    b"ole" => {
                        // <hp:ole> 직접 출현 (switch 없이)
                        if let Some(ctrl) = parse_hp_ole_element(ce, reader)? {
                            text_parts.push("\u{0002}".to_string());
                            para.controls.push(ctrl);
                        }
                    }
                    b"secPr" => {
                        // 문단 내 섹션 정의 파싱
                        let mut sd = SectionDef::default();
                        parse_section_def_start(ce, &mut sd);
                        let col_def_opt = parse_sec_pr_children(reader, &mut sd)?;
                        sec_def = Some(sd);
                        // colPr이 있으면 ColumnDef 컨트롤 추가 (초기 단 정의)
                        if let Some(cd) = col_def_opt {
                            para.controls.push(Control::ColumnDef(cd));
                        }
                    }
                    b"linesegarray" => {
                        // lineseg 배열 파싱
                        parse_lineseg_array(reader, &mut para)?;
                    }
                    b"rect" | b"ellipse" | b"line" | b"arc" | b"polygon" | b"curve" => {
                        // 그리기 객체 파싱
                        let shape = parse_shape_object(local, ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(shape);
                    }
                    b"container" => {
                        // 묶음(그룹) 객체 파싱
                        let group = parse_container(ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(group);
                    }
                    b"ctrl" => {
                        parse_ctrl(ce, reader, &mut para.controls, &mut text_parts)?;
                    }
                    b"compose" => {
                        // 글자겹침 (CharOverlap)
                        let ctrl = parse_compose(ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    b"dutmal" => {
                        // 덧말 (Ruby)
                        let ctrl = parse_dutmal(ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    b"equation" => {
                        // 수식 — 개체(ShapeObject)로 처리
                        let ctrl = parse_equation(ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    b"btn" => {
                        let ctrl = parse_form_object(FormType::PushButton, ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    b"checkBtn" => {
                        let ctrl = parse_form_object(FormType::CheckBox, ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    b"radioBtn" => {
                        let ctrl = parse_form_object(FormType::RadioButton, ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    b"comboBox" => {
                        let ctrl = parse_form_object(FormType::ComboBox, ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    b"edit" => {
                        let ctrl = parse_form_object(FormType::Edit, ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(ctrl);
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"lineBreak" | b"softHyphen" => {
                        text_parts.push("\n".to_string());
                    }
                    b"columnBreak" => {
                        text_parts.push("\n".to_string());
                    }
                    b"tab" => {
                        text_parts.push("\t".to_string());
                        // HWPX 인라인 탭 속성 파싱 → tab_extended에 저장
                        let mut ext = [0u16; 7];
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => ext[0] = parse_u16(&attr),
                                b"leader" => ext[1] = parse_u16(&attr),
                                b"type" => ext[2] = parse_u16(&attr),
                                _ => {}
                            }
                        }
                        para.tab_extended.push(ext);
                    }
                    b"lineseg" => {
                        // 단독 lineseg (linesegarray 밖에 나올 경우)
                        para.line_segs.push(parse_lineseg_element(ce));
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name(); if local_name(eename.as_ref()) == b"p" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("paragraph: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // 텍스트 조립: 제어 문자(\u{0002}, \u{0003}, \u{0004})는 HWP와 동일하게 텍스트에서 제외
    // HWP에서 컨트롤 위치는 char_offsets의 갭으로 표현되므로 원본 순서를 유지해 계산한다.
    let mut visual_text = String::new();
    let mut char_offsets: Vec<u32> = Vec::new();
    let mut utf16_pos: u32 = 0;

    for part in &text_parts {
        match part.as_str() {
            "\u{0002}" | "\u{0003}" | "\u{0004}" => {
                utf16_pos += 8;
            }
            _ => {
                for c in part.chars() {
                    char_offsets.push(utf16_pos);
                    visual_text.push(c);
                    let width = if c == '\t' { 8 } else if (c as u32) > 0xFFFF { 2 } else { 1 };
                    utf16_pos += width;
                }
            }
        }
    }

    para.text = visual_text;
    para.char_offsets = char_offsets;
    para.char_count = utf16_pos + 1; // +1 for 끝 마커
    para.has_para_text = !para.text.is_empty() || !para.controls.is_empty();

    // char_shapes는 원본 문단 순서(text_parts)를 기준으로 계산한 위치를 그대로 사용한다.
    para.char_shapes = char_shape_changes.into_iter()
        .map(|(pos, id)| CharShapeRef { start_pos: pos, char_shape_id: id })
        .collect();

    // 기본 line_seg (빈 문단이라도 최소 1개)
    if para.line_segs.is_empty() {
        para.line_segs.push(LineSeg {
            text_start: 0,
            tag: 0x00060000,
            ..Default::default()
        });
    }

    Ok((para, sec_def))
}

/// secPr의 자식 요소들 (pagePr, margin, colPr 등) 파싱
/// 반환: 파싱된 ColumnDef (없으면 None)
fn parse_sec_pr_children(reader: &mut Reader<&[u8]>, sec_def: &mut SectionDef) -> Result<Option<ColumnDef>, HwpxError> {
    let mut buf = Vec::new();
    let mut col_def: Option<ColumnDef> = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let ename = e.name(); let local = local_name(ename.as_ref());
                match local {
                    b"pagePr" => parse_page_pr(e, &mut sec_def.page_def),
                    b"margin" => parse_page_margin(e, &mut sec_def.page_def),
                    b"colPr" => { col_def = Some(parse_col_pr(e)); }
                    b"startNum" => parse_start_num(e, sec_def),
                    b"visibility" => parse_visibility(e, sec_def),
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let ename = e.name(); let local = local_name(ename.as_ref());
                match local {
                    b"pagePr" => parse_page_pr(e, &mut sec_def.page_def),
                    b"margin" => parse_page_margin(e, &mut sec_def.page_def),
                    b"colPr" => { col_def = Some(parse_col_pr(e)); }
                    b"startNum" => parse_start_num(e, sec_def),
                    b"visibility" => parse_visibility(e, sec_def),
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let ename = e.name();
                if local_name(ename.as_ref()) == b"secPr" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("secPr: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(col_def)
}

/// <hp:startNum> 요소 파싱
fn parse_start_num(e: &quick_xml::events::BytesStart, sec_def: &mut SectionDef) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"page" => sec_def.page_num = parse_u16(&attr),
            b"pic" => sec_def.picture_num = parse_u16(&attr),
            b"tbl" => sec_def.table_num = parse_u16(&attr),
            b"equation" => sec_def.equation_num = parse_u16(&attr),
            _ => {}
        }
    }
}

/// <hp:visibility> 요소 파싱
fn parse_visibility(e: &quick_xml::events::BytesStart, sec_def: &mut SectionDef) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"hideFirstHeader" => sec_def.hide_header = attr_str(&attr) == "1",
            b"hideFirstFooter" => sec_def.hide_footer = attr_str(&attr) == "1",
            b"hideFirstMasterPage" => sec_def.hide_master_page = attr_str(&attr) == "1",
            b"border" => sec_def.hide_border = attr_str(&attr) == "HIDE_ALL",
            b"fill" => sec_def.hide_fill = attr_str(&attr) == "HIDE_ALL",
            b"hideFirstEmptyLine" => sec_def.hide_empty_line = attr_str(&attr) == "1",
            _ => {}
        }
    }
}

/// <hp:colPr> 요소 파싱 → ColumnDef
fn parse_col_pr(e: &quick_xml::events::BytesStart) -> ColumnDef {
    let mut cd = ColumnDef::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => {
                cd.column_type = match attr_str(&attr).as_str() {
                    "NEWSPAPER" => ColumnType::Normal,
                    "BalancedNewspaper" => ColumnType::Distribute,
                    "Parallel" => ColumnType::Parallel,
                    _ => ColumnType::Normal,
                };
            }
            b"layout" => {
                cd.direction = match attr_str(&attr).as_str() {
                    "RIGHT" => ColumnDirection::RightToLeft,
                    _ => ColumnDirection::LeftToRight,
                };
            }
            b"colCount" => cd.column_count = parse_u16(&attr),
            b"sameSz" => cd.same_width = parse_u8(&attr) != 0,
            b"sameGap" => cd.spacing = parse_i16(&attr),
            _ => {}
        }
    }
    cd
}

/// <hp:linesegarray> 내부의 <hp:lineseg> 요소들을 파싱한다.
fn parse_lineseg_array(reader: &mut Reader<&[u8]>, para: &mut Paragraph) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                let ename = e.name(); let local = local_name(ename.as_ref());
                if local == b"lineseg" {
                    para.line_segs.push(parse_lineseg_element(e));
                }
            }
            Ok(Event::End(ref e)) => {
                let ename = e.name();
                if local_name(ename.as_ref()) == b"linesegarray" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("linesegarray: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// 단일 <hp:lineseg> 요소의 속성을 LineSeg로 변환한다.
fn parse_lineseg_element(e: &quick_xml::events::BytesStart) -> LineSeg {
    let mut seg = LineSeg::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"textpos" => seg.text_start = parse_u32(&attr),
            b"vertpos" => seg.vertical_pos = parse_i32(&attr),
            b"vertsize" => seg.line_height = parse_i32(&attr),
            b"textheight" => seg.text_height = parse_i32(&attr),
            b"baseline" => seg.baseline_distance = parse_i32(&attr),
            b"spacing" => seg.line_spacing = parse_i32(&attr),
            b"horzpos" => seg.column_start = parse_i32(&attr),
            b"horzsize" => seg.segment_width = parse_i32(&attr),
            b"flags" => seg.tag = parse_u32(&attr),
            _ => {}
        }
    }
    seg
}

/// <hp:t> 텍스트 컨텐츠를 읽는다.
/// 탭 확장 데이터도 함께 반환 (HWPX 인라인 탭의 leader/type/width)
fn read_text_content(reader: &mut Reader<&[u8]>) -> Result<String, HwpxError> {
    let (text, _) = read_text_content_with_tabs(reader)?;
    Ok(text)
}

fn read_text_content_with_tabs(reader: &mut Reader<&[u8]>) -> Result<(String, Vec<[u16; 7]>), HwpxError> {
    let mut text = String::new();
    let mut tab_ext_buf: Vec<[u16; 7]> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                text.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::End(ref e)) => {
                let tn = e.name(); if local_name(tn.as_ref()) == b"t" {
                    break;
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"lineBreak" | b"columnBreak" => text.push('\n'),
                    b"tab" => {
                        text.push('\t');
                        // HWPX 인라인 탭 속성 → tab_ext_buf에 임시 저장
                        let mut ext = [0u16; 7];
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => ext[0] = parse_u16(&attr),
                                b"leader" => ext[1] = parse_u16(&attr),
                                b"type" => ext[2] = parse_u16(&attr),
                                _ => {}
                            }
                        }
                        tab_ext_buf.push(ext);
                    }
                    b"nbSpace" => text.push('\u{00A0}'),
                    b"fwSpace" => text.push('\u{2007}'),
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("text: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok((text, tab_ext_buf))
}

// ─── Table ───

fn parse_table(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Table, HwpxError> {
    let mut table = Table::default();

    // 표 기본 속성
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"rowCnt" => table.row_count = parse_u16(&attr),
            b"colCnt" => table.col_count = parse_u16(&attr),
            b"cellSpacing" => table.cell_spacing = parse_i16(&attr),
            b"borderFillIDRef" => table.border_fill_id = parse_u16(&attr),
            b"pageBreak" => {
                let val = attr_str(&attr);
                table.page_break = match val.as_str() {
                    "CELL" | "CELL_BREAK" => TablePageBreak::CellBreak,
                    "ROW" | "ROW_BREAK" => TablePageBreak::RowBreak,
                    _ => TablePageBreak::None,
                };
            }
            b"repeatHeader" => {
                table.repeat_header = attr_str(&attr) == "1";
            }
            b"textWrap" => {
                table.common.text_wrap = match attr_str(&attr).as_str() {
                    "TOP_AND_BOTTOM" => crate::model::shape::TextWrap::TopAndBottom,
                    "BEHIND_TEXT" => crate::model::shape::TextWrap::BehindText,
                    "IN_FRONT_OF_TEXT" => crate::model::shape::TextWrap::InFrontOfText,
                    _ => crate::model::shape::TextWrap::Square,
                };
            }
            _ => {}
        }
    }

    // 표 내용 파싱 (행/셀)
    let mut buf = Vec::new();
    let mut current_row: u16 = 0;
    let mut row_sizes: Vec<HwpUnit16> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"tr" => {
                        // 새 행
                    }
                    b"tc" => {
                        // 셀 파싱
                        let cell = parse_table_cell(ce, reader, current_row)?;
                        table.cells.push(cell);
                    }
                    b"caption" => {
                        let caption = parse_table_caption(ce, reader)?;
                        table.caption = Some(caption);
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"sz" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => { table.common.width = parse_u32(&attr); }
                                b"height" => { table.common.height = parse_u32(&attr); }
                                _ => {}
                            }
                        }
                    }
                    b"pos" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"treatAsChar" => {
                                    table.common.treat_as_char = attr_str(&attr) == "1" || attr_str(&attr) == "true";
                                }
                                b"vertRelTo" => {
                                    table.common.vert_rel_to = match attr_str(&attr).as_str() {
                                        "PAPER" => crate::model::shape::VertRelTo::Paper,
                                        "PAGE" => crate::model::shape::VertRelTo::Page,
                                        _ => crate::model::shape::VertRelTo::Para,
                                    };
                                }
                                b"horzRelTo" => {
                                    table.common.horz_rel_to = match attr_str(&attr).as_str() {
                                        "PAPER" => crate::model::shape::HorzRelTo::Paper,
                                        "PAGE" => crate::model::shape::HorzRelTo::Page,
                                        "COLUMN" => crate::model::shape::HorzRelTo::Column,
                                        _ => crate::model::shape::HorzRelTo::Para,
                                    };
                                }
                                b"vertAlign" => {
                                    table.common.vert_align = match attr_str(&attr).as_str() {
                                        "TOP" => crate::model::shape::VertAlign::Top,
                                        "CENTER" => crate::model::shape::VertAlign::Center,
                                        "BOTTOM" => crate::model::shape::VertAlign::Bottom,
                                        "INSIDE" => crate::model::shape::VertAlign::Inside,
                                        "OUTSIDE" => crate::model::shape::VertAlign::Outside,
                                        _ => crate::model::shape::VertAlign::Top,
                                    };
                                }
                                b"horzAlign" => {
                                    table.common.horz_align = match attr_str(&attr).as_str() {
                                        "LEFT" => crate::model::shape::HorzAlign::Left,
                                        "CENTER" => crate::model::shape::HorzAlign::Center,
                                        "RIGHT" => crate::model::shape::HorzAlign::Right,
                                        "INSIDE" => crate::model::shape::HorzAlign::Inside,
                                        "OUTSIDE" => crate::model::shape::HorzAlign::Outside,
                                        _ => crate::model::shape::HorzAlign::Left,
                                    };
                                }
                                b"vertOffset" => { table.common.vertical_offset = parse_i32(&attr) as u32; }
                                b"horzOffset" => { table.common.horizontal_offset = parse_i32(&attr) as u32; }
                                _ => {}
                            }
                        }
                    }
                    b"outMargin" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => table.outer_margin_left = parse_i16(&attr),
                                b"right" => table.outer_margin_right = parse_i16(&attr),
                                b"top" => table.outer_margin_top = parse_i16(&attr),
                                b"bottom" => table.outer_margin_bottom = parse_i16(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"inMargin" => {
                        // 표 안쪽 여백 → table.padding
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => table.padding.left = parse_i16(&attr),
                                b"right" => table.padding.right = parse_i16(&attr),
                                b"top" => table.padding.top = parse_i16(&attr),
                                b"bottom" => table.padding.bottom = parse_i16(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"cellzone" => {
                        let mut zone = crate::model::table::TableZone::default();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"startColAddr" => zone.start_col = parse_u16(&attr),
                                b"startRowAddr" => zone.start_row = parse_u16(&attr),
                                b"endColAddr" => zone.end_col = parse_u16(&attr),
                                b"endRowAddr" => zone.end_row = parse_u16(&attr),
                                b"borderFillIDRef" => zone.border_fill_id = parse_u16(&attr),
                                _ => {}
                            }
                        }
                        table.zones.push(zone);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name(); let local = local_name(eename.as_ref());
                match local {
                    b"tr" => current_row += 1,
                    b"tbl" => break,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("table: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // row_sizes 설정 (행별 셀 높이의 최대값)
    for r in 0..table.row_count {
        let max_h = table.cells.iter()
            .filter(|c| c.row == r && c.row_span == 1)
            .map(|c| c.height as i16)
            .max()
            .unwrap_or(0);
        row_sizes.push(max_h);
    }
    table.row_sizes = row_sizes;

    table.rebuild_grid();
    Ok(table)
}

/// 표 캡션 파싱
fn parse_table_caption(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<crate::model::shape::Caption, HwpxError> {
    use crate::model::shape::{Caption, CaptionDirection};

    let mut caption = Caption::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"side" => {
                caption.direction = match attr_str(&attr).as_str() {
                    "LEFT" => CaptionDirection::Left,
                    "RIGHT" => CaptionDirection::Right,
                    "TOP" => CaptionDirection::Top,
                    "BOTTOM" => CaptionDirection::Bottom,
                    _ => CaptionDirection::Bottom,
                };
            }
            b"gap" => caption.spacing = parse_i16(&attr),
            b"width" => caption.width = parse_i32(&attr) as u32,
            b"lastWidth" => caption.max_width = parse_i32(&attr) as u32,
            b"fullSz" => caption.include_margin = attr_str(&attr) == "1",
            _ => {}
        }
    }

    // subList 내 문단 파싱
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"p" {
                    let (para, _) = parse_paragraph(ce, reader)?;
                    caption.paragraphs.push(para);
                }
            }
            Ok(Event::End(ref end)) => {
                if local_name(end.name().as_ref()) == b"caption" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("caption: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(caption)
}

fn parse_table_cell(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    current_row: u16,
) -> Result<Cell, HwpxError> {
    let mut cell = Cell::default();
    cell.row = current_row;
    cell.col_span = 1;
    cell.row_span = 1;

    // <hp:tc> 요소 자체의 속성 파싱
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"borderFillIDRef" => cell.border_fill_id = parse_u16(&attr),
            b"header" => cell.is_header = attr_str(&attr) == "1",
            _ => {}
        }
    }

    // 셀 자식 요소 파싱
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"cellAddr" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"colAddr" => { cell.col = parse_u16(&attr); }
                                b"rowAddr" => cell.row = parse_u16(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"cellSpan" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"colSpan" => cell.col_span = parse_u16(&attr).max(1),
                                b"rowSpan" => cell.row_span = parse_u16(&attr).max(1),
                                _ => {}
                            }
                        }
                    }
                    b"cellSz" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => cell.width = parse_u32(&attr),
                                b"height" => cell.height = parse_u32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"cellMargin" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => cell.padding.left = parse_i16(&attr),
                                b"right" => cell.padding.right = parse_i16(&attr),
                                b"top" => cell.padding.top = parse_i16(&attr),
                                b"bottom" => cell.padding.bottom = parse_i16(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"tcPr" => {
                        // 셀 속성 (legacy format)
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"borderFillIDRef" => cell.border_fill_id = parse_u16(&attr),
                                b"textDirection" => {
                                    let val = attr_str(&attr);
                                    cell.text_direction = if val == "VERTICAL" { 1 } else { 0 };
                                }
                                b"vAlign" => {
                                    cell.vertical_align = match attr_str(&attr).as_str() {
                                        "CENTER" => VerticalAlign::Center,
                                        "BOTTOM" => VerticalAlign::Bottom,
                                        _ => VerticalAlign::Top,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    b"subList" => {
                        // subList: vertAlign 속성 파싱
                        for attr in ce.attributes().flatten() {
                            if attr.key.as_ref() == b"vertAlign" {
                                cell.vertical_align = match attr_str(&attr).as_str() {
                                    "CENTER" => VerticalAlign::Center,
                                    "BOTTOM" => VerticalAlign::Bottom,
                                    _ => VerticalAlign::Top,
                                };
                            }
                        }
                    }
                    b"cellPr" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"borderFillIDRef" => cell.border_fill_id = parse_u16(&attr),
                                b"textDirection" => {
                                    let val = attr_str(&attr);
                                    cell.text_direction = if val == "VERTICAL" { 1 } else { 0 };
                                }
                                b"vAlign" => {
                                    cell.vertical_align = match attr_str(&attr).as_str() {
                                        "CENTER" => VerticalAlign::Center,
                                        "BOTTOM" => VerticalAlign::Bottom,
                                        _ => VerticalAlign::Top,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    b"p" => {
                        // 셀 내 문단 (secDef는 무시)
                        let (para, _) = parse_paragraph(ce, reader)?;
                        cell.paragraphs.push(para);
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"cellAddr" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"colAddr" => { cell.col = parse_u16(&attr); }
                                b"rowAddr" => cell.row = parse_u16(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"cellSpan" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"colSpan" => cell.col_span = parse_u16(&attr).max(1),
                                b"rowSpan" => cell.row_span = parse_u16(&attr).max(1),
                                _ => {}
                            }
                        }
                    }
                    b"cellSz" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => cell.width = parse_u32(&attr),
                                b"height" => cell.height = parse_u32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"cellMargin" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => cell.padding.left = parse_i16(&attr),
                                b"right" => cell.padding.right = parse_i16(&attr),
                                b"top" => cell.padding.top = parse_i16(&attr),
                                b"bottom" => cell.padding.bottom = parse_i16(&attr),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name(); if local_name(eename.as_ref()) == b"tc" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("tc: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // 셀에 문단이 없으면 빈 문단 추가
    if cell.paragraphs.is_empty() {
        cell.paragraphs.push(Paragraph::new_empty());
    }

    Ok(cell)
}

// ─── Picture ───

fn parse_picture(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut img_attr = ImageAttr::default();
    let mut common = CommonObjAttr::default();
    let mut shape_attr = ShapeComponentAttr::default();
    let mut crop = CropInfo::default();
    let mut padding = crate::model::Padding::default();

    // <hp:pic> 요소 자체의 속성 파싱
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"zOrder" => common.z_order = parse_i32(&attr),
            b"textWrap" => {
                common.text_wrap = match attr_str(&attr).as_str() {
                    "SQUARE" => TextWrap::Square,
                    "TIGHT" => TextWrap::Tight,
                    "THROUGH" => TextWrap::Through,
                    "TOP_AND_BOTTOM" => TextWrap::TopAndBottom,
                    "BEHIND_TEXT" => TextWrap::BehindText,
                    "IN_FRONT_OF_TEXT" => TextWrap::InFrontOfText,
                    _ => TextWrap::Square,
                };
            }
            b"instid" => common.instance_id = parse_u32(&attr),
            b"groupLevel" => shape_attr.group_level = attr_str(&attr).parse().unwrap_or(0),
            _ => {}
        }
    }

    // 이미지 속성 읽기
    let mut has_pos = false; // <pos> 파싱 여부 — <offset>이 덮어쓰지 않도록 방지
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"sz" => {
                        // 최종 표시 크기 (최우선)
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => { let v = parse_u32(&attr); if v > 0 { common.width = v; } }
                                b"height" => { let v = parse_u32(&attr); if v > 0 { common.height = v; } }
                                _ => {}
                            }
                        }
                    }
                    b"curSz" => {
                        // 현재 크기 → common + shape_attr.current_width/height
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.current_width = v;
                                    if v > 0 { common.width = v; }
                                }
                                b"height" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.current_height = v;
                                    if v > 0 { common.height = v; }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"orgSz" => {
                        // 원본 크기 → shape_attr.original_width/height (렌더러 이미지 Fill 크기에 사용)
                        // curSz/sz가 없을 때 common.width/height 폴백으로도 사용
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.original_width = v;
                                    if common.width == 0 { common.width = v; }
                                }
                                b"height" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.original_height = v;
                                    if common.height == 0 { common.height = v; }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"pos" => {
                        has_pos = true;
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"treatAsChar" => {
                                    common.treat_as_char = attr_str(&attr) == "1" || attr_str(&attr) == "true";
                                }
                                b"vertRelTo" => {
                                    common.vert_rel_to = match attr_str(&attr).as_str() {
                                        "PAPER" => VertRelTo::Paper,
                                        "PAGE" => VertRelTo::Page,
                                        "PARA" => VertRelTo::Para,
                                        _ => VertRelTo::Para,
                                    };
                                }
                                b"horzRelTo" => {
                                    common.horz_rel_to = match attr_str(&attr).as_str() {
                                        "PAPER" => HorzRelTo::Paper,
                                        "PAGE" => HorzRelTo::Page,
                                        "COLUMN" => HorzRelTo::Column,
                                        "PARA" => HorzRelTo::Para,
                                        _ => HorzRelTo::Para,
                                    };
                                }
                                b"vertAlign" => {
                                    common.vert_align = match attr_str(&attr).as_str() {
                                        "TOP" => VertAlign::Top,
                                        "CENTER" => VertAlign::Center,
                                        "BOTTOM" => VertAlign::Bottom,
                                        "INSIDE" => VertAlign::Inside,
                                        "OUTSIDE" => VertAlign::Outside,
                                        _ => VertAlign::Top,
                                    };
                                }
                                b"horzAlign" => {
                                    common.horz_align = match attr_str(&attr).as_str() {
                                        "LEFT" => HorzAlign::Left,
                                        "CENTER" => HorzAlign::Center,
                                        "RIGHT" => HorzAlign::Right,
                                        "INSIDE" => HorzAlign::Inside,
                                        "OUTSIDE" => HorzAlign::Outside,
                                        _ => HorzAlign::Left,
                                    };
                                }
                                b"vertOffset" => common.vertical_offset = parse_i32(&attr) as u32,
                                b"horzOffset" => common.horizontal_offset = parse_i32(&attr) as u32,
                                _ => {}
                            }
                        }
                    }
                    b"outMargin" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => common.margin.left = parse_i16(&attr),
                                b"right" => common.margin.right = parse_i16(&attr),
                                b"top" => common.margin.top = parse_i16(&attr),
                                b"bottom" => common.margin.bottom = parse_i16(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"inMargin" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => padding.left = parse_i16(&attr),
                                b"right" => padding.right = parse_i16(&attr),
                                b"top" => padding.top = parse_i16(&attr),
                                b"bottom" => padding.bottom = parse_i16(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"imgClip" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => crop.left = parse_i32(&attr),
                                b"right" => crop.right = parse_i32(&attr),
                                b"top" => crop.top = parse_i32(&attr),
                                b"bottom" => crop.bottom = parse_i32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"img" | b"image" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"binaryItemIDRef" => {
                                    // "image1" → BinData ID 1
                                    let val = attr_str(&attr);
                                    let num: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                                    img_attr.bin_data_id = num.parse().unwrap_or(0);
                                }
                                b"bright" => img_attr.brightness = parse_i8(&attr),
                                b"contrast" => img_attr.contrast = parse_i8(&attr),
                                b"effect" => {
                                    img_attr.effect = match attr_str(&attr).as_str() {
                                        "REAL_PIC" => ImageEffect::RealPic,
                                        "GRAY_SCALE" => ImageEffect::GrayScale,
                                        "BLACK_WHITE" => ImageEffect::BlackWhite,
                                        _ => ImageEffect::RealPic,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    b"offset" => {
                        // <offset>은 개체 내부의 shape-transform 오프셋이다.
                        // shape_attr.offset_x/offset_y에 항상 저장 (그룹 내부 좌표용).
                        // <pos>가 이미 파싱된 경우 페이지 레벨 좌표(vertOffset/horzOffset)는
                        // 덮어쓰지 않는다. <pos>가 없는 경우에만 폴백으로 적용한다.
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.offset_x = v as i32;
                                    if !has_pos {
                                        common.horizontal_offset = v;
                                    }
                                }
                                b"y" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.offset_y = v as i32;
                                    if !has_pos {
                                        common.vertical_offset = v;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"renderingInfo" => {
                        // 그룹 내 자식의 아핀 변환 행렬 파싱
                        parse_rendering_info(reader, &mut shape_attr)?;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name(); if local_name(eename.as_ref()) == b"pic" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("pic: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    let mut pic = crate::model::image::Picture::default();
    pic.image_attr = img_attr;
    pic.common = common;
    pic.shape_attr = shape_attr;
    pic.crop = crop;
    pic.padding = padding;

    Ok(Control::Picture(Box::new(pic)))
}

// ─── 그리기 객체 공통 속성 파싱 ───

/// `<hp:pic>`, `<hp:rect>`, `<hp:container>` 등 개체의 공통 속성을 요소 속성에서 파싱한다.
fn parse_object_element_attrs(
    e: &quick_xml::events::BytesStart,
    common: &mut CommonObjAttr,
    shape_attr: &mut ShapeComponentAttr,
) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"zOrder" => common.z_order = parse_i32(&attr),
            b"textWrap" => {
                common.text_wrap = match attr_str(&attr).as_str() {
                    "SQUARE" => TextWrap::Square,
                    "TIGHT" => TextWrap::Tight,
                    "THROUGH" => TextWrap::Through,
                    "TOP_AND_BOTTOM" => TextWrap::TopAndBottom,
                    "BEHIND_TEXT" => TextWrap::BehindText,
                    "IN_FRONT_OF_TEXT" => TextWrap::InFrontOfText,
                    _ => TextWrap::Square,
                };
            }
            b"instid" => common.instance_id = parse_u32(&attr),
            b"groupLevel" => shape_attr.group_level = attr_str(&attr).parse().unwrap_or(0),
            _ => {}
        }
    }
}

/// 개체 자식 요소에서 공통 레이아웃 속성(pos, sz, curSz, orgSz, offset, outMargin)을 파싱한다.
fn parse_object_layout_child(
    local: &[u8],
    ce: &quick_xml::events::BytesStart,
    common: &mut CommonObjAttr,
    shape_attr: &mut ShapeComponentAttr,
    has_pos: &mut bool,
) {
    match local {
        b"sz" => {
            for attr in ce.attributes().flatten() {
                match attr.key.as_ref() {
                    b"width" => { let v = parse_u32(&attr); if v > 0 { common.width = v; } }
                    b"height" => { let v = parse_u32(&attr); if v > 0 { common.height = v; } }
                    _ => {}
                }
            }
        }
        b"curSz" => {
            for attr in ce.attributes().flatten() {
                match attr.key.as_ref() {
                    b"width" => {
                        let v = parse_u32(&attr);
                        shape_attr.current_width = v;
                        if v > 0 { common.width = v; }
                    }
                    b"height" => {
                        let v = parse_u32(&attr);
                        shape_attr.current_height = v;
                        if v > 0 { common.height = v; }
                    }
                    _ => {}
                }
            }
        }
        b"orgSz" => {
            for attr in ce.attributes().flatten() {
                match attr.key.as_ref() {
                    b"width" => {
                        let v = parse_u32(&attr);
                        shape_attr.original_width = v;
                        if common.width == 0 { common.width = v; }
                    }
                    b"height" => {
                        let v = parse_u32(&attr);
                        shape_attr.original_height = v;
                        if common.height == 0 { common.height = v; }
                    }
                    _ => {}
                }
            }
        }
        b"pos" => {
            *has_pos = true;
            for attr in ce.attributes().flatten() {
                match attr.key.as_ref() {
                    b"treatAsChar" => {
                        common.treat_as_char = attr_str(&attr) == "1" || attr_str(&attr) == "true";
                    }
                    b"vertRelTo" => {
                        common.vert_rel_to = match attr_str(&attr).as_str() {
                            "PAPER" => VertRelTo::Paper,
                            "PAGE" => VertRelTo::Page,
                            "PARA" => VertRelTo::Para,
                            _ => VertRelTo::Para,
                        };
                    }
                    b"horzRelTo" => {
                        common.horz_rel_to = match attr_str(&attr).as_str() {
                            "PAPER" => HorzRelTo::Paper,
                            "PAGE" => HorzRelTo::Page,
                            "COLUMN" => HorzRelTo::Column,
                            "PARA" => HorzRelTo::Para,
                            _ => HorzRelTo::Para,
                        };
                    }
                    b"vertAlign" => {
                        common.vert_align = match attr_str(&attr).as_str() {
                            "TOP" => VertAlign::Top,
                            "CENTER" => VertAlign::Center,
                            "BOTTOM" => VertAlign::Bottom,
                            "INSIDE" => VertAlign::Inside,
                            "OUTSIDE" => VertAlign::Outside,
                            _ => VertAlign::Top,
                        };
                    }
                    b"horzAlign" => {
                        common.horz_align = match attr_str(&attr).as_str() {
                            "LEFT" => HorzAlign::Left,
                            "CENTER" => HorzAlign::Center,
                            "RIGHT" => HorzAlign::Right,
                            "INSIDE" => HorzAlign::Inside,
                            "OUTSIDE" => HorzAlign::Outside,
                            _ => HorzAlign::Left,
                        };
                    }
                    b"vertOffset" => common.vertical_offset = parse_i32(&attr) as u32,
                    b"horzOffset" => common.horizontal_offset = parse_i32(&attr) as u32,
                    _ => {}
                }
            }
        }
        b"offset" => {
            for attr in ce.attributes().flatten() {
                match attr.key.as_ref() {
                    b"x" => {
                        let v = parse_u32(&attr);
                        shape_attr.offset_x = v as i32;
                        if !*has_pos {
                            common.horizontal_offset = v;
                        }
                    }
                    b"y" => {
                        let v = parse_u32(&attr);
                        shape_attr.offset_y = v as i32;
                        if !*has_pos {
                            common.vertical_offset = v;
                        }
                    }
                    _ => {}
                }
            }
        }
        b"outMargin" => {
            for attr in ce.attributes().flatten() {
                match attr.key.as_ref() {
                    b"left" => common.margin.left = parse_i16(&attr),
                    b"right" => common.margin.right = parse_i16(&attr),
                    b"top" => common.margin.top = parse_i16(&attr),
                    b"bottom" => common.margin.bottom = parse_i16(&attr),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// `<hp:renderingInfo>` 파싱: 아핀 변환 행렬 합성 → shape_attr.render_*
///
/// HWPX 구조:
/// ```xml
/// <hp:renderingInfo>
///   <hp:transMatrix e1 e2 e3 e4 e5 e6/>   ← 이동
///   <hp:scaMatrix e1 e2 e3 e4 e5 e6/>     ← 스케일
///   <hp:rotMatrix e1 e2 e3 e4 e5 e6/>     ← 회전
///   ... (sca/rot 쌍이 추가될 수 있음)
/// </hp:renderingInfo>
/// ```
///
/// 행렬 [a, b, tx, c, d, ty] → (x',y') = (a*x+b*y+tx, c*x+d*y+ty)
/// 합성 순서: HWP 바이너리와 동일하게 trans × rot × sca
fn parse_rendering_info(
    reader: &mut Reader<&[u8]>,
    shape_attr: &mut ShapeComponentAttr,
) -> Result<(), HwpxError> {
    // 행렬 값 파싱 헬퍼
    fn read_matrix(ce: &quick_xml::events::BytesStart) -> [f64; 6] {
        let mut m = [0.0f64; 6];
        for attr in ce.attributes().flatten() {
            let val: f64 = attr_str(&attr).parse().unwrap_or(0.0);
            match attr.key.as_ref() {
                b"e1" => m[0] = val,
                b"e2" => m[1] = val,
                b"e3" => m[2] = val,
                b"e4" => m[3] = val,
                b"e5" => m[4] = val,
                b"e6" => m[5] = val,
                _ => {}
            }
        }
        m
    }
    // 아핀 행렬 합성: result = A × B
    fn compose(a: &[f64; 6], b: &[f64; 6]) -> [f64; 6] {
        [
            a[0]*b[0] + a[1]*b[3],          // a
            a[0]*b[1] + a[1]*b[4],          // b
            a[0]*b[2] + a[1]*b[5] + a[2],   // tx
            a[3]*b[0] + a[4]*b[3],          // c
            a[3]*b[1] + a[4]*b[4],          // d
            a[3]*b[2] + a[4]*b[5] + a[5],   // ty
        ]
    }

    let mut buf = Vec::new();
    let mut trans = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]; // identity
    let mut sca_rot_pairs: Vec<([f64; 6], [f64; 6])> = Vec::new();
    let mut pending_sca: Option<[f64; 6]> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"transMatrix" => trans = read_matrix(ce),
                    b"scaMatrix" => {
                        pending_sca = Some(read_matrix(ce));
                    }
                    b"rotMatrix" => {
                        let rot = read_matrix(ce);
                        let sca = pending_sca.take()
                            .unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
                        sca_rot_pairs.push((sca, rot));
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                if local_name(ee.name().as_ref()) == b"renderingInfo" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("renderingInfo: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // sca만 있고 rot이 없는 경우 처리
    if let Some(sca) = pending_sca {
        sca_rot_pairs.push((sca, [1.0, 0.0, 0.0, 0.0, 1.0, 0.0]));
    }

    // HWP 바이너리와 동일한 합성: result = trans, 그 후 각 쌍마다 result = result × rot × sca
    let mut result = trans;
    for (sca, rot) in &sca_rot_pairs {
        result = compose(&result, rot);
        result = compose(&result, sca);
    }

    shape_attr.render_sx = result[0]; // a
    shape_attr.render_b  = result[1]; // b (회전/전단)
    shape_attr.render_tx = result[2]; // tx
    shape_attr.render_c  = result[3]; // c (회전/전단)
    shape_attr.render_sy = result[4]; // d
    shape_attr.render_ty = result[5]; // ty

    Ok(())
}

/// `<hp:lineShape>` 요소에서 ShapeBorderLine을 파싱한다.
fn parse_line_shape_attr(e: &quick_xml::events::BytesStart) -> ShapeBorderLine {
    let mut bl = ShapeBorderLine::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"color" => {
                let s = attr_str(&attr);
                if let Some(hex) = s.strip_prefix('#') {
                    bl.color = u32::from_str_radix(hex, 16).unwrap_or(0);
                }
            }
            b"width" => bl.width = parse_i32(&attr),
            b"style" => {
                // 선 스타일 → attr 비트 플래그 (하위 바이트)
                let style_val: u8 = match attr_str(&attr).as_str() {
                    "NONE" => 0,
                    "SOLID" => 1,
                    "DASH" => 2,
                    "DOT" => 3,
                    "DASH_DOT" => 4,
                    "DASH_DOT_DOT" => 5,
                    "LONG_DASH" => 6,
                    "CIRCLE" => 7,
                    "DOUBLE_SLIM" => 8,
                    "SLIM_THICK" => 9,
                    "THICK_SLIM" => 10,
                    "SLIM_THICK_SLIM" => 11,
                    _ => 1,
                };
                bl.attr = (bl.attr & !0xFF) | style_val as u32;
            }
            b"outlineStyle" => {
                bl.outline_style = match attr_str(&attr).as_str() {
                    "NORMAL" => 0,
                    "OUTER" => 1,
                    "INNER" => 2,
                    _ => 0,
                };
            }
            _ => {}
        }
    }
    bl
}

/// shape 내부의 `<hp:fillBrush>` 자식 요소를 파싱하여 Fill을 반환한다.
fn parse_shape_fill_brush(reader: &mut Reader<&[u8]>) -> Result<Fill, HwpxError> {
    use crate::model::style::{FillType, SolidFill, ImageFill, GradientFill, ImageFillMode};
    let mut fill = Fill::default();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref ce)) | Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"winBrush" => {
                        fill.fill_type = FillType::Solid;
                        let mut solid = SolidFill::default();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"faceColor" => solid.background_color = parse_color(&attr),
                                b"hatchColor" => solid.pattern_color = parse_color(&attr),
                                b"alpha" => {
                                    let val = attr_str(&attr);
                                    if let Ok(f) = val.parse::<f64>() {
                                        fill.alpha = (f.clamp(0.0, 1.0) * 255.0) as u8;
                                    }
                                }
                                _ => {}
                            }
                        }
                        fill.solid = Some(solid);
                    }
                    b"gradation" => {
                        fill.fill_type = FillType::Gradient;
                        let mut grad = GradientFill::default();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"type" => grad.gradient_type = parse_i16(&attr),
                                b"angle" => grad.angle = parse_i16(&attr),
                                b"centerX" => grad.center_x = parse_i16(&attr),
                                b"centerY" => grad.center_y = parse_i16(&attr),
                                _ => {}
                            }
                        }
                        fill.gradient = Some(grad);
                    }
                    b"imgBrush" => {
                        fill.fill_type = FillType::Image;
                        let mut img = ImageFill::default();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"mode" => {
                                    img.fill_mode = match attr_str(&attr).as_str() {
                                        "TILE" | "TILE_ALL" => ImageFillMode::TileAll,
                                        "FIT" | "FIT_TO_SIZE" | "STRETCH" | "TOTAL" => ImageFillMode::FitToSize,
                                        "CENTER" => ImageFillMode::Center,
                                        _ => ImageFillMode::TileAll,
                                    };
                                }
                                _ => {}
                            }
                        }
                        fill.image = Some(img);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                if local_name(ee.name().as_ref()) == b"fillBrush" { break; }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("fillBrush: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(fill)
}

/// `<hp:drawText>` 내부의 `<hp:subList>` → `<hp:p>` 문단을 파싱한다.
fn parse_draw_text(
    reader: &mut Reader<&[u8]>,
    text_box: &mut TextBox,
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"subList" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"vertAlign" => {
                                    text_box.vertical_align = match attr_str(&attr).as_str() {
                                        "CENTER" => VerticalAlign::Center,
                                        "BOTTOM" => VerticalAlign::Bottom,
                                        _ => VerticalAlign::Top,
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                    b"p" => {
                        // subList 내 p를 독립 파싱
                        let (para, _) = parse_paragraph(ce, reader)?;
                        text_box.paragraphs.push(para);
                    }
                    b"textMargin" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => text_box.margin_left = parse_i16(&attr),
                                b"right" => text_box.margin_right = parse_i16(&attr),
                                b"top" => text_box.margin_top = parse_i16(&attr),
                                b"bottom" => text_box.margin_bottom = parse_i16(&attr),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"drawText" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("drawText: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

// ─── 그리기 객체 파싱 (rect, ellipse, line, arc, polygon, curve) ───

/// `<hp:rect>`, `<hp:ellipse>` 등 그리기 객체를 파싱하여 `Control::Shape`를 반환한다.
fn parse_shape_object(
    shape_type: &[u8],
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut common = CommonObjAttr::default();
    let mut shape_attr = ShapeComponentAttr::default();
    let mut border_line = ShapeBorderLine::default();
    let mut fill = Fill::default();
    let mut text_box: Option<TextBox> = None;
    let mut has_pos = false;
    let mut x_coords = [0i32; 4];
    let mut y_coords = [0i32; 4];

    parse_object_element_attrs(e, &mut common, &mut shape_attr);

    let tag_name = String::from_utf8_lossy(shape_type).to_string();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"sz" | b"curSz" | b"orgSz" | b"pos" | b"offset" | b"outMargin" => {
                        parse_object_layout_child(local, ce, &mut common, &mut shape_attr, &mut has_pos);
                    }
                    b"lineShape" => {
                        border_line = parse_line_shape_attr(ce);
                    }
                    b"drawText" => {
                        let mut tb = TextBox::default();
                        tb.max_width = common.width;
                        parse_draw_text(reader, &mut tb)?;
                        text_box = Some(tb);
                    }
                    b"pt0" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x_coords[0] = parse_i32(&attr),
                                b"y" => y_coords[0] = parse_i32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"pt1" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x_coords[1] = parse_i32(&attr),
                                b"y" => y_coords[1] = parse_i32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"pt2" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x_coords[2] = parse_i32(&attr),
                                b"y" => y_coords[2] = parse_i32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"pt3" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x_coords[3] = parse_i32(&attr),
                                b"y" => y_coords[3] = parse_i32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"renderingInfo" => {
                        parse_rendering_info(reader, &mut shape_attr)?;
                    }
                    b"fillBrush" => {
                        fill = parse_shape_fill_brush(reader)?;
                    }
                    b"shadow" => {
                        // shadow는 무시 (Start 이벤트인 경우 내부 소비)
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == shape_type {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("{}: {}", tag_name, e))),
            _ => {}
        }
        buf.clear();
    }

    // curSz width=0이면 orgSz 또는 sz에서 폴백
    if shape_attr.current_width == 0 && shape_attr.original_width > 0 {
        shape_attr.current_width = shape_attr.original_width;
        if common.width == 0 {
            common.width = shape_attr.original_width;
        }
    }

    let drawing = DrawingObjAttr {
        shape_attr,
        border_line,
        fill,
        text_box,
        ..Default::default()
    };

    let shape = match shape_type {
        b"rect" => ShapeObject::Rectangle(RectangleShape {
            common,
            drawing,
            round_rate: 0,
            x_coords,
            y_coords,
        }),
        b"ellipse" => ShapeObject::Ellipse(EllipseShape {
            common,
            drawing,
            ..Default::default()
        }),
        b"line" => ShapeObject::Line(LineShape {
            common,
            drawing,
            ..Default::default()
        }),
        b"arc" => ShapeObject::Arc(ArcShape {
            common,
            drawing,
            ..Default::default()
        }),
        b"polygon" => ShapeObject::Polygon(PolygonShape {
            common,
            drawing,
            ..Default::default()
        }),
        b"curve" => ShapeObject::Curve(CurveShape {
            common,
            drawing,
            ..Default::default()
        }),
        _ => ShapeObject::Rectangle(RectangleShape {
            common,
            drawing,
            round_rate: 0,
            x_coords,
            y_coords,
        }),
    };

    Ok(Control::Shape(Box::new(shape)))
}

// ─── 묶음(그룹) 객체 파싱 ───

/// `<hp:container>` 요소를 파싱하여 `Control::Shape(GroupShape)`를 반환한다.
fn parse_container(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut common = CommonObjAttr::default();
    let mut shape_attr = ShapeComponentAttr::default();
    let mut has_pos = false;
    let mut children = Vec::new();

    parse_object_element_attrs(e, &mut common, &mut shape_attr);

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"sz" | b"curSz" | b"orgSz" | b"pos" | b"offset" | b"outMargin" => {
                        parse_object_layout_child(local, ce, &mut common, &mut shape_attr, &mut has_pos);
                    }
                    b"pic" => {
                        // 자식 그림 객체
                        let child = parse_picture(ce, reader)?;
                        if let Control::Picture(pic) = child {
                            children.push(ShapeObject::Picture(pic));
                        }
                    }
                    b"rect" | b"ellipse" | b"line" | b"arc" | b"polygon" | b"curve" => {
                        // 자식 그리기 객체
                        let child = parse_shape_object(local, ce, reader)?;
                        if let Control::Shape(shape) = child {
                            children.push(*shape);
                        }
                    }
                    b"container" => {
                        // 중첩 그룹
                        let child = parse_container(ce, reader)?;
                        if let Control::Shape(shape) = child {
                            children.push(*shape);
                        }
                    }
                    b"renderingInfo" => {
                        parse_rendering_info(reader, &mut shape_attr)?;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"container" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("container: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    let group = GroupShape {
        common,
        shape_attr,
        children,
        caption: None,
    };

    Ok(Control::Shape(Box::new(ShapeObject::Group(group))))
}

// ─── <hp:ctrl> 파싱 ───

/// `<hp:ctrl>` 내부 자식 요소를 파싱하여 해당 컨트롤을 추가한다.
/// ForChars.java 매핑 기준: header, footer, footNote, endNote, autoNum, newNum,
/// pageHiding, pageNum, bookmark, hiddenComment, fieldBegin, fieldEnd, colPr
fn parse_ctrl(
    _e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    controls: &mut Vec<Control>,
    text_parts: &mut Vec<String>,
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"colPr" => {
                        let cd = parse_col_pr(ce);
                        controls.push(Control::ColumnDef(cd));
                        skip_element(reader, b"colPr")?;
                    }
                    b"header" => {
                        let ctrl = parse_ctrl_header(ce, reader)?;
                        controls.push(ctrl);
                    }
                    b"footer" => {
                        let ctrl = parse_ctrl_footer(ce, reader)?;
                        controls.push(ctrl);
                    }
                    b"footNote" => {
                        let ctrl = parse_ctrl_footnote(ce, reader)?;
                        controls.push(ctrl);
                    }
                    b"endNote" => {
                        let ctrl = parse_ctrl_endnote(ce, reader)?;
                        controls.push(ctrl);
                    }
                    b"autoNum" => {
                        let ctrl = parse_ctrl_autonum(ce, reader)?;
                        controls.push(ctrl);
                        // AutoNumber: 공백 placeholder 추가 (HWP 바이너리와 동일)
                        // → apply_auto_numbers_to_composed에서 "  "(연속 2공백)으로 번호 삽입
                        text_parts.push(" ".to_string());
                    }
                    b"hiddenComment" => {
                        let ctrl = parse_ctrl_hidden_comment(reader)?;
                        controls.push(ctrl);
                    }
                    b"fieldBegin" => {
                        let ctrl = parse_ctrl_field_begin(ce, reader)?;
                        controls.push(ctrl);
                        // FIELD_BEGIN 제어 문자 추가 (Task #11)
                        text_parts.push("\u{0003}".to_string());
                    }
                    b"fieldEnd" => {
                        skip_element(reader, b"fieldEnd")?;
                        // FIELD_END 제어 문자 추가 (Task #11)
                        text_parts.push("\u{0004}".to_string());
                    }
                    b"pageHiding" => {
                        let ph = parse_page_hiding_attrs(ce);
                        controls.push(Control::PageHide(ph));
                        skip_element(reader, b"pageHiding")?;
                    }
                    b"pageNum" => {
                        let pn = parse_page_num_attrs(ce);
                        controls.push(Control::PageNumberPos(pn));
                        skip_element(reader, b"pageNum")?;
                    }
                    b"bookmark" => {
                        let bm = parse_bookmark_attrs(ce);
                        controls.push(Control::Bookmark(bm));
                        skip_element(reader, b"bookmark")?;
                    }
                    b"newNum" => {
                        let nn = parse_new_num_attrs(ce);
                        controls.push(Control::NewNumber(nn));
                        skip_element(reader, b"newNum")?;
                    }
                    _ => {
                        let tag = local.to_vec();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"colPr" => {
                        let cd = parse_col_pr(ce);
                        controls.push(Control::ColumnDef(cd));
                    }
                    b"pageHiding" => {
                        let ph = parse_page_hiding_attrs(ce);
                        controls.push(Control::PageHide(ph));
                    }
                    b"pageNum" => {
                        let pn = parse_page_num_attrs(ce);
                        controls.push(Control::PageNumberPos(pn));
                    }
                    b"bookmark" => {
                        let bm = parse_bookmark_attrs(ce);
                        controls.push(Control::Bookmark(bm));
                    }
                    b"newNum" => {
                        let nn = parse_new_num_attrs(ce);
                        controls.push(Control::NewNumber(nn));
                    }
                    b"autoNum" => {
                        let an = parse_autonum_attrs(ce);
                        controls.push(Control::AutoNumber(an));
                        text_parts.push(" ".to_string());
                    }
                    b"fieldBegin" => {
                        let f = parse_field_begin_attrs(ce);
                        controls.push(Control::Field(f));
                        text_parts.push("\u{0003}".to_string());
                    }
                    b"fieldEnd" => {
                        text_parts.push("\u{0004}".to_string());
                    }
                    b"hiddenComment" => {}
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"ctrl" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("ctrl: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

// ─── ctrl 자식 요소 속성 파싱 헬퍼 ───

fn parse_bool_attr(attr: &quick_xml::events::attributes::Attribute) -> bool {
    let s = attr_str(attr);
    s == "1" || s == "true"
}

fn parse_page_hiding_attrs(e: &quick_xml::events::BytesStart) -> PageHide {
    let mut ph = PageHide::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"hideHeader" => ph.hide_header = parse_bool_attr(&attr),
            b"hideFooter" => ph.hide_footer = parse_bool_attr(&attr),
            b"hideMasterPage" => ph.hide_master_page = parse_bool_attr(&attr),
            b"hideBorder" => ph.hide_border = parse_bool_attr(&attr),
            b"hideFill" => ph.hide_fill = parse_bool_attr(&attr),
            b"hidePageNum" => ph.hide_page_num = parse_bool_attr(&attr),
            _ => {}
        }
    }
    ph
}

fn parse_page_num_attrs(e: &quick_xml::events::BytesStart) -> PageNumberPos {
    let mut pn = PageNumberPos::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"pos" => {
                pn.position = match attr_str(&attr).as_str() {
                    "NONE" => 0,
                    "TOP_LEFT" => 1,
                    "TOP_CENTER" => 2,
                    "TOP_RIGHT" => 3,
                    "BOTTOM_LEFT" => 4,
                    "BOTTOM_CENTER" => 5,
                    "BOTTOM_RIGHT" => 6,
                    "OUTSIDE_TOP" => 7,
                    "OUTSIDE_BOTTOM" => 8,
                    "INSIDE_TOP" => 9,
                    "INSIDE_BOTTOM" => 10,
                    _ => 5, // 기본: 가운데 아래
                };
            }
            b"formatType" => {
                pn.format = match attr_str(&attr).as_str() {
                    "DIGIT" => 0,
                    "CIRCLE_DIGIT" => 1,
                    "ROMAN_CAPITAL" => 2,
                    "ROMAN_SMALL" => 3,
                    "LATIN_CAPITAL" => 4,
                    "LATIN_SMALL" => 5,
                    "HANGUL" => 6,
                    "HANJA" => 7,
                    _ => 0,
                };
            }
            b"sideChar" => {
                let s = attr_str(&attr);
                pn.dash_char = s.chars().next().unwrap_or('-');
            }
            _ => {}
        }
    }
    pn
}

fn parse_bookmark_attrs(e: &quick_xml::events::BytesStart) -> Bookmark {
    let mut bm = Bookmark::default();
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"name" {
            bm.name = attr_str(&attr);
        }
    }
    bm
}

fn parse_new_num_attrs(e: &quick_xml::events::BytesStart) -> NewNumber {
    let mut nn = NewNumber::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"num" => nn.number = parse_u16(&attr),
            b"numType" => nn.number_type = parse_num_type(&attr_str(&attr)),
            _ => {}
        }
    }
    nn
}

fn parse_autonum_attrs(e: &quick_xml::events::BytesStart) -> AutoNumber {
    let mut an = AutoNumber::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"num" => { an.number = parse_u16(&attr); an.assigned_number = an.number; }
            b"numType" => an.number_type = parse_num_type(&attr_str(&attr)),
            _ => {}
        }
    }
    an
}

fn parse_field_begin_attrs(e: &quick_xml::events::BytesStart) -> Field {
    let mut f = Field::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => f.field_type = parse_field_type(&attr_str(&attr)),
            b"name" => f.command = attr_str(&attr),
            _ => {}
        }
    }
    f
}

/// numType 문자열 → AutoNumberType 변환
fn parse_num_type(s: &str) -> AutoNumberType {
    match s {
        "PAGE" => AutoNumberType::Page,
        "FOOTNOTE" => AutoNumberType::Footnote,
        "ENDNOTE" => AutoNumberType::Endnote,
        "FIGURE" | "PICTURE" => AutoNumberType::Picture,
        "TABLE" => AutoNumberType::Table,
        "EQUATION" => AutoNumberType::Equation,
        _ => AutoNumberType::Page,
    }
}

/// FieldType 문자열 → FieldType 변환
fn parse_field_type(s: &str) -> FieldType {
    match s {
        "DATE" => FieldType::Date,
        "DOC_DATE" | "DOCDATE" => FieldType::DocDate,
        "PATH" => FieldType::Path,
        "BOOKMARK" => FieldType::Bookmark,
        "MAILMERGE" => FieldType::MailMerge,
        "CROSSREF" => FieldType::CrossRef,
        "FORMULA" => FieldType::Formula,
        "CLICK_HERE" | "CLICKHERE" => FieldType::ClickHere,
        "SUMMARY" => FieldType::Summary,
        "USER_INFO" | "USERINFO" => FieldType::UserInfo,
        "HYPERLINK" => FieldType::Hyperlink,
        "MEMO" => FieldType::Memo,
        "PRIVATE_INFO" | "PRIVATEINFO" => FieldType::PrivateInfoSecurity,
        "TABLE_OF_CONTENTS" | "TABLEOFCONTENTS" => FieldType::TableOfContents,
        _ => FieldType::Unknown,
    }
}

/// applyPageType 문자열 → HeaderFooterApply 변환
fn parse_apply_page_type(s: &str) -> HeaderFooterApply {
    match s {
        "EVEN" => HeaderFooterApply::Even,
        "ODD" => HeaderFooterApply::Odd,
        _ => HeaderFooterApply::Both,
    }
}

// ─── ctrl 자식 요소별 파싱 함수 ───

/// `<hp:ctrl>` → `<header applyPageType="..." id="...">` → subList → paragraphs
fn parse_ctrl_header(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut header = Header::default();
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"applyPageType" {
            header.apply_to = parse_apply_page_type(&attr_str(&attr));
        }
    }
    header.paragraphs = parse_sublist_paragraphs(reader, b"header")?;
    Ok(Control::Header(Box::new(header)))
}

/// `<hp:ctrl>` → `<footer applyPageType="..." id="...">` → subList → paragraphs
fn parse_ctrl_footer(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut footer = Footer::default();
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"applyPageType" {
            footer.apply_to = parse_apply_page_type(&attr_str(&attr));
        }
    }
    footer.paragraphs = parse_sublist_paragraphs(reader, b"footer")?;
    Ok(Control::Footer(Box::new(footer)))
}

/// `<hp:ctrl>` → `<footNote number="..." suffixChar="..." instId="...">` → subList → paragraphs
fn parse_ctrl_footnote(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut note = Footnote::default();
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"number" {
            note.number = parse_u16(&attr);
        }
    }
    note.paragraphs = parse_sublist_paragraphs(reader, b"footNote")?;
    Ok(Control::Footnote(Box::new(note)))
}

/// `<hp:ctrl>` → `<endNote number="..." suffixChar="..." instId="...">` → subList → paragraphs
fn parse_ctrl_endnote(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut note = Endnote::default();
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"number" {
            note.number = parse_u16(&attr);
        }
    }
    note.paragraphs = parse_sublist_paragraphs(reader, b"endNote")?;
    Ok(Control::Endnote(Box::new(note)))
}

/// `<hp:ctrl>` → `<autoNum num="..." numType="...">` + `<autoNumFormat .../>` 자식
fn parse_ctrl_autonum(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut an = parse_autonum_attrs(e);
    // autoNumFormat 등 자식 요소 파싱
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if local == b"autoNumFormat" {
                    for attr in ce.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"type" => an.format = parse_u8(&attr),
                            b"userChar" => {
                                let s = attr_str(&attr);
                                an.user_symbol = s.chars().next().unwrap_or('\0');
                            }
                            b"prefixChar" => {
                                let s = attr_str(&attr);
                                an.prefix_char = s.chars().next().unwrap_or('\0');
                            }
                            b"suffixChar" => {
                                let s = attr_str(&attr);
                                an.suffix_char = s.chars().next().unwrap_or('\0');
                            }
                            b"supscript" => an.superscript = parse_bool_attr(&attr),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"autoNum" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("autoNum: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(Control::AutoNumber(an))
}

/// `<hp:ctrl>` → `<hiddenComment>` → subList → paragraphs
fn parse_ctrl_hidden_comment(
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut hc = HiddenComment::default();
    hc.paragraphs = parse_sublist_paragraphs(reader, b"hiddenComment")?;
    Ok(Control::HiddenComment(Box::new(hc)))
}

/// `<hp:ctrl>` → `<fieldBegin type="..." name="..." ...>` + `<parameters>` 자식
fn parse_ctrl_field_begin(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut f = parse_field_begin_attrs(e);
    // parameters 자식에서 Command 값 추출
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if local == b"parameters" {
                    parse_field_parameters(reader, &mut f)?;
                } else {
                    let tag = local.to_vec();
                    skip_element(reader, &tag)?;
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"fieldBegin" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("fieldBegin: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(Control::Field(f))
}

/// `<parameters>` 내부에서 Command 문자열 파라미터를 추출한다.
fn parse_field_parameters(
    reader: &mut Reader<&[u8]>,
    field: &mut Field,
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    let mut in_command = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if local == b"stringParam" {
                    for attr in ce.attributes().flatten() {
                        if attr.key.as_ref() == b"name" && attr_str(&attr) == "Command" {
                            in_command = true;
                        }
                    }
                }
            }
            Ok(Event::Text(ref t)) => {
                if in_command {
                    field.command = t.decode().unwrap_or_default().to_string();
                    in_command = false;
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"parameters" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("parameters: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

/// 서브리스트(subList) 내의 문단들을 파싱한다.
/// header, footer, footnote, endnote, hiddenComment에서 공통 사용.
fn parse_sublist_paragraphs(
    reader: &mut Reader<&[u8]>,
    end_tag: &[u8],
) -> Result<Vec<Paragraph>, HwpxError> {
    let mut paragraphs = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if local == b"p" {
                    let (para, _) = parse_paragraph(ce, reader)?;
                    paragraphs.push(para);
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(
                format!("{}: {}", String::from_utf8_lossy(end_tag), e)
            )),
            _ => {}
        }
        buf.clear();
    }
    Ok(paragraphs)
}

// ─── 문단 레벨 컨트롤 파싱 (compose, dutmal, equation) ───

/// `<hp:compose>` 요소 (글자겹침/CharOverlap)를 파싱한다.
fn parse_compose(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut co = CharOverlap::default();
    // 요소 속성 파싱
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"circleType" => {
                co.border_type = match attr_str(&attr).as_str() {
                    "CHAR" => 0,
                    "SHAPE_CIRCLE" => 1,
                    "SHAPE_REVERSAL_CIRCLE" => 2,
                    "SHAPE_RECTANGLE" => 3,
                    "SHAPE_REVERSAL_RECTANGLE" => 4,
                    "SHAPE_TRIANGLE" => 5,
                    "SHAPE_REVERSAL_TIRANGLE" => 6,
                    _ => 0,
                };
            }
            b"charSz" => co.inner_char_size = parse_i8(&attr),
            b"composeType" => {
                co.expansion = match attr_str(&attr).as_str() {
                    "OVERLAP" => 1,
                    _ => 0, // SPREAD
                };
            }
            _ => {}
        }
    }
    // 자식 요소 파싱 (composeText, charPr)
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if local == b"composeText" {
                    let text = read_compose_text(reader)?;
                    co.chars = text.chars().collect();
                } else {
                    let tag = local.to_vec();
                    skip_element(reader, &tag)?;
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if local == b"charPr" {
                    for attr in ce.attributes().flatten() {
                        if attr.key.as_ref() == b"prIDRef" {
                            co.char_shape_ids.push(parse_u32(&attr));
                        }
                    }
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"compose" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("compose: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(Control::CharOverlap(co))
}

/// `<composeText>` 내부 텍스트를 읽는다.
fn read_compose_text(reader: &mut Reader<&[u8]>) -> Result<String, HwpxError> {
    let mut text = String::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                text.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"composeText" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("composeText: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(text)
}

/// `<hp:dutmal>` 요소 (덧말/Ruby)를 파싱한다.
fn parse_dutmal(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut ruby = Ruby::default();
    // 요소 속성
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"posType" => {
                ruby.alignment = match attr_str(&attr).as_str() {
                    "TOP" => 0,
                    "BOTTOM" => 1,
                    _ => 0,
                };
            }
            b"align" => {
                ruby.alignment = match attr_str(&attr).as_str() {
                    "LEFT" => 0,
                    "RIGHT" => 1,
                    "CENTER" => 2,
                    _ => 0,
                };
            }
            _ => {}
        }
    }
    // 자식 요소 파싱 (subText)
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if local == b"subText" {
                    ruby.ruby_text = read_dutmal_text(reader, b"subText")?;
                } else if local == b"mainText" {
                    // mainText는 이미 문단 텍스트에 포함되므로 스킵
                    skip_element(reader, b"mainText")?;
                } else {
                    let tag = local.to_vec();
                    skip_element(reader, &tag)?;
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"dutmal" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("dutmal: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(Control::Ruby(ruby))
}

/// dutmal 내부 텍스트 요소(mainText, subText)의 텍스트를 읽는다.
fn read_dutmal_text(reader: &mut Reader<&[u8]>, end_tag: &[u8]) -> Result<String, HwpxError> {
    let mut text = String::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                text.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(
                format!("{}: {}", String::from_utf8_lossy(end_tag), e)
            )),
            _ => {}
        }
        buf.clear();
    }
    Ok(text)
}

/// `<hp:equation>` 요소 (수식)를 파싱한다.
/// 수식 속성(version, baseLine, textColor, baseUnit, font)과
/// `<hp:script>` 하위 요소에서 수식 스크립트를 추출하여 `Control::Equation`을 생성한다.
fn parse_equation(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut common = CommonObjAttr::default();
    let mut shape_attr = ShapeComponentAttr::default();
    let mut has_pos = false;

    // 수식 전용 속성
    let mut version_info = String::new();
    let mut baseline: i16 = 0;
    let mut color: u32 = 0;
    let mut font_size: u32 = 1000;
    let mut font_name = String::new();

    // 공통 개체 속성 + 수식 속성 파싱
    parse_object_element_attrs(e, &mut common, &mut shape_attr);
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"version" => version_info = attr_str(&attr),
            b"baseLine" => baseline = attr_str(&attr).parse().unwrap_or(0),
            b"textColor" => color = parse_color(&attr),
            b"baseUnit" => font_size = parse_u32(&attr),
            b"font" => font_name = attr_str(&attr),
            _ => {}
        }
    }

    let mut script = String::new();
    let mut in_script = false;

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"sz" | b"curSz" | b"orgSz" | b"pos" | b"offset" | b"outMargin" => {
                        parse_object_layout_child(local, ce, &mut common, &mut shape_attr, &mut has_pos);
                    }
                    b"script" => { in_script = true; }
                    _ => {}
                }
            }
            Ok(Event::Text(ref txt)) => {
                if in_script {
                    if let Ok(s) = txt.decode() {
                        script.push_str(&s);
                    }
                }
            }
            Ok(Event::GeneralRef(ref r)) => {
                if in_script {
                    if let Ok(Some(ch)) = r.resolve_char_ref() {
                        script.push(ch);
                    } else if let Ok(name) = r.decode() {
                        match name.as_ref() {
                            "lt" => script.push('<'),
                            "gt" => script.push('>'),
                            "amp" => script.push('&'),
                            "quot" => script.push('"'),
                            "apos" => script.push('\''),
                            _ => {
                                script.push('&');
                                script.push_str(&name);
                                script.push(';');
                            }
                        }
                    }
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                let local = local_name(eename.as_ref());
                if local == b"script" {
                    in_script = false;
                } else if local == b"equation" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("equation: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    let equation = Equation {
        common,
        script,
        font_size,
        color,
        baseline,
        font_name,
        version_info,
        raw_ctrl_data: Vec::new(),
    };
    Ok(Control::Equation(Box::new(equation)))
}

// ─── 유틸리티 (section 전용) ───

/// 텍스트 파트들의 UTF-16 길이 합산
/// 탭 문자는 HWP 바이너리와 동일하게 8 code unit으로 계산
fn calc_utf16_len_from_parts(parts: &[String]) -> u32 {
    parts.iter()
        .map(|s| match s.as_str() {
            "\u{0002}" | "\u{0003}" | "\u{0004}" => 8,
            _ => s.chars()
                .map(|c| if c == '\t' { 8u32 } else if (c as u32) > 0xFFFF { 2 } else { 1 })
                .sum(),
        })
        .sum()
}

// ─── 양식 컨트롤 파싱 ───

/// HWPX 양식 컨트롤 요소(`<hp:btn>`, `<hp:checkBtn>`, `<hp:radioBtn>`,
/// `<hp:comboBox>`, `<hp:edit>`)를 파싱하여 `Control::Form`으로 반환한다.
///
/// 요소는 `<hp:run>` 직접 자식으로 위치하며, `<hp:sz>` / `<hp:listItem>` /
/// `<hp:text>` / `<hp:formCharPr>` 등의 자식 요소를 포함한다.
fn parse_form_object(
    form_type: FormType,
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut form = FormObject {
        form_type,
        enabled: true,
        ..Default::default()
    };

    // 요소 속성 파싱 (AbstractFormObjectType + AbstractButtonObjectType)
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name"       => form.name    = attr_str(&attr),
            b"caption"    => form.caption = attr_str(&attr),
            b"foreColor"  => form.fore_color  = parse_color(&attr),
            b"backColor"  => form.back_color  = parse_color(&attr),
            b"enabled"    => form.enabled = parse_bool(&attr),
            b"value"      => form.value   = if attr_str(&attr) == "CHECKED" { 1 } else { 0 },
            b"selectedValue" => form.text = attr_str(&attr), // comboBox 선택값
            _ => {}
        }
    }

    // 자식 요소 순회
    let end_tag = local_name(e.name().as_ref()).to_vec();
    let mut buf = Vec::new();
    let mut list_items: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"text" => {
                        // <hp:text> 자식 (edit 컨트롤) — 텍스트 내용 읽기
                        let mut tbuf = Vec::new();
                        loop {
                            match reader.read_event_into(&mut tbuf) {
                                Ok(Event::Text(ref t)) => {
                                    if let Ok(s) = t.decode() {
                                        form.text.push_str(&s);
                                    }
                                }
                                Ok(Event::End(_)) => break,
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            tbuf.clear();
                        }
                    }
                    _ => { skip_element(reader, local)?; }
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"sz" => {
                        // <hp:sz width="..." height="..."/>
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width"  => form.width  = parse_u32(&attr),
                                b"height" => form.height = parse_u32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"listItem" => {
                        // <hp:listItem value="..."/> (comboBox 항목)
                        for attr in ce.attributes().flatten() {
                            if attr.key.as_ref() == b"value" {
                                list_items.push(attr_str(&attr));
                            }
                        }
                    }
                    _ => {} // formCharPr 등 무시
                }
            }
            Ok(Event::End(ref ee)) => {
                if local_name(ee.name().as_ref()) == end_tag.as_slice() {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("form_object: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // comboBox 항목 목록을 properties에 저장
    if !list_items.is_empty() {
        for (i, item) in list_items.iter().enumerate() {
            form.properties.insert(format!("listItem{}", i), item.clone());
        }
    }

    Ok(Control::Form(Box::new(form)))
}

// ---------------- HWPX switch / chart / ole 핸들러 ----------------

/// `<hp:switch>`를 열고 내부에서 OOXML 차트(hp:chart)를 우선적으로,
/// 없으면 OLE fallback(hp:ole)을 파싱하여 Control로 반환
fn parse_switch_chart_or_ole(reader: &mut Reader<&[u8]>) -> Result<Option<Control>, HwpxError> {
    let mut chart_ctrl: Option<Control> = None;
    let mut ole_ctrl: Option<Control> = None;
    let mut buf = Vec::new();
    let mut in_case = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"case" => { in_case = true; }
                    b"default" => { in_case = false; }
                    b"chart" => {
                        if chart_ctrl.is_none() {
                            chart_ctrl = parse_hp_chart_element(ce, reader)?;
                        } else {
                            skip_element(reader, b"chart")?;
                        }
                    }
                    b"ole" => {
                        if ole_ctrl.is_none() {
                            ole_ctrl = parse_hp_ole_element(ce, reader)?;
                        } else {
                            skip_element(reader, b"ole")?;
                        }
                    }
                    _ => {}
                }
                let _ = in_case;
            }
            Ok(Event::End(ref ee)) => {
                if local_name(ee.name().as_ref()) == b"switch" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("switch: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(chart_ctrl.or(ole_ctrl))
}

/// `<hp:chart chartIDRef="Chart/chartN.xml" zOrder="..." textWrap="..." ...>` 내부를 OLE 모델로 변환
fn parse_hp_chart_element(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Option<Control>, HwpxError> {
    use crate::model::shape::OleShape;

    let mut common = CommonObjAttr::default();
    let mut chart_num: u16 = 0;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"zOrder" => common.z_order = parse_i32(&attr),
            b"textWrap" => {
                common.text_wrap = match attr_str(&attr).as_str() {
                    "SQUARE" => TextWrap::Square,
                    "TIGHT" => TextWrap::Tight,
                    "THROUGH" => TextWrap::Through,
                    "TOP_AND_BOTTOM" => TextWrap::TopAndBottom,
                    "BEHIND_TEXT" => TextWrap::BehindText,
                    "IN_FRONT_OF_TEXT" => TextWrap::InFrontOfText,
                    _ => TextWrap::Square,
                };
            }
            b"chartIDRef" => {
                // "Chart/chart1.xml" → 1
                let s = attr_str(&attr);
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                chart_num = digits.parse().unwrap_or(0);
            }
            b"instid" => common.instance_id = parse_u32(&attr),
            _ => {}
        }
    }

    parse_common_shape_children(reader, &mut common, b"chart")?;

    if chart_num == 0 {
        return Ok(None);
    }

    let mut ole = OleShape::default();
    ole.common = common;
    ole.bin_data_id = 60000u32 + chart_num as u32;
    ole.extent_x = 7200;
    ole.extent_y = 7200;
    Ok(Some(Control::Shape(Box::new(ShapeObject::Ole(Box::new(ole))))))
}

/// `<hp:ole binaryItemIDRef="oleN" ...>` 내부를 OLE 모델로 변환 (fallback용)
fn parse_hp_ole_element(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Option<Control>, HwpxError> {
    use crate::model::shape::OleShape;

    let mut common = CommonObjAttr::default();
    let mut bin_id: u32 = 0;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"zOrder" => common.z_order = parse_i32(&attr),
            b"textWrap" => {
                common.text_wrap = match attr_str(&attr).as_str() {
                    "SQUARE" => TextWrap::Square,
                    "TIGHT" => TextWrap::Tight,
                    "THROUGH" => TextWrap::Through,
                    "TOP_AND_BOTTOM" => TextWrap::TopAndBottom,
                    "BEHIND_TEXT" => TextWrap::BehindText,
                    "IN_FRONT_OF_TEXT" => TextWrap::InFrontOfText,
                    _ => TextWrap::Square,
                };
            }
            b"binaryItemIDRef" => {
                let s = attr_str(&attr);
                let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
                bin_id = digits.parse().unwrap_or(0);
            }
            b"instid" => common.instance_id = parse_u32(&attr),
            _ => {}
        }
    }

    parse_common_shape_children(reader, &mut common, b"ole")?;

    let mut ole = OleShape::default();
    ole.common = common;
    ole.bin_data_id = bin_id;
    ole.extent_x = 7200;
    ole.extent_y = 7200;
    Ok(Some(Control::Shape(Box::new(ShapeObject::Ole(Box::new(ole))))))
}

/// `<hp:sz>`, `<hp:pos>`, `<hp:outMargin>` 등 공통 자식 요소를 공통 속성에 반영한다.
fn parse_common_shape_children(
    reader: &mut Reader<&[u8]>,
    common: &mut CommonObjAttr,
    end_tag: &[u8],
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"sz" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => common.width = parse_u32(&attr),
                                b"height" => common.height = parse_u32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"pos" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"vertRelTo" => {
                                    common.vert_rel_to = match attr_str(&attr).as_str() {
                                        "PAPER" => VertRelTo::Paper,
                                        "PAGE" => VertRelTo::Page,
                                        _ => VertRelTo::Para,
                                    };
                                }
                                b"horzRelTo" => {
                                    common.horz_rel_to = match attr_str(&attr).as_str() {
                                        "PAPER" => HorzRelTo::Paper,
                                        "PAGE" => HorzRelTo::Page,
                                        "COLUMN" => HorzRelTo::Column,
                                        _ => HorzRelTo::Para,
                                    };
                                }
                                b"vertAlign" => {
                                    common.vert_align = match attr_str(&attr).as_str() {
                                        "CENTER" => VertAlign::Center,
                                        "BOTTOM" => VertAlign::Bottom,
                                        "INSIDE" => VertAlign::Inside,
                                        "OUTSIDE" => VertAlign::Outside,
                                        _ => VertAlign::Top,
                                    };
                                }
                                b"horzAlign" => {
                                    common.horz_align = match attr_str(&attr).as_str() {
                                        "CENTER" => HorzAlign::Center,
                                        "RIGHT" => HorzAlign::Right,
                                        "INSIDE" => HorzAlign::Inside,
                                        "OUTSIDE" => HorzAlign::Outside,
                                        _ => HorzAlign::Left,
                                    };
                                }
                                b"vertOffset" => common.vertical_offset = parse_u32(&attr),
                                b"horzOffset" => common.horizontal_offset = parse_u32(&attr),
                                b"treatAsChar" => common.treat_as_char = parse_bool(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"outMargin" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"left" => common.margin.left = parse_i32(&attr) as i16,
                                b"right" => common.margin.right = parse_i32(&attr) as i16,
                                b"top" => common.margin.top = parse_i32(&attr) as i16,
                                b"bottom" => common.margin.bottom = parse_i32(&attr) as i16,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                if local_name(ee.name().as_ref()) == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("shape_children: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_section() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>Hello World</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert_eq!(section.paragraphs.len(), 1);
        assert_eq!(section.paragraphs[0].text, "Hello World");
        assert_eq!(section.paragraphs[0].para_shape_id, 0);
    }

    #[test]
    fn test_parse_linebreak_preserves_offsets() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>줄바꿈A<hp:lineBreak/>줄바꿈B</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.text, "줄바꿈A\n줄바꿈B");
        assert_eq!(para.char_offsets, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_parse_control_keeps_interleaved_offsets() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:t>A</hp:t></hp:run>
    <hp:tbl rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="0">
      <hp:inMargin left="0" right="0" top="0" bottom="0"/>
      <hp:tr>
        <hp:tc name="0" header="0" hasMargin="0" editable="0" dirty="0" borderFillIDRef="0" textDirection="HORIZONTAL" vertAlign="TOP" colAddr="0" rowAddr="0" colSpan="1" rowSpan="1" width="1000" height="1000">
          <hp:cellAddr colAddr="0" rowAddr="0"/>
          <hp:cellSpan colSpan="1" rowSpan="1"/>
          <hp:cellSz width="1000" height="1000"/>
          <hp:cellMargin left="0" right="0" top="0" bottom="0"/>
          <hp:subList><hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>T</hp:t></hp:run></hp:p></hp:subList>
          <hp:lineBreak/>
        </hp:tc>
      </hp:tr>
    </hp:tbl>
    <hp:run charPrIDRef="0"><hp:t>B</hp:t></hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.text, "AB");
        assert_eq!(para.char_offsets, vec![0, 9]);
        assert_eq!(para.char_shapes[0].start_pos, 0);
        assert_eq!(para.char_shapes[1].start_pos, 9);
        assert_eq!(para.controls.len(), 1);
    }

    #[test]
    fn test_parse_empty_section() {
        let xml = r#"<?xml version="1.0"?><hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"/>"#;
        let section = parse_hwpx_section(xml).unwrap();
        assert!(section.paragraphs.is_empty());
    }
}
