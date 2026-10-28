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

use super::section::{
    build_secpr_run_with_storage, render_hp_p_open_with_storage,
    render_paragraph_parts_with_storage,
};
use super::table::write_caption_with_storage;
use super::utils::{
    empty_tag_with_storage, end_tag_with_storage, start_tag_attrs_with_storage,
    start_tag_with_storage, text_with_storage, write_xml_decl_with_storage, xml_error,
    xml_escape_with_storage, xml_value,
};
use std::io::Write;

use quick_xml::Writer;

use crate::model::shape::{
    CommonObjAttr, DrawingObjAttr, HorzAlign, HorzRelTo, LineShape, ObjectNumberingType,
    OleDrawingAspect, OleShape, RectangleShape, ShapeComponentAttr, SizeCriterion, TextBox,
    TextFlow, TextWrap, VertAlign, VertRelTo,
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
    write_rect_with_storage(w, rect, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_rect_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    rect: &RectangleShape,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let c = &rect.common;
    let sa = &rect.drawing.shape_attr;
    // 속성 (부모 AbstractShapeObjectType + 자신):
    // id, zOrder, numberingType, textWrap, textFlow, lock, dropcapstyle,
    // href, groupLevel, instid, ratio
    let id_str = xml_value(storage, c.instance_id)?;
    let z_order = xml_value(storage, c.z_order)?;
    let tw = text_wrap_str(c.text_wrap);
    let tf = text_flow_str(c.text_flow);
    let group_level = xml_value(storage, sa.group_level)?;
    // 파서는 instid 를 drawing.inst_id 에 보존한다 (0이면 instance_id 대체).
    let instid = xml_value(
        storage,
        if rect.drawing.inst_id != 0 {
            rect.drawing.inst_id
        } else {
            c.instance_id
        },
    )?;
    let ratio = xml_value(storage, rect.round_rate)?;

    start_tag_attrs_with_storage(
        w,
        "hp:rect",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", numbering_type_str(c.numbering_type)),
            ("textWrap", tw),
            ("textFlow", tf),
            // [#2840] lock(개체 잠금) — 파서가 읽기 시작했으므로 IR 값을 방출한다.
            ("lock", bool01(c.locked)),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", &group_level),
            ("instid", &instid),
            ("ratio", &ratio),
        ],
        storage,
    )?;

    // 자식 순서 — 한컴 원본 실측 (tbox-v-flow-01) 기준.
    write_offset_with_storage(w, sa, storage)?;
    write_org_sz_with_storage(w, sa, storage)?;
    write_cur_sz_with_storage(w, sa, storage)?;
    write_flip_with_storage(w, sa, storage)?;
    write_rotation_info_with_storage(w, sa, storage)?;
    write_rendering_info_with_storage(w, sa, storage)?;
    write_line_shape_with_storage(w, &rect.drawing.border_line, storage)?;
    write_fill_brush_with_storage(w, &rect.drawing.fill, ctx, storage)?;
    write_shadow_with_storage(w, &rect.drawing, storage)?;

    // drawText: 글상자 내부 문단
    if let Some(ref tb) = rect.drawing.text_box {
        if !tb.paragraphs.is_empty() {
            write_draw_text_with_storage(w, tb, ctx, storage)?;
        }
    }

    // 꼭짓점 4점 (hc: 접두어)
    write_rect_pts_with_storage(w, &rect.x_coords, &rect.y_coords, storage)?;

    write_sz_with_storage(w, c, storage)?;
    write_pos_with_storage(w, c, storage)?;
    write_out_margin_with_storage(w, c, storage)?;
    // 캡션 (#1403) — OWPML AbstractShapeObjectType 순서: outMargin 과 shapeComment 사이
    if let Some(cap) = &rect.drawing.caption {
        write_caption_with_storage(w, cap, ctx, storage)?;
    }
    write_shape_comment_with_storage(w, c, storage)?;

    end_tag_with_storage(w, "hp:rect", storage)?;
    Ok(())
}

// =====================================================================
// <hp:line>
// =====================================================================

/// [Issue #1943] LinkLineType → HWPX `type` 속성 문자열 (parse_connect_line_type_attr 역).
fn link_line_type_str(t: crate::model::shape::LinkLineType) -> &'static str {
    use crate::model::shape::LinkLineType::*;
    match t {
        StraightNoArrow => "STRAIGHT_NOARROW",
        StraightOneWay => "STRAIGHT_ONEWAY",
        StraightBoth => "STRAIGHT_BOTH",
        StrokeNoArrow => "STROKE_NOARROW",
        StrokeOneWay => "STROKE_ONEWAY",
        StrokeBoth => "STROKE_BOTH",
        ArcNoArrow => "ARC_NOARROW",
        ArcOneWay => "ARC_ONEWAY",
        ArcBoth => "ARC_BOTH",
    }
}

