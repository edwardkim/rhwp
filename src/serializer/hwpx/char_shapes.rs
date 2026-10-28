//! HWPX 글자모양 직렬화 — `hh:charPr` 테이블과 문단 run 경계.
//!
//! #3500: HWP5 `PARA_CHAR_SHAPE` 는 같은 `char_shape_id` 라도 `start_pos` 가
//! 다르면 별도 entry 다. HWPX 는 run 시작으로만 그 경계를 표현하므로, 연속
//! 동일 id 도 접지 않고 `<hp:run>` 을 나눈다 (#3739 와 같은 축).

use std::io::Write;

use quick_xml::Writer;

use crate::model::paragraph::{CharShapeRef, Paragraph};
use crate::model::style::CharShape;
use crate::model::ColorRef;

use super::char_shape_tables::{
    line_shape_str, outline_type_str, shadow_type_str, strike_shape_str, sym_mark_str,
    underline_type_str, LANG_ATTRS, SHADE_NONE,
};
use super::utils::{empty_tag_with_storage as empty_tag, xml_value};
use super::SerializeError;

/// `(start_pos, char_shape_id)` — 직렬화가 접지 않는 원본 시퀀스.
pub fn plan_run_boundaries(refs: &[CharShapeRef]) -> Vec<(u32, u32)> {
    refs.iter()
        .map(|cs| (cs.start_pos, cs.char_shape_id))
        .collect()
}

/// 문단 IR 에서 run 경계를 그대로 옮긴다. 연속 동일 id 를 합치지 않는다.
pub fn plan_run_boundaries_of(para: &Paragraph) -> Vec<(u32, u32)> {
    plan_run_boundaries(&para.char_shapes)
}

/// 동일 id 축약. 렌더 등가는 맞지만 #3500 IR 비교는 이 결과를 쓰지 않는다.
pub fn collapse_same_id(refs: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for &(start, id) in refs {
        if out.last().is_some_and(|(_, last_id)| *last_id == id) {
            continue;
        }
        out.push((start, id));
    }
    out
}

/// 동일 id 축약으로 사라지는 entry 수. 0 이면 접을 경계가 없다.
pub fn same_id_extra_count(refs: &[(u32, u32)]) -> usize {
    refs.len().saturating_sub(collapse_same_id(refs).len())
}

/// 방출된 `<hp:run charPrIDRef>` 를 등장 순으로 모은다.
pub fn char_pr_id_refs_in_xml(xml: &str) -> Vec<u32> {
    let needle = "charPrIDRef=\"";
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(at) = rest.find(needle) {
        let tail = &rest[at + needle.len()..];
        if let Some(end) = tail.find('"') {
            if let Ok(id) = tail[..end].parse::<u32>() {
                out.push(id);
            }
            rest = &tail[end + 1..];
        } else {
            break;
        }
    }
    out
}

/// 연속 동일 id 경계가 XML run 개수로 남았는지. 접히면 false.
pub fn xml_preserves_same_id_runs(xml: &str, refs: &[(u32, u32)]) -> bool {
    let ids: Vec<u32> = refs.iter().map(|(_, id)| *id).collect();
    char_pr_id_refs_in_xml(xml) == ids
}

fn color_hex<S: crate::storage::Storage>(
    color: ColorRef,
    storage: &S,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    if color == SHADE_NONE {
        return xml_value(storage, "none");
    }
    let a = ((color >> 24) & 0xFF) as u8;
    let r = (color & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let b = ((color >> 16) & 0xFF) as u8;
    if a == 0 {
        xml_value(storage, format_args!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        xml_value(
            storage,
            format_args!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b),
        )
    }
}

fn bool01(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}

fn write_lang_attrs<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    vals: &[i32; 7],
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let s0 = xml_value(storage, vals[0])?;
    let s1 = xml_value(storage, vals[1])?;
    let s2 = xml_value(storage, vals[2])?;
    let s3 = xml_value(storage, vals[3])?;
    let s4 = xml_value(storage, vals[4])?;
    let s5 = xml_value(storage, vals[5])?;
    let s6 = xml_value(storage, vals[6])?;
    empty_tag(
        w,
        name,
        &[
            (LANG_ATTRS[0], &s0),
            (LANG_ATTRS[1], &s1),
            (LANG_ATTRS[2], &s2),
            (LANG_ATTRS[3], &s3),
            (LANG_ATTRS[4], &s4),
            (LANG_ATTRS[5], &s5),
            (LANG_ATTRS[6], &s6),
        ],
        storage,
    )
}

