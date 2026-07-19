---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-2124/README.md
last_verified: 2026-07-16
---

# Task M100 #2124 - 프론트 baseline manifest

- 이슈: #2124
- 작성일: 2026-07-10
- 측정일: 2026-07-11 KST
- metrics generatedAt: `2026-07-11T06:21:30.736Z`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 작업 브랜치: `task2124-frontend-baseline`
- 공식 snapshot:
  - `mydocs/metrics/frontend/2026-07-11/metrics.json`
  - `mydocs/metrics/frontend/2026-07-11/summary.md`
- metrics scope: `mydocs/tech/investigations/issue-2124/task_m100_2124_frontend_metrics_scope.md`

## 1. 목적

이 문서는 #2124 Phase 0 frontend baseline freeze의 공식 측정 환경을 고정한다. 이후 Phase A/B
리팩터링은 이 manifest와 schema v2 metrics snapshot을 기준으로 전후 차이를 비교한다.

이 baseline은 advisory snapshot이다. #2124 리뷰 승인 전에는 CC>25, CC>100 또는 SOLID 평가를
PR fail gate로 사용하지 않는다.

## 2. Git 기준과 측정 상태

| 항목 | 값 |
|------|----|
| `HEAD` | `3077f96d1f9931c50d6d62be77b389d4f66470a9` |
| `upstream/devel` | `3077f96d1f9931c50d6d62be77b389d4f66470a9` |
| 기준 상태 | `HEAD`와 `upstream/devel` 일치 |
| 전체 worktree | #2124 문서·도구 작성으로 dirty |
| 측정 대상 소스 | clean (`measuredSourceDirtyPaths: []`) |

전체 dirty path 목록은 `metrics.json`의 `git.dirtyPaths`에 보존한다. 측정 결과를 바꾸는 프론트
소스 변경은 없었으며, 측정 도구와 문서 변경만 존재하는 상태에서 snapshot을 생성했다.

## 3. 실행 환경과 재현성

| 항목 | 값 |
|------|----|
| OS release | Darwin `25.5.0`, arm64 |
| Node.js | `v24.15.0` |
| npm | `11.12.1` |
| TypeScript | `6.0.3` |
| ESLint | `10.6.0` |
| eslint-plugin-sonarjs | `4.1.0` |
| @typescript-eslint/parser | `8.63.0` |
| Docker context | `colima`, Linux arm64 |
| Docker client / server | `29.4.0` / `29.2.1` |
| Docker Compose | `5.1.3` |
| metrics script SHA-256 | `6984e6eb7b019e76c040d98360c403449994da0275d03e4dd0978c8a315a496b` |
| metrics package-lock SHA-256 | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |
| Studio package-lock SHA-256 | `a9992df61824d3778c206e59ad89ecd8156e2835af728752e9ffc77bee4885dc` |

도구 의존성은 `scripts/frontend-metrics/` private package에 고정했다. 루트 `package.json`은 없다.
commit, dirty 상태, 플랫폼, 도구 버전과 입력 파일 해시는 `metrics.json`의 `git` 및 `tools`에 함께 보존한다.

## 4. 브라우저 후보

| 브라우저 | 버전 |
|----------|------|
| Google Chrome | `149.0.7827.201` |
| Mozilla Firefox | `152.0.4` |
| Safari | `26.3.1` |
| Safari build | `21623.2.7.111.2` |

이 버전은 수동 smoke 후보 환경 기록이다. 자동 build·dist 계약과 수동 브라우저 smoke 결과는
`mydocs/tech/investigations/issue-2124/task_m100_2124_smoke_manifest.md`에서 별도로 추적한다.

## 5. 폰트 경로

| 항목 | 값 |
|------|----|
| canonical 현재 위치 | `web/fonts` |
| studio public 경로 | `rhwp-studio/public/fonts -> ../../web/fonts` |
| WOFF2 파일 수 | 36 |
| WOFF2 총 크기 | 22,630,940 bytes |
| license/index 문서 | `web/fonts/FONTS.md`, `THIRD_PARTY_LICENSES.md`, `web/fonts/SourceHanSerifK-OFL.txt` |

36개 파일과 세 문서의 정확한 byte 수 및 SHA-256은 `metrics.json`의 `fontAssets`에 보존한다.
Phase A #2125에서 canonical 위치 이전을 검토하며, #2124에서는 경로·파일·fallback을 변경하지 않는다.

## 6. 공식 모집단과 제외군

공식 모집단과 제외군은 `mydocs/tech/investigations/issue-2124/task_m100_2124_frontend_metrics_scope.md`를 기준으로 한다.

포함 group:

- `studio-runtime`
- `chrome-extension`
- `firefox-extension`
- `safari-extension`
- `shared-frontend`
- `vscode-extension`
- `npm-editor`
- `legacy-web`

주요 제외군:

- `node_modules/`, `dist/`, `pkg/`
- generated WASM glue/declaration: `web/rhwp.js`, `web/rhwp.d.ts`, `web/rhwp_bg.wasm.d.ts`
- `*.min.js`, vendored/generated data
- 모든 `test`, `tests`, `e2e` 디렉터리 및 `*.test.*`, `*.spec.*`, `web/clipboard_test.html`
- font binary와 `web/fonts/`, 향후 `assets/fonts/`
- icons, `_locales`, certs, snapshot, `output/`, cache 파일

테스트 코드는 제품 복잡도 총량을 왜곡하지 않도록 전 group에서 제외한다. 제외된 추적 파일은
`metrics.json`의 `excludedTrackedFilesConsidered`로 확인할 수 있다.

## 7. 공식 snapshot 요약

| Group | Files | Lines | Functions | Total CC | Top 20 sum | CC>25 | CC>25 sum | CC>100 | Max CC | any | as any | this:any | exports |
|------|------:|------:|----------:|---------:|-----------:|------:|----------:|-------:|-------:|----:|-------:|---------:|--------:|
| Studio runtime | 145 | 59,783 | 3,952 | 9,505 | 2,523 | 47 | 3,346 | 6 | 453 | 455 | 241 | 82 | 595 |
| Chrome extension | 15 | 2,330 | 130 | 444 | 295 | 4 | 132 | 0 | 40 | 0 | 0 | 0 | 31 |
| Firefox extension | 15 | 2,325 | 137 | 444 | 295 | 4 | 132 | 0 | 40 | 0 | 0 | 0 | 31 |
| Safari extension | 3 | 1,305 | 107 | 299 | 259 | 3 | 115 | 0 | 43 | 0 | 0 | 0 | 0 |
| Shared frontend | 9 | 864 | 40 | 107 | 98 | 0 | 0 | 0 | 14 | 0 | 0 | 0 | 19 |
| VS Code extension | 4 | 1,387 | 86 | 159 | 126 | 0 | 0 | 0 | 23 | 8 | 4 | 0 | 5 |
| npm editor wrapper | 2 | 266 | 25 | 19 | 19 | 0 | 0 | 0 | 8 | 0 | 0 | 0 | 6 |
| legacy `/web` | 10 | 6,592 | 251 | 828 | 442 | 4 | 207 | 0 | 86 | 0 | 0 | 0 | 6 |

전체 official snapshot:

- 포함 파일: 203
- lines / code lines: 74,852 / 67,155
- functions / reported CC functions: 4,728 / 2,282
- Total CC: 11,805
- 전체 Top 20 CC 합: 2,581
- CC>25: 62개, 합계 3,932
- CC>100: 6개, 합계 1,732
- Max CC: 453
- ESLint parse/fatal diagnostics: 0

group의 `Top 20 sum`은 각 group 안의 상위 20개 합이다. 따라서 group 합을 더한 값은 전체 모집단의
상위 20개 합 2,581과 같은 의미가 아니다.

Phase 0 baseline의 승인된 complexity 예외는 0건이다. 후속 PR의 일시적 CC>25 증가는 자동 허용하지
않으며 `task_m100_2124_frontend_metrics_scope.md`의 예외 심사 조건과 기록 양식을 따른다.

## 8. 상위 cognitive complexity

| CC | 함수 LOC | 함수 | 위치 |
|---:|---------:|------|------|
| 453 | 995 | `onClick` | `rhwp-studio/src/engine/input-handler-mouse.ts:262` |
| 444 | 909 | `onKeyDown` | `rhwp-studio/src/engine/input-handler-keyboard.ts:371` |
| 348 | 381 | `handleOk` | `rhwp-studio/src/ui/picture-props-dialog.ts:1930` |
| 212 | 286 | `populateFromProps` | `rhwp-studio/src/ui/picture-props-dialog.ts:2316` |
| 142 | 354 | `finishResizeDrag` | `rhwp-studio/src/engine/input-handler-table.ts:708` |
| 133 | 328 | `onMouseMove` | `rhwp-studio/src/engine/input-handler-mouse.ts:1405` |
| 88 | 326 | `fillSnapshotFromWasm` | `rhwp-studio/src/compare/diff-engine.ts:929` |
| 86 | 206 | `<anonymous>` | `web/editor.js:187` |
| 73 | 129 | `findPictureAtClick` | `rhwp-studio/src/engine/input-handler-picture.ts:135` |
| 73 | 124 | `renderPictureObjectSelection` | `rhwp-studio/src/engine/input-handler-picture.ts:302` |

## 9. 상위 함수 LOC

| LOC | 함수 | 위치 |
|----:|------|------|
| 995 | `onClick` | `rhwp-studio/src/engine/input-handler-mouse.ts:262` |
| 909 | `onKeyDown` | `rhwp-studio/src/engine/input-handler-keyboard.ts:371` |
| 706 | `<anonymous>` | `web/font_substitution.js:10` |
| 625 | `<anonymous>` | `rhwp-firefox/content-script.js:4` |
| 610 | `<anonymous>` | `rhwp-safari/src/content-script.js:5` |
| 606 | `<anonymous>` | `rhwp-chrome/content-script.js:4` |
| 381 | `handleOk` | `rhwp-studio/src/ui/picture-props-dialog.ts:1930` |
| 354 | `finishResizeDrag` | `rhwp-studio/src/engine/input-handler-table.ts:708` |
| 329 | `getHtml` | `rhwp-vscode/src/hwp-editor-provider.ts:172` |
| 328 | `onMouseMove` | `rhwp-studio/src/engine/input-handler-mouse.ts:1405` |