/// `<hp:line>` / `<hp:connectLine>` 직렬화 진입점. LineShape IR → XML.
///
/// [Issue #1943] 종전 write_line 은 골격 속성(startX/Y/endX/Y attr + sz/pos/outMargin)
/// 만 방출해 (A) connector 보유 시에도 무조건 hp:line 으로 변질하고, (B) 컴포넌트
/// 블록(offset/orgSz/curSz/flip/rotationInfo/renderingInfo)·lineShape(색·굵기)·
/// fillBrush/shadow·좌표(hp:startPt/endPt 자식) 전체를 소실시켰다. 파서는 좌표를
/// startPt/endPt **자식 요소**로만 읽으므로(startX/Y attr 무시) 종전 좌표는 dead
/// 였다. write_rect 와 동형으로 전 구조를 방출한다.
pub fn write_line<W: Write>(
    w: &mut Writer<W>,
    line: &LineShape,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    write_line_with_storage(w, line, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_line_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    line: &LineShape,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let c = &line.common;
    let sa = &line.drawing.shape_attr;
    let is_connector = line.connector.is_some();
    let tag = if is_connector {
        "hp:connectLine"
    } else {
        "hp:line"
    };

    let id_str = xml_value(storage, c.instance_id)?;
    let z_order = xml_value(storage, c.z_order)?;
    let group_level = xml_value(storage, sa.group_level)?;
    let instid = xml_value(
        storage,
        if line.drawing.inst_id != 0 {
            line.drawing.inst_id
        } else {
            c.instance_id
        },
    )?;
    let srb = bool01(line.started_right_or_bottom);

    let mut attrs: Vec<(&str, &str)> = vec![
        ("id", &id_str),
        ("zOrder", &z_order),
        ("numberingType", numbering_type_str(c.numbering_type)),
        ("textWrap", text_wrap_str(c.text_wrap)),
        ("textFlow", text_flow_str(c.text_flow)),
        // [#2840] lock(개체 잠금) — IR 보존 값 방출.
        ("lock", bool01(c.locked)),
        ("dropcapstyle", "None"),
        ("href", ""),
        ("groupLevel", &group_level),
        ("instid", &instid),
    ];
    if let Some(conn) = &line.connector {
        storage
            .push(&mut attrs, ("type", link_line_type_str(conn.link_type)))
            .map_err(xml_error)?;
    }
    storage
        .push(&mut attrs, ("isReverseHV", srb))
        .map_err(xml_error)?;
    start_tag_attrs_with_storage(w, tag, &attrs, storage)?;

    // 컴포넌트 블록 + 지오메트리 (write_rect 동형): offset/orgSz/curSz/flip/
    // rotationInfo/renderingInfo → lineShape → fillBrush → shadow.
    write_shape_component_block_with_storage(w, sa, storage)?;
    write_line_shape_with_storage(w, &line.drawing.border_line, storage)?;
    write_fill_brush_with_storage(w, &line.drawing.fill, ctx, storage)?;
    write_shadow_with_storage(w, &line.drawing, storage)?;

    // 좌표 — startPt/endPt 자식 (파서가 읽는 유일 경로. 프리픽스 무관 로컬명
    // 매칭). 네임스페이스는 요소별로 다르다: hp:line 의 자식은 hc: (XSD LineType
    // — hp: 로 쓰면 한컴오피스가 문서를 거부한다), hp:connectLine 의 자식은
    // hp: (ConnectPointType 로컬 요소, subjectIDRef/subjectIdx 포함).
    let (sub_start_ref, sub_start_idx, sub_end_ref, sub_end_idx) = line
        .connector
        .as_ref()
        .map(|conn| {
            (
                conn.start_subject_id,
                conn.start_subject_index,
                conn.end_subject_id,
                conn.end_subject_index,
            )
        })
        .unwrap_or((0, 0, 0, 0));
    let (sx, sy) = (
        xml_value(storage, line.start.x)?,
        xml_value(storage, line.start.y)?,
    );
    let (ex, ey) = (
        xml_value(storage, line.end.x)?,
        xml_value(storage, line.end.y)?,
    );
    if is_connector {
        let (ssr, ssi) = (
            xml_value(storage, sub_start_ref)?,
            xml_value(storage, sub_start_idx)?,
        );
        let (esr, esi) = (
            xml_value(storage, sub_end_ref)?,
            xml_value(storage, sub_end_idx)?,
        );
        empty_tag_with_storage(
            w,
            "hp:startPt",
            &[
                ("x", &sx),
                ("y", &sy),
                ("subjectIDRef", &ssr),
                ("subjectIdx", &ssi),
            ],
            storage,
        )?;
        empty_tag_with_storage(
            w,
            "hp:endPt",
            &[
                ("x", &ex),
                ("y", &ey),
                ("subjectIDRef", &esr),
                ("subjectIdx", &esi),
            ],
            storage,
        )?;
    } else {
        empty_tag_with_storage(w, "hc:startPt", &[("x", &sx), ("y", &sy)], storage)?;
        empty_tag_with_storage(w, "hc:endPt", &[("x", &ex), ("y", &ey)], storage)?;
    }

    // connectLine 제어점 (꺾인/곡선 커넥터의 경로).
    if let Some(conn) = &line.connector {
        if !conn.control_points.is_empty() {
            start_tag_with_storage(w, "hp:controlPoints", storage)?;
            for p in &conn.control_points {
                let (px, py, pt) = (
                    xml_value(storage, p.x)?,
                    xml_value(storage, p.y)?,
                    xml_value(storage, p.point_type)?,
                );
                empty_tag_with_storage(
                    w,
                    "hp:point",
                    &[("x", &px), ("y", &py), ("type", &pt)],
                    storage,
                )?;
            }
            end_tag_with_storage(w, "hp:controlPoints", storage)?;
        }
    }

    write_sz_with_storage(w, c, storage)?;
    write_pos_with_storage(w, c, storage)?;
    write_out_margin_with_storage(w, c, storage)?;
    // 캡션 (#1403) — OWPML AbstractShapeObjectType 순서: outMargin 뒤
    if let Some(cap) = &line.drawing.caption {
        write_caption_with_storage(w, cap, ctx, storage)?;
    }
    // [#1588] 도형 설명 — caption 뒤 (write_rect/container 와 동형).
    write_shape_comment_with_storage(w, c, storage)?;

    end_tag_with_storage(w, tag, storage)?;
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
    write_container_open_with_storage(w, common, sa, &crate::storage::Unbounded)
}

pub(super) fn write_container_open_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    common: &CommonObjAttr,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let id_str = xml_value(storage, common.instance_id)?;
    let z_order = xml_value(storage, common.z_order)?;
    let tw = text_wrap_str(common.text_wrap);
    let tf = text_flow_str(common.text_flow);
    // [#5716] groupLevel 은 IR(shape_attr.group_level)을 보존한다. 종전 "0" 하드코딩은
    // 다른 그룹의 멤버인 컨테이너의 중첩 레벨이 저장 시 전부 0 으로 유실됐다
    // (리프 도형 경로는 #2746 에서 이미 보존 — 컨테이너 자신만 누락).
    let group_level = xml_value(storage, sa.group_level)?;

    start_tag_attrs_with_storage(
        w,
        "hp:container",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", numbering_type_str(common.numbering_type)),
            ("textWrap", tw),
            ("textFlow", tf),
            // [#2840] lock(개체 잠금) — IR 보존 값 방출.
            ("lock", bool01(common.locked)),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", &group_level),
            ("instid", &id_str),
        ],
        storage,
    )?;

    // 그룹 자신의 좌표계 — 자식 도형 앞에 방출 (write_rect 패턴과 동일 순서).
    write_offset_with_storage(w, sa, storage)?;
    write_org_sz_with_storage(w, sa, storage)?;
    write_cur_sz_with_storage(w, sa, storage)?;
    write_flip_with_storage(w, sa, storage)?;
    write_rotation_info_with_storage(w, sa, storage)?;
    write_rendering_info_with_storage(w, sa, storage)?;

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
    write_container_close_with_storage(w, caption, common, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_container_close_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    caption: Option<&crate::model::shape::Caption>,
    common: &CommonObjAttr,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    write_sz_with_storage(w, common, storage)?;
    write_pos_with_storage(w, common, storage)?;
    write_out_margin_with_storage(w, common, storage)?;
    if let Some(cap) = caption {
        write_caption_with_storage(w, cap, ctx, storage)?;
    }
    // 설명 (#1392) — caption 직후
    write_shape_comment_with_storage(w, common, storage)?;
    end_tag_with_storage(w, "hp:container", storage)
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
    write_ole_with_storage(w, ole, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_ole_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ole: &OleShape,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let c = &ole.common;
    // [#4669] 원본 `id` 는 `instid` 와 별개 값이다 — 파서가 보존한 원문을 되쓰고,
    // HWP5 출신(None)은 종전대로 instance_id 를 겸용한다.
    let id_str = xml_value(storage, ole.hwpx_ole_id.unwrap_or(c.instance_id))?;
    let instid_str = xml_value(storage, c.instance_id)?;
    let z_order = xml_value(storage, c.z_order)?;
    let tw = text_wrap_str(c.text_wrap);
    let tf = text_flow_str(c.text_flow);
    // owned 으로 변환해 ctx 불변 borrow 를 즉시 해제(이후 write_caption 의 &mut 사용).
    // [버그] ole.bin_data_id 는 HWP5 바이너리 상 4바이트(u32) 필드지만 BinData 테이블은
    // u16 ID 로 관리된다. 종전에는 `as u16` 로 상위 비트를 자른 뒤 조회해, 65536 이상인
    // (잘못된/조작된) 값이 하위 16비트가 같은 다른 BinData 항목을 가리키는 오참조를
    // 일으킬 수 있었다. try_from 으로 범위를 벗어나면 미등록으로 취급해 빈 참조로
    // 남긴다(기존 "미등록 id → 빈 문자열" 처리와 동일한 안전한 폴백).
    let bidref = xml_value(
        storage,
        u16::try_from(ole.bin_data_id)
            .ok()
            .and_then(|id| ctx.resolve_bin_id(id))
            .unwrap_or(""),
    )?;
    let draw_aspect = match ole.drawing_aspect {
        OleDrawingAspect::Icon => "ICON",
        OleDrawingAspect::Thumbnail => "THUMBNAIL",
        OleDrawingAspect::DocPrint => "DOCPRINT",
        OleDrawingAspect::Content => "CONTENT",
    };
    // [#2931] 개체 잠금(lock) — IR(common.locked)을 보존(종전 "0" 하드코딩 제거).
    let lock = if c.locked { "1" } else { "0" };
    // [#5716] groupLevel — 컨테이너와 동형. IR(drawing.shape_attr.group_level)을
    // 보존한다(종전 "0" 하드코딩은 그룹 내 OLE 의 중첩 레벨을 저장 시 유실).
    let group_level = xml_value(storage, ole.drawing.shape_attr.group_level)?;

    start_tag_attrs_with_storage(
        w,
        "hp:ole",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", numbering_type_str(c.numbering_type)),
            ("textWrap", tw),
            ("textFlow", tf),
            // [#2840] lock(개체 잠금) — IR 보존 값 방출.
            ("lock", bool01(c.locked)),
            ("dropcapstyle", "None"),
            ("href", ""),
            ("groupLevel", &group_level),
            ("instid", &instid_str),
            ("objectType", "UNKNOWN"),
            ("binaryItemIDRef", &bidref),
            ("hasMoniker", "0"),
            ("drawAspect", draw_aspect),
            ("eqBaseLine", "0"),
        ],
        storage,
    )?;

    // shape_attr 블록 (offset/orgSz/curSz/flip/rotationInfo/renderingInfo)
    write_shape_component_block_with_storage(w, &ole.drawing.shape_attr, storage)?;
    // 개체 영역
    let ex = xml_value(storage, ole.extent_x)?;
    let ey = xml_value(storage, ole.extent_y)?;
    empty_tag_with_storage(w, "hc:extent", &[("x", &ex), ("y", &ey)], storage)?;
    write_line_shape_with_storage(w, &ole.drawing.border_line, storage)?;
    write_sz_with_storage(w, c, storage)?;
    write_pos_with_storage(w, c, storage)?;
    write_out_margin_with_storage(w, c, storage)?;
    if let Some(cap) = &ole.caption {
        write_caption_with_storage(w, cap, ctx, storage)?;
    }
    write_shape_comment_with_storage(w, c, storage)?;

    end_tag_with_storage(w, "hp:ole", storage)?;
    Ok(())
}

