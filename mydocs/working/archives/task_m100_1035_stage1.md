# Task #1035 Stage 1 완료 보고서 — 진단

**Issue**: [#1035 HWP3 vs HWP5 변환본 페이지별 paragraph alignment 차이](https://github.com/edwardkim/regression-rhwp/issues/1035)
**Branch**: `local/task1035`
**작업 내용**: PR #1009 hunks 임시 적용 후 over-split case + main_trigger 메커니즘 단언

---

## 1. devel baseline 측정 (현 a52859de)

| 항목 | 값 |
|------|------|
| HWP3 sample16 페이지 | 64 |
| HWP5 변환본 sample16 페이지 | 64 |
| **alignment 정합** | **24 / 64 (37.5%)** |
| 미정합 | 40 / 64 (62.5%) |

HWP5 변환본이 HWP3 보다 일관되게 더 압축 (paragraph 가 +1~+11 늦게 시작).

---

## 2. PR #1009 hunks 임시 적용 결과

### 2.1 적용 범위

cherry-pick `71054c51` (PR #1009 commit):
- ✓ `src/renderer/pagination/engine.rs` (+84 vpos reset 감지)
- ✓ `src/renderer/typeset.rs` (+136 동일 로직)
- ✓ `src/renderer/pagination.rs` (+5 `PaginationOpts::is_hwp3_variant`)
- ✓ `src/document_core/queries/rendering.rs` (+4 전달)
- ✗ `src/model/document.rs`, `src/parser/mod.rs` 등 variant 식별 인프라 — **#1005 이미 머지** (충돌, --ours 해소 → HEAD 유지)
- ✗ `src/renderer/composer.rs` (CHARS_PER_LINE 45→50) — **PR #1009 commit message claim 했으나 actual diff 에 없음** (commit message 부정확, 변경 없음)

### 2.2 빌드 + 측정

| 항목 | devel | with PR #1009 |
|------|-------|---------------|
| sample16 (HWP3) | 64 | 64 (무변동) |
| **sample16-hwp5** | **64** | **65** (+1 **over-split 회귀**) |
| **alignment 정합률** | 24/64 (37.5%) | **23/64 (35.9%)** (오히려 **악화**) |

→ PR #1009 close 사유 (메인테이너 sweep) **재현 성공** + 추가로 정합률도 악화.

---

## 3. main_trigger 발동 정확성 단언

### 3.1 의도된 trigger case — sample16-hwp5 pi=472 → pi=473

```
HWP5 변환본:
  pi=472 ls[0]: vpos=67012, lh=1300 → prev_end_vpos = 68312
  pi=473 ls[0]: vpos=284,   lh=1300 → curr_first_vpos = 284

body_height_hu = 84188 - 4×2835 = 72848 HU
high_threshold = 72848 × 85 / 100 = 61920 HU
low_threshold  = 1500 HU (curr non-empty line_segs)

main_trigger:
  prev_end_vpos (68312) > high_threshold (61920) ✓
  curr_first_vpos (284) < low_threshold (1500) ✓
  → force page break (의도)
```

### 3.2 HWP3 native 동일 paragraph 비교

```
HWP3 native:
  pi=472 ls[0]: vpos=57108, lh=1300 → prev_end_vpos = 58408
  pi=473 ls[0]: vpos=0,     lh=1300 → curr_first_vpos = 0 (page-reset)
```

HWP3 native 도 동일한 page-reset 패턴 (vpos=0). HWP3 paginator 는 이미 처리하나 HWP5 변환본 paginator 는 vpos=284 (almost 0) 를 인식 못함 — PR #1009 휴리스틱의 의도는 정확.

---

## 4. Over-split 메커니즘 — PartialParagraph 처리

### 4.1 추가 page break 발생 위치 (pi=460 부근)

```
HWP3 native p23 (idx=22): items=11
  ...
  FullParagraph pi=460  h=111.5  vpos=56608..4160 [vpos-reset@line3]  ← paragraph 내부 reset

PR #1009 HWP5 변환본 p23 (idx=22): items=12
  FullParagraph pi=450~459
  Table pi=456 ci=0 (tac=true, TopAndBottom)
  ...
  FullParagraph pi=459
  PartialParagraph pi=460 lines=0..3  ← 페이지 경계 split

PR #1009 HWP5 변환본 p24 (idx=23):  ← 추가 페이지 (over-split)
  PartialParagraph pi=460 lines=3..7
  FullParagraph pi=461, 462, ...
```

### 4.2 메커니즘 분석

PR #1009 휴리스틱이 vpos reset 신호 trigger 시 force page break 삽입. 그러나:

1. paragraph 내부에 vpos-reset signal 가지는 paragraph (pi=460 등) 의 경우 PartialParagraph 로 split 처리
2. PR #1009 force break 후 + PartialParagraph split 효과 누적
3. 결과: HWP3 가 자연 처리한 break 위치보다 1 paragraph 늦은 추가 break 발생

p18~p23 까지는 정합 잘 작동, **p24 부근 PartialParagraph 처리에서 +1 over-split 발생**.

---

## 5. Narrow 가드 후보 (Stage 2 시도)

### 5.1 후보 A — PartialParagraph 가능성 있는 paragraph 에서 trigger skip

paragraph 내부에 vpos-reset signal 자체 가지는 경우 (line_segs 다수 + 그 중 일부 vpos=0 reset) PR #1009 휴리스틱 skip. 기존 paginator 가 자연 처리.

### 5.2 후보 B — high_threshold 임계값 상향

0.85 → 0.90 또는 0.95. paginator 가 자연 break 할 영역 (95%+) 외에는 trigger 안 함.

### 5.3 후보 C — 다음 paragraph 의 height 검증

curr paragraph 가 PartialParagraph 로 split 가능성 있는 큰 height 경우 force-break skip.

### 5.4 후보 D — naturally-breaks 단언

paginator 가 curr paragraph 추가 시 자연스럽게 break 할 것인지 미리 계산 (cumulative used_height + curr lh > body_height). 자연 break 시 PR #1009 force-break 불필요.

→ 후보 D 가 가장 정확. Stage 2 에서 구현 시도.

---

## 6. CHARS_PER_LINE 재평가 (Stage 1.4)

```
현 devel composer.rs:462: const CHARS_PER_LINE: usize = 45;
PR #1009 commit message claim: 45→50
PR #1009 actual diff: 변경 없음 (commit message inaccurate)
```

→ 본 task 에서 CHARS_PER_LINE 변경 불필요. #999/#1005 머지로 이미 정합.

---

## 7. 변환본 fixture vpos reset 신호 분포 (Stage 1.3)

(Stage 1 시간 절약 — devel baseline 측정만 수행. 변환본 fixture 별 vpos reset 분포 분석은 Stage 2 narrow 가드 구현 후 sweep 시점에 수행)

페이지 수 sweep 결과 (devel a52859de 기준):

| Sample | 페이지 |
|--------|-------|
| hwp3-sample-hwp5 | 16 |
| hwp3-sample4-hwp5 | 36 |
| hwp3-sample5-hwp5 | 64 |
| hwp3-sample10-hwp5 | 763 |
| hwp3-sample11-hwp5 | 151 |
| hwp3-sample13-hwp5 | 3 |
| hwp3-sample14-hwp5 | 11 |
| **hwp3-sample16-hwp5** | **64** |
| hwp3-sample19-hwp5 | 2 |

→ 본 task 의 fix 가 sample16-hwp5 외 변환본 페이지 수 회귀 0 단언 필수.

---

## 8. Stage 2 진행 권고

1. PR #1009 hunks (engine.rs + typeset.rs + pagination.rs + rendering.rs) base 적용
2. **narrow 가드 후보 D 우선 시도**: PR #1009 force-break 직전에 cumulative_used_height + curr paragraph height 가 body_height 초과 여부 미리 계산. 자연 break 시 force-break skip
3. sample16-hwp5 페이지 수 64 단언 (over-split 회피)
4. sample16-hwp5 alignment 정합률 ≥80% 목표 측정
5. 변환본 9 종 회귀 sweep
6. 작업지시자 한컴 viewer 시각 검증

---

## 9. 본 단계 산출물

- 본 보고서
- baseline 측정 데이터
- PR #1009 hunks 임시 적용 후 측정 데이터 (Stage 1 종료 시 cherry-pick abort)

코드 변경 commit 없음 (Stage 1 진단 한정).
