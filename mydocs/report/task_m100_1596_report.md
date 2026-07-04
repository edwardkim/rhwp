# Task #1596 — 최종 결과보고서

**제목**: HWPX generic-shape 지오메트리 직렬화 완성 (페이지 붕괴 잔여)
**마일스톤**: M100 · **이슈**: #1596 · **브랜치**: `local/task1596`

## 1. 근본원인
`render_common_shape_xml`(section.rs:1327)이 polygon/ellipse/arc/curve 의 지오메트리
(`<hp:lineShape>`·`<hp:shadow>`·`<hc:pt>` 꼭짓점)를 드롭. 도형 형상·테두리 소실 → 렌더 크기 변동
→ 페이지 붕괴(#1589 잔여 ~8%). IR 보유(파서 정상), serializer-only.

## 2. 수정
`render_common_shape_xml` 리팩터: `drawing: Option<&DrawingObjAttr>` + `points: &[Point]` 수신.
shape_block 직후 lineShape·fillBrush(조건부)·shadow(조건부)·hc:pt 방출(write_rect 동형). 태그 부수
속성(numberingType/dropcapstyle/href/groupLevel/instid) + pos 속성 보강. dispatch 갱신.

## 3. 검증
| 검사 | 결과 |
|------|------|
| 단위 RED→GREEN | PASS (hc:pt/lineShape/shadow) |
| cargo test --lib | 1970/0 |
| baseline | 4/4 |
| IR 통제 비교(11874) | IR_DIFF 4(회귀 0) |
| 한글: 36396457 | 11→4 붕괴 → 11→11 해소, 지오메트리 보존 |
| 한글 악화 | 이전 OK 40/40 유지 (0) |

## 4. 영향
#1589 잔여 붕괴(shape 관련)의 근본. 누적(#1594 holdAnchorAndSO + #1595 ClickHere + #1596 shape)으로
페이지 붕괴 군집 ~95%+ 해소. 도형 충실도(테두리/그림자/형상)도 복원.

## 5. 산출물
소스: section.rs(render_common_shape_xml/dispatch), shape.rs(헬퍼 pub). 가드: task1596_polygon_geometry_serialized.
