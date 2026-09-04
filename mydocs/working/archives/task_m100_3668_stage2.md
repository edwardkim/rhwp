---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3668.md
last_verified: 2026-08-01
---

# Task #3668 Stage 2 보고 — 원장 게이트 + baseline 확정

## 게이트 구현

`tests/overflow_cell_baseline.rs` — samples 전수(hwp/hwpx)를 `DocumentCore` 공개 API 로
전 페이지 렌더해 문서별 `LAYOUT_OVERFLOW_CELL` 줄 수를 집계. 스레드 스코프 워커
(≤8)로 병렬. 판정:

- baseline 에 없는 문서가 0 이 아니게 되면 **실패** (신규 발생)
- 기록 문서의 수치 **증가**면 실패, 감소·해소는 통과(dump 대조 후 래칫 조임)
- baseline 에 있으나 코퍼스에서 사라진 문서는 실패(원장 부패 감지)
- `RHWP_OVERFLOW_CELL_DUMP=<path>` 로 현재값 TSV dump (4.3.1 절차 준용)

## baseline 실측 (원장 동결)

**샘플 662건(스킵 3 = 암호 fixture) / 0 아닌 문서 22종 / 총 4,896줄 / 2회 결정성 확인.**
`tests/fixtures/overflow_cell_baseline.tsv` 로 동결. 상위 항목:

| 문서 | 줄수 | 메모 |
|---|---:|---|
| basic/issue2007_nested_cell_pagination_42065.hwp | **2,980** | 작업지시자 지목 샘플 — 24쪽 중 6~18쪽 발현, 페이지당 최대 403줄. #2007 중첩 셀 페이지네이션 계열 |
| table_giant_cell_overfill.hwpx / task1718 .hwp | 649 × 2 | 파일명 자체가 이 현상 |
| 2025 행정업무운영 편람 .hwpx / .hwp | 96 / 53 | **#3674(쪽수 386 vs 한컴 383) 단서** — 저장 전후 차이도 관찰 |
| 86712_regulatory_analysis .hwp / issue1891 .hwpx | 66 × 2 | #1891/#2105 계열 fixture |
| 나머지 14종 | 1~49 | rowbreak-problem-pages, #1937 각주 과분할, task2097 등 |

전체 22행은 baseline TSV 참조. **의미**: 이 원장은 곧 "셀 내용이 조용히 사라지는 문서"
목록이다 — #3236 이 외부 리포트로야 잡혔던 계열의 내부 후보 지도.

## 작업지시자 지목 샘플 즉답 (세션 중 질의)

`issue2007_nested_cell_pagination_42065.hwp`: **발현 — 2,980줄**. 새 봉투 카운트와
stderr 진단 건수가 정확히 일치(2,980=2,980)해 카운터의 대규모 교차 검증을 겸했다.

## 실행 시간 — 결정 요청

전수 게이트 **2분07초**(8 워커, release-test). 계획서 §6 의 기준(ir_field_sweep
36초 수준)을 초과하므로 작업지시자 결정 필요:

- **(a) 기본 스위트 편입** (권고) — 이 이슈의 존재 이유가 "아무도 안 보는 진단은
  썩는다"이다. `#[ignore]` 는 같은 문제를 재생산한다. CI 는 8-shard 구조라 2분급
  테스트를 이미 수용한다.
- (b) `#[ignore]` + 수동/CI 별도 잡 — 스위트는 빠르지만 실행 배선을 따로 유지해야
  하고, 배선이 끊기면 무감시로 회귀한다.

> **결정: (a) 기본 스위트 편입** — 작업지시자 승인 (2026-08-01). 현행 구현 그대로
> (`#[ignore]` 없음), 코드 변경 불요.
