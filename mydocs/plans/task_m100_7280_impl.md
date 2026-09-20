# Task #7280 — 조판 코드 책임 분리 및 변경 관리 구현계획

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 상태: 2026-09-20 작업지시자 승인. [R1 완료](../working/task_m100_7280_stage2.md), [R2a 완료](../working/task_m100_7280_stage3.md), [R2b 문단 구성 분리·기본 feature 전체 회귀 완료](../working/task_m100_7280_stage4.md), [R2c inline 계획/상태 분리](../working/task_m100_7280_stage5.md), [R2d fit 보정 조회/소비 분리·집중 검증 완료](../working/task_m100_7280_stage6.md). R2 전체 흐름 조정 분리는 남아 있다.
- 수행계획: [task_m100_7280.md](task_m100_7280.md)
- 기준 조사: [stage1](../working/task_m100_7280_stage1.md), 커밋 `816f57818`.
- 제품 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 기준 회귀: 10,096건 통과 / 실패 0건 / 제외 50건. 기본 feature 로컬 nextest 결과이며 전체 CI·시각 정확성 판정은 아니다.
- 후속 진행: [R2e 분할 경계 보정 분리·집중 검증 완료](../working/task_m100_7280_stage7.md).
  R2 줄 스캔·배치 및 표 문단 흐름 조정 분리는 계속 진행한다.
- 현재 절편: [R2f 줄 후보 스캔 분리·집중 검증 완료](../working/task_m100_7280_stage8.md).
  페이지 관측값과 읽기 전용 후보 계산을 분리하며 실제 배치·이월 조정은 후속에 남긴다.
- 후속 절편: [R2g 문단 분할 배치·이월 조정 분리·집중 검증 완료](../working/task_m100_7280_stage9.md).
  줄 분할 반복의 조정과 배치 계획/상태 적용을 구분한다. 진입 fit와 표 문단 흐름은 남아 있다.
- 후속 절편: [R2h 전체 문단 배치 책임 분리·집중 검증 완료](../working/task_m100_7280_stage10.md).
  일반 전체 배치의 순서 판단, 메트릭 조정, 상태 반영을 분리했다. 진입 fit·특수 배치와
  표 문단 흐름, 최종 통합 검증은 아직 남아 있다.
- 후속 절편: [R2i 문단 넘침 허용 경로 분리·집중 검증 완료](../working/task_m100_7280_stage11.md).
  atomic/tail의 읽기 전용 판단과 서로 다른 상태 적용을 분리했다. 문단 진입 fit·빈 구성 결과와
  표 문단 흐름, 최종 통합 검증은 후속에 남아 있다.

## 1. 구현 목표와 비범위

여러 기여자가 조판 코드를 추가·수정·삭제할 때 **어디를 바꾸고, 무엇에 영향을 주며,
무엇을 검증해야 하는지** 코드 경계와 문서에서 찾을 수 있도록 한다.
SOLID와 CQRS를 적용하되 문단 중심의 HWP/HWPX 공통 IR을 출발점으로 삼는다.

이번 변경은 기존 동작을 보존한다. 페이지 수, 빈 줄·표 높이, 분할 위치, 한컴 피델리티,
성능 문제를 개선하지 않는다. 기존 예외·상수도 근거가 불명확하다는 이유로 제거하지 않는다.
의심되는 규칙은 구현 위치와 소비자를 기록해 후속 대상으로 분리한다.
반면 구조 이동 때문에 새로 생긴 회귀는 이번 작업에서 해결한다.

공통 IR 스키마, HWP/HWPX 파서·저장, public API, PageItem 직렬화 의미, CI workflow와
test policy는 변경하지 않는다. trait·범용 rule engine·동적 등록·이벤트 버스를 신설하지 않는다.

## 2. 구조와 의존 방향

다음은 목표 모듈 배치다. 기존 `typeset/inline_flow.rs`는 유지한다.
모듈은 해당 절편에서 실제 책임을 옮길 때 만들며 빈 디렉터리를 먼저 대량 생성하지 않는다.

