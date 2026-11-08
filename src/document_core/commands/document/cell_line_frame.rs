//! 셀 글줄의 인라인 축을 로드·명시적 재조판·중첩 셀에서 공유한다.

use crate::model::table::Cell;
use crate::model::Padding;
use crate::renderer::hwpunit_to_px;

pub(super) fn inline_extent_px(
    cell: &Cell,
    owner_width_hu: i32,
    table_padding: &Padding,
    dpi: f64,
) -> Option<f64> {
    let padding = cell.paragraph_frame_padding(table_padding);
    if cell.text_direction == 0 {
        return Some(crate::renderer::composer::cell_inner_text_width(
            hwpunit_to_px(owner_width_hu, dpi),
            hwpunit_to_px(i32::from(padding.left), dpi),
            hwpunit_to_px(i32::from(padding.right), dpi),
            dpi,
        ));
    }
    // 세로쓰기의 글 진행 축은 칸 높이다. 높이가 없거나 미지 센티널이면
    // 가로 폭을 빌려 합성 캐시를 만들지 않고 최종 배치 상자에서 구성한다.
    let height = i32::try_from(cell.height).ok()?;
    let inner = height
        .saturating_sub(i32::from(padding.top))
        .saturating_sub(i32::from(padding.bottom));
    (inner > 0).then(|| hwpunit_to_px(inner, dpi))
}
