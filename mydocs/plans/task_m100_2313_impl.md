# Task M100 #2313 구현 계획서 — legacy `/web` 제거 및 current 계약 정리

- 이슈: #2313
- 상위 추적: #2022
- 선행 계획: #2023 v2 / PR #2080
- 선행 기준선: #2124 / PR #2174
- 선행 Phase A: #2125 / PR #2254, merge `7d5f39a4`
- 마일스톤: M100 / v1.0.0
- 브랜치: `task2313-legacy-web-removal`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `4817f2c1b5c7fde57a2d4d9b2ec115b02c437d20`
- 작성일: 2026-07-17
- 단계: 구현 계획서
- 선행 수행 계획서: `mydocs/plans/task_m100_2313.md`

## 1. 구현 목표

tracked `web/` 18개 entry를 제거하고, current CI·metrics·font contract·한/영 manual과 guardrail을
Studio 및 `assets/fonts` 중심 구조로 맞춘다.

구현은 다음 계약을 동시에 만족해야 한다.

1. legacy source를 Studio나 새 archive directory로 이동·복제하지 않는다.
2. `assets/fonts` 36개, 22,651,296 bytes와 승인 hash를 바꾸지 않는다.
3. `rhwp-studio/public/fonts -> ../../assets/fonts`와 runtime `fonts/...` URL을 바꾸지 않는다.
4. Chrome/Firefox/Safari/VS Code/npm editor의 public·security·distribution 계약을 바꾸지 않는다.
5. #2124 schema v2 snapshot과 dated evidence를 수정하지 않는다.
6. metrics 감소를 legacy 모집단 삭제분과 non-legacy delta로 분리한다.
7. fresh frontend gate를 통과한 뒤에만 완료 보고와 GitHub 초안을 작성한다.

## 2. 확정 구현 결정

| 항목 | 결정 |
|------|------|
| `/web` 처리 | `git rm -r web`으로 tracked tree 전체 삭제 |
| legacy 코드 보관 | repository 안에 복사본·stub·redirect를 만들지 않고 Git history 사용 |
| current web app | `rhwp-studio` Vite app만 지원 경로로 유지 |
| WASM output | ignored `pkg/` fresh build를 authority로 유지, tracked legacy glue 삭제 |
| font canonical | `assets/fonts` 유지, binary·license·fallback 변경 없음 |
| Studio font | `rhwp-studio/public/fonts` link와 `fonts/...` runtime URL 유지 |
| CI detector | 존재하지 않게 되는 `web/` directory prefix 한 줄만 제거 |
| metrics | `legacy-web` group과 `web/*` 전용 분기·metadata 제거, schemaVersion 2 유지 |
| font contract | legacy link assertion만 제거, canonical/Studio/artifact 검증 유지 |
| local manual | legacy Python HTTPS 절 삭제, Studio Vite workflow만 유지 |
| guardrail | post-Phase-A/post-removal current 계약으로 현행화 |
| provenance | Studio 포팅 출처 주석 2개는 지원 경로가 아니므로 유지 |
| Safari | source/manifest/Chrome dist 상속 static gate만 실행 |
| changelog | historical 언급은 보존하고 이번 내부 cleanup 항목은 추가하지 않음 |

`fontAssets.legacyWebLink`는 #2125 이후 current metrics에 추가된 optional metadata다. 공식 #2124 snapshot에는
해당 field가 없고 snapshot comparator는 `schemaVersion`과 `cognitiveComplexityEntries`를 사용한다. 따라서
field 제거는 function comparison schema를 바꾸지 않으며 schemaVersion 2를 유지한다.

## 3. 변경 파일과 경로

### 3.1 제거 대상

| 분류 | 파일/경로 | 구현 |
|------|-----------|------|
| legacy runtime | `web/app.js`, `char_shape_dialog.js`, `editor.js`, `font_substitution.js`, `format_toolbar.js`, `text_selection.js` | 삭제 |
| legacy UI | `web/editor.css`, `style.css`, `editor.html`, `index.html` | 삭제 |
| legacy dev/test | `web/https_server.py`, `clipboard_test.html`, `certs/**` | 삭제 |
| generated glue | `web/rhwp.js`, `rhwp.d.ts`, `rhwp_bg.wasm.d.ts` | 삭제 |
| compatibility | `web/fonts -> ../assets/fonts` | 삭제 |

