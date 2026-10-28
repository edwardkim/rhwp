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
    border_width_mm_str, para_line_wrap_str, Alignment, BorderFill, BorderLine, BorderLineType,
    CenterLine, CharShape, DiagonalLine, FillType, Font, HeadType, LineSpacingType, Numbering,
    ParaShape, Style, SubstFont, TabDef,
};
use crate::model::ColorRef;
use crate::parser::tags;

use super::canonical_defaults::FONTFACE_LANG_NAMES;
use super::context::SerializeContext;
use super::utils::{
    empty_tag_with_storage as empty_tag, end_tag_with_storage as end_tag,
    start_tag_attrs_with_storage as start_tag_attrs, start_tag_with_storage as start_tag,
    text_with_storage as text, write_xml_decl_with_storage as write_xml_decl, xml_error, xml_value,
};
use super::SerializeError;

/// `header.xml` 바이트 생성. Stage 1 진입점.
pub fn write_header(doc: &Document, ctx: &SerializeContext) -> Result<Vec<u8>, SerializeError> {
    write_header_with_storage(doc, ctx, &crate::storage::Unbounded)
}

pub(crate) fn write_header_with_storage<S: crate::storage::Storage>(
    doc: &Document,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<Vec<u8>, SerializeError>
where
    S::Error: std::fmt::Display,
{
    let mut w = Writer::new(crate::serializer::budget_cursor::BudgetCursor::new(
        storage,
        usize::MAX,
    ));
    write_xml_decl(&mut w, storage)?;

    // <hh:head> 루트 + 전체 네임스페이스 (parser가 기대하는 접두어 모두 선언)
    // secCnt 는 실제 직렬화하는 섹션 파일 수(`doc.sections`)와 일치해야 한다 (#1557).
    // doc_properties.section_count 는 파서가 갱신하지 않아 stale(1)일 수 있어, 그대로
    // 쓰면 secCnt < 실제 섹션 수가 되어 한글이 뒤 구역을 로드하지 않고 페이지가 붕괴한다.
    let sec_cnt = xml_value(storage, doc.sections.len().max(1))?;
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
        storage,
    )?;

    write_begin_num(&mut w, &doc.doc_properties, storage)?;

    // <hh:refList>: 모든 리소스 테이블을 감싸는 컨테이너
    start_tag(&mut w, "hh:refList", storage)?;
    write_fontfaces(&mut w, &doc.doc_info, ctx, storage)?;
    write_border_fills(&mut w, &doc.doc_info, ctx, storage)?;
    write_char_properties(&mut w, &doc.doc_info, ctx, storage)?;
    write_tab_properties(&mut w, &doc.doc_info, storage)?;
    write_numberings(&mut w, &doc.doc_info, storage)?;
    write_bullets(&mut w, &doc.doc_info, storage)?;
    write_para_properties(
        &mut w,
        &doc.doc_info,
        ctx,
        doc.hwpx_aux_entry("version.xml")
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
            .is_none_or(crate::parser::hwpx::physical_para_margin_units_from_version),
        storage,
    )?;
    write_styles(&mut w, &doc.doc_info, ctx, storage)?;
    // memoProperties(메모 모양 정의: 테두리/색상)는 refList 마지막 자식으로,
    // parse_memo_shape가 만드는 extra_records(HWPTAG_MEMO_SHAPE, hwpx→hwp5
    // 변환 전용)는 여기서 쓰이지 않는다. 원본 verbatim splice로 hwpx→hwpx
    // 라운드트립 시 memoPr 소실을 막는다(#3xxx).
    if let Some(memo_properties) = &doc.doc_info.memo_properties_xml {
        w.get_mut()
            .write_all(memo_properties.as_bytes())
            .map_err(|e| SerializeError::XmlError(format!("memoProperties splice: {e}")))?;
    }
    end_tag(&mut w, "hh:refList", storage)?;

    // 문서 설정 tail: 원본 HWPX 가 있으면 그대로 splice(compatibleDocument/
    // docOption/trackchageConfig 무손실 보존), 없으면 하드코딩 폴백.
    match &doc.doc_info.hwpx_head_tail {
        Some(tail) => {
            w.get_mut()
                .write_all(tail.as_bytes())
                .map_err(|e| SerializeError::XmlError(format!("head tail splice: {e}")))?;
        }
        None => {
            write_compatible_document(&mut w, storage)?;
            write_doc_option(&mut w, storage)?;
            write_track_change_config(&mut w, track_change_flags(&doc.doc_info), storage)?;
        }
    }

    end_tag(&mut w, "hh:head", storage)?;
    Ok(w.into_inner().into_inner())
}

// =====================================================================
// <hh:beginNum>
// =====================================================================
fn write_begin_num<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    props: &DocProperties,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    empty_tag(
        w,
        "hh:beginNum",
        &[
            ("page", &xml_value(storage, props.page_start_num.max(1))?),
            (
                "footnote",
                &xml_value(storage, props.footnote_start_num.max(1))?,
            ),
            (
                "endnote",
                &xml_value(storage, props.endnote_start_num.max(1))?,
            ),
            ("pic", &xml_value(storage, props.picture_start_num.max(1))?),
            ("tbl", &xml_value(storage, props.table_start_num.max(1))?),
            (
                "equation",
                &xml_value(storage, props.equation_start_num.max(1))?,
            ),
        ],
        storage,
    )
}

// =====================================================================
// <hh:fontfaces> — 7 언어 그룹
// =====================================================================
fn write_fontfaces<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    // IR의 font_faces는 항상 7개 언어 그룹을 유지한다고 기대하나,
    // 비어있거나 크기가 다를 수 있으므로 안전하게 처리.
    let groups: [&Vec<Font>; 7] =
        std::array::from_fn(|i| doc_info.font_faces.get(i).unwrap_or(&EMPTY_FONT_VEC));

    let item_cnt = groups.iter().filter(|g| !g.is_empty()).count();
    if item_cnt == 0 {
        return Ok(());
    }

    start_tag_attrs(
        w,
        "hh:fontfaces",
        &[(
            "itemCnt",
            &xml_value(storage, groups.iter().filter(|g| !g.is_empty()).count())?,
        )],
        storage,
    )?;
    for (lang_idx, fonts) in groups.iter().enumerate() {
        if fonts.is_empty() {
            continue;
        }
        let lang = FONTFACE_LANG_NAMES[lang_idx];
        start_tag_attrs(
            w,
            "hh:fontface",
            &[
                ("lang", lang),
                ("fontCnt", &xml_value(storage, fonts.len())?),
            ],
            storage,
        )?;
        for (id, font) in fonts.iter().enumerate() {
            let id_str = xml_value(storage, id)?;
            let binary_item_id_ref = resolve_font_binary_item_id_ref(
                ctx,
                font.resolved_bin_data_id,
                &font.bin_item_id_ref,
                &font.name,
                storage,
            )?;
            let font_attrs = [
                ("id", id_str.as_str()),
                ("face", font.name.as_str()),
                ("type", font_type_str(font.alt_type)),
                ("isEmbedded", if font.is_embedded { "1" } else { "0" }),
                ("binaryItemIDRef", binary_item_id_ref.as_str()),
            ];
            let font_attrs = &font_attrs[..if binary_item_id_ref.is_empty() { 4 } else { 5 }];
            // substFont(대체 글꼴)·typeInfo(파나포스 10바이트)가 IR에 있으면
            // 자식으로 복원한다. 원본 순서는 substFont → typeInfo. 둘 다 없으면
            // 종전대로 self-closing.
            if font.subst_font.is_some() || font.type_info.is_some() {
                start_tag_attrs(w, "hh:font", font_attrs, storage)?;
                if let Some(sf) = &font.subst_font {
                    write_subst_font(w, sf, ctx, storage)?;
                }
                if let Some(ti) = &font.type_info {
                    write_font_type_info(w, ti, storage)?;
                }
                end_tag(w, "hh:font", storage)?;
            } else {
                empty_tag(w, "hh:font", font_attrs, storage)?;
            }
        }
        end_tag(w, "hh:fontface", storage)?;
    }
    end_tag(w, "hh:fontfaces", storage)?;
    Ok(())
}

static EMPTY_FONT_VEC: Vec<Font> = Vec::new();

fn resolve_font_binary_item_id_ref<S: crate::storage::Storage>(
    ctx: &SerializeContext,
    resolved_bin_data_id: Option<u16>,
    original_ref: &str,
    font_name: &str,
    storage: &S,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    let value = match resolved_bin_data_id {
        Some(id) => ctx.resolve_bin_id(id).ok_or_else(|| {
            SerializeError::XmlError(format!(
                "embedded font `{font_name}` references missing BinData id {id}"
            ))
        })?,
        None => original_ref,
    };
    xml_value(storage, value)
}

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
fn write_font_type_info<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ti: &[u8; 10],
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let weight = xml_value(storage, ti[2])?;
    let proportion = xml_value(storage, ti[3])?;
    let contrast = xml_value(storage, ti[4])?;
    let stroke_variation = xml_value(storage, ti[5])?;
    let arm_style = xml_value(storage, ti[6])?;
    let letterform = xml_value(storage, ti[7])?;
    let midline = xml_value(storage, ti[8])?;
    let x_height = xml_value(storage, ti[9])?;
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
        storage,
    )
}

