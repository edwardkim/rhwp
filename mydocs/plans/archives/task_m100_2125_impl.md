# Task M100 #2125 구현 계획서 — assets/fonts canonical 이전

- 이슈: #2125
- 상위 추적: #2022
- 선행 계획: #2023 v2
- 선행 기준선: #2124 / PR #2174
- 선행 CI·계약: #2183 / PR #2216, #2186 / PR #2187
- font provenance: #2190 / PR #2196
- 마일스톤: M100 / v1.0.0
- 브랜치: `task2125-assets-fonts-canonical`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `e750e02f0c020cd3e5e7a94bef07586a2ec14820`
- 작성일: 2026-07-13
- 단계: 구현 계획서
- 선행 수행 계획서: `mydocs/plans/task_m100_2125.md`

## 1. 구현 목표

`web/fonts`가 가진 36개 WOFF2와 license/inventory 문서를 공통 `assets/fonts`로 이동하고, 모든
frontend 소비자가 canonical root 또는 명시적인 distribution path를 사용하게 한다.

구현은 다음 계약을 동시에 만족해야 한다.

1. WOFF2 filename, bytes, SHA-256과 총 크기 22,651,296 bytes를 바꾸지 않는다.
2. `NotoSansKR-Regular.woff2`는 #2190 이후 hash
   `d1bf8649914a4fe9477a8735bf056383e44e466141fb3d61897252e06d900c1a`를 보존한다.
3. Studio와 legacy `/web`의 runtime `fonts/...` URL은 바꾸지 않는다.
4. Chrome/Firefox/Safari extension의 `dist/fonts`와 WAR/CSP 계약을 바꾸지 않는다.
5. VS Code의 11개 subset과 `dist/media/fonts` 계약을 바꾸지 않는다.
6. `assets/fonts`만 변경된 PR에서도 CI와 Render Diff가 시작되고 frontend gate가 실행된다.
7. #2124 공식 snapshot과 역사 문서는 당시 사실로 보존한다.

## 2. 확정 구현 결정

| 항목 | 결정 |
|------|------|
| canonical source | `assets/fonts` |
| binary 이동 | 현재 `web/fonts`의 36개 WOFF2를 byte 변경 없이 `git mv` |
| canonical 문서 | `FONTS.md`, `SourceHanSerifK-OFL.txt`도 `assets/fonts`로 이동 |
| Studio public path | `rhwp-studio/public/fonts -> ../../assets/fonts` |
| legacy `/web` | `web/fonts -> ../assets/fonts` compatibility symlink |
| Chrome/Firefox | build script가 `ROOT/assets/fonts` 전체 WOFF2를 `dist/fonts`로 복사 |
| Safari | Chrome dist를 상속하는 기존 구조 유지, 로컬 `build.sh` 전체 검증 |
| VS Code | webpack source만 `../assets/fonts`로 변경, 기존 11개 목록 유지 |
| font contract test | `scripts/frontend-font-assets.test.mjs` 신설 |
| extension dist test | canonical source를 `assets/fonts`로 변경하고 기존 security assertions 유지 |
| metrics | inventory source/canonical/link를 새 구조로 갱신, #2124 snapshot은 불변 |
| CI trigger | broad `assets/**` ignore를 현재 비런타임 asset 하위 경로로 좁힘 |
| frontend detector | `assets/fonts/` prefix 추가 |
| Render Diff | `assets/fonts/**` path 추가, old source path trigger 제거 |
| runtime URL/fallback | 변경하지 않음 |

`web/fonts` compatibility symlink를 선택하는 이유는 legacy `web/editor.html`의 상대 URL을 바꾸지 않으면서
binary source 중복을 제거할 수 있기 때문이다. 프로젝트는 이미 `rhwp-studio/public/fonts`와 extension
shared source에서 Git symlink를 사용한다. Chrome/Firefox/VS Code build는 symlink를 거치지 않고
`assets/fonts`를 직접 읽으므로 Windows checkout에서 legacy symlink가 제한되더라도 배포 build source가
암묵적으로 바뀌지 않는다.

## 3. 변경 파일

### 3.1 canonical asset와 link

