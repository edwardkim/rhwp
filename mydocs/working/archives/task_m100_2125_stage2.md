# Task M100 #2125 Stage 2 완료 보고 — canonical font 이전

- 이슈: #2125
- 브랜치: `task2125-assets-fonts-canonical`
- source 기준: `upstream/devel` `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- 계획 commit: `ecb3f70b9ae2419ff2994d2370d5e9c0dbd26207`
- Stage 1 commit: `6f1aec9b`
- 완료일: 2026-07-13
- 상태: Stage 2 완료, Stage 3 승인 대기

## 1. 수행 내용

1. `web/fonts`의 36개 WOFF2와 2개 문서를 `assets/fonts`로 byte 변경 없이 이동했다.
2. `web/fonts -> ../assets/fonts`, `rhwp-studio/public/fonts -> ../../assets/fonts` 링크를 구성했다.
3. Chrome, Firefox, VS Code build와 subset 생성 도구의 source를 canonical path로 변경했다.
4. extension dist test, CanvasKit coverage, frontend metrics의 source ownership을 갱신했다.
5. canonical/link/distribution hash를 검증하는 `scripts/frontend-font-assets.test.mjs`를 추가했다.
6. `assets/fonts` 변경이 CI를 시작하고 frontend gate로 분류되도록 workflow trigger와 detector를 함께 갱신했다.
7. Render Diff의 font trigger를 canonical path로 변경했다.

## 2. asset 보존 결과

| 항목 | 결과 |
|------|------|
| canonical WOFF2 | 36개 |
| total bytes | 22,651,296 |
| `e750e02f:web/fonts` 대비 filename mismatch | 0 |
| `e750e02f:web/fonts` 대비 bytes/hash mismatch | 0 |
| `NotoSansKR-Regular.woff2` SHA-256 | `d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a` |
| canonical 문서 | `FONTS.md`, `SourceHanSerifK-OFL.txt` 보존 |
| legacy link | `web/fonts -> ../assets/fonts` |
| Studio link | `rhwp-studio/public/fonts -> ../../assets/fonts` |

## 3. CI와 계약 변경

`assets/fonts` 단독 변경이 누락되지 않도록 두 gate를 함께 수정했다.

- workflow `paths-ignore`의 broad `assets/**`를 store image 하위 경로별 ignore로 축소했다.
- frontend preflight detector에 `assets/fonts/`를 추가했다.
- Chrome, Firefox, Studio, VS Code artifact 생성 후 font asset contract test를 실행한다.
- Render Diff path filter를 `web/fonts/**`에서 `assets/fonts/**`로 변경했다.

신규 contract test는 Stage 4 full build 이후 다음을 검증한다.

- canonical count, total bytes, license 파일, #2190 Noto hash
- 두 compatibility symlink의 exact target
- Studio/Chrome/Firefox의 36개 font filename, bytes, SHA-256
- VS Code의 승인된 11개 subset filename, bytes, SHA-256

Safari는 Chrome dist를 상속하는 기존 build 구조를 유지하며 Stage 4 로컬 build 결과를 별도로 비교한다.

## 4. 정적 검증

| 검증 | 결과 |
|------|------|
| old source와 canonical filename/bytes/SHA-256 비교 | PASS, mismatch 0 |
| canonical inventory·symlink contract test | PASS, 2 tests |
| JavaScript/MJS `node --check` | PASS |
| Python source compile | PASS |
| `git diff --check` | PASS |
| `actionlint .github/workflows/ci.yml` | PASS |
| `actionlint -shellcheck= .github/workflows/render-diff.yml` | PASS |
| 전체 `actionlint` | 기준 브랜치와 동일한 기존 `SC2086` 1건 |
| runtime URL/fallback/manifest/package 범위 guard | PASS, diff 0 |

`render-diff.yml:296`의 `SC2086`는 `upstream/devel` 원문에도 동일하게 재현된다. Stage 2 변경은 path
filter 한 줄이며 해당 shell block은 수정하지 않았다.

## 5. 이 단계에서 실행하지 않은 검증

Stage 2는 canonical move와 source consumer의 정적 정합성을 고정하는 단계다. 다음 artifact 검증은 계획서의
Stage 4에서 fresh build 후 실행한다.

- Studio production build와 browser smoke/E2E
- Chrome/Firefox/VS Code package build와 dist contract test
- Safari local build와 Chrome parity 비교
- frontend metrics 재수집과 source/runtime reference 최종 대조

## 6. 범위 보존과 다음 단계

다음 runtime 계약에는 diff가 없다.

- Studio와 legacy `/web`의 `fonts/...` URL 및 fallback 정책
- extension manifest WAR/CSP와 VS Code webview runtime path
- `@rhwp/editor` package와 MessageChannel 계약
- Rust/WASM source와 public API

현재 운영 문서의 `web/fonts` 표현은 Stage 3 범위이므로 아직 수정하지 않았다. #2124 snapshot과 과거 plan,
report, feedback 등 역사 문서는 당시 사실을 보존한다. 작업지시자 승인 전에는 Stage 3 운영 문서 갱신을
시작하지 않는다.
