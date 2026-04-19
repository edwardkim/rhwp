//! 그리기 개체 파싱 (도형, 그림)
//!
//! GSO(General Shape Object) 컨트롤과 관련 서브타입을 파싱한다.

use crate::parser::body_text::parse_paragraph_list;
use crate::parser::byte_reader::ByteReader;
use crate::parser::record::Record;
use crate::parser::tags;
use crate::model::control::Control;
use crate::model::image::{ImageEffect, Picture};
use crate::model::shape::{
    ArcShape, Caption, CaptionDirection, ChartShape, ChartType, CommonObjAttr, CurveShape, DrawingObjAttr,
    EllipseShape, GroupShape, HorzAlign, HorzRelTo, LineShape, OleDrawingAspect, OleShape, PolygonShape,
    RectangleShape, ShapeComponentAttr, ShapeObject, TextWrap, VertAlign, VertRelTo,
};
use crate::model::style::{Fill, ShapeBorderLine};
use crate::model::Point;
use crate::model::Padding;
use crate::parser::doc_info;
use super::parse_caption;

// ============================================================
// 그리기 개체 ('gso ')
// ============================================================

/// 그리기 개체 컨트롤 파싱
pub(crate) fn parse_gso_control(ctrl_data: &[u8], child_records: &[Record]) -> Control {
    // CTRL_HEADER에서 공통 개체 속성 파싱
    let common = parse_common_obj_attr(ctrl_data);

    // 자식 레코드에서 SHAPE_COMPONENT와 개별 도형 태그 찾기
    let mut drawing = DrawingObjAttr::default();
    let mut shape_tag_id: Option<u16> = None;
    let mut shape_tag_data: &[u8] = &[];
    let mut text_paragraphs = Vec::new();
    let mut is_container = false;
    // Task #195: 차트/OLE 감지
    let mut chart_data_bytes: Option<Vec<u8>> = None;
    let mut ole_tag_data: Option<&[u8]> = None;

    // 레벨 기반 필터링: 첫 번째 레코드(SHAPE_COMPONENT)의 레벨을 기준으로
    // 자신의 레코드만 처리하고, 중첩 컨트롤의 하위 레코드는 무시
    let base_level = child_records.first().map(|r| r.level).unwrap_or(0);
    let mut shape_component_parsed = false;

    // 그룹 컨테이너 사전 감지
    let has_container_tag = child_records.iter().any(|r| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT_CONTAINER);
    let has_child_sc = if let Some((sc_idx, sc_rec)) = child_records.iter().enumerate()
        .find(|(_, r)| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT) {
        let sc_level = sc_rec.level;
        child_records[sc_idx + 1..].iter().any(|r|
            r.tag_id == tags::HWPTAG_SHAPE_COMPONENT && r.level == sc_level + 1
        )
    } else { false };
    if has_container_tag {
        is_container = true;
    } else if has_child_sc {
        is_container = true;
    }
    // SHAPE_COMPONENT 인덱스 찾기 (캡션/텍스트박스 LIST_HEADER 구분에 필요)
    let shape_comp_idx = child_records.iter().position(|r|
        r.tag_id == tags::HWPTAG_SHAPE_COMPONENT || r.tag_id == tags::HWPTAG_SHAPE_COMPONENT_CONTAINER
    );

    for record in child_records {
        match record.tag_id {
            tags::HWPTAG_SHAPE_COMPONENT if !shape_component_parsed => {
                shape_component_parsed = true;
                let parsed = parse_shape_component_full(&record.data);
                drawing.shape_attr = parsed.attr;
                drawing.border_line = parsed.border;
                drawing.fill = parsed.fill;
                drawing.shadow_type = parsed.shadow_type;
                drawing.shadow_color = parsed.shadow_color;
                drawing.shadow_offset_x = parsed.shadow_offset_x;
                drawing.shadow_offset_y = parsed.shadow_offset_y;
                drawing.inst_id = parsed.inst_id;
                drawing.shadow_alpha = parsed.shadow_alpha;
            }
            tags::HWPTAG_SHAPE_COMPONENT_CONTAINER if !shape_component_parsed => {
                shape_component_parsed = true;
                is_container = true;
                let parsed = parse_shape_component_full(&record.data);
                drawing.shape_attr = parsed.attr;
            }
            tags::HWPTAG_SHAPE_COMPONENT_LINE
            | tags::HWPTAG_SHAPE_COMPONENT_RECTANGLE
            | tags::HWPTAG_SHAPE_COMPONENT_ELLIPSE
            | tags::HWPTAG_SHAPE_COMPONENT_ARC
            | tags::HWPTAG_SHAPE_COMPONENT_POLYGON
            | tags::HWPTAG_SHAPE_COMPONENT_CURVE
            | tags::HWPTAG_SHAPE_COMPONENT_PICTURE
                if record.level <= base_level + 1 =>
            {
                shape_tag_id = Some(record.tag_id);
                shape_tag_data = &record.data;
            }
            // Task #195: OLE 태그 (도형 타입으로 분류)
            tags::HWPTAG_SHAPE_COMPONENT_OLE if record.level <= base_level + 1 => {
                shape_tag_id = Some(record.tag_id);
                shape_tag_data = &record.data;
                ole_tag_data = Some(&record.data);
            }
            // Task #195: 차트 데이터 (하위 레코드 트리 전체를 raw로 병합 보존)
            tags::HWPTAG_CHART_DATA => {
                let mut buf = record.data.clone();
                // 이 CHART_DATA 이후의 더 깊은 레벨 하위 태그 전체를 병합 (단순화: 직접 자식만)
                // 단계 3 범위는 CHART_DATA 본문만으로 라운드트립 충분
                // (단계 5에서 필요 시 하위 태그 구조화 파싱)
                let _ = &mut buf; // suppress warning
                chart_data_bytes = Some(record.data.clone());
            }
            tags::HWPTAG_LIST_HEADER => {}
            _ => {}
        }
    }

    // 캡션 파싱: SHAPE_COMPONENT 앞의 LIST_HEADER는 캡션
    let mut caption: Option<Caption> = None;
    if let Some(comp_idx) = shape_comp_idx {
        let caption_start = child_records[..comp_idx]
            .iter()
            .position(|r| r.tag_id == tags::HWPTAG_LIST_HEADER);
        if let Some(start) = caption_start {
            let caption_records: Vec<Record> =
                child_records[start..comp_idx].iter().cloned().collect();
            if !caption_records.is_empty() {
                caption = Some(parse_caption(&caption_records));
            }
        }
    }

    // 텍스트박스 LIST_HEADER + 문단 수집: SHAPE_COMPONENT 이후의 LIST_HEADER
    let mut list_started = false;
    let mut list_header_data: Option<&[u8]> = None;
    let mut list_records: Vec<&Record> = Vec::new();
    let after_shape_comp = shape_comp_idx.map(|i| i + 1).unwrap_or(0);
    for record in &child_records[after_shape_comp..] {
        if !list_started && record.tag_id == tags::HWPTAG_LIST_HEADER {
            list_started = true;
            list_header_data = Some(&record.data);
            continue;
        }
        if list_started {
            list_records.push(record);
        }
    }
    if !list_records.is_empty() {
        let owned: Vec<Record> = list_records.iter().map(|r| (*r).clone()).collect();
        text_paragraphs = parse_paragraph_list(&owned);
    }

    // 글상자 설정
    if !text_paragraphs.is_empty() {
        let mut text_box = crate::model::shape::TextBox {
            paragraphs: text_paragraphs,
            ..Default::default()
        };

        // LIST_HEADER 데이터 파싱: para_count(4) + list_attr(4) + margins(8) + max_width(4) = 20 bytes
        // Note: para_count는 스펙상 INT16이지만 실제 HWP 파일에서는 UINT32로 저장됨
        if let Some(lh_data) = list_header_data {
            let mut lr = ByteReader::new(lh_data);
            let _para_count = lr.read_u32().unwrap_or(0);
            text_box.list_attr = lr.read_u32().unwrap_or(0);
            // list_attr bit 5~6: 세로 정렬 (표 67)
            let v_align = ((text_box.list_attr >> 5) & 0x03) as u8;
            text_box.vertical_align = match v_align {
                1 => crate::model::table::VerticalAlign::Center,
                2 => crate::model::table::VerticalAlign::Bottom,
                _ => crate::model::table::VerticalAlign::Top,
            };
            text_box.margin_left = lr.read_i16().unwrap_or(0);
            text_box.margin_right = lr.read_i16().unwrap_or(0);
            text_box.margin_top = lr.read_i16().unwrap_or(0);
            text_box.margin_bottom = lr.read_i16().unwrap_or(0);
            text_box.max_width = lr.read_u32().unwrap_or(0);
            // 나머지 바이트 보존 (라운드트립용)
            if lr.remaining() > 0 {
                text_box.raw_list_header_extra = lr.read_bytes(lr.remaining())
                    .unwrap_or_default().to_vec();
            }
        }

        drawing.text_box = Some(text_box);
    }

    // 캡션을 drawing에 저장 (도형 공통)
    drawing.caption = caption;

    // Task #195: 차트 우선 분기 (CHART_DATA가 있으면 GSO 다른 태그 종류와 무관하게 차트로 분류)
    if let Some(raw_chart) = chart_data_bytes.take() {
        let mut chart = ChartShape::default();
        chart.common = common.clone();
        chart.drawing = drawing;
        chart.raw_chart_data = raw_chart;
        chart.caption = chart.drawing.caption.take();
        // 단계 4에서 chart_type/title/series를 raw_chart_data에서 추출
        return Control::Shape(Box::new(ShapeObject::Chart(Box::new(chart))));
    }

    // Task #195: OLE 개체 분기
    if shape_tag_id == Some(tags::HWPTAG_SHAPE_COMPONENT_OLE) {
        let ole_data = ole_tag_data.unwrap_or(shape_tag_data);
        let mut ole = parse_ole_shape(common.clone(), drawing, ole_data);
        ole.caption = ole.drawing.caption.take();
        return Control::Shape(Box::new(ShapeObject::Ole(Box::new(ole))));
    }

    // 그림 개체
    if shape_tag_id == Some(tags::HWPTAG_SHAPE_COMPONENT_PICTURE) {
        let mut picture = parse_picture(common, drawing.shape_attr.clone(), shape_tag_data);
        // 캡션은 drawing에서 이미 파싱됨 → picture.caption에 복사
        picture.caption = drawing.caption;
        return Control::Picture(Box::new(picture));
    }

    // 묶음 개체 (Group/Container)
    if is_container {
        let mut group = GroupShape::default();
        group.common = common;
        group.shape_attr = drawing.shape_attr;
        group.children = parse_container_children(child_records);
        group.caption = drawing.caption;
        return Control::Shape(Box::new(ShapeObject::Group(group)));
    }

    // 일반 도형
    match shape_tag_id {
        Some(tags::HWPTAG_SHAPE_COMPONENT_LINE) => {
            let mut line = LineShape::default();
            line.common = common;
            let is_connector = drawing.shape_attr.ctrl_id == tags::SHAPE_CONNECTOR_ID;
            line.drawing = drawing;
            parse_line_shape_data(shape_tag_data, &mut line, is_connector);
            Control::Shape(Box::new(ShapeObject::Line(line)))
        }
        Some(tags::HWPTAG_SHAPE_COMPONENT_RECTANGLE) => {
            let mut rect = RectangleShape::default();
            rect.common = common;
            rect.drawing = drawing;
            parse_rect_shape_data(shape_tag_data, &mut rect);
            Control::Shape(Box::new(ShapeObject::Rectangle(rect)))
        }
        Some(tags::HWPTAG_SHAPE_COMPONENT_ELLIPSE) => {
            let mut ellipse = EllipseShape::default();
            ellipse.common = common;
            ellipse.drawing = drawing;
            parse_ellipse_shape_data(shape_tag_data, &mut ellipse);
            Control::Shape(Box::new(ShapeObject::Ellipse(ellipse)))
        }
        Some(tags::HWPTAG_SHAPE_COMPONENT_ARC) => {
            let mut arc = ArcShape::default();
            arc.common = common;
            arc.drawing = drawing;
            parse_arc_shape_data(shape_tag_data, &mut arc);
            Control::Shape(Box::new(ShapeObject::Arc(arc)))
        }
        Some(tags::HWPTAG_SHAPE_COMPONENT_POLYGON) => {
            let mut poly = PolygonShape::default();
            poly.common = common;
            poly.drawing = drawing;
            parse_polygon_shape_data(shape_tag_data, &mut poly);
            Control::Shape(Box::new(ShapeObject::Polygon(poly)))
        }
        Some(tags::HWPTAG_SHAPE_COMPONENT_CURVE) => {
            let mut curve = CurveShape::default();
            curve.common = common;
            curve.drawing = drawing;
            parse_curve_shape_data(shape_tag_data, &mut curve);
            Control::Shape(Box::new(ShapeObject::Curve(curve)))
        }
        _ => {
            // 알 수 없는 도형 → 사각형으로 대체
            // Task #195 이후: CHART_DATA/OLE은 위에서 분기되므로 이 경로로 오지 않음
            let mut rect = RectangleShape::default();
            rect.common = common;
            rect.drawing = drawing;
            Control::Shape(Box::new(ShapeObject::Rectangle(rect)))
        }
    }
}