삭제 직전 `git ls-tree`, line count와 symlink target을 Stage 1 보고서에 기록한다. 삭제 명령은 구현 승인과
Stage 1 승인 뒤에만 실행한다.

### 3.2 CI와 자동 검증

| 파일 | 정확한 변경 |
|------|-------------|
| `.github/workflows/ci.yml` | `directoryPrefixes`의 `'web/',` 제거. 다른 trigger, preflight와 required job 이름은 유지 |
| `scripts/frontend-metrics.mjs` | §4의 legacy 전용 include/exclude/helper/link 처리를 제거 |
| `scripts/frontend-font-assets.test.mjs` | Studio와 legacy link 복합 test를 Studio link 단독 test로 축소 |

다음 파일은 검증만 하고 수정하지 않는다.

- `rhwp-studio/public/fonts`
- `rhwp-chrome/build.mjs`, `rhwp-firefox/build.mjs`
- `rhwp-safari/build.sh`, `rhwp-safari/src/manifest.json`
- `rhwp-vscode/webpack.config.js`
- `scripts/frontend-extension-dist.test.mjs`
- `rhwp-studio/src/core/font-loader.ts`, `font-substitution.ts`
- `npm/editor/**`

### 3.3 current 문서

| 파일 | 정확한 변경 |
|------|-------------|
| `mydocs/manual/local_web_server.md` | Studio heading의 과도기 표현 제거, `[web/] Python HTTPS 서버` 이하 삭제 |
| `mydocs/eng/manual/local_web_server.md` | 한국어 manual과 같은 구조로 legacy Python HTTPS 절 삭제 |
| `mydocs/tech/font_fallback_strategy.md` | 2026-07 current note와 §1.2에서 legacy link 설명만 제거, historical 본문 유지 |
| `mydocs/tech/task_m100_2023_frontend_contract_guardrails.md` | current 계약 지도·§3·VS Code font source·framework/금지/후속 결정을 현행화 |

`mydocs/tech/task_m100_2023_frontend_contract_guardrails.md`는 dated snapshot이 아니라 후속 실행 이슈의
guardrail authority이므로 current section을 갱신한다. 반면 diagnosis, baseline manifest, plan, stage와 final
report는 당시 사실을 설명하므로 수정하지 않는다.

### 3.4 작업 문서

| 파일 | 변경 |
|------|------|
| `mydocs/orders/20260717.md` | stage 진행과 완료 시점 갱신 |
| `mydocs/working/task_m100_2313_stage{1..5}.md` | 각 관문 결과 기록 |
| `mydocs/report/task_m100_2313_report.md` | 완료 기준, metrics, gate와 잔여 위험 결산 |

## 4. `scripts/frontend-metrics.mjs` 구현 상세

### 4.1 제거할 include/exclude 정의

- `INCLUDE_GROUPS`의 `legacy-web` object 전체
- `EXCLUDE_RULES`의 `web/rhwp.js`, `web/rhwp.d.ts`, `web/rhwp_bg.wasm.d.ts`
- `EXCLUDE_RULES`의 `web/clipboard_test.html`, `web/fonts/`
- `isLegacyWebFile()` helper 전체
- `isTestFile()`의 `file === 'web/clipboard_test.html'` 조건
- `isExcluded()`의 generated glue exact-match 조건
- `isExcluded()`의 `file.startsWith('web/fonts/')` 조건만 제거하고 `assets/fonts/` 조건은 유지
- `fontReferences()`의 `/web\/fonts/g` pattern

generic `certs/`, `test/tests/e2e`, `dist`, `pkg`, `icons`, `_locales` 제외 규칙은 다른 frontend target에도
적용될 수 있으므로 유지한다. legacy 삭제를 이유로 일반 규칙까지 정리하지 않는다.

