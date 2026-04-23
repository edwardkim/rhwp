//! header.xml 파싱 — HWPX 문서 메타데이터를 DocInfo로 변환
//!
//! header.xml은 글꼴, 글자모양, 문단모양, 스타일, 테두리/배경 등
//! 문서 전체에서 참조하는 리소스 테이블을 포함한다.

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::model::document::{DocInfo, DocProperties};
use crate::model::style::*;

use super::HwpxError;
use super::utils::{local_name, attr_str, parse_u8, parse_i8, parse_u16, parse_i16, parse_u32, parse_i32, parse_color, parse_bool};

/// `<hh:strikeout shape="..."/>` 의 shape 값이 실제 렌더링되는 취소선인지
/// 판정한다 (화이트리스트).
///
/// ## 배경
///
/// 한컴오피스 HWPX 익스포터는 본문 charPr 정의에 `<hh:strikeout shape="3D"/>`
/// 를 placeholder 기본값으로 넣어두는 경우가 많다. "3D"는 OWPML 스펙상
/// 유효한 취소선 모양이 아니며, 한컴 뷰어에서도 취소선으로 그리지 않는다.
/// 따라서 이를 진짜 strikethrough로 해석하면 정상 본문 전체가 취소선으로
/// 렌더링되는 버그가 생긴다.
///
/// 또한 한컴이 향후 다른 placeholder 값("Ghost", "4D" 등)을 추가할 가능성이
/// 있으므로, 블랙리스트(\"NONE\" | \"3D\" 제외)보다는 화이트리스트가 더
/// 안전하다. 알 수 없는 값은 fail-closed로 no-strike 처리한다.
///
/// ## 허용 값
///
/// 본 함수가 `true`를 반환하는 값은 OWPML `LineSym2` 열거(표 27 선 종류)와
/// shape.rs 의 `strike_shape` 매핑 표에서 모두 실제 선으로 인정되는 13종:
///
/// `SOLID`, `DASH`, `DOT`, `DASH_DOT`, `DASH_DOT_DOT`, `LONG_DASH`,
/// `CIRCLE`, `DOUBLE_SLIM`, `SLIM_THICK`, `THICK_SLIM`, `SLIM_THICK_SLIM`,
/// `WAVE`, `DOUBLE_WAVE`.
///
/// `NONE`, `3D`, 기타 모든 값은 `false` (취소선 없음).
pub(crate) fn is_real_strike_shape(shape: &str) -> bool {
    matches!(
        shape,
        "SOLID"
            | "DASH"
            | "DOT"
            | "DASH_DOT"
            | "DASH_DOT_DOT"
            | "LONG_DASH"
            | "CIRCLE"
            | "DOUBLE_SLIM"
            | "SLIM_THICK"
            | "THICK_SLIM"
            | "SLIM_THICK_SLIM"
            | "WAVE"
            | "DOUBLE_WAVE"
    )
}

/// header.xml을 파싱하여 DocInfo와 DocProperties를 생성한다.
pub fn parse_hwpx_header(xml: &str) -> Result<(DocInfo, DocProperties), HwpxError> {
    let mut doc_info = DocInfo::default();
    let mut doc_props = DocProperties::default();

    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    // 기본값: 7개 언어별 빈 글꼴 목록
    doc_info.font_faces = vec![Vec::new(); 7];

    // 현재 <fontface lang="..."> 컨텍스트 추적
    // HANGUL=0, LATIN=1, HANJA=2, JAPANESE=3, OTHER=4, SYMBOL=5, USER=6
    let mut current_font_group: usize = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name(); let local = local_name(name.as_ref());
                match local {
                    b"fontface" => {
                        // <hh:fontface lang="HANGUL"> → 언어 그룹 설정
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"lang" {
                                current_font_group = match attr_str(&attr).as_str() {
                                    "HANGUL" => 0,
                                    "LATIN" => 1,
                                    "HANJA" => 2,
                                    "JAPANESE" => 3,
                                    "OTHER" => 4,
                                    "SYMBOL" => 5,
                                    "USER" => 6,
                                    _ => 0,
                                };
                            }
                        }
                    }
                    b"beginNum" => parse_begin_num(e, &mut doc_props),
                    b"font" => parse_font(e, &mut doc_info, current_font_group),
                    b"charPr" => {
                        parse_char_shape(e, &mut reader, &mut doc_info)?;
                    }
                    b"paraPr" => {
                        parse_para_shape(e, &mut reader, &mut doc_info)?;
                    }
                    b"style" => parse_style(e, &mut doc_info),
                    b"borderFill" => {
                        parse_border_fill(e, &mut reader, &mut doc_info)?;
                    }
                    b"tabPr" => {
                        parse_tab_def(e, &mut reader, &mut doc_info)?;
                    }
                    b"numbering" => {
                        parse_numbering(e, &mut reader, &mut doc_info)?;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name = e.name(); let local = local_name(name.as_ref());
                match local {
                    b"beginNum" => parse_begin_num(e, &mut doc_props),
                    b"font" => parse_font(e, &mut doc_info, current_font_group),
                    b"style" => parse_style(e, &mut doc_info),
                    b"tabPr" => {
                        // 자기 닫힘 태그: 빈 TabDef만 push
                        let mut td = TabDef::default();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"autoTabLeft" => td.auto_tab_left = attr_str(&attr) == "1",
                                b"autoTabRight" => td.auto_tab_right = attr_str(&attr) == "1",
                                _ => {}
                            }
                        }
                        doc_info.tab_defs.push(td);
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("header.xml: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    doc_props.section_count = 1; // content.hpf에서 갱신됨

    Ok((doc_info, doc_props))
}

// ─── beginNum ───

fn parse_begin_num(e: &quick_xml::events::BytesStart, props: &mut DocProperties) {
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"page" => props.page_start_num = parse_u16(&attr),
            b"footnote" => props.footnote_start_num = parse_u16(&attr),
            b"endnote" => props.endnote_start_num = parse_u16(&attr),
            b"pic" => props.picture_start_num = parse_u16(&attr),
            b"tbl" => props.table_start_num = parse_u16(&attr),
            b"equation" => props.equation_start_num = parse_u16(&attr),
            _ => {}
        }
    }
}

