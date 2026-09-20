# Task #7280 Stage 5 — R2c inline 계획 조회와 상태 반영

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2b](task_m100_7280_stage4.md), 시작 head `7262bfa7e`.
- 상태: 구현 완료, 고정 제품 SHA 검증 대기. R2 전체 완료가 아니다.
- 제품 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db` 유지.

## 1. 책임 분리와 보존 범위

기존 `typeset/inline_flow.rs`의 build closure 계산을 `inline_flow/plan.rs`로 분리한다.
`build_plan`은 읽기 전용 `InlineFlowInput`, 문단, 스타일, 측정 표, dpi만 받는다.
TypesetEngine/TypesetState 또는 가변 페이지 상태를 받지 않으며 기존 renderer inline planner를
재사용한다. 별도 규칙 엔진이나 새로운 줄 나눔 알고리즘을 만들지 않는다.

`state.rs`는 현재 단의 조회, 참조 입력 제공, fit된 계획의 5개 상태 변경을 소유한다.
배제 영역의 복사는 이전과 같이 스타일 조회 성공 후에만 수행한다. 입력 생성은 단 기하와
페이지 크기를 읽고 참조만 보관하며, 계산 중 원래 상태에 대한 쓰기/콜백은 없다.
현재 단의 기하 조회를 사용하는 다른 호출자도 같은 메서드를 사용한다.

상위 inline 흐름 조정자는 기존 eligibility → 계획 → carved 확인 → fit → 필요 시 다음 단
후보 → 단 전진 → 실제 단 재계산 → 확정 순서를 보존한다. 특히 전진 후 재계산 실패 시
이미 전진한 상태로 기존 문단 분할기에 fallback하는 동작도 변경하지 않는다.
이는 기존 동작의 보존이지 해당 fallback의 타당성을 새로 승인한 것이 아니다.

계획의 상대 좌표 변환은 한 번만 수행하고 같은 plan의 end/배치 metadata를 함께 반영한다.
조건, 상수, 연산 순서, public API/IR, 테스트·baseline·golden·ignore는 변경하지 않는다.
상위 `typeset_paragraph` 호출 순서와 이후 높이 보정, renderer 소비자는 그대로다.

## 2. 구조의 한계와 후속 항목

`inline_flow.rs`의 wildcard import를 명시적 의존으로 바꿨다. 계산 모듈에서는 상태 쓰기가
불가능하지만, TypesetState 전체의 필드는 아직 parent 소유이며 조정자는 fit/전진을 위해
이를 조회한다. 다른 메서드의 상태 직접 쓰기까지 차단한 완전한 캡슐화는 아니다.
문단 fit/분할, table paragraph와 다른 컨트롤 흐름의 책임 분리는 R2 후속 절편에 남는다.

## 3. 검증 계획과 증거

원래 build closure, 확정 block, 단 기하 조회의 본문을 복원 비교하고 그 외 호출 순서를 확인한다.
검증 스크립트와 실행 로그는 `output/7280/stage5/`에 둔다.

고정 제품 SHA review worktree에서 suite 준비 후 fmt, manifest/unit-tier 정책 검사,
native Clippy와 기존 inline/어울림/문단 상태 관련 집중 계약을 순차 실행한다.
이전 R2b 전체 10,096 PASS를 이번 제품 SHA의 실행 결과로 재사용하지 않는다.
전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM·직접 시각 검증은 최종 통합
게이트에 남는다. 이번에 시각 통과나 PR 제출 준비 완료를 선언하지 않는다.
원격 push·PR·댓글은 수행하지 않는다.
