//! section*.xml 파싱 — HWPX 섹션 본문을 Section 모델로 변환
//!
//! 섹션 XML의 문단(<hp:p>), 텍스트 런(<hp:run>), 표(<hp:tbl>),
//! 이미지(<hp:pic>) 등을 기존 Document 모델로 변환한다.

use quick_xml::events::{BytesRef, Event};
use quick_xml::Reader;

use crate::model::control::{
    AutoNumber, AutoNumberType, Bookmark, CharOverlap, Control, Equation, Field, FieldType,
    FormObject, FormType, HiddenComment, NewNumber, PageHide, PageNumberPos, Ruby,
};
use crate::model::document::{Section, SectionDef};
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply, MasterPage};
use crate::model::image::{CropInfo, ImageAttr, ImageEffect};
use crate::model::page::{
    BindingMethod, ColumnDef, ColumnDirection, ColumnType, PageBorderBasis, PageBorderFill,
    PageBorderUiBasis, PageDef,
};
use crate::model::paragraph::{CharShapeRef, FieldRange, LineSeg, Paragraph};
use crate::model::shape::{
    ArcShape, CommonObjAttr, CurveShape, DrawingObjAttr, EllipseShape, GroupShape, HorzAlign,
    HorzRelTo, LineShape, PolygonShape, RectangleShape, ShapeComponentAttr, ShapeObject,
    SizeCriterion, TextBox, TextWrap, VertAlign, VertRelTo,
};
use crate::model::style::{Fill, ShapeBorderLine};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::model::HwpUnit16;
use crate::parser::tags;

use super::utils::{
    attr_str, local_name, parse_bool, parse_color, parse_gradient_type, parse_hatch_style,
    parse_i16, parse_i32, parse_i32_wrapping, parse_i8, parse_u16, parse_u32, parse_u8,
    skip_element,
};
use super::HwpxError;

