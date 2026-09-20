# Task #7280 Stage 33 — R2ae 일반 문단 최상위 흐름 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2ad](task_m100_7280_stage32.md), 시작 head `562144ad4`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 SHA 검증 예정.

## 1. 책임과 보존 범위

`typeset_paragraph`의 일반 문단 흐름을 `paragraph/flow.rs`로 이동한다.
부모는 기존 시그니처의 진입점을 유지하고, 차용 입력 `ParagraphFlowInput`으로
문단 인덱스·IR·구성 결과·문단 목록·스타일·섹션 말미 여부를 전달한다.
최상위 조정자와 세부 fit/배치/분할 담당자의 책임을 구분하며 알고리즘은 바꾸지 않는다.

- 예산 준비 → rowbreak guide → 다단 배치 → 빈 문단 흡수의 조기 반환 순서를 보존한다.
- 강제 쪽 경계 준비 → 흐름 힌트 → whole-fit → fitted → overflow → failed-fit 순서다.
- 처음 계산한 `available`은 이후에도 그대로 전달하고 중간에 재계산하지 않는다.
- 엔진 profile은 `FnOnce` 조회를 예산 준비의 기존 인자 위치에서 호출한다.
  조정자에 엔진 전체나 Cell 쓰기 권한을 넘기지 않는다.
- state profile은 강제 경계 준비 후 `paragraph_flow_profile` 읽기 전용 getter로 읽는다.
  서로 다른 두 profile을 합치거나 미리 읽지 않는다.
- 수치·판별식·배치/분할·IR·public API·테스트·assertion·baseline·golden·ignore·CI 변경은 없다.

## 2. 검증 계획과 한계

`output/7280/stage33/verify-paragraph-flow.mjs`가 조회/인자 전달을 원래 표현으로 펼쳐
전체 본체·순서·조기 반환·부모 나머지·기존 state/paragraph 담당자의 불변을 대조한다.
정적 대조는 실행 또는 직접 시각 검증을 대체하지 않는다.

별도 review worktree에서 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 이전 절편과 동일한 313건 집중 nextest를 순차 실행한다.
모든 일반 문단 경로의 완전한 실행 커버리지나 한컴 조판 일치를 주장하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM·직접 시각 대조는
구현계획 §7의 책임 묶음/제출 전 통합 게이트에 남는다. 원격 push·PR·댓글은 범위 밖이다.

## 3. 후속

초기 제품 `97eab495bb52512b4af563ed5decbc26208ccdc8` 검증에서
`use super::{self as paragraph, ...}`가 E0432를 일으켜 Clippy가 종료되었다.
import를 절대 모듈 경로로 고쳤으며 수정 head에서 검증을 다시 수행한다.
이 실패는 회귀 테스트 실행 결과가 아니다. 초기 로그는 `clippy-native-initial.log`로 보존한다.

R2 문단/컨트롤 책임 묶음의 잔여 경계와 통합 검증 대상을 점검한다.
표 포맷·분할·이어받기, 각주 등록 본체 및 최종 state 캡슐화는 후속 책임 묶음이다.