/// `parse_subst_font` 의 역함수. 4개 속성을 원본 순서(face·type·isEmbedded·
/// binaryItemIDRef)로 복원한다. `binaryItemIDRef` 는 비임베드 시에도 빈 문자열로
/// 항상 출력한다(한컴 원본과 동일).
fn write_subst_font<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sf: &SubstFont,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let binary_item_id_ref = resolve_font_binary_item_id_ref(
        ctx,
        sf.resolved_bin_data_id,
        &sf.bin_item_id_ref,
        &sf.face,
        storage,
    )?;
    empty_tag(
        w,
        "hh:substFont",
        &[
            ("face", &sf.face),
            ("type", font_type_str(sf.font_type)),
            ("isEmbedded", if sf.is_embedded { "1" } else { "0" }),
            ("binaryItemIDRef", binary_item_id_ref.as_str()),
        ],
        storage,
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
fn write_border_fills<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    if doc_info.border_fills.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:borderFills",
        &[("itemCnt", &xml_value(storage, doc_info.border_fills.len())?)],
        storage,
    )?;
    // HWPX borderFill의 id는 1부터 시작 (관찰값: ref_empty.hwpx).
    // 그러나 rhwp parser는 인덱스 기반으로 저장하므로 id는 배열 인덱스 그대로 사용.
    for (idx, bf) in doc_info.border_fills.iter().enumerate() {
        write_border_fill(w, idx as u16, bf, ctx, storage)?;
    }
    end_tag(w, "hh:borderFills", storage)?;
    Ok(())
}

fn write_border_fill<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u16,
    bf: &BorderFill,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let attr = effective_border_fill_attr(bf);
    // [#2965] 표 24 bit0=3D 효과, bit1=그림자 효과 — 파서가 attr 에 보존하는
    // 실값을 read_border_fill 이 되돌리지 않고 상수로 방출하던 결함 수정.
    let three_d = bool01(attr & 1 != 0);
    let shadow = bool01(attr & (1 << 1) != 0);

    // 속성 순서 (BorderFillType.cpp:64-68): id, threeD, shadow, centerLine, breakCellSeparateLine
    start_tag_attrs(
        w,
        "hh:borderFill",
        &[
            ("id", &xml_value(storage, id + 1)?), // HWPX 관찰: id는 1-based
            ("threeD", three_d),
            ("shadow", shadow),
            ("centerLine", center_line_type(bf)),
            ("breakCellSeparateLine", "0"),
        ],
        storage,
    )?;

    // 자식 순서 (BorderFillType.cpp:51-58):
    // slash, backSlash, leftBorder, rightBorder, topBorder, bottomBorder, diagonal, fillBrush
    write_diag_line(
        w,
        "hh:slash",
        diagonal_shape_type(((attr >> 2) & 0x07) as u8),
        (attr >> 8) & 0x03,
        attr & (1 << 11) != 0,
        storage,
    )?;
    write_diag_line(
        w,
        "hh:backSlash",
        diagonal_shape_type(((attr >> 5) & 0x07) as u8),
        (attr >> 10) & 0x01,
        attr & (1 << 12) != 0,
        storage,
    )?;
    write_border_line(w, "hh:leftBorder", &bf.borders[0], storage)?;
    write_border_line(w, "hh:rightBorder", &bf.borders[1], storage)?;
    write_border_line(w, "hh:topBorder", &bf.borders[2], storage)?;
    write_border_line(w, "hh:bottomBorder", &bf.borders[3], storage)?;
    write_diagonal(w, &bf.diagonal, storage)?;

    // fillBrush: 도형과 동일한 fillBrush 구조를 공유한다.
    // 종전 Stage 1 은 빈 래퍼만 출력해 winBrush(배경색)/gradation/imgBrush 를
    // 전부 잃었다. 파서가 채운 Fill 을 shape 의 검증된 역매핑으로 직렬화한다.
    super::shape::write_fill_brush_with_storage(w, &bf.fill, ctx, storage)?;

    end_tag(w, "hh:borderFill", storage)?;
    Ok(())
}

fn write_diag_line<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    type_str: &str,
    crooked: u16,
    is_counter: bool,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let crooked = xml_value(storage, crooked)?;
    let is_counter = if is_counter { "1" } else { "0" };
    empty_tag(
        w,
        name,
        &[
            ("type", type_str),
            ("Crooked", crooked.as_str()),
            ("isCounter", is_counter),
        ],
        storage,
    )
}

// [#5309] '문단 머리 정보'(표 41) attr bits 0–1 → OWPML paraHead@align 토큰.
// 0=왼쪽·1=가운데·2=오른쪽 (HWP5 스펙). 3은 미사용이라 LEFT 로 폴백.
fn numbering_head_align_str(attr: u32) -> &'static str {
    match attr & 0x03 {
        1 => "CENTER",
        2 => "RIGHT",
        _ => "LEFT",
    }
}

/// [#7418] 문단 머리 정보 속성 bit4 — 본문과의 거리 단위(0 글자 크기 비율 · 1 HWPUNIT).
fn para_head_text_offset_type_str(attr: u32) -> &'static str {
    if (attr >> 4) & 0x01 != 0 {
        "HWPUNIT"
    } else {
        "PERCENT"
    }
}

// [#2947] parser 측 parse_numbering_format_code() (표 43) 의 역매핑.
fn numbering_format_str(code: u8) -> &'static str {
    match code {
        1 => "CIRCLED_DIGIT",
        2 => "ROMAN_CAPITAL",
        3 => "ROMAN_SMALL",
        4 => "LATIN_CAPITAL",
        5 => "LATIN_SMALL",
        8 => "HANGUL_SYLLABLE",
        12 => "HANGUL_NUMBER",
        13 => "HANJA_NUMBER",
        _ => "DIGIT",
    }
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

fn write_border_line<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    line: &BorderLine,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let type_str = border_line_type_str(line.line_type);
    let width_mm = xml_value(storage, format_args!("{} mm", border_width_mm(line.width)))?;
    let color = color_hex(line.color, storage)?;
    empty_tag(
        w,
        name,
        &[("type", type_str), ("width", &width_mm), ("color", &color)],
        storage,
    )
}

fn write_diagonal<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    d: &DiagonalLine,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
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
    let width_mm = xml_value(storage, format_args!("{} mm", border_width_mm(d.width)))?;
    let color = color_hex(d.color, storage)?;
    empty_tag(
        w,
        "hh:diagonal",
        &[("type", type_str), ("width", &width_mm), ("color", &color)],
        storage,
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
        // HWPX LineType2의 DASH/DOT 표기는 HWP5 BORDER_FILL 코드 2/3과 역순이다.
        // parser::hwpx::header::parse_border_line_type와 짝을 맞춰 HWPX 왕복에서도
        // 코드 2(파선), 3(점선)을 보존한다.
        Dash => "DOT",
        Dot => "DASH",
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

fn color_hex<S: crate::storage::Storage>(c: ColorRef, storage: &S) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    // ColorRef = u32. HWP 내부 저장: 상위 바이트가 비투명 플래그(0이면 유효 색상).
    // 0xFFFFFFFF = 투명/없음 센티넬 → "none"
    if c == 0xFFFFFFFF {
        return xml_value(storage, "none");
    }
    // HWPX는 "#RRGGBB" 또는 "#AARRGGBB".
    let a = ((c >> 24) & 0xFF) as u8;
    let r = (c & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = ((c >> 16) & 0xFF) as u8;
    if a == 0 {
        xml_value(storage, format_args!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        xml_value(
            storage,
            format_args!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b),
        )
    }
}

// =====================================================================
// <hh:charProperties>
// =====================================================================
fn write_char_properties<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let _ = ctx;
    if doc_info.char_shapes.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:charProperties",
        &[("itemCnt", &xml_value(storage, doc_info.char_shapes.len())?)],
        storage,
    )?;
    for (idx, cs) in doc_info.char_shapes.iter().enumerate() {
        super::char_shapes::write_char_pr_with_storage(w, idx as u32, cs, storage)?;
    }
    end_tag(w, "hh:charProperties", storage)?;
    Ok(())
}

fn write_char_pr<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u32,
    cs: &CharShape,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    // #3500: hh:charPr 방출은 char_shapes 모듈이 단일 출처다.
    super::char_shapes::write_char_pr_with_storage(w, id, cs, storage)
}

fn bool01(b: bool) -> &'static str {
    if b {
        "1"
    } else {
        "0"
    }
}

// [#3038] 예약값 3 은 NONE. 표는 char_shape_tables 가 정본이다.
fn shadow_type_str(t: u8) -> &'static str {
    super::char_shape_tables::shadow_type_str(t)
}

// =====================================================================
// <hh:tabProperties>
// =====================================================================
fn write_tab_properties<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    if doc_info.tab_defs.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:tabProperties",
        &[("itemCnt", &xml_value(storage, doc_info.tab_defs.len())?)],
        storage,
    )?;
    for (idx, td) in doc_info.tab_defs.iter().enumerate() {
        write_tab_pr(w, idx as u16, td, storage)?;
    }
    end_tag(w, "hh:tabProperties", storage)?;
    Ok(())
}

