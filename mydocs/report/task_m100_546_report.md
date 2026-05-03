# Task #546 보고서 — exam_science p2 페이지네이션 회귀 (Task #525 로 자연 해소)

## 요약

이슈 #546 ("exam_science.hwp 2페이지 페이지네이션 회귀 — PR #506 머지 후 본문 누락, 4 → 6 페이지") 는 **Task #525 (비-TAC Picture Square wrap 호스트 텍스트 중복 emit 정정) 의 부수 효과로 자연 해소**.

본 task 는 코드 변경 없이 회귀 해소 확인 + 보고 + close.

**상태**: 자연 해소 / close.

## 1. 회귀 확인 — 시점별 측정

| 시점 | commit | exam_science 페이지 | p2 단 0 items / used | 비고 |
|------|--------|---------------------|---------------------|------|
| v0.7.9 (PR #506 직전) | `65e275d` | 4 | 37 / 1133.6 px | 정상 baseline |
| **PR #506 머지 후** | `c7330cf` | **6** | **2 / 132.7 px** | **#546 회귀 origin** |
| Task #524 머지 후 (devel) | `09875a2` 직후 | 6 (예상) | 2 / 132.7 px (예상) | 회귀 잔존 (#524 로 일부 정정 — 그러나 #546 본질 미해소) |
| **Task #525 push 직전** | `2dbbd07` | **6** | **2 / 132.7 px** | **회귀 잔존** (직접 측정 확인) |
| **Task #525 머지 후 (현재)** | `f5ad122` | **4** | **37 / 1133.6 px** | **회귀 해소 ★** (직접 측정 확인) |

`2dbbd07` (origin/devel 마지막 commit, #525 push 직전) 기준 dump-pages 결과:

```
=== 페이지 1 (global_idx=0, section=0, page_num=1) ===
  단 0 (items=31, used=1122.6px)
=== 페이지 2 (global_idx=1, section=0, page_num=2) ===
  단 0 (items=2, used=132.7px)        ← 회귀
=== 페이지 3 (global_idx=2, section=0, page_num=3) ===
  단 0 (items=35, used=1121.4px, ..., diff=+60.7px)
=== 페이지 4 (global_idx=3, section=0, page_num=4) ===
  단 0 (items=44, used=1193.8px, ..., diff=+55.1px)
=== 페이지 5 (global_idx=4, section=0, page_num=5) ===
  단 0 (items=25, used=1064.4px, ..., diff=-83.7px)
```

현재 devel HEAD (`f5ad122`) 기준:

```
=== 페이지 1 ===  단 0 (items=31, used=1122.6px)
=== 페이지 2 ===  단 0 (items=37, used=1133.6px)   ← 정상 (v0.7.9 baseline 정합)
=== 페이지 3 ===  단 0 (items=25, used=1064.4px, ..., diff=-83.7px)
=== 페이지 4 ===  단 0 (items=40, used=1046.7px, ..., diff=-87.7px)
```

**4 페이지 / p2 = 37 items / 1133.6 px** — v0.7.9 baseline 과 정확히 일치.

## 2. 자연 해소 메커니즘

#546 회귀 본질: PR #506 의 Square wrap 어울림 렌더링 (`layout_wrap_around_paras`) 이 비-TAC Picture wrap=Square host paragraph 의 호스트 자기 텍스트를 **두 곳** (`layout.rs:3106 layout_shape_item` + `:3534 layout_column_shapes_pass`) 에서 중복 emit. 이 중복 emit 이:

1. **시각 결함** (글자 중첩, x 위치 distinct) — Task #525 의 보고된 표면 증상
2. **페이지네이션 영향** (호스트 paragraph 의 layout 결과가 모호하여 후속 paragraph 의 위치 산출 불안정) — #546 의 보고된 표면 증상

→ 두 표면 증상의 본질 (root cause) 가 동일. Task #525 의 정정 (Picture Square wrap 의 wrap-around 호출 두 곳 모두 제거, +14 / -69 LOC) 이 두 증상 모두 해소.

## 3. Task #524 와 #525 의 역할 분담

| Task | 본질 | 효과 |
|------|------|------|
| #524 | typeset.rs 의 비-TAC Square wrap 그림 `wrap_around_pic_bottom_px` 산출 시 `body_y` 가 `current_height` (문단 BOTTOM) 사용 → 그림 anchor (vert_align=Top) 가 BOTTOM 으로 산출 → wrap_zone 종료 보정 시 inflation | 페이지 6 → 4 일부 정정 |
| #525 | `layout_wrap_around_paras` 의 호스트 텍스트 중복 emit (Picture Square wrap 케이스) | 시각 결함 + 페이지네이션 본질 정정 (#546 회귀 완전 해소) |

#524 만으로는 #546 회귀 잔존 (origin/devel `2dbbd07` 시점에서 6 페이지 잔존 확인). #525 가 추가로 정정해야 4 페이지 회복.

이는 PR #506 의 회귀가 **두 본질의 결합** 이었음을 의미 — Task #524 가 anchor 위치 본질 1 정정, Task #525 가 wrap-around 호출 본질 2 정정.

## 4. 검증 (코드 변경 없음)

| 검증 | 결과 |
|------|------|
| 현재 devel HEAD `dump-pages` | exam_science 4 페이지, p2 = 37 items / 1133.6 px ✓ |
| Task #525 회귀 검증 (Stage 3) | 7 샘플 170 페이지 168 byte-identical, exam_science 의도 정정만 |
| 회귀 baseline (v0.7.9 `65e275d`) 정합 | 4 페이지 + p2 37 items 일치 ✓ |
| pi=37 ls[0..7] 시각 dup | 0 (Task #525 검증) |

## 5. 종료 조건

- [x] 회귀 해소 확인 (현재 devel HEAD 측정)
- [x] 자연 해소 메커니즘 분석 (#525 본질과의 연결)
- [x] 본 task 코드 변경 0 (Task #525 가 본질 정정 수행)
- [ ] orders 갱신
- [ ] gh issue close 546

## 6. 메모

- 이슈 본문의 bisect 가 PR #506 (어울림 렌더링 도입) 을 회귀 origin 으로 정확히 식별. 본질도 PR #506 의 호스트 텍스트 중복 emit 으로 확정.
- 이슈 본문 진단 절차 ("PR #506 51 commits 중 binary bisect") 는 본 자연 해소로 불필요.
- Task #525 의 보고서 (`mydocs/report/task_m100_525_report.md`) 가 본 회귀의 본질을 상세 다룸.
