//! 완전한 다줄 정렬 프레임의 기준선을 손실된 정수 퍼센트로 다시 만들지 않는다.

use super::layout::{LayoutBox, LayoutKind};
use crate::model::control::Equation;
use crate::renderer::{px_to_hwpunit, DEFAULT_DPI};

fn owns_alignment_frame(value: &LayoutBox) -> bool {
    match &value.kind {
        LayoutKind::EqAlign { .. } => true,
        LayoutKind::Color { body, .. } | LayoutKind::FontStyle { body, .. } => {
            owns_alignment_frame(body)
        }
        LayoutKind::Row(items) if items.len() == 1 => owns_alignment_frame(&items[0]),
        _ => false,
    }
}

pub(super) fn metrics(equation: &Equation, intrinsic: &LayoutBox) -> Option<(u32, u32, i32)> {
    if !owns_alignment_frame(intrinsic) {
        return None;
    }
    let height = px_to_hwpunit(intrinsic.height, DEFAULT_DPI);
    if height <= 0 || equation.common.height != height as u32 {
        // 다른 예약 높이나 보호된 프레임을 자연 상자로 바꾸지 않는다.
        // 보호·글꼴·lineMode의 수용 조건은 공통 호출자가 검사한다.
        return None;
    }
    // 저장된 폭과 높이는 유지한다. 첫 행의 같은 구성 결과가 측정과
    // 실제 배치에 기준선을 공급하므로 paint의 별도 y 보정도 필요 없다.
    Some((
        equation.common.width,
        equation.common.height,
        px_to_hwpunit(intrinsic.baseline, DEFAULT_DPI),
    ))
}
