//! 그리기 개체 (도형) 직렬화 — Rectangle / Line / Container 뼈대.
//!
//! Stage 5 (#182): 대표 도형 3종(Rectangle, Line, Container)의 `<hp:rect>`, `<hp:line>`,
//! `<hp:container>` 요소 뼈대를 구현한다. 완전한 속성 커버리지는 별도 이슈로 이월.
//!
//! 속성·자식 순서는 한컴 OWPML 공식 (hancom-io/hwpx-owpml-model, Apache 2.0) 기준.
//!
//! ## 범위 한정
//!
//! - Stage 5 에서는 **도형 뼈대 출력** 기능만 제공 (section.rs dispatcher 연결은 #186).
//! - Arc / Polygon / Curve / Group 등은 향후 이슈에서 확장.
//! - DrawingObjAttr (선/채우기 세부 속성) 은 최소 기본값 출력.

#![allow(dead_code)]

use std::io::Write;

use quick_xml::Writer;

use crate::model::paragraph::{LineSeg, Paragraph};
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, LineShape, RectangleShape, TextBox, TextFlow, TextWrap,
    VertAlign, VertRelTo,
};

use super::utils::{empty_tag, end_tag, start_tag, start_tag_attrs};
use super::SerializeError;

// =====================================================================
// <hp:rect>
// =====================================================================

/// `<hp:rect>` 직렬화 진입점. Rectangle IR → XML.
pub fn write_rect<W: Write>(
    w: &mut Writer<W>,
    rect: &RectangleShape,
) -> Result<(), SerializeError> {
    let c = &rect.common;
    // 속성 (부모 AbstractShapeObjectType + 자신):
    // id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle,
    // href, groupLevel, instid, ratio
    let id_str = c.instance_id.to_string();
    let z_order = c.z_order.to_string();
    let tw = text_wrap_str(c.text_wrap);
    let tf = text_flow_str(c.text_flow);

    start_tag_attrs(
        w,
        "hp:rect",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", "NONE"),
            ("textWrap", tw),
            ("textFlow", tf),
            ("lock", "0"),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", "0"),
            ("instid", &id_str),
            ("ratio", "0"),
        ],
    )?;

    // 기본 자식: sz, pos, outMargin
    write_sz(w, c)?;
    write_pos(w, c)?;
    write_out_margin(w, c)?;

    // drawText: 글상자 내부 문단
    if let Some(ref tb) = rect.drawing.text_box {
        if !tb.paragraphs.is_empty() {
            write_draw_text(w, tb)?;
        }
    }

    end_tag(w, "hp:rect")?;
    Ok(())
}

// =====================================================================
// <hp:line>
// =====================================================================

/// `<hp:line>` 직렬화 진입점. LineShape IR → XML.
pub fn write_line<W: Write>(w: &mut Writer<W>, line: &LineShape) -> Result<(), SerializeError> {
    let c = &line.common;
    let id_str = c.instance_id.to_string();
    let z_order = c.z_order.to_string();
    let tw = text_wrap_str(c.text_wrap);
    let tf = text_flow_str(c.text_flow);
    let sx = line.start.x.to_string();
    let sy = line.start.y.to_string();
    let ex = line.end.x.to_string();
    let ey = line.end.y.to_string();
    let srb = bool01(line.started_right_or_bottom);

    start_tag_attrs(
        w,
        "hp:line",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", "NONE"),
            ("textWrap", tw),
            ("textFlow", tf),
            ("lock", "0"),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", "0"),
            ("instid", &id_str),
            ("startX", &sx),
            ("startY", &sy),
            ("endX", &ex),
            ("endY", &ey),
            ("isReverseHV", srb),
        ],
    )?;

    write_sz(w, c)?;
    write_pos(w, c)?;
    write_out_margin(w, c)?;

    end_tag(w, "hp:line")?;
    Ok(())
}

// =====================================================================
// <hp:container> — 묶음 개체 (GroupShape). Stage 5 뼈대만.
// =====================================================================

/// `<hp:container>` 뼈대 — 내부 자식 도형 루프는 dispatcher에서 처리.
pub fn write_container_open<W: Write>(
    w: &mut Writer<W>,
    common: &CommonObjAttr,
) -> Result<(), SerializeError> {
    let id_str = common.instance_id.to_string();
    let z_order = common.z_order.to_string();
    let tw = text_wrap_str(common.text_wrap);
    let tf = text_flow_str(common.text_flow);

    start_tag_attrs(
        w,
        "hp:container",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", "NONE"),
            ("textWrap", tw),
            ("textFlow", tf),
            ("lock", "0"),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", "0"),
            ("instid", &id_str),
        ],
    )?;

    write_sz(w, common)?;
    write_pos(w, common)?;
    write_out_margin(w, common)?;

    Ok(())
}

