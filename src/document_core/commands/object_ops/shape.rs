//! 도형 생성/속성/그룹 native 명령 (object_ops 분할, #1904).

use super::MIN_SHAPE_SIZE;
use crate::document_core::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{
    common_obj_offsets, Caption, CaptionDirection, CaptionVertAlign, ShapeObject,
};

impl DocumentCore {
    fn shape_caption_ref(shape: &ShapeObject) -> Option<&Caption> {
        match shape {
            ShapeObject::Line(s) => s.drawing.caption.as_ref(),
            ShapeObject::Rectangle(s) => s.drawing.caption.as_ref(),
            ShapeObject::Ellipse(s) => s.drawing.caption.as_ref(),
            ShapeObject::Arc(s) => s.drawing.caption.as_ref(),
            ShapeObject::Polygon(s) => s.drawing.caption.as_ref(),
            ShapeObject::Curve(s) => s.drawing.caption.as_ref(),
            ShapeObject::Group(s) => s.caption.as_ref(),
            ShapeObject::Picture(s) => s.caption.as_ref(),
            ShapeObject::Chart(s) => s.caption.as_ref(),
            ShapeObject::Ole(s) => s.caption.as_ref(),
        }
    }

    fn shape_caption_mut(shape: &mut ShapeObject) -> &mut Option<Caption> {
        match shape {
            ShapeObject::Line(s) => &mut s.drawing.caption,
            ShapeObject::Rectangle(s) => &mut s.drawing.caption,
            ShapeObject::Ellipse(s) => &mut s.drawing.caption,
            ShapeObject::Arc(s) => &mut s.drawing.caption,
            ShapeObject::Polygon(s) => &mut s.drawing.caption,
            ShapeObject::Curve(s) => &mut s.drawing.caption,
            ShapeObject::Group(s) => &mut s.caption,
            ShapeObject::Picture(s) => &mut s.caption,
            ShapeObject::Chart(s) => &mut s.caption,
            ShapeObject::Ole(s) => &mut s.caption,
        }
    }

    fn clear_shape_caption(shape: &mut ShapeObject) -> bool {
        let had_caption = Self::shape_caption_ref(shape).is_some()
            || shape
                .drawing()
                .is_some_and(|drawing| drawing.caption.is_some());
        match shape {
            ShapeObject::Line(s) => s.drawing.caption = None,
            ShapeObject::Rectangle(s) => s.drawing.caption = None,
            ShapeObject::Ellipse(s) => s.drawing.caption = None,
            ShapeObject::Arc(s) => s.drawing.caption = None,
            ShapeObject::Polygon(s) => s.drawing.caption = None,
            ShapeObject::Curve(s) => s.drawing.caption = None,
            ShapeObject::Group(s) => s.caption = None,
            ShapeObject::Picture(s) => s.caption = None,
            ShapeObject::Chart(s) => {
                s.caption = None;
                s.drawing.caption = None;
            }
            ShapeObject::Ole(s) => {
                s.caption = None;
                s.drawing.caption = None;
            }
        }
        if had_caption {
            shape.common_mut().attr &= !(1 << 29);
        }
        had_caption
    }

    fn format_shape_caption_props_json(shape: &ShapeObject) -> String {
        let caption = Self::shape_caption_ref(shape);
        format!(
            ",\"hasCaption\":{},\"captionDirection\":\"{}\",\"captionVertAlign\":\"{}\",\"captionWidth\":{},\"captionSpacing\":{},\"captionMaxWidth\":{},\"captionIncludeMargin\":{}",
            caption.is_some(),
            caption.map_or("Bottom", |cap| match cap.direction {
                CaptionDirection::Left => "Left",
                CaptionDirection::Right => "Right",
                CaptionDirection::Top => "Top",
                CaptionDirection::Bottom => "Bottom",
            }),
            caption.map_or("Top", |cap| match cap.vert_align {
                CaptionVertAlign::Top => "Top",
                CaptionVertAlign::Center => "Center",
                CaptionVertAlign::Bottom => "Bottom",
            }),
            caption.map_or(0u32, |cap| cap.width),
            caption.map_or(0i16, |cap| cap.spacing),
            caption.map_or(0u32, |cap| cap.max_width),
            caption.map_or(false, |cap| cap.include_margin),
        )
    }

    fn apply_shape_caption_props(shape: &mut ShapeObject, props_json: &str) -> bool {
        use crate::document_core::helpers::{json_bool, json_i16, json_str, json_u32};

        let Some(has_caption) = json_bool(props_json, "hasCaption") else {
            return false;
        };
        if !has_caption {
            return Self::clear_shape_caption(shape);
        }

        let default_max_width = shape.common().width;
        let mut created = false;
        {
            let caption_slot = Self::shape_caption_mut(shape);
            if caption_slot.is_none() {
                let mut caption = Caption {
                    max_width: default_max_width,
                    ..Default::default()
                };
                let auto_number = crate::model::control::AutoNumber {
                    number_type: crate::model::control::AutoNumberType::Picture,
                    suffix_char: '.',
                    ..Default::default()
                };
                let mut para = Paragraph::new_empty();
                // 한컴 그림/OLE 캡션은 AutoNumber 앞에 "그림" 접두어를 함께 표시한다.
                para.text = "그림  ".to_string();
                para.char_count = 13;
                para.char_count_msb = true;
                para.control_mask = 1u32 << 0x12;
                para.char_offsets = vec![0, 1, 2, 11];
                para.controls
                    .push(crate::model::control::Control::AutoNumber(auto_number));
                para.ctrl_data_records.push(None);
                caption.paragraphs.push(para);
                *caption_slot = Some(caption);
                created = true;
            }

            if let Some(caption) = caption_slot.as_mut() {
                if let Some(v) = json_str(props_json, "captionDirection") {
                    caption.direction = match v.as_str() {
                        "Left" => CaptionDirection::Left,
                        "Right" => CaptionDirection::Right,
                        "Top" => CaptionDirection::Top,
                        _ => CaptionDirection::Bottom,
                    };
                }
                if let Some(v) = json_str(props_json, "captionVertAlign") {
                    caption.vert_align = match v.as_str() {
                        "Center" => CaptionVertAlign::Center,
                        "Bottom" => CaptionVertAlign::Bottom,
                        _ => CaptionVertAlign::Top,
                    };
                }
                if let Some(v) = json_u32(props_json, "captionWidth") {
                    caption.width = v;
                }
                if let Some(v) = json_i16(props_json, "captionSpacing") {
                    caption.spacing = v;
                }
                if let Some(v) = json_bool(props_json, "captionIncludeMargin") {
                    caption.include_margin = v;
                }
            }
        }

        if created {
            shape.common_mut().attr |= 1 << 29;
        }
        created
    }