// ============================================================
// Task #195: OLE 개체 파싱
// ============================================================

/// HWPTAG_SHAPE_COMPONENT_OLE 레코드 파싱
///
/// 1.hwp 실측 바이트 레이아웃 (30바이트):
/// ```
/// 01 00 00 00   u32 property/type (1)
/// 20 1C 00 00   u32 extent_x (HWPUNIT)
/// 20 1C 00 00   u32 extent_y
/// 01 00 00 00   u32 bin_data_id  ← DocInfo BinData 목록의 storage_id
/// 00 00 00 00   u32 reserved/flags
/// 00 00 00 00   u32 reserved
/// 00 00 00 00   u32 reserved
/// 00 00         u16 reserved/aspect
/// ```
pub(crate) fn parse_ole_shape(common: CommonObjAttr, drawing: DrawingObjAttr, tag_data: &[u8]) -> OleShape {
    let mut ole = OleShape::default();
    ole.common = common;
    ole.drawing = drawing;
    ole.raw_tag_data = tag_data.to_vec();

    let mut r = ByteReader::new(tag_data);
    let _property = r.read_u32().unwrap_or(0);
    ole.extent_x = r.read_i32().unwrap_or(0);
    ole.extent_y = r.read_i32().unwrap_or(0);
    ole.bin_data_id = r.read_u32().unwrap_or(0);
    // 뒤에 flags/aspect 필드가 있을 수 있으나 스펙 불확실 — 기본값 유지
    ole.flags = 0;
    ole.drawing_aspect = OleDrawingAspect::Content;
    ole
}

