# Task #7280 Stage 13 — R2k 문단 진입과 fit 예산 준비 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2j](task_m100_7280_stage12.md), 시작 head `da9e67b4f`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현·집중 검증 진행 중. 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

문단 진입의 저장 꼬리 판정과 편집 세션 그림의 공간 요구를 `paragraph/entry.rs`로 분리한다.
Query는 IR·구성 결과와 필요한 읽기 전용 값만 받으며 가변 state를 받지 않는다.
`available_height()`는 진단을 동반할 수 있으므로 closure로 전달하여 원래 단락 평가 위치와
횟수를 보존한다. 저장 꼬리 판정 후 높이 적용 시 별도 조회하는 기존 동작도 유지한다.

`paragraph::prepare_fit_budget`이 저장 꼬리 → 진입 진단 → 편집 그림 이월 → 엄격 fit flag
소비 → 안전여백 → float 배제 → 각주 반환 → 꼬리 허용 → 최종 예산 순서를 조정한다.
엄격 fit 소비와 저장 꼬리 높이 적용은 state command가 소유한다. 기존 R2d 보정 command는
재사용하며 조건·상수·호출 순서·flag 수명은 바꾸지 않는다. 엔진의 session-edited 값은
진입 인자로 관측한다. 이 구간은 엔진 profile을 변경하지 않는다.

소비 경로: 진입/그림 판정 → 기존 높이/페이지 전이 → 기존 보정으로 산출한 available →
상위의 빈 문단 흡수·강제 경계·fit → 기존 전체/분할 배치. 이번에는 예산을 산출하는
책임만 분리하며 이후 분기에서의 재계산·좌표 선택·paint는 변경하지 않는다.
기존 heuristic을 올바른 한컴 조판 규칙으로 새로 승인하는 작업이 아니다.

빈 문단 흡수, 강제 저장 경계, 최종 fit 선택, 표 문단 흐름은 후속에 남긴다.
IR/API, 테스트 assertion·ID, baseline·golden·ignore, CI 정책은 수정하지 않는다.

## 2. 검증 계획

이동 block의 복원 비교로 조건/연산/상태 적용 순서를 확인하고, review worktree에서
파생 suite 준비·fmt·고정 baseline 대비 manifest/unit-tier·native Clippy·집중 nextest를 순차 실행한다.
typeset/composer와 진입 저장 꼬리·빈 host float 및 기존 문단 fit 관련 계약을 포함한다.
전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 최종 통합
게이트에 남긴다. 집중 PASS를 전체 CI나 한컴 시각 일치로 보고하지 않는다.
원격 push·PR·댓글은 수행하지 않는다.
