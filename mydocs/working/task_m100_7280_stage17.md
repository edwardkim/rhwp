# Task #7280 Stage 17 — R2o 다단 문단·흐름 관측값 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2n](task_m100_7280_stage16.md), 시작 head `e3d807627`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 후 고정 SHA 집중 검증 준비. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

`paragraph/columns.rs`는 저장 LineSeg의 다단 경계 조회와 조각 계산을 소유한다.
단 0의 any-decrease 경계와 비-0 단의 near-top 되감김 조건, 미주 흐름 제외,
`NEAR_TOP_RATIO = 0.15`를 그대로 유지한다. 기존 조건의 타당성을 새로 승인하거나
정책을 바꾸는 작업이 아니다.

`paragraph::try_place_multicolumn_paragraph`는 guide 흡수 뒤, 빈 문단 흡수 전의
기존 위치에서 다단 경로를 선택한다. 구성 줄 수를 벗어난 조각에서 루프를 종료해도
기존처럼 문단을 소비한 것으로 반환하고 일반 fit 경로로 재진입하지 않는다.
경계 간 조각 범위와 `line_advances_sum` 계산, Full/PartialParagraph 선택을 보존한다.

`state`는 확정 조각 추가 → 높이 누적과 단 전환을 별도 명령으로 소유한다.
일반 줄 분할의 `commit_split_paragraph_fragment`와 달리 다단 경로는
`vpos_prev_trimmed_sb_px`를 초기화하지 않는다. 중간 단에서는 flush → 단 증가 → 높이 0,
마지막 단에서는 `advance_column_or_new_page` 순서를 그대로 유지한다.
현재 다단 속성은 조정자가 읽으며 최종 상태 캡슐화는 R5에 남긴다.

`paragraph/metrics.rs::flow_hints`는 본문 마지막 저장 vpos와 문단 위 간격 trim 조건,
native HWP5의 trim 복원 gate를 읽기 전용 결과로 반환한다. 이 세 값을 fit 높이로
합치지 않으며 기존 전체/넘침/분할 소비자에게 그대로 전달한다.
IR/API·진단 정책·테스트 source/assertion/ID·baseline/golden/ignore는 변경하지 않는다.

유일한 관측 시점 차이는 다단 경로 선택 전 본문 높이 snapshot이다.
`PageLayout::available_body_height`는 레이아웃 수치의 산술 조회만 수행하며 상태를 쓰지 않는다.
원래 비-0 단 분기에서만 읽던 값을 미리 읽어도 선택된 경계나 상태 효과는 달라지지 않는다.

## 2. 검증 계획

`output/7280/stage17/verify-columns.mjs`로 원본 helper 본문, routing 조건, 조각 계산,
루프 복원, 상태 명령의 효과 및 기존 parent/조정자/메트릭의 불변을 비교한다.
별도 review worktree에서 파생 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 실행한다. R2n의 179개에 기존 #2320 다단 계약 3개를
추가하며 테스트 source는 수정하지 않는다.

미주·잘못된 조각 범위 등 모든 입력 조합의 실행 검증이나 한컴 출력 일치를 주장하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM/직접 시각
대조는 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 범위가 아니다.
