# Task #1596 — Stage 1-3 완료보고서 (RED + 지오메트리 방출)

**브랜치**: `local/task1596`

## Stage 1 (RED)
`task1596_polygon_geometry_serialized`: polygon 직렬화 후 hc:pt/lineShape/shadow 방출 검증 → RED.

## Stage 2-3 (수정)
`render_common_shape_xml` 리팩터: 시그니처 `sa` → `drawing: Option<&DrawingObjAttr>` + `points`.
shape_block 직후 지오메트리(lineShape·fillBrush(조건부)·shadow(조건부)·hc:pt) 방출. 태그 부수
속성(numberingType/dropcapstyle/href/groupLevel/instid) + pos 속성(affectLSpacing/flowWithText/
allowOverlap/holdAnchorAndSO) 보강. write_line_shape/write_shadow/numbering_type_str pub(crate).
dispatch(ellipse/arc/polygon/curve/chart)가 drawing+points 전달.

## 검증
- RED GREEN, `cargo test --lib` 1970/0, baseline 4/4, clippy 무경고.

## 다음
Stage 4 — fidelity 통제 비교 + 한글 오라클(36396457 등 잔여 붕괴 해소).
