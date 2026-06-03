# 구현계획서 — Task M100 #1200

**이슈**: [#1200](https://github.com/edwardkim/rhwp/issues/1200) HWPX 도형(curve) 외곽선 미렌더링
**브랜치**: `local/task1200`
**작성일**: 2026-06-01

---

## 단계 분할 (3단계)

### Stage 1 — 파서: `<hp:seg>` → curve points

`src/parser/hwpx/section.rs` `parse_shape_object()`:
- `b"pt"`(`<hc:pt>`) 분기 옆에 `b"seg"`(`<hp:seg>`) 분기 추가.
- x1/y1/x2/y2 파싱. `polygon_points` 가 비어 있으면 `(x1,y1)` 먼저 push, 이후 항상 `(x2,y2)` push (chain 폴리라인).
- `segment_types` 미설정(빈 채 유지) → 렌더러가 LineTo 폴리라인으로 그림.

산출물: 빌드 성공 + curve.points 채워짐.

### Stage 2 — 회귀 테스트

`section.rs` tests 모듈:
- `<hp:curve>` + `<hp:seg>` 3~4개로 구성된 최소 XML 파싱 → `CurveShape.points` 가 seg endpoint chain 으로 채워지는지 검증.
- 기존 `<hc:pt>` polygon 경로 회귀 없음 확인(기존 테스트 유지).

산출물: `cargo test` 통과.

### Stage 3 — 시각 검증

1. `rhwp export-svg samples/3-09월_교육_통합_2022.hwpx -o output/svg/t1200/ -p 8`.
2. "다른 풀이" 태그 박스 외곽선이 `<path ... stroke>` 로 렌더되는지 확인(rsvg 래스터 → 한글 2022 PDF 9쪽 대조).
3. `rustfmt`(변경 파일) → 최종 보고서.

산출물: 검증 결과 + 최종 보고서.

---

## 영향 범위

- 수정 파일: `src/parser/hwpx/section.rs` 1개(+테스트).
- 모델/렌더러/HWP3/공통 무변경.
