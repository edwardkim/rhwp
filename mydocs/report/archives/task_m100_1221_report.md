# 최종 결과보고서 — Task M100 #1221

**이슈**: [#1221](https://github.com/edwardkim/rhwp/issues/1221) HWP5 표 셀 세로정렬/줄높이 정합 — z-표 행 압축·열 정렬 어긋남
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task1221` (← `devel`)
**완료일**: 2026-06-01

## 1. 문제

`samples/3-09월_교육_통합_2023.hwp` 4쪽 문26 z-표: z-열 "1.0"/"1.1" 겹침("1.01."), 첫 행 z 비고, P-열과 정렬 어긋남.

## 2. 근본 원인 (계측 확정)

z값은 인라인 수식. 수식-only 셀 문단은 `paragraph_layout.rs` 의 **빈-runs 줄 tac 렌더 블록**에서 tac 를 줄 char 범위 `[char_start, next.char_start)` 로 매핑. 그러나 수식-only 문단은 텍스트가 없어 **모든 LINE_SEG.text_start=0 → 모든 줄 char_start=0**(degenerate) → line0 범위 `[0,0)` 빈 범위(수식 0), line1 `[0,MAX)`가 모든 수식 흡수 → p[0] 의 1.0/1.1 이 같은 줄(line1)에 가로 인접 렌더 → 겹침 + 행1 비고.

> 배제 과정(중요): line-height 클램프(무효)·셀 valign centering(정렬됨)·표준 셀수식 경로(`table_layout:2661`)·분할 셀수식 경로(`table_partial:1095`)·shape 경로 모두 z값 미통과. 실제 경로는 `paragraph_layout` 빈-runs 줄 블록(2번째 수식 렌더).

## 3. 수정

`src/renderer/layout/paragraph_layout.rs` 빈-runs 줄 tac 블록: **모든 줄 빈 runs + char_start degenerate + 줄수==tac수** 이면 tac→줄 매핑을 **순서(index) 1:1** 로 전환. 그 외는 기존 char-범위 유지(동작 불변). 23줄, 단일 블록.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 4쪽 문26 z-표 | 1.0/1.1/1.2/1.3 각 행 분리, P-열 정렬 (한글 2022 PDF 정합) |
| `cargo test --release` | **1896 passed / 0 failed** (svg_snapshot 회귀 0) |
| rustfmt | clean |

## 5. 산출물

- 소스: `src/renderer/layout/paragraph_layout.rs`
- 단계: `mydocs/working/task_m100_1221_stage{1,2_attempt,3,3b,4_fix}.md`
- 최종: 본 문서