fn write_tab_pr<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u16,
    td: &TabDef,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let attrs = [
        ("id", xml_value(storage, id)?),
        ("autoTabLeft", xml_value(storage, bool01(td.auto_tab_left))?),
        (
            "autoTabRight",
            xml_value(storage, bool01(td.auto_tab_right))?,
        ),
    ];
    let attrs_ref = attrs.each_ref().map(|(k, v)| (*k, v.as_str()));

    if td.tabs.is_empty() {
        empty_tag(w, "hh:tabPr", &attrs_ref, storage)?;
    } else {
        start_tag_attrs(w, "hh:tabPr", &attrs_ref, storage)?;
        for tab in &td.tabs {
            write_tab_item_switch(w, tab, storage)?;
        }
        end_tag(w, "hh:tabPr", storage)?;
    }
    Ok(())
}

/// tabItem 을 한컴 원본과 동일하게 `<hp:switch>`(case/default) 구조로 쓴다 (#3551).
///
/// `parse_tab_def` 의 정확한 역. 파서는 HwpUnitChar `case` 값을 ×2 하여 IR 에
/// 적재하므로(`stored = case × 2`, parser/hwpx/header.rs:1789), 역으로:
///   - `case`(HwpUnitChar) 값 = 저장값 / 2, `unit="HWPUNIT"` 동반
///   - `default` 값 = 저장값 그대로
///
/// 종전엔 switch 없이 `hh:tabItem` 만 방출해 한컴 원본의 구조가 소실됐다
/// (실측: 서울시 공개 서식 4건에서 tabItem 이 정확히 절반). 같은 파일의
/// `write_para_margin_switch` 와 동형이다.
///
/// 홀수 저장값은 `case` 로 정확히 표현할 수 없으므로(`101/2 = 50`), 파서가
/// `default != case × 2` 일 때 `default` 원값을 채택해 되살린다 — #3368 이
/// paraPr 여백에서 지적한 것과 같은 계약이다.
fn write_tab_item_switch<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    tab: &crate::model::style::TabItem,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    start_tag(w, "hp:switch", storage)?;

    start_tag_attrs(
        w,
        "hp:case",
        &[(
            "hp:required-namespace",
            "http://www.hancom.co.kr/hwpml/2016/HwpUnitChar",
        )],
        storage,
    )?;
    write_tab_item(w, tab, true, storage)?;
    end_tag(w, "hp:case", storage)?;

    start_tag(w, "hp:default", storage)?;
    write_tab_item(w, tab, false, storage)?;
    end_tag(w, "hp:default", storage)?;

    end_tag(w, "hp:switch", storage)?;
    Ok(())
}

/// `<hh:tabItem>` 한 개. `half=true` 면 HwpUnitChar case 용으로 저장값의 절반과
/// `unit="HWPUNIT"` 를 쓴다.
fn write_tab_item<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    tab: &crate::model::style::TabItem,
    half: bool,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let pos = if half { tab.position / 2 } else { tab.position };
    let pos = xml_value(storage, pos)?;
    let attrs = [
        ("pos", pos.as_str()),
        ("type", tab_type_str(tab.tab_type)),
        ("leader", tab_leader_str(tab.fill_type)),
        ("unit", "HWPUNIT"),
    ];
    let attrs = &attrs[..if half { 4 } else { 3 }];
    empty_tag(w, "hh:tabItem", attrs, storage)?;
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
        // OWPML LineType3 스펙 리터럴(Core XML schema.xml 335~349행)로 방출한다.
        // 종전엔 THIN_THICK/THICK_THIN/TRIM 이라는 비표준 이름을 썼는데, rhwp
        // 자기 자신은 다시 읽을 수 있어도(파서가 두 이름을 모두 허용) 한/글 등
        // 스펙을 따르는 다른 구현체에는 정의되지 않은 값이라 상호운용에 문제가
        // 있었다(#2857). 파서는 하위호환을 위해 옛 이름도 계속 받는다.
        9 => "SLIM_THICK",
        10 => "THICK_SLIM",
        11 => "SLIM_THICK_SLIM",
        _ => "NONE",
    }
}

// =====================================================================
// <hh:numberings>
// =====================================================================
fn write_numberings<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    if doc_info.numberings.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:numberings",
        &[("itemCnt", &xml_value(storage, doc_info.numberings.len())?)],
        storage,
    )?;
    for (idx, n) in doc_info.numberings.iter().enumerate() {
        write_numbering(w, idx as u16, n, storage)?;
    }
    end_tag(w, "hh:numberings", storage)?;
    Ok(())
}

fn write_numbering<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u16,
    n: &Numbering,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    start_tag_attrs(
        w,
        "hh:numbering",
        &[
            ("id", &xml_value(storage, id + 1)?), // 관찰: 1-based
            ("start", &xml_value(storage, n.start_number)?),
        ],
        storage,
    )?;
    // 원본 HWPX paraHead 영역이 있으면 그대로 splice(10수준 + align/
    // useInstWidth/autoIndent/checkable/형식문자열 무손실 복원). 모델의 7수준
    // NumberingHead 로는 표현 못하는 정보를 보존한다. 없으면(HWP5 경로 등)
    // 아래 하드코딩 뼈대로 폴백.
    if let Some(raw) = &n.raw_para_heads {
        w.get_mut()
            .write_all(raw.as_bytes())
            .map_err(|e| SerializeError::XmlError(format!("numbering paraHead splice: {e}")))?;
        end_tag(w, "hh:numbering", storage)?;
        return Ok(());
    }
    // Stage 1: 10 레벨 paraHead 뼈대 출력. 실제 값은 NumberingHead 참조해 생성.
    for level in 0..10usize {
        let idx = level.min(6);
        let h = &n.heads[idx];
        let start = n.level_start_numbers.get(idx).copied().unwrap_or(1);
        let level_s = xml_value(storage, level + 1)?;
        let start_s = xml_value(storage, start)?;
        let wa = xml_value(storage, h.width_adjust)?;
        // [#2947] h.number_format 을 무시하고 "DIGIT" 로 고정 방출하면 HWP5 경유
        // 저장 시 로마자/한글/원문자 등 문단 번호 형식이 전부 아라비아 숫자로 뒤바뀐다.
        let num_format = numbering_format_str(h.number_format);
        let text_offset_s = xml_value(storage, h.text_distance)?;
        let char_pr_id_ref_s = xml_value(storage, h.char_shape_id)?;
        // [#5309] '문단 머리 정보'(표 41) attr 저위 비트에서 정렬·번호너비·자동내어쓰기를
        // 유도한다. 종전엔 align="LEFT"/useInstWidth="1"/autoIndent="1" 상수만 방출해,
        // HWP5→HWPX 저장 때마다 문단 번호의 정렬·들여쓰기 설정이 기본값으로 리셋됐다.
        // NumberingHead 모델엔 이 값의 lexical 필드가 없어 attr 비트가 유일한 원천이다.
        // 비트↔토큰은 한컴 저작 HWPX 쌍(143E433F…)으로 직접 1:1 대응 확증. numFormat
        // de-hardcode(#2947)와 동형.
        let align = numbering_head_align_str(h.attr);
        let use_inst_width = if (h.attr >> 2) & 0x01 != 0 { "1" } else { "0" };
        let auto_indent = if (h.attr >> 3) & 0x01 != 0 { "1" } else { "0" };
        let attrs = [
            ("start", start_s.as_str()),
            ("level", level_s.as_str()),
            ("align", align),
            ("useInstWidth", use_inst_width),
            ("autoIndent", auto_indent),
            ("widthAdjust", wa.as_str()),
            ("textOffsetType", para_head_text_offset_type_str(h.attr)),
            ("textOffset", text_offset_s.as_str()),
            ("numFormat", num_format),
            ("charPrIDRef", char_pr_id_ref_s.as_str()),
            ("checkable", "0"),
        ];
        // [#3838] 번호 형식 문자열은 `hh:paraHead` 의 **텍스트 내용**이다("^1." 등).
        // 자기닫힘으로 내보내면 형식이 사라져 문단 번호가 아예 렌더되지 않는다.
        // 한컴 원본 실측: paraHead 232개 중 172개가 내용을 갖는다.
        let format_str = n.level_formats.get(idx).map(String::as_str).unwrap_or("");
        if format_str.is_empty() {
            empty_tag(w, "hh:paraHead", &attrs, storage)?;
        } else {
            start_tag_attrs(w, "hh:paraHead", &attrs, storage)?;
            text(w, format_str, storage)?;
            end_tag(w, "hh:paraHead", storage)?;
        }
    }
    end_tag(w, "hh:numbering", storage)?;
    Ok(())
}

