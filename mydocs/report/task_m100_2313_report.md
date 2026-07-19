# Task M100 #2313 최종 결과보고서 — legacy `/web` 제거

- 이슈: #2313
- 상위 추적: #2022
- 브랜치: `task2313-legacy-web-removal`
- 기준 브랜치: `upstream/devel`
- upstream 기준 commit: `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20`
- 최종 검증 commit: `060c45aa2067a7a3112a9a1825f9524ec1b0d8d9`
- 완료일: 2026-07-17
- 상태: 로컬 구현·검증 완료, push·PR 생성과 review·merge 대기

## 1. 결론

repository production/build/package에서 소비하지 않던 legacy `/web` 개발 앱 18개 tracked entry를 제거했다.
Git history를 archive로 사용하며 compatibility stub이나 Studio 내부 복사본은 만들지 않았다.

font canonical source는 #2125의 `assets/fonts`를 유지하고 Studio, Chrome, Firefox, Safari, VS Code와 npm
editor의 current 계약은 변경하지 않았다. current CI detector, metrics, font contract와 한/영 manual은 legacy
경로가 없는 tree에 맞게 현행화했다.

fresh WASM과 모든 frontend 소비자 gate가 통과했고 same-base non-legacy 함수 delta는 0건이다. #2313의
코드량·CC 감소는 dead legacy 모집단 삭제 결과이며 current Studio SOLID 개선으로 과장하지 않는다.

## 2. 제거한 범위

| 분류 | 처리 |
|------|------|
| legacy HTML/CSS/JavaScript app | 제거, Studio로 이동·복제하지 않음 |
| Python HTTPS server와 clipboard test | 제거 |
| self-signed cert/key | 제거 |
| tracked generated WASM glue/declaration | 제거, current authority는 ignored `pkg/` |
| `web/fonts` compatibility link | 제거, canonical `assets/fonts` 유지 |

filesystem과 Git index의 `web/` entry는 모두 0이다. Studio source에 남은 `web/editor.html`과
`web/font_substitution.js` 두 표현은 지원 path가 아니라 TypeScript 포팅 출처 provenance다.

## 3. tooling과 문서 변경

### 3.1 current tooling

- `.github/workflows/ci.yml`: 존재하지 않는 `web/` frontend prefix 제거
- `scripts/frontend-metrics.mjs`: `legacy-web` group, legacy exclude와 link metadata 제거
- `scripts/frontend-font-assets.test.mjs`: `web/fonts` assertion 제거, Studio canonical link 유지

schema v2와 Total CC, Top 20, threshold 합·개수, stable function comparator는 유지했다. `assets/fonts`, Studio,
browser extension, VS Code, shared와 npm editor 영향 범위도 유지했다.

### 3.2 current 문서

- 한/영 local server manual을 Studio Vite server만 지원하도록 동기화
- font fallback current ownership을 `assets/fonts`와 Studio link 기준으로 현행화
- #2023 frontend guardrail을 post-removal 상태로 현행화

2026-04-07 font 조사, #2124 공식 snapshot, 과거 plans/reports/working/changelog의 `/web` 기록은 당시 사실인
historical evidence로 보존했다.

## 4. 보존한 계약

- `@rhwp/editor` iframe/MessageChannel public contract와 dependency 0
- React/Vue/Svelte 등 runtime UI framework 미도입
- Studio runtime `fonts/...`, fallback과 CanvasKit registration
- canonical font 36개, 22,651,296 bytes와 license/hash
- Chrome/Firefox CSP, WAR와 `publicDir: false`
- Safari Chrome dist inheritance와 stricter WAR
- VS Code approved 11-font subset와 webview CSP
- Rust/WASM API와 renderer output

기능, 보안 정책과 Phase B hotspot refactor를 legacy 제거 PR에 섞지 않았다.

## 5. 검증 결과

| 검증 | 결과 |
|------|------|
| Docker fresh WASM release build + `wasm-opt` | PASS |
| WASM binding/editor embed static contract | PASS, 3/3 |
| `@rhwp/editor` | PASS, 15/15 |
| shared/Chrome/Firefox service worker | PASS, 88/88 |
| Studio unit/contract | PASS, 298/298 |
| Studio production build | PASS |
| Studio text-flow browser smoke | PASS, 5 assertions |
| iframe embed browser smoke | PASS, 10/10 |
| CanvasKit font coverage | PASS |
| Chrome/Firefox builds | PASS, fonts 각 36개 |
| extension dist contract | PASS, 3/3 |
| VS Code compile | PASS, fonts 11개 |
| full font asset contract | PASS, 4/4 |
| Safari source/manifest/inheritance static gate | PASS |

