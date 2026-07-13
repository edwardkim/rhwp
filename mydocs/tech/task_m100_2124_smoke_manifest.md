# Task M100 #2124 - frontend smoke manifest

- 이슈: #2124
- 단계: Stage 4 - local gate 완료 / maintainer 승인
- 작성일: 2026-07-10
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 관련 산출물:
  - `mydocs/metrics/frontend/2026-07-11/metrics.json`
  - `mydocs/tech/task_m100_2124_baseline_manifest.md`
  - `mydocs/tech/task_m100_2124_public_contract_snapshot.md`
  - `mydocs/tech/task_m100_2124_wasm_json_schema_snapshot.md`
  - `mydocs/tech/task_m100_2124_font_inventory.md`
  - `mydocs/tech/task_m100_2124_extension_security_snapshot.md`

## 1. 목적

이 문서는 #2124에서 실제 실행한 frontend gate와 Phase A/B에서 변경 유형별로 재사용할 gate를
구분한다. #2124는 runtime·asset 동작을 바꾸지 않지만 generated WASM을 소비하는 모든 package가 같은
fresh output에서 build되는지 확인한다.

## 2. gate 등급

| 등급 | 의미 | 예 |
|------|------|----|
| Local PASS | 현재 branch에서 실제 통과 | fresh WASM, unit/contract test, package build |
| Release/manual | 브라우저·샘플·시각 판정 필요 | E2E, render diff, extension install smoke |
| Advisory | 추세 추적용이며 fail gate 아님 | frontend metrics, 미채점 SOLID evidence |

## 3. fresh WASM 환경

| 항목 | 값 |
|------|----|
| runtime | Colima Docker, Linux arm64 |
| Docker client / server | `29.4.0` / `29.2.1` |
| Docker Compose | `5.1.3` |
| build command | `docker-compose --env-file .env.docker run --rm wasm` |
| build result | PASS, release profile + `wasm-opt`, 약 1분 56초 |

로컬 `wasm-pack`을 직접 실행하지 않고 repository Docker service를 사용했다. 생성된 `pkg/`와 package
`dist/`는 ignored build output이며 commit하지 않는다.

## 4. stale detection과 fresh 판정

초기 기준선 검증(`782059d9`)의 첫 Docker build 전에 `scripts/frontend-wasm-bindings.test.mjs`가 stale
`pkg/rhwp.d.ts`에서 다음 네 Rust explicit export 누락을 탐지했다.

- `flushDeferredPagination`
- `getCursorRectByPathNear`
- `getStructure`
- `insertTextInCellDeferredPagination`

이 pre-build 실패는 tracked source 결함으로 판정하지 않았다. fresh WASM 생성 직후 동일 검사가
통과했고, 이전에 type declaration 때문에 막혔던 Studio build와 VS Code compile도 통과했다. 이 전후
결과는 binding gate가 stale generated output을 정확히 식별한다는 근거다.

최종 기준을 `6f1bd284`로 옮겼을 때는 기존 `pkg`도 explicit export 검사를 통과했다. 다만 upstream
Rust 렌더·편집·cursor rect 변경이 포함돼 repository Docker service로 WASM을 다시 생성했고, 최종
binding과 모든 consumer gate를 그 output에서 재검증했다.

최신 `3077f96d`에도 #2184/#2191의 Rust renderer/layout과 Studio CanvasKit 변경이 포함돼 fresh WASM을
다시 생성했다. binding/editor, Studio build·unit, VS Code compile, Chrome/Firefox build, extension dist와
renderer contract를 같은 output에서 재검증했다.

## 5. 실행 결과

| 명령 | 결과 | 비고 |
|------|------|------|
| `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs` | PASS, 2 tests | fresh binding과 editor contract |
| combined frontend contract/shared tests | PASS, 68 tests | binding 1, editor 1, extension 3, shared SW 63 |
| `npm --prefix rhwp-studio run build` | PASS | fresh WASM TypeScript + Vite production build |
| `npm --prefix rhwp-studio run test` | PASS, 185 tests | Studio unit tests |
| `npm --prefix rhwp-vscode run compile` | PASS | extension host와 webview production compile |
| `npm --prefix rhwp-chrome run build` | PASS | fresh WASM과 36 fonts copy |
| `npm --prefix rhwp-firefox run build` | PASS | fresh WASM과 36 fonts copy |
| `node --test scripts/frontend-extension-dist.test.mjs` | PASS, 3 tests | manifest/CSP/WAR/inline script/font/WASM dist contract |
| `npm --prefix rhwp-studio run e2e:renderer-contract` | PASS | #2184/#2191 CanvasKit 변경 선택 gate |

Chrome/Firefox build의 Vite externalization, asset runtime resolution, chunk-size 메시지는 warning이며 build를
실패시키지 않았다.

