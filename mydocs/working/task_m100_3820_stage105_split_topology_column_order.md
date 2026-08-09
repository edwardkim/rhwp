---
kind: investigation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-10
---

# Task #3820 Stage 105 — 저장본 셀 분할 topology의 열 순서 한정

## 범위와 시작 상태

- 브랜치: `task/3820-production-fidelity`
- 시작 commit: `61d9a71da`
- 대상: `src/renderer/typeset.rs`의
  `is_reparsed_single_column_cell_split_row`
- 저장 경계 회귀: `tests/issue_4138_split_cell_stale_linesegs.rs`
- 선행 근거:
  `mydocs/working/task_m100_3820_stage98_4138_split_cell_page_count_oracle.md`

Stage 98은 native HWP의 `dirty_flag`가 저장 경계를 넘지 못하는 조건에서, 저장·재파싱
뒤에도 남는 `split_cell_into(1×2)` 구조만 continuation strict-cut의 근거로 사용했다.
현재 작업트리에는 이 판별을 **원문 셀 col 0 / 빈 템플릿 셀 col 1** 순서로 더 좁히는
소스 변경과 음성 단위 회귀가 미커밋 상태로 남아 있다. 또한 #4138 저장·재파싱 fixture가
원문 문단의 기존 instanceId와 새 빈 peer의 zeroed instanceId를 직접 고정하는 회귀도
추가돼 있다. 이 stage는 그 provenance 보정만 다루며, 정책연구 PDF의 다음 시각 후보
판정은 후속 stage로 분리한다.

## 구현 전 문제 분석

기존 helper는 같은 폭·높이·border·padding의 1×1 셀 두 개, 하나의 실제 본문 셀, 하나의
`new_from_template`형 빈 셀, 다른 행의 전폭 span을 확인한다. 그러나 본문 셀과 빈 셀을
각각 찾은 뒤 **두 셀의 좌우 순서까지는 계약에 포함하지 않는다**. 따라서 빈 셀이 col 0,
본문 셀이 col 1인 역방향 2열 행도 나머지 clone 서명만 우연히 충족하면 strict-cut의
양성으로 판정될 수 있다.

`Table::split_cell_into`의 실제 1열→2열 동작은 이 역방향 형상을 만들지 않는다.

1. 원래 셀은 `primary`로 유지되며 기존 `target_col == 0`에서 폭만 줄어든다.
2. 새 짝 셀은 `ci == 1`에서 `Cell::new_from_template`로 생성되어 col 1에 놓인다.
3. 원문 문단은 원래 셀에 남고, 새 우측 셀만 템플릿 빈 문단을 가진다.

그러므로 열 순서는 단순한 휴리스틱 추가가 아니라, Stage 98이 의도한 편집 provenance를
완성하는 필요조건이다.

## 가설

helper에 다음 조건을 함께 요구하면 실제 `split_cell_into(1×2)` 저장본은 유지하면서,
빈 왼쪽/본문 오른쪽의 자연 2열 행을 strict-cut 대상에서 제외할 수 있다.

```text
content_cell.col == 0
template_cell.col == 1
```

양성 fixture는 원문 셀이 col 0이고 zeroed-template 셀이 col 1이므로 계속 통과해야 한다.
같은 fixture에서 두 셀의 col만 맞바꾼 음성 fixture는 실패해야 한다.

## 최소 수정 범위

1. `is_reparsed_single_column_cell_split_row`에서 content/template 셀을 확정한 직후 열 순서를
   검사한다.
2. 기존 helper 단위 테스트에 빈 왼쪽/본문 오른쪽 역방향 반례를 추가한다.
3. #4138 저장→재파싱 fixture에서 col 0 원문 문단의 instanceId는 non-zero, col 1 빈 peer의
   instanceId는 zero임을 두 분할 API 경로에서 직접 고정한다.
4. 페이지 높이, 줄바꿈, 표 fragment spacing, caption, 각주 owner 및 기존 fixture 기대값은
   변경하지 않는다.

## 불변 조건

- `issue_4138_split_cell_stale_linesegs` 저장·재파싱 결과 197쪽과 stale/vpos 계약을
  보존한다.
- 원본 issue1949 giant-cell 115쪽, #2430 셀 재래핑 임계, #2097 한컴 oracle 2쪽을
  보존한다.
- Stage 98의 `has_para_text`, zeroed instanceId clone, char shape/line clone 및 다른 행
  전폭 span 조건은 완화하지 않는다.
- 저장 경계 회귀는 모델 내부 상태가 아니라 실제 native HWP export→재파싱 결과를
  검사한다.
- 이번 stage에서는 열 순서 외의 새로운 렌더러 휴리스틱을 추가하지 않는다.

## 검증 계획

공통 환경은 `CARGO_TARGET_DIR=target/pr-review CARGO_INCREMENTAL=0`, profile은
`release-test`로 한다. 코드 보정 뒤 사용자가 지정한 순서대로 #2430을 먼저 실행한다.

1. `issue_2430_cell_rewrap_threshold` 2건
2. `reparsed_single_column_split_topology_is_narrow` helper 단위 테스트
3. `issue_4138_split_cell_stale_linesegs` 2건, 저장·재파싱 197쪽 및 원문/빈 peer의
   non-zero/zero instanceId provenance
4. `issue_2097_bottom_squeeze_page_pins`
5. #2424 원본 페이지 고정 회귀
6. `cargo fmt --all -- --check`, `git diff --check`
7. `cargo clippy --all-targets -- -D warnings`

## 검증 결과

2026-08-10에 `CARGO_TARGET_DIR=target/pr-review`, `CARGO_INCREMENTAL=0`,
`release-test` profile로 실행했다.

- `issue_2430_cell_rewrap_threshold`: 2/2 passed. 수정 후 첫 게이트로 실행했다.
- `reparsed_single_column_split_topology_is_narrow`: 1/1 passed. 빈 왼쪽/본문 오른쪽
  역방향 행을 음성으로 고정했다.
- `issue_4138_split_cell_stale_linesegs`: 2/2 passed. 두 편집 API 경로 모두 native HWP
  저장→재파싱 후 197쪽을 유지하고, col 0 원문 instanceId non-zero / col 1 빈
  peer instanceId zero를 직접 단언했다.
- `issue_2097_bottom_squeeze_page_pins`: 1/1 passed. 일반 2열 표의 2쪽 한컴
  oracle을 유지했다.
- `issue_1858`: 1/1 passed. 빈 왼쪽/본문 오른쪽 자연 표를 포함한 1쪽 pin을
  유지했다.
- `issue2424_resumable_pagination_commits_only_after_final_fragment`: 1/1 passed,
  원본 115쪽을 유지했다.
- `cargo fmt --all -- --check`, `git diff --check`,
  `cargo clippy --profile release-test --all-targets -- -D warnings`: 모두 exit 0.

## 판정

열 순서와 저장본 instanceId는 `split_cell_into(1×2)`가 실제로 만드는 오른쪽 빈
peer를 식별하는 provenance이다. 이를 요구해도 #4138의 197쪽은 유지됐고,
#2097·#1858의 자연 표는 침범하지 않았다. 따라서 이 stage의 최소 보정을
수용하고, 정책연구 215쪽의 남은 시각 후보 직접 대조는 다음 stage에서 계속한다.
