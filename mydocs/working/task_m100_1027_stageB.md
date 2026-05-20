# Stage B 완료보고서 — #1027: overlay-shape bypass 가드 순수 함수 추출

- 타스크: #1027 / 브랜치 `local/task1027`
- 작성일: 2026-05-20
- 단계: Stage B — 공유 측정 엔진 리팩터 2단계 (무동작 추출)

## 변경
`layout.rs` VPOS_CORR 의 bypass 가드(이전 문단이 글앞뒤/위아래+vert=Para 비-TAC
Shape/Picture 를 가지면 vpos 보정 base 산출 제외)를 순수 함수로 추출:

```rust
pub(crate) fn para_has_overlay_shape(para: &Paragraph) -> bool
```
- 렌더러는 `paragraphs.get(prev_pi).map(para_has_overlay_shape).unwrap_or(false)` 로 호출. 로직 동일.
- tac=true 제외(#539) 규칙 보존. `pub(crate)` 로 페이지네이터 공유 대비.

## 무동작 검증 (병합본 baseline)
| 지표 | baseline | Stage B |
|------|----------|---------|
| 노트 | 9쪽 | 9쪽 ✅ |
| 페이지 수 | 185 | 185 ✅ |
| LAYOUT_OVERFLOW | 13 | 13 ✅ |
| svg_snapshot | 5 pass/3 debt | 5 pass/3 debt ✅ |
| clippy | 0 | 0 ✅ |

## 다음 (Stage C)
`HeightCursor`(height-only) 구현 — prev_vpos_end/base 상태 추적 + Stage A
`vpos_corrected_end_y` + Stage B `para_has_overlay_shape` 가드 결합. 렌더러
y_offset 진행과 1:1 parity 단위테스트(여러 샘플) 선행 후 Stage D 에서 typeset 교체.
(Stage C/D 부터 실제 동작 변경·고위험 — 광범위 골든 재판정.)
