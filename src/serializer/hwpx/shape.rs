//! 그리기 개체 (도형) 직렬화 — Rectangle / Line / Container.
//!
//! Stage 5 (#182)에서 뼈대를 만들고, #1379 3단계에서 rect 하위 요소 보존으로 확장했다.
//!
//! 속성·자식 순서는 한컴 원본 XML 실측(tbox-v-flow-01) 기준:
//! `offset → orgSz → curSz → flip → rotationInfo → renderingInfo → lineShape →
//! fillBrush → shadow → drawText → hc:pt0~pt3 → sz → pos → outMargin → shapeComment`
//!
//! ## 범위 한정
//!
//! - rect 우선 (#1379 3단계). Arc / Polygon / Curve 등 타 도형 확대는 측정 후 별도 분류.
//! - 글상자(drawText) 문단은 본문·셀과 동일한 `render_paragraph_parts` 공유 경로로
//!   직렬화한다 (컨트롤 슬롯 + run 분할 + lineseg IR 보존).

#![allow(dead_code)]

use std::io::Write;

use quick_xml::Writer;

use crate::model::shape::{
    CommonObjAttr, DrawingObjAttr, HorzAlign, HorzRelTo, LineShape, ObjectNumberingType,
    OleDrawingAspect, OleShape, RectangleShape, ShapeComponentAttr, TextBox, TextFlow, TextWrap,
    VertAlign, VertRelTo,
};
use crate::model::style::{Fill, FillType, ImageFillMode, ShapeBorderLine, SolidFill};
use crate::model::ColorRef;

use super::context::SerializeContext;
use super::section::{render_hp_p_open, render_paragraph_parts};
use super::table::write_caption;
use super::utils::{empty_tag, end_tag, start_tag, start_tag_attrs, text};
use super::SerializeError;

// =====================================================================
// <hp:rect>
// =====================================================================

/// `<hp:rect>` 직렬화 진입점. Rectangle IR → XML.
pub fn write_rect<W: Write>(
    w: &mut Writer<W>,
    rect: &RectangleShape,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    let c = &rect.common;
    let sa = &rect.drawing.shape_attr;
    // 속성 (부모 AbstractShapeObjectType + 자신):
    // id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle,
    // href, groupLevel, instid, ratio
    let id_str = c.instance_id.to_string();
    let z_order = c.z_order.to_string();
    let tw = text_wrap_str(c.text_wrap);
    let tf = text_flow_str(c.text_flow);
    let group_level = sa.group_level.to_string();
    // 파서는 instid 를 drawing.inst_id 에 보존한다 (0이면 instance_id 대체).
    let instid = if rect.drawing.inst_id != 0 {
        rect.drawing.inst_id
    } else {
        c.instance_id
    }
    .to_string();
    let ratio = rect.round_rate.to_string();

    start_tag_attrs(
        w,
        "hp:rect",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", numbering_type_str(c.numbering_type)),
            ("textWrap", tw),
            ("textFlow", tf),
            ("lock", "0"),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", &group_level),
            ("instid", &instid),
            ("ratio", &ratio),
        ],
    )?;

    // 자식 순서 — 한컴 원본 실측 (tbox-v-flow-01) 기준.
    write_offset(w, sa)?;
    write_org_sz(w, sa)?;
    write_cur_sz(w, sa)?;
    write_flip(w, sa)?;
    write_rotation_info(w, sa)?;
    write_rendering_info(w, sa)?;
    write_line_shape(w, &rect.drawing.border_line)?;
    write_fill_brush(w, &rect.drawing.fill, ctx)?;
    write_shadow(w, &rect.drawing)?;

    // drawText: 글상자 내부 문단
    if let Some(ref tb) = rect.drawing.text_box {
        if !tb.paragraphs.is_empty() {
            write_draw_text(w, tb, ctx)?;
        }
    }

    // 꼭짓점 4점 (hc: 접두어)
    write_rect_pts(w, &rect.x_coords, &rect.y_coords)?;

    write_sz(w, c)?;
    write_pos(w, c)?;
    write_out_margin(w, c)?;
    // 캡션 (#1403) — OWPML AbstractShapeObjectType 순서: outMargin 과 shapeComment 사이
    if let Some(cap) = &rect.drawing.caption {
        write_caption(w, cap, ctx)?;
    }
    write_shape_comment(w, c)?;

    end_tag(w, "hp:rect")?;
    Ok(())
}

// =====================================================================
// <hp:line>
// =====================================================================

/// `<hp:line>` 직렬화 진입점. LineShape IR → XML.
pub fn write_line<W: Write>(
    w: &mut Writer<W>,
    line: &LineShape,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
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
            ("numberingType", numbering_type_str(c.numbering_type)),
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
    // 캡션 (#1403) — OWPML AbstractShapeObjectType 순서: outMargin 뒤
    if let Some(cap) = &line.drawing.caption {
        write_caption(w, cap, ctx)?;
    }
    // [#1588] 도형 설명 — caption 뒤 (write_rect/container 와 동형). 누락 시 선 도형
    // shapeComment("선입니다." 등)가 저장 시 드롭됐다.
    write_shape_comment(w, c)?;

    end_tag(w, "hp:line")?;
    Ok(())
}

// =====================================================================
// <hp:container> — 묶음 개체 (GroupShape). Stage 5 뼈대만.
// =====================================================================

