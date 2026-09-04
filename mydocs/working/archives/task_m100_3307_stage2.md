---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3307.md
last_verified: 2026-08-01
---

# Task #3307 Stage 2 보고 — 기본 개요 모양 fallback 구현 + red-check

## 구현

1. `renderer/layout/utils.rs` — `default_outline_numbering()` 신설: 전 수준
   `level_formats = "^N"`, 아라비아(기본 head), 시작 1. 근거(한컴 2020 실측)를
   주석에 명기.
2. `renderer/layout/paragraph_layout.rs` — 번호 합성 지점의 `return None` 두 경로
   (id=0, 정의 조회 실패)를 `checked_sub` 매치로 통합하고, **`HeadType::Outline`
   한정**으로 합성 기본 Numbering fallback. NUMBER/BULLET/None 경로 불변.
   카운터는 기존 `numbering_state.advance(0, …)` 재사용(실정의 id 는 1-based 라
   0 키가 기본 개요 전용으로 충돌 없음).

diff 규모: utils +18줄(주석 포함), paragraph_layout 조건 재구성 ~12줄.

## fixture 효과

p7 전 항목 정답지 일치: **1.~4. 자동번호 복원** + 5.~6. 리터럴 불변. 쪽수 9 불변.

## red-check + 과발동 가드

- 회귀 테스트 `tests/issue_3307_outline_default_numbering.rs`:
  ①쪽수 9 ②p7 에 1.~6. 전부 ③**과발동 가드** — 비개요 문단(p1 "1. 개정이유"
  리터럴)에 이중 번호("1.1.")가 붙지 않음을 단언.
- red-check: fallback 분기 제거 시 "1. 인적사항 … 없다"에서 정확히 FAILED → 복원.
  (교훈: 미커밋 수정 상태에서 `git checkout` 원복으로 수정 자체가 지워져 재적용 —
  red-check 원복은 커밋 후 하거나 별도 백업으로 할 것.)

## 인접 회귀 (focused)

- 번호/개요/자동번호 계열 9개 테스트 파일(doclang_export, #3492 개요 marker,
  #3504 autonumber, #1755 host heading, pr_1136 셀 문단 번호 등): **18 passed**.
- `--lib numbering` 단위: 30 passed. fmt 통과.

## Stage 3 계획

정답지 이미지 스왑(p7) · samples 쪽수 A/B · **신규 fixture 이중 baseline**(IR sweep +
overflow 원장 — 4.3.1 신판 첫 적용) · release-test 전체·clippy·Skia 3종·wasm.