```text
renderer/typeset.rs                  공개 진입점·호환 경로
renderer/typeset/
  section.rs                         구역의 문단 순회와 실행 순서 조정
  state.rs                           페이지/단 상태와 확정 결과 적용
  paragraph.rs                       문단 구성·fit/분할의 조정
  paragraph/line_queries.rs           구성된 줄의 범위·텍스트/컨트롤 참여 조회
  paragraph/metrics.rs                문단 구성 결과와 기존 높이/전진량 조회
  paragraph/context.rs                문단 구성에 허용된 엔진 관측 인터페이스
  paragraph/format.rs                 재구성 선택·줄 메트릭·구성 결과 조립
  paragraph/fit.rs                    문단 fit 보정의 읽기 전용 계산
  paragraph/scan.rs                   페이지 관측값으로 줄 수용 후보·저장 꼬리 증거 계산
  paragraph/split.rs                  줄 스캔 후보의 분할 경계 보정
  paragraph/placement.rs              보정된 컷의 항목/높이 계획과 전체 fit 재확인
  paragraph/overflow.rs               일반 fit 실패 뒤 atomic/tail 넘침 허용의 읽기 전용 판정
  paragraph/stored_lines.rs           기존 저장 줄 정보 선택·해석 정책
  controls.rs                        소유 문단 내 TAC/float 경로 선택·지연 배치 조정
  table.rs                           표 진입·준비와 기존 결과 타입
  table/scan.rs                      가용 영역에서 행/셀 분할 후보 계산
  table/continuation.rs              표 조각 적용·재개 커서와 종료
  notes.rs                           각주 예약·표 각주 큐 조정
  notes/endnotes.rs                  미주 내용 준비·구성·배치 조정
  inline_flow.rs                     기존 공통 InlineFlowPlan의 흐름 적용
```

### 책임과 비책임

| 책임 소유자 | 소유할 판단/상태 | 여기서 하지 않을 일 |
| --- | --- | --- |
| section | 원본 문단 순서, 영역/컨트롤 담당자 호출 순서 | 셀 컷 계산·미주 줄 구성의 세부 구현 |
| paragraph | 문단의 구성 결과와 줄 범위, 기존 stored/reflow 선택 | 문서 IR 수정, 표 셀 내부의 별도 문단 규칙 창설 |
| controls | 소유 문단·컨트롤의 순서와 배치 경로 조정 | TAC를 같은 줄로 간주하거나 float를 무조건 문단 아래에 쌓기 |
| table | 표의 준비 결과·행 도메인·컷과 continuation | 출력 backend가 원본 표 소유권을 재추정하도록 새 정보 누락 |
| notes | 각주/미주 내용과 예약·이어받기 수명 | 본문 높이 규칙을 자체 재정의 |
| state | 확정 항목·흐름 위치·페이지/단 전이 | 원본 문서 해석과 fit 조건의 재계산 |

의존은 `IR/서식/기존 구성·측정 결과 → 조회·계산 → 흐름 조정 → 상태 적용`으로 정리한다.
paragraph와 table의 조정 계층이 필요한 다른 담당자를 호출할 수는 있지만,
조회·계산 함수가 다른 도메인의 가변 상태를 직접 수정하지 않도록 한다.

현재 typeset→LayoutEngine 컷 측정, typeset→document_core helper와 같은 역방향 의존은
별도 목록으로 남긴다. 이미 공용 renderer 모듈이 적합한 소유자이면 동작 보존 이동을 검토하되,
이번에 composer/height_measurer/layout/document_core 전체를 다시 설계하지 않는다.
불가피하게 남는 의존에는 호출 위치·이유·소유자를 명시한다. 완전한 단방향화를 달성했다고 주장하지 않는다.

## 3. 입력·결과와 CQRS 계약

### 문단과 IR 경계

- 문단 식별과 컨트롤 순서에는 기존 Paragraph와 위치 접근자를 사용한다.
  `controls` 배열 순서만으로 텍스트 삽입 위치를 다시 만들지 않는다.
