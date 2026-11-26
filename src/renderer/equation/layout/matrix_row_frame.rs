//! 줄의 논리 프레임과 실제 glyph 상자의 기준선을 연결한다.
use super::{EqLayout, LayoutBox, LayoutKind};
impl EqLayout {
    pub(super) fn matrix_cell_frame(&self, cell: &LayoutBox, fs: f64) -> (f64, f64) {
        if !(self.hancom_eqn || self.hancom_math_face) {
            return (cell.height, cell.baseline);
        }
        // 높이·기준선은 두 수식 face가 같은 구조 프레임을 쓴다(hancom_frame 근거).
        // #8927 첨부의 한컴 재조판본에서도 HYhwpEQ와 HancomEQN의 높이·기준선이
        // 수식 7개 모두 같았다. 글리프 전진폭만 face별로 다르다.
        super::super::hancom_frame::metrics(cell, fs, self.dpi)
            .unwrap_or((cell.height, cell.baseline))
    }
    pub(super) fn matrix_row_box(&self, mut child: LayoutBox, fs: f64) -> LayoutBox {
        if child.height == 0.0 {
            child.height = fs;
            child.baseline = fs
                * if self.hancom_math_face || self.hancom_eqn {
                    0.86
                } else {
                    0.8
                };
            return child;
        }
        let (height, baseline) = self.matrix_cell_frame(&child, fs);
        if height == child.height && baseline == child.baseline {
            return child;
        }
        let width = child.width;
        child.y = baseline - child.baseline;
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height,
            baseline,
            kind: LayoutKind::Row(vec![child]),
        }
    }
}