/// `<hp:container>` 뼈대 — 내부 자식 도형 루프는 dispatcher에서 처리.
///
/// 한컴 실측(hwpx-h-01) 컨테이너 직계 순서:
/// offset → orgSz → curSz → flip → rotationInfo → renderingInfo → [자식 도형들]
/// → sz → pos → outMargin → shapeComment.
/// 그룹 자신의 shape_attr(orgSz/curSz/offset/renderingInfo)이 누락되면 렌더러가
/// 그룹 스케일·자식 기준 좌표를 계산하지 못해 자식이 그룹 원점에 고유 크기로 붕괴한다.
pub fn write_container_open<W: Write>(
    w: &mut Writer<W>,
    common: &CommonObjAttr,
    sa: &ShapeComponentAttr,
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
            ("numberingType", numbering_type_str(common.numbering_type)),
            ("textWrap", tw),
            ("textFlow", tf),
            ("lock", "0"),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", "0"),
            ("instid", &id_str),
        ],
    )?;

    // 그룹 자신의 좌표계 — 자식 도형 앞에 방출 (write_rect 패턴과 동일 순서).
    write_offset(w, sa)?;
    write_org_sz(w, sa)?;
    write_cur_sz(w, sa)?;
    write_flip(w, sa)?;
    write_rotation_info(w, sa)?;
    write_rendering_info(w, sa)?;

    Ok(())
}

/// `<hp:container>` 닫기 — 자식 도형 뒤에 sz → pos → outMargin → caption(#1403) →
/// shapeComment(#1392) 순으로 방출 (한컴 실측 hwpx-h-01/aift 순서).
pub fn write_container_close<W: Write>(
    w: &mut Writer<W>,
    caption: Option<&crate::model::shape::Caption>,
    common: &CommonObjAttr,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    write_sz(w, common)?;
    write_pos(w, common)?;
    write_out_margin(w, common)?;
    if let Some(cap) = caption {
        write_caption(w, cap, ctx)?;
    }
    // 설명 (#1392) — caption 직후
    write_shape_comment(w, common)?;
    end_tag(w, "hp:container")
}

// =====================================================================
// <hp:ole> — OLE 개체 (차트 등 포함)
//
// 종전 직렬화는 OLE 를 legacy 공용 경로(sz/pos/outMargin 만)로 내보내 binaryItemIDRef·
// extent·shape_attr 를 빠뜨렸다. 그 결과 라운드트립에서 OLE 데이터 참조가 소실되어
// 렌더가 placeholder 로 강등됐다(143E: RawSvg→Placeholder). picture 패턴으로 복원한다.
// =====================================================================
pub(crate) fn write_ole<W: Write>(
    w: &mut Writer<W>,
    ole: &OleShape,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    let c = &ole.common;
    let id_str = c.instance_id.to_string();
    let z_order = c.z_order.to_string();
    let tw = text_wrap_str(c.text_wrap);
    let tf = text_flow_str(c.text_flow);
    // owned 으로 변환해 ctx 불변 borrow 를 즉시 해제(이후 write_caption 의 &mut 사용).
    let bidref = ctx
        .resolve_bin_id(ole.bin_data_id as u16)
        .unwrap_or("")
        .to_string();
    let draw_aspect = match ole.drawing_aspect {
        OleDrawingAspect::Icon => "ICON",
        OleDrawingAspect::Thumbnail => "THUMBNAIL",
        OleDrawingAspect::DocPrint => "DOCPRINT",
        OleDrawingAspect::Content => "CONTENT",
    };

    start_tag_attrs(
        w,
        "hp:ole",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", numbering_type_str(c.numbering_type)),
            ("textWrap", tw),
            ("textFlow", tf),
            ("lock", "0"),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", "0"),
            ("instid", &id_str),
            ("objectType", "UNKNOWN"),
            ("binaryItemIDRef", &bidref),
            ("hasMoniker", "0"),
            ("drawAspect", draw_aspect),
            ("eqBaseLine", "0"),
        ],
    )?;

    // shape_attr 블록 (offset/orgSz/curSz/flip/rotationInfo/renderingInfo)
    write_shape_component_block(w, &ole.drawing.shape_attr)?;
    // 개체 영역
    let ex = ole.extent_x.to_string();
    let ey = ole.extent_y.to_string();
    empty_tag(w, "hc:extent", &[("x", &ex), ("y", &ey)])?;
    write_line_shape(w, &ole.drawing.border_line)?;
    write_sz(w, c)?;
    write_pos(w, c)?;
    write_out_margin(w, c)?;
    if let Some(cap) = &ole.caption {
        write_caption(w, cap, ctx)?;
    }
    write_shape_comment(w, c)?;

    end_tag(w, "hp:ole")?;
    Ok(())
}

// =====================================================================
// <hp:drawText> — 글상자 내부 텍스트
// =====================================================================

