//! 그림 속성/삽입/삭제 + 표 생성 + 셀 bbox 관련 native 메서드

use super::super::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject};

/// 도형 최소 크기 (HWPUNIT).
/// 0으로 내려가면 Rectangle은 x_coords=[0,0,0,0]이 되고,
/// Group은 current/original 스케일이 0이 되어 자식이 전부 사라진다.
/// table_ops의 MIN_CELL_SIZE와 동일한 기준을 사용한다.
const MIN_SHAPE_SIZE: u32 = 200;

impl DocumentCore {
    const COMMON_OBJ_ATTR_KNOWN_MASK: u32 = 0x01
        | (0x03 << 3)
        | (0x07 << 5)
        | (0x03 << 8)
        | (0x07 << 10)
        | (1 << 13)
        | (1 << 14)
        | (0x07 << 15)
        | (0x03 << 18)
        | (1 << 20)
        | (0x07 << 21)
        | (0x03 << 24)
        | (1 << 26)
        | (1 << 28);

    fn sync_common_obj_attr_known_bits(c: &mut crate::model::shape::CommonObjAttr) {
        let packed =
            crate::document_core::converters::common_obj_attr_writer::pack_common_attr_bits(c);
        c.attr = (c.attr & !Self::COMMON_OBJ_ATTR_KNOWN_MASK)
            | (packed & Self::COMMON_OBJ_ATTR_KNOWN_MASK);
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

    fn resolve_picture_control_ref(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<&crate::model::image::Picture, HwpError> {
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
            Control::Picture(p) => Ok(p),
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Picture(p) => Ok(p),
                _ => Err(HwpError::RenderError(
                    "지정된 Shape 컨트롤이 그림이 아닙니다".to_string(),
                )),
            },
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 그림이 아닙니다".to_string(),
            )),
        }
    }

    fn resolve_picture_control_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<&mut crate::model::image::Picture, HwpError> {
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
            Control::Picture(p) => Ok(p),
            Control::Shape(shape) => match shape.as_mut() {
                ShapeObject::Picture(p) => Ok(p),
                _ => Err(HwpError::RenderError(
                    "지정된 Shape 컨트롤이 그림이 아닙니다".to_string(),
                )),
            },
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 그림이 아닙니다".to_string(),
            )),
        }
    }

    pub fn get_picture_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let pic = self.resolve_picture_control_ref(section_idx, parent_para_idx, control_idx)?;
        Self::format_picture_properties_json(pic)
    }

    fn picture_crop_extent_hu(pic: &crate::model::image::Picture) -> (i32, i32) {
        let width = if pic.shape_attr.original_width > 0 {
            pic.shape_attr.original_width
        } else {
            pic.shape_attr.current_width
        };
        let height = if pic.shape_attr.original_height > 0 {
            pic.shape_attr.original_height
        } else {
            pic.shape_attr.current_height
        };
        (
            i32::try_from(width).unwrap_or(i32::MAX),
            i32::try_from(height).unwrap_or(i32::MAX),
        )
    }

    fn picture_crop_ui_amounts(pic: &crate::model::image::Picture) -> (i32, i32, i32, i32) {
        let (extent_w, extent_h) = Self::picture_crop_extent_hu(pic);
        let left = pic.crop.left.max(0);
        let top = pic.crop.top.max(0);
        let right = if extent_w > 0 && pic.crop.right > left {
            (extent_w - pic.crop.right).max(0)
        } else {
            0
        };
        let bottom = if extent_h > 0 && pic.crop.bottom > top {
            (extent_h - pic.crop.bottom).max(0)
        } else {
            0
        };
        (left, top, right, bottom)
    }

    fn set_picture_crop_from_ui_amounts(
        pic: &mut crate::model::image::Picture,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    ) {
        let (extent_w, extent_h) = Self::picture_crop_extent_hu(pic);
        pic.crop.left = left.max(0);
        pic.crop.top = top.max(0);
        if extent_w > 0 {
            pic.crop.right = (extent_w - right.max(0)).max(pic.crop.left);
        } else {
            pic.crop.right = right.max(0);
        }
        if extent_h > 0 {
            pic.crop.bottom = (extent_h - bottom.max(0)).max(pic.crop.top);
        } else {
            pic.crop.bottom = bottom.max(0);
        }
    }

    fn picture_props_touch_shape_transform(props_json: &str) -> bool {
        const TRANSFORM_KEYS: [&str; 7] = [
            "\"width\"",
            "\"height\"",
            "\"vertOffset\"",
            "\"horzOffset\"",
            "\"rotationAngle\"",
            "\"horzFlip\"",
            "\"vertFlip\"",
        ];
        TRANSFORM_KEYS.iter().any(|key| props_json.contains(key))
    }

    fn picture_rotated_bounds(width: u32, height: u32, angle: i16) -> (u32, u32) {
        if width == 0 || height == 0 || angle.rem_euclid(360) == 0 {
            return (width, height);
        }

        let theta = (angle as f64).to_radians();
        let cos = theta.cos().abs();
        let sin = theta.sin().abs();
        let rotated_width = width as f64 * cos + height as f64 * sin;
        let rotated_height = width as f64 * sin + height as f64 * cos;
        (
            rotated_width.round().max(1.0) as u32,
            rotated_height.round().max(1.0) as u32,
        )
    }

    fn refresh_picture_rotation_layout_for_save(pic: &mut crate::model::image::Picture) {
        let cur_w = if pic.shape_attr.current_width > 0 {
            pic.shape_attr.current_width
        } else {
            pic.common.width
        };
        let cur_h = if pic.shape_attr.current_height > 0 {
            pic.shape_attr.current_height
        } else {
            pic.common.height
        };

        if cur_w == 0 || cur_h == 0 {
            return;
        }

        pic.shape_attr.current_width = cur_w;
        pic.shape_attr.current_height = cur_h;

        let old_center_x =
            pic.common.horizontal_offset as i32 as i64 + (pic.common.width as i64 / 2);
        let old_center_y =
            pic.common.vertical_offset as i32 as i64 + (pic.common.height as i64 / 2);
        let (bbox_w, bbox_h) =
            Self::picture_rotated_bounds(cur_w, cur_h, pic.shape_attr.rotation_angle);

        if pic.shape_attr.rotation_angle.rem_euclid(360) != 0 {
            pic.common.width = bbox_w;
            pic.common.height = bbox_h;
            pic.common.horizontal_offset = (old_center_x - (bbox_w as i64 / 2)) as i32 as u32;
            pic.common.vertical_offset = (old_center_y - (bbox_h as i64 / 2)) as i32 as u32;
        } else {
            pic.common.width = cur_w;
            pic.common.height = cur_h;
            pic.common.horizontal_offset = (old_center_x - (cur_w as i64 / 2)) as i32 as u32;
            pic.common.vertical_offset = (old_center_y - (cur_h as i64 / 2)) as i32 as u32;
        }

        pic.shape_attr.rotation_center.x = (pic.common.width / 2) as i32;
        pic.shape_attr.rotation_center.y = (pic.common.height / 2) as i32;
        pic.shape_attr.rotate_image = true;
        pic.shape_attr.flip |= 0x0008_0000;
    }

    fn apply_picture_display_width(pic: &mut crate::model::image::Picture, width: u32) {
        let old_common_width = pic.common.width;
        let old_current_width = pic.shape_attr.current_width;
        pic.common.width = width;
        if pic.shape_attr.rotation_angle.rem_euclid(360) != 0
            && old_common_width > 0
            && old_current_width > 0
        {
            pic.shape_attr.current_width =
                ((old_current_width as f64 * width as f64 / old_common_width as f64).round())
                    .max(1.0) as u32;
        } else {
            pic.shape_attr.current_width = width;
        }
    }

    fn apply_picture_display_height(pic: &mut crate::model::image::Picture, height: u32) {
        let old_common_height = pic.common.height;
        let old_current_height = pic.shape_attr.current_height;
        pic.common.height = height;
        if pic.shape_attr.rotation_angle.rem_euclid(360) != 0
            && old_common_height > 0
            && old_current_height > 0
        {
            pic.shape_attr.current_height =
                ((old_current_height as f64 * height as f64 / old_common_height as f64).round())
                    .max(1.0) as u32;
        } else {
            pic.shape_attr.current_height = height;
        }
    }

    /// [Task #825] 머리말/꼬리말 안 그림의 속성 조회.
    /// path: section[si].paragraphs[outer_para].controls[outer_ctrl] = Header/Footer
    ///       → .paragraphs[inner_para].controls[inner_ctrl] = Picture
    pub fn get_header_footer_picture_properties_native(
        &self,
        section_idx: usize,
        outer_para_idx: usize,
        outer_control_idx: usize,
        inner_para_idx: usize,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let outer_para = section.paragraphs.get(outer_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("외부 문단 인덱스 {} 범위 초과", outer_para_idx))
        })?;
        let outer_ctrl = outer_para.controls.get(outer_control_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "외부 컨트롤 인덱스 {} 범위 초과",
                outer_control_idx
            ))
        })?;

        let inner_paras: &[crate::model::paragraph::Paragraph] = match outer_ctrl {
            crate::model::control::Control::Header(h) => &h.paragraphs,
            crate::model::control::Control::Footer(f) => &f.paragraphs,
            _ => {
                return Err(HwpError::RenderError(
                    "외부 컨트롤이 머리말/꼬리말이 아닙니다".to_string(),
                ))
            }
        };

        let inner_para = inner_paras.get(inner_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("내부 문단 인덱스 {} 범위 초과", inner_para_idx))
        })?;
        let inner_ctrl = inner_para.controls.get(inner_control_idx).ok_or_else(|| {
            HwpError::RenderError(format!(
                "내부 컨트롤 인덱스 {} 범위 초과",
                inner_control_idx
            ))
        })?;

        let pic = match inner_ctrl {
            crate::model::control::Control::Picture(p) => p,
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 내부 컨트롤이 그림이 아닙니다".to_string(),
                ))
            }
        };
        Self::format_picture_properties_json(pic)
    }

    fn format_picture_properties_json(
        pic: &crate::model::image::Picture,
    ) -> Result<String, HwpError> {
        let c = &pic.common;
        let vert_rel = match c.vert_rel_to {
            crate::model::shape::VertRelTo::Paper => "Paper",
            crate::model::shape::VertRelTo::Page => "Page",
            crate::model::shape::VertRelTo::Para => "Para",
        };
        let vert_align = match c.vert_align {
            crate::model::shape::VertAlign::Top => "Top",
            crate::model::shape::VertAlign::Center => "Center",
            crate::model::shape::VertAlign::Bottom => "Bottom",
            crate::model::shape::VertAlign::Inside => "Inside",
            crate::model::shape::VertAlign::Outside => "Outside",
        };
        let horz_rel = match c.horz_rel_to {
            crate::model::shape::HorzRelTo::Paper => "Paper",
            crate::model::shape::HorzRelTo::Page => "Page",
            crate::model::shape::HorzRelTo::Column => "Column",
            crate::model::shape::HorzRelTo::Para => "Para",
        };
        let horz_align = match c.horz_align {
            crate::model::shape::HorzAlign::Left => "Left",
            crate::model::shape::HorzAlign::Center => "Center",
            crate::model::shape::HorzAlign::Right => "Right",
            crate::model::shape::HorzAlign::Inside => "Inside",
            crate::model::shape::HorzAlign::Outside => "Outside",
        };
        let text_wrap = match c.text_wrap {
            crate::model::shape::TextWrap::Square => "Square",
            crate::model::shape::TextWrap::Tight => "Tight",
            crate::model::shape::TextWrap::Through => "Through",
            crate::model::shape::TextWrap::TopAndBottom => "TopAndBottom",
            crate::model::shape::TextWrap::BehindText => "BehindText",
            crate::model::shape::TextWrap::InFrontOfText => "InFrontOfText",
        };
        let effect = match pic.image_attr.effect {
            crate::model::image::ImageEffect::RealPic => "RealPic",
            crate::model::image::ImageEffect::GrayScale => "GrayScale",
            crate::model::image::ImageEffect::BlackWhite => "BlackWhite",
            crate::model::image::ImageEffect::Pattern8x8 => "Pattern8x8",
        };
        // description 내 JSON 제어 문자 이스케이프
        let desc_escaped = super::super::helpers::json_escape(&c.description);
        // [Task #741 후속] 외부 file path (HWP3 외부 그림) 영역 영역 dialog 표시 영역
        let external_path_field = match &pic.image_attr.external_path {
            Some(p) => format!(
                ",\"externalPath\":\"{}\"",
                super::super::helpers::json_escape(p)
            ),
            None => String::new(),
        };

        let sa = &pic.shape_attr;
        let (crop_left, crop_top, crop_right, crop_bottom) = Self::picture_crop_ui_amounts(pic);

        Ok(format!(
            concat!(
                "{{\"width\":{},\"height\":{},\"treatAsChar\":{},",
                "\"vertRelTo\":\"{}\",\"vertAlign\":\"{}\",",
                "\"horzRelTo\":\"{}\",\"horzAlign\":\"{}\",",
                "\"vertOffset\":{},\"horzOffset\":{},",
                "\"textWrap\":\"{}\",\"restrictInPage\":{},\"allowOverlap\":{},\"sizeProtect\":{},",
                "\"brightness\":{},\"contrast\":{},\"effect\":\"{}\",\"transparency\":{},",
                "\"description\":\"{}\",",
                // 회전/대칭
                "\"rotationAngle\":{},\"horzFlip\":{},\"vertFlip\":{},",
                // 원본 크기
                "\"originalWidth\":{},\"originalHeight\":{},",
                // 자르기
                "\"cropLeft\":{},\"cropTop\":{},\"cropRight\":{},\"cropBottom\":{},",
                // 안쪽 여백 (그림 여백)
                "\"paddingLeft\":{},\"paddingTop\":{},\"paddingRight\":{},\"paddingBottom\":{},",
                // 바깥 여백
                "\"outerMarginLeft\":{},\"outerMarginTop\":{},\"outerMarginRight\":{},\"outerMarginBottom\":{},",
                // 테두리
                "\"borderColor\":{},\"borderWidth\":{},",
                // 캡션
                "\"hasCaption\":{},\"captionDirection\":\"{}\",\"captionVertAlign\":\"{}\",",
                "\"captionWidth\":{},\"captionSpacing\":{},\"captionMaxWidth\":{},\"captionIncludeMargin\":{}{}}}"
            ),
            c.width, c.height, c.treat_as_char,
            vert_rel, vert_align,
            horz_rel, horz_align,
            c.vertical_offset as i32, c.horizontal_offset as i32,
            text_wrap, c.flow_with_text, c.allow_overlap, c.size_protect,
            pic.image_attr.brightness,
            pic.image_attr.contrast,
            effect,
            pic.image_attr.clamped_transparency(),
            desc_escaped,
            // 회전/대칭
            sa.rotation_angle, sa.horz_flip, sa.vert_flip,
            // 원본 크기
            sa.original_width, sa.original_height,
            // 자르기
            crop_left, crop_top, crop_right, crop_bottom,
            // 안쪽 여백
            pic.padding.left, pic.padding.top, pic.padding.right, pic.padding.bottom,
            // 바깥 여백
            c.margin.left, c.margin.top, c.margin.right, c.margin.bottom,
            // 테두리
            pic.border_color, pic.border_width,
            // 캡션
            pic.caption.is_some(),
            pic.caption.as_ref().map_or("Bottom", |cap| match cap.direction {
                crate::model::shape::CaptionDirection::Left => "Left",
                crate::model::shape::CaptionDirection::Right => "Right",
                crate::model::shape::CaptionDirection::Top => "Top",
                crate::model::shape::CaptionDirection::Bottom => "Bottom",
            }),
            pic.caption.as_ref().map_or("Top", |cap| match cap.vert_align {
                crate::model::shape::CaptionVertAlign::Top => "Top",
                crate::model::shape::CaptionVertAlign::Center => "Center",
                crate::model::shape::CaptionVertAlign::Bottom => "Bottom",
            }),
            pic.caption.as_ref().map_or(0u32, |cap| cap.width),
            pic.caption.as_ref().map_or(0i16, |cap| cap.spacing),
            pic.caption.as_ref().map_or(0u32, |cap| cap.max_width),
            pic.caption.as_ref().map_or(false, |cap| cap.include_margin),
            external_path_field,
        ))
    }

    /// 그림 컨트롤의 속성을 변경한다 (네이티브).
    pub fn set_picture_properties_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        // JSON 파싱 (serde_json 사용 대신 수동 파싱 — 기존 패턴)
        // [Task #825] 픽쳐 속성 mutation 은 helper 로 분리 (머리말/꼬리말 path 와 공유).
        let (caption_created, should_migrate_to_inline, should_migrate_to_floating) = {
            let pic =
                self.resolve_picture_control_mut(section_idx, parent_para_idx, control_idx)?;
            // [Task #1151 v2] tac false→true migration 검출용 snapshot.
            let was_tac = pic.common.treat_as_char;
            let caption_created = Self::apply_picture_props_inner(pic, props_json);
            let now_tac = pic.common.treat_as_char;
            (caption_created, !was_tac && now_tac, was_tac && !now_tac)
        };

        // [Task #1151 v2] floating → inline migration (H1 정합, samples/tac-verify/).
        // 한컴 산출물 Scenario A~D 분석: tac false→true 시 picture 의 control 위치는
        // 불변이고, 4 필드만 갱신 (treat_as_char / h/v_rel_to=Para / h/v_offset=0 /
        // parent line_segs[0]). text/char_offsets/paragraph 수 변화 없음.
        if should_migrate_to_inline || should_migrate_to_floating {
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
            if should_migrate_to_inline {
                let crate::model::paragraph::Paragraph {
                    line_segs,
                    controls,
                    ..
                } = &mut *para;
                match controls.get_mut(control_idx) {
                    Some(Control::Picture(pic_box)) => {
                        Self::migrate_picture_floating_to_inline(line_segs, pic_box.as_mut());
                    }
                    Some(Control::Shape(shape)) => {
                        if let ShapeObject::Picture(pic) = shape.as_mut() {
                            Self::migrate_picture_floating_to_inline(line_segs, pic);
                        }
                    }
                    _ => {}
                }
            } else {
                Self::migrate_empty_picture_para_inline_to_floating(para);
            }
        }
        // 캡션 생성 시 AutoNumber 재할당 + 텍스트 생성 (본문 path 만 — 머리말/꼬리말은 별도).
        if caption_created {
            crate::parser::assign_auto_numbers(&mut self.document);
            let pic_mut =
                self.resolve_picture_control_mut(section_idx, parent_para_idx, control_idx)?;
            let para = &mut pic_mut.caption.as_mut().unwrap().paragraphs[0];
            para.text = "그림  ".to_string();
            para.char_offsets = vec![0, 1, 2, 11];
            para.char_count = 13;
        }
        // 리플로우
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        // [Task #1151 v5] page tree cache invalidate — 다른 picture/shape setter (셀 shape
        // by_path / 셀 picture by_path / header-footer picture / shape 등) 모두 호출하나
        // 본 본문 picture setter 만 누락되어 있어 studio 가 stale page tree 반환 → tac toggle
        // 후 시각 변화 없음 증상의 root cause.
        self.invalidate_page_tree_cache();
        self.event_log.push(DocumentEvent::PictureResized {
            section: section_idx,
            para: parent_para_idx,
            ctrl: control_idx,
        });
        if caption_created {
            let char_offset = self
                .resolve_picture_control_ref(section_idx, parent_para_idx, control_idx)?
                .caption
                .as_ref()
                .map_or(0, |c| {
                    c.paragraphs.first().map_or(0, |p| p.text.chars().count())
                });
            Ok(format!(
                "{{\"ok\":true,\"captionCharOffset\":{}}}",
                char_offset
            ))
        } else {
            Ok("{\"ok\":true}".to_string())
        }
    }

    /// [Task #825] 머리말/꼬리말 안 그림 속성 변경.
    /// path: section[si].paragraphs[outer_para].controls[outer_ctrl] = Header/Footer
    ///       → .paragraphs[inner_para].controls[inner_ctrl] = Picture
    /// 캡션 신규 생성은 본 함수에서 미지원 (현 dialog UI 가 머리말 picture 캡션
    /// 변경을 노출하지 않음). caption_created 검출 시 NotSupported 에러.
    pub fn set_header_footer_picture_properties_native(
        &mut self,
        section_idx: usize,
        outer_para_idx: usize,
        outer_control_idx: usize,
        inner_para_idx: usize,
        inner_control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let caption_created;
        {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let outer_para = section.paragraphs.get_mut(outer_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("외부 문단 인덱스 {} 범위 초과", outer_para_idx))
            })?;
            let outer_ctrl = outer_para
                .controls
                .get_mut(outer_control_idx)
                .ok_or_else(|| {
                    HwpError::RenderError(format!(
                        "외부 컨트롤 인덱스 {} 범위 초과",
                        outer_control_idx
                    ))
                })?;
            let inner_paras: &mut Vec<crate::model::paragraph::Paragraph> = match outer_ctrl {
                crate::model::control::Control::Header(h) => &mut h.paragraphs,
                crate::model::control::Control::Footer(f) => &mut f.paragraphs,
                _ => {
                    return Err(HwpError::RenderError(
                        "외부 컨트롤이 머리말/꼬리말이 아닙니다".to_string(),
                    ))
                }
            };
            let inner_para = inner_paras.get_mut(inner_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("내부 문단 인덱스 {} 범위 초과", inner_para_idx))
            })?;
            let inner_ctrl = inner_para
                .controls
                .get_mut(inner_control_idx)
                .ok_or_else(|| {
                    HwpError::RenderError(format!(
                        "내부 컨트롤 인덱스 {} 범위 초과",
                        inner_control_idx
                    ))
                })?;
            let pic = match inner_ctrl {
                crate::model::control::Control::Picture(p) => p,
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 내부 컨트롤이 그림이 아닙니다".to_string(),
                    ))
                }
            };
            caption_created = Self::apply_picture_props_inner(pic, props_json);
        }
        if caption_created {
            return Err(HwpError::RenderError(
                "머리말/꼬리말 그림에 캡션 신규 생성은 본 버전에서 지원하지 않습니다".to_string(),
            ));
        }
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.event_log.push(DocumentEvent::PictureResized {
            section: section_idx,
            para: outer_para_idx,
            ctrl: outer_control_idx,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// [Task #1151 v2] Floating picture → inline 마이그레이션 (H1 정합).
    ///
    /// 한컴 2022 산출물 (`samples/tac-verify/scenario-{a,b,c,d}-after.hwp`) 분석
    /// 결과: floating picture 의 `treat_as_char` 가 false→true 로 토글될 때
    /// 한컴은 다음만 갱신한다 (자세한 분석: `mydocs/tech/hancom_picture_tac_toggle.md`).
    ///
    /// Picture 자체: `horz_rel_to = Para`, `vert_rel_to = Para`,
    /// `horizontal_offset = 0`, `vertical_offset = 0`. (`treat_as_char = true` 와 attr
    /// 비트는 `apply_picture_props_inner` 가 이미 처리.)
    ///
    /// Parent paragraph 의 `line_segs[0]`: `line_height = picture.common.height`,
    /// `text_height = picture.common.height`, `baseline_distance = round(line_height × 0.85)`.
    /// 비율 0.85 는 한컴 산출물 4 시나리오 (5331/16038/4847/19019) 모두 정확 관찰.
    /// `line_segs` 가 비어있으면 신설 (line_spacing=600 기본).
    ///
    /// 변경 없음: paragraph.text / char_offsets / char_shapes / paragraph 수, picture
    /// control 의 paragraph 위치 (sentinel char 추가하지 않음, 셀 안 이동 / 새 paragraph
    /// 분리 모두 없음 — H1 정합).
    pub(crate) fn migrate_picture_floating_to_inline(
        line_segs: &mut Vec<crate::model::paragraph::LineSeg>,
        pic: &mut crate::model::image::Picture,
    ) {
        use crate::model::shape::{HorzRelTo, VertRelTo};
        pic.common.horz_rel_to = HorzRelTo::Para;
        pic.common.vert_rel_to = VertRelTo::Para;
        pic.common.horizontal_offset = 0;
        pic.common.vertical_offset = 0;

        let picture_height_hu = pic.common.height as i32;
        let baseline = (picture_height_hu as f64 * 0.85).round() as i32;
        if let Some(seg) = line_segs.first_mut() {
            seg.line_height = picture_height_hu;
            seg.text_height = picture_height_hu;
            seg.baseline_distance = baseline;
        } else {
            line_segs.push(crate::model::paragraph::LineSeg {
                line_height: picture_height_hu,
                text_height: picture_height_hu,
                baseline_distance: baseline,
                line_spacing: 600,
                ..Default::default()
            });
        }
    }

    /// TAC 그림을 자리차지 개체로 되돌릴 때, 텍스트 없는 그림 전용 문단의
    /// LINE_SEG를 남은 TAC 개체 수에 맞춰 재구성한다.
    ///
    /// 기존 false→true 마이그레이션은 첫 LINE_SEG를 그림 높이로 키운다. 반대로
    /// true→false가 되면 그 그림은 더 이상 inline 글자 슬롯이 아니므로, 같은
    /// 문단의 남은 TAC 그림만 빈 줄에 1개씩 매핑되어야 한다. 한컴 저장본
    /// `투명도0-50-2nd그림글차처럼off.hwp`처럼 TopAndBottom 예약 높이는 첫 TAC
    /// 줄의 `vertical_pos`에 반영한다.
    pub(crate) fn migrate_empty_picture_para_inline_to_floating(
        para: &mut crate::model::paragraph::Paragraph,
    ) {
        if !para.text.is_empty() || !para.char_offsets.is_empty() {
            return;
        }

        let old_seg = para.line_segs.first().cloned().unwrap_or_default();
        let line_spacing = if old_seg.line_spacing > 0 {
            old_seg.line_spacing
        } else {
            600
        };
        let reserved_hu = Self::topbottom_reserved_height_for_empty_picture_para(&para.controls);
        let tac_heights = para
            .controls
            .iter()
            .filter_map(Self::tac_control_height_for_empty_picture_para)
            .collect::<Vec<_>>();

        if tac_heights.is_empty() {
            para.line_segs = vec![crate::model::paragraph::LineSeg {
                text_start: 0,
                vertical_pos: reserved_hu,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing,
                segment_width: old_seg.segment_width,
                column_start: old_seg.column_start,
                tag: old_seg.tag,
            }];
            return;
        }

        let mut vpos = reserved_hu;
        let mut rebuilt = Vec::with_capacity(tac_heights.len());
        for (idx, height) in tac_heights.into_iter().enumerate() {
            let line_height = height.max(1);
            rebuilt.push(crate::model::paragraph::LineSeg {
                text_start: (idx as u32) * 8,
                vertical_pos: vpos,
                line_height,
                text_height: line_height,
                baseline_distance: (line_height as f64 * 0.85).round() as i32,
                line_spacing,
                segment_width: old_seg.segment_width,
                column_start: old_seg.column_start,
                tag: old_seg.tag,
            });
            vpos += line_height + line_spacing;
        }
        para.line_segs = rebuilt;
    }

    fn tac_control_height_for_empty_picture_para(ctrl: &Control) -> Option<i32> {
        match ctrl {
            Control::Picture(pic) if pic.common.treat_as_char => Some(pic.common.height as i32),
            Control::Shape(shape) if shape.common().treat_as_char => {
                let common_h = shape.common().height as i32;
                let current_h = shape.shape_attr().current_height as i32;
                Some(common_h.max(current_h))
            }
            Control::Table(table) if table.common.treat_as_char => Some(table.common.height as i32),
            Control::Equation(eq) if eq.common.treat_as_char => Some(eq.common.height as i32),
            _ => None,
        }
    }

    fn topbottom_reserved_height_for_empty_picture_para(controls: &[Control]) -> i32 {
        controls
            .iter()
            .map(|ctrl| match ctrl {
                Control::Picture(pic)
                    if !pic.common.treat_as_char
                        && matches!(
                            pic.common.text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        ) =>
                {
                    pic.common.height as i32
                        + pic.common.margin.top as i32
                        + pic.common.margin.bottom as i32
                }
                Control::Shape(shape)
                    if !shape.common().treat_as_char
                        && matches!(
                            shape.common().text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        ) =>
                {
                    let common = shape.common();
                    common.height as i32 + common.margin.top as i32 + common.margin.bottom as i32
                }
                Control::Table(table)
                    if !table.common.treat_as_char
                        && matches!(
                            table.common.text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        ) =>
                {
                    table.common.height as i32
                        + table.outer_margin_top as i32
                        + table.outer_margin_bottom as i32
                }
                _ => 0,
            })
            .sum()
    }

    /// [Task #1151 v7] cell_path JSON → Vec<(controlIdx, cellIdx, cellParaIdx)>.
    /// 4 개 by_path setter/getter (cell picture/shape × set/get) 의 공통 파싱.
    /// 빈 path 면 Err 반환.
    fn parse_cell_path_json(json: &str) -> Result<Vec<(usize, usize, usize)>, HwpError> {
        let path: Vec<(usize, usize, usize)> = serde_json::from_str::<Vec<serde_json::Value>>(json)
            .map_err(|e| HwpError::RenderError(format!("cell_path JSON 파싱 실패: {}", e)))?
            .iter()
            .map(|v| {
                let c = v
                    .get("controlIdx")
                    .or_else(|| v.get("controlIndex"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as usize;
                let ci = v
                    .get("cellIdx")
                    .or_else(|| v.get("cellIndex"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as usize;
                let cpi = v
                    .get("cellParaIdx")
                    .or_else(|| v.get("cellParaIndex"))
                    .and_then(|x| x.as_u64())
                    .unwrap_or(0) as usize;
                (c, ci, cpi)
            })
            .collect();
        if path.is_empty() {
            return Err(HwpError::RenderError(
                "cell_path 가 비어있습니다".to_string(),
            ));
        }
        Ok(path)
    }

    /// [Task #1151 v7] section + parent_para_idx + path → target paragraph (mut).
    /// 2 개 set_cell_*_by_path_native (Picture / Shape) 의 공통 traversal.
    /// immutable 버전은 cursor_nav.rs 의 `resolve_paragraph_by_path` 가 담당하며,
    /// [Task #1171] 이후 표 셀과 글상자(Shape text_box, cell_index=0 sentinel) 를 모두
    /// 처리하도록 immutable 짝과 동일하게 맞춘다.
    fn resolve_cell_paragraph_mut<'a>(
        section: &'a mut crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<&'a mut crate::model::paragraph::Paragraph, HwpError> {
        let mut current_para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let ctrl = current_para.controls.get_mut(ctrl_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[{}]: controls[{}] 범위 초과", i, ctrl_idx))
            })?;
            current_para = match ctrl {
                crate::model::control::Control::Table(t) => {
                    let cell = t.cells.get_mut(cell_idx).ok_or_else(|| {
                        HwpError::RenderError(format!("경로[{}]: cells[{}] 범위 초과", i, cell_idx))
                    })?;
                    cell.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: paragraphs[{}] 범위 초과",
                            i, cell_para_idx
                        ))
                    })?
                }
                // [Task #1171] 글상자(Shape text_box) — cell_index=0 sentinel.
                crate::model::control::Control::Shape(shape) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 글상자의 cell_index는 0이어야 합니다 ({})",
                            i, cell_idx
                        )));
                    }
                    let text_box = get_textbox_from_shape_mut(shape).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다",
                            i, ctrl_idx
                        ))
                    })?;
                    text_box.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: 글상자문단 {} 범위 초과",
                            i, cell_para_idx
                        ))
                    })?
                }
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "경로[{}]: controls[{}] 가 표/글상자가 아닙니다",
                        i, ctrl_idx
                    )))
                }
            };
        }
        Ok(current_para)
    }

    fn required_cell_height_for_picture(
        cell: &crate::model::table::Cell,
        pic: &crate::model::image::Picture,
    ) -> u32 {
        Self::required_cell_height_for_picture_padding(cell.padding.top, cell.padding.bottom, pic)
    }

    fn required_cell_height_for_picture_padding(
        padding_top: i16,
        padding_bottom: i16,
        pic: &crate::model::image::Picture,
    ) -> u32 {
        let vert_offset = (pic.common.vertical_offset as i32).max(0) as u32;
        let visual_height = if pic.shape_attr.rotation_angle.rem_euclid(360) != 0
            && pic.shape_attr.current_width > 0
            && pic.shape_attr.current_height > 0
        {
            pic.common.height
        } else {
            let (_, height) = Self::picture_rotated_bounds(
                pic.common.width,
                pic.common.height,
                pic.shape_attr.rotation_angle,
            );
            height
        };
        vert_offset
            .saturating_add(visual_height)
            .saturating_add(padding_top.max(0) as u32)
            .saturating_add(padding_bottom.max(0) as u32)
    }

    fn take_place_picture_flow_offset(pic: &crate::model::image::Picture) -> Option<i32> {
        if pic.common.treat_as_char
            || !matches!(
                pic.common.text_wrap,
                crate::model::shape::TextWrap::TopAndBottom
            )
            || !matches!(pic.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
        {
            return None;
        }

        let visual_height = if pic.shape_attr.rotation_angle.rem_euclid(360) != 0
            && pic.shape_attr.current_width > 0
            && pic.shape_attr.current_height > 0
        {
            pic.common.height
        } else {
            let (_, height) = Self::picture_rotated_bounds(
                pic.common.width,
                pic.common.height,
                pic.shape_attr.rotation_angle,
            );
            height
        };
        Some(
            (pic.common.vertical_offset as i32)
                .saturating_add(visual_height.min(i32::MAX as u32) as i32)
                .max(0),
        )
    }

    fn sync_direct_owner_cell_for_picture(
        section: &mut crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        inner_control_idx: usize,
    ) -> Result<(), HwpError> {
        if path.len() != 1 {
            return Ok(());
        }

        let (table_ctrl_idx, cell_idx, cell_para_idx) = path[0];
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let existing_line_height = para
            .line_segs
            .first()
            .map(|seg| seg.line_height)
            .unwrap_or(0);
        let table = match para.controls.get_mut(table_ctrl_idx) {
            Some(Control::Table(table)) => table,
            _ => return Ok(()),
        };
        let line_height_extra = (existing_line_height - table.common.height as i32).max(0);
        let mut line_seg_update: Option<(i32, i32)> = None;

        let required_height = {
            let cell = table.cells.get(cell_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[0]: cells[{}] 범위 초과", cell_idx))
            })?;
            let cell_para = cell.paragraphs.get(cell_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[0]: paragraphs[{}] 범위 초과", cell_para_idx))
            })?;
            let pic = match cell_para.controls.get(inner_control_idx) {
                Some(Control::Picture(pic)) => pic,
                _ => return Ok(()),
            };
            let take_place_flow_offset = Self::take_place_picture_flow_offset(pic);
            if table.common.treat_as_char {
                if let Some(flow_offset) = take_place_flow_offset {
                    let vertical_pos = if pic.common.flow_with_text {
                        0
                    } else {
                        flow_offset
                    };
                    line_seg_update = Some((vertical_pos, line_height_extra));
                }
            }
            if pic.common.flow_with_text {
                Some(Self::required_cell_height_for_picture(cell, pic))
            } else {
                None
            }
        };

        if let (Some(required_height), Some(cell)) =
            (required_height, table.cells.get_mut(cell_idx))
        {
            let synced_height = required_height.max(MIN_SHAPE_SIZE);
            if cell.height != synced_height {
                cell.height = synced_height;
            }
        }
        table.update_ctrl_dimensions();
        table.dirty = true;
        let new_table_height = table.common.height as i32;
        if let Some((vertical_pos, line_height_extra)) = line_seg_update {
            if let Some(seg) = para.line_segs.first_mut() {
                let line_height = new_table_height
                    .saturating_add(line_height_extra)
                    .max(MIN_SHAPE_SIZE as i32);
                seg.vertical_pos = vertical_pos;
                seg.line_height = line_height;
                seg.text_height = line_height;
                seg.baseline_distance =
                    ((line_height as i64 * 17 + 10) / 20).min(i32::MAX as i64) as i32;
            }
        }
        Ok(())
    }

    fn clamp_direct_owner_cell_picture_offsets(
        section: &mut crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        inner_control_idx: usize,
        clamp_horz: bool,
        clamp_vert: bool,
    ) -> Result<(), HwpError> {
        if path.len() != 1 || (!clamp_horz && !clamp_vert) {
            return Ok(());
        }

        let (table_ctrl_idx, cell_idx, cell_para_idx) = path[0];
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let table = match para.controls.get_mut(table_ctrl_idx) {
            Some(Control::Table(table)) => table,
            _ => return Ok(()),
        };
        let cell = table.cells.get_mut(cell_idx).ok_or_else(|| {
            HwpError::RenderError(format!("경로[0]: cells[{}] 범위 초과", cell_idx))
        })?;

        let inner_width = cell
            .width
            .saturating_sub(cell.padding.left.max(0) as u32)
            .saturating_sub(cell.padding.right.max(0) as u32) as i64;
        let cell_para = cell.paragraphs.get_mut(cell_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("경로[0]: paragraphs[{}] 범위 초과", cell_para_idx))
        })?;
        let pic = match cell_para.controls.get_mut(inner_control_idx) {
            Some(Control::Picture(pic)) => pic,
            _ => return Ok(()),
        };

        if !pic.common.flow_with_text {
            return Ok(());
        }

        if clamp_horz {
            let max_horz = (inner_width - pic.common.width as i64)
                .max(0)
                .min(i32::MAX as i64);
            let horz = (pic.common.horizontal_offset as i32).clamp(0, max_horz as i32);
            pic.common.horizontal_offset = horz as u32;
        }
        if clamp_vert {
            let vert = (pic.common.vertical_offset as i32).max(0);
            pic.common.vertical_offset = vert as u32;
        }
        Ok(())
    }

    /// path 의 마지막 엔트리가 글상자(Shape text_box)를 가리키는지 판정한다.
    ///
    /// 표 셀 picture 삽입은 한컴 정합상 parent paragraph 의 sibling floating
    /// picture 로 처리하지만, 글상자 내부 picture 는 text_box paragraph 안에
    /// 실제 Picture control 로 들어가야 한다. `resolve_cell_by_path` 는 마지막
    /// 엔트리가 표일 때만 성공하므로, insert path 에서는 표/글상자를 먼저 구분한다.
    fn cell_path_terminates_at_textbox(
        section: &crate::model::document::Section,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
    ) -> Result<bool, HwpError> {
        let mut current_para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;

        for (i, &(ctrl_idx, cell_idx, cell_para_idx)) in path.iter().enumerate() {
            let ctrl = current_para.controls.get(ctrl_idx).ok_or_else(|| {
                HwpError::RenderError(format!("경로[{}]: controls[{}] 범위 초과", i, ctrl_idx))
            })?;
            match ctrl {
                crate::model::control::Control::Table(table) => {
                    let cell = table.cells.get(cell_idx).ok_or_else(|| {
                        HwpError::RenderError(format!("경로[{}]: cells[{}] 범위 초과", i, cell_idx))
                    })?;
                    if i == path.len() - 1 {
                        return Ok(false);
                    }
                    current_para = cell.paragraphs.get(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: paragraphs[{}] 범위 초과",
                            i, cell_para_idx
                        ))
                    })?;
                }
                crate::model::control::Control::Shape(shape) => {
                    if cell_idx != 0 {
                        return Err(HwpError::RenderError(format!(
                            "경로[{}]: 글상자의 cell_index는 0이어야 합니다 ({})",
                            i, cell_idx
                        )));
                    }
                    let text_box = get_textbox_from_shape(shape.as_ref()).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: controls[{}]가 텍스트 글상자가 아닙니다",
                            i, ctrl_idx
                        ))
                    })?;
                    if i == path.len() - 1 {
                        return Ok(true);
                    }
                    current_para = text_box.paragraphs.get(cell_para_idx).ok_or_else(|| {
                        HwpError::RenderError(format!(
                            "경로[{}]: 글상자문단 {} 범위 초과",
                            i, cell_para_idx
                        ))
                    })?;
                }
                _ => {
                    return Err(HwpError::RenderError(format!(
                        "경로[{}]: controls[{}] 가 표/글상자가 아닙니다",
                        i, ctrl_idx
                    )))
                }
            }
        }

        Err(HwpError::RenderError("경로가 비어있습니다".to_string()))
    }

    /// [Task #825] Picture 속성 JSON 적용 (mutation only). 후처리 (AutoNumber /
    /// recompose / paginate / event log) 는 호출자 책임.
    /// 반환: caption_created (true 면 호출자가 AutoNumber 후처리 필요).
    fn apply_picture_props_inner(pic: &mut crate::model::image::Picture, props_json: &str) -> bool {
        use super::super::helpers::{json_bool, json_i16, json_i32, json_str, json_u32};

        let transform_changed = Self::picture_props_touch_shape_transform(props_json);
        let mut rotation_changed = false;

        // 크기 변경
        if let Some(w) = json_u32(props_json, "width") {
            Self::apply_picture_display_width(pic, w);
        }
        if let Some(h) = json_u32(props_json, "height") {
            Self::apply_picture_display_height(pic, h);
        }

        // 위치 속성
        if let Some(tac) = json_bool(props_json, "treatAsChar") {
            pic.common.treat_as_char = tac;
            // attr 비트 갱신
            if tac {
                pic.common.attr |= 0x01;
            } else {
                pic.common.attr &= !0x01;
            }
        }
        if let Some(v) = json_str(props_json, "vertRelTo") {
            pic.common.vert_rel_to = match v.as_str() {
                "Paper" => crate::model::shape::VertRelTo::Paper,
                "Page" => crate::model::shape::VertRelTo::Page,
                "Para" => crate::model::shape::VertRelTo::Para,
                _ => pic.common.vert_rel_to,
            };
        }
        if let Some(v) = json_str(props_json, "horzRelTo") {
            pic.common.horz_rel_to = match v.as_str() {
                "Paper" => crate::model::shape::HorzRelTo::Paper,
                "Page" => crate::model::shape::HorzRelTo::Page,
                "Column" => crate::model::shape::HorzRelTo::Column,
                "Para" => crate::model::shape::HorzRelTo::Para,
                _ => pic.common.horz_rel_to,
            };
        }
        if let Some(v) = json_str(props_json, "vertAlign") {
            pic.common.vert_align = match v.as_str() {
                "Top" => crate::model::shape::VertAlign::Top,
                "Center" => crate::model::shape::VertAlign::Center,
                "Bottom" => crate::model::shape::VertAlign::Bottom,
                _ => pic.common.vert_align,
            };
        }
        if let Some(v) = json_str(props_json, "horzAlign") {
            pic.common.horz_align = match v.as_str() {
                "Left" => crate::model::shape::HorzAlign::Left,
                "Center" => crate::model::shape::HorzAlign::Center,
                "Right" => crate::model::shape::HorzAlign::Right,
                _ => pic.common.horz_align,
            };
        }
        if let Some(v) = json_str(props_json, "textWrap") {
            pic.common.text_wrap = match v.as_str() {
                "Square" => crate::model::shape::TextWrap::Square,
                "Tight" => crate::model::shape::TextWrap::Tight,
                "Through" => crate::model::shape::TextWrap::Through,
                "TopAndBottom" => crate::model::shape::TextWrap::TopAndBottom,
                "BehindText" => crate::model::shape::TextWrap::BehindText,
                "InFrontOfText" => crate::model::shape::TextWrap::InFrontOfText,
                _ => pic.common.text_wrap,
            };
        }
        if let Some(v) = json_bool(props_json, "restrictInPage") {
            pic.common.flow_with_text = v;
            if v {
                pic.common.attr |= 1 << 13;
                pic.common.allow_overlap = false;
                pic.common.attr &= !(1 << 14);
            } else {
                pic.common.attr &= !(1 << 13);
            }
        }
        if let Some(v) = json_bool(props_json, "allowOverlap") {
            pic.common.allow_overlap = v;
            if v {
                pic.common.attr |= 1 << 14;
            } else {
                pic.common.attr &= !(1 << 14);
            }
        }
        if let Some(v) = json_bool(props_json, "sizeProtect") {
            pic.common.size_protect = v;
            if v {
                pic.common.attr |= 1 << 20;
            } else {
                pic.common.attr &= !(1 << 20);
            }
        }
        if pic.common.flow_with_text {
            pic.common.allow_overlap = false;
            pic.common.attr &= !(1 << 14);
        }
        if let Some(v) = json_i32(props_json, "vertOffset") {
            pic.common.vertical_offset = v as u32;
        }
        if let Some(v) = json_i32(props_json, "horzOffset") {
            pic.common.horizontal_offset = v as u32;
        }
        Self::sync_common_obj_attr_known_bits(&mut pic.common);
        if transform_changed {
            pic.shape_attr.raw_rendering.clear();
            pic.shape_attr.render_tx = pic.shape_attr.offset_x as f64;
            pic.shape_attr.render_ty = pic.shape_attr.offset_y as f64;
            pic.shape_attr.render_sx = 1.0;
            pic.shape_attr.render_sy = 1.0;
            pic.shape_attr.render_b = 0.0;
            pic.shape_attr.render_c = 0.0;
        }

        // 이미지 속성
        if let Some(v) = json_i32(props_json, "brightness") {
            pic.image_attr.brightness = v as i8;
        }
        if let Some(v) = json_i32(props_json, "contrast") {
            pic.image_attr.contrast = v as i8;
        }
        if let Some(v) = json_i32(props_json, "transparency") {
            pic.image_attr.transparency = v.clamp(0, 100) as u8;
        }
        if let Some(v) = json_str(props_json, "effect") {
            pic.image_attr.effect = match v.as_str() {
                "GrayScale" => crate::model::image::ImageEffect::GrayScale,
                "BlackWhite" => crate::model::image::ImageEffect::BlackWhite,
                "Pattern8x8" => crate::model::image::ImageEffect::Pattern8x8,
                _ => crate::model::image::ImageEffect::RealPic,
            };
        }

        // 회전/대칭
        if let Some(v) = json_i16(props_json, "rotationAngle") {
            pic.shape_attr.rotation_angle = v;
            rotation_changed = true;
        }
        if let Some(v) = json_bool(props_json, "horzFlip") {
            pic.shape_attr.horz_flip = v;
            if v {
                pic.shape_attr.flip |= 0x01;
            } else {
                pic.shape_attr.flip &= !0x01;
            }
        }
        if let Some(v) = json_bool(props_json, "vertFlip") {
            pic.shape_attr.vert_flip = v;
            if v {
                pic.shape_attr.flip |= 0x02;
            } else {
                pic.shape_attr.flip &= !0x02;
            }
        }
        if rotation_changed {
            Self::refresh_picture_rotation_layout_for_save(pic);
        }

        // 자르기: HWP 내부 crop은 원본 이미지의 source rect 좌표이고,
        // 속성 창 UI는 네 방향에서 잘라낸 양을 표시한다.
        let crop_left = json_i32(props_json, "cropLeft");
        let crop_top = json_i32(props_json, "cropTop");
        let crop_right = json_i32(props_json, "cropRight");
        let crop_bottom = json_i32(props_json, "cropBottom");
        if crop_left.is_some()
            || crop_top.is_some()
            || crop_right.is_some()
            || crop_bottom.is_some()
        {
            let (mut left, mut top, mut right, mut bottom) = Self::picture_crop_ui_amounts(pic);
            if let Some(v) = crop_left {
                left = v;
            }
            if let Some(v) = crop_top {
                top = v;
            }
            if let Some(v) = crop_right {
                right = v;
            }
            if let Some(v) = crop_bottom {
                bottom = v;
            }
            Self::set_picture_crop_from_ui_amounts(pic, left, top, right, bottom);
        }

        // 안쪽 여백 (그림 여백)
        if let Some(v) = json_i16(props_json, "paddingLeft") {
            pic.padding.left = v;
        }
        if let Some(v) = json_i16(props_json, "paddingTop") {
            pic.padding.top = v;
        }
        if let Some(v) = json_i16(props_json, "paddingRight") {
            pic.padding.right = v;
        }
        if let Some(v) = json_i16(props_json, "paddingBottom") {
            pic.padding.bottom = v;
        }

        // 바깥 여백
        if let Some(v) = json_i16(props_json, "outerMarginLeft") {
            pic.common.margin.left = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginTop") {
            pic.common.margin.top = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginRight") {
            pic.common.margin.right = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginBottom") {
            pic.common.margin.bottom = v;
        }

        // 테두리
        if let Some(v) = json_u32(props_json, "borderColor") {
            pic.border_color = v;
        }
        if let Some(v) = json_i32(props_json, "borderWidth") {
            pic.border_width = v;
        }

        // description
        if let Some(v) = json_str(props_json, "description") {
            pic.common.description = v;
        }

        let mut caption_created = false;

        // 캡션
        if let Some(has_cap) = json_bool(props_json, "hasCaption") {
            if has_cap {
                // 캡션이 없으면 새로 생성 (기본 문단 포함)
                if pic.caption.is_none() {
                    let mut cap = crate::model::shape::Caption::default();
                    // AutoNumber 컨트롤 생성 (번호 할당은 아래에서)
                    let an = crate::model::control::AutoNumber {
                        number_type: crate::model::control::AutoNumberType::Picture,
                        ..Default::default()
                    };
                    cap.paragraphs
                        .push(crate::model::paragraph::Paragraph::default());
                    // 캡션 텍스트 최대 폭 = 개체 폭
                    cap.max_width = pic.common.width;
                    pic.caption = Some(cap);
                    caption_created = true;
                    // 번호 할당을 위해 컨트롤을 임시로 캡션에 추가
                    pic.caption.as_mut().unwrap().paragraphs[0]
                        .controls
                        .push(crate::model::control::Control::AutoNumber(an));
                    // attr bit 29: 캡션 존재 플래그 (한컴 호환성)
                    pic.common.attr |= 1 << 29;
                }
                let cap = pic.caption.as_mut().unwrap();
                if let Some(v) = json_str(props_json, "captionDirection") {
                    cap.direction = match v.as_str() {
                        "Left" => crate::model::shape::CaptionDirection::Left,
                        "Right" => crate::model::shape::CaptionDirection::Right,
                        "Top" => crate::model::shape::CaptionDirection::Top,
                        _ => crate::model::shape::CaptionDirection::Bottom,
                    };
                }
                if let Some(v) = json_str(props_json, "captionVertAlign") {
                    cap.vert_align = match v.as_str() {
                        "Center" => crate::model::shape::CaptionVertAlign::Center,
                        "Bottom" => crate::model::shape::CaptionVertAlign::Bottom,
                        _ => crate::model::shape::CaptionVertAlign::Top,
                    };
                }
                if let Some(v) = json_u32(props_json, "captionWidth") {
                    cap.width = v;
                }
                if let Some(v) = json_i16(props_json, "captionSpacing") {
                    cap.spacing = v;
                }
                if let Some(v) = json_bool(props_json, "captionIncludeMargin") {
                    cap.include_margin = v;
                }
            } else {
                // 캡션 제거 — 현재는 None 처리하지 않음 (캡션에 텍스트가 있을 수 있으므로)
            }
        }

        caption_created
    }

    /// 그림 컨트롤을 문단에서 삭제한다 (네이티브).
    pub fn delete_picture_control_native(
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
                "부모 문단 인덱스 {} 범위 초과",
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
        // 그림 컨트롤인지 확인
        if !matches!(
            &para.controls[control_idx],
            crate::model::control::Control::Picture(_)
        ) {
            return Err(HwpError::RenderError(
                "지정된 컨트롤이 그림이 아닙니다".to_string(),
            ));
        }

        // 컨트롤이 차지하는 갭의 시작 위치를 찾아 char_offsets 조정
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

        // char_offsets 조정
        if let Some(gs) = gap_start {
            let threshold = gs + 8;
            for offset in para.char_offsets.iter_mut() {
                if *offset >= threshold {
                    *offset -= 8;
                }
            }
        }

        // 컨트롤 및 ctrl_data_record 제거
        para.controls.remove(control_idx);
        if control_idx < para.ctrl_data_records.len() {
            para.ctrl_data_records.remove(control_idx);
        }

        // char_count 갱신
        if para.char_count >= 8 {
            para.char_count -= 8;
        }

        // line_segs 재계산: 그림 높이가 반영된 line_segs를 텍스트 기반으로 리셋
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

    /// 컨트롤 삭제 후 문단의 line_segs를 재계산한다.
    ///
    /// 그림/도형 삭제 시 문단의 line_segs에 컨트롤 높이가 그대로 남아,
    /// 레이아웃이 갱신되지 않는 문제를 방지한다.
    pub(crate) fn reflow_paragraph_line_segs_after_control_delete(
        para: &mut Paragraph,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        dpi: f64,
    ) {
        // 남은 컨트롤 중 가장 큰 높이 계산
        let max_remaining_ctrl_height = para
            .controls
            .iter()
            .map(|ctrl| match ctrl {
                Control::Picture(pic) => pic.common.height as i32,
                Control::Shape(shape) => shape.common().height as i32,
                Control::Equation(eq) => eq.common.height as i32,
                _ => 0,
            })
            .max()
            .unwrap_or(0);

        if max_remaining_ctrl_height > 0 {
            // 아직 컨트롤이 남아있으면 가장 큰 컨트롤 높이로 설정
            if let Some(ls) = para.line_segs.first_mut() {
                ls.line_height = max_remaining_ctrl_height;
                ls.text_height = max_remaining_ctrl_height;
                ls.baseline_distance = (max_remaining_ctrl_height * 850) / 1000;
            }
        } else if para.text.is_empty() {
            // 텍스트도 컨트롤도 없음 → 기본 텍스트 높이로 리셋
            if let Some(ls) = para.line_segs.first_mut() {
                ls.line_height = 1000;
                ls.text_height = 1000;
                ls.baseline_distance = 850;
                ls.line_spacing = 600;
            }
        } else {
            // 텍스트가 있으면 reflow_line_segs로 재계산
            let seg_width = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
            let available_width_px = crate::renderer::hwpunit_to_px(seg_width, dpi);
            crate::renderer::composer::reflow_line_segs(para, available_width_px, styles, dpi);
        }
    }

    /// 커서 위치에 새 표를 삽입한다 (네이티브).
    ///
    /// 1. PageDef에서 편집 영역 폭 계산
    /// 2. 균등 열 폭으로 row_count × col_count 셀 생성
    /// 3. Table + Paragraph 조립
    /// 4. 커서 위치에 삽입 (빈 문단이면 교체, 아니면 분할 후 삽입)
    /// 5. 표 아래에 빈 문단 추가 (HWP 표준)
    pub fn create_table_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        row_count: u16,
        col_count: u16,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::style::{BorderFill, BorderLine, BorderLineType, DiagonalLine, Fill};
        use crate::model::table::{Cell, Table, TablePageBreak};

        // 유효성 검사
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }
        if row_count == 0 || col_count == 0 || col_count > 256 {
            return Err(HwpError::RenderError(format!(
                "행/열 수 범위 오류 (행={}, 열={}, 열은 1~256)",
                row_count, col_count
            )));
        }

        // --- 1. 편집 영역 폭 계산 ---
        let pd = &self.document.sections[section_idx].section_def.page_def;
        let outer_margin_lr: i32 = 283 * 2; // outer_margin left + right (~2mm)
        let content_width =
            (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32 - outer_margin_lr)
                .max(7200) as u32;

        // --- 2. 한컴 기본값 기반 셀 생성 (blank_h_saved.hwp 참조) ---
        let col_width = content_width / col_count as u32;
        // 한컴 기본: 셀 패딩 L=510 R=510 T=141 B=141
        let cell_pad = crate::model::Padding {
            left: 510,
            right: 510,
            top: 141,
            bottom: 141,
        };
        // 한컴 기본: 셀 높이 = top + bottom padding (빈 셀 최소 높이)
        let cell_height: u32 = (cell_pad.top + cell_pad.bottom) as u32;
        // 한컴 기본: 행 렌더링 높이 = padding_top + line_height(1000) + padding_bottom
        let rendered_row_height: u32 = cell_pad.top as u32 + 1000 + cell_pad.bottom as u32;
        let total_width = col_width * col_count as u32;
        let total_height = rendered_row_height * row_count as u32;

        // BorderFill: 실선 테두리가 있는 기존 항목 재사용, 없으면 새로 생성
        let cell_border_fill_id = {
            let existing = self.document.doc_info.border_fills.iter().position(|bf| {
                bf.borders
                    .iter()
                    .all(|b| b.line_type == BorderLineType::Solid && b.width >= 1)
            });
            if let Some(idx) = existing {
                (idx + 1) as u16 // 1-based
            } else {
                // 실선 BorderFill이 없으면 새로 생성
                let solid_border = BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                };
                let new_bf = BorderFill {
                    raw_data: None,
                    attr: 0,
                    borders: [solid_border, solid_border, solid_border, solid_border],
                    diagonal: DiagonalLine {
                        diagonal_type: 1,
                        width: 0,
                        color: 0,
                    },
                    fill: Fill::default(),
                };
                self.document.doc_info.border_fills.push(new_bf);
                self.document.doc_info.raw_stream = None;
                self.document.doc_info.border_fills.len() as u16 // 1-based
            }
        };

        // 커서 위치 문단의 속성을 기본값으로 상속 (한컴 동작 일치)
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 셀 목록 생성
        let mut cells = Vec::with_capacity((row_count as usize) * (col_count as usize));
        for r in 0..row_count {
            for c in 0..col_count {
                let mut cell = Cell::new_empty(c, r, col_width, cell_height, cell_border_fill_id);
                cell.padding = cell_pad;
                cell.vertical_align = crate::model::table::VerticalAlign::Center; // 한컴 기본값
                                                                                  // 셀 문단 보정: char_count_msb, raw_header_extra, para/char shape
                for cp in &mut cell.paragraphs {
                    cp.char_count_msb = true;
                    cp.para_shape_id = default_para_shape_id;
                    if cp.raw_header_extra.len() < 10 {
                        let mut rhe = vec![0u8; 10];
                        rhe[0..2].copy_from_slice(&1u16.to_le_bytes()); // n_char_shapes=1
                        rhe[4..6].copy_from_slice(&1u16.to_le_bytes()); // n_line_segs=1
                        cp.raw_header_extra = rhe;
                    }
                    // line_segs 보정: new_empty()의 기본 LineSeg는 line_height=0이므로 항상 교체
                    let seg_w = (col_width as i32) - 141 - 141; // 셀 폭 - 좌우 패딩
                    cp.line_segs = vec![LineSeg {
                        text_start: 0,
                        line_height: 1000,
                        text_height: 1000,
                        baseline_distance: 850,
                        line_spacing: 600,
                        segment_width: seg_w,
                        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                        ..Default::default()
                    }];
                }
                // raw_list_extra: 빈 벡터 (cell.width 필드가 LIST_HEADER에 직렬화됨)
                cell.raw_list_extra = Vec::new();
                cells.push(cell);
            }
        }

        // --- 3. Table 구조체 조립 (한컴 기본 속성값) ---
        let row_sizes: Vec<i16> = (0..row_count).map(|_| col_count as i16).collect();

        // raw_ctrl_data: CommonObjAttr 바이너리 (파서 호환)
        // 바이트 레이아웃: flags(4) + v_offset(4) + h_offset(4) + width(4) + height(4)
        //                 + z_order(4) + margin_l(2) + margin_r(2) + margin_t(2) + margin_b(2)
        //                 + instance_id(4) = 36바이트 (+ 여유 2바이트 = 38)
        // vert=Para(2), horz=Para(3), wrap=TopAndBottom(1)
        // width_criterion=Absolute(4), height_criterion=Absolute(2)
        let flags: u32 = (2 << 3) | (3 << 8) | (4 << 15) | (2 << 18) | (1 << 21);
        let outer_margin: i16 = 283; // ~1mm
        let mut raw_ctrl_data = vec![0u8; 38];
        raw_ctrl_data[common_obj_offsets::FLAGS].copy_from_slice(&flags.to_le_bytes());
        // vertical_offset/horizontal_offset/z_order = 0
        raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_LEFT].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
            .copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_TOP].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
            .copy_from_slice(&outer_margin.to_le_bytes());
        // instance_id (해시 기반, 비-0 필수)
        let instance_id: u32 = {
            let mut h: u32 = 0x7c150000;
            h = h.wrapping_add(row_count as u32 * 0x1000);
            h = h.wrapping_add(col_count as u32 * 0x100);
            h = h.wrapping_add(total_width);
            h = h.wrapping_add(total_height.wrapping_mul(0x1b));
            if h == 0 {
                h = 0x7c154b69;
            }
            h
        };
        raw_ctrl_data[common_obj_offsets::INSTANCE_ID].copy_from_slice(&instance_id.to_le_bytes());

        let mut table = Table {
            attr: 0x082A2210, // 한컴 기본값 (blank_h_saved.hwp)
            row_count,
            col_count,
            cell_spacing: 0,
            padding: crate::model::Padding {
                left: 510,
                right: 510,
                top: 141,
                bottom: 141,
            },
            row_sizes,
            border_fill_id: cell_border_fill_id, // 한컴: 표와 셀이 같은 BorderFill 사용
            zones: Vec::new(),
            cells,
            cell_grid: Vec::new(),
            page_break: TablePageBreak::None,
            repeat_header: false,
            caption: None,
            common: crate::model::shape::CommonObjAttr {
                treat_as_char: false,
                text_wrap: crate::model::shape::TextWrap::TopAndBottom,
                vert_rel_to: crate::model::shape::VertRelTo::Para,
                horz_rel_to: crate::model::shape::HorzRelTo::Para,
                vert_align: crate::model::shape::VertAlign::Top,
                horz_align: crate::model::shape::HorzAlign::Left,
                width: total_width,
                height: total_height,
                ..Default::default()
            },
            outer_margin_left: 283,
            outer_margin_right: 283,
            outer_margin_top: 283,
            outer_margin_bottom: 283,
            raw_ctrl_data,
            raw_table_record_attr: 0x00000006, // 한컴 기본값 (bit1=셀분리금지, bit2=repeat_header)
            raw_table_record_extra: vec![0u8; 2],
            dirty: true,
            local_resize_rows: Vec::new(),
            local_resize_cols: Vec::new(),
            local_resize_cell_widths: Vec::new(),
            local_resize_cell_heights: Vec::new(),
        };
        table.rebuild_grid();

        // --- 4. Table을 포함하는 Paragraph 생성 ---
        // para_shape_id: 커서 위치 문단의 값 상속 (한컴 동작 일치)
        let table_para_shape_id = default_para_shape_id;

        let mut table_raw_header_extra = vec![0u8; 10];
        table_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes());
        table_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());

        let table_para = Paragraph {
            text: String::new(),
            char_count: 9, // 확장 제어문자(8 code units) + 문단끝(1)
            control_mask: 0x00000800,
            char_offsets: vec![],
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
                segment_width: 0, // 한컴 표준: 표 문단의 segment_width는 0
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            para_shape_id: table_para_shape_id,
            style_id: 0,
            controls: vec![Control::Table(Box::new(table))],
            ctrl_data_records: vec![None],
            has_para_text: true,
            raw_header_extra: table_raw_header_extra,
            char_count_msb: false,
            ..Default::default()
        };

        // --- 5. 커서 위치에 삽입 ---
        self.document.sections[section_idx].raw_stream = None;

        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let is_empty_para = para.text.is_empty() && para.controls.is_empty();

        let insert_para_idx;
        if is_empty_para && char_offset == 0 {
            // 빈 문단이면 교체
            self.document.sections[section_idx].paragraphs[para_idx] = table_para;
            insert_para_idx = para_idx;
        } else if char_offset == 0 && para.controls.is_empty() {
            // 문단 맨 앞이면 바로 앞에 삽입
            self.document.sections[section_idx]
                .paragraphs
                .insert(para_idx, table_para);
            insert_para_idx = para_idx;
        } else {
            // 문단 중간이면 분할 후 삽입
            if char_offset > 0 && !para.text.is_empty() {
                let new_para =
                    self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, new_para);
                // 표 문단은 분할된 뒤에 삽입
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, table_para);
                insert_para_idx = para_idx + 1;
            } else {
                // char_offset == 0이지만 컨트롤이 있는 경우 → 뒤에 삽입
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(para_idx + 1, table_para);
                insert_para_idx = para_idx + 1;
            }
        }

        // 표 아래에 빈 문단 추가 (HWP 표준, 한컴 blank_h_saved.hwp 참조)
        let mut empty_raw_header_extra = vec![0u8; 10];
        empty_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes());
        empty_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes());
        let empty_para = Paragraph {
            text: String::new(),
            char_count: 1,
            char_count_msb: false,
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
                segment_width: content_width as i32, // 한컴 표준: 편집 영역 폭
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            has_para_text: false,
            raw_header_extra: empty_raw_header_extra,
            ..Default::default()
        };
        self.document.sections[section_idx]
            .paragraphs
            .insert(insert_para_idx + 1, empty_para);

        // --- 6. 스타일 갱신 + 리플로우 + 페이지네이션 ---
        // 새 BorderFill 추가 시 styles.border_styles 갱신이 필요하므로 rebuild_section 사용
        self.rebuild_section(section_idx);

        self.event_log.push(DocumentEvent::TableRowInserted {
            section: section_idx,
            para: insert_para_idx,
            ctrl: 0,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":0",
            insert_para_idx
        )))
    }

    /// 커서 위치에 표를 삽입한다 (확장, JSON 옵션).
    ///
    /// 기본 create_table_native의 확장판으로, treat_as_char(인라인) 등 세부 속성을 지정할 수 있다.
    /// treat_as_char=true인 경우:
    ///   - 별도 문단을 생성하지 않고 기존 문단의 controls에 표를 추가
    ///   - 텍스트 흐름에 8 UTF-16 코드유닛 자리를 삽입
    ///   - 표 아래 빈 문단 미생성
    pub fn create_table_ex_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        row_count: u16,
        col_count: u16,
        treat_as_char: bool,
        col_widths_hu: Option<&[u32]>,
        row_heights_hu: Option<&[u32]>,
    ) -> Result<String, HwpError> {
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::style::{BorderFill, BorderLine, BorderLineType, DiagonalLine, Fill};
        use crate::model::table::{Cell, Table, TablePageBreak};

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
        if row_count == 0 || col_count == 0 || col_count > 256 {
            return Err(HwpError::RenderError(format!(
                "행/열 수 범위 오류 (행={}, 열={})",
                row_count, col_count
            )));
        }

        if !treat_as_char {
            return self.create_table_native(
                section_idx,
                para_idx,
                char_offset,
                row_count,
                col_count,
            );
        }

        // ── 인라인 TAC 표 생성 ──

        let pd = &self.document.sections[section_idx].section_def.page_def;
        let outer_margin: i16 = 283;
        let outer_margin_lr = (outer_margin * 2) as i32;
        let content_width =
            (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32 - outer_margin_lr)
                .max(7200) as u32;

        // 열 폭 결정
        let col_ws: Vec<u32> = if let Some(widths) = col_widths_hu {
            if widths.len() == col_count as usize {
                widths.to_vec()
            } else {
                let w = content_width / col_count as u32;
                vec![w; col_count as usize]
            }
        } else {
            let w = content_width / col_count as u32;
            vec![w; col_count as usize]
        };
        let total_width: u32 = col_ws.iter().sum();

        let cell_pad = crate::model::Padding {
            left: 510,
            right: 510,
            top: 141,
            bottom: 141,
        };
        let min_row_height: u32 = cell_pad.top as u32 + 1000 + cell_pad.bottom as u32;
        let row_heights: Vec<u32> = if let Some(heights) = row_heights_hu {
            if heights.len() == row_count as usize {
                heights.iter().map(|h| (*h).max(min_row_height)).collect()
            } else {
                vec![min_row_height; row_count as usize]
            }
        } else {
            vec![min_row_height; row_count as usize]
        };
        let total_height: u32 = row_heights.iter().sum();

        // BorderFill
        let cell_border_fill_id = {
            let existing = self.document.doc_info.border_fills.iter().position(|bf| {
                bf.borders
                    .iter()
                    .all(|b| b.line_type == BorderLineType::Solid && b.width >= 1)
            });
            if let Some(idx) = existing {
                (idx + 1) as u16
            } else {
                let solid_border = BorderLine {
                    line_type: BorderLineType::Solid,
                    width: 1,
                    color: 0,
                };
                let new_bf = BorderFill {
                    raw_data: None,
                    attr: 0,
                    borders: [solid_border, solid_border, solid_border, solid_border],
                    diagonal: DiagonalLine {
                        diagonal_type: 1,
                        width: 0,
                        color: 0,
                    },
                    fill: Fill::default(),
                };
                self.document.doc_info.border_fills.push(new_bf);
                self.document.doc_info.raw_stream = None;
                self.document.doc_info.border_fills.len() as u16
            }
        };

        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para
            .char_shapes
            .first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 셀 생성
        let mut cells = Vec::with_capacity((row_count as usize) * (col_count as usize));
        for r in 0..row_count {
            let row_height = row_heights[r as usize];
            for c in 0..col_count {
                let col_w = col_ws[c as usize];
                let mut cell = Cell::new_empty(c, r, col_w, row_height, cell_border_fill_id);
                cell.padding = cell_pad;
                cell.vertical_align = crate::model::table::VerticalAlign::Center;
                for cp in &mut cell.paragraphs {
                    cp.char_count_msb = true;
                    cp.para_shape_id = default_para_shape_id;
                    if cp.raw_header_extra.len() < 10 {
                        let mut rhe = vec![0u8; 10];
                        rhe[0..2].copy_from_slice(&1u16.to_le_bytes());
                        rhe[4..6].copy_from_slice(&1u16.to_le_bytes());
                        cp.raw_header_extra = rhe;
                    }
                    let seg_w = (col_w as i32) - 141 - 141;
                    let text_height =
                        row_height.saturating_sub((cell_pad.top + cell_pad.bottom) as u32);
                    cp.line_segs = vec![LineSeg {
                        text_start: 0,
                        line_height: text_height as i32,
                        text_height: text_height as i32,
                        baseline_distance: (text_height as f64 * 0.85) as i32,
                        line_spacing: 600,
                        segment_width: seg_w,
                        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                        ..Default::default()
                    }];
                }
                cell.raw_list_extra = Vec::new();
                cells.push(cell);
            }
        }

        // Table 구조체
        let row_sizes: Vec<i16> = (0..row_count).map(|_| col_count as i16).collect();
        // raw_ctrl_data: treat_as_char + vert=Page(0) + horz=Para(3) + wrap=TopAndBottom(1)
        #[allow(clippy::identity_op)]
        let flags: u32 = (1 << 0) /* treat_as_char */
            | (0 << 3) /* vert=Page */
            | (3 << 8) /* horz=Para */
            | (4 << 15) /* width_criterion=Absolute */
            | (2 << 18) /* height_criterion=Absolute */
            | (1 << 21) /* wrap=TopAndBottom */;
        let mut raw_ctrl_data = vec![0u8; 38];
        raw_ctrl_data[common_obj_offsets::FLAGS].copy_from_slice(&flags.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::HEIGHT].copy_from_slice(&total_height.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_LEFT].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_RIGHT]
            .copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_TOP].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[common_obj_offsets::MARGIN_BOTTOM]
            .copy_from_slice(&outer_margin.to_le_bytes());
        let instance_id: u32 = {
            let mut h: u32 = 0x7c160000;
            h = h.wrapping_add(row_count as u32 * 0x1000);
            h = h.wrapping_add(col_count as u32 * 0x100);
            h = h.wrapping_add(total_width);
            if h == 0 {
                h = 0x7c164b69;
            }
            h
        };
        raw_ctrl_data[common_obj_offsets::INSTANCE_ID].copy_from_slice(&instance_id.to_le_bytes());

        let mut table = Table {
            attr: 0x04000006,
            row_count,
            col_count,
            cell_spacing: 0,
            padding: cell_pad,
            row_sizes,
            border_fill_id: cell_border_fill_id,
            zones: Vec::new(),
            cells,
            cell_grid: Vec::new(),
            page_break: TablePageBreak::RowBreak,
            repeat_header: false,
            caption: None,
            common: crate::model::shape::CommonObjAttr {
                treat_as_char: true,
                text_wrap: crate::model::shape::TextWrap::TopAndBottom,
                vert_rel_to: crate::model::shape::VertRelTo::Page,
                horz_rel_to: crate::model::shape::HorzRelTo::Para,
                vert_align: crate::model::shape::VertAlign::Top,
                horz_align: crate::model::shape::HorzAlign::Left,
                width: total_width,
                height: total_height,
                ..Default::default()
            },
            outer_margin_left: outer_margin,
            outer_margin_right: outer_margin,
            outer_margin_top: outer_margin,
            outer_margin_bottom: outer_margin,
            raw_ctrl_data,
            raw_table_record_attr: 0x04000006,
            raw_table_record_extra: vec![0u8; 2],
            dirty: true,
            local_resize_rows: Vec::new(),
            local_resize_cols: Vec::new(),
            local_resize_cell_widths: Vec::new(),
            local_resize_cell_heights: Vec::new(),
        };
        table.rebuild_grid();

        // ── 기존 문단에 인라인 삽입 ──
        self.document.sections[section_idx].raw_stream = None;
        let para = &mut self.document.sections[section_idx].paragraphs[para_idx];

        // controls에 표 추가
        let ctrl_idx = para.controls.len();
        para.controls.push(Control::Table(Box::new(table)));
        para.ctrl_data_records.push(None);

        // char_offsets에 8 UTF-16 코드유닛 갭 삽입
        // 확장 제어문자는 8 코드유닛을 차지
        let insert_utf16_pos = if char_offset < para.char_offsets.len() {
            para.char_offsets[char_offset]
        } else if !para.char_offsets.is_empty() {
            let last_idx = para.char_offsets.len() - 1;
            let last_char_len = para
                .text
                .chars()
                .nth(last_idx)
                .map(|c| c.len_utf16() as u32)
                .unwrap_or(1);
            para.char_offsets[last_idx] + last_char_len
        } else {
            0
        };

        // 이후 char_offsets를 8만큼 shift
        for offset in para.char_offsets.iter_mut() {
            if *offset >= insert_utf16_pos {
                *offset += 8;
            }
        }

        // char_count 갱신 (확장 제어문자 8 + 기존)
        para.char_count += 8;

        // LINE_SEG 갱신: 표 높이를 반영
        if let Some(seg) = para.line_segs.first_mut() {
            let new_lh = (total_height as i32).max(seg.line_height);
            if new_lh > seg.line_height {
                seg.line_height = new_lh;
                seg.text_height = new_lh;
                seg.baseline_distance = (new_lh as f64 * 0.85) as i32;
            }
        }

        // rebuild
        self.rebuild_section(section_idx);

        self.event_log.push(DocumentEvent::TableRowInserted {
            section: section_idx,
            para: para_idx,
            ctrl: ctrl_idx,
        });
        // 표 바로 뒤의 논리적 오프셋 계산
        let logical_after = super::super::helpers::text_to_logical_offset(
            &self.document.sections[section_idx].paragraphs[para_idx],
            char_offset,
        ) + 1;
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{},\"logicalOffset\":{}",
            para_idx, ctrl_idx, logical_after
        )))
    }

    /// 커서 위치에 그림을 삽입한다 (네이티브).
    ///
    /// - `cell_path` 가 비어있으면 본문 paragraph 에 inline (treat_as_char=true) 삽입.
    /// - `cell_path` 가 있으면 표 셀 영역에 floating picture (tac=false, wrap=Square,
    ///   Page-relative offset) 로 삽입한다. 셀 자체는 비어있는 채로 유지되어 cursor
    ///   클릭이 정상 동작 (#1151). 한컴 2022 의 셀 이미지 삽입 패턴과 동일
    ///   (incellpicture.hwp 검증).
    ///
    /// `paper_offset_x_hu / paper_offset_y_hu`: 셀 floating 분기에서 사용할 paper-relative
    /// 좌표 (HWPUNIT). `None` 이면 셀 좌상단 (`compute_cell_page_offset`) 을 default 로 사용
    /// — 기존 동작 + API caller 호환. studio drag 좌표 기반 호출은 `Some` 으로 전달.
    /// 본문 inline 분기 (cell_path 비어있음) 는 본 매개변수를 사용하지 않는다.
    #[allow(clippy::too_many_arguments)]
    pub fn insert_picture_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        cell_path: &[(usize, usize, usize)],
        image_data: &[u8],
        width: u32,
        height: u32,
        natural_width_px: u32,
        natural_height_px: u32,
        extension: &str,
        description: &str,
        paper_offset_x_hu: Option<i32>,
        paper_offset_y_hu: Option<i32>,
    ) -> Result<String, HwpError> {
        use crate::model::bin_data::{
            BinData, BinDataCompression, BinDataContent, BinDataStatus, BinDataType,
        };
        use crate::model::image::{CropInfo, ImageAttr, ImageEffect, Picture};
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::shape::{CommonObjAttr, HorzRelTo, ShapeComponentAttr, VertRelTo};
        // 유효성 검사
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과",
                para_idx
            )));
        }
        if image_data.is_empty() {
            return Err(HwpError::RenderError(
                "이미지 데이터가 비어 있습니다".to_string(),
            ));
        }
        // cell_path 가 있으면 경로가 유효한지 사전 검증한다.
        //
        // 표 셀 picture 는 한컴 정합상 표 sibling floating 으로 삽입하지만,
        // 글상자(text_box) 내부 picture 는 글상자 문단의 control 로 들어가야 한다.
        // 기존 resolve_cell_by_path 는 마지막 엔트리가 표일 때만 성공하므로
        // 먼저 표/글상자를 구분한다.
        let cell_path_is_textbox = if !cell_path.is_empty() {
            let section = &self.document.sections[section_idx];
            let is_textbox = Self::cell_path_terminates_at_textbox(section, para_idx, cell_path)?;
            if !is_textbox {
                self.resolve_cell_by_path(section_idx, para_idx, cell_path)?;
            }
            is_textbox
        } else {
            false
        };

        // --- 1. BinDataContent 추가 ---
        let next_id = self.document.bin_data_content.len() as u16 + 1;
        self.document.bin_data_content.push(BinDataContent {
            id: next_id,
            data: image_data.to_vec(),
            extension: extension.to_string(),
        });

        // --- 2. BinData 메타데이터 추가 ---
        // attr: bits 0-3=1(Embedding), bits 4-5=0(Default), bits 8-9=1(Success)
        let bin_attr: u16 = 0x0101;
        self.document.doc_info.bin_data_list.push(BinData {
            raw_data: None,
            attr: bin_attr,
            data_type: BinDataType::Embedding,
            compression: BinDataCompression::Default,
            status: BinDataStatus::Success,
            abs_path: None,
            rel_path: None,
            storage_id: next_id,
            extension: Some(extension.to_string()),
        });
        self.document.doc_info.raw_stream = None; // DocInfo 재직렬화

        // --- 공통 자원 ---
        let shape_attr = ShapeComponentAttr {
            original_width: width,
            original_height: height,
            current_width: width,
            current_height: height,
            local_file_version: 1,
            render_sx: 1.0,
            render_sy: 1.0,
            ..Default::default()
        };
        let bx = [0i32, 0, width as i32, 0];
        let by = [width as i32, height as i32, 0, height as i32];
        let crop = CropInfo {
            left: 0,
            top: 0,
            right: (natural_width_px * 75) as i32,
            bottom: (natural_height_px * 75) as i32,
        };
        let image_attr = ImageAttr {
            bin_data_id: next_id,
            brightness: 0,
            contrast: 0,
            effect: ImageEffect::RealPic,
            transparency: 0,
            external_path: None,
        };

        if !cell_path.is_empty() {
            if cell_path_is_textbox {
                // === 글상자 내부 picture 분기 (#1322 maintainer fix) ===
                // hitTest 의 글상자 sentinel path (`cellIdx=0`) 가 넘어온 경우에는
                // Picture 를 body paragraph 의 sibling 으로 띄우지 않고, 실제 text_box
                // paragraph 안에 삽입한다. 글상자 내부 좌표계는 text_box content box
                // 기준이므로 caller 가 전달한 offset 은 Para-relative 로 해석한다.
                let (offset_x_hu, offset_y_hu) = match (paper_offset_x_hu, paper_offset_y_hu) {
                    (Some(x), Some(y)) => (x, y),
                    _ => (0, 0),
                };

                // CommonObjAttr (text_box 내부 floating):
                //   bits 3-4=vert_rel_to(2=Para), bits 8-10=horz_rel_to(3=Para),
                //   bits 15-17=width_criterion(4=Absolute),
                //   bits 18-20=height_criterion(2=Absolute),
                //   bits 21-23=text_wrap(0=Square)
                let common_attr: u32 = (2 << 3) | (3 << 8) | (4 << 15) | (2 << 18);
                let common = CommonObjAttr {
                    ctrl_id: 0x67736F20,
                    attr: common_attr,
                    treat_as_char: false,
                    vert_rel_to: VertRelTo::Para,
                    horz_rel_to: HorzRelTo::Para,
                    text_wrap: crate::model::shape::TextWrap::Square,
                    horizontal_offset: offset_x_hu.max(0) as u32,
                    vertical_offset: offset_y_hu.max(0) as u32,
                    width,
                    height,
                    z_order: 1,
                    description: description.to_string(),
                    ..Default::default()
                };
                let pic = Picture {
                    common,
                    shape_attr,
                    border_x: bx,
                    border_y: by,
                    crop,
                    image_attr,
                    ..Default::default()
                };

                let (new_ctrl_idx, logical_after) = {
                    let section = &mut self.document.sections[section_idx];
                    section.raw_stream = None;
                    let target_para =
                        Self::resolve_cell_paragraph_mut(section, para_idx, cell_path)?;
                    let new_ctrl_idx = target_para.controls.len();
                    target_para.controls.push(Control::Picture(Box::new(pic)));
                    target_para.ctrl_data_records.push(None);
                    target_para.control_mask |= 0x00000800;
                    let logical_positions =
                        super::super::helpers::find_logical_control_positions(target_para);
                    let logical_after = logical_positions
                        .get(new_ctrl_idx)
                        .copied()
                        .unwrap_or_else(|| target_para.text.chars().count())
                        + 1;
                    (new_ctrl_idx, logical_after)
                };

                self.mark_section_dirty(section_idx);
                self.recompose_section(section_idx);
                self.paginate_if_needed();
                self.invalidate_page_tree_cache();

                self.event_log.push(DocumentEvent::PictureInserted {
                    section: section_idx,
                    para: para_idx,
                });
                return Ok(super::super::helpers::json_ok_with(&format!(
                    "\"paraIdx\":{},\"controlIdx\":{},\"logicalOffset\":{}",
                    para_idx, new_ctrl_idx, logical_after
                )));
            }

            // === 셀 floating picture 분기 (#1151 v2 — 한컴 패턴 정합) ===
            // Picture 는 표가 들어있는 paragraph 의 sibling control 로 append 된다.
            // tac=false, wrap=Square (어울림), horz/vert_rel_to=Paper, offset 은 사용자 클릭/드래그 위치.
            // [Task #1151 v8] 결함 A fix: 한컴 native default 가 Paper (incellpicture.hwp dump
            // 확인 — horz_rel_to=Paper offset=11845, vert_rel_to=Paper offset=15595).
            // [Task #1151 v8] 결함 C fix: 사용자가 클릭/드래그한 좌표 (paper-relative HU) 사용 —
            // 한컴 native 동작 정합. caller (studio) 가 None 전달 시 셀 좌상단 default.
            let (offset_x_hu, offset_y_hu) = match (paper_offset_x_hu, paper_offset_y_hu) {
                (Some(x), Some(y)) => (x, y),
                _ => self.compute_cell_page_offset(section_idx, para_idx, cell_path),
            };

            // CommonObjAttr (floating):
            //   bits 3-4=vert_rel_to(0=Paper), bits 8-10=horz_rel_to(0=Paper),
            //   bits 15-17=width_criterion(4=Absolute), bits 18-20=height_criterion(2=Absolute),
            //   bits 21-23=text_wrap(0=Square)
            let common_attr: u32 = (4 << 15) | (2 << 18);
            let common = CommonObjAttr {
                ctrl_id: 0x67736F20,
                attr: common_attr,
                treat_as_char: false,
                vert_rel_to: VertRelTo::Paper,
                horz_rel_to: HorzRelTo::Paper,
                text_wrap: crate::model::shape::TextWrap::Square,
                horizontal_offset: offset_x_hu.max(0) as u32,
                vertical_offset: offset_y_hu.max(0) as u32,
                width,
                height,
                z_order: 1,
                description: description.to_string(),
                ..Default::default()
            };
            let pic = Picture {
                common,
                shape_attr,
                border_x: bx,
                border_y: by,
                crop,
                image_attr,
                ..Default::default()
            };

            // table 같은 paragraph 의 sibling control 로 append.
            self.document.sections[section_idx].raw_stream = None;
            let parent = &mut self.document.sections[section_idx].paragraphs[para_idx];
            let new_ctrl_idx = parent.controls.len();
            parent.controls.push(Control::Picture(Box::new(pic)));
            parent.ctrl_data_records.push(None);
            let logical_positions = super::super::helpers::find_logical_control_positions(parent);
            let logical_after = logical_positions
                .get(new_ctrl_idx)
                .copied()
                .unwrap_or_else(|| parent.text.chars().count())
                + 1;

            // outer table dirty 마킹 (재측정 유도)
            let outer_ctrl = cell_path[0].0;
            if let Some(Control::Table(t)) = self.document.sections[section_idx].paragraphs
                [para_idx]
                .controls
                .get_mut(outer_ctrl)
            {
                t.dirty = true;
            }
            self.mark_section_dirty(section_idx);
            self.paginate_if_needed();
            // [Task #1151 v9 결함 F] page tree cache invalidate — v5 와 동일 결함 (다른
            // setter 들은 모두 호출하나 본 insert path 의 셀 분기만 누락). 두 picture
            // 연속 insert + toggle 시 cache stale → studio 화면 불일치.
            self.invalidate_page_tree_cache();

            self.event_log.push(DocumentEvent::PictureInserted {
                section: section_idx,
                para: para_idx,
            });
            return Ok(super::super::helpers::json_ok_with(&format!(
                "\"paraIdx\":{},\"controlIdx\":{},\"logicalOffset\":{}",
                para_idx, new_ctrl_idx, logical_after
            )));
        }

        // === 본문 floating picture 분기 (Task #1151 v9 결함 E — 셀 분기와 동일 패턴) ===
        //
        // 한컴 native 동작 (사용자 시연 2026-05-30): 본문 picture 신규 삽입 시
        // 글자처럼 취급 default = **미체크** (tac=false, floating). 셀 안 picture
        // 와 동일. 이전 rhwp 본문 path 는 새 paragraph 생성 + inline glyph (tac=true)
        // 로 만들어 한컴 default 와 불일치 — 재설계하여 셀 분기와 통합.
        let (offset_x_hu, offset_y_hu) = match (paper_offset_x_hu, paper_offset_y_hu) {
            (Some(x), Some(y)) => (x, y),
            _ => (0, 0),
        };

        // CommonObjAttr (floating, 셀 분기와 동일):
        //   bits 3-4=vert_rel_to(0=Paper), bits 8-10=horz_rel_to(0=Paper),
        //   bits 15-17=width_criterion(4=Absolute), bits 18-20=height_criterion(2=Absolute),
        //   bits 21-23=text_wrap(0=Square)
        let common_attr: u32 = (4 << 15) | (2 << 18);
        let common = CommonObjAttr {
            ctrl_id: 0x67736F20, // "gso " — GenShape
            attr: common_attr,
            treat_as_char: false,
            vert_rel_to: VertRelTo::Paper,
            horz_rel_to: HorzRelTo::Paper,
            text_wrap: crate::model::shape::TextWrap::Square,
            horizontal_offset: offset_x_hu.max(0) as u32,
            vertical_offset: offset_y_hu.max(0) as u32,
            width,
            height,
            z_order: 1,
            description: description.to_string(),
            ..Default::default()
        };

        let pic = Picture {
            common,
            shape_attr,
            border_x: bx,
            border_y: by,
            crop,
            image_attr,
            ..Default::default()
        };

        // 현재 paragraph 의 sibling control 로 append (새 paragraph 생성 X).
        self.document.sections[section_idx].raw_stream = None;
        let parent = &mut self.document.sections[section_idx].paragraphs[para_idx];
        let new_ctrl_idx = parent.controls.len();
        parent.controls.push(Control::Picture(Box::new(pic)));
        parent.ctrl_data_records.push(None);
        let logical_positions = super::super::helpers::find_logical_control_positions(parent);
        let logical_after = logical_positions
            .get(new_ctrl_idx)
            .copied()
            .unwrap_or_else(|| parent.text.chars().count())
            + 1;

        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();
        // [Task #1151 v9 결함 F] page tree cache invalidate (v5 패턴).
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{},\"logicalOffset\":{}",
            para_idx, new_ctrl_idx, logical_after
        )))
    }

    /// 표 셀의 page-relative 좌상단 좌표를 HWPUNIT 단위로 계산 (#1151 floating).
    ///
    /// render tree 를 순회하여 cell_path 와 매칭되는 TableCell 노드를 찾고
    /// bbox.x / bbox.y (px) 를 HWPUNIT 으로 환산 (× 75).
    ///
    /// 매칭 실패 / 페이지 미빌드 시 (0, 0) fallback — picture 가 페이지 좌상단에
    /// 떠도 사용자가 드래그로 위치 조정 가능.
    pub(crate) fn compute_cell_page_offset(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path: &[(usize, usize, usize)],
    ) -> (i32, i32) {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        if cell_path.is_empty() {
            return (0, 0);
        }

        fn matches_cell_run(
            node: &RenderNode,
            parent_para: usize,
            path: &[(usize, usize, usize)],
        ) -> bool {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                return tr.cell_context.as_ref().is_some_and(|ctx| {
                    ctx.parent_para_index == parent_para
                        && ctx.path.len() == path.len()
                        && ctx
                            .path
                            .iter()
                            .zip(path.iter())
                            .all(|(a, b)| a.control_index == b.0 && a.cell_index == b.1)
                });
            }
            for child in &node.children {
                if matches!(child.node_type, RenderNodeType::Table(_)) {
                    continue;
                }
                if matches_cell_run(child, parent_para, path) {
                    return true;
                }
            }
            false
        }

        fn find_cell(
            node: &RenderNode,
            parent_para: usize,
            path: &[(usize, usize, usize)],
        ) -> Option<(f64, f64)> {
            if let RenderNodeType::Table(_) = node.node_type {
                if matches_cell_run(node, parent_para, path) {
                    let target_cell = path.last().unwrap().1;
                    for child in node.children.iter() {
                        if let RenderNodeType::TableCell(ref tc) = child.node_type {
                            if tc.model_cell_index == Some(target_cell as u32) {
                                return Some((child.bbox.x, child.bbox.y));
                            }
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(found) = find_cell(child, parent_para, path) {
                    return Some(found);
                }
            }
            None
        }

        let total_pages = self.page_count();
        for p in 0..total_pages {
            if let Ok(tree) = self.build_page_tree(p) {
                if let Some((px, py)) = find_cell(&tree.root, parent_para_idx, cell_path) {
                    // px → HWPUNIT (1px = 75 HWPUNIT at 96 DPI 가정).
                    // 단, section_idx 가 의미 있는 단위 정합을 위해 section 자체의
                    // 보정은 호출 측 (Picture.horz/vert_rel_to=Page) 가 처리.
                    let _ = section_idx;
                    return ((px * 75.0) as i32, (py * 75.0) as i32);
                }
            }
        }
        (0, 0)
    }

    /// 표의 모든 셀 bbox를 반환한다 (네이티브).
    pub(crate) fn get_table_cell_bboxes_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        self.get_table_cell_bboxes_from_page(section_idx, parent_para_idx, control_idx, 0)
    }

    /// page_hint부터 탐색하여 표의 셀 bbox를 반환한다 (네이티브).
    /// page_hint에서 못 찾으면 앞쪽도 탐색한다 (페이지 분할된 표 대응).
    pub(crate) fn get_table_cell_bboxes_from_page(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        page_hint: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        // 렌더 트리에서 해당 표 노드를 찾아 셀 bbox를 수집
        fn find_table_cells(
            node: &RenderNode,
            sec: usize,
            ppi: usize,
            ci: usize,
            page_idx: usize,
            result: &mut Vec<String>,
        ) -> bool {
            if let RenderNodeType::Table(ref tn) = node.node_type {
                if tn.section_index == Some(sec)
                    && tn.para_index == Some(ppi)
                    && tn.control_index == Some(ci)
                {
                    for (_child_idx, child) in node.children.iter().enumerate() {
                        if let RenderNodeType::TableCell(ref cn) = child.node_type {
                            // cellIdx: 모델의 cells 배열에서 (row, col)로 검색한 인덱스
                            let model_cell_idx = cn.model_cell_index.unwrap_or(0) as usize;
                            result.push(format!(
                                "{{\"cellIdx\":{},\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{},\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}}}",
                                model_cell_idx, cn.row, cn.col, cn.row_span, cn.col_span,
                                page_idx,
                                child.bbox.x, child.bbox.y, child.bbox.width, child.bbox.height
                            ));
                        }
                    }
                    return true; // 찾음
                }
            }
            for child in &node.children {
                if find_table_cells(child, sec, ppi, ci, page_idx, result) {
                    return true;
                }
            }
            false
        }

        let mut cells = Vec::new();
        let total_pages = self.page_count() as usize;
        let start = page_hint.min(total_pages.saturating_sub(1));

        // page_hint부터 뒤쪽 탐색
        let mut found = false;
        for page_num in start..total_pages {
            let tree = self.build_page_tree_cached(page_num as u32)?;
            if find_table_cells(
                &tree.root,
                section_idx,
                parent_para_idx,
                control_idx,
                page_num,
                &mut cells,
            ) {
                found = true;
            } else if found {
                break;
            }
        }

        // page_hint에서 못 찾았으면 앞쪽 탐색 (페이지 분할 표가 hint 이전 페이지에서 시작될 수 있음)
        if !found && start > 0 {
            for page_num in (0..start).rev() {
                let tree = self.build_page_tree_cached(page_num as u32)?;
                if find_table_cells(
                    &tree.root,
                    section_idx,
                    parent_para_idx,
                    control_idx,
                    page_num,
                    &mut cells,
                ) {
                    found = true;
                    // 이 페이지에서 찾음 — hint까지 다시 정방향 탐색하여 누락된 페이지 수집
                    for fwd in (page_num + 1)..=start {
                        let tree2 = self.build_page_tree_cached(fwd as u32)?;
                        if !find_table_cells(
                            &tree2.root,
                            section_idx,
                            parent_para_idx,
                            control_idx,
                            fwd,
                            &mut cells,
                        ) {
                            break;
                        }
                    }
                    break;
                }
            }
        }

        Ok(format!("[{}]", cells.join(",")))
    }

    // ── 글상자(Shape) CRUD ─────────────────────────────────

    /// CommonObjAttr → JSON 문자열 (Shape/Picture 공용 속성)
    fn common_obj_attr_to_json(c: &crate::model::shape::CommonObjAttr) -> String {
        let vert_rel = match c.vert_rel_to {
            crate::model::shape::VertRelTo::Paper => "Paper",
            crate::model::shape::VertRelTo::Page => "Page",
            crate::model::shape::VertRelTo::Para => "Para",
        };
        let vert_align = match c.vert_align {
            crate::model::shape::VertAlign::Top => "Top",
            crate::model::shape::VertAlign::Center => "Center",
            crate::model::shape::VertAlign::Bottom => "Bottom",
            crate::model::shape::VertAlign::Inside => "Inside",
            crate::model::shape::VertAlign::Outside => "Outside",
        };
        let horz_rel = match c.horz_rel_to {
            crate::model::shape::HorzRelTo::Paper => "Paper",
            crate::model::shape::HorzRelTo::Page => "Page",
            crate::model::shape::HorzRelTo::Column => "Column",
            crate::model::shape::HorzRelTo::Para => "Para",
        };
        let horz_align = match c.horz_align {
            crate::model::shape::HorzAlign::Left => "Left",
            crate::model::shape::HorzAlign::Center => "Center",
            crate::model::shape::HorzAlign::Right => "Right",
            crate::model::shape::HorzAlign::Inside => "Inside",
            crate::model::shape::HorzAlign::Outside => "Outside",
        };
        let text_wrap = match c.text_wrap {
            crate::model::shape::TextWrap::Square => "Square",
            crate::model::shape::TextWrap::Tight => "Tight",
            crate::model::shape::TextWrap::Through => "Through",
            crate::model::shape::TextWrap::TopAndBottom => "TopAndBottom",
            crate::model::shape::TextWrap::BehindText => "BehindText",
            crate::model::shape::TextWrap::InFrontOfText => "InFrontOfText",
        };
        let desc_escaped = super::super::helpers::json_escape(&c.description);
        format!(
            "\"width\":{},\"height\":{},\"treatAsChar\":{},\
             \"vertRelTo\":\"{}\",\"vertAlign\":\"{}\",\
             \"horzRelTo\":\"{}\",\"horzAlign\":\"{}\",\
             \"vertOffset\":{},\"horzOffset\":{},\
             \"textWrap\":\"{}\",\"restrictInPage\":{},\"allowOverlap\":{},\"sizeProtect\":{},\
             \"zOrder\":{},\"instanceId\":{},\
             \"outerMarginLeft\":{},\"outerMarginTop\":{},\
             \"outerMarginRight\":{},\"outerMarginBottom\":{},\
             \"description\":\"{}\"",
            c.width,
            c.height,
            c.treat_as_char,
            vert_rel,
            vert_align,
            horz_rel,
            horz_align,
            c.vertical_offset,
            c.horizontal_offset,
            text_wrap,
            c.flow_with_text,
            c.allow_overlap,
            c.size_protect,
            c.z_order,
            c.instance_id,
            c.margin.left,
            c.margin.top,
            c.margin.right,
            c.margin.bottom,
            desc_escaped,
        )
    }

    /// JSON → CommonObjAttr 필드 업데이트 (Shape/Picture 공용)
    fn apply_common_obj_attr_from_json(
        c: &mut crate::model::shape::CommonObjAttr,
        props_json: &str,
    ) {
        use super::super::helpers::{json_bool, json_i16, json_str, json_u32};

        if let Some(w) = json_u32(props_json, "width") {
            c.width = w.max(MIN_SHAPE_SIZE);
        }
        if let Some(h) = json_u32(props_json, "height") {
            c.height = h.max(MIN_SHAPE_SIZE);
        }
        if let Some(tac) = json_bool(props_json, "treatAsChar") {
            c.treat_as_char = tac;
            if tac {
                c.attr |= 0x01;
            } else {
                c.attr &= !0x01;
            }
        }
        if let Some(v) = json_str(props_json, "vertRelTo") {
            c.vert_rel_to = match v.as_str() {
                "Paper" => crate::model::shape::VertRelTo::Paper,
                "Page" => crate::model::shape::VertRelTo::Page,
                "Para" => crate::model::shape::VertRelTo::Para,
                _ => c.vert_rel_to,
            };
        }
        if let Some(v) = json_str(props_json, "horzRelTo") {
            c.horz_rel_to = match v.as_str() {
                "Paper" => crate::model::shape::HorzRelTo::Paper,
                "Page" => crate::model::shape::HorzRelTo::Page,
                "Column" => crate::model::shape::HorzRelTo::Column,
                "Para" => crate::model::shape::HorzRelTo::Para,
                _ => c.horz_rel_to,
            };
        }
        if let Some(v) = json_str(props_json, "vertAlign") {
            c.vert_align = match v.as_str() {
                "Top" => crate::model::shape::VertAlign::Top,
                "Center" => crate::model::shape::VertAlign::Center,
                "Bottom" => crate::model::shape::VertAlign::Bottom,
                _ => c.vert_align,
            };
        }
        if let Some(v) = json_str(props_json, "horzAlign") {
            c.horz_align = match v.as_str() {
                "Left" => crate::model::shape::HorzAlign::Left,
                "Center" => crate::model::shape::HorzAlign::Center,
                "Right" => crate::model::shape::HorzAlign::Right,
                _ => c.horz_align,
            };
        }
        if let Some(v) = json_str(props_json, "textWrap") {
            c.text_wrap = match v.as_str() {
                "Square" => crate::model::shape::TextWrap::Square,
                "Tight" => crate::model::shape::TextWrap::Tight,
                "Through" => crate::model::shape::TextWrap::Through,
                "TopAndBottom" => crate::model::shape::TextWrap::TopAndBottom,
                "BehindText" => crate::model::shape::TextWrap::BehindText,
                "InFrontOfText" => crate::model::shape::TextWrap::InFrontOfText,
                _ => c.text_wrap,
            };
        }
        if let Some(v) = json_bool(props_json, "restrictInPage") {
            c.flow_with_text = v;
            if v {
                c.attr |= 1 << 13;
                c.allow_overlap = false;
                c.attr &= !(1 << 14);
            } else {
                c.attr &= !(1 << 13);
            }
        }
        if let Some(v) = json_bool(props_json, "allowOverlap") {
            c.allow_overlap = v;
            if v {
                c.attr |= 1 << 14;
            } else {
                c.attr &= !(1 << 14);
            }
        }
        if let Some(v) = json_bool(props_json, "sizeProtect") {
            c.size_protect = v;
            if v {
                c.attr |= 1 << 20;
            } else {
                c.attr &= !(1 << 20);
            }
        }
        if c.flow_with_text {
            c.allow_overlap = false;
            c.attr &= !(1 << 14);
        }
        if let Some(v) = json_u32(props_json, "vertOffset") {
            c.vertical_offset = v;
        }
        if let Some(v) = json_u32(props_json, "horzOffset") {
            c.horizontal_offset = v;
        }
        if let Some(v) = json_str(props_json, "description") {
            c.description = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginLeft") {
            c.margin.left = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginTop") {
            c.margin.top = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginRight") {
            c.margin.right = v;
        }
        if let Some(v) = json_i16(props_json, "outerMarginBottom") {
            c.margin.bottom = v;
        }
        Self::sync_common_obj_attr_known_bits(c);
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

        Ok(format!(
            "{{{}{}{}{}{}}}",
            common_json, tb_json, extra_json, round_json, connector_json
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
        use super::super::helpers::{json_bool, json_i32, json_str};

        let shape = self.resolve_shape_control_mut(section_idx, parent_para_idx, control_idx)?;

        // CommonObjAttr 업데이트
        // 리사이즈 핸들을 반대편으로 끌어당길 때 studio가 width/height=0 을 보내
        // 도형이 렌더러상 사라지는 버그 방어: 최소 크기 clamp.
        let c = shape.common_mut();
        let new_w =
            super::super::helpers::json_u32(props_json, "width").map(|w| w.max(MIN_SHAPE_SIZE));
        let new_h =
            super::super::helpers::json_u32(props_json, "height").map(|h| h.max(MIN_SHAPE_SIZE));
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
            if let Some(v) = super::super::helpers::json_u32(props_json, "shadowType") {
                d.shadow_type = v;
            }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowColor") {
                d.shadow_color = v as u32;
            }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowOffsetX") {
                d.shadow_offset_x = v;
            }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowOffsetY") {
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
            if let Some(v) = super::super::helpers::json_i32(props_json, "roundRate") {
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
    fn format_shape_props_inner(
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

        Ok(format!(
            "{{{}{}{}{}{}}}",
            common_json, tb_json, extra_json, round_json, connector_json
        ))
    }

    /// [Task #1138] Shape 속성 JSON 적용 (mutation only). 후처리 (recompose /
    /// paginate / cache invalidate / event log) 는 호출자 책임.
    /// set_shape_properties_native + set_cell_shape_properties_by_path_native 공유.
    fn apply_shape_props_inner(shape: &mut crate::model::shape::ShapeObject, props_json: &str) {
        use super::super::helpers::{json_bool, json_i32, json_str};

        let c = shape.common_mut();
        let new_w =
            super::super::helpers::json_u32(props_json, "width").map(|w| w.max(MIN_SHAPE_SIZE));
        let new_h =
            super::super::helpers::json_u32(props_json, "height").map(|h| h.max(MIN_SHAPE_SIZE));
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
            if let Some(v) = super::super::helpers::json_u32(props_json, "shadowType") {
                d.shadow_type = v;
            }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowColor") {
                d.shadow_color = v as u32;
            }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowOffsetX") {
                d.shadow_offset_x = v;
            }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowOffsetY") {
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
            if let Some(v) = super::super::helpers::json_i32(props_json, "roundRate") {
                rect.round_rate = v as u8;
            }
        }

        if let crate::model::shape::ShapeObject::Rectangle(ref mut rect) = shape {
            let w = rect.common.width as i32;
            let h = rect.common.height as i32;
            rect.x_coords = [0, w, w, 0];
            rect.y_coords = [0, 0, h, h];
        }

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
    }

    /// [Task #1138] 표 셀 내 Shape 속성 조회 (by_path).
    /// [Task #1151 v4] 셀 안 picture 속성 조회 (cell_path 기반).
    /// `get_cell_shape_properties_by_path_native` Picture 버전.
    pub fn get_cell_picture_properties_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        // [Task #1171] 표 셀과 글상자(Shape text_box) 를 모두 처리하는 resolver 사용.
        // (기존 resolve_cell_by_path 는 마지막 세그먼트가 표 셀이어야 했음.)
        let cell_para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, &path)?;
        let ctrl = cell_para.controls.get(inner_control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
        })?;
        let pic = match ctrl {
            Control::Picture(p) => p,
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 셀 내 컨트롤이 그림이 아닙니다".to_string(),
                ))
            }
        };
        Self::format_picture_properties_json(pic)
    }

    pub fn get_cell_shape_properties_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        let cell = self.resolve_cell_by_path(section_idx, parent_para_idx, &path)?;
        let last_cell_para_idx = path.last().unwrap().2;
        let cell_para = cell.paragraphs.get(last_cell_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("셀 내 문단 {} 범위 초과", last_cell_para_idx))
        })?;
        let ctrl = cell_para.controls.get(inner_control_idx).ok_or_else(|| {
            HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
        })?;
        let shape_ref = match ctrl {
            Control::Shape(s) => s.as_ref(),
            _ => {
                return Err(HwpError::RenderError(
                    "지정된 셀 내 컨트롤이 Shape이 아닙니다".to_string(),
                ))
            }
        };
        Self::format_shape_props_inner(shape_ref)
    }

    /// [Task #1138] 표 셀 내 Shape 속성 변경 (by_path).
    /// [Task #1151 v4] 셀 안 picture 속성 변경 (cell_path 기반).
    ///
    /// `set_cell_shape_properties_by_path_native` 와 동일 패턴 — 셀 path 순회 후
    /// inner_control_idx 의 Picture 에 대해 `apply_picture_props_inner` 적용.
    /// v2 의 tac 토글 마이그레이션 path 는 본 셀 안 picture path 에서는 적용되지
    /// 않는다 (셀 안 inline picture 는 이미 셀 안 위치에 있고, 한컴은 inline→floating
    /// 자동 변환을 별도 path 로 처리. 본 PR 의 v2 scope 는 floating→inline 만).
    pub fn set_cell_picture_properties_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        use super::super::helpers::{json_bool, json_i32};

        let path = Self::parse_cell_path_json(cell_path_json)?;
        let restrict_change = json_bool(props_json, "restrictInPage");
        let restrict_enabled_by_this_call = restrict_change.unwrap_or(false);
        let clamp_horz =
            restrict_enabled_by_this_call || json_i32(props_json, "horzOffset").is_some();
        let clamp_vert =
            restrict_enabled_by_this_call || json_i32(props_json, "vertOffset").is_some();
        {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let current_para = Self::resolve_cell_paragraph_mut(section, parent_para_idx, &path)?;
            let ctrl = current_para
                .controls
                .get_mut(inner_control_idx)
                .ok_or_else(|| {
                    HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
                })?;
            let pic = match ctrl {
                Control::Picture(p) => p,
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 셀 내 컨트롤이 그림이 아닙니다".to_string(),
                    ))
                }
            };
            Self::apply_picture_props_inner(pic, props_json);
        }
        let section = &mut self.document.sections[section_idx];
        Self::clamp_direct_owner_cell_picture_offsets(
            section,
            parent_para_idx,
            &path,
            inner_control_idx,
            clamp_horz,
            clamp_vert,
        )?;
        Self::sync_direct_owner_cell_for_picture(
            section,
            parent_para_idx,
            &path,
            inner_control_idx,
        )?;
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        let outer_table_ctrl = path.first().unwrap().0;
        self.event_log.push(DocumentEvent::PictureResized {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_table_ctrl,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// [Task #1171 / PR #1254] 표 셀/글상자 내부 Picture 삭제 (cell_path 기반).
    pub fn delete_cell_picture_control_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let para = Self::resolve_cell_paragraph_mut(section, parent_para_idx, &path)?;
            if inner_control_idx >= para.controls.len() {
                return Err(HwpError::RenderError(format!(
                    "셀 내 컨트롤 {} 범위 초과",
                    inner_control_idx
                )));
            }
            if !matches!(&para.controls[inner_control_idx], Control::Picture(_)) {
                return Err(HwpError::RenderError(
                    "지정된 셀 내 컨트롤이 그림이 아닙니다".to_string(),
                ));
            }

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
                    if ci == inner_control_idx {
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
                    if ci == inner_control_idx {
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

            para.controls.remove(inner_control_idx);
            if inner_control_idx < para.ctrl_data_records.len() {
                para.ctrl_data_records.remove(inner_control_idx);
            }
            if para.char_count >= 8 {
                para.char_count -= 8;
            }
            Self::reflow_paragraph_line_segs_after_control_delete(para, &self.styles, self.dpi);
        }

        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        let outer_ctrl = path.first().unwrap().0;
        self.event_log.push(DocumentEvent::PictureDeleted {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_ctrl,
        });
        Ok("{\"ok\":true}".to_string())
    }

    pub fn set_cell_shape_properties_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        cell_path_json: &str,
        inner_control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path_json(cell_path_json)?;
        {
            let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
                HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
            })?;
            let current_para = Self::resolve_cell_paragraph_mut(section, parent_para_idx, &path)?;
            let ctrl = current_para
                .controls
                .get_mut(inner_control_idx)
                .ok_or_else(|| {
                    HwpError::RenderError(format!("셀 내 컨트롤 {} 범위 초과", inner_control_idx))
                })?;
            let shape = match ctrl {
                Control::Shape(s) => s.as_mut(),
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 셀 내 컨트롤이 Shape이 아닙니다".to_string(),
                    ))
                }
            };
            Self::apply_shape_props_inner(shape, props_json);
        }
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
        let outer_table_ctrl = path.first().unwrap().0;
        self.event_log.push(DocumentEvent::PictureResized {
            section: section_idx,
            para: parent_para_idx,
            ctrl: outer_table_ctrl,
        });
        Ok("{\"ok\":true}".to_string())
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
        Ok(super::super::helpers::json_ok_with(&format!(
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
                return Ok(super::super::helpers::json_ok_with(&format!(
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

        Ok(super::super::helpers::json_ok_with(&format!(
            "\"zOrder\":{}",
            new_z
        )))
    }

    /// 연결선의 SubjectID를 갱신한다 (연결선 생성 후 호출)
    pub fn update_connector_subject_ids(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        start_subject_id: u32,
        start_subject_index: u32,
        end_subject_id: u32,
        end_subject_index: u32,
    ) {
        if let Some(section) = self.document.sections.get_mut(section_idx) {
            if let Some(para) = section.paragraphs.get_mut(para_idx) {
                if let Some(Control::Shape(ref mut shape)) = para.controls.get_mut(control_idx) {
                    if let ShapeObject::Line(ref mut line) = shape.as_mut() {
                        if let Some(ref mut conn) = line.connector {
                            conn.start_subject_id = start_subject_id;
                            conn.start_subject_index = start_subject_index;
                            conn.end_subject_id = end_subject_id;
                            conn.end_subject_index = end_subject_index;
                        }
                    }
                }
            }
        }
    }

    /// 연결선 제어점을 연결점 방향에 따라 재계산한다.
    /// start_idx/end_idx: 0=상, 1=우, 2=하, 3=좌
    pub fn recalculate_connector_routing(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        start_idx: u32,
        end_idx: u32,
    ) {
        use crate::model::shape::ConnectorControlPoint;

        let section = match self.document.sections.get_mut(section_idx) {
            Some(s) => s,
            None => return,
        };
        let para = match section.paragraphs.get_mut(para_idx) {
            Some(p) => p,
            None => return,
        };
        let ctrl = match para.controls.get_mut(control_idx) {
            Some(c) => c,
            None => return,
        };

        let line = match ctrl {
            Control::Shape(ref mut s) => match s.as_mut() {
                ShapeObject::Line(ref mut l) => l,
                _ => return,
            },
            _ => return,
        };

        let conn = match &mut line.connector {
            Some(c) => c,
            None => return,
        };

        let sx = line.start.x;
        let sy = line.start.y;
        let ex = line.end.x;
        let ey = line.end.y;
        let w = line.common.width as i32;
        let h = line.common.height as i32;

        // 직선 연결선: 제어점 불필요
        if !conn.link_type.is_stroke() && !conn.link_type.is_arc() {
            conn.control_points.clear();
            return;
        }

        // 연결점 방향: 0=상, 1=우, 2=하, 3=좌
        if conn.link_type.is_arc() {
            // ─── 곡선 연결선: 파워포인트 스타일 S곡선 ───
            // ctrl1: 시작점에서 시작 방향으로 중간지점까지 뻗음
            // ctrl2: 끝점에서 끝 방향으로 중간지점까지 뻗음
            // → 중간지점에서 위아래(또는 좌우)가 반전되는 S자
            // 한컴 공식: 수평 연결(우/좌)은 midX 기준, 수직 연결(상/하)은 midY 기준
            // ctrl1 = (midX, startY) / (startX, midY), ctrl2 = (midX, endY) / (endX, midY)
            let mid_x = (sx + ex) / 2;
            let mid_y = (sy + ey) / 2;
            let start_is_horz = start_idx == 1 || start_idx == 3; // 우/좌
            let end_is_horz = end_idx == 1 || end_idx == 3;

            let (c1x, c1y, c2x, c2y) = if start_is_horz && end_is_horz {
                // 우↔좌: midX 기준 S곡선
                (mid_x, sy, mid_x, ey)
            } else if !start_is_horz && !end_is_horz {
                // 상↔하: midY 기준 S곡선
                (sx, mid_y, ex, mid_y)
            } else if start_is_horz {
                // 우/좌 → 상/하: 수평 출발 → midX까지, 수직 진입 → midY까지
                (mid_x, sy, ex, mid_y)
            } else {
                // 상/하 → 우/좌: 수직 출발 → midY까지, 수평 진입 → midX까지
                (sx, mid_y, mid_x, ey)
            };

            conn.control_points = vec![
                ConnectorControlPoint {
                    x: sx,
                    y: sy,
                    point_type: 3,
                }, // 시작 앵커
                ConnectorControlPoint {
                    x: c1x,
                    y: c1y,
                    point_type: 2,
                }, // 베지어 ctrl1
                ConnectorControlPoint {
                    x: c2x,
                    y: c2y,
                    point_type: 2,
                }, // 베지어 ctrl2
                ConnectorControlPoint {
                    x: ex,
                    y: ey,
                    point_type: 26,
                }, // 끝 앵커
            ];
        } else {
            // ─── 꺽인 연결선: 직각 꺾임점 ───
            let mut pts = Vec::new();
            pts.push(ConnectorControlPoint {
                x: sx,
                y: sy,
                point_type: 3,
            });

            match (start_idx, end_idx) {
                (1, 3) | (3, 1) => {
                    let mid_x = (sx + ex) / 2;
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: sy,
                        point_type: 2,
                    });
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: ey,
                        point_type: 2,
                    });
                }
                (2, 0) | (0, 2) => {
                    let mid_y = (sy + ey) / 2;
                    pts.push(ConnectorControlPoint {
                        x: sx,
                        y: mid_y,
                        point_type: 2,
                    });
                    pts.push(ConnectorControlPoint {
                        x: ex,
                        y: mid_y,
                        point_type: 2,
                    });
                }
                (1, 0) | (1, 2) | (3, 0) | (3, 2) => {
                    pts.push(ConnectorControlPoint {
                        x: ex,
                        y: sy,
                        point_type: 2,
                    });
                }
                (0, 1) | (0, 3) | (2, 1) | (2, 3) => {
                    pts.push(ConnectorControlPoint {
                        x: sx,
                        y: ey,
                        point_type: 2,
                    });
                }
                _ => {
                    let mid_x = (sx + ex) / 2;
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: sy,
                        point_type: 2,
                    });
                    pts.push(ConnectorControlPoint {
                        x: mid_x,
                        y: ey,
                        point_type: 2,
                    });
                }
            }

            pts.push(ConnectorControlPoint {
                x: ex,
                y: ey,
                point_type: 26,
            });
            conn.control_points = pts;
        }
    }

    /// 구역 내 모든 연결선을 스캔하여 연결된 도형의 현재 위치에 맞게 갱신한다.
    pub fn update_connectors_in_section(&mut self, section_idx: usize) {
        let section = match self.document.sections.get(section_idx) {
            Some(s) => s,
            None => return,
        };

        // 1) SC inst_id → 연결점 좌표 맵 구축 (SubjectID = drawing.inst_id)
        let mut conn_points: std::collections::HashMap<u32, [(i32, i32); 4]> =
            std::collections::HashMap::new();
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let (common, inst_id, _is_line) = match ctrl {
                    Control::Shape(s) => {
                        let sc_inst = s.drawing().map(|d| d.inst_id).unwrap_or(0);
                        (
                            s.common(),
                            sc_inst,
                            matches!(s.as_ref(), ShapeObject::Line(_)),
                        )
                    }
                    Control::Picture(p) => (&p.common, 0u32, false),
                    _ => continue,
                };
                if _is_line {
                    continue;
                }
                let x = common.horizontal_offset as i32;
                let y = common.vertical_offset as i32;
                let w = common.width as i32;
                let h = common.height as i32;
                let cx = x + w / 2;
                let cy = y + h / 2;
                let pts = [(cx, y), (x + w, cy), (cx, y + h), (x, cy)];
                // SC inst_id (= SubjectID) 등록
                if inst_id != 0 {
                    conn_points.insert(inst_id, pts);
                }
                // CTRL_HEADER instance_id로도 등록 (폴백)
                if common.instance_id != 0 {
                    conn_points.insert(common.instance_id, pts);
                    conn_points.insert((common.instance_id & 0x3FFFFFFF) + 1, pts);
                }
            }
        }

        // 2) 커넥터 찾기 및 좌표 갱신
        let section = match self.document.sections.get_mut(section_idx) {
            Some(s) => s,
            None => return,
        };
        for para in &mut section.paragraphs {
            for ctrl in &mut para.controls {
                let line = match ctrl {
                    Control::Shape(ref mut s) => match s.as_mut() {
                        ShapeObject::Line(ref mut l) if l.connector.is_some() => l,
                        _ => continue,
                    },
                    _ => continue,
                };

                let conn = line.connector.as_ref().unwrap();
                let start_pts = conn_points.get(&conn.start_subject_id);
                let end_pts = conn_points.get(&conn.end_subject_id);

                // 연결된 도형을 찾지 못하면 건너뜀 (연결 끊어진 상태)
                if start_pts.is_none() || end_pts.is_none() {
                    continue;
                }

                let si = conn.start_subject_index as usize;
                let ei = conn.end_subject_index as usize;
                let (gsx, gsy) = start_pts.unwrap()[si.min(3)];
                let (gex, gey) = end_pts.unwrap()[ei.min(3)];

                // 커넥터 bbox 재계산
                let min_x = gsx.min(gex);
                let min_y = gsy.min(gey);
                let max_x = gsx.max(gex);
                let max_y = gsy.max(gey);
                let new_w = (max_x - min_x).max(1) as u32;
                let new_h = (max_y - min_y).max(1) as u32;

                line.common.horizontal_offset = min_x as u32;
                line.common.vertical_offset = min_y as u32;
                line.common.width = new_w;
                line.common.height = new_h;

                // 로컬 시작/끝 좌표
                line.start.x = gsx - min_x;
                line.start.y = gsy - min_y;
                line.end.x = gex - min_x;
                line.end.y = gey - min_y;

                // shape_attr 동기화
                line.drawing.shape_attr.current_width = new_w;
                line.drawing.shape_attr.original_width = new_w;
                line.drawing.shape_attr.current_height = new_h;
                line.drawing.shape_attr.original_height = new_h;
                line.drawing.shape_attr.rotation_center.x = new_w as i32 / 2;
                line.drawing.shape_attr.rotation_center.y = new_h as i32 / 2;
                line.drawing.shape_attr.raw_rendering = Vec::new();
            }
        }

        // 3) 제어점 재계산 (인덱스 수집 후 별도 루프 — borrow checker 대응)
        let mut routing_targets: Vec<(usize, usize, u32, u32)> = Vec::new();
        {
            let section = match self.document.sections.get(section_idx) {
                Some(s) => s,
                None => return,
            };
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(ref s) = ctrl {
                        if let ShapeObject::Line(ref l) = s.as_ref() {
                            if let Some(ref c) = l.connector {
                                if c.link_type.is_stroke() || c.link_type.is_arc() {
                                    routing_targets.push((
                                        pi,
                                        ci,
                                        c.start_subject_index,
                                        c.end_subject_index,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (pi, ci, si, ei) in routing_targets {
            self.recalculate_connector_routing(section_idx, pi, ci, si, ei);
        }
    }

    /// 직선 끝점 이동: 글로벌 좌표(HWPUNIT)로 시작/끝점을 직접 설정
    pub fn move_line_endpoint_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    ) -> Result<String, HwpError> {
        let section = self
            .document
            .sections
            .get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let para = section
            .paragraphs
            .get_mut(para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;
        let ctrl = para
            .controls
            .get_mut(control_idx)
            .ok_or_else(|| HwpError::RenderError("컨트롤 범위 초과".to_string()))?;
        let line = match ctrl {
            Control::Shape(ref mut s) => match s.as_mut() {
                ShapeObject::Line(ref mut l) => l,
                _ => return Err(HwpError::RenderError("직선이 아닙니다".to_string())),
            },
            _ => return Err(HwpError::RenderError("Shape이 아닙니다".to_string())),
        };

        let min_x = start_x.min(end_x);
        let min_y = start_y.min(end_y);
        let w = (start_x - end_x).abs().max(1);
        let h = (start_y - end_y).abs().max(0);

        line.common.horizontal_offset = min_x as u32;
        line.common.vertical_offset = min_y as u32;
        line.common.width = w as u32;
        line.common.height = h.max(1) as u32;
        line.start.x = start_x - min_x;
        line.start.y = start_y - min_y;
        line.end.x = end_x - min_x;
        line.end.y = end_y - min_y;

        line.drawing.shape_attr.current_width = w as u32;
        line.drawing.shape_attr.original_width = w as u32;
        line.drawing.shape_attr.current_height = h.max(1) as u32;
        line.drawing.shape_attr.original_height = h.max(1) as u32;
        line.drawing.shape_attr.rotation_center.x = w / 2;
        line.drawing.shape_attr.rotation_center.y = h / 2;
        line.drawing.shape_attr.raw_rendering = Vec::new();

        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.update_connectors_in_section(section_idx);

        Ok("{\"ok\":true}".to_string())
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
        Ok(super::super::helpers::json_ok_with(&format!(
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

    /// 수식 컨트롤의 속성을 조회한다 (네이티브).
    /// 표 셀 내 또는 본문의 수식 컨트롤을 찾아 불변 참조를 반환한다.
    fn find_equation_ref(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
    ) -> Result<&crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;

        let ctrl = if let (Some(ci), Some(cpi)) = (cell_idx, cell_para_idx) {
            // 표 셀 내 수식
            let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            let table = match para.controls.get(control_idx) {
                Some(Control::Table(t)) => t,
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 컨트롤이 표가 아닙니다".to_string(),
                    ))
                }
            };
            let cell = table
                .cells
                .get(ci)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", ci)))?;
            let cell_para = cell.paragraphs.get(cpi).ok_or_else(|| {
                HwpError::RenderError(format!("셀 문단 인덱스 {} 범위 초과", cpi))
            })?;
            // 셀 문단의 첫 번째 수식 컨트롤을 찾는다
            cell_para
                .controls
                .iter()
                .find(|c| matches!(c, Control::Equation(_)))
                .ok_or_else(|| {
                    HwpError::RenderError("셀 문단에 수식 컨트롤이 없습니다".to_string())
                })?
        } else {
            // 본문 수식
            let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            para.controls.get(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
            })?
        };

        match ctrl {
            Control::Equation(e) => Ok(e),
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 수식이 아닙니다".to_string(),
            )),
        }
    }

    /// 표 셀 내 또는 본문의 수식 컨트롤을 찾아 가변 참조를 반환한다.
    fn find_equation_mut(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
    ) -> Result<&mut crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;

        let ctrl = if let (Some(ci), Some(cpi)) = (cell_idx, cell_para_idx) {
            // 표 셀 내 수식
            let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            let table = match para.controls.get_mut(control_idx) {
                Some(Control::Table(t)) => t,
                _ => {
                    return Err(HwpError::RenderError(
                        "지정된 컨트롤이 표가 아닙니다".to_string(),
                    ))
                }
            };
            let cell = table
                .cells
                .get_mut(ci)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", ci)))?;
            let cell_para = cell.paragraphs.get_mut(cpi).ok_or_else(|| {
                HwpError::RenderError(format!("셀 문단 인덱스 {} 범위 초과", cpi))
            })?;
            cell_para
                .controls
                .iter_mut()
                .find(|c| matches!(c, Control::Equation(_)))
                .ok_or_else(|| {
                    HwpError::RenderError("셀 문단에 수식 컨트롤이 없습니다".to_string())
                })?
        } else {
            // 본문 수식
            let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
            })?;
            para.controls.get_mut(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx))
            })?
        };

        match ctrl {
            Control::Equation(e) => Ok(e),
            _ => Err(HwpError::RenderError(
                "지정된 컨트롤이 수식이 아닙니다".to_string(),
            )),
        }
    }

    fn find_note_equation_ref(
        &self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
    ) -> Result<&crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let note_para = match para.controls.get(note_control_idx) {
            Some(Control::Footnote(note)) if kind == "footnote" => {
                note.paragraphs.get(note_para_idx)
            }
            Some(Control::Endnote(note)) if kind == "endnote" => note.paragraphs.get(note_para_idx),
            _ => None,
        }
        .ok_or_else(|| {
            HwpError::RenderError(format!(
                "각주/미주 문단을 찾을 수 없습니다: kind={} sec={} para={} ctrl={} note_para={}",
                kind, section_idx, parent_para_idx, note_control_idx, note_para_idx
            ))
        })?;
        match note_para.controls.get(inner_control_idx) {
            Some(Control::Equation(eq)) => Ok(eq),
            _ => Err(HwpError::RenderError(format!(
                "각주/미주 내부 컨트롤 {}은 수식이 아닙니다",
                inner_control_idx
            ))),
        }
    }

    fn find_note_equation_mut(
        &mut self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
    ) -> Result<&mut crate::model::control::Equation, HwpError> {
        let section = self.document.sections.get_mut(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
            HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx))
        })?;
        let note_para = match para.controls.get_mut(note_control_idx) {
            Some(Control::Footnote(note)) if kind == "footnote" => {
                note.paragraphs.get_mut(note_para_idx)
            }
            Some(Control::Endnote(note)) if kind == "endnote" => {
                note.paragraphs.get_mut(note_para_idx)
            }
            _ => None,
        }
        .ok_or_else(|| {
            HwpError::RenderError(format!(
                "각주/미주 문단을 찾을 수 없습니다: kind={} sec={} para={} ctrl={} note_para={}",
                kind, section_idx, parent_para_idx, note_control_idx, note_para_idx
            ))
        })?;
        match note_para.controls.get_mut(inner_control_idx) {
            Some(Control::Equation(eq)) => Ok(eq),
            _ => Err(HwpError::RenderError(format!(
                "각주/미주 내부 컨트롤 {}은 수식이 아닙니다",
                inner_control_idx
            ))),
        }
    }

    fn equation_properties_json(eq: &crate::model::control::Equation) -> String {
        let common_json = Self::common_obj_attr_to_json(&eq.common);
        let script_escaped = super::super::helpers::json_escape(&eq.script);
        let font_name_escaped = super::super::helpers::json_escape(&eq.font_name);

        format!(
            concat!(
                "{{{},\"script\":\"{}\",\"fontSize\":{},\"color\":{},",
                "\"baseline\":{},\"fontName\":\"{}\",",
                "\"hasCaption\":false,\"captionDirection\":\"None\",",
                "\"captionWidth\":0,\"captionSpacing\":0}}"
            ),
            common_json, script_escaped, eq.font_size, eq.color, eq.baseline, font_name_escaped,
        )
    }

    fn apply_equation_properties(
        eq: &mut crate::model::control::Equation,
        dpi: f64,
        props_json: &str,
    ) {
        use super::super::helpers::{json_i32, json_str, json_u32};
        use crate::renderer::equation::layout::EqLayout;
        use crate::renderer::equation::parser::EqParser;
        use crate::renderer::equation::tokenizer::tokenize;
        use crate::renderer::hwpunit_to_px;

        if let Some(s) = json_str(props_json, "script") {
            eq.script = s;
        }
        if let Some(fs) = json_u32(props_json, "fontSize") {
            eq.font_size = fs;
        }
        if let Some(c) = json_u32(props_json, "color") {
            eq.color = c;
        }
        if let Some(bl) = json_i32(props_json, "baseline") {
            eq.baseline = bl as i16;
        }
        if let Some(fn_) = json_str(props_json, "fontName") {
            eq.font_name = fn_;
        }
        Self::apply_common_obj_attr_from_json(&mut eq.common, props_json);

        let font_size_px = hwpunit_to_px(eq.font_size as i32, dpi);
        let tokens = tokenize(&eq.script);
        let ast = EqParser::new(tokens).parse();
        let layout_box = EqLayout::new(font_size_px).layout(&ast);
        let new_w = crate::renderer::px_to_hwpunit(layout_box.width, dpi).max(0) as u32;
        let new_h = crate::renderer::px_to_hwpunit(layout_box.height, dpi).max(0) as u32;
        eq.common.width = new_w;
        eq.common.height = new_h;
    }

    pub fn get_equation_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
    ) -> Result<String, HwpError> {
        let eq = self.find_equation_ref(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;

        Ok(Self::equation_properties_json(eq))
    }

    /// 수식 컨트롤의 속성을 변경한다 (네이티브).
    pub fn set_equation_properties_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let dpi = self.dpi;
        let eq = self.find_equation_mut(
            section_idx,
            parent_para_idx,
            control_idx,
            cell_idx,
            cell_para_idx,
        )?;
        Self::apply_equation_properties(eq, dpi, props_json);

        // 표 셀 내 수식인 경우 표 dirty 플래그 설정
        if cell_idx.is_some() {
            if let Some(Control::Table(t)) = self.document.sections[section_idx].paragraphs
                [parent_para_idx]
                .controls
                .get_mut(control_idx)
            {
                t.dirty = true;
            }
        }

        // 재조판
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(super::super::helpers::json_ok())
    }

    pub fn get_note_equation_properties_native(
        &self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
    ) -> Result<String, HwpError> {
        let eq = self.find_note_equation_ref(
            kind,
            section_idx,
            parent_para_idx,
            note_control_idx,
            note_para_idx,
            inner_control_idx,
        )?;
        Ok(Self::equation_properties_json(eq))
    }

    pub fn set_note_equation_properties_native(
        &mut self,
        kind: &str,
        section_idx: usize,
        parent_para_idx: usize,
        note_control_idx: usize,
        note_para_idx: usize,
        inner_control_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        let dpi = self.dpi;
        let eq = self.find_note_equation_mut(
            kind,
            section_idx,
            parent_para_idx,
            note_control_idx,
            note_para_idx,
            inner_control_idx,
        )?;
        Self::apply_equation_properties(eq, dpi, props_json);

        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        Ok(super::super::helpers::json_ok())
    }

    /// 수식 스크립트를 SVG로 렌더링하여 반환한다 (미리보기 전용).
    pub fn render_equation_preview_native(
        &self,
        script: &str,
        font_size_hwpunit: u32,
        color: u32,
    ) -> Result<String, HwpError> {
        use crate::renderer::equation::layout::EqLayout;
        use crate::renderer::equation::parser::EqParser;
        use crate::renderer::equation::svg_render::{eq_color_to_svg, render_equation_svg};
        use crate::renderer::equation::tokenizer::tokenize;

        let font_size_px = crate::renderer::hwpunit_to_px(font_size_hwpunit as i32, self.dpi);
        let tokens = tokenize(script);
        let ast = EqParser::new(tokens).parse();
        let layout_box = EqLayout::new(font_size_px).layout(&ast);
        let color_str = eq_color_to_svg(color);
        let svg_fragment = render_equation_svg(&layout_box, &color_str, font_size_px);

        let w = layout_box.width;
        let h = layout_box.height;
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {:.2} {:.2}\" width=\"{:.2}\" height=\"{:.2}\">{}</svg>",
            w, h, w, h, svg_fragment,
        );
        Ok(svg)
    }

    /// 수식(Equation) 컨트롤을 문단에서 삭제한다.
    pub fn delete_equation_control_native(
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
        if !matches!(&para.controls[control_idx], Control::Equation(_)) {
            return Err(HwpError::RenderError(
                "지정된 컨트롤이 수식이 아닙니다".to_string(),
            ));
        }

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

    // ─── 각주 삽입/삭제 API ──────────────────────────────

    fn footnote_shape_number_format_code(format: crate::model::footnote::NumberFormat) -> u8 {
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

    fn footnote_numbering_name(
        numbering: crate::model::footnote::FootnoteNumbering,
    ) -> &'static str {
        use crate::model::footnote::FootnoteNumbering;
        match numbering {
            FootnoteNumbering::Continue => "continue",
            FootnoteNumbering::RestartSection => "restartSection",
            FootnoteNumbering::RestartPage => "restartPage",
        }
    }

    fn footnote_numbering_from_str(
        value: &str,
        fallback: crate::model::footnote::FootnoteNumbering,
    ) -> crate::model::footnote::FootnoteNumbering {
        use crate::model::footnote::FootnoteNumbering;
        match value {
            "continue" | "CONTINUOUS" | "continuous" => FootnoteNumbering::Continue,
            "restartSection" | "ON_SECTION" | "RESTART_SECTION" | "onSection" => {
                FootnoteNumbering::RestartSection
            }
            "restartPage" | "ON_PAGE" | "RESTART_PAGE" | "onPage" => FootnoteNumbering::RestartPage,
            _ => fallback,
        }
    }

    fn footnote_placement_name(
        placement: crate::model::footnote::FootnotePlacement,
    ) -> &'static str {
        use crate::model::footnote::FootnotePlacement;
        match placement {
            FootnotePlacement::EachColumn => "documentEnd",
            FootnotePlacement::BelowText => "sectionEnd",
            FootnotePlacement::RightColumn => "rightColumn",
        }
    }

    fn footnote_placement_from_str(
        value: &str,
        fallback: crate::model::footnote::FootnotePlacement,
    ) -> crate::model::footnote::FootnotePlacement {
        use crate::model::footnote::FootnotePlacement;
        match value {
            "documentEnd" | "eachColumn" => FootnotePlacement::EachColumn,
            "sectionEnd" | "belowText" => FootnotePlacement::BelowText,
            "rightColumn" => FootnotePlacement::RightColumn,
            _ => fallback,
        }
    }

    fn encode_footnote_shape_attr(shape: &crate::model::footnote::FootnoteShape) -> u32 {
        shape.encode_attr()
    }

    fn first_char_or_nul(value: &str) -> char {
        value.chars().next().unwrap_or('\0')
    }

    fn json_escape_note_char(ch: char) -> String {
        if ch == '\0' {
            String::new()
        } else {
            crate::document_core::helpers::json_escape(&ch.to_string())
        }
    }

    fn hwpunit16_from_json(json: &str, key: &str) -> Option<i16> {
        crate::document_core::helpers::json_i32(json, key)
            .map(|v| v.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
    }

    fn make_note_inner_paragraph(
        number_type: crate::model::control::AutoNumberType,
        number: u16,
        format: u8,
        prefix_char: char,
        suffix_char: char,
        default_char_shape_id: u32,
        para_shape_id: u16,
        style_id: u8,
    ) -> crate::model::paragraph::Paragraph {
        use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

        let auto_num = crate::model::control::AutoNumber {
            number_type,
            format,
            superscript: false,
            number,
            assigned_number: number,
            user_symbol: '\0',
            prefix_char,
            suffix_char,
        };

        Paragraph {
            text: "  ".to_string(),
            char_count: 10,
            char_count_msb: true,
            control_mask: 1u32 << 0x12,
            char_offsets: vec![0, 8],
            para_shape_id,
            style_id,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            controls: vec![crate::model::control::Control::AutoNumber(auto_num)],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: 0,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            has_para_text: true,
            ..Default::default()
        }
    }

    fn endnote_style_defaults(&self, section_idx: usize, para_idx: usize) -> (u32, u16, u8) {
        let section = &self.document.sections[section_idx];

        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Endnote(en) = ctrl {
                    if let Some(ep) = en.paragraphs.first() {
                        let char_shape_id = ep
                            .char_shapes
                            .first()
                            .map(|cs| cs.char_shape_id)
                            .unwrap_or(0);
                        return (char_shape_id, ep.para_shape_id, ep.style_id);
                    }
                }
            }
        }

        for (idx, style) in self.document.doc_info.styles.iter().enumerate() {
            if style.local_name == "미주" || style.english_name.eq_ignore_ascii_case("Endnote") {
                return (
                    style.char_shape_id as u32,
                    style.para_shape_id,
                    idx.min(u8::MAX as usize) as u8,
                );
            }
        }

        let current_para = &section.paragraphs[para_idx];
        (
            current_para
                .char_shapes
                .first()
                .map(|cs| cs.char_shape_id)
                .unwrap_or(0),
            current_para.para_shape_id,
            current_para.style_id,
        )
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

    fn renumber_paragraph_endnotes_with_shape(
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

    /// 각주를 삽입한다.
    /// 커서 위치에 각주 컨트롤을 추가하고 빈 문단 1개를 생성한다.
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N, "footnoteNumber":N}`
    pub fn insert_footnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::footnote::Footnote;
        use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};

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

        // 각주 번호: 삽입 위치 이전의 모든 각주 수 + 1
        // 본문 문단 + 표 셀 + 글상자 내부의 각주를 모두 포함
        let footnote_number = {
            let mut count = 0u16;
            let section = &self.document.sections[section_idx];
            for (pi, para) in section.paragraphs.iter().enumerate() {
                let is_before = pi < para_idx;
                let is_same = pi == para_idx;
                // 본문 문단의 각주
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Footnote(_) => {
                            if is_before {
                                count += 1;
                            } else if is_same {
                                let positions =
                                    crate::document_core::helpers::find_control_text_positions(
                                        para,
                                    );
                                let pos = positions.get(ci).copied().unwrap_or(usize::MAX);
                                if pos <= char_offset {
                                    count += 1;
                                }
                            }
                        }
                        // 표 셀 내 각주
                        Control::Table(table) if is_before || is_same => {
                            for cell in &table.cells {
                                for cp in &cell.paragraphs {
                                    count +=
                                        cp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Footnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        // 글상자 내 각주
                        Control::Shape(shape) if is_before || is_same => {
                            if let Some(text_box) =
                                shape.drawing().and_then(|d| d.text_box.as_ref())
                            {
                                for tp in &text_box.paragraphs {
                                    count +=
                                        tp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Footnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            count + 1
        };

        // 각주 내부 문단 생성: 기존 각주의 스타일을 참조하여 동일한 스타일 적용
        // 기존 각주가 없으면 본문 문단 스타일 사용
        let (default_char_shape_id, default_para_shape_id) = {
            let section = &self.document.sections[section_idx];
            let mut found = None;
            // 본문 문단의 각주에서 스타일 참조
            'outer: for para in &section.paragraphs {
                for ctrl in &para.controls {
                    if let Control::Footnote(fn_) = ctrl {
                        if let Some(fp) = fn_.paragraphs.first() {
                            found = Some((
                                fp.char_shapes
                                    .first()
                                    .map(|cs| cs.char_shape_id)
                                    .unwrap_or(0),
                                fp.para_shape_id,
                            ));
                            break 'outer;
                        }
                    }
                    // 표 셀 내 각주에서도 참조
                    if let Control::Table(table) = ctrl {
                        for cell in &table.cells {
                            for cp in &cell.paragraphs {
                                for cc in &cp.controls {
                                    if let Control::Footnote(fn_) = cc {
                                        if let Some(fp) = fn_.paragraphs.first() {
                                            found = Some((
                                                fp.char_shapes
                                                    .first()
                                                    .map(|cs| cs.char_shape_id)
                                                    .unwrap_or(0),
                                                fp.para_shape_id,
                                            ));
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            found.unwrap_or_else(|| {
                let current_para = &section.paragraphs[para_idx];
                (
                    current_para
                        .char_shapes
                        .first()
                        .map(|cs| cs.char_shape_id)
                        .unwrap_or(0),
                    current_para.para_shape_id,
                )
            })
        };

        // [Task #1058 reopen Round 5] 신규 각주 inner paragraph 한컴 contract 정합:
        //   - style_id = 11 (각주 style, 한컴 DocInfo 기본 각주 style ID)
        //   - para_shape_id = 0 (각주 default ParaShape)
        //   - controls = [AutoNumber] (각주 번호 inline 컨트롤, char index 0 위치)
        //   - text = "  " (placeholder space ×2, AutoNumber 가 두 space 사이 8 cu 차지)
        //   - char_offsets = [0, 8] (첫 space pos 0, AutoNumber anchor 점유 pos 0~7, 두 번째 space pos 8)
        //   - char_count = 10 (2 placeholder + 8 AutoNumber inline ctrl)
        //   - has_para_text = true
        // 한컴 정답지 samples/footnote-01.hwp 의 각주 inner_para 와 동일한 contract.
        // 사용자 입력은 두 placeholder 뒤 (char_offset=2) 부터 시작 — insert_text_at 의
        // 일반 분기가 char_offsets[i] = base + sum(widths) 시프트 (jump 8 보존).
        let auto_num = crate::model::control::AutoNumber {
            number_type: crate::model::control::AutoNumberType::Footnote,
            format: 0, // Digit
            superscript: false,
            number: footnote_number,
            assigned_number: footnote_number,
            user_symbol: '\0',
            prefix_char: '\0',
            suffix_char: ')',
        };
        let inner_para = Paragraph {
            text: "  ".to_string(), // placeholder space ×2 (정답지 정합)
            char_count: 10,         // 2 placeholder + 8 (AutoNumber inline ctrl)
            char_count_msb: true,
            control_mask: 1u32 << 0x12, // bit 18 (AutoNumber)
            char_offsets: vec![0, 8],   // AutoNumber 가 두 space 사이 8 cu 차지
            para_shape_id: 0,
            style_id: 11, // 각주 style
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: default_char_shape_id,
            }],
            controls: vec![crate::model::control::Control::AutoNumber(auto_num)],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                segment_width: 0,
                tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            }],
            has_para_text: true,
            ..Default::default()
        };
        // default_para_shape_id 변수가 위에서 unused 가 되지 않도록 (caller paragraph 의 ps 정보는
        // 본 본문 paragraph 의 contract 보존 — 각주 본문은 ps_id=0 사용)
        let _ = default_para_shape_id;

        let footnote = Footnote {
            number: footnote_number,
            paragraphs: vec![inner_para],
            // [Task #1050] HWP5 CTRL_FOOTNOTE 한컴 default
            after_decoration_letter: 0x0029, // ')'
            ..Default::default()
        };

        // 문단에 각주 컨트롤 삽입
        self.document.sections[section_idx].raw_stream = None;
        let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

        // 삽입 위치 결정 (char_offset 기준)
        let insert_idx = {
            let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
            let mut idx = paragraph.controls.len();
            for (i, &pos) in positions.iter().enumerate() {
                if pos > char_offset {
                    idx = i;
                    break;
                }
            }
            idx
        };

        paragraph
            .controls
            .insert(insert_idx, Control::Footnote(Box::new(footnote)));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        // char_offsets 조정: char_offset 위치에 8바이트 갭 생성
        // char_offsets[i]는 텍스트 i번째 문자의 UTF-16 오프셋 (컨트롤은 갭으로 표현)
        // 주의: char_offset은 텍스트 기준 인덱스이지만, char_offsets 배열 길이는 text.chars().count()
        // text에 포함되지 않는 제어 문자(cc - text_len 차이)가 있을 수 있으므로 범위 확인
        if !paragraph.char_offsets.is_empty() {
            let text_len = paragraph.text.chars().count();
            let safe_offset = char_offset.min(text_len);
            for co in paragraph.char_offsets[safe_offset..].iter_mut() {
                *co += 8;
            }
        }
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 0x0011; // 각주/미주 비트
        paragraph.has_para_text = true;

        // 전체 각주 순서 번호 재계산 (1부터 순차)
        // 본문 문단 + 표 셀 + 글상자 내부의 각주를 모두 포함
        {
            let mut num = 1u16;
            for pi in 0..self.document.sections[section_idx].paragraphs.len() {
                for ci in 0..self.document.sections[section_idx].paragraphs[pi]
                    .controls
                    .len()
                {
                    match &mut self.document.sections[section_idx].paragraphs[pi].controls[ci] {
                        Control::Footnote(ref mut fn_) => {
                            fn_.number = num;
                            num += 1;
                        }
                        Control::Table(ref mut table) => {
                            for cell in &mut table.cells {
                                for cp in &mut cell.paragraphs {
                                    for cc in &mut cp.controls {
                                        if let Control::Footnote(ref mut fn_) = cc {
                                            fn_.number = num;
                                            num += 1;
                                        }
                                    }
                                }
                            }
                        }
                        Control::Shape(ref mut shape) => {
                            if let Some(text_box) =
                                shape.drawing_mut().and_then(|d| d.text_box.as_mut())
                            {
                                for tp in &mut text_box.paragraphs {
                                    for tc in &mut tp.controls {
                                        if let Control::Footnote(ref mut fn_) = tc {
                                            fn_.number = num;
                                            num += 1;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // 각주 내부 문단 리플로우
        self.reflow_footnote_paragraph(section_idx, para_idx, insert_idx, 0);

        // 본문 문단 리플로우 (각주 마커 폭으로 인한 줄넘김 변경 반영)
        {
            use crate::renderer::composer::reflow_line_segs;
            use crate::renderer::hwpunit_to_px;
            let page_def = &self.document.sections[section_idx].section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            let available_width = hwpunit_to_px(text_width, self.dpi);
            let para_style = self.styles.para_styles.get(
                self.document.sections[section_idx].paragraphs[para_idx].para_shape_id as usize,
            );
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let final_width = (available_width - margin_left - margin_right).max(0.0);
            let body_para = &mut self.document.sections[section_idx].paragraphs[para_idx];
            reflow_line_segs(body_para, final_width, &self.styles, self.dpi);
        }

        // 리플로우 + 페이지네이션
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{},\"footnoteNumber\":{}}}",
            para_idx, insert_idx, footnote_number
        ))
    }

    /// 미주를 삽입한다.
    /// 커서 위치에 미주 컨트롤을 추가하고 빈 미주 문단 1개를 생성한다.
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N, "endnoteNumber":N}`
    pub fn insert_endnote_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::model::footnote::Endnote;

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

        let shape = self.document.sections[section_idx]
            .section_def
            .endnote_shape
            .clone();
        let start_number = shape.start_number.max(1);
        let number_format_code = Self::footnote_shape_number_format_code(shape.number_format);
        let endnote_number = {
            let mut count = 0u16;
            let section = &self.document.sections[section_idx];
            for (pi, para) in section.paragraphs.iter().enumerate() {
                let is_before = pi < para_idx;
                let is_same = pi == para_idx;
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Endnote(_) => {
                            if is_before {
                                count += 1;
                            } else if is_same {
                                let positions =
                                    crate::document_core::helpers::find_control_text_positions(
                                        para,
                                    );
                                let pos = positions.get(ci).copied().unwrap_or(usize::MAX);
                                if pos <= char_offset {
                                    count += 1;
                                }
                            }
                        }
                        Control::Table(table) if is_before || is_same => {
                            for cell in &table.cells {
                                for cp in &cell.paragraphs {
                                    count +=
                                        cp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Endnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        Control::Shape(shape) if is_before || is_same => {
                            if let Some(text_box) =
                                shape.drawing().and_then(|d| d.text_box.as_ref())
                            {
                                for tp in &text_box.paragraphs {
                                    count +=
                                        tp.controls
                                            .iter()
                                            .filter(|c| matches!(c, Control::Endnote(_)))
                                            .count() as u16;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            start_number.saturating_add(count)
        };

        let (default_char_shape_id, para_shape_id, style_id) =
            self.endnote_style_defaults(section_idx, para_idx);
        let prefix_char = if shape.prefix_char == '\0' {
            '\0'
        } else {
            shape.prefix_char
        };
        let suffix_char = if shape.suffix_char == '\0' {
            ')'
        } else {
            shape.suffix_char
        };

        let inner_para = Self::make_note_inner_paragraph(
            crate::model::control::AutoNumberType::Endnote,
            endnote_number,
            number_format_code,
            prefix_char,
            suffix_char,
            default_char_shape_id,
            para_shape_id,
            style_id,
        );

        let endnote = Endnote {
            number: endnote_number,
            paragraphs: vec![inner_para],
            before_decoration_letter: prefix_char as u16,
            after_decoration_letter: suffix_char as u16,
            number_shape: number_format_code as u32,
            ..Default::default()
        };

        self.document.sections[section_idx].raw_stream = None;
        let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

        let insert_idx = {
            let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
            let mut idx = paragraph.controls.len();
            for (i, &pos) in positions.iter().enumerate() {
                if pos > char_offset {
                    idx = i;
                    break;
                }
            }
            idx
        };

        paragraph
            .controls
            .insert(insert_idx, Control::Endnote(Box::new(endnote)));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        if !paragraph.char_offsets.is_empty() {
            let text_len = paragraph.text.chars().count();
            let safe_offset = char_offset.min(text_len);
            for co in paragraph.char_offsets[safe_offset..].iter_mut() {
                *co += 8;
            }
        }
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 0x0011;
        paragraph.has_para_text = true;

        let mut next_number = start_number;
        Self::renumber_paragraph_endnotes_with_shape(
            &mut self.document.sections[section_idx].paragraphs,
            &mut next_number,
            number_format_code,
            prefix_char,
            suffix_char,
        );

        self.reflow_footnote_paragraph(section_idx, para_idx, insert_idx, 0);

        {
            use crate::renderer::composer::reflow_line_segs;
            use crate::renderer::hwpunit_to_px;
            let page_def = &self.document.sections[section_idx].section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            let available_width = hwpunit_to_px(text_width, self.dpi);
            let para_style = self.styles.para_styles.get(
                self.document.sections[section_idx].paragraphs[para_idx].para_shape_id as usize,
            );
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let final_width = (available_width - margin_left - margin_right).max(0.0);
            let body_para = &mut self.document.sections[section_idx].paragraphs[para_idx];
            reflow_line_segs(body_para, final_width, &self.styles, self.dpi);
        }

        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{},\"endnoteNumber\":{}}}",
            para_idx, insert_idx, endnote_number
        ))
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
            shape.separator_length = v.max(0);
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

        Ok(super::super::helpers::json_ok())
    }

    /// 본문 문단에 수식을 삽입한다 (표 셀/글상자 내부는 미지원).
    /// 커서 위치에 수식 컨트롤을 추가한다.
    /// 반환: JSON `{"ok":true, "paraIdx":N, "controlIdx":N}`
    pub fn insert_equation_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        script: &str,
        font_size: u32,
        color: u32,
    ) -> Result<String, HwpError> {
        use crate::model::control::Equation;
        use crate::model::shape::CommonObjAttr;
        use crate::parser::tags::CTRL_EQUATION;

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

        let equation = Equation {
            common: CommonObjAttr {
                ctrl_id: CTRL_EQUATION,
                treat_as_char: true,
                width: 0,
                height: 0,
                ..Default::default()
            },
            script: script.to_string(),
            font_size,
            color,
            font_name: "HYhwpEQ".to_string(),
            ..Default::default()
        };

        self.document.sections[section_idx].raw_stream = None;
        let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

        let insert_idx = {
            let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
            let mut idx = paragraph.controls.len();
            for (i, &pos) in positions.iter().enumerate() {
                if pos > char_offset {
                    idx = i;
                    break;
                }
            }
            idx
        };

        paragraph
            .controls
            .insert(insert_idx, Control::Equation(Box::new(equation)));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        if !paragraph.char_offsets.is_empty() {
            let text_len = paragraph.text.chars().count();
            let safe_offset = char_offset.min(text_len);
            for co in paragraph.char_offsets[safe_offset..].iter_mut() {
                *co += 8;
            }
        }
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 11;
        paragraph.has_para_text = true;

        // 본문 문단 리플로우
        {
            use crate::renderer::composer::reflow_line_segs;
            use crate::renderer::hwpunit_to_px;
            let page_def = &self.document.sections[section_idx].section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            let available_width = hwpunit_to_px(text_width, self.dpi);
            let para_style = self.styles.para_styles.get(
                self.document.sections[section_idx].paragraphs[para_idx].para_shape_id as usize,
            );
            let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
            let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
            let final_width = (available_width - margin_left - margin_right).max(0.0);
            let body_para = &mut self.document.sections[section_idx].paragraphs[para_idx];
            reflow_line_segs(body_para, final_width, &self.styles, self.dpi);
        }

        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        self.event_log.push(DocumentEvent::PictureInserted {
            section: section_idx,
            para: para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{}}}",
            para_idx, insert_idx
        ))
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

impl crate::document_core::DocumentCore {
    pub fn insert_new_number_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        start_num: u16,
    ) -> Result<String, crate::error::HwpError> {
        use crate::error::HwpError;
        use crate::model::control::{AutoNumberType, Control, NewNumber};

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

        let new_number = NewNumber {
            number_type: AutoNumberType::Page,
            number: start_num,
        };

        self.document.sections[section_idx].raw_stream = None;
        let paragraph = &mut self.document.sections[section_idx].paragraphs[para_idx];

        let insert_idx = {
            let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
            let mut idx = paragraph.controls.len();
            for (i, &pos) in positions.iter().enumerate() {
                if pos > char_offset {
                    idx = i;
                    break;
                }
            }
            idx
        };

        paragraph
            .controls
            .insert(insert_idx, Control::NewNumber(new_number));
        paragraph.ctrl_data_records.insert(insert_idx, None);

        if !paragraph.char_offsets.is_empty() {
            let text_len = paragraph.text.chars().count();
            let safe_offset = char_offset.min(text_len);
            for co in paragraph.char_offsets[safe_offset..].iter_mut() {
                *co += 8;
            }
        }
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 0x0012;
        paragraph.has_para_text = true;

        self.reflow_paragraph(section_idx, para_idx);
        self.recompose_section(section_idx);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();

        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"controlIdx\":{}",
            insert_idx
        )))
    }
}