// =====================================================================
// <hh:bullets> — 글머리표 정의
//
// 종전 직렬화는 bullets 를 전혀 쓰지 않아 라운드트립에서 글머리표 정의(❏/※/❍ 등)가
// 소실되고, 글머리표 문단의 마커 글리프가 렌더에서 사라졌다. 파서(parse_bullet_hwpx)는
// `char`/`useImage` 만 읽으므로 그 둘 + paraHead 뼈대를 방출하면 round-trip 무손실이다.
// =====================================================================
fn write_bullets<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    if doc_info.bullets.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:bullets",
        &[("itemCnt", &xml_value(storage, doc_info.bullets.len())?)],
        storage,
    )?;
    for (idx, b) in doc_info.bullets.iter().enumerate() {
        write_bullet(w, idx as u16, b, storage)?;
    }
    end_tag(w, "hh:bullets", storage)?;
    Ok(())
}

fn write_bullet<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u16,
    b: &crate::model::style::Bullet,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let id_s = xml_value(storage, id + 1)?; // 관찰: 1-based, ParaShape.numbering_id 참조와 정합
    let char_s = xml_value(storage, b.bullet_char)?;
    let use_image = if b.image_bullet > 0 { "1" } else { "0" };
    // [#3028] checkedChar(체크 글머리표 문자) — 값이 있을 때만 방출한다.
    let has_checked_char = b.check_bullet_char != '\0';
    let checked_char_s = xml_value(storage, b.check_bullet_char)?;
    let attrs = [
        ("id", id_s.as_str()),
        ("char", char_s.as_str()),
        if has_checked_char {
            ("checkedChar", checked_char_s.as_str())
        } else {
            ("useImage", use_image)
        },
        ("useImage", use_image),
    ];
    let attrs = &attrs[..if has_checked_char { 4 } else { 3 }];
    start_tag_attrs(w, "hh:bullet", attrs, storage)?;
    // 원본 HWPX paraHead 구간이 있으면(=이 파일에서 파싱된 bullet) 그대로 splice 해
    // align/useInstWidth/autoIndent/textOffsetType/checkable 등 Bullet 필드로 표현
    // 못하는 속성까지 무손실 복원(numbering.raw_para_heads 와 동일 패턴, #2790).
    // 없으면(HWP5 경로 등) widthAdjust/textOffset/charPrIDRef 만 필드값으로 채운
    // 뼈대로 폴백 — checkable 은 checkedChar 유무를 따른다(#3028).
    if let Some(raw) = &b.raw_para_head {
        w.get_mut()
            .write_all(raw.as_bytes())
            .map_err(|e| SerializeError::XmlError(format!("bullet paraHead splice: {e}")))?;
    } else {
        empty_tag(
            w,
            "hh:paraHead",
            &[
                ("level", "0"),
                ("align", numbering_head_align_str(b.attr)),
                (
                    "useInstWidth",
                    if (b.attr >> 2) & 0x01 != 0 { "1" } else { "0" },
                ),
                (
                    "autoIndent",
                    if (b.attr >> 3) & 0x01 != 0 { "1" } else { "0" },
                ),
                ("widthAdjust", &xml_value(storage, b.width_adjust)?),
                ("textOffsetType", para_head_text_offset_type_str(b.attr)),
                ("textOffset", &xml_value(storage, b.text_distance)?),
                ("numFormat", "DIGIT"),
                ("charPrIDRef", &xml_value(storage, b.char_shape_id)?),
                ("checkable", if has_checked_char { "1" } else { "0" }),
            ],
            storage,
        )?;
    }
    end_tag(w, "hh:bullet", storage)?;
    Ok(())
}

// =====================================================================
// <hh:paraProperties>
// =====================================================================
fn write_para_properties<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
    physical_margin_units: bool,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let _ = ctx;
    if doc_info.para_shapes.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:paraProperties",
        &[("itemCnt", &xml_value(storage, doc_info.para_shapes.len())?)],
        storage,
    )?;
    for (idx, ps) in doc_info.para_shapes.iter().enumerate() {
        if !physical_margin_units && !ps.hwpx_plain_para_margin {
            // 구판 case는 IR 값, default는 그 두 배다. 기존 switch 작성기의
            // 반감 규칙에 역단위를 전달해 읽기·재저장 결과를 보존한다.
            let mut stored = storage.clone_record(ps).map_err(xml_error)?;
            stored.margin_left = stored.margin_left.saturating_mul(2);
            stored.margin_right = stored.margin_right.saturating_mul(2);
            stored.indent = stored.indent.saturating_mul(2);
            stored.spacing_before = stored.spacing_before.saturating_mul(2);
            stored.spacing_after = stored.spacing_after.saturating_mul(2);
            if stored.line_spacing_type != LineSpacingType::Percent {
                stored.line_spacing = stored.line_spacing.saturating_mul(2);
            }
            write_para_pr(w, idx as u16, &stored, storage)?;
        } else {
            write_para_pr(w, idx as u16, ps, storage)?;
        }
    }
    end_tag(w, "hh:paraProperties", storage)?;
    Ok(())
}

fn write_para_pr<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u16,
    ps: &ParaShape,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    // 속성 순서 (ParaShapeType.cpp:62-68): id, tabPrIDRef, condense,
    // fontLineHeight, snapToGrid, suppressLineNumbers, checked
    //
    // condense/fontLineHeight/snapToGrid 는 attr1 비트로 보존된다(파서 역매핑):
    //   snapToGrid = bit8, condense = bits9..15, fontLineHeight = bit22.
    // 종전엔 상수("0"/"0"/"1")로 하드코딩해 condense(20 등)와 snapToGrid(0)을 잃었다.
    let condense = xml_value(storage, (ps.attr1 >> 9) & 0x7f)?;
    let font_line_height = xml_value(storage, (ps.attr1 >> 22) & 1)?;
    let snap_to_grid = xml_value(storage, (ps.attr1 >> 8) & 1)?;
    start_tag_attrs(
        w,
        "hh:paraPr",
        &[
            ("id", &xml_value(storage, id)?),
            ("tabPrIDRef", &xml_value(storage, ps.tab_def_id)?),
            ("condense", &condense),
            ("fontLineHeight", &font_line_height),
            ("snapToGrid", &snap_to_grid),
            ("suppressLineNumbers", "0"),
            ("checked", "0"),
        ],
        storage,
    )?;

    // 자식 순서 (한컴 원본 관찰):
    // align, heading, breakSetting, autoSpacing, switch(margin+lineSpacing), border
    //
    // 종전엔 align@vertical, breakSetting@{breakNonLatinWord, widowOrphan,
    // keepWithNext, keepLines, pageBreakBefore} 를 상수로 하드코딩해, 파서가
    // attr1 비트로 보존한 값을 직렬화에서 모두 잃었다(예: vertical=CENTER →
    // BASELINE, breakNonLatinWord=BREAK_WORD → KEEP_WORD). 이제 보존 비트에서
    // 역매핑한다. lineWrap 은 attr2 bits 0-1 에서 역매핑한다(#6875).
    let vertical = vertical_alignment_str((ps.attr1 >> 20) & 0x03);
    // attr1 bit7: KEEP_WORD=1, BREAK_WORD=0 (parse_para_shape_child 와 정합).
    let break_non_latin = if (ps.attr1 >> 7) & 1 == 1 {
        "KEEP_WORD"
    } else {
        "BREAK_WORD"
    };
    // [#1986] breakLatinWord 는 HWPX 원문 보존값(HWPX 소스). HWP5 소스는 파서가 이 렉시컬
    // 필드를 attr1 비트로만 두고 채우지 않아(doc_info.rs) 종전엔 무조건 "KEEP_WORD" 로
    // 강등했다 — 라틴 줄나눔 설정(하이픈·글자 단위)이 h2x 저장에서 통째로 사라졌다
    // (10k 코퍼스 실측: 500 문서 중 222 문서가 비-KEEP 설정, para_shape 2,810개).
    // breakNonLatinWord(bit7)와 같은 축으로 HWP5 attr1 bits5-6 에서 역매핑한다. 렉시컬
    // 값이 있으면(HWPX 소스) 그것을 우선하므로 x2x 는 종전과 동일하다.
    let break_latin = ps
        .break_latin_word
        .as_deref()
        .unwrap_or_else(|| latin_break_from_attr1((ps.attr1 >> 5) & 0x03));
    let widow_orphan = xml_value(storage, (ps.attr1 >> 16) & 1)?;
    let keep_with_next = xml_value(storage, (ps.attr1 >> 17) & 1)?;
    let keep_lines = xml_value(storage, (ps.attr1 >> 18) & 1)?;
    let page_break_before = xml_value(storage, (ps.attr1 >> 19) & 1)?;
    let auto_space_kr_en = xml_value(storage, (ps.attr2 >> 4) & 1)?;
    let auto_space_kr_num = xml_value(storage, (ps.attr2 >> 5) & 1)?;
    empty_tag(
        w,
        "hh:align",
        &[
            ("horizontal", alignment_str(ps.alignment)),
            ("vertical", vertical),
        ],
        storage,
    )?;
    empty_tag(
        w,
        "hh:heading",
        &[
            ("type", head_type_str(ps.head_type)),
            ("idRef", &xml_value(storage, ps.numbering_id)?),
            ("level", &xml_value(storage, ps.para_level)?),
        ],
        storage,
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
            // [#6875] 종전 상수 "BREAK" — "한 줄로 입력" 문단(attr2 bits 0-1 = 1)을
            // 한/글이 여러 줄로 다시 나눠 h2x 쪽수가 갈렸다.
            ("lineWrap", para_line_wrap_str(ps.attr2)),
        ],
        storage,
    )?;

    empty_tag(
        w,
        "hh:autoSpacing",
        &[
            ("eAsianEng", &auto_space_kr_en),
            ("eAsianNum", &auto_space_kr_num),
        ],
        storage,
    )?;

    // margin + lineSpacing 은 한컴 원본과 동일하게 <hp:switch>(case/default)로 감싼다.
    //
    // [#4898] 단, 원본 HWPX 가 switch 없이 평문으로 적었으면 그 표기를 지킨다. 한글은
    // 패키지 xmlVersion 1.4 이상의 물리 여백만 공통 IR의 절반으로 되쓴다.
    // 고정 줄간격은 여백과 별도 계약이므로 기존 평문 저장값을 유지한다.
    if ps.hwpx_plain_para_margin {
        write_para_margin(w, ps, ps.hwpx_plain_para_margin_physical, storage)?;
        write_para_line_spacing(w, ps, false, storage)?;
    } else {
        write_para_margin_switch(w, ps, storage)?;
    }

    let border_connect = if (ps.attr1 >> 28) & 1 != 0 { "1" } else { "0" };
    let border_ignore_margin = if (ps.attr1 >> 29) & 1 != 0 { "1" } else { "0" };

    empty_tag(
        w,
        "hh:border",
        &[
            ("borderFillIDRef", &xml_value(storage, ps.border_fill_id)?),
            ("offsetLeft", &xml_value(storage, ps.border_spacing[0])?),
            ("offsetRight", &xml_value(storage, ps.border_spacing[1])?),
            ("offsetTop", &xml_value(storage, ps.border_spacing[2])?),
            ("offsetBottom", &xml_value(storage, ps.border_spacing[3])?),
            ("connect", border_connect),
            ("ignoreMargin", border_ignore_margin),
        ],
        storage,
    )?;

    end_tag(w, "hh:paraPr", storage)?;
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
fn write_para_margin_switch<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ps: &ParaShape,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    start_tag(w, "hp:switch", storage)?;

    start_tag_attrs(
        w,
        "hp:case",
        &[(
            "hp:required-namespace",
            "http://www.hancom.co.kr/hwpml/2016/HwpUnitChar",
        )],
        storage,
    )?;
    write_para_margin(w, ps, true, storage)?;
    write_para_line_spacing(w, ps, true, storage)?;
    end_tag(w, "hp:case", storage)?;

    start_tag(w, "hp:default", storage)?;
    write_para_margin(w, ps, false, storage)?;
    write_para_line_spacing(w, ps, false, storage)?;
    end_tag(w, "hp:default", storage)?;

    end_tag(w, "hp:switch", storage)?;
    Ok(())
}

