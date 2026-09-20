# Task #7280 Stage 15 — R2m 전체 문단 fit 선택 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2l](task_m100_7280_stage14.md), 시작 head `b8b0cf557`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현·집중 검증 진행 중. 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

이번 절편은 이미 계산된 강제 경계를 소비하는 전체 문단 fit 선택을 분리한다.
강제 경계 후보 chain은 각주/저장 줄 helper와 연결되어 있어 다음 절편으로 남긴다.

`paragraph/whole_fit.rs`는 원본/구성 결과와 제한된 페이지 관측으로 저장 마지막 줄 신뢰,
목록 꼬리 신뢰, 요구 높이, 되감김 및 Hangul 2024 재수용 근거를 반환한다. `available_height()`는
진단을 포함하므로 closure로 기존 단락 평가 위치에서만 호출한다. base 높이 관측은 부수효과가 없다.
그림 배제 영역을 포함한 점유 높이와 흐름 높이의 차이, full advance와 fit 높이의 차이,
저장 경계·회수량 조건·기존 상수는 보존하며 서로 합치지 않는다.

`paragraph::decide_whole_fit`은 조회 → 되감김 진단 → 해당 빈 문단의 spill 기록 → 기존 spill
관측 → fit 진단 → 전체 fit 선택 순서를 조정한다. spill 쓰기는 기존 state command를 재사용한다.
진단보다 앞에 쓰거나 fit 선택 이후로 미루지 않는다. 결과의 overflow 되감김은 기존 분할 진입에
전달하고, fits는 기존 전체 배치에 전달한다. fit 실패 뒤 overflow/분할 경로도 그대로 둔다.

소비 경로: 기존 저장 경계/fit 예산 → 읽기 전용 fit 근거 → 호환성 상태 기록 → 전체 fit 선택 →
기존 배치 또는 넘침 허용/분할. 배치 컷·높이·paint와 IR/API는 변경하지 않는다.
기존 heuristic의 타당성 승인이나 조판 수정이 아니며 테스트 assertion·ID, baseline·golden·ignore,
CI 정책도 변경하지 않는다.

## 2. 검증 계획

Query와 조정자의 결과/입력을 원래 block으로 복원 비교하여 조건·진단·spill 쓰기 순서를 확인한다.
별도 review worktree에서 파생 suite 준비 → fmt·고정 baseline 대비 manifest/unit-tier →
native Clippy → 집중 nextest를 순차 실행한다. 기존 문단 fit 계약과 저장 되감김/실제 float 점유
회귀를 포함하되 전체 호환성 조합의 직접 검증이나 한컴 시각 일치를 주장하지 않는다.

전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 통합 게이트에
남긴다. 원격 push·PR·댓글은 수행하지 않는다.
