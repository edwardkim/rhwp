//! Contents/header.xml — DocInfo 리소스 테이블 동적 직렬화.
//!
//! Stage 1 (#182): IR의 `doc_info` 에 담긴 리소스를 역방향으로 HWPX XML로 출력한다.
//! IR이 비어있으면 해당 섹션도 비어있게 출력한다 (IR에 없는 리소스를 자동 생성하지 않음).
//!
//! 속성·자식 순서는 한컴 OWPML 공식 구현(hancom-io/hwpx-owpml-model, Apache 2.0)의
//! `Class/Head/*.cpp` 파일 `WriteElement()`, `InitMap()` 을 기준으로 맞춘다.
//!
//! ## 범위
//!
//! - 1단계 목표: 기존 HWPX 문서를 parse→serialize 했을 때 한컴2020이 온전히 다시 연다
//! - 완전히 새 빈 문서 생성은 1단계 범위 밖 (기본값 채우기 로직 없음)

use std::io::Write;

use quick_xml::Writer;

use crate::model::document::{DocInfo, DocProperties, Document};
use crate::model::style::{
    border_width_mm_str, Alignment, BorderFill, BorderLine, BorderLineType, CenterLine, CharShape,
    DiagonalLine, FillType, Font, HeadType, LineSpacingType, Numbering, ParaShape, Style,
    SubstFont, TabDef,
};
use crate::model::ColorRef;

use super::canonical_defaults::FONTFACE_LANG_NAMES;
use super::context::SerializeContext;
use super::utils::{empty_tag, end_tag, start_tag_attrs, write_xml_decl};
use super::SerializeError;

/// `header.xml` 바이트 생성. Stage 1 진입점.
pub fn write_header(doc: &Document, ctx: &SerializeContext) -> Result<Vec<u8>, SerializeError> {
    let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
    write_xml_decl(&mut w)?;

    // <hh:head> 루트 + 전체 네임스페이스 (parser가 기대하는 접두어 모두 선언)
    // secCnt 는 실제 직렬화하는 섹션 파일 수(`doc.sections`)와 일치해야 한다 (#1557).
    // doc_properties.section_count 는 파서가 갱신하지 않아 stale(1)일 수 있어, 그대로
    // 쓰면 secCnt < 실제 섹션 수가 되어 한글이 뒤 구역을 로드하지 않고 페이지가 붕괴한다.
    let sec_cnt = doc.sections.len().max(1).to_string();
    // HWPML 스키마 버전: 원본 보존값(문서별 상이, 1.2~1.5). 없으면 "1.2" 폴백.
    let hwpml_version = doc.doc_info.hwpml_version.as_deref().unwrap_or("1.2");
    start_tag_attrs(
        &mut w,
        "hh:head",
        &[
            ("xmlns:ha", "http://www.hancom.co.kr/hwpml/2011/app"),
            ("xmlns:hp", "http://www.hancom.co.kr/hwpml/2011/paragraph"),
            ("xmlns:hp10", "http://www.hancom.co.kr/hwpml/2016/paragraph"),
            ("xmlns:hs", "http://www.hancom.co.kr/hwpml/2011/section"),
            ("xmlns:hc", "http://www.hancom.co.kr/hwpml/2011/core"),
            ("xmlns:hh", "http://www.hancom.co.kr/hwpml/2011/head"),
            ("xmlns:hhs", "http://www.hancom.co.kr/hwpml/2011/history"),
            ("xmlns:hm", "http://www.hancom.co.kr/hwpml/2011/master-page"),
            ("xmlns:dc", "http://purl.org/dc/elements/1.1/"),
            ("xmlns:opf", "http://www.idpf.org/2007/opf/"),
            ("xmlns:epub", "http://www.idpf.org/2007/ops"),
            (
                "xmlns:ooxmlchart",
                "http://www.hancom.co.kr/hwpml/2016/ooxmlchart",
            ),
            (
                "xmlns:hwpunitchar",
                "http://www.hancom.co.kr/hwpml/2016/HwpUnitChar",
            ),
            ("xmlns:hpf", "http://www.hancom.co.kr/schema/2011/hpf"),
            (
                "xmlns:config",
                "urn:oasis:names:tc:opendocument:xmlns:config:1.0",
            ),
            ("version", hwpml_version),
            ("secCnt", &sec_cnt),
        ],
    )?;

    write_begin_num(&mut w, &doc.doc_properties)?;

    // <hh:refList>: 모든 리소스 테이블을 감싸는 컨테이너
    super::utils::start_tag(&mut w, "hh:refList")?;
    write_fontfaces(&mut w, &doc.doc_info)?;
    write_border_fills(&mut w, &doc.doc_info, ctx)?;
    write_char_properties(&mut w, &doc.doc_info, ctx)?;
    write_tab_properties(&mut w, &doc.doc_info)?;
    write_numberings(&mut w, &doc.doc_info)?;
    write_bullets(&mut w, &doc.doc_info)?;
    write_para_properties(&mut w, &doc.doc_info, ctx)?;
    write_styles(&mut w, &doc.doc_info, ctx)?;
    end_tag(&mut w, "hh:refList")?;

    // 문서 설정 tail: 원본 HWPX 가 있으면 그대로 splice(compatibleDocument/
    // docOption/trackchageConfig 무손실 보존), 없으면 하드코딩 폴백.
    match &doc.doc_info.hwpx_head_tail {
        Some(tail) => {
            w.get_mut()
                .write_all(tail.as_bytes())
                .map_err(|e| SerializeError::XmlError(format!("head tail splice: {e}")))?;
        }
        None => {
            write_compatible_document(&mut w)?;
            write_doc_option(&mut w)?;
            write_track_change_config(&mut w)?;
        }
    }

    end_tag(&mut w, "hh:head")?;
    Ok(w.into_inner())
}

// =====================================================================
// <hh:beginNum>
// =====================================================================
fn write_begin_num<W: Write>(
    w: &mut Writer<W>,
    props: &DocProperties,
) -> Result<(), SerializeError> {
    empty_tag(
        w,
        "hh:beginNum",
        &[
            ("page", &props.page_start_num.max(1).to_string()),
            ("footnote", &props.footnote_start_num.max(1).to_string()),
            ("endnote", &props.endnote_start_num.max(1).to_string()),
            ("pic", &props.picture_start_num.max(1).to_string()),
            ("tbl", &props.table_start_num.max(1).to_string()),
            ("equation", &props.equation_start_num.max(1).to_string()),
        ],
    )
}

// =====================================================================
// <hh:fontfaces> — 7 언어 그룹
// =====================================================================
fn write_fontfaces<W: Write>(w: &mut Writer<W>, doc_info: &DocInfo) -> Result<(), SerializeError> {
    // IR의 font_faces는 항상 7개 언어 그룹을 유지한다고 기대하나,
    // 비어있거나 크기가 다를 수 있으므로 안전하게 처리.
    let groups: Vec<&Vec<Font>> = (0..7)
        .map(|i| doc_info.font_faces.get(i).unwrap_or(&EMPTY_FONT_VEC))
        .collect();

    let item_cnt = groups.iter().filter(|g| !g.is_empty()).count();
    if item_cnt == 0 {
        return Ok(());
    }

    start_tag_attrs(
        w,
        "hh:fontfaces",
        &[(
            "itemCnt",
            &groups.iter().filter(|g| !g.is_empty()).count().to_string(),
        )],
    )?;
    for (lang_idx, fonts) in groups.iter().enumerate() {
        if fonts.is_empty() {
            continue;
        }
        let lang = FONTFACE_LANG_NAMES[lang_idx];
        start_tag_attrs(
            w,
            "hh:fontface",
            &[("lang", lang), ("fontCnt", &fonts.len().to_string())],
        )?;
        for (id, font) in fonts.iter().enumerate() {
            let id_str = id.to_string();
            let font_attrs = [
                ("id", id_str.as_str()),
                ("face", font.name.as_str()),
                ("type", font_type_str(font.alt_type)),
                ("isEmbedded", "0"),
            ];
            // substFont(대체 글꼴)·typeInfo(파나포스 10바이트)가 IR에 있으면
            // 자식으로 복원한다. 원본 순서는 substFont → typeInfo. 둘 다 없으면
            // 종전대로 self-closing.
            if font.subst_font.is_some() || font.type_info.is_some() {
                start_tag_attrs(w, "hh:font", &font_attrs)?;
                if let Some(sf) = &font.subst_font {
                    write_subst_font(w, sf)?;
                }
                if let Some(ti) = &font.type_info {
                    write_font_type_info(w, ti)?;
                }
                end_tag(w, "hh:font")?;
            } else {
                empty_tag(w, "hh:font", &font_attrs)?;
            }
        }
        end_tag(w, "hh:fontface")?;
    }
    end_tag(w, "hh:fontfaces")?;
    Ok(())
}

static EMPTY_FONT_VEC: Vec<Font> = Vec::new();

fn font_type_str(alt_type: u8) -> &'static str {
    match alt_type {
        1 => "TTF",
        2 => "HFT",
        _ => "TTF", // 기본: TTF (한컴 샘플 관찰값)
    }
}

