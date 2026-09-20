# Task #7280 Stage 7 — R2e 문단 분할 경계 보정 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2d](task_m100_7280_stage6.md), 시작 head `178667990`.
- 상태: 구현 완료, 고정 SHA 집중 검증 대기. R2 전체 완료가 아니다.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.

## 1. 책임과 범위

`typeset_paragraph`의 줄 스캔이 만든 후보를 실제 분할 경계로 보정하는 책임을
`paragraph/split.rs`의 `refine_split_boundary`로 분리한다. 문서 IR·문단 구성 결과와
다음 문단, 현재 줄 범위·예산·호환 flag·dpi를 받으며 TypesetState/TypesetEngine은 받지 않는다.
상태를 변경하거나 다음 페이지를 생성하지 않는다.

`SplitBoundary`는 exclusive 끝 줄과 해당 후보의 누적 advance를 함께 반환한다.
두 값을 따로 재추정하지 않는다. 기존 조건·상수·연산 순서·PageItem 의미·기대값은 유지한다.
기존 조건의 타당성 또는 한컴 피델리티 개선을 이번에 판정하는 것은 아니다.

보정 순서는 기존대로 최소 한 줄 보장 → 뒤따르는 RowBreak 앵커 표 앞의 한 줄 되돌림 →
저장 줄 위치 되감김 복원이다. 최소 한 줄 보장 시 cumulative를 다시 계산하지 않던 동작도
그대로 둔다. 표 앵커/되감김 보정에서만 기존 `line_advances_sum`으로 누적량을 다시 계산한다.

본문 높이의 기존 메서드는 읽기 전용 `layout.available_body_height()`이고 profile 조회도
불변 상태를 읽는다. 보정 block 내부에는 상태 변경이 없으므로 호출 시 값으로 전달한다.
각주/여백 등을 차감한 `avail_for_lines`와 기본 본문 높이를 하나로 합치지 않는다.

## 2. 생산·소비 경계

- 생산: 기존 `for li` 줄 스캔의 `end_line`, `cumulative`. 저장 꼬리 채택 flag도 기존 위치에서 생산한다.
- 보정: 새 Query가 같은 end/advance 쌍을 반환한다. 시작 줄은 조정자가 소유한다.
- 소비: 반환 end로 `part_line_height`와 마지막 spacing_after를 구하고 part_height를 조립한다.
  전체 수용 경로는 반환 cumulative로 trailing spacing 제외/overflow를 재확인한다.
- 확정: 기존 FullParagraph/PartialParagraph 추가 → trimmed spacing 초기화 → 높이 전진.
  끝 줄이면 종료하고, 아니면 단/쪽 전환 후 cursor를 end로 바꾸는 순서는 변경하지 않는다.

줄 스캔 자체의 saved-tail/각주/강제 경계 정책, 표 셀/rowspan/continuation은 이번 절편 밖이다.
새 Query가 전체 문단 분할을 소유한다고 주장하지 않는다. `is_synthetic_line_seg`와
`LADDER_FIT_EPSILON_PX`의 parent 의존은 명시적 import로 남아 있으며 최종 소유권은 후속 정리한다.

## 3. 검증

`output/7280/stage7/verify-split.mjs`는 입력/반환 매핑을 확인한 뒤 Query 본문을 원래 위치로
복원 비교한다. 상수·조건·주석·계산 순서 및 그 외 parent 코드가 일치한다.
formatter 공백/후행 쉼표와 짧아진 변수명 때문에 생긴 단일 식 closure 중괄호 차이만 정규화한다.
기존 테스트·golden·baseline·ignore·CI 정책과 public API/IR은 변경하지 않는다.

집중 계약은 기존 typeset/composer와 `issue_6542`, `issue_6718_native_hwp5_zero_vpos_rewind`,
`issue_6718_zero_rewind_in_split_paragraph`를 선택한다. 실제 render tree의 본문 하단,
여유 예산이 남은 되감김, sub-pixel fit을 포함한다. 최소 진행/다음 표 앵커의 모든 조건 조합을
새로 실행 검증했다고 주장하지 않는다. 정적 보존 증거나 자동 검사를 직접 시각 판정으로 승격하지 않는다.

고정 SHA review worktree에서 suite 준비, fmt·manifest/unit-tier 정책·native Clippy와 집중 검사를
순차 실행한다. 전체 회귀, 최종 WASM/workspace lint와 native build, Native Skia,
fresh Docker WASM·직접 시각 검증은 최종 통합 게이트에 남는다. 원격 쓰기는 하지 않는다.