/// `<hp:drawText>` 직렬화 — TextBox의 paragraphs를 subList로 출력 (#1379 3단계).
///
/// - 문단은 본문·셀과 동일한 `render_paragraph_parts` 공유 경로 (컨트롤 슬롯 +
///   run 분할 + lineseg IR 보존/fallback)
/// - `textDirection` VERTICAL/VERTICALALL 구분은 `TextBox.vertical_all` 로 보존
/// - `textMargin` 은 subList **뒤** (한컴 원본 순서, footnote-tbox-01 실측),
///   네 여백 모두 0이면 원본 부재로 간주하여 미방출
pub fn write_draw_text<W: Write>(
    w: &mut Writer<W>,
    tb: &TextBox,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    let lw = tb.max_width.to_string();
    start_tag_attrs(
        w,
        "hp:drawText",
        &[("lastWidth", &lw), ("name", ""), ("editable", "0")],
    )?;

    // textDirection: list_attr bit 0~2 (1=세로쓰기), vertical_all 로 ALL 구분.
    let text_direction = if tb.list_attr & 0x07 == 1 {
        if tb.vertical_all {
            "VERTICALALL"
        } else {
            "VERTICAL"
        }
    } else {
        "HORIZONTAL"
    };
    let vert_align = match tb.vertical_align {
        crate::model::table::VerticalAlign::Center => "CENTER",
        crate::model::table::VerticalAlign::Bottom => "BOTTOM",
        crate::model::table::VerticalAlign::Top => "TOP",
    };

    start_tag_attrs(
        w,
        "hp:subList",
        &[
            ("id", ""),
            ("textDirection", text_direction),
            ("lineWrap", "BREAK"),
            ("vertAlign", vert_align),
            ("linkListIDRef", "0"),
            ("linkListNextIDRef", "0"),
            ("textWidth", "0"),
            ("textHeight", "0"),
            ("hasTextRef", "0"),
            ("hasNumRef", "0"),
        ],
    )?;

    // sub_list_depth: 글상자 경로 한정 colPr 인라인 방출 스코프 (#1379 3단계).
    ctx.sub_list_depth += 1;
    let mut vert_cursor: u32 = 0;
    for para in tb.paragraphs.iter() {
        ctx.para_shape_ids.reference(para.para_shape_id);
        ctx.style_ids.reference(para.style_id as u16);

        let (runs, linesegs, advance) = render_paragraph_parts(para, vert_cursor, ctx);
        vert_cursor = advance;
        let mut p_xml = render_hp_p_open(para, ctx.next_para_id());
        p_xml.push_str(&runs);
        p_xml.push_str(&linesegs);
        p_xml.push_str("</hp:p>");
        w.get_mut()
            .write_all(p_xml.as_bytes())
            .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    }
    ctx.sub_list_depth -= 1;

    end_tag(w, "hp:subList")?;

    if tb.margin_left != 0 || tb.margin_right != 0 || tb.margin_top != 0 || tb.margin_bottom != 0 {
        let ml = tb.margin_left.to_string();
        let mr = tb.margin_right.to_string();
        let mt = tb.margin_top.to_string();
        let mb = tb.margin_bottom.to_string();
        empty_tag(
            w,
            "hp:textMargin",
            &[("left", &ml), ("right", &mr), ("top", &mt), ("bottom", &mb)],
        )?;
    }

    end_tag(w, "hp:drawText")?;
    Ok(())
}

// =====================================================================
// ShapeComponentAttr 하위 요소 (offset / orgSz / curSz / flip / rotationInfo / renderingInfo)
// =====================================================================

/// AbstractShapeComponentType 의 좌표계 블록을 한컴 순서로 방출한다:
/// offset → orgSz → curSz → flip → rotationInfo → renderingInfo.
/// 누락 시 회전/뒤집힘·그룹 내 좌표가 소실되어 렌더가 어긋난다(ellipse/arc/polygon/curve 공용).
pub(crate) fn write_shape_component_block<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    write_offset(w, sa)?;
    write_org_sz(w, sa)?;
    write_cur_sz(w, sa)?;
    write_flip(w, sa)?;
    write_rotation_info(w, sa)?;
    write_rendering_info(w, sa)?;
    Ok(())
}

fn write_offset<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    let x = sa.offset_x.to_string();
    let y = sa.offset_y.to_string();
    empty_tag(w, "hp:offset", &[("x", &x), ("y", &y)])
}

fn write_org_sz<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    let width = sa.original_width.to_string();
    let height = sa.original_height.to_string();
    empty_tag(w, "hp:orgSz", &[("width", &width), ("height", &height)])
}

fn write_cur_sz<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    let width = sa.current_width.to_string();
    let height = sa.current_height.to_string();
    empty_tag(w, "hp:curSz", &[("width", &width), ("height", &height)])
}

fn write_flip<W: Write>(w: &mut Writer<W>, sa: &ShapeComponentAttr) -> Result<(), SerializeError> {
    empty_tag(
        w,
        "hp:flip",
        &[
            ("horizontal", bool01(sa.horz_flip)),
            ("vertical", bool01(sa.vert_flip)),
        ],
    )
}

fn write_rotation_info<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    let angle = sa.rotation_angle.to_string();
    let cx = sa.rotation_center.x.to_string();
    let cy = sa.rotation_center.y.to_string();
    empty_tag(
        w,
        "hp:rotationInfo",
        &[
            ("angle", &angle),
            ("centerX", &cx),
            ("centerY", &cy),
            ("rotateimage", bool01(sa.rotate_image)),
        ],
    )
}

/// `<hp:renderingInfo>` — `raw_rendering` (cnt u16 LE + trans 6×f64 + cnt×(sca, rot))
/// 를 디코드해 행렬을 재구성한다 (`parse_rendering_info` 의 역). raw 비정합/빈 경우
/// identity 3행렬 fallback. pic 자식도 공유 (그룹 내 자식 transMatrix 보존).
pub(crate) fn write_rendering_info<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let (trans, pairs) =
        decode_raw_rendering(&sa.raw_rendering).unwrap_or((IDENTITY, vec![(IDENTITY, IDENTITY)]));

    start_tag(w, "hp:renderingInfo")?;
    write_matrix(w, "hc:transMatrix", &trans)?;
    for (sca, rot) in &pairs {
        write_matrix(w, "hc:scaMatrix", sca)?;
        write_matrix(w, "hc:rotMatrix", rot)?;
    }
    end_tag(w, "hp:renderingInfo")
}

type RenderMatrices = ([f64; 6], Vec<([f64; 6], [f64; 6])>);

fn decode_raw_rendering(raw: &[u8]) -> Option<RenderMatrices> {
    if raw.len() < 2 + 48 {
        return None;
    }
    let cnt = u16::from_le_bytes([raw[0], raw[1]]) as usize;
    if raw.len() < 2 + 48 + cnt * 96 {
        return None;
    }
    let read6 = |off: usize| -> [f64; 6] {
        let mut m = [0.0f64; 6];
        for (i, v) in m.iter_mut().enumerate() {
            let p = off + i * 8;
            *v = f64::from_le_bytes(raw[p..p + 8].try_into().unwrap());
        }
        m
    };
    let trans = read6(2);
    let mut pairs = Vec::with_capacity(cnt);
    for k in 0..cnt {
        let base = 2 + 48 + k * 96;
        pairs.push((read6(base), read6(base + 48)));
    }
    Some((trans, pairs))
}