Safari는 source, manifest와 Xcode project를 변경하지 않았으므로 signed/unsigned Xcode build를 #2313 gate로
추가하지 않았다. Chrome dist 복제, `fonts/*`, stricter WAR와 JavaScript/shell syntax를 검증했다. release/CI가
Safari signing을 요구하는 시점의 build pipeline 개선은 이 task와 독립적이다.

## 6. metrics 결산

maintainer의 #1904/#2130 교훈을 반영해 Max CC만 보지 않고 Total CC, global Top 20, CC>25/100 합·개수와
stable function diff를 함께 비교했다.

| #2313 same-base 직접 delta | 결과 |
|----------------------------|------:|
| included files | -10 |
| reported CC functions | -149 |
| Total CC | -828 |
| global Top 20 합 | -38 |
| CC>25 개수 / 합 | -4 / -207 |
| CC>100 개수 / 합 | 0 / 0 |
| Max CC | 0 |
| non-legacy stable function diff | 0건 |

삭제된 legacy group은 10 files, 6,592 lines, 251 AST functions와 Total CC 828이었다. 149개 CC function
diff는 모두 `web/` removal이다. current 함수의 복잡도를 이동하거나 낮춘 결과는 없다.

#2124 official 대비 current는 Total CC +485, Top 20 +81, CC>25 +7건/+340, CC>100 +1건/+107,
Max 0이다. 이는 official snapshot 이후 upstream 누적치이므로 #2313 성과와 분리한다. official snapshot
artifact는 변경하지 않았다.

## 7. Phase B 입력과 판단

legacy 제외 current 모집단은 214 files, Total CC 12,290이다. Studio가 Total CC의 87.5%이고 global Top 20을
모두 차지한다.

- `diff-engine.ts`: Total CC 897, CC>25 8개 — 총량 hotspot
- `input-handler-mouse.ts`: Max CC 453, 최대 함수 999 LOC — 단일 함수 hotspot
- `input-handler-keyboard.ts`: Max CC 444, 최대 함수 909 LOC — 단일 함수 hotspot
- `picture-props-dialog.ts`: Max CC 348 — 비교적 격리 가능한 dialog 후보
- `input-handler.ts`, `wasm-bridge.ts`: 파일은 크지만 Max CC가 각각 27, 12 — 크기와 함수 복잡도를 분리해 판단

SOLID guide는 책임 분리·의존 방향을 확인하는 정성 기준으로 사용한다. 전체 frontend를 하나의 총점으로
평가하지 않는다. 다음 실행 이슈는 characterization gate와 금지 계약을 먼저 고정하고, 위험과 감소 잠재력을
함께 검토한 뒤 작업지시자 승인으로 분리한다.

## 8. 완료 기준

| 기준 | 판정 |
|------|------|
| tracked root `web/` 제거 | 충족 |
| current direct consumer 0 | 충족 |
| current 한/영 manual에서 legacy 실행 안내 제거 | 충족 |
| CI/metrics/font contract post-removal 정합 | 충족 |
| #2124와 historical evidence 보존 | 충족 |
| frontend package/browser/font gate | 충족 |
| Phase B legacy 제외 모집단 | 충족 |

local 완료 기준은 모두 충족했다. 외부 완료 조건인 PR CI, maintainer review, merge와 issue close는 아직 남아
있다.

## 9. 남은 작업

1. 작업지시자에게 PR·GitHub 코멘트 초안 제시와 승인
2. branch push와 draft PR 생성
3. #2313 진행 코멘트와 maintainer review 요청
4. CI/review 반영과 작업지시자 판단에 따른 Ready/merge
5. merge 후 #2313 완료 근거, close 승인과 #2022 checkbox 갱신
6. 별도 승인 후 Phase B 후보 실행 이슈 결정

계획 문서는 merge 후 `mydocs/plans/archives/`로 이동한다. PR 전에는 archive하지 않는다.
