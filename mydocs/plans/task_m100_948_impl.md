# 구현 계획서 — Task #948: PageLayerTree textRun displayText/displayPositions contract

- 이슈: [#948](https://github.com/edwardkim/rhwp/issues/948)
- 수행 계획서: `mydocs/plans/task_m100_948.md`
- Stage 1 보고서: `mydocs/working/task_m100_948_stage1.md`
- 브랜치: `task-948-pagelayertree-display-text`

## 1. 구현 목표

PageLayerTree JSON 의 `textRun` op 에 source text 와 draw/display text 를 분리해 제공한다.

기존 필드 의미는 바꾸지 않는다.

- `text`: 기존처럼 `run.text` 원문
- `positions`: 기존처럼 `run.text` 기준 위치
- `clusters`: 기존처럼 `run.text` 기준 cluster/source range

신규 additive field:

- `displayText`: 실제 downstream renderer 가 draw 해야 하는 텍스트
- `displayPositions`: `displayText` 기준 위치

소비자 fallback:

```text
drawText = displayText ?? text
drawPositions = displayPositions ?? positions
```

## 2. 수정 파일

### 2.1 `src/renderer/composer.rs`

렌더링용 plain text helper 를 공용 함수로 추가한다.

후보 함수:

```rust
pub fn expand_pua_display_text(text: &str) -> String
```

처리 순서:

1. `U+F081C` 는 출력에서 제거
2. `pua_plain_text_display(ch)` 적용
   - `U+F012B -> "(인)"`
3. `map_pua_old_hangul(ch)` 적용
4. `layout::map_pua_bullet_char(ch)` 적용
5. 알 수 없는 PUA 는 원문 유지

기존 `expand_pua_render_text()` 는 호환성을 위해 유지하되, 가능하면 새 helper 를 호출하도록 정리한다.

주의:

- `effective_text_for_metrics()` 의 `U+F081C` 원문 측정 예외는 유지한다.
- CharOverlap 전용 숫자 변환(`pua_to_display_text`)은 기존 역할을 유지한다.

### 2.2 `src/paint/json.rs`

`PaintOp::TextRun` 직렬화에 display field 를 추가한다.

구현 후보:

```rust
fn display_text_for_text_run(run: &TextRunNode) -> Option<String>
```

규칙:

- `composer::expand_pua_display_text(&run.text)` 결과가 `run.text` 와 다르면 `Some(display)`
- 같으면 `None`
- `displayText` 는 `display_text_for_text_run()` 이 `Some` 인 경우에만 출력
- `displayPositions` 는 동일 display 문자열 기준 `compute_char_positions(display, &run.style)` 로 출력
- 단, `displayText == ""` 인 숨김 filler 케이스는 `displayPositions: []` 로 출력

신규 helper:

```rust
fn write_text_positions_for_text(buf: &mut String, text: &str, style: &TextStyle)
```

기존 `write_text_positions(buf, run)` 은 source-compatible 유지하되 내부에서 새 helper 를 재사용한다.

`clusters` 는 이번 task 에서 변경하지 않는다. source range contract 는 원문 기준이어야 하며, display text 는 별도 draw hint 로만 제공한다.

### 2.3 `src/paint/schema.rs`

PageLayerTree additive contract 이므로 schema minor 를 올린다.

- `schema_minor_version: 11 -> 12`
- `layer_tree_schema_contract_is_stable` 테스트 갱신

### 2.4 `src/paint/json.rs` metadata

`usedFeatures` / `knownFeatures` 에 display text feature 를 반영한다.

제안 feature name:

```text
text.displayText
```

규칙:

- `knownFeatures` 에 항상 포함
- 실제 display field 가 존재하는 tree 에만 `usedFeatures` 에 포함
- `requiredFeatures` 는 비워 둔다. 기존 소비자는 `text` / `positions` fallback 이 가능해야 한다.

이를 위해 text feature 탐색 구조체에 `has_display_text` 를 추가한다.

## 3. 테스트 계획

### 3.1 `tests/issue_948.rs` 신규

fixture:

- `samples/복학원서.hwp`, page 0

검증:

1. PageLayerTree JSON 에 원문 `U+F012B` 가 `text` 또는 `textSources` 에 보존됨
2. `displayText` 가 존재하고 `(인)(Signature)` 를 포함
3. `displayText` 에 `U+F012B` / `U+F081C` 가 포함되지 않음
4. `displayPositions` 가 존재
5. `displayPositions` 원소 수가 `displayText.chars().count() + 1` 과 일치하는 textRun 이 존재
6. 일반 textRun 은 `displayText` 를 불필요하게 출력하지 않음

JSON 파싱은 `serde_json::Value` 를 사용한다. string contains 기반 검증만으로는 `displayPositions` 길이와 해당 `displayText` 의 pair 를 안정적으로 검증하기 어렵기 때문이다.

### 3.2 `src/paint/json.rs` 단위 테스트 추가

작은 synthetic `TextRunNode` 로 다음을 고정한다.

- `text = "\u{F012B}(Signature)"`
- JSON 에 `text:"\uF012B(Signature)"` 보존
- JSON 에 `displayText:"(인)(Signature)"` 출력
- `positions` 는 source 기준
- `displayPositions` 는 display 기준
- `usedFeatures` 에 `text.displayText` 포함

또 다른 synthetic 일반 textRun:

- `text = "ABC"`
- `displayText` 미출력
- `text.displayText` feature 미사용

## 4. 검증 명령

우선 실행:

```bash
cargo test --test issue_948
cargo test --test issue_937
cargo test page_layer_tree_export
cargo test -p rhwp paint::json
cargo test -p rhwp paint::schema
```

최종 최소 검증:

```bash
cargo test
```

시간이 길거나 환경 문제가 있으면 한정 테스트 결과와 미실행 범위를 보고한다.

## 5. 호환성 판단

이 변경은 additive 이다.

- 기존 소비자: `text` / `positions` 계속 사용 가능
- 신규 소비자: `displayText` / `displayPositions` 우선 사용 가능
- schema major 는 유지
- schema minor 만 additive revision 으로 갱신
- `requiredFeatures` 는 변경하지 않음

## 6. 위험 및 대응

| 위험 | 대응 |
|------|------|
| source range 와 display text range 혼동 | `clusters` 는 source 기준 유지, displayPositions 만 display 기준 명시 |
| JSON 크기 증가 | `displayText != text` 인 경우에만 신규 field 출력 |
| helper 중복 | `composer.rs` 공용 helper 를 JSON / 렌더러에서 공유 |
| `U+F081C` 측정 회귀 | `effective_text_for_metrics()` 예외 유지, display helper 에서만 제거 |
| schema feature drift | schema minor + `knownFeatures` / `usedFeatures` 테스트로 고정 |

## 7. 구현 순서

1. `composer.rs` 에 공용 display text helper 추가 및 기존 helper 정리
2. `json.rs` 에 display text 산출 / display positions writer 추가
3. `json.rs` metadata feature flag 추가
4. `schema.rs` minor version 갱신
5. synthetic JSON 단위 테스트 추가
6. `tests/issue_948.rs` 통합 테스트 추가
7. 한정 테스트 실행
8. Stage 3 보고서 작성
