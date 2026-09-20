# Task #7280 Stage 22 — R2t 지연 표 큐 처리·상태 반영 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2s](task_m100_7280_stage21.md), 시작 head `489141f96`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2t 구현 완료, 고정 제품 SHA 집중 검증 예정. R2 전체 완료가 아니다.

## 1. 책임 경계와 보존 계약

`controls/deferred.rs`가 `DeferredTableControl`, `DeferredTableFlushPoint`와
`keeps_pending` 조회를 소유한다. 결과 타입의 필드는 typeset 내부에서만 공유하며 공개 API는
늘리지 않는다. 기존 후보 조회와 profile 읽기 시점은 불변이다.

`controls::flush_deferred_tables`는 빈 큐 확인 → 인출 → 원래 순서로 보류/배치 → 보류 큐 복원을
조정한다. 개별 표 배치는 정적 callback으로 기존 엔진에 위임하며, 조회 모듈에 엔진/가변 상태를
넘기거나 새 trait·동적 디스패치 계층을 도입하지 않는다.

`state.rs`가 큐 확인·추가·인출·복원과 배치 후 vpos 갱신을 맡는다. 빈 큐에서는 take하지 않는다.
보류 큐는 종전처럼 **교체**하며, 처리 중 큐에 추가된 항목과 임의로 병합하는 의미 변경은 하지 않는다.
큐 필드와 초기화 자체는 부모 TypesetState에 남아 있으므로 최종 상태 캡슐화 완료는 아니다.

### 실제 호출과 소비

- 구역 순회는 기존과 같이 표 문단의 강제 쪽/단 나눔 적용 **전**에 BeforeTableParagraph를,
  표 문단 처리 뒤에 AfterTableParagraph를, 구역 끝에 SectionEnd를 전달한다.
- Before는 `idx <= para_index`를 먼저 검사해 이후 슬라이스 접근을 단락 평가한다.
  사이에 가시 본문이 있으면 보류한다. After는 원래 문단 이후인지, SectionEnd는 항상 배치인
  기존 조건을 유지한다. 가시 문자와 빈 줄 점유를 새롭게 등치시키지 않는다.
- `place_deferred_table_control`은 원래 loop 본문 중 개별 표 처리만 담당한다.
  잘못된 문단/컨트롤 참조 또는 재판별 탈락 시의 `continue`는 이 함수의 `return`으로 옮겨져
  **그 항목만** 버리고 큐 순회는 계속한다. 전체 flush를 조기 종료하지 않는다.
- 실제 현재 열 너비로 문단 format → 후보 재판별 → 표 format → measured table 조회 →
  block 표 배치 → 미등록 각주 수집 → vpos 반영 순서를 보존한다.
- 렌더 원점에는 배치 시점 `current_height`, 분할 예산에는 원래 `deferred.para_start_height`를
  계속 전달한다. 각주 처리 뒤의 단일 단 조건과 마지막 항목 판별로 vpos를 갱신한다.
  상태 snapshot을 미리 만들어 이전 페이지 정보를 소비하게 하지 않는다.

컷/높이/예약/paint 산식, IR/API, 테스트 source/assertion/ID, baseline/golden/ignore와 CI 정책은
바꾸지 않는다. 조판 결함 수정이나 기존 heuristic의 정당성 재승인이 아닌 구조 이동이다.

## 2. 검증 계획과 한계

`output/7280/stage22/verify-deferred-flush.mjs`가 조회·큐 명령·callback을 기존 표현으로 복원해
원본 flush 본문과 대조한다. 타입/기존 후보 조회, 부모의 나머지 코드·테스트, 기존 상태 명령의
불변도 검사한다. 정적 비교는 실행 검증의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier, fmt,
native Clippy, 집중 nextest를 순차 실행한다. R2s의 228건에 후속 문단 선행 채움 #1753 1건과
`rowbreak_table_cell_footnotes_keep_the_pdf_fragment_boundary` 1건을 추가한다.
전자는 이월 전후 문단·표 소유와 중복을, 후자는 셀 각주와 표 조각 경계를 확인하는 기존 계약이다.
모든 flush 지점·유효하지 않은 큐 참조·각주 조합의 독립 실행 추적을 새로 추가하지는 않는다.
기존 계약의 통과를 직접 시각 판독이나 한컴 전체 일치로 격상하지 않는다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.
