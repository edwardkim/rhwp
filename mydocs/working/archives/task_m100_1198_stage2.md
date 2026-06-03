# Stage 2 보고서 — Task M100-1198: Studio 붙여넣기 라우팅과 HTML path API

## 목표

내부 클립보드뿐 아니라 외부 HTML 붙여넣기도 중첩 표 셀에서는 `cellPath`를 사용하게 한다.
붙여넣기 후 커서가 같은 중첩 셀 경로에 남도록 `DocumentPosition` 갱신도 함께 보정한다.

## 변경

### `src/document_core/commands/html_import.rs`

- HTML 파싱 결과 문단을 셀 문단 목록에 삽입하는 공통 헬퍼를 추가했다.
- 기존 `paste_html_in_cell_native(...)`는 얕은 표 셀 경로를 유지하면서 공통 헬퍼를 사용한다.
- 신규 `paste_html_in_cell_by_path_native(...)`를 추가했다.
  - `cellPath`가 가리키는 최종 중첩 셀 문단 목록에 삽입한다.
  - 최외곽 표 dirty, `raw_stream` 무효화, section dirty, pagination 갱신을 수행한다.

### `src/wasm_api.rs`

- `pasteHtmlInCellByPath(section, parentPara, pathJson, charOffset, html)` 바인딩을 추가했다.
- Stage 1의 `pasteInternalInCellByPath(...)`와 같은 `DocumentCore::parse_cell_path(...)` 계약을 사용한다.

### `rhwp-studio/src/core/wasm-bridge.ts`

- `pasteInternalInCellByPath(...)`, `pasteHtmlInCellByPath(...)` 래퍼를 추가했다.

### `rhwp-studio/src/engine/input-handler-keyboard.ts`

- `DocumentPosition.cellPath.length > 1`이면 내부 클립보드 붙여넣기에서 `pasteInternalInCellByPath(...)`를 호출한다.
- HTML 붙여넣기도 중첩 셀에서는 `pasteHtmlInCellByPath(...)`를 호출한다.
- 붙여넣기 결과의 `cellParaIdx`를 `cellParaIndex`와 `cellPath` 마지막 엔트리에 반영한다.
- 업스트림의 navigation shortcut helper와 충돌한 위치는 두 helper를 모두 유지하도록 병합했다.

### `tests/issue_1198_nested_cell_paste.rs`

- 내부 클립보드 path 붙여넣기 테스트에 더해 HTML path 붙여넣기 테스트를 추가했다.

## 업스트림 반영

- `upstream/devel` `c884205d`로 fast-forward했다.
- `Cargo.toml` 버전은 `0.7.13`이다.
- `HEAD...upstream/devel`은 `0 0`이다.
- 업스트림이 새로 추적하게 된 기존 로컬 미추적 파일은 보존 stash로 분리했다.

## 검증

```text
cargo test --test issue_1198_nested_cell_paste -- --nocapture
```

결과:

```text
2 passed
```

```text
cargo test --test issue_850_answer_sheet_name_hit_test issue_850_exam_social_answer_sheet_name_cell_keeps_outer_path -- --nocapture
```

결과:

```text
1 passed
```

```text
cd rhwp-studio && npm test
```

결과:

```text
49 passed
```

```text
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npm run build
```

결과:

```text
wasm-pack: success, rhwp v0.7.13
npm run build: success
```

## 메모

첫 `npm run build`는 로컬 `pkg/` 타입 선언이 최신 업스트림 API(`getShapeBBox`)와 맞지 않아 실패했다.
`pkg/`를 `wasm-pack build --target web --out-dir pkg`로 `0.7.13` 기준 재생성한 뒤 빌드는 통과했다.
`pkg/`는 Git 추적 대상이 아니므로 PR에는 포함하지 않는다.