pub fn write_container_close<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    end_tag(w, "hp:container")
}

// =====================================================================
// <hp:drawText> — 글상자 내부 텍스트
// =====================================================================

/// `<hp:drawText>` 직렬화 — TextBox의 paragraphs를 subList로 출력.
pub fn write_draw_text<W: Write>(w: &mut Writer<W>, tb: &TextBox) -> Result<(), SerializeError> {
    let ml = tb.margin_left.to_string();
    let mr = tb.margin_right.to_string();
    let mt = tb.margin_top.to_string();
    let mb = tb.margin_bottom.to_string();
    let mw = tb.max_width.to_string();

    start_tag_attrs(w, "hp:drawText", &[("lastWidth", &mw)])?;

    empty_tag(
        w,
        "hp:textMargin",
        &[("left", &ml), ("right", &mr), ("top", &mt), ("bottom", &mb)],
    )?;

    start_tag_attrs(
        w,
        "hp:subList",
        &[
            ("id", ""),
            ("textDirection", "HORIZONTAL"),
            ("lineWrap", "BREAK"),
            ("vertAlign", "TOP"),
            ("linkListIDRef", "0"),
            ("linkListNextIDRef", "0"),
            ("textWidth", "0"),
            ("textHeight", "0"),
            ("hasTextRef", "0"),
            ("hasNumRef", "0"),
        ],
    )?;

    for (idx, p) in tb.paragraphs.iter().enumerate() {
        write_draw_text_paragraph(w, p, idx)?;
    }

    end_tag(w, "hp:subList")?;
    end_tag(w, "hp:drawText")?;
    Ok(())
}

fn write_draw_text_paragraph<W: Write>(
    w: &mut Writer<W>,
    p: &Paragraph,
    idx: usize,
) -> Result<(), SerializeError> {
    let id = idx.to_string();
    let ps_id = p.para_shape_id.to_string();
    let st_id = p.style_id.to_string();

    start_tag_attrs(
        w,
        "hp:p",
        &[
            ("id", &id),
            ("paraPrIDRef", &ps_id),
            ("styleIDRef", &st_id),
            ("pageBreak", "0"),
            ("columnBreak", "0"),
            ("merged", "0"),
        ],
    )?;

    let cs = p.char_shapes.first().map(|r| r.char_shape_id).unwrap_or(0);
    let cs_str = cs.to_string();
    start_tag_attrs(w, "hp:run", &[("charPrIDRef", &cs_str)])?;

    // simple text output — XML escape
    start_tag(w, "hp:t")?;
    w.write_event(quick_xml::events::Event::Text(
        quick_xml::events::BytesText::new(&super::utils::xml_escape(&p.text)),
    ))
    .map_err(|e| SerializeError::XmlError(format!("drawText text: {e}")))?;
    end_tag(w, "hp:t")?;

    end_tag(w, "hp:run")?;

    // minimal lineseg
    start_tag(w, "hp:linesegarray")?;
    let line_flags = LineSeg::TAG_SINGLE_SEGMENT_LINE.to_string();
    empty_tag(
        w,
        "hp:lineseg",
        &[
            ("textpos", "0"),
            ("vertpos", "0"),
            ("vertsize", "1000"),
            ("textheight", "1000"),
            ("baseline", "850"),
            ("spacing", "600"),
            ("horzpos", "0"),
            ("horzsize", "42520"),
            ("flags", line_flags.as_str()),
        ],
    )?;
    end_tag(w, "hp:linesegarray")?;

    end_tag(w, "hp:p")?;
    Ok(())
}

// =====================================================================
// 공통 자식 요소 (sz / pos / outMargin)
// =====================================================================

fn write_sz<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let width = c.width.to_string();
    let height = c.height.to_string();
    empty_tag(
        w,
        "hp:sz",
        &[
            ("width", &width),
            ("widthRelTo", "ABSOLUTE"),
            ("height", &height),
            ("heightRelTo", "ABSOLUTE"),
            ("protect", "0"),
        ],
    )
}

