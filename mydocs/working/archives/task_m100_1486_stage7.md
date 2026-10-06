# Task M100 #1486 Stage 7: PR 검증 중 RowBreak 회귀 보정

- 이슈: #1486
- 브랜치: `local/task_m100_1486`
- 작성일: 2026-06-24
- 방법론: Hyper-Waterfall
- 선행 커밋:
  - `515da5d1 task 1486: HWPX 경고 보정 상태 초기화`

## 배경

PR 준비 로컬 검증 중 `cargo test --profile release-test --tests`에서
`tests/issue_1156_rowbreak_fragment_fit.rs`의
`synam_001_page5_splits_large_rowspan_block_like_hancom` 테스트가 실패했다.

실패 내용:

- 샘플: `samples/synam-001.hwp`
- 문단/표: `pi=69`, RowBreak 8x3 표
- 기대: 5쪽 첫 조각이 `end_cut=[2, 2]`
- 실제: 5쪽 첫 조각이 `end_cut=[2, 3]`

## 분석

- Stage 5에서 #1105 회귀를 막기 위해 RowBreak rowspan block의 행 offset 보정을
  `hard-break가 블록 첫 행에 있는 경우`로 좁혔다.
- `synam-001.hwp`의 `pi=69`는 rowspan 보호 블록 안에서 뒤 행의 hard-break를 기준으로
  첫 visible slice를 보존해야 하는 기존 회귀 가드다.
- 실제 실패는 block cut 조건이 아니라 일반 행 컷(`advance_row_cut`)에서 발생했다.
- block cut 경로는 같은 문단 내부 hard-break 직전 유닛을 되감아 orphan slice를 피하지만,
  일반 행 컷에는 이 보호가 없어 `end_cut=[2, 3]`까지 전진했다.
- 되감기 보호를 모든 RowBreak 행에 적용하면 #1486의 마지막 TAC 로고가 다음 쪽으로 밀렸다.
  따라서 적용 범위를 "이전 행에서 시작한 rowspan 셀이 현재 행을 덮는 경우"로 제한했다.

## 수정 내용

- `advance_row_cut`에서 RowBreak hard-break를 만났을 때, 현재 행이 이전 행의 rowspan에
  포함된 행이면 `rewind_rowbreak_orphan_before_hard_break`를 적용한다.
- `row_has_prior_rowspan_cover` 헬퍼를 추가해 일반 RowBreak 행에는 기존 컷을 유지한다.
- 단위 테스트 `test_advance_row_cut_rowbreak_rewinds_internal_hard_break_orphan`를 추가해
  rowspan 내부 행의 same-paragraph hard-break orphan 되감기를 고정했다.

## 검증 계획

- `cargo test --release --lib test_advance_row_cut_rowbreak_rewinds_internal_hard_break_orphan -- --nocapture`
  - 통과
- `cargo test --profile release-test --test issue_1156_rowbreak_fragment_fit -- --nocapture`
  - 통과
- `cargo test --profile release-test --test issue_1105 -- --nocapture`
  - 통과
- `cargo test --release --test issue_1486_hwpx_partial_tac_table -- --nocapture`
  - 통과
- `cargo test --profile release-test --tests`
  - 통과
