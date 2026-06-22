## 요약

Closes #1470

- 한컴 도움말 기준 스타일 적용/편집 흐름을 보정해 왼쪽 여백 값이 배율처럼 두 배로 보이는 문제와 줄간격 미반영 문제를 focused 회귀로 고정했습니다.
- 표 생성 상세 옵션, TAC 표 중복 렌더, 표가 다음 페이지로 밀리는 흐름을 보정했습니다.
- 표 캡션 AutoNumber를 유지하면서 `표 1`, `표 2` 형태로 표시하고, 중간 캡션 삭제 후 남은 번호가 다시 1부터 재정렬되도록 했습니다.
- PR #1446에서 들어왔던 `모양 붙여넣기` 명시 경로가 사라진 회귀를 복구했습니다.

## 주요 변경

- 스타일 적용/편집
  - 스타일의 왼쪽 여백/줄간격 값을 문단 속성에 정확히 반영합니다.
  - 스타일 적용/수정 시 기존 직접 글자 서식을 보존합니다.
  - 선택 블록 대상 문단에 스타일을 순회 적용합니다.
- 표/캡션
  - `createTableEx` 상세 옵션 전달과 표 크기 적용을 보정했습니다.
  - TAC 표가 캡션과 함께 있어도 같은 표 컨트롤이 한 번만 렌더되도록 했습니다.
  - 표 캡션 생성/수정/삭제 뒤 AutoNumber를 재배정합니다.
  - 표 캡션 기본 문단을 `"표  "` + AutoNumber 앵커 구조로 생성합니다.
- Studio 모양복사
  - `EditorContext.hasCopiedFormat`과 `InputHandler.hasCopiedFormat()`을 복구했습니다.
  - `InputHandler.performFormatPaste()`와 `edit:format-paste` 커맨드를 복구했습니다.
  - 편집 메뉴와 기본/표 셀 컨텍스트 메뉴에 `모양 붙여넣기`를 복구했습니다.
  - 기존 `Alt+C` 일회성 모양복사 동작은 유지했습니다.
  - `format-paste-availability.ts`와 `format-paste-command.test.ts`를 추가해 모양 붙여넣기 명시 경로가 다시 사라지지 않도록 회귀 테스트를 고정했습니다.

## 수동 검증 절차

1. 로컬 앱 준비
   - 필요 시 `wasm-pack build --target web --out-dir pkg`
   - `cd rhwp-studio && ./node_modules/.bin/vite --host 0.0.0.0 --port 7700 --force`
   - 브라우저/IAB에서 `http://localhost:7700/` 접속

2. 스타일 검증
   - 새 문서에서 본문 문단을 만든다.
   - 스타일 편집/추가에서 왼쪽 여백을 `15`로 지정하고 줄간격 값을 변경한다.
   - 스타일을 적용한 뒤 다시 속성을 확인한다.
   - 기대값: 왼쪽 여백이 `30`으로 바뀌지 않고 `15` 기준으로 유지되며, 줄간격이 화면에 반영된다.
   - 직접 글자 서식이 있는 문단에 스타일을 적용해도 글자 서식이 불필요하게 초기화되지 않는다.

3. 표 생성/TAC 검증
   - 표 만들기 상세 옵션으로 표를 생성한다.
   - 기대값: 표가 현재 위치에 생성되고 다음 페이지로 밀리지 않는다.
   - TAC 옵션이 켜진 표와 캡션이 있는 TAC 표를 각각 생성한다.
   - 기대값: 같은 표가 중복으로 렌더되지 않는다.

4. 표 캡션 검증
   - 표 3개를 만들고 각 표에 캡션을 넣는다.
   - 기대값: 캡션이 `표 1`, `표 2`, `표 3` 형태로 보인다.
   - 중간 표의 캡션을 삭제한다.
   - 기대값: 남은 캡션이 `표 1`, `표 2`로 재정렬되고 stale `표 3`이 남지 않는다.
   - 마지막 표 캡션의 방향/폭/간격/세로 정렬을 수정한다.
   - 기대값: AutoNumber가 사라지지 않고 번호와 캡션 속성이 유지된다.
   - 그림 캡션도 함께 확인한다.
   - 기대값: 기존처럼 `그림 1`, `그림 2` 형태를 유지한다.

5. 모양복사/모양 붙여넣기 검증
   - 모양 복사 전에는 `모양 붙여넣기`가 비활성인지 확인한다.
   - 서식이 있는 원본 텍스트에 커서를 두고 `Alt+C` 또는 `모양 복사`를 실행한다.
   - 대상 텍스트를 선택하고 편집 메뉴 또는 우클릭 메뉴의 `모양 붙여넣기`를 실행한다.
   - 기대값: 복사된 글자/문단 모양이 대상에 적용되고, 실패 시 현재 위치 모양을 새로 복사하지 않는다.
   - 표 셀에서도 원본 셀의 셀 속성/테두리/배경을 복사한 뒤 셀 선택 범위에 `모양 붙여넣기`를 실행한다.
   - 기대값: 선택된 셀에만 셀 속성이 적용되고 선택 밖 셀은 유지된다.

## 검증

- `cargo fmt && cargo test --release issue_1470_table_caption --lib`
- `cargo test --release issue_1470 --lib`
- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test` (110 passed)
- `git diff --check`
- 사용자 직접 수동 검증 완료
