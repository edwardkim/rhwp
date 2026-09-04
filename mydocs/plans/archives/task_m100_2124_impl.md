# Task M100 #2124 — 프론트 웹 Phase 0 baseline freeze 구현 계획

- 이슈: #2124
- 상위 umbrella: #2022
- 선행 계획 이슈: #2023
- 선행 문서 PR: #2080
- 브랜치: `task2124-frontend-baseline`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 작성일: 2026-07-10
- 단계: 구현 계획서
- 선행 수행 계획서: `mydocs/plans/task_m100_2124.md`

## 1. 구현 목표

이 구현은 프론트 웹 구조 리팩터링이 아니라 Phase 0 기준선 고정이다. 목표는 다음이다.

1. frontend metrics 도구와 실행 방식을 저장소에 남긴다.
2. 공식 모집단/제외군 기준으로 metrics snapshot을 만든다.
3. public contract, WASM JSON schema, font, extension security, smoke gate, SOLID evidence 문서를 만든다.
4. #2124 체크리스트를 각 산출물 근거와 연결할 수 있게 만든다.

현재 Stage 1~4는 fresh WASM을 포함해 로컬 완료했고, Stage 5는 maintainer 최종 승인 후 최신 devel
재검증과 merge를 진행하는 상태다. issue checklist/close는 merge 후 별도 승인까지 수행하지 않는다.

## 2. 확정 구현 결정

| 항목 | 결정 |
|------|------|
| metrics script | `scripts/frontend-metrics.mjs`를 신설한다. 기존 `scripts/metrics.sh`는 Rust 중심이라 이번 이슈에서는 확장하지 않는다. |
| ESLint 실행 환경 | `scripts/frontend-metrics/` private package에 `eslint`, `eslint-plugin-sonarjs`, `@typescript-eslint/parser`, 호환 TypeScript를 고정한다. 루트 `package.json`은 만들지 않는다. |
| 실행 방식 | root script가 metrics tool package의 ESLint/parser/plugin을 로드해 전체 프론트 모집단을 측정한다. |
| complexity 지위 | `sonarjs/cognitive-complexity`는 Phase 0에서 snapshot/advisory로만 사용한다. CI fail gate는 추가하지 않는다. |
| metrics schema | schema v2에 분포 지표와 CC 총합·상위 20 합·CC>25 합을 함께 저장하고, 전체 함수 entry를 함수별 diff 입력으로 보존한다. |
| 재현성 | HEAD/upstream, dirty path, measured-source clean 여부, script/lockfile SHA-256을 snapshot에 기록한다. |
| 출력 경로 | `mydocs/metrics/frontend/2026-07-11/metrics.json`과 `summary.md`를 공식 snapshot으로 둔다. |
| legacy `/web` | runtime target과 분리한 legacy group으로 측정하고, 삭제 판단은 하지 않는다. |
| package 영향 | `@rhwp/editor`에는 dependency를 추가하지 않는다. runtime dependency 변경은 하지 않는다. |
| SOLID 총점 | 초기 54/100 초안은 공식 기준선에서 폐기하고 보정 이력으로만 설명한다. 리뷰 전까지 `미채점`으로 둔다. |
| 과도기 예외 | CC>25 +1~2를 자동 허용하지 않는다. v2 조건을 충족한 PR만 functionId·사유·소유·해소 조건·다음 anchor를 기록해 reviewer 승인을 받는다. |

## 3. 변경 예정 파일

### 스크립트와 패키지

| 파일 | 변경 |
|------|------|
| `scripts/frontend-metrics.mjs` | frontend metrics 수집 스크립트 신설 |
| `scripts/frontend-editor-embed.test.mjs` | `@rhwp/editor` 무의존 iframe/message contract smoke |
| `scripts/frontend-extension-dist.test.mjs` | Chrome/Firefox dist asset/CSP/WAR/font smoke |
| `scripts/frontend-wasm-bindings.test.mjs` | Rust 명시 export와 fresh `pkg/rhwp.d.ts` 일치 gate |
| `scripts/frontend-metrics/package.json` | 분석 도구와 호환 TypeScript exact version 고정 |
| `scripts/frontend-metrics/package-lock.json` | 측정 도구 dependency lock |
| `rhwp-studio/package.json` | metrics 실행 명령만 제공, 제품 TypeScript 유지 |

### 공식 산출물