// =====================================================================
// <hp:chart> — HWPX OOXML 차트 원형 재방출 (#3546)
// 파서는 hp:chart 를 OLE 모델(bin_data_id=60000+N)로 변환한다. 종전 저장은
// 이를 일반 OLE 로 방출해 hp:chart → hp:ole 치환·Chart/chartN.xml →
// BinData/image6000N.ooxml_chart 이동이 일어났고, 한컴오피스가 차트 XML 을
// OLE 복합문서로 해석했다. chart_id_ref 표식이 있으면 원형 구조로 되쓴다.
// =====================================================================
pub(crate) fn write_ole_or_chart<W: Write>(
    w: &mut Writer<W>,
    ole: &OleShape,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    write_ole_or_chart_with_storage(w, ole, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_ole_or_chart_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ole: &OleShape,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    match ole.chart_id_ref.as_deref() {
        Some(chart_id_ref) => write_chart_switch_with_storage(w, ole, chart_id_ref, ctx, storage),
        None => write_ole_with_storage(w, ole, ctx, storage),
    }
}

/// hp:chart 재방출 — 원본이 <hp:switch> 래핑(fallback OLE 보유)이었으면 같은
/// 구조로, bare <hp:chart> 였으면 같은 형태로 되쓴다.
const OOXML_CHART_REQUIRED_NS: &str = "http://www.hancom.co.kr/hwpml/2016/ooxmlchart";

fn write_chart_switch<W: Write>(
    w: &mut Writer<W>,
    ole: &OleShape,
    chart_id_ref: &str,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    write_chart_switch_with_storage(w, ole, chart_id_ref, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_chart_switch_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ole: &OleShape,
    chart_id_ref: &str,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    match &ole.chart_switch_fallback {
        Some(fallback) => {
            start_tag_with_storage(w, "hp:switch", storage)?;
            start_tag_attrs_with_storage(
                w,
                "hp:case",
                &[("hp:required-namespace", OOXML_CHART_REQUIRED_NS)],
                storage,
            )?;
            write_chart_element_with_storage(w, ole, chart_id_ref, ctx, storage)?;
            end_tag_with_storage(w, "hp:case", storage)?;
            start_tag_with_storage(w, "hp:default", storage)?;
            write_ole_with_storage(w, fallback, ctx, storage)?;
            end_tag_with_storage(w, "hp:default", storage)?;
            end_tag_with_storage(w, "hp:switch", storage)?;
            Ok(())
        }
        None => write_chart_element_with_storage(w, ole, chart_id_ref, ctx, storage),
    }
}

fn write_chart_element<W: Write>(
    w: &mut Writer<W>,
    ole: &OleShape,
    chart_id_ref: &str,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    write_chart_element_with_storage(w, ole, chart_id_ref, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_chart_element_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    ole: &OleShape,
    chart_id_ref: &str,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let c = &ole.common;
    let id_str = xml_value(storage, c.instance_id)?;
    let z_order = xml_value(storage, c.z_order)?;
    start_tag_attrs_with_storage(
        w,
        "hp:chart",
        &[
            ("id", &id_str),
            ("zOrder", &z_order),
            ("numberingType", numbering_type_str(c.numbering_type)),
            ("textWrap", text_wrap_str(c.text_wrap)),
            ("textFlow", text_flow_str(c.text_flow)),
            ("lock", bool01(c.locked)),
            ("dropcapstyle", "None"),
            ("chartIDRef", chart_id_ref),
        ],
        storage,
    )?;
    write_sz_with_storage(w, c, storage)?;
    write_pos_with_storage(w, c, storage)?;
    write_out_margin_with_storage(w, c, storage)?;
    // [#4319] 캡션 — 종전엔 hp:chart 재방출 경로에만 write_caption 호출이 없어
    // 파서를 고쳐도(ole.caption 정상 적재) 저장 시 다시 유실됐다(hp:ole 방출
    // 경로인 write_ole 는 이미 캡션을 쓴다 — 그쪽과 동형으로 맞춘다).
    if let Some(cap) = &ole.caption {
        write_caption_with_storage(w, cap, ctx, storage)?;
    }
    end_tag_with_storage(w, "hp:chart", storage)?;
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
///   명시적 여백은 0도 보존한다. 기존 생성기의 부재/0 기본값은 미방출한다.
pub fn write_draw_text<W: Write>(
    w: &mut Writer<W>,
    tb: &TextBox,
    ctx: &mut SerializeContext,
) -> Result<(), SerializeError> {
    write_draw_text_with_storage(w, tb, ctx, &crate::storage::Unbounded)
}

pub(super) fn write_draw_text_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    tb: &TextBox,
    ctx: &mut SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let depth = ctx.sub_list_depth;
    let result = (|| -> Result<(), SerializeError> {
        let lw = xml_value(storage, tb.max_width)?;
        start_tag_attrs_with_storage(
            w,
            "hp:drawText",
            &[("lastWidth", &lw), ("name", ""), ("editable", "0")],
            storage,
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

        start_tag_attrs_with_storage(
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
            storage,
        )?;

        // sub_list_depth: 글상자 경로 한정 colPr 인라인 방출 스코프 (#1379 3단계).
        ctx.sub_list_depth += 1;
        let mut vert_cursor: u32 = 0;
        for para in tb.paragraphs.iter() {
            ctx.para_shape_ids.reference(para.para_shape_id);
            let sid = ctx.effective_style_id(para.style_id);
            ctx.style_ids.reference(sid as u16);

            let (runs, linesegs, advance) =
                render_paragraph_parts_with_storage(para, vert_cursor, ctx, storage)?;
            vert_cursor = advance;
            let pid = ctx.next_para_id();
            let mut p_xml = render_hp_p_open_with_storage(para, pid, sid, storage)?;
            storage.push_str(&mut p_xml, &runs).map_err(xml_error)?;
            storage.push_str(&mut p_xml, &linesegs).map_err(xml_error)?;
            storage.push_str(&mut p_xml, "</hp:p>").map_err(xml_error)?;
            w.get_mut()
                .write_all(p_xml.as_bytes())
                .map_err(|e| SerializeError::XmlError(e.to_string()))?;
        }
        ctx.sub_list_depth -= 1;

        end_tag_with_storage(w, "hp:subList", storage)?;

        if tb.text_margin_explicit
            || tb.margin_left != 0
            || tb.margin_right != 0
            || tb.margin_top != 0
            || tb.margin_bottom != 0
        {
            let ml = xml_value(storage, tb.margin_left)?;
            let mr = xml_value(storage, tb.margin_right)?;
            let mt = xml_value(storage, tb.margin_top)?;
            let mb = xml_value(storage, tb.margin_bottom)?;
            empty_tag_with_storage(
                w,
                "hp:textMargin",
                &[("left", &ml), ("right", &mr), ("top", &mt), ("bottom", &mb)],
                storage,
            )?;
        }

        end_tag_with_storage(w, "hp:drawText", storage)?;
        Ok(())
    })();
    ctx.sub_list_depth = depth;
    result
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
    write_shape_component_block_with_storage(w, sa, &crate::storage::Unbounded)
}

pub(super) fn write_shape_component_block_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    write_offset_with_storage(w, sa, storage)?;
    write_org_sz_with_storage(w, sa, storage)?;
    write_cur_sz_with_storage(w, sa, storage)?;
    write_flip_with_storage(w, sa, storage)?;
    write_rotation_info_with_storage(w, sa, storage)?;
    write_rendering_info_with_storage(w, sa, storage)?;
    Ok(())
}

pub(super) fn write_offset<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    write_offset_with_storage(w, sa, &crate::storage::Unbounded)
}

pub(super) fn write_offset_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    // [#3544] hp:offset x/y 는 OWPML XSD 상 unsigned. 한컴 산출물은 음수 오프셋을
    // u32 wraparound 십진수로 기록하고(예: -2429 → "4294964867"), 파서도
    // `parse_u32 as i32` 로 같은 관례를 복호한다. IR 은 레이아웃 계산을 위해
    // signed 가 정당하므로 값은 두고, XML 경계에서만 부호화를 복원한다 —
    // signed 그대로 문자열화하면 `y="-2"` 류 스키마 위반이 된다.
    let x = xml_value(storage, sa.offset_x as u32)?;
    let y = xml_value(storage, sa.offset_y as u32)?;
    empty_tag_with_storage(w, "hp:offset", &[("x", &x), ("y", &y)], storage)
}

fn write_org_sz<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    write_org_sz_with_storage(w, sa, &crate::storage::Unbounded)
}

pub(super) fn write_org_sz_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    // [#4669] HWP5 저장을 위해 유효 extent로 materialize했더라도 HWPX 원문의
    // `orgSz=0` sentinel은 그대로 되돌린다.
    let width = xml_value(
        storage,
        if sa.original_width_was_zero {
            0
        } else {
            sa.original_width
        },
    )?;
    let height = xml_value(
        storage,
        if sa.original_height_was_zero {
            0
        } else {
            sa.original_height
        },
    )?;
    empty_tag_with_storage(
        w,
        "hp:orgSz",
        &[("width", &width), ("height", &height)],
        storage,
    )
}

