# Task 1274 Stage 14

## 대상

- 최종 `cargo test --tests`에서 실패한 `tests/issue_241.rs`
  - `issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height`

## 관찰

- Stage13 커밋 후 작업 트리는 깨끗했다.
- 최종 단계라 `cargo fmt --check`, `cargo test --tests`를 실행했다.
- `cargo fmt --check`는 통과했다.
- `cargo test --tests`는 `issue_241`의 HWPX 도장 그림 host paragraph 줄 높이 회귀로 실패했다.
- 실패 메시지:
  - stamp y: 887.4
  - pi=10 y: 877.3
  - pi=9 도장 host paragraph가 다음 문단보다 앞서 line advance를 확보해야 한다.

## 초기 가설

- Stage13의 compact 미주 TAC 그림 vpos rewind 허용 조건이 일반 본문 TAC 그림 host까지 열렸을 수 있다.
- `suppress_large_forward_jump`만으로는 모든 호출 지점에서 미주 compact 흐름임을 보장하지 못한다.
- 조건을 미주 흐름으로 더 좁히면 2024-09-between20의 TAC 그림 되감기는 유지하면서 issue_241 본문 도장 host 줄 높이 회귀를 막을 수 있다.
- 아래 추가 검증으로 이 가설은 기각했다.

## 추가 검증

- `RHWP_VPOS_DEBUG=1 cargo test --test issue_241 issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height -- --nocapture`
  - 실패는 재현된다.
  - 출력상 Stage13의 `compact_endnote_tac_picture_rewind` 분기를 타지 않는다.
- 별도 worktree에서 `upstream/devel` (`f4a49682`) 기준 같은 단일 테스트를 실행했다.
  - `cargo test --test issue_241 issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height -- --nocapture`
  - 동일하게 실패한다.

## 판단

- `issue_241` 실패는 Stage13 변경의 회귀가 아니라 현재 `upstream/devel` 기준에서도 재현되는 기존 최종 CI 실패다.
- Task 1274의 여섯 PDF/PNG 1:1 비교 목표와 overflow 정리는 Stage13 검증 결과로 완료 상태다.
- 이번 stage에서는 소스 코드를 수정하지 않고 최종 CI의 기준 브랜치 실패 근거만 남긴다.

## 초기 검증 계획

- `cargo test --test issue_241 -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
- 필요 시 `cargo fmt --check`, `cargo test --tests` 재실행

## 검증 결과

- `cargo fmt --check`: 통과.
- `cargo test --tests`: `tests/issue_241.rs`의 `issue_241_hwpx_stamp_host_paragraph_keeps_flow_line_height` 1건 실패.
- `upstream/devel` 별도 worktree 단일 테스트: 같은 `issue_241` 1건 실패.
