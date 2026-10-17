//! Stored absolute group coordinates and full source affine.
use crate::model::shape::ShapeObject;
pub(super) fn assign_group_coordinates(node: &mut ShapeObject, level: u16, _parent_origin: (f64, f64)) {
    let attr = node.shape_attr_mut();
    attr.group_level = level;
    if level == 0 {
        // 최상위 개체는 문단/그림 틀의 앵커를 사용하는 별도 경로다.
        attr.render_sx = 1.0;
        attr.render_b = 0.0;
        attr.render_tx = 0.0;
        attr.render_c = 0.0;
        attr.render_sy = 1.0;
        attr.render_ty = 0.0;
        attr.raw_rendering.clear();
    } else {
        // 공통 그룹 소비자는 틀 기준으로 합성된 행렬을 읽는다. HWP3의
        // 상대 offset 메타데이터를 paint에서 다시 합성하지 않는다.
        attr.raw_rendering = crate::serializer::control::serialize_affine_rendering(attr);
    }
    if let ShapeObject::Group(group) = node {
        for child in &mut group.children {
            assign_group_coordinates(child, level.saturating_add(1), (0.0, 0.0));
        }
    }
}
