# Task #948 Stage 4 검증 보고서

- 이슈: [#948](https://github.com/edwardkim/rhwp/issues/948)
- 브랜치: `task-948-pagelayertree-display-text`
- 작성일: 2026-05-18

## 1. 검증 범위

Stage 3 구현의 검증 범위는 다음과 같다.

- PageLayerTree JSON contract 변경 검증
- #947 SVG 렌더링 회귀 보존
- schema minor 갱신 정합
- 전체 Rust 테스트 회귀 확인
- clippy strict 확인
- diff whitespace 확인

## 2. 실행 결과

실행한 명령:

```bash
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

| 명령 | 결과 |
|------|------|
| `cargo test --test issue_948` | 통과 — 1 passed |
| `cargo test paint::json` | 통과 — 9 passed |
| `cargo test page_layer_tree_export` | 통과 — 2 passed |
| `cargo test --test issue_937` | 통과 — 4 passed |
| `cargo test paint::schema` | 통과 — 1 passed |
| `cargo test` | 통과 — lib 1299 passed, 2 ignored + integration/doc tests 통과 |
| `cargo clippy -- -D warnings` | 통과 |
| `git diff --check` | 통과 |

## 3. 확인된 contract

`textRun` JSON 은 다음 계약을 만족한다.

- `text` 는 source text 로 유지
- `positions` 는 source `text` 기준 유지
- `displayText` 는 source text 와 렌더링용 텍스트가 다를 때만 출력
- `displayPositions` 는 `displayText` 기준으로 출력
- `U+F012B` 는 source `text` 에 보존되고 `displayText` 에서는 `(인)` 으로 노출
- `U+F081C` filler 는 `displayText: ""`, `displayPositions: []` 로 노출
- 일반 textRun 은 `displayText` 없이 기존 fallback 을 유지
- `knownFeatures` 에 `text.displayText` 포함
- 실제 display field 가 있는 tree 만 `usedFeatures` 에 `text.displayText` 포함

## 4. 추가 확인 사항

검증 중 `mydocs/orders/20260518.md` 가 upstream 에 이미 존재하는 파일임을 확인했다. 초기 할일 등록 단계에서 기존 내용을 축약해 덮어쓴 상태였으므로, Stage 4 중 upstream 원문을 복원하고 #948 섹션만 추가하는 형태로 정정했다.

현재 diff 상 `mydocs/orders/20260518.md` 변경은 #948 섹션 추가뿐이다.

## 5. 잔여 위험

남은 위험은 낮다.

- JSON consumer 가 기존 `text` / `positions` 만 사용하는 경우 기존 동작 유지
- 신규 consumer 는 `displayText` / `displayPositions` 를 우선 사용 가능
- schema major 변경 없이 minor additive revision 으로만 노출

다음 단계는 최종 보고서 작성과 PR 준비이다.
