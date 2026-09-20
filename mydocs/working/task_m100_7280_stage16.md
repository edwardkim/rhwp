# Task #7280 Stage 16 — R2n 강제 쪽 경계 조회·선택 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2m](task_m100_7280_stage15.md), 시작 head `82fdbbcf6`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현·집중 검증 진행 중. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

`paragraph/boundary.rs`는 컨트롤의 inline metadata 판정과 저장 줄 기반 문단 내부 쪽 경계,
HWPX 명시적 다음 쪽 앞의 tail 경계 조회를 소유한다. 기존 세 helper의 본문·상수·비교 조건은
보존한다. 별도 좌표/높이 계산이나 저장 LineSeg 수용 범위 변경은 하지 않는다.

`paragraph::prepare_forced_page_boundary`는 기존 각주 경계를 **먼저** 계산하고, 현재 쪽
vpos 기준을 조회한 뒤 후보를 다음의 기존 순서로 선택한다.

1. 내부 저장 경계와 HWPX 현재 흐름 일치 필터
2. HWPX 명시적 쪽나누기 앞 tail
3. 첫 각주와 겹치는 본문 경계
4. 저장 줄 누락의 trailing line 경계
5. 큰 TAC TopAndBottom 그림 앞 본문 경계
6. 앞서 계산해 둔 기존 각주 reset 경계

`or_else`/`then`/`filter`의 지연 평가와 마지막 `or`를 유지한다. 모든 후보를 미리 계산한
목록으로 바꾸지 않는다. 각주 조회는 composer와 진단을 호출할 수 있으므로 평가 시점을 보존한다.
조정자는 `&TypesetState`만 읽고 상태를 쓰지 않는다. 기존 각주/그림 helper의 전체 상태 읽기
의존은 이행 항목이며 R4 및 R5의 좁은 관측·상태 경계 분리에서 다룬다.

결과 세 값은 서로 합치지 않는다. 선택된 `forced_page_break_line`은 전체 fit/넘침/분할로,
`native_hwp5_existing_footnote_reset_line`은 별도로 후속 줄 스캔으로,
`current_page_vpos_base`는 fit의 저장 bounds와 분할 경계로 그대로 전달한다.
`scan_lines`의 줄 컷 선택 → 기존 경계 보정 → 기존 조각 높이/각주 예약 → 페이지 전환 및
실제 배치 경로는 수정하지 않는다. 문단 전후의 fit 예산, 빈 줄/다단 조기 반환, trim 계산도 유지한다.

이번 절편은 기존 구현의 구조 이동이지 heuristic의 타당성 승인이나 조판 결함 수정이 아니다.
IR/API·테스트 assertion/ID·baseline/golden/ignore·CI 정책은 변경하지 않는다.
기존 private 테스트는 parent의 좁은 import로 연결해 모듈 ID와 검사를 보존한다.

## 2. 검증 계획

원래 helper 본문과 조정 block으로 복원 비교하여 조건·우선순위·지연 호출·반환값을 확인한다.
별도 review worktree에서 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 실행한다. 기존 typeset/composer·저장 경계·각주 계약을
선택하며, 후보 충돌의 모든 조합이나 최종 출력의 한컴 일치를 입증했다고 주장하지 않는다.

전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 통합 게이트에
남긴다. 이번 절편에서는 원격 push·PR·댓글을 수행하지 않는다.
