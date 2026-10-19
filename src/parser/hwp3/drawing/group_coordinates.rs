//! HWP3 묶음 자식의 상대 위치를 공통 렌더링 좌표계로 합성한다.

use crate::model::shape::ShapeObject;

/// 최상위 개체의 페이지 앵커는 CommonObjAttr가 담당한다. 자식의 상대 위치만
/// 누적해 최상위 묶음의 로컬 좌표로 변환하며, 메타데이터 offset은 그대로 보존한다.
/// 공통 렌더러와 HWP/HWPX 저장기가 같은 합성 결과를 소비한다.
/// 그룹 깊이도 보존해 HWP5 저장기의 행렬 쌍 개수(`group_level + 1`)를 유지한다.
pub(super) fn assign_group_coordinates(
    node: &mut ShapeObject,
    level: u16,
    parent_origin: (f64, f64),
) {
    let attr = node.shape_attr_mut();
    attr.group_level = level;
    let origin = if level == 0 {
        (0.0, 0.0)
    } else {
        (
            parent_origin.0 + f64::from(attr.offset_x),
            parent_origin.1 + f64::from(attr.offset_y),
        )
    };
    attr.render_tx = origin.0;
    attr.render_ty = origin.1;

    if let ShapeObject::Group(group) = node {
        for child in &mut group.children {
            assign_group_coordinates(child, level.saturating_add(1), origin);
        }
    }
}