/// section*.xml을 파싱하여 Section 모델로 변환한다.
pub fn parse_hwpx_section(xml: &str) -> Result<Section, HwpxError> {
    let mut section = Section::default();
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
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

/// section XML의 `<hp:masterPage idRef="...">` 참조를 문서 순서대로 수집한다.
pub fn collect_hwpx_section_master_page_refs(xml: &str) -> Result<Vec<String>, HwpxError> {
    let mut reader = Reader::from_str(xml);
    let mut refs = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if local_name(e.name().as_ref()) == b"masterPage" {
                    push_master_page_id_ref(e, &mut refs);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(HwpxError::XmlError(format!(
                    "section masterPage refs: {}",
                    e
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(refs)
}

fn push_master_page_id_ref(e: &quick_xml::events::BytesStart, refs: &mut Vec<String>) {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"idRef" {
            let id_ref = attr_str(&attr);
            if !id_ref.is_empty() {
                refs.push(id_ref);
            }
        }
    }
}

/// masterpage*.xml을 파싱하여 기존 HWP 바탕쪽 모델로 변환한다.
pub fn parse_hwpx_master_page(xml: &str) -> Result<MasterPage, HwpxError> {
    let mut master_page = MasterPage::default();
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut root_sub_list_seen = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
                match local {
                    b"masterPage" => parse_master_page_start(e, &mut master_page),
                    b"subList" if !root_sub_list_seen => {
                        parse_master_page_sub_list(e, &mut master_page);
                        root_sub_list_seen = true;
                    }
                    b"p" => {
                        let (para, _) = parse_paragraph(e, &mut reader)?;
                        master_page.paragraphs.push(para);
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
                match local {
                    b"masterPage" => parse_master_page_start(e, &mut master_page),
                    b"subList" if !root_sub_list_seen => {
                        parse_master_page_sub_list(e, &mut master_page);
                        root_sub_list_seen = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("masterpage: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    if master_page.text_width > 0 || master_page.text_height > 0 {
        master_page.raw_list_header = build_hwpx_master_page_list_header(&master_page);
    }

    Ok(master_page)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HwpxMasterPageType {
    Both,
    Even,
    Odd,
    LastPage,
    OptionalPage,
}

fn parse_hwpx_master_page_type(value: &str) -> HwpxMasterPageType {
    let normalized: String = value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    match normalized.as_str() {
        "EVEN" => HwpxMasterPageType::Even,
        "ODD" => HwpxMasterPageType::Odd,
        "LASTPAGE" => HwpxMasterPageType::LastPage,
        "OPTIONALPAGE" => HwpxMasterPageType::OptionalPage,
        _ => HwpxMasterPageType::Both,
    }
}

fn parse_master_page_start(e: &quick_xml::events::BytesStart, master_page: &mut MasterPage) {
    let mut is_last_page = false;
    let mut is_optional_page = false;
    let mut page_duplicate: Option<bool> = None;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => {
                let value = attr_str(&attr);
                match parse_hwpx_master_page_type(&value) {
                    HwpxMasterPageType::Even => master_page.apply_to = HeaderFooterApply::Even,
                    HwpxMasterPageType::Odd => master_page.apply_to = HeaderFooterApply::Odd,
                    HwpxMasterPageType::LastPage => {
                        is_last_page = true;
                        master_page.apply_to = HeaderFooterApply::Both;
                        master_page.is_extension = true;
                    }
                    HwpxMasterPageType::OptionalPage => {
                        is_optional_page = true;
                        master_page.apply_to = HeaderFooterApply::Both;
                        master_page.is_extension = true;
                    }
                    HwpxMasterPageType::Both => master_page.apply_to = HeaderFooterApply::Both,
                }
            }
            b"pageDuplicate" => {
                let duplicate = attr_str(&attr) != "0";
                page_duplicate = Some(duplicate);
                master_page.overlap = duplicate;
            }
            b"pageNumber" => master_page.hwpx_page_number = Some(parse_u16(&attr)),
            _ => {}
        }
    }
    // 한컴 HWPX -> HWP5 저장본은 LAST_PAGE 바탕쪽을 확장 바탕쪽으로 저장하면서
    // pageDuplicate="0"인 경우에도 overlap bit를 함께 세운다.
    if is_last_page {
        master_page.replace_base = page_duplicate == Some(false);
        master_page.overlap = true;
    }
    if is_optional_page {
        master_page.overlap = true;
    }
    master_page.ext_flags = u16::from(master_page.overlap)
        | if master_page.is_extension { 0x02 } else { 0 }
        | if is_optional_page { 0x04 } else { 0 };
}

fn parse_master_page_sub_list(e: &quick_xml::events::BytesStart, master_page: &mut MasterPage) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"textWidth" => master_page.text_width = parse_u32(&attr),
            b"textHeight" => master_page.text_height = parse_u32(&attr),
            b"hasTextRef" => master_page.text_ref = parse_u8(&attr),
            b"hasNumRef" => master_page.num_ref = parse_u8(&attr),
            _ => {}
        }
    }
}

fn build_hwpx_master_page_list_header(master_page: &MasterPage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(34);
    bytes.extend_from_slice(&(master_page.paragraphs.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&master_page.text_width.to_le_bytes());
    bytes.extend_from_slice(&master_page.text_height.to_le_bytes());
    bytes.push(master_page.text_ref);
    bytes.push(master_page.num_ref);
    bytes.extend_from_slice(&master_page.ext_flags.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 14]);
    bytes
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
            b"masterPageCnt" => {
                let count = parse_u32(&attr).min(3);
                sec_def.flags = (sec_def.flags & !(0x03 << 30)) | (count << 30);
            }
            // [Task #1058] 한컴 HWP5 spec 표 129 정합:
            //   - spaceColumns → column_spacing (HWPUNIT16, default 1134 for 다단)
            //   - outlineShapeIDRef → outline_numbering_id (UINT16, 1=기본 번호 문단 모양)
            b"spaceColumns" => {
                let v = parse_u32(&attr);
                sec_def.column_spacing = v as i16;
            }
            b"outlineShapeIDRef" => {
                sec_def.outline_numbering_id = parse_u16(&attr);
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
            // [#1166] HWPX 용지 방향. OWPML landscape 값:
            //   WIDELY  = 세로(Portrait)  → landscape=false
            //   NARROWLY= 가로(Landscape) → landscape=true
            // (hwplib ForSecPr: Portrait→WIDELY, Landscape→NARROWLY 매핑 권위.)
            // width/height 는 HWP 바이너리와 동일하게 짧은변=width/긴변=height 로
            // 저장되고, landscape=true 일 때 렌더러가 swap 한다(page.rs). 종전엔
            // landscape 를 무시해 가로 용지 HWPX 가 항상 세로로 렌더되는 결함.
            b"landscape" => {
                page.landscape = attr_str(&attr).eq_ignore_ascii_case("NARROWLY");
            }
            b"gutterType" => {
                let value = attr_str(&attr);
                let binding_code = match value.as_str() {
                    "LEFT_RIGHT" => 1,
                    "TOP_BOTTOM" => 2,
                    _ => 0,
                };
                page.attr = (page.attr & !(0x03 << 1)) | (binding_code << 1);
                page.binding = match binding_code {
                    1 => BindingMethod::DuplexSided,
                    2 => BindingMethod::TopFlip,
                    _ => BindingMethod::SingleSided,
                };
            }
            _ => {}
        }
    }
}

fn parse_grid(e: &quick_xml::events::BytesStart, sec_def: &mut SectionDef) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"lineGrid" => sec_def.line_grid = parse_i32(&attr) as i16,
            b"charGrid" => sec_def.char_grid = parse_i32(&attr) as i16,
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
    // [Task #1058 후속] HWPX `<hp:p id>` → HWP PARA_HEADER instance_id (UINT32) 직접 매핑.
    // HWPX 의 id 값 ("0" 또는 "2147483648"=0x80000000) 이 한컴 정답지의 instance_id 패턴과
    // 정확 일치. 누락 시 한컴편집기가 각주 추가 시 본문 다단계 목록 부여 (Task #1058 본질).
    let mut hp_p_id: u32 = 0;
    let mut has_column_break_attr = false;
    let mut has_page_break_attr = false;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"id" => {
                if let Ok(s) = std::str::from_utf8(&attr.value) {
                    hp_p_id = s.parse::<u32>().unwrap_or(0);
                }
            }
            b"paraPrIDRef" => para.para_shape_id = parse_u16(&attr),
            b"styleIDRef" => para.style_id = parse_u8(&attr),
            b"columnBreak" => {
                if parse_u8(&attr) == 1 {
                    has_column_break_attr = true;
                    para.column_type = crate::model::paragraph::ColumnBreakType::Column;
                }
            }
            b"pageBreak" => {
                if parse_u8(&attr) == 1 {
                    has_page_break_attr = true;
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
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
                        sec_def = Some(sd.clone());
                        // [Task #901] SectionDef 도 HWP 바이너리에서 8 utf16 inline marker —
                        // line_seg.text_start (file 값) 가 HWP 인코딩 가정. HWPX parser
                        // 가 utf16_pos 동기화하지 않으면 paragraph 0 의 compose_lines 가
                        // 모든 chars 를 line 0 에 packing. \u{0002} 추가로 8 utf16 정합.
                        para.controls.push(Control::SectionDef(Box::new(sd)));
                        text_parts.push("\u{0002}".to_string());
                        // colPr이 있으면 ColumnDef 컨트롤 추가 (초기 단 정의) + 8 utf16.
                        if let Some(cd) = col_def_opt {
                            para.controls.push(Control::ColumnDef(cd));
                            text_parts.push("\u{0002}".to_string());
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"run" => {
                        // self-closing 빈 run (예: <hp:run charPrIDRef="42"/>)
                        // 빈 paragraph 의 char_shape 가 누락되어 default(id=0) 로
                        // 처리되면 line height 계산이 어긋나 pagination 차이 발생.
                        for attr in ce.attributes().flatten() {
                            if attr.key.as_ref() == b"charPrIDRef" {
                                current_char_shape_id = parse_u32(&attr);
                            }
                        }
                        let utf16_pos = calc_utf16_len_from_parts(&text_parts);
                        char_shape_changes.push((utf16_pos, current_char_shape_id));
                    }
                    b"lineBreak" | b"softHyphen" => {
                        text_parts.push("\n".to_string());
                    }
                    b"columnBreak" => {
                        text_parts.push("\n".to_string());
                    }
                    b"tab" => {
                        text_parts.push("\t".to_string());
                        para.tab_extended.push(parse_tab_extension(ce));
                    }
                    b"lineseg" => {
                        // 단독 lineseg (linesegarray 밖에 나올 경우)
                        para.line_segs.push(parse_lineseg_element(ce));
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"p" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("paragraph: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // FIELD_BEGIN/FIELD_END 쌍은 HWP PARA_TEXT에서 각각 8 code unit을 차지한다.
    // HWPX 파싱 결과를 HWP로 다시 저장할 때 FIELD_END를 복원하려면, visible text
    // 범위와 해당 Field 컨트롤 index를 field_ranges에 남겨야 한다.
    let mut field_ranges: Vec<FieldRange> = Vec::new();
    let mut field_stack: Vec<(usize, usize)> = Vec::new();
    let mut control_idx: usize = 0;
    let mut visible_char_idx: usize = 0;

    for part in &text_parts {
        match part.as_str() {
            "\u{0003}" => {
                if matches!(para.controls.get(control_idx), Some(Control::Field(_))) {
                    field_stack.push((visible_char_idx, control_idx));
                }
                control_idx += 1;
            }
            "\u{0004}" => {
                if let Some((start_char_idx, control_idx)) = field_stack.pop() {
                    field_ranges.push(FieldRange {
                        start_char_idx,
                        end_char_idx: visible_char_idx,
                        control_idx,
                    });
                }
            }
            "\u{0002}" => {
                control_idx += 1;
            }
            "\u{0012}" => {
                control_idx += 1;
                visible_char_idx += 1;
            }
            _ => {
                visible_char_idx += part.chars().count();
            }
        }
    }
    para.field_ranges = field_ranges;

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
            "\u{0012}" => {
                // [Task #1050] AUTO_NUMBER (0x12) — HWP PARA_TEXT 정합:
                //   char_offsets.push(pos) + text.push(' ') (placeholder) + jump 8.
                char_offsets.push(utf16_pos);
                visual_text.push(' ');
                utf16_pos += 8;
            }
            _ => {
                for c in part.chars() {
                    char_offsets.push(utf16_pos);
                    visual_text.push(c);
                    let width = if c == '\t' {
                        8
                    } else if (c as u32) > 0xFFFF {
                        2
                    } else {
                        1
                    };
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
    // [Task #1058 후속] 같은 char_shape_id 연속 dedup — HWPX 의 여러 run 이 같은
    // charPrIDRef 일 때 HWP PARA_CHAR_SHAPE 는 첫 entry 1개만 유지하므로 정합.
    let mut deduped_cs: Vec<CharShapeRef> = Vec::new();
    for (pos, id) in char_shape_changes {
        if let Some(last) = deduped_cs.last() {
            if last.char_shape_id == id {
                continue;
            }
        }
        deduped_cs.push(CharShapeRef {
            start_pos: pos,
            char_shape_id: id,
        });
    }
    para.char_shapes = deduped_cs;

    // [Task #1058 후속] column_type/raw_break_type — HWP 정합 (스펙 표 59):
    //   bit 0 (0x01) = 구역 나누기, bit 1 (0x02) = 다단 나누기,
    //   bit 2 (0x04) = 쪽 나누기,  bit 3 (0x08) = 단 나누기
    // HWPX는 pageBreak/columnBreak attr 과 secPr/colPr 구조를 분리해 저장하므로, HWP5
    // PARA_HEADER 에서는 각 축을 bitwise 로 합성해야 한다. 후속 구역이라고 해서
    // 무조건 0x04(쪽 나누기)로 덮으면, pageBreak 없는 "구역+다단" 문단이 한컴에서
    // 다른 layout contract 로 해석된다.
    let has_section = sec_def.is_some();
    let has_column_def = para
        .controls
        .iter()
        .any(|c| matches!(c, Control::ColumnDef(_)));
    if para.raw_break_type == 0 {
        let mut break_type = 0u8;
        if has_section {
            break_type |= 0x01;
        }
        if has_column_def {
            break_type |= 0x02;
        }
        if has_page_break_attr {
            break_type |= 0x04;
        }
        if has_column_break_attr {
            break_type |= 0x08;
        }

        if break_type != 0 {
            para.raw_break_type = break_type;
            para.column_type = if break_type & 0x04 != 0 {
                crate::model::paragraph::ColumnBreakType::Page
            } else if break_type & 0x08 != 0 {
                crate::model::paragraph::ColumnBreakType::Column
            } else if break_type & 0x01 != 0 {
                crate::model::paragraph::ColumnBreakType::Section
            } else {
                crate::model::paragraph::ColumnBreakType::MultiColumn
            };
        }
    }

    // 기본 line_seg (빈 문단이라도 최소 1개)
    if para.line_segs.is_empty() {
        para.line_segs.push(LineSeg {
            text_start: 0,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        });
    }

    // [Task #1058 후속] HWPX `<hp:p id>` → HWP PARA_HEADER instance_id 매핑.
    // raw_header_extra 구조 (serializer 정합 — body_text.rs:241):
    //   raw_header_extra[0..6] = numCharShapes(2) + numRangeTags(2) + numLineSegs(2)
    //                              ← serializer 가 건너뜀 (실제 데이터 기반 재계산)
    //   raw_header_extra[6..10] = instanceId (UINT32 LE) ← HWPX `id` 매핑
    // raw_header_extra 가 비어 있으면 serializer 가 instance_id=0 으로 작성.
    // 한컴편집기 호환을 위해 HWPX 의 id 값을 정확히 보존.
    let mut header_extra = Vec::with_capacity(10);
    header_extra.extend_from_slice(&[0u8; 6]); // numCharShapes/numRangeTags/numLineSegs 자리
    header_extra.extend_from_slice(&hp_p_id.to_le_bytes()); // instanceId
    para.raw_header_extra = header_extra;

    Ok((para, sec_def))
}

/// secPr의 자식 요소들 (pagePr, margin, colPr 등) 파싱
/// 반환: 파싱된 ColumnDef (없으면 None)
fn parse_sec_pr_children(
    reader: &mut Reader<&[u8]>,
    sec_def: &mut SectionDef,
) -> Result<Option<ColumnDef>, HwpxError> {
    let mut buf = Vec::new();
    let mut col_def: Option<ColumnDef> = None;
    let mut page_border_fill_count = 0usize;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
                match local {
                    b"pagePr" => parse_page_pr(e, &mut sec_def.page_def),
                    b"margin" => parse_page_margin(e, &mut sec_def.page_def),
                    b"grid" => parse_grid(e, sec_def),
                    b"colPr" => {
                        col_def = Some(parse_col_pr_with_children(e, reader)?);
                    }
                    b"startNum" => parse_start_num(e, sec_def),
                    b"visibility" => parse_visibility(e, sec_def),
                    b"pageBorderFill" => {
                        let pbf = parse_page_border_fill(e, reader)?;
                        push_page_border_fill(sec_def, pbf, &mut page_border_fill_count);
                    }
                    // [Task #1050] footNotePr / endNotePr 의 자식 (autoNumFormat, noteLine 등)
                    // 파싱 — 한컴 정답 footnote 영역 렌더링을 위한 FootnoteShape contract.
                    b"footNotePr" => {
                        parse_note_pr_children(reader, &mut sec_def.footnote_shape, b"footNotePr")?;
                    }
                    b"endNotePr" => {
                        parse_note_pr_children(reader, &mut sec_def.endnote_shape, b"endNotePr")?;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
                match local {
                    b"pagePr" => parse_page_pr(e, &mut sec_def.page_def),
                    b"margin" => parse_page_margin(e, &mut sec_def.page_def),
                    b"grid" => parse_grid(e, sec_def),
                    b"colPr" => {
                        col_def = Some(parse_col_pr(e));
                    }
                    b"startNum" => parse_start_num(e, sec_def),
                    b"visibility" => parse_visibility(e, sec_def),
                    b"pageBorderFill" => {
                        let pbf = parse_page_border_fill_empty(e);
                        push_page_border_fill(sec_def, pbf, &mut page_border_fill_count);
                    }
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

/// [Task #1050] `<hp:footNotePr>` / `<hp:endNotePr>` 의 자식 요소 파싱:
///   - `<hp:autoNumFormat type="DIGIT" suffixChar=")" prefixChar="" userChar="">` → FootnoteShape
///   - `<hp:noteLine length="-1" type="SOLID" width="0.12 mm" color="#000000">` → separator_*
///   - `<hp:noteSpacing betweenNotes="" belowLine="" aboveLine="">` → spacing
///   - `<hp:numbering type="CONTINUOUS" newNum="1">` → numbering
///   - `<hp:placement place="EACH_COLUMN" beneathText="0">` → placement
fn parse_note_pr_children(
    reader: &mut Reader<&[u8]>,
    shape: &mut crate::model::footnote::FootnoteShape,
    end_tag: &[u8],
) -> Result<(), HwpxError> {
    let is_end_note = end_tag == b"endNotePr";
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
                match local {
                    b"autoNumFormat" => {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"suffixChar" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Some(c) = s.chars().next() {
                                            shape.suffix_char = c;
                                        }
                                    }
                                }
                                b"prefixChar" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Some(c) = s.chars().next() {
                                            shape.prefix_char = c;
                                        }
                                    }
                                }
                                b"userChar" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Some(c) = s.chars().next() {
                                            shape.user_char = c;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"noteLine" => {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"length" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Ok(v) = s.parse::<i32>() {
                                            shape.separator_length = if v < 0 {
                                                v as i16
                                            } else {
                                                (v as u32 as u16) as i16
                                            };
                                        }
                                    }
                                }
                                b"type" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        shape.separator_line_type = match s {
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
                                            "NONE" => 0,
                                            _ => 1, // default SOLID
                                        };
                                    }
                                }
                                b"width" => {
                                    // 미주/각주 구분선 굵기도 테두리 굵기 raw 코드와 같은 표를 쓴다.
                                    // 예: 0.12mm → 1, 0.7mm → 9.
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        shape.separator_line_width = parse_hwpx_line_width(s);
                                    }
                                }
                                b"color" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        // "#RRGGBB" → ColorRef (0xBBGGRR LE = HWP 표준)
                                        if let Some(hex) = s.strip_prefix('#') {
                                            if let Ok(rgb) = u32::from_str_radix(hex, 16) {
                                                let r = (rgb >> 16) & 0xFF;
                                                let g = (rgb >> 8) & 0xFF;
                                                let b = rgb & 0xFF;
                                                shape.separator_color = b << 16 | g << 8 | r;
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"noteSpacing" => {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                // [Task #1050] HWP5 spec 의 정답지 매핑:
                                // betweenNotes → raw_unknown (실제는 between-notes spacing)
                                // belowLine → note_spacing
                                // aboveLine → separator_margin_bottom
                                b"betweenNotes" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Ok(v) = s.parse::<u16>() {
                                            shape.raw_unknown = v;
                                        }
                                    }
                                }
                                b"belowLine" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Ok(v) = s.parse::<i16>() {
                                            shape.note_spacing = v;
                                        }
                                    }
                                }
                                b"aboveLine" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Ok(v) = s.parse::<i16>() {
                                            shape.separator_margin_bottom = v;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        // [Task #1050] separator_margin_top: HWPX 미보유 → 한컴 default -1 (sentinel).
                        // 단, 미주 noteLine type="NONE" 에서는 한컴 저장본이 0을 유지한다.
                        if shape.separator_margin_top == 0 && shape.separator_line_type != 0 {
                            shape.separator_margin_top =
                                if is_end_note && shape.separator_length > 0 {
                                    224
                                } else {
                                    -1
                                };
                        }
                    }
                    b"numbering" => {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"type" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        let (numbering, attr_bits) = match s {
                                            "CONTINUOUS" | "continue" => (
                                                crate::model::footnote::FootnoteNumbering::Continue,
                                                0u32,
                                            ),
                                            "ON_SECTION" | "RESTART_SECTION" | "restartSection" => (
                                                crate::model::footnote::FootnoteNumbering::RestartSection,
                                                1u32,
                                            ),
                                            "ON_PAGE" | "RESTART_PAGE" | "restartPage" => (
                                                crate::model::footnote::FootnoteNumbering::RestartPage,
                                                2u32,
                                            ),
                                            _ => continue,
                                        };
                                        shape.numbering = numbering;
                                        shape.attr = (shape.attr & !(0x03 << 8)) | (attr_bits << 8);
                                    }
                                }
                                b"newNum" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        if let Ok(v) = s.parse::<u16>() {
                                            shape.start_number = v;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"placement" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"place" {
                                if let Ok(s) = std::str::from_utf8(&attr.value) {
                                    let (placement, attr_bits) = match s {
                                        "END_OF_SECTION" | "BELOW_TEXT" | "sectionEnd"
                                        | "belowText" => (
                                            crate::model::footnote::FootnotePlacement::BelowText,
                                            1u32,
                                        ),
                                        "RIGHT_COLUMN" | "rightColumn" => (
                                            crate::model::footnote::FootnotePlacement::RightColumn,
                                            2u32,
                                        ),
                                        "END_OF_DOCUMENT" | "EACH_COLUMN" | "documentEnd"
                                        | "eachColumn" => (
                                            crate::model::footnote::FootnotePlacement::EachColumn,
                                            0u32,
                                        ),
                                        _ => continue,
                                    };
                                    shape.placement = placement;
                                    shape.attr = (shape.attr & !(0x03 << 8)) | (attr_bits << 8);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                if local_name(e.name().as_ref()) == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(HwpxError::XmlError(format!(
                    "{}: {}",
                    std::str::from_utf8(end_tag).unwrap_or("notePr"),
                    e
                )))
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn push_page_border_fill(
    sec_def: &mut SectionDef,
    page_border_fill: PageBorderFill,
    count: &mut usize,
) {
    if *count == 0 {
        sec_def.page_border_fill = page_border_fill;
    } else {
        sec_def.extra_page_border_fills.push(page_border_fill);
    }
    *count += 1;
}

fn parse_page_border_fill(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<PageBorderFill, HwpxError> {
    let mut page_border_fill = parse_page_border_fill_empty(e);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref child)) | Ok(Event::Empty(ref child)) => {
                if local_name(child.name().as_ref()) == b"offset" {
                    parse_page_border_fill_offset(child, &mut page_border_fill);
                }
            }
            Ok(Event::End(ref end)) => {
                if local_name(end.name().as_ref()) == b"pageBorderFill" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(HwpxError::XmlError(format!("pageBorderFill: {}", err)));
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(page_border_fill)
}

fn parse_page_border_fill_empty(e: &quick_xml::events::BytesStart) -> PageBorderFill {
    let mut page_border_fill = PageBorderFill::default();
    let mut text_border = String::new();
    let mut fill_area = String::new();
    let mut apply_type = String::new();
    let mut header_inside = false;
    let mut footer_inside = false;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"borderFillIDRef" => page_border_fill.border_fill_id = parse_u16(&attr),
            b"textBorder" => text_border = attr_str(&attr),
            b"fillArea" => fill_area = attr_str(&attr),
            b"type" => apply_type = attr_str(&attr),
            b"headerInside" => header_inside = parse_bool(&attr),
            b"footerInside" => footer_inside = parse_bool(&attr),
            _ => {}
        }
    }

    page_border_fill.attr = page_border_fill_attr(
        &text_border,
        &fill_area,
        &apply_type,
        header_inside,
        footer_inside,
    );
    page_border_fill.ui_basis = if text_border.eq_ignore_ascii_case("PAPER") {
        // Task #1129 Stage 28: textBorder=PAPER is shown as page basis in the
        // dialog and renders from the page/body area edge.
        page_border_fill.basis = PageBorderBasis::BodyBased;
        PageBorderUiBasis::Page
    } else {
        page_border_fill.basis = PageBorderBasis::PaperBased;
        PageBorderUiBasis::Paper
    };
    page_border_fill
}

fn parse_page_border_fill_offset(
    e: &quick_xml::events::BytesStart,
    page_border_fill: &mut PageBorderFill,
) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"left" => page_border_fill.spacing_left = parse_i16(&attr),
            b"right" => page_border_fill.spacing_right = parse_i16(&attr),
            b"top" => page_border_fill.spacing_top = parse_i16(&attr),
            b"bottom" => page_border_fill.spacing_bottom = parse_i16(&attr),
            _ => {}
        }
    }
}

fn page_border_fill_attr(
    text_border: &str,
    fill_area: &str,
    apply_type: &str,
    header_inside: bool,
    footer_inside: bool,
) -> u32 {
    let mut attr = 0u32;

    if text_border.eq_ignore_ascii_case("PAPER") {
        attr |= 0x0000_0001;
    }
    if header_inside {
        attr |= 0x0000_0002;
    }
    if footer_inside {
        attr |= 0x0000_0004;
    }

    attr |= match fill_area {
        area if area.eq_ignore_ascii_case("PAGE") => 0x0000_0008,
        area if area.eq_ignore_ascii_case("BORDER") => 0x0000_0010,
        _ => 0,
    };

    attr
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
            b"hideFirstHeader" => {
                sec_def.hide_header = attr_str(&attr) == "1";
                if sec_def.hide_header {
                    sec_def.flags |= 0x0001;
                } else {
                    sec_def.flags &= !0x0001;
                }
            }
            b"hideFirstFooter" => {
                sec_def.hide_footer = attr_str(&attr) == "1";
                if sec_def.hide_footer {
                    sec_def.flags |= 0x0002;
                } else {
                    sec_def.flags &= !0x0002;
                }
            }
            b"hideFirstMasterPage" => {
                sec_def.hide_master_page = attr_str(&attr) == "1";
                if sec_def.hide_master_page {
                    sec_def.flags |= 0x0004;
                } else {
                    sec_def.flags &= !0x0004;
                }
            }
            b"border" => {
                sec_def.hide_border = attr_str(&attr) == "HIDE_ALL";
                if sec_def.hide_border {
                    sec_def.flags |= 0x0008;
                } else {
                    sec_def.flags &= !0x0008;
                }
            }
            b"fill" => {
                sec_def.hide_fill = attr_str(&attr) == "HIDE_ALL";
                if sec_def.hide_fill {
                    sec_def.flags |= 0x0010;
                } else {
                    sec_def.flags &= !0x0010;
                }
            }
            b"hideFirstEmptyLine" => sec_def.hide_empty_line = attr_str(&attr) == "1",
            _ => {}
        }
    }
}

/// <hp:colPr> 요소의 속성 파싱 → ColumnDef
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

/// <hp:colPr> 요소의 속성과 자식 <hp:colLine> 파싱 → ColumnDef
fn parse_col_pr_with_children(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<ColumnDef, HwpxError> {
    let mut cd = parse_col_pr(e);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                match local_name(cname.as_ref()) {
                    b"colLine" => parse_col_line(ce, &mut cd),
                    _ => {}
                }
            }
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"colLine" => {
                        parse_col_line(ce, &mut cd);
                        skip_element(reader, b"colLine")?;
                    }
                    _ => {
                        let tag = local.to_vec();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Ok(Event::End(ref ee)) => {
                if local_name(ee.name().as_ref()) == b"colPr" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("colPr: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(cd)
}

fn parse_col_line(e: &quick_xml::events::BytesStart, cd: &mut ColumnDef) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => cd.separator_type = parse_hwpx_line_type(&attr_str(&attr)),
            b"width" => cd.separator_width = parse_hwpx_line_width(&attr_str(&attr)),
            b"color" => cd.separator_color = parse_color(&attr),
            _ => {}
        }
    }
}

fn parse_hwpx_line_type(value: &str) -> u8 {
    match value {
        "NONE" => 0,
        "SOLID" => 1,
        "DASH" => 2,
        "DOT" => 3,
        "DASH_DOT" => 4,
        "DASH_DOT_DOT" => 5,
        "LONG_DASH" => 6,
        "CIRCLE" => 7,
        _ => 1,
    }
}

fn parse_hwpx_line_width(value: &str) -> u8 {
    let mm: f64 = value
        .split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.12);

    if mm <= 0.10 {
        0
    } else if mm <= 0.12 {
        1
    } else if mm <= 0.15 {
        2
    } else if mm <= 0.20 {
        3
    } else if mm <= 0.25 {
        4
    } else if mm <= 0.30 {
        5
    } else if mm <= 0.40 {
        6
    } else if mm <= 0.50 {
        7
    } else if mm <= 0.60 {
        8
    } else if mm <= 0.70 {
        9
    } else if mm <= 1.00 {
        10
    } else if mm <= 1.50 {
        11
    } else if mm <= 2.00 {
        12
    } else if mm <= 3.00 {
        13
    } else if mm <= 4.00 {
        14
    } else {
        15
    }
}

/// <hp:linesegarray> 내부의 <hp:lineseg> 요소들을 파싱한다.
fn parse_lineseg_array(reader: &mut Reader<&[u8]>, para: &mut Paragraph) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                let ename = e.name();
                let local = local_name(ename.as_ref());
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

fn decode_xml_general_ref(r: &BytesRef<'_>) -> String {
    if let Ok(Some(ch)) = r.resolve_char_ref() {
        return ch.to_string();
    }

    let name = r.decode().unwrap_or_default();
    match name.as_ref() {
        "lt" => "<".to_string(),
        "gt" => ">".to_string(),
        "amp" => "&".to_string(),
        "quot" => "\"".to_string(),
        "apos" => "'".to_string(),
        _ => format!("&{};", name),
    }
}

fn read_text_content_with_tabs(
    reader: &mut Reader<&[u8]>,
) -> Result<(String, Vec<[u16; 7]>), HwpxError> {
    let mut text = String::new();
    let mut tab_ext_buf: Vec<[u16; 7]> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                text.push_str(&t.decode().unwrap_or_default());
            }
            Ok(Event::GeneralRef(ref r)) => {
                text.push_str(&decode_xml_general_ref(r));
            }
            Ok(Event::End(ref e)) => {
                let tn = e.name();
                if local_name(tn.as_ref()) == b"t" {
                    break;
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"lineBreak" | b"columnBreak" => text.push('\n'),
                    b"tab" => {
                        text.push('\t');
                        tab_ext_buf.push(parse_tab_extension(ce));
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

fn parse_tab_extension(e: &quick_xml::events::BytesStart) -> [u16; 7] {
    let mut ext = [0u16; 7];
    ext[6] = 0x0009;
    let mut leader = 0u16;
    let mut tab_type = 0u16;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"width" => ext[0] = parse_u16(&attr),
            b"leader" => leader = parse_u16(&attr) & 0x00ff,
            b"type" => tab_type = parse_u16(&attr) & 0x00ff,
            _ => {}
        }
    }
    ext[2] = (tab_type << 8) | leader;

    ext
}

// ─── Table ───

fn parse_table(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Table, HwpxError> {
    let mut table = Table::default();
    let mut table_record_flags = 0u32;

    // 표 기본 속성
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"id" | b"instid" => table.common.instance_id = parse_u32(&attr),
            b"zOrder" => table.common.z_order = parse_i32(&attr),
            b"rowCnt" => table.row_count = parse_u16(&attr),
            b"colCnt" => table.col_count = parse_u16(&attr),
            b"cellSpacing" => table.cell_spacing = parse_i16(&attr),
            b"borderFillIDRef" => table.border_fill_id = parse_u16(&attr),
            b"noAdjust" => {
                if attr_str(&attr) == "1" {
                    table_record_flags |= 0x08;
                }
            }
            b"pageBreak" => {
                let val = attr_str(&attr);
                table.page_break = match val.as_str() {
                    // HWPX pageBreak="CELL" is serialized by Hancom as HWP5
                    // row-break (TABLE attr bit 1). HWPX pageBreak="TABLE"
                    // is serialized as HWP5 cell/table break (bit 0).
                    "TABLE" | "TABLE_BREAK" => TablePageBreak::CellBreak,
                    "CELL" | "CELL_BREAK" => TablePageBreak::RowBreak,
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
            b"textFlow" => {
                table.common.text_flow = match attr_str(&attr).as_str() {
                    "LEFT_ONLY" => crate::model::shape::TextFlow::LeftOnly,
                    "RIGHT_ONLY" => crate::model::shape::TextFlow::RightOnly,
                    "LARGEST_ONLY" => crate::model::shape::TextFlow::LargestOnly,
                    _ => crate::model::shape::TextFlow::BothSides,
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"sz" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => {
                                    table.common.width = parse_u32(&attr);
                                }
                                b"height" => {
                                    table.common.height = parse_u32(&attr);
                                }
                                b"widthRelTo" => {
                                    table.common.width_criterion =
                                        parse_size_criterion(&attr_str(&attr), true);
                                }
                                b"heightRelTo" => {
                                    table.common.height_criterion =
                                        parse_size_criterion(&attr_str(&attr), false);
                                }
                                _ => {}
                            }
                        }
                    }
                    b"pos" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"treatAsChar" => {
                                    table.common.treat_as_char =
                                        attr_str(&attr) == "1" || attr_str(&attr) == "true";
                                }
                                b"flowWithText" => table.common.flow_with_text = parse_bool(&attr),
                                b"allowOverlap" => table.common.allow_overlap = parse_bool(&attr),
                                b"holdAnchorAndSO" => {
                                    table.common.prevent_page_break =
                                        if parse_bool(&attr) { 1 } else { 0 };
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
                                b"vertOffset" => {
                                    table.common.vertical_offset = parse_i32(&attr) as u32;
                                }
                                b"horzOffset" => {
                                    table.common.horizontal_offset = parse_i32(&attr) as u32;
                                }
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
                let eename = ee.name();
                let local = local_name(eename.as_ref());
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
        let max_h = table
            .cells
            .iter()
            .filter(|c| c.row == r && c.row_span == 1)
            .map(|c| c.height as i16)
            .max()
            .unwrap_or(0);
        row_sizes.push(max_h);
    }
    table.row_sizes = row_sizes;

    materialize_hwpx_table_attrs(&mut table, table_record_flags);
    table.rebuild_grid();
    Ok(table)
}

fn parse_size_criterion(value: &str, allow_column_para: bool) -> SizeCriterion {
    match value {
        "PAPER" => SizeCriterion::Paper,
        "PAGE" => SizeCriterion::Page,
        "COLUMN" if allow_column_para => SizeCriterion::Column,
        "PARA" if allow_column_para => SizeCriterion::Para,
        _ => SizeCriterion::Absolute,
    }
}

fn materialize_hwpx_table_attrs(table: &mut Table, table_record_flags: u32) {
    const HWPX_TABLE_NUMBERING_BIT: u32 = 0x0800_0000;

    table.common.attr = pack_hwpx_common_obj_attr(&table.common) | HWPX_TABLE_NUMBERING_BIT;
    // HWPX keeps semantic placement in hp:pos, while legacy layout code still reads
    // table.attr bit0 for some inline-table decisions. Only mirror the minimum
    // renderer compatibility bit here; the HWP5 storage attr is packed later by
    // the HWP adapter.
    table.attr = if table.common.treat_as_char && table.common.flow_with_text {
        0x01
    } else {
        0
    };
    let mut record_attr = match table.page_break {
        TablePageBreak::CellBreak => 0x01,
        TablePageBreak::RowBreak => 0x02,
        TablePageBreak::None => 0,
    };
    if table.repeat_header {
        record_attr |= 0x04;
    }
    if table_record_flags & 0x08 != 0 {
        record_attr |= 0x08;
    }
    if table.padding.left != 0
        || table.padding.right != 0
        || table.padding.top != 0
        || table.padding.bottom != 0
    {
        record_attr |= 0x0400_0000;
    }
    table.raw_table_record_attr = record_attr;
}

fn pack_hwpx_common_obj_attr(common: &CommonObjAttr) -> u32 {
    let mut attr = 0u32;
    if common.treat_as_char {
        attr |= 0x01;
    }
    if common.flow_with_text {
        attr |= 1 << 13;
    }
    if common.allow_overlap {
        attr |= 1 << 14;
    }
    if common.size_protect {
        attr |= 1 << 20;
    }
    if common.hwp5_gen_shape_attr_bit26 {
        attr |= 1 << 26;
    }
    if common.hwp5_gen_shape_attr_bit28 {
        attr |= 1 << 28;
    }

    attr |= (match common.vert_rel_to {
        VertRelTo::Paper => 0,
        VertRelTo::Page => 1,
        VertRelTo::Para => 2,
    }) << 3;
    attr |= (match common.vert_align {
        VertAlign::Top => 0,
        VertAlign::Center => 1,
        VertAlign::Bottom => 2,
        VertAlign::Inside => 3,
        VertAlign::Outside => 4,
    }) << 5;
    attr |= (match common.horz_rel_to {
        HorzRelTo::Paper => 0,
        HorzRelTo::Page => 1,
        HorzRelTo::Column => 2,
        HorzRelTo::Para => 3,
    }) << 8;
    attr |= (match common.horz_align {
        HorzAlign::Left => 0,
        HorzAlign::Center => 1,
        HorzAlign::Right => 2,
        HorzAlign::Inside => 3,
        HorzAlign::Outside => 4,
    }) << 10;
    attr |= (match common.width_criterion {
        SizeCriterion::Paper => 0,
        SizeCriterion::Page => 1,
        SizeCriterion::Column => 2,
        SizeCriterion::Para => 3,
        SizeCriterion::Absolute => 4,
    }) << 15;
    attr |= (match common.height_criterion {
        SizeCriterion::Paper => 0,
        SizeCriterion::Page => 1,
        _ => 2,
    }) << 18;
    attr |= (match common.text_wrap {
        TextWrap::Square | TextWrap::Tight | TextWrap::Through => 0,
        TextWrap::TopAndBottom => 1,
        TextWrap::BehindText => 2,
        TextWrap::InFrontOfText => 3,
    }) << 21;
    attr |= (match common.text_flow {
        crate::model::shape::TextFlow::BothSides => 0,
        crate::model::shape::TextFlow::LeftOnly => 1,
        crate::model::shape::TextFlow::RightOnly => 2,
        crate::model::shape::TextFlow::LargestOnly => 3,
    }) << 24;

    attr
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
            b"hasMargin" => cell.apply_inner_margin = attr_str(&attr) == "1",
            _ => {}
        }
    }

    // 셀 자식 요소 파싱
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"cellAddr" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"colAddr" => {
                                    cell.col = parse_u16(&attr);
                                }
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"cellAddr" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"colAddr" => {
                                    cell.col = parse_u16(&attr);
                                }
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
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"tc" {
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
    common.hwp5_gen_shape_attr_bit26 = true;
    let mut shape_attr = ShapeComponentAttr::default();
    let mut crop = CropInfo::default();
    let mut padding = crate::model::Padding::default();
    let mut border_x = [0i32; 4];
    let mut border_y = [0i32; 4];
    let mut href: Option<String> = None;
    let mut picture_instance_id = 0;

    // <hp:pic> 요소 자체의 속성 파싱
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"id" => common.instance_id = parse_u32(&attr),
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
            b"textFlow" => {
                common.text_flow = match attr_str(&attr).as_str() {
                    "LEFT_ONLY" => crate::model::shape::TextFlow::LeftOnly,
                    "RIGHT_ONLY" => crate::model::shape::TextFlow::RightOnly,
                    "LARGEST_ONLY" => crate::model::shape::TextFlow::LargestOnly,
                    _ => crate::model::shape::TextFlow::BothSides,
                };
            }
            b"instid" => picture_instance_id = parse_u32(&attr),
            b"href" => {
                let value = attr_str(&attr);
                if !value.is_empty() {
                    href = Some(value);
                }
            }
            b"groupLevel" => shape_attr.group_level = attr_str(&attr).parse().unwrap_or(0),
            _ => {}
        }
    }

    // 이미지 속성 읽기
    let mut has_pos = false; // <pos> 파싱 여부 — <offset>이 덮어쓰지 않도록 방지
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"imgRect" => {
                parse_picture_img_rect(reader, &mut border_x, &mut border_y)?;
            }
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"shapeComment" => {
                common.description = read_dutmal_text(reader, b"shapeComment")?;
            }
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"sz" => {
                        // 최종 표시 크기 (최우선)
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => {
                                    let v = parse_u32(&attr);
                                    if v > 0 {
                                        common.width = v;
                                    }
                                }
                                b"height" => {
                                    let v = parse_u32(&attr);
                                    if v > 0 {
                                        common.height = v;
                                    }
                                }
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
                                    if v > 0 {
                                        common.width = v;
                                    }
                                }
                                b"height" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.current_height = v;
                                    if v > 0 {
                                        common.height = v;
                                    }
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
                                    if common.width == 0 {
                                        common.width = v;
                                    }
                                }
                                b"height" => {
                                    let v = parse_u32(&attr);
                                    shape_attr.original_height = v;
                                    if common.height == 0 {
                                        common.height = v;
                                    }
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
                                    common.treat_as_char =
                                        attr_str(&attr) == "1" || attr_str(&attr) == "true";
                                }
                                b"flowWithText" => common.flow_with_text = parse_bool(&attr),
                                b"allowOverlap" => common.allow_overlap = parse_bool(&attr),
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
                                b"vertOffset" => {
                                    common.vertical_offset = parse_i32_wrapping(&attr) as u32
                                }
                                b"horzOffset" => {
                                    common.horizontal_offset = parse_i32_wrapping(&attr) as u32
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
                                    let num: String =
                                        val.chars().filter(|c| c.is_ascii_digit()).collect();
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
                    b"flip" => {
                        parse_shape_flip(ce, &mut shape_attr);
                    }
                    b"rotationInfo" => {
                        parse_shape_rotation_info(ce, &mut shape_attr);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == b"pic" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("pic: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    if common.instance_id == 0 && picture_instance_id != 0 {
        common.instance_id = picture_instance_id;
    }

    materialize_shape_hwp_storage_defaults(&mut common, &mut shape_attr, ShapeStorageKind::Picture);

    let mut pic = crate::model::image::Picture::default();
    pic.image_attr = img_attr;
    pic.common = common;
    pic.shape_attr = shape_attr;
    pic.href = href;
    pic.crop = crop;
    pic.padding = padding;
    pic.border_x = border_x;
    pic.border_y = border_y;
    pic.instance_id = picture_instance_id;

    Ok(Control::Picture(Box::new(pic)))
}

// ─── 그리기 객체 공통 속성 파싱 ───

#[derive(Clone, Copy)]
enum ShapeStorageKind {
    Picture,
    Group,
    Drawing,
    TextBoxDrawing,
}

#[derive(Default)]
struct ObjectElementIds {
    instid: u32,
    round_rate: u8,
}

/// HWPX 일부 샘플은 `<hp:curSz width="0" height="0">`를 기록하면서 실제 크기는
/// `<hp:orgSz>`와 `renderingInfo` scale로 표현한다. HWP 저장/재로드 경로에서는
/// current size 0이 effective size 0으로 해석되므로, 저장 가능한 IR에서는 current
/// size를 org size로 materialize한다.
fn materialize_shape_current_size_from_original(
    common: &mut CommonObjAttr,
    shape_attr: &mut ShapeComponentAttr,
) {
    if shape_attr.current_width == 0 && shape_attr.original_width > 0 {
        shape_attr.current_width = shape_attr.original_width;
        if common.width == 0 {
            common.width = shape_attr.original_width;
        }
    }
    if shape_attr.current_height == 0 && shape_attr.original_height > 0 {
        shape_attr.current_height = shape_attr.original_height;
        if common.height == 0 {
            common.height = shape_attr.original_height;
        }
    }
}

/// HWP SHAPE_COMPONENT 저장 경로가 기대하는 storage 전용 필드를 materialize한다.
///
/// HWPX에는 같은 정보가 `flip`, `rotationInfo`, `imgRect` 같은 XML 자식 요소로
/// 분산되어 있다. 이 값을 SHAPE_COMPONENT 레코드 필드에 싣지 않으면 한컴은 그림/그룹
/// 개체 이후의 레코드 스트림을 정상적으로 이어 읽지 못하는 케이스가 있다.
fn materialize_shape_hwp_storage_defaults(
    common: &mut CommonObjAttr,
    shape_attr: &mut ShapeComponentAttr,
    kind: ShapeStorageKind,
) {
    materialize_shape_current_size_from_original(common, shape_attr);
    common.attr = pack_hwpx_common_obj_attr(common);

    if shape_attr.local_file_version == 0
        && (shape_attr.original_width > 0
            || shape_attr.original_height > 0
            || shape_attr.current_width > 0
            || shape_attr.current_height > 0
            || common.width > 0
            || common.height > 0)
    {
        shape_attr.local_file_version = 1;
    }

    if shape_attr.flip == 0 {
        let mut flip = match kind {
            ShapeStorageKind::Picture => 0x2400_0000,
            ShapeStorageKind::Group => 0x0009_0000,
            ShapeStorageKind::TextBoxDrawing => 0x0100_0000,
            ShapeStorageKind::Drawing => 0,
        };
        if shape_attr.horz_flip {
            flip |= 0x01;
        }
        if shape_attr.vert_flip {
            flip |= 0x02;
        }
        shape_attr.flip = flip;
    }

    if shape_attr.rotate_image {
        shape_attr.flip |= 0x0008_0000;
    }
}

/// `<hp:pic>`, `<hp:rect>`, `<hp:container>` 등 개체의 공통 속성을 요소 속성에서 파싱한다.
fn parse_object_element_attrs(
    e: &quick_xml::events::BytesStart,
    common: &mut CommonObjAttr,
    shape_attr: &mut ShapeComponentAttr,
) -> ObjectElementIds {
    common.hwp5_gen_shape_attr_bit26 = true;
    let mut ids = ObjectElementIds::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"id" => common.instance_id = parse_u32(&attr),
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
            b"textFlow" => {
                common.text_flow = match attr_str(&attr).as_str() {
                    "LEFT_ONLY" => crate::model::shape::TextFlow::LeftOnly,
                    "RIGHT_ONLY" => crate::model::shape::TextFlow::RightOnly,
                    "LARGEST_ONLY" => crate::model::shape::TextFlow::LargestOnly,
                    _ => crate::model::shape::TextFlow::BothSides,
                };
            }
            b"instid" => ids.instid = parse_u32(&attr),
            b"groupLevel" => shape_attr.group_level = attr_str(&attr).parse().unwrap_or(0),
            b"ratio" => ids.round_rate = parse_u8(&attr).min(100),
            _ => {}
        }
    }

    if common.instance_id == 0 && ids.instid != 0 {
        common.instance_id = ids.instid;
    }

    ids
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
                    b"width" => {
                        let v = parse_u32(&attr);
                        if v > 0 {
                            common.width = v;
                        }
                    }
                    b"height" => {
                        let v = parse_u32(&attr);
                        if v > 0 {
                            common.height = v;
                        }
                    }
                    b"widthRelTo" => {
                        common.width_criterion = parse_size_criterion(&attr_str(&attr), true);
                    }
                    b"heightRelTo" => {
                        common.height_criterion = parse_size_criterion(&attr_str(&attr), false);
                    }
                    b"protect" => common.size_protect = parse_bool(&attr),
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
                        if v > 0 {
                            common.width = v;
                        }
                    }
                    b"height" => {
                        let v = parse_u32(&attr);
                        shape_attr.current_height = v;
                        if v > 0 {
                            common.height = v;
                        }
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
                        if common.width == 0 {
                            common.width = v;
                        }
                    }
                    b"height" => {
                        let v = parse_u32(&attr);
                        shape_attr.original_height = v;
                        if common.height == 0 {
                            common.height = v;
                        }
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
                    b"flowWithText" => common.flow_with_text = parse_bool(&attr),
                    b"allowOverlap" => common.allow_overlap = parse_bool(&attr),
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
                    b"vertOffset" => common.vertical_offset = parse_i32_wrapping(&attr) as u32,
                    b"horzOffset" => common.horizontal_offset = parse_i32_wrapping(&attr) as u32,
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
        b"flip" => parse_shape_flip(ce, shape_attr),
        b"rotationInfo" => parse_shape_rotation_info(ce, shape_attr),
        _ => {}
    }
}