/// CTRL_HEADER 데이터에서 공통 개체 속성 파싱
pub(crate) fn parse_common_obj_attr(ctrl_data: &[u8]) -> CommonObjAttr {
    let mut common = CommonObjAttr::default();
    let mut r = ByteReader::new(ctrl_data);

    let attr = r.read_u32().unwrap_or(0);
    common.attr = attr;
    common.treat_as_char = attr & 0x01 != 0;
    common.vert_rel_to = match (attr >> 3) & 0x03 {
        1 => VertRelTo::Page,
        2 => VertRelTo::Para,
        _ => VertRelTo::Paper,
    };
    common.vert_align = match (attr >> 5) & 0x07 {
        0 => VertAlign::Top,
        1 => VertAlign::Center,
        2 => VertAlign::Bottom,
        3 => VertAlign::Inside,
        4 => VertAlign::Outside,
        _ => VertAlign::Top,
    };
    common.horz_rel_to = match (attr >> 8) & 0x03 {
        0 => HorzRelTo::Paper,  // 종이 영역 (HWPCTL 스펙)
        1 => HorzRelTo::Page,   // 쪽 영역
        2 => HorzRelTo::Column, // 다단 영역
        3 => HorzRelTo::Para,   // 문단 영역
        _ => HorzRelTo::Paper,
    };
    common.horz_align = match (attr >> 10) & 0x07 {
        0 => HorzAlign::Left,
        1 => HorzAlign::Center,
        2 => HorzAlign::Right,
        3 => HorzAlign::Inside,
        4 => HorzAlign::Outside,
        _ => HorzAlign::Left,
    };
    // bit 15-17: WidthCriterion (너비 기준)
    common.width_criterion = match (attr >> 15) & 0x07 {
        0 => crate::model::shape::SizeCriterion::Paper,
        1 => crate::model::shape::SizeCriterion::Page,
        2 => crate::model::shape::SizeCriterion::Column,
        3 => crate::model::shape::SizeCriterion::Para,
        _ => crate::model::shape::SizeCriterion::Absolute,
    };
    // bit 18-19: HeightCriterion (높이 기준)
    common.height_criterion = match (attr >> 18) & 0x03 {
        0 => crate::model::shape::SizeCriterion::Paper,
        1 => crate::model::shape::SizeCriterion::Page,
        _ => crate::model::shape::SizeCriterion::Absolute,
    };
    // hwplib 기준 TextFlowMethod: 0=어울림, 1=자리차지, 2=글뒤로, 3=글앞으로
    common.text_wrap = match (attr >> 21) & 0x07 {
        0 => TextWrap::Square,        // 어울림 (FitWithText)
        1 => TextWrap::TopAndBottom,   // 자리차지 (TakePlace)
        2 => TextWrap::BehindText,     // 글 뒤로
        3 => TextWrap::InFrontOfText,  // 글 앞으로
        _ => TextWrap::Square,
    };

    common.vertical_offset = r.read_u32().unwrap_or(0);
    common.horizontal_offset = r.read_u32().unwrap_or(0);
    common.width = r.read_u32().unwrap_or(0);
    common.height = r.read_u32().unwrap_or(0);
    common.z_order = r.read_i32().unwrap_or(0);

    // 바깥 여백
    common.margin = Padding {
        left: r.read_i16().unwrap_or(0),
        right: r.read_i16().unwrap_or(0),
        top: r.read_i16().unwrap_or(0),
        bottom: r.read_i16().unwrap_or(0),
    };

    common.instance_id = r.read_u32().unwrap_or(0);

    // 쪽나눔 방지 (INT32, 4바이트)
    if r.remaining() >= 4 {
        common.prevent_page_break = r.read_i32().unwrap_or(0);
    }

    // 설명문 (있으면)
    if r.remaining() >= 2 {
        common.description = r.read_hwp_string().unwrap_or_default();
    }

    // 남은 바이트 보존 (라운드트립용)
    if r.remaining() > 0 {
        common.raw_extra = r.read_bytes(r.remaining()).unwrap_or_default().to_vec();
    }
    common
}