### 4.2 font inventory metadata

`fontAssetInventory()`에서 다음만 제거한다.

```text
const legacyWebLink = path.join(ROOT, 'web/fonts')
fontAssets.legacyWebLink
```

`studioPublicLink`, canonical font files, license files, fileCount와 totalBytes는 그대로 유지한다.
`readlinkSync`는 Studio link와 `fileFingerprint()`에서 계속 사용하므로 import에서 제거하지 않는다.

### 4.3 기대 metrics 구조

post-removal output에는 `includeGroups`와 `groupTotals`에 `legacy-web` key가 없어야 하고
`fontAssets.legacyWebLink`도 없어야 한다. 다음은 기준 커밋에서의 expected direct delta다.

| 지표 | expected delta |
|------|---------------:|
| included files | -10 |
| reported CC functions | -149 |
| Total CC | -828 |
| global Top 20 합 | -38 |
| CC>25 개수 / 합 | -4 / -207 |
| CC>100 개수 / 합 | 0 / 0 |
| Max CC | 0 |

pre/post 사이 non-legacy stable function delta는 0건이어야 한다. upstream drift가 있으면 최신 같은 base에서
pre 값을 재생성해 expected table을 먼저 갱신하고 작업지시자에게 보고한다.

## 5. `scripts/frontend-font-assets.test.mjs` 구현 상세

현재 `Studio and legacy web font links target the canonical directory` test의 link array에서 legacy entry를
삭제하고 test 이름을 Studio 단독 계약으로 바꾼다.

검증은 다음을 계속 포함한다.

- canonical 36개와 총 22,651,296 bytes
- `NotoSansKR-Regular.woff2` 승인 SHA-256
- `FONTS.md`, `SourceHanSerifK-OFL.txt`
- `rhwp-studio/public/fonts`가 symlink이고 target이 `../../assets/fonts`
- Studio/Chrome/Firefox artifact 36개 exact filename·bytes·SHA-256
- VS Code approved 11개 subset exact filename·bytes·SHA-256

test 구조를 불필요하게 다시 설계하거나 Safari artifact 검증을 추가하지 않는다. Safari static contract는
기존 extension dist test와 Stage 4 명령으로 확인한다.

## 6. 문서 구현 상세

### 6.1 한/영 local server manual

- 제목은 유지한다.
- Studio heading을 각각 `Vite 개발 서버 (현재 지원)` / `Vite Development Server (Supported)`로 바꾼다.
- Studio prerequisites, Docker WASM build, Vite 실행, port와 one-shot command는 유지한다.
- legacy Python server, certificate 생성, `/web/*.html`, clipboard test와 legacy troubleshooting을 삭제한다.
- 새 migration 설명이나 feature 소개를 추가하지 않는다.

### 6.2 font fallback strategy

- 상단 현행화 note를 `assets/fonts`와 Studio runtime URL만 설명하도록 바꾼다.
- §1.2에서 legacy link 문장을 제거한다.
- 2026-04-07 조사 당시 `web/fonts` 표현은 명시된 historical section이므로 그대로 둔다.
- font family, substitution, license 판단과 fallback roadmap은 변경하지 않는다.

### 6.3 #2023 contract guardrail

다음 current 표현을 post-removal 상태로 바꾼다.

1. 보호 표면에서 `/web`을 current app/asset root로 보지 않고 제거된 legacy evidence로 분류한다.
2. Studio font row를 `../../assets/fonts`로 갱신한다.
3. font license authority를 `assets/fonts/FONTS.md`로 갱신한다.
4. §3을 `assets/fonts` ownership과 legacy 복원 금지 규칙으로 다시 쓴다.
5. VS Code CopyPlugin source를 `assets/fonts`로 갱신한다.
6. runtime framework 도입 금지를 후보가 아닌 확정 guardrail로 표현한다.
7. 이미 결정된 canonical 위치와 `/web` 제거를 open decision 목록에서 제거한다.
8. 후속 단계 입력을 legacy 제외 metrics의 Phase B hotspot 재평가로 갱신한다.

