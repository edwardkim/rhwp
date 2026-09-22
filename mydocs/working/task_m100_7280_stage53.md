# Task #7280 Stage 53 — R3 책임 묶음 구현·검증

- 이전: [Stage52](task_m100_7280_stage52.md).
- 승인 계획: [구현계획 §4.1](../plans/task_m100_7280_impl.md#41-r3r5-책임-묶음-진행으로-전환).
- 시작 head: `fad75d8e8`, 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 중. 작은 helper별 승인·문서를 반복하지 않고 R3 묶음의 기록을 여기 누적한다.
  R3 종료·R4/R5 완료·최종 제출 게이트 통과를 뜻하지 않는다.

## 책임 분리

- `table/block`: 전체 표 배치 진입 → 분할 준비 → 재개 실행의 조정자.
  `entry`는 기존 조기 배치·반환 순서, `whole_fit`은 상태 변경 없는 전체 수용 조회,
  `prepare`는 분할용 행 기하·각주 예약·호스트 선행 배치와 context 준비를 담당한다.
  `HostPlacementConstraint`는 전체/분할 배치가 공유하던 점유 밴드 제약을 읽기만 한다.
- `table/scan`: 원본 표와 컷용 행 도메인을 유지한다. `runner`가 물리 꼬리 밴드 및
  행 반복을 소유하고, rowspan 블록/일반 행 step은 읽기 전용 페이지 관측과 기존 Query를
  소비해 컷·누적 높이·다음 행을 반환한다. `BlockTableRowScan`과 `BlockRowScanVars`도 이 도메인에 둔다.
- `table/continuation`: 커서·준비 상태·shadow flow의 수명과 기존 crate-visible 재개 API를 소유한다.
  `fragment`는 시작 컷을 한 번 복제한 뒤 예산 준비 → 읽기 전용 scan/refit → emit을 호출한다.
  `emit`이 기존 순서로 항목·각주·커서·다음 쪽을 확정하며, native drain과 WASM job은
  동일한 fragment 구현을 호출한다. facade 재수출로 기존 API와 private 테스트 경로를 유지한다.
- `table/footnotes`: 표 조각에 붙은 각주 큐의 연결을 표 도메인으로 이동했다.
  일반 각주/미주 알고리즘의 전체 책임 정리는 R4이고, TypesetState 외부 직접 쓰기의 최종
  캡슐화는 R5다. 이행 중인 직접 쓰기를 최종 상태 분리 완료로 표시하지 않는다.

기존 조건·숫자·예외·판정 순서·측정 API·테스트 기대값은 변경하지 않는다.
기존 예외를 올바른 조판 규칙으로 새로 승인하거나 결함을 고쳤다는 주장은 하지 않는다.

## 호출·상태 경계 점검

`typeset_block_table_inner → prepare_block_table_entry → prepare_block_table_continuation`
순서다. 조기 배치된 표는 준비/재개로 진입하지 않는다. whole-fit의 저장 프레임 조회 이후에만
기존 흐름 위치를 갱신한다. 준비 완료 후 placeholder로 페이지 상태를 옮기는 시점과
`suspend_before_drain` 반환 시점은 유지한다.

재개 경로는 `context.step → step_block_table_fragment → prepare_table_fragment_budget →
scan_table_fragment → emit_table_fragment`다. 시작/끝 컷과 행 도메인을 재추측하지 않고
기존 scanner 결과를 그대로 받는다. 각주 refit도 동일 scanner를 호출하며, 예산을 변경한
재계산과 실제 예약·배치를 서로 다른 단계로 둔다. 물리 tail 밴드·패딩·반복 제목행·캡션·
각주 예약의 기존 계산 및 마지막 유닛 소비 후 종료 순서를 보존한다.

## 중간 정적·컴파일 검사

- `output/7280/r3-batch/prove.mjs`, `structural-proof.json`: 18개 경계 대조 통과.
  진입/whole-fit/분할 준비, budget/scan/refit/emit, rowspan/일반 행 step,
  native drain, 기존 재개 API 및 표 각주 함수의 계산·분기·명령 본문을 비교했다.
  주석·공백·포맷용 쉼표와 명시된 함수 연결 치환을 제외한다. 새 연결/불변 alias의 수명과
  반복 dispatch는 별도 코드 검토·실행 대상이며 정적 비교가 전체 동등성 증명은 아니다.
- 중간 `cargo check --locked --lib --target-dir target/pr-review` 통과.
  분리 중 누락된 closure 호출 표기와 `fmt` 입력은 컴파일 오류로 검출해 수정했다.
- source-side 테스트·integration 원본·기준값·ignore 변경 없음.
- 고정 head의 focused/정책/lint, 전체 회귀와 Native/fresh Docker WASM 비교는 아직 실행 전이다.

기존 clean review worktree `rhwp-review-7280-r3k`와 공유 target을 재사용한다.
이전 로그/산출물은 보존하며 별도 대용량 worktree를 늘리지 않는다. 원격 push·PR·댓글은 하지 않는다.