/// `parse_font_type_info` 의 역함수.
///
/// IR 의 `type_info` 10바이트 배열을 `<hh:typeInfo>` 엘리먼트로 복원한다.
/// 바이트 배치(파서와 동일): [0]=familyType, [1]=serifType(합성값, XML 미노출),
/// [2]=weight, [3]=proportion, [4]=contrast, [5]=strokeVariation,
/// [6]=armStyle, [7]=letterform, [8]=midline, [9]=xHeight.
/// `[1]` 은 파서가 글꼴 이름/유형에서 합성하므로 재파싱 시 동일하게 재생성된다 —
/// 따라서 직렬화하지 않아도 라운드트립이 정확하다.
fn write_font_type_info<W: Write>(w: &mut Writer<W>, ti: &[u8; 10]) -> Result<(), SerializeError> {
    let weight = ti[2].to_string();
    let proportion = ti[3].to_string();
    let contrast = ti[4].to_string();
    let stroke_variation = ti[5].to_string();
    let arm_style = ti[6].to_string();
    let letterform = ti[7].to_string();
    let midline = ti[8].to_string();
    let x_height = ti[9].to_string();
    empty_tag(
        w,
        "hh:typeInfo",
        &[
            ("familyType", font_family_type_str(ti[0])),
            ("weight", &weight),
            ("proportion", &proportion),
            ("contrast", &contrast),
            ("strokeVariation", &stroke_variation),
            ("armStyle", &arm_style),
            ("letterform", &letterform),
            ("midline", &midline),
            ("xHeight", &x_height),
        ],
    )
}

/// `parse_subst_font` 의 역함수. 4개 속성을 원본 순서(face·type·isEmbedded·
/// binaryItemIDRef)로 복원한다. `binaryItemIDRef` 는 비임베드 시에도 빈 문자열로
/// 항상 출력한다(한컴 원본과 동일).
fn write_subst_font<W: Write>(w: &mut Writer<W>, sf: &SubstFont) -> Result<(), SerializeError> {
    empty_tag(
        w,
        "hh:substFont",
        &[
            ("face", &sf.face),
            ("type", font_type_str(sf.font_type)),
            ("isEmbedded", if sf.is_embedded { "1" } else { "0" }),
            ("binaryItemIDRef", &sf.bin_item_id_ref),
        ],
    )
}

/// `parser::hwpx::header::font_family_type_to_u8` 의 역함수. 0/미상은 OWPML
/// 표준값 `FCAT_UNKNOWN` 으로 복원한다.
fn font_family_type_str(v: u8) -> &'static str {
    match v {
        1 => "FCAT_MYUNGJO",
        2 => "FCAT_GOTHIC",
        3 => "FCAT_SSERIF",
        4 => "FCAT_BRUSHSCRIPT",
        5 => "FCAT_DECORATIVE",
        6 => "FCAT_NONRECTMJ",
        7 => "FCAT_NONRECTGT",
        _ => "FCAT_UNKNOWN",
    }
}

// =====================================================================
// <hh:borderFills>
// =====================================================================
fn write_border_fills<W: Write>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    if doc_info.border_fills.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:borderFills",
        &[("itemCnt", &doc_info.border_fills.len().to_string())],
    )?;
    // HWPX borderFill의 id는 1부터 시작 (관찰값: ref_empty.hwpx).
    // 그러나 rhwp parser는 인덱스 기반으로 저장하므로 id는 배열 인덱스 그대로 사용.
    for (idx, bf) in doc_info.border_fills.iter().enumerate() {
        write_border_fill(w, idx as u16, bf, ctx)?;
    }
    end_tag(w, "hh:borderFills")?;
    Ok(())
}

fn write_border_fill<W: Write>(
    w: &mut Writer<W>,
    id: u16,
    bf: &BorderFill,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    let attr = effective_border_fill_attr(bf);

    // 속성 순서 (BorderFillType.cpp:64-68): id, threeD, shadow, centerLine, breakCellSeparateLine
    start_tag_attrs(
        w,
        "hh:borderFill",
        &[
            ("id", &(id + 1).to_string()), // HWPX 관찰: id는 1-based
            ("threeD", "0"),
            ("shadow", "0"),
            ("centerLine", center_line_type(bf)),
            ("breakCellSeparateLine", "0"),
        ],
    )?;

    // 자식 순서 (BorderFillType.cpp:51-58):
    // slash, backSlash, leftBorder, rightBorder, topBorder, bottomBorder, diagonal, fillBrush
    write_diag_line(
        w,
        "hh:slash",
        diagonal_shape_type(((attr >> 2) & 0x07) as u8),
        (attr >> 8) & 0x03,
        attr & (1 << 11) != 0,
    )?;
    write_diag_line(
        w,
        "hh:backSlash",
        diagonal_shape_type(((attr >> 5) & 0x07) as u8),
        (attr >> 10) & 0x01,
        attr & (1 << 12) != 0,
    )?;
    write_border_line(w, "hh:leftBorder", &bf.borders[0])?;
    write_border_line(w, "hh:rightBorder", &bf.borders[1])?;
    write_border_line(w, "hh:topBorder", &bf.borders[2])?;
    write_border_line(w, "hh:bottomBorder", &bf.borders[3])?;
    write_diagonal(w, &bf.diagonal)?;

    // fillBrush: 도형과 동일한 fillBrush 구조를 공유한다.
    // 종전 Stage 1 은 빈 래퍼만 출력해 winBrush(배경색)/gradation/imgBrush 를
    // 전부 잃었다. 파서가 채운 Fill 을 shape 의 검증된 역매핑으로 직렬화한다.
    super::shape::write_fill_brush(w, &bf.fill, ctx)?;

    end_tag(w, "hh:borderFill")?;
    Ok(())
}

fn write_diag_line<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    type_str: &str,
    crooked: u16,
    is_counter: bool,
) -> Result<(), SerializeError> {
    let crooked = crooked.to_string();
    let is_counter = if is_counter { "1" } else { "0" };
    empty_tag(
        w,
        name,
        &[
            ("type", type_str),
            ("Crooked", crooked.as_str()),
            ("isCounter", is_counter),
        ],
    )
}

fn diagonal_shape_type(code: u8) -> &'static str {
    match code & 0x07 {
        0 => "NONE",
        0b010 => "CENTER",
        0b011 => "CENTER_BELOW",
        0b110 => "CENTER_ABOVE",
        _ => "ALL",
    }
}

fn center_line_type(bf: &BorderFill) -> &'static str {
    effective_center_line(bf).as_hwpx()
}

fn effective_center_line(bf: &BorderFill) -> CenterLine {
    if bf.center_line != CenterLine::None {
        bf.center_line
    } else {
        CenterLine::from_hwp_attr(bf.attr)
    }
}

fn effective_border_fill_attr(bf: &BorderFill) -> u16 {
    let center_line = effective_center_line(bf);
    if center_line == CenterLine::None {
        return bf.attr;
    }

    let mut attr = bf.attr;
    attr &=
        !((0x07 << 2) | (0x07 << 5) | (0x03 << 8) | (1 << 10) | (1 << 11) | (1 << 12) | (1 << 13));
    attr | center_line.hwp_attr_bits()
}

fn write_border_line<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    line: &BorderLine,
) -> Result<(), SerializeError> {
    let type_str = border_line_type_str(line.line_type);
    let width_mm = format!("{} mm", border_width_mm(line.width));
    let color = color_hex(line.color);
    empty_tag(
        w,
        name,
        &[("type", type_str), ("width", &width_mm), ("color", &color)],
    )
}

fn write_diagonal<W: Write>(w: &mut Writer<W>, d: &DiagonalLine) -> Result<(), SerializeError> {
    // diagonal_type 코드 0 = 대각선 없음 → 엘리먼트 자체를 생략한다. 한컴 원본도
    // 대각선이 없으면 <hh:diagonal> 를 쓰지 않고, 렌더러도 diagonal_type==0 을
    // 미표시로 처리한다. 종전엔 width==0 으로 NONE 을 추론해 대각선 없는 borderFill
    // 마다 <hh:diagonal type="NONE"/> 을 과다 출력했고(원본에 없던 요소 추가),
    // 그 회피책으로 파서가 width 를 max(1) 로 띄워 0.1mm 대각선을 0.12mm 로 변질시켰다.
    // 이제 선 종류는 width 가 아니라 diagonal_type 코드에서 직접 복원한다.
    if d.diagonal_type == 0 {
        return Ok(());
    }
    let type_str = border_line_type_str(border_line_type_from_code(d.diagonal_type));
    let width_mm = format!("{} mm", border_width_mm(d.width));
    let color = color_hex(d.color);
    empty_tag(
        w,
        "hh:diagonal",
        &[("type", type_str), ("width", &width_mm), ("color", &color)],
    )
}