- UTF-16/control 슬롯, 구성 결과의 문자 인덱스, 저장 줄의 정규화 축을 구별한다.
  HWPX `line_seg_text_start`, 저장 vpos/보강 줄/dirty 정보의 현재 의미를 유지한다.
- 하위 표·셀·각주도 문단을 소유한다. 논리 소속과 실제 쪽·단·문단 좌표 기준을 분리한다.
- 새로운 범용 문단 트리나 IR 복사본을 만들지 않는다. 필요한 함수에 기존 참조와 관련 필드만 전달한다.

### 조회·계산(Query)

- `ComposedParagraph/Line`, `FormattedParagraph/Table`, `MeasuredTable`, `BlockTableRowScan`,
  `InlineFlowPlan`을 우선 재사용한다. 모든 도메인을 담는 새 거대 Result는 만들지 않는다.
- 가용 영역은 현재 페이지/단의 읽기 전용 관측값으로 전달한다. `&mut TypesetState`를 받지 않는다.
- 계산의 반환에는 기존 fit 높이·컷·물리 잔여 높이의 구별을 보존한다. 서로 다른 높이를 하나로 합치지 않는다.
- 내부 캐시는 별도 의존으로 식별한다. 호출 순서에 영향을 주는 엔진 flag는 순수 캐시로 위장하지 않는다.
  부수효과가 있는 경로는 조정 함수로 남기고 그 안에서 독립 계산 부분만 분리한다.

### 확정 적용(Command)과 조정

- 확정 결과로 PageItem 추가, 흐름 전진, 각주 예약, 커서 갱신을 수행한다.
  상태 소유 모듈의 의미 있는 메서드로 접근하고 범용 `set(field, value)`를 노출하지 않는다.
- 페이지 전환·예약·컷 전진의 **기존 실행 순서**를 보존한다. 이를 새로운 원자적 트랜잭션으로 바꾸지 않는다.
- 상태 변경 뒤 다시 측정하는 기존 경로는 조정 단계의 별도 호출로 드러낸다.
  “Command는 재판정하지 않는다”를 맞추려고 계산 시점이나 조건을 바꾸지 않는다.
- `ResumableTablePaginationJob`과 continuation context의 소유권·완료·재개 계약을 그대로 유지한다.
  native 일괄 경로와 WASM 재개 경로가 별도 표 분할 정책을 갖도록 만들지 않는다.

### 접근 제어

- 기존 `TypesetEngine`와 crate-visible 재개 API의 경로는 facade에서 유지한다.
  내부 모듈은 private, 필요한 연결만 `pub(super)` 또는 typeset 범위로 제한한다.
- state 필드는 최종적으로 state 소유 모듈 안에서 변경한다. Query에는 필요한 값/읽기 전용 관점만 준다.
- 이전 중 호환 import는 허용하지만 모든 하위 모듈에 `use super::*`와 전체 상태 변경 권한을
  배포한 것을 완료 상태로 삼지 않는다. 기존 규칙별 소유 모듈과 명시적 import로 수렴시킨다.
- shared 결과 타입의 필드와 가변 진행 상태의 필드는 같은 공개 정책을 적용하지 않는다.
  테스트 편의를 위한 public API 확장은 하지 않는다.

## 4. 구현 절편과 승인 경계

| 절편 | 실제 이동/분리 대상 | 종료 증거 |
| --- | --- | --- |
| R1 줄 조회 | `composed_line_char_end`, `line_has_strict_tac_control`, `line_has_visible_text`, `line_has_text_span`을 `paragraph/line_queries.rs`로 이동 | 함수 본문·호출 순서 불변, 모든 호출/테스트 접근 확인, focused·정책 검사 |
| R2 문단/컨트롤 | 저장 줄 helper, `format_paragraph_for_flow`, `typeset_paragraph`, `typeset_table_paragraph`, 기존 inline flow의 담당 경계 분리 | 계산과 조정 구별, 원본 순서·TAC/float·빈 줄·stored/reflow 계약 보존 |
| R3 표 계산/재개 | `format_table`, `scan_block_table_split_rows`, `typeset_block_table_inner`, continuation 타입·step | 원본/유효 표 도메인·시작/끝 컷·물리 밴드·각주 큐·재개/종료 동일 |
| R4 각주/미주 | footnote 예약 helper 및 endnote 준비·측정·emit·문단 흐름 | 본문/각주 예산·번호·예약 순서·상태 변화 없는 측정 계약 보존 |
| R5 상태/구역 | TypesetState 전이와 `typeset_section_with_variant`의 조정 역할 확정 | 도메인별 직접 쓰기 제거, 명시적인 호출 순서, facade 호환, 잔여 의존 목록 |
| R6 변경 관리/통합 | 실제 모듈 지도·규칙 연결·기여 절차 정리, 전체 검증·재계측 | 추가·수정·삭제 경로를 코드/테스트로 추적 가능, 무회귀·최종 gate 증거 |