문서의 기존 #2023 기준 commit은 provenance로 유지하고 `현행화: #2125, #2313` metadata를 추가한다.
과거 경로가 왜 제거됐는지 필요한 최소 문맥만 남기고 legacy 파일 목록을 current 계약으로 반복하지 않는다.

## 7. Stage 1 — 제거 전 기준선 고정

### 7.1 사전 준비와 계획 문서 commit

구현 승인 후 먼저 orders·수행 계획·구현 계획을 하나의 docs commit으로 고정한다.

```bash
git status --short --branch
git rev-parse HEAD upstream/devel
git diff --check
git add mydocs/orders/20260717.md mydocs/plans/task_m100_2313.md mydocs/plans/task_m100_2313_impl.md
git commit -m "Task #2313: legacy web 제거 계획 확정"
```

commit 전 upstream drift가 있으면 자동 rebase/merge하지 않고 사용자에게 통합 방식을 승인받는다.

### 7.2 inventory와 dependency scan

```bash
git ls-tree -r --name-only HEAD web
git ls-tree HEAD web/fonts rhwp-studio/public/fonts
readlink web/fonts
readlink rhwp-studio/public/fonts
wc -l web/*.js web/*.css web/*.html web/*.py web/*.d.ts
git grep -n -E 'web/(app|char_shape_dialog|clipboard_test|editor|font_substitution|fonts|format_toolbar|https_server|index|rhwp|style|text_selection)'
```

검색 결과를 production/build, tooling, current docs, provenance, historical evidence로 분류한다. repository
내 production/build/package 직접 소비자는 0이어야 한다. Studio source의 포팅 주석 2개는 provenance로
명시한다.

### 7.3 pre-removal metrics

```bash
npm --prefix scripts/frontend-metrics ci
node scripts/frontend-metrics.mjs \
  --compare mydocs/metrics/frontend/2026-07-11/metrics.json \
  --out output/frontend-metrics/task2313/pre/metrics.json \
  --summary output/frontend-metrics/task2313/pre/summary.md
shasum -a 256 scripts/frontend-metrics.mjs scripts/frontend-metrics/package-lock.json \
  output/frontend-metrics/task2313/pre/metrics.json
```

Stage 1 보고서에는 base commit, clean/dirty metadata, included files, Total CC, global Top 20, threshold 합·개수,
max, legacy group과 tool hash를 기록한다. output은 ignored evidence이며 commit하지 않는다.

### 7.4 Stage 1 관문과 산출물

- tracked entry 18개와 compatibility target 확인
- unclassified direct consumer 0
- pre-removal metrics 재현
- `mydocs/working/task_m100_2313_stage1.md` 작성
- `Task #2313: legacy web 제거 기준선 고정` commit

Stage 1 완료 후 작업지시자 승인을 받고 Stage 2로 넘어간다.

## 8. Stage 2 — tree와 tooling coupling 제거

### 8.1 변경 순서

1. 승인 후 `git rm -r web`을 실행한다.
2. `ci.yml`의 detector prefix 한 줄을 제거한다.
3. §4에 따라 metrics legacy code를 제거한다.
4. §5에 따라 font link contract를 축소한다.
5. source 수정 직후 정적 검증과 metrics smoke를 실행한다.

`web/` 삭제는 명시적으로 승인된 범위이므로 별도 archive나 redirect를 만들지 않는다. 다른 source 파일에
기능 코드를 옮기지 않는다.

### 8.2 정적 검증

```bash
test ! -e web
git ls-files web
readlink rhwp-studio/public/fonts
node --check scripts/frontend-metrics.mjs
node --check scripts/frontend-font-assets.test.mjs
actionlint .github/workflows/ci.yml
node scripts/frontend-metrics.mjs \
  --compare output/frontend-metrics/task2313/pre/metrics.json \
  --out output/frontend-metrics/task2313/post/metrics.json \
  --summary output/frontend-metrics/task2313/post/summary.md
jq '.schemaVersion, .includeGroups, .groupTotals, .fontAssets' \
  output/frontend-metrics/task2313/post/metrics.json
git diff --check
```