/// `<hh:margin>` (자식 5개: intent/left/right/prev/next). `half=true` 면 HwpUnitChar
/// case 용으로 저장값의 절반을 쓴다.
fn write_para_margin<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ps: &ParaShape,
    half: bool,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    start_tag(w, "hh:margin", storage)?;
    write_margin_child(w, "hc:intent", ps.indent, half, storage)?;
    write_margin_child(w, "hc:left", ps.margin_left, half, storage)?;
    write_margin_child(w, "hc:right", ps.margin_right, half, storage)?;
    write_margin_child(w, "hc:prev", ps.spacing_before, half, storage)?;
    write_margin_child(w, "hc:next", ps.spacing_after, half, storage)?;
    end_tag(w, "hh:margin", storage)?;
    Ok(())
}

/// `<hh:lineSpacing>`. HwpUnitChar case(`case=true`)에서 PERCENT 는 저장값과 동일,
/// 그 외 유형은 저장값의 절반을 쓴다(파서가 ×2 적재).
fn write_para_line_spacing<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ps: &ParaShape,
    case: bool,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
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
            ("value", &xml_value(storage, value)?),
            ("unit", "HWPUNIT"),
        ],
        storage,
    )
}

/// margin 자식(`<hc:intent value="…" unit="HWPUNIT"/>`). 한컴 원본 속성 순서는
/// value, unit 이며 네임스페이스는 `hc:` 다.
/// `<hh:margin>` 자식 하나. `half=true`(HwpUnitChar `case`)에서 하위 비트가1이면
/// 기본 스타일 em에 대한 상대 단위 CHAR, 0이면 절대 HWPUNIT으로 쓴다.
/// 같은 CHAR80도 기본 스타일10pt에서는400HU,20pt에서는800HU다.
/// 나머지는 단위 비트를 제거한 값이며 0.5HU 반올림 표시가 아니다.
///
/// [#6875] 종전에는 단위를 `HWPUNIT` 으로 고정하고 값만 `x / 2` 로 적었다. 그러면 홀수의
/// 최하위 비트(= 한컴이 `CHAR` 로 표시하는 그 자리)가 사라지고, 한/글은 `case` 를 우선
/// 읽으므로 **문단 간격을 종전보다 작게** 잡는다. 07939(소방방재 점검메뉴얼)에서 문단마다
/// 22.02pt 가 18.78pt 로 줄어 558쪽이 545쪽이 됐다(−13쪽).
///
/// 한컴 자신의 HWP→HWPX 변환본에서 이 규칙은 예외가 없다 — 같은 문서 margin 항목
/// 2,170개 전수에서 `저장값이 홀수 ⟺ case 단위가 CHAR` 가 위반 0 이다.
///
/// 값은 `stored = case × 2 (+1 when CHAR)` 로 되돌릴 수 있게 적는다(파서의 정확한 역).
fn write_margin_child<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    stored: i32,
    half: bool,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    if !half {
        return empty_tag(
            w,
            name,
            &[("value", &xml_value(storage, stored)?), ("unit", "HWPUNIT")],
            storage,
        );
    }
    let odd = stored % 2 != 0;
    let value = if odd { (stored - 1) / 2 } else { stored / 2 };
    empty_tag(
        w,
        name,
        &[
            ("value", &xml_value(storage, value)?),
            ("unit", if odd { "CHAR" } else { "HWPUNIT" }),
        ],
        storage,
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

/// HWP5 para_shape attr1 bits5-6 → HWPX `breakLatinWord` 토큰(라틴 문자의 줄나눔 단위).
/// `breakNonLatinWord`(bit7) 역매핑과 같은 축이다. HWPX 소스는 렉시컬 값을 우선하므로
/// 이 함수는 HWP5 소스(렉시컬 None)에서만 쓰인다. 0(=KEEP_WORD)은 코퍼스 지배값이자
/// 종전 기본과 같아, 값이 다른 문서만 실제로 바뀐다.
fn latin_break_from_attr1(bits: u32) -> &'static str {
    match bits {
        1 => "HYPHENATION",
        2 => "BREAK_WORD",
        _ => "KEEP_WORD",
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
fn write_styles<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    doc_info: &DocInfo,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let _ = ctx;
    if doc_info.styles.is_empty() {
        return Ok(());
    }
    start_tag_attrs(
        w,
        "hh:styles",
        &[("itemCnt", &xml_value(storage, doc_info.styles.len())?)],
        storage,
    )?;
    for (idx, st) in doc_info.styles.iter().enumerate() {
        write_style(w, idx as u16, st, storage)?;
    }
    end_tag(w, "hh:styles", storage)?;
    Ok(())
}

fn write_style<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u16,
    st: &Style,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let type_str = if st.style_type == 1 { "CHAR" } else { "PARA" };
    empty_tag(
        w,
        "hh:style",
        &[
            ("id", &xml_value(storage, id)?),
            ("type", type_str),
            ("name", &st.local_name),
            ("engName", &st.english_name),
            ("paraPrIDRef", &xml_value(storage, st.para_shape_id)?),
            ("charPrIDRef", &xml_value(storage, st.char_shape_id)?),
            ("nextStyleIDRef", &xml_value(storage, st.next_style_id)?),
            // 파서가 Style.lang_id 로 읽는 값을 그대로 되돌린다. 종전 "1042"
            // 하드코딩은 비한국어 langID(예: 1033)를 왕복마다 1042 로 바꿨다.
            ("langID", &xml_value(storage, st.lang_id)?),
            ("lockForm", bool01(st.lock_form)),
        ],
        storage,
    )
}

// =====================================================================
// <hh:compatibleDocument>, <hh:docOption>, <hh:trackchageConfig>
// =====================================================================
fn write_compatible_document<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    start_tag_attrs(
        w,
        "hh:compatibleDocument",
        &[("targetProgram", "HWP201X")],
        storage,
    )?;
    // 한컴 산출물(2014 저장 빈 문서 · samples/hwpx/ref/ref_empty.hwpx)과 같이 자기닫힘으로
    // 쓴다. 종전의 <hh:char/><hh:paragraph/><hh:section/><hh:object/><hh:field/> 다섯은
    // HWP5 LayoutCompatibility 레코드의 필드 묶음 이름이지 HWPX 요소가 아니며, 엄격한
    // 소비자(한글 2014)는 이 다섯이 있으면 열기에서 종료한다. 원본 꼬리를 splice 하는
    // 경로(`write_header_splices_doc_settings_tail_verbatim`)와 같은 꼴이 된다.
    empty_tag(w, "hh:layoutCompatibility", &[], storage)?;
    end_tag(w, "hh:compatibleDocument", storage)?;
    Ok(())
}