/// SHAPE_COMPONENT 파싱 결과
struct ShapeComponentParsed {
    attr: ShapeComponentAttr,
    border: ShapeBorderLine,
    fill: Fill,
    shadow_type: u32,
    shadow_color: u32,
    shadow_offset_x: i32,
    shadow_offset_y: i32,
    inst_id: u32,
    shadow_alpha: u8,
}

/// SHAPE_COMPONENT 레코드 전체 파싱 (ShapeComponentAttr + border_line + fill + shadow)
///
/// 레코드 구조:
/// - 컨트롤 ID (4바이트) × 1~2회 (GenShapeObject이면 2회)
/// - ShapeComponentAttr (42바이트)
/// - Rendering 정보 (2 + 48 + cnt×96 바이트)
/// - 테두리 선 정보 (13바이트: color 4 + width 4 + attr 4 + outline 1)
/// - 채우기 정보 (가변)
/// - 그림자 정보 (16바이트: type 4 + color 4 + offsetX 4 + offsetY 4)
fn parse_shape_component_full(data: &[u8]) -> ShapeComponentParsed {
    // 컨트롤 ID 건너뛰기: GenShapeObject(top-level)이면 ID 2회, group child이면 1회
    let is_two_ctrl_id = data.len() >= 8 && data[0..4] == data[4..8];
    let id_offset = if is_two_ctrl_id { 8 } else { 4 };

    if data.len() < id_offset {
        return ShapeComponentParsed {
            attr: ShapeComponentAttr::default(),
            border: ShapeBorderLine::default(),
            fill: Fill::default(),
            shadow_type: 0, shadow_color: 0, shadow_offset_x: 0, shadow_offset_y: 0,
            inst_id: 0, shadow_alpha: 0,
        };
    }

    // ctrl_id 보존 (라운드트립용)
    let ctrl_id = if data.len() >= 4 {
        u32::from_le_bytes([data[0], data[1], data[2], data[3]])
    } else {
        0
    };

    let mut r = ByteReader::new(&data[id_offset..]);

    // ShapeComponentAttr (42바이트)
    let mut attr = ShapeComponentAttr::default();
    attr.ctrl_id = ctrl_id;
    attr.is_two_ctrl_id = is_two_ctrl_id;
    attr.offset_x = r.read_i32().unwrap_or(0);
    attr.offset_y = r.read_i32().unwrap_or(0);
    attr.group_level = r.read_u16().unwrap_or(0);
    attr.local_file_version = r.read_u16().unwrap_or(0);
    attr.original_width = r.read_u32().unwrap_or(0);
    attr.original_height = r.read_u32().unwrap_or(0);
    attr.current_width = r.read_u32().unwrap_or(0);
    attr.current_height = r.read_u32().unwrap_or(0);

    let flip = r.read_u32().unwrap_or(0);
    attr.flip = flip;
    attr.horz_flip = flip & 0x01 != 0;
    attr.vert_flip = flip & 0x02 != 0;

    attr.rotation_angle = r.read_i16().unwrap_or(0);
    attr.rotation_center.x = r.read_i32().unwrap_or(0);
    attr.rotation_center.y = r.read_i32().unwrap_or(0);

    // 렌더링 행렬 시작 위치
    let rendering_start = id_offset + r.position();

    // Rendering 정보 파싱 (변환 행렬) — 합성 변환 계산 후 border/fill 파싱
    let cnt = r.read_u16().unwrap_or(0) as usize;
    {
        // 아핀 변환 행렬: [a, b, tx, c, d, ty] → (x',y') = (a*x+b*y+tx, c*x+d*y+ty)
        fn read_matrix(r: &mut ByteReader) -> [f64; 6] {
            let mut m = [0.0f64; 6];
            for v in m.iter_mut() {
                let bytes = r.read_bytes(8).unwrap_or_default();
                if bytes.len() == 8 {
                    *v = f64::from_le_bytes([bytes[0],bytes[1],bytes[2],bytes[3],bytes[4],bytes[5],bytes[6],bytes[7]]);
                }
            }
            m
        }
        // 두 아핀 행렬 합성: result = A × B
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

        let translation = read_matrix(&mut r);
        let mut result = translation;
        for _ in 0..cnt {
            let scale = read_matrix(&mut r);
            let rotation = read_matrix(&mut r);
            result = compose(&result, &rotation);
            result = compose(&result, &scale);
        }
        // 합성된 아핀 행렬 [a, b, tx, c, d, ty] 추출
        attr.render_sx = result[0]; // a
        attr.render_b  = result[1]; // b (회전/전단 성분)
        attr.render_tx = result[2]; // tx
        attr.render_c  = result[3]; // c (회전/전단 성분)
        attr.render_sy = result[4]; // d
        attr.render_ty = result[5]; // ty
    }

    // raw_rendering: 렌더링 행렬만 보존 (border/fill/shadow 제외)
    // 속성 편집 후에도 렌더링 행렬을 라운드트립 보존하기 위함
    let rendering_end = id_offset + r.position();
    if rendering_start < data.len() {
        attr.raw_rendering = data[rendering_start..rendering_end.min(data.len())].to_vec();
    }

    // 테두리 선 정보 (13바이트: color 4 + width 4 + attr 4 + outline 1)
    // hwplib 참조: color=readUInt4, thickness=readSInt4, property=readUInt4, outlineStyle=readUInt1
    let mut border = ShapeBorderLine::default();
    if r.remaining() >= 13 {
        border.color = r.read_color_ref().unwrap_or(0);
        border.width = r.read_i32().unwrap_or(0);
        border.attr = r.read_u32().unwrap_or(0);
        border.outline_style = r.read_u8().unwrap_or(0);
    }

    // 채우기 정보
    let fill = if r.remaining() >= 4 {
        doc_info::parse_fill(&mut r)
    } else {
        Fill::default()
    };

    // 그림자 정보 (hwplib ForShapeComponent.shadowInfo 참조)
    // type(u32) + color(u32) + offsetX(i32) + offsetY(i32) = 16바이트
    let (shadow_type, shadow_color, shadow_offset_x, shadow_offset_y) = if r.remaining() >= 16 {
        (
            r.read_u32().unwrap_or(0),
            r.read_u32().unwrap_or(0),
            r.read_i32().unwrap_or(0),
            r.read_i32().unwrap_or(0),
        )
    } else {
        (0, 0, 0, 0)
    };

    // 인스턴스 ID (4바이트) + 예약 (1바이트) + 그림자 투명도 (1바이트) = 6바이트
    let inst_id = if r.remaining() >= 4 { r.read_u32().unwrap_or(0) } else { 0 };
    if r.remaining() >= 1 { let _ = r.read_u8(); } // 예약 (skip)
    let shadow_alpha = if r.remaining() >= 1 { r.read_u8().unwrap_or(0) } else { 0 };

    ShapeComponentParsed { attr, border, fill, shadow_type, shadow_color, shadow_offset_x, shadow_offset_y, inst_id, shadow_alpha }
}

