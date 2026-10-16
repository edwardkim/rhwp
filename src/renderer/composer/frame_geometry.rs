//! 재조판한 프레임의 좌표를 측정과 배치에 함께 전달한다.

use super::ComposedParagraph;
use crate::model::paragraph::{LineSeg, Paragraph, SpaceMetric};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub(crate) struct FrameGeometry {
    pub(super) line_segs: Vec<LineSeg>,
    pub(super) space_metrics: Vec<(u32, SpaceMetric)>,
    pub(super) inline_table_heights: Vec<(usize, u32)>,
}

impl ComposedParagraph {
    /// 여러 인라인 표의 같은 줄/다른 줄 소속을 현재 프레임이 발행했는지 본다.
    pub(crate) fn frame_owns_multiple_inline_tables(&self) -> bool {
        self.frame_geometry
            .as_ref()
            .is_some_and(|frame| frame.inline_table_heights.len() > 1)
    }

    /// 저장 줄과 현재 프레임은 같은 원시 제어문 슬롯 축을 소비한다.
    pub(crate) fn control_line_indices(&self, para: &Paragraph) -> Option<Vec<usize>> {
        if self
            .frame_geometry
            .as_ref()
            .is_some_and(|frame| !frame.inline_table_heights.is_empty())
        {
            (0..para.controls.len())
                .map(|index| super::control_slot_line_index(para, index))
                .collect()
        } else {
            crate::renderer::float_placement::stored_control_line_indices(para)
        }
    }

    /// 원본 표의 식별자는 유지하고 앵커 계산에는 현재 프레임의 본체 높이를 쓴다.
    pub(crate) fn inline_table_height_hu(&self, index: usize) -> Option<u32> {
        self.frame_geometry
            .as_ref()?
            .inline_table_heights
            .iter()
            .find_map(|&(owner, height)| (owner == index).then_some(height))
    }

    /// 실제 프레임 줄의 점유 높이에 포함해 발행한 인라인 표인지 조회한다.
    pub(crate) fn frame_owns_inline_table(&self, index: usize) -> bool {
        self.frame_geometry.as_ref().is_some_and(|frame| {
            frame
                .inline_table_heights
                .iter()
                .any(|&(owner, _)| owner == index)
        })
    }

    /// 원본의 내용·속성을 유지하고 줄 좌표만 실제 재조판 결과로 읽는다.
    /// 좌표가 바뀌지 않은 문단은 복사 없이 원본을 빌린다.
    pub(crate) fn paragraph_with_frame_geometry<'a>(
        &self,
        source: &'a Paragraph,
    ) -> Cow<'a, Paragraph> {
        match &self.frame_geometry {
            Some(frame) => {
                let mut paragraph = source.clone();
                paragraph.replace_line_segs_with_space_metrics(
                    frame.line_segs.clone(),
                    frame.space_metrics.clone(),
                );
                for &(index, height) in &frame.inline_table_heights {
                    if let Some(crate::model::control::Control::Table(table)) =
                        paragraph.controls.get_mut(index)
                    {
                        table.common.height = height;
                    }
                }
                Cow::Owned(paragraph)
            }
            None => Cow::Borrowed(source),
        }
    }
}