fn write_pos<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let treat = bool01(c.treat_as_char);
    let vert_offset = c.vertical_offset.to_string();
    let horz_offset = c.horizontal_offset.to_string();
    empty_tag(
        w,
        "hp:pos",
        &[
            ("treatAsChar", treat),
            ("affectLSpacing", "0"),
            ("flowWithText", "1"),
            ("allowOverlap", "0"),
            ("holdAnchorAndSO", "0"),
            ("vertRelTo", vert_rel_to_str(c.vert_rel_to)),
            ("horzRelTo", horz_rel_to_str(c.horz_rel_to)),
            ("vertAlign", vert_align_str(c.vert_align)),
            ("horzAlign", horz_align_str(c.horz_align)),
            ("vertOffset", &vert_offset),
            ("horzOffset", &horz_offset),
        ],
    )
}

fn write_out_margin<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    let l = c.margin.left.to_string();
    let r = c.margin.right.to_string();
    let t = c.margin.top.to_string();
    let b = c.margin.bottom.to_string();
    empty_tag(
        w,
        "hp:outMargin",
        &[("left", &l), ("right", &r), ("top", &t), ("bottom", &b)],
    )
}

fn bool01(b: bool) -> &'static str {
    if b {
        "1"
    } else {
        "0"
    }
}

fn text_wrap_str(w: TextWrap) -> &'static str {
    use TextWrap::*;
    match w {
        Square => "SQUARE",
        Tight => "TIGHT",
        Through => "THROUGH",
        TopAndBottom => "TOP_AND_BOTTOM",
        BehindText => "BEHIND_TEXT",
        InFrontOfText => "IN_FRONT_OF_TEXT",
    }
}

fn text_flow_str(f: TextFlow) -> &'static str {
    match f {
        TextFlow::BothSides => "BOTH_SIDES",
        TextFlow::LeftOnly => "LEFT_ONLY",
        TextFlow::RightOnly => "RIGHT_ONLY",
        TextFlow::LargestOnly => "LARGEST_ONLY",
    }
}

fn vert_rel_to_str(v: VertRelTo) -> &'static str {
    use VertRelTo::*;
    match v {
        Paper => "PAPER",
        Page => "PAGE",
        Para => "PARA",
    }
}

fn horz_rel_to_str(h: HorzRelTo) -> &'static str {
    use HorzRelTo::*;
    match h {
        Paper => "PAPER",
        Page => "PAGE",
        Column => "COLUMN",
        Para => "PARA",
    }
}

fn vert_align_str(v: VertAlign) -> &'static str {
    use VertAlign::*;
    match v {
        Top => "TOP",
        Center => "CENTER",
        Bottom => "BOTTOM",
        Inside => "INSIDE",
        Outside => "OUTSIDE",
    }
}

fn horz_align_str(h: HorzAlign) -> &'static str {
    use HorzAlign::*;
    match h {
        Left => "LEFT",
        Center => "CENTER",
        Right => "RIGHT",
        Inside => "INSIDE",
        Outside => "OUTSIDE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::shape::{LineShape, RectangleShape};
    use crate::model::Point;

    fn serialize_rect(rect: &RectangleShape) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_rect(&mut w, rect).expect("write_rect");
        String::from_utf8(w.into_inner()).unwrap()
    }

    fn serialize_line(line: &LineShape) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_line(&mut w, line).expect("write_line");
        String::from_utf8(w.into_inner()).unwrap()
    }

    #[test]
    fn rect_emits_root_tag() {
        let mut rect = RectangleShape::default();
        rect.common.width = 1000;
        rect.common.height = 500;
        let xml = serialize_rect(&rect);
        assert!(xml.contains("<hp:rect "));
        assert!(xml.contains("</hp:rect>"));
    }

    #[test]
    fn rect_has_canonical_attrs() {
        let rect = RectangleShape::default();
        let xml = serialize_rect(&rect);
        assert!(xml.contains(r#"id=""#));
        assert!(xml.contains(r#"zOrder=""#));
        assert!(xml.contains(r#"textWrap=""#));
        assert!(xml.contains(r#"textFlow="BOTH_SIDES""#));
    }

    #[test]
    fn line_emits_start_end_attrs() {
        let mut line = LineShape::default();
        line.start = Point { x: 100, y: 200 };
        line.end = Point { x: 300, y: 400 };
        let xml = serialize_line(&line);
        assert!(xml.contains(r#"startX="100""#));
        assert!(xml.contains(r#"startY="200""#));
        assert!(xml.contains(r#"endX="300""#));
        assert!(xml.contains(r#"endY="400""#));
    }

    #[test]
    fn rect_has_sz_pos_out_margin() {
        let rect = RectangleShape::default();
        let xml = serialize_rect(&rect);
        assert!(xml.contains("<hp:sz "));
        assert!(xml.contains("<hp:pos "));
        assert!(xml.contains("<hp:outMargin "));
    }
}