/// 묶음 개체(Container)의 자식 도형 파싱
///
/// SHAPE_COMPONENT_CONTAINER 또는 첫 SHAPE_COMPONENT 이후의 레코드에서
/// SHAPE_COMPONENT + 도형 태그 쌍을 찾아 각각을 개별 ShapeObject로 파싱한다.
fn parse_container_children(child_records: &[Record]) -> Vec<ShapeObject> {
    let mut children = Vec::new();

    // SHAPE_COMPONENT_CONTAINER 이후 또는 구버전 그룹의 첫 SHAPE_COMPONENT 이후
    let container_idx = child_records
        .iter()
        .position(|r| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT_CONTAINER);

    // 첫 SHAPE_COMPONENT 위치 찾기 (캡션 등이 앞에 올 수 있음)
    let first_sc_idx = child_records.iter().position(|r| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT);

    let start = match container_idx {
        Some(idx) => idx + 1,
        None => {
            // 구버전: CONTAINER 태그 없이 첫 SHAPE_COMPONENT가 그룹 자체
            if let Some(sc_idx) = first_sc_idx {
                sc_idx + 1 // 첫 SHAPE_COMPONENT(그룹 자체) 건너뛰고 자식부터 시작
            } else {
                return children;
            }
        }
    };

    let records = &child_records[start..];

    // SHAPE_COMPONENT를 기준으로 자식 도형 경계 식별
    // 직접 자식 레벨의 SHAPE_COMPONENT만 경계로 사용 (인라인 컨트롤의 깊은 레벨 제외)
    let parent_level = first_sc_idx
        .map(|i| child_records[i].level)
        .or_else(|| child_records.first().map(|r| r.level))
        .unwrap_or(0);
    let child_level = parent_level + 1;
    let mut comp_indices: Vec<usize> = Vec::new();
    for (i, record) in records.iter().enumerate() {
        if record.tag_id == tags::HWPTAG_SHAPE_COMPONENT && record.level == child_level {
            comp_indices.push(i);
        }
    }


    for (ci, &comp_start) in comp_indices.iter().enumerate() {
        let comp_end = if ci + 1 < comp_indices.len() {
            comp_indices[ci + 1]
        } else {
            records.len()
        };

        let child_slice = &records[comp_start..comp_end];
        if child_slice.is_empty() {
            continue;
        }

        // SHAPE_COMPONENT에서 속성+테두리+채우기+그림자 파싱
        let parsed = parse_shape_component_full(&child_slice[0].data);
        let mut child_drawing = DrawingObjAttr {
            shape_attr: parsed.attr,
            border_line: parsed.border,
            fill: parsed.fill,
            shadow_type: parsed.shadow_type,
            shadow_color: parsed.shadow_color,
            shadow_offset_x: parsed.shadow_offset_x,
            shadow_offset_y: parsed.shadow_offset_y,
            inst_id: parsed.inst_id,
            shadow_alpha: parsed.shadow_alpha,
            text_box: None,
            caption: None,
        };
        // tb_attr: SHAPE_COMPONENT 인라인 텍스트 속성 (미구현, 항상 None)
        let tb_attr: Option<(i16, i16, i16, i16, u32, u32)> = None;

        // 도형 태그 찾기 (직접 자식 level만 — 중첩 Group 자식 제외)
        let direct_child_level = child_slice[0].level + 1;
        let mut shape_tag_id: Option<u16> = None;
        let mut shape_tag_data: &[u8] = &[];
        for record in &child_slice[1..] {
            if record.level != direct_child_level { continue; }
            match record.tag_id {
                tags::HWPTAG_SHAPE_COMPONENT_LINE
                | tags::HWPTAG_SHAPE_COMPONENT_RECTANGLE
                | tags::HWPTAG_SHAPE_COMPONENT_ELLIPSE
                | tags::HWPTAG_SHAPE_COMPONENT_ARC
                | tags::HWPTAG_SHAPE_COMPONENT_POLYGON
                | tags::HWPTAG_SHAPE_COMPONENT_CURVE
                | tags::HWPTAG_SHAPE_COMPONENT_PICTURE => {
                    shape_tag_id = Some(record.tag_id);
                    shape_tag_data = &record.data;
                    break;
                }
                _ => {}
            }
        }

        // LIST_HEADER 이후 문단 수집 (자식 범위 내)
        let mut list_started = false;
        let mut list_header_data: Option<&[u8]> = None;
        let mut list_records: Vec<&Record> = Vec::new();
        for record in &child_slice[1..] {
            if record.tag_id == tags::HWPTAG_LIST_HEADER && !list_started {
                list_started = true;
                list_header_data = Some(&record.data);
                continue;
            }
            if list_started {
                list_records.push(record);
            }
        }
        if !list_records.is_empty() {
            let owned: Vec<Record> = list_records.iter().map(|r| (*r).clone()).collect();
            let paragraphs = parse_paragraph_list(&owned);
            if !paragraphs.is_empty() {
                let mut text_box = crate::model::shape::TextBox {
                    paragraphs,
                    ..Default::default()
                };
                // LIST_HEADER 데이터에서 글상자 속성 파싱
                if let Some(lh_data) = list_header_data {
                    let mut lr = ByteReader::new(lh_data);
                    let _para_count = lr.read_u32().unwrap_or(0);
                    text_box.list_attr = lr.read_u32().unwrap_or(0);
                    let v_align = ((text_box.list_attr >> 5) & 0x03) as u8;
                    text_box.vertical_align = match v_align {
                        1 => crate::model::table::VerticalAlign::Center,
                        2 => crate::model::table::VerticalAlign::Bottom,
                        _ => crate::model::table::VerticalAlign::Top,
                    };
                    text_box.margin_left = lr.read_i16().unwrap_or(0);
                    text_box.margin_right = lr.read_i16().unwrap_or(0);
                    text_box.margin_top = lr.read_i16().unwrap_or(0);
                    text_box.margin_bottom = lr.read_i16().unwrap_or(0);
                    text_box.max_width = lr.read_u32().unwrap_or(0);
                    if lr.remaining() > 0 {
                        text_box.raw_list_header_extra = lr.read_bytes(lr.remaining())
                            .unwrap_or_default().to_vec();
                    }
                } else if let Some((ml, mr, mt, mb, max_w, list_attr)) = tb_attr {
                    // LIST_HEADER 레코드가 없으면 SHAPE_COMPONENT 인라인 속성 사용
                    text_box.margin_left = ml;
                    text_box.margin_right = mr;
                    text_box.margin_top = mt;
                    text_box.margin_bottom = mb;
                    text_box.max_width = max_w;
                    text_box.list_attr = list_attr;
                    let v_align = ((list_attr >> 5) & 0x03) as u8;
                    text_box.vertical_align = match v_align {
                        1 => crate::model::table::VerticalAlign::Center,
                        2 => crate::model::table::VerticalAlign::Bottom,
                        _ => crate::model::table::VerticalAlign::Top,
                    };
                }
                child_drawing.text_box = Some(text_box);
            }
        }

        // 중첩 Group 감지: shape_tag_id가 없고 하위 SHAPE_COMPONENT가 있으면 재귀
        let has_nested_shapes = shape_tag_id.is_none() && child_slice.len() > 1
            && child_slice[1..].iter().any(|r|
                r.tag_id == tags::HWPTAG_SHAPE_COMPONENT && r.level > child_slice[0].level);
        // CONTAINER 태그가 있거나 하위 SHAPE_COMPONENT가 있으면 중첩 Group
        let has_container_tag = child_slice[1..].iter()
            .any(|r| r.tag_id == tags::HWPTAG_SHAPE_COMPONENT_CONTAINER);
        if has_container_tag || has_nested_shapes {
            let mut group = GroupShape::default();
            group.shape_attr = child_drawing.shape_attr.clone();
            group.children = parse_container_children(child_slice);
            children.push(ShapeObject::Group(group));
            continue;
        }

        // 도형 생성
        let shape = match shape_tag_id {
            Some(tags::HWPTAG_SHAPE_COMPONENT_LINE) => {
                let mut line = LineShape::default();
                let is_connector = child_drawing.shape_attr.ctrl_id == tags::SHAPE_CONNECTOR_ID;
                line.drawing = child_drawing;
                parse_line_shape_data(shape_tag_data, &mut line, is_connector);
                ShapeObject::Line(line)
            }
            Some(tags::HWPTAG_SHAPE_COMPONENT_RECTANGLE) => {
                let mut rect = RectangleShape::default();
                rect.drawing = child_drawing;
                parse_rect_shape_data(shape_tag_data, &mut rect);
                ShapeObject::Rectangle(rect)
            }
            Some(tags::HWPTAG_SHAPE_COMPONENT_ELLIPSE) => {
                let mut ellipse = EllipseShape::default();
                ellipse.drawing = child_drawing;
                parse_ellipse_shape_data(shape_tag_data, &mut ellipse);
                ShapeObject::Ellipse(ellipse)
            }
            Some(tags::HWPTAG_SHAPE_COMPONENT_ARC) => {
                let mut arc = ArcShape::default();
                arc.drawing = child_drawing;
                parse_arc_shape_data(shape_tag_data, &mut arc);
                ShapeObject::Arc(arc)
            }
            Some(tags::HWPTAG_SHAPE_COMPONENT_POLYGON) => {
                let mut poly = PolygonShape::default();
                poly.drawing = child_drawing;
                parse_polygon_shape_data(shape_tag_data, &mut poly);
                ShapeObject::Polygon(poly)
            }
            Some(tags::HWPTAG_SHAPE_COMPONENT_CURVE) => {
                let mut curve = CurveShape::default();
                curve.drawing = child_drawing;
                parse_curve_shape_data(shape_tag_data, &mut curve);
                ShapeObject::Curve(curve)
            }
            Some(tags::HWPTAG_SHAPE_COMPONENT_PICTURE) => {
                let picture = parse_picture(
                    CommonObjAttr::default(),
                    child_drawing.shape_attr.clone(),
                    shape_tag_data,
                );
                ShapeObject::Picture(Box::new(picture))
            }
            _ => {
                let mut rect = RectangleShape::default();
                rect.drawing = child_drawing;
                ShapeObject::Rectangle(rect)
            }
        };

        children.push(shape);
    }

    children
}