/// 행렬 값 포맷: 정수는 정수 문자열, 비정수는 f32 정밀도 (파서가 f32 로 적재 —
/// 원본 "1.579917" 보존).
fn fmt_matrix_val(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v as f32)
    }
}

fn write_matrix<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    m: &[f64; 6],
) -> Result<(), SerializeError> {
    let vals: Vec<String> = m.iter().map(|v| fmt_matrix_val(*v)).collect();
    empty_tag(
        w,
        name,
        &[
            ("e1", &vals[0]),
            ("e2", &vals[1]),
            ("e3", &vals[2]),
            ("e4", &vals[3]),
            ("e5", &vals[4]),
            ("e6", &vals[5]),
        ],
    )
}

// =====================================================================
// lineShape / fillBrush / shadow
// =====================================================================

/// `<hp:lineShape>` — `parse_line_shape_attr` 의 역매핑.
/// headStyle/tailStyle/alpha 는 파서 미적재 → "NORMAL"/"0" 고정 방출.
pub(crate) fn write_line_shape<W: Write>(
    w: &mut Writer<W>,
    bl: &ShapeBorderLine,
) -> Result<(), SerializeError> {
    let color = color_to_hex(bl.color);
    let width = bl.width.to_string();
    // style 은 attr 하위 6비트. 정본 코드(0=NONE/1=SOLID/2=DASH…)는 표 borderFill 의
    // border_line_type_from_code 및 HWP5 doc_info 와 동일. 종전에는 0 이 _ => SOLID 로
    // 떨어져 "선 없음" 도형 외곽선이 라운드트립에서 사각형 박스로 살아났다(#1531).
    let style = match bl.attr & 0x3F {
        0 => "NONE",
        1 => "SOLID",
        2 => "DASH",
        3 => "DOT",
        4 => "DASH_DOT",
        5 => "DASH_DOT_DOT",
        6 => "LONG_DASH",
        7 => "CIRCLE",
        8 => "DOUBLE_SLIM",
        9 => "SLIM_THICK",
        10 => "THICK_SLIM",
        11 => "SLIM_THICK_SLIM",
        _ => "SOLID",
    };
    let end_cap = match (bl.attr >> 6) & 0x0F {
        1 => "FLAT",
        2 => "SQUARE",
        _ => "ROUND",
    };
    let headfill = bool01(bl.attr & 0x8000_0000 != 0);
    let tailfill = bool01(bl.attr & 0x4000_0000 != 0);
    let head_sz = arrow_size_str((bl.attr >> 22) & 0x0F);
    let tail_sz = arrow_size_str((bl.attr >> 26) & 0x0F);
    let outline = match bl.outline_style {
        1 => "OUTER",
        2 => "INNER",
        _ => "NORMAL",
    };
    empty_tag(
        w,
        "hp:lineShape",
        &[
            ("color", &color),
            ("width", &width),
            ("style", style),
            ("endCap", end_cap),
            ("headStyle", "NORMAL"),
            ("tailStyle", "NORMAL"),
            ("headfill", headfill),
            ("tailfill", tailfill),
            ("headSz", head_sz),
            ("tailSz", tail_sz),
            ("outlineStyle", outline),
            ("alpha", "0"),
        ],
    )
}

/// `parse_line_shape_attr::arrow_size` 의 역매핑 (0~8).
fn arrow_size_str(v: u32) -> &'static str {
    match v {
        1 => "SMALL_MEDIUM",
        2 => "SMALL_BIG",
        3 => "MEDIUM_SMALL",
        4 => "MEDIUM_MEDIUM",
        5 => "MEDIUM_BIG",
        6 => "BIG_SMALL",
        7 => "BIG_MEDIUM",
        8 => "BIG_BIG",
        _ => "SMALL_SMALL",
    }
}

/// `<hc:fillBrush><hc:winBrush .../></hc:fillBrush>` 를 방출하는 단일 소유 함수.
/// FillType::Solid 경로와 보존된 빈 채우기(FillType::None + solid 존재) 경로가
/// 동일한 직렬화를 쓰도록 한 곳에 모은다(균일 규칙).
fn write_win_brush<W: Write>(
    w: &mut Writer<W>,
    solid: &SolidFill,
    alpha: u8,
) -> Result<(), SerializeError> {
    let face = color_to_hex(solid.background_color);
    let hatch = color_to_hex(solid.pattern_color);
    let alpha = fill_alpha_str(alpha);
    start_tag(w, "hc:fillBrush")?;
    if solid.pattern_type >= 1 {
        empty_tag(
            w,
            "hc:winBrush",
            &[
                ("faceColor", &face),
                ("hatchColor", &hatch),
                ("hatchStyle", hatch_style_str(solid.pattern_type)),
                ("alpha", &alpha),
            ],
        )?;
    } else {
        empty_tag(
            w,
            "hc:winBrush",
            &[
                ("faceColor", &face),
                ("hatchColor", &hatch),
                ("alpha", &alpha),
            ],
        )?;
    }
    end_tag(w, "hc:fillBrush")
}

