# 최종 결과보고서 — Task M100 #1200

**이슈**: [#1200](https://github.com/edwardkim/rhwp/issues/1200) HWPX 도형(curve) 외곽선 미렌더링 — "다른 풀이" 태그 박스 미표시
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task1200` (← `stream/devel`, #1199와 독립)
**완료일**: 2026-06-01

---

## 1. 문제

`samples/3-09월_교육_통합_2022.hwpx` 9쪽 우측 단 "다른 풀이"가 PDF에는 태그 박스 안에 표시되나, 우리 렌더는 텍스트만 있고 박스가 없음.

## 2. 근본 원인

"다른 풀이"는 `<hp:curve id="0">` 도형의 `drawText`. 박스는 이 curve 외곽선.
HWPX 이 curve geometry 가 **`<hp:seg x1 y1 x2 y2>` 417개**로 인코딩(`<hc:pt>` 0개)되는데, 파서 `parse_shape_object()`는 가변 꼭짓점을 **`<hc:pt>`에서만** 수집 → `CurveShape.points` empty → `curve_to_path_commands_scaled()` 빈 path → 외곽선 미렌더. (drawText 텍스트는 별도 경로라 텍스트만 보임.)

렌더 인프라(stroke→SVG/Skia)는 이미 완비 — 점만 채우면 그려짐.

## 3. 수정

`src/parser/hwpx/section.rs` `parse_shape_object()` 에 `b"seg"` 분기 추가:
- 첫 seg `(x1,y1)` + 각 seg `(x2,y2)` → `polygon_points` 폴리라인.
- `segment_types` 비움 → 렌더러 LineTo (seg 는 sampled 꼭짓점, 제어점 아님).
- 65줄 추가(분기 1 + 테스트 1), 삭제 0. 모델/렌더러/HWP3/공통 무변경.

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo build --release` | 성공 |
| `cargo test --release` (전체) | 통과, 회귀 0 |
| 신규 회귀 테스트 1건 | 통과 |
| 9쪽 SVG path | 417-seg `<path stroke>` 외곽선 신규 (d_len 105→15577) |
| 한글 2022 PDF 9쪽 대조 | "다른 풀이" 태그 박스 시각 정합 |
| rustfmt (변경 파일) | clean |

## 5. 범위 외

- `<hp:seg>` CURVE 타입의 진짜 베지어 정밀화(점-대-점 폴리라인으로 충분). 필요 시 후속.
- 11쪽 "[다른 풀이]"는 본문 리터럴 텍스트(도형 아님) — 무관.

## 6. 산출물

- 소스: `src/parser/hwpx/section.rs`
- 계획: `mydocs/plans/task_m100_1200.md`, `_impl.md`
- 단계 보고: `mydocs/working/task_m100_1200_stage{1,2,3}.md`
- 최종 보고: 본 문서
