//! 연결선의 방향과 원본 선 스타일의 크기를 분리한다.

use crate::model::shape::LinkLineType;
use crate::renderer::{ArrowStyle, LineStyle};

/// 연결 방식은 화살촉의 방향만 정한다. HWP/HWPX의 headSz/tailSz는 보존한다.
pub(super) fn with_direction(mut style: LineStyle, kind: LinkLineType) -> LineStyle {
    match kind {
        LinkLineType::StraightOneWay | LinkLineType::StrokeOneWay | LinkLineType::ArcOneWay => {
            style.end_arrow = ArrowStyle::Arrow;
        }
        LinkLineType::StraightBoth | LinkLineType::StrokeBoth | LinkLineType::ArcBoth => {
            style.start_arrow = ArrowStyle::Arrow;
            style.end_arrow = ArrowStyle::Arrow;
        }
        _ => {}
    }
    style
}
