# Task #7280 Stage 18 — R2p 저장 TAC 표 문단 경로 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2o](task_m100_7280_stage17.md), 시작 head `eae87f9be`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 후 고정 SHA 집중 검증 준비. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

표 문단의 기존 `진입 진단 → float 배제 영역 소비 → 호스트 문단 구성` 뒤에 위치한
저장 TAC 줄 수용 경로만 분리한다. 일반 TAC 카운트/pre-flush, float 및 지연 배치,
표 행/셀 컷과 continuation은 이번 절편에서 변경하지 않는다.

- `controls/stored_tac.rs`: profile/side-wrap gate, 기존 composer의 줄 소속 조회,
  흐름·저장 원점의 max 선택, 선언 높이와 실측 높이 대조, 가용 높이 fit 및 표별 배치 계산.
- `controls::try_place_stored_tac_paragraph`: 계획 수용 시 표별 계산·확정을 원래 순서로 수행.
  불수용은 기존 일반 컨트롤 경로로, 수용은 기존 문단 조기 반환으로 연결한다.
- `state`: 좁은 페이지 관측값 제공 및 확정 metadata → PageItem → current_height 반영.

`composer::stored_tac_lines`가 서로 다른 물리 줄의 표만 수용하는 기존 계약을 재사용한다.
TAC라는 이유로 모든 표를 같은 줄로 보거나 높이를 합산하지 않는다. 편집 세션, dirty/합성 저장 줄,
같은 줄의 복수 표, 다른 컨트롤과의 혼재를 다루는 기존 수용/배제 조건은 바꾸지 않는다.
기존 `measured_fits` 검사 뒤에 `fits`를 **별도로** 실행하며, 앞 검사가 false라는 이유로
뒤 검사를 생략하지 않는다. `available_height`의 환경 변수 기반 진단과 호출 횟수를 유지하려고
기존 `all` 내부에서 closure로 조회한다. 계산이 끝나기 전에는 상태를 쓰지 않는다.

### 좌표·높이 소비 경로

`composer::StoredTacLine(top/end/occupied_end)` → `stored_tac::prepare`의
`max(flow_origin, saved_origin)` 및 `max(occupied_end, end)` fit →
`StoredTacPlan::placement`의 top/advance_end → `state::commit_stored_tac_control`의
`inline_placements`와 `current_height` → 기존 LayoutEngine의 inline metadata 소비.
composer/측정/LayoutEngine/paint 소비자는 변경하지 않는다. 마지막 소유 표에만 문단 아래 간격을
더하는 순서와 음수 저장 간격을 그대로 유지한다. 표별 계산과 확정은 계속 교대로 수행한다.

이번 작업은 기존 동작의 구조 분리이며 조판 오류 수정이나 기존 heuristic의 타당성 승인이 아니다.
IR/API·테스트 source/assertion/ID·baseline/golden/ignore·CI 정책은 변경하지 않는다.
controls의 나머지 경로와 최종 상태 캡슐화는 후속 이행 항목이다.

## 2. 검증 계획

`output/7280/stage18/verify-stored-tac.mjs`로 기존 parent 분기와 query/계획/상태 명령을
복원 비교한다. 수용 조건, 두 all의 평가 순서, 원점/끝점 산식, 항목/상태 쓰기 순서와
나머지 parent·기존 state 명령·테스트의 불변을 확인한다.

별도 review worktree에서 파생 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 실행한다. typeset/composer/float_placement와 기존
#7103, #7150, #6601, #6812, maintainer_nested_table_lines 계약을 선택한다.
원본 저장 문서의 좌표 계약과 수동 변경한 입력의 합성 경계 계약은 구분한다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM/직접 시각 대조는
통합 게이트에 남긴다. 이번 절편은 원격 push·PR·댓글이나 시각 통과 선언을 포함하지 않는다.