fn write_cur_sz<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    write_cur_sz_with_storage(w, sa, &crate::storage::Unbounded)
}

pub(super) fn write_cur_sz_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    // [#2017] 파싱 시 orgSz로 materialize된 dimension 은 원본 `0` sentinel 로 복원.
    let width = xml_value(
        storage,
        if sa.current_width_was_zero {
            0
        } else {
            sa.current_width
        },
    )?;
    let height = xml_value(
        storage,
        if sa.current_height_was_zero {
            0
        } else {
            sa.current_height
        },
    )?;
    empty_tag_with_storage(
        w,
        "hp:curSz",
        &[("width", &width), ("height", &height)],
        storage,
    )
}

fn write_flip<W: Write>(w: &mut Writer<W>, sa: &ShapeComponentAttr) -> Result<(), SerializeError> {
    write_flip_with_storage(w, sa, &crate::storage::Unbounded)
}

pub(super) fn write_flip_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    empty_tag_with_storage(
        w,
        "hp:flip",
        &[
            ("horizontal", bool01(sa.horz_flip)),
            ("vertical", bool01(sa.vert_flip)),
        ],
        storage,
    )
}

fn write_rotation_info<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    write_rotation_info_with_storage(w, sa, &crate::storage::Unbounded)
}

pub(super) fn write_rotation_info_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let angle = xml_value(storage, sa.rotation_angle)?;
    let cx = xml_value(storage, sa.rotation_center.x)?;
    let cy = xml_value(storage, sa.rotation_center.y)?;
    empty_tag_with_storage(
        w,
        "hp:rotationInfo",
        &[
            ("angle", &angle),
            ("centerX", &cx),
            ("centerY", &cy),
            ("rotateimage", bool01(sa.rotate_image)),
        ],
        storage,
    )
}

/// `<hp:renderingInfo>` — `raw_rendering` (cnt u16 LE + trans 6×f64 + cnt×(sca, rot))
/// 를 디코드해 행렬을 재구성한다 (`parse_rendering_info` 의 역). raw 비정합/빈 경우
/// HWP와 같은 편집 변환을 생성한다. pic 자식도 공유 (그룹 내 자식 transMatrix 보존).
pub(crate) fn write_rendering_info<W: Write>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
) -> Result<(), SerializeError> {
    write_rendering_info_with_storage(w, sa, &crate::storage::Unbounded)
}

pub(super) fn write_rendering_info_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    sa: &ShapeComponentAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let rendering =
        crate::serializer::control::try_shape_rendering_bytes(sa, storage).map_err(xml_error)?;
    let (trans, pairs) = match decode_raw_rendering_with_storage(&rendering, storage)? {
        Some(value) => value,
        None => {
            let mut pairs = storage.vec(1).map_err(xml_error)?;
            storage
                .push(&mut pairs, (IDENTITY, IDENTITY))
                .map_err(xml_error)?;
            (IDENTITY, pairs)
        }
    };

    start_tag_with_storage(w, "hp:renderingInfo", storage)?;
    write_matrix_with_storage(w, "hc:transMatrix", &trans, storage)?;
    for (sca, rot) in &pairs {
        write_matrix_with_storage(w, "hc:scaMatrix", sca, storage)?;
        write_matrix_with_storage(w, "hc:rotMatrix", rot, storage)?;
    }
    end_tag_with_storage(w, "hp:renderingInfo", storage)
}

type RenderMatrices = ([f64; 6], Vec<([f64; 6], [f64; 6])>);

fn decode_raw_rendering(raw: &[u8]) -> Option<RenderMatrices> {
    decode_raw_rendering_with_storage(raw, &crate::storage::Unbounded)
        .expect("unbounded XML writer")
}

pub(super) fn decode_raw_rendering_with_storage<S: crate::storage::Storage>(
    raw: &[u8],
    storage: &S,
) -> Result<Option<RenderMatrices>, SerializeError>
where
    S::Error: std::fmt::Display,
{
    if raw.len() < 2 + 48 {
        return Ok(None);
    }
    let cnt = u16::from_le_bytes([raw[0], raw[1]]) as usize;
    if raw.len() < 2 + 48 + cnt * 96 {
        return Ok(None);
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
    let mut pairs = storage.vec(cnt).map_err(xml_error)?;
    for k in 0..cnt {
        let base = 2 + 48 + k * 96;
        storage
            .push(&mut pairs, (read6(base), read6(base + 48)))
            .map_err(xml_error)?;
    }
    Ok(Some((trans, pairs)))
}

/// 행렬 값 포맷: 정수는 정수 문자열, 비정수는 f32 정밀도 (파서가 f32 로 적재 —
/// 원본 "1.579917" 보존).
fn fmt_matrix_val(v: f64) -> String {
    fmt_matrix_val_with_storage(v, &crate::storage::Unbounded).expect("unbounded XML writer")
}

pub(super) fn fmt_matrix_val_with_storage<S: crate::storage::Storage>(
    v: f64,
    storage: &S,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    Ok(if v.fract() == 0.0 {
        xml_value(storage, format_args!("{}", v as i64))?
    } else {
        xml_value(storage, format_args!("{}", v as f32))?
    })
}

fn write_matrix<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    m: &[f64; 6],
) -> Result<(), SerializeError> {
    write_matrix_with_storage(w, name, m, &crate::storage::Unbounded)
}

pub(super) fn write_matrix_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    m: &[f64; 6],
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let mut vals = storage.vec(6).map_err(xml_error)?;
    for v in m {
        storage
            .push(&mut vals, fmt_matrix_val_with_storage(*v, storage)?)
            .map_err(xml_error)?;
    }
    empty_tag_with_storage(
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
        storage,
    )
}

// =====================================================================
// lineShape / fillBrush / shadow
// =====================================================================

/// `<hp:lineShape>` — `parse_line_shape_attr` 의 역매핑.
/// alpha 는 파서 미적재 → "0" 고정 방출. headStyle/tailStyle 은 bit 10~21 에서 복원한다(#2956).
pub(crate) fn write_line_shape<W: Write>(
    w: &mut Writer<W>,
    bl: &ShapeBorderLine,
) -> Result<(), SerializeError> {
    write_line_shape_with_storage(w, bl, &crate::storage::Unbounded)
}

pub(super) fn write_line_shape_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    bl: &ShapeBorderLine,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let color = color_to_hex_with_storage(bl.color, storage)?;
    let width = xml_value(storage, bl.width)?;
    // style 은 attr 하위 6비트. 정본 코드(0=NONE/1=SOLID/2=파선…)는 표 borderFill 의
    // border_line_type_from_code 및 HWP5 doc_info 와 동일. 종전에는 0 이 _ => SOLID 로
    // 떨어져 "선 없음" 도형 외곽선이 라운드트립에서 사각형 박스로 살아났다(#1531).
    let style = match bl.attr & 0x3F {
        0 => "NONE",
        1 => "SOLID",
        2 => "DOT",
        3 => "DASH",
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
    let head_fill_b = bl.attr & 0x8000_0000 != 0;
    let tail_fill_b = bl.attr & 0x4000_0000 != 0;
    let headfill = bool01(head_fill_b);
    let tailfill = bool01(tail_fill_b);
    let head_style = arrow_style_str((bl.attr >> 10) & 0x3F, head_fill_b);
    let tail_style = arrow_style_str((bl.attr >> 16) & 0x3F, tail_fill_b);
    let head_sz = arrow_size_str((bl.attr >> 22) & 0x0F);
    let tail_sz = arrow_size_str((bl.attr >> 26) & 0x0F);
    let outline = match bl.outline_style {
        1 => "OUTER",
        2 => "INNER",
        _ => "NORMAL",
    };
    empty_tag_with_storage(
        w,
        "hp:lineShape",
        &[
            ("color", &color),
            ("width", &width),
            ("style", style),
            ("endCap", end_cap),
            ("headStyle", head_style),
            ("tailStyle", tail_style),
            ("headfill", headfill),
            ("tailfill", tailfill),
            ("headSz", head_sz),
            ("tailSz", tail_sz),
            ("outlineStyle", outline),
            ("alpha", "0"),
        ],
        storage,
    )
}

/// HWP5 화살표 모양 값(hwplib LineArrowShape, `arrow_type_from_hwp` 참조) →
/// OWPML Core `ArrowType` 역매핑. fill 은 채움 여부(bit 30/31)다.
fn arrow_style_str(v: u32, fill: bool) -> &'static str {
    match v {
        1 => "ARROW",
        2 => "SPEAR",
        3 => "CONCAVE_ARROW",
        4 if fill => "FILLED_DIAMOND",
        4 => "EMPTY_DIAMOND",
        5 if fill => "FILLED_CIRCLE",
        5 => "EMPTY_CIRCLE",
        6 if fill => "FILLED_BOX",
        6 => "EMPTY_BOX",
        _ => "NORMAL",
    }
}