fn parse_shape_flip(e: &quick_xml::events::BytesStart, shape_attr: &mut ShapeComponentAttr) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"horizontal" => shape_attr.horz_flip = parse_bool(&attr),
            b"vertical" => shape_attr.vert_flip = parse_bool(&attr),
            _ => {}
        }
    }

    if shape_attr.flip != 0 {
        if shape_attr.horz_flip {
            shape_attr.flip |= 0x01;
        } else {
            shape_attr.flip &= !0x01;
        }
        if shape_attr.vert_flip {
            shape_attr.flip |= 0x02;
        } else {
            shape_attr.flip &= !0x02;
        }
    }
}

fn parse_shape_rotation_info(
    e: &quick_xml::events::BytesStart,
    shape_attr: &mut ShapeComponentAttr,
) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"angle" => shape_attr.rotation_angle = parse_i16(&attr),
            b"centerX" => shape_attr.rotation_center.x = parse_i32(&attr),
            b"centerY" => shape_attr.rotation_center.y = parse_i32(&attr),
            b"rotateimage" => shape_attr.rotate_image = parse_bool(&attr),
            _ => {}
        }
    }
}

fn parse_picture_img_rect(
    reader: &mut Reader<&[u8]>,
    border_x: &mut [i32; 4],
    border_y: &mut [i32; 4],
) -> Result<(), HwpxError> {
    let mut pts = [(0i32, 0i32); 4];
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let index = match local_name(ce.name().as_ref()) {
                    b"pt0" => Some(0),
                    b"pt1" => Some(1),
                    b"pt2" => Some(2),
                    b"pt3" => Some(3),
                    _ => None,
                };
                if let Some(index) = index {
                    for attr in ce.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"x" => pts[index].0 = parse_i32(&attr),
                            b"y" => pts[index].1 = parse_i32(&attr),
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(ref ee)) => {
                if local_name(ee.name().as_ref()) == b"imgRect" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("imgRect: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // HWP SHAPE_PICTURE 레코드는 HWPX 꼭짓점을 x/y 배열이 아니라 4개 스칼라씩
    // 앞뒤로 나누어 저장한다. 한컴 변환 정답지와 같은 순서로 materialize한다.
    *border_x = [pts[0].0, pts[0].1, pts[1].0, pts[1].1];
    *border_y = [pts[2].0, pts[2].1, pts[3].0, pts[3].1];

    Ok(())
}

/// `<hp:renderingInfo>` 파싱.
///
/// HWP5 SHAPE_COMPONENT는 rendering block을 `cnt + transMatrix + cnt개의
/// (scaMatrix, rotMatrix)` 형태로 저장한다. HWPX source에도 같은 matrix sequence가
/// 있으므로, 합성된 affine 값과 함께 HWP5 writer가 그대로 사용할 raw_rendering도 보존한다.
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
    fn hwp5_matrix_value(raw: f64) -> f64 {
        if raw.fract() == 0.0 {
            raw
        } else {
            f64::from(raw as f32)
        }
    }

    // 행렬 값 파싱 헬퍼
    fn read_matrix(ce: &quick_xml::events::BytesStart) -> [f64; 6] {
        let mut m = [0.0f64; 6];
        for attr in ce.attributes().flatten() {
            let val: f64 = attr_str(&attr)
                .parse()
                .map(hwp5_matrix_value)
                .unwrap_or(0.0);
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
            a[0] * b[0] + a[1] * b[3],        // a
            a[0] * b[1] + a[1] * b[4],        // b
            a[0] * b[2] + a[1] * b[5] + a[2], // tx
            a[3] * b[0] + a[4] * b[3],        // c
            a[3] * b[1] + a[4] * b[4],        // d
            a[3] * b[2] + a[4] * b[5] + a[5], // ty
        ]
    }
    fn push_matrix_le(out: &mut Vec<u8>, matrix: &[f64; 6]) {
        for value in matrix {
            out.extend_from_slice(&value.to_le_bytes());
        }
    }
    fn make_raw_rendering(trans: &[f64; 6], pairs: &[([f64; 6], [f64; 6])]) -> Vec<u8> {
        let mut raw = Vec::with_capacity(2 + 48 + pairs.len() * 96);
        raw.extend_from_slice(&(pairs.len() as u16).to_le_bytes());
        push_matrix_le(&mut raw, trans);
        for (sca, rot) in pairs {
            push_matrix_le(&mut raw, sca);
            push_matrix_le(&mut raw, rot);
        }
        raw
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
                        let sca = pending_sca.take().unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
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
    shape_attr.render_b = result[1]; // b (회전/전단)
    shape_attr.render_tx = result[2]; // tx
    shape_attr.render_c = result[3]; // c (회전/전단)
    shape_attr.render_sy = result[4]; // d
    shape_attr.render_ty = result[5]; // ty
    shape_attr.raw_rendering = make_raw_rendering(&trans, &sca_rot_pairs);

    Ok(())
}

/// `<hp:lineShape>` 요소에서 ShapeBorderLine을 파싱한다.
fn parse_line_shape_attr(e: &quick_xml::events::BytesStart) -> ShapeBorderLine {
    fn arrow_size(value: &str) -> Option<u32> {
        match value {
            "SMALL_SMALL" => Some(0),
            "SMALL_MEDIUM" => Some(1),
            "SMALL_BIG" | "SMALL_LARGE" => Some(2),
            "MEDIUM_SMALL" => Some(3),
            "MEDIUM_MEDIUM" => Some(4),
            "MEDIUM_BIG" | "MEDIUM_LARGE" => Some(5),
            "BIG_SMALL" | "LARGE_SMALL" => Some(6),
            "BIG_MEDIUM" | "LARGE_MEDIUM" => Some(7),
            "BIG_BIG" | "LARGE_LARGE" => Some(8),
            _ => None,
        }
    }

    let mut bl = ShapeBorderLine::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"color" => bl.color = parse_color(&attr),
            b"width" => bl.width = parse_i32(&attr),
            b"style" => {
                // 선 스타일 → attr 비트 플래그 (하위 바이트)
                let style_val: u32 = match attr_str(&attr).as_str() {
                    "NONE" => 0x40,
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
                bl.attr = (bl.attr & !0xFF) | style_val;
            }
            b"endCap" => {
                let end_cap: u32 = match attr_str(&attr).as_str() {
                    "ROUND" => 0,
                    "FLAT" => 1,
                    "SQUARE" => 2,
                    _ => 0,
                };
                bl.attr = (bl.attr & !(0x0F << 6)) | ((end_cap & 0x0F) << 6);
            }
            b"headfill" => {
                if parse_bool(&attr) {
                    bl.attr |= 0x8000_0000;
                } else {
                    bl.attr &= !0x8000_0000;
                }
            }
            b"tailfill" => {
                if parse_bool(&attr) {
                    bl.attr |= 0x4000_0000;
                } else {
                    bl.attr &= !0x4000_0000;
                }
            }
            b"headSz" => {
                if let Some(size) = arrow_size(&attr_str(&attr)) {
                    bl.attr = (bl.attr & !(0x0F << 22)) | ((size & 0x0F) << 22);
                }
            }
            b"tailSz" => {
                if let Some(size) = arrow_size(&attr_str(&attr)) {
                    bl.attr = (bl.attr & !(0x0F << 26)) | ((size & 0x0F) << 26);
                }
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
    use crate::model::style::{FillType, GradientFill, ImageFill, ImageFillMode, SolidFill};
    let mut fill = Fill::default();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref ce)) | Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"winBrush" => {
                        fill.fill_type = FillType::Solid;
                        let mut solid = SolidFill {
                            pattern_type: -1,
                            ..SolidFill::default()
                        };
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"faceColor" => solid.background_color = parse_color(&attr),
                                b"hatchColor" => solid.pattern_color = parse_color(&attr),
                                b"hatchStyle" => {
                                    if let Some(pattern_type) = parse_hatch_style(&attr_str(&attr))
                                    {
                                        solid.pattern_type = pattern_type;
                                    }
                                }
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
                                b"type" => {
                                    grad.gradient_type = parse_gradient_type(&attr_str(&attr))
                                }
                                b"angle" => grad.angle = parse_i16(&attr),
                                b"centerX" => grad.center_x = parse_i16(&attr),
                                b"centerY" => grad.center_y = parse_i16(&attr),
                                b"blur" | b"step" => grad.blur = parse_i16(&attr),
                                b"stepCenter" => grad.step_center = parse_u8(&attr),
                                b"alpha" => {
                                    let val = attr_str(&attr);
                                    if let Ok(f) = val.parse::<f64>() {
                                        fill.alpha = (f.clamp(0.0, 1.0) * 255.0) as u8;
                                    }
                                }
                                _ => {}
                            }
                        }
                        fill.gradient = Some(grad);
                    }
                    b"color" => {
                        // <hc:color value="#RRGGBB"/> -- shape gradation child.
                        // Header BorderFill already handles the same construct; shape-local
                        // fillBrush needs the same color stop materialization for rendering.
                        if let Some(ref mut grad) = fill.gradient {
                            for attr in ce.attributes().flatten() {
                                if attr.key.as_ref() == b"value" {
                                    grad.colors.push(parse_color(&attr));
                                }
                            }
                        }
                    }
                    b"imgBrush" => {
                        fill.fill_type = FillType::Image;
                        let mut img = ImageFill::default();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"mode" => {
                                    img.fill_mode = match attr_str(&attr).as_str() {
                                        "TILE" | "TILE_ALL" => ImageFillMode::TileAll,
                                        "FIT" | "FIT_TO_SIZE" | "STRETCH" | "TOTAL" => {
                                            ImageFillMode::FitToSize
                                        }
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
                if local_name(ee.name().as_ref()) == b"fillBrush" {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("fillBrush: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(fill)
}

fn parse_shape_shadow_attr(e: &quick_xml::events::BytesStart) -> (u32, u32, i32, i32, u8) {
    let mut shadow_type = 0_u32;
    let mut shadow_color = 0_u32;
    let mut shadow_offset_x = 0_i32;
    let mut shadow_offset_y = 0_i32;
    let mut shadow_alpha = 0_u8;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => {
                shadow_type = match attr_str(&attr).as_str() {
                    "NONE" => 0,
                    "LEFT_TOP" => 1,
                    "RIGHT_TOP" => 2,
                    "LEFT_BOTTOM" => 3,
                    "RIGHT_BOTTOM" => 4,
                    "CENTER" | "INSIDE" | "OUTSIDE" => 5,
                    _ => 0,
                };
            }
            b"color" => shadow_color = parse_color(&attr),
            b"offsetX" => shadow_offset_x = parse_i32(&attr),
            b"offsetY" => shadow_offset_y = parse_i32(&attr),
            b"alpha" => {
                let raw = attr_str(&attr);
                shadow_alpha = raw
                    .parse::<f64>()
                    .map(|value| {
                        if value <= 1.0 {
                            (value.clamp(0.0, 1.0) * 255.0) as u8
                        } else {
                            value.clamp(0.0, 255.0) as u8
                        }
                    })
                    .unwrap_or(0);
            }
            _ => {}
        }
    }

    (
        shadow_type,
        shadow_color,
        shadow_offset_x,
        shadow_offset_y,
        shadow_alpha,
    )
}

/// `<hp:drawText>` 내부의 `<hp:subList>` → `<hp:p>` 문단을 파싱한다.
fn parse_draw_text(reader: &mut Reader<&[u8]>, text_box: &mut TextBox) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"subList" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"vertAlign" => {
                                    let align_code = match attr_str(&attr).as_str() {
                                        "CENTER" => 1_u32,
                                        "BOTTOM" => 2_u32,
                                        _ => 0_u32,
                                    };
                                    text_box.vertical_align = match align_code {
                                        1 => VerticalAlign::Center,
                                        2 => VerticalAlign::Bottom,
                                        _ => VerticalAlign::Top,
                                    };
                                    text_box.list_attr =
                                        (text_box.list_attr & !(0b11 << 5)) | (align_code << 5);
                                }
                                // [Task #1028] HWPX 글상자 세로쓰기 (textDirection)
                                // 파싱. HWP5 LIST_HEADER 의 list_attr bit 0~2
                                // (text_direction) 영역에 set — renderer 의
                                // shape_layout.rs:1652 `(list_attr & 0x07)` 분기
                                // 가 세로쓰기 (`layout_vertical_textbox_text_with_paras`)
                                // 활성화. "VERTICAL"/"VERTICALALL" 모두 code 1.
                                b"textDirection" => {
                                    let direction_code: u32 = match attr_str(&attr).as_str() {
                                        "VERTICAL" | "VERTICALALL" => 1,
                                        _ => 0,
                                    };
                                    text_box.list_attr =
                                        (text_box.list_attr & !0b111) | direction_code;
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
    let mut shadow_acc: Option<(u32, u32, i32, i32, u8)> = None;
    let mut has_pos = false;
    let mut x_coords = [0i32; 4];
    let mut y_coords = [0i32; 4];
    // [Task #1067] polygon / curve 의 가변 꼭짓점 `<hc:pt x=... y=.../>` 누적.
    // 기존 pt0/pt1/pt2/pt3 (rect 의 4 꼭짓점) 와 별개.
    let mut polygon_points: Vec<crate::model::Point> = Vec::new();

    let object_ids = parse_object_element_attrs(e, &mut common, &mut shape_attr);

    let tag_name = String::from_utf8_lossy(shape_type).to_string();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"shapeComment" => {
                common.description = read_dutmal_text(reader, b"shapeComment")?;
            }
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"sz" | b"curSz" | b"orgSz" | b"pos" | b"offset" | b"outMargin" | b"flip"
                    | b"rotationInfo" => {
                        parse_object_layout_child(
                            local,
                            ce,
                            &mut common,
                            &mut shape_attr,
                            &mut has_pos,
                        );
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
                    // [Task #1067] polygon / curve 의 가변 꼭짓점 (<hc:pt x="..." y="..."/>).
                    // pt0/pt1/pt2/pt3 (rect 의 4 꼭짓점) 매칭 후 fall-through 로 본 분기 도달.
                    b"pt" => {
                        let mut px: i32 = 0;
                        let mut py: i32 = 0;
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => px = parse_i32(&attr),
                                b"y" => py = parse_i32(&attr),
                                _ => {}
                            }
                        }
                        polygon_points.push(crate::model::Point { x: px, y: py });
                    }
                    // [#1200] curve 의 가변 꼭짓점이 `<hp:seg x1 y1 x2 y2>` (점-대-점 chain)
                    // 으로 인코딩된 경우. `<hc:pt>` 미사용 curve 는 이 경로로 점을 채운다.
                    // seg 는 제어점이 아닌 sampled 꼭짓점이므로 폴리라인(LineTo)으로 재구성:
                    // 첫 seg 의 시작점 1회 + 각 seg 의 끝점.
                    b"seg" => {
                        let mut x1: i32 = 0;
                        let mut y1: i32 = 0;
                        let mut x2: i32 = 0;
                        let mut y2: i32 = 0;
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x1" => x1 = parse_i32(&attr),
                                b"y1" => y1 = parse_i32(&attr),
                                b"x2" => x2 = parse_i32(&attr),
                                b"y2" => y2 = parse_i32(&attr),
                                _ => {}
                            }
                        }
                        if polygon_points.is_empty() {
                            polygon_points.push(crate::model::Point { x: x1, y: y1 });
                        }
                        polygon_points.push(crate::model::Point { x: x2, y: y2 });
                    }
                    b"startPt" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x_coords[0] = parse_i32(&attr),
                                b"y" => y_coords[0] = parse_i32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"endPt" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x_coords[1] = parse_i32(&attr),
                                b"y" => y_coords[1] = parse_i32(&attr),
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
                        shadow_acc = Some(parse_shape_shadow_attr(ce));
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

    let storage_kind = if text_box.is_some() {
        ShapeStorageKind::TextBoxDrawing
    } else {
        ShapeStorageKind::Drawing
    };
    materialize_shape_hwp_storage_defaults(&mut common, &mut shape_attr, storage_kind);

    let (shadow_type, shadow_color, shadow_offset_x, shadow_offset_y, shadow_alpha) =
        shadow_acc.unwrap_or((0, 0, 0, 0, 0));

    let drawing = DrawingObjAttr {
        shape_attr,
        border_line,
        fill,
        shadow_type,
        shadow_color,
        shadow_offset_x,
        shadow_offset_y,
        shadow_alpha,
        inst_id: object_ids.instid,
        text_box,
        ..Default::default()
    };

    let shape = match shape_type {
        b"rect" => ShapeObject::Rectangle(RectangleShape {
            common,
            drawing,
            round_rate: object_ids.round_rate,
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
            start: crate::model::Point {
                x: x_coords[0],
                y: y_coords[0],
            },
            end: crate::model::Point {
                x: x_coords[1],
                y: y_coords[1],
            },
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
            // [Task #1067] HWPX `<hc:pt>` 점들을 PolygonShape::points 로 매핑.
            // 누락 시 polygon path 가 빈 상태로 렌더링되어 도형 미표시 (rhwp-studio + 한컴 둘 다).
            points: polygon_points,
            raw_trailing: Vec::new(),
        }),
        b"curve" => ShapeObject::Curve(CurveShape {
            common,
            drawing,
            // CurveShape 도 동일 패턴 — 누락 시 곡선 미표시. segment_types 는 별개로 추후 task.
            points: polygon_points,
            ..Default::default()
        }),
        _ => ShapeObject::Rectangle(RectangleShape {
            common,
            drawing,
            round_rate: object_ids.round_rate,
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"sz" | b"curSz" | b"orgSz" | b"pos" | b"offset" | b"outMargin" | b"flip"
                    | b"rotationInfo" => {
                        parse_object_layout_child(
                            local,
                            ce,
                            &mut common,
                            &mut shape_attr,
                            &mut has_pos,
                        );
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

    materialize_shape_hwp_storage_defaults(&mut common, &mut shape_attr, ShapeStorageKind::Group);

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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"colPr" => {
                        let cd = parse_col_pr_with_children(ce, reader)?;
                        controls.push(Control::ColumnDef(cd));
                        // [Task #901] ColumnDef 도 8 utf16 inline marker (HWP 정합).
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"header" => {
                        let ctrl = parse_ctrl_header(ce, reader)?;
                        controls.push(ctrl);
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"footer" => {
                        let ctrl = parse_ctrl_footer(ce, reader)?;
                        controls.push(ctrl);
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"footNote" => {
                        let ctrl = parse_ctrl_footnote(ce, reader)?;
                        controls.push(ctrl);
                        // [Task #1050] HWP 정합 — extended ctrl: 8 code unit (16 byte) 차지만
                        // text/char_offsets 에는 placeholder 미 push.
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"endNote" => {
                        let ctrl = parse_ctrl_endnote(ce, reader)?;
                        controls.push(ctrl);
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"autoNum" => {
                        let ctrl = parse_ctrl_autonum(ce, reader)?;
                        controls.push(ctrl);
                        // [Task #1050] AUTO_NUMBER (0x12) 는 HWP PARA_TEXT 에서:
                        //   char_offsets.push(pos) + text.push(' ') + pos += 8 (16 byte)
                        // 본 컨트롤은 placeholder space 1 char 점하고 jump 8 처리.
                        // \u{0012} 표시자 사용 — 후속 visual_text 조립 단계에서 처리.
                        text_parts.push("\u{0012}".to_string());
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
                        text_parts.push("\u{0002}".to_string());
                        skip_element(reader, b"pageHiding")?;
                    }
                    b"pageNum" => {
                        let pn = parse_page_num_attrs(ce);
                        controls.push(Control::PageNumberPos(pn));
                        text_parts.push("\u{0002}".to_string());
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
                        // HWPX newNum is an inline page-control marker in HWP5
                        // PARA_TEXT. It occupies 8 UTF-16 code units like
                        // pageHiding, but it must not synthesize a visible
                        // placeholder space; that behavior is only for autoNum.
                        text_parts.push("\u{0002}".to_string());
                        skip_element(reader, b"newNum")?;
                    }
                    _ => {
                        let tag = local.to_vec();
                        skip_element(reader, &tag)?;
                    }
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"colPr" => {
                        let cd = parse_col_pr(ce);
                        controls.push(Control::ColumnDef(cd));
                        // [Task #901] ColumnDef 도 8 utf16 inline marker (HWP 정합).
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"pageHiding" => {
                        let ph = parse_page_hiding_attrs(ce);
                        controls.push(Control::PageHide(ph));
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"pageNum" => {
                        let pn = parse_page_num_attrs(ce);
                        controls.push(Control::PageNumberPos(pn));
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"bookmark" => {
                        let bm = parse_bookmark_attrs(ce);
                        controls.push(Control::Bookmark(bm));
                    }
                    b"newNum" => {
                        let nn = parse_new_num_attrs(ce);
                        controls.push(Control::NewNumber(nn));
                        // See the Start branch above. Without this marker the
                        // following pageHiding/header controls drift behind the
                        // visible text when saved back to HWP5.
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"autoNum" => {
                        let an = parse_autonum_attrs(ce);
                        controls.push(Control::AutoNumber(an));
                        // [Task #1050] AUTO_NUMBER inline (Empty 분기): placeholder space.
                        text_parts.push("\u{0012}".to_string());
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
            b"num" => {
                an.number = parse_u16(&attr);
                an.assigned_number = an.number;
            }
            b"numType" => an.number_type = parse_num_type(&attr_str(&attr)),
            _ => {}
        }
    }
    an
}

fn parse_field_begin_attrs(e: &quick_xml::events::BytesStart) -> Field {
    let mut f = Field::default();
    let mut field_name: Option<String> = None;
    let mut id_attr: Option<u32> = None;
    let mut fieldid_attr: Option<u32> = None;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => f.field_type = parse_field_type(&attr_str(&attr)),
            b"name" => field_name = Some(attr_str(&attr)),
            // [Task #852 Stage 2.5] HWP5 직렬화에 필요한 필드 메타
            b"id" => {
                if let Ok(v) = attr_str(&attr).parse::<u32>() {
                    id_attr = Some(v);
                }
            }
            b"fieldid" => {
                if let Ok(v) = attr_str(&attr).parse::<u32>() {
                    // fieldid (instance ID) — 정답지의 CTRL_HEADER 끝에 저장
                    fieldid_attr = Some(v);
                }
            }
            b"editable" => {
                // properties bit 0 = editable in form
                if attr_str(&attr) == "1" {
                    f.properties |= 1;
                }
            }
            _ => {}
        }
    }
    f.field_id = if matches!(f.field_type, FieldType::Memo) {
        id_attr.or(fieldid_attr).unwrap_or(0)
    } else {
        fieldid_attr.or(id_attr).unwrap_or(0)
    };
    // [Task #852 Stage 2.5] field_type → ctrl_id 매핑.
    // 정답지 (samples/form-01.hwp) reverse engineering: ClickHere CTRL_HEADER 의 ctrl_id 가
    // "%clk" (FIELD_CLICKHERE). HWPX parser 가 이전엔 ctrl_id 미설정 → serializer 가
    // 0x00000000 작성 → 한컴이 무효 컨트롤로 인식 (JS 핸들러 reference 끊김).
    f.ctrl_id = match f.field_type {
        FieldType::Date => tags::FIELD_DATE,
        FieldType::DocDate => tags::FIELD_DOCDATE,
        FieldType::Path => tags::FIELD_PATH,
        FieldType::Bookmark => tags::FIELD_BOOKMARK,
        FieldType::MailMerge => tags::FIELD_MAILMERGE,
        FieldType::CrossRef => tags::FIELD_CROSSREF,
        FieldType::Formula => tags::FIELD_FORMULA,
        FieldType::ClickHere => tags::FIELD_CLICKHERE,
        FieldType::Summary => tags::FIELD_SUMMARY,
        FieldType::UserInfo => tags::FIELD_USERINFO,
        FieldType::Hyperlink => tags::FIELD_HYPERLINK,
        FieldType::Memo => tags::FIELD_MEMO,
        FieldType::PrivateInfoSecurity => tags::FIELD_PRIVATE_INFO,
        FieldType::TableOfContents => tags::FIELD_TOC,
        FieldType::Unknown => 0,
    };
    // ClickHere 의 extra_properties 정답지 관찰값: 0x09
    if matches!(f.field_type, FieldType::ClickHere) {
        f.extra_properties = 0x09;
    }
    // command 가 비어있으면 fieldBegin 의 name 사용 (CTRL_DATA name 으로도 활용)
    if f.command.is_empty() {
        if let Some(name) = field_name.as_ref() {
            f.ctrl_data_name = Some(name.clone());
        }
    } else if let Some(name) = field_name.as_ref() {
        f.ctrl_data_name = Some(name.clone());
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
        match attr.key.as_ref() {
            b"applyPageType" => {
                header.apply_to = parse_apply_page_type(&attr_str(&attr));
            }
            b"id" => {
                header
                    .raw_ctrl_extra
                    .extend_from_slice(&parse_u32(&attr).to_le_bytes());
            }
            _ => {}
        }
    }
    let sublist = parse_sublist_paragraphs_with_layout(reader, b"header")?;
    header.paragraphs = sublist.paragraphs;
    header.list_attr = sublist.list_attr;
    header.text_width = sublist.text_width;
    header.text_height = sublist.text_height;
    header.text_ref = sublist.text_ref;
    header.num_ref = sublist.num_ref;
    Ok(Control::Header(Box::new(header)))
}

/// `<hp:ctrl>` → `<footer applyPageType="..." id="...">` → subList → paragraphs
fn parse_ctrl_footer(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut footer = Footer::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"applyPageType" => {
                footer.apply_to = parse_apply_page_type(&attr_str(&attr));
            }
            b"id" => {
                footer
                    .raw_ctrl_extra
                    .extend_from_slice(&parse_u32(&attr).to_le_bytes());
            }
            _ => {}
        }
    }
    let sublist = parse_sublist_paragraphs_with_layout(reader, b"footer")?;
    footer.paragraphs = sublist.paragraphs;
    footer.list_attr = sublist.list_attr;
    footer.text_width = sublist.text_width;
    footer.text_height = sublist.text_height;
    footer.text_ref = sublist.text_ref;
    footer.num_ref = sublist.num_ref;
    Ok(Control::Footer(Box::new(footer)))
}

/// `<hp:ctrl>` → `<footNote number="..." suffixChar="..." instId="...">` → subList → paragraphs
fn parse_ctrl_footnote(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut note = Footnote::default();
    // [Task #1050] HWP5 CTRL_FOOTNOTE 한컴 default 매핑:
    // suffixChar → after_decoration_letter (default 0x29 ')')
    // instId → instance_id (UInt4)
    note.after_decoration_letter = 0x0029; // default ')'
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"number" => note.number = parse_u16(&attr),
            // [#1199] prefixChar(코드포인트 숫자) → before_decoration_letter
            // 누락 시 0 유지(접두 없음). 예: "47928" = 0xBB38 '문'
            b"prefixChar" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u16>()
                {
                    note.before_decoration_letter = v;
                }
            }
            b"suffixChar" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u16>()
                {
                    note.after_decoration_letter = v;
                }
            }
            b"instId" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u32>()
                {
                    note.instance_id = v;
                }
            }
            _ => {}
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
    // [Task #1050] Footnote 와 동일 매핑
    note.after_decoration_letter = 0x0029;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"number" => note.number = parse_u16(&attr),
            // [#1199] prefixChar(코드포인트 숫자) → before_decoration_letter
            // 누락 시 0 유지(접두 없음). 예: "47928" = 0xBB38 '문'
            b"prefixChar" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u16>()
                {
                    note.before_decoration_letter = v;
                }
            }
            b"suffixChar" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u16>()
                {
                    note.after_decoration_letter = v;
                }
            }
            b"instId" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u32>()
                {
                    note.instance_id = v;
                }
            }
            _ => {}
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"autoNumFormat" {
                    for attr in ce.attributes().flatten() {
                        match attr.key.as_ref() {
                            // autoNumFormat type 은 문자열 enum (DIGIT/CIRCLE_DIGIT/…).
                            // 과거 parse_u8 은 문자열을 0으로만 떨궈 DIGIT 외 형식을 잃었다.
                            // pageNum formatType 과 동일한 문자열→코드 매핑을 사용한다.
                            b"type" => {
                                an.format = match attr_str(&attr).as_str() {
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
fn parse_ctrl_hidden_comment(reader: &mut Reader<&[u8]>) -> Result<Control, HwpxError> {
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"parameters" {
                    parse_field_parameters(reader, &mut f)?;
                } else if local == b"subList" && f.field_type == FieldType::Memo {
                    f.memo_paragraphs = parse_sublist_paragraphs(reader, b"subList")?;
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
fn parse_field_parameters(reader: &mut Reader<&[u8]>, field: &mut Field) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    let mut in_command = false;
    let mut in_memo_number = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"stringParam" {
                    for attr in ce.attributes().flatten() {
                        if attr.key.as_ref() == b"name" && attr_str(&attr) == "Command" {
                            in_command = true;
                            field.command.clear();
                        }
                    }
                } else if local == b"integerParam" {
                    for attr in ce.attributes().flatten() {
                        if attr.key.as_ref() == b"name" && attr_str(&attr) == "Number" {
                            in_memo_number = true;
                        }
                    }
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"stringParam" {
                    for attr in ce.attributes().flatten() {
                        if attr.key.as_ref() == b"name" && attr_str(&attr) == "Command" {
                            field.command.clear();
                        }
                    }
                }
            }
            Ok(Event::Text(ref t)) => {
                if in_command {
                    field.command.push_str(&t.decode().unwrap_or_default());
                } else if in_memo_number {
                    if let Ok(value) = t.decode().unwrap_or_default().trim().parse::<u32>() {
                        field.memo_index = value;
                    }
                }
            }
            Ok(Event::GeneralRef(ref r)) => {
                if in_command {
                    field.command.push_str(&decode_xml_general_ref(r));
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                let local = local_name(eename.as_ref());
                if local == b"stringParam" {
                    in_command = false;
                } else if local == b"integerParam" {
                    in_memo_number = false;
                } else if local == b"parameters" {
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
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
            Err(e) => {
                return Err(HwpxError::XmlError(format!(
                    "{}: {}",
                    String::from_utf8_lossy(end_tag),
                    e
                )))
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(paragraphs)
}

#[derive(Default)]
struct HwpxSubListLayout {
    paragraphs: Vec<Paragraph>,
    list_attr: u32,
    text_width: u32,
    text_height: u32,
    text_ref: u8,
    num_ref: u8,
}

/// HWPX header/footer subList는 HWP5 LIST_HEADER의 layout 필드로 materialize해야 한다.
fn parse_sublist_paragraphs_with_layout(
    reader: &mut Reader<&[u8]>,
    end_tag: &[u8],
) -> Result<HwpxSubListLayout, HwpxError> {
    let mut layout = HwpxSubListLayout::default();
    let mut root_sub_list_seen = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"subList" if !root_sub_list_seen => {
                        parse_hwpx_sublist_layout_attrs(ce, &mut layout);
                        root_sub_list_seen = true;
                    }
                    b"p" => {
                        let (para, _) = parse_paragraph(ce, reader)?;
                        layout.paragraphs.push(para);
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"subList" && !root_sub_list_seen {
                    parse_hwpx_sublist_layout_attrs(ce, &mut layout);
                    root_sub_list_seen = true;
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(HwpxError::XmlError(format!(
                    "{}: {}",
                    String::from_utf8_lossy(end_tag),
                    e
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(layout)
}

fn parse_hwpx_sublist_layout_attrs(
    e: &quick_xml::events::BytesStart,
    layout: &mut HwpxSubListLayout,
) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"vertAlign" => {
                layout.list_attr |= match attr_str(&attr).as_str() {
                    "CENTER" => 1 << 21,
                    "BOTTOM" => 2 << 21,
                    _ => 0,
                };
            }
            b"textWidth" => layout.text_width = parse_u32(&attr),
            b"textHeight" => layout.text_height = parse_u32(&attr),
            b"hasTextRef" => layout.text_ref = parse_u8(&attr),
            b"hasNumRef" => layout.num_ref = parse_u8(&attr),
            _ => {}
        }
    }
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
            // 한컴 HWPX는 `composeText="장"`처럼 속성에 글자를 넣기도 한다.
            // 자식 element form(<composeText>장</composeText>)이 뒤에 나오면 그쪽이 덮어쓴다.
            b"composeText" => co.chars = attr_str(&attr).chars().collect(),
            _ => {}
        }
    }
    // 자식 요소 파싱 (composeText, charPr)
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"composeText" {
                    let text = read_compose_text(reader)?;
                    co.chars = text.chars().collect();
                } else {
                    let tag = local.to_vec();
                    skip_element(reader, &tag)?;
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
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
            Ok(Event::GeneralRef(ref r)) => {
                text.push_str(&decode_xml_general_ref(r));
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
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
            Ok(Event::GeneralRef(ref r)) => {
                text.push_str(&decode_xml_general_ref(r));
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                if local_name(eename.as_ref()) == end_tag {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(HwpxError::XmlError(format!(
                    "{}: {}",
                    String::from_utf8_lossy(end_tag),
                    e
                )))
            }
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
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"sz" | b"curSz" | b"orgSz" | b"pos" | b"offset" | b"outMargin" => {
                        parse_object_layout_child(
                            local,
                            ce,
                            &mut common,
                            &mut shape_attr,
                            &mut has_pos,
                        );
                    }
                    b"script" => {
                        in_script = true;
                    }
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
        unknown: 0,
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
    parts
        .iter()
        .map(|s| match s.as_str() {
            "\u{0002}" | "\u{0003}" | "\u{0004}" => 8,
            _ => s
                .chars()
                .map(|c| {
                    if c == '\t' {
                        8u32
                    } else if (c as u32) > 0xFFFF {
                        2
                    } else {
                        1
                    }
                })
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
    // [Task #852 Stage 2.4] HWP5 직렬화에 필요한 ComboBox/Edit/Button 속성 보존
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name" => form.name = attr_str(&attr),
            b"caption" => form.caption = attr_str(&attr),
            b"foreColor" => form.fore_color = parse_color(&attr),
            b"backColor" => form.back_color = parse_color(&attr),
            b"enabled" => form.enabled = parse_bool(&attr),
            b"value" => form.value = if attr_str(&attr) == "CHECKED" { 1 } else { 0 },
            b"selectedValue" => form.text = attr_str(&attr), // comboBox 선택값
            // ComboBox 전용 속성 (HWP5 ComboBoxSet 직렬화에 필요)
            b"listBoxRows" => {
                form.properties
                    .insert("ListBoxRows".to_string(), attr_str(&attr));
            }
            b"listBoxWidth" => {
                form.properties
                    .insert("ListBoxWidth".to_string(), attr_str(&attr));
            }
            b"editEnable" => {
                form.properties
                    .insert("EditEnable".to_string(), attr_str(&attr));
            }
            // 공통 속성 (HWP5 CommonSet 직렬화에 필요)
            b"groupName" => {
                form.properties
                    .insert("GroupName".to_string(), attr_str(&attr));
            }
            b"tabStop" => {
                form.properties
                    .insert("TabStop".to_string(), attr_str(&attr));
            }
            b"editable" => {
                form.properties
                    .insert("Editable".to_string(), attr_str(&attr));
            }
            b"tabOrder" => {
                form.properties
                    .insert("TabOrder".to_string(), attr_str(&attr));
            }
            b"borderTypeIDRef" => {
                form.properties
                    .insert("BorderType".to_string(), attr_str(&attr));
            }
            b"drawFrame" => {
                form.properties
                    .insert("DrawFrame".to_string(), attr_str(&attr));
            }
            b"printable" => {
                form.properties
                    .insert("Printable".to_string(), attr_str(&attr));
            }
            b"command" => {
                form.properties
                    .insert("Command".to_string(), attr_str(&attr));
            }
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
                                Ok(Event::GeneralRef(ref r)) => {
                                    form.text.push_str(&decode_xml_general_ref(r));
                                }
                                Ok(Event::End(_)) => break,
                                Ok(Event::Eof) => break,
                                _ => {}
                            }
                            tbuf.clear();
                        }
                    }
                    _ => {
                        skip_element(reader, local)?;
                    }
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
                                b"width" => form.width = parse_u32(&attr),
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
                    b"formCharPr" => {
                        // <hp:formCharPr charPrIDRef="0" followContext="0" autoSz="1" wordWrap="0"/>
                        // [Task #852 Stage 2.4] HWP5 CharShapeSet 직렬화에 필요한 속성 보존
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"charPrIDRef" => {
                                    form.properties
                                        .insert("CharShapeID".to_string(), attr_str(&attr));
                                }
                                b"followContext" => {
                                    form.properties
                                        .insert("FollowContext".to_string(), attr_str(&attr));
                                }
                                b"autoSz" => {
                                    form.properties
                                        .insert("AutoSize".to_string(), attr_str(&attr));
                                }
                                b"wordWrap" => {
                                    form.properties
                                        .insert("WordWrap".to_string(), attr_str(&attr));
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
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
            form.properties
                .insert(format!("listItem{}", i), item.clone());
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
                    b"case" => {
                        in_case = true;
                    }
                    b"default" => {
                        in_case = false;
                    }
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
    common.hwp5_gen_shape_attr_bit26 = true;
    let mut chart_num: u16 = 0;
    let mut numbering_type_picture = false;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"numberingType" => {
                numbering_type_picture = attr_str(&attr).eq_ignore_ascii_case("PICTURE");
            }
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
            b"textFlow" => {
                common.text_flow = match attr_str(&attr).as_str() {
                    "LEFT_ONLY" => crate::model::shape::TextFlow::LeftOnly,
                    "RIGHT_ONLY" => crate::model::shape::TextFlow::RightOnly,
                    "LARGEST_ONLY" => crate::model::shape::TextFlow::LargestOnly,
                    _ => crate::model::shape::TextFlow::BothSides,
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
    if numbering_type_picture {
        common.hwp5_gen_shape_attr_bit28 = true;
    }
    common.attr = pack_hwpx_common_obj_attr(&common);

    if chart_num == 0 {
        return Ok(None);
    }

    let mut ole = OleShape::default();
    ole.common = common;
    ole.bin_data_id = 60000u32 + chart_num as u32;
    ole.extent_x = 7200;
    ole.extent_y = 7200;
    apply_hwpx_ole_shape_component_contract(&mut ole);
    Ok(Some(Control::Shape(Box::new(ShapeObject::Ole(Box::new(
        ole,
    ))))))
}

/// `<hp:ole binaryItemIDRef="oleN" ...>` 내부를 OLE 모델로 변환 (fallback용)
fn parse_hp_ole_element(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Option<Control>, HwpxError> {
    use crate::model::shape::OleShape;

    let mut common = CommonObjAttr::default();
    common.hwp5_gen_shape_attr_bit26 = true;
    let mut bin_id: u32 = 0;
    let mut numbering_type_picture = false;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"numberingType" => {
                numbering_type_picture = attr_str(&attr).eq_ignore_ascii_case("PICTURE");
            }
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
            b"textFlow" => {
                common.text_flow = match attr_str(&attr).as_str() {
                    "LEFT_ONLY" => crate::model::shape::TextFlow::LeftOnly,
                    "RIGHT_ONLY" => crate::model::shape::TextFlow::RightOnly,
                    "LARGEST_ONLY" => crate::model::shape::TextFlow::LargestOnly,
                    _ => crate::model::shape::TextFlow::BothSides,
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
    if numbering_type_picture {
        common.hwp5_gen_shape_attr_bit28 = true;
    }
    common.attr = pack_hwpx_common_obj_attr(&common);

    let mut ole = OleShape::default();
    ole.common = common;
    ole.bin_data_id = bin_id;
    ole.extent_x = 7200;
    ole.extent_y = 7200;
    apply_hwpx_ole_shape_component_contract(&mut ole);
    Ok(Some(Control::Shape(Box::new(ShapeObject::Ole(Box::new(
        ole,
    ))))))
}

fn apply_hwpx_ole_shape_component_contract(ole: &mut crate::model::shape::OleShape) {
    let extent_w = if ole.extent_x > 0 {
        ole.extent_x as u32
    } else {
        7200
    };
    let extent_h = if ole.extent_y > 0 {
        ole.extent_y as u32
    } else {
        7200
    };
    let shape_attr = &mut ole.drawing.shape_attr;
    shape_attr.ctrl_id = tags::SHAPE_OLE_ID;
    shape_attr.is_two_ctrl_id = true;
    if shape_attr.local_file_version == 0 {
        shape_attr.local_file_version = 1;
    }
    if shape_attr.original_width == 0 {
        shape_attr.original_width = extent_w;
    }
    if shape_attr.original_height == 0 {
        shape_attr.original_height = extent_h;
    }
    if shape_attr.current_width == 0 {
        shape_attr.current_width = shape_attr.original_width;
    }
    if shape_attr.current_height == 0 {
        shape_attr.current_height = shape_attr.original_height;
    }
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
                                b"protect" => common.size_protect = parse_bool(&attr),
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
                                b"flowWithText" => common.flow_with_text = parse_bool(&attr),
                                b"allowOverlap" => common.allow_overlap = parse_bool(&attr),
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
    fn test_parse_text_preserves_xml_general_refs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>&lt; A &amp; B &gt; &quot;q&quot; &apos;s&apos; &#x25B3;</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert_eq!(section.paragraphs.len(), 1);
        assert_eq!(section.paragraphs[0].text, "< A & B > \"q\" 's' △");
    }

    #[test]
    fn test_parse_endnote_long_note_line_keeps_hwp5_low_word() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" outlineShapeIDRef="1" masterPageCnt="1">
        <hp:pagePr landscape="WIDELY" width="77102" height="111685" gutterType="LEFT_RIGHT">
          <hp:margin header="4960" footer="3401" gutter="0" left="5300" right="5300" top="6236" bottom="5952"/>
        </hp:pagePr>
        <hp:endNotePr>
          <hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>
          <hp:noteLine length="14692344" type="SOLID" width="0.12 mm" color="#000000"/>
          <hp:noteSpacing betweenNotes="0" belowLine="567" aboveLine="850"/>
          <hp:numbering type="CONTINUOUS" newNum="1"/>
          <hp:placement place="END_OF_DOCUMENT" beneathText="0"/>
        </hp:endNotePr>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();

        assert_eq!(section.section_def.endnote_shape.separator_length, 0x2ff8);
        assert_eq!(section.section_def.endnote_shape.separator_margin_top, 224);
        assert_eq!(
            section.section_def.endnote_shape.separator_margin_bottom,
            850
        );
        assert_eq!(section.section_def.endnote_shape.note_spacing, 567);
        assert_eq!(
            section.section_def.endnote_shape.separator_line_width, 1,
            "HWPX noteLine width도 공통 선 굵기 코드표를 사용해야 함"
        );
        assert_eq!(
            section.section_def.endnote_shape.placement,
            crate::model::footnote::FootnotePlacement::EachColumn
        );
    }

    #[test]
    fn test_parse_endnote_placement_end_of_section() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" outlineShapeIDRef="1" masterPageCnt="0">
        <hp:endNotePr>
          <hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>
          <hp:noteLine length="0" type="NONE" width="0.12 mm" color="#000000"/>
          <hp:noteSpacing betweenNotes="0" belowLine="567" aboveLine="850"/>
          <hp:numbering type="CONTINUOUS" newNum="1"/>
          <hp:placement place="END_OF_SECTION" beneathText="0"/>
        </hp:endNotePr>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();

        assert_eq!(
            section.section_def.endnote_shape.placement,
            crate::model::footnote::FootnotePlacement::BelowText
        );
        assert_eq!((section.section_def.endnote_shape.attr >> 8) & 0x03, 1);
    }

    #[test]
    fn test_parse_endnote_numbering_restart_section() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" outlineShapeIDRef="1" masterPageCnt="0">
        <hp:endNotePr>
          <hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>
          <hp:noteLine length="0" type="NONE" width="0.12 mm" color="#000000"/>
          <hp:noteSpacing betweenNotes="0" belowLine="567" aboveLine="850"/>
          <hp:numbering type="ON_SECTION" newNum="5"/>
        </hp:endNotePr>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();

        assert_eq!(
            section.section_def.endnote_shape.numbering,
            crate::model::footnote::FootnoteNumbering::RestartSection
        );
        assert_eq!(section.section_def.endnote_shape.start_number, 5);
        assert_eq!((section.section_def.endnote_shape.attr >> 8) & 0x03, 1);
    }

    /// [#1199] HWPX 미주/각주 ctrl 의 prefixChar(코드포인트 숫자) 가
    /// before_decoration_letter 로 매핑되어야 한다. 누락 시 마커 접두문자('문')가 탈락.
    #[test]
    fn test_parse_note_prefix_char_maps_to_before_decoration_letter() {
        // prefixChar="47928"(0xBB38 '문'), suffixChar="65289"(0xFF09 '）')
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ctrl>
        <hp:endNote number="1" prefixChar="47928" suffixChar="65289" instId="100">
          <hp:subList>
            <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>note body</hp:t></hp:run></hp:p>
          </hp:subList>
        </hp:endNote>
      </hp:ctrl>
      <hp:ctrl>
        <hp:footNote number="1" prefixChar="47928" suffixChar="65289" instId="200">
          <hp:subList>
            <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>note body</hp:t></hp:run></hp:p>
          </hp:subList>
        </hp:footNote>
      </hp:ctrl>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();
        let controls: Vec<&Control> = section
            .paragraphs
            .iter()
            .flat_map(|p| p.controls.iter())
            .collect();

        let endnote = controls
            .iter()
            .find_map(|c| match c {
                Control::Endnote(n) => Some(n),
                _ => None,
            })
            .expect("endnote ctrl");
        assert_eq!(
            endnote.before_decoration_letter, 47928,
            "endnote prefixChar"
        );
        assert_eq!(endnote.after_decoration_letter, 65289, "endnote suffixChar");

        let footnote = controls
            .iter()
            .find_map(|c| match c {
                Control::Footnote(n) => Some(n),
                _ => None,
            })
            .expect("footnote ctrl");
        assert_eq!(
            footnote.before_decoration_letter, 47928,
            "footnote prefixChar"
        );
        assert_eq!(
            footnote.after_decoration_letter, 65289,
            "footnote suffixChar"
        );
    }

    /// [#1199] prefixChar 속성이 없으면 before_decoration_letter 는 0(접두 없음) 유지 — 회귀 방지.
    #[test]
    fn test_parse_note_without_prefix_char_keeps_zero_before_letter() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ctrl>
        <hp:endNote number="1" suffixChar="41" instId="100">
          <hp:subList>
            <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run></hp:p>
          </hp:subList>
        </hp:endNote>
      </hp:ctrl>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();
        let endnote = section
            .paragraphs
            .iter()
            .flat_map(|p| p.controls.iter())
            .find_map(|c| match c {
                Control::Endnote(n) => Some(n),
                _ => None,
            })
            .expect("endnote ctrl");
        assert_eq!(endnote.before_decoration_letter, 0);
        assert_eq!(endnote.after_decoration_letter, 41); // ')'
    }

    /// [#1200] curve 도형의 geometry 가 `<hp:seg x1 y1 x2 y2>` (점-대-점 chain)
    /// 으로 인코딩된 경우 CurveShape.points 가 채워져야 한다. 누락 시 외곽선 미렌더.
    #[test]
    fn test_parse_curve_seg_populates_points() {
        // seg chain: (10,10)->(90,10)->(90,90)->(10,10) (폐곡선)
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:curve id="0" zOrder="0" numberingType="NONE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" href="" groupLevel="0" instid="1">
        <hp:offset x="0" y="0"/>
        <hp:orgSz width="100" height="100"/>
        <hp:curSz width="100" height="100"/>
        <hp:lineShape color="#000000" width="113" style="SOLID"/>
        <hp:seg type="LINE" x1="10" y1="10" x2="90" y2="10"/>
        <hp:seg type="LINE" x1="90" y1="10" x2="90" y2="90"/>
        <hp:seg type="LINE" x1="90" y1="90" x2="10" y2="10"/>
      </hp:curve>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();
        let curve = section
            .paragraphs
            .iter()
            .flat_map(|p| p.controls.iter())
            .find_map(|c| match c {
                Control::Shape(s) => match s.as_ref() {
                    crate::model::shape::ShapeObject::Curve(cv) => Some(cv),
                    _ => None,
                },
                _ => None,
            })
            .expect("curve shape");

        // 첫 seg 시작점 + 각 seg 끝점 = 4점 chain
        let pts: Vec<(i32, i32)> = curve.points.iter().map(|p| (p.x, p.y)).collect();
        assert_eq!(pts, vec![(10, 10), (90, 10), (90, 90), (10, 10)]);
    }

    #[test]
    fn test_parse_page_pr_gutter_type_materializes_hwp5_binding_attr() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL">
        <hp:pagePr landscape="WIDELY" width="77102" height="111685" gutterType="LEFT_RIGHT">
          <hp:margin header="4960" footer="3401" gutter="0" left="5300" right="5300" top="6236" bottom="5952"/>
        </hp:pagePr>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();

        assert_eq!(
            section.section_def.page_def.binding,
            BindingMethod::DuplexSided
        );
        assert_eq!(section.section_def.page_def.attr & (0x03 << 1), 0x02);
    }

    #[test]
    fn test_parse_page_border_fill_basis_from_text_border() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL">
        <hp:pageBorderFill type="BOTH" borderFillIDRef="1" textBorder="CONTENT" fillArea="PAPER">
          <hp:offset left="1417" right="1417" top="1417" bottom="1417"/>
        </hp:pageBorderFill>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert_eq!(section.section_def.page_border_fill.attr & 0x01, 0);
        assert_eq!(
            section.section_def.page_border_fill.basis,
            PageBorderBasis::PaperBased
        );
        assert_eq!(
            section.section_def.page_border_fill.ui_basis,
            PageBorderUiBasis::Paper
        );

        let xml = xml.replace(r#"textBorder="CONTENT""#, r#"textBorder="PAPER""#);
        let section = parse_hwpx_section(&xml).unwrap();
        assert_eq!(section.section_def.page_border_fill.attr & 0x01, 0x01);
        assert_eq!(
            section.section_def.page_border_fill.basis,
            PageBorderBasis::BodyBased
        );
        assert_eq!(
            section.section_def.page_border_fill.ui_basis,
            PageBorderUiBasis::Page
        );
    }

    #[test]
    fn test_parse_section_grid_preserves_line_and_char_grid() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL">
        <hp:grid lineGrid="1200" charGrid="900" wonggojiFormat="0"/>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();

        assert_eq!(section.section_def.line_grid, 1200);
        assert_eq!(section.section_def.char_grid, 900);
    }

    #[test]
    fn test_parse_section_col_pr_break_type_without_page_break() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL">
        <hp:colPr type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="1" sameGap="1134"/>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.raw_break_type, 0x03);
        assert_eq!(
            para.column_type,
            crate::model::paragraph::ColumnBreakType::Section
        );
    }

    #[test]
    fn test_parse_section_col_pr_break_type_with_page_break() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0" pageBreak="1" columnBreak="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL">
        <hp:colPr type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="1" sameGap="1134"/>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.raw_break_type, 0x07);
        assert_eq!(
            para.column_type,
            crate::model::paragraph::ColumnBreakType::Page
        );
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
    fn test_parse_hwpx_tab_extension_uses_hwp5_inline_format() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>A<hp:tab width="17283" leader="3" type="2"/>(페이지 표기)</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.text, "A\t(페이지 표기)");
        assert_eq!(para.tab_extended, vec![[17283, 0, 0x0203, 0, 0, 0, 9]]);
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
        // [Task #1058] 같은 char_shape_id 연속 dedup — HWP PARA_CHAR_SHAPE 는 첫 entry 1개만 유지.
        // 두 run 모두 charPrIDRef="0" 이므로 dedup 후 char_shapes.len() = 1.
        assert_eq!(para.char_shapes[0].start_pos, 0);
        assert_eq!(para.char_shapes.len(), 1);
        assert_eq!(para.controls.len(), 1);
    }

    #[test]
    fn test_parse_table_cell_has_margin() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:tbl rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="0">
      <hp:inMargin left="0" right="0" top="0" bottom="0"/>
      <hp:tr>
        <hp:tc name="" header="0" hasMargin="1" borderFillIDRef="0">
          <hp:subList><hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>T</hp:t></hp:run></hp:p></hp:subList>
          <hp:cellAddr colAddr="0" rowAddr="0"/>
          <hp:cellSpan colSpan="1" rowSpan="1"/>
          <hp:cellSz width="1000" height="1000"/>
          <hp:cellMargin left="141" right="141" top="113" bottom="113"/>
        </hp:tc>
      </hp:tr>
    </hp:tbl>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let table = match &section.paragraphs[0].controls[0] {
            crate::model::control::Control::Table(table) => table,
            other => panic!("expected table, got {:?}", other),
        };
        assert!(table.cells[0].apply_inner_margin);
        assert_eq!(table.cells[0].padding.left, 141);
        assert_eq!(table.cells[0].padding.top, 113);
    }

    #[test]
    fn test_parse_table_page_break_table_vs_cell_mapping() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:tbl rowCnt="1" colCnt="1" pageBreak="TABLE" repeatHeader="1" cellSpacing="0" borderFillIDRef="0">
      <hp:tr><hp:tc borderFillIDRef="0"><hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="1000" height="1000"/></hp:tc></hp:tr>
    </hp:tbl>
    <hp:tbl rowCnt="1" colCnt="1" pageBreak="CELL" repeatHeader="1" cellSpacing="0" borderFillIDRef="0">
      <hp:tr><hp:tc borderFillIDRef="0"><hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="1000" height="1000"/></hp:tc></hp:tr>
    </hp:tbl>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let tables: Vec<_> = section.paragraphs[0]
            .controls
            .iter()
            .filter_map(|control| match control {
                crate::model::control::Control::Table(table) => Some(table),
                _ => None,
            })
            .collect();

        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0].page_break, TablePageBreak::CellBreak);
        assert_eq!(tables[1].page_break, TablePageBreak::RowBreak);
    }

    #[test]
    fn test_parse_hwpx_table_materializes_hwp_common_attrs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:tbl numberingType="TABLE" textWrap="TOP_AND_BOTTOM" pageBreak="CELL"
            repeatHeader="1" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="0"
            noAdjust="1">
      <hp:sz width="30613" widthRelTo="ABSOLUTE" height="8580" heightRelTo="ABSOLUTE"/>
      <hp:pos treatAsChar="1" flowWithText="1" allowOverlap="0"
              vertRelTo="PARA" horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT"
              vertOffset="0" horzOffset="0"/>
      <hp:outMargin left="141" right="141" top="141" bottom="141"/>
      <hp:inMargin left="0" right="0" top="283" bottom="283"/>
      <hp:tr>
        <hp:tc borderFillIDRef="0">
          <hp:cellAddr colAddr="0" rowAddr="0"/>
          <hp:cellSpan colSpan="1" rowSpan="1"/>
          <hp:cellSz width="30613" height="8580"/>
        </hp:tc>
      </hp:tr>
    </hp:tbl>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let table = match &section.paragraphs[0].controls[0] {
            crate::model::control::Control::Table(table) => table,
            other => panic!("expected table, got {:?}", other),
        };

        assert!(table.common.treat_as_char);
        assert_eq!(table.common.text_wrap, TextWrap::TopAndBottom);
        assert_eq!(table.common.attr, 0x082a_2211);
        assert_eq!(table.attr, 0x01);
        assert_eq!(table.raw_table_record_attr, 0x0400_000e);
    }

    #[test]
    fn test_parse_hwpx_masterpage_line_materializes_shape_common_attr() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<masterPage xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core"
            id="masterpage0" type="BOTH" pageNumber="0">
  <hp:subList textWidth="66502" textHeight="91136">
    <hp:p paraPrIDRef="0" styleIDRef="0">
      <hp:run charPrIDRef="0">
        <hp:line id="1" zOrder="0" textWrap="BEHIND_TEXT" instid="2">
          <hp:offset x="0" y="0"/>
          <hp:orgSz width="100" height="100"/>
          <hp:curSz width="1" height="92409"/>
          <hp:rotationInfo angle="0" centerX="0" centerY="46204" rotateimage="1"/>
          <hp:lineShape color="#000000" width="113" style="SOLID"
                        endCap="FLAT" headfill="1" tailfill="1"
                        headSz="MEDIUM_MEDIUM" tailSz="MEDIUM_MEDIUM"
                        outlineStyle="NORMAL"/>
          <hp:sz width="1" widthRelTo="ABSOLUTE" height="92409" heightRelTo="ABSOLUTE"/>
          <hp:pos treatAsChar="0" flowWithText="0" allowOverlap="1"
                  vertRelTo="PAPER" horzRelTo="PARA" vertAlign="TOP" horzAlign="CENTER"
                  vertOffset="9912" horzOffset="0"/>
          <hc:startPt x="0" y="0"/>
          <hc:endPt x="100" y="100"/>
        </hp:line>
      </hp:run>
    </hp:p>
  </hp:subList>
</masterPage>"##;

        let master_page = parse_hwpx_master_page(xml).unwrap();
        assert_eq!(master_page.hwpx_page_number, Some(0));
        let line = match &master_page.paragraphs[0].controls[0] {
            crate::model::control::Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Line(line) => line,
                other => panic!("expected line shape, got {:?}", other),
            },
            other => panic!("expected shape control, got {:?}", other),
        };

        assert_eq!(line.common.attr, 0x044a_4700);
        assert_eq!(line.common.text_wrap, TextWrap::BehindText);
        assert_eq!(line.common.width_criterion, SizeCriterion::Absolute);
        assert_eq!(line.common.height_criterion, SizeCriterion::Absolute);
        assert_eq!(line.drawing.border_line.color, 0x000000);
        assert_eq!(line.drawing.border_line.width, 113);
        assert_eq!(line.drawing.border_line.attr, 0xd100_0041);
        assert_eq!(line.drawing.border_line.outline_style, 0);
        assert_eq!(line.start.x, 0);
        assert_eq!(line.start.y, 0);
        assert_eq!(line.end.x, 100);
        assert_eq!(line.end.y, 100);
    }

    #[test]
    fn test_parse_field_begin_end_materializes_field_range() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ctrl>
        <hp:fieldBegin type="MEMO" id="2135782115" fieldid="623209829"/>
      </hp:ctrl>
      <hp:t>ABC</hp:t>
      <hp:ctrl>
        <hp:fieldEnd beginIDRef="2135782115" fieldid="623209829"/>
      </hp:ctrl>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];

        assert_eq!(para.text, "ABC");
        assert_eq!(para.char_offsets, vec![8, 9, 10]);
        assert_eq!(para.char_count, 20);
        assert_eq!(para.controls.len(), 1);
        assert_eq!(para.field_ranges.len(), 1);

        let range = &para.field_ranges[0];
        assert_eq!(range.start_char_idx, 0);
        assert_eq!(range.end_char_idx, 3);
        assert_eq!(range.control_idx, 0);
    }

    #[test]
    fn test_rendering_info_materializes_hwp5_raw_rendering_count() {
        let xml = r#"<hp:renderingInfo xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
          <hc:transMatrix e1="1" e2="0" e3="10" e4="0" e5="1" e6="20"/>
          <hc:scaMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>
          <hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>
          <hc:scaMatrix e1="2" e2="0" e3="0" e4="0" e5="3" e6="0"/>
          <hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>
        </hp:renderingInfo>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut shape_attr = ShapeComponentAttr::default();

        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if local_name(e.name().as_ref()) == b"renderingInfo" => {
                    parse_rendering_info(&mut reader, &mut shape_attr).unwrap();
                    break;
                }
                Event::Eof => panic!("renderingInfo not found"),
                _ => {}
            }
            buf.clear();
        }

        fn read_f64(raw: &[u8], offset: usize) -> f64 {
            f64::from_le_bytes(raw[offset..offset + 8].try_into().unwrap())
        }

        assert_eq!(shape_attr.raw_rendering.len(), 2 + 48 + 2 * 96);
        assert_eq!(
            u16::from_le_bytes([shape_attr.raw_rendering[0], shape_attr.raw_rendering[1],]),
            2
        );
        assert_eq!(read_f64(&shape_attr.raw_rendering, 2 + 16), 10.0);
        assert_eq!(read_f64(&shape_attr.raw_rendering, 2 + 40), 20.0);
        assert_eq!(read_f64(&shape_attr.raw_rendering, 2 + 48 + 96), 2.0);
        assert_eq!(read_f64(&shape_attr.raw_rendering, 2 + 48 + 96 + 32), 3.0);
    }

    #[test]
    fn test_rendering_info_quantizes_fractional_matrix_values_like_hwp5() {
        let xml = r#"<hp:renderingInfo xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
          <hc:transMatrix e1="1" e2="0" e3="-310" e4="0" e5="1" e6="0"/>
          <hc:scaMatrix e1="0.723629" e2="0" e3="310" e4="0" e5="0.723636" e6="0"/>
          <hc:rotMatrix e1="1" e2="0" e3="0" e4="0" e5="1" e6="0"/>
        </hp:renderingInfo>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut shape_attr = ShapeComponentAttr::default();

        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if local_name(e.name().as_ref()) == b"renderingInfo" => {
                    parse_rendering_info(&mut reader, &mut shape_attr).unwrap();
                    break;
                }
                Event::Eof => panic!("renderingInfo not found"),
                _ => {}
            }
            buf.clear();
        }

        fn read_f64(raw: &[u8], offset: usize) -> f64 {
            f64::from_le_bytes(raw[offset..offset + 8].try_into().unwrap())
        }

        let scale_start = 2 + 48;
        assert_eq!(
            read_f64(&shape_attr.raw_rendering, scale_start),
            f64::from(0.723629f32)
        );
        assert_eq!(
            read_f64(&shape_attr.raw_rendering, scale_start + 32),
            f64::from(0.723636f32)
        );
        assert_eq!(read_f64(&shape_attr.raw_rendering, scale_start + 16), 310.0);
    }

    #[test]
    fn test_parse_memo_field_parameters_preserves_number_as_memo_index() {
        let xml = r#"<hp:parameters xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph">
  <hp:stringParam name="Command">MEMO/65535/2/1650281184/31247371/user/\;;</hp:stringParam>
  <hp:integerParam name="Number">2</hp:integerParam>
</hp:parameters>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut field = Field {
            field_type: FieldType::Memo,
            ..Default::default()
        };

        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if local_name(e.name().as_ref()) == b"parameters" => {
                    parse_field_parameters(&mut reader, &mut field).unwrap();
                    break;
                }
                Event::Eof => panic!("parameters not found"),
                _ => {}
            }
            buf.clear();
        }

        assert_eq!(field.command, "MEMO/65535/2/1650281184/31247371/user/\\;;");
        assert_eq!(field.memo_index, 2);
    }

    #[test]
    fn test_parse_memo_field_begin_uses_id_as_hwp5_field_id() {
        let xml = r#"<hp:fieldBegin xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" type="MEMO" id="2135782115" fieldid="623209829" />"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Empty(ref e) | Event::Start(ref e)
                    if local_name(e.name().as_ref()) == b"fieldBegin" =>
                {
                    let field = parse_field_begin_attrs(e);
                    assert_eq!(field.field_type, FieldType::Memo);
                    assert_eq!(field.field_id, 2_135_782_115);
                    assert_eq!(field.ctrl_id, tags::FIELD_MEMO);
                    break;
                }
                Event::Eof => panic!("fieldBegin not found"),
                _ => {}
            }
            buf.clear();
        }
    }

    #[test]
    fn test_collect_hwpx_section_master_page_refs() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:masterPage idRef="masterpage0"/>
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:t>body</hp:t></hp:run>
  </hp:p>
  <masterPage idRef="masterpage1"/>
