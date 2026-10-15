//! 재조판한 프레임의 좌표를 측정과 배치에 함께 전달한다.

use super::ComposedParagraph;
use crate::model::paragraph::{LineSeg, Paragraph, SpaceMetric};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub(crate) struct FrameGeometry {
    pub(super) line_segs: Vec<LineSeg>,
    pub(super) space_metrics: Vec<(u32, SpaceMetric)>,
}

impl ComposedParagraph {
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
                Cow::Owned(paragraph)
            }
            None => Cow::Borrowed(source),
        }
    }
}
