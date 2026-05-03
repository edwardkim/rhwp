# Task #526 Stage 3 — 회귀 검증

## 요약

- **단위 테스트**: `cargo test --lib --release` 1111 passed / 0 failed (기존 테스트 무회귀).
- **Clippy**: `cargo clippy --release --lib -- -D warnings` warning 0.
- **SVG byte 회귀**: 7 샘플 170 페이지 중 167 페이지 byte-identical, 3 페이지 변경 (exam_science_002/003/004). **변경된 3 페이지 모두 Stage 1 §5 에서 식별한 영향 문단(pi=61, 79, 110, 118, 120) 의 의도된 정정**. 다른 페이지/샘플 회귀 0 건.
- **Stack 해소**: 영향 5개 문단의 인라인 수식 stack 인스턴스 합 33개 → 0개. 모두 distinct 좌표로 분산.

## 1. 검증 게이트 결과

| 게이트 | 명령 | 결과 |
|--------|------|------|
| 빌드 | `cargo build --release` | ✓ Finished in 12.71s |
| 단위 테스트 | `cargo test --lib --release` | ✓ 1111 passed; 0 failed; 1 ignored |
| Clippy | `cargo clippy --release --lib -- -D warnings` | ✓ Finished, warning 0 |
| 회귀 검증 | `scripts/svg_regression_diff.sh build HEAD~1 HEAD` | ✓ 167/170 byte-identical, 의도된 정정만 3페이지 |

## 2. SVG 회귀 검증 상세

`scripts/svg_regression_diff.sh build 6b4c949 8e07672` (HEAD~1=구현 계획서 commit, HEAD=Stage 2 commit)

```
2010-01-06:    total=6  same=6  diff=0
aift:          total=77 same=77 diff=0
exam_eng:      total=8  same=8  diff=0
exam_kor:      total=20 same=20 diff=0
exam_math:     total=20 same=20 diff=0
exam_science:  total=4  same=1  diff=3  diff_pages=[exam_science_002.svg exam_science_003.svg exam_science_004.svg]
synam-001:     total=35 same=35 diff=0
---
TOTAL: pages=170 same=167 diff=3
```

**의도된 정정 페이지** (3건):

| 페이지 | 영향 문단 | 영향 내용 |
|--------|----------|-----------|
| `exam_science_002.svg` | pi=61 (12번 문제 후속) | 9 수식 stack 해소 |
| `exam_science_003.svg` | pi=79, pi=110 | 각 9, 7 수식 stack 해소 |
| `exam_science_004.svg` | pi=118, pi=120 | 각 8, 1 수식 stack 해소 |

**다른 샘플 회귀 0 건** — 표만 있는 단락은 `inline_tac_controls` 에 표만 들어가 기존 동작과 동일, segments-control 1:1 정렬 유지.

## 3. Stack 해소 비교 (좌표별 ≥4 인스턴스)

`grep -oE '<g transform="translate\([0-9.]+,[0-9.]+\)'` 로 동일 좌표 클러스터 카운트.

### exam_science_002.svg (페이지 2)

| 좌표 | BEFORE | AFTER |
|------|--------|-------|
| `(534.8, 1206.91)` | 9 ← pi=61 stack | 0 ✓ |

### exam_science_003.svg (페이지 3)

| 좌표 | BEFORE | AFTER |
|------|--------|-------|
| `(534.8, 387.52)` | 9 ← pi=79 stack | 0 ✓ |

### exam_science_004.svg (페이지 4)

| 좌표 | BEFORE | AFTER |
|------|--------|-------|
| `(534.8, 422.13)` | 8 ← pi=118 stack | 0 ✓ |
| `(70.67, 1169.43)` | 7 ← pi=110 stack | 0 ✓ |

**합계**: 33개 stack 인스턴스 → 0개. 모든 영향 문단의 수식이 distinct (gx, gy) 좌표로 분산.

## 4. 신규 좌표 분포 검증 (pi=61 예시)

`exam_science_002.svg` AFTER 의 1100~1250 y범위 수식 좌표 (pi=61 영역):

```
560.87, 1179.04
570.87, 1215.21
581.87, 1157.57
689.87, 1179.04
...
```

- `gy=1179.04` ≈ ls[1] (vpos=77442 기반 baseline) — 첫 텍스트 줄 인라인 수식
- `gy=1215.21` ≈ ls[2] (vpos=79052 기반 baseline) — 둘째 텍스트 줄 인라인 수식

수식들이 두 텍스트 줄 사이에 분산 배치됨. 구현 계획서 Stage 2 완료 기준 충족.

## 5. 디버그 로그 검증 (pi=61)

```
LAYOUT_INLINE_TABLE_PARA: pi=61 sec=0 col_x=534.8 col_w=422.6
                          ls_count=3 tables=1 equations=9
  LAYOUT_INLINE_TAC[0]: ctrl_idx=0 kind=Table rows=2 cols=1 w=14745 h=2864 wrap=TopAndBottom
  LAYOUT_INLINE_TAC[1]: ctrl_idx=1 kind=Equation w=675  h=1125 script="rmX"
  LAYOUT_INLINE_TAC[2]: ctrl_idx=2 kind=Equation w=825  h=1125 script="rmA"
  LAYOUT_INLINE_TAC[3]: ctrl_idx=3 kind=Equation w=675  h=1125 script="rmB"
  LAYOUT_INLINE_TAC[4]: ctrl_idx=4 kind=Equation w=675  h=1125 script="rmC"
  LAYOUT_INLINE_TAC[5]: ctrl_idx=5 kind=Equation w=750  h=1125 script="rmD"
  LAYOUT_INLINE_TAC[6]: ctrl_idx=6 kind=Equation w=2558 h=1125 script="m-4"
  LAYOUT_INLINE_TAC[7]: ctrl_idx=7 kind=Equation w=2558 h=1125 script="m-2"
  LAYOUT_INLINE_TAC[8]: ctrl_idx=8 kind=Equation w=2558 h=1125 script="m+2"
  LAYOUT_INLINE_TAC[9]: ctrl_idx=9 kind=Equation w=2558 h=1125 script="m+4"
```

`equations=9` 가 정확히 인식됨, ctrl_idx 1~9 의 수식 폭/스크립트 모두 정상 추출.

## 6. 시각 정합 (작업지시자 판정 필요)

다음 SVG 출력 수동 비교 필요 (한컴 PDF 와 정합):
- `/tmp/svg_diff_after/exam_science/exam_science_002.svg` — 12번 문제 (pi=61, 9 수식)
- `/tmp/svg_diff_after/exam_science/exam_science_003.svg` — pi=79 (9), pi=110 (7)
- `/tmp/svg_diff_after/exam_science/exam_science_004.svg` — pi=118 (8), pi=120 (1)

`/tmp/svg_diff_before/` 와 비교하여 인라인 수식이 텍스트 흐름에 정합한지 확인.

## 7. 결론 — 완료 기준 충족

수행 계획서 Stage 3 완료 기준: "다른 샘플 byte-identical, exam_science 변경은 의도된 정정만." — 충족.

- 6 샘플 byte-identical
- exam_science 3 페이지 변경 = 영향 5개 문단의 의도된 정정
- 회귀 0 건
- 단위 테스트 1111 pass / Clippy warning 0

다음 단계 Stage 4 (최종 보고서 + orders 갱신 + merge + close) 진행 가능.

---

승인 요청: 회귀 검증 결과 + 시각 정합 검증 + Stage 4 진행 가능 여부.