R1은 조사에서 입력과 부수효과 범위를 확인한 네 함수만 대상으로 한다.
기존 조건의 의미·이름·비교 연산·상수·반환값을 함께 고치지 않는다.
먼저 작은 절편에서 private 테스트 접근과 CI 정책을 검증한 뒤 더 큰 책임을 분리한다.

R2–R5는 한 번에 대량 이동하지 않는다. 각 절편의 실제 호출자·쓰기 위치·기존 테스트 목록을
작업 기록에 고정하고 계산 → 조정 → 상태 적용 순으로 작은 커밋을 만든다.
최종 상태 캡슐화 전까지 남는 직접 쓰기는 이행 항목으로 기록하며 완료로 표시하지 않는다.
줄 수를 줄이려고 거대 함수를 다른 파일에 그대로 옮긴 상태에서 종료하지 않는다.

이 구현계획 승인 후 위 절편을 순서대로 진행한다. 기존 동작 보존이 불가능하거나
IR/API/정책 변경·조판 규칙 수정이 필요해지면 그 부분을 멈추고 별도 판단을 요청한다.
기존 예외를 제거하거나 새로운 공통 알고리즘을 도입하는 승인으로 해석하지 않는다.

## 5. 테스트의 위치와 보존

- 기존 `typeset.rs`의 네 `#[cfg(test)]` 모듈은 우선 같은 파일과 모듈 ID에 유지한다.
  helper 이동은 parent의 좁은 import로 연결하여 기존 assertion을 보존한다.
- source-side 테스트를 새 파일/모듈로 옮기거나 복제하지 않는다. `include!`로 숨겨
  테스트 inventory를 우회하는 것도 하지 않는다.
- 상태 캡슐화 시 기존 테스트의 직접 필드 접근은 실제 제품에서도 사용하는 생성/조회/전이
  API로 바꾼다. 관측하던 내용·컷·전이 의미를 유지하고 기존 검사 삭제로 정책을 맞추지 않는다.
  유지 가능한 접근 경계를 찾지 못하면 캡슐화 방식을 재검토한다.
- 새 외부 계약은 `tests/cases/` 원본에 추가한다. private 함수를 검사하려고 공개 API를 늘리지 않는다.
- `rust-unit-test-tiers.mjs --check --base-ref <고정 실제 base>`로 테스트·모듈·support item을
  확인한다. 신규 모듈·기준 상향 금지와 Git rename 인식 제약을 따른다.
- generated suite/manifest는 review worktree에서만 준비하고 stage하지 않는다.

우선 연결할 기존 계약은 줄 범위/문단 분할, saved-vpos/빈 문단, `issue_6935` 컷 공간,
`issue2424` continuation 수명·재개, `test_measure_endnote_advance_side_effect_free`다.
suite 번호를 고정하지 않고 실제 원본과 테스트 이름으로 찾는다. 매 절편의 정확한 실행 목록은
변경된 소비 경로와 대조한다. 페이지/항목 수만 비교하는 검사를 동작 전체의 증거로 격상하지 않는다.

## 6. 기여자 변경 절차와 규칙 관리

장기 구조 정본은 `mydocs/tech/typesetting_architecture.md`로 제안한다.
문단→컨트롤→하위 문단 흐름, 실제 모듈 지도, 의존 방향, 책임/비책임과 변경 절차를 담는다.
표의 의미 규칙은 기존 `table_layout_rules.md`를 참조하고 내용을 중복 정의하지 않는다.
기존 기술 지도와 기여 안내에는 필요한 링크만 추가한다.

