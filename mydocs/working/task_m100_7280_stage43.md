# Task #7280 Stage 43 — R3b 표 호스트 간격 Query·포맷 조립 분리

- Issue: #7280. 이전: [Stage42](task_m100_7280_stage42.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `193e49238`. 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현·정적 대조 준비. 고정 제품 head의 집중 검증 결과는 아래 후속 기록에 추가한다.

## 이번 경계와 보존 계약

`TypesetEngine::format_table`을 입력 연결 façade로 남기고, 읽기 전용 포맷 조립을
`table::format`으로 이동한다. 측정 조회·보정/fallback → TAC 판단 → host 간격 →
행/셀/누적 높이 clone → 총높이 → 각주 수집 → FormattedTable 조립 순서를 보존한다.
profile/TAC 질의는 기존 위치에서 지연 실행한다. 각주 수집은 기존 순회 순서를 유지하며
예약·페이지 전이·각주 분할 helper 본체를 수정하지 않는다.

간격 Query는 `table/host_spacing.rs`가 맡는다. 입력은 문단·표·스타일 참조, 컨트롤 인덱스,
다음 문단과 단 상단/TAC 관측값이다. 전체 엔진·가변 페이지 상태·측정 표 목록을 전달하지 않는다.
`HostSpacing`/`FormattedTable` 타입은 기존 분할 소비자가 사용하는 부모 위치에 유지한다.
공개 API, HWP/HWPX IR, 기존 조건/상수와 짧은 평가 순서에 의한 profile 조회를 바꾸지 않는다.

독립적인 동작 기준은 고정 baseline과 기존 정식 회귀 계약이다. 기존 호환 예외를 이동하는 것이며
해당 조건의 조판 타당성을 새로 승인하거나 #7195/#7090의 레이아웃 문제를 해결하는 작업이 아니다.

## 생산·소비 경로

- `host_spacing::resolve`의 `after`는 실제 흐름용이다. `table::format`의 총높이는
  `effective_height + before + after`로 기존과 동일하게 조립한다.
- `after_for_fit = after - outer_bottom_flow_only`는 빈 호스트 표의 기존 fit 제외분을 보존한다.
  블록 표의 `host_spacing_total`과 `controls/empty_float.rs`의 수용 판단이 이를 소비한다.
- 첫 조각의 원점은 `before`, 이어받기 마지막 조각에는 `spacing_after_only`가 별도로 전달된다.
  `strict_following_plain_text_fit`은 기존 양수 빈 호스트 RowBreak 꼬리 여부를 그대로 운반한다.
- 일반/TAC 연결은 `controls/flow_table.rs`, 지연 표는 `controls/deferred_placement.rs`,
  장식 표는 `controls/paragraph_flow.rs`의 기존 façade 호출을 유지한다.
- 이후 블록 배치의 앵커 선택·높이 덮어쓰기, 원본 MeasuredTable을 별도로 소비하는 행 컷,
  누적 예약·continuation·paint는 변경하지 않는다. 이번 이동이 모든 측정/배치 경로의
  공통 결과 통일을 입증하는 것은 아니다. 소비자 전체의 불변을 정적 대조한다.

## 검증 계획과 범위

원본 format 본문과 새 조정자/간격 Query를 재결합해 dpi/profile/TAC 조회 표기·포맷만
정규화하여 비교한다. 이전 fit Query와 부모의 나머지 본문도 불변인지 대조한다.
Stage42의 기존 집중 계약 352건에 #6147 빈 앵커의 최종 띠-본문 간격 계약을 추가한다.
source-side 테스트는 신규 추가하지 않는다. 기존 단위 테스트에서 사용하던 import만
제품 경로에서 해당 테스트 모듈로 옮기며 assertion·입력·기대값은 그대로 둔다.
tests/cases·baseline·ignore·생성 suite 정책은 수정하지 않는다.

고정 head review worktree에서 suite 준비, baseline 명시 manifest/unit-tier 정책, fmt,
native Clippy, 집중 nextest를 순차 실행한다. 공유 `target/pr-review`는 그대로 보존한다.
호스트 16 logical CPUs / RAM 31 GiB, 가용 약 19 GiB를 확인했다. 빌드 jobs 4 / test threads 8.

§7.2 R3 전체 회귀·Native/fresh Docker WASM 시각 비교와 §7.3 최종 제출 lint/build는
아직 수행하지 않는다. 정적 일치는 모든 guard 조합의 동적 실행이나 한컴 출력 일치의 증거가 아니다.
이번 승인 범위에서는 원격 push·PR·댓글을 하지 않는다.
