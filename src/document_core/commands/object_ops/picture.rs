//! 그림 삽입/속성/crop native 명령 (object_ops 분할, #1904).

use super::MIN_SHAPE_SIZE;
use crate::document_core::helpers::{get_textbox_from_shape, get_textbox_from_shape_mut};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::model::shape::{common_obj_offsets, ShapeObject};

impl DocumentCore {
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
    pub(crate) fn picture_rotated_bounds(width: u32, height: u32, angle: i16) -> (u32, u32) {
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
    pub(crate) fn format_picture_properties_json(
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
        let desc_escaped = crate::document_core::helpers::json_escape(&c.description);
        // [Task #741 후속] 외부 file path (HWP3 외부 그림) 영역 영역 dialog 표시 영역
        let external_path_field = match &pic.image_attr.external_path {
            Some(p) => format!(
                ",\"externalPath\":\"{}\"",
                crate::document_core::helpers::json_escape(p)
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
        let (
            caption_created,
            caption_removed,
            should_migrate_to_inline,
            should_migrate_to_floating,
        ) = {
            let pic =
                self.resolve_picture_control_mut(section_idx, parent_para_idx, control_idx)?;
            // [Task #1151 v2] tac false→true migration 검출용 snapshot.
            let was_tac = pic.common.treat_as_char;
            let had_caption = pic.caption.is_some();
            let caption_created = Self::apply_picture_props_inner(pic, props_json);
            let now_tac = pic.common.treat_as_char;
            (
                caption_created,
                had_caption && pic.caption.is_none(),
                !was_tac && now_tac,
                was_tac && !now_tac,
            )
        };

        // [Task #1151 v2] floating → inline migration (H1 정합, samples/tac-verify/).
        // 한컴 산출물 Scenario A~D 분석: tac false→true 시 picture 의 control 위치는
        // 불변이고, 4 필드만 갱신 (treat_as_char / h/v_rel_to=Para / h/v_offset=0 /
        // parent line_segs[0]). text/char_offsets/paragraph 수 변화 없음.
        let mut reflow_text_para_after_floating = false;
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
            } else if para.text.is_empty() && para.char_offsets.is_empty() {
                Self::migrate_empty_picture_para_inline_to_floating(para);
            } else if !para.text.is_empty() && parent_para_idx < body_len {
                // 텍스트가 있는 문단: false→true 마이그레이션이 line_segs[0] 를
                // 그림 높이로 키워 놓았으므로, 그림이 inline 흐름에서 빠지는 시점에
                // 남은 인라인 콘텐츠 기준으로 재계산해야 한다. 그림 삭제 경로
                // (reflow_paragraph_line_segs_after_control_delete) 와 동일한 원리.
                // paragraph &mut borrow 가 끝난 뒤 reflow_paragraph 로 수행.
                // 텍스트 없이 컨트롤 문자만 있는 문단(표 문단 등)은 기존 동작 유지.
                reflow_text_para_after_floating = true;
            }
        }
        if reflow_text_para_after_floating {
            self.reflow_paragraph(section_idx, parent_para_idx);
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                parent_para_idx,
            );
        }
        // 캡션 생성/삭제 시 AutoNumber 재할당. 생성 path 는 문단 placeholder 도 보강한다.
        if caption_created || caption_removed {
            crate::parser::assign_auto_numbers(&mut self.document);
        }
        if caption_created {
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
    pub(crate) fn take_place_picture_flow_offset(
        pic: &crate::model::image::Picture,
    ) -> Option<i32> {
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
    /// [Task #825] Picture 속성 JSON 적용 (mutation only). 후처리 (AutoNumber /
    /// recompose / paginate / event log) 는 호출자 책임.
    /// 반환: caption_created (true 면 호출자가 AutoNumber 후처리 필요).
    pub(crate) fn apply_picture_props_inner(
        pic: &mut crate::model::image::Picture,
        props_json: &str,
    ) -> bool {
        use crate::document_core::helpers::{json_bool, json_i16, json_i32, json_str, json_u32};

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
                pic.caption = None;
                pic.common.attr &= !(1 << 29);
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
        // bin_data_id(위치, 1-based 순번)와 storage id(BIN%04X 스트림 이름)는
        // 다른 의미다. 위치는 배열 순번으로 유지하고, storage id 는 기존 id 와
        // 충돌하지 않게 최댓값+1 로 채번한다 — 순번 채번은 storage id 에 구멍이
        // 있는 문서에서 기존 이미지와 스트림 이름이 충돌해 저장 시 이미지가
        // 뒤바뀌거나 소실된다.
        let position_id = self.document.bin_data_content.len() as u16 + 1;
        let storage_id = self.document.next_bin_data_storage_id();
        self.document.bin_data_content.push(BinDataContent {
            id: storage_id,
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
            storage_id,
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
            bin_data_id: position_id,
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
                        crate::document_core::helpers::find_logical_control_positions(target_para);
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
                return Ok(crate::document_core::helpers::json_ok_with(&format!(
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
            let logical_positions =
                crate::document_core::helpers::find_logical_control_positions(parent);
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
            return Ok(crate::document_core::helpers::json_ok_with(&format!(
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
        let logical_positions =
            crate::document_core::helpers::find_logical_control_positions(parent);
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
        Ok(crate::document_core::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{},\"logicalOffset\":{}",
            para_idx, new_ctrl_idx, logical_after
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

#[cfg(test)]
mod bindata_storage_id_collision_tests {
    //! Issue #2038: BinData storage id 채번 충돌 — 순번(len+1) 채번은 storage id 에 구멍이
    //! 있는 문서에서 기존 id 와 충돌해, 저장 시 같은 이름(BIN%04X)의 스트림이
    //! 겹쳐 이미지가 뒤바뀌거나 소실된다. storage id 는 최댓값+1 로 채번하되
    //! 위치 의미인 `ImageAttr.bin_data_id` 는 순번을 유지해야 한다.

    use super::*;
    use crate::model::bin_data::{
        BinData, BinDataCompression, BinDataContent, BinDataStatus, BinDataType,
    };
    use std::collections::HashSet;

    const EXISTING_IMAGE: &[u8] = &[0xAA, 0xBB, 0xCC];

    fn minimal_png() -> Vec<u8> {
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00,
            0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    /// 실제 문서를 로드한 뒤 storage id 2 만 존재하는 구멍(1 없음)을 주입한다.
    /// 구채번(len+1 = 2)이라면 기존 id 2 와 충돌하는 구성.
    fn make_core_with_storage_hole() -> DocumentCore {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/english.hwp");
        let bytes = std::fs::read(&path).expect("read english.hwp");
        let mut core = DocumentCore::from_bytes(&bytes).expect("load english.hwp");
        assert!(
            core.document.bin_data_content.is_empty(),
            "전제: english.hwp 에는 BinData 가 없어야 함"
        );
        core.document.bin_data_content.push(BinDataContent {
            id: 2,
            data: EXISTING_IMAGE.to_vec(),
            extension: "png".to_string(),
        });
        core.document.doc_info.bin_data_list.push(BinData {
            raw_data: None,
            attr: 0x0101,
            data_type: BinDataType::Embedding,
            compression: BinDataCompression::Default,
            status: BinDataStatus::Success,
            abs_path: None,
            rel_path: None,
            storage_id: 2,
            extension: Some("png".to_string()),
        });
        core
    }

    #[test]
    fn insert_picture_storage_id_skips_existing_ids() {
        let mut core = make_core_with_storage_hole();
        core.insert_picture_native(
            0,
            0,
            0,
            &[],
            &minimal_png(),
            9000,
            6000,
            1,
            1,
            "png",
            "",
            None,
            None,
        )
        .expect("insert_picture_native");

        let content_ids: Vec<u16> = core
            .document
            .bin_data_content
            .iter()
            .map(|c| c.id)
            .collect();
        let unique: HashSet<u16> = content_ids.iter().copied().collect();
        assert_eq!(
            unique.len(),
            content_ids.len(),
            "BinDataContent.id 가 중복되면 저장 시 스트림 이름이 충돌한다: {content_ids:?}"
        );
        let storage_ids: Vec<u16> = core
            .document
            .doc_info
            .bin_data_list
            .iter()
            .map(|b| b.storage_id)
            .collect();
        let unique_storage: HashSet<u16> = storage_ids.iter().copied().collect();
        assert_eq!(
            unique_storage.len(),
            storage_ids.len(),
            "BinData.storage_id 중복: {storage_ids:?}"
        );

        // bin_data_id 는 위치(1-based 순번) 의미 유지 — 신규 그림은 2번째 content
        let new_bin_id = core.document.sections[0]
            .paragraphs
            .iter()
            .flat_map(|p| p.controls.iter())
            .find_map(|c| match c {
                Control::Picture(pic) => Some(pic.image_attr.bin_data_id),
                _ => None,
            })
            .expect("삽입된 그림 컨트롤");
        assert_eq!(
            new_bin_id, 2,
            "bin_data_id 는 content 배열 1-based 순번이어야 함"
        );
        let by_position = &core.document.bin_data_content[(new_bin_id - 1) as usize];
        assert_eq!(
            by_position.data,
            minimal_png(),
            "위치 기반 조회로 신규 그림 데이터가 나와야 함"
        );
        // 기존 이미지 데이터 불변
        assert_eq!(core.document.bin_data_content[0].data, EXISTING_IMAGE);
    }

    #[test]
    fn insert_into_storage_hole_doc_survives_hwp_save_roundtrip() {
        let mut core = make_core_with_storage_hole();
        core.insert_picture_native(
            0,
            0,
            0,
            &[],
            &minimal_png(),
            9000,
            6000,
            1,
            1,
            "png",
            "",
            None,
            None,
        )
        .expect("insert_picture_native");

        let saved = core.export_hwp_with_adapter().expect("export_hwp");
        let reloaded = DocumentCore::from_bytes(&saved).expect("재로드");
        let datas: Vec<&[u8]> = reloaded
            .document()
            .bin_data_content
            .iter()
            .map(|c| c.data.as_slice())
            .collect();
        assert!(
            datas.contains(&EXISTING_IMAGE),
            "저장 왕복 후 기존 이미지가 소실됨 (스트림 이름 충돌): {:?}",
            reloaded
                .document()
                .bin_data_content
                .iter()
                .map(|c| (c.id, c.data.len()))
                .collect::<Vec<_>>()
        );
        assert!(
            datas.iter().any(|d| *d == minimal_png().as_slice()),
            "저장 왕복 후 신규 이미지가 소실됨"
        );
    }
}
