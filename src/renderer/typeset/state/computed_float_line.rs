//! 계산 빈 호스트의 표와 다음 글줄이 실제 점유하는 상자로 흐름을 결정한다.

use super::TypesetState;
use crate::renderer::float_placement::{FloatPushedFirstLine, ParagraphFloatFlow};
use crate::renderer::typeset::paragraph::metrics::FormattedParagraph;

impl TypesetState {
    pub(in crate::renderer::typeset) fn computed_float_owns_band(&self, owner: usize) -> bool {
        self.data
            .paragraph_float_placements
            .iter()
            .any(|(&(para, _), placement)| {
                para == owner
                    && matches!(placement.flow, ParagraphFloatFlow::ComputedEmptyHost { .. })
            })
    }

    pub(in crate::renderer::typeset) fn resolve_computed_float_first_line(
        &mut self,
        para_index: usize,
        fmt: &FormattedParagraph,
    ) {
        let height = fmt
            .line_heights
            .first()
            .copied()
            .unwrap_or(fmt.height_for_fit);
        if !height.is_finite() || height <= 0.0 {
            return;
        }
        let mut line_top = self.data.current_height + fmt.spacing_before;
        let mut handled = false;
        // 앞 간격은 글줄의 잉크 상자가 아니다. 자연 줄 원점을 먼저 정하고,
        // 그 상자만 표와 겹칠 때 옮긴다. 표 위에 들어가는 줄은 그대로 둔다.
        // 여러 밴드가 순서 없이 등록돼도 앞 밴드로 이동한 결과를 다시 확인한다.
        loop {
            let previous = line_top;
            for zone in &self.data.visible_float_exclusions {
                if zone.para_index >= para_index || !self.computed_float_owns_band(zone.para_index)
                {
                    continue;
                }
                handled = true;
                if line_top < zone.bottom && line_top + height > zone.top {
                    line_top = zone.bottom;
                }
            }
            if line_top == previous {
                break;
            }
        }
        if handled {
            let origin = line_top - fmt.spacing_before;
            self.data.current_height = origin;
            self.data.float_pushed_line_spacings.insert(
                para_index,
                FloatPushedFirstLine {
                    origin: Some(origin),
                    spacing_before: fmt.spacing_before,
                },
            );
        }
    }
}