/// `parser::hwpx::header::parse_border_line_type_code` 의 역함수. 대각선 선 종류
/// 코드(u8)를 [`BorderLineType`] 으로 되돌려 `border_line_type_str` 로 문자열화한다.
fn border_line_type_from_code(code: u8) -> BorderLineType {
    use BorderLineType::*;
    match code {
        0 => None,
        1 => Solid,
        2 => Dash,
        3 => Dot,
        4 => DashDot,
        5 => DashDotDot,
        6 => LongDash,
        7 => Circle,
        8 => Double,
        9 => ThinThickDouble,
        10 => ThickThinDouble,
        11 => ThinThickThinTriple,
        12 => Wave,
        13 => DoubleWave,
        14 => Thick3D,
        15 => Thick3DReverse,
        16 => Thin3D,
        17 => Thin3DReverse,
        _ => Solid,
    }
}

fn border_line_type_str(t: BorderLineType) -> &'static str {
    use BorderLineType::*;
    match t {
        None => "NONE",
        Solid => "SOLID",
        Dash => "DASH",
        Dot => "DOT",
        DashDot => "DASH_DOT",
        DashDotDot => "DASH_DOT_DOT",
        LongDash => "LONG_DASH",
        Circle => "CIRCLE",
        Double => "DOUBLE_SLIM",
        ThinThickDouble => "SLIM_THICK",
        ThickThinDouble => "THICK_SLIM",
        ThinThickThinTriple => "SLIM_THICK_SLIM",
        Wave => "WAVE",
        DoubleWave => "DOUBLE_WAVE",
        Thick3D => "THICK3D",
        Thick3DReverse => "THICKREV3D",
        Thin3D => "3D",
        Thin3DReverse => "REV3D",
    }
}

fn border_width_mm(w: u8) -> &'static str {
    // 파서 parse_border_width 와 동일한 한컴 표준 16단계 테이블을 공유한다.
    border_width_mm_str(w)
}

fn color_hex(c: ColorRef) -> String {
    // ColorRef = u32. HWP 내부 저장: 상위 바이트가 비투명 플래그(0이면 유효 색상).
    // 0xFFFFFFFF = 투명/없음 센티넬 → "none"
    if c == 0xFFFFFFFF {
        return "none".to_string();
    }
    // HWPX는 "#RRGGBB" 또는 "#AARRGGBB".
    let a = ((c >> 24) & 0xFF) as u8;
    let r = (c & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = ((c >> 16) & 0xFF) as u8;
    if a == 0 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b)
    }
}

// =====================================================================
// <hh:charProperties>
// =====================================================================
fn write_char_properties<W: Write>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    let _ = ctx;
    if doc_info.char_shapes.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:charProperties",
        &[("itemCnt", &doc_info.char_shapes.len().to_string())],
    )?;
    for (idx, cs) in doc_info.char_shapes.iter().enumerate() {
        write_char_pr(w, idx as u32, cs)?;
    }
    end_tag(w, "hh:charProperties")?;
    Ok(())
}

fn write_char_pr<W: Write>(
    w: &mut Writer<W>,
    id: u32,
    cs: &CharShape,
) -> Result<(), SerializeError> {
    // 속성 순서 (CharShapeType.cpp:79-86): id, height, textColor, shadeColor,
    // useFontSpace, useKerning, symMark, borderFillIDRef
    let shade = if cs.shade_color == 0 {
        "none".to_string()
    } else {
        color_hex(cs.shade_color)
    };
    start_tag_attrs(
        w,
        "hh:charPr",
        &[
            ("id", &id.to_string()),
            ("height", &cs.base_size.to_string()),
            ("textColor", &color_hex(cs.text_color)),
            ("shadeColor", &shade),
            ("useFontSpace", bool01(cs.use_font_space)),
            ("useKerning", bool01(cs.kerning)),
            ("symMark", sym_mark_str(cs.emphasis_dot)),
            ("borderFillIDRef", &cs.border_fill_id.to_string()),
        ],
    )?;

    // 자식 순서 (CharShapeType.cpp:59-73):
    // fontRef, ratio, spacing, relSz, offset, italic, bold, underline, strikeout, outline,
    // shadow, emboss, engrave, supscript, subscript
    write_lang_attrs(w, "hh:fontRef", &cs.font_ids.map(|v| v as i32))?;
    write_lang_attrs(w, "hh:ratio", &cs.ratios.map(|v| v as i32))?;
    write_lang_attrs(w, "hh:spacing", &cs.spacings.map(|v| v as i32))?;
    write_lang_attrs(w, "hh:relSz", &cs.relative_sizes.map(|v| v as i32))?;
    write_lang_attrs(w, "hh:offset", &cs.char_offsets.map(|v| v as i32))?;
    if cs.italic {
        empty_tag(w, "hh:italic", &[])?;
    }
    if cs.bold {
        empty_tag(w, "hh:bold", &[])?;
    }
    // underline/strikeout/outline/shadow: 한컴은 비활성(NONE)이어도 항상 출력한다.
    // 모델은 파서가 NONE 일 때도 shape/color/offset 을 보존하므로(역매핑 가능),
    // 무조건 출력해 원본과 동일한 구조를 만든다.
    empty_tag(
        w,
        "hh:underline",
        &[
            ("type", underline_type_str(cs.underline_type)),
            ("shape", line_shape_str(cs.underline_shape)),
            ("color", &color_hex(cs.underline_color)),
        ],
    )?;
    // strikeout: 파서가 shape 값으로 strikethrough 여부를 결정하므로(is_real_strike_shape),
    // 비활성일 때는 반드시 shape="NONE" 으로 출력해야 재파싱 시 켜지지 않는다.
    empty_tag(
        w,
        "hh:strikeout",
        &[
            (
                "shape",
                if cs.strikethrough {
                    line_shape_str(cs.strike_shape)
                } else {
                    "NONE"
                },
            ),
            ("color", &color_hex(cs.strike_color)),
        ],
    )?;
    empty_tag(
        w,
        "hh:outline",
        &[("type", outline_type_str(cs.outline_type))],
    )?;
    empty_tag(
        w,
        "hh:shadow",
        &[
            (
                "type",
                if cs.shadow_type == 0 {
                    "NONE"
                } else {
                    "CONTINUOUS"
                },
            ),
            ("color", &color_hex(cs.shadow_color)),
            ("offsetX", &cs.shadow_offset_x.to_string()),
            ("offsetY", &cs.shadow_offset_y.to_string()),
        ],
    )?;
    if cs.emboss {
        empty_tag(w, "hh:emboss", &[])?;
    }
    if cs.engrave {
        empty_tag(w, "hh:engrave", &[])?;
    }
    if cs.superscript {
        empty_tag(w, "hh:supscript", &[])?;
    }
    if cs.subscript {
        empty_tag(w, "hh:subscript", &[])?;
    }

    end_tag(w, "hh:charPr")?;
    Ok(())
}

fn write_lang_attrs<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    vals: &[i32; 7],
) -> Result<(), SerializeError> {
    let s0 = vals[0].to_string();
    let s1 = vals[1].to_string();
    let s2 = vals[2].to_string();
    let s3 = vals[3].to_string();
    let s4 = vals[4].to_string();
    let s5 = vals[5].to_string();
    let s6 = vals[6].to_string();
    empty_tag(
        w,
        name,
        &[
            ("hangul", &s0),
            ("latin", &s1),
            ("hanja", &s2),
            ("japanese", &s3),
            ("other", &s4),
            ("symbol", &s5),
            ("user", &s6),
        ],
    )
}

fn bool01(b: bool) -> &'static str {
    if b {
        "1"
    } else {
        "0"
    }
}

fn sym_mark_str(em: u8) -> &'static str {
    match em {
        0 => "NONE",
        1 => "DOT_ABOVE",
        2 => "RING_ABOVE",
        3 => "TILDE",
        4 => "CARON",
        5 => "SIDE",
        6 => "COLON",
        _ => "NONE",
    }
}

fn underline_type_str(t: crate::model::style::UnderlineType) -> &'static str {
    use crate::model::style::UnderlineType::*;
    match t {
        None => "NONE",
        Bottom => "BOTTOM",
        Top => "TOP",
    }
}

fn line_shape_str(s: u8) -> &'static str {
    match s {
        0 => "SOLID",
        1 => "DASH",
        2 => "DOT",
        3 => "DASH_DOT",
        4 => "DASH_DOT_DOT",
        5 => "LONG_DASH",
        6 => "CIRCLE",
        7 => "DOUBLE_SLIM",
        8 => "SLIM_THICK",
        9 => "THICK_SLIM",
        10 => "SLIM_THICK_SLIM",
        11 => "WAVE",
        12 => "DOUBLE_WAVE",
        _ => "SOLID",
    }
}

fn outline_type_str(t: u8) -> &'static str {
    match t {
        0 => "NONE",
        1 => "SOLID",
        2 => "DASH",
        3 => "DOT",
        _ => "NONE",
    }
}

// =====================================================================
// <hh:tabProperties>
// =====================================================================
fn write_tab_properties<W: Write>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
) -> Result<(), SerializeError> {
    if doc_info.tab_defs.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:tabProperties",
        &[("itemCnt", &doc_info.tab_defs.len().to_string())],
    )?;
    for (idx, td) in doc_info.tab_defs.iter().enumerate() {
        write_tab_pr(w, idx as u16, td)?;
    }
    end_tag(w, "hh:tabProperties")?;
    Ok(())
}

