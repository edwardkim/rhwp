---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1370/README.md
last_verified: 2026-07-16
---

# Task #1370 Stage 1 — A3 발산 카탈로그 (회귀 13건)

> 검증 경로: `dump_page_items` / `build_page_render_tree` / `RHWP_EN_SSOT_DEBUG` (통합테스트 경로).
> A3 `export-svg` CLI(158쪽 폭발)는 사용하지 않음.

## 0. 공통 근본 메커니즘 (전 13건 공통) — **Stage 2 정밀진단으로 정정**

### 0.1 1차 가설(Stage 1)과 그 한계

Stage 1 은 "게이트 임계가 정확 sim 에서 어긋나 break 지점이 밀린다"고 가설했다. Stage 2 정밀진단으로
이 가설은 **부분 정정**된다 — 진짜 원인은 게이트 임계가 아니라 **렌더-높이 자체의 hancom 발산**이다.

### 0.2 정정된 근본 원인 (실증)

A3 는 매 미주 para 누적 후 `st.current_height` 를 **정확 전-단 렌더 sim**
(`simulate_endnote_column_bottom_y` → `measure_endnote_column_bottom`, build_single_column 1회 순차
렌더)으로 **스냅**한다(typeset.rs:3637-3648). 따라서 break/split 게이트가 보는 높이는 per-para 휴리스틱이
아니라 **우리 렌더러의 실제 단 렌더 높이**다.

**문제: 우리 렌더러가 rewind/빈 미주 문단을 hancom 보다 높게 그린다.**

실증 (2022_nov, `measure_endnote_column_bottom` = A2sim):
| 단 | A3 렌더 sim(=used) | hancom(B 통과 시) | 발산 |
|----|-------------------|------------------|------|
| p16 단1 | **1013.4px (overflow>1001.6)** | 932.3px (pi=786 split 여유) | +81px |
| p17 단1 | **1013.7px (overflow)** | 936.5px | +77px |

- `internal_vpos_rewind` para 의 acc 가 A 이상에서 `line_advances_sum`(전체 줄높이; pi=800 169.3)인 반면
  B 는 saved-vpos delta(55.8) — **3배 차이**. 단, acc 는 A2sim 스냅으로 즉시 덮어쓰므로(3647) **최종
  높이는 exact 렌더가 지배**. 즉 exact 렌더 자체가 hancom 보다 ~80px/단 더 높다.
- 결과: A3 단이 hancom 보다 일찍 가득 차 경계 para(pi=786 등)를 split 못 하고 통째로 다음 쪽/단으로
  밀어냄 → 하류 cascade(제목 y 밀림·overflow·분배 어긋남).

### 0.3 비단조 핵심 (메모리 정합 `tech_endnote_overflow_nonmonotonic_gate`)

- `line_advances_sum`(전체 높이): `issue_1082` overflow→0 을 **고침**(렌더 정합), 그러나 13건 hancom
  **compact 배치를 깨뜨림**(과충전).
- saved-vpos delta(compact): 13건 배치엔 가깝지만 `issue_1082` overflow **재발**.
- → 단일 높이모델로 양립 불가. **break-결정 높이를 렌더-높이에서 조건부 디커플**(hancom 의 compact
  overlap 을 모델링)하는 것이 유일한 길. 그 조건을 전 exam 동시 green 으로 찾는 것이 본 타스크 본질.

### 0.4 재보정 방향 (정정)

게이트 임계 미세조정이 아니라, **rewind/빈/TAC 미주 para 의 "break-결정 높이"를 hancom compact 배치에
맞춰 조건부로 재표현**한다. 후보 레버:
1. exact 렌더 sim 을 hancom compact(overlap) 모델로 보정(measure 경로) — 근본적이나 광범위.
2. break/split 게이트 한정 compact 높이 override(렌더는 불변) — B 휴리스틱의 조건부 재도입.
3. (1)+(2) 혼합 — 1082 overflow 안전선만 exact, 그 외 compact.

**비단조** → 전 exam(1082/1139/1189/1209/1284) 동시 green 게이트 필수.

## 1. 13건 발산 표