/// 그림 개체 파싱
fn parse_picture(
    common: CommonObjAttr,
    shape_attr: ShapeComponentAttr,
    data: &[u8],
) -> Picture {
    let mut pic = Picture::default();
    pic.common = common;
    pic.shape_attr = shape_attr;

    let mut r = ByteReader::new(data);

    pic.border_color = r.read_color_ref().unwrap_or(0);
    pic.border_width = r.read_i32().unwrap_or(0);

    // 테두리 속성 (attr u32, 표 87 참조)
    let border_attr_raw = r.read_u32().unwrap_or(0);
    pic.border_attr = ShapeBorderLine {
        color: pic.border_color,
        width: pic.border_width,
        attr: border_attr_raw,
        outline_style: 0,
    };

    // 꼭짓점 좌표 (4개씩)
    for i in 0..4 {
        pic.border_x[i] = r.read_i32().unwrap_or(0);
    }
    for i in 0..4 {
        pic.border_y[i] = r.read_i32().unwrap_or(0);
    }

    // 자르기 정보
    pic.crop.left = r.read_i32().unwrap_or(0);
    pic.crop.top = r.read_i32().unwrap_or(0);
    pic.crop.right = r.read_i32().unwrap_or(0);
    pic.crop.bottom = r.read_i32().unwrap_or(0);

    // 안쪽 여백
    pic.padding = Padding {
        left: r.read_i16().unwrap_or(0),
        right: r.read_i16().unwrap_or(0),
        top: r.read_i16().unwrap_or(0),
        bottom: r.read_i16().unwrap_or(0),
    };

    // 이미지 속성
    pic.image_attr.brightness = r.read_i8().unwrap_or(0);
    pic.image_attr.contrast = r.read_i8().unwrap_or(0);
    let effect = r.read_u8().unwrap_or(0);
    pic.image_attr.effect = match effect {
        1 => ImageEffect::GrayScale,
        2 => ImageEffect::BlackWhite,
        3 => ImageEffect::Pattern8x8,
        _ => ImageEffect::RealPic,
    };
    pic.image_attr.bin_data_id = r.read_u16().unwrap_or(0);

    // 남은 바이트 보존 (라운드트립용)
    if r.remaining() > 0 {
        pic.raw_picture_extra = r.read_bytes(r.remaining()).unwrap_or_default().to_vec();
    }

    pic
}