## 10. 검증

실행한 검증:

```bash
npm --prefix rhwp-studio run metrics:frontend -- --out mydocs/metrics/frontend/2026-07-11/metrics.json --summary mydocs/metrics/frontend/2026-07-11/summary.md
node -e "JSON.parse(require('fs').readFileSync('mydocs/metrics/frontend/2026-07-11/metrics.json','utf8')); console.log('ok')"
node scripts/frontend-metrics.mjs --compare mydocs/metrics/frontend/2026-07-11/metrics.json --out /tmp/task2124-metrics-compare.json --summary /tmp/task2124-metrics-compare.md
docker-compose --env-file .env.docker run --rm wasm
node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs
npm --prefix rhwp-studio run build
npm --prefix rhwp-studio run test
npm --prefix rhwp-vscode run compile
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
node --test scripts/frontend-extension-dist.test.mjs
npm --prefix rhwp-studio run e2e:renderer-contract
git rev-parse HEAD upstream/devel
```

검증 결과:

- JSON parse: `ok`
- `HEAD`와 `upstream/devel`: 동일 commit
- 측정 대상 소스: clean
- schema v2 자체 비교: aggregate delta 0, function diff 0
- repository Docker service fresh WASM build: 통과
- Rust explicit export/generated declaration 및 editor contract: 2 tests 통과
- Studio build/185 unit tests, VS Code compile: 통과
- Chrome/Firefox build와 3개 extension dist contract tests: 통과
- #2184/#2191 CanvasKit 변경에 대한 Studio renderer contract: 통과

## 11. 다음 단계

Stage 3 계약 snapshot과 Stage 4 local gate를 완료했고 maintainer가 최종 rebase·재검증 후 merge를
승인했다. PR merge 전에는 #2124 close 또는 #2125 착수를 선언하지 않는다.

## 12. 리뷰 동기화 addendum

`ebf05268` 이후 dependency-only rebase에서는 측정 소스가 변하지 않아 snapshot을 유지했다. 이후
#2188이 Studio CanvasKit source와 Rust policy를 변경했으므로 PR merge 전 최종 기준선을 `6f1bd284`로
재생성했다. 직전 snapshot 대비 Total CC는 14 증가했고 Top 20, CC>25/100 분포는 변하지 않았다.
함수별 diff는 `renderTextRun` +16, `recordTextRunCoverageGaps` -2를 정확히 식별했다.

이후 `acc841c9`는 `mydocs/manual/memory/`만 변경한 문서 전용 전진분이다. 최신 devel provenance로
snapshot을 재생성한 결과 정량 지표와 함수별 diff는 모두 0-delta였고, frontend/Rust/package 입력이
동일하므로 fresh WASM과 consumer gate는 `6f1bd284` 통과 결과를 유지했다.

이어진 `3077f96d`에는 #2184/#2191의 Studio CanvasKit과 Rust renderer/layout 변경이 포함됐다.
`acc841c9` 대비 code lines +86, functions +3, Total CC +17, CC>25 count/sum +1/+31이며 함수별 diff는
`renderTextRun` +11, `create` +3, `renderShapedScriptText` +2, 익명 함수 +1이다. 최신 upstream을 공식
영점으로 다시 고정하고 repository Docker fresh WASM과 consumer gate를 재실행했다.

dependency rebase에서 Studio가 TypeScript 7.0.2로 갱신됐고,
`@typescript-eslint/parser@8.63.0`의 지원 범위(`<6.1.0`) 및 `eslint-plugin-sonarjs@4.1.0`의 compiler API
사용 방식과 호환되지 않는 것도 확인됐다.

제품 빌드의 TypeScript 7은 유지하고 측정 의존성은 `scripts/frontend-metrics/` private package로
분리했다. 최종 리뷰 동기화 환경은 다음 명령으로 설치한다.

```bash
npm ci --prefix scripts/frontend-metrics
```

최종 도구는 ESLint 10.6.0, SonarJS 4.1.0, parser 8.63.0, TypeScript 6.0.3을 그대로 사용한다.
재생성한 공식 snapshot 자기 비교 결과는 aggregate delta 0, function diff 0이다. 최종 hash는 다음과 같다.

| 항목 | SHA-256 |
|------|---------|
| metrics script | `6984e6eb7b019e76c040d98360c403449994da0275d03e4dd0978c8a315a496b` |
| metrics package-lock | `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd` |
| Studio package-lock | `a9992df61824d3778c206e59ad89ecd8156e2835af728752e9ffc77bee4885dc` |

실행 결과의 `tools`에는 `metricsPackageLockSha256`과 `studioPackageLockSha256`을 함께 기록한다.
