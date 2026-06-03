# Stage 1 완료보고서 — Task M100 #1200

**단계**: 파서 — `<hp:seg>` → curve points
**브랜치**: `local/task1200`

## 변경

`src/parser/hwpx/section.rs` `parse_shape_object()`:
- `b"pt"`(`<hc:pt>`) 분기 옆에 `b"seg"`(`<hp:seg>`) 분기 추가.
- x1/y1/x2/y2 파싱. `polygon_points` 가 비어 있으면 첫 seg 의 `(x1,y1)` push, 이후 모든 seg 의 `(x2,y2)` push → chain 폴리라인.
- `segment_types` 미설정(빈 채 유지) → 렌더러가 LineTo 폴리라인으로 그림 (seg 는 제어점이 아닌 sampled 꼭짓점).

## 검증

- `cargo build --release` 성공.
- 9쪽 SVG: 417-segment `<path stroke="#000000">` (curve 외곽선) 신규 생성 확인 (수정 전 최대 path d_len=105 → 후 15577).
- 추가만(분기 1개), 기존 `<hc:pt>` 경로 불변.
