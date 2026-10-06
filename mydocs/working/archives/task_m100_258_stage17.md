# Task M100-258 Stage 17 — 누름틀 전체 선택과 복사/붙여넣기

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `d654785b` (`task 258: 셀 필드 char_count 테스트 정정`)

## 1. 문제

누름틀 텍스트 전체를 선택할 수 없거나, 전체 선택처럼 보이더라도 복사/붙여넣기 시
ClickHere 누름틀 속성이 보존되지 않는다. 이 때문에 누름틀을 양식 필드 단위로 복제할 수 없다.

## 2. 조사 방향

- cursor/selection이 ClickHere `startCharIdx..endCharIdx` 범위를 선택할 수 있는지 확인한다.
- clipboard copy/export 경로가 선택 범위 안의 `FieldRange`와 `Control::Field`를 보존하는지 확인한다.
- paste 경로가 field range/control/CTRL_DATA를 새 위치에 재생성하는지 확인한다.

## 3. 원인

`copy_selection_native()`가 텍스트 클립보드에서 `SectionDef`/`ColumnDef`를 제거할 때
`controls`만 `retain()`으로 줄이고 `field_ranges.control_idx`와 `ctrl_data_records`는
재매핑하지 않았다.

`누름틀-2024.hwp` 첫 문단은 `SectionDef`, `ColumnDef`, `Field` 순서의 컨트롤을 갖기 때문에
구조 컨트롤 제거 후 `Field`는 0번으로 이동하지만 `FieldRange.control_idx`는 기존 2번을
가리켜 복사한 누름틀을 붙여넣을 때 ClickHere 필드로 수집되지 않았다.

## 4. 수정

- 텍스트 클립보드용 구조 컨트롤 제거 헬퍼를 추가했다.
- 구조 컨트롤 제거 시 `controls`, `ctrl_data_records`, `field_ranges.control_idx`,
  `control_mask`를 같은 기준으로 재구성한다.
- 본문 선택 복사와 표 셀 내부 선택 복사에 동일한 정규화 경로를 적용했다.
- 누름틀 전체 선택 복사 후 `clipboardHasControl()`이 false인 상태로 내부 텍스트/필드
  붙여넣기 경로를 타고, 붙여넣은 값이 다시 ClickHere 필드로 수집되는 회귀 테스트를 추가했다.

## 5. 검증

- `cargo fmt`
- `git diff --check`
- `cargo test --test issue_258_clickhere_form_mode copying_clickhere_preserves_field_control_after_structural_controls_are_stripped -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
