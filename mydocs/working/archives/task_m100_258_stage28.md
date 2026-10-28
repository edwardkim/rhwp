# Task M100-258 Stage 28 — 인접 누름틀 복사/붙여넣기 표시 보정

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-16
- 선행 커밋: `9b1f0c3d` (`task 258: 누름틀 선택 색상 회귀 보정`)

## 1. 문제

인접한 누름틀 `[123][123]`을 선택해 복사/붙여넣기하면 붙여넣은 위치에서 두 누름틀이 모두
정상적으로 보이지 않는다. 붙여넣기 직후에는 한쪽 누름틀이 보이지 않고, 보이는 누름틀을 하나
삭제하면 숨겨져 있던 누름틀이 뒤늦게 보인다.

## 2. 수정 방향

- 인접 누름틀 선택 복사 시 HTML/내부 클립보드에 두 필드의 경계가 분리되어 보존되는지 확인한다.
- 붙여넣기 시 새 필드들의 `startCharIdx/endCharIdx`, 값, 표시 문자열이 겹치지 않는지 확인한다.
- 렌더/active field 캐시가 붙여넣은 필드 중 하나를 guide/active 상태로 잘못 숨기는지 확인한다.
- 원인이 확인되면 복사/붙여넣기 경계 처리 또는 누름틀 표시 상태 갱신을 보정한다.

## 3. 원인

- 내부 클립보드의 `Control::Field.field_id`를 그대로 붙여넣어 원본 누름틀과 복사본 누름틀의 ID가 중복됐다.
- `split_at`/`merge_from` 이후 누름틀 `field_ranges`가 남아 있는 문단의 `char_offsets/char_count`가
  FIELD_BEGIN/FIELD_END gap 기준으로 재계산되지 않아 renderer가 줄의 마지막 텍스트 범위를 너무 짧게 잡았다.
  이 때문에 `[123][123]` 중 두 번째 `123`이 모델에는 있으나 SVG/화면에는 보이지 않았다.

## 4. 수정

- 붙여넣기 직전 내부 클립보드 문단의 모든 필드 ID를 현재 문서의 최대 field ID 이후 값으로 재부여한다.
- 본문/셀/중첩 셀 붙여넣기 경로에 같은 ID 재부여를 적용한다.
- 본문/셀 붙여넣기와 문단 분리 후 누름틀 `field_ranges`가 있는 영향 문단은 `rebuild_char_offsets`로
  FIELD_BEGIN/FIELD_END gap을 다시 반영한다.
- `copying_adjacent_clickheres_preserves_separate_pasted_fields` 회귀 테스트를 추가해 다음을 고정했다.
  - 원본 2개 + 붙여넣은 누름틀 2개가 서로 다른 field ID를 갖는다.
  - 붙여넣은 문단의 두 누름틀 범위가 `0..3`, `3..6`으로 분리된다.
  - SVG 렌더에 원본/붙여넣기 합산 `1/2/3` 글자가 각각 4개씩 보인다.
  - 붙여넣은 누름틀 하나를 삭제해도 남은 누름틀 값이 즉시 보존된다.

## 5. 검증 결과

- `abc[123][123]` 구성 후 `123123` 범위 복사/붙여넣기 재현
- 붙여넣기 후 필드 목록이 원본 2개 + 복사본 2개로 분리되고 각 값이 `123`인지 확인
- 붙여넣은 필드 중 하나 삭제 후 남은 필드가 처음부터 정상 표시되는지 확인
- 작업지시자 시각 확인: 복사/붙여넣기 표시 정상
- `cargo test --test issue_258_clickhere_form_mode`: 통과 (12 passed)
- `wasm-pack build --target web --out-dir pkg`: 통과
- `cd rhwp-studio && npm run build`: 통과
- Browser plugin: `http://localhost:7700/`, title `rhwp-studio`, toolbar 표시, console error/warn 없음
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

## 6. 후속

- Home/End 키가 누름틀 경계 밖 첫/끝 컬럼으로 이동하지 않는 문제는 Stage29에서 별도 보정한다.