fn write_doc_option<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    start_tag(w, "hh:docOption", storage)?;
    empty_tag(
        w,
        "hh:linkinfo",
        &[("path", ""), ("pageInherit", "0"), ("footnoteInherit", "0")],
        storage,
    )?;
    end_tag(w, "hh:docOption", storage)?;
    Ok(())
}

/// `<hh:trackchageConfig flags>` 값을 원본 DocInfo 에서 되찾는다.
///
/// 종전에는 `0` 을 하드코딩했다. 이 값은 이름과 달리 **조판에 영향을 준다** — 한글이
/// `flags="0"` 문서를 열면 같은 내용이 한 쪽 더 늘어난다(00295 실측: 원본 1쪽,
/// rhwp 저장본 2쪽, 이 값만 56 으로 바꾸면 다시 1쪽). 10k 스윕의 h2x 쪽수 불일치
/// 978 경로 중 +1쪽이 526 건인데, 그 계열의 근인이다.
///
/// HWP5 는 `HWPTAG_TRACKCHANGE` 레코드 첫 `u32` 에 싣는다(파서가 모델링하지 않아
/// `extra_records` 에 원시 보존된다). 레코드가 없으면 56 — 한컴 실측이다:
/// 코퍼스 6,473 문서 중 4,588 이 레코드를 갖고 그중 4,555 가 56 이며, 레코드가
/// **없는** 문서(00000·00001)를 한글로 저장해도 `flags="56"` 이 나온다.
fn track_change_flags(doc_info: &DocInfo) -> u32 {
    doc_info
        .extra_records
        .iter()
        .find(|r| r.tag_id == tags::HWPTAG_TRACKCHANGE)
        .and_then(|r| r.data.get(..4))
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .unwrap_or(TRACK_CHANGE_FLAGS_DEFAULT)
}

/// 한컴이 `HWPTAG_TRACKCHANGE` 없는 문서에 쓰는 값 (실측).
const TRACK_CHANGE_FLAGS_DEFAULT: u32 = 56;

