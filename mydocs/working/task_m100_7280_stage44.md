# Task #7280 Stage 44 — R3c rowspan 블록 분류·요구 높이 Query 분리

- Issue: #7280. 이전: [Stage43](task_m100_7280_stage43.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `11da183aa`. 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 head의 집중 검증 결과는 후속 기록한다.

## 이번 경계

`scan_block_table_split_rows`의 rowspan 보호 블록 분류, 행 오프셋, 컷 조각 높이 및
요구 높이 계산을 `table/scan.rs`의 `RowBlockQuery`로 분리한다. 입력은 원본 MeasuredTable,
실제 행 기하 Table, LayoutEngine, 스타일, 컷용 행 높이, rowspan 표식과 행 간격의 읽기 전용 값이다.
TypesetEngine/TypesetState나 가변 문서·페이지 상태를 전달하지 않는다.

분류는 기존 반복문의 같은 위치에서 수행한다. 블록 분기 진입 뒤에만 행 오프셋과 요구 높이를
계산한다. 소모한 유닛 컷과 물리 높이는 합치지 않는다. 시작 행의 빈 물리 밴드를 수용하는
`start_row_height_override`의 선행 continue, offset 컷의 재시도, 결과 반영은 부모에 남긴다.
기존 조건·허용치·float 연산/호출 순서와 반환을 보존하고 조판 정책의 타당성을 새로 승인하지 않는다.

## 호출·컷·높이·실제 배치 연결

1. continuation의 일반 스캔과 각주 예약량을 되돌린 refit 스캔 모두 기존
   `scan_block_table_split_rows`를 호출한다. 양쪽의 `row_geometry_table`, 원본 `mt`,
   `cut_row_h`/`whole_row_fit_h`, 시작 컷 및 가용 높이 구분은 유지한다.
2. Query의 분류 결과가 보호 블록/RowBreak rowspan 블록 경로와 행 오프셋 사용을 고른다.
   해당 블록의 첫 행이 아니거나 분류가 비해당이면 기존 일반 행 경로로 계속한다.
3. `required_height`는 시작 컷 유무·행 오프셋 여부에 따라 기존 행 합 또는 컷 내용 높이를 반환한다.
   이 높이를 부모가 `consumed + cs_before + block_h`로 예산과 비교하고 수용 시 누적한다.
4. 예산 실패 시 남은 예산으로 `advance_row_block_cut[_with_row_offsets]`를 호출한다.
   fresh 페이지 높이 비교, 밴드 재시도·강제 수용·다음 행 이월 판단은 이동하지 않는다.
5. 분할 수용 시 `fragment_height`의 행별 합과 컷 워크 높이를 기존 허용치로 대조한다.
   `split_end_cut`/`split_end_limit`/`split_block_start` 및 누적 높이는 기존 부모가 결정한다.
   이후 PartialTable 발행·커서 전진·각주·종료·paint의 별도 보정도 그대로다.

이번 분리는 컷 소유권·물리 밴드·요구 높이의 기존 관계를 보존하는 것이지 모든 경로를
단일 측정 결과로 교체하는 작업이 아니다. 블록 컷 이외 일반 행/거대 행·rowspan 끝행,
단일 행 컷의 판단은 후속 절편으로 남긴다.

## 검증 계획과 한계

고정 baseline의 기존 정식 계약을 독립적인 동작 기준으로 삼는다. 정적 대조에서 원본 분류식·
오프셋·컷 높이·요구 높이를 새 Query와 비교하고, 두 스캔 호출 및 나머지 부모 전체의 불변을 검사한다.
기존 집중 353건에 #6024, #6756, #6803, #7226의 rowspan 이어받기·중복·유닛 분할·본문 하한
계약을 추가한다. tests/cases 원본·기대값·baseline·ignore·source-side 테스트/support는 바꾸지 않는다.

전용 review worktree에서 suite 준비, 고정 baseline 명시 정책, fmt, native Clippy, 집중 nextest를
공유 `target/pr-review`에서 순차 실행한다. §7.2 R3 책임 묶음 전체 회귀·Native/fresh Docker WASM
시각 검증과 §7.3 최종 제출 게이트는 아직 미실행이다. 정적 동등성과 계약 통과를 모든 분기의
동적 실행이나 한컴 시각 일치로 해석하지 않는다. 원격 push·PR·댓글은 승인 범위가 아니다.