/// `<hc:fillBrush>` — `parse_shape_fill_brush` 의 역매핑.
/// `FillType::None` 은 원본에 fillBrush 부재로 간주하되, 파서가 보존한 solid 가
/// 있으면(빈 winBrush) 복원한다.
///
/// 도형(rect/line)뿐 아니라 `header.rs` 의 borderFill 도 같은 fillBrush 구조를
/// 쓰므로 `pub(crate)` 로 공유한다.
pub(crate) fn write_fill_brush<W: Write>(
    w: &mut Writer<W>,
    fill: &Fill,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    match fill.fill_type {
        // FillType::None 이지만 solid 데이터가 보존돼 있으면(원본 winBrush 가
        // faceColor="none"+무늬없음 으로 빈 채우기였던 경우) winBrush 를 그대로
        // 복원한다. solid 가 없으면 원본에 fillBrush 가 없었던 것이므로 미방출.
        FillType::None => match &fill.solid {
            Some(solid) => write_win_brush(w, solid, fill.alpha),
            None => Ok(()),
        },
        FillType::Solid => {
            let solid = fill.solid.unwrap_or_default();
            write_win_brush(w, &solid, fill.alpha)
        }
        FillType::Gradient => {
            let grad = fill.gradient.clone().unwrap_or_default();
            let gtype = match grad.gradient_type {
                1 => "LINEAR".to_string(),
                2 => "RADIAL".to_string(),
                3 => "CONICAL".to_string(),
                4 => "SQUARE".to_string(),
                other => other.to_string(),
            };
            let angle = grad.angle.to_string();
            let cx = grad.center_x.to_string();
            let cy = grad.center_y.to_string();
            let step = grad.blur.to_string();
            let step_center = grad.step_center.to_string();
            let alpha = fill_alpha_str(fill.alpha);
            start_tag(w, "hc:fillBrush")?;
            start_tag_attrs(
                w,
                "hc:gradation",
                &[
                    ("type", &gtype),
                    ("angle", &angle),
                    ("centerX", &cx),
                    ("centerY", &cy),
                    ("step", &step),
                    ("stepCenter", &step_center),
                    ("alpha", &alpha),
                ],
            )?;
            for c in &grad.colors {
                let v = color_to_hex(*c);
                empty_tag(w, "hc:color", &[("value", &v)])?;
            }
            end_tag(w, "hc:gradation")?;
            end_tag(w, "hc:fillBrush")
        }
        FillType::Image => {
            let img = fill.image.clone().unwrap_or_default();
            let mode = match img.fill_mode {
                ImageFillMode::FitToSize => "FIT",
                ImageFillMode::Total => "TOTAL",
                ImageFillMode::Center => "CENTER",
                _ => "TILE",
            };
            start_tag(w, "hc:fillBrush")?;
            // bin_data_id 가 ctx 에 등록돼 있으면 <hc:img> 참조를 방출(셀/쪽 배경 이미지
            // 보존). 미등록(예: body shape 의 fill 파서가 bin_data_id 미캡처)이면 종전대로
            // 빈 imgBrush — 잘못된 image0 참조로 3-way 단언을 깨지 않는다.
            match ctx.resolve_bin_id(img.bin_data_id) {
                Some(manifest_id) => {
                    start_tag_attrs(w, "hc:imgBrush", &[("mode", mode)])?;
                    let bright = img.brightness.to_string();
                    let contrast = img.contrast.to_string();
                    let effect = match img.effect {
                        1 => "GRAY_SCALE",
                        2 => "BLACK_WHITE",
                        _ => "REAL_PIC",
                    };
                    empty_tag(
                        w,
                        "hc:img",
                        &[
                            ("binaryItemIDRef", manifest_id),
                            ("bright", &bright),
                            ("contrast", &contrast),
                            ("effect", effect),
                            ("alpha", "0"),
                        ],
                    )?;
                    end_tag(w, "hc:imgBrush")?;
                }
                None => {
                    empty_tag(w, "hc:imgBrush", &[("mode", mode)])?;
                }
            }
            end_tag(w, "hc:fillBrush")
        }
    }
}

/// winBrush/gradation alpha — 파서가 `f.clamp(0,1)*255` 로 적재하므로 0~1 분수로 방출.
fn fill_alpha_str(alpha: u8) -> String {
    if alpha == 0 {
        "0".to_string()
    } else {
        format!("{}", alpha as f64 / 255.0)
    }
}

/// `hatch_style` (1~6) → OWPML hatchStyle. `parse_hatch_style` 의 역매핑.
fn hatch_style_str(pattern_type: i32) -> &'static str {
    match pattern_type {
        1 => "HORIZONTAL",
        2 => "VERTICAL",
        3 => "BACK_SLASH",
        4 => "SLASH",
        5 => "CROSS",
        _ => "CROSS_DIAGONAL",
    }
}

/// `<hp:shadow>` — `parse_shape_shadow_attr` 의 역매핑.
/// 전 필드 0 이면 원본에 shadow 부재로 간주하여 미방출.
/// alpha 는 정수 방출 (파서의 `>1.0` 경로와 정합 — 0/1 경계값만 비가역).
pub(crate) fn write_shadow<W: Write>(
    w: &mut Writer<W>,
    d: &DrawingObjAttr,
) -> Result<(), SerializeError> {
    if d.shadow_type == 0
        && d.shadow_color == 0
        && d.shadow_offset_x == 0
        && d.shadow_offset_y == 0
        && d.shadow_alpha == 0
    {
        return Ok(());
    }
    let ty = match d.shadow_type {
        1 => "LEFT_TOP",
        2 => "RIGHT_TOP",
        3 => "LEFT_BOTTOM",
        4 => "RIGHT_BOTTOM",
        5 => "CENTER",
        _ => "NONE",
    };
    let color = color_to_hex(d.shadow_color);
    let ox = d.shadow_offset_x.to_string();
    let oy = d.shadow_offset_y.to_string();
    let alpha = d.shadow_alpha.to_string();
    empty_tag(
        w,
        "hp:shadow",
        &[
            ("type", ty),
            ("color", &color),
            ("offsetX", &ox),
            ("offsetY", &oy),
            ("alpha", &alpha),
        ],
    )
}

// =====================================================================
// 꼭짓점 / shapeComment
// =====================================================================

fn write_rect_pts<W: Write>(
    w: &mut Writer<W>,
    x: &[i32; 4],
    y: &[i32; 4],
) -> Result<(), SerializeError> {
    for (i, name) in ["hc:pt0", "hc:pt1", "hc:pt2", "hc:pt3"].iter().enumerate() {
        let px = x[i].to_string();
        let py = y[i].to_string();
        empty_tag(w, name, &[("x", &px), ("y", &py)])?;
    }
    Ok(())
}

