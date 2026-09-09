# Task #1598 Stage 2 — 파서 + 직렬화 구현

## 파서 (`src/parser/hwpx/section.rs`)

- 헬퍼 `parse_xy(e, &mut Point)` 추가 — `<hc:* x y/>` 의 x/y 속성 적재.
- `parse_shape_object` 자식 루프에 7개 분기 추가:
  `b"center"|"ax1"|"ax2"|"start1"|"end1"|"start2"|"end2" => parse_xy(...)`.
- ellipse 생성자: center/axis1/axis2/start1/end1/start2/end2 적재.
- arc 생성자: center/axis1/axis2 적재(호는 시작끝점 없음).

## 직렬화 (`src/serializer/hwpx/section.rs`)

- 디스패치에서 shape 별 `geom_tail` 문자열 빌드:
  - ellipse → center/ax1/ax2/start1/end1/start2/end2 (7개 `<hc:*>`).
  - arc → center/ax1/ax2 (3개).
  - 그 외 → 빈 문자열.
- `render_common_shape_xml` 시그니처에 `geom_tail: &str` 추가, shadow 직후 방출
  (`{ls}{fb}{sh}{pts}{geom_tail}`). hc:pt(polygon/curve) 와 상호배타.

## 검증

- 단위 테스트 `issue_1598_ellipse_geometry_roundtrip`: **PASS**.
- 지오메트리 값 정확 일치(orig==rt): center(460,460)/ax1(460,0)/ax2(920,460)/start·end(0,0).
- IR diff=0 유지(회귀 없음).
- polygon/curve points 는 #1067/#1200 으로 이미 정상 — 무영향.