#[cfg(test)]
mod issue_1151_cell_picture_insert_tests {
    //! Issue #1151: 표 셀 안 이미지 삽입이 항상 표 밖 본문 문단에 들어가는 결함.
    //!
    //! v2 설계 — 한컴 정합 floating picture 접근:
    //! 셀 안 삽입 (cell_path 비어있지 않음) 시 picture 는 셀 내부 paragraph 에
    //! inline 삽입되지 않고, 표가 있는 같은 paragraph 의 sibling control 로
    //! floating (tac=false) 삽입된다. 셀 자체는 비어있는 채로 유지되어 사용자가
    //! 클릭으로 cursor 를 셀에 위치시킬 수 있다.

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
        core.set_document(doc);
        core
    }

    fn minimal_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00,
            0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    fn collect_picture_transparencies(doc: &Document) -> Vec<u8> {
        let mut values = Vec::new();
        for section in &doc.sections {
            collect_picture_transparencies_from_paragraphs(&section.paragraphs, &mut values);
        }
        values
    }

    fn collect_picture_transparencies_from_paragraphs(
        paragraphs: &[Paragraph],
        values: &mut Vec<u8>,
    ) {
        for para in paragraphs {
            for control in &para.controls {
                collect_picture_transparencies_from_control(control, values);
            }
        }
    }

    fn collect_picture_transparencies_from_control(control: &Control, values: &mut Vec<u8>) {
        match control {
            Control::Picture(pic) => {
                values.push(pic.image_attr.clamped_transparency());
                if let Some(caption) = &pic.caption {
                    collect_picture_transparencies_from_paragraphs(&caption.paragraphs, values);
                }
            }
            Control::Table(table) => {
                for cell in &table.cells {
                    collect_picture_transparencies_from_paragraphs(&cell.paragraphs, values);
                }
            }
            Control::Shape(shape) => collect_picture_transparencies_from_shape(shape, values),
            Control::Header(header) => {
                collect_picture_transparencies_from_paragraphs(&header.paragraphs, values);
            }
            Control::Footer(footer) => {
                collect_picture_transparencies_from_paragraphs(&footer.paragraphs, values);
            }
            Control::Footnote(footnote) => {
                collect_picture_transparencies_from_paragraphs(&footnote.paragraphs, values);
            }
            Control::Endnote(endnote) => {
                collect_picture_transparencies_from_paragraphs(&endnote.paragraphs, values);
            }
            _ => {}
        }
    }

    fn collect_picture_transparencies_from_shape(
        shape: &crate::model::shape::ShapeObject,
        values: &mut Vec<u8>,
    ) {
        match shape {
            crate::model::shape::ShapeObject::Picture(pic) => {
                values.push(pic.image_attr.clamped_transparency());
                if let Some(caption) = &pic.caption {
                    collect_picture_transparencies_from_paragraphs(&caption.paragraphs, values);
                }
            }
            crate::model::shape::ShapeObject::Group(group) => {
                for child in &group.children {
                    collect_picture_transparencies_from_shape(child, values);
                }
                if let Some(caption) = &group.caption {
                    collect_picture_transparencies_from_paragraphs(&caption.paragraphs, values);
                }
            }
            _ => {
                if let Some(drawing) = shape.drawing() {
                    if let Some(text_box) = &drawing.text_box {
                        collect_picture_transparencies_from_paragraphs(
                            &text_box.paragraphs,
                            values,
                        );
                    }
                    if let Some(caption) = &drawing.caption {
                        collect_picture_transparencies_from_paragraphs(&caption.paragraphs, values);
                    }
                }
            }
        }
    }

    fn parse_idx(res: &str, key: &str) -> usize {
        res.split(&format!("\"{}\":", key))
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("missing {key} in {res}"))
    }

    #[test]
    fn issue1151_insert_picture_into_table_cell_is_floating_sibling() {
        let mut core = make_test_core();

        // 1×1 표 생성
        let table_res = core
            .create_table_native(0, 0, 0, 1, 1)
            .expect("create 1x1 table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");

        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 0, 0)];
        let image = minimal_png();
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            5000,
            5000,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert picture (floating)");

        // 셀 안은 그대로 비어있어야 한다 (floating 은 셀에 들어가지 않음).
        let table_ctrl =
            &core.document.sections[0].paragraphs[table_para_idx].controls[table_ctrl_idx];
        let table = match table_ctrl {
            Control::Table(t) => t,
            _ => panic!("expected Control::Table"),
        };
        let cell0_para0 = &table.cells[0].paragraphs[0];
        assert!(
            cell0_para0
                .controls
                .iter()
                .all(|c| !matches!(c, Control::Picture(_))),
            "cell 안에 picture 가 들어가면 안 된다 (floating 방식). got: {:?}",
            cell0_para0.controls
        );

        // table 같은 paragraph 의 sibling control 로 Picture 가 존재해야 한다.
        let parent_para = &core.document.sections[0].paragraphs[table_para_idx];
        let picture = parent_para
            .controls
            .iter()
            .find_map(|c| match c {
                Control::Picture(p) => Some(p.as_ref()),
                _ => None,
            })
            .expect("expected sibling Picture in parent paragraph");

        // floating 속성 검증
        assert!(
            !picture.common.treat_as_char,
            "floating picture 는 treat_as_char=false 여야 한다"
        );
        assert!(
            matches!(
                picture.common.text_wrap,
                crate::model::shape::TextWrap::Square
            ),
            "floating picture wrap=Square (어울림) 이어야 한다. got: {:?}",
            picture.common.text_wrap
        );
    }

    #[test]
    fn issue1151_v9_insert_picture_body_floating_default() {
        // [Task #1151 v9 결함 E] 한컴 native 정합: 본문 picture 신규 삽입 시 default =
        // tac=false (floating, 글자처럼 미체크). 셀 분기와 동일 패턴.
        let mut core = make_test_core();
        let image = minimal_png();
        core.insert_picture_native(
            0,
            0,
            0,
            &[], // 빈 cell_path → 본문 floating (v9 fix 후)
            &image,
            5000,
            5000,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert picture body");

        let body_para = &core.document.sections[0].paragraphs[0];
        let pic_in_body = body_para.controls.iter().find_map(|c| match c {
            Control::Picture(p) => Some(p.as_ref()),
            _ => None,
        });
        let picture = pic_in_body.expect("expected Picture in body paragraph (sibling control)");

        // 한컴 native 정합: tac=false, rel_to=Paper, wrap=Square
        assert!(
            !picture.common.treat_as_char,
            "본문 picture default = tac=false (한컴 native 정합, v9 결함 E fix)"
        );
        assert!(
            matches!(
                picture.common.horz_rel_to,
                crate::model::shape::HorzRelTo::Paper
            ),
            "본문 picture horz_rel_to = Paper (셀 분기와 동일)"
        );
        assert!(
            matches!(
                picture.common.vert_rel_to,
                crate::model::shape::VertRelTo::Paper
            ),
            "본문 picture vert_rel_to = Paper"
        );
        assert!(matches!(
            picture.common.text_wrap,
            crate::model::shape::TextWrap::Square
        ));

        // 새 paragraph 생성 안 함 — 기존 paragraph 의 sibling control 로 append
        assert_eq!(
            core.document.sections[0].paragraphs.len(),
            1,
            "본문 picture 삽입 시 새 paragraph 생성 안 함 (sibling control)"
        );
    }

    #[test]
    fn issue1452_insert_picture_returns_logical_offset_after_picture() {
        let mut core = make_test_core();
        core.insert_text_native(0, 0, 0, "abc")
            .expect("insert text");

        let image = minimal_png();
        let result = core
            .insert_picture_native(
                0,
                0,
                3,
                &[],
                &image,
                5000,
                5000,
                1,
                1,
                "png",
                "test",
                None,
                None,
            )
            .expect("insert picture body");

        assert_eq!(parse_idx(&result, "paraIdx"), 0);
        assert_eq!(parse_idx(&result, "controlIdx"), 0);
        assert_eq!(
            parse_idx(&result, "logicalOffset"),
            4,
            "본문 텍스트 'abc' 뒤에 그림 1개를 넣으면 그림 뒤 커서 offset은 4여야 한다: {result}"
        );
    }

    #[test]
    fn issue1452_enter_after_dropped_inline_picture_keeps_next_para_below_picture() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        fn collect_image_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Image(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for child in &node.children {
                collect_image_bboxes(child, out);
            }
        }

        fn collect_para_end_runs(
            node: &RenderNode,
            out: &mut Vec<(usize, Option<usize>, f64, f64, f64, f64)>,
        ) {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                if run.is_para_end {
                    if let Some(para_idx) = run.para_index {
                        out.push((
                            para_idx,
                            run.char_start,
                            node.bbox.x,
                            node.bbox.y,
                            node.bbox.width,
                            node.bbox.height,
                        ));
                    }
                }
            }
            for child in &node.children {
                collect_para_end_runs(child, out);
            }
        }

        let mut core = make_test_core();
        let image = minimal_png();
        let pic_w = 30000u32;
        let pic_h = 9000u32;

        let result = core
            .insert_picture_native(
                0,
                0,
                0,
                &[],
                &image,
                pic_w,
                pic_h,
                1,
                1,
                "png",
                "drop",
                None,
                None,
            )
            .expect("insert dropped picture");
        let ctrl_idx = parse_idx(&result, "controlIdx");
        let logical_offset = parse_idx(&result, "logicalOffset");

        core.set_picture_properties_native(0, 0, ctrl_idx, r#"{"treatAsChar":true}"#)
            .expect("dropped picture becomes treat-as-char");
        core.split_paragraph_native(0, 0, logical_offset)
            .expect("Enter after dropped picture");

        assert_eq!(
            core.document.sections[0].paragraphs.len(),
            2,
            "그림 뒤 Enter 는 새 빈 문단을 만들어야 한다"
        );
        assert_eq!(
            core.document.sections[0].paragraphs[0].line_segs[0].line_height, pic_h as i32,
            "TAC 그림만 남은 첫 문단은 그림 높이를 줄 높이로 유지해야 한다"
        );
        assert!(
            core.document.sections[0].paragraphs[1].line_segs[0].line_height < pic_h as i32 / 2,
            "새 빈 문단은 그림 높이를 물려받지 않고 기본 줄 높이로 시작해야 한다"
        );

        let tree = core.build_page_tree(0).expect("build page tree");
        let mut images = Vec::new();
        collect_image_bboxes(&tree.root, &mut images);
        assert_eq!(images.len(), 1, "drop 그림 ImageNode 1개 필요");

        let mut para_ends = Vec::new();
        collect_para_end_runs(&tree.root, &mut para_ends);
        let image = images[0];
        let image_right = image.0 + image.2;
        let image_bottom = image.1 + image.3;
        let para0_end = para_ends
            .iter()
            .find(|(para_idx, _, _, _, _, _)| *para_idx == 0)
            .expect("첫 문단 끝 표시");
        let para1_end = para_ends
            .iter()
            .find(|(para_idx, _, _, _, _, _)| *para_idx == 1)
            .expect("새 빈 문단 끝 표시");

        assert_eq!(
            para0_end.1,
            Some(logical_offset),
            "첫 문단 끝 표시는 그림 뒤 logical offset에 놓여야 한다"
        );
        assert!(
            para0_end.2 >= image_right - 0.5,
            "첫 문단부호 x는 그림 뒤에 있어야 한다: mark_x={}, image_right={}",
            para0_end.2,
            image_right
        );
        assert!(
            para1_end.3 >= image_bottom - 0.5,
            "새 빈 문단부호는 그림 아래 줄에 있어야 한다: mark_y={}, image_bottom={}",
            para1_end.3,
            image_bottom
        );
    }

    #[test]
    fn issue1452_picture_text_wrap_updates_hwp_attr_bits() {
        let mut core = make_test_core();
        let image = minimal_png();
        core.insert_picture_native(
            0,
            0,
            0,
            &[],
            &image,
            5000,
            5000,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert picture body");

        {
            let pic = match &mut core.document.sections[0].paragraphs[0].controls[0] {
                Control::Picture(p) => p.as_mut(),
                _ => panic!("expected picture"),
            };
            pic.common.attr |= 1 << 30;
        }

        let cases = [
            (
                "InFrontOfText",
                crate::model::shape::TextWrap::InFrontOfText,
                3u32,
            ),
            (
                "BehindText",
                crate::model::shape::TextWrap::BehindText,
                2u32,
            ),
            (
                "TopAndBottom",
                crate::model::shape::TextWrap::TopAndBottom,
                1u32,
            ),
            ("Square", crate::model::shape::TextWrap::Square, 0u32),
        ];

        for (name, expected_wrap, expected_bits) in cases {
            let json = format!(r#"{{"textWrap":"{}"}}"#, name);
            core.set_picture_properties_native(0, 0, 0, &json)
                .unwrap_or_else(|err| panic!("set textWrap={name} failed: {err}"));
            let pic = match &core.document.sections[0].paragraphs[0].controls[0] {
                Control::Picture(p) => p.as_ref(),
                _ => panic!("expected picture"),
            };
            assert_eq!(pic.common.text_wrap, expected_wrap);
            assert_eq!(
                (pic.common.attr >> 21) & 0x07,
                expected_bits,
                "HWP 저장용 attr textWrap bit가 stale이면 안 된다: {name}"
            );
            assert_ne!(
                pic.common.attr & (1 << 30),
                0,
                "알 수 없는 원본 attr 비트는 보존되어야 한다"
            );
        }
    }

    #[test]
    fn issue1452_picture_transparency_props_roundtrip() {
        let mut core = make_test_core();
        let image = minimal_png();
        core.insert_picture_native(
            0,
            0,
            0,
            &[],
            &image,
            5000,
            5000,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert picture body");

        core.set_picture_properties_native(0, 0, 0, r#"{"transparency":50}"#)
            .expect("set transparency");
        let props = core
            .get_picture_properties_native(0, 0, 0)
            .expect("get picture properties");
        assert!(
            props.contains(r#""transparency":50"#),
            "그림 속성 JSON은 투명도 50%를 반환해야 한다: {props}"
        );

        let pic = match &core.document.sections[0].paragraphs[0].controls[0] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!("expected picture"),
        };
        assert_eq!(pic.image_attr.clamped_transparency(), 50);
        assert!((pic.image_attr.opacity() - 0.5).abs() < f64::EPSILON);

        core.set_picture_properties_native(0, 0, 0, r#"{"transparency":200}"#)
            .expect("set clamped transparency");
        let pic = match &core.document.sections[0].paragraphs[0].controls[0] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!("expected picture"),
        };
        assert_eq!(
            pic.image_attr.clamped_transparency(),
            100,
            "속성 API로 들어온 범위 밖 투명도는 0~100으로 clamp되어야 한다"
        );
    }

    #[test]
    fn issue1452_picture_transparency_samples_parse_as_ui_percent() {
        for path in ["samples/투명도0-50.hwp", "samples/투명도0-50.hwpx"] {
            let data =
                std::fs::read(path).unwrap_or_else(|err| panic!("fixture 읽기 실패 {path}: {err}"));
            let core =
                DocumentCore::from_bytes(&data).unwrap_or_else(|err| panic!("parse {path}: {err}"));
            let transparencies = collect_picture_transparencies(&core.document);
            assert!(
                transparencies.len() >= 2,
                "샘플에는 최소 두 개의 그림이 있어야 한다: {path}, got {transparencies:?}"
            );
            assert_eq!(
                &transparencies[..2],
                &[0, 50],
                "샘플 첫 번째/두 번째 그림 투명도는 각각 0%, 50%여야 한다: {path}"
            );
        }
    }

    #[test]
    fn issue1452_picture_transparency_samples_render_once_with_opacity() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        fn collect_images(node: &RenderNode, out: &mut Vec<(Option<usize>, Option<usize>, f64)>) {
            if let RenderNodeType::Image(img) = &node.node_type {
                out.push((img.para_index, img.control_index, img.opacity));
            }
            for child in &node.children {
                collect_images(child, out);
            }
        }

        for path in ["samples/투명도0-50.hwp", "samples/투명도0-50.hwpx"] {
            let data =
                std::fs::read(path).unwrap_or_else(|err| panic!("fixture 읽기 실패 {path}: {err}"));
            let core =
                DocumentCore::from_bytes(&data).unwrap_or_else(|err| panic!("parse {path}: {err}"));
            let tree = core
                .build_page_tree(0)
                .unwrap_or_else(|err| panic!("render tree {path}: {err}"));
            let mut images = Vec::new();
            collect_images(&tree.root, &mut images);

            assert_eq!(
                images.len(),
                2,
                "투명도 샘플의 그림은 두 번만 렌더되어야 한다: {path}, got {images:?}"
            );

            let mut identities = images
                .iter()
                .map(|(para, control, _)| (*para, *control))
                .collect::<Vec<_>>();
            identities.sort_unstable();
            identities.dedup();
            assert_eq!(
                identities.len(),
                2,
                "같은 그림 control 이 중복 렌더되면 안 된다: {path}, got {images:?}"
            );

            let mut opacities = images
                .iter()
                .map(|(_, _, opacity)| (opacity * 100.0).round() as i32)
                .collect::<Vec<_>>();
            opacities.sort_unstable();
            assert_eq!(
                opacities,
                vec![50, 100],
                "렌더 트리 불투명도는 투명도 0/50%를 100/50%로 보존해야 한다: {path}"
            );
        }
    }

    #[test]
    fn issue1452_enter_after_second_tac_picture_keeps_both_pictures() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        fn collect_images(node: &RenderNode, out: &mut Vec<(Option<usize>, Option<usize>, f64)>) {
            if let RenderNodeType::Image(img) = &node.node_type {
                out.push((img.para_index, img.control_index, img.opacity));
            }
            for child in &node.children {
                collect_images(child, out);
            }
        }

        let data = std::fs::read("samples/투명도0-50.hwp")
            .expect("fixture 읽기 실패 samples/투명도0-50.hwp");
        let mut core = DocumentCore::from_bytes(&data).expect("parse samples/투명도0-50.hwp");

        core.split_paragraph_native(0, 0, 2)
            .expect("두 번째 TAC 그림 뒤 Enter");

        assert_eq!(
            core.document.sections[0].paragraphs.len(),
            2,
            "그림 뒤 Enter 는 새 빈 문단을 만들어야 한다"
        );
        assert!(
            core.document.sections[0].paragraphs[0].line_segs.len() >= 2,
            "원래 문단은 두 TAC 그림 줄을 유지해야 한다: {:?}",
            core.document.sections[0].paragraphs[0].line_segs
        );

        let tree = core.build_page_tree(0).expect("build page tree");
        let mut images = Vec::new();
        collect_images(&tree.root, &mut images);
        assert_eq!(
            images.len(),
            2,
            "Enter 후에도 두 그림이 모두 렌더되어야 한다: {images:?}"
        );

        let mut identities = images
            .iter()
            .map(|(para, control, _)| (*para, *control))
            .collect::<Vec<_>>();
        identities.sort_unstable();
        identities.dedup();
        assert_eq!(
            identities.len(),
            2,
            "두 그림 control 이 각각 렌더되어야 한다: {images:?}"
        );
    }

    #[test]
    fn issue1151_invalid_cell_path_returns_error() {
        let mut core = make_test_core();
        let _ = core
            .create_table_native(0, 0, 0, 1, 1)
            .expect("create table");
        let bad_path: Vec<(usize, usize, usize)> = vec![(0, 5, 0)]; // cell 5 는 1×1 표에 없음
        let image = minimal_png();
        let res = core.insert_picture_native(
            0, 0, 0, &bad_path, &image, 5000, 5000, 1, 1, "png", "test", None, None,
        );
        assert!(
            res.is_err(),
            "out-of-range cell path → Err 기대, got {res:?}"
        );
    }
}

#[cfg(test)]
mod issue_1151_v2_tac_toggle_tests {
    //! Issue #1151 v2: floating picture → "글자처럼 취급" 토글 시 한컴 정합 (H1).
    //!
    //! 한컴 산출물 분석 (samples/tac-verify/scenario-{a,b,c,d}-after.hwp) 결과:
    //! tac false→true 토글 시 picture 의 control 위치는 불변이고, 4 가지 필드만
    //! 갱신된다. (a) treat_as_char=true, (b) horz/vert_rel_to=Para, (c) h/v_offset=0,
    //! (d) parent paragraph 의 line_segs[0] 의 line_height = picture height,
    //!     text_height = picture height, baseline_distance = round(lh*0.85).
    //! paragraph.text / char_offsets / paragraph 수 변화 없음.

    use super::*;
    use crate::model::document::{Document, Section, SectionDef};
    use crate::model::image::{ImageAttr, Picture};
    use crate::model::page::PageDef;
    use crate::model::paragraph::LineSeg;
    use crate::model::shape::{CommonObjAttr, HorzRelTo, ShapeComponentAttr, TextWrap, VertRelTo};

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
        core.set_document(doc);
        core
    }

    fn minimal_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00,
            0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    fn parse_idx(res: &str, key: &str) -> usize {
        res.split(&format!("\"{}\":", key))
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("missing {key} in {res}"))
    }

    /// 본문 (또는 임의 paragraph) 에 floating picture 를 직접 push 한다.
    /// 한컴이 만든 floating picture 의 model 상태 (tac=false, Paper-relative, offset 있음)
    /// 와 동등.
    fn push_body_floating_picture(
        para: &mut Paragraph,
        width_hu: u32,
        height_hu: u32,
        offset_h: u32,
        offset_v: u32,
        bin_id: u16,
    ) -> usize {
        let common_attr: u32 = (1 << 3) | (1 << 8) | (4 << 15) | (2 << 18);
        let pic = Picture {
            common: CommonObjAttr {
                ctrl_id: 0x67736F20,
                attr: common_attr,
                treat_as_char: false,
                vert_rel_to: VertRelTo::Paper,
                horz_rel_to: HorzRelTo::Paper,
                text_wrap: TextWrap::Square,
                horizontal_offset: offset_h,
                vertical_offset: offset_v,
                width: width_hu,
                height: height_hu,
                z_order: 0,
                ..Default::default()
            },
            shape_attr: ShapeComponentAttr {
                original_width: width_hu,
                original_height: height_hu,
                current_width: width_hu,
                current_height: height_hu,
                ..Default::default()
            },
            border_x: [0i32, 0, width_hu as i32, 0],
            border_y: [width_hu as i32, height_hu as i32, 0, height_hu as i32],
            image_attr: ImageAttr {
                bin_data_id: bin_id,
                ..Default::default()
            },
            ..Default::default()
        };
        let idx = para.controls.len();
        para.controls.push(Control::Picture(Box::new(pic)));
        para.ctrl_data_records.push(None);
        idx
    }

    /// 한컴 산출물에서 관찰된 baseline 비율: lh × 0.85 (round).
    fn expected_baseline(lh: i32) -> i32 {
        (lh as f64 * 0.85).round() as i32
    }

    // ─── Scenario A 등가 ───────────────────────────────────────────────
    #[test]
    fn tac_toggle_table_sibling_floating_to_inline() {
        let mut core = make_test_core();

        // 1×1 표 생성
        let table_res = core
            .create_table_native(0, 0, 0, 1, 1)
            .expect("create 1x1 table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");

        // 셀 안 floating picture 삽입 (v1 path, h=5331 HU)
        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 0, 0)];
        let image = minimal_png();
        let pic_w = 5977u32;
        let pic_h = 5331u32;
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            pic_w,
            pic_h,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert floating picture in cell");

        // picture 는 표 sibling 위치 (= 마지막 control)
        let pic_ctrl_idx = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;
        let before_paragraph_count = core.document.sections[0].paragraphs.len();
        let before_controls_count = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len();

        // tac false→true 토글
        let res = core.set_picture_properties_native(
            0,
            table_para_idx,
            pic_ctrl_idx,
            r#"{"treatAsChar":true}"#,
        );
        assert!(res.is_ok(), "set_picture_properties_native failed: {res:?}");

        let para = &core.document.sections[0].paragraphs[table_para_idx];
        let pic = match &para.controls[pic_ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!("picture not at expected ctrl_idx"),
        };

        // (1) picture 위치 / paragraph 수 불변
        assert_eq!(para.controls.len(), before_controls_count);
        assert_eq!(
            core.document.sections[0].paragraphs.len(),
            before_paragraph_count
        );

        // (2) 4 필드 갱신
        assert!(pic.common.treat_as_char, "treat_as_char true");
        assert_eq!(pic.common.attr & 0x01, 0x01, "attr 비트 0 셋");
        assert!(
            matches!(pic.common.horz_rel_to, HorzRelTo::Para),
            "horz_rel_to=Para, got {:?}",
            pic.common.horz_rel_to
        );
        assert!(
            matches!(pic.common.vert_rel_to, VertRelTo::Para),
            "vert_rel_to=Para, got {:?}",
            pic.common.vert_rel_to
        );
        assert_eq!(pic.common.horizontal_offset, 0, "h_offset=0");
        assert_eq!(pic.common.vertical_offset, 0, "v_offset=0");

        // (3) LINE_SEG[0] 갱신
        let seg = &para.line_segs[0];
        assert_eq!(
            seg.line_height, pic_h as i32,
            "line_segs[0].line_height = picture height"
        );
        assert_eq!(
            seg.text_height, pic_h as i32,
            "line_segs[0].text_height = picture height"
        );
        assert_eq!(
            seg.baseline_distance,
            expected_baseline(pic_h as i32),
            "line_segs[0].baseline_distance = round(lh*0.85)"
        );

        // (4) text / char_offsets 불변 (sentinel char 추가하지 않음)
        assert_eq!(para.text, "");
        assert_eq!(para.char_offsets.len(), 0);
    }

    // ─── [Task #1151 v8 결함 A regression] v1 셀 floating 의 초기 rel_to=Paper ─
    //
    // 사용자 한컴 native 시연 (2026-05-30): 한컴이 셀 안 picture 신규 삽입 시
    // 가로/세로 기준 = "종이" (Paper). v1 plan 의 incellpicture.hwp dump 분석 정합.
    // v1 구현이 Page 로 잘못 설정한 결함을 정정.
    #[test]
    fn v8_cell_floating_picture_uses_paper_rel_to() {
        let mut core = make_test_core();
        let table_res = core
            .create_table_native(0, 0, 0, 1, 1)
            .expect("create 1x1 table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");
        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 0, 0)];
        let image = minimal_png();
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            5977,
            5331,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert floating picture in cell");
        let pic_ctrl_idx = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;
        let para = &core.document.sections[0].paragraphs[table_para_idx];
        let pic = match &para.controls[pic_ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!("not Picture"),
        };

        // (A) typed field 가 Paper
        assert!(
            matches!(pic.common.horz_rel_to, HorzRelTo::Paper),
            "horz_rel_to = Paper (한컴 native default), got {:?}",
            pic.common.horz_rel_to
        );
        assert!(
            matches!(pic.common.vert_rel_to, VertRelTo::Paper),
            "vert_rel_to = Paper, got {:?}",
            pic.common.vert_rel_to
        );

        // (B) attr 비트 정합 — bit 3-4 (vert) = 0, bit 8-10 (horz) = 0 (둘 다 Paper)
        let bits_vert = (pic.common.attr >> 3) & 0b11;
        let bits_horz = (pic.common.attr >> 8) & 0b111;
        assert_eq!(bits_vert, 0, "attr bits 3-4 = Paper(0)");
        assert_eq!(bits_horz, 0, "attr bits 8-10 = Paper(0)");

        // (C) tac=false, wrap=Square 그대로
        assert!(!pic.common.treat_as_char);
        assert!(matches!(
            pic.common.text_wrap,
            crate::model::shape::TextWrap::Square
        ));
    }

    // ─── [Task #1151 v9 결함 D regression v2] 큰 picture 2 장 wrap 시나리오 ───
    //
    // 사용자 시연 (2026-05-30 후속): 큰 picture 2 장 (page 폭 초과) 글자처럼 토글 시
    // 한컴 native 는 wrap (다음 line). Stage 23 fix 첫 버전은 pic_y 결정이 pic_x wrap
    // 처리 전이라 wrap 후 line_top_y 가 갱신됐어도 pic_y 가 wrap 전 값 → 두 picture
    // 같은 위치 겹침. Fix: pic_y 결정을 pic_x 뒤로 옮김 (wrap 후 state 반영).
    #[test]
    fn v9_two_large_pictures_wrap_to_next_line() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        fn collect_image_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Image(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for child in &node.children {
                collect_image_bboxes(child, out);
            }
        }

        let mut core = make_test_core();
        let table_res = core.create_table_native(0, 0, 0, 1, 1).expect("table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");
        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 0, 0)];
        let image = minimal_png();

        // 큰 picture 2 장 — 각 80mm × 60mm (22680 × 17010 HU)
        // page 본문 폭 ≈ 150mm. 두 picture 합 160mm > 150mm → wrap 발생해야 함.
        let pic_w = 22680u32;
        let pic_h = 17010u32;

        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            pic_w,
            pic_h,
            1,
            1,
            "png",
            "p1",
            None,
            None,
        )
        .expect("insert pic1");
        let pic1_ctrl = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;
        core.set_picture_properties_native(0, table_para_idx, pic1_ctrl, r#"{"treatAsChar":true}"#)
            .expect("toggle pic1");

        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            pic_w,
            pic_h,
            1,
            1,
            "png",
            "p2",
            None,
            None,
        )
        .expect("insert pic2");
        let pic2_ctrl = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;
        core.set_picture_properties_native(0, table_para_idx, pic2_ctrl, r#"{"treatAsChar":true}"#)
            .expect("toggle pic2");

        let tree = core.build_page_tree_cached(0).expect("build");
        let mut images = vec![];
        collect_image_bboxes(&tree.root, &mut images);
        let pic_h_px = pic_h as f64 * 96.0 / 7200.0;

        assert_eq!(images.len(), 2, "두 picture 모두 render 되어야 함");
        let (x1, y1, _, _) = images[0];
        let (x2, y2, _, _) = images[1];

        // (A) 둘째 picture y 가 첫 picture y + pic_h 만큼 진행 (wrap)
        let y_diff = y2 - y1;
        assert!(
            (y_diff - pic_h_px).abs() < 1.0,
            "wrap: y_diff {:.2} ≈ pic_h {:.2} (한 picture height 만큼 진행) — got y1={}, y2={}",
            y_diff,
            pic_h_px,
            y1,
            y2
        );

        // (B) x 동일 (wrap 후 둘째 picture 가 새 line 의 좌측에서 시작)
        assert!(
            (x1 - x2).abs() < 1.0,
            "wrap: x 동일 (둘 다 새 line 의 좌측) — got x1={}, x2={}",
            x1,
            x2
        );
    }

    #[test]
    fn v9_two_tac_pictures_horizontal_distribute() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        fn collect_image_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Image(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for child in &node.children {
                collect_image_bboxes(child, out);
            }
        }

        let mut core = make_test_core();
        let table_res = core.create_table_native(0, 0, 0, 1, 1).expect("table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");
        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 0, 0)];
        let image = minimal_png();

        // picture 1 삽입 + tac
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            5670,
            5670,
            1,
            1,
            "png",
            "test1",
            None,
            None,
        )
        .expect("insert pic1");
        let pic1_ctrl = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;
        core.set_picture_properties_native(0, table_para_idx, pic1_ctrl, r#"{"treatAsChar":true}"#)
            .expect("toggle pic1");

        // picture 2 삽입 + tac
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            5670,
            5670,
            1,
            1,
            "png",
            "test2",
            None,
            None,
        )
        .expect("insert pic2");
        let pic2_ctrl = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;
        core.set_picture_properties_native(0, table_para_idx, pic2_ctrl, r#"{"treatAsChar":true}"#)
            .expect("toggle pic2");

        // render tree 의 image bbox 검증
        let tree = core.build_page_tree_cached(0).expect("build page 0");
        let mut images = vec![];
        collect_image_bboxes(&tree.root, &mut images);
        assert_eq!(images.len(), 2, "두 picture 모두 render 되어야 함");

        let (x1, y1, w1, _h1) = images[0];
        let (x2, y2, _w2, _h2) = images[1];

        // (A) y 동일 (한 line) — 가로 분배 정합
        assert!(
            (y1 - y2).abs() < 0.5,
            "두 picture y 동일 (가로 분배) — got y1={}, y2={}",
            y1,
            y2
        );

        // (B) x 다름 (가로 누적) — pic2 x = pic1 x + pic1 width
        assert!(
            x2 > x1 + 0.5,
            "두 picture x 다름 (가로 누적) — got x1={}, x2={}",
            x1,
            x2
        );
        assert!(
            (x2 - (x1 + w1)).abs() < 0.5,
            "pic2 x ≈ pic1 x + pic1 width — got x1={}, x2={}, w1={}",
            x1,
            x2,
            w1
        );
    }

    // ─── Scenario D 등가 ───────────────────────────────────────────────
    #[test]
    fn tac_toggle_body_floating_to_inline() {
        let mut core = make_test_core();
        let pic_h = 19019u32;
        let pic_w = 20863u32;
        let ctrl_idx = {
            let para = &mut core.document.sections[0].paragraphs[0];
            push_body_floating_picture(para, pic_w, pic_h, 13428, 13568, 1)
        };

        let res = core.set_picture_properties_native(0, 0, ctrl_idx, r#"{"treatAsChar":true}"#);
        assert!(res.is_ok(), "set_picture_properties_native failed: {res:?}");

        let para = &core.document.sections[0].paragraphs[0];
        let pic = match &para.controls[ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!("picture not at expected ctrl_idx"),
        };

        assert!(pic.common.treat_as_char);
        assert!(matches!(pic.common.horz_rel_to, HorzRelTo::Para));
        assert!(matches!(pic.common.vert_rel_to, VertRelTo::Para));
        assert_eq!(pic.common.horizontal_offset, 0);
        assert_eq!(pic.common.vertical_offset, 0);

        let seg = &para.line_segs[0];
        assert_eq!(seg.line_height, pic_h as i32);
        assert_eq!(seg.text_height, pic_h as i32);
        assert_eq!(seg.baseline_distance, expected_baseline(pic_h as i32));

        assert_eq!(para.text, "");
        assert_eq!(para.char_offsets.len(), 0);
    }

    // ─── Scenario C 등가 ───────────────────────────────────────────────
    #[test]
    fn tac_toggle_3x3_center_cell_floating_to_inline() {
        let mut core = make_test_core();

        let table_res = core
            .create_table_native(0, 0, 0, 3, 3)
            .expect("create 3x3 table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");

        // (1,1) 중앙 셀의 cell_path: (outer_ctrl_idx, row, col)
        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 1, 1)];
        let image = minimal_png();
        let pic_w = 5434u32;
        let pic_h = 4847u32;
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            pic_w,
            pic_h,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert floating picture in center cell");

        let pic_ctrl_idx = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;
        let res = core.set_picture_properties_native(
            0,
            table_para_idx,
            pic_ctrl_idx,
            r#"{"treatAsChar":true}"#,
        );
        assert!(res.is_ok(), "set_picture_properties_native failed: {res:?}");

        let para = &core.document.sections[0].paragraphs[table_para_idx];
        let pic = match &para.controls[pic_ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!("picture not at expected ctrl_idx"),
        };
        assert!(pic.common.treat_as_char);
        assert_eq!(pic.common.horizontal_offset, 0);
        assert_eq!(pic.common.vertical_offset, 0);
        assert_eq!(para.line_segs[0].line_height, pic_h as i32);
        assert_eq!(
            para.line_segs[0].baseline_distance,
            expected_baseline(pic_h as i32)
        );
    }

    // ─── [Task #1151 v5] v1 path → tac toggle → page tree cache invalidate 검증 ─
    //
    // 사용자 보고 (2026-05-30): "rhwp 신규 표 + 셀 안 이미지 → tac 토글 시
    // 시각 변화 없음". 진단 결과 model + composer + paragraph_layout 모두 정상
    // 동작 (picture 가 표 아래 정확 위치 156.9 px 에 inline 렌더) 인데, studio
    // 가 stale page tree 받음. root cause: set_picture_properties_native 의
    // invalidate_page_tree_cache 호출 누락 — 다른 picture/shape setter (셀 picture
    // by_path / 셀 shape by_path / header-footer / shape 등) 는 모두 호출.
    //
    // 본 테스트는 v1 path → tac toggle 후 build_page_render_tree 가 picture 가
    // 표 아래로 이동한 새 위치로 ImageNode 를 emit 하는지 검증 — cache 갱신 정합.
    #[test]
    fn v5_tac_toggle_invalidates_page_tree_and_emits_inline_picture_below_table() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        fn collect_image_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Image(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for child in &node.children {
                collect_image_bboxes(child, out);
            }
        }
        fn collect_table_bboxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Table(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for child in &node.children {
                collect_table_bboxes(child, out);
            }
        }

        let mut core = make_test_core();
        let table_res = core
            .create_table_native(0, 0, 0, 1, 1)
            .expect("create 1x1 table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");
        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 0, 0)];
        let image = minimal_png();
        let pic_w = 5977u32;
        let pic_h = 5331u32;
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            pic_w,
            pic_h,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert floating picture in cell");
        let pic_ctrl_idx = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;

        // toggle 전: build_page_tree_cached 호출 → cache 채움.
        let tree_before = core
            .build_page_tree_cached(0)
            .expect("build_page_tree_cached pre-toggle");
        let mut image_before: Vec<(f64, f64, f64, f64)> = vec![];
        collect_image_bboxes(&tree_before.root, &mut image_before);
        assert_eq!(image_before.len(), 1, "toggle 전 ImageNode 1 개 필요");
        let (_x0, y_before, _w0, _h0) = image_before[0];

        // tac false → true 토글
        core.set_picture_properties_native(
            0,
            table_para_idx,
            pic_ctrl_idx,
            r#"{"treatAsChar":true}"#,
        )
        .expect("toggle");

        // toggle 후: build_page_tree_cached 다시 호출. fix 적용 시 invalidate_page_tree_cache
        // 가 작동하여 새 tree 반환 (picture 위치 = 표 아래). fix 미적용 시 stale cache 반환.
        let tree_after = core
            .build_page_tree_cached(0)
            .expect("build_page_tree_cached post-toggle");
        let mut image_after: Vec<(f64, f64, f64, f64)> = vec![];
        collect_image_bboxes(&tree_after.root, &mut image_after);
        let mut table_after: Vec<(f64, f64, f64, f64)> = vec![];
        collect_table_bboxes(&tree_after.root, &mut table_after);

        assert_eq!(image_after.len(), 1, "toggle 후 ImageNode 1 개 필요");
        assert_eq!(table_after.len(), 1, "toggle 후 Table 1 개 필요");
        let (_x_a, y_after, _w_a, _h_a) = image_after[0];
        let (_tx, ty, _tw, th) = table_after[0];
        let table_bottom = ty + th;

        // (A) cache invalidate 검증: toggle 전후 picture y 가 다름 (stale cache 아님).
        assert!(
            (y_before - y_after).abs() > 0.5,
            "FAIL: page tree cache invalidate 누락 — toggle 후에도 picture y 동일 (before={}, after={})",
            y_before,
            y_after
        );

        // (B) toggle 후 picture 가 표 아래 위치 (한컴 정합).
        assert!(
            y_after > table_bottom,
            "FAIL: picture 가 표 아래에 미배치 — picture y={}, table bottom={}",
            y_after,
            table_bottom
        );
    }

    // ─── [Task #1151 v6] 한컴 정합 (scenario-a-after.hwp) render tree baseline ──
    //
    // v6 root cause 진단 베이스라인 — 한컴 정합 model 의 render tree 가 표를
    // 정확한 셀 size 로 그리고 picture 가 표 아래에 배치됨을 확인. v6 fix
    // (Table::update_ctrl_dimensions 가 self.common 동기화) 가 적용된 후 rhwp
    // v1 path + 셀 size 조절 + tac toggle 의 render tree 가 이 baseline 과 같은
    // 패턴 (image y > table bottom) 을 따르는지가 v6 fix 정합 기준.
    #[test]
    fn v6_render_tree_scenario_a_after_baseline() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        fn collect_image(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Image(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for c in &node.children {
                collect_image(c, out);
            }
        }
        fn collect_table(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Table(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for c in &node.children {
                collect_table(c, out);
            }
        }

        let bytes = std::fs::read("samples/tac-verify/scenario-a-after.hwp")
            .expect("read scenario-a-after.hwp");
        let doc = crate::parser::parse_hwp(&bytes).expect("parse scenario-a-after.hwp");
        let mut core = DocumentCore::new_empty();
        core.set_document(doc);
        let tree = core.build_page_tree_cached(0).expect("build page 0");
        let mut images = vec![];
        let mut tables = vec![];
        collect_image(&tree.root, &mut images);
        collect_table(&tree.root, &mut tables);

        // baseline 단언: 표 와 picture 가 분리되어 표 아래에 picture 배치
        assert_eq!(tables.len(), 1, "한컴 정합 표 1개");
        assert_eq!(images.len(), 1, "한컴 정합 picture 1개");
        let (_tx, ty, _tw, th) = tables[0];
        let (_ix, iy, _iw, _ih) = images[0];
        assert!(
            iy > ty + th,
            "한컴 baseline: picture 가 표 아래 (iy={}, table_bottom={})",
            iy,
            ty + th
        );
    }

    // ─── [Task #1151 v6 regression] rhwp v1 path + 셀 height 조절 + tac toggle ─
    //
    // Root cause: Table::update_ctrl_dimensions 가 raw_ctrl_data 만 갱신하고
    // self.common.width / self.common.height 는 동기화하지 않아 paragraph_layout 의
    // v3 helper 가 stale 값 (cell 조절 전) 을 사용 → picture 가 표 아래로 충분히
    // 안 밀려나고 표 박스 안에 들어감 (사용자 보고 2026-05-30).
    //
    // Fix: update_ctrl_dimensions 에서 self.common.width / height 동기화.
    // 검증: cell.height = 11498 조절 후 tac toggle → table.common.height == 11498
    // 및 picture y > table bottom.
    #[test]
    fn v6_resize_cell_then_tac_toggle_picture_below_table() {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        fn collect_image(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Image(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for c in &node.children {
                collect_image(c, out);
            }
        }
        fn collect_table(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
            if matches!(node.node_type, RenderNodeType::Table(_)) {
                out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
            }
            for c in &node.children {
                collect_table(c, out);
            }
        }

        let mut core = make_test_core();
        let table_res = core.create_table_native(0, 0, 0, 1, 1).expect("table");
        let table_para_idx = parse_idx(&table_res, "paraIdx");
        let table_ctrl_idx = parse_idx(&table_res, "controlIdx");

        // 셀 height 를 한컴 정합 size (12498 HU) 와 유사하게 조절.
        // default cell.height = 1282 → delta = 12498 - 1282 = 11216
        core.resize_table_cells_native(
            0,
            table_para_idx,
            table_ctrl_idx,
            r#"[{"cellIdx":0,"heightDelta":11216}]"#,
        )
        .expect("resize cell");

        // v6 fix 1: resize 후 table.common.height 가 cell.height 와 동기화
        let table =
            match &core.document.sections[0].paragraphs[table_para_idx].controls[table_ctrl_idx] {
                Control::Table(t) => t,
                _ => panic!(),
            };
        assert_eq!(
            table.common.height, 11498,
            "v6 fix: table.common.height 가 cell 조절 후 동기화 (raw_ctrl_data 뿐 아니라 self.common 도)"
        );
        assert_eq!(table.cells[0].height, 11498);

        // picture 삽입 (v1 path)
        let cell_path: Vec<(usize, usize, usize)> = vec![(table_ctrl_idx, 0, 0)];
        let image = minimal_png();
        core.insert_picture_native(
            0,
            table_para_idx,
            0,
            &cell_path,
            &image,
            5977,
            5331,
            1,
            1,
            "png",
            "test",
            None,
            None,
        )
        .expect("insert");
        let pic_ctrl_idx = core.document.sections[0].paragraphs[table_para_idx]
            .controls
            .len()
            - 1;

        // tac toggle
        core.set_picture_properties_native(
            0,
            table_para_idx,
            pic_ctrl_idx,
            r#"{"treatAsChar":true}"#,
        )
        .expect("toggle");

        // v6 fix 2: render tree 의 picture 가 표 box 아래에 배치되는지 확인.
        let tree = core.build_page_tree_cached(0).expect("build page 0");
        let mut images = vec![];
        let mut tables = vec![];
        collect_image(&tree.root, &mut images);
        collect_table(&tree.root, &mut tables);
        assert_eq!(tables.len(), 1);
        assert_eq!(images.len(), 1);
        let (_tx, ty, _tw, th) = tables[0];
        let (_ix, iy, _iw, _ih) = images[0];
        assert!(
            iy > ty + th,
            "v6 fix: picture 가 표 아래 (iy={}, table_bottom={}) — table.common.height 동기화 정합",
            iy,
            ty + th
        );
    }

    // ─── 이미 tac=true 인 picture 의 다른 속성 변경 — migration 미진입 ─────
    #[test]
    fn tac_toggle_when_already_tac_true_no_migration() {
        let mut core = make_test_core();
        let pic_h = 5000u32;
        let ctrl_idx = {
            let para = &mut core.document.sections[0].paragraphs[0];
            push_body_floating_picture(para, 5000, pic_h, 1000, 1000, 1)
        };

        // 먼저 tac=true 로 마이그레이션
        core.set_picture_properties_native(0, 0, ctrl_idx, r#"{"treatAsChar":true}"#)
            .expect("first migration");
        let lh_after_first = core.document.sections[0].paragraphs[0].line_segs[0].line_height;

        // 두 번째 호출: tac 변경 없이 다른 속성 변경 — migration 미진입
        core.set_picture_properties_native(0, 0, ctrl_idx, r#"{"brightness":50}"#)
            .expect("second call no-op for migration");

        let para = &core.document.sections[0].paragraphs[0];
        // line_height 가 더 자라지 않아야 함 (이미 picture height 인 채로 유지)
        assert_eq!(para.line_segs[0].line_height, lh_after_first);
        // brightness 는 적용됨
        let pic = match &para.controls[ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!(),
        };
        assert_eq!(pic.image_attr.brightness, 50);
    }

    // ─── tac=true → false 토글 — 빈 그림 문단 LINE_SEG 재구성 ──────────
    #[test]
    fn tac_toggle_true_to_false_restores_empty_picture_para_line_seg() {
        let mut core = make_test_core();
        let pic_h = 5000u32;
        let ctrl_idx = {
            let para = &mut core.document.sections[0].paragraphs[0];
            push_body_floating_picture(para, 5000, pic_h, 1000, 1000, 1)
        };
        // 먼저 tac=true 로
        core.set_picture_properties_native(0, 0, ctrl_idx, r#"{"treatAsChar":true}"#)
            .expect("forward migration");
        let lh_after_forward = core.document.sections[0].paragraphs[0].line_segs[0].line_height;
        assert_eq!(lh_after_forward, pic_h as i32);

        // tac=false 로 — 빈 그림 전용 문단에는 더 이상 inline 슬롯이 없으므로 기본 빈 줄로 복원.
        core.set_picture_properties_native(0, 0, ctrl_idx, r#"{"treatAsChar":false}"#)
            .expect("reverse toggle");
        let para = &core.document.sections[0].paragraphs[0];
        assert_eq!(para.line_segs.len(), 1);
        assert_eq!(
            para.line_segs[0].line_height, 1000,
            "남은 TAC 개체가 없으면 기본 빈 줄 높이로 복원"
        );
        assert_eq!(
            para.line_segs[0].baseline_distance, 850,
            "기본 빈 줄 기준선으로 복원"
        );
        let pic = match &para.controls[ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!(),
        };
        assert!(!pic.common.treat_as_char, "tac 비트는 false 로 토글");
    }

    // ─── 빈 line_segs paragraph 의 토글 — line_seg 신설 ────────────────
    #[test]
    fn tac_toggle_with_empty_line_segs_creates_new_seg() {
        let mut core = make_test_core();
        let pic_h = 7000u32;
        let ctrl_idx = {
            let para = &mut core.document.sections[0].paragraphs[0];
            para.line_segs.clear(); // 빈 line_segs 강제
            push_body_floating_picture(para, 7000, pic_h, 1000, 1000, 1)
        };
        core.set_picture_properties_native(0, 0, ctrl_idx, r#"{"treatAsChar":true}"#)
            .expect("migration");

        let para = &core.document.sections[0].paragraphs[0];
        assert!(
            !para.line_segs.is_empty(),
            "빈 line_segs 였다면 신설되어야 한다"
        );
        let seg = &para.line_segs[0];
        assert_eq!(seg.line_height, pic_h as i32);
        assert_eq!(seg.text_height, pic_h as i32);
        assert_eq!(seg.baseline_distance, expected_baseline(pic_h as i32));
    }

    // LineSeg 빈 케이스 직접 검증용 (별도 helper 미사용 check)
    #[test]
    #[allow(dead_code)]
    fn _lineseg_default_for_test() {
        let seg = LineSeg::default();
        assert_eq!(seg.line_height, 0);
    }

    // ═══════════════════════════════════════════════════════════════════
    //  통합 검증 (Stage 2): 한컴 산출물 정합
    //
    //  samples/tac-verify/scenario-{a,b,c,d}-before.hwp 를 rhwp 가 파싱한 후
    //  set_picture_properties_native 로 tac false→true 토글한 결과가
    //  scenario-{a,b,c,d}-after.hwp 의 model 과 dump 동치인지 검증한다.
    //  v2 fix 가 만든 model 이 한컴이 만든 model 과 양방향 정합임을 보장.
    // ═══════════════════════════════════════════════════════════════════

    /// 양방향 정합 검증의 공통 단언 — paragraph 0.0 의 picture / line_segs 비교.
    fn assert_toggle_matches_hancom(scenario: &str) {
        let before_bytes =
            std::fs::read(format!("samples/tac-verify/scenario-{scenario}-before.hwp"))
                .expect("read before.hwp");
        let after_bytes =
            std::fs::read(format!("samples/tac-verify/scenario-{scenario}-after.hwp"))
                .expect("read after.hwp");

        let before_doc = crate::parser::parse_hwp(&before_bytes).expect("parse before");
        let after_doc = crate::parser::parse_hwp(&after_bytes).expect("parse after");

        let mut core = DocumentCore::new_empty();
        core.set_document(before_doc);

        // picture 위치 찾기 (paragraph 0.0 의 첫 Picture control)
        let pic_ctrl_idx = core.document.sections[0].paragraphs[0]
            .controls
            .iter()
            .position(|c| matches!(c, Control::Picture(_)))
            .unwrap_or_else(|| panic!("scenario-{scenario}-before: no Picture control"));

        core.set_picture_properties_native(0, 0, pic_ctrl_idx, r#"{"treatAsChar":true}"#)
            .expect("toggle");

        // 토글된 picture
        let toggled_para = &core.document.sections[0].paragraphs[0];
        let toggled_pic = match &toggled_para.controls[pic_ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!("not Picture after toggle"),
        };

        // 한컴 after 의 picture
        let after_para = &after_doc.sections[0].paragraphs[0];
        let after_pic_ctrl_idx = after_para
            .controls
            .iter()
            .position(|c| matches!(c, Control::Picture(_)))
            .unwrap_or_else(|| panic!("scenario-{scenario}-after: no Picture control"));
        let after_pic = match &after_para.controls[after_pic_ctrl_idx] {
            Control::Picture(p) => p.as_ref(),
            _ => panic!(),
        };

        // (a) picture 4 필드 비교
        assert_eq!(
            toggled_pic.common.treat_as_char, after_pic.common.treat_as_char,
            "scenario-{scenario}: treat_as_char mismatch"
        );
        assert_eq!(
            toggled_pic.common.horizontal_offset, after_pic.common.horizontal_offset,
            "scenario-{scenario}: horizontal_offset mismatch"
        );
        assert_eq!(
            toggled_pic.common.vertical_offset, after_pic.common.vertical_offset,
            "scenario-{scenario}: vertical_offset mismatch"
        );
        assert_eq!(
            toggled_pic.common.horz_rel_to as u8, after_pic.common.horz_rel_to as u8,
            "scenario-{scenario}: horz_rel_to mismatch"
        );
        assert_eq!(
            toggled_pic.common.vert_rel_to as u8, after_pic.common.vert_rel_to as u8,
            "scenario-{scenario}: vert_rel_to mismatch"
        );

        // (b) line_segs[0] 비교
        let toggled_seg = &toggled_para.line_segs[0];
        let after_seg = &after_para.line_segs[0];
        assert_eq!(
            toggled_seg.line_height, after_seg.line_height,
            "scenario-{scenario}: line_height mismatch"
        );
        assert_eq!(
            toggled_seg.text_height, after_seg.text_height,
            "scenario-{scenario}: text_height mismatch"
        );
        assert_eq!(
            toggled_seg.baseline_distance, after_seg.baseline_distance,
            "scenario-{scenario}: baseline_distance mismatch (round(lh*0.85) 정합)"
        );

        // (c) paragraph 수 / picture 위치 불변
        assert_eq!(
            core.document.sections[0].paragraphs.len(),
            after_doc.sections[0].paragraphs.len(),
            "scenario-{scenario}: paragraph count mismatch"
        );
        assert_eq!(
            pic_ctrl_idx, after_pic_ctrl_idx,
            "scenario-{scenario}: picture control_idx mismatch"
        );

        // (d) paragraph.text 불변
        assert_eq!(
            toggled_para.text, after_para.text,
            "scenario-{scenario}: paragraph.text mismatch"
        );
    }

    #[test]
    fn integration_tac_toggle_matches_hancom_scenario_a() {
        assert_toggle_matches_hancom("a");
    }

    #[test]
    fn integration_tac_toggle_matches_hancom_scenario_b() {
        assert_toggle_matches_hancom("b");
    }

    #[test]
    fn integration_tac_toggle_matches_hancom_scenario_c() {
        assert_toggle_matches_hancom("c");
    }

    #[test]
    fn integration_tac_toggle_matches_hancom_scenario_d() {
        assert_toggle_matches_hancom("d");
    }
}

#[cfg(test)]
mod issue_1280_textbox_creation_tests {
    //! Issue #1280: rhwp-studio가 삽입한 글상자가 text_box 없는 Rectangle로 생성되어
    //! 커서 진입·타이핑·붙여넣기가 모두 실패하던 결함.
    //!
    //! 근본 결함은 프런트(`input-handler.ts`)가 `shapeType: 'rectangle'`을 전달한 것이고,
    //! 백엔드 `create_shape_control_native`는 `shape_type == "textbox"`일 때 text_box(내부 문단)를
    //! 정상 구성한다. 본 테스트는 그 백엔드 계약(글상자=text_box 있음, 사각형=없음)을 고정하여
    //! 프런트 수정과 함께 회귀를 막는다.

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
        core.set_document(doc);
        core
    }

    fn parse_idx(res: &str, key: &str) -> usize {
        res.split(&format!("\"{}\":", key))
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("missing {key} in {res}"))
    }

    fn minimal_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00,
            0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    /// 도형 생성 후 (para_idx, ctrl_idx) 반환. 글상자는 한컴 기본값과 동일하게 treat_as_char=true.
    fn create_shape(core: &mut DocumentCore, shape_type: &str) -> (usize, usize) {
        let treat_as_char = shape_type == "textbox";
        // 인자: section_idx, para_idx, char_offset, width, height, horz_offset, vert_offset,
        // treat_as_char, text_wrap_str, shape_type, line_flip_x, line_flip_y, polygon_points
        let res = core
            .create_shape_control_native(
                0,
                0,
                0,
                21600,
                7200,
                0,
                0,
                treat_as_char,
                "TopAndBottom",
                shape_type,
                false,
                false,
                &[],
            )
            .unwrap_or_else(|e| panic!("create {shape_type} failed: {e:?}"));
        (parse_idx(&res, "paraIdx"), parse_idx(&res, "controlIdx"))
    }

    fn textbox_of<'a>(
        core: &'a DocumentCore,
        para_idx: usize,
        ctrl_idx: usize,
    ) -> Option<&'a crate::model::shape::TextBox> {
        match &core.document.sections[0].paragraphs[para_idx].controls[ctrl_idx] {
            Control::Shape(s) => crate::document_core::helpers::get_textbox_from_shape(s.as_ref()),
            other => panic!("expected Control::Shape, got {other:?}"),
        }
    }

    fn common_of<'a>(
        core: &'a DocumentCore,
        para_idx: usize,
        ctrl_idx: usize,
    ) -> &'a crate::model::shape::CommonObjAttr {
        match &core.document.sections[0].paragraphs[para_idx].controls[ctrl_idx] {
            Control::Shape(s) => s.common(),
            other => panic!("expected Control::Shape, got {other:?}"),
        }
    }

    /// 글상자를 직접 인자로 생성(treat_as_char/text_wrap 명시). (para_idx, ctrl_idx) 반환.
    fn create_textbox_with(
        core: &mut DocumentCore,
        treat_as_char: bool,
        text_wrap: &str,
    ) -> (usize, usize) {
        let res = core
            .create_shape_control_native(
                0,
                0,
                0,
                21600,
                7200,
                1000,
                2000,
                treat_as_char,
                text_wrap,
                "textbox",
                false,
                false,
                &[],
            )
            .unwrap_or_else(|e| panic!("create textbox failed: {e:?}"));
        (parse_idx(&res, "paraIdx"), parse_idx(&res, "controlIdx"))
    }

    /// [Task #1280 v2] 삽입 글상자를 floating(treat_as_char=false)+InFrontOfText 로 만들면
    /// 한컴 정답값(Paper/Paper/글앞으로)으로 생성되고 text_box 는 그대로 유지된다.
    /// 권위 샘플 samples/textbox-under-image.hwp 실측 정합.
    #[test]
    fn create_floating_textbox_is_in_front_paper() {
        use crate::model::shape::{HorzRelTo, TextWrap, VertRelTo};
        let mut core = make_test_core();
        let (para, ctrl) = create_textbox_with(&mut core, false, "InFrontOfText");

        // text_box 유지 (글상자 기능 보존 — floating 에서도)
        assert!(
            textbox_of(&core, para, ctrl).is_some(),
            "floating 글상자도 text_box 를 가져야 한다"
        );

        let c = common_of(&core, para, ctrl);
        assert!(!c.treat_as_char, "floating: treat_as_char=false");
        assert_eq!(
            c.text_wrap,
            TextWrap::InFrontOfText,
            "글앞으로(InFrontOfText)"
        );
        assert_eq!(c.vert_rel_to, VertRelTo::Paper, "vert_rel_to=Paper");
        assert_eq!(c.horz_rel_to, HorzRelTo::Paper, "horz_rel_to=Paper");
        // 직렬화 attr 비트 정합 (serializer 는 common.attr!=0 이면 그대로 사용).
        assert_eq!(c.attr & 0x01, 0, "attr bit0(treat_as_char)=0");
        assert_eq!((c.attr >> 3) & 0x03, 0, "attr bit3-4(vert_rel_to)=Paper(0)");
        assert_eq!((c.attr >> 8) & 0x03, 0, "attr bit8-9(horz_rel_to)=Paper(0)");
        assert_eq!(
            (c.attr >> 21) & 0x07,
            3,
            "attr bit21-23(text_wrap)=InFrontOfText(3)"
        );
    }

    /// inline 글상자(treat_as_char=true)는 #1280 본편 배치(Para/Column)를 그대로 보존한다(회귀 가드).
    #[test]
    fn create_inline_textbox_preserves_para_column() {
        use crate::model::shape::{HorzRelTo, VertRelTo};
        let mut core = make_test_core();
        let (para, ctrl) = create_textbox_with(&mut core, true, "Square");
        let c = common_of(&core, para, ctrl);
        assert!(c.treat_as_char, "inline: treat_as_char=true");
        assert_eq!(c.vert_rel_to, VertRelTo::Para, "inline vert_rel_to=Para");
        assert_eq!(
            c.horz_rel_to,
            HorzRelTo::Column,
            "inline horz_rel_to=Column"
        );
    }

    /// floating 글상자에도 텍스트 입력이 정상 동작(#1280 본편 회귀 없음).
    #[test]
    fn insert_text_into_floating_textbox() {
        let mut core = make_test_core();
        let (para, ctrl) = create_textbox_with(&mut core, false, "InFrontOfText");
        core.insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, "플로팅")
            .expect("floating 글상자 텍스트 입력 성공");
        let tb = textbox_of(&core, para, ctrl).expect("text_box 존재");
        assert_eq!(
            tb.paragraphs[0].text, "플로팅",
            "floating 글상자 내부 텍스트 보존"
        );
    }

    /// 글상자 안에서 이미지 배치 영역을 드래그한 경우, 그림은 body sibling 이 아니라
    /// text_box 내부 paragraph 의 Picture control 로 들어가야 한다.
    #[test]
    fn insert_picture_into_textbox_uses_textbox_paragraph_control() {
        use crate::model::shape::{HorzRelTo, TextWrap, VertRelTo};

        let mut core = make_test_core();
        let (para, ctrl) = create_textbox_with(&mut core, false, "InFrontOfText");
        let body_control_count_before = core.document.sections[0].paragraphs[para].controls.len();
        let cell_path = vec![(ctrl, 0, 0)];
        let image = minimal_png();

        core.insert_picture_native(
            0,
            para,
            0,
            &cell_path,
            &image,
            5000,
            4000,
            1,
            1,
            "png",
            "textbox picture",
            Some(750),
            Some(1500),
        )
        .expect("글상자 내부 picture 삽입 성공");

        let body = &core.document.sections[0].paragraphs[para];
        assert_eq!(
            body.controls.len(),
            body_control_count_before,
            "글상자 내부 삽입은 body sibling control 을 추가하면 안 된다"
        );

        let tb = textbox_of(&core, para, ctrl).expect("글상자 text_box 존재");
        let picture = tb.paragraphs[0]
            .controls
            .iter()
            .find_map(|c| match c {
                Control::Picture(p) => Some(p.as_ref()),
                _ => None,
            })
            .expect("글상자 내부 문단에 Picture control 이 있어야 한다");

        assert!(!picture.common.treat_as_char);
        assert_eq!(picture.common.horz_rel_to, HorzRelTo::Para);
        assert_eq!(picture.common.vert_rel_to, VertRelTo::Para);
        assert_eq!(picture.common.text_wrap, TextWrap::Square);
        assert_eq!(picture.common.horizontal_offset, 750);
        assert_eq!(picture.common.vertical_offset, 1500);
        assert_eq!(picture.common.width, 5000);
        assert_eq!(picture.common.height, 4000);
    }

    #[test]
    fn create_textbox_has_textbox() {
        let mut core = make_test_core();
        let (para, ctrl) = create_shape(&mut core, "textbox");
        assert!(
            textbox_of(&core, para, ctrl).is_some(),
            "글상자(shape_type=textbox)는 text_box를 가져야 한다 (#1280)"
        );
    }

    #[test]
    fn create_rectangle_has_no_textbox() {
        let mut core = make_test_core();
        let (para, ctrl) = create_shape(&mut core, "rectangle");
        assert!(
            textbox_of(&core, para, ctrl).is_none(),
            "일반 사각형(shape_type=rectangle)은 text_box가 없어야 한다 (글상자/사각형 경로 분리)"
        );
    }

    #[test]
    fn insert_text_into_created_textbox() {
        let mut core = make_test_core();
        let (para, ctrl) = create_shape(&mut core, "textbox");

        // 글상자 내부(cell_idx=0 무시, cell_para_idx=0, char_offset=0)에 텍스트 삽입.
        // 수정 전 프런트 경로에서는 text_box가 없어 "지정된 Shape 컨트롤에 텍스트 박스가 없습니다"로 실패했다.
        core.insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, "테스트")
            .expect("글상자에 텍스트 입력이 성공해야 한다 (#1280)");

        let tb = textbox_of(&core, para, ctrl).expect("글상자 text_box 존재");
        assert_eq!(
            tb.paragraphs[0].text, "테스트",
            "글상자 내부 첫 문단에 입력 텍스트가 보존되어야 한다"
        );
    }

    /// #1280 이슈가 기대 동작에 명시한 "글상자 안 붙여넣기"를 실측한다.
    /// 본문 텍스트를 copy_selection 으로 복사한 뒤 글상자 안에 paste_internal_in_cell 로 붙여넣는다.
    /// 수정 전(text_box 없는 Rectangle)이면 이 경로가 "글상자 없음"(clipboard.rs:512)으로 실패한다.
    ///
    /// 이미지/컨트롤 붙여넣기는 merge_from 이 controls 를 병합하지 않아 조용히 누락되던
    /// 별개 결함(#1323)이 있었으며, merge_from 보강으로 해소되었다.
    /// 회귀 테스트는 `paste_picture_into_textbox` 참고.
    #[test]
    fn paste_text_into_textbox() {
        let mut core = make_test_core();

        // 1. 본문에 텍스트 입력 후 선택 영역 복사 → 내부 클립보드에 텍스트 적재(controls 없음)
        core.insert_text_native(0, 0, 0, "복사원본")
            .expect("본문 텍스트 입력");
        core.copy_selection_native(0, 0, 0, 0, 4)
            .expect("본문 텍스트 복사");

        // 2. 글상자 생성
        let (tb_para, tb_ctrl) = create_shape(&mut core, "textbox");

        // 3. 글상자 안에 붙여넣기 (cell_idx=0, cell_para_idx=0, char_offset=0)
        core.paste_internal_in_cell_native(0, tb_para, tb_ctrl, 0, 0, 0)
            .expect("글상자에 붙여넣기가 성공해야 한다 (#1280; 수정 전엔 \"글상자 없음\")");

        // 4. 글상자 내부 첫 문단에 붙여넣은 텍스트가 들어갔는지 확인
        let tb = textbox_of(&core, tb_para, tb_ctrl).expect("글상자 text_box 존재");
        assert!(
            tb.paragraphs.iter().any(|p| p.text.contains("복사원본")),
            "붙여넣기 후 글상자 내부 문단에 복사한 텍스트가 있어야 한다"
        );
    }

    /// #1323: 글상자 안 이미지(그림 컨트롤) 붙여넣기 회귀 테스트.
    /// 본문 그림을 copy_control 로 복사한 뒤 글상자 안에 paste_internal_in_cell 로
    /// 붙여넣는다. merge_from 이 controls 를 병합하지 않던 수정 전에는 그림이
    /// 에러 없이 조용히 누락되었다.
    #[test]
    fn paste_picture_into_textbox() {
        let mut core = make_test_core();

        // 1. 본문에 그림 삽입 (BinData 등록 포함)
        let res = core
            .insert_picture_native(
                0,
                0,
                0,
                &[],
                &minimal_png(),
                5000,
                5000,
                1,
                1,
                "png",
                "",
                None,
                None,
            )
            .expect("본문 그림 삽입");
        let pic_para = parse_idx(&res, "paraIdx");
        let pic_ctrl = parse_idx(&res, "controlIdx");

        // 2. 그림 복사 → 내부 클립보드
        core.copy_control_native(0, pic_para, &[], pic_ctrl)
            .expect("그림 복사");

        // 3. 글상자 생성 + 안에 붙여넣기 (cell_idx=0 무시, cell_para_idx=0, char_offset=0)
        let (tb_para, tb_ctrl) = create_shape(&mut core, "textbox");
        core.paste_internal_in_cell_native(0, tb_para, tb_ctrl, 0, 0, 0)
            .expect("글상자에 그림 붙여넣기");

        // 4. 글상자 내부 문단에 그림 컨트롤 보존 확인
        let tb = textbox_of(&core, tb_para, tb_ctrl).expect("글상자 text_box 존재");
        let pic_count: usize = tb
            .paragraphs
            .iter()
            .map(|p| {
                p.controls
                    .iter()
                    .filter(|c| matches!(c, Control::Picture(_)))
                    .count()
            })
            .sum();
        assert_eq!(
            pic_count, 1,
            "글상자 안에 붙여넣은 그림 컨트롤이 보존되어야 한다 (#1323)"
        );
    }
}
