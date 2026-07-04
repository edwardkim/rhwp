# PR #1447 처리 보고서 — CanvasKit replay 계약 가드

## 1. 결정

**merge 수용 권고** — 검증 통과. GitHub review 승인, 문서 commit push, CI 재확인 후 merge 진행 대상.

## 2. 변경 본질

CanvasKit direct replay의 확장 범위를 직접 늘리지 않고, replay 계약을 깨뜨리는 변경을 조기에 잡는 Studio-side E2E 계약 테스트를 추가했다.

- `LayerPaintOp` variant 전체가 `CanvasKitLayerRenderer.renderOp`에 명시되는지 확인
- direct replay 대상과 fallback/unsupported 대상의 경계 확인
- `glyphOutline` payload status guard 유지 확인
- CanvasKit direct replay source가 Canvas2D/DOM drawing API에 의존하지 않는지 확인
- `npm run e2e:renderer-contract` script 추가

## 3. 판단 근거

- PR base는 `devel`, merge state는 `CLEAN`.
- 변경은 2파일 +172/-0으로 제한적이다.
- 새 테스트는 현재 renderer contract와 직접 맞물려 있고, public rendering behavior 변경은 없다.
- 관련 이슈는 `Refs #536`이며 자동 close 대상이 아니다.
- @seo-rii의 기존 render 시리즈 맥락(P27 이후 P28 계약 가드)과도 일관된다.

## 4. 검증 결과

| 항목 | 결과 |
|------|------|
| GitHub CI | Build & Test / CodeQL / Render Diff 성공 |
| `git diff --check upstream/devel...pr-1447` | 통과 |
| merge 시뮬레이션 | 충돌 없음 |
| `node --check e2e/renderer-contract.test.mjs` | 통과 |
| `npm run e2e:renderer-contract` | 통과 |
| `wasm-pack build --target web --release` | 통과 |
| `npm run build` | 통과 |
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과 (1879 passed, 6 ignored) |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

`npm ci`에서 low severity audit 알림 1건이 있었지만, 이번 PR은 lockfile을 변경하지 않는다.

## 5. 후속

- GitHub review는 approve 권고.
- 문서 commit을 PR head에 push하면 CI 재실행을 기다린 뒤 merge한다.
- merge 후 #536은 open 유지한다.