/// 직선 도형 데이터 파싱
fn parse_line_shape_data(data: &[u8], line: &mut LineShape, is_connector: bool) {
    use crate::model::shape::{ConnectorData, ConnectorControlPoint, LinkLineType};
    let mut r = ByteReader::new(data);
    line.start.x = r.read_i32().unwrap_or(0);
    line.start.y = r.read_i32().unwrap_or(0);
    line.end.x = r.read_i32().unwrap_or(0);
    line.end.y = r.read_i32().unwrap_or(0);

    if is_connector {
        // 연결선: type(u32) + ssid(u32) + ssidx(u32) + esid(u32) + esidx(u32) + countCP(u32) + CPs + trailing
        let lt = r.read_u32().unwrap_or(0);
        let link_type = LinkLineType::from_u32(lt);
        let start_subject_id = r.read_u32().unwrap_or(0);
        let start_subject_index = r.read_u32().unwrap_or(0);
        let end_subject_id = r.read_u32().unwrap_or(0);
        let end_subject_index = r.read_u32().unwrap_or(0);
        let count = r.read_u32().unwrap_or(0) as usize;
        let mut control_points = Vec::with_capacity(count);
        for _ in 0..count {
            let x = r.read_i32().unwrap_or(0);
            let y = r.read_i32().unwrap_or(0);
            let point_type = r.read_u16().unwrap_or(0);
            control_points.push(ConnectorControlPoint { x, y, point_type });
        }
        // 나머지 바이트 보존 (패딩 등)
        let raw_trailing = if r.remaining() > 0 {
            r.read_bytes(r.remaining()).unwrap_or_default().to_vec()
        } else {
            Vec::new()
        };
        line.connector = Some(ConnectorData {
            link_type,
            start_subject_id,
            start_subject_index,
            end_subject_id,
            end_subject_index,
            control_points,
            raw_trailing,
        });
    } else {
        // 일반 선
        line.started_right_or_bottom = r.read_i32().unwrap_or(0) != 0;
    }
}