규칙별로 의미 기반 식별자, 근거, 적용/비적용 조건, 담당 구현, 입력/결과·상태 소유자,
소비자와 관련 테스트를 연결한다. 이슈 번호는 변경 이력이며 규칙 이름의 대체물이 아니다.
기존 예외의 근거가 불명확하면 “근거 확인 필요”로 표시하며 타당한 규칙으로 새로 승인하지 않는다.
우선 이번에 이동한 규칙을 대상으로 작성하고 전체 엔진의 규칙 목록이 완성됐다고 주장하지 않는다.

| 작업 | 기여자가 확인할 것 | 리뷰 기준 |
| --- | --- | --- |
| 추가 | 기존 규칙 재사용 가능성, 담당 모듈, 입력/결과, 적용·비적용 사례 | 새 규칙이 다른 모듈의 가변 상태를 직접 소유하지 않는가 |
| 수정 | 규칙 소비자·좌표/단위·분기 경로, 변경 전후 계약과 증거 | 계산만 바꾸고 실제 소비자를 누락하지 않았는가 |
| 삭제/대체 | 모든 호출부·fallback·공유 타입·테스트·문서, 대체 책임 | 다른 규칙의 보호 계약까지 삭제하지 않았는가 |
| 구조 이동 | 원래 함수와 새 위치, 가시성, 실행 순서, 상태 수명 | 동작 변경이 섞이거나 중복 소유가 생기지 않았는가 |

별도 양식·새 CI 서비스를 강제하지 않고 기존 PR 설명과 코드 문서에 이 연결을 남긴다.
모듈별 문서만 만드는 대신 Rust 접근 제어와 명시적 입출력으로 가능한 경계를 강제한다.

## 7. 검증 및 중단 기준

1. 각 절편: 원본/변경 함수·호출자 비교, fmt·관련 focused test, 적용되는 source-side 정책 확인.
2. 책임 묶음 완료: 동일 입력·설정의 baseline과 변경본에서 내용 소유권, 컷, 기하 및
   실제 영향 페이지의 native/fresh Docker WASM 출력을 대조한다. 기존 잘못된 출력을 보존한 것은
   구조 이동의 무회귀 증거이지 올바른 조판이라는 승인이 아니다.
3. 제출 전: [local_validation §4.3](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)의
   fmt·native/WASM/workspace Clippy·workspace build·전체 회귀·Native Skia·WASM 및
   [시각 검증](../manual/verification/visual_verification_governance.md)의 적용 gate를 정확한 head에서 수행한다.
   현재 baseline 실행을 변경 head 결과로 재사용하지 않는다.
4. Cargo는 고정 review target에서 순차 실행한다. 실제 제출 base가 갱신되면 base/head를 다시
   고정하여 정책·회귀를 확인한다. 기존 baseline과 통합 후 baseline은 구분한다.
5. 복잡도는 같은 계측 조건으로 재측정하되 총 줄 수·CC 감소율을 단독 통과 기준으로 쓰지 않는다.
   책임별 변경 위치·쓰기 권한·남은 역방향 의존이 명확해졌는지가 주된 구조 검토 항목이다.

새 출력 차이 또는 실제 회귀는 해당 절편에서 추적하고 해소한 뒤 진행한다. 기준값·ignore·golden을
완화하지 않는다. 원인이 기존 조판 결함이면 증거를 보존해 별도 작업으로 분리하며,
리팩토링에 섞어 고치거나 한컴 정답에 맞추기 위한 범위 확대를 하지 않는다.

## 8. 승인과 착수

1단계 결과를 `816f57818`로 커밋한 후 이 구현계획을 작성했다.
이번 절차에서는 제품·테스트·CI 코드와 원격 저장소를 변경하지 않았다.
2026-09-20 작업지시자의 “승인합니다”에 따라 **R1 줄 조회 책임 분리**부터 진행한다.