fn write_tab_pr<W: Write>(w: &mut Writer<W>, id: u16, td: &TabDef) -> Result<(), SerializeError> {
    let attrs = [
        ("id", id.to_string()),
        ("autoTabLeft", bool01(td.auto_tab_left).to_string()),
        ("autoTabRight", bool01(td.auto_tab_right).to_string()),
    ];
    let attrs_ref: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();

    if td.tabs.is_empty() {
        empty_tag(w, "hh:tabPr", &attrs_ref)?;
    } else {
        start_tag_attrs(w, "hh:tabPr", &attrs_ref)?;
        for tab in &td.tabs {
            empty_tag(
                w,
                "hh:tabItem",
                &[
                    ("pos", &tab.position.to_string()),
                    ("type", tab_type_str(tab.tab_type)),
                    ("leader", tab_leader_str(tab.fill_type)),
                ],
            )?;
        }
        end_tag(w, "hh:tabPr")?;
    }
    Ok(())
}

fn tab_type_str(t: u8) -> &'static str {
    match t {
        0 => "LEFT",
        1 => "RIGHT",
        2 => "CENTER",
        3 => "DECIMAL",
        _ => "LEFT",
    }
}

fn tab_leader_str(f: u8) -> &'static str {
    match f {
        0 => "NONE",
        1 => "SOLID",
        2 => "DOT",
        3 => "DASH",
        4 => "DASH_DOT",
        5 => "DASH_DOT_DOT",
        6 => "LONG_DASH",
        7 => "CIRCLE",
        8 => "DOUBLE_SLIM",
        _ => "NONE",
    }
}

// =====================================================================
// <hh:numberings>
// =====================================================================
fn write_numberings<W: Write>(w: &mut Writer<W>, doc_info: &DocInfo) -> Result<(), SerializeError> {
    if doc_info.numberings.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:numberings",
        &[("itemCnt", &doc_info.numberings.len().to_string())],
    )?;
    for (idx, n) in doc_info.numberings.iter().enumerate() {
        write_numbering(w, idx as u16, n)?;
    }
    end_tag(w, "hh:numberings")?;
    Ok(())
}

fn write_numbering<W: Write>(
    w: &mut Writer<W>,
    id: u16,
    n: &Numbering,
) -> Result<(), SerializeError> {
    start_tag_attrs(
        w,
        "hh:numbering",
        &[
            ("id", &(id + 1).to_string()), // 관찰: 1-based
            ("start", &n.start_number.to_string()),
        ],
    )?;
    // 원본 HWPX paraHead 영역이 있으면 그대로 splice(10수준 + align/
    // useInstWidth/autoIndent/checkable/형식문자열 무손실 복원). 모델의 7수준
    // NumberingHead 로는 표현 못하는 정보를 보존한다. 없으면(HWP5 경로 등)
    // 아래 하드코딩 뼈대로 폴백.
    if let Some(raw) = &n.raw_para_heads {
        w.get_mut()
            .write_all(raw.as_bytes())
            .map_err(|e| SerializeError::XmlError(format!("numbering paraHead splice: {e}")))?;
        end_tag(w, "hh:numbering")?;
        return Ok(());
    }
    // Stage 1: 10 레벨 paraHead 뼈대 출력. 실제 값은 NumberingHead 참조해 생성.
    for level in 0..10usize {
        let idx = level.min(6);
        let h = &n.heads[idx];
        let start = n.level_start_numbers.get(idx).copied().unwrap_or(1);
        let level_s = (level + 1).to_string();
        let start_s = start.to_string();
        let wa = h.width_adjust.to_string();
        empty_tag(
            w,
            "hh:paraHead",
            &[
                ("start", &start_s),
                ("level", &level_s),
                ("align", "LEFT"),
                ("useInstWidth", "1"),
                ("autoIndent", "1"),
                ("widthAdjust", &wa),
                ("textOffsetType", "PERCENT"),
                ("textOffset", "50"),
                ("numFormat", "DIGIT"),
                ("charPrIDRef", &u32::MAX.to_string()),
                ("checkable", "0"),
            ],
        )?;
    }
    end_tag(w, "hh:numbering")?;
    Ok(())
}

// =====================================================================
// <hh:bullets> — 글머리표 정의
//
// 종전 직렬화는 bullets 를 전혀 쓰지 않아 라운드트립에서 글머리표 정의(❏/※/❍ 등)가
// 소실되고, 글머리표 문단의 마커 글리프가 렌더에서 사라졌다. 파서(parse_bullet_hwpx)는
// `char`/`useImage` 만 읽으므로 그 둘 + paraHead 뼈대를 방출하면 round-trip 무손실이다.
// =====================================================================
fn write_bullets<W: Write>(w: &mut Writer<W>, doc_info: &DocInfo) -> Result<(), SerializeError> {
    if doc_info.bullets.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:bullets",
        &[("itemCnt", &doc_info.bullets.len().to_string())],
    )?;
    for (idx, b) in doc_info.bullets.iter().enumerate() {
        write_bullet(w, idx as u16, b)?;
    }
    end_tag(w, "hh:bullets")?;
    Ok(())
}

fn write_bullet<W: Write>(
    w: &mut Writer<W>,
    id: u16,
    b: &crate::model::style::Bullet,
) -> Result<(), SerializeError> {
    let id_s = (id + 1).to_string(); // 관찰: 1-based, ParaShape.numbering_id 참조와 정합
    let char_s = b.bullet_char.to_string();
    let use_image = if b.image_bullet > 0 { "1" } else { "0" };
    start_tag_attrs(
        w,
        "hh:bullet",
        &[("id", &id_s), ("char", &char_s), ("useImage", use_image)],
    )?;
    // paraHead 뼈대 (파서는 무시하나 OWPML 유효성/한컴 호환 위해 방출).
    empty_tag(
        w,
        "hh:paraHead",
        &[
            ("level", "0"),
            ("align", "LEFT"),
            ("useInstWidth", "0"),
            ("autoIndent", "1"),
            ("widthAdjust", &b.width_adjust.to_string()),
            ("textOffsetType", "PERCENT"),
            ("textOffset", "50"),
            ("numFormat", "DIGIT"),
            ("charPrIDRef", &u32::MAX.to_string()),
            ("checkable", "0"),
        ],
    )?;
    end_tag(w, "hh:bullet")?;
    Ok(())
}

// =====================================================================
// <hh:paraProperties>
// =====================================================================
fn write_para_properties<W: Write>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    let _ = ctx;
    if doc_info.para_shapes.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:paraProperties",
        &[("itemCnt", &doc_info.para_shapes.len().to_string())],
    )?;
    for (idx, ps) in doc_info.para_shapes.iter().enumerate() {
        write_para_pr(w, idx as u16, ps)?;
    }
    end_tag(w, "hh:paraProperties")?;
    Ok(())
}

