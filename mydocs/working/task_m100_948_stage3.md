# Task #948 Stage 3 구현 보고서

- 이슈: [#948](https://github.com/edwardkim/rhwp/issues/948)
- 브랜치: `task-948-pagelayertree-display-text`
- 작성일: 2026-05-18

## 1. 구현 요약

PageLayerTree `textRun` JSON 에 source text 와 display text 를 분리하는 additive contract 를 추가했다.

기존 필드는 유지:

- `text`: 기존 `run.text` 원문 유지
- `positions`: 기존 `run.text` 기준 위치 유지
- `clusters`: 기존 source text 기준 유지

신규 필드:

- `displayText`: core 의 PUA display text helper 로 계산한 렌더링용 텍스트
- `displayPositions`: `displayText` 기준 위치 배열

`displayText == text` 인 일반 textRun 은 신규 필드를 생략한다.

## 2. 변경 파일

### `src/renderer/composer.rs`

- `expand_pua_display_text()` 공용 helper 추가
- 기존 `expand_pua_render_text()` 는 새 helper 를 호출하도록 정리
- 처리 범위:
  - `U+F081C` TAC filler 제거
  - `U+F012B -> "(인)"`
  - Hanyang-PUA 옛한글 자모 시퀀스 확장
  - 기존 PUA bullet mapping 적용
  - 알 수 없는 PUA 는 원문 유지

### `src/paint/json.rs`

- `PaintOp::TextRun` 직렬화에 `displayText` / `displayPositions` 추가
- `displayText != text` 인 경우에만 출력
- `displayText == ""` 인 숨김 filler 는 `displayPositions: []` 로 출력
- `text.displayText` 를 `knownFeatures` 에 추가
- 실제 display field 가 있는 tree 에만 `usedFeatures` 에 `text.displayText` 추가
- 기존 `positions` writer 를 source/display 공용 helper 로 분리

### `src/paint/schema.rs`

- PageLayerTree additive revision 으로 `schemaMinorVersion` 11 → 12 갱신

### `tests/issue_948.rs`

- `samples/복학원서.hwp` page 0 PageLayerTree JSON 통합 테스트 추가
- `U+F012B` source 보존, `(인)` display 제공, display/source positions 기준 분리를 검증

## 3. 테스트 보강

`src/paint/json.rs` 단위 테스트 추가:

- PUA textRun 의 `displayText` / `displayPositions` 출력
- `U+F081C` 숨김 filler 의 빈 display text / 빈 display positions 출력
- 일반 textRun 은 display field 를 출력하지 않는 fallback 유지

`tests/issue_948.rs` 통합 테스트 추가:

- PageLayerTree metadata 의 `text.displayText` feature 노출
- source `text` 의 `U+F012B` 보존
- `displayText` 의 `(인)` 제공
- `displayText` 의 `U+F012B` / `U+F081C` 누출 방지
- `positions` 는 source text 기준, `displayPositions` 는 display text 기준

## 4. 검증 결과

실행한 명령:

```bash
cargo test --test issue_948
cargo test paint::json
cargo test page_layer_tree_export
cargo test --test issue_937
cargo test paint::schema
cargo test
```

결과:

- `issue_948`: 1 passed
- `paint::json`: 9 passed
- `page_layer_tree_export`: 2 passed
- `issue_937`: 4 passed
- `paint::schema`: 1 passed
- 전체 `cargo test`: 통과
  - lib: 1299 passed, 2 ignored
  - integration tests 포함 전체 통과

기존 경고는 유지:

- `duplicated attribute`
- `unused_parens`
- 일부 `non_snake_case`
- 일부 `unused_must_use`

본 task 변경으로 새로 발생한 실패는 없다.

## 5. 특이사항

`samples/복학원서.hwp` PageLayerTree 에서 `U+F012B` 와 `(Signature)` 는 동일 textRun 이 아니라 런 단위로 분리되어 있었다. 따라서 통합 테스트는 `U+F012B` source run 의 display contract 를 직접 검증하는 방식으로 작성했다.

## 6. 남은 단계

- Stage 4: 검증 결과 검토 및 필요 시 추가 범위 테스트
- Stage 5: 최종 보고서 작성 및 PR 준비