fn write_track_change_config<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    flags: u32,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    empty_tag(
        w,
        "hh:trackchageConfig",
        &[("flags", &xml_value(storage, flags)?)],
        storage,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `trackchageConfig flags` 는 원본 값을 실어야 한다.
    ///
    /// 이름과 달리 조판에 영향을 준다 — 한글은 `flags="0"` 문서를 같은 내용인데도
    /// 한 쪽 더 늘려 연다(00295 실측: 원본 1쪽 / 저장본 2쪽 / 이 값만 56 으로
    /// 바꾸면 1쪽). 종전에는 `0` 을 하드코딩했다.
    #[test]
    fn track_change_flags_come_from_the_original_record() {
        use crate::model::document::RawRecord;

        let mut info = DocInfo::default();
        assert_eq!(
            track_change_flags(&info),
            56,
            "레코드가 없으면 한컴이 쓰는 값(56)으로 떨어진다 — 00000·00001 실측"
        );

        info.extra_records.push(RawRecord {
            tag_id: tags::HWPTAG_TRACKCHANGE,
            level: 0,
            data: 60_u32.to_le_bytes().to_vec(),
        });
        assert_eq!(track_change_flags(&info), 60, "원본 값을 그대로 옮긴다");
    }

    /// 4바이트가 안 되는 손상 레코드는 기본값으로 떨어진다 — 쓰레기값을 쓰지 않는다.
    #[test]
    fn truncated_track_change_record_falls_back_to_the_default() {
        use crate::model::document::RawRecord;

        let mut info = DocInfo::default();
        info.extra_records.push(RawRecord {
            tag_id: tags::HWPTAG_TRACKCHANGE,
            level: 0,
            data: vec![0x38, 0x00],
        });
        assert_eq!(track_change_flags(&info), 56);
    }

    use crate::parser::hwpx::parse_hwpx;

    #[test]
    fn write_style_emits_ir_lang_id() {
        // 파서가 읽은 Style.lang_id 가 그대로 방출돼야 한다.
        // 종전엔 "1042" 하드코딩으로 비한국어 langID 가 왕복마다 뭉개졌다.
        let st = Style {
            lang_id: 1033,
            ..Default::default()
        };
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_style(&mut w, 0, &st, &crate::storage::Unbounded).expect("write_style");
        let xml = String::from_utf8(w.into_inner()).unwrap();
        assert!(
            xml.contains(r#"langID="1033""#),
            "Style.lang_id 가 방출돼야 함: {xml}"
        );
    }

    #[test]
    fn write_style_emits_ir_lock_form() {
        // [Task #2839] 파서가 읽은 Style.lock_form 이 그대로 방출돼야 한다.
        // 종전엔 "0" 하드코딩으로 lockForm="1" 스타일이 왕복마다 잠금 해제로 뭉개졌다.
        let st = Style {
            lock_form: true,
            ..Default::default()
        };
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_style(&mut w, 0, &st, &crate::storage::Unbounded).expect("write_style");
        let xml = String::from_utf8(w.into_inner()).unwrap();
        assert!(
            xml.contains(r#"lockForm="1""#),
            "Style.lock_form 이 방출돼야 함: {xml}"
        );
    }

    #[test]
    fn task_m100_shadow_type_str_reserved_value_maps_to_none() {
        // [#3038] 계약(0~2) 밖의 예약값(3)은 CONTINUOUS 로 둔갑하지 않고
        // 안전한 기본값 NONE 으로 방출돼야 한다.
        assert_eq!(shadow_type_str(0), "NONE");
        assert_eq!(shadow_type_str(1), "DROP");
        assert_eq!(shadow_type_str(2), "CONTINUOUS");
        assert_eq!(shadow_type_str(3), "NONE");
        assert_eq!(shadow_type_str(99), "NONE");
    }

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
    fn write_header_preserves_memo_properties_roundtrip() {
        // aift.hwpx 실물 헤더에는 refList 마지막 자식으로
        // <hh:memoProperties itemCnt="1"><hh:memoPr .../></hh:memoProperties>
        // 가 있다. parse_memo_shape()가 만드는 extra_records(HWPTAG_MEMO_SHAPE)는
        // hwpx→hwp5 변환 전용이라 write_header가 참조하지 않으므로, splice
        // 보존이 없으면 hwpx→hwpx 라운드트립에서 memoPr(메모 모양: 테두리/색상)이
        // 통째로 사라진다.
        let bytes = include_bytes!("../../../samples/hwpx/aift.hwpx");
        let doc = parse_hwpx(bytes).expect("parse aift.hwpx");
        assert!(
            doc.doc_info.memo_properties_xml.is_some(),
            "파서가 memoProperties 원문을 보존해야 함"
        );
        let ctx = SerializeContext::collect_from_document(&doc);
        let header_xml =
            String::from_utf8(write_header(&doc, &ctx).expect("write header")).unwrap();
        assert!(
            header_xml.contains("<hh:memoProperties"),
            "재직렬화된 header.xml에 memoProperties가 있어야 함: {header_xml}"
        );
        assert!(
            header_xml.contains(r#"lineType="SOLID""#)
                && header_xml.contains(r##"fillColor="#CBFF99""##),
            "memoPr 원본 속성(lineType/fillColor 등)이 그대로 보존돼야 함: {header_xml}"
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
            xml.contains("<hh:layoutCompatibility/></hh:compatibleDocument>"),
            "원본 부재 시 폴백도 한컴 산출물과 같은 자기닫힘이어야 함: {xml}"
        );
        assert!(
            !xml.contains("<hh:char/>"),
            "폴백에 HWPX 요소가 아닌 자식(<hh:char/> 등)이 없어야 함: {xml}"
        );
        // 종전에는 `flags="0"` 을 못 박았다. 그 값은 실측으로 틀렸다 — 한글은 0 인 문서를
        // 같은 내용인데도 한 쪽 더 늘려 연다(00295: 원본 1쪽 / 저장본 2쪽 / 이 값만 56 으로
        // 바꾸면 1쪽). 이제 원본 레코드 값을 싣고, 없으면 한컴이 쓰는 56 으로 떨어진다.
        assert!(
            xml.contains(r#"<hh:trackchageConfig flags="56"/>"#),
            "폴백은 한컴 기본값 56 이어야 한다: {xml}"
        );
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
        write_font_type_info(&mut writer, &ti, &crate::storage::Unbounded).expect("write typeInfo");
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
            resolved_bin_data_id: None,
        };
        let mut writer = Writer::new(Vec::new());
        write_subst_font(
            &mut writer,
            &sf,
            &SerializeContext::default(),
            &crate::storage::Unbounded,
        )
        .expect("write substFont");
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
                resolved_bin_data_id: None,
            }),
            ..Default::default()
        });
        let mut writer = Writer::new(Vec::new());
        write_fontfaces(
            &mut writer,
            &doc_info,
            &SerializeContext::default(),
            &crate::storage::Unbounded,
        )
        .expect("write fontfaces");
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
        write_fontfaces(
            &mut writer,
            &doc_info,
            &SerializeContext::default(),
            &crate::storage::Unbounded,
        )
        .expect("write fontfaces");
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
    fn embedded_font_references_use_the_serialized_manifest_id() {
        use crate::model::bin_data::BinDataContent;

        let mut doc = Document::default();
        doc.bin_data_content.push(BinDataContent {
            id: 2,
            data: vec![0, 1, 2].into(),
            extension: "ttf".to_string(),
        });
        doc.doc_info.font_faces = vec![Vec::new(); 7];
        doc.doc_info.font_faces[0].push(Font {
            name: "Embedded Face".to_string(),
            is_embedded: true,
            bin_item_id_ref: "font-resource-alpha".to_string(),
            resolved_bin_data_id: Some(2),
            subst_font: Some(SubstFont {
                face: "Embedded Substitute".to_string(),
                font_type: 1,
                is_embedded: true,
                bin_item_id_ref: "font-resource-beta".to_string(),
                resolved_bin_data_id: Some(2),
            }),
            ..Default::default()
        });

        let ctx = SerializeContext::collect_from_document(&doc);
        assert_eq!(ctx.resolve_bin_id(2), Some("image2"));
        let xml = String::from_utf8(write_header(&doc, &ctx).unwrap()).unwrap();
        assert_eq!(xml.matches(r#"binaryItemIDRef="image2""#).count(), 2);
        assert!(!xml.contains("font-resource-alpha"));
        assert!(!xml.contains("font-resource-beta"));
    }

    #[test]
    fn embedded_font_reference_to_missing_bin_data_is_an_error() {
        let mut doc = Document::default();
        doc.doc_info.font_faces = vec![Vec::new(); 7];
        doc.doc_info.font_faces[0].push(Font {
            name: "Missing Embedded Face".to_string(),
            is_embedded: true,
            bin_item_id_ref: "stale-font-ref".to_string(),
            resolved_bin_data_id: Some(9),
            ..Default::default()
        });

        let ctx = SerializeContext::collect_from_document(&doc);
        let error = write_header(&doc, &ctx).expect_err("dangling font reference must fail");
        assert!(error.to_string().contains("missing BinData id 9"));
        assert!(!error.to_string().contains("stale-font-ref"));
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
            &crate::storage::Unbounded,
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
    fn write_border_fill_uses_hwpx_dash_dot_contract() {
        let mut bf = BorderFill::default();
        bf.borders[0].line_type = BorderLineType::Dash;
        bf.borders[1].line_type = BorderLineType::Dot;

        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            0,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
            &crate::storage::Unbounded,
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"<hh:leftBorder type="DOT""#),
            "HWP5 code 2(Dash)는 HWPX DOT으로 방출돼야 함: {xml}"
        );
        assert!(
            xml.contains(r#"<hh:rightBorder type="DASH""#),
            "HWP5 code 3(Dot)는 HWPX DASH로 방출돼야 함: {xml}"
        );
    }

    #[test]
    fn write_border_fill_preserves_three_d_and_shadow_bits() {
        // [#2965] bf.attr bit0=3D, bit1=그림자. 종전엔 threeD/shadow 가 항상
        // "0" 으로 하드코딩되어 파서가 보존한 값이 왕복에서 소실됐다.
        let mut bf = BorderFill::default();
        bf.attr = 0b11; // bit0(3D)=1, bit1(shadow)=1

        let mut writer = Writer::new(Vec::new());
        write_border_fill(
            &mut writer,
            0,
            &bf,
            &SerializeContext::collect_from_document(&Default::default()),
            &crate::storage::Unbounded,
        )
        .expect("write borderFill");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"threeD="1""#) && xml.contains(r#"shadow="1""#),
            "attr bit0/bit1 이 threeD/shadow 로 방출돼야 함: {xml}"
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
            &crate::storage::Unbounded,
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
            &crate::storage::Unbounded,
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
            &crate::storage::Unbounded,
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
            &crate::storage::Unbounded,
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
            &crate::storage::Unbounded,
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
    fn write_tab_pr_emits_tab_item_switch_case_default() {
        // [#3551] tabItem 은 한컴 원본과 동일하게 <hp:switch>(case=저장값/2 +
        // unit="HWPUNIT", default=저장값) 로 나가야 한다. 종전엔 switch 없이
        // hh:tabItem 만 방출해 실물 문서에서 tabItem 이 정확히 절반이 됐다.
        use crate::model::style::{TabDef, TabItem};

        let mut td = TabDef::default();
        td.tabs = vec![TabItem {
            position: 3288,
            tab_type: 0,
            fill_type: 0,
        }];

        let mut writer = Writer::new(Vec::new());
        write_tab_pr(&mut writer, 2, &td, &crate::storage::Unbounded).expect("write tabPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(
                r#"<hp:case hp:required-namespace="http://www.hancom.co.kr/hwpml/2016/HwpUnitChar"><hh:tabItem pos="1644" type="LEFT" leader="NONE" unit="HWPUNIT"/></hp:case>"#
            ),
            "case 는 절반값(1644) + unit=HWPUNIT: {xml}"
        );
        assert!(
            xml.contains(
                r#"<hp:default><hh:tabItem pos="3288" type="LEFT" leader="NONE"/></hp:default>"#
            ),
            "default 는 저장값(3288), unit 없음: {xml}"
        );
        assert_eq!(
            xml.matches("<hp:switch>").count(),
            1,
            "탭 1개당 switch 1개: {xml}"
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
        write_para_pr(&mut writer, 1, &ps, &crate::storage::Unbounded).expect("write paraPr");
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
        // attr1 보존 비트에서 역매핑돼야 한다.
        let mut ps = ParaShape::default();
        ps.break_latin_word = Some("HYPHENATION".to_string());
        ps.attr1 = (2 << 20) // vertical = CENTER
            | (1 << 16) // widowOrphan = 1
            | (1 << 19); // pageBreakBefore = 1
        ps.attr1 &= !(1 << 7); // breakNonLatinWord = BREAK_WORD (bit7=0)
        ps.attr2 = 1 << 5; // eAsianNum = 1

        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 1, &ps, &crate::storage::Unbounded).expect("write paraPr");
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
        assert!(
            xml.contains(r#"<hh:autoSpacing eAsianEng="0" eAsianNum="1""#),
            "autoSpacing은 attr2 4/5에서 역매핑해야 함: {xml}"
        );
    }

    #[test]
    fn write_para_pr_does_not_treat_auto_spacing_num_as_widow_orphan() {
        let mut ps = ParaShape::default();
        ps.attr2 = 1 << 5; // autoSpaceKrNum = 1

        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 1, &ps, &crate::storage::Unbounded).expect("write paraPr");
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"widowOrphan="0""#),
            "attr2 bit 5는 widowOrphan으로 해석되면 안 됨: {xml}"
        );
        assert!(
            xml.contains(r#"<hh:autoSpacing eAsianEng="0" eAsianNum="1""#),
            "attr2 bit 5는 eAsianNum으로만 방출해야 함: {xml}"
        );
    }

    #[test]
    fn write_para_pr_default_vertical_is_baseline() {
        // 기본 ParaShape(attr1=0) 의 vertical 은 BASELINE(bits 20..21 = 0). 파싱된
        // 문단은 항상 bit7 을 명시 설정하므로 round-trip 은 정확하다(이 테스트는
        // 신규 생성 기본 ParaShape 의 vertical 만 검증).
        let ps = ParaShape::default();
        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 0, &ps, &crate::storage::Unbounded).expect("write paraPr");
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
        write_char_pr(&mut writer, 0, &cs, &crate::storage::Unbounded).expect("write charPr");
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
    fn write_char_pr_preserves_opaque_black_shade() {
        let cs = CharShape {
            shade_color: 0x0000_0000,
            ..Default::default()
        };
        let xml = write_single_char_pr(&cs);

        assert!(
            xml.contains(r##"shadeColor="#000000""##),
            "실제 검정 음영은 none 으로 소거하면 안 된다: {xml}"
        );
        assert!(
            !xml.contains(r#"shadeColor="none""#),
            "0x00000000은 음영 없음 sentinel 이 아니다: {xml}"
        );
    }

    /// [#2695] charPr 자식 하나를 담은 최소 hh:head 를 파싱해 CharShape 를 얻는다.
    fn parse_single_char_pr(child: &str) -> CharShape {
        let xml = format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<hh:head xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head">
  <hh:refList>
    <hh:charProperties itemCnt="1">
      <hh:charPr id="0" height="1000" textColor="#000000" shadeColor="none">
        <hh:fontRef hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/>
        {child}
      </hh:charPr>
    </hh:charProperties>
  </hh:refList>
</hh:head>"##
        );
        let (doc_info, _) =
            crate::parser::hwpx::header::parse_hwpx_header(&xml).expect("parse hh:head");
        doc_info.char_shapes[0].clone()
    }

    /// [#2695] CharShape 를 charPr 로 직렬화한 XML 문자열.
    fn write_single_char_pr(cs: &CharShape) -> String {
        let mut writer = Writer::new(Vec::new());
        write_char_pr(&mut writer, 0, cs, &crate::storage::Unbounded).expect("write charPr");
        String::from_utf8(writer.into_inner()).unwrap()
    }

    #[test]
    fn char_pr_outline_type_roundtrips_all_eight_values() {
        // [#2695] 외곽선은 HWP5 attr bits 8-10(3비트) = 8종이다. 종전엔 파서가
        // NONE/SOLID/DASH/DOT 4종만 알고 나머지를 `_ => 0` 으로, 직렬화가
        // `_ => "NONE"` 으로 떨궈 4~7 지정 시 외곽선이 통째로 사라졌다.
        for (name, expected) in [
            ("NONE", 0u8),
            ("SOLID", 1),
            ("DASH", 2),
            ("DOT", 3),
            ("DASH_DOT", 4),
            ("DASH_DOT_DOT", 5),
            ("LONG_DASH", 6),
            ("CIRCLE", 7),
        ] {
            let cs = parse_single_char_pr(&format!(r#"<hh:outline type="{name}"/>"#));
            assert_eq!(
                cs.outline_type, expected,
                "outline type={name} 은 IR {expected} 로 파싱돼야 함"
            );

            let xml = write_single_char_pr(&cs);
            assert!(
                xml.contains(&format!(r#"<hh:outline type="{name}"/>"#)),
                "outline_type={expected} 은 type=\"{name}\" 으로 방출돼야 함: {xml}"
            );
        }
    }

    #[test]
    fn char_pr_shadow_drop_survives_roundtrip_with_offsets() {
        // [#2695] 비연속(DROP)은 offsetX/offsetY 로 본체와 떨어뜨려 그리는 종류다.
        // 종전엔 파서가 DROP|CONTINUOUS 를 모두 1 로 합치고 직렬화가 0 이외를
        // 무조건 CONTINUOUS 로 써서 DROP 이 방출 불가능했고, 보존된 오프셋도
        // 해석 주체를 잃었다.
        let cs = parse_single_char_pr(
            r##"<hh:shadow type="DROP" color="#C0C0C0" offsetX="10" offsetY="-7"/>"##,
        );
        assert_eq!(cs.shadow_type, 1, "DROP 은 IR 1(비연속)");
        assert_eq!(cs.shadow_offset_x, 10);
        assert_eq!(cs.shadow_offset_y, -7);

        let xml = write_single_char_pr(&cs);
        assert!(
            xml.contains(r#"<hh:shadow type="DROP""#),
            "shadow_type=1 은 type=\"DROP\" 으로 방출돼야 함: {xml}"
        );
        assert!(
            xml.contains(r#"offsetX="10" offsetY="-7""#),
            "오프셋도 함께 보존돼야 함: {xml}"
        );
    }

    #[test]
    fn char_pr_shadow_continuous_is_ir_two_not_one() {
        // [#2695] 연속은 HWP5 attr bits 11-12 = 2 다. 종전 파서는 1 로 읽어
        // HWPX 를 경유한 .hwp 재저장 시 비트값이 2→1 로 손상됐다.
        let cs = parse_single_char_pr(r##"<hh:shadow type="CONTINUOUS" color="#808080"/>"##);
        assert_eq!(cs.shadow_type, 2, "CONTINUOUS 는 IR 2(연속)");

        let xml = write_single_char_pr(&cs);
        assert!(
            xml.contains(r#"<hh:shadow type="CONTINUOUS""#),
            "shadow_type=2 는 type=\"CONTINUOUS\" 로 방출돼야 함: {xml}"
        );
    }

    #[test]
    fn write_para_pr_emits_condense_fontlineheight_snaptogrid_from_attr1() {
        // condense/fontLineHeight/snapToGrid 는 상수가 아니라 attr1 비트에서 나와야 한다.
        let mut ps = ParaShape::default();
        // condense=20 (bits9..15), fontLineHeight=1 (bit22), snapToGrid=0 (bit8 clear)
        ps.attr1 = (20 << 9) | (1 << 22);
        let mut writer = Writer::new(Vec::new());
        write_para_pr(&mut writer, 0, &ps, &crate::storage::Unbounded).expect("write paraPr");
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
        write_para_pr(&mut writer, 0, &ps, &crate::storage::Unbounded).expect("write paraPr");
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
        write_diagonal(&mut w, &none, &crate::storage::Unbounded).expect("write");
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
        write_diagonal(&mut w, &solid, &crate::storage::Unbounded).expect("write");
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
        write_numbering(&mut writer, 0, &n, &crate::storage::Unbounded).unwrap();
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
        write_numbering(&mut writer, 0, &n, &crate::storage::Unbounded).unwrap();
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert_eq!(xml.matches("<hh:paraHead").count(), 10);
        assert!(xml.starts_with(r#"<hh:numbering id="1" start="0">"#));
    }

    #[test]
    fn write_numbering_skeleton_emits_level_format_string_as_text() {
        // [#3838] 번호 형식 문자열("^1." 등)은 `hh:paraHead` 의 **텍스트 내용**이다.
        // 폴백이 자기닫힘 태그로 속성만 내보내면 형식이 사라져 문단 번호가 아예
        // 렌더되지 않는다 — IR diff 는 0 이라 `--verify` 게이트가 못 잡는다.
        //
        // 한컴 원본 실측(HWPX 40건): paraHead 232개 중 172개가 내용을 갖는다.
        let mut n = Numbering::default();
        n.level_formats[0] = "^1.".to_string();
        n.level_formats[1] = "(^2)".to_string();

        let mut writer = Writer::new(Vec::new());
        write_numbering(&mut writer, 0, &n, &crate::storage::Unbounded).unwrap();
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(">^1.</hh:paraHead>"),
            "level 1 형식 문자열이 텍스트 내용으로 나가야 한다: {xml}"
        );
        assert!(
            xml.contains(">(^2)</hh:paraHead>"),
            "level 2 형식 문자열이 텍스트 내용으로 나가야 한다: {xml}"
        );
        // 형식이 빈 수준은 종전대로 자기닫힘 — 빈 내용을 만들지 않는다.
        assert!(
            xml.contains("checkable=\"0\"/>"),
            "형식이 없는 수준은 자기닫힘을 유지해야 한다: {xml}"
        );
    }

    #[test]
    fn write_numbering_skeleton_uses_number_format_not_hardcoded_digit() {
        // [#2947] HWP5 경유(raw_para_heads 없음) 시 h.number_format(예: ROMAN_CAPITAL=2)
        // 을 무시하고 "DIGIT" 로 고정 방출하면 로마자/한글 문단 번호가 유실된다.
        let mut n = Numbering::default();
        n.heads[0].number_format = 2; // ROMAN_CAPITAL

        let mut writer = Writer::new(Vec::new());
        write_numbering(&mut writer, 0, &n, &crate::storage::Unbounded).unwrap();
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"numFormat="ROMAN_CAPITAL""#),
            "level 1 paraHead 는 number_format=2 를 ROMAN_CAPITAL 로 방출해야 한다: {xml}"
        );
    }

    #[test]
    fn write_numbering_skeleton_preserves_text_distance_and_char_shape_id() {
        // HWP5 경로(raw_para_heads 없음)에서 NumberingHead.text_distance /
        // char_shape_id 는 DocInfo 파서가 실제 값으로 채워 넣는 필드인데,
        // 폴백 스켈레톤이 textOffset="50" / charPrIDRef=u32::MAX 로 하드코딩해
        // 유실시키면 안 된다 (write_bullet 의 대응 필드 처리와 대칭이어야 함).
        let mut n = Numbering::default();
        n.heads[0].text_distance = 130;
        n.heads[0].char_shape_id = 7;

        let mut writer = Writer::new(Vec::new());
        write_numbering(&mut writer, 0, &n, &crate::storage::Unbounded).unwrap();
        let xml = String::from_utf8(writer.into_inner()).unwrap();

        assert!(
            xml.contains(r#"textOffset="130""#),
            "text_distance 유실: {xml}"
        );
        assert!(
            xml.contains(r#"charPrIDRef="7""#),
            "char_shape_id 유실: {xml}"
        );
    }

    #[test]
    fn tab_leader_str_emits_double_and_triple_line_types() {
        // fill_type 9/10/11 이 "NONE" 으로 유실되지 않고 OWPML LineType3 스펙 리터럴
        // (Core XML schema.xml 335~349행)로 방출돼야 한다 (#2857). 파서는 옛 이름
        // (THIN_THICK/THICK_THIN/TRIM)도 하위호환으로 계속 받는다.
        assert_eq!(tab_leader_str(9), "SLIM_THICK");
        assert_eq!(tab_leader_str(10), "THICK_SLIM");
        assert_eq!(tab_leader_str(11), "SLIM_THICK_SLIM");
        assert_eq!(tab_leader_str(8), "DOUBLE_SLIM");
    }
}