| 파일/경로 | 변경 |
|-----------|------|
| `web/fonts/*` | 36 WOFF2와 2개 문서를 `assets/fonts/`로 이동 |
| `web/fonts` | `../assets/fonts` compatibility symlink로 교체 |
| `rhwp-studio/public/fonts` | target을 `../../assets/fonts`로 변경 |
| `.gitignore` | 보호 WOFF2 ignore를 `/assets/fonts/...`로 이동 |

binary는 편집 도구로 다시 쓰지 않고 filesystem move만 수행한다. 이동 전후 manifest를 basename 기준으로
비교하고 Git diff에서 rename으로 인식되는지 확인한다.

### 3.2 build와 생성 도구

| 파일 | 변경 |
|------|------|
| `rhwp-chrome/build.mjs` | canonical source를 `assets/fonts`로 변경, 설명 주석 갱신 |
| `rhwp-firefox/build.mjs` | Chrome과 동일하게 변경 |
| `rhwp-vscode/webpack.config.js` | 11개 CopyPlugin source를 `assets/fonts`로 변경 |
| `tools/subset_noto_sans_kr_regular.py` | 기본 WOFF2 출력 경로를 `assets/fonts`로 변경 |
| `ttfs/opensource/README.md` | 대응 WOFF2 생성 위치를 canonical path로 명시 |

`rhwp-safari/build.sh`는 Chrome dist를 그대로 소비하므로 source path 코드는 변경하지 않는다. 변경 없는
파일도 Stage 4에서 실제 full build로 계약을 검증한다.

### 3.3 test와 metrics

| 파일 | 변경 |
|------|------|
| `scripts/frontend-extension-dist.test.mjs` | source inventory root를 `assets/fonts`로 변경 |
| `scripts/frontend-font-assets.test.mjs` | canonical/link/distribution hash 계약 신설 |
| `scripts/frontend-metrics.mjs` | asset inventory, license path, reference pattern, symlink metadata 갱신 |
| `rhwp-studio/e2e/canvaskit-font-coverage.test.mjs` | 직접 읽는 source를 `assets/fonts`로 변경 |

신규 font contract test는 build 이후 다음을 검증한다.

- canonical WOFF2가 정확히 36개이고 총 22,651,296 bytes다.
- `assets/fonts/FONTS.md`와 `SourceHanSerifK-OFL.txt`가 존재한다.
- `rhwp-studio/public/fonts`와 `web/fonts`가 각각 승인된 상대 target을 가진 symlink다.
- Studio `dist/fonts`의 파일명·bytes·SHA-256이 canonical source와 일치한다.
- Chrome/Firefox `dist/fonts`의 파일명·bytes·SHA-256이 canonical source와 일치한다.
- Chrome과 Firefox dist가 서로 exact parity다.
- VS Code `dist/media/fonts`가 승인된 11개 subset과 정확히 일치하고 각 hash가 source와 같다.
- #2190 Noto Sans KR hash가 migration 중 바뀌지 않았다.

Safari dist는 CI에서 생성되지 않으므로 신규 test에서 optional skip으로 숨기지 않는다. Safari는 로컬
Stage 4 full build 뒤 별도 hash 비교 명령으로 검증한다.

### 3.4 CI와 workflow

| 파일 | 변경 |
|------|------|
| `.github/workflows/ci.yml` | trigger ignore 축소, detector prefix 추가, font contract test step 추가 |
| `.github/workflows/render-diff.yml` | canonical font path trigger 반영 |

`ci.yml`의 push/PR `paths-ignore`에서 `assets/**`를 제거하고 다음 현재 비런타임 경로로 대체한다.

```text
assets/chrome/**
assets/edge/**
assets/logo/**
assets/screenshots/**
```

이렇게 하면 `assets/fonts/**`는 workflow를 시작하지만 기존 store marketing image 변경은 계속 skip한다.
preflight `directoryPrefixes`에는 `assets/fonts/`를 추가한다. 새 font contract test는 Chrome/Firefox build와
VS Code compile이 끝난 뒤 실행해 clean runner의 실제 artifact를 검증한다.

