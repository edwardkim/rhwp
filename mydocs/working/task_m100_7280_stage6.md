# Task #7280 Stage 6 — R2d 문단 fit 보정의 조회·소비 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2c](task_m100_7280_stage5.md), 시작 head `c3fb3c41f`.
- 상태: 구현 완료, 고정 제품 SHA 검증 대기. R2 전체 완료가 아니다.
- 고정 제품 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.

## 1. 범위와 기존 실행 순서

문단의 전체 fit/줄 분할에 앞서 수행하는 예산 보정만 분리한다. 조판 규칙·상수·조건·연산 순서와
기존 예외는 그대로 보존한다. 본문/표 문단 분할 알고리즘, IR, public API, paint는 변경하지 않는다.
기존 heuristic의 근거를 이번 리팩토링으로 새로 승인하는 것이 아니다.

`paragraph/fit.rs`는 문서의 drift 안전여백, float 배제 영역 probe 높이, 저장 꼬리 fit 여유를
읽기 전용으로 계산한다. TypesetState/TypesetEngine이나 가변 flag를 받지 않는다.
`state.rs`의 의미 있는 소비 메서드는 안전여백 면제, 각주 안전여백 반환, 저장 꼬리 증거를 각각
한 번 소비한다. 읽기 전용 계산을 하는 것처럼 보이던 가변 flag 접근을 호출 이름에 드러낸다.
범용 setter나 트랜잭션, 새로운 거대 context/result는 만들지 않는다.

상위 조정자 `typeset_paragraph`의 순서는 다음과 같이 유지한다.

1. 기존 strict plain-text flag 소비 및 drift 안전여백 조회
2. 직전 PartialTable 확인 → 안전여백 면제 소비
3. float probe 계산 → `apply_visible_float_exclusions`
4. 각주 안전여백 반환 소비 → 저장 꼬리 증거 소비/fit 여유 계산
5. `(available_height - safety + footnote_margin_addback + tail_overflow).max(0)`
6. 기존 빈 문단·강제 경계·전체 fit·줄 분할 경로로 진행

float 배제 영역 적용은 높이를 바꿀 수 있으므로 2와 4를 합치거나 앞당기지 않는다.
strict flag가 참일 때 면제/반환/꼬리 증거를 철회하는 순서도 그대로다.
저장 꼬리는 기존대로 `.take()` 이후 `base_available_height`와 현재 각주 높이를 읽는다.

## 2. 소비자와 남은 책임

`available`은 전체 fit와 초기 잔여 예산에 사용한다. 이후 줄 분할 루프는 기존대로
`base_available_height`, 각주/zone과 `layout_drift_safety_px`를 다시 반영한다.
이를 하나의 영구 예산으로 치환하지 않는다. 이번 절편에서 값과 생산 시점이 같으므로
PageItem/줄 컷·후속 높이 변경·배치 소비 경로는 모두 기존 구현에 둔다.

`fit.rs`가 공유하는 parent helper/상수는 명시적 import로 남겼다:
`section_has_zero_high_attr_rowbreak_table`, `saved_bounds_overlap_current_flow`,
`BODY_BOTTOM_SEAT_PX`, `SAVED_FRAME_FLOW_DRIFT_TOLERANCE_PX`.
다른 소비자가 있는 helper의 최종 소유권과 state의 다른 직접 쓰기는 후속 정리 대상이다.
strict flag의 기존 helper/테스트 위치, flag를 설정하는 구역 순회는 변경하지 않았다.
전체 상태 캡슐화나 paragraph/table paragraph 분해가 끝난 것으로 보고하지 않는다.

## 3. 검증

`output/7280/stage6/verify-fit.mjs`는 분리된 5개 block을 원래 호출 위치로 복원하고,
입력 인자 매핑·조건·상수·주석·실행 순서 및 그 외 parent 코드를 비교한다.
저장 꼬리 계산 helper 본문도 동일성을 확인한다. formatter 공백/후행 쉼표만 정규화한다.
정적 보존 검사는 직접 시각 판정이나 한컴 피델리티 증거를 대체하지 않는다.

기존 typeset/composer/float 계약과 저장 꼬리·문단 간격 관련 `issue_5941_tail_overflow_drift_gate`,
`issue_6031`, `issue_6753`의 기존 계약을 집중 실행한다. source 테스트·integration source·
baseline/golden/ignore·CI 정책은 수정하지 않는다. 고정 SHA review worktree에서 파생 suite를
준비하고 fmt·정책·native Clippy 후 집중 테스트를 순차 실행한다.

이번 결과를 R2 전체 완료 또는 제출 준비 완료로 보고하지 않는다. 전체 회귀, 최종
native/WASM/workspace lint 묶음, Native Skia, fresh Docker WASM/직접 시각 검증은
별도 통합 게이트에 남는다. 이전 절편의 실행 결과를 이번 제품 SHA의 PASS로 재사용하지 않는다.
원격 push·PR·댓글은 수행하지 않는다.