/// `parse_line_shape_attr::arrow_size` 의 역매핑 (0~8).
fn arrow_size_str(v: u32) -> &'static str {
    // OWPML Core 스키마 hc:ArrowSize 정본 리터럴은 *_LARGE (Core XML schema.xml:407).
    // "*_BIG" 는 스펙에 없는 비실재 토큰 — 파서는 하위호환을 위해 관용 수용하지만
    // (src/parser/hwpx/section.rs 의 parse_line_shape_attr), 직렬화는 정본만 방출한다.
    match v {
        1 => "SMALL_MEDIUM",
        2 => "SMALL_LARGE",
        3 => "MEDIUM_SMALL",
        4 => "MEDIUM_MEDIUM",
        5 => "MEDIUM_LARGE",
        6 => "LARGE_SMALL",
        7 => "LARGE_MEDIUM",
        8 => "LARGE_LARGE",
        _ => "SMALL_SMALL",
    }
}

/// `<hc:fillBrush><hc:winBrush .../></hc:fillBrush>` 를 방출하는 단일 소유 함수.
/// FillType::Solid 경로와 보존된 빈 채우기(FillType::None + solid 존재) 경로가
/// 동일한 직렬화를 쓰도록 한 곳에 모은다(균일 규칙).
pub(crate) fn write_fill_brush<W: Write>(
    w: &mut Writer<W>,
    fill: &Fill,
    ctx: &SerializeContext,
) -> Result<(), SerializeError> {
    write_fill_brush_with_storage(w, fill, ctx, &crate::storage::Unbounded)
}

fn write_win_brush<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    solid: &SolidFill,
    alpha: u8,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    use super::utils::{
        empty_tag_with_storage as empty_tag, end_tag_with_storage as end_tag,
        start_tag_attrs_with_storage as start_tag_attrs, start_tag_with_storage as start_tag,
        xml_value,
    };
    let face = color_to_hex_with_storage(solid.background_color, storage)?;
    let hatch = color_to_hex_with_storage(solid.pattern_color, storage)?;
    let alpha = fill_alpha_str_with_storage(alpha, storage)?;
    start_tag(w, "hc:fillBrush", storage)?;
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
            storage,
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
            storage,
        )?;
    }
    end_tag(w, "hc:fillBrush", storage)
}

/// `<hc:fillBrush>` — `parse_shape_fill_brush` 의 역매핑.
/// `FillType::None` 은 원본에 fillBrush 부재로 간주하되, 파서가 보존한 solid 가
/// 있으면(빈 winBrush) 복원한다.
///
/// 도형(rect/line)뿐 아니라 `header.rs` 의 borderFill 도 같은 fillBrush 구조를
/// 쓰므로 `pub(crate)` 로 공유한다.
pub(crate) fn write_fill_brush_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    fill: &Fill,
    ctx: &SerializeContext,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    use super::utils::{
        empty_tag_with_storage as empty_tag, end_tag_with_storage as end_tag,
        start_tag_attrs_with_storage as start_tag_attrs, start_tag_with_storage as start_tag,
        xml_value,
    };
    match fill.fill_type {
        // FillType::None 이지만 solid 데이터가 보존돼 있으면(원본 winBrush 가
        // faceColor="none"+무늬없음 으로 빈 채우기였던 경우) winBrush 를 그대로
        // 복원한다. solid 가 없으면 원본에 fillBrush 가 없었던 것이므로 미방출.
        FillType::None => match &fill.solid {
            Some(solid) => write_win_brush(w, solid, fill.alpha, storage),
            None => Ok(()),
        },
        FillType::Solid => {
            let solid = fill.solid.unwrap_or_default();
            write_win_brush(w, &solid, fill.alpha, storage)
        }
        FillType::Gradient => {
            let fallback = Default::default();
            let grad = fill.gradient.as_ref().unwrap_or(&fallback);
            let unknown_type;
            let gtype = match grad.gradient_type {
                1 => "LINEAR",
                2 => "RADIAL",
                3 => "CONICAL",
                4 => "SQUARE",
                other => {
                    unknown_type = xml_value(storage, other)?;
                    &unknown_type
                }
            };
            let angle = xml_value(storage, grad.angle)?;
            let cx = xml_value(storage, grad.center_x)?;
            let cy = xml_value(storage, grad.center_y)?;
            let step = xml_value(storage, grad.blur)?;
            let step_center = xml_value(storage, grad.step_center)?;
            let alpha = fill_alpha_str_with_storage(fill.alpha, storage)?;
            start_tag(w, "hc:fillBrush", storage)?;
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
                storage,
            )?;
            for c in &grad.colors {
                let v = color_to_hex_with_storage(*c, storage)?;
                empty_tag(w, "hc:color", &[("value", &v)], storage)?;
            }
            end_tag(w, "hc:gradation", storage)?;
            end_tag(w, "hc:fillBrush", storage)
        }
        FillType::Image => {
            let fallback = Default::default();
            let img = fill.image.as_ref().unwrap_or(&fallback);
            // [#2943] parse_shape_fill_brush(section.rs)는 12종 mode 를 모두 개별
            // ImageFillMode 로 적재하는데, 종전엔 여기서 3종만 구분하고 나머지 7종
            // (TileHorzTop/Bottom, TileVertLeft/Right, CenterTop/Bottom, LeftTop)이
            // 전부 "TILE"로 붕괴해 저장 시 실제 배치가 유실됐다.
            let mode = match img.fill_mode {
                ImageFillMode::TileHorzTop => "TILE_HORZ_TOP",
                ImageFillMode::TileHorzBottom => "TILE_HORZ_BOTTOM",
                ImageFillMode::TileVertLeft => "TILE_VERT_LEFT",
                ImageFillMode::TileVertRight => "TILE_VERT_RIGHT",
                ImageFillMode::FitToSize => "TOTAL",
                // HWP5 mode 15 is emitted as ZOOM by Hancom when saving HWPX.
                // It is an aspect-preserving fit, not the wildcard TILE fallback.
                // This writer is shared by shape fills and header BorderFill.
                ImageFillMode::Zoom | ImageFillMode::None => "ZOOM",
                ImageFillMode::Total => "TOTAL",
                ImageFillMode::Center => "CENTER",
                ImageFillMode::CenterTop => "CENTER_TOP",
                ImageFillMode::CenterBottom => "CENTER_BOTTOM",
                ImageFillMode::LeftTop => "TOP_LEFT_ALIGN",
                _ => "TILE",
            };
            start_tag(w, "hc:fillBrush", storage)?;
            // bin_data_id 가 ctx 에 등록돼 있으면 <hc:img> 참조를 방출(셀/쪽 배경 이미지
            // 보존). 미등록(예: body shape 의 fill 파서가 bin_data_id 미캡처)이면 종전대로
            // 빈 imgBrush — 잘못된 image0 참조로 3-way 단언을 깨지 않는다.
            match ctx.resolve_bin_id(img.bin_data_id) {
                Some(manifest_id) => {
                    start_tag_attrs(w, "hc:imgBrush", &[("mode", mode)], storage)?;
                    // [#6895] `ImageFill` 은 이진 HWP5 저장 순서를 담으므로 HWPX 속성명으로
                    // 나갈 때 되돌린다. 종전엔 그대로 써서 **HWPX 를 열어 저장할 때마다 두
                    // 값이 뒤바뀌었다**(두 번 저장하면 원상 복귀). 한/글은 같은 문서에서
                    // bright/contrast 를 보존한다.
                    let (display_bright, display_contrast) = img.display_brightness_contrast();
                    let bright = xml_value(storage, display_bright)?;
                    let contrast = xml_value(storage, display_contrast)?;
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
                        storage,
                    )?;
                    end_tag(w, "hc:imgBrush", storage)?;
                }
                None => {
                    empty_tag(w, "hc:imgBrush", &[("mode", mode)], storage)?;
                }
            }
            end_tag(w, "hc:fillBrush", storage)
        }
    }
}

/// winBrush/gradation alpha — 파서가 `f.clamp(0,1)*255` 로 적재하므로 0~1 분수로 방출.
fn fill_alpha_str_with_storage<S: crate::storage::Storage>(
    alpha: u8,
    storage: &S,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    super::utils::xml_value(storage, alpha as f64 / 255.0)
}

/// `hatch_style` (1~6) → OWPML hatchStyle. `parse_hatch_style` 의 역매핑.
fn hatch_style_str(pattern_type: i32) -> &'static str {
    match pattern_type {
        1 => "HORIZONTAL",
        2 => "VERTICAL",
        3 => "BACK_SLASH",
        4 => "SLASH",
        5 => "CROSS",
        6 => "CROSS_DIAGONAL",
        // 계약(1~6) 밖의 값은 무늬 정보가 없다는 뜻이므로 임의의 무늬로
        // 둔갑시키지 않고 가장 무난한 HORIZONTAL 로 방출한다.
        _ => "HORIZONTAL",
    }
}