// ─── Font ───

fn parse_font(e: &quick_xml::events::BytesStart, doc_info: &mut DocInfo, font_group: usize) {
    let mut name = String::new();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"face" => name = attr_str(&attr),
            _ => {}
        }
    }

    if !name.is_empty() {
        let font = Font {
            name,
            ..Default::default()
        };
        // fontface lang 컨텍스트에 따라 해당 언어 그룹에 추가
        if font_group < doc_info.font_faces.len() {
            doc_info.font_faces[font_group].push(font);
        }
    }
}

// ─── CharShape ───

fn parse_char_shape(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    doc_info: &mut DocInfo,
) -> Result<(), HwpxError> {
    let mut cs = CharShape::default();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"height" => cs.base_size = parse_i32(&attr),
            b"textColor" => cs.text_color = parse_color(&attr),
            b"shadeColor" => cs.shade_color = parse_color(&attr),
            b"useFontSpace" | b"useKerning" | b"symMark" => {}
            b"borderFillIDRef" => cs.border_fill_id = parse_u16(&attr),
            _ => {}
        }
    }

    // 자식 요소 파싱
    if !is_empty_event(e) {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref ce)) | Ok(Event::Start(ref ce)) => {
                    let cname = ce.name(); let local = local_name(cname.as_ref());
                    match local {
                        b"fontRef" => {
                            for attr in ce.attributes().flatten() {
                                let val = parse_u16(&attr);
                                match attr.key.as_ref() {
                                    b"hangul" => cs.font_ids[0] = val,
                                    b"latin" => cs.font_ids[1] = val,
                                    b"hanja" => cs.font_ids[2] = val,
                                    b"japanese" => cs.font_ids[3] = val,
                                    b"other" => cs.font_ids[4] = val,
                                    b"symbol" => cs.font_ids[5] = val,
                                    b"user" => cs.font_ids[6] = val,
                                    _ => {}
                                }
                            }
                        }
                        b"ratio" => {
                            for attr in ce.attributes().flatten() {
                                let val = parse_u8(&attr);
                                match attr.key.as_ref() {
                                    b"hangul" => cs.ratios[0] = val,
                                    b"latin" => cs.ratios[1] = val,
                                    b"hanja" => cs.ratios[2] = val,
                                    b"japanese" => cs.ratios[3] = val,
                                    b"other" => cs.ratios[4] = val,
                                    b"symbol" => cs.ratios[5] = val,
                                    b"user" => cs.ratios[6] = val,
                                    _ => {}
                                }
                            }
                        }
                        b"spacing" => {
                            for attr in ce.attributes().flatten() {
                                let val = parse_i8(&attr);
                                match attr.key.as_ref() {
                                    b"hangul" => cs.spacings[0] = val,
                                    b"latin" => cs.spacings[1] = val,
                                    b"hanja" => cs.spacings[2] = val,
                                    b"japanese" => cs.spacings[3] = val,
                                    b"other" => cs.spacings[4] = val,
                                    b"symbol" => cs.spacings[5] = val,
                                    b"user" => cs.spacings[6] = val,
                                    _ => {}
                                }
                            }
                        }
                        b"relSz" => {
                            for attr in ce.attributes().flatten() {
                                let val = parse_u8(&attr);
                                match attr.key.as_ref() {
                                    b"hangul" => cs.relative_sizes[0] = val,
                                    b"latin" => cs.relative_sizes[1] = val,
                                    b"hanja" => cs.relative_sizes[2] = val,
                                    b"japanese" => cs.relative_sizes[3] = val,
                                    b"other" => cs.relative_sizes[4] = val,
                                    b"symbol" => cs.relative_sizes[5] = val,
                                    b"user" => cs.relative_sizes[6] = val,
                                    _ => {}
                                }
                            }
                        }
                        b"offset" => {
                            for attr in ce.attributes().flatten() {
                                let val = parse_i8(&attr);
                                match attr.key.as_ref() {
                                    b"hangul" => cs.char_offsets[0] = val,
                                    b"latin" => cs.char_offsets[1] = val,
                                    b"hanja" => cs.char_offsets[2] = val,
                                    b"japanese" => cs.char_offsets[3] = val,
                                    b"other" => cs.char_offsets[4] = val,
                                    b"symbol" => cs.char_offsets[5] = val,
                                    b"user" => cs.char_offsets[6] = val,
                                    _ => {}
                                }
                            }
                        }
                        b"bold" => cs.bold = true,
                        b"italic" => cs.italic = true,
                        b"underline" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => {
                                        cs.underline_type = match attr_str(&attr).as_str() {
                                            "BOTTOM" => UnderlineType::Bottom,
                                            "TOP" => UnderlineType::Top,
                                            _ => UnderlineType::None,
                                        };
                                    }
                                    b"color" => {
                                        cs.underline_color = parse_color(&attr);
                                    }
                                    b"shape" => {
                                        // 밑줄 모양 13종 (표 27 선 종류 + 물결선)
                                        cs.underline_shape = match attr_str(&attr).as_str() {
                                            "SOLID" => 0,
                                            "DASH" => 1,
                                            "DOT" => 2,
                                            "DASH_DOT" => 3,
                                            "DASH_DOT_DOT" => 4,
                                            "LONG_DASH" => 5,
                                            "CIRCLE" => 6,
                                            "DOUBLE_SLIM" => 7,
                                            "SLIM_THICK" => 8,
                                            "THICK_SLIM" => 9,
                                            "SLIM_THICK_SLIM" => 10,
                                            "WAVE" => 11,
                                            "DOUBLE_WAVE" => 12,
                                            _ => 0,
                                        };
                                    }
                                    _ => {}
                                }
                            }
                        }
                        b"strikeout" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"shape" => {
                                        let val = attr_str(&attr);
                                        // 화이트리스트 방식: 한컴이 실제 렌더링하는
                                        // OWPML LineSym2 값만 취소선으로 인정한다.
                                        // "NONE", "3D" 같은 placeholder 및 알 수 없는
                                        // 값은 fail-closed로 no-strike 처리.
                                        // is_real_strike_shape() 독스트링 참고.
                                        cs.strikethrough = is_real_strike_shape(&val);
                                        cs.strike_shape = match val.as_str() {
                                            "SOLID" => 0,
                                            "DASH" => 1,
                                            "DOT" => 2,
                                            "DASH_DOT" => 3,
                                            "DASH_DOT_DOT" => 4,
                                            "LONG_DASH" => 5,
                                            "CIRCLE" => 6,
                                            "DOUBLE_SLIM" => 7,
                                            "SLIM_THICK" => 8,
                                            "THICK_SLIM" => 9,
                                            "SLIM_THICK_SLIM" => 10,
                                            "WAVE" => 11,
                                            "DOUBLE_WAVE" => 12,
                                            _ => 0,
                                        };
                                    }
                                    b"color" => {
                                        cs.strike_color = parse_color(&attr);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        b"outline" => {
                            for attr in ce.attributes().flatten() {
                                if attr.key.as_ref() == b"type" {
                                    let val = attr_str(&attr);
                                    cs.outline_type = match val.as_str() {
                                        "NONE" => 0,
                                        "SOLID" => 1,
                                        "DASH" => 2,
                                        "DOT" => 3,
                                        _ => 0,
                                    };
                                }
                            }
                        }
                        b"shadow" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => {
                                        let val = attr_str(&attr);
                                        cs.shadow_type = match val.as_str() {
                                            "NONE" => 0,
                                            "DROP" | "CONTINUOUS" => 1,
                                            _ => 0,
                                        };
                                    }
                                    b"color" => cs.shadow_color = parse_color(&attr),
                                    _ => {}
                                }
                            }
                        }
                        b"emboss" => { cs.attr |= 1 << 13; cs.emboss = true; }
                        b"engrave" => { cs.attr |= 1 << 14; cs.engrave = true; }
                        b"supscript" => cs.superscript = true,
                        b"subscript" => cs.subscript = true,
                        _ => {}
                    }
                }
                Ok(Event::End(ref ee)) => {
                    let ename = ee.name(); if local_name(ename.as_ref()) == b"charPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(HwpxError::XmlError(format!("charPr: {}", e))),
                _ => {}
            }
            buf.clear();
        }
    }

    doc_info.char_shapes.push(cs);
    Ok(())
}

