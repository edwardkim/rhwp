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