/// `<hp:shapeComment>` 직렬화 — 도형(rect)·그림(#1392)·수식(#1392)·묶음(#1392) 공유.
///
/// 빈 description 은 미방출 (한컴 원본도 설명 부재 시 요소 없음).
pub(super) fn write_shape_comment<W: Write>(
    w: &mut Writer<W>,
    c: &CommonObjAttr,
) -> Result<(), SerializeError> {
    if c.description.is_empty() {
        return Ok(());
    }
    start_tag(w, "hp:shapeComment")?;
    text(w, &c.description)?;
    end_tag(w, "hp:shapeComment")
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
    let hold = bool01(c.prevent_page_break != 0); // [#1594] IR 보존
    empty_tag(
        w,
        "hp:pos",
        &[
            ("treatAsChar", treat),
            ("affectLSpacing", "0"),
            ("flowWithText", bool01(c.flow_with_text)),
            ("allowOverlap", bool01(c.allow_overlap)),
            ("holdAnchorAndSO", hold),
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

/// ColorRef (0xAABBGGRR) → "#RRGGBB" / "#AARRGGBB". 0xFFFFFFFF → "none".
/// `parse_color_str` 의 역매핑 (header.rs `color_hex` 와 동일 규칙).
pub(crate) fn color_to_hex(c: ColorRef) -> String {
    if c == 0xFFFFFFFF {
        return "none".to_string();
    }
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

pub(crate) fn numbering_type_str(n: ObjectNumberingType) -> &'static str {
    match n {
        ObjectNumberingType::Picture => "PICTURE",
        ObjectNumberingType::Table => "TABLE",
        ObjectNumberingType::Equation => "EQUATION",
        ObjectNumberingType::None => "NONE",
    }
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
    use crate::model::paragraph::Paragraph;
    use crate::model::shape::{LineShape, RectangleShape};
    use crate::model::Point;

    fn serialize_rect(rect: &RectangleShape) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        let mut ctx = SerializeContext::collect_from_document(&Default::default());
        write_rect(&mut w, rect, &mut ctx).expect("write_rect");
        String::from_utf8(w.into_inner()).unwrap()
    }

    fn serialize_line(line: &LineShape) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        let mut ctx = SerializeContext::collect_from_document(&Default::default());
        write_line(&mut w, line, &mut ctx).expect("write_line");
        String::from_utf8(w.into_inner()).unwrap()
    }

    fn line_shape_style(attr: u32) -> String {
        use crate::model::style::ShapeBorderLine;
        let bl = ShapeBorderLine {
            attr,
            ..Default::default()
        };
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_line_shape(&mut w, &bl).expect("write_line_shape");
        let xml = String::from_utf8(w.into_inner()).unwrap();
        let i = xml.find("style=\"").expect("style attr") + 7;
        xml[i..].split('"').next().unwrap().to_string()
    }

    /// #1531: 선 없음(style code 0) 도형 외곽선이 라운드트립에서 SOLID(사각형 박스)로
    /// 살아나면 안 된다. endCap(bit 6~9)이 함께 설정돼도 NONE 이 보존돼야 한다.
    #[test]
    fn task1531_line_shape_none_preserved() {
        assert_eq!(line_shape_style(0), "NONE"); // 정본 코드 0 = NONE
        assert_eq!(line_shape_style(1), "SOLID"); // 1 = SOLID
        assert_eq!(line_shape_style(2), "DASH"); // 2 = DASH (회귀 방지)
        let none_with_flat_end_cap = 1 << 6;
        assert_eq!(line_shape_style(none_with_flat_end_cap), "NONE");
    }

    /// #1588: 선 도형 설명(shapeComment)이 저장 시 방출돼야 한다.
    /// write_rect/container 는 호출하나 write_line 만 누락 → 드롭(RED).
    #[test]
    fn task1588_line_shape_comment_emitted() {
        let mut line = LineShape::default();
        line.common.description = "선입니다.".to_string();
        let xml = serialize_line(&line);
        assert!(
            xml.contains("<hp:shapeComment>선입니다.</hp:shapeComment>"),
            "선 도형 shapeComment 방출돼야 한다 (현재 드롭): {xml}"
        );
    }

    /// #1588: 설명 없는 선 도형은 shapeComment 미방출 (빈 태그 금지).
    #[test]
    fn task1588_line_shape_no_comment_when_empty() {
        let line = LineShape::default();
        let xml = serialize_line(&line);
        assert!(!xml.contains("<hp:shapeComment"), "빈 설명 미방출: {xml}");
    }

    fn cs(start_pos: u32, char_shape_id: u32) -> crate::model::paragraph::CharShapeRef {
        crate::model::paragraph::CharShapeRef {
            start_pos,
            char_shape_id,
        }
    }

    fn rect_with_text_paragraph(p: Paragraph) -> RectangleShape {
        let mut tb = TextBox::default();
        tb.paragraphs.push(p);
        let mut rect = RectangleShape::default();
        rect.drawing.text_box = Some(tb);
        rect
    }

    #[test]
    fn task1378_drawtext_multi_run_split() {
        // 글상자 문단 다중 char_shapes → 경계 기준 다중 run 분할 (#1378 3단계).
        let mut p = Paragraph::default();
        p.text = "abcd".to_string();
        p.char_offsets = vec![0, 1, 2, 3];
        p.char_count = 5;
        p.char_shapes = vec![cs(0, 3), cs(2, 4)];
        let xml = serialize_rect(&rect_with_text_paragraph(p));
        assert!(
            xml.contains(
                r#"<hp:run charPrIDRef="3"><hp:t>ab</hp:t></hp:run><hp:run charPrIDRef="4"><hp:t>cd</hp:t></hp:run>"#
            ),
            "글상자 문단이 경계에서 2 run 으로 분할되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1378_drawtext_tab_and_linebreak_rendered() {
        // 탭/lineBreak 가 raw 제어문자 대신 인라인 요소로 방출된다.
        let mut p = Paragraph::default();
        p.text = "a\tb\nc".to_string();
        p.tab_extended = vec![[2000, 0, 0x0100, 0, 0, 0, 0]];
        let xml = serialize_rect(&rect_with_text_paragraph(p));
        assert!(
            xml.contains(
                r#"<hp:t>a<hp:tab width="2000" leader="0" type="1"/>b<hp:lineBreak/>c</hp:t>"#
            ),
            "글상자 텍스트는 hp:tab/hp:lineBreak 인라인 요소로 방출되어야 함: {}",
            xml
        );
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

    // ================= #1379 3단계 =================

    #[test]
    fn task1379_drawtext_paragraph_emits_picture_control() {
        // 글상자 문단의 Picture 컨트롤이 hp:pic 으로 방출되어야 함 (#1379 3단계).
        let mut p = Paragraph::default();
        p.char_count = 9; // 슬롯 1개(8 유닛) + 종단 1
        {
            let mut pic = crate::model::image::Picture::default();
            pic.image_attr.bin_data_id = 1;
            p.controls
                .push(crate::model::control::Control::Picture(Box::new(pic)));
        }
        let rect = rect_with_text_paragraph(p);

        let mut doc = crate::model::document::Document::default();
        doc.bin_data_content
            .push(crate::model::bin_data::BinDataContent {
                id: 1,
                data: vec![0u8; 4],
                extension: "png".to_string(),
            });
        let mut ctx = SerializeContext::collect_from_document(&doc);
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_rect(&mut w, &rect, &mut ctx).expect("write_rect");
        let xml = String::from_utf8(w.into_inner()).unwrap();
        assert!(
            xml.contains("<hp:pic "),
            "글상자 문단의 Picture 가 hp:pic 으로 방출되어야 함: {}",
            xml
        );
    }

    #[test]
    fn task1379_drawtext_vertical_direction_preserved() {
        // textDirection VERTICAL / VERTICALALL / HORIZONTAL 구분 보존.
        let mut p = Paragraph::default();
        p.text = "a".to_string();
        let mut rect = rect_with_text_paragraph(p);
        {
            let tb = rect.drawing.text_box.as_mut().unwrap();
            tb.list_attr = 1; // 세로쓰기
            tb.vertical_all = true;
            tb.vertical_align = crate::model::table::VerticalAlign::Center;
        }
        let xml = serialize_rect(&rect);
        assert!(
            xml.contains(r#"textDirection="VERTICALALL""#),
            "VERTICALALL 보존: {}",
            xml
        );
        assert!(xml.contains(r#"vertAlign="CENTER""#), "vertAlign: {}", xml);

        rect.drawing.text_box.as_mut().unwrap().vertical_all = false;
        let xml = serialize_rect(&rect);
        assert!(
            xml.contains(r#"textDirection="VERTICAL""#) && !xml.contains("VERTICALALL"),
            "VERTICAL (ALL 아님) 보존: {}",
            xml
        );
    }

    #[test]
    fn task1379_rect_emits_pts_and_element_order() {
        // hc:pt0~pt3 방출 + 자식 순서 (offset→…→drawText→pt→sz→pos→outMargin→comment).
        let mut p = Paragraph::default();
        p.text = "x".to_string();
        let mut rect = rect_with_text_paragraph(p);
        rect.x_coords = [0, 13514, 13514, 0];
        rect.y_coords = [0, 0, 14898, 14898];
        rect.common.description = "사각형입니다.".to_string();
        rect.drawing.shape_attr.original_width = 13514;
        rect.drawing.shape_attr.current_width = 21351;
        let xml = serialize_rect(&rect);

        assert!(
            xml.contains(r#"<hc:pt1 x="13514" y="0"/>"#),
            "pt1 좌표: {}",
            xml
        );
        let order = [
            "<hp:offset ",
            "<hp:orgSz ",
            "<hp:curSz ",
            "<hp:flip ",
            "<hp:rotationInfo ",
            "<hp:renderingInfo>",
            "<hp:lineShape ",
            "<hp:drawText ",
            "<hc:pt0 ",
            "<hp:sz ",
            "<hp:pos ",
            "<hp:outMargin ",
            "<hp:shapeComment>",
        ];
        let mut last = 0usize;
        for tag in order {
            let pos = xml
                .find(tag)
                .unwrap_or_else(|| panic!("{} 누락: {}", tag, xml));
            assert!(
                pos > last,
                "{} 순서 오류 (pos={}, last={}): {}",
                tag,
                pos,
                last,
                xml
            );
            last = pos;
        }
        assert!(
            xml.contains("<hp:shapeComment>사각형입니다.</hp:shapeComment>"),
            "shapeComment 보존: {}",
            xml
        );
    }

    #[test]
    fn task1379_rect_line_fill_shadow_attrs() {
        // lineShape/fillBrush/shadow 속성 역매핑.
        let mut rect = RectangleShape::default();
        rect.drawing.border_line.color = 0; // #000000
        rect.drawing.border_line.width = 33;
        // SOLID(1) + endCap FLAT(1<<6) + headfill/tailfill + headSz/tailSz MEDIUM_MEDIUM(4)
        rect.drawing.border_line.attr =
            1 | (1 << 6) | 0x8000_0000 | 0x4000_0000 | (4 << 22) | (4 << 26);
        rect.drawing.fill.fill_type = FillType::Solid;
        rect.drawing.fill.solid = Some(crate::model::style::SolidFill {
            background_color: 0x00FFFFFF, // #FFFFFF
            pattern_color: 0,
            pattern_type: -1,
        });
        rect.drawing.shadow_type = 0;
        rect.drawing.shadow_color = crate::parser::hwpx::utils::parse_color_str("#B2B2B2");
        let xml = serialize_rect(&rect);

        assert!(
            xml.contains(
                r##"<hp:lineShape color="#000000" width="33" style="SOLID" endCap="FLAT" headStyle="NORMAL" tailStyle="NORMAL" headfill="1" tailfill="1" headSz="MEDIUM_MEDIUM" tailSz="MEDIUM_MEDIUM" outlineStyle="NORMAL" alpha="0"/>"##
            ),
            "lineShape 역매핑: {}",
            xml
        );
        assert!(
            xml.contains(
                r##"<hc:fillBrush><hc:winBrush faceColor="#FFFFFF" hatchColor="#000000" alpha="0"/></hc:fillBrush>"##
            ),
            "fillBrush winBrush: {}",
            xml
        );
        assert!(
            xml.contains(
                r##"<hp:shadow type="NONE" color="#B2B2B2" offsetX="0" offsetY="0" alpha="0"/>"##
            ),
            "shadow NONE + 색상 보존: {}",
            xml
        );
    }

    #[test]
    fn task1379_tbox_v_flow_01_roundtrip_preserves_textbox() {
        // 이슈 대표 샘플 — roundtrip 후 글상자 rect 의 구조 보존.
        fn find_rect(doc: &crate::model::document::Document) -> Option<&RectangleShape> {
            doc.sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .flat_map(|p| &p.controls)
                .find_map(|c| match c {
                    crate::model::control::Control::Shape(s) => match s.as_ref() {
                        crate::model::shape::ShapeObject::Rectangle(r) => Some(r),
                        _ => None,
                    },
                    _ => None,
                })
        }
        let bytes = std::fs::read("samples/hwpx/tbox-v-flow-01.hwpx").expect("샘플 읽기");
        let doc1 = crate::parser::hwpx::parse_hwpx(&bytes).expect("파싱");
        let r1 = find_rect(&doc1).expect("원본 rect");
        let tb1 = r1.drawing.text_box.as_ref().expect("원본 글상자");
        assert!(tb1.vertical_all, "원본 textDirection=VERTICALALL");
        let n_para = tb1.paragraphs.len();
        assert!(n_para > 1, "글상자 다문단 샘플");

        let out = crate::serializer::hwpx::serialize_hwpx(&doc1).expect("직렬화");
        let doc2 = crate::parser::hwpx::parse_hwpx(&out).expect("재파싱");
        let r2 = find_rect(&doc2).expect("roundtrip rect");
        let tb2 = r2.drawing.text_box.as_ref().expect("roundtrip 글상자");

        assert_eq!(tb2.paragraphs.len(), n_para, "글상자 문단 수 보존");
        assert!(tb2.vertical_all, "VERTICALALL 보존");
        assert_eq!(
            tb2.paragraphs[0].text, tb1.paragraphs[0].text,
            "첫 문단 텍스트 보존"
        );
        assert_eq!(
            r2.common.numbering_type,
            crate::model::shape::ObjectNumberingType::Picture,
            "numberingType=PICTURE 보존"
        );
        assert_eq!(r2.x_coords, r1.x_coords, "pt 꼭짓점 x 보존");
        assert_eq!(
            (r2.common.flow_with_text, r2.common.allow_overlap),
            (r1.common.flow_with_text, r1.common.allow_overlap),
            "pos flowWithText/allowOverlap 보존"
        );
        // 비정수 scaMatrix 값 f32 정밀도 보존 (원본 "1.579917")
        let cursor = std::io::Cursor::new(&out);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive
            .by_name("Contents/section0.xml")
            .expect("section0.xml");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read section0");
        assert!(xml.contains("1.579917"), "scaMatrix 비정수 값 보존");
    }

    // ---------- #1403: 도형 캡션 직렬화 ----------

    fn caption_with_text(text: &str) -> crate::model::shape::Caption {
        let mut para = Paragraph::default();
        para.text = text.to_string();
        let mut caption = crate::model::shape::Caption::default();
        caption.width = 8504;
        caption.spacing = 850;
        caption.max_width = 47624;
        caption.paragraphs.push(para);
        caption
    }

    #[test]
    fn task1403_rect_caption_is_serialized() {
        // OWPML AbstractShapeObjectType 순서: outMargin → caption → shapeComment.
        let mut rect = RectangleShape::default();
        rect.common.description = "설명".to_string();
        rect.drawing.caption = Some(caption_with_text("사각형 캡션"));
        let xml = serialize_rect(&rect);
        assert!(
            xml.contains("<hp:t>사각형 캡션</hp:t>"),
            "캡션 subList 문단 텍스트가 방출되어야 함: {}",
            xml
        );
        let om = xml.find("<hp:outMargin").unwrap();
        let cp = xml.find("<hp:caption").unwrap();
        let sc = xml.find("<hp:shapeComment").unwrap();
        assert!(
            om < cp && cp < sc,
            "caption 은 outMargin 과 shapeComment 사이"
        );
    }

    #[test]
    fn task1403_line_caption_is_serialized() {
        let mut line = LineShape::default();
        line.drawing.caption = Some(caption_with_text("선 캡션"));
        let xml = serialize_line(&line);
        assert!(
            xml.contains("<hp:t>선 캡션</hp:t>"),
            "캡션 subList 문단 텍스트가 방출되어야 함: {}",
            xml
        );
        let om = xml.find("<hp:outMargin").unwrap();
        let cp = xml.find("<hp:caption").unwrap();
        assert!(om < cp, "caption 은 outMargin 뒤");
    }

    #[test]
    fn task1403_shape_without_caption_emits_none() {
        let xml = serialize_rect(&RectangleShape::default());
        assert!(!xml.contains("<hp:caption"), "캡션 부재 시 미방출: {}", xml);
        let xml = serialize_line(&LineShape::default());
        assert!(!xml.contains("<hp:caption"), "캡션 부재 시 미방출: {}", xml);
    }
}
