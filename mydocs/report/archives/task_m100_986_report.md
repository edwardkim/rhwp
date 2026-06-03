# Task #986 최종 보고서 — 가로 문서 다중 TopAndBottom 표 lane 정합

- 이슈: [#986](https://github.com/edwardkim/rhwp/issues/986)
- PR: [#1051](https://github.com/edwardkim/rhwp/pull/1051)
- 브랜치: `issue-986-landscape-table-flow`
- 기준: `upstream/devel` `bc5683ff`
- 일자: 2026-05-21

## 1. 작업 결과

가로 방향 `receipt.hwp` 에서 한 빈 host paragraph 안의 비글자취급
`TopAndBottom + VertRelTo::Para` 표들이 세로로 밀려 3페이지로 분할되던 문제를
해소했다.

### 변경 파일

- `src/renderer/float_placement.rs`
  - para-relative TopAndBottom float 판정, signed HWPUNIT 해석, horizontal range 계산,
    float lane reservation helper 추가
- `src/renderer/typeset.rs`
  - 동일 빈 host paragraph 안에 TopAndBottom para-float 표가 2개 이상 있을 때
    horizontal lane 단위로 pagination reservation
  - 마지막 빈 paragraph 의 prior trailing drift 흡수 보정
- `src/renderer/layout.rs`
  - render tree 배치도 pagination 과 같은 horizontal lane 규칙으로 정합
- `src/renderer/pagination/engine.rs`
  - fallback Paginator 경로의 마지막 빈 paragraph drift 보정
- `src/renderer/composer.rs`
  - decreasing LINE_SEG text_start 방어. debug overlay 중 underflow panic 차단
- `samples/issue-986-receipt.hwp`
  - 이슈 제보 fixture 추가
- `tests/issue_986.rs`
  - 제보 문서 1페이지 정합과 오른쪽 표 lane 보존 회귀 테스트 추가

## 2. Root Cause

제보 문서는 `pi=0` 안에 비글자취급 TopAndBottom 표 `ci=2..8` 이 여러 개 들어 있다.
왼쪽 큰 표와 오른쪽 작은 표들은 서로 다른 x lane 을 차지하므로 같은 y top 에 놓일 수
있어야 한다.

기존 pagination/layout 은 같은 paragraph 안의 비-TAC 표들을 하나의 전역 vertical cursor
로 누적했다. 그래서 오른쪽 lane 표 `ci=4`, `ci=6` 이 왼쪽 큰 표 `ci=2` 아래로 밀리고,
일부 표가 `PartialTable` 로 잘리면서 문서가 3페이지로 늘어났다.

## 3. Fix

### Lane reservation

`FloatLaneSet` 이 x range overlap 여부로 lane bottom 을 관리한다.

- x range 가 겹치지 않으면 같은 raw top 유지
- x range 가 겹치면 해당 lane bottom 아래로 push
- pagination 과 layout 이 같은 helper 로 horizontal range 를 계산

### 범위 제한

새 lane path 는 다음 조건에만 적용한다.

- host paragraph 에 visible text 없음
- table 이 non-TAC `TopAndBottom`
- vertical relation 이 paragraph 기준
- 같은 host paragraph 안에 해당 para-float table 이 2개 이상

이 제한으로 issue #986 의 다중 표 lane 문제는 해결하면서, issue #157 같은 단일
table-only float 문단의 기존 페이지 분배는 보존했다.

### Trailing empty paragraph

표 lane 수정 후 본문 마지막 빈 문단이 직전 trailing line spacing drift 때문에 단독
빈 페이지로 밀리는 케이스가 남았다. 직전 item 도 빈 paragraph 인 경우에만 prior drift 를
숨김 처리해 `receipt.hwp` 를 1페이지로 유지했다. 기존 `fit_fail_within_safety` 동작은
보존해 visible trailing paragraph 회귀를 피했다.

## 4. 검증

### 정량 결과

`receipt.hwp` 기본 TypesetEngine:

```text
문서 로드: /private/tmp/rhwp-issue-986/receipt.hwp (1페이지)
page 1 items=8
pi=0 ci=2..8 Table
pi=1 FullParagraph "(빈)"
```

`receipt.hwp` fallback Paginator:

```text
RHWP_USE_PAGINATOR=1 ... dump-pages /private/tmp/rhwp-issue-986/receipt.hwp
문서 로드: /private/tmp/rhwp-issue-986/receipt.hwp (1페이지)
page 1 items=8
pi=0 ci=2..8 Table
pi=1 FullParagraph "(빈)"
```

issue #986 bbox:

```text
ci2=[x 37.8..707.4, y 18.9..631.0]
ci4=[x 719.1..886.9, y 18.9..297.1]
ci6=[x 898.1..1084.7, y 18.9..342.7]
```

오른쪽 표 `ci=4`, `ci=6` 이 왼쪽 큰 표 `ci=2` 아래로 밀리지 않고 같은 y top 에서
독립 lane 으로 배치된다.

### 테스트

```bash
cargo fmt --all --check
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --release --lib
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_986 -- --nocapture
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test issue_676_trailing_empty_para --test issue_712 --test issue_713 --test issue_775
TMPDIR=/private/tmp/rhwp-issue-986-rusttmp cargo test --test svg_snapshot
```

결과:

- `cargo fmt --all --check`: 통과
- `cargo test --release --lib`: 1326 passed, 0 failed, 6 ignored
- `issue_986`: 2 passed
- `issue_676_trailing_empty_para`: 3 passed
- `issue_712`, `issue_713`, `issue_775`: 각 1 passed
- `svg_snapshot`: 8 passed

### 시각 검증

작업지시자가 `rhwp-studio` 로 `receipt.hwp` 렌더를 확인했고, 정상으로 판정했다.
시각검증용 WASM/package 및 `rhwp-studio` 임시 sample copy 는 PR 대상에서 제외했다.

## 5. 회귀 방어

- issue #986 fixture 로 page count 1 고정
- `ci=2..8` 이 page 1 에만 렌더되는지 검증
- `ci=4`, `ci=6` 의 x lane 과 y top 을 render tree bbox 로 검증
- issue #157 `svg_snapshot` 로 단일 table-only float 문단 회귀 차단
- issue #676 trailing empty paragraph 세트로 기존 빈 문단 흡수 동작 보존

## 6. 산출물

- `mydocs/plans/task_m100_986.md`
- `mydocs/plans/task_m100_986_impl.md`
- `mydocs/working/task_m100_986_stage1.md`
- `mydocs/working/task_m100_986_stage2.md`
- `mydocs/working/task_m100_986_stage3.md`
- `mydocs/working/task_m100_986_stage4.md`
- `mydocs/working/task_m100_986_stage5.md`
- `mydocs/working/task_m100_986_stage6.md`
- 본 보고서

## 7. PR 방침

컨트리뷰터 워크플로우에 따라 fork remote `origin` 의
`issue-986-landscape-table-flow` 브랜치로 push 하고, 원본 저장소
`edwardkim/rhwp` 의 `devel` 브랜치를 base 로 draft PR #1051 을 생성했다.
이슈 close 는 작업지시자 승인 전에는 수행하지 않는다.