fn write_para_pr<W: Write>(
    w: &mut Writer<W>,
    id: u16,
    ps: &ParaShape,
) -> Result<(), SerializeError> {
    // 속성 순서 (ParaShapeType.cpp:62-68): id, tabPrIDRef, condense,
    // fontLineHeight, snapToGrid, suppressLineNumbers, checked
    //
    // condense/fontLineHeight/snapToGrid 는 attr1 비트로 보존된다(파서 역매핑):
    //   snapToGrid = bit8, condense = bits9..15, fontLineHeight = bit22.
    // 종전엔 상수("0"/"0"/"1")로 하드코딩해 condense(20 등)와 snapToGrid(0)을 잃었다.
    let condense = ((ps.attr1 >> 9) & 0x7f).to_string();
    let font_line_height = ((ps.attr1 >> 22) & 1).to_string();
    let snap_to_grid = ((ps.attr1 >> 8) & 1).to_string();
    start_tag_attrs(
        w,
        "hh:paraPr",
        &[
            ("id", &id.to_string()),
            ("tabPrIDRef", &ps.tab_def_id.to_string()),
            ("condense", &condense),
            ("fontLineHeight", &font_line_height),
            ("snapToGrid", &snap_to_grid),
            ("suppressLineNumbers", "0"),
            ("checked", "0"),
        ],
    )?;

    // 자식 순서 (한컴 원본 관찰):
    // align, heading, breakSetting, autoSpacing, switch(margin+lineSpacing), border
    //
    // 종전엔 align@vertical, breakSetting@{breakNonLatinWord, widowOrphan,
    // keepWithNext, keepLines, pageBreakBefore} 를 상수로 하드코딩해, 파서가
    // attr1/attr2 비트로 보존한 값을 직렬화에서 모두 잃었다(예: vertical=CENTER →
    // BASELINE, breakNonLatinWord=BREAK_WORD → KEEP_WORD). 이제 보존 비트에서
    // 역매핑한다. (breakLatinWord/lineWrap 은 파서가 아직 미수집 → 상수 유지.)
    let vertical = vertical_alignment_str((ps.attr1 >> 20) & 0x03);
    // attr1 bit7: KEEP_WORD=1, BREAK_WORD=0 (parse_para_shape_child 와 정합).
    let break_non_latin = if (ps.attr1 >> 7) & 1 == 1 {
        "KEEP_WORD"
    } else {
        "BREAK_WORD"
    };
    // [#1986] breakLatinWord 는 IR 원문 보존값(없으면 KEEP_WORD 기본).
    let break_latin = ps.break_latin_word.as_deref().unwrap_or("KEEP_WORD");
    let widow_orphan = ((ps.attr2 >> 5) & 1).to_string();
    let keep_with_next = ((ps.attr2 >> 6) & 1).to_string();
    let keep_lines = ((ps.attr2 >> 7) & 1).to_string();
    let page_break_before = ((ps.attr2 >> 8) & 1).to_string();
    empty_tag(
        w,
        "hh:align",
        &[
            ("horizontal", alignment_str(ps.alignment)),
            ("vertical", vertical),
        ],
    )?;
    empty_tag(
        w,
        "hh:heading",
        &[
            ("type", head_type_str(ps.head_type)),
            ("idRef", &ps.numbering_id.to_string()),
            ("level", &ps.para_level.to_string()),
        ],
    )?;
    empty_tag(
        w,
        "hh:breakSetting",
        &[
            ("breakLatinWord", break_latin),
            ("breakNonLatinWord", break_non_latin),
            ("widowOrphan", &widow_orphan),
            ("keepWithNext", &keep_with_next),
            ("keepLines", &keep_lines),
            ("pageBreakBefore", &page_break_before),
            ("lineWrap", "BREAK"),
        ],
    )?;

    empty_tag(
        w,
        "hh:autoSpacing",
        &[("eAsianEng", "0"), ("eAsianNum", "0")],
    )?;

    // margin + lineSpacing 은 한컴 원본과 동일하게 <hp:switch>(case/default)로 감싼다.
    write_para_margin_switch(w, ps)?;

    let border_connect = if (ps.attr1 >> 28) & 1 != 0 { "1" } else { "0" };
    let border_ignore_margin = if (ps.attr1 >> 29) & 1 != 0 { "1" } else { "0" };

    empty_tag(
        w,
        "hh:border",
        &[
            ("borderFillIDRef", &ps.border_fill_id.to_string()),
            ("offsetLeft", &ps.border_spacing[0].to_string()),
            ("offsetRight", &ps.border_spacing[1].to_string()),
            ("offsetTop", &ps.border_spacing[2].to_string()),
            ("offsetBottom", &ps.border_spacing[3].to_string()),
            ("connect", border_connect),
            ("ignoreMargin", border_ignore_margin),
        ],
    )?;

    end_tag(w, "hh:paraPr")?;
    Ok(())
}

/// paraPr 의 margin + lineSpacing 을 한컴 원본과 동일하게 `<hp:switch>` 구조로 쓴다.
///
/// `parse_para_shape_switch` 의 정확한 역. 파서는 HwpUnitChar `case` 값을 ×2 하여
/// IR 에 적재하므로(`stored = case × 2`), 역으로:
///   - `default` 값 = IR 저장값 (`ps.indent` 등)
///   - `case`(HwpUnitChar) 값 = 저장값 / 2 (margin), lineSpacing 은 PERCENT=저장값,
///     그 외(Fixed/SpaceOnly/Minimum)=저장값/2
///
/// 파서는 `case` 를 우선 읽으므로 라운드트립 시 `case × 2 = 저장값` 으로 IR 이
/// 정확히 복원된다(한컴 원본의 저장값은 항상 짝수).
fn write_para_margin_switch<W: Write>(
    w: &mut Writer<W>,
    ps: &ParaShape,
) -> Result<(), SerializeError> {
    super::utils::start_tag(w, "hp:switch")?;

    start_tag_attrs(
        w,
        "hp:case",
        &[(
            "hp:required-namespace",
            "http://www.hancom.co.kr/hwpml/2016/HwpUnitChar",
        )],
    )?;
    write_para_margin(w, ps, true)?;
    write_para_line_spacing(w, ps, true)?;
    end_tag(w, "hp:case")?;

    super::utils::start_tag(w, "hp:default")?;
    write_para_margin(w, ps, false)?;
    write_para_line_spacing(w, ps, false)?;
    end_tag(w, "hp:default")?;

    end_tag(w, "hp:switch")?;
    Ok(())
}

/// `<hh:margin>` (자식 5개: intent/left/right/prev/next). `half=true` 면 HwpUnitChar
/// case 용으로 저장값의 절반을 쓴다.
fn write_para_margin<W: Write>(
    w: &mut Writer<W>,
    ps: &ParaShape,
    half: bool,
) -> Result<(), SerializeError> {
    let v = |x: i32| if half { x / 2 } else { x };
    super::utils::start_tag(w, "hh:margin")?;
    write_margin_child(w, "hc:intent", v(ps.indent))?;
    write_margin_child(w, "hc:left", v(ps.margin_left))?;
    write_margin_child(w, "hc:right", v(ps.margin_right))?;
    write_margin_child(w, "hc:prev", v(ps.spacing_before))?;
    write_margin_child(w, "hc:next", v(ps.spacing_after))?;
    end_tag(w, "hh:margin")?;
    Ok(())
}

/// `<hh:lineSpacing>`. HwpUnitChar case(`case=true`)에서 PERCENT 는 저장값과 동일,
/// 그 외 유형은 저장값의 절반을 쓴다(파서가 ×2 적재).
fn write_para_line_spacing<W: Write>(
    w: &mut Writer<W>,
    ps: &ParaShape,
    case: bool,
) -> Result<(), SerializeError> {
    let value = if case && !matches!(ps.line_spacing_type, LineSpacingType::Percent) {
        ps.line_spacing / 2
    } else {
        ps.line_spacing
    };
    empty_tag(
        w,
        "hh:lineSpacing",
        &[
            ("type", line_spacing_type_str(ps.line_spacing_type)),
            ("value", &value.to_string()),
            ("unit", "HWPUNIT"),
        ],
    )
}

/// margin 자식(`<hc:intent value="…" unit="HWPUNIT"/>`). 한컴 원본 속성 순서는
/// value, unit 이며 네임스페이스는 `hc:` 다.
fn write_margin_child<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    value: i32,
) -> Result<(), SerializeError> {
    empty_tag(
        w,
        name,
        &[("value", &value.to_string()), ("unit", "HWPUNIT")],
    )
}

fn alignment_str(a: Alignment) -> &'static str {
    use Alignment::*;
    match a {
        Justify => "JUSTIFY",
        Left => "LEFT",
        Right => "RIGHT",
        Center => "CENTER",
        Distribute => "DISTRIBUTE",
        Split => "DISTRIBUTE_SPACE",
    }
}

/// `parse_vertical_alignment_bits` 의 역함수. attr1 bits 20..21 → OWPML 문자열.
fn vertical_alignment_str(bits: u32) -> &'static str {
    match bits {
        1 => "TOP",
        2 => "CENTER",
        3 => "BOTTOM",
        _ => "BASELINE",
    }
}

fn head_type_str(h: HeadType) -> &'static str {
    use HeadType::*;
    match h {
        None => "NONE",
        Outline => "OUTLINE",
        Number => "NUMBER",
        Bullet => "BULLET",
    }
}

fn line_spacing_type_str(t: LineSpacingType) -> &'static str {
    use LineSpacingType::*;
    match t {
        Percent => "PERCENT",
        Fixed => "FIXED",
        SpaceOnly => "BETWEEN_LINES",
        Minimum => "AT_LEAST",
    }
}

// =====================================================================
// <hh:styles>
// =====================================================================
fn write_styles<W: Write>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    let _ = ctx;
    if doc_info.styles.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:styles",
        &[("itemCnt", &doc_info.styles.len().to_string())],
    )?;
    for (idx, st) in doc_info.styles.iter().enumerate() {
        write_style(w, idx as u16, st)?;
    }
    end_tag(w, "hh:styles")?;
    Ok(())
}

fn write_style<W: Write>(w: &mut Writer<W>, id: u16, st: &Style) -> Result<(), SerializeError> {
    let type_str = if st.style_type == 1 { "CHAR" } else { "PARA" };
    empty_tag(
        w,
        "hh:style",
        &[
            ("id", &id.to_string()),
            ("type", type_str),
            ("name", &st.local_name),
            ("engName", &st.english_name),
            ("paraPrIDRef", &st.para_shape_id.to_string()),
            ("charPrIDRef", &st.char_shape_id.to_string()),
            ("nextStyleIDRef", &st.next_style_id.to_string()),
            ("langID", "1042"),
            ("lockForm", "0"),
        ],
    )
}

// =====================================================================
// <hh:compatibleDocument>, <hh:docOption>, <hh:trackchageConfig>
// =====================================================================
fn write_compatible_document<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    start_tag_attrs(w, "hh:compatibleDocument", &[("targetProgram", "HWP201X")])?;
    super::utils::start_tag(w, "hh:layoutCompatibility")?;
    empty_tag(w, "hh:char", &[])?;
    empty_tag(w, "hh:paragraph", &[])?;
    empty_tag(w, "hh:section", &[])?;
    empty_tag(w, "hh:object", &[])?;
    empty_tag(w, "hh:field", &[])?;
    end_tag(w, "hh:layoutCompatibility")?;
    end_tag(w, "hh:compatibleDocument")?;
    Ok(())
}