</hs:sec>"#;

        let refs = collect_hwpx_section_master_page_refs(xml).unwrap();
        assert_eq!(refs, vec!["masterpage0", "masterpage1"]);
    }

    #[test]
    fn test_collect_hwpx_section_master_page_refs_ignores_root_masterpage_without_id_ref() {
        let xml = r#"<masterPage xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            id="masterpage0" type="EVEN">
  <hp:subList textWidth="1000" textHeight="2000"/>
</masterPage>"#;

        let refs = collect_hwpx_section_master_page_refs(xml).unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_parse_hwpx_master_page_type_accepts_official_and_sample_spellings() {
        assert_eq!(
            parse_hwpx_master_page_type("BOTH"),
            HwpxMasterPageType::Both
        );
        assert_eq!(
            parse_hwpx_master_page_type("Both"),
            HwpxMasterPageType::Both
        );
        assert_eq!(
            parse_hwpx_master_page_type("both"),
            HwpxMasterPageType::Both
        );
        assert_eq!(
            parse_hwpx_master_page_type("EVEN"),
            HwpxMasterPageType::Even
        );
        assert_eq!(
            parse_hwpx_master_page_type("Even"),
            HwpxMasterPageType::Even
        );
        assert_eq!(
            parse_hwpx_master_page_type("even"),
            HwpxMasterPageType::Even
        );
        assert_eq!(parse_hwpx_master_page_type("ODD"), HwpxMasterPageType::Odd);
        assert_eq!(parse_hwpx_master_page_type("Odd"), HwpxMasterPageType::Odd);
        assert_eq!(parse_hwpx_master_page_type("odd"), HwpxMasterPageType::Odd);
        assert_eq!(
            parse_hwpx_master_page_type("LAST_PAGE"),
            HwpxMasterPageType::LastPage
        );
        assert_eq!(
            parse_hwpx_master_page_type("LastPage"),
            HwpxMasterPageType::LastPage
        );
        assert_eq!(
            parse_hwpx_master_page_type("lastPage"),
            HwpxMasterPageType::LastPage
        );
        assert_eq!(
            parse_hwpx_master_page_type("OPTIONAL_PAGE"),
            HwpxMasterPageType::OptionalPage
        );
        assert_eq!(
            parse_hwpx_master_page_type("OptionalPage"),
            HwpxMasterPageType::OptionalPage
        );
        assert_eq!(
            parse_hwpx_master_page_type("optionalPage"),
            HwpxMasterPageType::OptionalPage
        );
    }

    #[test]
    fn test_parse_master_page_mixed_case_type_attrs() {
        fn parse_type(type_value: &str) -> MasterPage {
            let xml = format!(
                r#"<masterPage xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            type="{type_value}" pageNumber="4" pageDuplicate="0">
  <hp:subList textWidth="1000" textHeight="2000" hasTextRef="0" hasNumRef="0"/>
</masterPage>"#
            );
            parse_hwpx_master_page(&xml).unwrap()
        }

        let both = parse_type("Both");
        assert_eq!(both.apply_to, HeaderFooterApply::Both);
        assert!(!both.is_extension);

        let even = parse_type("Even");
        assert_eq!(even.apply_to, HeaderFooterApply::Even);
        assert!(!even.is_extension);

        let odd = parse_type("odd");
        assert_eq!(odd.apply_to, HeaderFooterApply::Odd);
        assert!(!odd.is_extension);

        let last_page = parse_type("LastPage");
        assert_eq!(last_page.apply_to, HeaderFooterApply::Both);
        assert!(last_page.is_extension);
        assert!(last_page.overlap);
        assert!(last_page.replace_base);
        assert_eq!(last_page.ext_flags, 0x0003);

        let optional_page = parse_type("optionalPage");
        assert_eq!(optional_page.apply_to, HeaderFooterApply::Both);
        assert!(optional_page.is_extension);
        assert!(optional_page.overlap);
        assert!(!optional_page.replace_base);
        assert_eq!(optional_page.ext_flags, 0x0007);
    }

    #[test]
    fn test_parse_master_page_last_page_extension() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<masterPage xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            type="LAST_PAGE" pageDuplicate="0">
  <hp:subList textWidth="1000" textHeight="2000" hasTextRef="1" hasNumRef="0">
    <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
      <hp:run charPrIDRef="0">
        <hp:t>last page</hp:t>
      </hp:run>
    </hp:p>
  </hp:subList>
</masterPage>"#;

        let master_page = parse_hwpx_master_page(xml).unwrap();
        assert_eq!(master_page.apply_to, HeaderFooterApply::Both);
        assert!(master_page.is_extension);
        assert!(master_page.overlap);
        assert!(master_page.replace_base);
        assert_eq!(master_page.ext_flags, 0x0003);
        assert_eq!(master_page.text_width, 1000);
        assert_eq!(master_page.text_height, 2000);
        assert_eq!(master_page.text_ref, 1);
        assert_eq!(master_page.paragraphs.len(), 1);
        assert_eq!(master_page.paragraphs[0].text, "last page");
        assert_eq!(master_page.raw_list_header.len(), 34);
    }

    #[test]
    fn test_parse_master_page_optional_page_extension() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<masterPage xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            type="OPTIONAL_PAGE" pageNumber="4" pageDuplicate="0">
  <hp:subList textWidth="1000" textHeight="2000" hasTextRef="0" hasNumRef="0">
    <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
      <hp:run charPrIDRef="0">
        <hp:t>optional page</hp:t>
      </hp:run>
    </hp:p>
  </hp:subList>
