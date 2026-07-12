# Task M100 #2124 — 프론트 metrics 공식 모집단·제외군

- 이슈: #2124
- 작성일: 2026-07-10
- 기준 커밋: `upstream/devel` `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 관련 계획: `mydocs/plans/task_m100_2124.md`
- 구현 계획: `mydocs/plans/task_m100_2124_impl.md`
- 측정 스크립트: `scripts/frontend-metrics.mjs`

## 1. 목적

이 문서는 #2124 Phase 0에서 사용할 프론트 웹 metrics의 공식 모집단과 제외군을 고정한다.

핵심 원칙은 다음이다.

- runtime source를 기본 모집단으로 둔다.
- legacy `/web`는 별도 group으로 측정하되 runtime 개선 목표에는 섞지 않는다.
- generated glue, build output, test/e2e, font binary, icon/locales/cert 자산은 제외한다.
- 측정 결과는 Phase 0에서 advisory snapshot이며 fail gate가 아니다.

### 1.1. #1904/#2130 산식 보정 근거

#1904의 17라운드 결산에서는 최대 CC가 288에서 117로, 상위 20 합이 2,336에서 1,726으로
감소했지만 CC 총합은 11,435에서 11,701로 2.3% 증가했다. 최대값과 임계 초과 개수만 추적하면
함수 분할·통이동으로 복잡도를 다른 함수에 옮긴 결과를 개선으로 계상할 수 있다는 한계가 확인됐다.

#2130은 이 교훈을 반영해 CC 총합·상위 20 합·CC>25 합과 함수별 diff를 공식화하고, 총량 순감소를
리팩터링 성공 기준으로 두었다. frontend schema v2도 같은 거버넌스를 적용한다. 요약 상위 목록과
임계 개수만 저장하지 않고 모든 SonarJS 보고 함수의 `cognitiveComplexityEntries`를 보존하며,
`--compare`는 aggregate delta와 stable function id별 diff를 함께 산출한다. Rust와 frontend의 도구별
절대 CC 값은 직접 비교하지 않지만, 복잡도 이동을 성과로 계상하지 않는 판정 원칙은 공유한다.

## 2. 도구

| 항목 | 값 |
|------|----|
| 실행 스크립트 | `node scripts/frontend-metrics.mjs` |
| ESLint | `10.6.0` |
| eslint-plugin-sonarjs | `4.1.0` |
| @typescript-eslint/parser | `8.63.0` |
| TypeScript | `6.0.3` |
| 의존성 위치 | `scripts/frontend-metrics/` private tool package |

루트에는 `package.json`을 만들지 않는다. `npm ci --prefix scripts/frontend-metrics`로 측정 도구만
설치하고 스크립트는 해당 package의 ESLint, sonarjs, parser, TypeScript를 사용한다. Studio의 제품
TypeScript 버전과 측정 parser 호환 범위를 분리해 upstream dependency update가 metrics를 깨뜨리지
않게 한다. 입력은 tracked 파일과 non-ignored untracked 파일을 함께 보며, 출력 인자의 상대 경로는
실행 cwd와 무관하게 저장소 루트를 기준으로 해석한다.

## 3. 포함 모집단

| Group id | 라벨 | 포함 경로 | 비고 |
|----------|------|-----------|------|
| `studio-runtime` | Studio runtime | `rhwp-studio/src/**/*.{ts,tsx,js,mjs}` | 주요 개선 대상 |
| `chrome-extension` | Chrome extension | `rhwp-chrome/**/*.{js,mjs,ts}` | build script 포함 |
| `firefox-extension` | Firefox extension | `rhwp-firefox/**/*.{js,mjs,ts}` | build script 포함 |
| `safari-extension` | Safari extension | `rhwp-safari/src/**/*.{js,mjs,ts}` | Safari source 기준 |
| `shared-frontend` | Shared frontend | `rhwp-shared/**/*.{js,mjs,ts}` | security/shared helper |
| `vscode-extension` | VS Code extension | `rhwp-vscode/src/**/*.{ts,js}` | extension/webview host |
| `npm-editor` | npm editor wrapper | `npm/editor/**/*.{js,ts}` | public embed wrapper |
| `legacy-web` | legacy `/web` | `web/*.{js,mjs,css,html}` | 별도 legacy group |

## 4. 제외군

| 제외군 | 이유 |
|--------|------|
| `node_modules/` | 외부 dependency |
| `dist/` | build output |
| `pkg/` | WASM build output |
| `web/rhwp.js`, `web/rhwp.d.ts`, `web/rhwp_bg.wasm.d.ts` | generated WASM glue/declaration |
| `*.min.js` | minified artifact |
| vendored/generated data | 사람이 작성한 runtime source가 아님 |
| 모든 `test`/`tests`/`e2e` 디렉터리, `*.test.*`, `*.spec.*` | runtime source와 분리 |
| `web/clipboard_test.html` | legacy test page이며 runtime legacy 모집단에서 제외 |
| `web/fonts/`, 향후 `assets/fonts/` | font binary/license inventory 대상이지 complexity 대상이 아님 |
| icons, `_locales`, certs | 정적 자산 |
| snapshot, `output/`, cache 파일 | 산출물/임시 파일 |

## 5. 수집 지표

`scripts/frontend-metrics.mjs`는 다음 지표를 수집한다.

| 지표 | 설명 |
|------|------|
| 파일 LOC | physical line과 nonblank code line |
| 파일 크기 임계 | 1,200 LOC 초과, 2,000 LOC 초과 |
| 함수 LOC | TypeScript AST 기준 function-like node physical LOC |
| cognitive complexity | `sonarjs/cognitive-complexity` threshold 0 advisory 결과 |
| 분포 지표 | 최대 CC, CC>25/CC>100 함수 수 |
| 총량 지표 | CC 총합, 상위 20 합, CC>25 합. #1904/#2130 교훈 반영 |
| 함수별 diff | file/kind/name/occurrence 기반 stable id로 schema v2 snapshot 비교 |
| type surface | TypeScript AST 기준 `any`, `as any`, `this: any` 카운트 |
| export surface | TypeScript AST 기준 export 이름·종류·source 목록 |
| browser duplicate 후보 | 상대 경로 후보에 content SHA-256, byte, symlink target, exact-identical 여부 추가 |
| font reference map | `web/fonts`, `fonts/`, `.woff2`, `FONTS.md`, `THIRD_PARTY_LICENSES` 참조 라인 |
| font asset manifest | 36개 WOFF2와 license 문서의 byte/SHA-256, Studio symlink target |
| 재현성 | HEAD/canonical devel commit, dirty path, measured-source clean, script/lockfile SHA-256 |
| ESLint diagnostics | cognitive complexity 외 fatal/error 진단. inline config는 무시 |

canonical devel commit은 `upstream/devel`, `origin/devel` 순서로 조회한다. 두 remote-tracking ref가
모두 없으면 측정 자체를 중단하지 않고 `upstreamDevelCommit` 속성을 생략한다. fork와 본가 clone에서
같은 도구를 실행하기 위한 provenance fallback이며, `HEAD`와 나머지 재현성 정보는 항상 기록한다.

## 6. 출력 형식

기본 실행:

```bash
node scripts/frontend-metrics.mjs
```

명시 실행:

```bash
node scripts/frontend-metrics.mjs \
  --out output/frontend-metrics/metrics.json \
  --summary output/frontend-metrics/summary.md
```

직전 schema v2 snapshot 비교:

```bash
node scripts/frontend-metrics.mjs \
  --compare mydocs/metrics/frontend/2026-07-11/metrics.json \
  --out output/frontend-metrics/metrics-next.json \
  --summary output/frontend-metrics/summary-next.md
```

Stage 1에서는 `output/frontend-metrics/`에 검증용 임시 산출물을 만든다. 공식 snapshot은 Stage 2에서
다음 경로에 저장한다.

```text
mydocs/metrics/frontend/2026-07-11/metrics.json
mydocs/metrics/frontend/2026-07-11/summary.md
```

## 7. 해석 규칙

- Phase 0 결과는 기준선 고정용이다. CC>25가 있더라도 PR fail gate로 쓰지 않는다.
- `legacy-web` 수치는 `/web` 삭제 여부 판단의 입력일 뿐, `rhwp-studio` runtime 개선 수치와 합산하지 않는다.
- `any`/`as any`/`this: any`는 AST 기반이며 포함 관계가 있으므로 단순 합산하지 않는다.
- cognitive complexity 값은 SonarJS 산식이다. v1 heuristic CC 및 Rust clippy cognitive complexity와 직접 비교하지 않는다.
- hotspot 분포 개선과 총량 감소를 구분한다. 통이동은 총량이 줄지 않으면 개선 성과로 계상하지 않는다.
- 전체 worktree가 dirty여도 `measuredSourceClean=true`여야 공식 snapshot으로 인정한다.
- 스크립트가 기록한 `eslintDiagnostics`는 산식 해석 보조 정보이며, Stage 2 snapshot에 함께 보존한다.

### 과도기 증가와 예외 심사

Phase A/B 추출 PR에서 CC>25 count가 일시적으로 +1~2 늘 수 있으나 자동 허용하지 않는다. 다음 조건을
모두 만족할 때 reviewer가 예외를 승인할 수 있다.

- 대상 hotspot의 Max CC 또는 함수 LOC가 감소한다.
- PR의 변경 책임이 하나로 제한된다.
- 변경 유형에 해당하는 smoke/contract gate가 통과한다.
- Total CC와 CC>25 sum 변화가 공개되고, 단순 복잡도 이동이 아님을 설명한다.
- PR 본문에 다음 anchor와 해소 계획이 있다.

예외 기록 표에는 `functionId`, before/after CC, 사유, 소유 영역, 해소 조건, 다음 anchor, 재검토 Phase를
포함한다. 예외는 영구 면제가 아니며 후속 PR에서 해소한다. Phase 0 baseline의 승인 예외는 0건이다.

## 8. Stage 1 검증 결과

다음 명령을 실행해 스크립트 동작을 확인했다.

```bash
npm ci --prefix scripts/frontend-metrics
node scripts/frontend-metrics.mjs --help
node --check scripts/frontend-metrics.mjs
node scripts/frontend-metrics.mjs --out output/frontend-metrics/metrics.json --summary output/frontend-metrics/summary.md
node scripts/frontend-metrics.mjs --compare output/frontend-metrics/metrics.json --out /tmp/frontend-metrics-compare.json --summary /tmp/frontend-metrics-compare.md
```

검증용 실행 결과:

| 항목 | 값 |
|------|---:|
| schema | 2 |
| 포함 파일 | 203 |
| CC 총합 | 11,805 |
| 상위 20 합 | 2,581 |
| CC>25 | 62 |
| CC>25 합 | 3,932 |
| CC>100 | 6 |
| ESLint diagnostics | 0 |

기존 210개에서 extension/shared test 6개와 `web/clipboard_test.html` 1개를 제외했다. schema v2 자기
비교 결과 총량 delta와 함수별 diff가 모두 0이었다.

기준 커밋을 `782059d9`에서 `ebf05268`로 이동하는 동안 제품 code lines는 66,968에서 67,037로,
functions는 4,720에서 4,725로, Total CC는 11,768에서 11,774로 변했다. 최종 merge 직전 #2188을
반영한 `6f1bd284`에서는 code lines 67,069, Total CC 11,788이 됐다. 함수별 diff는 #2188의
`renderTextRun` +16과 `recordTextRunCoverageGaps` -2를 식별했고 Top 20 및 CC>25/100 지표는 변하지
않았다. 이어진 `acc841c9`는 측정 대상 밖의 문서 전용 변경이므로 모든 지표와 함수별 diff가 0-delta였다.
공식 snapshot은 최신 upstream provenance를 포함한 `acc841c9` 기준으로 다시 생성했다.

그 뒤 `3077f96d`의 #2184/#2191이 Studio CanvasKit source를 변경해 code lines 67,155, functions 4,728,
Total CC 11,805, CC>25 62개/합 3,932가 됐다. `acc841c9` 대비 Total CC +17은 네 함수 diff로 전부
설명됐고 Top 20, CC>100, Max는 변하지 않았다. 공식 snapshot은 이 최신 upstream을 새 영점으로
재생성했다.

같은 산식으로 Stage 2 공식 baseline을 `mydocs/metrics/frontend/2026-07-11/`에 저장했다.
