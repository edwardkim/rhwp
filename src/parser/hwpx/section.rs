//! section*.xml 파싱 — HWPX 섹션 본문을 Section 모델로 변환
//!
//! 섹션 XML의 문단(<hp:p>), 텍스트 런(<hp:run>), 표(<hp:tbl>),
//! 이미지(<hp:pic>) 등을 기존 Document 모델로 변환한다.

use quick_xml::events::{BytesRef, Event};
use quick_xml::Reader;

use crate::model::control::{
    AutoNumber, AutoNumberType, Bookmark, CharOverlap, Control, Equation, Field, FieldType,
    FormObject, FormType, HiddenComment, IndexMark, NewNumber, PageHide, PageNumCtrl,
    PageNumberPos, PageStartsOn, Parameter, ParameterList, Ruby, EQUATION_LINE_MODE_BIT,
};
use crate::model::document::{Section, SectionDef};
use crate::model::footnote::{Endnote, Footnote};
use crate::model::header_footer::{Footer, Header, HeaderFooterApply, MasterPage};
use crate::model::image::{
    CropInfo, EffectColor, EffectPoint, EffectRgb, ImageAttr, ImageEffect, PictureEffects,
    PictureShadow,
};
use crate::model::page::{
    BindingMethod, ColumnDef, ColumnDirection, ColumnType, PageBorderBasis, PageBorderFill,
    PageBorderUiBasis, PageDef,
};
use crate::model::paragraph::{
    CharShapeRef, FieldRange, LineSeg, OrphanFieldEnd, Paragraph, TitleMark,
};
use crate::model::shape::{
    ArcShape, CommonObjAttr, ConnectorControlPoint, ConnectorData, CurveShape, DrawingObjAttr,
    EllipseShape, GroupShape, HorzAlign, HorzRelTo, LineShape, LinkLineType, PolygonShape,
    RectangleShape, ShapeComponentAttr, ShapeObject, SizeCriterion, TextBox, TextWrap, VertAlign,
    VertRelTo,
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

    link_orphan_field_ends(&mut section.paragraphs);

    Ok(section)
}