// ─── ParaShape ───

fn parse_para_shape(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    doc_info: &mut DocInfo,
) -> Result<(), HwpxError> {
    let mut ps = ParaShape::default();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"tabPrIDRef" => ps.tab_def_id = parse_u16(&attr),
            b"condense" => {}
            _ => {}
        }
    }

    if !is_empty_event(e) {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref ce)) | Ok(Event::Start(ref ce)) => {
                    let cname = ce.name(); let local = local_name(cname.as_ref());
                    match local {
                        b"align" => {
                            for attr in ce.attributes().flatten() {
                                if attr.key.as_ref() == b"horizontal" {
                                    ps.alignment = parse_alignment(&attr);
                                }
                            }
                        }
                        b"heading" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => {
                                        let val = attr_str(&attr);
                                        ps.head_type = match val.as_str() {
                                            "OUTLINE" => HeadType::Outline,
                                            "NUMBER" | "NUMBERING" => HeadType::Number,
                                            "BULLET" => HeadType::Bullet,
                                            _ => HeadType::None,
                                        };
                                    }
                                    b"idRef" => ps.numbering_id = parse_u16(&attr),
                                    b"level" => ps.para_level = parse_u8(&attr),
                                    _ => {}
                                }
                            }
                        }
                        b"margin" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"left" => ps.margin_left = parse_i32(&attr),
                                    b"right" => ps.margin_right = parse_i32(&attr),
                                    b"indent" => ps.indent = parse_i32(&attr),
                                    b"prev" => ps.spacing_before = parse_i32(&attr),
                                    b"next" => ps.spacing_after = parse_i32(&attr),
                                    _ => {}
                                }
                            }
                        }
                        b"lineSpacing" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => {
                                        let val = attr_str(&attr);
                                        ps.line_spacing_type = match val.as_str() {
                                            "PERCENT" => LineSpacingType::Percent,
                                            "FIXED" => LineSpacingType::Fixed,
                                            "SPACEONLY" | "SPACE_ONLY" => LineSpacingType::SpaceOnly,
                                            "MINIMUM" | "AT_LEAST" => LineSpacingType::Minimum,
                                            _ => LineSpacingType::Percent,
                                        };
                                    }
                                    b"value" => ps.line_spacing = parse_i32(&attr),
                                    _ => {}
                                }
                            }
                        }
                        b"border" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"borderFillIDRef" => ps.border_fill_id = parse_u16(&attr),
                                    b"offsetLeft" => ps.border_spacing[0] = parse_i16(&attr),
                                    b"offsetRight" => ps.border_spacing[1] = parse_i16(&attr),
                                    b"offsetTop" => ps.border_spacing[2] = parse_i16(&attr),
                                    b"offsetBottom" => ps.border_spacing[3] = parse_i16(&attr),
                                    _ => {}
                                }
                            }
                        }
                        b"breakSetting" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"widowOrphan" => if parse_bool(&attr) {
                                        ps.attr2 |= 1 << 5;
                                    },
                                    b"keepWithNext" => if parse_bool(&attr) {
                                        ps.attr2 |= 1 << 6;
                                    },
                                    b"keepLines" => if parse_bool(&attr) {
                                        ps.attr2 |= 1 << 7;
                                    },
                                    b"pageBreakBefore" => if parse_bool(&attr) {
                                        ps.attr2 |= 1 << 8;
                                    },
                                    _ => {}
                                }
                            }
                        }
                        b"autoSpacing" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"eAsianEng" => if parse_bool(&attr) {
                                        ps.attr1 |= 1 << 20;
                                    },
                                    b"eAsianNum" => if parse_bool(&attr) {
                                        ps.attr1 |= 1 << 21;
                                    },
                                    _ => {}
                                }
                            }
                        }
                        b"switch" => {
                            // <switch>/<case>/<default> 네임스페이스 분기 처리
                            // HwpUnitChar case를 우선 적용, 없으면 default 사용
                            parse_para_shape_switch(reader, &mut ps)?;
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref ee)) => {
                    let ename = ee.name(); if local_name(ename.as_ref()) == b"paraPr" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(HwpxError::XmlError(format!("paraPr: {}", e))),
                _ => {}
            }
            buf.clear();
        }
    }

    doc_info.para_shapes.push(ps);
    Ok(())
}

