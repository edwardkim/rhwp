# Stage 4 완료보고 — (B) block-continuation 연계 분석 (#1658 라운드3)

- 브랜치: local/task1658-vpos / 성격: 분석(소스 무변경). 결론: **분리 (안전 우선)**.

## 1. 통합 원리 (#1658 vpos 회계)
content-height 는 render 와 pagination 이 **동일 기준**으로 측정해야 한다 — double-count(over) 도,
stack 누락(under) 도 없이. 두 manifestation 은 이 원리의 **반대 방향** 위반이다.

| | (A) valign over-count [Stage 2 해결] | (B) byeolpyo4 block-continuation [본 분석] |
|---|---|---|
| 방향 | **over**-count | **under**-count(pagination) |
| 경로 | INLINE 셀 `table_layout.rs:2461` | 블록 배치 `typeset` consumed 누적 |
| 증상 | total≈2× → offset≈0 → 상단정렬 | consumed(178.7) < render(249.6) → 가드 통과 → overflow/클리핑 |
| 원인 | 중첩 표 높이 **가산** double-count | 다중 행 블록을 **max(단일 셀)** 로 측정(Σ-stack 누락) |
| 수정 | 가산 제거(measure **less**) | Σ-stack 도입(measure **more**) |

→ **방향·함수가 반대**라 Stage 2 수정(가산 제거)은 (B)에 **그대로 적용 불가**.

## 2. 독립성 확인 (데이터)
Stage 2 valign fix 적용/미적용 모두 **byeolpyo4 = 28쪽 / 클리핑 23.5px 불변**. valign fix 는
has_nested INLINE 셀의 세로정렬만 바꾸며, 별표4 거대 셀 분할 배치(블록 경로)와 무관함을 확정.

## 3. (B) 적용 가능성 — 고위험·분리 사유
- (B)의 본질(라운드2 5층 진단): pagination 의 블록 선행 행 consumed(158.9) < render Σ-stack(229.8).
  근본은 `row_block_content_height` 가 `max_h`(단일 셀) 반환 → 다중 행 블록 과소. render 는
  Σ per-row(stack) 로 그림.
- **라운드2 시도 기록**: `row_block_content_height` max→max(max,Σstack) = 전 게이트 무회귀이나
  **byeolpyo4 무개선**(선행 행 consumed 경로가 이 함수가 아닌 typeset 블록 누적 — 정확 함수 미-pinpoint).
  `advance_row_block_cut` 누적-offset = **`test_block_cut_rowspan_giant_split` 계약 위반**(이 테스트가
  max 기반 버그 동작을 assert). 즉 (B)는 **계약 테스트의 한글-검증 갱신** + typeset 블록 consumed
  정확 함수 pinpoint 가 선행되어야 하는 **코어 아키텍처 작업**.
- **인프라 의존**: (B) 검증은 round-2 PR #1688 산출물(clipping_gate/baseline/byeolpyo4 샘플)에 의존하나
  본 branch(upstream/devel 기반)에 미반영(#1688 미머지).

## 4. 결정 — 분리 (안전 우선)
구현계획서 Stage 4 기준("불가·고위험이면 분리 보고, 안전 우선") 충족:
- (A) valign over-count: round-3 에서 **해결·landed**(Stage 2).
- (B) byeolpyo4 block-continuation: **round-4 전용 작업으로 분리**. 선행 조건:
  1. PR #1688 머지(clipping 인프라/byeolpyo4 샘플 + continuation ≤3 확보),
  2. typeset 블록 선행 행 consumed 정확 함수 pinpoint(라운드2 미완 지점),
  3. `test_block_cut_rowspan_giant_split` 계약을 한글 권위로 재검증 후 max→Σ-stack 갱신,
  4. 6게이트(특히 클리핑) 동시 통제.

## 5. 완료 기준 충족
- (B) 적용 가능성 분석 + 독립성 데이터 확정. ✓
- 고위험 판정 → 분리 보고(round-4 선행조건 명시), 안전 ceiling 유지. ✓
- round-3 소스 = valign fix 단독(Stage 2), byeolpyo4 미변경.

## 6. 다음 (Stage 5) — 최종 검증·보고
round-3 (A) 최종 게이트 재확인 + 합성 fixture 상시 편입 + 최종 보고서. **승인 후 착수.**