fn write_doc_option<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    super::utils::start_tag(w, "hh:docOption")?;
    empty_tag(
        w,
        "hh:linkinfo",
        &[("path", ""), ("pageInherit", "0"), ("footnoteInherit", "0")],
    )?;
    end_tag(w, "hh:docOption")?;
    Ok(())
}

fn write_track_change_config<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    empty_tag(w, "hh:trackchageConfig", &[("flags", "0")])
}

// 내부에서 쓰는 start_tag 별명
use super::utils::start_tag;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::hwpx::parse_hwpx;

    #[test]
    fn write_header_runs_on_empty_document() {
        let doc = Document::default();
        let ctx = SerializeContext::collect_from_document(&doc);
        let bytes = write_header(&doc, &ctx).expect("write_header");
        let xml = std::str::from_utf8(&bytes).unwrap();
        assert!(xml.contains("<hh:head"));
        assert!(xml.contains("</hh:head>"));
        // [Finding 15] hwpunitchar 네임스페이스 선언 누락 금지(원본 hh:head 에 존재).
        assert!(
            xml.contains(r#"xmlns:hwpunitchar="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar""#),
            "hh:head 에 xmlns:hwpunitchar 선언이 있어야 함"
        );
        // [Finding 17] hwpml_version 미지정 시 "1.2" 폴백.
        assert!(xml.contains(r#"version="1.2""#), "기본 버전 폴백은 1.2");
    }

    #[test]
    fn write_header_seccnt_matches_section_count() {
        // #1557: secCnt 는 실제 직렬화 섹션 수(doc.sections)와 일치해야 한다.
        // doc_properties.section_count 가 stale(1) 이어도 섹션 수가 우선 — 불일치 시
        // 한글이 뒤 구역을 로드하지 않아 다중 페이지 문서가 1쪽으로 붕괴한다.
        use crate::model::document::Section;
        let mut doc = Document::default();
        doc.sections = vec![Section::default(), Section::default(), Section::default()];
        doc.doc_properties.section_count = 1; // stale 모사
        let ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_header(&doc, &ctx).expect("write_header")).unwrap();
        assert!(
            xml.contains(r#"secCnt="3""#),
            "secCnt 가 섹션 수(3)와 일치해야 함(붕괴 회귀 가드)"
        );
    }

    #[test]
    fn write_header_emits_preserved_hwpml_version() {
        // [Finding 17] 원본 HWPML 버전(문서별 상이, 예: 1.5)을 하드코딩 1.2 로
        // 변질시키지 않고 그대로 재방출해야 한다.
        let mut doc = Document::default();
        doc.doc_info.hwpml_version = Some("1.5".to_string());
        let ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_header(&doc, &ctx).expect("write_header")).unwrap();
        assert!(
            xml.contains(r#"version="1.5""#),
            "보존된 hwpml_version=1.5 가 방출되어야 함"
        );
        assert!(
            !xml.contains(r#"version="1.2""#),
            "하드코딩 1.2 가 남아있으면 안 됨"
        );
    }

    #[test]
    fn write_header_preserves_char_shape_count() {
        let bytes = include_bytes!("../../../samples/hwpx/ref/ref_empty.hwpx");
        let doc = parse_hwpx(bytes).expect("parse ref_empty");
        let ctx = SerializeContext::collect_from_document(&doc);
        let header_bytes = write_header(&doc, &ctx).expect("write header");
        let xml = std::str::from_utf8(&header_bytes).unwrap();
        // ref_empty.hwpx 의 charPr 개수는 관찰 결과 7개
        let expected = doc.doc_info.char_shapes.len();
        let actual = xml.matches("<hh:charPr ").count();
        assert_eq!(actual, expected, "charPr count mismatch");
    }

    #[test]
    fn write_header_splices_doc_settings_tail_verbatim() {
        let bytes = include_bytes!("../../../samples/hwpx/ref/ref_empty.hwpx");
        let mut doc = parse_hwpx(bytes).expect("parse");
        // 원본 설정 tail(빈 layoutCompatibility, pageInherit=1, trackchange=56).
        doc.doc_info.hwpx_head_tail = Some(
            r#"<hh:compatibleDocument targetProgram="HWP201X"><hh:layoutCompatibility/></hh:compatibleDocument><hh:docOption><hh:linkinfo path="" pageInherit="1" footnoteInherit="0"/></hh:docOption><hh:trackchageConfig flags="56"/>"#
                .to_string(),
        );
        let ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_header(&doc, &ctx).unwrap()).unwrap();
        assert!(
            xml.contains("<hh:layoutCompatibility/></hh:compatibleDocument>"),
            "원본의 빈 layoutCompatibility 가 그대로 보존되어야 함: {xml}"
        );
        assert!(xml.contains(r#"pageInherit="1""#), "pageInherit=1 보존");
        assert!(
            xml.contains(r#"<hh:trackchageConfig flags="56"/>"#),
            "trackchange flags=56 보존"
        );
        assert!(
            !xml.contains("<hh:char/><hh:paragraph/>"),
            "하드코딩 layoutCompatibility 자식이 없어야 함"
        );
    }

    #[test]
    fn write_header_falls_back_to_hardcoded_tail_when_absent() {
        let bytes = include_bytes!("../../../samples/hwpx/ref/ref_empty.hwpx");
        let mut doc = parse_hwpx(bytes).expect("parse");
        doc.doc_info.hwpx_head_tail = None;
        let ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_header(&doc, &ctx).unwrap()).unwrap();
        assert!(
            xml.contains("<hh:layoutCompatibility><hh:char/><hh:paragraph/><hh:section/><hh:object/><hh:field/></hh:layoutCompatibility>"),
            "원본 부재 시 하드코딩 폴백: {xml}"
        );
        assert!(xml.contains(r#"<hh:trackchageConfig flags="0"/>"#));
    }

    #[test]
    fn write_header_emits_seven_fontfaces_when_populated() {
        let bytes = include_bytes!("../../../samples/hwpx/ref/ref_empty.hwpx");
        let doc = parse_hwpx(bytes).expect("parse");
        let ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_header(&doc, &ctx).unwrap()).unwrap();
        assert_eq!(xml.matches("<hh:fontface ").count(), 7);
    }

    #[test]
    fn write_font_type_info_is_inverse_of_parser_layout() {
        // 파서 parse_font_type_info 의 바이트 배치를 그대로 역매핑해야 한다.
        // FCAT_GOTHIC=2, [1]=serif(미노출), weight=6, 나머지=strokeVariation..xHeight=1.
        let ti = [2u8, 3, 6, 0, 0, 1, 1, 1, 1, 1];
        let mut writer = Writer::new(Vec::new());
        write_font_type_info(&mut writer, &ti).expect("write typeInfo");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        assert_eq!(
            xml,
            r#"<hh:typeInfo familyType="FCAT_GOTHIC" weight="6" proportion="0" contrast="0" strokeVariation="1" armStyle="1" letterform="1" midline="1" xHeight="1"/>"#,
            "typeInfo 9개 속성이 원본 순서/값으로 복원되어야 함"
        );
        // serif 바이트 [1] 은 XML 에 노출되지 않는다(파서가 글꼴명에서 합성).
        assert!(!xml.contains("serif"));
    }

    #[test]
    fn write_subst_font_emits_all_four_attributes_in_order() {
        let sf = SubstFont {
            face: "한컴바탕".to_string(),
            font_type: 1,
            is_embedded: false,
            bin_item_id_ref: String::new(),
        };
        let mut writer = Writer::new(Vec::new());
        write_subst_font(&mut writer, &sf).expect("write substFont");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        assert_eq!(
            xml,
            r#"<hh:substFont face="한컴바탕" type="TTF" isEmbedded="0" binaryItemIDRef=""/>"#,
            "substFont 4개 속성이 원본 순서/값으로 복원되어야 함(비임베드도 빈 binaryItemIDRef 출력)"
        );
    }

    #[test]
    fn write_fontfaces_emits_subst_font_before_type_info() {
        let mut doc_info = DocInfo::default();
        doc_info.font_faces = vec![Vec::new(); 7];
        // substFont 와 typeInfo 를 모두 가진 글꼴: 원본 순서는 substFont → typeInfo.
        doc_info.font_faces[0].push(Font {
            name: "바탕".to_string(),
            alt_type: 1,
            type_info: Some([2, 3, 6, 0, 0, 1, 1, 1, 1, 1]),
            subst_font: Some(SubstFont {
                face: "한컴바탕".to_string(),
                font_type: 1,
                is_embedded: false,
                bin_item_id_ref: String::new(),
            }),
            ..Default::default()
        });
        let mut writer = Writer::new(Vec::new());
        write_fontfaces(&mut writer, &doc_info).expect("write fontfaces");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        let sub = xml.find("<hh:substFont ").expect("substFont present");
        let ti = xml.find("<hh:typeInfo ").expect("typeInfo present");
        assert!(sub < ti, "substFont 가 typeInfo 보다 먼저 와야 함: {xml}");
        assert!(
            xml.contains(
                r#"<hh:font id="0" face="바탕" type="TTF" isEmbedded="0"><hh:substFont face="한컴바탕""#
            ),
            "substFont 가 font 의 첫 자식이어야 함: {xml}"
        );
    }

    #[test]
    fn write_fontfaces_emits_type_info_child_only_when_present() {
        let mut doc_info = DocInfo::default();
        doc_info.font_faces = vec![Vec::new(); 7];
        // 한 그룹에 typeInfo 가 있는 글꼴과 없는 글꼴을 같이 둔다.
        doc_info.font_faces[0].push(Font {
            name: "굴림".to_string(),
            alt_type: 1,
            ..Default::default()
        });
        doc_info.font_faces[0].push(Font {
            name: "바탕".to_string(),
            alt_type: 1,
            type_info: Some([2, 3, 6, 0, 0, 1, 1, 1, 1, 1]),
            ..Default::default()
        });
        let mut writer = Writer::new(Vec::new());
        write_fontfaces(&mut writer, &doc_info).expect("write fontfaces");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        // typeInfo 없는 글꼴은 self-closing 유지.
        assert!(
            xml.contains(r#"<hh:font id="0" face="굴림" type="TTF" isEmbedded="0"/>"#),
            "typeInfo 없는 글꼴은 self-closing: {xml}"
        );
        // typeInfo 있는 글꼴은 자식으로 복원.
        assert!(
            xml.contains(
                r#"<hh:font id="1" face="바탕" type="TTF" isEmbedded="0"><hh:typeInfo familyType="FCAT_GOTHIC""#
            ),
            "typeInfo 있는 글꼴은 자식 복원: {xml}"
        );
        assert_eq!(xml.matches("<hh:typeInfo ").count(), 1);
    }

    #[test]
    fn canonical_attr_order_charpr() {
        let bytes = include_bytes!("../../../samples/hwpx/ref/ref_empty.hwpx");
        let doc = parse_hwpx(bytes).expect("parse");
        let ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_header(&doc, &ctx).unwrap()).unwrap();
        let snippet = xml
            .find("<hh:charPr ")
            .and_then(|i| {
                let end = xml[i..].find('>').map(|e| i + e)?;
                Some(&xml[i..=end])
            })
            .expect("charPr tag");
        // 속성이 id → height → textColor → shadeColor → useFontSpace → useKerning → symMark → borderFillIDRef 순서여야 함
        let ip = snippet.find("id=").unwrap();
        let hp = snippet.find("height=").unwrap();
        let tc = snippet.find("textColor=").unwrap();
        let sc = snippet.find("shadeColor=").unwrap();
        let uf = snippet.find("useFontSpace=").unwrap();
        let uk = snippet.find("useKerning=").unwrap();
        let sm = snippet.find("symMark=").unwrap();
        let bf = snippet.find("borderFillIDRef=").unwrap();
        assert!(ip < hp && hp < tc && tc < sc && sc < uf && uf < uk && uk < sm && sm < bf);
    }

    #[test]
    fn write_border_fill_preserves_slash_and_backslash_shape_types() {
        let mut bf = BorderFill::default();
        bf.attr = (0b010 << 2) | (0b011 << 5);

        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            0,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"<hh:slash type="CENTER" Crooked="0" isCounter="0"/>"#),
            "slash 방향 비트가 CENTER로 보존되어야 함: {xml}"
        );
        assert!(
            xml.contains(r#"<hh:backSlash type="CENTER_BELOW" Crooked="0" isCounter="0"/>"#),
            "backSlash 방향 비트가 CENTER_BELOW로 보존되어야 함: {xml}"
        );
    }

    #[test]
    fn write_border_fill_preserves_center_line_type() {
        let mut bf = BorderFill::default();
        bf.center_line = CenterLine::Vertical;

        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            0,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"centerLine="VERTICAL""#),
            "centerLine 방향이 보존되어야 함: {xml}"
        );
        assert!(
            xml.contains(r#"<hh:slash type="NONE" Crooked="3" isCounter="0"/>"#),
            "VERTICAL 중심선의 HWP attr 보조 비트가 Crooked=3 으로 보존되어야 함: {xml}"
        );
    }

    #[test]
    fn write_border_fill_preserves_slash_crooked_two_bit_value() {
        let mut bf = BorderFill::default();
        bf.attr = (2 << 8) | (0b010 << 5);

        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            0,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"<hh:slash type="NONE" Crooked="2" isCounter="0"/>"#),
            "slash Crooked=2 가 보존되어야 함: {xml}"
        );
        assert!(
            xml.contains(r#"<hh:backSlash type="CENTER" Crooked="0" isCounter="0"/>"#),
            "backSlash CENTER 가 보존되어야 함: {xml}"
        );
    }

    #[test]
    fn write_border_fill_serializes_solid_winbrush_not_empty() {
        // borderFill 의 Solid 배경색(셀 음영)이 빈 fillBrush 로 누락되지 않고
        // 도형과 동일한 winBrush 구조로 직렬화되어야 한다.
        use crate::model::style::{Fill, SolidFill};
        let mut bf = BorderFill::default();
        bf.fill = Fill {
            fill_type: FillType::Solid,
            solid: Some(SolidFill {
                background_color: 0x00D6_D6D6, // #D6D6D6 (0xAABBGGRR, a=0)
                pattern_color: 0x0000_0000,    // #000000
                pattern_type: -1,
            }),
            alpha: 0,
            ..Default::default()
        };

        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            5,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(
                r##"<hc:fillBrush><hc:winBrush faceColor="#D6D6D6" hatchColor="#000000" alpha="0"/></hc:fillBrush>"##
            ),
            "Solid 배경색이 winBrush 로 직렬화되어야 함(빈 fillBrush 아님): {xml}"
        );
    }

    #[test]
    fn write_border_fill_omits_fillbrush_when_none() {
        // FillType::None + solid 없음 = 원본에 fillBrush 부재 → 미출력.
        let bf = BorderFill::default();
        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            0,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        assert!(
            !xml.contains("fillBrush"),
            "FillType::None + solid 없음 은 fillBrush 미출력: {xml}"
        );
    }

    #[test]
    fn write_border_fill_restores_empty_winbrush_when_none_with_solid() {
        // [Finding 12] 원본 winBrush faceColor="none"+무늬없음 은 파서가 렌더용으로
        // FillType::None 을 두되 solid 데이터를 보존한다. 직렬화기는 이 solid 로
        // 빈 winBrush 를 그대로 복원해야 round-trip 무손실이 된다(요소 누락 금지).
        use crate::model::style::{Fill, SolidFill};
        let mut bf = BorderFill::default();
        bf.fill = Fill {
            fill_type: FillType::None,
            solid: Some(SolidFill {
                background_color: 0xFFFF_FFFF, // faceColor="none" 센티넬
                pattern_color: 0xFF00_0000,    // hatchColor="#FF000000" (a=FF 보존)
                pattern_type: -1,
            }),
            alpha: 0,
            ..Default::default()
        };

        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            1,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(
                r##"<hc:fillBrush><hc:winBrush faceColor="none" hatchColor="#FF000000" alpha="0"/></hc:fillBrush>"##
            ),
            "보존된 빈 winBrush 가 faceColor=none 으로 복원되어야 함: {xml}"
        );
    }

    #[test]
    fn write_para_pr_emits_margin_switch_case_default() {
        // paraPr margin/lineSpacing 은 <hp:switch>(case=저장값/2, default=저장값)
        // 구조로, margin 자식은 hc: 네임스페이스(value, unit 순)로 출력되어야 한다.
        let mut ps = ParaShape::default();
        ps.indent = -2620; // default; case = -1310
        ps.line_spacing = 130;
        ps.line_spacing_type = LineSpacingType::Percent;

        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 1, &ps).expect("write paraPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        // case: HwpUnitChar required-namespace + 절반 값 + hc: 네임스페이스
        assert!(
            xml.contains(
                r#"<hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar"><hh:margin><hc:intent value="-1310" unit="HWPUNIT"/>"#
            ),
            "case 는 HwpUnitChar + intent 절반(-1310) + hc: 네임스페이스: {xml}"
        );
        // default: 저장값 그대로
        assert!(
            xml.contains(r#"<hp:default><hh:margin><hc:intent value="-2620" unit="HWPUNIT"/>"#),
            "default 는 저장값(-2620): {xml}"
        );
        // PERCENT lineSpacing 은 case/default 동일
        assert_eq!(
            xml.matches(r#"<hh:lineSpacing type="PERCENT" value="130" unit="HWPUNIT"/>"#)
                .count(),
            2,
            "PERCENT lineSpacing 은 case/default 양쪽에 동일 값: {xml}"
        );
        // 자식 순서: autoSpacing → switch → border
        let auto = xml.find("autoSpacing").unwrap();
        let sw = xml.find("<hp:switch>").unwrap();
        let border = xml.find("<hh:border ").unwrap();
        assert!(
            auto < sw && sw < border,
            "순서 autoSpacing<switch<border: {xml}"
        );
        // 옛 평면 hh: 마진 네임스페이스는 더 이상 없어야 한다.
        assert!(
            !xml.contains("<hh:intent"),
            "margin 자식이 hh: 네임스페이스로 남으면 안 됨: {xml}"
        );
    }

    #[test]
    fn write_para_pr_emits_align_and_break_from_preserved_bits() {
        // [Finding 18] align@vertical, breakSetting@{breakNonLatinWord, widowOrphan,
        // keepWithNext, keepLines, pageBreakBefore} 가 상수 하드코딩이 아니라
        // attr1/attr2 보존 비트에서 역매핑돼야 한다.
        let mut ps = ParaShape::default();
        ps.break_latin_word = Some("HYPHENATION".to_string());
        ps.attr1 = (2 << 20) // vertical = CENTER
            & !(1 << 7); // breakNonLatinWord = BREAK_WORD (bit7=0)
        ps.attr2 = (1 << 5) // widowOrphan = 1
            | (1 << 8); // pageBreakBefore = 1

        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 1, &ps).expect("write paraPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"vertical="CENTER""#),
            "vertical 은 보존 비트(CENTER)에서 와야 함: {xml}"
        );
        assert!(
            xml.contains(r#"breakLatinWord="HYPHENATION""#),
            "breakLatinWord 는 HWPX 원문 보존값에서 와야 함: {xml}"
        );
        assert!(
            xml.contains(r#"breakNonLatinWord="BREAK_WORD""#),
            "breakNonLatinWord 는 보존 비트(BREAK_WORD)에서 와야 함: {xml}"
        );
        assert!(
            xml.contains(r#"widowOrphan="1" keepWithNext="0" keepLines="0" pageBreakBefore="1""#),
            "widowOrphan/pageBreakBefore 보존 비트 역매핑: {xml}"
        );
    }

    #[test]
    fn write_para_pr_default_vertical_is_baseline() {
        // 기본 ParaShape(attr1=0) 의 vertical 은 BASELINE(bits 20..21 = 0). 파싱된
        // 문단은 항상 bit7 을 명시 설정하므로 round-trip 은 정확하다(이 테스트는
        // 신규 생성 기본 ParaShape 의 vertical 만 검증).
        let ps = ParaShape::default();
        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 0, &ps).expect("write paraPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        assert!(
            xml.contains(r#"vertical="BASELINE""#),
            "기본 vertical=BASELINE: {xml}"
        );
    }

    #[test]
    fn write_char_pr_always_emits_underline_strikeout_outline_shadow() {
        // 한컴은 비활성(NONE)이어도 네 요소를 항상 출력한다. 기본 CharShape 도
        // 동일 구조를 내야 하며, 특히 strikeout 은 shape="NONE" 이어야 재파싱 시
        // 취소선이 켜지지 않는다.
        let cs = CharShape::default();
        let mut writer = Writer::new(Vec::new());
        write_char_pr(&mut writer, 0, &cs).expect("write charPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r##"<hh:underline type="NONE" shape="SOLID" color="#000000"/>"##),
            "underline NONE 항상 출력: {xml}"
        );
        assert!(
            xml.contains(r##"<hh:strikeout shape="NONE" color="#000000"/>"##),
            "strikeout 은 비활성 시 shape=NONE: {xml}"
        );
        assert!(
            xml.contains(r#"<hh:outline type="NONE"/>"#),
            "outline NONE 항상 출력: {xml}"
        );
        assert!(
            xml.contains(r#"<hh:shadow type="NONE""#),
            "shadow NONE 항상 출력: {xml}"
        );
    }

    #[test]
    fn write_para_pr_emits_condense_fontlineheight_snaptogrid_from_attr1() {
        // condense/fontLineHeight/snapToGrid 는 상수가 아니라 attr1 비트에서 나와야 한다.
        let mut ps = ParaShape::default();
        // condense=20 (bits9..15), fontLineHeight=1 (bit22), snapToGrid=0 (bit8 clear)
        ps.attr1 = (20 << 9) | (1 << 22);
        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 0, &ps).expect("write paraPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        assert!(
            xml.contains(r#"condense="20""#),
            "condense=20 가 attr1 에서 직렬화되어야 함: {xml}"
        );
        assert!(
            xml.contains(r#"fontLineHeight="1""#),
            "fontLineHeight=1: {xml}"
        );
        assert!(
            xml.contains(r#"snapToGrid="0""#),
            "snapToGrid=0(bit8 clear): {xml}"
        );

        // snapToGrid=1 (bit8 set), condense=0
        ps.attr1 = 1 << 8;
        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 0, &ps).expect("write paraPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        assert!(
            xml.contains(r#"snapToGrid="1""#) && xml.contains(r#"condense="0""#),
            "snapToGrid=1 + condense=0: {xml}"
        );
    }

    #[test]
    fn diagonal_shape_type_matches_hwpx_parser_codes() {
        assert_eq!(diagonal_shape_type(0), "NONE");
        assert_eq!(diagonal_shape_type(0b010), "CENTER");
        assert_eq!(diagonal_shape_type(0b011), "CENTER_BELOW");
        assert_eq!(diagonal_shape_type(0b110), "CENTER_ABOVE");
        assert_eq!(diagonal_shape_type(0b111), "ALL");
    }

    #[test]
    fn write_diagonal_omits_when_none_and_restores_type_from_code() {
        // diagonal_type 코드 0 = 대각선 없음 → 엘리먼트 자체를 출력하지 않는다.
        let none = DiagonalLine::default();
        let mut w = Writer::new(Vec::new());
        write_diagonal(&mut w, &none).expect("write");
        assert_eq!(
            String::from_utf8(w.into_inner()).unwrap(),
            "",
            "대각선 없음(code 0)은 미출력이어야 함"
        );

        // diagonal_type 1(SOLID) + width index 0(0.1mm): 선 종류는 코드에서 복원하고,
        // width 는 max(1) 변질 없이 0.1mm 그대로여야 한다.
        let solid = DiagonalLine {
            diagonal_type: 1,
            width: 0,
            color: 0,
        };
        let mut w = Writer::new(Vec::new());
        write_diagonal(&mut w, &solid).expect("write");
        assert_eq!(
            String::from_utf8(w.into_inner()).unwrap(),
            r##"<hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/>"##,
            "SOLID 대각선은 type 코드 복원 + width 무변질"
        );
    }

    #[test]
    fn write_char_pr_use_font_space_roundtrip() {
        // use_font_space=true 인 CharShape를 직렬화하면 useFontSpace="1" 이 출력되어야 한다.
        let mut doc = Document::default();
        doc.doc_info.char_shapes.push(CharShape {
            use_font_space: true,
            ..Default::default()
        });
        doc.doc_info.char_shapes.push(CharShape {
            use_font_space: false,
            ..Default::default()
        });
        let ctx = SerializeContext::collect_from_document(&doc);
        let xml = String::from_utf8(write_header(&doc, &ctx).unwrap()).unwrap();

        // 첫 번째 charPr(id=0)에 useFontSpace="1"
        let first = xml.find("useFontSpace=").expect("useFontSpace attribute");
        assert!(
            xml[first..].starts_with(r#"useFontSpace="1""#),
            "use_font_space=true → useFontSpace=\"1\": {xml}"
        );
        // 두 번째 charPr(id=1)에 useFontSpace="0"
        let second = xml[first + 1..]
            .find("useFontSpace=")
            .expect("second useFontSpace");
        let second_abs = first + 1 + second;
        assert!(
            xml[second_abs..].starts_with(r#"useFontSpace="0""#),
            "use_font_space=false → useFontSpace=\"0\": {xml}"
        );
    }

    #[test]
    fn write_numbering_splices_raw_para_heads_verbatim() {
        // Finding 21: 원본 paraHead 구간이 있으면 그대로 splice 되어, 모델의
        // 7수준으로 표현 못하는 level 8 self-closing 까지 byte-exact 복원.
        let inner = r##"<hh:paraHead start="1" level="1" numFormat="DIGIT" charPrIDRef="4294967295" checkable="0">^1.</hh:paraHead><hh:paraHead start="1" level="8" useInstWidth="0" charPrIDRef="0"/>"##;
        let n = Numbering {
            start_number: 0,
            raw_para_heads: Some(inner.to_string()),
            ..Numbering::default()
        };

        let mut writer = Writer::new(Vec::new());
        write_numbering(&mut writer, 0, &n).unwrap();
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert_eq!(
            xml,
            format!(r##"<hh:numbering id="1" start="0">{inner}</hh:numbering>"##)
        );
    }

    #[test]
    fn write_numbering_falls_back_to_skeleton_when_no_raw() {
        // 원본 구간이 없으면(HWP5 경로 등) 10 레벨 하드코딩 뼈대로 폴백.
        let n = Numbering::default();

        let mut writer = Writer::new(Vec::new());
        write_numbering(&mut writer, 0, &n).unwrap();
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert_eq!(xml.matches("<hh:paraHead").count(), 10);
        assert!(xml.starts_with(r#"<hh:numbering id="1" start="0">"#));
    }
}