/// `<switch>` 내부의 `<case>`/`<default>` 분기에서 margin, lineSpacing을 파싱.
/// HwpUnitChar 네임스페이스 case를 우선 적용한다.
fn parse_para_shape_switch(
    reader: &mut Reader<&[u8]>,
    ps: &mut ParaShape,
) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    let mut in_hwpunitchar_case = false;
    let mut in_default = false;
    let mut found_case = false;
    // default 값을 임시 저장 (case가 없을 때 폴백)
    let mut def_margin_left: Option<i32> = None;
    let mut def_margin_right: Option<i32> = None;
    let mut def_indent: Option<i32> = None;
    let mut def_prev: Option<i32> = None;
    let mut def_next: Option<i32> = None;
    let mut def_line_spacing_type: Option<LineSpacingType> = None;
    let mut def_line_spacing: Option<i32> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                match local {
                    b"case" => {
                        // required-namespace 속성 확인
                        let is_hwpunitchar = ce.attributes().flatten().any(|attr| {
                            let val = attr_str(&attr);
                            val.contains("HwpUnitChar")
                        });
                        if is_hwpunitchar {
                            in_hwpunitchar_case = true;
                        }
                    }
                    b"default" => {
                        in_default = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref ce)) => {
                let cname = ce.name(); let local = local_name(cname.as_ref());
                if in_hwpunitchar_case || in_default {
                    match local {
                        b"margin" | b"intent" | b"left" | b"right" | b"prev" | b"next" => {
                            // margin 하위 요소들: <left value="..." />, <prev value="..." /> 등
                            let tag_name = local;
                            for attr in ce.attributes().flatten() {
                                if attr.key.as_ref() == b"value" {
                                    let val = parse_i32(&attr);
                                    if in_hwpunitchar_case {
                                        // HwpUnitChar 값은 실제 HWPUNIT(1× 스케일)이므로
                                        // HWP 바이너리와 동일한 2× 스케일로 변환
                                        let val2x = val * 2;
                                        match tag_name {
                                            b"left" => ps.margin_left = val2x,
                                            b"right" => ps.margin_right = val2x,
                                            b"intent" => ps.indent = val2x,
                                            b"prev" => ps.spacing_before = val2x,
                                            b"next" => ps.spacing_after = val2x,
                                            _ => {}
                                        }
                                        found_case = true;
                                    } else if in_default {
                                        match tag_name {
                                            b"left" => def_margin_left = Some(val),
                                            b"right" => def_margin_right = Some(val),
                                            b"intent" => def_indent = Some(val),
                                            b"prev" => def_prev = Some(val),
                                            b"next" => def_next = Some(val),
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                        b"lineSpacing" => {
                            let mut ls_type = None;
                            let mut ls_val = None;
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => {
                                        ls_type = Some(match attr_str(&attr).as_str() {
                                            "PERCENT" => LineSpacingType::Percent,
                                            "FIXED" => LineSpacingType::Fixed,
                                            "SPACEONLY" | "SPACE_ONLY" => LineSpacingType::SpaceOnly,
                                            "MINIMUM" | "AT_LEAST" => LineSpacingType::Minimum,
                                            _ => LineSpacingType::Percent,
                                        });
                                    }
                                    b"value" => ls_val = Some(parse_i32(&attr)),
                                    _ => {}
                                }
                            }
                            if in_hwpunitchar_case {
                                if let Some(t) = ls_type { ps.line_spacing_type = t; }
                                if let Some(v) = ls_val {
                                    // Fixed/SpaceOnly/Minimum은 HWPUNIT이므로 2× 스케일 변환
                                    let effective_type = ls_type.unwrap_or(ps.line_spacing_type);
                                    ps.line_spacing = match effective_type {
                                        LineSpacingType::Percent => v,
                                        _ => v * 2,
                                    };
                                }
                                found_case = true;
                            } else if in_default {
                                def_line_spacing_type = ls_type;
                                def_line_spacing = ls_val;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref ee)) => {
                let ename = ee.name(); let local = local_name(ename.as_ref());
                match local {
                    b"case" => { in_hwpunitchar_case = false; }
                    b"default" => { in_default = false; }
                    b"switch" => break,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("switch: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // HwpUnitChar case가 없으면 default 값 적용
    if !found_case {
        if let Some(v) = def_margin_left { ps.margin_left = v; }
        if let Some(v) = def_margin_right { ps.margin_right = v; }
        if let Some(v) = def_indent { ps.indent = v; }
        if let Some(v) = def_prev { ps.spacing_before = v; }
        if let Some(v) = def_next { ps.spacing_after = v; }
        if let Some(t) = def_line_spacing_type { ps.line_spacing_type = t; }
        if let Some(v) = def_line_spacing { ps.line_spacing = v; }
    }

    Ok(())
}

// ─── Style ───

fn parse_style(e: &quick_xml::events::BytesStart, doc_info: &mut DocInfo) {
    let mut style = Style::default();
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name" => style.local_name = attr_str(&attr),
            b"engName" => style.english_name = attr_str(&attr),
            b"type" => {
                let val = attr_str(&attr);
                style.style_type = match val.as_str() {
                    "PARA" | "PARAGRAPH" => 0,
                    "CHAR" | "CHARACTER" => 1,
                    _ => 0,
                };
            }
            b"paraPrIDRef" => style.para_shape_id = parse_u16(&attr),
            b"charPrIDRef" => style.char_shape_id = parse_u16(&attr),
            b"nextStyleIDRef" => style.next_style_id = parse_u8(&attr),
            _ => {}
        }
    }
    doc_info.styles.push(style);
}

// ─── BorderFill ───

fn parse_border_fill(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    doc_info: &mut DocInfo,
) -> Result<(), HwpxError> {
    let mut bf = BorderFill::default();

    if !is_empty_event(e) {
        let mut buf = Vec::new();
        let mut border_idx = 0usize;
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref ce)) | Ok(Event::Start(ref ce)) => {
                    let cname = ce.name(); let local = local_name(cname.as_ref());
                    match local {
                        b"leftBorder" | b"rightBorder" | b"topBorder" | b"bottomBorder" => {
                            let idx = match local {
                                b"leftBorder" => 0,
                                b"rightBorder" => 1,
                                b"topBorder" => 2,
                                b"bottomBorder" => 3,
                                _ => { border_idx += 1; border_idx - 1 }
                            };
                            if idx < 4 {
                                for attr in ce.attributes().flatten() {
                                    match attr.key.as_ref() {
                                        b"type" => bf.borders[idx].line_type = parse_border_line_type(&attr),
                                        b"width" => bf.borders[idx].width = parse_border_width(&attr),
                                        b"color" => bf.borders[idx].color = parse_color(&attr),
                                        _ => {}
                                    }
                                }
                            }
                        }
                        b"diagonal" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => bf.diagonal.diagonal_type = parse_u8(&attr),
                                    b"width" => bf.diagonal.width = parse_border_width(&attr),
                                    b"color" => bf.diagonal.color = parse_color(&attr),
                                    _ => {}
                                }
                            }
                        }
                        b"fillBrush" => {
                            // fillBrush 자식 요소를 파싱
                            // Start 이벤트이면 자식을 읽어야 함
                        }
                        b"winBrush" => {
                            bf.fill.fill_type = FillType::Solid;
                            let mut solid = SolidFill::default();
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"faceColor" => solid.background_color = parse_color(&attr),
                                    b"hatchColor" => solid.pattern_color = parse_color(&attr),
                                    b"alpha" => {
                                        // HWPX alpha: 0.0=완전투명 ~ 1.0=불투명 (float string)
                                        let val = attr_str(&attr);
                                        if let Ok(f) = val.parse::<f64>() {
                                            bf.fill.alpha = (f.clamp(0.0, 1.0) * 255.0) as u8;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            bf.fill.solid = Some(solid);
                        }
                        b"gradation" => {
                            bf.fill.fill_type = FillType::Gradient;
                            let mut grad = GradientFill::default();
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => grad.gradient_type = parse_i16(&attr),
                                    b"angle" => grad.angle = parse_i16(&attr),
                                    b"centerX" => grad.center_x = parse_i16(&attr),
                                    b"centerY" => grad.center_y = parse_i16(&attr),
                                    b"blur" => grad.blur = parse_i16(&attr),
                                    _ => {}
                                }
                            }
                            bf.fill.gradient = Some(grad);
                        }
                        b"color" => {
                            // <hh:color value="#RRGGBB"/> — gradation 자식
                            if let Some(ref mut grad) = bf.fill.gradient {
                                for attr in ce.attributes().flatten() {
                                    if attr.key.as_ref() == b"value" {
                                        grad.colors.push(parse_color(&attr));
                                    }
                                }
                            }
                        }
                        b"imgBrush" => {
                            bf.fill.fill_type = FillType::Image;
                            let mut img_fill = ImageFill::default();
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"mode" => {
                                        img_fill.fill_mode = match attr_str(&attr).as_str() {
                                            "TILE" | "TILE_ALL" => ImageFillMode::TileAll,
                                            "TILE_HORZ_TOP" => ImageFillMode::TileHorzTop,
                                            "TILE_HORZ_BOTTOM" => ImageFillMode::TileHorzBottom,
                                            "TILE_VERT_LEFT" => ImageFillMode::TileVertLeft,
                                            "TILE_VERT_RIGHT" => ImageFillMode::TileVertRight,
                                            "CENTER" => ImageFillMode::Center,
                                            "CENTER_TOP" => ImageFillMode::CenterTop,
                                            "CENTER_BOTTOM" => ImageFillMode::CenterBottom,
                                            "FIT" | "FIT_TO_SIZE" | "STRETCH" | "TOTAL" => ImageFillMode::FitToSize,
                                            "TOP_LEFT_ALIGN" => ImageFillMode::LeftTop,
                                            _ => ImageFillMode::TileAll,
                                        };
                                    }
                                    b"bright" => img_fill.brightness = parse_i8(&attr),
                                    b"contrast" => img_fill.contrast = parse_i8(&attr),
                                    _ => {}
                                }
                            }
                            bf.fill.image = Some(img_fill);
                        }
                        b"img" | b"image" => {
                            // imgBrush 내부의 이미지 참조
                            if let Some(ref mut img_fill) = bf.fill.image {
                                for attr in ce.attributes().flatten() {
                                    if attr.key.as_ref() == b"binaryItemIDRef" {
                                        let val = attr_str(&attr);
                                        let num: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                                        img_fill.bin_data_id = num.parse().unwrap_or(0);
                                    }
                                }
                            }
                        }
                        b"slash" => {
                            for attr in ce.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"type" => bf.diagonal.diagonal_type = parse_u8(&attr),
                                    b"width" => bf.diagonal.width = parse_border_width(&attr),
                                    b"color" => bf.diagonal.color = parse_color(&attr),
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref ee)) => {
                    let ename = ee.name(); if local_name(ename.as_ref()) == b"borderFill" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(HwpxError::XmlError(format!("borderFill: {}", e))),
                _ => {}
            }
            buf.clear();
        }
    }

    doc_info.border_fills.push(bf);
    Ok(())
}