draft PR CI에서는 `frontend_required=true`와
`frontend_reason=frontend-path:assets/fonts/<filename>`을 확인해 trigger와 detector 두 층을 모두 검증한다.

Render Diff의 `web/fonts/**`는 `assets/fonts/**`로 바꾼다. 역사 문서나 compatibility symlink만 바뀌는
경우에는 `mydocs/**`, `rhwp-studio/**` 등 기존 path가 필요에 따라 trigger를 담당한다.

### 3.5 현재 운영 문서

| 파일 | 변경 |
|------|------|
| `assets/fonts/FONTS.md` | canonical 경로와 subset 생성 출력 경로 갱신 |
| `THIRD_PARTY_LICENSES.md` | license index를 `assets/fonts`로 갱신 |
| `npm/editor/README.md` | self-hosting source ownership과 runtime `/fonts` 설명 분리 |
| `mydocs/manual/chrome_edge_extension_build_deploy.md` | source 36개 전체 복사와 dist 계약 현행화 |
| `mydocs/manual/publish_guide.md` | AMO source archive 입력을 `assets/fonts`로 변경 |
| `mydocs/tech/font_fallback_strategy.md` | 현재 canonical 경로·36개 inventory로 factual section 현행화, fallback 정책 불변 |
| `mydocs/tech/investigations/issue-139/equation_font_selection.md` | Latin Modern Math source 경로를 canonical로 갱신 |
| `mydocs/tech/investigations/issue-2125/task_m100_2125_font_ownership.md` | current manifest, #2190 provenance, target copy matrix 신설 |

다음 문서는 당시 경로와 작업 결과를 설명하는 역사 자료이므로 일괄 치환하지 않는다.

- `mydocs/metrics/frontend/2026-07-11/**`
- #2023/#2124 plan, tech, report, feedback
- #2190 stage/report와 과거 orders
- `mydocs/plans/archives/**`, `mydocs/report/archives/**`, `mydocs/working/archives/**`
- version별 `mydocs/feedback/*amo_notes.md`

현재 배포 절차의 authority는 `mydocs/manual/publish_guide.md`로 갱신한다. 과거 AMO 제출 notes는 당시
source archive를 재현하는 기록으로 보존한다.

### 3.6 작업 문서

| 파일 | 변경 |
|------|------|
| `mydocs/orders/20260713.md` | Stage 상태 갱신 |
| `mydocs/working/task_m100_2125_stage{N}.md` | 각 stage 결과 기록 |
| `mydocs/report/task_m100_2125_report.md` | 최종 gate와 이슈 체크리스트 근거 |

## 4. 변경하지 않는 계약

- `rhwp-studio/src/core/font-loader.ts`의 `fonts/...` runtime URL
- `rhwp-studio/src/view/canvaskit-renderer.ts`의 runtime fetch URL
- `web/editor.html`의 legacy `fonts/...` URL
- font family, substitution chain, `font-display`, unicode range
- Chrome/Firefox/Safari manifest의 `fonts/*` WAR
- extension CSP와 `publicDir: false`
- VS Code webview CSP, `asWebviewUri`, `dist/media/fonts` runtime path
- `@rhwp/editor` package files, zero runtime dependency, MessageChannel 계약
- WOFF2 파일명·내용과 VS Code subset 목록
- Rust/WASM source와 public API

위 파일에 diff가 생기면 기계적 source path 변경으로 간주하지 않고 범위 이탈로 중단한다.

## 5. Stage 1 — current inventory와 ownership 문서

### 5.1 작업

1. base commit `e750e02f`의 `web/fonts` 36개 basename, bytes, SHA-256을 수집한다.
2. `mydocs/tech/investigations/issue-2125/task_m100_2125_font_ownership.md`에 전체 manifest와 소비자 copy matrix를 기록한다.
3. #2124 snapshot과 비교해 #2190 한 파일 외 mismatch가 0임을 기록한다.
4. source path, runtime URL, history-only reference를 분류한다.

### 5.2 검증