/// 같은 문단 목록 안에서 끝난 다문단 fieldEnd에 짝 fieldBegin의 HWP5 control id를 연결한다.
///
/// HWPX fieldEnd는 beginIDRef와 fieldid만 보관하므로, HWP5 PARA_TEXT로 다시 쓸 때 필요한
/// field control fourcc는 앞 문단의 fieldBegin에서 찾아야 한다. 짝을 찾지 못한 종료 마커는
/// 그대로 남긴다. 임의의 필드 종류를 만들어 내는 것보다 보존 실패를 명시하는 편이 안전하다.
fn link_orphan_field_ends(paragraphs: &mut [Paragraph]) {
    let mut open_fields: Vec<(u32, u32)> = Vec::new();

    for para in paragraphs.iter_mut() {
        for orphan in &mut para.orphan_field_ends {
            let Some((field_id, ctrl_id)) = open_fields.last().copied() else {
                continue;
            };

            // HWPX는 beginIDRef로 짝을 식별한다. 0은 손상·부분 입력 호환을 위한
            // 미지정값이므로 HWP5 parser와 같이 현재 열린 필드에 연결한다.
            if orphan.begin_id_ref != 0 && orphan.begin_id_ref != field_id {
                continue;
            }

            open_fields.pop();
            if orphan.begin_id_ref == 0 {
                orphan.begin_id_ref = field_id;
            }
            orphan.begin_ctrl_id = ctrl_id;
        }

        for (control_idx, control) in para.controls.iter().enumerate() {
            let Control::Field(field) = control else {
                continue;
            };
            let closes_in_this_paragraph = para
                .field_ranges
                .iter()
                .any(|range| range.control_idx == control_idx);
            if !closes_in_this_paragraph && field.field_id != 0 {
                open_fields.push((field.field_id, field.ctrl_id));
            }
        }
    }
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
            // 표지(첫 쪽) 전용 바탕쪽. serializer 는 방출하나 종전엔 미독 →
            // pageFront="1" 바탕쪽이 왕복 시 "0" 으로 적용 범위가 바뀌었다.
            b"pageFront" => master_page.page_front = attr_str(&attr) != "0",
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
            // 세로쓰기 바탕쪽(hp:subList@textDirection). serializer 는 항상
            // HORIZONTAL 로 고정 출력하지만 종전엔 파서가 미독 →
            // textDirection="VERTICAL" 바탕쪽이 왕복 시 가로쓰기로 바뀌었다.
            b"textDirection" => {
                master_page.text_direction = if attr_str(&attr) == "VERTICAL" { 1 } else { 0 };
            }
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
            // [#2779] memoShapeIDRef → memo_shape_id (UINT16, header.xml `hh:memoPr@id` 참조).
            // 종전엔 수집하지 않아 저장 시 템플릿 상수 "0" 으로 리셋됐다(실측 14 secPr/9 파일).
            b"memoShapeIDRef" => {
                sec_def.memo_shape_id = parse_u16(&attr);
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

/// [#4759] HWPX 섹션 본문의 상호재귀 — 문단↔표↔셀(`parse_paragraph`↔`parse_table`
/// ↔`parse_table_cell`), 글상자 `drawText`, 서브리스트 각주 — 는 파일이 정하는
/// 중첩 깊이로 무한 재귀할 수 있다. 상한이 없으면 `<hp:tbl><hp:tr><hp:tc><hp:p>…` 를
/// 수만 겹 중첩한 section XML 하나로 네이티브 스택을 고갈시켜 프로세스를 죽인다
/// (패닉과 달리 catch_unwind 로 못 잡는 SIGSEGV). 이 재귀 계열은 **전부
/// `parse_paragraph` 를 경유**하므로, 그 진입 깊이를 스레드-로컬 카운터로 세어 한
/// 곳에서 전 경로를 막는다(파라미터를 여러 호출부에 관통시키지 않는다). 그룹
/// (`<hp:container>`) 자기재귀는 별도로 `MAX_HWPX_CONTAINER_DEPTH` 가 막는다.
/// 상한 64 는 그 형제 가드와 같은 값이다(실문서의 표 중첩은 이에 한참 못 미친다).
const MAX_HWPX_SECTION_DEPTH: u32 = 64;

thread_local! {
    // 완전 경로 — 이 모듈의 `Cell` 은 이미 `crate::model::table::Cell`(표 셀)이다.
    static HWPX_SECTION_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// `parse_paragraph` 진입 시 재귀 깊이를 +1 하고 이탈(Drop, 오류 전파·조기 반환
/// 포함) 시 되돌리는 RAII 가드. 상한 초과면 스택을 고갈시키기 전에 오류로 거부한다.
struct SectionDepthGuard;

impl SectionDepthGuard {
    fn enter() -> Result<SectionDepthGuard, HwpxError> {
        HWPX_SECTION_DEPTH.with(|d| {
            if d.get() >= MAX_HWPX_SECTION_DEPTH {
                return Err(HwpxError::XmlError(format!(
                    "section nesting exceeds {} levels",
                    MAX_HWPX_SECTION_DEPTH
                )));
            }
            d.set(d.get() + 1);
            Ok(SectionDepthGuard)
        })
    }
}

impl Drop for SectionDepthGuard {
    fn drop(&mut self) {
        HWPX_SECTION_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

fn parse_paragraph(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<(Paragraph, Option<SectionDef>), HwpxError> {
    // [#4759] 문단-경유 상호재귀(표·글상자·서브리스트) 깊이 상한 — 위 가드 참고.
    let _depth_guard = SectionDepthGuard::enter()?;
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
                                                              // 템플릿 첫 run 의 secPr/colPr 뒤에는 같은 charPrIDRef 의 텍스트 run 이 한 번 더
                                                              // 올 수 있다. 이 경우만 HWP PARA_CHAR_SHAPE 의 단일 시작 entry 로 정규화한다.
                                                              // 일반 동일-ID run 경계는 위치 자체가 IR이 보존할 정보이므로 제거하면 안 된다 (#3739).
    let mut preceding_run_had_sec_pr = false;
    let mut preceding_run_char_shape_id: Option<u32> = None;
    // [Task #1556] fieldEnd 의 (beginIDRef, fieldid) 를 출현 순서대로 보관 — text_parts 의
    // `\u{0004}` 와 1:1 대응. 고아 fieldEnd 복원에 사용.
    let mut field_end_attrs: Vec<(u32, u32)> = Vec::new();

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
                        let template_sec_pr_handoff = preceding_run_had_sec_pr
                            && preceding_run_char_shape_id == Some(current_char_shape_id);
                        if !template_sec_pr_handoff {
                            char_shape_changes.push((utf16_pos, current_char_shape_id));
                        }
                        preceding_run_had_sec_pr = false;
                        preceding_run_char_shape_id = Some(current_char_shape_id);
                    }
                    b"t" => {
                        // 텍스트 읽기 (탭 확장 데이터 포함)
                        let (parts, tab_exts) = read_text_content_with_tabs(reader)?;
                        text_parts.extend(parts);
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
                        preceding_run_had_sec_pr = true;
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
                    b"rect" | b"ellipse" | b"line" | b"connectLine" | b"arc" | b"polygon"
                    | b"curve" => {
                        // 그리기 객체 파싱
                        let shape = parse_shape_object(local, ce, reader)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(shape);
                    }
                    b"container" => {
                        // 묶음(그룹) 객체 파싱 (최상위 그룹 — 깊이 0)
                        let group = parse_container(ce, reader, 0)?;
                        text_parts.push("\u{0002}".to_string());
                        para.controls.push(group);
                    }
                    b"ctrl" => {
                        parse_ctrl(
                            ce,
                            reader,
                            &mut para.controls,
                            &mut text_parts,
                            &mut field_end_attrs,
                        )?;
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
                        // "데이터 없음" 마커(width=0, #4403)는 tab_extended 에 싣지 않는다 —
                        // 렌더러가 TabDef 기준으로 다시 계산하도록 원본처럼 비워 둔다.
                        let ext = parse_tab_extension(ce);
                        if !is_tab_no_data_marker(&ext) {
                            para.tab_extended.push(ext);
                        }
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
    let mut orphan_field_ends: Vec<OrphanFieldEnd> = Vec::new();
    let mut field_stack: Vec<(usize, usize)> = Vec::new();
    let mut control_idx: usize = 0;
    let mut visible_char_idx: usize = 0;
    let mut field_end_idx: usize = 0;

    for part in &text_parts {
        match part.as_str() {
            "\u{0003}" => {
                if matches!(para.controls.get(control_idx), Some(Control::Field(_))) {
                    field_stack.push((visible_char_idx, control_idx));
                }
                control_idx += 1;
            }
            "\u{0004}" => {
                let (begin_id_ref, field_id) = field_end_attrs
                    .get(field_end_idx)
                    .copied()
                    .unwrap_or((0, 0));
                field_end_idx += 1;
                if let Some((start_char_idx, begin_control_idx)) = field_stack.pop() {
                    // 필드 안쪽 컨트롤 슬롯 수 — 표·그림을 감싼 누름틀이 텍스트 축에서
                    // 0길이가 되어도 직렬화기가 fieldEnd 위치를 잃지 않게 한다.
                    // (begin 자신의 컨트롤 1개는 제외)
                    let inner_slot_count = control_idx.saturating_sub(begin_control_idx + 1);
                    field_ranges.push(FieldRange {
                        start_char_idx,
                        end_char_idx: visible_char_idx,
                        control_idx: begin_control_idx,
                        end_field_id: field_id,
                        inner_slot_count,
                    });
                } else {
                    // [Task #1556] 짝 fieldBegin 이 다른 문단에 있는 다단락 필드의 종료 마커.
                    // 현 문단에 컨트롤·FieldRange 가 없으므로 위치+attrs 를 기록해 직렬화기가 복원.
                    orphan_field_ends.push(OrphanFieldEnd {
                        char_idx: visible_char_idx,
                        begin_id_ref,
                        field_id,
                        // HWPX `<hp:fieldEnd>` 는 필드 종류를 싣지 않는다 — 짝
                        // `fieldBegin` 은 다른 문단에 있어 여기서는 알 수 없다.
                        begin_ctrl_id: 0,
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
            // 제목 차례 표시는 `text_parts` 안에서는 두 문자 센티널이지만 실제 본문
            // 텍스트에는 들어가지 않는다. fieldRange는 visual_text 좌표를 쓰므로 여기서
            // 센티널 길이를 더하면 그 뒤 fieldBegin/fieldEnd가 두 글자 밀린다.
            TITLE_MARK_PART_IGNORE | TITLE_MARK_PART_KEEP => {}
            _ => {
                visible_char_idx += part.chars().count();
            }
        }
    }
    para.field_ranges = field_ranges;
    para.orphan_field_ends = orphan_field_ends;

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
            TITLE_MARK_PART_IGNORE | TITLE_MARK_PART_KEEP => {
                para.title_marks.push(TitleMark {
                    char_idx: visual_text.chars().count(),
                    ignore: part.as_str() == TITLE_MARK_PART_IGNORE,
                });
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
    para.has_para_text =
        !para.text.is_empty() || !para.controls.is_empty() || !para.title_marks.is_empty();

    // char_shapes는 원본 문단 순서(text_parts)를 기준으로 계산한 위치를 그대로 사용한다.
    // 같은 char_shape_id라도 run 시작 위치가 다르면 HWP PARA_CHAR_SHAPE 의 의미 있는
    // 경계다. secPr 템플릿 handoff만 run 시작 시점에 별도로 정규화한다 (#3739).
    para.char_shapes = char_shape_changes
        .into_iter()
        .map(|(pos, id)| CharShapeRef {
            start_pos: pos,
            char_shape_id: id,
        })
        .collect();

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

    // [#1380] 원본에 `<hp:linesegarray>` 가 없는 문단은 line_segs 를 빈 채로 유지한다.
    // 종전에는 zero-default LineSeg 1개를 합성 주입했으나, serializer 가 이 주입분을
    // `vertsize="0" ...` lineseg 로 방출하여 원본 무 → RT 유 비대칭을 만들었다.
    // 한컴은 lineseg 가 없으면 열 때 재계산하므로 빈 채 보존이 안전하다.

    // [#2070] 전부 0 높이(lh=0, th=0)인 linesegarray 는 부재로 정규화한다.
    // 생성계 문서(80168 등 규제영향분석서)는 lineseg 를 0 으로 채워 저장하는데,
    // 0 높이 lineseg 는 배치 권위가 없고(한글은 열 때 재계산) 실저장 취급 시
    // NO_LS 성장 경로가 죽어 셀/문단 높이가 선언값으로 붕괴한다 (#1380 대칭,
    // body_text.rs parse_para_line_seg 와 동일 규칙).
    if !para.line_segs.is_empty()
        && para
            .line_segs
            .iter()
            .all(|s| s.line_height == 0 && s.text_height == 0)
    {
        para.line_segs.clear();
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
                        let (pbf, apply_type) = parse_page_border_fill(e, reader)?;
                        push_page_border_fill(
                            sec_def,
                            pbf,
                            &apply_type,
                            &mut page_border_fill_count,
                        );
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
                        let (pbf, apply_type) = parse_page_border_fill_empty(e);
                        push_page_border_fill(
                            sec_def,
                            pbf,
                            &apply_type,
                            &mut page_border_fill_count,
                        );
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
    let mut saw_above_line = false;
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
                                b"type" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        shape.number_format =
                                            crate::model::footnote::FootnoteShape::number_format_from_name(
                                                s,
                                                shape.number_format,
                                            );
                                    }
                                }
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
                                b"supscript" => {
                                    shape.number_code_superscript = parse_bool_attr(&attr);
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
                                            // 한컴 미주 기본값 "14692344"(전폭 sentinel)는 i16을
                                            // 넘으므로 절단하지 않고 그대로 보존한다. 렌더러가 col
                                            // 폭으로 clamp → 전폭. (i16 절단 시 12280 → 짧은 구분선)
                                            shape.separator_length = v;
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
                                // 공식 미주/각주 모양 의미:
                                // betweenNotes → 앞 번호 주석 내용과 다음 번호 주석 내용 사이
                                // belowLine → 구분선과 주석 내용 사이
                                // aboveLine → 본문과 구분선 사이
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
                                            shape.separator_margin_top = v;
                                            saw_above_line = true;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        // 일부 오래된 HWPX에는 aboveLine 이 생략될 수 있으므로 기존 sentinel
                        // fallback 만 유지한다. aboveLine 이 있으면 공식 "구분선 위" 값으로 쓴다.
                        if !saw_above_line
                            && shape.separator_margin_top == 0
                            && shape.separator_line_type != 0
                        {
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
                                        let numbering = match s {
                                            "CONTINUOUS" | "continue" => {
                                                crate::model::footnote::FootnoteNumbering::Continue
                                            }
                                            "ON_SECTION" | "RESTART_SECTION" | "restartSection" => {
                                                crate::model::footnote::FootnoteNumbering::RestartSection
                                            }
                                            "ON_PAGE" | "RESTART_PAGE" | "restartPage" => {
                                                crate::model::footnote::FootnoteNumbering::RestartPage
                                            }
                                            _ => continue,
                                        };
                                        shape.numbering = numbering;
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
                            match attr.key.as_ref() {
                                b"place" => {
                                    if let Ok(s) = std::str::from_utf8(&attr.value) {
                                        // [#2779] OWPML 스키마(ParaList placement@place)의 정식
                                        // 토큰은 컨텍스트마다 다르지만 HWP5 attr bits 8-9 코드
                                        // 공간은 공유한다:
                                        //   각주 EACH_COLUMN(0)·MERGED_COLUMN(1)·RIGHT_MOST_COLUMN(2)
                                        //   미주 END_OF_DOCUMENT(0)·END_OF_SECTION(1)
                                        // 종전엔 MERGED_COLUMN/RIGHT_MOST_COLUMN 이 표에 없어
                                        // `_ => continue` 로 떨어져, 통단·오른쪽단 각주가 파싱
                                        // 단계에서 기본값(각 단마다)으로 소실됐다.
                                        // (BELOW_TEXT/RIGHT_COLUMN 은 스키마 밖 관용 표기 — 수용 유지.)
                                        let placement = match s {
                                            "END_OF_SECTION" | "MERGED_COLUMN" | "BELOW_TEXT"
                                            | "sectionEnd" | "belowText" => {
                                                crate::model::footnote::FootnotePlacement::BelowText
                                            }
                                            "RIGHT_MOST_COLUMN" | "RIGHT_COLUMN"
                                            | "rightColumn" => {
                                                crate::model::footnote::FootnotePlacement::RightColumn
                                            }
                                            "END_OF_DOCUMENT" | "EACH_COLUMN" | "documentEnd"
                                            | "eachColumn" => {
                                                crate::model::footnote::FootnotePlacement::EachColumn
                                            }
                                            _ => continue,
                                        };
                                        shape.placement = placement;
                                    }
                                }
                                b"beneathText" => {
                                    shape.print_inline_after_text = parse_bool_attr(&attr);
                                }
                                _ => {}
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
    shape.attr = shape.encode_attr();
    Ok(())
}

/// `type`(BOTH/EVEN/ODD) 속성 값을 기준으로 슬롯을 배정한다. XML 등장 순서가
/// BOTH → EVEN → ODD 를 보장하지 않으므로(#2885), 파싱된 `type` 값을 우선 사용하고
/// 인식하지 못하는/누락된 값에 한해서만 기존 등장 순서 기반 폴백을 적용한다.
fn push_page_border_fill(
    sec_def: &mut SectionDef,
    page_border_fill: PageBorderFill,
    apply_type: &str,
    count: &mut usize,
) {
    match apply_type.to_ascii_uppercase().as_str() {
        "BOTH" => sec_def.page_border_fill = page_border_fill,
        "EVEN" => {
            if sec_def.extra_page_border_fills.is_empty() {
                sec_def.extra_page_border_fills.push(page_border_fill);
            } else {
                sec_def.extra_page_border_fills[0] = page_border_fill;
            }
        }
        "ODD" => {
            while sec_def.extra_page_border_fills.is_empty() {
                sec_def
                    .extra_page_border_fills
                    .push(PageBorderFill::default());
            }
            if sec_def.extra_page_border_fills.len() < 2 {
                sec_def.extra_page_border_fills.push(page_border_fill);
            } else {
                sec_def.extra_page_border_fills[1] = page_border_fill;
            }
        }
        _ => {
            // type 값이 없거나 인식 불가 — 기존 등장 순서 기반 폴백(회귀 방지).
            if *count == 0 {
                sec_def.page_border_fill = page_border_fill;
            } else {
                sec_def.extra_page_border_fills.push(page_border_fill);
            }
        }
    }
    *count += 1;
}

fn parse_page_border_fill(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<(PageBorderFill, String), HwpxError> {
    let (mut page_border_fill, apply_type) = parse_page_border_fill_empty(e);
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
    Ok((page_border_fill, apply_type))
}

fn parse_page_border_fill_empty(e: &quick_xml::events::BytesStart) -> (PageBorderFill, String) {
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
    (page_border_fill, apply_type)
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
            // 쪽 번호 시작 종류(0=이어서/1=홀수/2=짝수, flags bit20-21). 종전엔
            // 미독이라 HWPX 왕복 시 홀/짝 시작이 유실됐다(serializer 는 BOTH 고정).
            b"pageStartsOn" => {
                sec_def.page_num_type = match attr_str(&attr).as_str() {
                    "ODD" => 1,
                    "EVEN" => 2,
                    _ => 0,
                };
            }
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
            b"hideFirstEmptyLine" => {
                sec_def.hide_empty_line = attr_str(&attr) == "1";
                if sec_def.hide_empty_line {
                    sec_def.flags |= 0x0008_0000;
                } else {
                    sec_def.flags &= !0x0008_0000;
                }
            }
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

/// <hp:colPr> 요소의 속성과 자식 <hp:colLine>/<hp:colSz> 파싱 → ColumnDef
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
                    b"colSz" => parse_col_sz(ce, &mut cd),
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
                    b"colSz" => {
                        parse_col_sz(ce, &mut cd);
                        skip_element(reader, b"colSz")?;
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

/// <hp:colSz width="..." gap="..."/> 파싱 → ColumnDef.widths/gaps (#4387).
///
/// `sameSz="false"` 일 때 단 개수만큼(최대 255) 반복되는 요소로, 단별 절대
/// HWPUNIT 너비·뒤 간격을 담는다 (`mydocs/manual/OWPML SCHEMA/ParaList XML
/// schema.xml:1415` ColumnDefType). HWPX 는 절대값이므로 `proportional_widths`
/// 는 건드리지 않는다 — `ColumnDef::default()` 의 `false` 가 이미 정답이다
/// (HWP 5.0 바이너리 파서(body_text.rs)만 비례값이라 true 로 켠다).
///
/// [#4387 후속] 스키마상 `width` 는 `xs:positiveInteger`(상한 없음)인데
/// `ColumnDef.widths/gaps: Vec<HwpUnit16>` 은 i16(최대 32767 HWPUNIT ≈
/// 115.6mm)이다. A3 등 큰 용지나 비대칭 다단(예: 35000+13000)처럼 실측치가
/// i16 범위를 넘으면 공용 `parse_i16`(무경고 0-폴백)이 조용히 0으로 떨어뜨려
/// 단이 통째로 사라진다 — IR 폭 확장은 HWP5 바이너리 경로까지 파급이 커
/// 이번 범위를 넘으므로, 대신 saturating 클램프로 "조용한 소실/부호反전"을
/// "포화값으로 잘림 + 경고"로 좁힌다. 근본 해결(IR 타입 확장)은 별도 이슈로
/// 추적한다.
fn parse_col_sz(e: &quick_xml::events::BytesStart, cd: &mut ColumnDef) {
    let mut width: HwpUnit16 = 0;
    let mut gap: HwpUnit16 = 0;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"width" => width = parse_hwpunit16_saturating(&attr, "colSz@width"),
            // gap 은 스키마상 xs:nonNegativeInteger — 음수 폴백 없이 0 이상만 허용.
            b"gap" => gap = parse_hwpunit16_saturating(&attr, "colSz@gap").max(0),
            _ => {}
        }
    }
    cd.widths.push(width);
    cd.gaps.push(gap);
}

/// XML 정수 속성을 `HwpUnit16`(i16)로 saturating 변환한다.
///
/// 공용 `parse_i16`(utils.rs)은 `str::parse::<i16>()` 오버플로 시 무경고
/// `unwrap_or(0)`이라 `positiveInteger` 등 무제한 스키마 값이 i16 범위를 넘으면
/// 조용히 0이 된다(#4387 후속 — colSz 처럼 HWPX 가 절대 HWPUNIT 을 그대로
/// 담는 자리에서 실측 재현됨). i64 로 먼저 파싱해 i16 범위로 clamp 하고,
/// 실제로 잘렸을 때만 stderr 경고를 남긴다(section.rs 의 다른 속성 파서들과
/// 달리 이 값은 손실 시 단이 통째로 사라지는 시각적 결함으로 이어져 무음
/// 폴백이 특히 위험하다).
fn parse_hwpunit16_saturating(
    attr: &quick_xml::events::attributes::Attribute,
    field: &str,
) -> HwpUnit16 {
    let raw = attr_str(attr);
    match raw.parse::<i64>() {
        Ok(v) => {
            let clamped = v.clamp(HwpUnit16::MIN as i64, HwpUnit16::MAX as i64) as HwpUnit16;
            if clamped as i64 != v {
                eprintln!(
                    "경고: {} 값 {} 이(가) HwpUnit16 범위를 초과해 {} 로 잘렸습니다",
                    field, v, clamped
                );
            }
            clamped
        }
        Err(_) => 0,
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

/// `text_parts` 안에서 제목 차례 표시를 나타내는 센티널 — `ignore="1"` 쪽.
///
/// 표시는 텍스트가 아니라 8유닛 슬롯이라 `visual_text` 에 실리지 않는다. 표(`\u{0002}`)
/// 처럼 조각 하나를 통째로 차지하는 마커로 두고, 문단 조립 루프가 위치만 걷어 간다.
const TITLE_MARK_PART_IGNORE: &str = "\u{0008}1";
/// `text_parts` 안의 제목 차례 표시 센티널 — `ignore="0"` 쪽.
const TITLE_MARK_PART_KEEP: &str = "\u{0008}0";

/// <hp:t> 텍스트 컨텐츠를 읽는다.
/// 탭 확장 데이터도 함께 반환 (HWPX 인라인 탭의 leader/type/width)
fn read_text_content(reader: &mut Reader<&[u8]>) -> Result<String, HwpxError> {
    let (parts, _) = read_text_content_with_tabs(reader)?;
    Ok(parts
        .into_iter()
        .filter(|p| p != TITLE_MARK_PART_IGNORE && p != TITLE_MARK_PART_KEEP)
        .collect())
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
) -> Result<(Vec<String>, Vec<[u16; 7]>), HwpxError> {
    let mut parts: Vec<String> = Vec::new();
    let mut text = String::new();
    let mut tab_ext_buf: Vec<[u16; 7]> = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(ref t)) => {
                text.push_str(&t.decode().unwrap_or_default());
            }
            // 본문 런 텍스트가 CDATA 로 저장된 경우. 이 분기가 없으면 `_ => {}` 로
            // 버려져 문단 텍스트가 통째로 소실된다(#2916·#2951·#2974 와 같은 결함
            // 클래스이나, 여기는 수식·덧말이 아닌 일반 <hp:t> 경로다).
            Ok(Event::CData(ref cdata)) => {
                text.push_str(&String::from_utf8_lossy(cdata.as_ref()));
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
                        // "데이터 없음" 마커(width=0, #4403)는 tab_extended 에 싣지 않는다 —
                        // 렌더러가 TabDef 기준으로 다시 계산하도록 원본처럼 비워 둔다.
                        let ext = parse_tab_extension(ce);
                        if !is_tab_no_data_marker(&ext) {
                            tab_ext_buf.push(ext);
                        }
                    }
                    b"nbSpace" => text.push('\u{00A0}'),
                    b"fwSpace" => text.push('\u{2007}'),
                    // 소프트 하이픈 — 줄바꿈 자리에서만 보인다. 리터럴 '-' 와 구별해야
                    // 저장 왕복에서 단어가 갈라지지 않는다(ParaList XML schema.xml:291).
                    b"hyphen" => text.push('\u{00AD}'),
                    // 제목 차례 표시 — 8유닛 슬롯이라 텍스트가 아니라 조각 마커로 끊는다.
                    // 이걸 흘리면 저장본 축이 8유닛 짧아져 한글이 본문을 통째로 버린다.
                    b"titleMark" => {
                        if !text.is_empty() {
                            parts.push(std::mem::take(&mut text));
                        }
                        let ignore = ce
                            .attributes()
                            .flatten()
                            .find(|a| a.key.as_ref() == b"ignore")
                            .map(|a| {
                                let v = String::from_utf8_lossy(&a.value).to_lowercase();
                                v == "1" || v == "true"
                            })
                            .unwrap_or(false);
                        parts.push(
                            if ignore {
                                TITLE_MARK_PART_IGNORE
                            } else {
                                TITLE_MARK_PART_KEEP
                            }
                            .to_string(),
                        );
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("text: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    if !text.is_empty() || parts.is_empty() {
        parts.push(text);
    }
    Ok((parts, tab_ext_buf))
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

/// `<hp:tab width="0" leader="0" type="1"/>` — 서식기(`serializer/hwpx/section.rs`
/// `TAB_NO_DATA_WIDTH_MARKER`)가 `tab_extended` 항목이 없던 "암묵적 기본 탭"을 내보낼 때
/// 쓰는 정확한 마커다(#4403). 실제 탭은 폭 0 이 나올 수 없으므로(시각적으로 아무 효과가 없어
/// 한컴도 만들지 않는다) 안전한 신호로 쓴다. `leader`/`type` 까지 우리 서식기의 고정 폴백값과
/// 정확히 일치할 때만 마커로 인정해, width=0 인 (극히 드문) 진짜 캡처 데이터를 오인해 버리지
/// 않도록 한다. 이 마커를 만나면 `tab_extended` 에 항목을 추가하지 않아, 렌더러가 문단의 실제
/// `TabDef`/커서 위치 기준 `find_next_tab_stop` 으로 탭 정지를 다시 계산하게 한다 — HWP5
/// 바이너리 파서의 동형 널 마커 스킵(`parser/body_text.rs` `is_null_ext`, #1892)과 같은 규약.
fn is_tab_no_data_marker(ext: &[u16; 7]) -> bool {
    ext[0] == 0 && ext[2] == 0x0100
}

// ─── Table ───

fn parse_table(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Table, HwpxError> {
    let mut table = Table::default();
    let mut table_record_flags = 0u32;
    // [#2697] numberingType 부재 시 표의 자연 기본값은 TABLE (종전 방출 리터럴과 동일).
    table.common.numbering_type = crate::model::shape::ObjectNumberingType::Table;

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
                    // 표 textWrap 파서만 TIGHT/THROUGH arm 이 빠져 있어, 방출측
                    // (text_wrap_str)이 내는 이 두 값이 SQUARE 로 유실됐다. 도형/그림/차트
                    // 파서(같은 파일 2228/2795/5681)는 이미 처리하므로 표만 맞춘다.
                    "TIGHT" => crate::model::shape::TextWrap::Tight,
                    "THROUGH" => crate::model::shape::TextWrap::Through,
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
            // [#2697] 표만 numberingType arm 이 없어 캡션 번호 범주가 파싱 단계에서
            // 유실됐다. 도형 파서(같은 파일 2855)와 동형. 방출측은 종전 "TABLE" 하드코딩.
            b"numberingType" => {
                table.common.numbering_type = match attr_str(&attr).to_ascii_uppercase().as_str() {
                    "PICTURE" => crate::model::shape::ObjectNumberingType::Picture,
                    "TABLE" => crate::model::shape::ObjectNumberingType::Table,
                    "EQUATION" => crate::model::shape::ObjectNumberingType::Equation,
                    _ => crate::model::shape::ObjectNumberingType::None,
                };
            }
            // [#2855] 표만 lock arm 이 없어 개체 잠금이 파싱 단계에서 유실됐다. 도형/그림
            // 계열이 공유하는 parse_object_element_attrs(같은 파일 2905행, #2840)와 동형.
            b"lock" => table.common.locked = attr_str(&attr) == "1",
            _ => {}
        }
    }

    // 표 내용 파싱 (행/셀)
    let mut buf = Vec::new();
    let mut current_row: u16 = 0;

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
                        let caption = parse_caption(ce, reader)?;
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
                                // [#2697] 표만 protect arm 이 없어 "표 크기 보호"가 파싱
                                // 단계에서 유실됐다. 도형(2907)·사각형(5967)·양식(5590)
                                // 파서는 모두 같은 hp:sz@protect 를 읽는다.
                                b"protect" => table.common.size_protect = parse_bool(&attr),
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
                                // [#2784] affectLSpacing(줄 간격에 영향) — 표 pos 되읽기.
                                b"affectLSpacing" => {
                                    table.common.affect_line_spacing = parse_bool(&attr)
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
                                    table.common.vertical_offset = parse_i32_wrapping(&attr) as u32;
                                }
                                b"horzOffset" => {
                                    table.common.horizontal_offset =
                                        parse_i32_wrapping(&attr) as u32;
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

    // [Task #1772] outMargin → common.margin 동기화 (IR 계약).
    // 레이아웃의 쪽 고정 자리차지 표 예약 하단(calc_shape_bottom_y)은 common.margin 을
    // 참조하고, HWPX→HWP 어댑터(materialize_table_outer_margin)도 직렬화 시 동일하게
    // 동기화한다. 파서가 outer_margin_* 만 채우면 HWPX 직파스 문서에서만 표 바깥 여백이
    // 무시되어 본문이 저장 lineseg(한컴 위치)보다 위로 붙는다 (11.36px 군집).
    table.common.margin.left = table.outer_margin_left;
    table.common.margin.right = table.outer_margin_right;
    table.common.margin.top = table.outer_margin_top;
    table.common.margin.bottom = table.outer_margin_bottom;

    // row_sizes 설정 (행별 셀 수, HWP 스펙 UINT16[NRows] 계약과 동일 — 높이가 아니다).
    // model::table::Table::rebuild_row_sizes, parser::control(HWP5), html_table_import,
    // document_core::commands::object_ops::table 이 모두 이 필드를 "행별 셀 개수"로 채운다.
    table.row_sizes = (0..table.row_count)
        .map(|r| table.cells.iter().filter(|c| c.row == r).count() as i16)
        .collect();

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

    // [#2697] "표 번호" 비트는 numberingType 이 실제로 TABLE 일 때만 세운다. 종전 무조건 OR
    // 은 numberingType="PICTURE" 표에서 IR 모순(numbering_type=Picture ↔ attr=TABLE)을 만든다.
    // 차트 파서(5800)가 PICTURE 를 별도 비트로 분기하는 것과 같은 취지.
    let mut attr = pack_hwpx_common_obj_attr(&table.common);
    if table.common.numbering_type == crate::model::shape::ObjectNumberingType::Table {
        attr |= HWPX_TABLE_NUMBERING_BIT;
    }
    table.common.attr = attr;
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

fn parse_caption_sub_list_attrs(
    e: &quick_xml::events::BytesStart,
    caption: &mut crate::model::shape::Caption,
) {
    use crate::model::shape::CaptionVertAlign;

    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"vertAlign" {
            caption.vert_align = match attr_str(&attr).as_str() {
                "CENTER" => CaptionVertAlign::Center,
                "BOTTOM" => CaptionVertAlign::Bottom,
                // 누락·미지·미래 lexical 값은 모델 기본값을 쓴다. 다른 HWPX subList
                // enum 파서가 알 수 없는 값을 기본값으로 관용 처리하는 정책과 같다.
                _ => CaptionVertAlign::Top,
            };
        }
    }
}

/// `<hp:caption>` 파싱 — 표(#1387)·그림/도형/묶음(#1403) 공유.
fn parse_caption(
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
                match local {
                    b"subList" => parse_caption_sub_list_attrs(ce, &mut caption),
                    b"p" => {
                        let (para, _) = parse_paragraph(ce, reader)?;
                        caption.paragraphs.push(para);
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref ce)) if local_name(ce.name().as_ref()) == b"subList" => {
                parse_caption_sub_list_attrs(ce, &mut caption);
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
            b"header" => cell.set_header(parse_bool(&attr)),
            b"hasMargin" => cell.set_apply_inner_margin(parse_bool(&attr)),
            b"protect" => cell.set_cell_protect(parse_bool(&attr)),
            b"editable" => cell.set_editable_in_form(parse_bool(&attr)),
            b"dirty" => cell.dirty_flag = parse_bool(&attr),
            // 셀 필드 이름 (누름틀 셀 필드, #493). 직렬화기는 무명 셀도 name=""로
            // 항상 방출하므로 빈 값은 None — HWP5 파서(parse_cell_field_name)와
            // 동일 의미. 누락 시 HWPX 로드에서 getFieldList가 셀 필드를 반환하지 못하고
            // HWPX 라운드트립에서 셀 필드 이름이 유실된다.
            b"name" => {
                let v = attr_str(&attr);
                cell.field_name = if v.is_empty() { None } else { Some(v) };
            }
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
                        // subList: vertAlign + textDirection 속성 파싱
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"vertAlign" => {
                                    cell.vertical_align = match attr_str(&attr).as_str() {
                                        "CENTER" => VerticalAlign::Center,
                                        "BOTTOM" => VerticalAlign::Bottom,
                                        _ => VerticalAlign::Top,
                                    };
                                }
                                // 세로쓰기 셀(textDirection). serializer 는 셀 <hp:subList>
                                // 에 방출하지만 종전엔 vertAlign 만 읽어 세로쓰기가 왕복 시
                                // 유실됐다(cellPr 경로는 serializer 가 방출하지 않음).
                                b"textDirection" => {
                                    cell.text_direction =
                                        if attr_str(&attr) == "VERTICAL" { 1 } else { 0 };
                                }
                                _ => {}
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
    let mut img_dim: (u32, u32) = (0, 0); // [#1389] hp:imgDim 원본 이미지 픽셀 크기
    let mut href: Option<String> = None;
    let mut picture_instance_id = 0;
    let mut effects = PictureEffects::default();
    let mut reverse = false;
    let mut lock = false;

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
            // [#2861] 좌우 반전(한컴 Automation InsertPicture 의 reverse 옵션과 동일 개념).
            // 종전 미매칭으로 조용히 버려져 직렬화 시 항상 reverse="0" 하드코딩되던 유실.
            b"reverse" => reverse = attr_str(&attr) == "1",
            // [#2875] 개체 잠금(보호). 종전 미매칭으로 조용히 버려져 직렬화 시 항상
            // lock="0" 하드코딩되던 유실 — #2861(reverse), #2855(hp:tbl lock)과 동일 패턴.
            b"lock" => lock = attr_str(&attr) == "1",
            // dropcapstyle (개체를 감싼 문단의 드롭캡 표시 방식) 보존.
            // 미파싱 상태에서는 picture.rs 방출측이 항상 "None"으로 되돌려,
            // DoubleLine/TripleLine/Margin 드롭캡 문단에 있던 그림이 저장 시
            // 드롭캡 스타일을 잃는다.
            b"dropcapstyle" => {
                common.drop_cap_style = match attr_str(&attr).as_str() {
                    "DoubleLine" => crate::model::shape::DropCapStyle::DoubleLine,
                    "TripleLine" => crate::model::shape::DropCapStyle::TripleLine,
                    "Margin" => crate::model::shape::DropCapStyle::Margin,
                    _ => crate::model::shape::DropCapStyle::None,
                };
            }
            // [#2697 동형] numberingType (캡션 번호 범주) 보존 — 도형·표·그림 공통 속성.
            // 종전 미파싱으로 그림에 번호 범주를 NONE 등으로 변경한 HWPX에서 IR 기본값(None)으로
            // 떨어져 왕복 시 "PICTURE"로 강제복원되던 결함을 수정한다.
            b"numberingType" => {
                common.numbering_type = match attr_str(&attr).to_ascii_uppercase().as_str() {
                    "PICTURE" => crate::model::shape::ObjectNumberingType::Picture,
                    "TABLE" => crate::model::shape::ObjectNumberingType::Table,
                    "EQUATION" => crate::model::shape::ObjectNumberingType::Equation,
                    _ => crate::model::shape::ObjectNumberingType::None,
                };
            }
            _ => {}
        }
    }

    // 이미지 속성 읽기
    let mut has_pos = false; // <pos> 파싱 여부 — <offset>이 덮어쓰지 않도록 방지
    let mut caption: Option<crate::model::shape::Caption> = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"imgRect" => {
                parse_picture_img_rect(reader, &mut border_x, &mut border_y)?;
            }
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"shapeComment" => {
                common.description = read_dutmal_text(reader, b"shapeComment")?;
            }
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"effects" => {
                effects = parse_picture_effects(reader)?;
            }
            // 그림 캡션 (#1403) — 미적재 시 roundtrip 에서 캡션 subList 소실
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"caption" => {
                caption = Some(parse_caption(ce, reader)?);
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
                                // [#2712] 그림만 크기 기준·크기 보호 arm 이 없어 파싱 단계에서
                                // 유실됐다. 도형 파서(같은 파일 2901-2907)와 동형이며, 높이는
                                // 도형과 마찬가지로 allow_column_para=false 로 읽어 치역을
                                // {Paper, Page, Absolute} 로 제한한다.
                                b"widthRelTo" => {
                                    common.width_criterion =
                                        parse_size_criterion(&attr_str(&attr), true);
                                }
                                b"heightRelTo" => {
                                    common.height_criterion =
                                        parse_size_criterion(&attr_str(&attr), false);
                                }
                                b"protect" => common.size_protect = parse_bool(&attr),
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
                    // [#1389] 원본 이미지 픽셀 크기 — verbatim 적재
                    b"imgDim" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"dimwidth" => img_dim.0 = parse_u32(&attr),
                                b"dimheight" => img_dim.1 = parse_u32(&attr),
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
                                // [#2784] affectLSpacing(줄 간격에 영향) — 그림/도형 pos 되읽기.
                                b"affectLSpacing" => common.affect_line_spacing = parse_bool(&attr),
                                b"flowWithText" => common.flow_with_text = parse_bool(&attr),
                                b"allowOverlap" => common.allow_overlap = parse_bool(&attr),
                                // holdAnchorAndSO(쪽나눔 방지). 방출측은 모든 개체에 내지만
                                // 종전엔 표 파서만 되읽어, 그림/도형/차트/OLE 는 prevent_page_break
                                // 이 0 으로 유실됐다(표 파서와 동형으로 보강).
                                b"holdAnchorAndSO" => {
                                    common.prevent_page_break =
                                        if parse_bool(&attr) { 1 } else { 0 };
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
                                b"alpha" => {
                                    img_attr.transparency =
                                        parse_picture_transparency_attr(&attr_str(&attr));
                                }
                                b"effect" => {
                                    img_attr.effect = match attr_str(&attr).as_str() {
                                        "REAL_PIC" => ImageEffect::RealPic,
                                        "GRAY_SCALE" => ImageEffect::GrayScale,
                                        "BLACK_WHITE" => ImageEffect::BlackWhite,
                                        // 방출측 image_effect_str 은 Pattern8x8 을 이 문자열로
                                        // 낸다. 안 받으면 무늬(패턴) 효과가 왕복 시 RealPic 유실.
                                        "PATTERN_8_8" => ImageEffect::Pattern8x8,
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
                                    let v = parse_i32_wrapping(&attr);
                                    shape_attr.offset_x = v;
                                    if !has_pos {
                                        common.horizontal_offset = v as u32;
                                    }
                                }
                                b"y" => {
                                    let v = parse_i32_wrapping(&attr);
                                    shape_attr.offset_y = v;
                                    if !has_pos {
                                        common.vertical_offset = v as u32;
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
    pic.effects = effects;
    pic.caption = caption;
    pic.img_dim = img_dim;
    pic.reverse = reverse;
    pic.lock = lock;

    Ok(Control::Picture(Box::new(pic)))
}

fn parse_picture_effects(reader: &mut Reader<&[u8]>) -> Result<PictureEffects, HwpxError> {
    let mut effects = PictureEffects::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if local_name(e.name().as_ref()) == b"shadow" => {
                effects.shadow = Some(parse_picture_shadow(e, reader)?);
            }
            Ok(Event::Empty(ref e)) if local_name(e.name().as_ref()) == b"shadow" => {
                effects.shadow = Some(parse_picture_shadow_attrs(e));
            }
            Ok(Event::End(ref e)) if local_name(e.name().as_ref()) == b"effects" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("effects: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(effects)
}

fn parse_picture_shadow(
    e: &quick_xml::events::BytesStart<'_>,
    reader: &mut Reader<&[u8]>,
) -> Result<PictureShadow, HwpxError> {
    let mut shadow = parse_picture_shadow_attrs(e);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) => match local_name(e.name().as_ref()) {
                b"skew" => shadow.skew = Some(parse_effect_point(e)),
                b"scale" => shadow.scale = Some(parse_effect_point(e)),
                b"effectsColor" => {
                    shadow.color = Some(parse_effect_color_attrs(e));
                }
                _ => {}
            },
            Ok(Event::Start(ref e)) if local_name(e.name().as_ref()) == b"effectsColor" => {
                shadow.color = Some(parse_effect_color(e, reader)?);
            }
            Ok(Event::End(ref e)) if local_name(e.name().as_ref()) == b"shadow" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("shadow: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(shadow)
}

fn parse_picture_transparency_attr(raw: &str) -> u8 {
    let Ok(value) = raw.trim().parse::<f64>() else {
        return 0;
    };
    if !value.is_finite() {
        return 0;
    }
    if value <= 1.0 {
        (value * 100.0).round().clamp(0.0, 100.0) as u8
    } else {
        let alpha = value.clamp(0.0, 255.0).round() as u8;
        crate::model::image::alpha_byte_to_transparency_percent(alpha)
    }
}

fn parse_picture_shadow_attrs(e: &quick_xml::events::BytesStart<'_>) -> PictureShadow {
    let mut shadow = PictureShadow::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"style" => shadow.style = Some(attr_str(&attr)),
            b"alpha" => shadow.alpha = Some(attr_str(&attr)),
            b"radius" => shadow.radius = Some(attr_str(&attr)),
            b"direction" => shadow.direction = Some(attr_str(&attr)),
            b"distance" => shadow.distance = Some(attr_str(&attr)),
            b"alignStyle" => shadow.align_style = Some(attr_str(&attr)),
            b"rotationStyle" => shadow.rotation_style = Some(attr_str(&attr)),
            _ => {}
        }
    }
    shadow
}

fn parse_effect_point(e: &quick_xml::events::BytesStart<'_>) -> EffectPoint {
    let mut point = EffectPoint::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"x" => point.x = Some(attr_str(&attr)),
            b"y" => point.y = Some(attr_str(&attr)),
            _ => {}
        }
    }
    point
}

fn parse_effect_color(
    e: &quick_xml::events::BytesStart<'_>,
    reader: &mut Reader<&[u8]>,
) -> Result<EffectColor, HwpxError> {
    let mut color = parse_effect_color_attrs(e);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if local_name(e.name().as_ref()) == b"rgb" => {
                color.rgb = Some(parse_effect_rgb(e));
            }
            Ok(Event::End(ref e)) if local_name(e.name().as_ref()) == b"effectsColor" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("effectsColor: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    Ok(color)
}

fn parse_effect_color_attrs(e: &quick_xml::events::BytesStart<'_>) -> EffectColor {
    let mut color = EffectColor::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"type" => color.color_type = Some(attr_str(&attr)),
            b"schemeIdx" => color.scheme_idx = Some(attr_str(&attr)),
            b"systemIdx" => color.system_idx = Some(attr_str(&attr)),
            b"presetIdx" => color.preset_idx = Some(attr_str(&attr)),
            _ => {}
        }
    }
    color
}

fn parse_effect_rgb(e: &quick_xml::events::BytesStart<'_>) -> EffectRgb {
    let mut rgb = EffectRgb::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"r" => rgb.r = Some(attr_str(&attr)),
            b"g" => rgb.g = Some(attr_str(&attr)),
            b"b" => rgb.b = Some(attr_str(&attr)),
            _ => {}
        }
    }
    rgb
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
    is_reverse_hv: bool,
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
        // [#2017] HWPX 재직렬화 시 원본 curSz=0 을 복원하기 위해 materialize 여부를 기록.
        shape_attr.current_width_was_zero = true;
        if common.width == 0 {
            common.width = shape_attr.original_width;
        }
    }
    if shape_attr.current_height == 0 && shape_attr.original_height > 0 {
        shape_attr.current_height = shape_attr.original_height;
        shape_attr.current_height_was_zero = true;
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
            // HWPX에는 HWP5 SHAPE_COMPONENT의 저장 전용 상위 비트가 없다. Hancom
            // 2020이 같은 HWPX를 HWP5로 저장한 값은 그림=0x2000_0000, 글상자
            // 도형=0x0100_0000이다. 그룹 자식에는 0x0003_0000도 함께 붙는다.
            // 0x2400_0000을 쓴 종전 값은 표지 묶음의 자식 좌표계를 다르게 해석하게
            // 하여 한컴 PDF에서 축척·위치를 틀리게 만들었다(#3930).
            ShapeStorageKind::Picture => 0x2000_0000,
            ShapeStorageKind::Group => 0x0009_0000,
            ShapeStorageKind::TextBoxDrawing => 0x0100_0000,
            ShapeStorageKind::Drawing => 0,
        };
        if shape_attr.group_level > 0
            && matches!(
                kind,
                ShapeStorageKind::Picture | ShapeStorageKind::TextBoxDrawing
            )
        {
            flip |= 0x0003_0000;
        }
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
            // [Task #1379] numberingType (캡션 번호 범주) 보존 — exam_kor 등 광범위 사용.
            b"numberingType" => {
                common.numbering_type = match attr_str(&attr).to_ascii_uppercase().as_str() {
                    "PICTURE" => crate::model::shape::ObjectNumberingType::Picture,
                    "TABLE" => crate::model::shape::ObjectNumberingType::Table,
                    "EQUATION" => crate::model::shape::ObjectNumberingType::Equation,
                    _ => crate::model::shape::ObjectNumberingType::None,
                };
            }
            // 선/연결선의 방향 뒤집기(isReverseHV). serializer 는 방출하나 파서가
            // 되읽지 않아 HWPX 원본 선의 방향 반전이 왕복 시 유실됐다.
            b"isReverseHV" => ids.is_reverse_hv = attr_str(&attr) == "1",
            // [#2840] 개체 잠금(lock) — 종전 미파싱으로 <hp:equation> 직렬화 시
            // 항상 "0"으로 되돌아가 원본의 잠금 상태가 유실됐다.
            b"lock" => common.locked = attr_str(&attr) == "1",
            _ => {}
        }
    }

    if common.instance_id == 0 && ids.instid != 0 {
        common.instance_id = ids.instid;
    }

    // HWP5 공통 개체 attr bit 28은 한컴 2020이 `numberingType="PICTURE"`인
    // 일반 도형/그림/묶음을 HWP로 저장할 때 함께 기록한다. 차트·OLE 경로는 이미
    // 같은 보정을 하지만, 공용 개체 경로에서 빠지면 HWPX -> HWP 저장본의 바탕쪽과
    // 본문 PICTURE 개체가 한컴 저장본과 다른 attr을 갖는다.
    if common.numbering_type == crate::model::shape::ObjectNumberingType::Picture {
        common.hwp5_gen_shape_attr_bit28 = true;
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
                    // [#2784] affectLSpacing(줄 간격에 영향) — 공통 개체 pos 되읽기.
                    b"affectLSpacing" => common.affect_line_spacing = parse_bool(&attr),
                    b"flowWithText" => common.flow_with_text = parse_bool(&attr),
                    b"allowOverlap" => common.allow_overlap = parse_bool(&attr),
                    // holdAnchorAndSO(쪽나눔 방지). 방출측은 모든 개체에 내지만
                    // 종전엔 표 파서만 되읽어 개체 배치에선 prevent_page_break 이 유실됐다.
                    b"holdAnchorAndSO" => {
                        common.prevent_page_break = if parse_bool(&attr) { 1 } else { 0 };
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
                        let v = parse_i32_wrapping(&attr);
                        shape_attr.offset_x = v;
                        if !*has_pos {
                            common.horizontal_offset = v as u32;
                        }
                    }
                    b"y" => {
                        let v = parse_i32_wrapping(&attr);
                        shape_attr.offset_y = v;
                        if !*has_pos {
                            common.vertical_offset = v as u32;
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
                    // 정본 코드 0=NONE(표 borderFill·HWP5 doc_info 와 동일). 종전 0x40 은
                    // bit 6 이 endCap(bit 6~9)에 겹쳐 써져 소실됐다(#1531).
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

fn parse_connect_line_type_attr(e: &quick_xml::events::BytesStart) -> LinkLineType {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"type" {
            return match attr_str(&attr).to_ascii_uppercase().as_str() {
                "STRAIGHT_ONEWAY" => LinkLineType::StraightOneWay,
                "STRAIGHT_BOTH" => LinkLineType::StraightBoth,
                "STROKE_NOARROW" => LinkLineType::StrokeNoArrow,
                "STROKE_ONEWAY" => LinkLineType::StrokeOneWay,
                "STROKE_BOTH" => LinkLineType::StrokeBoth,
                "ARC_NOARROW" => LinkLineType::ArcNoArrow,
                "ARC_ONEWAY" => LinkLineType::ArcOneWay,
                "ARC_BOTH" => LinkLineType::ArcBoth,
                _ => LinkLineType::StraightNoArrow,
            };
        }
    }

    LinkLineType::StraightNoArrow
}

/// [#4388] `<hp:arc>` 전용 `type` 속성 (OWPML `CArcType::WriteElement` —
/// hancom-io/hwpx-owpml-model `ArcType.cpp`) — `g_ArcTypeList`: NORMAL(0)/PIE(1)/CHORD(2).
/// `ArcShape.arc_type` (0: Arc, 1: CircularSector, 2: Bow) 와 1:1 대응. 같은 이름의
/// `type` 속성이 `<hp:connectLine>`(연결선 화살표 종류) 등 다른 도형 태그에도 쓰이므로
/// 반드시 `<hp:arc>` 요소 자체(shape_type == b"arc")에서만 호출한다.
fn parse_arc_type_attr(e: &quick_xml::events::BytesStart) -> u8 {
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"type" {
            return match attr_str(&attr).to_ascii_uppercase().as_str() {
                "PIE" => 1,
                "CHORD" => 2,
                _ => 0,
            };
        }
    }
    0
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
                                // [#2563] 헤더(borderFill) 파서와 동일한 12종 매핑.
                                // 종전엔 4종만 받아 TOTAL 등 8종이 TILE 로 붕괴했다.
                                b"mode" => {
                                    img.fill_mode = match attr_str(&attr).as_str() {
                                        "TILE" | "TILE_ALL" => ImageFillMode::TileAll,
                                        "TILE_HORZ_TOP" => ImageFillMode::TileHorzTop,
                                        "TILE_HORZ_BOTTOM" => ImageFillMode::TileHorzBottom,
                                        "TILE_VERT_LEFT" => ImageFillMode::TileVertLeft,
                                        "TILE_VERT_RIGHT" => ImageFillMode::TileVertRight,
                                        "CENTER" => ImageFillMode::Center,
                                        "CENTER_TOP" => ImageFillMode::CenterTop,
                                        "CENTER_BOTTOM" => ImageFillMode::CenterBottom,
                                        "FIT" | "FIT_TO_SIZE" | "STRETCH" => {
                                            ImageFillMode::FitToSize
                                        }
                                        "TOTAL" => ImageFillMode::Total,
                                        "TOP_LEFT_ALIGN" => ImageFillMode::LeftTop,
                                        _ => ImageFillMode::TileAll,
                                    };
                                }
                                _ => {}
                            }
                        }
                        fill.image = Some(img);
                    }
                    // [#2563] <hc:imgBrush> 의 <hc:img> 자식. 종전엔 이 arm 이 없어
                    // binaryItemIDRef/bright/contrast/effect 가 전부 버려졌고,
                    // bin_data_id 가 0 이라 직렬화가 <hc:img> 를 아예 못 내
                    // 이미지로 채운 도형이 왕복 후 빈 도형이 됐다.
                    // 헤더(borderFill) 파서 header.rs 의 b"img" arm 과 동형.
                    b"img" | b"image" => {
                        if let Some(ref mut img_fill) = fill.image {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"binaryItemIDRef" => {
                                        let val = attr_str(&attr);
                                        let num: String =
                                            val.chars().filter(|c| c.is_ascii_digit()).collect();
                                        img_fill.bin_data_id = num.parse().unwrap_or(0);
                                    }
                                    b"bright" => img_fill.brightness = parse_i8(&attr),
                                    b"contrast" => img_fill.contrast = parse_i8(&attr),
                                    b"effect" => {
                                        img_fill.effect = match attr_str(&attr).as_str() {
                                            "GRAY_SCALE" => 1,
                                            "BLACK_WHITE" => 2,
                                            _ => 0, // REAL_PIC
                                        };
                                    }
                                    _ => {}
                                }
                            }
                        }
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

/// [Task #1598] `<hc:center x="" y="">` 류 점 요소의 x/y 속성을 Point 로 읽는다.
fn parse_xy(e: &quick_xml::events::BytesStart, p: &mut crate::model::Point) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"x" => p.x = parse_i32(&attr),
            b"y" => p.y = parse_i32(&attr),
            _ => {}
        }
    }
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
                                    let dir = attr_str(&attr);
                                    let direction_code: u32 = match dir.as_str() {
                                        "VERTICAL" | "VERTICALALL" => 1,
                                        _ => 0,
                                    };
                                    text_box.list_attr =
                                        (text_box.list_attr & !0b111) | direction_code;
                                    // [Task #1379] VERTICAL/VERTICALALL 구분 보존
                                    // — serializer 역방출용 (list_attr 만으로는 구분 불가).
                                    text_box.vertical_all = dir == "VERTICALALL";
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
    // [Task #1598] ellipse / arc 전용 지오메트리 (`<hc:center>`/`<hc:ax1>`/...).
    // 미적재 시 한글이 타원/호를 다르게 렌더 → 누적 레이아웃 변동 → 페이지 붕괴(#1589 잔여).
    let mut e_center = crate::model::Point::default();
    let mut e_axis1 = crate::model::Point::default();
    let mut e_axis2 = crate::model::Point::default();
    let mut e_start1 = crate::model::Point::default();
    let mut e_end1 = crate::model::Point::default();
    let mut e_start2 = crate::model::Point::default();
    let mut e_end2 = crate::model::Point::default();

    let object_ids = parse_object_element_attrs(e, &mut common, &mut shape_attr);
    let connect_line_type = parse_connect_line_type_attr(e);
    // [#4388] `<hp:arc>` 전용 `type` 속성 — 다른 태그의 동명 `type` 속성과
    // 섞이지 않도록 shape_type == b"arc" 로 한정한다.
    let arc_type = if shape_type == b"arc" {
        parse_arc_type_attr(e)
    } else {
        0
    };
    let mut connect_start_subject_id = 0_u32;
    let mut connect_start_subject_index = 0_u32;
    let mut connect_end_subject_id = 0_u32;
    let mut connect_end_subject_index = 0_u32;
    let mut connect_control_points = Vec::new();

    let tag_name = String::from_utf8_lossy(shape_type).to_string();
    let mut caption: Option<crate::model::shape::Caption> = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"shapeComment" => {
                common.description = read_dutmal_text(reader, b"shapeComment")?;
            }
            // 도형 캡션 (#1403) — 미적재 시 roundtrip 에서 캡션 subList 소실
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"caption" => {
                caption = Some(parse_caption(ce, reader)?);
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
                                b"subjectIDRef" => connect_start_subject_id = parse_u32(&attr),
                                b"subjectIdx" => connect_start_subject_index = parse_u32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"endPt" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x_coords[1] = parse_i32(&attr),
                                b"y" => y_coords[1] = parse_i32(&attr),
                                b"subjectIDRef" => connect_end_subject_id = parse_u32(&attr),
                                b"subjectIdx" => connect_end_subject_index = parse_u32(&attr),
                                _ => {}
                            }
                        }
                    }
                    b"point" => {
                        let mut point = ConnectorControlPoint::default();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => point.x = parse_i32(&attr),
                                b"y" => point.y = parse_i32(&attr),
                                b"type" => point.point_type = parse_u16(&attr),
                                _ => {}
                            }
                        }
                        connect_control_points.push(point);
                    }
                    // [Task #1598] ellipse / arc 전용 지오메트리. x/y 속성만 읽어 Point 채움.
                    b"center" => parse_xy(ce, &mut e_center),
                    b"ax1" => parse_xy(ce, &mut e_axis1),
                    b"ax2" => parse_xy(ce, &mut e_axis2),
                    b"start1" => parse_xy(ce, &mut e_start1),
                    b"end1" => parse_xy(ce, &mut e_end1),
                    b"start2" => parse_xy(ce, &mut e_start2),
                    b"end2" => parse_xy(ce, &mut e_end2),
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
        caption,
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
            // [Task #1598] 전용 지오메트리 적재 — 누락 시 한글 페이지 붕괴(#1589 잔여).
            center: e_center,
            axis1: e_axis1,
            axis2: e_axis2,
            start1: e_start1,
            end1: e_end1,
            start2: e_start2,
            end2: e_end2,
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
            started_right_or_bottom: object_ids.is_reverse_hv,
            ..Default::default()
        }),
        b"connectLine" => ShapeObject::Line(LineShape {
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
            connector: Some(ConnectorData {
                link_type: connect_line_type,
                start_subject_id: connect_start_subject_id,
                start_subject_index: connect_start_subject_index,
                end_subject_id: connect_end_subject_id,
                end_subject_index: connect_end_subject_index,
                control_points: connect_control_points,
                raw_trailing: Vec::new(),
            }),
            started_right_or_bottom: object_ids.is_reverse_hv,
        }),
        b"arc" => ShapeObject::Arc(ArcShape {
            common,
            drawing,
            // [Task #1598] 호 전용 지오메트리(center/축).
            // [#4388] arc_type 은 `<hp:arc>` 자체의 `type` 속성(NORMAL/PIE/CHORD) —
            // 태그속성으로 읽는다(hancom-io/hwpx-owpml-model `ArcType.cpp` 확인).
            arc_type,
            center: e_center,
            axis1: e_axis1,
            axis2: e_axis2,
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
            // CurveShape 도 동일 패턴 — 누락 시 곡선 미표시.
            points: polygon_points,
            // HWPX `hp:seg type="CURVE"`는 점-대-점 체인의 표기일 뿐 HWP5 `1`이 요구하는
            // 베지어 제어점 2개를 담지 않는다. 그대로 옮기면 renderer가 세 점씩 소비하므로
            // 비워서 기존의 LineTo 체인 렌더 계약을 유지한다(#1200, #4676).
            segment_types: Vec::new(),
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

/// [#4730] HWPX 그룹(`<hp:container>`)은 자기 자신을 자식으로 가질 수 있고, 그
/// 중첩 깊이는 파일에서 그대로 온다. 상한이 없으면 `<hp:container>` 를 수만 겹
/// 중첩한 section XML 하나로 네이티브 스택을 고갈시켜 프로세스를 죽일 수 있다
/// (패닉과 달리 catch_unwind 로 못 잡는다). 여는 태그가 ~14바이트라 100,000 겹도
/// ~1.4MB 로 어떤 입력 상한에도 걸리지 않는다. HWP3 `MAX_DRAWING_OBJECT_DEPTH`
/// (#4285)·HML `HmlLimits::max_depth` 와 같은 취지로 상한을 둔다. 다만 이 함수는
/// 큰 지역 상태를 가진 재귀 함수이므로, 기본 스레드 스택에서도 가드가 먼저
/// 동작하도록 HWP3/HML의 256보다 작은 64개 그룹으로 제한한다.
const MAX_HWPX_CONTAINER_DEPTH: u32 = 64;

/// `<hp:container>` 요소를 파싱하여 `Control::Shape(GroupShape)`를 반환한다.
///
/// `depth` 는 중첩 그룹의 현재 깊이다(최상위 호출은 0). 최대 64개 그룹을
/// 허용하며, 그 다음 그룹은 스택을 고갈시키기 전에 오류로 거부한다 — 위
/// `MAX_HWPX_CONTAINER_DEPTH` 참고.
fn parse_container(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    depth: u32,
) -> Result<Control, HwpxError> {
    if depth >= MAX_HWPX_CONTAINER_DEPTH {
        return Err(HwpxError::XmlError(format!(
            "container nesting exceeds {} levels",
            MAX_HWPX_CONTAINER_DEPTH
        )));
    }
    let mut common = CommonObjAttr::default();
    let mut shape_attr = ShapeComponentAttr::default();
    let mut has_pos = false;
    let mut children = Vec::new();

    parse_object_element_attrs(e, &mut common, &mut shape_attr);

    let mut caption: Option<crate::model::shape::Caption> = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            // 묶음 개체 캡션 (#1403) — 미적재 시 roundtrip 에서 캡션 subList 소실
            Ok(Event::Start(ref ce)) if local_name(ce.name().as_ref()) == b"caption" => {
                caption = Some(parse_caption(ce, reader)?);
            }
            // 묶음 개체 설명 (#1392) — 미적재 시 roundtrip 에서 소실
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
                    b"pic" => {
                        // 자식 그림 객체
                        let child = parse_picture(ce, reader)?;
                        if let Control::Picture(pic) = child {
                            children.push(ShapeObject::Picture(pic));
                        }
                    }
                    b"rect" | b"ellipse" | b"line" | b"connectLine" | b"arc" | b"polygon"
                    | b"curve" => {
                        // 자식 그리기 객체
                        let child = parse_shape_object(local, ce, reader)?;
                        if let Control::Shape(shape) = child {
                            children.push(*shape);
                        }
                    }
                    b"container" => {
                        // 중첩 그룹 — 깊이 +1 (상한 초과 시 위에서 거부)
                        let child = parse_container(ce, reader, depth + 1)?;
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
        caption,
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
    field_end_attrs: &mut Vec<(u32, u32)>,
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
                    // 찾아보기 표식 — 책갈피와 같이 8 유닛 자리를 차지한다. 한컴 산출물
                    // 실측(06926 section3 문단 347): 텍스트 342자에 표식 3개인데 lineseg
                    // 최대 `textpos` 가 348 이라, 표식이 자리를 잡지 않으면 범위 밖이 된다.
                    b"indexmark" => {
                        let im = parse_index_mark_element(reader)?;
                        controls.push(Control::IndexMark(im));
                        text_parts.push("\u{0002}".to_string());
                    }
                    b"fieldBegin" => {
                        let ctrl = parse_ctrl_field_begin(ce, reader)?;
                        controls.push(ctrl);
                        // FIELD_BEGIN 제어 문자 추가 (Task #11)
                        text_parts.push("\u{0003}".to_string());
                    }
                    b"fieldEnd" => {
                        // [Task #1556] beginIDRef/fieldid 포착 (고아 fieldEnd 복원용).
                        field_end_attrs.push(parse_field_end_attrs(ce));
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
                    b"pageNumCtrl" => {
                        controls.push(Control::PageNumCtrl(parse_page_num_ctrl_attrs(ce)));
                        text_parts.push("\u{0002}".to_string());
                        skip_element(reader, b"pageNumCtrl")?;
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
                        // [#4677] 책갈피도 HWP5 PARA_TEXT 에서 8 유닛 확장 제어문자 자리를
                        // 차지한다(한컴 원본 바이트: `16 00 6d 6b 6f 62 … 16 00`). 이 표시가
                        // 없으면 char_offsets 에 갭이 없어 HWP5 저장기가 제어문자를 제자리에
                        // 넣지 못하고 문단 **끝에 몰아서** 쓴다. 그러면 글자 모양 경계가
                        // 어긋나고(한컴 pos 77 → rhwp 53) 한글 2022 는 본문을 통째로 버린다.
                        text_parts.push("\u{0002}".to_string());
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
                    b"pageNumCtrl" => {
                        controls.push(Control::PageNumCtrl(parse_page_num_ctrl_attrs(ce)));
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
                        // 위 Start 분기와 같은 이유 — 8 유닛 자리를 반드시 잡아 둔다 (#4677).
                        text_parts.push("\u{0002}".to_string());
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
                        // [Task #1556] 자기닫힘 fieldEnd — beginIDRef/fieldid 포착.
                        field_end_attrs.push(parse_field_end_attrs(ce));
                        text_parts.push("\u{0004}".to_string());
                    }
                    b"hiddenComment" => {}
                    // 키가 하나도 없는 표식은 빈 요소로 온다 — 자리는 똑같이 차지한다.
                    b"indexmark" => {
                        controls.push(Control::IndexMark(IndexMark::default()));
                        text_parts.push("\u{0002}".to_string());
                    }
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

/// `<hp:fieldEnd beginIDRef=".." fieldid="..">` 속성 → (begin_id_ref, field_id) (Task #1556).
fn parse_field_end_attrs(e: &quick_xml::events::BytesStart) -> (u32, u32) {
    let mut begin_id_ref = 0u32;
    let mut field_id = 0u32;
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"beginIDRef" => begin_id_ref = parse_u32(&attr),
            b"fieldid" => field_id = parse_u32(&attr),
            _ => {}
        }
    }
    (begin_id_ref, field_id)
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
                    // [#XXXX] 스펙 표기는 "CIRCLED_DIGIT"(NumberType1). 과거 오탈자
                    // "CIRCLE_DIGIT"로 저장된 한컴 실물 파일과의 호환을 위해 둘 다 인식한다.
                    "CIRCLED_DIGIT" | "CIRCLE_DIGIT" => 1,
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

/// `<hp:indexmark><hp:firstKey>…</hp:firstKey><hp:secondKey>…</hp:secondKey></hp:indexmark>`
///
/// 한컴 실측(06926, 23건)은 `secondKey` 가 비면 요소 자체를 쓰지 않는다.
fn parse_index_mark_element(reader: &mut Reader<&[u8]>) -> Result<IndexMark, HwpxError> {
    let mut im = IndexMark::default();
    let mut cur: Option<&'static str> = None;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                cur = match local_name(e.name().as_ref()) {
                    b"firstKey" => Some("first"),
                    b"secondKey" => Some("second"),
                    _ => None,
                };
            }
            Ok(Event::Text(ref t)) => {
                let v = t.decode().unwrap_or_default().to_string();
                match cur {
                    Some("first") => im.first_key.push_str(&v),
                    Some("second") => im.second_key.push_str(&v),
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = local_name(e.name().as_ref()).to_vec();
                if name == b"indexmark" {
                    break;
                }
                cur = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("indexmark: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(im)
}

/// `<hp:pageNumCtrl pageStartsOn="BOTH|EVEN|ODD"/>` (ParaList XML schema.xml:134)
fn parse_page_num_ctrl_attrs(e: &quick_xml::events::BytesStart) -> PageNumCtrl {
    let mut pnc = PageNumCtrl::default();
    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"pageStartsOn" {
            pnc.page_starts_on =
                PageStartsOn::from_hwpx(&String::from_utf8_lossy(&attr.value).to_uppercase());
        }
    }
    pnc
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
            b"dirty" => {
                // properties bit 15 = 수정됨 표식 — 버리면 clear_initial_field_texts 의
                // 보존 게이트(#3380)가 HWPX 축에서 항상 열려 텍스트가 유실된다 (#3545)
                if parse_bool_attr(&attr) {
                    f.properties |= 1 << 15;
                }
            }
            _ => {}
        }
    }
    // field_id 는 필드별 고유 식별자여야 한다(모델 계약 "문서 내 고유 ID").
    // OWPML `id` 가 필드마다 고유하고, `<hp:fieldEnd beginIDRef>` 가 이 `id` 를
    // 참조하며, 직렬화도 `id="{field_id}"` 로 쓴다. 반면 `fieldid` 는 같은 종류 필드
    // (예: FORMULA 다수)에서 공유될 수 있어, 이를 우선하면 모든 필드가 동일 ID 로
    // 반환된다(#1512). Memo/비-Memo 모두 고유 `id` 우선으로 통일한다.
    f.field_id = id_attr.or(fieldid_attr).unwrap_or(0);
    // [#task-m100] `fieldid` 는 위 field_id 계산에 폴백으로만 쓰였고, `id` 가 존재하는
    // 실물 필드(예: id=1878228493, fieldid=627272811 — 서로 다름)에선 원본 fieldid 값이
    // 그대로 버려져 직렬화기가 이 속성을 영구히 방출하지 못했다. instance_id 로 별도 보존.
    f.instance_id = fieldid_attr;
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
        "TOTAL_PAGE" => AutoNumberType::TotalPage,
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
        "SUMMARY" | "SUMMERY" => FieldType::Summary,
        "USER_INFO" | "USERINFO" => FieldType::UserInfo,
        "HYPERLINK" => FieldType::Hyperlink,
        "MEMO" => FieldType::Memo,
        "PRIVATE_INFO" | "PRIVATEINFO" => FieldType::PrivateInfoSecurity,
        // 직렬화기(serializer/hwpx/field.rs)는 TableOfContents 를 "TOC" 로 방출하므로
        // 파서도 이를 받아야 hwpx 왕복에서 차례 필드 타입이 Unknown 으로 유실되지 않는다.
        "TABLE_OF_CONTENTS" | "TABLEOFCONTENTS" | "TOC" => FieldType::TableOfContents,
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
            // [#2716] flag = HWP5 CTRL_FOOTNOTE numberShape(UInt4). 한컴 HWP5/HWPX 쌍
            // (3-09월_교육_통합_2023) 각주/미주 46개 전수 대조에서 바이트 단위로 일치했다.
            // 값이 0 이면 한컴이 속성 자체를 생략하므로 default 0 유지.
            b"flag" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u32>()
                {
                    note.number_shape = v;
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
    for paragraph in &mut note.paragraphs {
        normalize_hwpx_note_line_vpos(paragraph);
    }
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
            // [#2716] flag = HWP5 CTRL_ENDNOTE numberShape(UInt4). footNote 와 동일 계약.
            b"flag" => {
                if let Ok(v) = std::str::from_utf8(&attr.value)
                    .unwrap_or("")
                    .parse::<u32>()
                {
                    note.number_shape = v;
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
    for paragraph in &mut note.paragraphs {
        normalize_hwpx_note_line_vpos(paragraph);
    }
    Ok(Control::Endnote(Box::new(note)))
}

fn normalize_hwpx_note_line_vpos(paragraph: &mut Paragraph) {
    if paragraph.line_segs.len() <= 1 {
        return;
    }

    let mut expected_vpos = None;
    for line_seg in &mut paragraph.line_segs {
        if let Some(expected) = expected_vpos {
            if line_seg.vertical_pos == 0 && expected > 0 {
                // HWPX 미주/각주 내부에는 실제 단/쪽 리셋이 아닌 후속 줄
                // vpos=0이 저장되는 사례가 있다. 본문 의미는 유지하고,
                // note 내부 연속줄만 이전 줄 advance 기준으로 복원한다.
                line_seg.vertical_pos = expected;
            }
        }

        expected_vpos = Some(
            line_seg
                .vertical_pos
                .saturating_add(line_seg.line_height)
                .saturating_add(line_seg.line_spacing),
        );
    }
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
                                    // [#2957] 실제 한컴 스펙 표기는 "CIRCLED_DIGIT" (pageNum
                                    // formatType 의 "CIRCLE_DIGIT" 와 다름). 구값도 겸용 인식.
                                    "CIRCLE_DIGIT" | "CIRCLED_DIGIT" => 1,
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
                    parse_field_parameters(ce, reader, &mut f)?;
                } else if local == b"subList" && f.field_type == FieldType::Memo {
                    for attr in ce.attributes().flatten() {
                        if attr.key.as_ref() == b"textDirection" {
                            let dir = attr_str(&attr);
                            if dir != "HORIZONTAL" {
                                f.memo_text_direction = Some(dir);
                            }
                        }
                    }
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
/// XML 텍스트/속성값 이스케이프 (#1391 parameters verbatim 재조립용).
fn escape_xml_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            // XML 1.0 허용 문자: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
            // 그 외(제어문자 등)는 제거 — 재조립된 문자열이 그대로 저장돼 불법 XML 이 되지 않도록 (#3382 계열)
            '\u{09}' | '\u{0A}' | '\u{0D}' => out.push(c),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                out.push(c)
            }
            _ => {} // XML 무효 문자 제거
        }
    }
    out
}

/// `parse_field_parameters` 트리 빌더의 스택 프레임 — 열린 파라미터 요소 하나.
/// `listParam`/루트 `parameters` 는 `List`, 나머지 4종은 스칼라 텍스트를 누적한다.
enum ParamFrame {
    List {
        name: Option<String>,
        items: Vec<Parameter>,
    },
    Boolean {
        name: Option<String>,
        text: String,
    },
    Integer {
        name: Option<String>,
        text: String,
    },
    Float {
        name: Option<String>,
        text: String,
    },
    String {
        name: Option<String>,
        text: String,
        preserve_space: bool,
    },
}

impl ParamFrame {
    fn push_text(&mut self, s: &str) {
        match self {
            ParamFrame::Boolean { text, .. }
            | ParamFrame::Integer { text, .. }
            | ParamFrame::Float { text, .. }
            | ParamFrame::String { text, .. } => text.push_str(s),
            ParamFrame::List { .. } => {}
        }
    }

    /// 프레임을 닫아 `Parameter` 로 만든다. 루트 프레임(List)은 호출부가 별도로
    /// `ParameterList` 로 직접 소비하므로 이 경로를 타지 않는다.
    fn finish(self) -> Parameter {
        match self {
            ParamFrame::List { name, items } => Parameter::List(ParameterList { name, items }),
            ParamFrame::Boolean { name, text } => Parameter::Boolean {
                name,
                value: matches!(text.trim(), "1" | "true"),
            },
            ParamFrame::Integer { name, text } => Parameter::Integer {
                name,
                value: text.trim().parse::<i64>().unwrap_or(0),
            },
            ParamFrame::Float { name, text } => Parameter::Float {
                name,
                value: text.trim().parse::<f32>().unwrap_or(0.0),
            },
            ParamFrame::String {
                name,
                text,
                preserve_space,
            } => Parameter::String {
                name,
                value: text,
                preserve_space,
            },
        }
    }
}

/// 파라미터 요소(local name)를 여는 프레임으로 변환한다. 5종 외에는 `None`
/// (스키마 밖 요소 — 원문 보존에는 영향 없이 트리에서만 건너뛴다).
fn open_param_frame<'a>(
    local: &[u8],
    attrs: impl Iterator<Item = quick_xml::events::attributes::Attribute<'a>>,
) -> Option<ParamFrame> {
    let mut name: Option<String> = None;
    let mut preserve_space = false;
    for attr in attrs {
        match attr.key.as_ref() {
            b"name" => name = Some(attr_str(&attr)),
            b"xml:space" if attr_str(&attr) == "preserve" => preserve_space = true,
            _ => {}
        }
    }
    match local {
        b"booleanParam" => Some(ParamFrame::Boolean {
            name,
            text: String::new(),
        }),
        b"integerParam" => Some(ParamFrame::Integer {
            name,
            text: String::new(),
        }),
        b"floatParam" => Some(ParamFrame::Float {
            name,
            text: String::new(),
        }),
        b"stringParam" => Some(ParamFrame::String {
            name,
            text: String::new(),
            preserve_space,
        }),
        b"listParam" => Some(ParamFrame::List {
            name,
            items: Vec::new(),
        }),
        _ => None,
    }
}

fn parse_field_parameters(
    start: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    field: &mut Field,
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    let mut in_command = false;
    let mut in_memo_number = false;

    // [#1391] parameters 요소 원문 verbatim 재조립 — 순수 HWPX 왕복(포맷을 안 벗어남)
    // 은 이 문자열을 그대로 재사용해 바이트 정확성을 보장한다(diff_documents 계약).
    // parameters 자식은 stringParam/integerParam(name 속성 + 텍스트)만으로
    // 단순하므로 이벤트 재방출이 안전하다.
    let mut raw = String::from("<hp:parameters");
    for attr in start.attributes().flatten() {
        raw.push(' ');
        raw.push_str(&String::from_utf8_lossy(attr.key.as_ref()));
        raw.push_str("=\"");
        raw.push_str(&escape_xml_text(&attr_str(&attr)));
        raw.push('"');
    }
    raw.push('>');

    // [#4396] 병행해서 트리도 만든다 — HWP5 왕복(포맷을 벗어남)에서 `raw_parameters_xml`
    // 이 무효화된 뒤에도 Prop/Direction/Path/Category 등이 Command 하나로 축소되지
    // 않도록. 루트(parameters) 프레임을 스택 바닥에 미리 얹어둔다.
    let root_name = start
        .attributes()
        .flatten()
        .find(|a| a.key.as_ref() == b"name")
        .map(|a| attr_str(&a));
    let mut stack: Vec<ParamFrame> = vec![ParamFrame::List {
        name: root_name,
        items: Vec::new(),
    }];

    // 현재 열린 파라미터 요소 태그(닫을 때 사용).
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                let tag = String::from_utf8_lossy(cname.as_ref()).to_string();
                raw.push('<');
                raw.push_str(&tag);
                for attr in ce.attributes().flatten() {
                    raw.push(' ');
                    raw.push_str(&String::from_utf8_lossy(attr.key.as_ref()));
                    raw.push_str("=\"");
                    raw.push_str(&escape_xml_text(&attr_str(&attr)));
                    raw.push('"');
                }
                raw.push('>');
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
                if let Some(frame) = open_param_frame(local, ce.attributes().flatten()) {
                    stack.push(frame);
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                raw.push('<');
                raw.push_str(&String::from_utf8_lossy(cname.as_ref()));
                for attr in ce.attributes().flatten() {
                    raw.push(' ');
                    raw.push_str(&String::from_utf8_lossy(attr.key.as_ref()));
                    raw.push_str("=\"");
                    raw.push_str(&escape_xml_text(&attr_str(&attr)));
                    raw.push('"');
                }
                raw.push_str("/>");
                if local == b"stringParam" {
                    for attr in ce.attributes().flatten() {
                        if attr.key.as_ref() == b"name" && attr_str(&attr) == "Command" {
                            field.command.clear();
                        }
                    }
                }
                // 자기닫힘(빈 값) — 여닫 없이 즉시 부모에 붙인다.
                if let Some(frame) = open_param_frame(local, ce.attributes().flatten()) {
                    if let Some(ParamFrame::List { items, .. }) = stack.last_mut() {
                        items.push(frame.finish());
                    }
                }
            }
            Ok(Event::Text(ref t)) => {
                let decoded = t.decode().unwrap_or_default();
                raw.push_str(&escape_xml_text(&decoded));
                if in_command {
                    field.command.push_str(&decoded);
                } else if in_memo_number {
                    if let Ok(value) = decoded.trim().parse::<u32>() {
                        field.memo_index = value;
                    }
                }
                if let Some(frame) = stack.last_mut() {
                    frame.push_text(&decoded);
                }
            }
            Ok(Event::GeneralRef(ref r)) => {
                let decoded = decode_xml_general_ref(r);
                raw.push_str(&escape_xml_text(&decoded));
                if in_command {
                    field.command.push_str(&decoded);
                }
                if let Some(frame) = stack.last_mut() {
                    frame.push_text(&decoded);
                }
            }
            // [CDATA] stringParam(Command)이 CDATA로 인코딩된 경우(예: 하이퍼링크 URL의
            // 쿼리스트링 `&`, 수식 필드의 비교연산자 `<`/`>`) 처리하지 않으면 필드 명령
            // 문자열이 소실된다. #2916/#2927의 hp:script CDATA 누락과 동일한 패턴.
            Ok(Event::CData(ref cdata)) => {
                let decoded = String::from_utf8_lossy(cdata.as_ref()).into_owned();
                raw.push_str(&escape_xml_text(&decoded));
                if in_command {
                    field.command.push_str(&decoded);
                }
                if let Some(frame) = stack.last_mut() {
                    frame.push_text(&decoded);
                }
            }
            Ok(Event::End(ref ee)) => {
                let eename = ee.name();
                let local = local_name(eename.as_ref());
                if local == b"parameters" {
                    raw.push_str("</hp:parameters>");
                    // 루트 프레임을 팝해 최종 트리로 확정한다.
                    if let Some(ParamFrame::List { name, items }) = stack.pop() {
                        field.parameters = ParameterList { name, items };
                    }
                    break;
                }
                // 임의 깊이 중첩(listParam 안의 stringParam 등)에서도 균형 잡힌 XML 을
                // 재조립하도록, 단일 open_param 추적 대신 End 이벤트 자신의 정규화 이름으로 닫는다.
                // 종전엔 open_param 이 마지막 Start 로 덮여, 바깥 태그의 닫는 태그가 누락됐다.
                let qn = String::from_utf8_lossy(eename.as_ref());
                raw.push_str("</");
                raw.push_str(&qn);
                raw.push('>');
                if local == b"stringParam" {
                    in_command = false;
                } else if local == b"integerParam" {
                    in_memo_number = false;
                }
                // 스키마 5종 중 하나를 닫는 End 라면 스택에서 팝해 부모 List 에 붙인다.
                // (스키마 밖 요소는 애초에 push 되지 않았으므로 이 조건이 걸리지 않는다.)
                if matches!(
                    local,
                    b"booleanParam"
                        | b"integerParam"
                        | b"floatParam"
                        | b"stringParam"
                        | b"listParam"
                ) {
                    if let Some(frame) = stack.pop() {
                        let param = frame.finish();
                        if let Some(ParamFrame::List { items, .. }) = stack.last_mut() {
                            items.push(param);
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("parameters: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    field.raw_parameters_xml = Some(raw);
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
            // [CDATA] composeText(글자겹치기) 본문이 CDATA로 인코딩된 경우 처리하지
            // 않으면 겹침 텍스트가 소실된다. #2916/#2935/#2951의 CDATA 누락과 동일한 패턴.
            Ok(Event::CData(ref cdata)) => {
                text.push_str(&String::from_utf8_lossy(cdata.as_ref()));
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
    // 요소 속성 (#1587 — posType/align 분리 보존 + szRatio/option/styleIDRef)
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"posType" => {
                ruby.pos_type = match attr_str(&attr).as_str() {
                    "TOP" => 0,
                    "BOTTOM" => 1,
                    _ => 0,
                };
            }
            b"align" => {
                ruby.align = match attr_str(&attr).as_str() {
                    "LEFT" => 0,
                    "RIGHT" => 1,
                    "CENTER" => 2,
                    _ => 0,
                };
            }
            b"szRatio" => {
                ruby.sz_ratio = attr_str(&attr).parse().unwrap_or(0);
            }
            b"option" => {
                ruby.option = attr_str(&attr).parse().unwrap_or(0);
            }
            b"styleIDRef" => {
                ruby.style_id_ref = attr_str(&attr).parse().unwrap_or(0);
            }
            _ => {}
        }
    }
    // 자식 요소 파싱 (mainText 기준 텍스트 + subText 덧말)
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                if local == b"subText" {
                    ruby.ruby_text = read_dutmal_text(reader, b"subText")?;
                } else if local == b"mainText" {
                    // [#1587] mainText(기준 텍스트)는 para.text 에 포함되지 않으므로
                    // 모델에 보존한다(종전 skip → 손실 → 직렬화 시 복원 불가였음).
                    ruby.main_text = read_dutmal_text(reader, b"mainText")?;
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
            // [CDATA] dutmal(덧말)의 mainText/subText가 CDATA로 인코딩된 경우 처리하지
            // 않으면 덧말 텍스트가 소실된다. #2916/#2935의 hp:script/stringParam CDATA
            // 누락과 동일한 패턴.
            Ok(Event::CData(ref cdata)) => {
                text.push_str(&String::from_utf8_lossy(cdata.as_ref()));
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
/// 수식 속성(version, baseLine, textColor, baseUnit, lineMode, font)과
/// `<hp:script>` 하위 요소에서 수식 스크립트를 추출하여 `Control::Equation`을 생성한다.
fn parse_equation(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
) -> Result<Control, HwpxError> {
    let mut common = CommonObjAttr::default();
    let mut shape_attr = ShapeComponentAttr::default();
    let mut has_pos = false;

    // 수식 전용 속성 — 초기값은 OWPML(ParaList 스키마 EquationType) 속성 기본값.
    // 속성이 생략된 파일에서 zero-계열 값으로 복원하면 직렬화기가 세 속성을
    // 무조건 방출하므로 라운드트립 시 version=""/baseLine="0"/font="" 으로 변형된다.
    let mut version_info = String::from("Equation Version 60");
    let mut baseline: i16 = 85;
    let mut color: u32 = 0;
    let mut font_size: u32 = 1000;
    let mut font_name = String::from("HYhwpEQ");
    // [#2727] lineMode(수식이 차지하는 범위) → EQEDIT attribute bit0.
    // OWPML 기본값은 CHAR 이므로 속성이 없으면 0(글자 단위)으로 둔다.
    // `attr`/`eqedit` 두 필드가 동일한 값을 보관하므로 함께 채운다.
    let mut eq_attr: u32 = 0;
    let mut eqedit: u32 = 0;

    // 공통 개체 속성 + 수식 속성 파싱
    parse_object_element_attrs(e, &mut common, &mut shape_attr);
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"version" => version_info = attr_str(&attr),
            b"baseLine" => baseline = attr_str(&attr).parse().unwrap_or(85),
            b"textColor" => color = parse_color(&attr),
            b"baseUnit" => font_size = parse_u32(&attr),
            // [#2727] LINE 이면 bit0 set. 종전엔 미파싱으로 왕복 시 CHAR 로 고정됐다.
            // `attr`/`eqedit` 두 필드가 동일한 값을 보관하므로 함께 채운다.
            b"lineMode" => {
                if attr_str(&attr).eq_ignore_ascii_case("LINE") {
                    eq_attr |= EQUATION_LINE_MODE_BIT;
                } else {
                    eq_attr &= !EQUATION_LINE_MODE_BIT;
                }
                eqedit = eq_attr;
            }
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
                    // 수식 설명 (#1392) — 미적재 시 roundtrip 에서 소실
                    b"shapeComment" => {
                        common.description = read_dutmal_text(reader, b"shapeComment")?;
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
            Ok(Event::CData(ref cdata)) => {
                // #2916: 수식 스크립트가 CDATA 로 저장된 경우(비교 연산자 등 XML
                // 예약 문자를 다량 포함해 엔티티 이스케이프 대신 CDATA 로 감싸는
                // 케이스), 이 분기가 없으면 script 가 통째로 빈 문자열이 된다.
                if in_script {
                    script.push_str(&String::from_utf8_lossy(cdata.as_ref()));
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
        // [#2727] HWPX lineMode → EQEDIT attribute bit0
        attr: eq_attr,
        script,
        font_size,
        color,
        baseline,
        unknown: 0,
        eqedit,
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
            // [#1382] \u{0012}(AUTO_NUMBER) 포함 — placeholder 공백을 포함해 8유닛
            // (offsets 조립 루프와 동일 축). 종전 `_` 분기(1유닛)로 빠져 char_shapes
            // 경계가 offsets 축과 어긋났다 (143E 각주 run 경계 2 → 정답 9).
            "\u{0002}" | "\u{0003}" | "\u{0004}" | "\u{0012}" => 8,
            TITLE_MARK_PART_IGNORE | TITLE_MARK_PART_KEEP => 8,
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
            // [Task #TBD] value 는 UNCHECKED/CHECKED/INDETERMINATE 3상태 열거형
            // (OWPML AbstractButtonObjectType). INDETERMINATE 를 UNCHECKED 로
            // 뭉개면 라운드트립 시 tri-state 체크박스의 중간 상태가 유실된다.
            b"value" => {
                form.value = match attr_str(&attr).as_str() {
                    "CHECKED" => 1,
                    "INDETERMINATE" => 2,
                    _ => 0,
                }
            }
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
            // 버튼류 전용 속성 (라운드트립 보존; writer 가 동일 키로 읽음)
            b"radioGroupName" => {
                form.properties
                    .insert("RadioGroupName".to_string(), attr_str(&attr));
            }
            b"triState" => {
                form.properties
                    .insert("TriState".to_string(), attr_str(&attr));
            }
            b"backStyle" => {
                form.properties
                    .insert("BackStyle".to_string(), attr_str(&attr));
            }
            // Edit 전용 속성 (라운드트립 보존)
            b"multiLine" => {
                form.properties
                    .insert("MultiLine".to_string(), attr_str(&attr));
            }
            b"passwordChar" => {
                form.properties
                    .insert("PasswordChar".to_string(), attr_str(&attr));
            }
            b"maxLength" => {
                form.properties
                    .insert("MaxLength".to_string(), attr_str(&attr));
            }
            b"scrollBars" => {
                form.properties
                    .insert("ScrollBars".to_string(), attr_str(&attr));
            }
            b"tabKeyBehavior" => {
                form.properties
                    .insert("TabKeyBehavior".to_string(), attr_str(&attr));
            }
            b"numOnly" => {
                form.properties
                    .insert("Number".to_string(), attr_str(&attr));
            }
            b"readOnly" => {
                form.properties
                    .insert("ReadOnly".to_string(), attr_str(&attr));
            }
            b"alignText" => {
                form.properties
                    .insert("AlignText".to_string(), attr_str(&attr));
            }
            _ => {}
        }
    }

    // 자식 요소 순회
    let end_tag = local_name(e.name().as_ref()).to_vec();
    let mut buf = Vec::new();
    // (value, displayText) 쌍으로 보존 — comboBox 항목
    let mut list_items: Vec<(String, String)> = Vec::new();

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
                                // 양식 개체(edit 컨트롤) 텍스트의 CDATA 저장 형태.
                                // #2916 과 같은 결함 클래스 — 없으면 form.text 가 빈다.
                                Ok(Event::CData(ref cdata)) => {
                                    form.text.push_str(&String::from_utf8_lossy(cdata.as_ref()));
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
                        // <hp:sz width="..." widthRelTo="..." height="..." heightRelTo="..." protect="..."/>
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => form.width = parse_u32(&attr),
                                b"height" => form.height = parse_u32(&attr),
                                b"widthRelTo" => {
                                    form.properties
                                        .insert("SzWidthRelTo".to_string(), attr_str(&attr));
                                }
                                b"heightRelTo" => {
                                    form.properties
                                        .insert("SzHeightRelTo".to_string(), attr_str(&attr));
                                }
                                b"protect" => {
                                    form.properties
                                        .insert("SzProtect".to_string(), attr_str(&attr));
                                }
                                _ => {}
                            }
                        }
                    }
                    b"pos" => {
                        // <hp:pos .../> 앵커링 (표준 ShapePositionType 11속성) — 라운드트립 보존
                        for attr in ce.attributes().flatten() {
                            let key = match attr.key.as_ref() {
                                b"treatAsChar" => "PosTreatAsChar",
                                b"affectLSpacing" => "PosAffectLSpacing",
                                b"flowWithText" => "PosFlowWithText",
                                b"allowOverlap" => "PosAllowOverlap",
                                b"holdAnchorAndSO" => "PosHoldAnchorAndSO",
                                b"vertRelTo" => "PosVertRelTo",
                                b"horzRelTo" => "PosHorzRelTo",
                                b"vertAlign" => "PosVertAlign",
                                b"horzAlign" => "PosHorzAlign",
                                b"vertOffset" => "PosVertOffset",
                                b"horzOffset" => "PosHorzOffset",
                                _ => continue,
                            };
                            form.properties.insert(key.to_string(), attr_str(&attr));
                        }
                    }
                    b"outMargin" => {
                        // <hp:outMargin left=".." right=".." top=".." bottom=".."/> — 라운드트립 보존
                        for attr in ce.attributes().flatten() {
                            let key = match attr.key.as_ref() {
                                b"left" => "OutMarginLeft",
                                b"right" => "OutMarginRight",
                                b"top" => "OutMarginTop",
                                b"bottom" => "OutMarginBottom",
                                _ => continue,
                            };
                            form.properties.insert(key.to_string(), attr_str(&attr));
                        }
                    }
                    b"listItem" => {
                        // <hp:listItem displayText="..." value="..."/> (comboBox 항목)
                        let mut value = String::new();
                        let mut display = String::new();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"value" => value = attr_str(&attr),
                                b"displayText" => display = attr_str(&attr),
                                _ => {}
                            }
                        }
                        list_items.push((value, display));
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

    // comboBox 항목 목록(값 + 표시 텍스트)을 properties에 저장
    if !list_items.is_empty() {
        for (i, (value, display)) in list_items.iter().enumerate() {
            form.properties
                .insert(format!("listItem{}", i), value.clone());
            form.properties
                .insert(format!("listItemDisplay{}", i), display.clone());
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
    // [#3546] 차트가 있으면 <hp:default> 의 fallback OLE 를 버리지 않고 차트에
    // 매달아 보존한다 — 저장 시 원형 <hp:switch>/<hp:case>/<hp:default> 구조
    // 재방출의 재료다(종전에는 fallback 이 소실되어 hp:ole 단독으로 되쓰였다).
    match (chart_ctrl, ole_ctrl) {
        (Some(mut chart), Some(ole)) => {
            if let (Control::Shape(chart_shape), Control::Shape(ole_shape)) = (&mut chart, ole) {
                if let (ShapeObject::Ole(chart_ole), ShapeObject::Ole(fallback)) =
                    (chart_shape.as_mut(), *ole_shape)
                {
                    chart_ole.chart_switch_fallback = Some(fallback);
                }
            }
            Ok(Some(chart))
        }
        (chart, ole) => Ok(chart.or(ole)),
    }
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
    let mut chart_id_ref: Option<String> = None;
    let mut id_attr: u32 = 0;
    let mut numbering_type_picture = false;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            // [#2882] common.numbering_type(ObjectNumberingType) 도 함께 채운다.
            // 직렬화기(numbering_type_str, serializer/hwpx/shape.rs)가 참조하는
            // 필드는 이것뿐이라, bool 지역 변수만으로는 저장 시 항상 NONE 으로
            // 되쓰인다(공용 도형 파서 section.rs:2892 와 동일 패턴으로 맞춤).
            b"numberingType" => {
                numbering_type_picture = attr_str(&attr).eq_ignore_ascii_case("PICTURE");
                common.numbering_type = match attr_str(&attr).to_ascii_uppercase().as_str() {
                    "PICTURE" => crate::model::shape::ObjectNumberingType::Picture,
                    "TABLE" => crate::model::shape::ObjectNumberingType::Table,
                    "EQUATION" => crate::model::shape::ObjectNumberingType::Equation,
                    _ => crate::model::shape::ObjectNumberingType::None,
                };
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
                // [#3546] 원문을 보존한다 — 저장 시 hp:chart 원형 재방출의 표식.
                chart_id_ref = Some(s);
            }
            // [#3546] 실물 hp:chart 는 instid 없이 id 만 기록한다 — 미파싱이면
            // 재방출 id 가 항상 "0" 으로 되쓰인다. instid 가 있으면 그쪽이 우선
            // (아래 arm 이 뒤에서 덮는 것이 아니라 후처리에서 판정).
            b"id" => id_attr = parse_u32(&attr),
            b"instid" => common.instance_id = parse_u32(&attr),
            // [#2931] 개체 잠금(lock) — 종전 미파싱으로 직렬화 시 항상 "0"으로
            // 되돌아가 차트 개체의 잠금 상태가 유실됐다.
            b"lock" => common.locked = attr_str(&attr) == "1",
            _ => {}
        }
    }
    if common.instance_id == 0 {
        common.instance_id = id_attr;
    }

    let mut extent: Option<(i32, i32)> = None;
    let mut shape_attr = ShapeComponentAttr::default();
    let mut caption: Option<crate::model::shape::Caption> = None;
    parse_common_shape_children(
        reader,
        &mut common,
        b"chart",
        &mut extent,
        &mut shape_attr,
        &mut caption,
    )?;
    if numbering_type_picture {
        common.hwp5_gen_shape_attr_bit28 = true;
    }
    common.attr = pack_hwpx_common_obj_attr(&common);

    if chart_num == 0 {
        return Ok(None);
    }

    let mut ole = OleShape::default();
    ole.common = common;
    ole.drawing.shape_attr = shape_attr;
    ole.bin_data_id = 60000u32 + chart_num as u32;
    ole.chart_id_ref = chart_id_ref;
    // <hc:extent> 가 있으면 원본 개체 크기를 보존한다(없으면 종전 기본값 7200).
    let (ext_x, ext_y) = extent.unwrap_or((7200, 7200));
    ole.extent_x = if ext_x > 0 { ext_x } else { 7200 };
    ole.extent_y = if ext_y > 0 { ext_y } else { 7200 };
    apply_hwpx_ole_shape_component_contract(&mut ole);
    // [#4319] HWP5 파서(parser/control/shape.rs:213)와 동형 정규화 — drawing.caption
    // 에 남기지 않고 OleShape 자신의 caption 필드로 옮긴다. 게이트(shape_caption,
    // serializer/hwpx/roundtrip.rs)는 `x.caption` 만 보므로 정규화하지 않으면
    // drawing.caption 잔류가 라운드트립 비교에서 보이지 않는다.
    ole.drawing.caption = caption;
    ole.caption = ole.drawing.caption.take();
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

    use crate::model::shape::OleDrawingAspect;

    let mut common = CommonObjAttr::default();
    common.hwp5_gen_shape_attr_bit26 = true;
    let mut bin_id: u32 = 0;
    let mut numbering_type_picture = false;
    let mut draw_aspect = OleDrawingAspect::default();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            // [#2882] common.numbering_type(ObjectNumberingType) 도 함께 채운다.
            // 직렬화기(numbering_type_str, serializer/hwpx/shape.rs)가 참조하는
            // 필드는 이것뿐이라, bool 지역 변수만으로는 저장 시 항상 NONE 으로
            // 되쓰인다(공용 도형 파서 section.rs:2892 와 동일 패턴으로 맞춤).
            b"numberingType" => {
                numbering_type_picture = attr_str(&attr).eq_ignore_ascii_case("PICTURE");
                common.numbering_type = match attr_str(&attr).to_ascii_uppercase().as_str() {
                    "PICTURE" => crate::model::shape::ObjectNumberingType::Picture,
                    "TABLE" => crate::model::shape::ObjectNumberingType::Table,
                    "EQUATION" => crate::model::shape::ObjectNumberingType::Equation,
                    _ => crate::model::shape::ObjectNumberingType::None,
                };
            }
            // 표시 방식(아이콘/썸네일/인쇄용/내용). serializer 는 방출하나 종전엔
            // 파서가 읽지 않아 ICON 등이 왕복 시 CONTENT 로 바뀌었다.
            b"drawAspect" => {
                draw_aspect = match attr_str(&attr).as_str() {
                    "ICON" => OleDrawingAspect::Icon,
                    "THUMBNAIL" => OleDrawingAspect::Thumbnail,
                    "DOCPRINT" => OleDrawingAspect::DocPrint,
                    _ => OleDrawingAspect::Content,
                };
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
            // [#2931] 개체 잠금(lock) — 종전 미파싱으로 직렬화 시 항상 "0"으로
            // 되돌아가 OLE 개체의 잠금 상태가 유실됐다.
            b"lock" => common.locked = attr_str(&attr) == "1",
            _ => {}
        }
    }

    let mut extent: Option<(i32, i32)> = None;
    let mut shape_attr = ShapeComponentAttr::default();
    let mut caption: Option<crate::model::shape::Caption> = None;
    parse_common_shape_children(
        reader,
        &mut common,
        b"ole",
        &mut extent,
        &mut shape_attr,
        &mut caption,
    )?;
    if numbering_type_picture {
        common.hwp5_gen_shape_attr_bit28 = true;
    }
    common.attr = pack_hwpx_common_obj_attr(&common);

    let mut ole = OleShape::default();
    ole.common = common;
    ole.drawing.shape_attr = shape_attr;
    ole.bin_data_id = bin_id;
    ole.drawing_aspect = draw_aspect;
    // <hc:extent> 가 있으면 원본 개체 크기를 보존한다(없으면 종전 기본값 7200).
    let (ext_x, ext_y) = extent.unwrap_or((7200, 7200));
    ole.extent_x = if ext_x > 0 { ext_x } else { 7200 };
    ole.extent_y = if ext_y > 0 { ext_y } else { 7200 };
    apply_hwpx_ole_shape_component_contract(&mut ole);
    // [#4319] HWP5 파서(parser/control/shape.rs:222)와 동형 정규화 — 차트와 동일한
    // 이유로 drawing.caption 이 아니라 ole.caption 에 남겨야 게이트가 검출한다.
    ole.drawing.caption = caption;
    ole.caption = ole.drawing.caption.take();
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
    // OLE 전용 `<hc:extent>`(원본 개체 크기) 수집용. 호출자(ole/chart)만 사용한다.
    // 종전엔 이 자식을 무시하고 호출자가 7200 을 하드코딩해 개체 크기가 유실됐다.
    extent_out: &mut Option<(i32, i32)>,
    // [#3546] `<hp:rotationInfo>` 수집용. 종전 미파싱으로 저장 시 기본값으로
    // 되쓰여 rotateimage="1" 등 원본 값이 뒤집혔다(#2726 sz 기준 유실과 동형).
    shape_attr_out: &mut ShapeComponentAttr,
    // [#4319] `<hp:caption>` 수집용. 종전엔 이 공용 자식 파서(차트·OLE 전용)에
    // caption arm 이 없어 캡션 subList 가 파싱 단계에서 완전히 유실됐다 —
    // 도형(parse_shape_object)·묶음(parse_container)·그림(parse_picture) 은 모두
    // 캡션을 읽지만 차트·OLE 만 빠져 있었다. HWP5 파서(parser/control/shape.rs:213,
    // 222)와 동형으로 drawing.caption 에 채운 뒤 호출자가 `.caption` 으로 정규화한다.
    caption_out: &mut Option<crate::model::shape::Caption>,
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) | Ok(Event::Empty(ref ce)) => {
                let cname = ce.name();
                let local = local_name(cname.as_ref());
                match local {
                    b"extent" => {
                        let mut x = 0i32;
                        let mut y = 0i32;
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"x" => x = parse_i32(&attr),
                                b"y" => y = parse_i32(&attr),
                                _ => {}
                            }
                        }
                        *extent_out = Some((x, y));
                    }
                    b"rotationInfo" => {
                        parse_shape_rotation_info(ce, shape_attr_out);
                    }
                    b"sz" => {
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"width" => common.width = parse_u32(&attr),
                                b"height" => common.height = parse_u32(&attr),
                                // [#2726] 공용 자식 파서(차트·OLE)만 크기 기준 arm 이 없어
                                // 파싱 단계에서 유실됐다. 도형 공용 파서(같은 파일 2925/2928)·
                                // 표(1702/1706)·그림(#2712)과 동형이며, 높이는 동일하게
                                // allow_column_para=false 로 읽어 치역을 {Paper, Page,
                                // Absolute} 로 제한한다.
                                b"widthRelTo" => {
                                    common.width_criterion =
                                        parse_size_criterion(&attr_str(&attr), true);
                                }
                                b"heightRelTo" => {
                                    common.height_criterion =
                                        parse_size_criterion(&attr_str(&attr), false);
                                }
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
                                // [버그 수정] chart/OLE 공용 <hp:pos> 파서만 유일하게 `parse_u32`
                                // 를 써서 음수 오프셋(왼쪽/위쪽 앵커 이탈)을 0 으로 뭉갰다 —
                                // 이미지·표 등 다른 개체 <hp:pos> 파서(위 parse_i32_wrapping 분기)
                                // 와 동형으로 맞춘다.
                                b"vertOffset" => {
                                    common.vertical_offset = parse_i32_wrapping(&attr) as u32
                                }
                                b"horzOffset" => {
                                    common.horizontal_offset = parse_i32_wrapping(&attr) as u32
                                }
                                b"treatAsChar" => common.treat_as_char = parse_bool(&attr),
                                // [#2784] affectLSpacing(줄 간격에 영향) — 공통 개체 pos 되읽기.
                                b"affectLSpacing" => common.affect_line_spacing = parse_bool(&attr),
                                b"flowWithText" => common.flow_with_text = parse_bool(&attr),
                                b"allowOverlap" => common.allow_overlap = parse_bool(&attr),
                                // holdAnchorAndSO(쪽나눔 방지). 방출측은 모든 개체에 내지만
                                // 종전엔 표 파서만 되읽어, 그림/도형/차트/OLE 는 prevent_page_break
                                // 이 0 으로 유실됐다(표 파서와 동형으로 보강).
                                b"holdAnchorAndSO" => {
                                    common.prevent_page_break =
                                        if parse_bool(&attr) { 1 } else { 0 };
                                }
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
                    // 개체 설명문(대체 텍스트) — 방출측(write_shape_comment)은 OLE/차트에도
                    // <hp:shapeComment>를 쓰지만 이 공용 자식 파서에 arm 이 없어 되읽지
                    // 못하고 유실됐다(OLE 라운드트립 ir-diff 로 실측: HWP5→HWPX→재파싱 후
                    // shape comment 사라짐).
                    b"shapeComment" => {
                        common.description = read_dutmal_text(reader, b"shapeComment")?;
                    }
                    // [#4319] 캡션 — 미적재 시 라운드트립에서 캡션 subList 소실(다른
                    // 도형 변형과 동형, parse_shape_object/parse_container 참고).
                    b"caption" => {
                        *caption_out = Some(parse_caption(ce, reader)?);
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

    // ---------- [#4759] 문단↔표↔셀 상호재귀 무한 중첩 → 스택 오버플로 DoS 가드 ----------

    fn nested_table_section_xml(depth: usize) -> String {
        // 각 겹 <hp:tbl><hp:tr><hp:tc><hp:p> 가 문단→표→셀→문단 재귀를 한 단계 판다.
        let mut xml = String::from(
            r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"><hp:p paraPrIDRef="0" styleIDRef="0">"#,
        );
        for _ in 0..depth {
            xml.push_str("<hp:tbl><hp:tr><hp:tc><hp:p>");
        }
        for _ in 0..depth {
            xml.push_str("</hp:p></hp:tc></hp:tr></hp:tbl>");
        }
        xml.push_str("</hp:p></hs:sec>");
        xml
    }

    #[test]
    fn table_nesting_beyond_limit_is_rejected() {
        // 상한을 넘는 표 중첩(문단↔표↔셀 사이클)은 스택을 고갈시키기 전에 오류로
        // 거부돼야 한다. 가드가 없으면 이 입력은 파싱돼(Ok) 이 단언이 실패하고, 실파일
        // 규모(수만 겹)에서는 catch_unwind 로도 못 잡는 SIGSEGV 가 난다. 컨테이너 가드와
        // 같은 이유로 넉넉한 스택 전용 스레드에서 경계를 결정적으로 시험한다.
        let rejected = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let xml = nested_table_section_xml(MAX_HWPX_SECTION_DEPTH as usize + 60);
                parse_hwpx_section(&xml).is_err()
            })
            .expect("파서 스레드 생성 실패")
            .join()
            .expect("파서 스레드 패닉");
        assert!(
            rejected,
            "상한 초과 표 중첩이 거부되지 않았다 — 상호재귀 깊이 가드 회귀"
        );
    }

    #[test]
    fn table_nesting_within_limit_still_parses() {
        // 상한 안쪽의 정상적인 표 중첩은 계속 성공해야 한다(가드가 과잉 차단 안 함).
        let xml = nested_table_section_xml(5);
        assert!(
            parse_hwpx_section(&xml).is_ok(),
            "정상 깊이 표 중첩이 거부됐다 — 가드가 과잉 차단"
        );
    }

    // ---------- [#4730] <hp:container> 무한 중첩 → 스택 오버플로 DoS 가드 ----------

    fn nested_container_section_xml(depth: usize) -> String {
        let mut xml = String::from(
            r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"><hp:p paraPrIDRef="0" styleIDRef="0">"#,
        );
        for _ in 0..depth {
            xml.push_str("<hp:container>");
        }
        for _ in 0..depth {
            xml.push_str("</hp:container>");
        }
        xml.push_str("</hp:p></hs:sec>");
        xml
    }

    #[test]
    fn container_nesting_beyond_limit_is_rejected_on_default_stack() {
        // 상한을 넘는 중첩 <hp:container> 는 스택을 고갈시키기 전에 오류로 거부돼야
        // 한다. 가드가 없으면 이 입력은 그대로 파싱돼(Ok) 이 단언이 실패하고, 실파일
        // 규모(수만 겹)에서는 catch_unwind 로도 못 잡는 SIGSEGV 가 난다.
        // 기본 테스트 스레드에서 실행해, 실제 호출자가 흔히 쓰는 스택에서도 가드가
        // 재귀 프레임 고갈보다 먼저 동작함을 검증한다.
        let xml = nested_container_section_xml(MAX_HWPX_CONTAINER_DEPTH as usize + 1);
        let rejected = parse_hwpx_section(&xml).is_err();
        assert!(
            rejected,
            "상한 초과 container 중첩이 거부되지 않았다 — 재귀 깊이 가드 회귀"
        );
    }

    #[test]
    fn container_nesting_at_limit_still_parses() {
        // 상한 안쪽의 정상적인 중첩은 계속 성공해야 한다(가드가 과잉 차단하지 않음).
        let xml = nested_container_section_xml(MAX_HWPX_CONTAINER_DEPTH as usize);
        assert!(
            parse_hwpx_section(&xml).is_ok(),
            "정상 깊이 container 가 거부됐다 — 가드가 과잉 차단"
        );
    }

    // ---------- #2957: autoNumFormat 원 문자(CIRCLED_DIGIT) 인식 ----------

    #[test]
    fn task2957_autonum_format_circled_digit_parses_as_1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:ctrl><hp:autoNum num="1" numType="FOOTNOTE"><hp:autoNumFormat type="CIRCLED_DIGIT" userChar="" prefixChar="" suffixChar="" supscript="0"/></hp:autoNum></hp:ctrl><hp:t> </hp:t></hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        let an = section.paragraphs[0]
            .controls
            .iter()
            .find_map(|c| match c {
                Control::AutoNumber(an) => Some(an),
                _ => None,
            })
            .expect("autoNum 컨트롤이 파싱돼야 함");
        assert_eq!(
            an.format, 1,
            "type=\"CIRCLED_DIGIT\" 는 format=1(circled digit) 로 인식돼야 함(#2957)"
        );
    }

    // ---------- #1382: autoNum 폭 축 일관화 ----------

    #[test]
    fn task1382_calc_counts_autonum_as_8_units() {
        // \u{0012}(AUTO_NUMBER) 는 placeholder 포함 8유닛 — offsets 축과 동일.
        let parts = vec!["\u{0012}".to_string(), " ".to_string()];
        assert_eq!(calc_utf16_len_from_parts(&parts), 9);
    }

    #[test]
    fn task1382_autonum_run_boundary_on_offsets_axis() {
        // 143E 각주 패턴: run1(ctrl autoNum + 공백) + run2(텍스트) →
        // run2 경계는 offsets 축 9 (autoNum 8 + 공백 1). 종전 1유닛 축에서는 2.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="10"><hp:ctrl><hp:autoNum num="1" numType="FOOTNOTE"/></hp:ctrl><hp:t> </hp:t></hp:run>
    <hp:run charPrIDRef="11"><hp:t>본문</hp:t></hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        let p = &section.paragraphs[0];
        assert_eq!(p.text, "  본문", "placeholder 공백 + 실제 공백 + 텍스트");
        assert_eq!(p.char_offsets, vec![0, 8, 9, 10]);
        assert_eq!(
            p.char_shapes
                .iter()
                .map(|c| (c.start_pos, c.char_shape_id))
                .collect::<Vec<_>>(),
            vec![(0, 10), (9, 11)],
            "run2 경계는 offsets 축 9"
        );
    }

    #[test]
    fn task1654_hide_first_empty_line_sets_hwp5_section_flag() {
        // HWPX visibility 값은 HWP 저장 경로가 읽는 SectionDef.flags bit 19와
        // 함께 동기화되어야 한다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr id="" textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" tabStopVal="4000" tabStopUnit="HWPUNIT" outlineShapeIDRef="1" memoShapeIDRef="0" textVerticalWidthHead="0" masterPageCnt="0">
        <hp:visibility hideFirstHeader="0" hideFirstFooter="0" hideFirstMasterPage="0" border="SHOW_ALL" fill="SHOW_ALL" hideFirstPageNum="0" hideFirstEmptyLine="1" showLineNumber="0"/>
      </hp:secPr>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert!(section.section_def.hide_empty_line);
        assert_ne!(section.section_def.flags & 0x0008_0000, 0);

        let Control::SectionDef(section_def) = &section.paragraphs[0].controls[0] else {
            panic!("첫 컨트롤은 SectionDef 여야 함");
        };
        assert!(section_def.hide_empty_line);
        assert_ne!(section_def.flags & 0x0008_0000, 0);
    }

    #[test]
    fn equation_missing_attrs_fall_back_to_owpml_defaults() {
        // OWPML 스키마(ParaList, EquationType)의 속성 기본값:
        //   version  = "Equation Version 60"
        //   baseLine = 85
        //   font     = "HYhwpEQ"
        // 속성이 생략된 수식을 파싱하면 스펙 기본값으로 복원되어야 한다.
        // (직렬화기는 세 속성을 무조건 방출하므로, 파서가 0/"" 로 복원하면
        //  왕복 시 baseLine="0" font="" version="" 으로 값이 변형된다.)
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:equation id="1" zOrder="0" numberingType="EQUATION" textWrap="TOP_AND_BOTTOM" lock="0">
        <hp:script>1 over 2</hp:script>
        <hp:sz width="2000" widthRelTo="ABSOLUTE" height="1000" heightRelTo="ABSOLUTE"/>
        <hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>
      </hp:equation>
    </hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        let eq = section.paragraphs[0]
            .controls
            .iter()
            .find_map(|c| match c {
                Control::Equation(e) => Some(e),
                _ => None,
            })
            .expect("수식 컨트롤");
        assert_eq!(eq.baseline, 85, "baseLine 생략 시 스펙 기본값 85");
        assert_eq!(eq.font_name, "HYhwpEQ", "font 생략 시 스펙 기본값 HYhwpEQ");
        assert_eq!(
            eq.version_info, "Equation Version 60",
            "version 생략 시 스펙 기본값"
        );
    }

    #[test]
    fn task1380_no_linesegarray_keeps_line_segs_empty() {
        // 원본에 <hp:linesegarray> 가 없는 문단은 zero-default 를 주입하지 않고
        // line_segs 를 빈 채 유지한다 (#1380 — 원본 무 → RT 무 대칭의 전제).
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>텍스트 있음</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert!(
            section.paragraphs[0].line_segs.is_empty(),
            "linesegarray 부재 문단에 zero-default 가 주입되면 안 됨: {:?}",
            section.paragraphs[0].line_segs
        );
    }

    #[test]
    fn task1380_linesegarray_values_loaded_as_is() {
        // <hp:linesegarray> 가 있으면 9개 필드를 그대로 적재한다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>한 줄</hp:t>
    </hp:run>
    <hp:linesegarray>
      <hp:lineseg textpos="0" vertpos="15360" vertsize="2197" textheight="2197" baseline="1867" spacing="1098" horzpos="0" horzsize="42520" flags="393216"/>
    </hp:linesegarray>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let segs = &section.paragraphs[0].line_segs;
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].vertical_pos, 15360);
        assert_eq!(segs[0].line_height, 2197);
        assert_eq!(segs[0].tag, 393216);
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
    fn run_text_preserve_cdata() {
        // <hp:t> 본문 런 텍스트가 CDATA 로 저장된 경우, read_text_content_with_tabs 에
        // Event::CData arm 이 없어 `_ => {}` 로 버려지면서 문단 텍스트가 통째로 소실되던
        // 결함. #2916·#2951·#2974 와 같은 결함 클래스이나, 이 경로는 수식·덧말이 아닌
        // 일반 본문이라 영향 범위가 가장 넓다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t><![CDATA[a<b & c]]></hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert_eq!(section.paragraphs.len(), 1);
        assert_eq!(
            section.paragraphs[0].text, "a<b & c",
            "본문 런 텍스트의 CDATA 가 소실되면 안 됨"
        );
    }

    #[test]
    fn form_edit_text_preserve_cdata() {
        // 양식 개체(<hp:edit>)의 <hp:text> 가 CDATA 로 저장된 경우 parse_form_object 의
        // arm 누락으로 form.text 가 비던 결함. 위와 같은 결함 클래스.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:edit id="1" name="edit1">
        <hp:text><![CDATA[a<b]]></hp:text>
      </hp:edit>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Form(form) = &section.paragraphs[0].controls[0] else {
            panic!("첫 컨트롤은 Form(양식 개체)이어야 함");
        };
        assert_eq!(
            form.text, "a<b",
            "양식 개체 텍스트의 CDATA 가 소실되면 안 됨"
        );
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

        assert_eq!(section.section_def.endnote_shape.separator_length, 14692344);
        assert_eq!(
            section
                .section_def
                .endnote_shape
                .separator_above_margin_hu(),
            850,
            "aboveLine은 공식 '구분선 위' 값"
        );
        assert_eq!(
            section
                .section_def
                .endnote_shape
                .separator_below_margin_hu(),
            567,
            "belowLine은 공식 '구분선 아래' 값"
        );
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
        assert_eq!((section.section_def.endnote_shape.attr >> 10) & 0x03, 0);
    }

    /// [#2779] 각주 placement 의 OWPML 정식 토큰 MERGED_COLUMN(통단)·
    /// RIGHT_MOST_COLUMN(가장 오른쪽 단)을 파서가 수용해야 한다. 종전엔 토큰 표에
    /// 없어 `_ => continue` 로 떨어져, 통단/오른쪽단 각주가 파싱 단계에서 기본값
    /// (각 단마다, 코드 0)으로 소실됐다.
    #[test]
    fn issue2779_footnote_placement_accepts_schema_column_tokens() {
        // (placement, attr bits 8-9 코드) 를 돌려준다.
        fn parse_place(place: &str) -> (crate::model::footnote::FootnotePlacement, u32) {
            let xml = format!(
                r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" outlineShapeIDRef="1" masterPageCnt="0">
        <hp:footNotePr>
          <hp:autoNumFormat type="DIGIT" userChar="" prefixChar="" suffixChar=")" supscript="0"/>
          <hp:noteLine length="-1" type="SOLID" width="0.12 mm" color="#000000"/>
          <hp:noteSpacing betweenNotes="283" belowLine="567" aboveLine="850"/>
          <hp:numbering type="CONTINUOUS" newNum="1"/>
          <hp:placement place="{place}" beneathText="0"/>
        </hp:footNotePr>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"##
            );
            let section = parse_hwpx_section(&xml).unwrap();
            let shape = &section.section_def.footnote_shape;
            (shape.placement, (shape.attr >> 8) & 0x03)
        }

        use crate::model::footnote::FootnotePlacement;
        assert_eq!(
            parse_place("MERGED_COLUMN"),
            (FootnotePlacement::BelowText, 1),
            "MERGED_COLUMN(통단으로 배열) = attr bits 8-9 코드 1"
        );
        assert_eq!(
            parse_place("RIGHT_MOST_COLUMN"),
            (FootnotePlacement::RightColumn, 2),
            "RIGHT_MOST_COLUMN(가장 오른쪽 단에 배열) = attr bits 8-9 코드 2"
        );
        // 기본 토큰은 종전대로 코드 0.
        assert_eq!(
            parse_place("EACH_COLUMN"),
            (FootnotePlacement::EachColumn, 0),
            "EACH_COLUMN(각 단마다 따로 배열) = attr bits 8-9 코드 0"
        );
    }

    /// [#2779] secPr@memoShapeIDRef 가 SectionDef.memo_shape_id 로 수집돼야 한다.
    /// 종전엔 파서가 속성을 읽지 않아 저장 시 템플릿 상수 "0" 으로 리셋됐다.
    #[test]
    fn issue2779_secpr_memo_shape_id_ref_parsed() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr id="" textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" tabStopVal="4000" tabStopUnit="HWPUNIT" outlineShapeIDRef="1" memoShapeIDRef="2" textVerticalWidthHead="0" masterPageCnt="0">
      </hp:secPr>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert_eq!(section.section_def.memo_shape_id, 2);
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
        assert_eq!((section.section_def.endnote_shape.attr >> 8) & 0x03, 0);
        assert_eq!((section.section_def.endnote_shape.attr >> 10) & 0x03, 1);
    }

    #[test]
    fn test_parse_endnote_shape_attr_table134_flags() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL" spaceColumns="1134" tabStop="8000" outlineShapeIDRef="1" masterPageCnt="0">
        <hp:endNotePr>
          <hp:autoNumFormat type="USER_CHAR" userChar="*" prefixChar="[" suffixChar="]" supscript="1"/>
          <hp:noteLine length="0" type="NONE" width="0.12 mm" color="#000000"/>
          <hp:noteSpacing betweenNotes="0" belowLine="567" aboveLine="850"/>
          <hp:numbering type="ON_PAGE" newNum="1"/>
          <hp:placement place="END_OF_SECTION" beneathText="1"/>
        </hp:endNotePr>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();
        let shape = &section.section_def.endnote_shape;

        assert_eq!(
            shape.number_format,
            crate::model::footnote::NumberFormat::UserChar
        );
        assert_eq!(shape.user_char, '*');
        assert!(shape.number_code_superscript);
        assert!(shape.print_inline_after_text);
        assert_eq!((shape.attr & 0xff), 0x81);
        assert_eq!((shape.attr >> 8) & 0x03, 1);
        assert_eq!((shape.attr >> 10) & 0x03, 2);
        assert_ne!(shape.attr & (1 << 12), 0);
        assert_ne!(shape.attr & (1 << 13), 0);
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

    /// [#1200, #4676] curve 도형의 geometry 가 `<hp:seg x1 y1 x2 y2>` (점-대-점 chain)
    /// 으로 인코딩된 경우 CurveShape.points 가 채워져야 한다. HWPX CURVE 타입은 HWP5
    /// 베지어 제어점 규약과 다르므로 segment_types를 비워 LineTo 체인으로 렌더한다.
    #[test]
    fn test_parse_curve_seg_populates_points_without_hwp5_bezier_types() {
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
        <hp:seg type="CURVE" x1="10" y1="10" x2="90" y2="10"/>
        <hp:seg type="LINE" x1="90" y1="10" x2="90" y2="90"/>
        <hp:seg type="CURVE" x1="90" y1="90" x2="10" y2="10"/>
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
        assert!(
            curve.segment_types.is_empty(),
            "HWPX CURVE 타입은 HWP5 베지어 제어점 규약으로 매핑하면 안 됨: {:?}",
            curve.segment_types
        );
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
    fn test_parse_page_border_fill_slot_by_type_not_by_order() {
        // #2885: type(BOTH/EVEN/ODD) 이 등장 순서와 다르게 기록된 경우에도
        // borderFillIDRef 가 type 값에 맞는 슬롯으로 들어가야 한다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:secPr textDirection="HORIZONTAL">
        <hp:pageBorderFill type="EVEN" borderFillIDRef="7" textBorder="CONTENT" fillArea="PAPER">
          <hp:offset left="0" right="0" top="0" bottom="0"/>
        </hp:pageBorderFill>
        <hp:pageBorderFill type="BOTH" borderFillIDRef="9" textBorder="CONTENT" fillArea="PAPER">
          <hp:offset left="0" right="0" top="0" bottom="0"/>
        </hp:pageBorderFill>
      </hp:secPr>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        assert_eq!(section.section_def.page_border_fill.border_fill_id, 9);
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

    /// 한컴은 제목 문단 첫머리에 `<hp:t><hp:titleMark ignore="1"/>제1장 …</hp:t>` 를 쓴다.
    /// 이 요소를 흘리면 문단 축이 8유닛 짧아져 다음 왕복에서 본문이 폐기된다.
    #[test]
    fn test_parse_title_mark_occupies_eight_units() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:t><hp:titleMark ignore="1"/>가나</hp:t></hp:run>
  </hp:p>
</hs:sec>"#;

        let para = &parse_hwpx_section(xml).unwrap().paragraphs[0];
        assert_eq!(para.text, "가나", "표시는 텍스트가 아니다");
        assert_eq!(para.char_offsets, vec![8, 9], "앞 8유닛을 점유한다");
        assert_eq!(
            para.title_marks,
            vec![TitleMark {
                char_idx: 0,
                ignore: true,
            }]
        );
        // 표시 8 + 글자 2 + 끝 마커 1
        assert_eq!(para.char_count, 11);
    }

    /// `ignore="0"` 은 `Mign` 과 짝이라 따로 구별해 읽어야 한다.
    #[test]
    fn test_parse_title_mark_ignore_off() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:t>가<hp:titleMark ignore="0"/>나</hp:t></hp:run>
  </hp:p>
</hs:sec>"#;

        let para = &parse_hwpx_section(xml).unwrap().paragraphs[0];
        assert_eq!(para.text, "가나");
        assert_eq!(para.char_offsets, vec![0, 9]);
        assert_eq!(
            para.title_marks,
            vec![TitleMark {
                char_idx: 1,
                ignore: false,
            }]
        );
    }

    #[test]
    fn title_mark_does_not_shift_following_field_range() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:t><hp:titleMark ignore="1"/>앞</hp:t><hp:ctrl><hp:fieldBegin id="100" type="HYPERLINK" name="" fieldid="100"/></hp:ctrl><hp:t>뒤</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="100" fieldid="100"/></hp:ctrl></hp:run>
  </hp:p>
</hs:sec>"#;

        let para = &parse_hwpx_section(xml).unwrap().paragraphs[0];
        assert_eq!(para.text, "앞뒤");
        assert_eq!(para.field_ranges.len(), 1);
        assert_eq!(para.field_ranges[0].start_char_idx, 1);
        assert_eq!(para.field_ranges[0].end_char_idx, 2);
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
    fn test_parse_hwpx_tab_width_zero_marker_not_recorded_as_ext() {
        // #4403: 직렬화기가 "데이터 없음" 마커(width=0)로 내보낸 암묵적 기본 탭은
        // 재적재 시 tab_extended 항목을 만들면 안 된다 — 만들면 렌더러가 그 폭을
        // 실제 계산값으로 신뢰해(`total + width`) 문단의 진짜 TabDef(예: 우측 정렬)를
        // 무시하고 커서 위치와 무관한 고정 거리만 전진시킨다. width=0 은 실제 탭에서
        // 나올 수 없는 값(폭 0인 탭은 시각 효과가 없음)이라 안전한 마커다. 탭 문자(\t)
        // 자체는 그대로 보존해야 한다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:t>I.소설의 이해<hp:tab width="0" leader="0" type="1"/>3</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.text, "I.소설의 이해\t3");
        assert!(
            para.tab_extended.is_empty(),
            "width=0 마커는 tab_extended 에 실리면 안 됨: {:?}",
            para.tab_extended
        );
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
        // 같은 ID여도 표 슬롯 뒤의 별도 run 경계(start_pos=9)는 보존해야 한다 (#3739).
        assert_eq!(
            para.char_shapes
                .iter()
                .map(|cs| (cs.start_pos, cs.char_shape_id))
                .collect::<Vec<_>>(),
            vec![(0, 0), (9, 0)]
        );
        assert_eq!(para.controls.len(), 1);
    }

    #[test]
    fn issue_3739_secpr_template_handoff_same_id_is_normalized() {
        // 첫 secPr run과 템플릿이 별도로 넣는 첫 텍스트 run은 같은 ID여도
        // HWP PARA_CHAR_SHAPE에서 하나의 시작 entry여야 한다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="5"><hp:secPr textDirection="HORIZONTAL"></hp:secPr></hp:run>
    <hp:run charPrIDRef="5"><hp:t>A</hp:t></hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let para = &section.paragraphs[0];
        assert_eq!(para.text, "A");
        assert_eq!(
            para.char_shapes
                .iter()
                .map(|cs| (cs.start_pos, cs.char_shape_id))
                .collect::<Vec<_>>(),
            vec![(0, 5)]
        );
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
    fn test_parse_table_row_sizes_is_cell_count_not_height() {
        // [#row_sizes 계약] HWP 스펙 UINT16[NRows]("행별 셀 수")과 동일해야 한다.
        // model::table::Table::rebuild_row_sizes, parser::control(HWP5),
        // html_table_import 모두 이 필드를 "행별 셀 개수"로 채우므로 HWPX 파서만
        // 높이를 채우면 계약이 깨진다. 2행 2열에서 각 셀 높이를 다르게 주어
        // 카운트(2)와 높이(예: 500/3000)를 구분한다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:tbl rowCnt="2" colCnt="2" cellSpacing="0" borderFillIDRef="0">
      <hp:inMargin left="0" right="0" top="0" bottom="0"/>
      <hp:tr>
        <hp:tc borderFillIDRef="0"><hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="1000" height="500"/></hp:tc>
        <hp:tc borderFillIDRef="0"><hp:cellAddr colAddr="1" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="1000" height="500"/></hp:tc>
      </hp:tr>
      <hp:tr>
        <hp:tc borderFillIDRef="0"><hp:cellAddr colAddr="0" rowAddr="1"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="1000" height="3000"/></hp:tc>
        <hp:tc borderFillIDRef="0"><hp:cellAddr colAddr="1" rowAddr="1"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="1000" height="3000"/></hp:tc>
      </hp:tr>
    </hp:tbl>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let table = match &section.paragraphs[0].controls[0] {
            crate::model::control::Control::Table(table) => table,
            other => panic!("expected table, got {:?}", other),
        };
        assert_eq!(table.row_sizes, vec![2, 2]);
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
              vertOffset="4294965296" horzOffset="0"/>
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
        assert_eq!(table.common.vertical_offset as i32, -2000);
        assert_eq!(table.common.attr, 0x082a_2211);
        assert_eq!(table.attr, 0x01);
        assert_eq!(table.raw_table_record_attr, 0x0400_000e);
    }

    #[test]
    fn table_textwrap_tight_and_through_survive_roundtrip() {
        // 표 textWrap="TIGHT"/"THROUGH" 가 파서 arm 누락으로 SQUARE 로 유실되던 결함.
        // 방출측 text_wrap_str 은 이 두 값을 내므로 왕복 보존돼야 한다.
        for (s, expect) in [("TIGHT", TextWrap::Tight), ("THROUGH", TextWrap::Through)] {
            let xml = format!(
                r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"><hp:p paraPrIDRef="0" styleIDRef="0"><hp:tbl numberingType="TABLE" textWrap="{s}" pageBreak="CELL" repeatHeader="0" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="0" noAdjust="0"><hp:sz width="1000" widthRelTo="ABSOLUTE" height="1000" heightRelTo="ABSOLUTE"/><hp:pos treatAsChar="0" flowWithText="1" allowOverlap="0" vertRelTo="PARA" horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/><hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="0" right="0" top="0" bottom="0"/><hp:tr><hp:tc borderFillIDRef="0"><hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="1000" height="1000"/></hp:tc></hp:tr></hp:tbl></hp:p></hs:sec>"#
            );
            let section = parse_hwpx_section(&xml).unwrap();
            let table = match &section.paragraphs[0].controls[0] {
                crate::model::control::Control::Table(t) => t,
                other => panic!("expected table, got {other:?}"),
            };
            assert_eq!(
                table.common.text_wrap, expect,
                "textWrap={s} 가 {expect:?} 로 파싱돼야 함(SQUARE 유실 방지)"
            );
        }
    }

    #[test]
    fn picture_pattern_8_8_effect_is_preserved() {
        // 방출측이 내는 PATTERN_8_8 효과가 기본값 RealPic 으로 되돌아가지 않아야 한다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:pic id="1" zOrder="0" textWrap="SQUARE" textFlow="BOTH_SIDES">
        <hp:img binaryItemIDRef="image1" effect="PATTERN_8_8"/>
      </hp:pic>
    </hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        let Control::Picture(picture) = &section.paragraphs[0].controls[0] else {
            panic!("첫 컨트롤은 그림이어야 함");
        };
        assert_eq!(
            picture.image_attr.effect,
            crate::model::image::ImageEffect::Pattern8x8,
            "PATTERN_8_8 그림 효과가 RealPic 으로 유실되면 안 됨"
        );
    }

    #[test]
    fn dutmal_maintext_subtext_preserve_cdata() {
        // hp:dutmal(덧말)의 mainText/subText가 CDATA로 인코딩된 경우
        // (예: 비교연산자 `<`/`>` 포함) 파서 arm 누락으로 소실되던 결함.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:dutmal posType="TOP" align="CENTER" szRatio="50" option="0" styleIDRef="0">
        <hp:mainText><![CDATA[a<b]]></hp:mainText>
        <hp:subText><![CDATA[c>d]]></hp:subText>
      </hp:dutmal>
    </hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        let Control::Ruby(ruby) = &section.paragraphs[0].controls[0] else {
            panic!("첫 컨트롤은 Ruby(덧말)여야 함");
        };
        assert_eq!(ruby.main_text, "a<b", "mainText CDATA 가 소실되면 안 됨");
        assert_eq!(ruby.ruby_text, "c>d", "subText CDATA 가 소실되면 안 됨");
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
    fn hwpx_storage_flip_defaults_follow_hancom_group_contract() {
        let mut top_level_picture = ShapeComponentAttr {
            rotate_image: true,
            ..Default::default()
        };
        materialize_shape_hwp_storage_defaults(
            &mut CommonObjAttr::default(),
            &mut top_level_picture,
            ShapeStorageKind::Picture,
        );
        assert_eq!(top_level_picture.flip, 0x2008_0000);

        let mut grouped_picture = ShapeComponentAttr {
            group_level: 1,
            rotate_image: true,
            ..Default::default()
        };
        materialize_shape_hwp_storage_defaults(
            &mut CommonObjAttr::default(),
            &mut grouped_picture,
            ShapeStorageKind::Picture,
        );
        assert_eq!(grouped_picture.flip, 0x200b_0000);

        let mut grouped_text_box = ShapeComponentAttr {
            group_level: 1,
            rotate_image: true,
            ..Default::default()
        };
        materialize_shape_hwp_storage_defaults(
            &mut CommonObjAttr::default(),
            &mut grouped_text_box,
            ShapeStorageKind::TextBoxDrawing,
        );
        assert_eq!(grouped_text_box.flip, 0x010b_0000);
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
                    let start = e.to_owned();
                    parse_field_parameters(&start, &mut reader, &mut field).unwrap();
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
    fn parse_field_parameters_reassembles_nested_params_balanced() {
        // 중첩 파라미터(listParam 안의 stringParam). 종전엔 open_param 이 마지막 Start 로
        // 덮여 바깥 </hp:listParam> 닫는 태그가 누락돼 raw_parameters_xml 이 불균형이었다.
        let xml = r#"<hp:parameters xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" cnt="1" name=""><hp:listParam cnt="1" name="L"><hp:stringParam name="A">x</hp:stringParam></hp:listParam></hp:parameters>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut field = Field::default();

        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if local_name(e.name().as_ref()) == b"parameters" => {
                    let start = e.to_owned();
                    parse_field_parameters(&start, &mut reader, &mut field).unwrap();
                    break;
                }
                Event::Eof => panic!("parameters not found"),
                _ => {}
            }
            buf.clear();
        }

        let raw = field.raw_parameters_xml.expect("raw_parameters_xml");
        assert!(raw.contains("</hp:stringParam>"), "inner close: {raw}");
        assert!(
            raw.contains("</hp:listParam>"),
            "바깥 </hp:listParam> 누락(중첩 불균형): {raw}"
        );
        assert!(raw.ends_with("</hp:parameters>"), "params close: {raw}");
    }

    #[test]
    fn test_parse_field_parameters_preserves_cdata_command() {
        let xml = r#"<hp:parameters xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph">
  <hp:stringParam name="Command"><![CDATA[HYPERLINK "https://example.com/?a=1&b=2"]]></hp:stringParam>
</hp:parameters>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let mut field = Field::default();

        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if local_name(e.name().as_ref()) == b"parameters" => {
                    let start = e.to_owned();
                    parse_field_parameters(&start, &mut reader, &mut field).unwrap();
                    break;
                }
                Event::Eof => panic!("parameters not found"),
                _ => {}
            }
            buf.clear();
        }

        assert_eq!(field.command, "HYPERLINK \"https://example.com/?a=1&b=2\"");
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

    // ---------- #1556: 다단락 필드의 고아 fieldEnd ----------

    #[test]
    fn task1556_orphan_field_end_recorded_in_end_paragraph() {
        // fieldBegin 은 문단 0, fieldEnd 는 문단 1 (다단락 누름틀 필드).
        // 문단 1 은 컨트롤·field_range 없이 8유닛 슬롯만 갖는다 → orphan_field_ends 로 기록.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="1878228493" type="CLICK_HERE" name="본문" fieldid="627272811"/></hp:ctrl><hp:t>본문시작</hp:t></hp:run>
  </hp:p>
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="3"><hp:t>끝.</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="1878228493" fieldid="627272811"/></hp:ctrl></hp:run>
    <hp:run charPrIDRef="30"><hp:t/></hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        // 문단 0: fieldBegin 보존 (Control::Field), 고아 없음.
        let p0 = &section.paragraphs[0];
        assert!(
            matches!(p0.controls.first(), Some(Control::Field(_))),
            "문단 0 은 fieldBegin 컨트롤 보존"
        );
        assert!(p0.orphan_field_ends.is_empty(), "문단 0 고아 없음");

        // 문단 1: 텍스트 "끝." (2자) + 고아 fieldEnd 8유닛.
        let p1 = &section.paragraphs[1];
        assert_eq!(p1.text, "끝.");
        assert_eq!(p1.orphan_field_ends.len(), 1, "고아 fieldEnd 1개 기록");
        let ofe = &p1.orphan_field_ends[0];
        assert_eq!(ofe.char_idx, 2, "텍스트 끝(인덱스 2) 위치");
        assert_eq!(ofe.begin_id_ref, 1_878_228_493);
        assert_eq!(ofe.field_id, 627_272_811);
        let begin_ctrl_id = match p0.controls.first() {
            Some(Control::Field(field)) => field.ctrl_id,
            other => panic!("fieldBegin 컨트롤이 아니다: {other:?}"),
        };
        assert_eq!(
            ofe.begin_ctrl_id, begin_ctrl_id,
            "HWP5 저장에 필요한 field control id를 앞 문단 fieldBegin에서 연결한다"
        );
        let hwp_roundtrip = crate::parser::body_text::parse_body_text_section(
            &crate::serializer::body_text::serialize_section(&section),
        )
        .expect("HWP5로 저장한 다문단 field가 다시 파싱돼야 한다");
        let hwp_end = hwp_roundtrip.paragraphs[1]
            .orphan_field_ends
            .first()
            .expect("HWP5 저장본에도 fieldEnd 슬롯이 남아야 한다");
        assert_eq!(hwp_end.begin_ctrl_id, begin_ctrl_id);
        // char_count = 텍스트 2 + fieldEnd 8 + 끝마커 1 = 11.
        assert_eq!(
            p1.char_count, 11,
            "고아 fieldEnd 8유닛이 char_count 에 반영"
        );
        // 두 번째 char_shape(run charPrIDRef=30)는 offsets 축 10 (텍스트 2 + 8).
        assert_eq!(
            p1.char_shapes
                .iter()
                .map(|c| (c.start_pos, c.char_shape_id))
                .collect::<Vec<_>>(),
            vec![(0, 3), (10, 30)],
        );
    }

    #[test]
    fn task1556_same_paragraph_field_uses_range_not_orphan() {
        // 동일 문단 내 begin+end 는 종전대로 field_ranges 로만 처리 (고아 0) — 회귀 가드.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0"><hp:ctrl><hp:fieldBegin id="100" type="HYPERLINK" name="" fieldid="100"/></hp:ctrl><hp:t>링크</hp:t><hp:ctrl><hp:fieldEnd beginIDRef="100" fieldid="100"/></hp:ctrl></hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        let p = &section.paragraphs[0];
        assert_eq!(p.field_ranges.len(), 1, "동일 문단 필드는 field_range");
        assert!(p.orphan_field_ends.is_empty(), "고아 기록 없음");
    }

    /// #1512: 비-Memo 필드도 고유 OWPML `id` 를 field_id 로 써야 한다. 같은 종류 필드가
    /// 공유하는 `fieldid` 를 우선하면 모든 필드가 동일 ID 로 반환된다(누름틀 구분 불가).
    #[test]
    fn task1512_non_memo_field_uses_unique_id() {
        fn parse_one(xml: &str) -> Field {
            let mut reader = Reader::from_str(xml);
            let mut buf = Vec::new();
            loop {
                match reader.read_event_into(&mut buf).unwrap() {
                    Event::Empty(ref e) | Event::Start(ref e)
                        if local_name(e.name().as_ref()) == b"fieldBegin" =>
                    {
                        return parse_field_begin_attrs(e);
                    }
                    Event::Eof => panic!("fieldBegin not found"),
                    _ => {}
                }
            }
        }
        // 공유 fieldid(627469685) + 서로 다른 고유 id → field_id 는 고유 id 여야 한다.
        let ns = "http://www.hancom.co.kr/hwpml/2011/paragraph";
        let a = parse_one(&format!(
            r#"<hp:fieldBegin xmlns:hp="{ns}" type="FORMULA" id="1685705574" fieldid="627469685"/>"#
        ));
        let b = parse_one(&format!(
            r#"<hp:fieldBegin xmlns:hp="{ns}" type="FORMULA" id="1685705575" fieldid="627469685"/>"#
        ));
        assert_eq!(a.field_id, 1_685_705_574);
        assert_eq!(b.field_id, 1_685_705_575);
        assert_ne!(
            a.field_id, b.field_id,
            "공유 fieldid 가 아닌 고유 id 로 구분돼야 함"
        );
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
    fn test_parse_hwpx_connect_line_materializes_connector() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:connectLine id="1522096658" zOrder="513" textWrap="IN_FRONT_OF_TEXT" textFlow="BOTH_SIDES" instid="448354835" type="STRAIGHT_ONEWAY">
        <hp:offset x="0" y="0"/>
        <hp:orgSz width="1257" height="1"/>
        <hp:curSz width="1257" height="0"/>
        <hp:pos treatAsChar="0" flowWithText="0" allowOverlap="1" vertRelTo="PAPER" horzRelTo="PAPER" vertOffset="25812" horzOffset="45538"/>
        <hp:lineShape color="#000000" width="141" style="SOLID" headStyle="NORMAL" tailStyle="ARROW" headfill="1" tailfill="1" headSz="MEDIUM_MEDIUM" tailSz="MEDIUM_MEDIUM"/>
        <hp:startPt x="0" y="0" subjectIDRef="11" subjectIdx="2"/>
        <hp:endPt x="1257" y="0" subjectIDRef="22" subjectIdx="3"/>
        <hp:controlPoints>
          <hp:point x="0" y="0" type="3"/>
          <hp:point x="100" y="0" type="26"/>
        </hp:controlPoints>
      </hp:connectLine>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Line(line) = shape.as_ref() else {
            panic!("expected line shape");
        };

        assert_eq!(line.common.instance_id, 1522096658);
        assert_eq!(line.common.horizontal_offset, 45538);
        assert_eq!(line.common.vertical_offset, 25812);
        assert_eq!(line.start.x, 0);
        assert_eq!(line.end.x, 1257);

        let connector = line.connector.as_ref().expect("connector data");
        assert_eq!(connector.link_type, LinkLineType::StraightOneWay);
        assert_eq!(connector.start_subject_id, 11);
        assert_eq!(connector.start_subject_index, 2);
        assert_eq!(connector.end_subject_id, 22);
        assert_eq!(connector.end_subject_index, 3);
        assert_eq!(connector.control_points.len(), 2);
        assert_eq!(connector.control_points[1].x, 100);
        assert_eq!(connector.control_points[1].point_type, 26);
    }

    #[test]
    fn bugfind_shape_offset_negative_x_y_not_dropped_to_zero() {
        // hp:offset(개체 내부 shape-transform 오프셋) x/y 는 음수일 수 있는데,
        // 종전엔 parse_u32 로 읽어 "-500" 같은 문자열이 파싱 실패로 0 이 됐다
        // (hp:pos 의 vertOffset/horzOffset 형제 필드는 이미 parse_i32_wrapping 사용).
        // hp:pos 가 없어 offset 이 common.horizontal_offset/vertical_offset 에도
        // 그대로 폴백되는 경로를 함께 확인한다.
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:connectLine id="1" zOrder="0" textWrap="SQUARE" textFlow="BOTH_SIDES" instid="1" type="STRAIGHT_ONEWAY">
        <hp:offset x="-500" y="-800"/>
        <hp:orgSz width="100" height="1"/>
        <hp:curSz width="100" height="0"/>
        <hp:lineShape color="#000000" width="141" style="SOLID" headStyle="NORMAL" tailStyle="NORMAL" headfill="1" tailfill="1" headSz="MEDIUM_MEDIUM" tailSz="MEDIUM_MEDIUM"/>
        <hp:startPt x="0" y="0" subjectIDRef="1" subjectIdx="0"/>
        <hp:endPt x="100" y="0" subjectIDRef="2" subjectIdx="0"/>
      </hp:connectLine>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"##;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Line(line) = shape.as_ref() else {
            panic!("expected line shape");
        };

        assert_eq!(
            line.drawing.shape_attr.offset_x, -500,
            "offset x=-500 이 0으로 뭉개지면 안 됨"
        );
        assert_eq!(
            line.drawing.shape_attr.offset_y, -800,
            "offset y=-800 이 0으로 뭉개지면 안 됨"
        );
        assert_eq!(
            line.common.horizontal_offset as i32, -500,
            "hp:pos 가 없으면 offset 이 common.horizontal_offset 으로 폴백돼야 함"
        );
        assert_eq!(
            line.common.vertical_offset as i32, -800,
            "hp:pos 가 없으면 offset 이 common.vertical_offset 으로 폴백돼야 함"
        );
    }

    #[test]
    fn test_parse_rect_ratio_as_round_rate() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:rect id="1" zOrder="0" ratio="50" numberingType="PICTURE">
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
        assert!(
            rect.common.hwp5_gen_shape_attr_bit28,
            "numberingType=PICTURE는 한컴 HWP5 공통 개체 bit 28로 저장돼야 한다"
        );
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
        <hp:pos treatAsChar="0" flowWithText="1" allowOverlap="1" holdAnchorAndSO="1" vertRelTo="PARA" horzRelTo="PARA"/>
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
        // holdAnchorAndSO="1" → prevent_page_break 이 비표 개체에서도 되읽혀야 한다.
        assert_eq!(rect.common.prevent_page_break, 1);
        assert_eq!(
            rect.common.text_flow,
            crate::model::shape::TextFlow::RightOnly
        );
    }

    /// [#2726] 공용 자식 파서(`parse_common_shape_children`)는 차트·OLE 를 담당하는데
    /// `widthRelTo`/`heightRelTo` arm 이 없어 크기 기준이 **파싱 단계에서** 유실됐다.
    /// 표(1702/1706)·도형(2925/2928)·그림(#2712) 파서와 동형으로 보강한다.
    #[test]
    fn issue2726_parse_chart_preserves_size_criteria() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:chart chartIDRef="Chart/chart1.xml" id="1" zOrder="0" textWrap="SQUARE">
        <hp:sz width="4000" height="3000" widthRelTo="COLUMN" heightRelTo="PAGE" protect="1"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
      </hp:chart>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected OLE(chart) shape");
        };
        assert_eq!(
            ole.common.width_criterion,
            SizeCriterion::Column,
            "widthRelTo=\"COLUMN\" 이 IR 에 적재되어야 한다"
        );
        assert_eq!(
            ole.common.height_criterion,
            SizeCriterion::Page,
            "heightRelTo=\"PAGE\" 가 IR 에 적재되어야 한다"
        );
        assert!(ole.common.size_protect, "protect=\"1\" 은 종전에도 읽혔다");
    }

    /// [#2726] 높이 기준은 `allow_column_para=false` 로 읽어 치역이
    /// `{Paper, Page, Absolute}` 3값이어야 한다. `COLUMN`/`PARA` 가 들어와도 `Absolute`
    /// 로 접혀야 직렬화기 `height_criterion_str` 와 정확한 역 관계가 유지된다.
    #[test]
    fn issue2726_parse_chart_height_folds_column_and_para_to_absolute() {
        for raw in ["COLUMN", "PARA"] {
            let xml = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:chart chartIDRef="Chart/chart1.xml" id="1" zOrder="0" textWrap="SQUARE">
        <hp:sz width="4000" height="3000" widthRelTo="{raw}" heightRelTo="{raw}" protect="0"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
      </hp:chart>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#
            );

            let section = parse_hwpx_section(&xml).unwrap();
            let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
                panic!("expected shape control");
            };
            let ShapeObject::Ole(ole) = shape.as_ref() else {
                panic!("expected OLE(chart) shape");
            };
            // 너비는 5값 전부 허용 → 원문 그대로.
            let expected_width = if raw == "COLUMN" {
                SizeCriterion::Column
            } else {
                SizeCriterion::Para
            };
            assert_eq!(
                ole.common.width_criterion, expected_width,
                "너비는 {raw} 를 그대로 보존해야 한다"
            );
            // 높이는 3값으로 접힘.
            assert_eq!(
                ole.common.height_criterion,
                SizeCriterion::Absolute,
                "높이 {raw} 는 Absolute 로 접혀야 한다"
            );
        }
    }

    #[test]
    fn test_parse_line_preserves_is_reverse_hv() {
        // <hp:line isReverseHV="1"> → LineShape.started_right_or_bottom.
        // 종전엔 파서가 isReverseHV 를 읽지 않아 방향 반전이 유실됐다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:line id="1" zOrder="0" textWrap="SQUARE" textFlow="BOTH_SIDES" isReverseHV="1">
        <hp:sz width="1000" height="0" protect="0"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
        <hp:pt0 x="0" y="0"/>
        <hp:pt1 x="1000" y="0"/>
      </hp:line>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Line(line) = shape.as_ref() else {
            panic!("expected line shape");
        };
        assert!(
            line.started_right_or_bottom,
            "isReverseHV=\"1\" 이 started_right_or_bottom 로 되읽혀야 함"
        );
    }

    // ---------- #2882: hp:ole/hp:chart numberingType 라운드트립 ----------

    #[test]
    fn issue2882_ole_numbering_type_picture_is_parsed_into_common_field() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ole id="1" zOrder="0" numberingType="PICTURE" binaryItemIDRef="ole1">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
      </hp:ole>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected ole shape");
        };
        assert_eq!(
            ole.common.numbering_type,
            crate::model::shape::ObjectNumberingType::Picture,
            "numberingType=\"PICTURE\" 가 common.numbering_type 에 매핑돼야 함(직렬화기가 참조하는 필드)"
        );
    }

    #[test]
    fn issue2882_chart_numbering_type_table_is_parsed_into_common_field() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:chart id="1" zOrder="0" numberingType="TABLE" chartIDRef="Chart/chart1.xml">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
      </hp:chart>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected ole shape (chart is modeled as OleShape)");
        };
        assert_eq!(
            ole.common.numbering_type,
            crate::model::shape::ObjectNumberingType::Table,
            "numberingType=\"TABLE\" 가 common.numbering_type 에 매핑돼야 함(직렬화기가 참조하는 필드)"
        );
    }

    #[test]
    fn test_parse_ole_preserves_extent_and_draw_aspect() {
        // <hc:extent> 원본 개체 크기와 drawAspect(표시 방식)가 IR 로 되읽혀야 한다.
        // 종전엔 extent 를 7200 으로 하드코딩하고 drawAspect 를 읽지 않아,
        // 모든 OLE 이 7200x7200 / CONTENT 로 왕복에서 뭉개졌다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ole id="1" zOrder="0" drawAspect="ICON" binaryItemIDRef="ole1">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
        <hc:extent x="12345" y="6789"/>
      </hp:ole>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected ole shape");
        };
        assert_eq!(ole.extent_x, 12345, "hc:extent x 가 보존돼야 함");
        assert_eq!(ole.extent_y, 6789, "hc:extent y 가 보존돼야 함");
        assert_eq!(
            ole.drawing_aspect,
            crate::model::shape::OleDrawingAspect::Icon,
            "drawAspect=ICON 이 보존돼야 함"
        );
    }

    #[test]
    fn bugfind_ole_negative_pos_offset_is_not_zeroed() {
        // [버그] `parse_common_shape_children` (chart/OLE 공용 `<hp:pos>` 파서)는
        // vertOffset/horzOffset 을 `parse_u32` 로 읽는다 — `str::parse::<u32>` 는
        // 부호 문자를 거부해 실패 시 `unwrap_or(0)` 로 조용히 0 이 된다. 반면 이미지/
        // 표 등 다른 개체의 `<hp:pos>` 파서(section.rs:3150-3151, parse_object_layout_child)
        // 는 `parse_i32_wrapping` 을 써서 음수 오프셋(왼쪽/위쪽으로 벗어난 앵커 상대
        // 배치)을 올바르게 보존한다. 우리 자신의 직렬화기(serializer/hwpx/shape.rs)가
        // signed 오프셋을 그대로 십진수로 방출하므로(예: "-100"), 그런 OLE/차트가
        // 저장 후 재로드되면 위치가 0 으로 뭉개진다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ole id="1" zOrder="0" binaryItemIDRef="ole1">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA" vertOffset="-200" horzOffset="-100"/>
      </hp:ole>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected ole shape");
        };
        assert_eq!(
            ole.common.horizontal_offset as i32, -100,
            "hp:pos horzOffset=\"-100\" 이 0 으로 뭉개지면 안 됨"
        );
        assert_eq!(
            ole.common.vertical_offset as i32, -200,
            "hp:pos vertOffset=\"-200\" 이 0 으로 뭉개지면 안 됨"
        );
    }

    #[test]
    fn bugfind_ole_unsigned_wrapped_pos_offset_is_preserved() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ole id="1" zOrder="0" binaryItemIDRef="ole1">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"
                vertOffset="4294965296" horzOffset="4294964867"/>
        <hc:extent x="1" y="1"/>
      </hp:ole>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected ole shape");
        };
        assert_eq!(ole.common.vertical_offset as i32, -2000);
        assert_eq!(ole.common.horizontal_offset as i32, -2429);
    }

    #[test]
    fn bugfind_ole_shape_comment_is_parsed_into_common_description() {
        // 실측: samples/bitmap.hwp 를 export-hwpx --verify 로 왕복하면
        // OLE 개체(그림판 개체)의 "OLE 개체입니다.\r\n개체 형식은 Paintbrush
        // Picture입니다." 설명문(hp:shapeComment)이 IR 차이 1건으로 검출됐다.
        // 방출측(write_shape_comment)은 <hp:shapeComment>를 정상적으로 쓰지만
        // OLE/차트 공용 자식 파서(parse_common_shape_children)에 shapeComment
        // arm 이 없어 되읽지 못하고 유실되었다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ole id="1" zOrder="0" binaryItemIDRef="ole1">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
        <hp:shapeComment>OLE 개체입니다.&#13;&#10;개체 형식은 Paintbrush Picture입니다.</hp:shapeComment>
      </hp:ole>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected ole shape");
        };
        assert_eq!(
            ole.common.description, "OLE 개체입니다.\r\n개체 형식은 Paintbrush Picture입니다.",
            "hp:shapeComment 가 ole.common.description 으로 되읽혀야 함"
        );
    }

    #[test]
    fn chart_shape_comment_is_parsed_into_common_description() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:chart id="1" zOrder="0" chartIDRef="Chart/chart1.xml">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
        <hp:shapeComment>분기별 매출 차트</hp:shapeComment>
      </hp:chart>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).expect("parse chart with shapeComment");
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(chart) = shape.as_ref() else {
            panic!("expected chart modeled as OLE shape");
        };
        assert_eq!(chart.common.description, "분기별 매출 차트");
    }

    #[test]
    fn test_shape_img_brush_preserves_image_ref_and_mode() {
        // [#2563] 도형 <hc:imgBrush> 의 <hc:img> 자식과 12종 mode 매핑.
        // 종전엔 mode 4종만 받아 TOTAL 이 TILE 로 붕괴했고, <hc:img> arm 이 없어
        // binaryItemIDRef/bright/contrast/effect 가 전부 버려졌다. bin_data_id 가
        // 0 이면 직렬화가 <hc:img> 를 못 내므로 이미지 도형이 빈 도형이 된다.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section"
        xmlns:hc="http://www.hancom.co.kr/hwpml/2011/core">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:rect id="1" zOrder="0" textWrap="SQUARE">
        <hp:sz width="2600" height="2600"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
        <hc:fillBrush>
          <hc:imgBrush mode="TOTAL">
            <hc:img binaryItemIDRef="image3" bright="10" contrast="-5" effect="GRAY_SCALE"/>
          </hc:imgBrush>
        </hc:fillBrush>
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
        let img = rect
            .drawing
            .fill
            .image
            .as_ref()
            .expect("imgBrush 는 ImageFill 을 남겨야 함");

        assert_eq!(img.bin_data_id, 3, "binaryItemIDRef 가 보존돼야 함");
        assert_eq!(img.brightness, 10, "bright 가 보존돼야 함");
        assert_eq!(img.contrast, -5, "contrast 가 보존돼야 함");
        assert_eq!(img.effect, 1, "effect=GRAY_SCALE 가 보존돼야 함");
        assert_eq!(
            img.fill_mode,
            crate::model::style::ImageFillMode::Total,
            "mode=TOTAL 이 TILE 로 붕괴하면 안 됨"
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
    fn issue4387_col_sz_parses_individual_widths_and_gaps() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ctrl>
        <hp:colPr type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="0" sameGap="0">
          <hp:colSz width="4000" gap="500"/>
          <hp:colSz width="6000" gap="0"/>
        </hp:colPr>
      </hp:ctrl>
      <hp:t>A</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"##;
        let section = parse_hwpx_section(xml).unwrap();
        let Control::ColumnDef(cd) = &section.paragraphs[0].controls[0] else {
            panic!("expected ColumnDef control");
        };
        assert!(!cd.same_width);
        assert_eq!(
            cd.widths,
            vec![4000, 6000],
            "단별 너비가 파싱돼야 함(#4387)"
        );
        assert_eq!(cd.gaps, vec![500, 0], "단별 간격이 파싱돼야 함(#4387)");
        assert!(!cd.proportional_widths, "HWPX colSz 는 절대 HWPUNIT");
    }

    /// [#4387 후속] `colSz@width` 는 스키마상 `xs:positiveInteger`(상한 없음)인데
    /// `ColumnDef.widths` 는 `Vec<i16>`(최대 32767). A3 등 큰 용지·비대칭 다단에서
    /// 나올 수 있는 40000(≈141mm) 처럼 i16 범위를 넘는 값을 공용 `parse_i16` 로
    /// 파싱하면 `str::parse::<i16>()` 오버플로 에러를 `unwrap_or(0)` 이 삼켜
    /// widths=[0, 13000] 처럼 무경고 0-폴백됐다(단이 통째로 사라짐 — 수정 전
    /// 코드로 직접 재현·확인). saturating 클램프로 i16::MAX 로 잘리는지
    /// 확인한다 — 0 이 되면 안 된다.
    #[test]
    fn issue4387_col_sz_width_overflow_saturates_not_zeroes() {
        let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ctrl>
        <hp:colPr type="NEWSPAPER" layout="LEFT" colCount="2" sameSz="0" sameGap="0">
          <hp:colSz width="40000" gap="500"/>
          <hp:colSz width="13000" gap="-7"/>
        </hp:colPr>
      </hp:ctrl>
      <hp:t>A</hp:t>
    </hp:run>
  </hp:p>
</hs:sec>"##;
        let section = parse_hwpx_section(xml).unwrap();
        let Control::ColumnDef(cd) = &section.paragraphs[0].controls[0] else {
            panic!("expected ColumnDef control");
        };
        assert_eq!(
            cd.widths[0],
            i16::MAX,
            "i16 범위를 넘는 width 는 0 이 아니라 i16::MAX 로 saturate 해야 함"
        );
        assert_eq!(cd.widths[1], 13000, "범위 내 값은 그대로 보존돼야 함");
        assert_eq!(
            cd.gaps[1], 0,
            "음수 gap(스키마상 nonNegativeInteger 위반)은 0 으로 클램프돼야 함"
        );
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

    /// #2916: `<hp:equation>`의 `<hp:script>` 본문이 CDATA 섹션으로 인코딩된 경우
    /// (실제 한글 저장 결과에서 관찰되는 형태 — 수식 스크립트에 `<`, `>` 등이
    /// 다수 포함되어 개별 엔티티 이스케이프 대신 CDATA 로 감싸짐), 파서가
    /// Event::CData 를 처리하지 않으면 script 가 빈 문자열로 소실된다.
    #[test]
    fn task_m100_2916_equation_script_cdata_not_lost() {
        let xml = r##"<hp:equation xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
            id="1" version="Equation Version 60" baseLine="0" textColor="#000000" baseUnit="1000" font="HYhwpEQ"><hp:script><![CDATA[a < b > c]]></hp:script></hp:equation>"##;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        let ctrl = loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if local_name(e.name().as_ref()) == b"equation" => {
                    break parse_equation(e, &mut reader).unwrap();
                }
                Event::Eof => panic!("equation not found"),
                _ => {}
            }
            buf.clear();
        };
        let Control::Equation(eq) = ctrl else {
            panic!("expected Equation control");
        };
        assert_eq!(
            eq.script, "a < b > c",
            "CDATA 로 감싸진 수식 스크립트가 소실되면 안 된다"
        );
    }

    #[test]
    fn parse_field_type_accepts_toc() {
        // 직렬화기(hwpx/field.rs)가 방출하는 "TOC" 가 TableOfContents 로 파싱돼야
        // hwpx 왕복에서 차례 필드 타입이 Unknown 으로 유실되지 않는다.
        assert_eq!(parse_field_type("TOC"), FieldType::TableOfContents);
        assert_eq!(
            parse_field_type("TABLE_OF_CONTENTS"),
            FieldType::TableOfContents
        );
    }

    #[test]
    fn compose_text_preserve_cdata() {
        // hp:compose(글자겹치기)의 composeText가 CDATA로 인코딩된 경우
        // (예: 비교연산자 `<`/`>` 포함) read_compose_text의 arm 누락으로 소실되던 결함(#2974).
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:compose circleType="CHAR" charSz="100" composeType="OVERLAP">
        <composeText><![CDATA[a<b]]></composeText>
      </hp:compose>
    </hp:run>
  </hp:p>
</hs:sec>"#;
        let section = parse_hwpx_section(xml).unwrap();
        let Control::CharOverlap(co) = &section.paragraphs[0].controls[0] else {
            panic!("첫 컨트롤은 CharOverlap(글자겹치기)이어야 함");
        };
        assert_eq!(
            co.chars,
            vec!['a', '<', 'b'],
            "composeText CDATA 가 소실되면 안 됨"
        );
    }

    #[test]
    fn task2931_chart_lock_attr_roundtrips_into_common() {
        // <hp:chart lock="1" .../> → common.locked 이 true 로 되읽혀야 한다.
        // 종전엔 parse_hp_chart_element 가 lock 속성을 매치하지 않아 항상 기본값(false)
        // 으로 남고, 직렬화 시에도 render_common_shape_xml 이 "0"을 하드코딩했다(#2931).
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:chart id="1" zOrder="0" numberingType="NONE" textWrap="SQUARE" textFlow="BOTH_SIDES" lock="1" chartIDRef="Chart/chart1.xml" instid="1"></hp:chart>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected chart (modeled as OLE) shape");
        };
        assert!(
            ole.common.locked,
            "lock=\"1\" 이 common.locked 에 보존돼야 한다"
        );
    }

    // ---------- #4319: 차트·OLE 캡션 파싱 ----------

    /// [#4319] `<hp:chart>` 내부 `<hp:caption>` — 종전엔 공용 자식 파서
    /// (`parse_common_shape_children`, 차트·OLE 전용)에 caption arm 이 없어
    /// 캡션 subList 가 파싱 단계에서 완전히 유실됐다(표/도형/묶음/그림은 모두
    /// 캡션을 읽지만 차트·OLE 만 빠져 있었다). 캡션 구조는 실 코퍼스 hp:pic
    /// 캡션 실측(outMargin 뒤·shapeComment 앞, side/fullSz/width/gap/lastWidth
    /// 속성 + subList/p/run/t)과 OWPML AbstractShapeObjectType 스키마
    /// (sz→pos→outMargin→caption→shapeComment 순서, hp:chart/hp:ole 모두 이
    /// 타입을 상속)를 그대로 따른다.
    #[test]
    fn issue4319_chart_caption_parses_into_caption_field() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:chart id="1" zOrder="0" numberingType="NONE" textWrap="SQUARE" textFlow="BOTH_SIDES" chartIDRef="Chart/chart1.xml" instid="1">
        <hp:sz width="4000" height="3000" widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE" protect="0"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
        <hp:outMargin left="0" right="0" top="0" bottom="0"/>
        <hp:caption side="BOTTOM" fullSz="0" width="4000" gap="850" lastWidth="4000">
          <hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">
            <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
              <hp:run charPrIDRef="0"><hp:t>그림 1. 매출 추이</hp:t></hp:run>
            </hp:p>
          </hp:subList>
        </hp:caption>
      </hp:chart>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected chart (modeled as OLE) shape");
        };
        let caption = ole
            .caption
            .as_ref()
            .expect("<hp:caption> 이 ole.caption 에 적재돼야 한다 (#4319)");
        assert_eq!(caption.paragraphs.len(), 1);
        assert_eq!(caption.paragraphs[0].text, "그림 1. 매출 추이");
        assert!(
            ole.drawing.caption.is_none(),
            "HWP5 파서와 동형 정규화 — drawing.caption 은 비어 있어야 한다 \
             (shape_caption 게이트는 x.caption 만 본다)"
        );
    }

    /// [#4319] `<hp:ole>` 내부 `<hp:caption>` — chart 와 동일한 결함, 동일한 수정.
    #[test]
    fn issue4319_ole_caption_parses_into_caption_field() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
    <hp:run charPrIDRef="0">
      <hp:ole id="1" zOrder="0" numberingType="NONE" textWrap="SQUARE" textFlow="BOTH_SIDES" binaryItemIDRef="ole1" instid="1">
        <hp:sz width="4000" height="3000" widthRelTo="ABSOLUTE" heightRelTo="ABSOLUTE" protect="0"/>
        <hp:pos treatAsChar="0" vertRelTo="PARA" horzRelTo="PARA"/>
        <hp:outMargin left="0" right="0" top="0" bottom="0"/>
        <hp:caption side="BOTTOM" fullSz="0" width="4000" gap="850" lastWidth="4000">
          <hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0">
            <hp:p id="0" paraPrIDRef="0" styleIDRef="0">
              <hp:run charPrIDRef="0"><hp:t>수식 1. 표준편차 계산</hp:t></hp:run>
            </hp:p>
          </hp:subList>
        </hp:caption>
      </hp:ole>
      <hp:t/>
    </hp:run>
  </hp:p>
</hs:sec>"#;

        let section = parse_hwpx_section(xml).unwrap();
        let Control::Shape(shape) = &section.paragraphs[0].controls[0] else {
            panic!("expected shape control");
        };
        let ShapeObject::Ole(ole) = shape.as_ref() else {
            panic!("expected OLE shape");
        };
        let caption = ole
            .caption
            .as_ref()
            .expect("<hp:caption> 이 ole.caption 에 적재돼야 한다 (#4319)");
        assert_eq!(caption.paragraphs.len(), 1);
        assert_eq!(caption.paragraphs[0].text, "수식 1. 표준편차 계산");
        assert!(
            ole.drawing.caption.is_none(),
            "HWP5 파서와 동형 정규화 — drawing.caption 은 비어 있어야 한다"
        );
    }
}