/// `<hh:charPr>` 한 칸. `header.rs` 가 테이블 루프로 호출한다.
pub fn write_char_pr<W: Write>(
    w: &mut Writer<W>,
    id: u32,
    cs: &CharShape,
) -> Result<(), SerializeError> {
    write_char_pr_with_storage(w, id, cs, &crate::storage::Unbounded)
}

pub(crate) fn write_char_pr_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    id: u32,
    cs: &CharShape,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    use super::utils::{
        end_tag_with_storage as end_tag, start_tag_attrs_with_storage as start_tag_attrs,
    };

    let shade = color_hex(cs.shade_color, storage)?;
    start_tag_attrs(
        w,
        "hh:charPr",
        &[
            ("id", &xml_value(storage, id)?),
            ("height", &xml_value(storage, cs.base_size)?),
            ("textColor", &color_hex(cs.text_color, storage)?),
            ("shadeColor", &shade),
            ("useFontSpace", bool01(cs.use_font_space)),
            ("useKerning", bool01(cs.kerning)),
            ("symMark", sym_mark_str(cs.emphasis_dot)),
            ("borderFillIDRef", &xml_value(storage, cs.border_fill_id)?),
        ],
        storage,
    )?;
    write_lang_attrs(w, "hh:fontRef", &cs.font_ids.map(|v| v as i32), storage)?;
    write_lang_attrs(w, "hh:ratio", &cs.ratios.map(|v| v as i32), storage)?;
    write_lang_attrs(w, "hh:spacing", &cs.spacings.map(|v| v as i32), storage)?;
    write_lang_attrs(w, "hh:relSz", &cs.relative_sizes.map(|v| v as i32), storage)?;
    write_lang_attrs(w, "hh:offset", &cs.char_offsets.map(|v| v as i32), storage)?;
    if cs.italic {
        empty_tag(w, "hh:italic", &[], storage)?;
    }
    if cs.bold {
        empty_tag(w, "hh:bold", &[], storage)?;
    }
    empty_tag(
        w,
        "hh:underline",
        &[
            ("type", underline_type_str(cs.underline_type)),
            ("shape", line_shape_str(cs.underline_shape)),
            ("color", &color_hex(cs.underline_color, storage)?),
        ],
        storage,
    )?;
    empty_tag(
        w,
        "hh:strikeout",
        &[
            ("shape", strike_shape_str(cs.strikethrough, cs.strike_shape)),
            ("color", &color_hex(cs.strike_color, storage)?),
        ],
        storage,
    )?;
    empty_tag(
        w,
        "hh:outline",
        &[("type", outline_type_str(cs.outline_type))],
        storage,
    )?;
    empty_tag(
        w,
        "hh:shadow",
        &[
            ("type", shadow_type_str(cs.shadow_type)),
            ("color", &color_hex(cs.shadow_color, storage)?),
            ("offsetX", &xml_value(storage, cs.shadow_offset_x)?),
            ("offsetY", &xml_value(storage, cs.shadow_offset_y)?),
        ],
        storage,
    )?;
    if cs.emboss {
        empty_tag(w, "hh:emboss", &[], storage)?;
    }
    if cs.engrave {
        empty_tag(w, "hh:engrave", &[], storage)?;
    }
    if cs.superscript {
        empty_tag(w, "hh:supscript", &[], storage)?;
    }
    if cs.subscript {
        empty_tag(w, "hh:subscript", &[], storage)?;
    }
    end_tag(w, "hh:charPr", storage)?;
    Ok(())
}

/// 시험·픽스처용 `hh:charPr` XML 문자열.
pub fn char_pr_xml(id: u32, cs: &CharShape) -> Result<String, SerializeError> {
    let mut writer = Writer::new(Vec::new());
    write_char_pr(&mut writer, id, cs)?;
    String::from_utf8(writer.into_inner()).map_err(|e| SerializeError::XmlError(e.to_string()))
}
