# Task #1598 Stage 1 — RED 테스트 + 통제 진단

## 진단 (통제 테스트로 근본 확정)

36385226(ellipse×9, section0)을 한글 오라클로 3-way 측정:

| 파일 | 한글 PageCount |
|------|---------------|
| orig | 3 |
| rt (지오메트리 드롭, 수정 전) | **2** ← 붕괴 |
| rt + orig 지오메트리 주입 | **3** ← 해소 |

→ ellipse 전용 지오메트리(`<hc:center>/<hc:ax1>/<hc:ax2>/<hc:start1>/<hc:end1>/`
`<hc:start2>/<hc:end2>`) 주입만으로 붕괴 완전 해소. **근본 확정**.

주입 스크립트: `output/poc/ellipse_test/inject_geom.py`, 측정: `measure3.py`.

## 근본원인

- 파서 `parse_shape_object`(section.rs)가 ellipse/arc 자식 점요소를 `_ => {}` 로 버림
  → `EllipseShape`/`ArcShape` 를 `..Default::default()` 로 생성(지오메트리 전부 0).
- 직렬화 `render_common_shape_xml` 도 미방출.
- IR diff 게이트는 ellipse 지오메트리 미비교(IR-invisible) → 한글 오라클만 검출.

## RED 테스트

`tests/issue_1598_ellipse_geometry_roundtrip.rs`:
- 36385226 파싱 → ellipse≥9 + 전용 지오메트리 nonzero 단언(수정 전엔 전부 0 → RED).
- serialize→reparse 후 7점 보존 + 2-round 안정.
- 가드 샘플: `samples/hwpx/opengov/36385226_...hwpx`.
