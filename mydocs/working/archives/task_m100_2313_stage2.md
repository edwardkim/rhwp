# Task M100 #2313 Stage 2 완료 보고 — legacy tree와 tooling coupling 제거

- 이슈: #2313
- 상위 추적: #2022
- 브랜치: `task2313-legacy-web-removal`
- 기준 브랜치: `upstream/devel`
- upstream 기준 commit: `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20`
- Stage 1 commit: `0c1e055b94dcd0ffabd32d4ce7bc2545c5cfe5c7`
- 완료일: 2026-07-17
- 상태: Stage 2 local 완료, Stage 3 승인 대기

## 1. Stage 2 판정

tracked `web/` 18개 entry를 모두 제거하고 current CI detector, frontend metrics와 font contract에서 legacy
경로 결합을 제거했다. filesystem과 Git index 모두 `web/` entry가 0이며 `assets/fonts`와 Studio public
link는 변경되지 않았다.

schema v2 pre/post compare는 계획한 구조적 삭제 delta와 정확히 일치했다. 149개 function diff는 모두
삭제된 `web/` entry이고 non-legacy function diff는 0건이다. 따라서 수치 감소는 current code의 함수 해체
성과가 아니라 legacy 모집단 삭제 결과다.

current 한/영 manual과 guardrail은 Stage 3 범위이므로 아직 수정하지 않았다. fresh WASM과 package/browser
gate도 Stage 4 범위로 남겨 두었다.

## 2. upstream과 변경 범위

| 항목 | 결과 |
|------|------|
| Stage 2 시작 `upstream/devel` | `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20` |
| merge-base | 동일 commit |
| branch 상태 | upstream 대비 ahead 2, behind 0 |
| upstream drift | 0 |
| product source 수정 | 0 |
| current 문서 수정 | 0 |
| 변경 관리 파일 | CI 1, metrics 1, font contract 1 |
| 삭제 | tracked `web/` 18 entries |

## 3. legacy tree 제거

다음 분류를 `git rm -r web`으로 함께 제거했다.

| 분류 | 개수 | 처리 |
|------|-----:|------|
| legacy JS/CSS/HTML app | 10 | 삭제, Studio로 이동·복제하지 않음 |
| Python server, clipboard test, cert/key | 4 | 삭제 |
| tracked generated WASM glue/declaration | 3 | 삭제, current authority는 ignored `pkg/` |
| `web/fonts` compatibility symlink | 1 | 삭제, canonical `assets/fonts` 유지 |

검증 결과:

- filesystem `web`: 없음
- `git ls-files web`: 출력 0
- repository 내부 archive/stub/redirect: 추가 0
- `rhwp-studio/public/fonts`: `../../assets/fonts` 유지

## 4. current tooling 변경

### 4.1 frontend CI detector

`.github/workflows/ci.yml`의 `directoryPrefixes`에서 존재하지 않게 된 `web/` 한 줄만 제거했다.

- `assets/fonts/`, Studio, Chrome, Firefox, Safari, VS Code, shared와 npm editor prefix 유지
- `src/wasm_api.rs`, `Cargo.lock`, workflow 자체와 `scripts/frontend-*.mjs` 판정 유지
- required job 이름, trigger와 preflight 의미 변경 없음

이번 branch는 workflow와 frontend script 자체를 변경하므로 PR에서 frontend gate 대상이 된다.

### 4.2 frontend metrics

`scripts/frontend-metrics.mjs`에서 다음 legacy 전용 처리만 제거했다.

- `legacy-web` include group과 `isLegacyWebFile()`
- tracked generated glue 3개, clipboard test와 `web/fonts` exclude label/분기
- `isTestFile()`의 clipboard exact path
- font reference map의 `web/fonts` pattern
- `fontAssets.legacyWebLink`와 link read

generic `certs`, test/e2e, generated `pkg`, `assets/fonts`, icons와 locale exclude 규칙은 유지했다.
`studioPublicLink`, canonical font inventory, license fingerprint와 schema v2 function comparator도 유지했다.

### 4.3 font contract

`scripts/frontend-font-assets.test.mjs`의 link test를 Studio 단독 계약으로 바꿨다.

- 제거: `web/fonts -> ../assets/fonts` assertion
- 유지: `rhwp-studio/public/fonts -> ../../assets/fonts`
- 유지: canonical 36개·bytes·license·Noto hash
- 유지: Studio/Chrome/Firefox dist와 VS Code 11개 subset hash gate

## 5. 정적 검증