/// `<hp:shadow>` — `parse_shape_shadow_attr` 의 역매핑.
/// 전 필드 0 이면 원본에 shadow 부재로 간주하여 미방출.
/// alpha 는 정수 방출 (파서의 `>1.0` 경로와 정합 — 0/1 경계값만 비가역).
pub(crate) fn write_shadow<W: Write>(
    w: &mut Writer<W>,
    d: &DrawingObjAttr,
) -> Result<(), SerializeError> {
    write_shadow_with_storage(w, d, &crate::storage::Unbounded)
}

pub(super) fn write_shadow_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    d: &DrawingObjAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
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
    let color = color_to_hex_with_storage(d.shadow_color, storage)?;
    let ox = xml_value(storage, d.shadow_offset_x)?;
    let oy = xml_value(storage, d.shadow_offset_y)?;
    let alpha = xml_value(storage, d.shadow_alpha)?;
    empty_tag_with_storage(
        w,
        "hp:shadow",
        &[
            ("type", ty),
            ("color", &color),
            ("offsetX", &ox),
            ("offsetY", &oy),
            ("alpha", &alpha),
        ],
        storage,
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
    write_rect_pts_with_storage(w, x, y, &crate::storage::Unbounded)
}

pub(super) fn write_rect_pts_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    x: &[i32; 4],
    y: &[i32; 4],
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    for (i, name) in ["hc:pt0", "hc:pt1", "hc:pt2", "hc:pt3"].iter().enumerate() {
        let px = xml_value(storage, x[i])?;
        let py = xml_value(storage, y[i])?;
        empty_tag_with_storage(w, name, &[("x", &px), ("y", &py)], storage)?;
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
    write_shape_comment_with_storage(w, c, &crate::storage::Unbounded)
}

pub(super) fn write_shape_comment_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    c: &CommonObjAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    if c.description.is_empty() {
        return Ok(());
    }
    start_tag_with_storage(w, "hp:shapeComment", storage)?;
    text_with_storage(w, &c.description, storage)?;
    end_tag_with_storage(w, "hp:shapeComment", storage)
}

// =====================================================================
// 공통 자식 요소 (sz / pos / outMargin)
// =====================================================================

fn write_sz<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    write_sz_with_storage(w, c, &crate::storage::Unbounded)
}

pub(super) fn write_sz_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    c: &CommonObjAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let width = xml_value(storage, c.width)?;
    let height = xml_value(storage, c.height)?;
    // [#2712] widthRelTo/heightRelTo/protect 는 IR 을 보존한다. 종전 "ABSOLUTE"/"0"
    // 하드코딩은 파서가 이미 읽어 둔 값(section.rs:2901/2904/2907, OLE 는 5967)을 저장 때
    // 버려, 단/쪽/문단에 맞춘 도형이 절대값으로 굳고 "크기 고정"이 풀렸다. 실제 한글 산출
    // 파일 samples/hwpx/143E433F503322BD33.hwpx 의 hp:rect·hp:ole 이 protect="1" 인데
    // 종전 방출은 이를 "0" 으로 되썼다. 이웃 write_pos 는 이미 모든 필드를 통과시키고,
    // 같은 hp:sz 를 다루는 form.rs:183-188 도 3속성 모두 보존한다. 표는 #2697/#2701 에서
    // 같은 결함을 같은 방식으로 정리했다.
    empty_tag_with_storage(
        w,
        "hp:sz",
        &[
            ("width", &width),
            ("widthRelTo", size_criterion_str(c.width_criterion)),
            ("height", &height),
            ("heightRelTo", height_criterion_str(c.height_criterion)),
            ("protect", bool01(c.size_protect)),
        ],
        storage,
    )
}

/// 너비 기준 → HWPX `widthRelTo`. 파서 `parse_size_criterion(_, true)` 의 정확한 역이다
/// (`parser/hwpx/section.rs:1844`, 호출부 `:2901`). 너비는 HWP5 attr bit 15-17 로 3비트라
/// 5값 전부를 담는다(`model/shape.rs:83`).
pub(super) fn size_criterion_str(c: SizeCriterion) -> &'static str {
    match c {
        SizeCriterion::Paper => "PAPER",
        SizeCriterion::Page => "PAGE",
        SizeCriterion::Column => "COLUMN",
        SizeCriterion::Para => "PARA",
        SizeCriterion::Absolute => "ABSOLUTE",
    }
}

/// 높이 기준 → HWPX `heightRelTo`. 파서는 높이를
/// `parse_size_criterion(_, allow_column_para = false)` 로 읽으므로(`section.rs:2904`)
/// 치역이 `{PAPER, PAGE, ABSOLUTE}` 3값뿐이다. 방출도 같은 3값으로 접어야 왕복이 정확한
/// 역이 된다. HWP5 측 `height_criterion_to_bits`(`common_obj_attr_writer.rs:160`)와 모델
/// 주석(`model/shape.rs:85`, bit 18-19)도 동일하게 접는다.
pub(super) fn height_criterion_str(c: SizeCriterion) -> &'static str {
    match c {
        SizeCriterion::Paper => "PAPER",
        SizeCriterion::Page => "PAGE",
        SizeCriterion::Column | SizeCriterion::Para | SizeCriterion::Absolute => "ABSOLUTE",
    }
}

fn write_pos<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    write_pos_with_storage(w, c, &crate::storage::Unbounded)
}

pub(super) fn write_pos_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    c: &CommonObjAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let treat = bool01(c.treat_as_char);
    let vert_offset = xml_value(storage, c.vertical_offset)?;
    let horz_offset = xml_value(storage, c.horizontal_offset)?;
    let hold = bool01(c.prevent_page_break != 0); // [#1594] IR 보존
    empty_tag_with_storage(
        w,
        "hp:pos",
        &[
            ("treatAsChar", treat),
            // [#2784] affectLSpacing 은 IR(affect_line_spacing)을 보존한다. 종전 "0"
            // 하드코딩은 "줄 간격에 영향" 켜진 도형이 저장 시 1→0 으로 드롭됐다.
            ("affectLSpacing", bool01(c.affect_line_spacing)),
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
        storage,
    )
}

fn write_out_margin<W: Write>(w: &mut Writer<W>, c: &CommonObjAttr) -> Result<(), SerializeError> {
    write_out_margin_with_storage(w, c, &crate::storage::Unbounded)
}

pub(super) fn write_out_margin_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    c: &CommonObjAttr,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let l = xml_value(storage, c.margin.left)?;
    let r = xml_value(storage, c.margin.right)?;
    let t = xml_value(storage, c.margin.top)?;
    let b = xml_value(storage, c.margin.bottom)?;
    empty_tag_with_storage(
        w,
        "hp:outMargin",
        &[("left", &l), ("right", &r), ("top", &t), ("bottom", &b)],
        storage,
    )
}