// ─── TabDef ───

fn parse_tab_item(ce: &quick_xml::events::BytesStart) -> TabItem {
    let mut item = TabItem::default();
    for attr in ce.attributes().flatten() {
        match attr.key.as_ref() {
            b"pos" => item.position = parse_u32(&attr),
            b"type" => {
                item.tab_type = match attr_str(&attr).as_str() {
                    "LEFT" => 0,
                    "RIGHT" => 1,
                    "CENTER" => 2,
                    "DECIMAL" => 3,
                    _ => 0,
                };
            }
            b"leader" => {
                // HWP fill_type: 0=없음, 1=실선, 2=파선, 3=점선,
                // 4=일점쇄선, 5=이점쇄선, 6=긴파선, 7=원형점선,
                // 8=이중실선, 9=얇고굵은이중선, 10=굵고얇은이중선, 11=삼중선
                // HWPML leader 명칭은 HWP 바이너리 fill_type과 직접 대응
                // "DASH"=점선(3), "DOT"=파선(2) — HWPML 명명이 직관과 반대
                item.fill_type = match attr_str(&attr).as_str() {
                    "NONE" => 0,
                    "SOLID" => 1,
                    "DOT" => 2,         // 파선
                    "DASH" => 3,        // 점선
                    "DASH_DOT" => 4,    // 일점쇄선
                    "DASH_DOT_DOT" => 5,// 이점쇄선
                    "LONG_DASH" => 6,   // 긴파선
                    "CIRCLE" => 7,      // 원형점선
                    "DOUBLE_LINE" => 8, // 이중실선
                    "THIN_THICK" => 9,  // 얇고 굵은 이중선
                    "THICK_THIN" => 10, // 굵고 얇은 이중선
                    "TRIM" => 11,       // 얇고 굵고 얇은 삼중선
                    _ => 0,
                };
            }
            _ => {}
        }
    }
    item
}

