# PR #1447 검토 — CanvasKit replay 계약 가드

- 작성일: 2026-06-21
- 컨트리뷰터: [@seo-rii](https://github.com/seo-rii) (Seohyun Lee)
- PR: https://github.com/edwardkim/rhwp/pull/1447
- base/head: `devel` `23dc197f` ← `seo-rii:render-p28` `21cda64c`
- 관련 이슈: `Refs #536` (멀티 렌더러 지원 트래킹, 자동 close 대상 아님)
- 규모: +172 / -0, 2 files
- 상태: open, draft 아님, `MERGEABLE` / `CLEAN`

## 1. PR 정보

변경 파일:

| 파일 | 변경 |
|------|------|
| `rhwp-studio/e2e/renderer-contract.test.mjs` | CanvasKit renderer 계약 테스트 신규 |
| `rhwp-studio/package.json` | `npm run e2e:renderer-contract` script 추가 |

커밋:

| SHA | 내용 | 작성자 |
|-----|------|--------|
| `f1ae3ce5` | `test(render): guard CanvasKit replay contract` | @seo-rii |
| `21cda64c` | `Merge branch 'devel' into render-p28` | @jangster77 |

## 2. 변경 내용 분석

이번 PR은 CanvasKit 직접 replay 범위를 넓히는 변경이 아니라, 다음 확장 PR에서 계약을 놓치지 않도록 하는 정적 가드다.

검사 내용:

1. `LayerPaintOp` union에 있는 모든 variant가 `CanvasKitLayerRenderer.renderOp`의 `switch (op.type)`에 명시적으로 존재하는지 검사한다.
2. 현재 CanvasKit 직접 replay 대상(`rectangle`, `ellipse`, `path`, `image`, `textRun`, `footnoteMarker`, `formObject`, `placeholder`, `pageBackground`, `line`)은 `this.render*` 경로로 dispatch되는지 검사한다.
3. 아직 직접 replay하지 않는 text sidecar/special visual op(`equation`, `rawSvg`, `charOverlap`, `glyphRun`, `tabLeader`, `textControlMark`, `textDecoration`)은 `unsupportedOps` fallback 경로에 남는지 검사한다.
4. `glyphOutline`은 payload status 확인 뒤 `colorLayers`에서만 직접 replay하고, 나머지는 unsupported 진단으로 남는 계약을 검사한다.
5. `src/view/canvaskit-renderer.ts`와 `src/view/canvaskit/**/*.ts`에서 Canvas2D/DOM drawing API 의존이 새로 들어오지 않는지 금지 패턴으로 검사한다.

## 3. 검토 의견

### 수용 근거

- PR 목적이 명확하다. 렌더링 동작 변경이 아니라 CanvasKit replay 계약을 보호하는 테스트 추가다.
- 현재 `LayerPaintOp` variant와 `renderOp` case 목록이 일치하며, 직접 replay/fallback 구분도 기존 구현과 맞다.
- Canvas2D/DOM API 금지 패턴은 CanvasKit direct replay helper가 Canvas2D 경로에 우회 의존하지 않도록 막는 데 유효하다.
- GitHub CI가 모두 성공했다.
  - CI / Build & Test: success
  - CodeQL: rust, javascript-typescript, python success
  - Render Diff / Canvas visual diff: success
  - WASM Build: skipped (CI 조건상 skip)
- 로컬에서도 PR 작성자가 제시한 검증과 저장소 merge 전 검증을 모두 통과했다.

### 리스크

- 테스트가 TypeScript AST parser가 아니라 정규식 기반이다. 현재 코드 구조에는 충분히 맞지만, `LayerPaintOp` union이나 `renderOp` case 작성 스타일이 크게 바뀌면 테스트 자체를 함께 정비해야 한다.
- 이 PR은 replay coverage를 추가하지 않는다. `GlyphRun`, `equation`, `rawSvg`, 특수 text visual op는 계속 fallback/unsupported 경로가 의도된 상태다.

## 4. 로컬 검증

검증 기준 브랜치: `pr1447-merge-test` (`21cda64c`, `upstream/devel` `23dc197f` 포함)

| 항목 | 결과 |
|------|------|
| `git diff --check upstream/devel...pr-1447` | 통과 |
| merge 시뮬레이션 (`pr-1447` + `upstream/devel`) | 충돌 없음, already up to date |
| `npm ci` | 설치 성공, 기존 lock 기준 low severity 1건 audit 알림 |
| `node --check e2e/renderer-contract.test.mjs` | 통과 |
| `npm run e2e:renderer-contract` | 통과 (`renderer backend contract guard passed`) |
| `wasm-pack build --target web --release` | 통과, `pkg/` 생성 |
| `npm run build` | 통과 (`tsc && vite build`) |
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과 (1879 passed, 6 ignored) |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

비고: 최초 `npm run build`는 로컬 `pkg/` 부재로 `@wasm/rhwp.js` 해석 실패가 났다. 절차대로 `wasm-pack build --target web --release` 후 재실행하여 통과했다. 이는 PR 코드 결함이 아니라 로컬 WASM 산출물 부재 문제다.

## 5. 권고

**수용 / merge 권고.**

변경 범위가 작고 PR 목적과 테스트가 일치한다. CI와 로컬 검증이 모두 통과했으며, 공개 렌더링 동작 변경이나 native/PDF export 변경은 없다. `Refs #536`은 트래킹 참조이므로 merge 후에도 이슈 close는 하지 않는다.