/// ColorRef (0xAABBGGRR) → "#RRGGBB" / "#AARRGGBB". 0xFFFFFFFF → "none".
/// `parse_color_str` 의 역매핑 (header.rs `color_hex` 와 동일 규칙).
pub(crate) fn color_to_hex(c: ColorRef) -> String {
    color_to_hex_with_storage(c, &crate::storage::Unbounded).expect("unbounded color")
}
pub(super) fn color_to_hex_with_storage<S: crate::storage::Storage>(
    c: ColorRef,
    storage: &S,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    if c == 0xFFFFFFFF {
        return super::utils::xml_value(storage, "none");
    }
    let a = ((c >> 24) & 0xFF) as u8;
    let r = (c & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = ((c >> 16) & 0xFF) as u8;
    if a == 0 {
        super::utils::xml_value(storage, format_args!("#{:02X}{:02X}{:02X}", r, g, b))
    } else {
        super::utils::xml_value(
            storage,
            format_args!("#{:02X}{:02X}{:02X}{:02X}", a, r, g, b),
        )
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

/// `dropcapstyle` 방출 문자열. OWPML Core 스키마 `DropCapStyleType`
/// (None/DoubleLine/TripleLine/Margin) 그대로 왕복한다.
pub(crate) fn drop_cap_style_str(s: crate::model::shape::DropCapStyle) -> &'static str {
    use crate::model::shape::DropCapStyle;
    match s {
        DropCapStyle::None => "None",
        DropCapStyle::DoubleLine => "DoubleLine",
        DropCapStyle::TripleLine => "TripleLine",
        DropCapStyle::Margin => "Margin",
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

    /// #2943: imgBrush mode 는 12종 중 3종만 구분하면 나머지 7종
    /// (TileHorzTop 등)이 저장 시 전부 TILE 로 붕괴한다. 각 모드가 자기 고유의
    /// mode 문자열로 방출돼야 한다.
    #[test]
    fn task2943_img_brush_mode_roundtrip_not_collapsed_to_tile() {
        use crate::model::style::{Fill, FillType, ImageFill, ImageFillMode};
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        let ctx = SerializeContext::collect_from_document(&Default::default());
        let fill = Fill {
            fill_type: FillType::Image,
            image: Some(ImageFill {
                fill_mode: ImageFillMode::TileHorzTop,
                ..Default::default()
            }),
            ..Default::default()
        };
        write_fill_brush(&mut w, &fill, &ctx).expect("write_fill_brush");
        let xml = String::from_utf8(w.into_inner()).unwrap();
        assert!(
            xml.contains("mode=\"TILE_HORZ_TOP\""),
            "TileHorzTop 이 TILE 로 붕괴함: {xml}"
        );
    }

    /// #1531: 선 없음(style code 0) 도형 외곽선이 라운드트립에서 SOLID(사각형 박스)로
    /// 살아나면 안 된다. endCap(bit 6~9)이 함께 설정돼도 NONE 이 보존돼야 한다.
    #[test]
    fn task1531_line_shape_none_preserved() {
        assert_eq!(line_shape_style(0), "NONE"); // 정본 코드 0 = NONE
        assert_eq!(line_shape_style(1), "SOLID"); // 1 = SOLID
        assert_eq!(line_shape_style(2), "DOT"); // HWP5 파선은 HWPX DOT로 저장한다.
        let none_with_flat_end_cap = 1 << 6;
        assert_eq!(line_shape_style(none_with_flat_end_cap), "NONE");
    }

    /// #2956: attr 에 파싱된 화살표 끝 모양(bit 16~21, 채움 bit 30)이 저장 시
    /// "NORMAL" 로 하드코딩되지 않고 보존돼야 한다.
    #[test]
    fn task2956_line_shape_arrow_style_preserved() {
        use crate::model::style::ShapeBorderLine;
        // tail = FILLED_DIAMOND(4) + tail_fill(bit30)
        let attr = (4u32 << 16) | (1 << 30);
        let bl = ShapeBorderLine {
            attr,
            ..Default::default()
        };
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_line_shape(&mut w, &bl).expect("write_line_shape");
        let xml = String::from_utf8(w.into_inner()).unwrap();
        assert!(
            xml.contains("tailStyle=\"FILLED_DIAMOND\""),
            "화살표 끝 모양이 소실됨: {xml}"
        );
    }

    /// #3022: hc:ArrowSize 의 스펙 리터럴은 `*_LARGE` 이다(Core XML schema.xml:407).
    /// headSz/tailSz 방출이 스펙에 없는 `*_BIG` 표기로 나가면 안 된다.
    #[test]
    fn task3022_arrow_size_uses_spec_large_literal() {
        assert_eq!(arrow_size_str(2), "SMALL_LARGE");
        assert_eq!(arrow_size_str(5), "MEDIUM_LARGE");
        assert_eq!(arrow_size_str(6), "LARGE_SMALL");
        assert_eq!(arrow_size_str(7), "LARGE_MEDIUM");
        assert_eq!(arrow_size_str(8), "LARGE_LARGE");
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
        // [Issue #1943] 좌표는 startPt/endPt 자식으로 방출한다 (파서가 읽는
        // 유일 경로). 종전 startX/Y attr 은 파서가 무시하는 dead 출력이었다.
        // hp:line 의 자식 네임스페이스는 hc: 다 (XSD LineType — hp: 로 쓰면
        // 한컴오피스가 문서를 거부한다).
        let mut line = LineShape::default();
        line.start = Point { x: 100, y: 200 };
        line.end = Point { x: 300, y: 400 };
        let xml = serialize_line(&line);
        assert!(
            xml.contains(r#"<hc:startPt x="100" y="200""#),
            "startPt 자식 방출: {xml}"
        );
        assert!(
            xml.contains(r#"<hc:endPt x="300" y="400""#),
            "endPt 자식 방출: {xml}"
        );
        // 컴포넌트 블록·lineShape 보존 (#1943 (B)).
        assert!(xml.contains("<hp:offset "), "컴포넌트 블록: {xml}");
        assert!(xml.contains("<hp:lineShape "), "lineShape: {xml}");
    }

    /// [Issue #1943 (A)] connector 보유 LineShape 는 hp:connectLine 으로 방출하고
    /// type/제어점을 보존한다 (종전 hp:line 변질로 커넥터 소실).
    #[test]
    fn connector_line_emits_connect_line_tag_and_type() {
        use crate::model::shape::{ConnectorControlPoint, ConnectorData, LinkLineType};
        let mut line = LineShape::default();
        line.start = Point { x: 0, y: 0 };
        line.end = Point { x: 1257, y: 0 };
        line.connector = Some(ConnectorData {
            link_type: LinkLineType::StrokeOneWay,
            control_points: vec![
                ConnectorControlPoint {
                    x: 0,
                    y: 0,
                    point_type: 3,
                },
                ConnectorControlPoint {
                    x: 100,
                    y: 0,
                    point_type: 26,
                },
            ],
            ..Default::default()
        });
        let xml = serialize_line(&line);
        assert!(xml.contains("<hp:connectLine "), "connectLine 태그: {xml}");
        assert!(
            xml.contains(r#"type="STROKE_ONEWAY""#),
            "connector type: {xml}"
        );
        assert!(xml.contains("<hp:controlPoints>"), "제어점 방출: {xml}");
        assert!(xml.contains(r#"<hp:point x="100" y="0" type="26"/>"#));
        assert!(
            !xml.contains("<hp:line "),
            "connectLine 이 hp:line 으로 변질 금지"
        );
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
                data: vec![0u8; 4].into(),
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

    /// [Issue #3544] hp:offset x/y 는 OWPML XSD 상 unsigned — 음수 IR 오프셋은
    /// 한컴 관례대로 u32 wraparound 십진수로 방출해야 한다 (파서 `parse_u32 as
    /// i32` 복호의 역함수). signed 그대로 문자열화하면 `y="-2"` 류 스키마 위반.
    #[test]
    fn issue3544_negative_offset_emitted_as_u32_wraparound() {
        let mut rect = RectangleShape::default();
        rect.drawing.shape_attr.offset_x = -8974;
        rect.drawing.shape_attr.offset_y = -2;
        let xml = serialize_rect(&rect);
        assert!(
            xml.contains(r#"<hp:offset x="4294958322" y="4294967294"/>"#),
            "음수 오프셋은 u32 wraparound 십진수로 방출되어야 한다: {xml}"
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

    // ---------- #2712: 도형 hp:sz 크기 기준·크기 보호 라운드트립 ----------

    /// 도형을 한 문단짜리 `<hs:sec>` 으로 감싸 다시 파싱한다(IR 수준 역검증용).
    fn reparse_shape_common(fragment: &str) -> crate::model::shape::CommonObjAttr {
        let xml = format!(
            concat!(
                r#"<?xml version="1.0" encoding="UTF-8"?>"#,
                r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph""#,
                r#" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">"#,
                r#"<hp:p id="0" paraPrIDRef="0" styleIDRef="0">"#,
                r#"<hp:run charPrIDRef="0">{}<hp:t/></hp:run></hp:p></hs:sec>"#
            ),
            fragment
        );
        let section = crate::parser::hwpx::section::parse_hwpx_section(&xml)
            .expect("파싱 가능한 도형 조각이어야 함");
        match &section.paragraphs[0].controls[0] {
            crate::model::control::Control::Shape(s) => s.common().clone(),
            other => panic!("도형 컨트롤이어야 함: {other:?}"),
        }
    }

    #[test]
    fn task2712_shape_sz_criteria_and_protect_emitted_from_ir() {
        // [#2712] hp:sz 의 widthRelTo/heightRelTo/protect 는 IR 을 보존해야 한다.
        // 종전 "ABSOLUTE"/"ABSOLUTE"/"0" 하드코딩은 파서가 이미 읽어 둔 값
        // (section.rs:2901/2904/2907)을 저장 때 버렸다. RED.
        let mut rect = RectangleShape::default();
        rect.common.width_criterion = SizeCriterion::Column;
        rect.common.height_criterion = SizeCriterion::Paper;
        rect.common.size_protect = true;
        let xml = serialize_rect(&rect);
        assert!(
            xml.contains(r#"widthRelTo="COLUMN""#),
            "widthRelTo 가 IR(Column)로 방출돼야 함(종전 ABSOLUTE 하드코딩): {xml}"
        );
        assert!(
            xml.contains(r#"heightRelTo="PAPER""#),
            "heightRelTo 가 IR(Paper)로 방출돼야 함(종전 ABSOLUTE 하드코딩): {xml}"
        );
        assert!(
            xml.contains(r#"protect="1""#),
            "protect 가 IR(size_protect=true)로 방출돼야 함(종전 0 하드코딩): {xml}"
        );

        // IR 수준 역검증 — 되읽었을 때 값이 그대로 복원돼야 한다.
        let back = reparse_shape_common(&xml);
        assert_eq!(back.width_criterion, SizeCriterion::Column);
        assert_eq!(back.height_criterion, SizeCriterion::Paper);
        assert!(back.size_protect);
    }

    #[test]
    fn task2712_line_sz_criteria_and_protect_emitted_from_ir() {
        // write_sz 는 rect/line/container/ole 이 공유하므로 선 도형에서도 확인한다.
        let mut line = LineShape::default();
        line.common.width_criterion = SizeCriterion::Page;
        line.common.size_protect = true;
        let xml = serialize_line(&line);
        assert!(xml.contains(r#"widthRelTo="PAGE""#), "{xml}");
        assert!(xml.contains(r#"protect="1""#), "{xml}");
    }

    #[test]
    fn task2712_height_criterion_is_exact_inverse_of_parser() {
        // 파서는 높이를 parse_size_criterion(_, allow_column_para=false) 로 읽어
        // 치역이 {PAPER, PAGE, ABSOLUTE} 3값뿐이다(section.rs:2904). 방출도 같은 3값으로
        // 접어야 왕복이 정확한 역이 된다. Column/Para 를 그대로 내보내면 되읽을 때
        // Absolute 로 떨어져 왕복이 깨진다.
        assert_eq!(height_criterion_str(SizeCriterion::Paper), "PAPER");
        assert_eq!(height_criterion_str(SizeCriterion::Page), "PAGE");
        assert_eq!(height_criterion_str(SizeCriterion::Absolute), "ABSOLUTE");
        assert_eq!(height_criterion_str(SizeCriterion::Column), "ABSOLUTE");
        assert_eq!(height_criterion_str(SizeCriterion::Para), "ABSOLUTE");

        // 너비는 5값 전부를 담는다(bit 15-17).
        assert_eq!(size_criterion_str(SizeCriterion::Paper), "PAPER");
        assert_eq!(size_criterion_str(SizeCriterion::Page), "PAGE");
        assert_eq!(size_criterion_str(SizeCriterion::Column), "COLUMN");
        assert_eq!(size_criterion_str(SizeCriterion::Para), "PARA");
        assert_eq!(size_criterion_str(SizeCriterion::Absolute), "ABSOLUTE");

        // 높이에 Column 을 넣어도 방출은 ABSOLUTE 로 접히고, 되읽어도 Absolute 다.
        let mut rect = RectangleShape::default();
        rect.common.height_criterion = SizeCriterion::Column;
        let xml = serialize_rect(&rect);
        assert!(xml.contains(r#"heightRelTo="ABSOLUTE""#), "{xml}");
        assert_eq!(
            reparse_shape_common(&xml).height_criterion,
            SizeCriterion::Absolute
        );
    }

    #[test]
    fn task2712_shape_sz_defaults_unchanged() {
        // 기본 IR(Absolute/Absolute/false)은 종전 출력과 동일해야 한다(무변화 보장).
        // 실측상 samples/hwpx 의 hp:rect 309개·hp:pic 188개가 모두 이 조합이다.
        let xml = serialize_rect(&RectangleShape::default());
        assert!(xml.contains(r#"widthRelTo="ABSOLUTE""#), "{xml}");
        assert!(xml.contains(r#"heightRelTo="ABSOLUTE""#), "{xml}");
        assert!(xml.contains(r#"protect="0""#), "{xml}");
    }

    #[test]
    fn task_m100_hatch_style_str_covers_all_six() {
        // 계약(1~6) 값 6개를 모두 명시적으로 매핑하는지 확인한다.
        // 이전에는 6이 catch-all(_) 분기에 얹혀 있어, 계약 밖의 값(예: 손상된
        // 원본의 pattern_type=99)도 CROSS_DIAGONAL 로 둔갑했다.
        assert_eq!(hatch_style_str(1), "HORIZONTAL");
        assert_eq!(hatch_style_str(2), "VERTICAL");
        assert_eq!(hatch_style_str(3), "BACK_SLASH");
        assert_eq!(hatch_style_str(4), "SLASH");
        assert_eq!(hatch_style_str(5), "CROSS");
        assert_eq!(hatch_style_str(6), "CROSS_DIAGONAL");
        assert_eq!(hatch_style_str(99), "HORIZONTAL");
    }

    /// [#4669] 재방출 `id` 는 파서가 보존한 원문(hwpx_ole_id)을 되쓰고 `instid`
    /// 는 instance_id 를 쓴다 — 원산 파일은 두 값이 다르다(실측 2141242094 /
    /// 1067500271). 종전엔 둘 다 instance_id 로 방출돼 id 원문이 유실됐다.
    /// curSz=0 센티널(#2017)의 OLE 경로 복원도 함께 고정한다.
    #[test]
    fn issue4669_write_ole_preserves_original_id_and_cur_sz_zero() {
        use crate::model::document::Document;

        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);

        let mut ole = OleShape::default();
        ole.hwpx_ole_id = Some(2141242094);
        ole.common.instance_id = 1067500271;
        ole.drawing.shape_attr.original_width = 42001;
        ole.drawing.shape_attr.original_height = 13501;
        ole.drawing.shape_attr.current_width = 42001;
        ole.drawing.shape_attr.current_height = 13501;
        ole.drawing.shape_attr.current_width_was_zero = true;
        ole.drawing.shape_attr.current_height_was_zero = true;

        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_ole(&mut w, &ole, &mut ctx).expect("write_ole");
        let xml = String::from_utf8(w.into_inner()).unwrap();

        assert!(xml.contains(r#"id="2141242094""#), "id 원문 보존: {xml}");
        assert!(
            xml.contains(r#"instid="1067500271""#),
            "instid 는 instance_id: {xml}"
        );
        assert!(
            xml.contains(r#"<hp:curSz width="0" height="0"/>"#),
            "curSz=0 원문 복원(#2017 센티널): {xml}"
        );
        assert!(
            xml.contains(r#"<hp:orgSz width="42001" height="13501"/>"#),
            "orgSz 보존: {xml}"
        );
    }

    /// [#4669] HWP5 출신(hwpx_ole_id=None)은 종전대로 id=instid=instance_id.
    #[test]
    fn issue4669_write_ole_without_original_id_falls_back_to_instance_id() {
        use crate::model::document::Document;

        let doc = Document::default();
        let mut ctx = SerializeContext::collect_from_document(&doc);

        let mut ole = OleShape::default();
        ole.common.instance_id = 77;

        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_ole(&mut w, &ole, &mut ctx).expect("write_ole");
        let xml = String::from_utf8(w.into_inner()).unwrap();
        assert!(xml.contains(r#"id="77""#), "{xml}");
        assert!(xml.contains(r#"instid="77""#), "{xml}");
    }

    /// [버그] OLE 의 `bin_data_id` 는 HWP5 바이너리상 u32 필드다. 종전 코드는
    /// `as u16` 로 상위 비트를 잘라 조회했기 때문에, 65536 이상인 id가 하위 16비트가
    /// 같은 *다른* BinData 항목(예: id=5)을 잘못 가리킬 수 있었다. 이 값이 등록된
    /// BinData 범위(u16)를 벗어나면 그 어떤 항목도 가리키지 않고 빈 참조로 남아야
    /// 한다(오참조 방지).
    #[test]
    fn ole_bin_data_id_beyond_u16_does_not_alias_truncated_entry() {
        use crate::model::bin_data::{BinData, BinDataContent, BinDataType};
        use crate::model::document::Document;

        let mut doc = Document::default();
        // 하위 16비트가 0x10005 와 같은(=5) 정상 BinData 항목을 등록해 둔다.
        doc.bin_data_content.push(BinDataContent {
            id: 5,
            data: vec![0, 1, 2].into(),
            extension: "png".to_string(),
        });
        doc.doc_info.bin_data_list.push(BinData {
            data_type: BinDataType::Embedding,
            storage_id: 5,
            extension: Some("png".to_string()),
            ..Default::default()
        });
        let mut ctx = SerializeContext::collect_from_document(&doc);
        // 등록 확인: id=5 는 정상적으로 조회돼야 한다.
        assert_eq!(ctx.resolve_bin_id(5), Some("image5"));

        let ole = OleShape {
            bin_data_id: 0x1_0005, // truncate 시 5 가 되는 값(범위 밖)
            drawing_aspect: OleDrawingAspect::Content,
            ..Default::default()
        };

        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        write_ole(&mut w, &ole, &mut ctx).expect("write_ole");
        let xml = String::from_utf8(w.into_inner()).unwrap();

        assert!(
            !xml.contains(r#"binaryItemIDRef="image5""#),
            "u32 bin_data_id 가 u16 로 잘려 무관한 image5 를 오참조함: {xml}"
        );
    }
}