fn parse_tab_def(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    doc_info: &mut DocInfo,
) -> Result<(), HwpxError> {
    let mut td = TabDef::default();

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"autoTabLeft" => td.auto_tab_left = attr_str(&attr) == "1",
            b"autoTabRight" => td.auto_tab_right = attr_str(&attr) == "1",
            _ => {}
        }
    }

    if !is_empty_event(e) {
        let mut buf = Vec::new();
        let mut in_hwpunitchar_case = false;
        let mut in_default = false;
        let mut found_case = false;
        let mut default_tabs: Vec<TabItem> = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref ce)) => {
                    let cname = ce.name(); let local = local_name(cname.as_ref());
                    match local {
                        b"case" => {
                            let is_hwpunitchar = ce.attributes().flatten().any(|attr| {
                                attr_str(&attr).contains("HwpUnitChar")
                            });
                            if is_hwpunitchar {
                                in_hwpunitchar_case = true;
                            }
                        }
                        b"default" => {
                            in_default = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref ce)) => {
                    let cname = ce.name(); let local = local_name(cname.as_ref());
                    if local == b"tabItem" {
                        let mut item = parse_tab_item(ce);
                        if in_hwpunitchar_case {
                            // HwpUnitChar 값은 실제 HWPUNIT(1× 스케일)이므로
                            // HWP 바이너리와 동일한 2× 스케일로 변환
                            item.position *= 2;
                            td.tabs.push(item);
                            found_case = true;
                        } else if in_default {
                            // default 값은 이미 2× 스케일
                            default_tabs.push(item);
                        } else {
                            // switch 바깥의 직접 tabItem (단위 불명, 그대로 사용)
                            td.tabs.push(item);
                        }
                    }
                }
                Ok(Event::End(ref ee)) => {
                    let ename = ee.name(); let local = local_name(ename.as_ref());
                    match local {
                        b"case" => { in_hwpunitchar_case = false; }
                        b"default" => { in_default = false; }
                        b"tabPr" => break,
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(HwpxError::XmlError(format!("tabPr: {}", e))),
                _ => {}
            }
            buf.clear();
        }
        // HwpUnitChar case가 없으면 default 값 적용
        if !found_case && !default_tabs.is_empty() {
            td.tabs = default_tabs;
        }
    }

    doc_info.tab_defs.push(td);
    Ok(())
}