```bash
git rev-parse HEAD upstream/devel
find web/fonts -maxdepth 1 -type f -name '*.woff2' | sort
find web/fonts -maxdepth 1 -type f -name '*.woff2' -exec shasum -a 256 {} \;
node -e "const fs=require('fs'),p=require('path');const f=fs.readdirSync('web/fonts').filter(x=>x.endsWith('.woff2'));console.log(f.length,f.reduce((n,x)=>n+fs.statSync(p.join('web/fonts',x)).size,0))"
git grep -n 'web/fonts' -- ':!mydocs/metrics/frontend/2026-07-11/**'
git diff --check
```

기대값은 36개, 22,651,296 bytes다. 이 단계에서는 asset 또는 consumer source를 변경하지 않는다.

### 5.3 산출물

- `mydocs/tech/investigations/issue-2125/task_m100_2125_font_ownership.md`
- `mydocs/working/task_m100_2125_stage1.md`

## 6. Stage 2 — canonical move와 source consumer 갱신

### 6.1 작업 순서

1. `web/fonts`의 tracked 파일 38개를 `assets/fonts`로 `git mv`한다.
2. `web/fonts -> ../assets/fonts` symlink를 만든다.
3. `rhwp-studio/public/fonts` target을 `../../assets/fonts`로 바꾼다.
4. `.gitignore`, Chrome/Firefox build, VS Code webpack, subset tool을 갱신한다.
5. extension dist test, CanvasKit coverage, metrics source path를 갱신한다.
6. `scripts/frontend-font-assets.test.mjs`를 추가한다.
7. CI trigger/detector/test step과 Render Diff path를 원자적으로 갱신한다.

binary move와 consumer path 변경은 같은 stage/구현 commit으로 묶는다. binary만 이동한 깨진 중간 commit은
만들지 않는다.

### 6.2 metrics 구현 상세

`scripts/frontend-metrics.mjs`는 다음처럼 갱신한다.

- `fontReferences()` pattern에 `assets/fonts`를 추가하고 legacy `web/fonts` pattern은 compatibility/historical
  reference 탐지를 위해 유지한다.
- `fontAssetInventory()`는 `assets/fonts/*.woff2`와 새 license path를 fingerprint한다.
- `canonicalDirectory`를 `assets/fonts`로 변경한다.
- `studioPublicLink` target이 `../../assets/fonts`인지 기록한다.
- `legacyWebLink` metadata를 추가해 `web/fonts -> ../assets/fonts`를 기록한다.
- complexity include/exclude와 schema v2 총량 산식은 변경하지 않는다.

#2124 snapshot은 old script/schema output으로 그대로 남긴다. 새 output은
`output/frontend-metrics/task2125/`에 생성한다.

### 6.3 정적 검증

```bash
git diff --check
git diff --summary
git ls-files -s web/fonts rhwp-studio/public/fonts
readlink web/fonts
readlink rhwp-studio/public/fonts
git grep -n 'web/fonts' -- ':!mydocs/**'
git check-ignore -v assets/fonts/hamchob-r.woff2
node --check scripts/frontend-metrics.mjs
node --check scripts/frontend-extension-dist.test.mjs
node --check scripts/frontend-font-assets.test.mjs
node --check rhwp-chrome/build.mjs
node --check rhwp-firefox/build.mjs
python3 -m py_compile tools/subset_noto_sans_kr_regular.py
actionlint .github/workflows/ci.yml .github/workflows/render-diff.yml
```

source reference 검사에서 허용되는 `web/fonts`는 compatibility symlink와 history-only 문서뿐이다.
`font-loader.ts`, manifest, legacy HTML의 runtime `fonts/...`는 변경하지 않는다.

### 6.4 산출물

- canonical asset/link/build/test/tool/workflow 변경
- `mydocs/working/task_m100_2125_stage2.md`

## 7. Stage 3 — 운영 문서와 contract 정합

### 7.1 작업

1. canonical `FONTS.md`, root license index, subset generation 문서를 갱신한다.
2. extension build/deploy와 AMO source archive 명령을 새 root로 갱신한다.
3. npm/editor self-hosting 문서에서 source `assets/fonts`와 runtime `/fonts`를 구분한다.
4. active font/equation 기술 문서의 현재 경로를 갱신한다.
5. history-only 문서가 변경되지 않았는지 확인한다.