`git ls-files web`는 staged deletion 뒤 출력이 비어 있어야 한다. `includeGroups`, `groupTotals`와
`fontAssets`에 legacy key가 없어야 하고 schemaVersion은 2여야 한다.

### 8.3 Stage 2 관문과 산출물

- filesystem과 index의 `web/` 0
- canonical font와 Studio link 불변
- metrics pre/post compare 실행 가능
- workflow·Node syntax PASS
- `mydocs/working/task_m100_2313_stage2.md`
- `Task #2313: legacy web와 tooling 결합 제거` commit

Stage 2 완료 후 작업지시자 승인을 받고 Stage 3으로 넘어간다.

## 9. Stage 3 — current 문서와 guardrail 현행화

### 9.1 변경 순서

1. 한/영 local server manual을 같은 의미의 Studio-only 구조로 편집한다.
2. font fallback의 current note와 ownership 문장만 보정한다.
3. #2023 contract guardrail current section을 §6.3 기준으로 갱신한다.
4. current direct reference scan과 historical path diff를 실행한다.

### 9.2 reference allowlist

current source scan에서 허용되는 legacy file path는 다음 provenance comments 두 개뿐이다.

- `rhwp-studio/src/core/font-loader.ts`: `web/editor.html` 포팅 출처
- `rhwp-studio/src/core/font-substitution.ts`: `web/font_substitution.js` 포팅 출처

current policy 문서는 `/web`이 제거됐다는 사실을 설명할 수 있지만 실행·지원 경로로 안내해서는 안 된다.

### 9.3 검증

```bash
git grep -n -E 'web/(app|char_shape_dialog|clipboard_test|editor|font_substitution|fonts|format_toolbar|https_server|index|rhwp|style|text_selection)' -- \
  .github scripts rhwp-studio rhwp-chrome rhwp-firefox rhwp-safari rhwp-vscode rhwp-shared npm README.md \
  mydocs/manual mydocs/eng/manual mydocs/tech/font_fallback_strategy.md \
  mydocs/tech/task_m100_2023_frontend_contract_guardrails.md
git diff --check
git diff --name-only
git diff -- mydocs/metrics/frontend/2026-07-11 mydocs/tech/task_m100_2124_baseline_manifest.md \
  mydocs/tech/task_m100_2124_frontend_metrics_scope.md mydocs/tech/task_m100_2124_public_contract_snapshot.md \
  mydocs/plans/task_m100_2023_frontend_refactoring_plan_v2.md mydocs/report/task_m100_2125_report.md CHANGELOG.md CHANGELOG_EN.md
```

마지막 historical diff는 비어 있어야 한다. current scan의 provenance 두 줄과 removal status 문서 외 결과는
각각 분류 근거가 없으면 실패다.

### 9.4 Stage 3 관문과 산출물

- 한/영 manual 의미 동기화
- current supported `/web` 실행 안내 0
- font/current guardrail post-removal 정합
- historical evidence diff 0
- `mydocs/working/task_m100_2313_stage3.md`
- `Task #2313: current web 문서와 guardrail 현행화` commit

Stage 3 완료 후 작업지시자 승인을 받고 Stage 4로 넘어간다.

## 10. Stage 4 — fresh frontend 소비자 gate

### 10.1 clean input 준비

```bash
docker compose --env-file .env.docker run --rm wasm
npm --prefix rhwp-studio ci
npm --prefix rhwp-chrome ci
npm --prefix rhwp-firefox ci
npm --prefix rhwp-vscode ci
```

Docker daemon이 unavailable이면 stale `pkg/`로 대체하지 않는다. daemon을 준비하고 fresh build를 다시
실행한다. `npm audit fix`, lockfile 변경과 dependency upgrade는 범위 밖이다.

### 10.2 CI parity contract와 package gate