    fn resolve_shape_control_ref(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<&ShapeObject, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;

        let body_len = section.paragraphs.len();
        let para = if parent_para_idx < body_len {
            section.paragraphs.get(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?
        } else {
            let mut virtual_idx = parent_para_idx - body_len;
            let mut found = None;
            'outer: for body_para in &section.paragraphs {
                for ctrl in &body_para.controls {
                    if let Control::Endnote(en) = ctrl {
                        if virtual_idx < en.paragraphs.len() {
                            found = en.paragraphs.get(virtual_idx);
                            break 'outer;
                        }
                        virtual_idx -= en.paragraphs.len();
                    }
                }
            }
            found.ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?
        };

        let ctrl = para.controls.get(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;
        match ctrl {
            Control::Shape(s) => Ok(s.as_ref()),
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 Shape이 아닙니다".to_string(),
            )),
        }
    }
    fn resolve_shape_control_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<&mut ShapeObject, HwpError> {
        let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;

        let body_len = section.paragraphs.len();
        let para = if parent_para_idx < body_len {
            section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?
        } else {
            let mut virtual_idx = parent_para_idx - body_len;
            let mut found = None;
            'outer: for body_para in &mut section.paragraphs {
                for ctrl in &mut body_para.controls {
                    if let Control::Endnote(en) = ctrl {
                        if virtual_idx < en.paragraphs.len() {
                            found = en.paragraphs.get_mut(virtual_idx);
                            break 'outer;
                        }
                        virtual_idx -= en.paragraphs.len();
                    }
                }
            }
            found.ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?
        };

        let ctrl = para.controls.get_mut(control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
        })?;
        match ctrl {
            Control::Shape(s) => Ok(s.as_mut()),
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 Shape이 아닙니다".to_string(),
            )),
        }
    }
    /// 글상자(Shape) 속성 조회 (네이티브).
    pub fn get_shape_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let shape = self.resolve_shape_control_ref(section_idx, parent_para_idx, control_idx)?;

        let c = shape.common();
        let common_json = Self::common_obj_attr_to_json(c);

        // TextBox 속성
        let tb_json = if let Some(tb) = get_textbox_from_shape(shape) {
            let va = match tb.vertical_align {
                crate::model::table::VerticalAlign::Top => "Top",
                crate::model::table::VerticalAlign::Center => "Center",
                crate::model::table::VerticalAlign::Bottom => "Bottom",
            };
            format!(
                ",\"tbMarginLeft\":{},\"tbMarginRight\":{},\"tbMarginTop\":{},\"tbMarginBottom\":{},\"tbVerticalAlign\":\"{}\"",
                tb.margin_left, tb.margin_right, tb.margin_top, tb.margin_bottom, va
            )
        } else {
            String::new()
        };

        // 테두리 / 회전 / 채우기 정보
        let drawing = shape.drawing();
        let extra_json = if let Some(d) = drawing {
            let sa = &d.shape_attr;
            let fill = &d.fill;
            let fill_type = match fill.fill_type {
                crate::model::style::FillType::None => "none",
                crate::model::style::FillType::Solid => "solid",
                crate::model::style::FillType::Gradient => "gradient",
                crate::model::style::FillType::Image => "image",
            };
            // borderAttr 비트필드 분해
            let bl = &d.border_line;
            let line_type = bl.attr & 0x3F; // bits 0-5: 선 종류 (0~17)
            let line_end_shape = (bl.attr >> 6) & 0x0F; // bits 6-9: 끝 모양
            let arrow_start = (bl.attr >> 10) & 0x3F; // bits 10-15: 화살표 시작 모양
            let arrow_end = (bl.attr >> 16) & 0x3F; // bits 16-21: 화살표 끝 모양
            let arrow_start_size = (bl.attr >> 22) & 0x0F; // bits 22-25: 화살표 시작 크기
            let arrow_end_size = (bl.attr >> 26) & 0x0F; // bits 26-29: 화살표 끝 크기

            let mut extra = format!(
                ",\"borderColor\":{},\"borderWidth\":{},\"borderAttr\":{},\"borderOutlineStyle\":{}\
                ,\"lineType\":{},\"lineEndShape\":{}\
                ,\"arrowStart\":{},\"arrowEnd\":{},\"arrowStartSize\":{},\"arrowEndSize\":{}\
                ,\"rotationAngle\":{},\"horzFlip\":{},\"vertFlip\":{}\
                ,\"fillType\":\"{}\"",
                bl.color, bl.width, bl.attr, bl.outline_style,
                line_type, line_end_shape,
                arrow_start, arrow_end, arrow_start_size, arrow_end_size,
                sa.rotation_angle, sa.horz_flip, sa.vert_flip,
                fill_type
            );
            // 단색 채우기
            if let Some(ref s) = fill.solid {
                extra.push_str(&format!(
                    ",\"fillBgColor\":{},\"fillPatColor\":{},\"fillPatType\":{}",
                    s.background_color, s.pattern_color, s.pattern_type
                ));
            }
            // 그러데이션 채우기
            if let Some(ref g) = fill.gradient {
                extra.push_str(&format!(
                    ",\"gradientType\":{},\"gradientAngle\":{},\"gradientCenterX\":{},\"gradientCenterY\":{},\"gradientBlur\":{}",
                    g.gradient_type, g.angle, g.center_x, g.center_y, g.blur
                ));
            }
            extra.push_str(&format!(",\"fillAlpha\":{}", fill.alpha));
            // 그림자
            extra.push_str(&format!(",\"shadowType\":{},\"shadowColor\":{},\"shadowOffsetX\":{},\"shadowOffsetY\":{},\"shadowAlpha\":{}",
                d.shadow_type, d.shadow_color, d.shadow_offset_x, d.shadow_offset_y, d.shadow_alpha));
            extra.push_str(&format!(",\"scInstId\":{}", d.inst_id));
            extra
        } else {
            String::new()
        };

        // Rectangle 전용: 모서리 곡률
        let round_json = if let crate::model::shape::ShapeObject::Rectangle(ref rect) = shape {
            format!(",\"roundRate\":{}", rect.round_rate)
        } else {
            String::new()
        };

        // 연결선 타입 + 제어점 좌표 (꺽임/곡선 중간 마커용)
        let connector_json = if let crate::model::shape::ShapeObject::Line(ref line) = shape {
            if let Some(ref conn) = line.connector {
                // type=2 제어점의 평균 좌표 (꺽임 모서리 / 곡선 중간점)
                let ctrl2_pts: Vec<&crate::model::shape::ConnectorControlPoint> = conn
                    .control_points
                    .iter()
                    .filter(|cp| cp.point_type == 2)
                    .collect();
                if !ctrl2_pts.is_empty() {
                    let avg_x: i32 =
                        ctrl2_pts.iter().map(|p| p.x).sum::<i32>() / ctrl2_pts.len() as i32;
                    let avg_y: i32 =
                        ctrl2_pts.iter().map(|p| p.y).sum::<i32>() / ctrl2_pts.len() as i32;
                    format!(
                        ",\"connectorType\":{},\"connectorMidX\":{},\"connectorMidY\":{}",
                        conn.link_type as u32, avg_x, avg_y
                    )
                } else {
                    format!(",\"connectorType\":{}", conn.link_type as u32)
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let caption_json = Self::format_shape_caption_props_json(shape);
        Ok(format!(
            "{{{}{}{}{}{}{}}}",
            common_json, tb_json, extra_json, round_json, connector_json, caption_json
        ))
    }
    /// 글상자(Shape) 속성 변경 (네이티브).
    pub fn set_shape_properties_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        use crate::document_core::helpers::{json_bool, json_i32, json_str};

        let shape = self.resolve_shape_control_mut(section_idx, parent_para_idx, control_idx)?;

        // CommonObjAttr 업데이트
        // 리사이즈 핸들을 반대편으로 끌어당길 때 studio가 width/height=0 을 보내
        // 도형이 렌더러상 사라지는 버그 방어: 최소 크기 clamp.
        let c = shape.common_mut();
        let new_w = crate::document_core::helpers::json_u32(props_json, "width")
            .map(|w| w.max(MIN_SHAPE_SIZE));
        let new_h = crate::document_core::helpers::json_u32(props_json, "height")
            .map(|h| h.max(MIN_SHAPE_SIZE));
        Self::apply_common_obj_attr_from_json(c, props_json);

        // Polygon/Curve: original_width/height는 생성 시 값으로 유지해야 렌더러의
        // 스케일 팩터(sx = current/original)가 올바르게 동작한다.
        let is_polygon_or_curve = matches!(
            shape,
            crate::model::shape::ShapeObject::Polygon(_)
                | crate::model::shape::ShapeObject::Curve(_)
        );
        let saved_orig_w = if is_polygon_or_curve {
            shape.drawing().map(|d| d.shape_attr.original_width)
        } else {
            None
        };
        let saved_orig_h = if is_polygon_or_curve {
            shape.drawing().map(|d| d.shape_attr.original_height)
        } else {
            None
        };

        // ShapeComponentAttr 크기/회전/채우기 동기화
        if let Some(d) = shape.drawing_mut() {
            if let Some(w) = new_w {
                d.shape_attr.current_width = w;
                d.shape_attr.original_width = w;
            }
            if let Some(h) = new_h {
                d.shape_attr.current_height = h;
                d.shape_attr.original_height = h;
            }

            // 회전/기울임
            if let Some(v) = json_i32(props_json, "rotationAngle") {
                d.shape_attr.rotation_angle = v as i16;
            }
            // 대칭(flip)
            if let Some(v) = json_bool(props_json, "horzFlip") {
                d.shape_attr.horz_flip = v;
                if v {
                    d.shape_attr.flip |= 1;
                } else {
                    d.shape_attr.flip &= !1;
                }
            }
            if let Some(v) = json_bool(props_json, "vertFlip") {
                d.shape_attr.vert_flip = v;
                if v {
                    d.shape_attr.flip |= 2;
                } else {
                    d.shape_attr.flip &= !2;
                }
            }

            // 테두리 선 — 색상/굵기
            if let Some(v) = json_i32(props_json, "borderColor") {
                d.border_line.color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "borderWidth") {
                d.border_line.width = v;
            }

            // 테두리 선 — attr 비트필드 개별 필드 업데이트
            {
                let mut attr = d.border_line.attr;
                if let Some(v) = json_i32(props_json, "lineType") {
                    attr = (attr & !0x3F) | ((v as u32) & 0x3F);
                }
                if let Some(v) = json_i32(props_json, "lineEndShape") {
                    attr = (attr & !(0x0F << 6)) | (((v as u32) & 0x0F) << 6);
                }
                if let Some(v) = json_i32(props_json, "arrowStart") {
                    attr = (attr & !(0x3F << 10)) | (((v as u32) & 0x3F) << 10);
                }
                if let Some(v) = json_i32(props_json, "arrowEnd") {
                    attr = (attr & !(0x3F << 16)) | (((v as u32) & 0x3F) << 16);
                }
                if let Some(v) = json_i32(props_json, "arrowStartSize") {
                    attr = (attr & !(0x0F << 22)) | (((v as u32) & 0x0F) << 22);
                }
                if let Some(v) = json_i32(props_json, "arrowEndSize") {
                    attr = (attr & !(0x0F << 26)) | (((v as u32) & 0x0F) << 26);
                }
                d.border_line.attr = attr;
            }

            // 채우기 (단색)
            if let Some(v) = json_str(props_json, "fillType") {
                d.fill.fill_type = match v.as_str() {
                    "solid" => crate::model::style::FillType::Solid,
                    "gradient" => crate::model::style::FillType::Gradient,
                    "image" => crate::model::style::FillType::Image,
                    _ => crate::model::style::FillType::None,
                };
            }
            if let Some(v) = json_i32(props_json, "fillBgColor") {
                let solid = d.fill.solid.get_or_insert_with(|| {
                    crate::model::style::SolidFill {
                        pattern_type: -1, // -1 = 단색 채우기 (0은 채우기 없음)
                        ..Default::default()
                    }
                });
                solid.background_color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "fillPatColor") {
                let solid = d
                    .fill
                    .solid
                    .get_or_insert_with(|| crate::model::style::SolidFill {
                        pattern_type: -1,
                        ..Default::default()
                    });
                solid.pattern_color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "fillPatType") {
                let solid = d
                    .fill
                    .solid
                    .get_or_insert_with(|| crate::model::style::SolidFill {
                        pattern_type: -1,
                        ..Default::default()
                    });
                solid.pattern_type = v;
            }
            if let Some(v) = json_i32(props_json, "fillAlpha") {
                d.fill.alpha = v as u8;
            }

            // 채우기 (그라디언트)
            if let Some(v) = json_i32(props_json, "gradientType") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.gradient_type = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientAngle") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.angle = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientCenterX") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.center_x = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientCenterY") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.center_y = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientBlur") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.blur = v as i16;
            }

            // 그림자
            if let Some(v) = crate::document_core::helpers::json_u32(props_json, "shadowType") {
                d.shadow_type = v;
            }
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "shadowColor") {
                d.shadow_color = v as u32;
            }
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "shadowOffsetX") {
                d.shadow_offset_x = v;
            }
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "shadowOffsetY") {
                d.shadow_offset_y = v;
            }

            // TextBox 속성 업데이트
            if let Some(ref mut tb) = d.text_box {
                if let Some(v) = json_i32(props_json, "tbMarginLeft") {
                    tb.margin_left = v as i16;
                }
                if let Some(v) = json_i32(props_json, "tbMarginRight") {
                    tb.margin_right = v as i16;
                }
                if let Some(v) = json_i32(props_json, "tbMarginTop") {
                    tb.margin_top = v as i16;
                }
                if let Some(v) = json_i32(props_json, "tbMarginBottom") {
                    tb.margin_bottom = v as i16;
                }
                if let Some(v) = json_str(props_json, "tbVerticalAlign") {
                    tb.vertical_align = match v.as_str() {
                        "Top" => crate::model::table::VerticalAlign::Top,
                        "Center" => crate::model::table::VerticalAlign::Center,
                        "Bottom" => crate::model::table::VerticalAlign::Bottom,
                        _ => tb.vertical_align,
                    };
                }
            }
        }

        // Rectangle 곡률
        if let crate::model::shape::ShapeObject::Rectangle(ref mut rect) = shape {
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "roundRate") {
                rect.round_rate = v as u8;
            }
        }

        // Rectangle 좌표 동기화
        if let crate::model::shape::ShapeObject::Rectangle(ref mut rect) = shape {
            let w = rect.common.width as i32;
            let h = rect.common.height as i32;
            rect.x_coords = [0, w, w, 0];
            rect.y_coords = [0, 0, h, h];
        }

        let caption_changed = Self::apply_shape_caption_props(shape, props_json);

        // Polygon/Curve: original_width/height 복원 (생성 시 값 유지 → 렌더러 스케일 팩터 정상화)
        if let Some(d) = shape.drawing_mut() {
            if let Some(w) = saved_orig_w {
                d.shape_attr.original_width = w;
            }
            if let Some(h) = saved_orig_h {
                d.shape_attr.original_height = h;
            }
        }

        // Group 리사이즈: original_width 유지, current_width만 변경 (렌더러가 스케일 적용)
        // 한컴 방식: 자식은 변경하지 않고, 컨테이너의 current/original 비율로 스케일 결정
        if let crate::model::shape::ShapeObject::Group(ref mut group) = shape {
            if let Some(nw) = new_w {
                group.shape_attr.current_width = nw;
                // original_width는 유지 (스케일 기준)
            }
            if let Some(nh) = new_h {
                group.shape_attr.current_height = nh;
            }
            // 회전 중심 갱신
            group.shape_attr.rotation_center.x = (group.common.width / 2) as i32;
            group.shape_attr.rotation_center.y = (group.common.height / 2) as i32;
            // raw_rendering 초기화 → 직렬화 시 스케일 행렬 재생성
            group.shape_attr.raw_rendering = Vec::new();
        }

        if caption_changed {
            crate::parser::assign_auto_numbers(&mut self.document);
        }

        // 리플로우 + 렌더 트리 캐시 무효화
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureResized {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }
    /// [Task #1138] Shape 속성 → JSON. get_shape_properties_native +
    /// get_cell_shape_properties_by_path_native 공유.
    pub(crate) fn format_shape_props_inner(
        shape: &crate::model::shape::ShapeObject,
    ) -> Result<String, HwpError> {
        let c = shape.common();
        let common_json = Self::common_obj_attr_to_json(c);

        // TextBox 속성
        let tb_json = if let Some(tb) = get_textbox_from_shape(shape) {
            let va = match tb.vertical_align {
                crate::model::table::VerticalAlign::Top => "Top",
                crate::model::table::VerticalAlign::Center => "Center",
                crate::model::table::VerticalAlign::Bottom => "Bottom",
            };
            format!(
                ",\"tbMarginLeft\":{},\"tbMarginRight\":{},\"tbMarginTop\":{},\"tbMarginBottom\":{},\"tbVerticalAlign\":\"{}\"",
                tb.margin_left, tb.margin_right, tb.margin_top, tb.margin_bottom, va
            )
        } else {
            String::new()
        };

        // 테두리 / 회전 / 채우기 정보
        let drawing = shape.drawing();
        let extra_json = if let Some(d) = drawing {
            let sa = &d.shape_attr;
            let fill = &d.fill;
            let fill_type = match fill.fill_type {
                crate::model::style::FillType::None => "none",
                crate::model::style::FillType::Solid => "solid",
                crate::model::style::FillType::Gradient => "gradient",
                crate::model::style::FillType::Image => "image",
            };
            let bl = &d.border_line;
            let line_type = bl.attr & 0x3F;
            let line_end_shape = (bl.attr >> 6) & 0x0F;
            let arrow_start = (bl.attr >> 10) & 0x3F;
            let arrow_end = (bl.attr >> 16) & 0x3F;
            let arrow_start_size = (bl.attr >> 22) & 0x0F;
            let arrow_end_size = (bl.attr >> 26) & 0x0F;

            let mut extra = format!(
                ",\"borderColor\":{},\"borderWidth\":{},\"borderAttr\":{},\"borderOutlineStyle\":{}\
                ,\"lineType\":{},\"lineEndShape\":{}\
                ,\"arrowStart\":{},\"arrowEnd\":{},\"arrowStartSize\":{},\"arrowEndSize\":{}\
                ,\"rotationAngle\":{},\"horzFlip\":{},\"vertFlip\":{}\
                ,\"fillType\":\"{}\"",
                bl.color, bl.width, bl.attr, bl.outline_style,
                line_type, line_end_shape,
                arrow_start, arrow_end, arrow_start_size, arrow_end_size,
                sa.rotation_angle, sa.horz_flip, sa.vert_flip,
                fill_type
            );
            if let Some(ref s) = fill.solid {
                extra.push_str(&format!(
                    ",\"fillBgColor\":{},\"fillPatColor\":{},\"fillPatType\":{}",
                    s.background_color, s.pattern_color, s.pattern_type
                ));
            }
            if let Some(ref g) = fill.gradient {
                extra.push_str(&format!(
                    ",\"gradientType\":{},\"gradientAngle\":{},\"gradientCenterX\":{},\"gradientCenterY\":{},\"gradientBlur\":{}",
                    g.gradient_type, g.angle, g.center_x, g.center_y, g.blur
                ));
            }
            extra.push_str(&format!(",\"fillAlpha\":{}", fill.alpha));
            extra.push_str(&format!(",\"shadowType\":{},\"shadowColor\":{},\"shadowOffsetX\":{},\"shadowOffsetY\":{},\"shadowAlpha\":{}",
                d.shadow_type, d.shadow_color, d.shadow_offset_x, d.shadow_offset_y, d.shadow_alpha));
            extra.push_str(&format!(",\"scInstId\":{}", d.inst_id));
            extra
        } else {
            String::new()
        };

        let round_json = if let crate::model::shape::ShapeObject::Rectangle(ref rect) = shape {
            format!(",\"roundRate\":{}", rect.round_rate)
        } else {
            String::new()
        };

        let connector_json = if let crate::model::shape::ShapeObject::Line(ref line) = shape {
            if let Some(ref conn) = line.connector {
                let ctrl2_pts: Vec<&crate::model::shape::ConnectorControlPoint> = conn
                    .control_points
                    .iter()
                    .filter(|cp| cp.point_type == 2)
                    .collect();
                if !ctrl2_pts.is_empty() {
                    let avg_x: i32 =
                        ctrl2_pts.iter().map(|p| p.x).sum::<i32>() / ctrl2_pts.len() as i32;
                    let avg_y: i32 =
                        ctrl2_pts.iter().map(|p| p.y).sum::<i32>() / ctrl2_pts.len() as i32;
                    format!(
                        ",\"connectorType\":{},\"connectorMidX\":{},\"connectorMidY\":{}",
                        conn.link_type as u32, avg_x, avg_y
                    )
                } else {
                    format!(",\"connectorType\":{}", conn.link_type as u32)
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let caption_json = Self::format_shape_caption_props_json(shape);
        Ok(format!(
            "{{{}{}{}{}{}{}}}",
            common_json, tb_json, extra_json, round_json, connector_json, caption_json
        ))
    }
    /// [Task #1138] Shape 속성 JSON 적용 (mutation only). 후처리 (recompose /
    /// paginate / cache invalidate / event log) 는 호출자 책임.
    /// set_shape_properties_native + set_cell_shape_properties_by_path_native 공유.
    /// 반환: caption_changed (true면 호출자가 AutoNumber 후처리 필요).
    pub(crate) fn apply_shape_props_inner(
        shape: &mut crate::model::shape::ShapeObject,
        props_json: &str,
    ) -> bool {
        use crate::document_core::helpers::{json_bool, json_i32, json_str};

        let c = shape.common_mut();
        let new_w = crate::document_core::helpers::json_u32(props_json, "width")
            .map(|w| w.max(MIN_SHAPE_SIZE));
        let new_h = crate::document_core::helpers::json_u32(props_json, "height")
            .map(|h| h.max(MIN_SHAPE_SIZE));
        Self::apply_common_obj_attr_from_json(c, props_json);

        let is_polygon_or_curve = matches!(
            shape,
            crate::model::shape::ShapeObject::Polygon(_)
                | crate::model::shape::ShapeObject::Curve(_)
        );
        let saved_orig_w = if is_polygon_or_curve {
            shape.drawing().map(|d| d.shape_attr.original_width)
        } else {
            None
        };
        let saved_orig_h = if is_polygon_or_curve {
            shape.drawing().map(|d| d.shape_attr.original_height)
        } else {
            None
        };

        if let Some(d) = shape.drawing_mut() {
            if let Some(w) = new_w {
                d.shape_attr.current_width = w;
                d.shape_attr.original_width = w;
            }
            if let Some(h) = new_h {
                d.shape_attr.current_height = h;
                d.shape_attr.original_height = h;
            }
            if let Some(v) = json_i32(props_json, "rotationAngle") {
                d.shape_attr.rotation_angle = v as i16;
            }
            if let Some(v) = json_bool(props_json, "horzFlip") {
                d.shape_attr.horz_flip = v;
                if v {
                    d.shape_attr.flip |= 1;
                } else {
                    d.shape_attr.flip &= !1;
                }
            }
            if let Some(v) = json_bool(props_json, "vertFlip") {
                d.shape_attr.vert_flip = v;
                if v {
                    d.shape_attr.flip |= 2;
                } else {
                    d.shape_attr.flip &= !2;
                }
            }
            if let Some(v) = json_i32(props_json, "borderColor") {
                d.border_line.color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "borderWidth") {
                d.border_line.width = v;
            }
            {
                let mut attr = d.border_line.attr;
                if let Some(v) = json_i32(props_json, "lineType") {
                    attr = (attr & !0x3F) | ((v as u32) & 0x3F);
                }
                if let Some(v) = json_i32(props_json, "lineEndShape") {
                    attr = (attr & !(0x0F << 6)) | (((v as u32) & 0x0F) << 6);
                }
                if let Some(v) = json_i32(props_json, "arrowStart") {
                    attr = (attr & !(0x3F << 10)) | (((v as u32) & 0x3F) << 10);
                }
                if let Some(v) = json_i32(props_json, "arrowEnd") {
                    attr = (attr & !(0x3F << 16)) | (((v as u32) & 0x3F) << 16);
                }
                if let Some(v) = json_i32(props_json, "arrowStartSize") {
                    attr = (attr & !(0x0F << 22)) | (((v as u32) & 0x0F) << 22);
                }
                if let Some(v) = json_i32(props_json, "arrowEndSize") {
                    attr = (attr & !(0x0F << 26)) | (((v as u32) & 0x0F) << 26);
                }
                d.border_line.attr = attr;
            }
            if let Some(v) = json_str(props_json, "fillType") {
                d.fill.fill_type = match v.as_str() {
                    "solid" => crate::model::style::FillType::Solid,
                    "gradient" => crate::model::style::FillType::Gradient,
                    "image" => crate::model::style::FillType::Image,
                    _ => crate::model::style::FillType::None,
                };
            }
            if let Some(v) = json_i32(props_json, "fillBgColor") {
                let solid = d
                    .fill
                    .solid
                    .get_or_insert_with(|| crate::model::style::SolidFill {
                        pattern_type: -1,
                        ..Default::default()
                    });
                solid.background_color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "fillPatColor") {
                let solid = d
                    .fill
                    .solid
                    .get_or_insert_with(|| crate::model::style::SolidFill {
                        pattern_type: -1,
                        ..Default::default()
                    });
                solid.pattern_color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "fillPatType") {
                let solid = d
                    .fill
                    .solid
                    .get_or_insert_with(|| crate::model::style::SolidFill {
                        pattern_type: -1,
                        ..Default::default()
                    });
                solid.pattern_type = v;
            }
            if let Some(v) = json_i32(props_json, "fillAlpha") {
                d.fill.alpha = v as u8;
            }
            if let Some(v) = json_i32(props_json, "gradientType") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.gradient_type = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientAngle") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.angle = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientCenterX") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.center_x = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientCenterY") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.center_y = v as i16;
            }
            if let Some(v) = json_i32(props_json, "gradientBlur") {
                let grad = d.fill.gradient.get_or_insert_with(Default::default);
                grad.blur = v as i16;
            }
            if let Some(v) = crate::document_core::helpers::json_u32(props_json, "shadowType") {
                d.shadow_type = v;
            }
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "shadowColor") {
                d.shadow_color = v as u32;
            }
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "shadowOffsetX") {
                d.shadow_offset_x = v;
            }
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "shadowOffsetY") {
                d.shadow_offset_y = v;
            }
            if let Some(ref mut tb) = d.text_box {
                if let Some(v) = json_i32(props_json, "tbMarginLeft") {
                    tb.margin_left = v as i16;
                }
                if let Some(v) = json_i32(props_json, "tbMarginRight") {
                    tb.margin_right = v as i16;
                }
                if let Some(v) = json_i32(props_json, "tbMarginTop") {
                    tb.margin_top = v as i16;
                }
                if let Some(v) = json_i32(props_json, "tbMarginBottom") {
                    tb.margin_bottom = v as i16;
                }
                if let Some(v) = json_str(props_json, "tbVerticalAlign") {
                    tb.vertical_align = match v.as_str() {
                        "Top" => crate::model::table::VerticalAlign::Top,
                        "Center" => crate::model::table::VerticalAlign::Center,
                        "Bottom" => crate::model::table::VerticalAlign::Bottom,
                        _ => tb.vertical_align,
                    };
                }
            }
        }

        if let crate::model::shape::ShapeObject::Rectangle(ref mut rect) = shape {
            if let Some(v) = crate::document_core::helpers::json_i32(props_json, "roundRate") {
                rect.round_rate = v as u8;
            }
        }

        if let crate::model::shape::ShapeObject::Rectangle(ref mut rect) = shape {
            let w = rect.common.width as i32;
            let h = rect.common.height as i32;
            rect.x_coords = [0, w, w, 0];
            rect.y_coords = [0, 0, h, h];
        }

        let caption_changed = Self::apply_shape_caption_props(shape, props_json);

        if let Some(d) = shape.drawing_mut() {
            if let Some(w) = saved_orig_w {
                d.shape_attr.original_width = w;
            }
            if let Some(h) = saved_orig_h {
                d.shape_attr.original_height = h;
            }
        }

        if let crate::model::shape::ShapeObject::Group(ref mut group) = shape {
            if let Some(nw) = new_w {
                group.shape_attr.current_width = nw;
            }
            if let Some(nh) = new_h {
                group.shape_attr.current_height = nh;
            }
            group.shape_attr.rotation_center.x = (group.common.width / 2) as i32;
            group.shape_attr.rotation_center.y = (group.common.height / 2) as i32;
            group.shape_attr.raw_rendering = Vec::new();
        }
        caption_changed
    }
    /// 글상자(Shape) 삭제 (네이티브).
    ///
    /// delete_picture_control_native()와 동일한 패턴.
    pub fn delete_shape_control_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        if parent_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                parent_para_idx
            )));
        }
        let para = &mut section.paragraphs[parent_para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {} 범위 초과",
                control_idx
            )));
        }
        if !matches!(&para.controls[control_idx], Control::Shape(_)) {
            return Err(HwpError::RenderError(
                "지정된 컨트롤이 Shape이 아닙니다".to_string(),
            ));
        }

        // char_offsets 조정 (delete_picture_control_native와 동일)
        let text_chars: Vec<char> = para.text.chars().collect();
        let mut ci = 0usize;
        let mut prev_end: u32 = 0;
        let mut gap_start: Option<u32> = None;
        'outer: for i in 0..text_chars.len() {
            let offset = if i < para.char_offsets.len() {
                para.char_offsets[i]
            } else {
                prev_end
            };
            while prev_end + 8 <= offset && ci < para.controls.len() {
                if ci == control_idx {
                    gap_start = Some(prev_end);
                    break 'outer;
                }
                ci += 1;
                prev_end += 8;
            }
            let char_size: u32 = if text_chars[i] == '\t' {
                8
            } else if text_chars[i].len_utf16() == 2 {
                2
            } else {
                1
            };
            prev_end = offset + char_size;
        }
        if gap_start.is_none() {
            while ci < para.controls.len() {
                if ci == control_idx {
                    gap_start = Some(prev_end);
                    break;
                }
                ci += 1;
                prev_end += 8;
            }
        }
        if let Some(gs) = gap_start {
            let threshold = gs + 8;
            for offset in para.char_offsets.iter_mut() {
                if *offset >= threshold {
                    *offset -= 8;
                }
            }
        }

        para.controls.remove(control_idx);
        if control_idx < para.ctrl_data_records.len() {
            para.ctrl_data_records.remove(control_idx);
        }
        if para.char_count >= 8 {
            para.char_count -= 8;
        }

        // line_segs 재계산: 도형 높이가 반영된 line_segs를 텍스트 기반으로 리셋
        Self::reflow_paragraph_line_segs_after_control_delete(para, &self.styles, self.dpi);

        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureDeleted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }
    /// 커서 위치에 글상자(Rectangle + TextBox)를 삽입한다 (네이티브).
    pub fn create_shape_control_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        width: u32,
        height: u32,
        horz_offset: u32,
        vert_offset: u32,
        treat_as_char: bool,
        text_wrap_str: &str,
        shape_type: &str,
        line_flip_x: bool,
        line_flip_y: bool,
        polygon_points: &[crate::model::Point],
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::shape::*;
        use crate::model::style::{Fill, ShapeBorderLine};

        // 유효성 검사
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }
        if width == 0 && height == 0 {
            return Err(HwpError::RenderError(
                "폭과 높이가 모두 0입니다".to_string(),
            ));
        }

        let text_wrap = match text_wrap_str {
            "Square" => TextWrap::Square,
            "Tight" => TextWrap::Tight,
            "Through" => TextWrap::Through,
            "TopAndBottom" => TextWrap::TopAndBottom,
            "BehindText" => TextWrap::BehindText,
            "InFrontOfText" => TextWrap::InFrontOfText,
            _ => TextWrap::InFrontOfText,
        };

        // 커서 위치 문단의 속성 상속
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 편집 영역 폭
        let pd = &self.document.sections[section_idx].section_def.page_def;
        let content_width =
            (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32).max(7200) as u32;

        // attr 비트 계산
        // 도형(line/ellipse/rectangle) 및 floating 글상자: 한컴 기본값 0x046A4000
        //   Paper/Top/Paper/Left/InFrontOfText + 절대크기 + allow_overlap + bit26
        // inline 글상자(treat_as_char=true): Para/Top/Column/Left/Square = 0x0A0210
        // [Task #1280 v2] 삽입 글상자는 한컴 정답값 floating(treat_as_char=false)+글앞으로(InFrontOfText).
        //   권위 샘플 samples/textbox-under-image.hwp 실측: 글상자 배치=글앞으로/Paper/Paper/false.
        //   serializer(control.rs:1768)는 common.attr!=0 이면 그대로 직렬화하므로 attr 와 enum 필드를
        //   함께 정합시킨다. treat_as_char=true 인 inline 글상자는 #1280 본편 동작을 그대로 보존.
        let inline_textbox = shape_type == "textbox" && treat_as_char;
        let mut attr: u32 = if inline_textbox { 0x0A0210 } else { 0x046A4000 };
        if treat_as_char {
            attr |= 0x01;
        }

        // --- 빈 문단 (글상자 내부용) ---
        let tb_inner_width = width.saturating_sub(1020); // 양쪽 여백 510+510
        let mut inner_raw_header_extra = vec![0u8; 10];
        inner_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes());
        inner_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());
        let inner_para = Paragraph {
            text: String::new(),
            char_count: 1,
            char_count_msb: true,
            control_mask: 0,
            para_shape_id: default_para_shape_id,
            style_id: 0,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: tb_inner_width as i32,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            has_para_text: false,
            raw_header_extra: inner_raw_header_extra,
            ..Default::default()
        };

        // --- 도형 구조 조립 ---
        let w_i = width as i32;
        let h_i = height as i32;
        let new_z_order = self.max_shape_z_order_in_section(section_idx) + 1;

        // ctrl_id 결정
        let is_connector = shape_type.starts_with("connector-");
        let ctrl_id: u32 = match shape_type {
            "line"
            | "connector-straight"
            | "connector-stroke"
            | "connector-arc"
            | "connector-straight-arrow"
            | "connector-stroke-arrow"
            | "connector-arc-arrow" => {
                if is_connector {
                    0x24636f6c
                } else {
                    0x246c696e
                }
            } // '$col' or '$lin'
            "ellipse" => 0x24656c6c, // '$ell'
            "polygon" => 0x24706f6c, // '$pol'
            "arc" => 0x24617263,     // '$arc'
            _ => 0x24726563,         // '$rec' (rectangle, textbox)
        };

        // instance_id 생성: 고유 해시 (z_order 기반 + 위치/크기)
        let instance_id: u32 = {
            let mut h: u32 = 0x7de30000;
            h = h.wrapping_add(new_z_order as u32 * 0x100);
            h = h.wrapping_add(horz_offset.wrapping_mul(3));
            h = h.wrapping_add(vert_offset.wrapping_mul(7));
            h = h.wrapping_add(width);
            h = h.wrapping_add(height.wrapping_mul(0x1b));
            h |= 0x40000000; // bit30 설정 (한컴 호환)
            if h == 0 {
                h = 0x7de34b69;
            }
            h
        };

        let common = CommonObjAttr {
            ctrl_id,
            attr,
            vertical_offset: vert_offset,
            horizontal_offset: horz_offset,
            width,
            height,
            z_order: new_z_order,
            instance_id,
            margin: if shape_type == "textbox" {
                crate::model::Padding {
                    left: 283,
                    right: 283,
                    top: 283,
                    bottom: 283,
                }
            } else {
                crate::model::Padding {
                    left: 0,
                    right: 0,
                    top: 0,
                    bottom: 0,
                }
            },
            treat_as_char,
            // [Task #1280 v2] inline 글상자만 Para/Column(본문 기준), floating 글상자·도형은 Paper.
            vert_rel_to: if inline_textbox {
                VertRelTo::Para
            } else {
                VertRelTo::Paper
            },
            vert_align: VertAlign::Top,
            horz_rel_to: if inline_textbox {
                HorzRelTo::Column
            } else {
                HorzRelTo::Paper
            },
            horz_align: HorzAlign::Left,
            text_wrap,
            description: match shape_type {
                "line" => "선입니다.".to_string(),
                "ellipse" => "타원입니다.".to_string(),
                "rectangle" => "사각형입니다.".to_string(),
                "textbox" => "글상자입니다.".to_string(),
                "polygon" => "다각형입니다.".to_string(),
                "arc" => "호입니다.".to_string(),
                "connector-straight" => "직선 연결선입니다.".to_string(),
                "connector-stroke" => "꺾인 연결선입니다.".to_string(),
                "connector-arc" => "곡선 연결선입니다.".to_string(),
                _ => "그리기 개체.".to_string(),
            },
            ..Default::default()
        };

        let has_textbox = shape_type == "textbox";
        let has_fill = shape_type != "line" && !is_connector;

        let drawing = DrawingObjAttr {
            shape_attr: ShapeComponentAttr {
                ctrl_id,
                is_two_ctrl_id: true,
                original_width: width,
                original_height: height,
                current_width: width,
                current_height: height,
                local_file_version: 1,
                flip: 0x00080000, // 한컴 기본값
                rotation_center: crate::model::Point {
                    x: (width / 2) as i32,
                    y: (height / 2) as i32,
                },
                ..Default::default()
            },
            border_line: ShapeBorderLine {
                color: 0,
                width: 33,
                attr: 0xD1000041,
                outline_style: 0,
            },
            fill: if has_fill {
                Fill {
                    fill_type: crate::model::style::FillType::Solid,
                    solid: Some(crate::model::style::SolidFill {
                        background_color: 0x00FFFFFF,
                        pattern_color: 0,
                        pattern_type: -1,
                    }),
                    gradient: None,
                    image: None,
                    alpha: 0,
                }
            } else {
                Fill::default()
            },
            text_box: if has_textbox {
                Some(TextBox {
                    list_attr: 0x20,
                    vertical_all: false,
                    vertical_align: crate::model::table::VerticalAlign::Top,
                    margin_left: 283,
                    margin_right: 283,
                    margin_top: 283,
                    margin_bottom: 283,
                    max_width: width,
                    raw_list_header_extra: vec![0u8; 13],
                    paragraphs: vec![inner_para],
                })
            } else {
                None
            },
            // inst_id: 한컴 SubjectID 기준 = (CTRL_HEADER instance_id & 0x3FFFFFFF) + 1
            inst_id: (instance_id & 0x3FFFFFFF) + 1,
            ..Default::default()
        };

        let shape_obj = match shape_type {
            "line"
            | "connector-straight"
            | "connector-stroke"
            | "connector-arc"
            | "connector-straight-arrow"
            | "connector-stroke-arrow"
            | "connector-arc-arrow" => {
                // 드래그 방향에 따라 시작/끝점 결정
                let (sx, sy, ex, ey) = match (line_flip_x, line_flip_y) {
                    (false, false) => (0, 0, w_i, h_i), // 좌상→우하
                    (false, true) => (0, h_i, w_i, 0),  // 좌하→우상
                    (true, false) => (w_i, 0, 0, h_i),  // 우상→좌하
                    (true, true) => (w_i, h_i, 0, 0),   // 우하→좌상
                };
                let connector = if is_connector {
                    use crate::model::shape::{ConnectorControlPoint, ConnectorData, LinkLineType};
                    let link_type = match shape_type {
                        "connector-straight" => LinkLineType::StraightNoArrow,
                        "connector-straight-arrow" => LinkLineType::StraightOneWay,
                        "connector-stroke" => LinkLineType::StrokeNoArrow,
                        "connector-stroke-arrow" => LinkLineType::StrokeOneWay,
                        "connector-arc" => LinkLineType::ArcNoArrow,
                        "connector-arc-arrow" => LinkLineType::ArcOneWay,
                        _ => LinkLineType::StraightNoArrow,
                    };
                    // 꺽인/곡선 연결선: 한컴 호환 제어점 생성
                    // 구조: 시작앵커(type=3) + 중간점(type=2) + 끝앵커(type=26)
                    let control_points = match link_type {
                        LinkLineType::StrokeNoArrow
                        | LinkLineType::StrokeOneWay
                        | LinkLineType::StrokeBoth
                        | LinkLineType::ArcNoArrow
                        | LinkLineType::ArcOneWay
                        | LinkLineType::ArcBoth => {
                            vec![
                                ConnectorControlPoint {
                                    x: sx,
                                    y: sy,
                                    point_type: 3,
                                }, // 시작 앵커
                                ConnectorControlPoint {
                                    x: ex,
                                    y: sy,
                                    point_type: 2,
                                }, // 중간 (직각 꺾임)
                                ConnectorControlPoint {
                                    x: ex,
                                    y: ey,
                                    point_type: 26,
                                }, // 끝 앵커
                            ]
                        }
                        _ => Vec::new(),
                    };
                    Some(ConnectorData {
                        link_type,
                        start_subject_id: 0,
                        start_subject_index: 0,
                        end_subject_id: 0,
                        end_subject_index: 0,
                        control_points,
                        raw_trailing: vec![0x1a, 0, 0, 0, 0, 0], // 한컴 호환 패딩
                    })
                } else {
                    None
                };
                ShapeObject::Line(LineShape {
                    common,
                    drawing,
                    start: crate::model::Point { x: sx, y: sy },
                    end: crate::model::Point { x: ex, y: ey },
                    started_right_or_bottom: if is_connector {
                        false
                    } else {
                        line_flip_x || line_flip_y
                    },
                    connector,
                })
            }
            "ellipse" => ShapeObject::Ellipse(EllipseShape {
                common,
                drawing,
                attr: 0,
                center: crate::model::Point {
                    x: w_i / 2,
                    y: h_i / 2,
                },
                axis1: crate::model::Point { x: w_i, y: h_i / 2 },
                axis2: crate::model::Point { x: w_i / 2, y: h_i },
                start1: crate::model::Point { x: w_i, y: h_i / 2 },
                end1: crate::model::Point { x: w_i, y: h_i / 2 },
                start2: crate::model::Point { x: w_i, y: h_i / 2 },
                end2: crate::model::Point { x: w_i, y: h_i / 2 },
            }),
            "polygon" => {
                let points = if !polygon_points.is_empty() {
                    polygon_points.to_vec()
                } else {
                    // 기본 삼각형 (bbox 내접)
                    vec![
                        crate::model::Point { x: w_i / 2, y: 0 },
                        crate::model::Point { x: w_i, y: h_i },
                        crate::model::Point { x: 0, y: h_i },
                    ]
                };
                ShapeObject::Polygon(PolygonShape {
                    common,
                    drawing,
                    points,
                    raw_trailing: Vec::new(),
                })
            }
            "arc" => {
                // 사각형에 내접하는 타원의 1/4 호 (우상 사분면)
                // center: bbox 중심, axis1: 우측 중앙, axis2: 상단 중앙
                ShapeObject::Arc(ArcShape {
                    common,
                    drawing,
                    arc_type: 0, // 0=Arc
                    center: crate::model::Point {
                        x: w_i / 2,
                        y: h_i / 2,
                    },
                    axis1: crate::model::Point { x: w_i, y: h_i / 2 },
                    axis2: crate::model::Point { x: w_i / 2, y: 0 },
                })
            }
            _ => ShapeObject::Rectangle(RectangleShape {
                common,
                drawing,
                round_rate: 0,
                x_coords: [0, w_i, w_i, 0],
                y_coords: [0, 0, h_i, h_i],
            }),
        };

        // --- 기존 문단에 인라인 컨트롤로 삽입 ---
        self.document.sections[section_idx].raw_stream = None;

        let insert_para_idx = para_idx;
        let insert_ctrl_idx;
        {
            let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

            // 컨트롤 삽입 위치 결정 (char_offset 기준)
            let insert_idx = {
                let positions =
                    crate::document_core::helpers::find_control_text_positions(paragraph);
                let mut idx = paragraph.controls.len();
                for (i, &pos) in positions.iter().enumerate() {
                    if pos > char_offset {
                        idx = i;
                        break;
                    }
                }
                idx
            };

            // 컨트롤 추가
            paragraph
                .controls
                .insert(insert_idx, Control::Shape(Box::new(shape_obj)));
            paragraph.ctrl_data_records.insert(insert_idx, None);

            // char_offsets에 raw offset 삽입
            if !paragraph.char_offsets.is_empty() {
                let raw_offset = if insert_idx > 0 && insert_idx <= paragraph.char_offsets.len() {
                    paragraph.char_offsets[insert_idx - 1] + 8
                } else if !paragraph.char_offsets.is_empty() {
                    let first = paragraph.char_offsets[0];
                    if first >= 8 {
                        first - 8
                    } else {
                        0
                    }
                } else {
                    (char_offset * 2) as u32
                };
                paragraph.char_offsets.insert(insert_idx, raw_offset);
            }

            // 삽입된 컨트롤 이후의 char_offsets를 8만큼 증가 (텍스트 매핑 유지)
            for co in paragraph.char_offsets.iter_mut().skip(insert_idx + 1) {
                *co += 8;
            }

            // char_count 갱신 (확장 컨트롤 = 8 code units)
            paragraph.char_count += 8;

            // control_mask에 GSO 비트 설정
            paragraph.control_mask |= 0x00000800;
            // has_para_text 보장
            paragraph.has_para_text = true;
            insert_ctrl_idx = insert_idx;
        }

        // 리플로우 + 페이지네이션
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: insert_para_idx,
        });
        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{}",
            insert_para_idx, insert_ctrl_idx
        )))
    }
    /// 글상자(Shape) z-order 변경 (네이티브).
    /// operation: "front" | "back" | "forward" | "backward"
    pub fn change_shape_z_order_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        operation: &str,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;

        // 구역 내 모든 Shape의 (z_order, para_idx, ctrl_idx) 수집
        let mut shape_infos: Vec<(i32, usize, usize)> = Vec::new();
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                if let Control::Shape(shape) = ctrl {
                    shape_infos.push((shape.z_order(), pi, ci));
                }
            }
        }

        // (z_order, para_idx, ctrl_idx) 기준 정렬 — 렌더링 순서와 동일
        shape_infos.sort();

        let target_pos = shape_infos
            .iter()
            .position(|&(_, pi, ci)| pi == para_idx && ci == control_idx)
            .ok_or_else(|| HwpError::RenderError("대상 Shape를 찾을 수 없습니다".to_string()))?;
        let current_z = shape_infos[target_pos].0;
        let last_pos = shape_infos.len() - 1;

        // (대상 새 z_order, 이웃 변경 정보 Option<(para_idx, ctrl_idx, 새 z_order)>)
        let changes: Option<(i32, Option<(usize, usize, i32)>)> = match operation {
            "front" => {
                if target_pos == last_pos {
                    None // 이미 맨 앞
                } else {
                    let max_z = shape_infos[last_pos].0;
                    Some((max_z + 1, None))
                }
            }
            "back" => {
                if target_pos == 0 {
                    None // 이미 맨 뒤
                } else {
                    let min_z = shape_infos[0].0;
                    Some((min_z - 1, None))
                }
            }
            "forward" => {
                if target_pos >= last_pos {
                    None // 이미 맨 앞
                } else {
                    let neighbor = shape_infos[target_pos + 1];
                    if current_z == neighbor.0 {
                        // 같은 z_order — 대상만 +1하여 이웃 위로 이동
                        Some((current_z + 1, None))
                    } else {
                        // 다른 z_order — 이웃과 z_order 교환
                        Some((neighbor.0, Some((neighbor.1, neighbor.2, current_z))))
                    }
                }
            }
            "backward" => {
                if target_pos == 0 {
                    None // 이미 맨 뒤
                } else {
                    let neighbor = shape_infos[target_pos - 1];
                    if current_z == neighbor.0 {
                        // 같은 z_order — 대상만 -1하여 이웃 아래로 이동
                        Some((current_z - 1, None))
                    } else {
                        // 다른 z_order — 이웃과 z_order 교환
                        Some((neighbor.0, Some((neighbor.1, neighbor.2, current_z))))
                    }
                }
            }
            _ => {
                return Err(HwpError::RenderError(format!(
                    "알 수 없는 operation: {}",
                    operation
                )))
            }
        };

        let (new_z, neighbor_change) = match changes {
            Some(c) => c,
            None => {
                return Ok(crate::document_core::helpers::json_ok_with(&format!(
                    "\"zOrder\":{}",
                    current_z
                )))
            }
        };

        // z_order 변경: 대상 + 이웃
        {
            let section = &mut self.document.sections[section_idx];
            if let Control::Shape(shape) = &mut section.paragraphs[para_idx].controls[control_idx] {
                shape.common_mut().z_order = new_z;
            }
            if let Some((n_pi, n_ci, n_z)) = neighbor_change {
                if let Control::Shape(shape) = &mut section.paragraphs[n_pi].controls[n_ci] {
                    shape.common_mut().z_order = n_z;
                }
            }
        }

        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"zOrder\":{}",
            new_z
        )))
    }
    /// 도형 내부 좌표만 스케일 (common/shape_attr은 변경하지 않음)
    fn scale_shape_coords(child: &mut crate::model::shape::ShapeObject, sx: f64, sy: f64) {
        use crate::model::shape::ShapeObject as SO;
        fn sp(v: i32, s: f64) -> i32 {
            (v as f64 * s).round() as i32
        }
        match child {
            SO::Line(ref mut s) => {
                s.start.x = sp(s.start.x, sx);
                s.start.y = sp(s.start.y, sy);
                s.end.x = sp(s.end.x, sx);
                s.end.y = sp(s.end.y, sy);
            }
            SO::Rectangle(ref mut s) => {
                let w = s.common.width as i32;
                let h = s.common.height as i32;
                s.x_coords = [0, w, w, 0];
                s.y_coords = [0, 0, h, h];
            }
            SO::Ellipse(ref mut s) => {
                s.center.x = sp(s.center.x, sx);
                s.center.y = sp(s.center.y, sy);
                s.axis1.x = sp(s.axis1.x, sx);
                s.axis1.y = sp(s.axis1.y, sy);
                s.axis2.x = sp(s.axis2.x, sx);
                s.axis2.y = sp(s.axis2.y, sy);
                s.start1.x = sp(s.start1.x, sx);
                s.start1.y = sp(s.start1.y, sy);
                s.end1.x = sp(s.end1.x, sx);
                s.end1.y = sp(s.end1.y, sy);
                s.start2.x = sp(s.start2.x, sx);
                s.start2.y = sp(s.start2.y, sy);
                s.end2.x = sp(s.end2.x, sx);
                s.end2.y = sp(s.end2.y, sy);
            }
            SO::Arc(ref mut s) => {
                s.center.x = sp(s.center.x, sx);
                s.center.y = sp(s.center.y, sy);
                s.axis1.x = sp(s.axis1.x, sx);
                s.axis1.y = sp(s.axis1.y, sy);
                s.axis2.x = sp(s.axis2.x, sx);
                s.axis2.y = sp(s.axis2.y, sy);
            }
            SO::Polygon(ref mut s) => {
                for p in &mut s.points {
                    p.x = sp(p.x, sx);
                    p.y = sp(p.y, sy);
                }
            }
            SO::Curve(ref mut s) => {
                for p in &mut s.points {
                    p.x = sp(p.x, sx);
                    p.y = sp(p.y, sy);
                }
            }
            _ => {}
        }
    }
    /// 그룹 자식 개체들을 비례 스케일 (크기/위치/도형좌표 포함)
    fn scale_group_children(children: &mut [crate::model::shape::ShapeObject], sx: f64, sy: f64) {
        use crate::model::shape::ShapeObject as SO;
        fn sp(v: i32, s: f64) -> i32 {
            (v as f64 * s).round() as i32
        }

        for child in children.iter_mut() {
            // CommonObjAttr 스케일
            let c = child.common_mut();
            c.horizontal_offset = (c.horizontal_offset as f64 * sx) as u32;
            c.vertical_offset = (c.vertical_offset as f64 * sy) as u32;
            c.width = ((c.width as f64 * sx).round().max(1.0)) as u32;
            c.height = ((c.height as f64 * sy).round().max(1.0)) as u32;
            let new_horz = c.horizontal_offset;
            let new_vert = c.vertical_offset;
            let new_cw = c.width;
            let new_ch = c.height;

            // 도형별 좌표 스케일
            match child {
                SO::Line(ref mut s) => {
                    s.start.x = sp(s.start.x, sx);
                    s.start.y = sp(s.start.y, sy);
                    s.end.x = sp(s.end.x, sx);
                    s.end.y = sp(s.end.y, sy);
                }
                SO::Rectangle(ref mut s) => {
                    let w = new_cw as i32;
                    let h = new_ch as i32;
                    s.x_coords = [0, w, w, 0];
                    s.y_coords = [0, 0, h, h];
                }
                SO::Ellipse(ref mut s) => {
                    s.center.x = sp(s.center.x, sx);
                    s.center.y = sp(s.center.y, sy);
                    s.axis1.x = sp(s.axis1.x, sx);
                    s.axis1.y = sp(s.axis1.y, sy);
                    s.axis2.x = sp(s.axis2.x, sx);
                    s.axis2.y = sp(s.axis2.y, sy);
                    s.start1.x = sp(s.start1.x, sx);
                    s.start1.y = sp(s.start1.y, sy);
                    s.end1.x = sp(s.end1.x, sx);
                    s.end1.y = sp(s.end1.y, sy);
                    s.start2.x = sp(s.start2.x, sx);
                    s.start2.y = sp(s.start2.y, sy);
                    s.end2.x = sp(s.end2.x, sx);
                    s.end2.y = sp(s.end2.y, sy);
                }
                SO::Arc(ref mut s) => {
                    s.center.x = sp(s.center.x, sx);
                    s.center.y = sp(s.center.y, sy);
                    s.axis1.x = sp(s.axis1.x, sx);
                    s.axis1.y = sp(s.axis1.y, sy);
                    s.axis2.x = sp(s.axis2.x, sx);
                    s.axis2.y = sp(s.axis2.y, sy);
                }
                SO::Polygon(ref mut s) => {
                    for p in &mut s.points {
                        p.x = sp(p.x, sx);
                        p.y = sp(p.y, sy);
                    }
                }
                SO::Curve(ref mut s) => {
                    for p in &mut s.points {
                        p.x = sp(p.x, sx);
                        p.y = sp(p.y, sy);
                    }
                }
                SO::Group(ref mut g) => {
                    g.shape_attr.current_width = new_cw;
                    g.shape_attr.original_width = new_cw;
                    g.shape_attr.current_height = new_ch;
                    g.shape_attr.original_height = new_ch;
                    Self::scale_group_children(&mut g.children, sx, sy);
                }
                SO::Picture(_) => {} // 그림은 크기만 변경
                SO::Chart(_) => {}   // 차트: 크기만 변경, 내부 좌표 스케일 없음 (Task #195 단계 2)
                SO::Ole(_) => {}     // OLE: 크기만 변경
            }

            // shape_attr 동기화
            let sa = match child {
                SO::Line(s) => &mut s.drawing.shape_attr,
                SO::Rectangle(s) => &mut s.drawing.shape_attr,
                SO::Ellipse(s) => &mut s.drawing.shape_attr,
                SO::Arc(s) => &mut s.drawing.shape_attr,
                SO::Polygon(s) => &mut s.drawing.shape_attr,
                SO::Curve(s) => &mut s.drawing.shape_attr,
                SO::Group(g) => &mut g.shape_attr,
                SO::Picture(p) => &mut p.shape_attr,
                SO::Chart(c) => &mut c.drawing.shape_attr,
                SO::Ole(o) => &mut o.drawing.shape_attr,
            };
            sa.offset_x = new_horz as i32;
            sa.offset_y = new_vert as i32;
            sa.current_width = new_cw;
            sa.original_width = new_cw;
            sa.current_height = new_ch;
            sa.original_height = new_ch;
            sa.render_tx = new_horz as f64;
            sa.render_ty = new_vert as f64;
            sa.raw_rendering = Vec::new();
        }
    }
    /// 구역 내 모든 Shape의 z_order 최대값을 반환 (새 Shape 생성 시 사용)
    fn max_shape_z_order_in_section(&self, section_idx: usize) -> i32 {
        self.document
            .sections
            .get(section_idx)
            .map(|section| {
                section
                    .paragraphs
                    .iter()
                    .flat_map(|p| p.controls.iter())
                    .filter_map(|ctrl| {
                        if let Control::Shape(shape) = ctrl {
                            Some(shape.z_order())
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(-1)
            })
            .unwrap_or(-1)
    }

    // ─── 개체 묶기/풀기 API ──────────────────────────────
    /// 선택된 개체들을 GroupShape로 묶는다.
    /// targets: [(para_idx, control_idx), ...] — 같은 구역 내 Shape 또는 Picture
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N}`
    pub fn group_shapes_native(
        &mut self,
        section_idx: usize,
        targets: &[(usize, usize)],
    ) -> Result<String, HwpError> {
        use crate::model::control::Control;
        use crate::model::shape::*;

        if targets.len() < 2 {
            return Err(HwpError::RenderError(
                "묶기 위해서는 2개 이상의 개체가 필요합니다".to_string(),
            ));
        }
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }

        // 1) 대상 개체들을 ShapeObject로 수집 (인덱스 유효성 검사 포함)
        let section = &self.document.sections[section_idx];
        let mut children: Vec<ShapeObject> = Vec::new();
        let mut group_min_x: i32 = i32::MAX;
        let mut group_min_y: i32 = i32::MAX;
        let mut group_max_x: i32 = i32::MIN;
        let mut group_max_y: i32 = i32::MIN;
        let mut first_common: Option<CommonObjAttr> = None;

        for &(pi, ci) in targets {
            if pi >= section.paragraphs.len() {
                return Err(HwpError::RenderError(format!(
                    "문단 인덱스 {} 범위 초과",
                    pi
                )));
            }
            if ci >= section.paragraphs[pi].controls.len() {
                return Err(HwpError::RenderError(format!(
                    "컨트롤 인덱스 {} 범위 초과 (문단 {})",
                    ci, pi
                )));
            }
            let ctrl = &section.paragraphs[pi].controls[ci];
            let (common, shape_obj) = match ctrl {
                Control::Shape(s) => {
                    let c = s.common().clone();
                    (c, (**s).clone())
                }
                Control::Picture(p) => {
                    let c = p.common.clone();
                    (c, ShapeObject::Picture(p.clone()))
                }
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "컨트롤 ({},{})은 Shape/Picture가 아닙니다",
                        pi, ci
                    )))
                }
            };

            // 합산 bbox 계산 (HWPUNIT 기준 — horizontal_offset, vertical_offset, width, height)
            let x1 = common.horizontal_offset as i32;
            let y1 = common.vertical_offset as i32;
            let x2 = x1 + common.width as i32;
            let y2 = y1 + common.height as i32;
            group_min_x = group_min_x.min(x1);
            group_min_y = group_min_y.min(y1);
            group_max_x = group_max_x.max(x2);
            group_max_y = group_max_y.max(y2);

            if first_common.is_none() {
                first_common = Some(common);
            }
            children.push(shape_obj);
        }

        let group_w = (group_max_x - group_min_x).max(1) as u32;
        let group_h = (group_max_y - group_min_y).max(1) as u32;
        let fc = first_common.unwrap();

        // 2) 자식 개체의 offset/render 좌표를 그룹 로컬 좌표로 변환
        for child in &mut children {
            // 그룹 내 로컬 좌표 계산
            let new_horz = ((child.common().horizontal_offset as i32 - group_min_x).max(0)) as u32;
            let new_vert = ((child.common().vertical_offset as i32 - group_min_y).max(0)) as u32;
            child.common_mut().horizontal_offset = new_horz;
            child.common_mut().vertical_offset = new_vert;

            // shape_attr: 렌더링에 사용되는 render_tx/ty와 offset_x/y 설정
            let sa = match child {
                ShapeObject::Line(s) => &mut s.drawing.shape_attr,
                ShapeObject::Rectangle(s) => &mut s.drawing.shape_attr,
                ShapeObject::Ellipse(s) => &mut s.drawing.shape_attr,
                ShapeObject::Arc(s) => &mut s.drawing.shape_attr,
                ShapeObject::Polygon(s) => &mut s.drawing.shape_attr,
                ShapeObject::Curve(s) => &mut s.drawing.shape_attr,
                ShapeObject::Group(g) => &mut g.shape_attr,
                ShapeObject::Picture(p) => &mut p.shape_attr,
                ShapeObject::Chart(c) => &mut c.drawing.shape_attr,
                ShapeObject::Ole(o) => &mut o.drawing.shape_attr,
            };
            sa.offset_x = new_horz as i32;
            sa.offset_y = new_vert as i32;
            sa.group_level = 1;
            sa.is_two_ctrl_id = false; // 그룹 자식은 ctrl_id 1번만
            sa.raw_rendering = Vec::new(); // 새로 생성 (직렬화 시 재계산)
                                           // 렌더러가 사용하는 변환 행렬 값 설정
            sa.render_tx = new_horz as f64;
            sa.render_ty = new_vert as f64;
            sa.render_sx = 1.0;
            sa.render_sy = 1.0;
            sa.render_b = 0.0;
            sa.render_c = 0.0;
        }

        // 3) GroupShape 조립
        let new_z_order = self.max_shape_z_order_in_section(section_idx) + 1;
        let group = GroupShape {
            common: CommonObjAttr {
                ctrl_id: 0x24636f6e, // '$con' — 그룹 컨테이너
                attr: fc.attr,
                vertical_offset: group_min_y as u32,
                horizontal_offset: group_min_x as u32,
                width: group_w,
                height: group_h,
                z_order: new_z_order,
                margin: fc.margin.clone(),
                treat_as_char: fc.treat_as_char,
                vert_rel_to: fc.vert_rel_to,
                vert_align: fc.vert_align,
                horz_rel_to: fc.horz_rel_to,
                horz_align: fc.horz_align,
                text_wrap: fc.text_wrap,
                description: "묶음 개체입니다.".to_string(),
                ..Default::default()
            },
            shape_attr: ShapeComponentAttr {
                ctrl_id: 0x24636f6e, // '$con'
                is_two_ctrl_id: true,
                original_width: group_w,
                original_height: group_h,
                current_width: group_w,
                current_height: group_h,
                local_file_version: 1,
                flip: 0x00080000,
                rotation_center: crate::model::Point {
                    x: (group_w / 2) as i32,
                    y: (group_h / 2) as i32,
                },
                ..Default::default()
            },
            children,
            caption: None,
        };

        let group_obj = ShapeObject::Group(group);

        // 4) 원래 개체들을 문단에서 제거 (큰 인덱스부터 제거해야 인덱스 밀림 방지)
        let mut sorted_targets: Vec<(usize, usize)> = targets.to_vec();
        sorted_targets.sort_by(|a, b| b.cmp(a)); // 역순 정렬

        // 첫 번째 삽입 위치 (원래 개체 중 가장 앞에 있는 것)
        let insert_target = *targets.iter().min().unwrap();

        for &(pi, ci) in &sorted_targets {
            let para = &mut self.document.sections[section_idx].paragraphs[pi];

            // char_offsets 조정
            let text_chars: Vec<char> = para.text.chars().collect();
            let mut ctrl_ci = 0usize;
            let mut prev_end: u32 = 0;
            let mut gap_start: Option<u32> = None;
            'outer: for i in 0..text_chars.len() {
                let offset = if i < para.char_offsets.len() {
                    para.char_offsets[i]
                } else {
                    prev_end
                };
                while prev_end + 8 <= offset && ctrl_ci < para.controls.len() {
                    if ctrl_ci == ci {
                        gap_start = Some(prev_end);
                        break 'outer;
                    }
                    ctrl_ci += 1;
                    prev_end += 8;
                }
                let char_size: u32 = if text_chars[i] == '\t' {
                    8
                } else if text_chars[i].len_utf16() == 2 {
                    2
                } else {
                    1
                };
                prev_end = offset + char_size;
            }
            if gap_start.is_none() {
                while ctrl_ci < para.controls.len() {
                    if ctrl_ci == ci {
                        gap_start = Some(prev_end);
                        break;
                    }
                    ctrl_ci += 1;
                    prev_end += 8;
                }
            }
            if let Some(gs) = gap_start {
                let threshold = gs + 8;
                for offset in para.char_offsets.iter_mut() {
                    if *offset >= threshold {
                        *offset -= 8;
                    }
                }
            }

            para.controls.remove(ci);
            if ci < para.ctrl_data_records.len() {
                para.ctrl_data_records.remove(ci);
            }
            if para.char_count >= 8 {
                para.char_count -= 8;
            }
        }

        // 5) 삽입 위치 인덱스 재계산 (제거 후 인덱스가 변했을 수 있음)
        //    insert_target의 para에서 그보다 앞에서 제거된 개체 수만큼 보정
        let (insert_pi, insert_ci_orig) = insert_target;
        let removed_before = sorted_targets
            .iter()
            .filter(|&&(pi, ci)| pi == insert_pi && ci < insert_ci_orig)
            .count();
        let insert_ci = insert_ci_orig - removed_before;

        // 6) GroupShape를 문단에 삽입
        {
            let para = &mut self.document.sections[section_idx].paragraphs[insert_pi];

            // controls/ctrl_data_records 삽입 (범위 보정)
            let ctrl_insert = insert_ci.min(para.controls.len());
            para.controls
                .insert(ctrl_insert, Control::Shape(Box::new(group_obj)));
            let cdr_insert = ctrl_insert.min(para.ctrl_data_records.len());
            para.ctrl_data_records.insert(cdr_insert, None);

            // char_offsets: 텍스트 문자 매핑이므로 컨트롤 인덱스와 무관
            // 기존 char_offsets에서 마지막 gap 위치 다음에 8바이트 추가
            if !para.char_offsets.is_empty() {
                // 모든 기존 char_offsets를 8씩 증가 (컨트롤이 앞에 삽입되므로)
                for co in para.char_offsets.iter_mut() {
                    *co += 8;
                }
            }
            para.char_count += 8;
            para.control_mask |= 0x00000800;
            para.has_para_text = true;
        }

        // 7) 리플로우 + 페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: insert_pi,
        });
        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{}",
            insert_pi, insert_ci
        )))
    }
    /// GroupShape를 풀어 자식 개체들을 개별 Shape/Picture로 복원한다.
    /// 스펙: 한 단계만 풀기 (중첩 그룹은 유지), 자식 cnt 1 감소
    pub fn ungroup_shape_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        use crate::model::control::Control;
        use crate::model::shape::*;

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }
        let para = &mut section.paragraphs[para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {} 범위 초과",
                control_idx
            )));
        }

        // GroupShape 추출
        match &para.controls[control_idx] {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Group(_) => {}
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 컨트롤이 GroupShape이 아닙니다".to_string(),
                    ))
                }
            },
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 컨트롤이 Shape이 아닙니다".to_string(),
                ))
            }
        };
        // GroupShape를 꺼냄
        let group_ctrl = para.controls.remove(control_idx);
        if control_idx < para.ctrl_data_records.len() {
            para.ctrl_data_records.remove(control_idx);
        }
        if para.char_count >= 8 {
            para.char_count -= 8;
        }

        let group_shape = match group_ctrl {
            Control::Shape(s) => match *s {
                ShapeObject::Group(g) => g,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        // 그룹의 글로벌 좌표
        let group_x = group_shape.common.horizontal_offset as i32;
        let group_y = group_shape.common.vertical_offset as i32;
        // 그룹 스케일 (리사이즈된 경우)
        let gsa = &group_shape.shape_attr;
        let group_sx = if gsa.original_width > 0 {
            gsa.current_width as f64 / gsa.original_width as f64
        } else {
            1.0
        };
        let group_sy = if gsa.original_height > 0 {
            gsa.current_height as f64 / gsa.original_height as f64
        } else {
            1.0
        };

        // 자식들을 개별 컨트롤로 복원
        let mut insert_idx = control_idx;
        for mut child in group_shape.children {
            // 파일에서 로드한 그룹 자식은 common이 기본값(0) → shape_attr에서 복원
            {
                let sa = child.shape_attr();
                let sa_w = sa.original_width;
                let sa_h = sa.original_height;
                let sa_ox = sa.offset_x;
                let sa_oy = sa.offset_y;
                let c = child.common_mut();
                if c.width == 0 && sa_w > 0 {
                    c.width = sa_w;
                }
                if c.height == 0 && sa_h > 0 {
                    c.height = sa_h;
                }
                if c.horizontal_offset == 0 && sa_ox > 0 {
                    c.horizontal_offset = sa_ox as u32;
                }
                if c.vertical_offset == 0 && sa_oy > 0 {
                    c.vertical_offset = sa_oy as u32;
                }
            }
            // 자식의 로컬 좌표를 글로벌 좌표로 변환 (그룹 스케일 적용)
            {
                let c = child.common_mut();
                c.horizontal_offset =
                    (group_x + (c.horizontal_offset as f64 * group_sx) as i32) as u32;
                c.vertical_offset = (group_y + (c.vertical_offset as f64 * group_sy) as i32) as u32;
                c.width = ((c.width as f64 * group_sx).round().max(1.0)) as u32;
                c.height = ((c.height as f64 * group_sy).round().max(1.0)) as u32;
                c.vert_rel_to = group_shape.common.vert_rel_to;
                c.vert_align = group_shape.common.vert_align;
                c.horz_rel_to = group_shape.common.horz_rel_to;
                c.horz_align = group_shape.common.horz_align;
                c.text_wrap = group_shape.common.text_wrap;
                c.attr = group_shape.common.attr;
                c.treat_as_char = group_shape.common.treat_as_char;
            }
            // 도형별 좌표에 그룹 스케일 적용
            if group_sx != 1.0 || group_sy != 1.0 {
                Self::scale_shape_coords(&mut child, group_sx, group_sy);
            }
            // shape_attr 갱신 (common 값 확정 후)
            let final_w = child.common().width;
            let final_h = child.common().height;
            {
                let sa = match &mut child {
                    ShapeObject::Line(s) => &mut s.drawing.shape_attr,
                    ShapeObject::Rectangle(s) => &mut s.drawing.shape_attr,
                    ShapeObject::Ellipse(s) => &mut s.drawing.shape_attr,
                    ShapeObject::Arc(s) => &mut s.drawing.shape_attr,
                    ShapeObject::Polygon(s) => &mut s.drawing.shape_attr,
                    ShapeObject::Curve(s) => &mut s.drawing.shape_attr,
                    ShapeObject::Group(g) => &mut g.shape_attr,
                    ShapeObject::Picture(p) => &mut p.shape_attr,
                    ShapeObject::Chart(c) => &mut c.drawing.shape_attr,
                    ShapeObject::Ole(o) => &mut o.drawing.shape_attr,
                };
                if sa.group_level > 0 {
                    sa.group_level -= 1;
                }
                sa.offset_x = 0;
                sa.offset_y = 0;
                sa.render_tx = 0.0;
                sa.render_ty = 0.0;
                sa.current_width = final_w;
                sa.original_width = final_w;
                sa.current_height = final_h;
                sa.original_height = final_h;
                sa.is_two_ctrl_id = true;
                sa.raw_rendering = Vec::new();
            }

            // 문단에 삽입
            para.controls
                .insert(insert_idx, Control::Shape(Box::new(child)));
            para.ctrl_data_records.insert(insert_idx, None);
            para.char_count += 8;
            para.control_mask |= 0x00000800;
            para.has_para_text = true;
            insert_idx += 1;
        }

        // char_offsets: 그룹 1개 → 자식 N개, net 변화 = (N-1) * 8
        let children_count = insert_idx - control_idx;
        if children_count > 1 && !para.char_offsets.is_empty() {
            let net_delta = ((children_count - 1) * 8) as u32;
            for co in para.char_offsets.iter_mut() {
                *co += net_delta;
            }
        }

        // 리플로우 + 페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureDeleted {
            section: section_idx,
            para: para_idx,
            ctrl: control_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    // ─── 수식 속성 API ──────────────────────────────────
    pub(crate) fn footnote_shape_number_format_code(
        format: crate::model::footnote::NumberFormat,
    ) -> u8 {
        crate::model::footnote::FootnoteShape::number_format_attr_code(format) as u8
    }
    fn footnote_shape_number_format_from_str(
        value: &str,
        fallback: crate::model::footnote::NumberFormat,
    ) -> crate::model::footnote::NumberFormat {
        crate::model::footnote::FootnoteShape::number_format_from_name(value, fallback)
    }
    fn footnote_shape_number_format_name(
        format: crate::model::footnote::NumberFormat,
    ) -> &'static str {
        use crate::model::footnote::NumberFormat;
        match format {
            NumberFormat::Digit => "digit",
            NumberFormat::CircledDigit => "circledDigit",
            NumberFormat::UpperRoman => "upperRoman",
            NumberFormat::LowerRoman => "lowerRoman",
            NumberFormat::UpperAlpha => "upperAlpha",
            NumberFormat::LowerAlpha => "lowerAlpha",
            NumberFormat::CircledUpperAlpha => "circledUpperAlpha",
            NumberFormat::CircledLowerAlpha => "circledLowerAlpha",
            NumberFormat::HangulSyllable => "hangulSyllable",
            NumberFormat::CircledHangulSyllable => "circledHangulSyllable",
            NumberFormat::HangulJamo => "hangulJamo",
            NumberFormat::CircledHangulJamo => "circledHangulJamo",
            NumberFormat::HangulDigit => "hangulDigit",
            NumberFormat::HanjaDigit => "hanjaDigit",
            NumberFormat::CircledHanjaDigit => "circledHanjaDigit",
            NumberFormat::HanjaGapEul => "hanjaGapEul",
            NumberFormat::HanjaGapEulHanja => "hanjaGapEulHanja",
            NumberFormat::FourSymbol => "fourSymbol",
            NumberFormat::UserChar => "userChar",
        }
    }
    fn encode_footnote_shape_attr(shape: &crate::model::footnote::FootnoteShape) -> u32 {
        shape.encode_attr()
    }
    fn sync_endnote_control_with_shape(
        endnote: &mut crate::model::footnote::Endnote,
        number_format_code: u8,
        prefix_char: char,
        suffix_char: char,
    ) {
        use crate::model::control::{AutoNumberType, Control};

        endnote.before_decoration_letter = if prefix_char == '\0' {
            0
        } else {
            prefix_char as u16
        };
        endnote.after_decoration_letter = if suffix_char == '\0' {
            0
        } else {
            suffix_char as u16
        };
        endnote.number_shape = number_format_code as u32;

        for para in &mut endnote.paragraphs {
            for ctrl in &mut para.controls {
                if let Control::AutoNumber(auto_num) = ctrl {
                    if auto_num.number_type == AutoNumberType::Endnote {
                        auto_num.format = number_format_code;
                        auto_num.prefix_char = prefix_char;
                        auto_num.suffix_char = suffix_char;
                        auto_num.number = endnote.number;
                        auto_num.assigned_number = endnote.number;
                    }
                }
            }
        }
    }
    pub(crate) fn renumber_paragraph_endnotes_with_shape(
        paragraphs: &mut [crate::model::paragraph::Paragraph],
        next_number: &mut u16,
        number_format_code: u8,
        prefix_char: char,
        suffix_char: char,
    ) {
        for para in paragraphs {
            for ctrl in &mut para.controls {
                match ctrl {
                    Control::Endnote(endnote) => {
                        endnote.number = *next_number;
                        Self::sync_endnote_control_with_shape(
                            endnote,
                            number_format_code,
                            prefix_char,
                            suffix_char,
                        );
                        *next_number = next_number.saturating_add(1);
                    }
                    Control::Table(table) => {
                        for cell in &mut table.cells {
                            Self::renumber_paragraph_endnotes_with_shape(
                                &mut cell.paragraphs,
                                next_number,
                                number_format_code,
                                prefix_char,
                                suffix_char,
                            );
                        }
                    }
                    Control::Shape(shape) => {
                        if let Some(text_box) =
                            shape.drawing_mut().and_then(|d| d.text_box.as_mut())
                        {
                            Self::renumber_paragraph_endnotes_with_shape(
                                &mut text_box.paragraphs,
                                next_number,
                                number_format_code,
                                prefix_char,
                                suffix_char,
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    /// 현재 구역의 미주 모양을 조회한다.
    pub fn get_endnote_shape_native(&self, section_idx: usize) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let shape = &section.section_def.endnote_shape;
        let separator_enabled = shape.separator_length != 0
            || shape.separator_line_type != 0
            || shape.separator_line_width != 0;
        let separator_color =
            crate::document_core::helpers::clipboard_color_to_css(shape.separator_color);

        Ok(format!(
            concat!(
                "{{\"ok\":true,",
                "\"numberFormat\":\"{}\",",
                "\"userChar\":\"{}\",",
                "\"prefixChar\":\"{}\",",
                "\"suffixChar\":\"{}\",",
                "\"startNumber\":{},",
                "\"separatorEnabled\":{},",
                "\"separatorLength\":{},",
                "\"separatorMarginTop\":{},",
                "\"separatorMarginBottom\":{},",
                "\"noteSpacing\":{},",
                "\"separatorLineType\":{},",
                "\"separatorLineWidth\":{},",
                "\"separatorColor\":\"{}\",",
                "\"numberCodeSuperscript\":{},",
                "\"printInlineAfterText\":{},",
                "\"numbering\":\"{}\",",
                "\"placement\":\"{}\"",
                "}}"
            ),
            Self::footnote_shape_number_format_name(shape.number_format),
            Self::json_escape_note_char(shape.user_char),
            Self::json_escape_note_char(shape.prefix_char),
            Self::json_escape_note_char(shape.suffix_char),
            shape.start_number,
            if separator_enabled { "true" } else { "false" },
            shape.separator_length,
            shape.separator_above_margin_hu(),
            shape.separator_below_margin_hu(),
            shape.between_notes_margin_hu(),
            shape.separator_line_type,
            shape.separator_line_width,
            separator_color,
            if shape.number_code_superscript {
                "true"
            } else {
                "false"
            },
            if shape.print_inline_after_text {
                "true"
            } else {
                "false"
            },
            Self::footnote_numbering_name(shape.numbering),
            Self::footnote_placement_name(shape.placement),
        ))
    }
    /// 현재 구역의 미주 모양을 적용한다.
    pub fn apply_endnote_shape_native(
        &mut self,
        section_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let shape = &mut section.section_def.endnote_shape;

        if let Some(v) = crate::document_core::helpers::json_str(props_json, "numberFormat") {
            shape.number_format =
                Self::footnote_shape_number_format_from_str(&v, shape.number_format);
        }
        if let Some(v) = crate::document_core::helpers::json_str(props_json, "userChar") {
            shape.user_char = Self::first_char_or_nul(&v);
        }
        if let Some(v) = crate::document_core::helpers::json_str(props_json, "prefixChar") {
            shape.prefix_char = Self::first_char_or_nul(&v);
        }
        if let Some(v) = crate::document_core::helpers::json_str(props_json, "suffixChar") {
            shape.suffix_char = Self::first_char_or_nul(&v);
        }
        if let Some(v) = crate::document_core::helpers::json_u16(props_json, "startNumber") {
            shape.start_number = v.max(1);
        }
        if let Some(v) = Self::hwpunit16_from_json(props_json, "separatorLength") {
            shape.separator_length = i32::from(v.max(0));
        }
        if let Some(v) = Self::hwpunit16_from_json(props_json, "separatorMarginTop") {
            let above = v.max(0);
            // HWP5 저장본은 구분선 위 값을 fallback 슬롯에 보관하는 경우가 있어 함께 갱신한다.
            shape.separator_margin_top = above;
            shape.separator_margin_bottom = above;
        }
        if let Some(v) = Self::hwpunit16_from_json(props_json, "separatorMarginBottom") {
            shape.note_spacing = v.max(0);
        }
        if let Some(v) = Self::hwpunit16_from_json(props_json, "noteSpacing") {
            shape.raw_unknown = v.max(0) as u16;
        }
        if let Some(v) = crate::document_core::helpers::json_u8(props_json, "separatorLineType") {
            shape.separator_line_type = v;
        }
        if let Some(v) = crate::document_core::helpers::json_u8(props_json, "separatorLineWidth") {
            shape.separator_line_width = v;
        }
        if let Some(v) = crate::document_core::helpers::json_color(props_json, "separatorColor") {
            shape.separator_color = v;
        }
        if let Some(v) = crate::document_core::helpers::json_str(props_json, "numbering") {
            shape.numbering = Self::footnote_numbering_from_str(&v, shape.numbering);
        }
        if let Some(v) = crate::document_core::helpers::json_str(props_json, "placement") {
            shape.placement = Self::footnote_placement_from_str(&v, shape.placement);
        }
        if let Some(v) =
            crate::document_core::helpers::json_bool(props_json, "numberCodeSuperscript")
        {
            shape.number_code_superscript = v;
        }
        if let Some(v) =
            crate::document_core::helpers::json_bool(props_json, "printInlineAfterText")
        {
            shape.print_inline_after_text = v;
        }
        if let Some(false) =
            crate::document_core::helpers::json_bool(props_json, "separatorEnabled")
        {
            shape.separator_length = 0;
            shape.separator_line_type = 0;
            shape.separator_line_width = 0;
        }
        shape.attr = Self::encode_footnote_shape_attr(shape);
        let start_number = shape.start_number.max(1);
        let number_format_code = Self::footnote_shape_number_format_code(shape.number_format);
        let prefix_char = shape.prefix_char;
        let suffix_char = shape.suffix_char;
        let mut next_number = start_number;
        Self::renumber_paragraph_endnotes_with_shape(
            &mut section.paragraphs,
            &mut next_number,
            number_format_code,
            prefix_char,
            suffix_char,
        );
        section.raw_stream = None;

        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        Ok(crate::document_core::helpers::json_ok())
    }
}

#[cfg(test)]
mod resize_clamp_tests {
    use super::*;
    use crate::model::document::{Document, Section, SectionDef};
    use crate::model::page::PageDef;

    fn make_test_core() -> DocumentCore {
        let mut doc = Document::default();
        doc.sections.push(Section {
            section_def: SectionDef {
                page_def: PageDef {
                    width: 59528,
                    height: 84188,
                    margin_left: 8504,
                    margin_right: 8504,
                    margin_top: 5668,
                    margin_bottom: 4252,
                    margin_header: 4252,
                    margin_footer: 4252,
                    ..Default::default()
                },
                ..Default::default()
            },
            paragraphs: vec![Paragraph::default()],
            raw_stream: None,
        });
        let mut core = DocumentCore::new_empty();
        // set_document이 composed/styles/pagination 벡터를 일관되게 초기화한다.
        core.set_document(doc);
        core
    }

    fn create_rectangle(core: &mut DocumentCore) -> (usize, usize) {
        let res = core
            .create_shape_control_native(
                0,
                0,
                0,
                9000,
                6750,
                0,
                0,
                false,
                "InFrontOfText",
                "rectangle",
                false,
                false,
                &[],
            )
            .expect("create rectangle");
        let para_idx = res
            .split("\"paraIdx\":")
            .nth(1)
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let ctrl_idx = res
            .split("\"controlIdx\":")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        (para_idx, ctrl_idx)
    }

    fn shape_common<'a>(
        core: &'a DocumentCore,
        para: usize,
        ctrl: usize,
    ) -> &'a crate::model::shape::CommonObjAttr {
        let c = &core.document.sections[0].paragraphs[para].controls[ctrl];
        match c {
            Control::Shape(s) => s.common(),
            _ => panic!("expected shape"),
        }
    }

    /// 리사이즈 핸들을 반대편 너머로 잡아끌 때 studio가 width=0 을 보내도
    /// 도형 공통 크기는 MIN_SHAPE_SIZE 이상을 유지해야 한다.
    #[test]
    fn resize_to_zero_width_clamps_to_min() {
        let mut core = make_test_core();
        let (para, ctrl) = create_rectangle(&mut core);

        core.set_shape_properties_native(0, para, ctrl, r#"{"width":0,"height":0}"#)
            .expect("resize to 0");

        let common = shape_common(&core, para, ctrl);
        assert!(
            common.width >= MIN_SHAPE_SIZE,
            "width clamped: {}",
            common.width
        );
        assert!(
            common.height >= MIN_SHAPE_SIZE,
            "height clamped: {}",
            common.height
        );
    }

    /// Rectangle은 common.width/height 를 기반으로 x_coords/y_coords 를 재계산한다.
    /// 0으로 내려가면 [0,0,0,0]이 되어 화면에서 사라졌던 버그 방어.
    #[test]
    fn rectangle_coords_nonzero_after_shrink_to_zero() {
        let mut core = make_test_core();
        let (para, ctrl) = create_rectangle(&mut core);

        core.set_shape_properties_native(0, para, ctrl, r#"{"width":0,"height":0}"#)
            .expect("resize to 0");

        let ctrl_ref = &core.document.sections[0].paragraphs[para].controls[ctrl];
        if let Control::Shape(shape) = ctrl_ref {
            if let ShapeObject::Rectangle(rect) = shape.as_ref() {
                assert_ne!(rect.x_coords, [0, 0, 0, 0], "Rectangle x_coords collapsed");
                assert_ne!(rect.y_coords, [0, 0, 0, 0], "Rectangle y_coords collapsed");
            } else {
                panic!("expected Rectangle variant");
            }
        }
    }

    /// 반복된 0-resize 후에도 원상 복구 가능한 양의 크기로 리사이즈할 수 있어야 한다.
    /// (사용자 보고 시나리오: 핸들 여러 번 클릭 → 도형 소실 → 되돌리기 불가)
    #[test]
    fn repeated_zero_resize_does_not_corrupt_state() {
        let mut core = make_test_core();
        let (para, ctrl) = create_rectangle(&mut core);

        for _ in 0..5 {
            core.set_shape_properties_native(0, para, ctrl, r#"{"width":0,"height":0}"#)
                .expect("repeated resize");
        }
        core.set_shape_properties_native(0, para, ctrl, r#"{"width":12000,"height":8000}"#)
            .expect("restore");

        let common = shape_common(&core, para, ctrl);
        assert_eq!(common.width, 12000);
        assert_eq!(common.height, 8000);
    }
}
