//! XML 본문 캐시가 단·쪽 좌표를 아직 갖지 않는 경우의 재조판 판정.

use crate::model::paragraph::{ColumnBreakType, Paragraph};

/// 여러 본문 문단의 저장 줄이 모두 원점에 머물면 연속 본문의 저장 위치가 아니다.
/// 현재 글자모양과 개체로 다시 만든 같은 줄 결과를 측정과 배치에 공급해야 한다.
/// 호출자는 XML import 계약을 확인하고, 셀의 정상 로컬 좌표와 구분해 본문만 넘긴다.
pub(super) fn body_ladder_is_unpositioned(paragraphs: &[Paragraph]) -> bool {
    if paragraphs.len() < 2
        || paragraphs
            .iter()
            .skip(1)
            .any(|para| para.column_type != ColumnBreakType::None)
    {
        return false;
    }

    let mut has_cache = false;
    for para in paragraphs {
        for line in &para.line_segs {
            // 양수 저장 위치는 프레임의 증거이고, 0높이는 의도적으로 접힌 줄일 수 있다.
            if line.vertical_pos != 0 || line.line_height <= 0 {
                return false;
            }
            has_cache = true;
        }
    }
    has_cache
}

/// 원본 줄 출처가 남아 있는 미계산 본문만 선언 수식 프레임을 보존한다.
/// 저장 줄이 전혀 없는 정상 문서와 구현이 만든 줄은 이 출처가 아니다.
pub(super) fn body_ladder_has_original_source(paragraphs: &[Paragraph]) -> bool {
    paragraphs
        .iter()
        .flat_map(|para| &para.line_segs)
        .all(|line| line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
}

pub(super) fn retain_declared_equation_frames(paragraph: &mut Paragraph) {
    for control in &mut paragraph.controls {
        if let crate::model::control::Control::Equation(equation) = control {
            crate::renderer::equation::declared_frame::retain_unpositioned_host_frame(equation);
        }
    }
}