| 파일/경로 | 변경 |
|-----------|------|
| `mydocs/metrics/frontend/2026-07-11/metrics.json` | 공식 frontend metrics snapshot |
| `mydocs/metrics/frontend/2026-07-11/summary.md` | 리뷰용 요약 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_frontend_metrics_scope.md` | 공식 모집단/제외군 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_baseline_manifest.md` | 기준 commit, OS, Node/npm, 브라우저, 폰트 경로, 제외군 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_public_contract_snapshot.md` | `@rhwp/editor`, `@rhwp/core`, VS Code, extension message 계약 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_wasm_json_schema_snapshot.md` | frontend advisory ownership과 Rust 계획 상호 참조 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_font_inventory.md` | `web/fonts` 파일, 참조 경로, license 문서 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_extension_security_snapshot.md` | CSP, WAR, sender/URL/file 검증 상태 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_smoke_manifest.md` | 표면별 무변동 게이트와 실행 가능 조건 |
| `mydocs/tech/investigations/issue-2124/task_m100_2124_frontend_solid_anchors.md` | 미채점 SOLID evidence와 reviewer 채점 절차 |
| `mydocs/working/task_m100_2124_stage{N}.md` | 단계별 완료 보고 |
| `mydocs/report/task_m100_2124_report.md` | 최종 보고서 |
| `mydocs/orders/20260710.md` | 진행 상태 갱신 |

## 4. 단계별 구현 계획

### Stage 1 — metrics 도구와 모집단 고정

작업:

- `scripts/frontend-metrics/package.json`에 ESLint/sonarjs/parser와 호환 TypeScript를 고정하고 lockfile을 생성한다.
- `scripts/frontend-metrics.mjs`를 추가한다.
- 스크립트는 공식 include/exclude를 코드 상수로 두고, 출력 JSON에 같은 내용을 기록한다.
- `mydocs/tech/investigations/issue-2124/task_m100_2124_frontend_metrics_scope.md`를 작성한다.

수집 최소 항목:

- 파일 LOC, 1,200 LOC 초과 파일, 2,000 LOC 초과 파일.
- 함수 LOC 상위.
- `sonarjs/cognitive-complexity` 상위 함수.
- CC 총합, 상위 20 합, CC>25 합, CC>25/CC>100 함수 수.
- stable function id 기반 snapshot 간 함수별 diff.
- `any`, `as any`, `this: any` 수.
- export/public surface 후보.
- 브라우저 확장 duplicated file/logic 후보.
- font reference map.
- WOFF2 byte/SHA-256와 license file hash.

검증:

```bash
npm ci --prefix scripts/frontend-metrics
node scripts/frontend-metrics.mjs --help
node scripts/frontend-metrics.mjs --out output/frontend-metrics/metrics.json --summary output/frontend-metrics/summary.md
node scripts/frontend-metrics.mjs --compare output/frontend-metrics/metrics.json --out /tmp/frontend-metrics-compare.json --summary /tmp/frontend-metrics-compare.md
```

완료 보고:

- `mydocs/working/task_m100_2124_stage1.md`

### Stage 2 — 공식 metrics snapshot 작성

작업:

- 기준 커밋 `3077f96d`의 measured source가 clean인 상태에서 Stage 1 metrics 명령을 실행한다.
- `mydocs/metrics/frontend/2026-07-11/metrics.json`을 저장한다.
- `mydocs/metrics/frontend/2026-07-11/summary.md`를 저장한다.
- `mydocs/tech/investigations/issue-2124/task_m100_2124_baseline_manifest.md`에 OS, Node/npm, 브라우저 후보, font path, 제외군,
  기준 커밋을 기록한다.

검증:

```bash
node -e "JSON.parse(require('fs').readFileSync('mydocs/metrics/frontend/2026-07-11/metrics.json','utf8')); console.log('ok')"
node -e "const m=require('./mydocs/metrics/frontend/2026-07-11/metrics.json'); if(m.schemaVersion!==2||!m.git.measuredSourceClean) process.exit(1)"
git rev-parse HEAD upstream/devel
node --version
npm --version
```

완료 보고:

- `mydocs/working/task_m100_2124_stage2.md`

### Stage 3 — public contract와 WASM JSON schema snapshot

작업:

- `mydocs/tech/investigations/issue-2124/task_m100_2124_public_contract_snapshot.md`를 작성한다.
- `@rhwp/editor` package metadata, `index.js`, `index.d.ts`, README API를 snapshot으로 정리한다.
- `@rhwp/core` README 중심 계약을 정리한다.
- local ignored `pkg`를 기준 commit의 권위 자료로 간주하지 않고 Rust `wasm_api.rs`와 fresh WASM 생성물을 구분한다.
- VS Code webview message, WASM/font media path, CSP/localResourceRoots 계약을 정리한다.
- Chrome/Firefox/Safari extension message 계약과 build copy 계약을 정리한다.
- `mydocs/tech/investigations/issue-2124/task_m100_2124_wasm_json_schema_snapshot.md`에 frontend 소비자 관점의 JSON boundary와
  Rust 리팩터링 계획 상호 참조를 남긴다.

검증:

```bash
rg -n "postMessage|rhwp-request|rhwp-response|createEditor|exportHwp|exportHwpx" npm/editor rhwp-studio rhwp-vscode rhwp-chrome rhwp-firefox rhwp-safari
rg -n "JSON|stringify|parse|optionsJson|Ex\\(" rhwp-studio/src npm
node --test scripts/frontend-editor-embed.test.mjs
# binding test는 fresh WASM 생성 직후 실행한다.
node --test scripts/frontend-wasm-bindings.test.mjs
```

완료 보고:

- `mydocs/working/task_m100_2124_stage3.md`

### Stage 4 — font, extension security, smoke, SOLID evidence 작성

작업:

- `mydocs/tech/investigations/issue-2124/task_m100_2124_font_inventory.md`를 작성한다.
- `web/fonts` 파일 hash, `web/fonts/FONTS.md`, 루트 `THIRD_PARTY_LICENSES.md`, 참조 경로를 정리한다.
- `mydocs/tech/investigations/issue-2124/task_m100_2124_extension_security_snapshot.md`를 작성한다.
- Chrome/Firefox/Safari CSP, `web_accessible_resources`, sender/URL/file signature/size 검증을 정리한다.
- `mydocs/tech/investigations/issue-2124/task_m100_2124_smoke_manifest.md`를 작성한다.
- `mydocs/tech/investigations/issue-2124/task_m100_2124_frontend_solid_anchors.md`를 작성한다.

검증 후보:

```bash
# 로컬에서는 저장소 규칙에 따라 Docker로 fresh pkg를 먼저 생성한다.
docker-compose --env-file .env.docker run --rm wasm
npm --prefix rhwp-chrome ci
npm --prefix rhwp-firefox ci
npm --prefix rhwp-vscode ci
npm --prefix rhwp-studio run build
npm --prefix rhwp-studio run test
npm --prefix rhwp-studio run e2e
npm --prefix rhwp-studio run e2e:renderer-contract
npm --prefix rhwp-studio run e2e:baseline:headless
npm --prefix rhwp-studio run e2e:render-diff
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
npm --prefix rhwp-vscode run compile
node --test scripts/frontend-editor-embed.test.mjs
node --test scripts/frontend-extension-dist.test.mjs
node --test scripts/frontend-wasm-bindings.test.mjs
```

주의:

- 실행 불가 항목은 실패로 숨기지 않고 미실행 사유를 smoke manifest와 stage 보고서에 남긴다.
- Safari는 Chrome dist 기반 영향과 Safari source/manifest asset path를 문서 검토한다.
- repository Docker service로 fresh WASM을 먼저 생성하고 binding, Studio, extension, VS Code gate를
  같은 output 기준으로 판정한다.

완료 보고:

- `mydocs/working/task_m100_2124_stage4.md`

### Stage 5 — 체크리스트 근거, 최종 보고, 다음 단계 판단

작업:

- #2124 산출물 체크리스트와 로컬 산출물 경로를 1:1로 연결한다.
- `mydocs/report/task_m100_2124_report.md`를 작성한다.
- `mydocs/orders/20260710.md`를 갱신한다.
- Phase A #2125는 #2124 리뷰 승인·종료 전까지 착수 금지로 정리한다.
- #2124 본문 체크리스트, 상태 코멘트, draft PR 본문을 로컬 초안으로 준비한다. 실제 GitHub 반영,
  push, PR 생성, issue close는 작업지시자 승인 후 수행한다.

검증:

```bash
git status --short --branch
rg -n "2124|Phase 0|frontend baseline|SOLID|smoke" mydocs/metrics/frontend/2026-07-11 mydocs/tech mydocs/report/task_m100_2124_report.md
```

완료 보고:

- `mydocs/working/task_m100_2124_stage5.md`

## 5. 리스크와 완화

| 리스크 | 완화 |
|--------|------|
| ESLint/sonarjs 설치가 네트워크를 요구 | 구현 시작 전 설치 명령을 명시하고, 실패 시 미설치 사유와 대체 방안을 보고 |
| sonarjs rule 메시지 포맷 의존 | complexity 값과 전체 함수 entry를 schema v2 JSON에 보존하고 self-diff로 검증 |
| 루트 package 부재 | 새 루트 package를 만들지 않고 `scripts/frontend-metrics/` private tool package로 제한 |
| metrics가 fail gate로 오해됨 | script와 문서에 advisory/snapshot 지위를 명시 |
| `@rhwp/editor` 무의존 계약 오해 | `@rhwp/editor` package는 변경하지 않고 public contract snapshot만 작성 |
| stale local `pkg` 오판 | repository Docker build 전 binding test로 stale 상태를 탐지하고, fresh build 후 재검증 |
| smoke 일부 미실행 | 실행 가능/미실행/수동 확인을 smoke manifest에 구분 |

## 6. 구현 승인 후 첫 작업

이 구현 계획서는 선행 승인 후 진행된 구현을 비판적 재검토 결과에 맞춰 보강했다. 로컬 단계별 커밋과
GitHub 초안은 분리하고, push·PR·코멘트 등록은 작업지시자 승인 전 수행하지 않는다.
