# Task M100-258 Stage 13 — 누름틀 삭제 시 Field 컨트롤 완전 제거

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 선행 커밋: `084b601d` (`task 258: 기존 누름틀 샘플 경계 보정`)

## 1. 문제

삭제 확인 대화상자의 `확인`을 눌러도 사용자가 보기에는 누름틀이 삭제되지 않은 것처럼 보인다.
Stage 12에서는 field range 제거와 화면 마커 제거만 확인했지만, 내부 `Control::Field`가 남아
있으면 저장/재로드나 raw control 기준으로는 누름틀 삭제가 불완전할 수 있다.

## 2. 수정 방향

- 누름틀 삭제는 본문 텍스트를 유지하되 `FieldRange`뿐 아니라 해당 `Control::Field`와
  대응 `ctrl_data_records`도 제거한다.
- 같은 문단의 다른 field range가 제거된 control 뒤에 있으면 `control_idx`를 보정한다.
- `char_offsets`/`char_count`를 재계산해 FIELD_BEGIN/END gap이 사라진 상태를 반영한다.

## 3. 수정 내용

- `remove_field_in_para()`가 삭제 대상 ClickHere의 `FieldRange`, `Control::Field`,
  `ctrl_data_records`를 함께 제거하도록 했다.
- 제거된 control 뒤의 다른 `field_ranges[*].control_idx`를 1씩 당겨 보정한다.
- 제거 후 `rebuild_char_offsets()`를 호출해 FIELD_BEGIN/END gap이 빠진 stream offset을
  재계산한다.
- `rebuild_char_offsets()`의 선행 컨트롤 수 추정을 현재 `controls.len()`으로 clamp해,
  제거 전 stale `char_offsets[0]`이 남아도 과거 control gap이 되살아나지 않게 했다.
- `누름틀-2024.hwp` 첫 ClickHere 삭제 시 텍스트는 유지하고 ClickHere control이 실제로
  줄어드는 회귀 테스트를 추가했다.

## 4. 검증 결과

- `cargo test --test issue_258_clickhere_form_mode removing_clickhere_keeps_text_but_removes_field_control -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test --lib rebuild_`
- `cargo fmt --check`
- `git diff --check`
- 삭제 후 `samples/누름틀-2024.hwp` 첫 문단의 ClickHere field/control이 실제로 줄어드는지 확인
- `wasm-pack build --target web --out-dir pkg`

WASM/Node 확인:

- 삭제 전 ClickHere 2개
- `removeFieldAt(0, 0, 8)` 결과 `{"ok":true}`
- 삭제 후 ClickHere 1개 (`222212212`)만 남음

Playwright `http://localhost:7700/` 확인:

- `누름틀-2024.hwp` 로드 후 `charOffset=8`에서 삭제 확인 대화상자 표시
- `확인` 버튼 클릭 후 첫 ClickHere가 `getFieldList()`에서 제거됨
- 같은 위치 `getFieldInfoAt(0,0,8)` 결과 `{"inField":false}`
- field marker DOM은 `display:none`
- 텍스트 `11223344`는 일반 본문으로 유지됨
- 스크린샷: `/tmp/task258-stage13-delete-confirm.png`