| # | 테스트 | 샘플 | 쪽 | 단언 종류 | 기대 | A3 실제 | 발산 모드 |
|---|--------|------|----|----------|------|--------|----------|
| 1 | `1139_page17_endnote_question30_starts_on_right_column` | 2022_sep | 17/18 | page-item | pi=928 p17 우단, pi=931 split 0..4(p17)/4..9(p18) | 분배 어긋남 | 늦은 break |
| 2 | `1139_2023_pages12_13_endnote_boundary` | 2023_sep | 12/13 | page-item | q14 title(611) 단0, graph host(613) 단1; tail 635/636 p12, graph 637 p13 | 경계 어긋남 | 경계 시프트 |
| 3 | `1209_2022_nov_p17_split_endnote_titles_keep_gap` | 2022_nov | 17 | y좌표 | q27(787) y 240~256, q28(801) y 204~216 | q27 y=397.5 | 늦은 break(제목 밀림) |
| 4 | `1209_2022_nov_p17_question29_keeps_gap_after_full_para` | 2022_nov | 17 | y좌표 | q29(812) y 806~818 | y=1000.7 (overflow) | advance 미발화 |
| 5 | `1189_2022_nov_p17_internal_rewind_keeps_formula_tail` | 2022_nov | 14/16/17 | page-item | pi=786 첫 줄만 p16, 1..5 p17 | split 위치 어긋남 | rewind split |
| 6 | `1189_2023_page19_question29_tail` | 2023_sep | 19 | page-item | pi=935 split 0..2/2..3, 946·952, 953 split 0..1 | 분배 어긋남 | 늦은 break |
| 7 | `1284_2024_between20_page18_late_question_titles` | 2024_between20 | 17/18 | page-item+y | pi=894 split 0..4(p17)/4..5(p18); q29 398~414, q30 366~386, q23 884~902 | 분배·y 어긋남 | 늦은 break |
| 8 | `1284_2024_between20_page21_question23_title_stays_in_left_tail` | 2024_between20 | 21/22 | y좌표 | q23(1054) 좌단하단 y 1064~1084, body 우단상단 | "문26 tail" 실패 | 경계 시프트 |
| 9 | `1284_2024_between20_page22_23_question_tail` | 2024_between20 | 22/23 | y좌표 | q28 title y ~856.9 | y=980.96 | 늦은 break(제목 밀림) |
| 10 | `1284_2023_sep_page14_question23_title_tail` | 2023_sep | 14/15 | page-item+bbox | q23(759) FullPara 단1 하단 y 1058~1084 | pi=759 FullPara 미발견(split/이월) | 경계 시프트 |
| 11 | `1284_2023_sep_page16_question27_title` | 2023_sep | 16 | bbox | q23(812) y 140~155 | y=799.44 | 늦은 break(상단 못 잡음) |
| 12 | `1284_2023_sep_page19_question29_tail_matches_pdf_frame` | 2023_sep | 19 | y좌표 | 문29 frame 내 | 밀림 | 늦은 break |
| 13 | `1284_2023_sep_page20_question30_title_stays_in_left_tail` | 2023_sep | 20 | page-item | pi=972 title+973/975 좌단, 976 우단 이어짐 | 분배 어긋남 | 경계 시프트 |

## 2. 책임 게이트 후보 매핑 (typeset.rs)

| 게이트 (행) | 역할 | 의심 발산 # |
|-------------|------|------------|
| `a2_overflow_with_para` (2729) — `simulate_endnote_column_bottom_y` > available+tol | 새 para 이어붙인 sim bottom 으로 단 overflow 판정 | 4, 3, 9, 11, 12 (advance 미발화) |
| `split_endnote_to_fit` (2760) + `single_line_tail_split_at_bottom` 필터 (2786) | 단 하단 para 줄단위 split | 5, 6, 7, 1 (split 위치) |
| `advance_large_between_single_line_rewind` (2811, `*0.85`) | 큰 미주사이 마지막 단 하단 첫줄 rewind para 이월 | 5, 12 (rewind) |
| `large_between_tail_render_overflows` (2838, `*0.85`) + `near_bottom_tail` (2893, `*0.90`) | 수식-only tail 뒤 한 줄 풀이 다음 단 이어가기 | 4, 9, 11, 13 (tail) |
| `compact_endnote_own_vpos_span_fits` (2744) | 자체 vpos span 이 남은 공간에 맞으면 split 안 함 | 2, 8, 10 (경계 시프트) |
| `capped_new_endnote_advance`/`stale_forward_vpos` (2654~2666) | advance 추정 캡 | 1, 2 (분배) |

> 정확한 책임 게이트는 Stage 2~4 에서 조건 토글로 실증 확정. 위는 코드 독해+발산 모드 기반 1차 가설.

## 3. 재보정 그룹 (Stage 매핑 확정)

- **그룹 A (Stage 2) — 경계 시프트 / advance 미발화**: #2, #8, #10, #13 + #4.
  핵심 게이트: `a2_overflow_with_para` 발화 임계, `compact_endnote_own_vpos_span_fits`,
  `advance_column_or_new_page` 조건을 sim bottom 기준으로 교정.
- **그룹 B (Stage 3) — split / full-para 간격**: #3, #4(잔여), #6, #1.
  `split_endnote_to_fit` split 위치·`single_line_tail_split_at_bottom` 필터·full-para 뒤 gap.
- **그룹 C (Stage 4) — tail / late titles / rewind**: #5, #7, #9, #11, #12.
  `large_between_tail_render_overflows`(`*0.85`)·`near_bottom_tail`(`*0.90`)·
  `advance_large_between_single_line_rewind`·`internal_rewind_split`.

> #4 는 advance 미발화(A)와 full-para 간격(B) 양쪽에 걸쳐 Stage 2/3 공동 대상. 그룹 경계는 상호결합이
> 강해, 매 Stage 전 exam(1082/1139) 동시 green 게이트로 cascade 를 상시 감시한다.

## 4. 샘플 ↔ 문서 매핑

| 샘플 | 약칭 | 관련 # |
|------|------|--------|
| `samples/3-09월_교육_통합_2022.hwp` | 2022_sep | 1 |
| `samples/3-09월_교육_통합_2023.hwp` | 2023_sep | 2, 6, 10, 11, 12, 13 |
| `samples/3-11월_실전_통합_2022.hwp` | 2022_nov | 3, 4, 5 |
| `samples/3-09월_교육_통합_2024-미주사이20.hwp` | 2024_between20 | 7, 8, 9 |
| `samples/*` (issue_1082 5종) | overflow 게이트 | (회귀 가드) |
