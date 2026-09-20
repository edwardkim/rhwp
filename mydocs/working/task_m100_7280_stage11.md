# Task #7280 Stage 11 — R2i 문단 넘침 허용 경로 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2h](task_m100_7280_stage10.md), 시작 head `e88aca799`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 후 고정 SHA 검증 준비. R2 전체 완료가 아니다.

## 1. 책임과 비범위

일반 전체 fit 실패 뒤의 기존 두 넘침 허용 경로만 옮긴다. `paragraph/overflow.rs`는
인라인 그림/도형의 atomic 수용과 쪽 경계 전 tail 후보를 읽기 전용으로 판정한다.
원본 Paragraph, FormattedParagraph와 작은 OverflowPage 값 관측만 받으며 state를 쓰지 않는다.
두 판정 사이 상태 변경이 없고 atomic 성공 시 즉시 반환하므로 같은 관측값을 공유할 수 있다.
관측의 base_available_height는 순수 layout 조회다. 진단 출력이 있는 available_height는
snapshot에 넣지 않고 기존처럼 tail 후보 통과 뒤에만 조정자가 호출한다.

`paragraph::try_place_overflow_paragraph`는 atomic → tail 순서와 성공 시 반환을 조정한다.
state는 atomic 항목 추가 → 기존 메트릭 계산 → 높이/underrun/저장 하단 반영 순서를 지킨다.
tail은 항목 추가 → trim 0 → total_height 전진 → 저장 하단 반영이며 underrun은 변경하지 않는다.
atomic은 반대로 기존 trim을 덮지 않는다. 같은 FullParagraph라는 이유로 일반 전체 배치
command와 합치지 않는다.

기존 60px 문턱, 폰트 drift/inkless 조건, 추론/명시적 쪽 경계 구별과 단락·컨트롤 조건을
변경하지 않는다. 주석의 기존 한컴 동작 설명을 이번 리팩토링의 독립적 사실 확인이나 승인으로
격상하지 않는다. 근거가 약한 heuristic의 수정은 별도 작업이다.
빈 구성 결과, 일반 fit, split, 표/셀 continuation, 실제 layout/paint 및 IR/public API는 비범위다.
테스트 assertion/모듈 ID와 baseline·golden·ignore를 유지한다.

## 2. 검증 계획과 범위

`output/7280/stage11/verify-overflow.mjs`는 두 Query, snapshot 매핑, 경로별 상태 쓰기,
조건부 available_height 호출·메트릭 계산·반환 순서를 원본과 복원 비교한다.
부수효과 없는 base 높이/필드 읽기는 Query 진입 snapshot에서 관측한다.
`height_for_fit`의 수용 판단과 `flow_advance_height`/`total_height`의 실제 전진을 합치지 않는다.
후속 PageItem의 layout/paint 해석은 그대로이며 분할 컷·이월 자체는 변경하지 않는다.

review worktree에서 suite 준비 → fmt·고정 baseline 정책 검사 → native Clippy → typeset/composer,
#6854의 HWP/HWPX 빈 쪽 방지, #6793 저장 쪽 경계 등 기존 관련 계약을 순차 실행한다.
이 계약은 영향 경로의 무회귀 증거이며 모든 atomic/폰트 drift 조건 조합의 직접 커버리지나
한컴 피델리티 승인으로 보고하지 않는다. 실행 범위와 미검증을 최종 결과에 구분한다.
전체 회귀·최종 WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM/직접 시각
대조는 통합 게이트에 남는다. 원격 push·PR·댓글은 수행하지 않는다.

## 3. 결과와 후속

고정 SHA 검증 완료 뒤 결과를 기록한다. 문단 진입 fit와 빈 구성 결과, 표 문단/컨트롤 흐름,
나머지 state 직접 쓰기와 최종 통합 검증은 후속에 남는다.