</masterPage>"#;

        let master_page = parse_hwpx_master_page(xml).unwrap();
        assert_eq!(master_page.apply_to, HeaderFooterApply::Both);
        assert!(master_page.is_extension);
        assert!(master_page.overlap);
        assert!(!master_page.replace_base);
        assert_eq!(master_page.ext_flags, 0x0007);
        assert_eq!(master_page.hwpx_page_number, Some(4));
        assert_eq!(master_page.raw_list_header.len(), 34);
    }

    #[test]
    fn test_parse_rect_ratio_as_round_rate() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:rect id="1" zOrder="0" ratio="50">
        <hp:sz width="100" height="50"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
      </hp:rect>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Rectangle(rect) = shape.as_ref() else {
            panic!("expected rectangle shape");
        };
        assert_eq!(rect.round_rate, 50);
    }

    #[test]
    fn test_parse_rect_preserves_size_protect() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:rect id="1" zOrder="0" textWrap="SQUARE" textFlow="RIGHT_ONLY">
        <hp:drawText>
          <hp:subList vertAlign="CENTER">
            <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>기</hp:t></hp:run></hp:p>
          </hp:subList>
        </hp:drawText>
        <hp:sz width="2600" height="2600" protect="1"/>
        <hp:pos treatAsChar="0" flowWithText="1" allowOverlap="1" vertRelTo="PARA" horzRelTo="PARA"/>
      </hp:rect>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Rectangle(rect) = shape.as_ref() else {
            panic!("expected rectangle shape");
        };
        assert!(rect.common.size_protect);
        assert!(rect.common.flow_with_text);
        assert!(rect.common.allow_overlap);
        assert_eq!(
            rect.common.text_flow,
            crate::model::shape::TextFlow::RightOnly
        );
    }

    #[test]
    fn test_task1124_col_pr_parses_col_line() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ctrl>
        <hp:colPr type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="1" sameGap="850">
          <hp:colLine type="SOLID" width="0.12 mm" color="#000000"/>
        </hp:colPr>
      </hp:ctrl>
      <hp:t>A</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.text, "A");
        assert_eq!(para.controls.len(), 1);
        let Control::ColumnDef(cd) = &para.controls[0] else {
            panic!("expected ColumnDef control");
        };
        assert_eq!(cd.column_count, 2);
        assert!(cd.same_width);
        assert_eq!(cd.spacing, 850);
        assert_eq!(cd.separator_type, 1);
        assert_eq!(cd.separator_width, 1);
        assert_eq!(cd.separator_color, 0x00000000);
    }

    #[test]
    fn test_task1124_col_line_type_and_width_mapping() {
        assert_eq!(parse_hwpx_line_type("NONE"), 0);
        assert_eq!(parse_hwpx_line_type("SOLID"), 1);
        assert_eq!(parse_hwpx_line_type("DASH"), 2);
        assert_eq!(parse_hwpx_line_type("DOT"), 3);
        assert_eq!(parse_hwpx_line_type("DASH_DOT"), 4);
        assert_eq!(parse_hwpx_line_type("DASH_DOT_DOT"), 5);
        assert_eq!(parse_hwpx_line_type("LONG_DASH"), 6);
        assert_eq!(parse_hwpx_line_type("CIRCLE"), 7);

        assert_eq!(parse_hwpx_line_width("0.1 mm"), 0);
        assert_eq!(parse_hwpx_line_width("0.12 mm"), 1);
        assert_eq!(parse_hwpx_line_width("0.4 mm"), 6);
        assert_eq!(parse_hwpx_line_width("0.7 mm"), 9);
        assert_eq!(parse_hwpx_line_width("5.0 mm"), 15);
    }

    #[test]
    fn test_parse_empty_section() {
        let xml = r#"<?xml version="1.0"?><hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"/>"#;
        let section = parse_hwpx_section(xml).unwrap();
        assert!(section.paragraphs.is_empty());
    }
}