// ─── Numbering ───

fn parse_numbering(
    e: &quick_xml::events::BytesStart,
    reader: &mut Reader<&[u8]>,
    doc_info: &mut DocInfo,
) -> Result<(), HwpxError> {
    let mut num = Numbering::default();

    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"start" {
            num.start_number = parse_u16(&attr);
        }
    }

    if !is_empty_event(e) {
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref ce)) | Ok(Event::Start(ref ce)) => {
                    let cname = ce.name(); let local = local_name(cname.as_ref());
                    if local == b"paraHead" {
                        let mut level: usize = 0;
                        let mut head = NumberingHead::default();
                        let mut format_str = String::new();
                        for attr in ce.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"level" => level = parse_u32(&attr) as usize,
                                b"start" => {
                                    let s = parse_u32(&attr);
                                    if level > 0 && level <= 7 {
                                        num.level_start_numbers[level - 1] = s;
                                    }
                                }
                                b"text" => format_str = attr_str(&attr),
                                b"numFormat" => head.number_format = parse_u8(&attr),
                                b"charPrIDRef" => head.char_shape_id = parse_u32(&attr),
                                _ => {}
                            }
                        }
                        if level > 0 && level <= 7 {
                            num.heads[level - 1] = head;
                            num.level_formats[level - 1] = format_str;
                        }
                    }
                }
                Ok(Event::End(ref ee)) => {
                    let ename = ee.name(); if local_name(ename.as_ref()) == b"numbering" {
                        break;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(HwpxError::XmlError(format!("numbering: {}", e))),
                _ => {}
            }
            buf.clear();
        }
    }

    doc_info.numberings.push(num);
    Ok(())
}

