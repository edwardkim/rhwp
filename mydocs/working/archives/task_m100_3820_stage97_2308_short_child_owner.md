---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 97 — #2308 native RowBreak short-child owner 경계

## 전 stage 인계

Stage 96 commit `27edbca77`은 86712 p28--p29의 1×1 mixed nested-cell fragment 경계와 SVG
local font fallback을 처리했다. 그 변경의 focused gate는 #2097 1건, #1921 5건, #2279 4건을
통과했다. 이후 전체 `release-test`가 멈춘 #2308 p81/p82 failure는 Stage 96 전 `HEAD` archive에서도
동일하게 재현되었으므로, 새 회귀로 오인하지 않고 별도 fidelity 결함으로 다룬다.

## PDF oracle과 재현

기준은 `samples/issue1891/76076_regulatory_analysis-2024.pdf`의 p81--p82다.

- PDF p81은 `구내운반차 안전조치를 통해 근로자와 부딪히는 등의 사고`까지 그린다.
- PDF p82는 단어를 중복하거나 반으로 자르지 않고 `를 예방함으로써 산업재해 감소`로 이어진다.
- 처음에는 `issue_2308_short_rowbreak_child_uses_owner_content_box_only`가 p81 첫 assertion에서
  실패했고, export-text도 목표 줄 전체를 p82에서 시작했다.

따라서 test baseline을 낮추거나 page count만 맞추는 해결은 허용하지 않는다.

## 현재 미커밋 보정의 구조 판정

현재 worktree의 `height_measurer.rs` 보정은 일반 마지막 nested tail의 64px/85% small-drift
guard는 그대로 두고, parent stored height보다 큰 **native short child**에만 큰 tail fit을
허용한다. 이에 앞서 pi=831의 50px small-drift가 다시 적용되어 p81에 p842의 잔여 viewport가
생기며, p842의 구조 예외가 첫 줄을 p81에 남긴다. 이 상태로 #2308 focused gate는 통과했다.

초기 진단에는 `구내운반차 안전조치` 문자열을 선택자로 쓴 흔적이 있다. 그 문자열은 동작 조건이
아니고 log 범위를 좁히기 위한 것뿐이지만, source에 fixture 본문 의존성을 남기지 않도록 구조
predicate 기반 generic diagnostic으로 교체한다.

원인은 text가 아니라 다음 구조로 표현해야 한다.

1. native HWP5 RowBreak parent의 마지막 row
2. text 없는 single-control host와 reset-only trailing paragraph
3. non-TAC 1×1 child, parent content box보다 큰 stored child height
4. 해당 row가 잔여 viewport에서 실제 fragmentable row-cut 경로를 타는지

## 다음 분석·수정 순서

1. fixture 문자열을 쓰는 diagnostic filter를 구조 predicate로 바꾼다.
2. #2308의 다섯 oracle, #2097, #1921, #2279 및 overflow-cell baseline을 다시 통과시킨다.
3. p81--p82 PDF 직접 비교와 wasm 사용자 검증을 별도로 남긴다.

## 보존해야 하는 반례

- 76076 p34 direct-benefit nested-table body와 우측 border
- 21217935의 8쪽 page count (#2097)
- 59043 p35의 PDF-owned intro fragment (#1921)
- 86712 p28의 3×12 표 조기 paint 방지 (#2279)

Stage 97은 위 구조 증거 없이 전역 RowBreak tolerance, font metric, 일반 stale-height 조건을
완화하지 않는다.

## 적용·검증 결과

- `fit_measured_table_nested_tail_to_declared_height`는 마지막 empty-host non-TAC 1×1 child의
  small-drift fit을 문단 수와 무관하게 보존하고, 큰 fit은 child가 3문단 이하이며 parent stored
  height보다 큰 short-child일 때만 허용한다.
- `issue_2308_render_normalized_derived_state`: 5 passed.
- #2097: 1 passed, #1921: 5 passed, #2279: 4 passed.
- `overflow_cell_baseline`: 678 samples (3 skipped), non-zero 17 documents, total 688 lines;
  baseline increase 없이 1 passed.
- `cargo fmt --check`와 diff whitespace check를 통과했다.

`stage97-issue2308-owner-after` visual sweep은 p81/p82 모두 structural flag 0,
frame 밖 ink 0, content-bottom delta 2px를 기록했다. review PNG와 SVG font-style 증적은
`mydocs/pr/assets/task_m100_3820_stage97_2308_short_child_owner/after/`에 보관한다. SVG에는
Stage 96의 `한양중고딕`/`휴먼명조` local font fallback도 함께 포함된다.

## 전체 회귀의 별도 baseline

전체 `cargo test --profile release-test --tests`는 `issue_4138_split_cell_stale_linesegs`의
두 page-count assertion(191 vs expected 195)에서 멈췄다. 변경 없는 Stage 96 commit `27edbca77`을
독립 target에서 같은 single test로 실행해도 정확히 같은 두 failure가 재현됐다. 따라서 이것은
Stage 97 보정이 도입한 회귀가 아니다. #4138의 195쪽 기대가 기준 PDF와 여전히 맞는지는 다음
stage에서 PDF와 edit 결과를 직접 대조해 판정하며, 이 stage에서 숫자 baseline을 임의로 바꾸지 않는다.
