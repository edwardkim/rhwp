//! 인라인 그림 전환의 앵커와 실제 혼합 글줄을 같은 소유자에서 갱신한다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use crate::renderer::composer::{reflow_line_segs, ParagraphBox};
use crate::renderer::style_resolver::ResolvedStyleSet;

pub(super) fn reflow_text_host(
    para: &mut Paragraph,
    frame: ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
) {
    // 개체의 줄 참여가 바뀌었으므로 이전 줄 소속도 현재 입력의 캐시가 아니다.
    para.invalidate_layout_inputs();
    reflow_line_segs(para, frame, styles, dpi);
}

impl DocumentCore {
    /// 본문은 실제 단, 미주·머리말·꼬리말은 독립된 내용 영역의 폭을 사용한다.
    pub(super) fn picture_host_width(
        &self,
        section_idx: usize,
        body_para_idx: Option<usize>,
    ) -> Result<f64, HwpError> {
        let section = self.document.sections.get(section_idx).ok_or_else(|| {
            HwpError::RenderError(format!("구역 인덱스 {} 범위 초과", section_idx))
        })?;
        if let Some(index) = body_para_idx.filter(|&index| index < section.paragraphs.len()) {
            let columns = Self::find_column_def_for_paragraph(&section.paragraphs, index);
            let layout = crate::renderer::page_layout::PageLayoutInfo::from_page_def(
                &section.section_def.page_def,
                &columns,
                self.dpi,
            );
            let column = self
                .para_column_map
                .get(section_idx)
                .and_then(|map| map.get(index))
                .copied()
                .unwrap_or(0) as usize;
            return Ok(layout
                .column_areas
                .get(column)
                .unwrap_or(&layout.column_areas[0])
                .width);
        }
        let page = &section.section_def.page_def;
        Ok(crate::renderer::hwpunit_to_px(
            page.width as i32 - page.margin_left as i32 - page.margin_right as i32,
            self.dpi,
        ))
    }

    pub(crate) fn migrate_picture_floating_to_inline(
        para: &mut Paragraph,
        control_idx: usize,
        frame: ParagraphBox,
        styles: &ResolvedStyleSet,
        dpi: f64,
    ) {
        use crate::model::shape::{HorzRelTo, VertRelTo};
        let pic = match para.controls.get_mut(control_idx) {
            Some(Control::Picture(picture)) => picture.as_mut(),
            Some(Control::Shape(shape)) => match shape.as_mut() {
                ShapeObject::Picture(picture) => picture,
                _ => return,
            },
            _ => return,
        };
        pic.common.horz_rel_to = HorzRelTo::Para;
        pic.common.vert_rel_to = VertRelTo::Para;
        pic.common.horizontal_offset = 0;
        pic.common.vertical_offset = 0;
        crate::serializer::control::sync_anchor_bits(&mut pic.common);

        let picture_height_hu = crate::renderer::composer::inline_picture_occupied_size_hu(pic).1;
        if !para.text.is_empty() {
            reflow_text_host(para, frame, styles, dpi);
            return;
        }
        // #1151의 독립 한컴 저장본은 텍스트 없는 개체 문단이다. 그 계약을
        // 혼합 줄에 일반화해 큰 글자나 다른 줄의 기준선을 덮지 않는다.
        let baseline = (picture_height_hu as f64 * 0.85).round() as i32;
        if let Some(seg) = para.line_segs.first_mut() {
            seg.line_height = picture_height_hu;
            seg.text_height = picture_height_hu;
            seg.baseline_distance = baseline;
        } else {
            para.line_segs.push(crate::model::paragraph::LineSeg {
                line_height: picture_height_hu,
                text_height: picture_height_hu,
                baseline_distance: baseline,
                line_spacing: 600,
                ..Default::default()
            });
        }
    }
}