```bash
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
npm --prefix npm/editor test --if-present
node --test rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs
npm --prefix rhwp-studio run test
npm --prefix rhwp-studio run build
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
npm --prefix rhwp-vscode run compile
node --test scripts/frontend-font-assets.test.mjs
```

font contract는 모든 dist가 생성된 뒤 실행한다.

### 10.3 browser smoke

```bash
npm --prefix rhwp-studio run e2e
npm --prefix rhwp-studio run e2e:embed
npm --prefix rhwp-studio run e2e:canvaskit-font-coverage
```

legacy 앱이 없어도 current Studio의 load/edit/export/embed와 canonical font loading이 동작함을 확인한다.
이번 변경은 renderer output을 바꾸지 않으므로 baseline/render-diff 전체는 기본 필수 gate가 아니다. browser
smoke에서 rendering delta가 발견되면 별도 원인 분석 전 PASS로 처리하지 않는다.

### 10.4 Safari static gate

```bash
bash -n rhwp-safari/build.sh
node --check rhwp-safari/src/background.js
node --check rhwp-safari/src/content-script.js
node --check rhwp-safari/src/options.js
rg -n 'CHROME_DIST|cp -R.*CHROME_DIST|fonts/\*' rhwp-safari/build.sh rhwp-safari/src/manifest.json
```

`scripts/frontend-extension-dist.test.mjs`의 Safari stricter WAR assertion과 Chrome dist 36-font 검증을 함께
근거로 사용한다. source, manifest, Chrome build와 asset path에 diff가 없으므로 Xcode project 생성,
unsigned/signed build와 설치는 실행하지 않는다.

### 10.5 Stage 4 관문과 산출물

- fresh WASM과 CI parity package gate PASS
- Studio text-flow/embed/font browser smoke PASS
- Chrome/Firefox/VS Code font artifact contract PASS
- Safari static inheritance/security contract PASS
- `mydocs/working/task_m100_2313_stage4.md`
- `Task #2313: frontend removal gate 검증 결과 정리` commit

Stage 4 완료 후 작업지시자 승인을 받고 Stage 5로 넘어간다.

## 11. Stage 5 — metrics 결산과 완료 문서

### 11.1 post-removal metrics 재생성

Stage 2 smoke output을 최종 commit 상태에서 다시 생성한다.

```bash
node scripts/frontend-metrics.mjs \
  --compare output/frontend-metrics/task2313/pre/metrics.json \
  --out output/frontend-metrics/task2313/post/metrics.json \
  --summary output/frontend-metrics/task2313/post/summary.md
node scripts/frontend-metrics.mjs \
  --compare mydocs/metrics/frontend/2026-07-11/metrics.json \
  --out output/frontend-metrics/task2313/post-vs-official/metrics.json \
  --summary output/frontend-metrics/task2313/post-vs-official/summary.md
shasum -a 256 scripts/frontend-metrics.mjs scripts/frontend-metrics/package-lock.json \
  output/frontend-metrics/task2313/pre/metrics.json output/frontend-metrics/task2313/post/metrics.json
```

최종 보고서에는 다음 세 비교를 분리한다.

1. same-base pre/post: #2313 직접 structural deletion delta
2. pre-removal legacy group: 삭제된 모집단의 files/lines/functions/CC
3. #2124 official 대비 post-removal: baseline 이후 upstream 누적 변화

non-legacy stable function diff 0, CC>100과 max 0-delta를 명시한다. expected와 다르면 함수별 diff로 원인을
설명하고 무관한 upstream/구현 delta를 제거하기 전 완료하지 않는다.

### 11.2 Phase B 재평가 입력

post-removal current hotspot에서 Total CC, global Top 20, CC>25/100과 function LOC 상위를 기록한다. 이는
#2022 다음 의사결정 입력일 뿐이며 다음 실행 이슈를 자동 생성하거나 SOLID 총점을 부여하지 않는다.

### 11.3 완료 문서와 GitHub 경계

