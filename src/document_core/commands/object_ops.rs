//! 그림 속성/삽입/삭제 + 표 생성 + 셀 bbox 관련 native 메서드

use crate::model::control::Control;
use crate::model::shape::ShapeObject;
use crate::model::paragraph::Paragraph;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::event::DocumentEvent;
use super::super::helpers::get_textbox_from_shape;

/// 도형 최소 크기 (HWPUNIT).
/// 0으로 내려가면 Rectangle은 x_coords=[0,0,0,0]이 되고,
/// Group은 current/original 스케일이 0이 되어 자식이 전부 사라진다.
/// table_ops의 MIN_CELL_SIZE와 동일한 기준을 사용한다.
const MIN_SHAPE_SIZE: u32 = 200;

impl DocumentCore {
    pub fn get_picture_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
        let ctrl = para.controls.get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)))?;

        let pic = match ctrl {
            crate::model::control::Control::Picture(p) => p,
            _ => return Err(HwpError::RenderError("지정된 컨트롤이 그림이 아닙니다".to_string())),
        };

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

        let sa = &pic.shape_attr;

        Ok(format!(
            concat!(
                "{{\"width\":{},\"height\":{},\"treatAsChar\":{},",
                "\"vertRelTo\":\"{}\",\"vertAlign\":\"{}\",",
                "\"horzRelTo\":\"{}\",\"horzAlign\":\"{}\",",
                "\"vertOffset\":{},\"horzOffset\":{},",
                "\"textWrap\":\"{}\",",
                "\"brightness\":{},\"contrast\":{},\"effect\":\"{}\",",
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
                "\"captionWidth\":{},\"captionSpacing\":{},\"captionMaxWidth\":{},\"captionIncludeMargin\":{}}}"
            ),
            c.width, c.height, c.treat_as_char,
            vert_rel, vert_align,
            horz_rel, horz_align,
            c.vertical_offset, c.horizontal_offset,
            text_wrap,
            pic.image_attr.brightness, pic.image_attr.contrast, effect,
            desc_escaped,
            // 회전/대칭
            sa.rotation_angle, sa.horz_flip, sa.vert_flip,
            // 원본 크기
            sa.original_width, sa.original_height,
            // 자르기
            pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
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
        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
        let ctrl = para.controls.get_mut(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)))?;

        let pic = match ctrl {
            crate::model::control::Control::Picture(p) => p,
            _ => return Err(HwpError::RenderError("지정된 컨트롤이 그림이 아닙니다".to_string())),
        };

        use super::super::helpers::{json_u32, json_i32, json_i16, json_bool, json_str};

        // 크기 변경
        if let Some(w) = json_u32(props_json, "width") { pic.common.width = w; pic.shape_attr.current_width = w; }
        if let Some(h) = json_u32(props_json, "height") { pic.common.height = h; pic.shape_attr.current_height = h; }

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
        if let Some(v) = json_u32(props_json, "vertOffset") { pic.common.vertical_offset = v; }
        if let Some(v) = json_u32(props_json, "horzOffset") { pic.common.horizontal_offset = v; }

        // 이미지 속성
        if let Some(v) = json_i32(props_json, "brightness") { pic.image_attr.brightness = v as i8; }
        if let Some(v) = json_i32(props_json, "contrast") { pic.image_attr.contrast = v as i8; }
        if let Some(v) = json_str(props_json, "effect") {
            pic.image_attr.effect = match v.as_str() {
                "GrayScale" => crate::model::image::ImageEffect::GrayScale,
                "BlackWhite" => crate::model::image::ImageEffect::BlackWhite,
                "Pattern8x8" => crate::model::image::ImageEffect::Pattern8x8,
                _ => crate::model::image::ImageEffect::RealPic,
            };
        }

        // 회전/대칭
        if let Some(v) = json_i16(props_json, "rotationAngle") { pic.shape_attr.rotation_angle = v; }
        if let Some(v) = json_bool(props_json, "horzFlip") {
            pic.shape_attr.horz_flip = v;
            if v { pic.shape_attr.flip |= 0x01; } else { pic.shape_attr.flip &= !0x01; }
        }
        if let Some(v) = json_bool(props_json, "vertFlip") {
            pic.shape_attr.vert_flip = v;
            if v { pic.shape_attr.flip |= 0x02; } else { pic.shape_attr.flip &= !0x02; }
        }

        // 자르기
        if let Some(v) = json_i32(props_json, "cropLeft") { pic.crop.left = v; }
        if let Some(v) = json_i32(props_json, "cropTop") { pic.crop.top = v; }
        if let Some(v) = json_i32(props_json, "cropRight") { pic.crop.right = v; }
        if let Some(v) = json_i32(props_json, "cropBottom") { pic.crop.bottom = v; }

        // 안쪽 여백 (그림 여백)
        if let Some(v) = json_i16(props_json, "paddingLeft") { pic.padding.left = v; }
        if let Some(v) = json_i16(props_json, "paddingTop") { pic.padding.top = v; }
        if let Some(v) = json_i16(props_json, "paddingRight") { pic.padding.right = v; }
        if let Some(v) = json_i16(props_json, "paddingBottom") { pic.padding.bottom = v; }

        // 바깥 여백
        if let Some(v) = json_i16(props_json, "outerMarginLeft") { pic.common.margin.left = v; }
        if let Some(v) = json_i16(props_json, "outerMarginTop") { pic.common.margin.top = v; }
        if let Some(v) = json_i16(props_json, "outerMarginRight") { pic.common.margin.right = v; }
        if let Some(v) = json_i16(props_json, "outerMarginBottom") { pic.common.margin.bottom = v; }

        // 테두리
        if let Some(v) = json_u32(props_json, "borderColor") { pic.border_color = v; }
        if let Some(v) = json_i32(props_json, "borderWidth") { pic.border_width = v; }

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
                    cap.paragraphs.push(crate::model::paragraph::Paragraph::default());
                    // 캡션 텍스트 최대 폭 = 개체 폭
                    cap.max_width = pic.common.width;
                    pic.caption = Some(cap);
                    caption_created = true;
                    // 번호 할당을 위해 컨트롤을 임시로 캡션에 추가
                    pic.caption.as_mut().unwrap().paragraphs[0].controls
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
                if let Some(v) = json_u32(props_json, "captionWidth") { cap.width = v; }
                if let Some(v) = json_i16(props_json, "captionSpacing") { cap.spacing = v; }
                if let Some(v) = json_bool(props_json, "captionIncludeMargin") { cap.include_margin = v; }
            } else {
                // 캡션 제거 — 현재는 None 처리하지 않음 (캡션에 텍스트가 있을 수 있으므로)
            }
        }

        // 캡션 생성 시 AutoNumber 재할당 + 텍스트 생성
        // 한컴 방식: "그림 " + [AutoNumber 제어문자 8 code units] + " "
        // AutoNumber가 번호를 렌더링하므로 텍스트에 번호를 넣지 않는다.
        if caption_created {
            crate::parser::assign_auto_numbers(&mut self.document);
            let pic_mut = match &mut self.document.sections[section_idx]
                .paragraphs[parent_para_idx].controls[control_idx] {
                crate::model::control::Control::Picture(p) => p,
                _ => unreachable!(),
            };
            let para = &mut pic_mut.caption.as_mut().unwrap().paragraphs[0];
            // "그림 " (3글자) + [AutoNumber 8 code units] + " " (1글자)
            para.text = "그림  ".to_string(); // 그림 + space + space (AutoNumber 사이)
            // char_offsets: 그(0) 림(1) space(2) space(11=3+8, AutoNumber 뒤)
            para.char_offsets = vec![0, 1, 2, 11];
            // char_count = 텍스트 4 code units + AutoNumber 8 + 끝마커 1 = 13
            para.char_count = 13;
        }

        // 리플로우
        let section = &mut self.document.sections[section_idx];
        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureResized { section: section_idx, para: parent_para_idx, ctrl: control_idx });
        if caption_created {
            let char_offset = match &self.document.sections[section_idx]
                .paragraphs[parent_para_idx].controls[control_idx] {
                crate::model::control::Control::Picture(p) => {
                    p.caption.as_ref().map_or(0, |c|
                        c.paragraphs.first().map_or(0, |p| p.text.chars().count()))
                }
                _ => 0,
            };
            Ok(format!("{{\"ok\":true,\"captionCharOffset\":{}}}", char_offset))
        } else {
            Ok("{\"ok\":true}".to_string())
        }
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
                "구역 인덱스 {} 범위 초과", section_idx
            )));
        }
        let section = &mut self.document.sections[section_idx];
        if parent_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "부모 문단 인덱스 {} 범위 초과", parent_para_idx
            )));
        }
        let para = &mut section.paragraphs[parent_para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!(
                "컨트롤 인덱스 {} 범위 초과", control_idx
            )));
        }
        // 그림 컨트롤인지 확인
        if !matches!(&para.controls[control_idx], crate::model::control::Control::Picture(_)) {
            return Err(HwpError::RenderError(
                "지정된 컨트롤이 그림이 아닙니다".to_string()
            ));
        }

        // 컨트롤이 차지하는 갭의 시작 위치를 찾아 char_offsets 조정
        let text_chars: Vec<char> = para.text.chars().collect();
        let mut ci = 0usize;
        let mut prev_end: u32 = 0;
        let mut gap_start: Option<u32> = None;
        'outer: for i in 0..text_chars.len() {
            let offset = if i < para.char_offsets.len() { para.char_offsets[i] } else { prev_end };
            while prev_end + 8 <= offset && ci < para.controls.len() {
                if ci == control_idx {
                    gap_start = Some(prev_end);
                    break 'outer;
                }
                ci += 1;
                prev_end += 8;
            }
            let char_size: u32 = if text_chars[i] == '\t' { 8 }
                else if text_chars[i].len_utf16() == 2 { 2 }
                else { 1 };
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

        self.event_log.push(DocumentEvent::PictureDeleted { section: section_idx, para: parent_para_idx, ctrl: control_idx });
        Ok("{\"ok\":true}".to_string())
    }

    /// 컨트롤 삭제 후 문단의 line_segs를 재계산한다.
    ///
    /// 그림/도형 삭제 시 문단의 line_segs에 컨트롤 높이가 그대로 남아,
    /// 레이아웃이 갱신되지 않는 문제를 방지한다.
    fn reflow_paragraph_line_segs_after_control_delete(
        para: &mut Paragraph,
        styles: &crate::renderer::style_resolver::ResolvedStyleSet,
        dpi: f64,
    ) {
        // 남은 컨트롤 중 가장 큰 높이 계산
        let max_remaining_ctrl_height = para.controls.iter().map(|ctrl| {
            match ctrl {
                Control::Picture(pic) => pic.common.height as i32,
                Control::Shape(shape) => shape.common().height as i32,
                _ => 0,
            }
        }).max().unwrap_or(0);

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
            let seg_width = para.line_segs.first()
                .map(|s| s.segment_width)
                .unwrap_or(0);
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
        use crate::model::table::{Table, Cell, TablePageBreak};
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::style::{BorderFill, BorderLine, BorderLineType, DiagonalLine, Fill};

        // 유효성 검사
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과", para_idx
            )));
        }
        if row_count == 0 || col_count == 0 || col_count > 256 {
            return Err(HwpError::RenderError(format!(
                "행/열 수 범위 오류 (행={}, 열={}, 열은 1~256)", row_count, col_count
            )));
        }

        // --- 1. 편집 영역 폭 계산 ---
        let pd = &self.document.sections[section_idx].section_def.page_def;
        let outer_margin_lr: i32 = 283 * 2; // outer_margin left + right (~2mm)
        let content_width = (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32 - outer_margin_lr).max(7200) as u32;

        // --- 2. 한컴 기본값 기반 셀 생성 (blank_h_saved.hwp 참조) ---
        let col_width = content_width / col_count as u32;
        // 한컴 기본: 셀 패딩 L=510 R=510 T=141 B=141
        let cell_pad = crate::model::Padding { left: 510, right: 510, top: 141, bottom: 141 };
        // 한컴 기본: 셀 높이 = top + bottom padding (빈 셀 최소 높이)
        let cell_height: u32 = (cell_pad.top + cell_pad.bottom) as u32;
        // 한컴 기본: 행 렌더링 높이 = padding_top + line_height(1000) + padding_bottom
        let rendered_row_height: u32 = cell_pad.top as u32 + 1000 + cell_pad.bottom as u32;
        let total_width = col_width * col_count as u32;
        let total_height = rendered_row_height * row_count as u32;

        // BorderFill: 실선 테두리가 있는 기존 항목 재사용, 없으면 새로 생성
        let cell_border_fill_id = {
            let existing = self.document.doc_info.border_fills.iter().position(|bf| {
                bf.borders.iter().all(|b| b.line_type == BorderLineType::Solid && b.width >= 1)
            });
            if let Some(idx) = existing {
                (idx + 1) as u16 // 1-based
            } else {
                // 실선 BorderFill이 없으면 새로 생성
                let solid_border = BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 };
                let new_bf = BorderFill {
                    raw_data: None,
                    attr: 0,
                    borders: [solid_border, solid_border, solid_border, solid_border],
                    diagonal: DiagonalLine { diagonal_type: 1, width: 0, color: 0 },
                    fill: Fill::default(),
                };
                self.document.doc_info.border_fills.push(new_bf);
                self.document.doc_info.raw_stream = None;
                self.document.doc_info.border_fills.len() as u16 // 1-based
            }
        };

        // 커서 위치 문단의 속성을 기본값으로 상속 (한컴 동작 일치)
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para.char_shapes.first()
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
                        tag: 0x00060000,
                        ..Default::default()
                    }];
                }
                // raw_list_extra: 빈 벡터 (cell.width 필드가 LIST_HEADER에 직렬화됨)
                cell.raw_list_extra = Vec::new();
                cells.push(cell);
            }
        }

        // --- 3. Table 구조체 조립 (한컴 기본 속성값) ---
        let row_sizes: Vec<i16> = (0..row_count)
            .map(|_| col_count as i16)
            .collect();

        // raw_ctrl_data: CommonObjAttr 바이너리 (파서 호환)
        // 바이트 레이아웃: flags(4) + v_offset(4) + h_offset(4) + width(4) + height(4)
        //                 + z_order(4) + margin_l(2) + margin_r(2) + margin_t(2) + margin_b(2)
        //                 + instance_id(4) = 36바이트 (+ 여유 2바이트 = 38)
        // vert=Para(2), horz=Para(3), wrap=TopAndBottom(1)
        // width_criterion=Absolute(4), height_criterion=Absolute(2)
        let flags: u32 = (2 << 3) | (3 << 8) | (4 << 15) | (2 << 18) | (1 << 21);
        let outer_margin: i16 = 283; // ~1mm
        let mut raw_ctrl_data = vec![0u8; 38];
        raw_ctrl_data[0..4].copy_from_slice(&flags.to_le_bytes());         // offset 0: flags
        // offset 4-7: vertical_offset = 0
        // offset 8-11: horizontal_offset = 0
        raw_ctrl_data[12..16].copy_from_slice(&total_width.to_le_bytes()); // offset 12: width
        raw_ctrl_data[16..20].copy_from_slice(&total_height.to_le_bytes());// offset 16: height
        // offset 20-23: z_order = 0
        raw_ctrl_data[24..26].copy_from_slice(&outer_margin.to_le_bytes());// offset 24: margin_left
        raw_ctrl_data[26..28].copy_from_slice(&outer_margin.to_le_bytes());// offset 26: margin_right
        raw_ctrl_data[28..30].copy_from_slice(&outer_margin.to_le_bytes());// offset 28: margin_top
        raw_ctrl_data[30..32].copy_from_slice(&outer_margin.to_le_bytes());// offset 30: margin_bottom
        // offset 32-35: instance_id (해시 기반, 비-0 필수)
        let instance_id: u32 = {
            let mut h: u32 = 0x7c150000;
            h = h.wrapping_add(row_count as u32 * 0x1000);
            h = h.wrapping_add(col_count as u32 * 0x100);
            h = h.wrapping_add(total_width);
            h = h.wrapping_add(total_height.wrapping_mul(0x1b));
            if h == 0 { h = 0x7c154b69; }
            h
        };
        raw_ctrl_data[32..36].copy_from_slice(&instance_id.to_le_bytes());

        let mut table = Table {
            attr: 0x082A2210, // 한컴 기본값 (blank_h_saved.hwp)
            row_count,
            col_count,
            cell_spacing: 0,
            padding: crate::model::Padding { left: 510, right: 510, top: 141, bottom: 141 },
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
                tag: 0x00060000,
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
            self.document.sections[section_idx].paragraphs.insert(para_idx, table_para);
            insert_para_idx = para_idx;
        } else {
            // 문단 중간이면 분할 후 삽입
            if char_offset > 0 && !para.text.is_empty() {
                let new_para = self.document.sections[section_idx].paragraphs[para_idx]
                    .split_at(char_offset);
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, new_para);
                // 표 문단은 분할된 뒤에 삽입
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, table_para);
                insert_para_idx = para_idx + 1;
            } else {
                // char_offset == 0이지만 컨트롤이 있는 경우 → 뒤에 삽입
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, table_para);
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
                tag: 0x00060000,
                ..Default::default()
            }],
            has_para_text: false,
            raw_header_extra: empty_raw_header_extra,
            ..Default::default()
        };
        self.document.sections[section_idx].paragraphs.insert(insert_para_idx + 1, empty_para);

        // --- 6. 스타일 갱신 + 리플로우 + 페이지네이션 ---
        // 새 BorderFill 추가 시 styles.border_styles 갱신이 필요하므로 rebuild_section 사용
        self.rebuild_section(section_idx);

        self.event_log.push(DocumentEvent::TableRowInserted { section: section_idx, para: insert_para_idx, ctrl: 0 });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"controlIdx\":0", insert_para_idx)))
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
    ) -> Result<String, HwpError> {
        use crate::model::table::{Table, Cell, TablePageBreak};
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::style::{BorderFill, BorderLine, BorderLineType, DiagonalLine, Fill};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과", section_idx)));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과", para_idx)));
        }
        if row_count == 0 || col_count == 0 || col_count > 256 {
            return Err(HwpError::RenderError(format!(
                "행/열 수 범위 오류 (행={}, 열={})", row_count, col_count)));
        }

        if !treat_as_char {
            return self.create_table_native(section_idx, para_idx, char_offset, row_count, col_count);
        }

        // ── 인라인 TAC 표 생성 ──

        let pd = &self.document.sections[section_idx].section_def.page_def;
        let outer_margin: i16 = 283;
        let outer_margin_lr = (outer_margin * 2) as i32;
        let content_width = (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32 - outer_margin_lr).max(7200) as u32;

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

        let cell_pad = crate::model::Padding { left: 510, right: 510, top: 141, bottom: 141 };
        let cell_height: u32 = (cell_pad.top + cell_pad.bottom) as u32;
        let rendered_row_height: u32 = cell_pad.top as u32 + 1000 + cell_pad.bottom as u32;
        let total_height = rendered_row_height * row_count as u32;

        // BorderFill
        let cell_border_fill_id = {
            let existing = self.document.doc_info.border_fills.iter().position(|bf| {
                bf.borders.iter().all(|b| b.line_type == BorderLineType::Solid && b.width >= 1)
            });
            if let Some(idx) = existing {
                (idx + 1) as u16
            } else {
                let solid_border = BorderLine { line_type: BorderLineType::Solid, width: 1, color: 0 };
                let new_bf = BorderFill {
                    raw_data: None, attr: 0,
                    borders: [solid_border, solid_border, solid_border, solid_border],
                    diagonal: DiagonalLine { diagonal_type: 1, width: 0, color: 0 },
                    fill: Fill::default(),
                };
                self.document.doc_info.border_fills.push(new_bf);
                self.document.doc_info.raw_stream = None;
                self.document.doc_info.border_fills.len() as u16
            }
        };

        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para.char_shapes.first()
            .map(|cs| cs.char_shape_id).unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 셀 생성
        let mut cells = Vec::with_capacity((row_count as usize) * (col_count as usize));
        for r in 0..row_count {
            for c in 0..col_count {
                let col_w = col_ws[c as usize];
                let mut cell = Cell::new_empty(c, r, col_w, cell_height, cell_border_fill_id);
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
                    cp.line_segs = vec![LineSeg {
                        text_start: 0, line_height: 1000, text_height: 1000,
                        baseline_distance: 850, line_spacing: 600,
                        segment_width: seg_w, tag: 0x00060000,
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
        raw_ctrl_data[0..4].copy_from_slice(&flags.to_le_bytes());
        raw_ctrl_data[12..16].copy_from_slice(&total_width.to_le_bytes());
        raw_ctrl_data[16..20].copy_from_slice(&total_height.to_le_bytes());
        raw_ctrl_data[24..26].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[26..28].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[28..30].copy_from_slice(&outer_margin.to_le_bytes());
        raw_ctrl_data[30..32].copy_from_slice(&outer_margin.to_le_bytes());
        let instance_id: u32 = {
            let mut h: u32 = 0x7c160000;
            h = h.wrapping_add(row_count as u32 * 0x1000);
            h = h.wrapping_add(col_count as u32 * 0x100);
            h = h.wrapping_add(total_width);
            if h == 0 { h = 0x7c164b69; }
            h
        };
        raw_ctrl_data[32..36].copy_from_slice(&instance_id.to_le_bytes());

        let mut table = Table {
            attr: 0x04000006,
            row_count, col_count, cell_spacing: 0,
            padding: cell_pad,
            row_sizes,
            border_fill_id: cell_border_fill_id,
            zones: Vec::new(), cells, cell_grid: Vec::new(),
            page_break: TablePageBreak::RowBreak,
            repeat_header: false, caption: None,
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
            let last_char_len = para.text.chars().nth(last_idx)
                .map(|c| c.len_utf16() as u32).unwrap_or(1);
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
            section: section_idx, para: para_idx, ctrl: ctrl_idx,
        });
        // 표 바로 뒤의 논리적 오프셋 계산
        let logical_after = super::super::helpers::text_to_logical_offset(
            &self.document.sections[section_idx].paragraphs[para_idx], char_offset) + 1;
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"paraIdx\":{},\"controlIdx\":{},\"logicalOffset\":{}",
            para_idx, ctrl_idx, logical_after
        )))
    }

    /// 커서 위치에 그림을 삽입한다 (네이티브).
    pub fn insert_picture_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        image_data: &[u8],
        width: u32,
        height: u32,
        natural_width_px: u32,
        natural_height_px: u32,
        extension: &str,
        description: &str,
    ) -> Result<String, HwpError> {
        use crate::model::image::{Picture, ImageAttr, ImageEffect, CropInfo};
        use crate::model::shape::{CommonObjAttr, ShapeComponentAttr, VertRelTo, HorzRelTo};
        use crate::model::bin_data::{BinData, BinDataType, BinDataCompression, BinDataStatus, BinDataContent};
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        // 유효성 검사
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)", section_idx, self.document.sections.len()
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 인덱스 {} 범위 초과", para_idx
            )));
        }
        if image_data.is_empty() {
            return Err(HwpError::RenderError("이미지 데이터가 비어 있습니다".to_string()));
        }

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

        // --- 3. Picture 컨트롤 생성 ---
        // CommonObjAttr: treat_as_char, vert_rel_to=Para, horz_rel_to=Column,
        // width_criterion=absolute(4), height_criterion=absolute(2)
        let common_attr: u32 = 0x01 | (2 << 3) | (2 << 8) | (4 << 15) | (2 << 18); // 0x0A0211
        let common = CommonObjAttr {
            ctrl_id: 0x67736F20, // "gso " — GenShape
            attr: common_attr,
            treat_as_char: true,
            vert_rel_to: VertRelTo::Para,
            horz_rel_to: HorzRelTo::Column,
            width,
            height,
            z_order: 0,
            description: description.to_string(),
            ..Default::default()
        };

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

        // border_x/border_y: 4 꼭짓점 좌표 (x,y 쌍으로 연속 저장)
        // [tl.x, tl.y, tr.x, tr.y], [br.x, br.y, bl.x, bl.y]
        let bx = [0i32, 0, width as i32, 0];
        let by = [width as i32, height as i32, 0, height as i32];

        // crop: 비크롭 시 이미지 원본 범위 (원본 크기 = 디스플레이 크기일 때)
        // crop: 이미지 원본 픽셀 크기 × 75 (HWPUNIT/pixel at 96DPI)
        let crop = CropInfo {
            left: 0,
            top: 0,
            right: (natural_width_px * 75) as i32,
            bottom: (natural_height_px * 75) as i32,
        };

        let pic = Picture {
            common,
            shape_attr,
            border_x: bx,
            border_y: by,
            crop,
            image_attr: ImageAttr {
                bin_data_id: next_id,
                brightness: 0,
                contrast: 0,
                effect: ImageEffect::RealPic,
            },
            ..Default::default()
        };

        // --- 4. 그림 포함 문단 생성 + 삽입 (createTable 패턴) ---
        let current_para = &self.document.sections[section_idx].paragraphs[para_idx];
        let default_char_shape_id: u32 = current_para.char_shapes.first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        let pd = &self.document.sections[section_idx].section_def.page_def;
        let content_width = (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32).max(7200) as u32;

        let mut pic_raw_header_extra = vec![0u8; 10];
        pic_raw_header_extra[0..2].copy_from_slice(&1u16.to_le_bytes()); // n_char_shapes=1
        pic_raw_header_extra[4..6].copy_from_slice(&1u16.to_le_bytes()); // n_line_segs=1

        let pic_para = Paragraph {
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
                line_height: height as i32,
                text_height: height as i32,
                baseline_distance: (height as i32 * 850) / 1000,
                line_spacing: 600,
                segment_width: content_width as i32,
                tag: 0x00060000,
                ..Default::default()
            }],
            para_shape_id: default_para_shape_id,
            style_id: 0,
            controls: vec![Control::Picture(Box::new(pic))],
            ctrl_data_records: vec![None],
            has_para_text: true,
            raw_header_extra: pic_raw_header_extra,
            char_count_msb: false,
            ..Default::default()
        };

        // 커서 위치에 삽입
        self.document.sections[section_idx].raw_stream = None;

        let para = &self.document.sections[section_idx].paragraphs[para_idx];
        let is_empty_para = para.text.is_empty() && para.controls.is_empty();

        let insert_para_idx;
        if is_empty_para && char_offset == 0 {
            self.document.sections[section_idx].paragraphs[para_idx] = pic_para;
            insert_para_idx = para_idx;
        } else if char_offset == 0 && para.controls.is_empty() {
            self.document.sections[section_idx].paragraphs.insert(para_idx, pic_para);
            insert_para_idx = para_idx;
        } else {
            if char_offset > 0 && !para.text.is_empty() {
                let new_para = self.document.sections[section_idx].paragraphs[para_idx]
                    .split_at(char_offset);
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, new_para);
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, pic_para);
                insert_para_idx = para_idx + 1;
            } else {
                self.document.sections[section_idx].paragraphs.insert(para_idx + 1, pic_para);
                insert_para_idx = para_idx + 1;
            }
        }

        // 그림 아래에 빈 문단 추가
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
                segment_width: content_width as i32,
                tag: 0x00060000,
                ..Default::default()
            }],
            has_para_text: false,
            raw_header_extra: empty_raw_header_extra,
            ..Default::default()
        };
        self.document.sections[section_idx].paragraphs.insert(insert_para_idx + 1, empty_para);

        // --- 5. 리플로우 + 페이지네이션 ---
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureInserted { section: section_idx, para: insert_para_idx });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"controlIdx\":0", insert_para_idx)))
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
            sec: usize, ppi: usize, ci: usize,
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
            if find_table_cells(&tree.root, section_idx, parent_para_idx, control_idx, page_num, &mut cells) {
                found = true;
            } else if found {
                break;
            }
        }

        // page_hint에서 못 찾았으면 앞쪽 탐색 (페이지 분할 표가 hint 이전 페이지에서 시작될 수 있음)
        if !found && start > 0 {
            for page_num in (0..start).rev() {
                let tree = self.build_page_tree_cached(page_num as u32)?;
                if find_table_cells(&tree.root, section_idx, parent_para_idx, control_idx, page_num, &mut cells) {
                    found = true;
                    // 이 페이지에서 찾음 — hint까지 다시 정방향 탐색하여 누락된 페이지 수집
                    for fwd in (page_num + 1)..=start {
                        let tree2 = self.build_page_tree_cached(fwd as u32)?;
                        if !find_table_cells(&tree2.root, section_idx, parent_para_idx, control_idx, fwd, &mut cells) {
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
             \"textWrap\":\"{}\",\"zOrder\":{},\"instanceId\":{},\"description\":\"{}\"",
            c.width, c.height, c.treat_as_char,
            vert_rel, vert_align,
            horz_rel, horz_align,
            c.vertical_offset, c.horizontal_offset,
            text_wrap, c.z_order, c.instance_id, desc_escaped,
        )
    }

    /// JSON → CommonObjAttr 필드 업데이트 (Shape/Picture 공용)
    fn apply_common_obj_attr_from_json(c: &mut crate::model::shape::CommonObjAttr, props_json: &str) {
        use super::super::helpers::{json_u32, json_bool, json_str};

        if let Some(w) = json_u32(props_json, "width") { c.width = w.max(MIN_SHAPE_SIZE); }
        if let Some(h) = json_u32(props_json, "height") { c.height = h.max(MIN_SHAPE_SIZE); }
        if let Some(tac) = json_bool(props_json, "treatAsChar") {
            c.treat_as_char = tac;
            if tac { c.attr |= 0x01; } else { c.attr &= !0x01; }
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
        if let Some(v) = json_u32(props_json, "vertOffset") { c.vertical_offset = v; }
        if let Some(v) = json_u32(props_json, "horzOffset") { c.horizontal_offset = v; }
        if let Some(v) = json_str(props_json, "description") { c.description = v; }
    }

    /// 글상자(Shape) 속성 조회 (네이티브).
    pub fn get_shape_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
        let ctrl = para.controls.get(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)))?;

        let shape = match ctrl {
            Control::Shape(s) => s.as_ref(),
            _ => return Err(HwpError::RenderError("지정된 컨트롤이 Shape이 아닙니다".to_string())),
        };

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
            let line_type = bl.attr & 0x3F;                   // bits 0-5: 선 종류 (0~17)
            let line_end_shape = (bl.attr >> 6) & 0x0F;       // bits 6-9: 끝 모양
            let arrow_start = (bl.attr >> 10) & 0x3F;         // bits 10-15: 화살표 시작 모양
            let arrow_end = (bl.attr >> 16) & 0x3F;           // bits 16-21: 화살표 끝 모양
            let arrow_start_size = (bl.attr >> 22) & 0x0F;    // bits 22-25: 화살표 시작 크기
            let arrow_end_size = (bl.attr >> 26) & 0x0F;      // bits 26-29: 화살표 끝 크기

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
                let ctrl2_pts: Vec<&crate::model::shape::ConnectorControlPoint> =
                    conn.control_points.iter().filter(|cp| cp.point_type == 2).collect();
                if !ctrl2_pts.is_empty() {
                    let avg_x: i32 = ctrl2_pts.iter().map(|p| p.x).sum::<i32>() / ctrl2_pts.len() as i32;
                    let avg_y: i32 = ctrl2_pts.iter().map(|p| p.y).sum::<i32>() / ctrl2_pts.len() as i32;
                    format!(",\"connectorType\":{},\"connectorMidX\":{},\"connectorMidY\":{}",
                        conn.link_type as u32, avg_x, avg_y)
                } else {
                    format!(",\"connectorType\":{}", conn.link_type as u32)
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        Ok(format!("{{{}{}{}{}{}}}", common_json, tb_json, extra_json, round_json, connector_json))
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

        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?;
        let para = section.paragraphs.get_mut(parent_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
        let ctrl = para.controls.get_mut(control_idx)
            .ok_or_else(|| HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)))?;

        let shape = match ctrl {
            Control::Shape(s) => s.as_mut(),
            _ => return Err(HwpError::RenderError("지정된 컨트롤이 Shape이 아닙니다".to_string())),
        };

        // CommonObjAttr 업데이트
        // 리사이즈 핸들을 반대편으로 끌어당길 때 studio가 width/height=0 을 보내
        // 도형이 렌더러상 사라지는 버그 방어: 최소 크기 clamp.
        let c = shape.common_mut();
        let new_w = super::super::helpers::json_u32(props_json, "width").map(|w| w.max(MIN_SHAPE_SIZE));
        let new_h = super::super::helpers::json_u32(props_json, "height").map(|h| h.max(MIN_SHAPE_SIZE));
        Self::apply_common_obj_attr_from_json(c, props_json);

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
                if v { d.shape_attr.flip |= 1; } else { d.shape_attr.flip &= !1; }
            }
            if let Some(v) = json_bool(props_json, "vertFlip") {
                d.shape_attr.vert_flip = v;
                if v { d.shape_attr.flip |= 2; } else { d.shape_attr.flip &= !2; }
            }

            // 테두리 선 — 색상/굵기
            if let Some(v) = json_i32(props_json, "borderColor") { d.border_line.color = v as u32; }
            if let Some(v) = json_i32(props_json, "borderWidth") { d.border_line.width = v; }

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
                        pattern_type: -1,  // -1 = 단색 채우기 (0은 채우기 없음)
                        ..Default::default()
                    }
                });
                solid.background_color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "fillPatColor") {
                let solid = d.fill.solid.get_or_insert_with(|| {
                    crate::model::style::SolidFill {
                        pattern_type: -1,
                        ..Default::default()
                    }
                });
                solid.pattern_color = v as u32;
            }
            if let Some(v) = json_i32(props_json, "fillPatType") {
                let solid = d.fill.solid.get_or_insert_with(|| {
                    crate::model::style::SolidFill {
                        pattern_type: -1,
                        ..Default::default()
                    }
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
            if let Some(v) = super::super::helpers::json_u32(props_json, "shadowType") { d.shadow_type = v; }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowColor") { d.shadow_color = v as u32; }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowOffsetX") { d.shadow_offset_x = v; }
            if let Some(v) = super::super::helpers::json_i32(props_json, "shadowOffsetY") { d.shadow_offset_y = v; }

            // TextBox 속성 업데이트
            if let Some(ref mut tb) = d.text_box {
                if let Some(v) = json_i32(props_json, "tbMarginLeft") { tb.margin_left = v as i16; }
                if let Some(v) = json_i32(props_json, "tbMarginRight") { tb.margin_right = v as i16; }
                if let Some(v) = json_i32(props_json, "tbMarginTop") { tb.margin_top = v as i16; }
                if let Some(v) = json_i32(props_json, "tbMarginBottom") { tb.margin_bottom = v as i16; }
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

        self.event_log.push(DocumentEvent::PictureResized { section: section_idx, para: parent_para_idx, ctrl: control_idx });
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
            return Err(HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)));
        }
        let section = &mut self.document.sections[section_idx];
        if parent_para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)));
        }
        let para = &mut section.paragraphs[parent_para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)));
        }
        if !matches!(&para.controls[control_idx], Control::Shape(_)) {
            return Err(HwpError::RenderError("지정된 컨트롤이 Shape이 아닙니다".to_string()));
        }

        // char_offsets 조정 (delete_picture_control_native와 동일)
        let text_chars: Vec<char> = para.text.chars().collect();
        let mut ci = 0usize;
        let mut prev_end: u32 = 0;
        let mut gap_start: Option<u32> = None;
        'outer: for i in 0..text_chars.len() {
            let offset = if i < para.char_offsets.len() { para.char_offsets[i] } else { prev_end };
            while prev_end + 8 <= offset && ci < para.controls.len() {
                if ci == control_idx { gap_start = Some(prev_end); break 'outer; }
                ci += 1;
                prev_end += 8;
            }
            let char_size: u32 = if text_chars[i] == '\t' { 8 }
                else if text_chars[i].len_utf16() == 2 { 2 }
                else { 1 };
            prev_end = offset + char_size;
        }
        if gap_start.is_none() {
            while ci < para.controls.len() {
                if ci == control_idx { gap_start = Some(prev_end); break; }
                ci += 1;
                prev_end += 8;
            }
        }
        if let Some(gs) = gap_start {
            let threshold = gs + 8;
            for offset in para.char_offsets.iter_mut() {
                if *offset >= threshold { *offset -= 8; }
            }
        }

        para.controls.remove(control_idx);
        if control_idx < para.ctrl_data_records.len() {
            para.ctrl_data_records.remove(control_idx);
        }
        if para.char_count >= 8 { para.char_count -= 8; }

        // line_segs 재계산: 도형 높이가 반영된 line_segs를 텍스트 기반으로 리셋
        Self::reflow_paragraph_line_segs_after_control_delete(para, &self.styles, self.dpi);

        section.raw_stream = None;
        self.recompose_section(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::PictureDeleted { section: section_idx, para: parent_para_idx, ctrl: control_idx });
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
        use crate::model::shape::*;
        use crate::model::paragraph::{CharShapeRef, LineSeg};
        use crate::model::style::{Fill, ShapeBorderLine};

        // 유효성 검사
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)));
        }
        if width == 0 && height == 0 {
            return Err(HwpError::RenderError("폭과 높이가 모두 0입니다".to_string()));
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
        let default_char_shape_id: u32 = current_para.char_shapes.first()
            .map(|cs| cs.char_shape_id)
            .unwrap_or(0);
        let default_para_shape_id: u16 = current_para.para_shape_id;

        // 편집 영역 폭
        let pd = &self.document.sections[section_idx].section_def.page_def;
        let content_width = (pd.width as i32 - pd.margin_left as i32 - pd.margin_right as i32).max(7200) as u32;

        // attr 비트 계산
        // textbox: Para/Top/Column/Left/Square = 0x0A0210
        // 도형(line/ellipse/rectangle): 한컴 기본값 0x046A4000
        //   Paper/Top/Paper/Left/InFrontOfText + textSide=2 + bit16-17=2 + objNumSort=2 + bit26=1
        let mut attr: u32 = if shape_type == "textbox" { 0x0A0210 } else { 0x046A4000 };
        if treat_as_char { attr |= 0x01; }

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
                tag: 0x00060000,
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
            "line" | "connector-straight" | "connector-stroke" | "connector-arc"
            | "connector-straight-arrow" | "connector-stroke-arrow" | "connector-arc-arrow"
                => if is_connector { 0x24636f6c } else { 0x246c696e }, // '$col' or '$lin'
            "ellipse" => 0x24656c6c,  // '$ell'
            "polygon" => 0x24706f6c,  // '$pol'
            "arc" => 0x24617263,      // '$arc'
            _ => 0x24726563,          // '$rec' (rectangle, textbox)
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
            if h == 0 { h = 0x7de34b69; }
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
                crate::model::Padding { left: 283, right: 283, top: 283, bottom: 283 }
            } else {
                crate::model::Padding { left: 0, right: 0, top: 0, bottom: 0 }
            },
            treat_as_char,
            vert_rel_to: if shape_type == "textbox" { VertRelTo::Para } else { VertRelTo::Paper },
            vert_align: VertAlign::Top,
            horz_rel_to: if shape_type == "textbox" { HorzRelTo::Column } else { HorzRelTo::Paper },
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
            "line" | "connector-straight" | "connector-stroke" | "connector-arc"
            | "connector-straight-arrow" | "connector-stroke-arrow" | "connector-arc-arrow" => {
                // 드래그 방향에 따라 시작/끝점 결정
                let (sx, sy, ex, ey) = match (line_flip_x, line_flip_y) {
                    (false, false) => (0,   0,   w_i, h_i), // 좌상→우하
                    (false, true)  => (0,   h_i, w_i, 0),   // 좌하→우상
                    (true,  false) => (w_i, 0,   0,   h_i), // 우상→좌하
                    (true,  true)  => (w_i, h_i, 0,   0),   // 우하→좌상
                };
                let connector = if is_connector {
                    use crate::model::shape::{ConnectorData, ConnectorControlPoint, LinkLineType};
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
                        LinkLineType::StrokeNoArrow | LinkLineType::StrokeOneWay | LinkLineType::StrokeBoth
                        | LinkLineType::ArcNoArrow | LinkLineType::ArcOneWay | LinkLineType::ArcBoth => {
                            vec![
                                ConnectorControlPoint { x: sx, y: sy, point_type: 3 },  // 시작 앵커
                                ConnectorControlPoint { x: ex, y: sy, point_type: 2 },  // 중간 (직각 꺾임)
                                ConnectorControlPoint { x: ex, y: ey, point_type: 26 }, // 끝 앵커
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
                    started_right_or_bottom: if is_connector { false } else { line_flip_x || line_flip_y },
                    connector,
                })
            }
            "ellipse" => ShapeObject::Ellipse(EllipseShape {
                common,
                drawing,
                attr: 0,
                center: crate::model::Point { x: w_i / 2, y: h_i / 2 },
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
                })
            }
            "arc" => {
                // 사각형에 내접하는 타원의 1/4 호 (우상 사분면)
                // center: bbox 중심, axis1: 우측 중앙, axis2: 상단 중앙
                ShapeObject::Arc(ArcShape {
                    common,
                    drawing,
                    arc_type: 0, // 0=Arc
                    center: crate::model::Point { x: w_i / 2, y: h_i / 2 },
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

            // 컨트롤 추가
            paragraph.controls.insert(insert_idx, Control::Shape(Box::new(shape_obj)));
            paragraph.ctrl_data_records.insert(insert_idx, None);

            // char_offsets에 raw offset 삽입
            if !paragraph.char_offsets.is_empty() {
                let raw_offset = if insert_idx > 0 && insert_idx <= paragraph.char_offsets.len() {
                    paragraph.char_offsets[insert_idx - 1] + 8
                } else if !paragraph.char_offsets.is_empty() {
                    let first = paragraph.char_offsets[0];
                    if first >= 8 { first - 8 } else { 0 }
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

        self.event_log.push(DocumentEvent::PictureInserted { section: section_idx, para: insert_para_idx });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"controlIdx\":{}", insert_para_idx, insert_ctrl_idx)))
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
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?;

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

        let target_pos = shape_infos.iter()
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
            _ => return Err(HwpError::RenderError(format!("알 수 없는 operation: {}", operation))),
        };

        let (new_z, neighbor_change) = match changes {
            Some(c) => c,
            None => return Ok(super::super::helpers::json_ok_with(&format!("\"zOrder\":{}", current_z))),
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

        Ok(super::super::helpers::json_ok_with(&format!("\"zOrder\":{}", new_z)))
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

        let section = match self.document.sections.get_mut(section_idx) { Some(s) => s, None => return };
        let para = match section.paragraphs.get_mut(para_idx) { Some(p) => p, None => return };
        let ctrl = match para.controls.get_mut(control_idx) { Some(c) => c, None => return };

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
                ConnectorControlPoint { x: sx, y: sy, point_type: 3 },   // 시작 앵커
                ConnectorControlPoint { x: c1x, y: c1y, point_type: 2 }, // 베지어 ctrl1
                ConnectorControlPoint { x: c2x, y: c2y, point_type: 2 }, // 베지어 ctrl2
                ConnectorControlPoint { x: ex, y: ey, point_type: 26 },  // 끝 앵커
            ];
        } else {
            // ─── 꺽인 연결선: 직각 꺾임점 ───
            let mut pts = Vec::new();
            pts.push(ConnectorControlPoint { x: sx, y: sy, point_type: 3 });

            match (start_idx, end_idx) {
                (1, 3) | (3, 1) => {
                    let mid_x = (sx + ex) / 2;
                    pts.push(ConnectorControlPoint { x: mid_x, y: sy, point_type: 2 });
                    pts.push(ConnectorControlPoint { x: mid_x, y: ey, point_type: 2 });
                }
                (2, 0) | (0, 2) => {
                    let mid_y = (sy + ey) / 2;
                    pts.push(ConnectorControlPoint { x: sx, y: mid_y, point_type: 2 });
                    pts.push(ConnectorControlPoint { x: ex, y: mid_y, point_type: 2 });
                }
                (1, 0) | (1, 2) | (3, 0) | (3, 2) => {
                    pts.push(ConnectorControlPoint { x: ex, y: sy, point_type: 2 });
                }
                (0, 1) | (0, 3) | (2, 1) | (2, 3) => {
                    pts.push(ConnectorControlPoint { x: sx, y: ey, point_type: 2 });
                }
                _ => {
                    let mid_x = (sx + ex) / 2;
                    pts.push(ConnectorControlPoint { x: mid_x, y: sy, point_type: 2 });
                    pts.push(ConnectorControlPoint { x: mid_x, y: ey, point_type: 2 });
                }
            }

            pts.push(ConnectorControlPoint { x: ex, y: ey, point_type: 26 });
            conn.control_points = pts;
        }
    }

    /// 구역 내 모든 연결선을 스캔하여 연결된 도형의 현재 위치에 맞게 갱신한다.
    pub fn update_connectors_in_section(&mut self, section_idx: usize) {
        let section = match self.document.sections.get(section_idx) { Some(s) => s, None => return };

        // 1) SC inst_id → 연결점 좌표 맵 구축 (SubjectID = drawing.inst_id)
        let mut conn_points: std::collections::HashMap<u32, [(i32, i32); 4]> = std::collections::HashMap::new();
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                let (common, inst_id, _is_line) = match ctrl {
                    Control::Shape(s) => {
                        let sc_inst = s.drawing().map(|d| d.inst_id).unwrap_or(0);
                        (s.common(), sc_inst, matches!(s.as_ref(), ShapeObject::Line(_)))
                    }
                    Control::Picture(p) => (&p.common, 0u32, false),
                    _ => continue,
                };
                if _is_line { continue; }
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
        let section = match self.document.sections.get_mut(section_idx) { Some(s) => s, None => return };
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
                if start_pts.is_none() || end_pts.is_none() { continue; }

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
            let section = match self.document.sections.get(section_idx) { Some(s) => s, None => return };
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(ref s) = ctrl {
                        if let ShapeObject::Line(ref l) = s.as_ref() {
                            if let Some(ref c) = l.connector {
                                if c.link_type.is_stroke() || c.link_type.is_arc() {
                                    routing_targets.push((pi, ci, c.start_subject_index, c.end_subject_index));
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
        start_x: i32, start_y: i32,
        end_x: i32, end_y: i32,
    ) -> Result<String, HwpError> {
        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역 범위 초과".to_string()))?;
        let para = section.paragraphs.get_mut(para_idx)
            .ok_or_else(|| HwpError::RenderError("문단 범위 초과".to_string()))?;
        let ctrl = para.controls.get_mut(control_idx)
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
        fn sp(v: i32, s: f64) -> i32 { (v as f64 * s).round() as i32 }
        match child {
            SO::Line(ref mut s) => {
                s.start.x = sp(s.start.x, sx); s.start.y = sp(s.start.y, sy);
                s.end.x = sp(s.end.x, sx); s.end.y = sp(s.end.y, sy);
            }
            SO::Rectangle(ref mut s) => {
                let w = s.common.width as i32; let h = s.common.height as i32;
                s.x_coords = [0, w, w, 0]; s.y_coords = [0, 0, h, h];
            }
            SO::Ellipse(ref mut s) => {
                s.center.x = sp(s.center.x, sx); s.center.y = sp(s.center.y, sy);
                s.axis1.x = sp(s.axis1.x, sx); s.axis1.y = sp(s.axis1.y, sy);
                s.axis2.x = sp(s.axis2.x, sx); s.axis2.y = sp(s.axis2.y, sy);
                s.start1.x = sp(s.start1.x, sx); s.start1.y = sp(s.start1.y, sy);
                s.end1.x = sp(s.end1.x, sx); s.end1.y = sp(s.end1.y, sy);
                s.start2.x = sp(s.start2.x, sx); s.start2.y = sp(s.start2.y, sy);
                s.end2.x = sp(s.end2.x, sx); s.end2.y = sp(s.end2.y, sy);
            }
            SO::Arc(ref mut s) => {
                s.center.x = sp(s.center.x, sx); s.center.y = sp(s.center.y, sy);
                s.axis1.x = sp(s.axis1.x, sx); s.axis1.y = sp(s.axis1.y, sy);
                s.axis2.x = sp(s.axis2.x, sx); s.axis2.y = sp(s.axis2.y, sy);
            }
            SO::Polygon(ref mut s) => {
                for p in &mut s.points { p.x = sp(p.x, sx); p.y = sp(p.y, sy); }
            }
            SO::Curve(ref mut s) => {
                for p in &mut s.points { p.x = sp(p.x, sx); p.y = sp(p.y, sy); }
            }
            _ => {}
        }
    }

    /// 그룹 자식 개체들을 비례 스케일 (크기/위치/도형좌표 포함)
    fn scale_group_children(children: &mut [crate::model::shape::ShapeObject], sx: f64, sy: f64) {
        use crate::model::shape::ShapeObject as SO;
        fn sp(v: i32, s: f64) -> i32 { (v as f64 * s).round() as i32 }

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
                    s.start.x = sp(s.start.x, sx); s.start.y = sp(s.start.y, sy);
                    s.end.x = sp(s.end.x, sx); s.end.y = sp(s.end.y, sy);
                }
                SO::Rectangle(ref mut s) => {
                    let w = new_cw as i32; let h = new_ch as i32;
                    s.x_coords = [0, w, w, 0]; s.y_coords = [0, 0, h, h];
                }
                SO::Ellipse(ref mut s) => {
                    s.center.x = sp(s.center.x, sx); s.center.y = sp(s.center.y, sy);
                    s.axis1.x = sp(s.axis1.x, sx); s.axis1.y = sp(s.axis1.y, sy);
                    s.axis2.x = sp(s.axis2.x, sx); s.axis2.y = sp(s.axis2.y, sy);
                    s.start1.x = sp(s.start1.x, sx); s.start1.y = sp(s.start1.y, sy);
                    s.end1.x = sp(s.end1.x, sx); s.end1.y = sp(s.end1.y, sy);
                    s.start2.x = sp(s.start2.x, sx); s.start2.y = sp(s.start2.y, sy);
                    s.end2.x = sp(s.end2.x, sx); s.end2.y = sp(s.end2.y, sy);
                }
                SO::Arc(ref mut s) => {
                    s.center.x = sp(s.center.x, sx); s.center.y = sp(s.center.y, sy);
                    s.axis1.x = sp(s.axis1.x, sx); s.axis1.y = sp(s.axis1.y, sy);
                    s.axis2.x = sp(s.axis2.x, sx); s.axis2.y = sp(s.axis2.y, sy);
                }
                SO::Polygon(ref mut s) => {
                    for p in &mut s.points {
                        p.x = sp(p.x, sx); p.y = sp(p.y, sy);
                    }
                }
                SO::Curve(ref mut s) => {
                    for p in &mut s.points {
                        p.x = sp(p.x, sx); p.y = sp(p.y, sy);
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
        self.document.sections.get(section_idx)
            .map(|section| {
                section.paragraphs.iter()
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
        use crate::model::shape::*;
        use crate::model::control::Control;

        if targets.len() < 2 {
            return Err(HwpError::RenderError("묶기 위해서는 2개 이상의 개체가 필요합니다".to_string()));
        }
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)));
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
                return Err(HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", pi)));
            }
            if ci >= section.paragraphs[pi].controls.len() {
                return Err(HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과 (문단 {})", ci, pi)));
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
                _ => return Err(HwpError::RenderError(format!("컨트롤 ({},{})은 Shape/Picture가 아닙니다", pi, ci))),
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
                let offset = if i < para.char_offsets.len() { para.char_offsets[i] } else { prev_end };
                while prev_end + 8 <= offset && ctrl_ci < para.controls.len() {
                    if ctrl_ci == ci { gap_start = Some(prev_end); break 'outer; }
                    ctrl_ci += 1;
                    prev_end += 8;
                }
                let char_size: u32 = if text_chars[i] == '\t' { 8 }
                    else if text_chars[i].len_utf16() == 2 { 2 }
                    else { 1 };
                prev_end = offset + char_size;
            }
            if gap_start.is_none() {
                while ctrl_ci < para.controls.len() {
                    if ctrl_ci == ci { gap_start = Some(prev_end); break; }
                    ctrl_ci += 1;
                    prev_end += 8;
                }
            }
            if let Some(gs) = gap_start {
                let threshold = gs + 8;
                for offset in para.char_offsets.iter_mut() {
                    if *offset >= threshold { *offset -= 8; }
                }
            }

            para.controls.remove(ci);
            if ci < para.ctrl_data_records.len() {
                para.ctrl_data_records.remove(ci);
            }
            if para.char_count >= 8 { para.char_count -= 8; }
        }

        // 5) 삽입 위치 인덱스 재계산 (제거 후 인덱스가 변했을 수 있음)
        //    insert_target의 para에서 그보다 앞에서 제거된 개체 수만큼 보정
        let (insert_pi, insert_ci_orig) = insert_target;
        let removed_before = sorted_targets.iter()
            .filter(|&&(pi, ci)| pi == insert_pi && ci < insert_ci_orig)
            .count();
        let insert_ci = insert_ci_orig - removed_before;

        // 6) GroupShape를 문단에 삽입
        {
            let para = &mut self.document.sections[section_idx].paragraphs[insert_pi];

            // controls/ctrl_data_records 삽입 (범위 보정)
            let ctrl_insert = insert_ci.min(para.controls.len());
            para.controls.insert(ctrl_insert, Control::Shape(Box::new(group_obj)));
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

        self.event_log.push(DocumentEvent::PictureInserted { section: section_idx, para: insert_pi });
        Ok(super::super::helpers::json_ok_with(&format!("\"paraIdx\":{},\"controlIdx\":{}", insert_pi, insert_ci)))
    }

    /// GroupShape를 풀어 자식 개체들을 개별 Shape/Picture로 복원한다.
    /// 스펙: 한 단계만 풀기 (중첩 그룹은 유지), 자식 cnt 1 감소
    pub fn ungroup_shape_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        control_idx: usize,
    ) -> Result<String, HwpError> {
        use crate::model::shape::*;
        use crate::model::control::Control;

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)));
        }
        let section = &mut self.document.sections[section_idx];
        if para_idx >= section.paragraphs.len() {
            return Err(HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)));
        }
        let para = &mut section.paragraphs[para_idx];
        if control_idx >= para.controls.len() {
            return Err(HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)));
        }

        // GroupShape 추출
        match &para.controls[control_idx] {
            Control::Shape(s) => match s.as_ref() {
                ShapeObject::Group(_) => {}
                _ => return Err(HwpError::RenderError("지정된 컨트롤이 GroupShape이 아닙니다".to_string())),
            },
            _ => return Err(HwpError::RenderError("지정된 컨트롤이 Shape이 아닙니다".to_string())),
        };
        // GroupShape를 꺼냄
        let group_ctrl = para.controls.remove(control_idx);
        if control_idx < para.ctrl_data_records.len() {
            para.ctrl_data_records.remove(control_idx);
        }
        if para.char_count >= 8 { para.char_count -= 8; }

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
        let group_sx = if gsa.original_width > 0 { gsa.current_width as f64 / gsa.original_width as f64 } else { 1.0 };
        let group_sy = if gsa.original_height > 0 { gsa.current_height as f64 / gsa.original_height as f64 } else { 1.0 };

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
                if c.width == 0 && sa_w > 0 { c.width = sa_w; }
                if c.height == 0 && sa_h > 0 { c.height = sa_h; }
                if c.horizontal_offset == 0 && sa_ox > 0 { c.horizontal_offset = sa_ox as u32; }
                if c.vertical_offset == 0 && sa_oy > 0 { c.vertical_offset = sa_oy as u32; }
            }
            // 자식의 로컬 좌표를 글로벌 좌표로 변환 (그룹 스케일 적용)
            {
                let c = child.common_mut();
                c.horizontal_offset = (group_x + (c.horizontal_offset as f64 * group_sx) as i32) as u32;
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
                if sa.group_level > 0 { sa.group_level -= 1; }
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
            para.controls.insert(insert_idx, Control::Shape(Box::new(child)));
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

        self.event_log.push(DocumentEvent::PictureDeleted { section: section_idx, para: para_idx, ctrl: control_idx });
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
        let section = self.document.sections.get(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?;

        let ctrl = if let (Some(ci), Some(cpi)) = (cell_idx, cell_para_idx) {
            // 표 셀 내 수식
            let para = section.paragraphs.get(parent_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
            let table = match para.controls.get(control_idx) {
                Some(Control::Table(t)) => t,
                _ => return Err(HwpError::RenderError("지정된 컨트롤이 표가 아닙니다".to_string())),
            };
            let cell = table.cells.get(ci)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", ci)))?;
            let cell_para = cell.paragraphs.get(cpi)
                .ok_or_else(|| HwpError::RenderError(format!("셀 문단 인덱스 {} 범위 초과", cpi)))?;
            // 셀 문단의 첫 번째 수식 컨트롤을 찾는다
            cell_para.controls.iter().find(|c| matches!(c, Control::Equation(_)))
                .ok_or_else(|| HwpError::RenderError("셀 문단에 수식 컨트롤이 없습니다".to_string()))?
        } else {
            // 본문 수식
            let para = section.paragraphs.get(parent_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
            para.controls.get(control_idx)
                .ok_or_else(|| HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)))?
        };

        match ctrl {
            Control::Equation(e) => Ok(e),
            _ => Err(HwpError::RenderError("지정된 컨트롤이 수식이 아닙니다".to_string())),
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
        let section = self.document.sections.get_mut(section_idx)
            .ok_or_else(|| HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)))?;

        let ctrl = if let (Some(ci), Some(cpi)) = (cell_idx, cell_para_idx) {
            // 표 셀 내 수식
            let para = section.paragraphs.get_mut(parent_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
            let table = match para.controls.get_mut(control_idx) {
                Some(Control::Table(t)) => t,
                _ => return Err(HwpError::RenderError("지정된 컨트롤이 표가 아닙니다".to_string())),
            };
            let cell = table.cells.get_mut(ci)
                .ok_or_else(|| HwpError::RenderError(format!("셀 인덱스 {} 범위 초과", ci)))?;
            let cell_para = cell.paragraphs.get_mut(cpi)
                .ok_or_else(|| HwpError::RenderError(format!("셀 문단 인덱스 {} 범위 초과", cpi)))?;
            cell_para.controls.iter_mut().find(|c| matches!(c, Control::Equation(_)))
                .ok_or_else(|| HwpError::RenderError("셀 문단에 수식 컨트롤이 없습니다".to_string()))?
        } else {
            // 본문 수식
            let para = section.paragraphs.get_mut(parent_para_idx)
                .ok_or_else(|| HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", parent_para_idx)))?;
            para.controls.get_mut(control_idx)
                .ok_or_else(|| HwpError::RenderError(format!("컨트롤 인덱스 {} 범위 초과", control_idx)))?
        };

        match ctrl {
            Control::Equation(e) => Ok(e),
            _ => Err(HwpError::RenderError("지정된 컨트롤이 수식이 아닙니다".to_string())),
        }
    }

    pub fn get_equation_properties_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: Option<usize>,
        cell_para_idx: Option<usize>,
    ) -> Result<String, HwpError> {
        let eq = self.find_equation_ref(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx)?;

        let script_escaped = super::super::helpers::json_escape(&eq.script);
        let font_name_escaped = super::super::helpers::json_escape(&eq.font_name);

        Ok(format!(
            concat!(
                "{{\"script\":\"{}\",\"fontSize\":{},\"color\":{},",
                "\"baseline\":{},\"fontName\":\"{}\"}}"
            ),
            script_escaped, eq.font_size, eq.color,
            eq.baseline, font_name_escaped,
        ))
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
        use super::super::helpers::{json_u32, json_i32, json_str};
        use crate::renderer::equation::tokenizer::tokenize;
        use crate::renderer::equation::parser::EqParser;
        use crate::renderer::equation::layout::EqLayout;
        use crate::renderer::hwpunit_to_px;

        let dpi = self.dpi;
        let eq = self.find_equation_mut(section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx)?;

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

        // 수식 레이아웃 실행 → 개체 크기(common.width/height) 갱신
        let font_size_px = hwpunit_to_px(eq.font_size as i32, dpi);
        let tokens = tokenize(&eq.script);
        let ast = EqParser::new(tokens).parse();
        let layout_box = EqLayout::new(font_size_px).layout(&ast);
        let new_w = crate::renderer::px_to_hwpunit(layout_box.width, dpi).max(0) as u32;
        let new_h = crate::renderer::px_to_hwpunit(layout_box.height, dpi).max(0) as u32;
        eq.common.width = new_w;
        eq.common.height = new_h;

        // 표 셀 내 수식인 경우 표 dirty 플래그 설정
        if cell_idx.is_some() {
            if let Some(Control::Table(t)) = self.document.sections[section_idx]
                .paragraphs[parent_para_idx].controls.get_mut(control_idx)
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

    /// 수식 스크립트를 SVG로 렌더링하여 반환한다 (미리보기 전용).
    pub fn render_equation_preview_native(
        &self,
        script: &str,
        font_size_hwpunit: u32,
        color: u32,
    ) -> Result<String, HwpError> {
        use crate::renderer::equation::tokenizer::tokenize;
        use crate::renderer::equation::parser::EqParser;
        use crate::renderer::equation::layout::EqLayout;
        use crate::renderer::equation::svg_render::{render_equation_svg, eq_color_to_svg};

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

    // ─── 각주 삽입/삭제 API ──────────────────────────────

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
        use crate::model::paragraph::{Paragraph, CharShapeRef, LineSeg};

        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx)));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!("문단 인덱스 {} 범위 초과", para_idx)));
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
                                let positions = crate::document_core::helpers::find_control_text_positions(para);
                                let pos = positions.get(ci).copied().unwrap_or(usize::MAX);
                                if pos <= char_offset { count += 1; }
                            }
                        }
                        // 표 셀 내 각주
                        Control::Table(table) if is_before || is_same => {
                            for cell in &table.cells {
                                for cp in &cell.paragraphs {
                                    count += cp.controls.iter()
                                        .filter(|c| matches!(c, Control::Footnote(_)))
                                        .count() as u16;
                                }
                            }
                        }
                        // 글상자 내 각주
                        Control::Shape(shape) if is_before || is_same => {
                            if let Some(text_box) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                                for tp in &text_box.paragraphs {
                                    count += tp.controls.iter()
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
                                fp.char_shapes.first().map(|cs| cs.char_shape_id).unwrap_or(0),
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
                                                fp.char_shapes.first().map(|cs| cs.char_shape_id).unwrap_or(0),
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
                    current_para.char_shapes.first().map(|cs| cs.char_shape_id).unwrap_or(0),
                    current_para.para_shape_id,
                )
            })
        };

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
                segment_width: 0,
                tag: 0x00060000,
                ..Default::default()
            }],
            has_para_text: false,
            ..Default::default()
        };

        let footnote = Footnote {
            number: footnote_number,
            paragraphs: vec![inner_para],
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

        paragraph.controls.insert(insert_idx, Control::Footnote(Box::new(footnote)));
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
        paragraph.control_mask |= 0x00000010; // 각주 비트
        paragraph.has_para_text = true;

        // 전체 각주 순서 번호 재계산 (1부터 순차)
        // 본문 문단 + 표 셀 + 글상자 내부의 각주를 모두 포함
        {
            let mut num = 1u16;
            for pi in 0..self.document.sections[section_idx].paragraphs.len() {
                for ci in 0..self.document.sections[section_idx].paragraphs[pi].controls.len() {
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
                            if let Some(text_box) = shape.drawing_mut().and_then(|d| d.text_box.as_mut()) {
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
            use crate::renderer::hwpunit_to_px;
            use crate::renderer::composer::reflow_line_segs;
            let page_def = &self.document.sections[section_idx].section_def.page_def;
            let text_width = page_def.width as i32
                - page_def.margin_left as i32
                - page_def.margin_right as i32;
            let available_width = hwpunit_to_px(text_width, self.dpi);
            let para_style = self.styles.para_styles.get(
                self.document.sections[section_idx].paragraphs[para_idx].para_shape_id as usize
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

        self.event_log.push(DocumentEvent::PictureInserted { section: section_idx, para: para_idx });
        Ok(format!("{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{},\"footnoteNumber\":{}}}", para_idx, insert_idx, footnote_number))
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
            .create_shape_control_native(0, 0, 0, 9000, 6750, 0, 0, false, "InFrontOfText", "rectangle", false, false, &[])
            .expect("create rectangle");
        let para_idx = res
            .split("\"paraIdx\":").nth(1).and_then(|s| s.split(',').next())
            .and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        let ctrl_idx = res
            .split("\"controlIdx\":").nth(1).and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        (para_idx, ctrl_idx)
    }

    fn shape_common<'a>(core: &'a DocumentCore, para: usize, ctrl: usize) -> &'a crate::model::shape::CommonObjAttr {
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
        assert!(common.width >= MIN_SHAPE_SIZE, "width clamped: {}", common.width);
        assert!(common.height >= MIN_SHAPE_SIZE, "height clamped: {}", common.height);
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