| 검증 | 결과 |
|------|------|
| `test ! -e web` | PASS |
| `git ls-files web` | PASS, 0 |
| `readlink rhwp-studio/public/fonts` | PASS, `../../assets/fonts` |
| `node --check scripts/frontend-metrics.mjs` | PASS |
| `node --check scripts/frontend-font-assets.test.mjs` | PASS |
| `actionlint .github/workflows/ci.yml` | PASS |
| `git diff --check` | PASS |
| canonical inventory + Studio link selected tests | PASS, 2/2 |

selected font tests는 build artifact가 필요 없는 canonical inventory와 Studio link만 실행했다. 전체 dist font
contract는 Chrome/Firefox/VS Code fresh build 뒤 Stage 4에서 실행한다.

## 6. active reference scan

tooling 변경 뒤 `.github`, `scripts`, supported frontend package, npm, root build/README를 검색했다.

| 결과 | 분류 |
|------|------|
| `rhwp-studio/src/core/font-loader.ts`의 `web/editor.html` | 포팅 출처 provenance, 유지 |
| `rhwp-studio/src/core/font-substitution.ts`의 `web/font_substitution.js` | 포팅 출처 provenance, 유지 |
| CI/metrics/font test legacy path | 0 |
| production/build/package 직접 소비자 | 0 |

한/영 manual과 current policy의 stale 안내는 Stage 3에서 제거한다. history-only task evidence는 검색 결과가
남아도 수정하지 않는다.

## 7. post-removal metrics

### 7.1 schema와 provenance

| 항목 | 결과 |
|------|------|
| schemaVersion | 2 |
| compare baseline | `output/frontend-metrics/task2313/pre/metrics.json` |
| `legacy-web` include/group key | 없음 |
| `fontAssets.legacyWebLink` | 없음 |
| canonical directory | `assets/fonts` |
| Studio public link | `../../assets/fonts` |
| canonical files / bytes | 36 / 22,651,296 |
| working tree clean | `false`, 승인된 Stage 2 변경 존재 |
| measured source clean | `true`, non-legacy 측정 source dirty 0 |

metrics script와 deleted legacy code는 측정 후 current 모집단에 포함되지 않으므로 working tree는 dirty지만
`measuredSourceClean=true`다. Stage 5에서 commit 상태로 최종 output을 다시 생성한다.

| artifact | SHA-256 |
|----------|---------|
| pre-removal metrics script | `4008301769eca77bfc25233556fa8fbe3bb9b2560f3083b49db030af9d5354d3` |
| post-removal metrics script | `5d100c90f47671240f463b0a48fe61d34eb8aedbf8c22bbe333f31241f11d087` |
| metrics package lock | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |
| pre-removal JSON | `f81a720f455b82d57809d1fcfc7de1272aa78c27f68d8cfbbfd46a68095e5aa5` |
| Stage 2 post-removal JSON | `424ff59e11ec74af7093d454335d93acb76f7e9dc4279dfbe18a33ab7422e94d` |

### 7.2 direct delta

| 지표 | pre | post | direct delta | 예상 일치 |
|------|----:|-----:|-------------:|-----------|
| included files | 224 | 214 | -10 | 일치 |
| reported CC functions | 2,516 | 2,367 | -149 | 일치 |
| Total CC | 13,118 | 12,290 | -828 | 일치 |
| global Top 20 합 | 2,700 | 2,662 | -38 | 일치 |
| CC>25 개수 / 합 | 73 / 4,479 | 69 / 4,272 | -4 / -207 | 일치 |
| CC>100 개수 / 합 | 7 / 1,839 | 7 / 1,839 | 0 / 0 | 일치 |
| Max CC | 453 | 453 | 0 | 일치 |

function diff 149건은 모두 `web/` 삭제이며 non-web diff는 0건이다. legacy group 내부 Top 20 합 442를
global 개선량으로 사용하지 않았다.

## 8. Stage 2 gate 결과

| 관문 | 결과 |
|------|------|
| tracked/filesystem `web/` 0 | PASS |
| canonical font와 Studio link 불변 | PASS |
| CI/metrics/font contract legacy 결합 제거 | PASS |
| schema v2 pre/post compare | PASS |
| expected structural delta | PASS, 전 항목 일치 |
| non-legacy stable function diff | PASS, 0 |
| workflow·Node syntax | PASS |
| current docs 변경 | 0, Stage 3 대기 |
| fresh package/browser gate | 미실행, Stage 4 예정 |

## 9. 다음 단계 경계

작업지시자 승인 후 Stage 3에서만 다음 current 문서를 현행화한다.

1. 한/영 local server manual을 Studio-only workflow로 정리
2. font fallback strategy의 current legacy link 표현 제거
3. #2023 contract guardrail을 post-removal 상태로 갱신
4. current reference allowlist와 historical evidence diff 검증

Stage 3 승인 전에는 위 문서를 수정하지 않는다. product source, package build와 public/security contract는
계속 변경하지 않는다.