- `mydocs/working/task_m100_2313_stage5.md`
- `mydocs/report/task_m100_2313_report.md`
- `mydocs/orders/20260717.md` 상태·완료 시각 갱신
- PR 제목·본문, maintainer 리뷰 요청 코멘트 초안
- #2313 체크리스트·완료 코멘트와 #2022 진행상황 갱신 초안

GitHub 게시, push, PR 생성과 issue close는 초안을 먼저 제시하고 작업지시자 승인 후 수행한다.

## 12. 단계별 커밋 계획

| 순서 | 커밋 | 포함 |
|------|------|------|
| 1 | `Task #2313: legacy web 제거 계획 확정` | orders, 수행 계획, 구현 계획 |
| 2 | `Task #2313: legacy web 제거 기준선 고정` | Stage 1 inventory/metrics 보고 |
| 3 | `Task #2313: legacy web와 tooling 결합 제거` | `web/` 삭제, CI, metrics, font contract, Stage 2 보고 |
| 4 | `Task #2313: current web 문서와 guardrail 현행화` | 한/영 manual, current tech docs, Stage 3 보고 |
| 5 | `Task #2313: frontend removal gate 검증 결과 정리` | Stage 4 결과 |
| 6 | `Task #2313: legacy web 제거 완료 정리` | Stage 5, final report, orders |

각 구현 commit은 해당 stage gate 통과와 작업지시자 승인 뒤 생성한다. source deletion과 current 문서 변경을
분리해 reviewer가 실제 runtime coupling 제거와 historical evidence 보존을 독립적으로 검토할 수 있게 한다.

## 13. 변경하지 않는 계약과 파일

- `assets/fonts/**`, `THIRD_PARTY_LICENSES.md`
- `rhwp-studio/public/fonts`, runtime `fonts/...`
- `rhwp-studio/src/**`의 동작 코드
- Chrome/Firefox/Safari/VS Code build와 manifest source
- npm editor package/API/dependency
- Rust/WASM source와 public API
- `.github/workflows/render-diff.yml`
- #2124 metrics·tech·smoke artifacts
- #2023 v2 plan/diagnosis, #2125 plan/stage/report
- historical orders, archives, feedback와 changelog

위 경로에 예상 밖 diff가 생기면 기계적 cleanup으로 간주하지 않고 중단 조건을 적용한다. 단,
`rhwp-studio/public/fonts`와 package build output은 수정하지 않지만 검증 과정에서 ignored dist가 재생성될 수
있으며 commit하지 않는다.

## 14. 중단 조건

다음 중 하나가 발생하면 자동으로 범위를 넓히지 않고 구현을 중단해 보고한다.

- repository production/build/package에서 미분류 `/web` 직접 소비자가 발견됨
- Studio에 legacy source에서만 존재하는 current 필수 기능이 확인됨
- `assets/fonts` filename/bytes/hash 또는 Studio link가 예상과 다름
- metrics schema v2가 `legacy-web` 또는 `legacyWebLink` 제거 뒤 old snapshot을 비교하지 못함
- non-legacy stable function 또는 complexity delta가 발생함
- runtime `fonts/...`, extension CSP/WAR/publicDir, VS Code subset 변경이 필요함
- `@rhwp/editor` public contract나 runtime dependency 변경이 필요함
- current manual 갱신 때문에 historical evidence 수정이 필요함
- Safari static contract가 Chrome dist 상속을 증명하지 못함
- fresh WASM/package/browser gate가 legacy 제거와 연관된 회귀를 보임
- upstream drift가 expected metrics 또는 수정 파일과 충돌함

중단 시 원인, 대안과 별도 이슈 필요성을 문서화하고 작업지시자 승인 전 compatibility stub, source 복원,
기능 수정 또는 후속 이슈 생성을 수행하지 않는다.

## 15. 구현 승인 후 첫 작업

본 구현 계획 승인 후 계획 문서 commit을 만들고 Stage 1 inventory와 pre-removal metrics만 수행한다. Stage 1
보고와 별도 승인 전에는 `web/` 삭제, CI·metrics·font test·manual 수정과 product build/E2E를 수행하지
않는다.