## 6. dependency 준비와 audit 기록

package-local `npm ci`는 Chrome, Firefox, VS Code에서 성공했다. npm audit 출력에는 Chrome 1 moderate·1
high, Firefox 1 high, VS Code 1 high가 기록되었다. #2124에서 lockfile을 임의 갱신하거나
`npm audit fix`를 실행하지 않는다. 취약점의 실제 배포 영향은 dependency audit 전용 범위에서 판정한다.

## 7. 정적 검증

다음 파일의 `node --check`가 통과했다.

- `scripts/frontend-metrics.mjs`
- `scripts/frontend-editor-embed.test.mjs`
- `scripts/frontend-extension-dist.test.mjs`
- `scripts/frontend-wasm-bindings.test.mjs`
- Chrome/Firefox `content-script.js`, `sw/fetch-security.js`, `sw/message-router.js`
- Safari `src/background.js`, `src/content-script.js`

metrics schema v2 JSON parse와 자체 비교도 통과했다: aggregate delta 0, function diff 0.

## 8. E2E와 수동 gate

| 표면 | 명령/절차 | 목적 | 등급 |
|------|-----------|------|------|
| Studio E2E | `npm --prefix rhwp-studio run e2e` | 기본 편집/렌더링 흐름 | Release/manual |
| renderer contract | `npm --prefix rhwp-studio run e2e:renderer-contract` | renderer contract 회귀 | 변경 범위에 따라 PR required |
| baseline headless | `npm --prefix rhwp-studio run e2e:baseline:headless` | sample baseline 확인 | Release/manual |
| render diff | `npm --prefix rhwp-studio run e2e:render-diff` | 시각 diff | Release/manual |
| extension install | Chrome/Firefox/Safari unpacked/install | CSP, WAR, font/WASM runtime load | Release/manual |
| VS Code smoke | extension host | webview CSP, asset path, document load | Release/manual |

#2124는 runtime·asset을 바꾸지 않는 기준선 작업이므로 위 수동 항목을 완료 조건으로 승격하지 않는다.
실제 font 이동, legacy `/web` 정리, WASM boundary 또는 extension 동작 변경 PR에서 관련 gate를 선택한다.

## 9. 변경 유형별 최소 gate

| 변경 유형 | 최소 gate |
|-----------|-----------|
| metrics script | syntax, snapshot JSON parse, schema v2 before/after diff |
| font path | Studio/Chrome/Firefox build, VS Code compile, exact 36-file inventory, browser font load smoke |
| extension message/security | shared SW tests, dist contract, extension install, private IP/redirect fetch tests |
| WASM JSON boundary | fresh binding test, Studio unit/build, renderer contract, VS Code compile/smoke |
| `@rhwp/editor` iframe API | editor embed contract, downstream method/byte compatibility, zero runtime dependencies |
| legacy `/web` cleanup | repository dependency scan, font reference map, all consuming package builds |
| large function split | relevant unit/E2E, metrics schema v2 function diff와 총량 변화 |

## 10. #2124 판정

repository Docker fresh WASM, binding/editor/extension/shared contract, Studio unit/build, Chrome/Firefox build,
VS Code compile까지 로컬 자동 gate가 모두 통과했다. #2124는 maintainer/collaborator 리뷰 승인 전까지
진행 중이며, 그 전에 체크리스트 완료·close 또는 #2125 착수를 하지 않는다.

## 11. #2186 이후 embed transport delta gate

#2124의 실행 결과와 metrics snapshot은 다시 생성하지 않는다. #2186처럼 `@rhwp/editor` transport를
바꾸는 PR은 다음 delta gate를 추가로 실행한다.

| 명령 | 검증 표면 |
|------|-----------|
| `node --test scripts/frontend-editor-embed.test.mjs` | 공개 `createEditor` API, zero runtime dependency, exact-origin v1 binary, 제한된 legacy fallback |
| `npm --prefix npm/editor test` | origin/source/session, malformed envelope, transferable, timeout와 destroy cleanup |
| `npm --prefix rhwp-studio test` | Studio protocol guard, RPC router, `hwpctl-load`와 legacy request/response |
| `npm --prefix rhwp-studio run build` | TypeScript/Vite consumer 계약 |
| `npm --prefix rhwp-studio run e2e:embed` | 실제 browser에서 public SDK load/export/destroy와 legacy flow |
| `npm --prefix npm/editor pack --dry-run --json` | `index.js`, `index.d.ts`, `transport.js` package surface |

frontend metrics는 공식 `mydocs/metrics/frontend/2026-07-11/metrics.json`을 `--compare`로만 읽고
`output/frontend-metrics/task2186/`에 임시 결과를 만든다. 따라서 Phase 0 snapshot hash와 내용은
변하지 않는다.
