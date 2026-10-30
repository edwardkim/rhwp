//! 닫힌 저장 행에서 그림 글줄의 내용 상자와 셀 안 여백을 구분한다.

use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, Paragraph},
};

/// 줄 구성과 같은 HWPUNIT 그림 점유 높이가 실제 저장 줄을 증명할 때만 수용한다.
/// 원래 전체 표를 닫는지, 편집·합성 줄인지 등은 공통 행 생산자가 추가 확인한다.
pub(super) fn controls_preserve_stored_picture_lines(para: &Paragraph) -> bool {
    // MultiColumn은 한 단 정의에도 붙는 IR 상태다. 실제 단/쪽 나눔과
    // 다단 배치는 이 단일 저장 행 인증에서 소비하지 않는다.
    let single_column = para
        .controls
        .iter()
        .any(|control| matches!(control, Control::ColumnDef(column) if column.column_count == 1));
    if para.column_type != ColumnBreakType::None
        && !(para.column_type == ColumnBreakType::MultiColumn && single_column)
    {
        return false;
    }
    let Some(owners) = super::stored_control_line_indices(para) else {
        return false;
    };
    if owners.len() != para.controls.len() {
        return false;
    }
    let mut occupied = vec![0; para.line_segs.len()];
    let mut has_picture = false;
    for (control, owner) in para.controls.iter().zip(owners) {
        let picture = match control {
            // 단 정의는 그림 줄 높이를 점유하는 개체가 아니다.
            Control::ColumnDef(column) if column.column_count == 1 => continue,
            Control::Picture(picture)
                if picture.common.treat_as_char && picture.caption.is_none() =>
            {
                picture
            }
            _ => return false,
        };
        let (width, height) = crate::renderer::composer::inline_picture_occupied_size_hu(picture);
        if width <= 0 || height <= 0 {
            return false;
        }
        let Some(line_height) = occupied.get_mut(owner) else {
            return false;
        };
        *line_height = (*line_height).max(height);
        has_picture = true;
    }
    has_picture
        && occupied
            .iter()
            .zip(&para.line_segs)
            .all(|(height, line)| *height == 0 || *height == line.line_height)
}