// ─── 유틸리티 함수 (header 전용) ───

fn is_empty_event(_e: &quick_xml::events::BytesStart) -> bool {
    // quick-xml의 Event::Empty vs Event::Start 구분으로 판단
    // 호출측에서 Empty/Start 구분 없이 패턴 매칭하므로 항상 false 반환
    // (자식 파싱 루프가 End 태그에서 break하므로 안전)
    false
}

fn parse_alignment(attr: &quick_xml::events::attributes::Attribute) -> Alignment {
    match attr_str(attr).as_str() {
        "JUSTIFY" => Alignment::Justify,
        "LEFT" => Alignment::Left,
        "RIGHT" => Alignment::Right,
        "CENTER" => Alignment::Center,
        "DISTRIBUTE" => Alignment::Distribute,
        _ => Alignment::Justify,
    }
}

fn parse_border_line_type(attr: &quick_xml::events::attributes::Attribute) -> BorderLineType {
    match attr_str(attr).as_str() {
        "NONE" => BorderLineType::None,
        "SOLID" => BorderLineType::Solid,
        "DASH" => BorderLineType::Dash,
        "DOT" => BorderLineType::Dot,
        "DASH_DOT" => BorderLineType::DashDot,
        "DASH_DOT_DOT" => BorderLineType::DashDotDot,
        "LONG_DASH" => BorderLineType::LongDash,
        "CIRCLE" => BorderLineType::Circle,
        "DOUBLE_SLIM" | "DOUBLE" => BorderLineType::Double,
        "SLIM_THICK" => BorderLineType::ThinThickDouble,
        "THICK_SLIM" => BorderLineType::ThickThinDouble,
        "SLIM_THICK_SLIM" => BorderLineType::ThinThickThinTriple,
        "WAVE" => BorderLineType::Wave,
        "DOUBLE_WAVE" => BorderLineType::DoubleWave,
        _ => BorderLineType::Solid,
    }
}

fn parse_border_width(attr: &quick_xml::events::attributes::Attribute) -> u8 {
    let s = attr_str(attr);
    // "0.12 mm", "0.4 mm" 등의 형식에서 두께 인덱스 추출
    let mm: f64 = s.split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.12);
    // 대략적인 HWP 두께 인덱스 매핑
    if mm <= 0.12 { 0 }
    else if mm <= 0.3 { 1 }
    else if mm <= 0.5 { 2 }
    else if mm <= 1.0 { 3 }
    else if mm <= 1.5 { 4 }
    else { 5 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color_rgb() {
        let attr_data = b"#FF0000";
        // 빨강: RRGGBB → 0x000000FF (BBGGRR)
        let xml = r##"<e color="#FF0000"/>"##.to_string();
        let mut reader = Reader::from_str(&xml);
        let mut buf = Vec::new();
        if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"color" {
                    assert_eq!(parse_color(&attr), 0x000000FF);
                }
            }
        }
    }

    #[test]
    fn test_parse_color_none() {
        let xml = r#"<e color="none"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"color" {
                    assert_eq!(parse_color(&attr), 0xFFFFFFFF);
                }
            }
        }
    }

    #[test]
    fn test_parse_alignment() {
        let xml = r#"<e horizontal="CENTER"/>"#;
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::new();
        if let Ok(Event::Empty(ref e)) = reader.read_event_into(&mut buf) {
            for attr in e.attributes().flatten() {
                if attr.key.as_ref() == b"horizontal" {
                    assert_eq!(parse_alignment(&attr), Alignment::Center);
                }
            }
        }
    }

    #[test]
    fn test_is_real_strike_shape_valid_shapes() {
        // OWPML LineSym2 전체 — 모두 true
        for shape in &[
            "SOLID",
            "DASH",
            "DOT",
            "DASH_DOT",
            "DASH_DOT_DOT",
            "LONG_DASH",
            "CIRCLE",
            "DOUBLE_SLIM",
            "SLIM_THICK",
            "THICK_SLIM",
            "SLIM_THICK_SLIM",
            "WAVE",
            "DOUBLE_WAVE",
        ] {
            assert!(
                is_real_strike_shape(shape),
                "{} should be a real strike shape",
                shape
            );
        }
    }

    #[test]
    fn test_is_real_strike_shape_placeholder_none() {
        assert!(!is_real_strike_shape("NONE"));
    }

    #[test]
    fn test_is_real_strike_shape_placeholder_3d() {
        // 한컴 익스포터의 대표 placeholder — 본문 전체가 취소선으로 찍히던 버그
        assert!(!is_real_strike_shape("3D"));
    }

    #[test]
    fn test_is_real_strike_shape_unknown_fail_closed() {
        // 미래에 한컴이 추가할 수 있는 placeholder. 블랙리스트였다면 true로
        // 오인식되어 본문에 취소선이 그려질 것이다. 화이트리스트는 false.
        assert!(!is_real_strike_shape("4D"));
        assert!(!is_real_strike_shape("Ghost"));
        assert!(!is_real_strike_shape(""));
        assert!(!is_real_strike_shape("solid")); // 대소문자 구분
    }
}