### 7.2 문서 판정 규칙

- 현재 사용자가 실행할 명령과 source ownership은 갱신한다.
- 과거 task가 실제로 사용한 경로는 수정하지 않는다.
- `web/fonts`가 검색된다는 이유만으로 전역 치환하지 않는다.
- fallback 정책 문구는 path/count의 factual correction 외에는 바꾸지 않는다.

### 7.3 검증

```bash
git diff --check
git diff --name-only
git grep -n 'web/fonts' -- THIRD_PARTY_LICENSES.md npm/editor/README.md mydocs/manual mydocs/tech/font_fallback_strategy.md mydocs/tech/investigations/issue-139/equation_font_selection.md ttfs/opensource/README.md
git diff -- mydocs/metrics/frontend/2026-07-11 mydocs/tech/investigations/issue-2124/task_m100_2124_font_inventory.md mydocs/report/task_m100_2190_report.md
```

마지막 명령의 diff는 비어 있어야 한다.

### 7.4 산출물

- current operational documentation
- `mydocs/working/task_m100_2125_stage3.md`

## 8. Stage 4 — fresh build와 소비자 gate

### 8.1 clean 입력 준비

저장소 규칙상 로컬 WASM은 Docker로만 생성한다.

```bash
docker compose --env-file .env.docker run --rm wasm
npm --prefix scripts/frontend-metrics ci
npm --prefix rhwp-studio ci
npm --prefix rhwp-chrome ci
npm --prefix rhwp-firefox ci
npm --prefix rhwp-vscode ci
```

Docker daemon이 실행되지 않으면 임의 stale `pkg/`를 사용하지 않는다. daemon을 시작한 뒤 fresh build를
재실행한다. `npm audit fix`나 lockfile 자동 변경은 범위 밖이다.

### 8.2 contract와 package gate

```bash
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
npm --prefix npm/editor test --if-present
node --test rhwp-shared/sw/*.test.js rhwp-chrome/sw/*.test.mjs rhwp-firefox/sw/*.test.mjs
npm --prefix rhwp-studio run test
npm --prefix rhwp-studio run build
npm --prefix rhwp-studio run e2e:canvaskit-font-coverage
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
npm --prefix rhwp-vscode run compile
node --test scripts/frontend-font-assets.test.mjs
```

font contract test는 build artifact가 생성된 뒤 실행한다. Studio production dist에는 public symlink를 통한
36개 font가 포함되고, extension/VS Code artifact는 source hash와 일치해야 한다.

### 8.3 Safari gate

현재 로컬 환경에는 Xcode 26.6이 확인됐다. 다음을 기본 필수 gate로 실행한다.

```bash
bash rhwp-safari/build.sh
```

실행 후 Safari `dist/fonts`와 `assets/fonts`의 filename/bytes/SHA-256을 비교하고 Safari manifest의 WAR가
기존 `['wasm/*', 'fonts/*', 'icons/*']`인지 확인한다. Xcode signing/환경 실패가 발생하면 font copy 이전
단계와 Xcode 단계 로그를 분리하고, 대체 evidence 초안을 제시해 작업지시자·reviewer 승인 전에는 PASS로
표시하지 않는다.

### 8.4 metrics gate

```bash
node scripts/frontend-metrics.mjs \
  --compare mydocs/metrics/frontend/2026-07-11/metrics.json \
  --out output/frontend-metrics/task2125/metrics.json \
  --summary output/frontend-metrics/task2125/summary.md
```

검토 항목:

- current `fontAssets.canonicalDirectory=assets/fonts`
- 36개, 22,651,296 bytes와 #2190 hash
- Studio/legacy link metadata
- Total CC, Top 20 합, CC>25 합, CC>25/100 count
- stable function diff와 원인
- historical snapshot 파일 무변경

path-only 작업이므로 product complexity는 0-delta를 기대한다. CI detector/test 추가가 공식 모집단에
들어오지 않더라도 결과를 생략하지 않고 report에 수치를 기록한다.

### 8.5 Stage 4 산출물

- `mydocs/working/task_m100_2125_stage4.md`
- package/dist/font hash 결과
- metrics compare 요약