/// 사각형 도형 데이터 파싱
/// hwplib: 좌표가 (x1,y1),(x2,y2),(x3,y3),(x4,y4) 인터리브 순서로 저장
fn parse_rect_shape_data(data: &[u8], rect: &mut RectangleShape) {
    let mut r = ByteReader::new(data);
    rect.round_rate = r.read_u8().unwrap_or(0);
    for i in 0..4 {
        rect.x_coords[i] = r.read_i32().unwrap_or(0);
        rect.y_coords[i] = r.read_i32().unwrap_or(0);
    }
}

/// 타원 도형 데이터 파싱 (60바이트)
fn parse_ellipse_shape_data(data: &[u8], ellipse: &mut EllipseShape) {
    let mut r = ByteReader::new(data);
    ellipse.attr = r.read_u32().unwrap_or(0);
    ellipse.center.x = r.read_i32().unwrap_or(0);
    ellipse.center.y = r.read_i32().unwrap_or(0);
    ellipse.axis1.x = r.read_i32().unwrap_or(0);
    ellipse.axis1.y = r.read_i32().unwrap_or(0);
    ellipse.axis2.x = r.read_i32().unwrap_or(0);
    ellipse.axis2.y = r.read_i32().unwrap_or(0);
    ellipse.start1.x = r.read_i32().unwrap_or(0);
    ellipse.start1.y = r.read_i32().unwrap_or(0);
    ellipse.end1.x = r.read_i32().unwrap_or(0);
    ellipse.end1.y = r.read_i32().unwrap_or(0);
    ellipse.start2.x = r.read_i32().unwrap_or(0);
    ellipse.start2.y = r.read_i32().unwrap_or(0);
    ellipse.end2.x = r.read_i32().unwrap_or(0);
    ellipse.end2.y = r.read_i32().unwrap_or(0);
}

/// 호 도형 데이터 파싱 (hwplib: UINT8 arcType + 6×INT32 좌표 = 25바이트)
fn parse_arc_shape_data(data: &[u8], arc: &mut ArcShape) {
    let mut r = ByteReader::new(data);
    arc.arc_type = r.read_u8().unwrap_or(0);
    arc.center.x = r.read_i32().unwrap_or(0);
    arc.center.y = r.read_i32().unwrap_or(0);
    arc.axis1.x = r.read_i32().unwrap_or(0);
    arc.axis1.y = r.read_i32().unwrap_or(0);
    arc.axis2.x = r.read_i32().unwrap_or(0);
    arc.axis2.y = r.read_i32().unwrap_or(0);
}

/// 다각형 도형 데이터 파싱
/// hwplib: INT32 count + (INT32 x, INT32 y) × count (plain HWPUNIT)
fn parse_polygon_shape_data(data: &[u8], poly: &mut PolygonShape) {
    let mut r = ByteReader::new(data);
    let cnt = r.read_i32().unwrap_or(0) as usize;
    poly.points.clear();
    for _ in 0..cnt {
        let x = r.read_i32().unwrap_or(0);
        let y = r.read_i32().unwrap_or(0);
        poly.points.push(Point { x, y });
    }
}

/// 곡선 도형 데이터 파싱
/// hwplib: INT32 count + (INT32 x, INT32 y) × count + BYTE[count-1] segment_types + skip(4)
fn parse_curve_shape_data(data: &[u8], curve: &mut CurveShape) {
    let mut r = ByteReader::new(data);
    let cnt = r.read_i32().unwrap_or(0) as usize;
    curve.points.clear();
    for _ in 0..cnt {
        let x = r.read_i32().unwrap_or(0);
        let y = r.read_i32().unwrap_or(0);
        curve.points.push(Point { x, y });
    }
    // 세그먼트 타입 (cnt-1개, 0=line, 1=curve)
    curve.segment_types.clear();
    if cnt > 0 {
        for _ in 0..(cnt - 1) {
            curve.segment_types.push(r.read_u8().unwrap_or(0));
        }
    }
    // hwplib: sr.skip(4) — 4바이트 패딩
    let _ = r.read_u32();
}

// ============================================================
// Task #195: 단위 테스트 (OLE/Chart 파싱)
// ============================================================

#[cfg(test)]
mod task195_tests {
    use super::*;

    #[test]
    fn test_parse_ole_shape_minimal() {
        // 1.hwp 레이아웃 실측 기반: property(4) + extent_x(4) + extent_y(4) + bin_data_id(4)
        let mut data = Vec::new();
        data.extend_from_slice(&1u32.to_le_bytes());       // property
        data.extend_from_slice(&1000i32.to_le_bytes());    // extent_x
        data.extend_from_slice(&2000i32.to_le_bytes());    // extent_y
        data.extend_from_slice(&5u32.to_le_bytes());       // bin_data_id
        data.extend_from_slice(&[0u8; 14]);                // padding

        let ole = parse_ole_shape(CommonObjAttr::default(), DrawingObjAttr::default(), &data);
        assert_eq!(ole.extent_x, 1000);
        assert_eq!(ole.extent_y, 2000);
        assert_eq!(ole.bin_data_id, 5);
        assert_eq!(ole.raw_tag_data.len(), data.len());
    }

    #[test]
    fn test_parse_ole_shape_bin_id_42() {
        let mut data = Vec::new();
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&0i32.to_le_bytes());
        data.extend_from_slice(&42u32.to_le_bytes());
        let ole = parse_ole_shape(CommonObjAttr::default(), DrawingObjAttr::default(), &data);
        assert_eq!(ole.bin_data_id, 42);
    }

    #[test]
    fn test_parse_ole_shape_truncated_graceful() {
        // 4바이트만 — 나머지 필드는 기본값으로 채워져야 함
        let data = [0x10, 0x00, 0x00, 0x00]; // property=16
        let ole = parse_ole_shape(CommonObjAttr::default(), DrawingObjAttr::default(), &data);
        assert_eq!(ole.extent_x, 0);
        assert_eq!(ole.extent_y, 0);
        assert_eq!(ole.bin_data_id, 0);
        assert_eq!(ole.raw_tag_data.len(), 4);
    }
}
