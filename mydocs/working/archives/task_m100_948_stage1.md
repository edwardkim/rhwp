# Task #948 Stage 1 진단 보고서

- 이슈: [#948](https://github.com/edwardkim/rhwp/issues/948)
- 브랜치: `task-948-pagelayertree-display-text`
- 기준 커밋: upstream/devel `39d90d9d`
- 작성일: 2026-05-18

## 1. 확인한 경로

PageLayerTree JSON 출력 경로는 다음과 같다.

```text
DocumentCore::get_page_layer_tree_native()
  -> build_page_layer_tree()
  -> LayerBuilder::build()
  -> RenderNodeType::TextRun(run)
  -> PaintOp::TextRun { bbox, run: run.clone() }
  -> PageLayerTree::to_json()
  -> src/paint/json.rs 의 textRun 직렬화
```

현재 `src/paint/json.rs` 의 `PaintOp::TextRun` 직렬화는 다음 기준을 사용한다.

- `text`: `run.text`
- `clusters`: `run.text` 기준
- `positions`: `compute_char_positions(&run.text, &run.style)` 기준

따라서 `U+F012B -> "(인)"` 처럼 source text 와 display text 길이가 달라지는 경우, JSON 소비자는 core 의 display text 의미를 알 수 없다.

## 2. #947 display text 규칙 위치

`src/renderer/composer.rs` 에 이미 #947 계열 helper 가 존재한다.

- `pua_plain_text_display(ch)`
  - `U+F012B -> "(인)"`
- `expand_pua_render_text(text)`
  - `U+F081C` 제거
  - `U+F012B` 같은 한컴 PUA 표시 문자열 확장
  - `map_pua_bullet_char` 적용
- `pua_to_display_text(ch)`
  - CharOverlap 렌더러용 PUA 표시 문자열

또한 `ComposedTextRun` 에는 `display_text: Option<String>` 이 있고, `convert_pua_display_text()` 가 옛한글 PUA / 한컴 PUA 표시 문자열을 계산한다.

하지만 `TextRunNode` 구조체에는 `display_text` 필드가 없고, 렌더 트리 생성 단계에서 `ComposedTextRun.display_text` 는 보존되지 않는다. 현재 built-in renderer 들은 최종 draw 단계에서 `expand_pua_render_text()` 또는 별도 옛한글 helper 를 다시 적용한다.

## 3. 현재 renderer 적용 상태

확인된 적용 경로:

- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/html.rs`
- `src/renderer/canvas.rs`
- `src/renderer/skia/text_replay.rs`

위 경로는 `expand_pua_render_text()` 를 사용해 `U+F012B` 와 `U+F081C` 렌더링을 처리한다.

반면 `src/paint/json.rs` 는 이 helper 를 사용하지 않는다.

## 4. 현행 테스트 결과

실행:

```bash
cargo test --test issue_937
cargo test page_layer_tree_export
```

결과:

- `issue_937`: 4 passed
- `page_layer_tree_export`: 2 passed

의미:

- SVG 계열 #947 회귀는 현재 통과한다.
- PageLayerTree 기본 schema/export 테스트는 통과하지만, display text contract 검증은 아직 없다.

## 5. 결론

Task #948 의 root cause 는 PageLayerTree JSON 직렬화가 source text 만 내보내고 display text 를 별도 contract 로 노출하지 않는 것이다.

구현은 다음 방향이 적절하다.

1. renderer 공용 display text helper 를 PageLayerTree JSON에서도 사용한다.
2. `text` / `positions` 는 기존 source-compatible 의미로 유지한다.
3. `displayText` / `displayPositions` 를 additive field 로 추가한다.
4. `displayText != text` 인 경우에만 신규 필드를 출력해 JSON 크기 증가를 줄인다.
5. 회귀 테스트는 `samples/복학원서.hwp` page 0 의 PageLayerTree JSON에서 `U+F012B` source 보존과 `(인)` display 제공을 동시에 검증한다.

## 6. Stage 2 구현 계획서 후보

구현 계획서에서 확정할 세부 항목:

- helper 위치: `composer.rs` 유지 + JSON 경로에서 재사용할지, 별도 text utility 로 이동할지
- 옛한글 PUA까지 `displayText` contract 에 포함할지 여부
- `displayPositions` 계산 함수 시그니처
- 테스트 위치: `tests/issue_937.rs` 확장 또는 `tests/issue_948.rs` 신규
