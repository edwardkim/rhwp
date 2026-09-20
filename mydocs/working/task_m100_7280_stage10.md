# Task #7280 Stage 10 — R2h 전체 문단 배치 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2g](task_m100_7280_stage9.md), 시작 head `158574ba0`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 후 고정 SHA 검증 준비. R2 전체 완료가 아니다.

## 1. 책임과 보존 범위

진입 fit을 통과한 일반 전체 문단 배치만 `paragraph::place_fitted_paragraph`로 옮긴다.
`placement::defer_preceding_float`는 기존 항목 slice와 IR을 읽어 앞 float의 순서 지연을
판정한다. `state::insert_fitted_paragraph`가 기존 pop/push 순서를 적용한다.
이후 조정자가 advance → 기존 진단 → trimmed spacing 계산을 수행하고,
`state::apply_fitted_paragraph_flow`가 trim → 높이 → underrun → 저장 본문 하단을 반영한다.

항목 삽입을 메트릭 계산 뒤로 옮기지 않는다. 두 메트릭 조회의 단락/다단, stored layout,
trim 복원, 다음 경계, lazy base 조건과 단락마다 관측하는 순서를 보존한다.
기존 FormattedParagraph 계산 결과를 사용하며 fit 높이와 실제 전진량을 합치지 않는다.
PageItem의 실제 layout/paint 소비 경로와 IR/public API는 변경하지 않는다.

fit의 조건, atomic 넘침 허용, inkless tail, 빈 구성 결과 및 분할 문단은 이번 이동 대상이
아니다. 이들은 trim/underrun 갱신 차이가 있으므로 새 command로 일괄 치환하지 않는다.
기존 heuristic을 정당한 조판 규칙으로 승인하거나 버그를 수정하는 절편이 아니다.
테스트 assertion/모듈 ID, baseline, golden, ignore를 변경하지 않는다.

## 2. 검증 계획

`output/7280/stage10/verify-full-placement.mjs`로 순서 Query, 항목 삽입, 흐름 메트릭,
진단과 상태 쓰기를 원래 block으로 복원 비교한다. 남은 root와 분할 조정자의 불변도 검사한다.
공백/후행 쉼표를 제외한 조건·연산 차이를 허용하지 않는다.

고정 review worktree에서 파생 suite 준비, fmt, baseline 대비 manifest/unit-tier 정책,
native Clippy를 순차 수행한 뒤 typeset/composer와 float host/spacing 관련 기존 계약을 실행한다.
검증은 native 집중 계약이며 전체 회귀나 모든 배치 분기의 직접 시각 검증을 대신하지 않는다.
최종 전체 회귀, WASM/workspace lint, workspace build, Native Skia 및 fresh Docker WASM/
시각 대조는 통합 게이트에 남는다. 원격 push·PR·댓글은 수행하지 않는다.

## 3. 결과와 다음 절편

고정 SHA 실행 뒤 결과를 기록한다. 남은 진입 fit·특수 배치, 표 문단/컨트롤 흐름,
나머지 state 직접 쓰기와 규칙/기여자 안내를 계속 분리해야 한다.