## 9. Stage 5 — 결산, legacy 판단, 리뷰 준비

### 9.1 이슈 체크리스트 매핑

| #2125 산출물 | 근거 |
|--------------|------|
| canonical ownership | `task_m100_2125_font_ownership.md`, `assets/fonts` |
| 이전 계획 또는 구현 PR | 수행/구현 계획, 로컬 commit/PR 초안 |
| target copy 계약 | ownership matrix와 font contract test |
| Studio | public symlink, Studio build/font gate |
| Chrome/Firefox/Safari | build 결과, extension/font contract, Safari gate |
| VS Code | webpack source, compile, 11개 hash |
| npm/editor | README와 package contract test |
| license/docs | FONTS, THIRD_PARTY_LICENSES, manuals/tech docs |
| `/web` 후속 판단 | compatibility dependency scan과 최종 보고서 |

### 9.2 legacy 후속 판단

canonical 이동 후 `web/`에서 font 외 tracked JS/HTML/CSS의 production/build 소비를 다시 검색한다.
이번 PR에서는 `web/fonts` compatibility symlink를 유지한다. `/web` 삭제가 가능하더라도 후속 이슈 제목,
범위, 선행 조건 초안만 작성하고 작업지시자 승인 전 GitHub에 등록하지 않는다.

### 9.3 최종 산출물

- `mydocs/report/task_m100_2125_report.md`
- `mydocs/orders/20260713.md` 상태 갱신
- #2125 본문 체크리스트/완료 코멘트 초안
- 필요 시 `/web` legacy 후속 이슈 초안
- PR 본문과 maintainer 리뷰 요청 코멘트 초안

push, PR 생성, GitHub 코멘트, issue close는 각각 초안을 먼저 제시하고 작업지시자 승인 후 수행한다.

## 10. 단계별 커밋 계획

| 순서 | 커밋 | 포함 |
|------|------|------|
| 1 | `Task #2125: font ownership 기준선 고정` | ownership manifest, Stage 1 보고 |
| 2 | `Task #2125: canonical font 경로와 소비자 계약 이전` | binary move, symlinks, build/test/tool/CI/workflow, Stage 2 보고 |
| 3 | `Task #2125: font ownership 운영 문서 갱신` | active manuals/tech/package docs, Stage 3 보고 |
| 4 | `Task #2125: frontend font gate 검증 결과 정리` | Stage 4 결과, metrics 요약 |
| 5 | `Task #2125: canonical font 이전 완료 정리` | final report, orders, Stage 5 결과 |

현재 orders·수행 계획·구현 계획은 구현 시작 전 계획 문서 commit으로 먼저 묶는다. 각 구현 commit은
해당 stage gate가 통과한 뒤 생성한다. binary만 이동해 build가 깨진 중간 commit은 만들지 않는다.

## 11. 중단 조건

다음 중 하나가 발생하면 자동으로 범위를 넓히지 않고 구현을 중단해 보고한다.

- current 36개 manifest에 #2190 외 미설명 hash mismatch가 발생
- symlink 방식이 supported checkout/build 환경에서 source를 제공하지 못함
- runtime `fonts/...` URL 변경이 필요함
- fallback/substitution/font binary 내용 변경이 필요함
- extension CSP/WAR/publicDir 변경이 필요함
- VS Code 11개 subset 변경이 필요함
- Safari가 Chrome dist font를 동일하게 상속하지 못함
- `assets/fonts` CI trigger를 위해 required check 이름이나 branch protection 변경이 필요함
- `/web` 삭제가 migration 완료에 필요함

중단 시 별도 선택지와 영향 범위를 문서화하고 작업지시자 승인 전 우회 구현을 추가하지 않는다.

## 12. 구현 승인 후 첫 작업

본 구현 계획 승인 후 Stage 1 ownership 문서와 migration manifest부터 작성한다. Stage 1 승인 전에는
font binary, symlink, build/test/tool/workflow를 변경하지 않는다. GitHub 작업은 로컬 Stage 1~4와
단계별 commit이 완료된 뒤 별도 초안 승인 절차로 진행한다.
