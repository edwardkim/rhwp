# Task #948 — 최종 보고서

- 이슈: [#948](https://github.com/edwardkim/rhwp/issues/948)
- 마일스톤: M100 / v1.0.0
- 브랜치: `task-948-pagelayertree-display-text`
- 기간: 2026-05-18

## 1. 작업 범위

PageLayerTree JSON 의 `textRun` op 에 source text 와 렌더링용 display text 를 분리하는 additive contract 를 추가했다.

이번 작업의 목표는 모든 PUA 문자의 의미를 추측하는 것이 아니라, rhwp core 가 이미 알고 있는 한컴/HWP PUA 표시 규칙을 renderer-independent paint contract 인 PageLayerTree JSON 에도 전달하는 것이다.

## 2. Root cause

기존 PageLayerTree JSON 은 `textRun` 에 다음 필드만 제공했다.

- `text`: `run.text`
- `positions`: `run.text` 기준 위치
- `clusters`: `run.text` 기준 cluster/source range

이 구조에서는 `U+F012B -> "(인)"` 처럼 source text 와 실제 렌더링 문자열 길이가 달라지는 경우, PageLayerTree 소비자가 다음 중 하나를 선택해야 했다.

- `text` 를 그대로 그려 PUA glyph 를 깨진 문자로 출력
- downstream renderer 에 rhwp core 의 PUA/display text 규칙을 중복 구현

이는 PageLayerTree 를 renderer-independent paint contract 로 쓰려는 방향과 맞지 않는다.

## 3. Fix

### 3.1 display text helper

`src/renderer/composer.rs` 에 `expand_pua_display_text()` 를 추가했다.

처리 범위:

- `U+F081C` TAC filler 제거
- `U+F012B -> "(인)"`
- Hanyang-PUA 옛한글 자모 시퀀스 확장
- 기존 PUA bullet mapping 적용
- 알 수 없는 PUA 는 원문 유지

기존 `expand_pua_render_text()` 는 새 helper 를 호출하도록 유지했다.

### 3.2 PageLayerTree JSON contract

`src/paint/json.rs` 의 `PaintOp::TextRun` 직렬화에 다음 필드를 추가했다.

- `displayText`: display text 가 source text 와 다를 때만 출력
- `displayPositions`: `displayText` 기준 위치 배열

기존 필드는 변경하지 않았다.

- `text`: source text 유지
- `positions`: source text 기준 유지
- `clusters`: source text 기준 유지

`U+F081C` 처럼 출력에서 사라지는 filler 는 `displayText: ""`, `displayPositions: []` 로 직렬화한다.

### 3.3 schema metadata

`src/paint/schema.rs`:

- `schemaMinorVersion`: 11 → 12

`src/paint/json.rs` metadata:

- `knownFeatures` 에 `text.displayText` 추가
- 실제 display field 가 있는 tree 의 `usedFeatures` 에만 `text.displayText` 추가
- `requiredFeatures` 는 변경하지 않음

## 4. 테스트

### 4.1 신규 테스트

`tests/issue_948.rs` 추가:

- `samples/복학원서.hwp` page 0 PageLayerTree JSON 검증
- `U+F012B` source 보존 확인
- `displayText` 에 `(인)` 제공 확인
- `displayText` 에 `U+F012B` / `U+F081C` 미노출 확인
- `positions` 는 source text 기준, `displayPositions` 는 display text 기준 확인
- 일반 textRun 은 display field 없이 fallback 유지 확인

`src/paint/json.rs` 단위 테스트 추가:

- PUA textRun 의 `displayText` / `displayPositions`
- `U+F081C` hidden filler 의 빈 display positions
- 일반 textRun 의 display field 생략

## 5. 검증 결과

실행한 명령:

```text
cargo test --test issue_948
cargo test paint::json
cargo test page_layer_tree_export
cargo test --test issue_937
cargo test paint::schema
cargo test
cargo clippy -- -D warnings
git diff --check
```

결과:

- `issue_948`: 1 passed
- `paint::json`: 9 passed
- `page_layer_tree_export`: 2 passed
- `issue_937`: 4 passed
- `paint::schema`: 1 passed
- 전체 `cargo test`: 통과
  - lib: 1299 passed, 2 ignored
  - integration/doc tests 통과
- `cargo clippy -- -D warnings`: 통과
- `git diff --check`: 통과

## 6. 호환성

이 변경은 additive 이다.

- 기존 소비자는 `text` / `positions` 를 그대로 사용할 수 있다.
- 신규 소비자는 `displayText ?? text`, `displayPositions ?? positions` fallback 규칙으로 렌더링 문자열을 선택할 수 있다.
- schema major 는 유지하고 minor 만 올렸다.
- `requiredFeatures` 는 변경하지 않았다.

## 7. 특이사항

검증 중 `mydocs/orders/20260518.md` 가 upstream 에 이미 존재하는 파일임을 확인했다. 초기 할일 등록 단계에서 기존 내용을 축약해 덮어쓴 상태였으므로, 원문을 복원하고 #948 섹션만 추가하는 형태로 정정했다.

또한 `samples/복학원서.hwp` PageLayerTree 에서 `U+F012B` 와 `(Signature)` 는 동일 textRun 이 아니라 런 단위로 분리되어 있었다. 따라서 통합 테스트는 `U+F012B` source run 의 display contract 를 직접 검증한다.

## 8. 판정

#948 의 목표인 PageLayerTree `textRun` JSON source/display text 분리와 display positions 제공은 완료로 판정한다.

남은 작업은 커밋, push, PR 생성이다.
