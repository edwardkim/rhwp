# 프론트 웹 리팩터링 계획 v2 — Task M100 #2023

- 이슈: #2023 (umbrella: #2022)
- 작성일: 2026-07-09
- v1 예비 진단 기준 커밋: `upstream/devel` `a23b8ae1c85fc80547c40d8e7bbbf37a07283468`
- 문서 PR 기준 커밋: 2026-07-09 fetch 기준 `upstream/devel` `3728abfb`
- 상태: **리뷰 반영 v2 — 재리뷰 요청 완료**
- v1 문서: `mydocs/plans/task_m100_2023_frontend_refactoring_plan.md`
- 관련 진단: `mydocs/tech/investigations/issue-2023/task_m100_2023_frontend_diagnosis.md`
- 관련 guardrail: `mydocs/tech/investigations/issue-2023/task_m100_2023_frontend_contract_guardrails.md`
- GitHub v2 리뷰 요청 댓글: https://github.com/edwardkim/rhwp/issues/2023#issuecomment-4917066612
- GitHub 문서 전용 PR: https://github.com/edwardkim/rhwp/pull/2080

## 0. 리뷰 반영 결론

maintainer와 collaborator 리뷰는 같은 방향으로 수렴했다.

- 계획 방향은 동의.
- #1883과 같은 **SOLID + 복잡도** 2축 거버넌스 적용은 타당.
- 현재 v1 계획은 바로 실행 이슈로 넓게 분리하지 않고 **v2 보완 후 계획 승인**이 필요.
- 실행 하위 이슈는 v2 승인 후에도 우선 **Phase 0 baseline freeze**와 **Phase A font
  ownership/canonical 이전**까지만 만든다.
- `InputHandler`, dialog, `WasmBridge`, `diff-engine` 등 Phase B 해체는 baseline과 guardrail이
  실제로 고정된 뒤 재평가한다.

작업지시자 결정으로 v2에는 다음 선택을 확정 반영한다.

| 항목 | v2 결정 |
|------|---------|
| TS 복잡도 측정 도구 | `eslint` + `sonarjs/cognitive-complexity` 우선. Phase 0에서는 snapshot/advisory |
| font canonical | 공통 루트 `assets/fonts` 신설 방향 |
| 하위 이슈 생성 | v2 승인 전 생성 금지. 승인 후 Phase 0/A만 먼저 생성 |

## 1. 실행 대원칙 — #1883 v2 stage-gate 준용

프론트 웹 리팩터링도 #1883 v2의 stage-gate 원칙을 그대로 준용한다.

1. **관문형 순차 진행**: Phase 0 → Phase A → 재평가 → Phase B 이후 순서로 진행한다.
2. **병행·중첩 금지**: 같은 파일·계약 표면을 건드리는 리팩터링을 동시에 진행하지 않는다.
3. **각 단계는 독립 종료 가능**해야 한다. 다음 Phase 없이도 `devel` 안정 상태로 남아야 한다.
4. **후속 범위 재결정**: Phase 경계마다 metric snapshot과 smoke 결과를 보고 다음 Phase 범위를
   축소·이연·중단할 수 있다.
5. **리팩터링과 기능/보안 변경 혼합 금지**: 무변동 판정을 보존하기 위해 구조 변경과 동작 변경은
   같은 PR에 섞지 않는다.

## 2. v1 예비 수치의 지위

`mydocs/tech/investigations/issue-2023/task_m100_2023_frontend_diagnosis.md`의 수치는 공식 baseline이 아니라
**heuristic baseline**이다.

이유:

- 현재 `rhwp-studio`에는 공식 ESLint/complexity 설정이 없다.
- v1의 CC는 `if`, `for`, `case`, `&&`, `||` 등 토큰 기반 근사치다.
- SonarJS cognitive complexity, ESLint cyclomatic complexity, Rust clippy cognitive complexity는
  같은 숫자를 만들지 않는다.
- 2026-07-09 현재 `upstream/devel`이 v1 기준 커밋보다 전진했으므로, Phase 0 공식 baseline은
  반드시 당시 최신 upstream 기준으로 재측정해야 한다.

따라서 v1의 다음 수치는 “우선순위 판단용 예비 신호”로만 사용한다.

| 항목 | v1 값 | v2에서의 지위 |
|------|------:|---------------|
| `rhwp-studio/src` LOC 53,525 | 유지 | LOC 예비 규모 |
| `rhwp-studio/src` CC>25 함수 49 | 격하 | 공식 도구 재측정 전까지 heuristic |
| 최대 CC 321 | 격하 | 공식 도구 재측정 전까지 heuristic |
| SOLID 54/100 | 격하 | 공식 metric·채점 앵커 확정 후 재고정 |

v2 승인 후 Phase 0에서 공식 baseline을 새로 만든다.

## 3. 복잡도 측정 도구와 산식

### 3.1 도구 선택

Phase 0에서 도입할 공식 후보는 다음으로 고정한다.

- ESLint 기반 실행.
- `eslint-plugin-sonarjs`의 `sonarjs/cognitive-complexity`를 주 metric으로 사용.
- ESLint 기본 `complexity`는 보조 metric 또는 비교용으로만 사용.
- `@typescript-eslint/parser`를 사용해 TypeScript를 파싱한다.

선택 이유:

- #1883의 Rust 측정 축이 cognitive complexity였으므로 의미가 가깝다.
- 함수 추출 과정에서 cyclomatic complexity보다 중첩·분기 구조 악화를 더 잘 드러낸다.
- Phase 0에서 advisory로 시작한 뒤, 실제 분포를 보고 fail gate 승격 여부를 결정할 수 있다.

### 3.2 Phase 0 적용 방식

Phase 0에서는 패키지·스크립트 변경을 별도 실행 이슈에서 수행한다.

권장 산출물:

- ESLint/sonarjs 설정 추가안.
- `scripts/frontend-metrics.*` 또는 `scripts/metrics.sh --frontend` 통합안.
- JSON snapshot: `mydocs/metrics/frontend/YYYY-MM-DD/metrics.json`
- dashboard snapshot: `mydocs/metrics/frontend/YYYY-MM-DD/dashboard.html` 또는 동등 산출물.

Phase 0에서는 **fail gate가 아니라 snapshot/advisory**로 시작한다. 기준선과 예외 정책이
승인되기 전까지 숫자만으로 PR을 막지 않는다.

### 3.3 모집단

공식 complexity 모집단은 기본적으로 runtime source만 포함한다.

포함:

- `rhwp-studio/src/**/*.{ts,tsx,js,mjs}`
- `rhwp-chrome/**/*.{js,mjs,ts}`
- `rhwp-firefox/**/*.{js,mjs,ts}`
- `rhwp-safari/src/**/*.{js,mjs,ts}`
- `rhwp-shared/**/*.{js,mjs,ts}`
- `rhwp-vscode/src/**/*.{ts,js}`
- `npm/editor/**/*.{js,ts}`
- legacy `/web`의 사람이 작성한 JS/CSS/HTML은 별도 legacy group으로 측정하되, runtime
  improvement target에는 섞지 않는다.

제외:

- `node_modules/`
- `dist/`
- `pkg/`
- generated WASM glue와 declaration: `web/rhwp.js`, `web/rhwp.d.ts`, `web/rhwp_bg.wasm.d.ts`
- `*.min.js`
- vendored/generated data
- `rhwp-studio/e2e/`
- `rhwp-studio/tests/`
- font binary와 `web/fonts/`, 향후 `assets/fonts/`
- icons, `_locales`, certs
- snapshot/output/cache 파일

`/web`는 특히 별도 표로 분리한다. `web/rhwp.js` 같은 배포 산출물 포함 여부에 따라 LOC와
복잡도 수치가 크게 달라지므로, legacy 사람이 작성한 코드와 generated glue를 같은 지표에 섞지
않는다.

### 3.4 지표

Phase 0 공식 snapshot은 다음 지표를 최소 포함한다.

- 파일 LOC, 1,200 LOC 초과 파일, 2,000 LOC 초과 파일.
- 함수 LOC 상위.
- `sonarjs/cognitive-complexity` 상위 함수.
- CC>25, CC>100 함수 수.
- `any`, `as any`, `this: any` 수.
- export 수와 public surface 후보.
- 브라우저 확장 duplicated file/logic 후보.
- font reference map.

### 3.5 과도기 허용과 예외 심사제

Rust 리팩터링에서 확인된 것처럼, 추출 직후에는 CC>25 개수가 일시적으로 +1~2 증가할 수 있다.
프론트도 다음 조건을 모두 만족할 때에만 과도기 증가를 허용한다.

- 대상 hotspot의 최대 complexity 또는 함수 LOC가 감소한다.
- 변경 책임이 한 가지로 제한된다.
- 무변동 게이트가 통과한다.
- PR 설명에 다음 앵커가 명시된다.
- 증가한 함수는 예외 목록에 등록하고 후속 PR에서 해소한다.

예외는 영구 면제가 아니다. 예외 항목에는 사유, 소유 영역, 해소 조건, 재검토 Phase를 적는다.

## 4. SOLID 채점 방식

`mydocs/manual/solid_scoring_guide.md`를 그대로 쓰되, 프론트 예시는 별도 부록으로 고정한다.

v2에서 즉시 적용하는 임시 앵커:

| 원칙 | 20점 | 16점 | 12점 | 8점 이하 |
|------|------|------|------|----------|
| S | event/input/dialog/diff 책임이 모듈 경계로 강제됨 | 일부 hotspot이 있으나 추적됨 | God function 누적 | 대형 coordinator가 동작 변경까지 흡수 |
| O | asset/browser/backend 추가가 확장 지점으로 끝남 | 일부 build path 수정 필요 | 여러 target 동시 수정 반복 | target 추가가 핵심 로직 변경을 강제 |
| L | editor/embed/webview/extension 계약이 구현별 의미 차이 없음 | 국소 wrapper 차이 | target별 예외가 문서 없이 존재 | 같은 API가 target마다 다르게 동작 |
| I | public type/API가 소비자별로 좁음 | 큰 타입 surface가 추적됨 | `core/types.ts`/bridge 표면 과대 | 대부분이 any/public blob에 의존 |
| D | core가 DOM/build/extension detail에 직접 의존하지 않음 | 얇은 adapter로 격리 | platform detail이 core 주변에 누적 | core 변경이 target별 수정으로 번짐 |

v1의 SOLID 54/100은 이 앵커가 공식화되기 전 예비 점수다. Phase 0 snapshot 이후 재채점한다.

## 5. 확정 금지 목록

다음 항목은 후보가 아니라 v2 기준 **확정 금지 규칙**이다.

- `@rhwp/editor` iframe/postMessage/public embed 계약 변경 금지.
- `@rhwp/editor` runtime dependency 추가 금지.
- React/Vue/Svelte 등 runtime UI framework 도입 금지(v1.0까지).
- 확장 CSP 완화 금지.
- inline script 추가 금지.
- `web_accessible_resources` 확대 금지.
- sender/URL/file signature/size 검증 약화 금지.
- `publicDir: false` 규칙 우회 금지.
- Chrome/Firefox `build.mjs`의 명시적 copy 계약을 암묵 publicDir 동작으로 대체 금지.
- 리팩터링 PR과 기능/보안 변경 커밋 혼합 금지.
- font fallback 변경과 font path 이전 혼합 금지.
- 저작권 보호 대상 한컴/MS 폰트 번들 금지.
- `/web` legacy 삭제와 font canonical 이전을 같은 PR에 혼합 금지.

판단이 애매하면 실행하지 않고 별도 이슈로 분리한다.

## 6. 3-browser sync 규칙

Chrome/Firefox/Safari 중 하나의 확장 코드나 asset copy 계약을 건드리면 다음 확인을 필수로 둔다.

- 나머지 두 브라우저의 대응 파일·빌드 경로 영향 확인.
- `rhwp-shared`로 올라갈 수 있는 보안/다운로드/URL 검증 로직인지 확인.
- manifest CSP와 `web_accessible_resources` 변화 없음 확인.
- `publicDir: false` 유지 확인.
- 빌드 산출물에서 필요한 asset 누락, 404, CSP 오류가 없는지 smoke 확인.

이 규칙은 Rust 쪽 renderer 다중 backend sweep 규칙의 프론트 확장판이다.

## 7. Phase 0 — Frontend Baseline Freeze

목표: 실행 전에 측정기, 산식, contract, smoke 기준을 먼저 고정한다.

Phase 0 산출물:

| 산출물 | 내용 | 필수 여부 |
|--------|------|----------|
| complexity tool 결정 | ESLint + sonarjs 설정, 실행 명령, JSON 출력 | 필수 |
| metrics snapshot | 공식 모집단 기준 LOC/complexity/any/export/duplication | 필수 |
| baseline manifest | 기준 commit, OS, node/npm, 브라우저, 폰트 경로, 제외군 | 필수 |
| public contract snapshot | `@rhwp/editor`, `@rhwp/core`, VS Code message, extension message | 필수 |
| WASM JSON schema snapshot | frontend 소비자 쪽 ownership. Rust 계획과 상호 참조 | advisory |
| font inventory | 현재 `web/fonts` 파일 목록, 참조 경로, license 문서 | 필수 |
| extension security snapshot | CSP, `web_accessible_resources`, sender/URL/file 검증 | 필수 |
| smoke manifest | 변경 표면별 무변동 게이트 목록 | 필수 |

권장 무변동 게이트:

| 표면 | 게이트 |
|------|--------|
| `rhwp-studio` 기본 | `npm run build`, `npm run test` |
| studio 입력/렌더 | `npm run e2e`, 관련 e2e smoke, 필요 시 `e2e:renderer-contract` |
| render/layout 영향 | `e2e:baseline:headless`, `e2e:render-diff` 중 실행 가능 항목 |
| Chrome extension | `npm run build` in `rhwp-chrome`, dist asset/CSP smoke |
| Firefox extension | `npm run build` in `rhwp-firefox`, dist asset/CSP smoke |
| Safari extension | Chrome dist 기반 영향 확인, Safari source/asset path 검토 |
| VS Code | `npm run compile` in `rhwp-vscode`, webview font/WASM smoke |
| `@rhwp/editor` | embed smoke, iframe/postMessage API snapshot |

Phase 0은 실행 이슈로 별도 등록한다. 이 v2 문서에서는 설정 파일이나 package manifest를 변경하지
않는다.

## 8. Phase A — `assets/fonts` canonical 이전

목표: legacy `/web`와 실사용 font asset root를 분리한다.

v2 결정:

- 새 canonical 위치는 공통 루트 `assets/fonts` 방향으로 둔다.
- `rhwp-studio/public/fonts`는 canonical이 아니라 build/copy/symlink 소비자 중 하나로 취급한다.
- `/web/fonts`는 최종적으로 canonical이 아니어야 한다.

실행 순서:

1. inventory: 현재 `web/fonts` 파일, license, 참조 경로, build copy 경로를 스냅샷으로 남긴다.
2. canonical 결정: `assets/fonts` 신설과 폴더 책임 문서를 확정한다.
3. target copy 계약: studio/chrome/firefox/safari/vscode/npm 각 target의 font 수신 방식을
   명시한다.
4. 문서 갱신: `FONTS.md`, `THIRD_PARTY_LICENSES.md`, npm/editor README, extension docs를 갱신한다.
5. legacy 검토: `web/fonts` 의존 제거 후 `/web` JS/HTML/CSS 삭제 또는 archive 여부를 별도 이슈로
   판단한다.

Phase A 금지:

- font path 이전과 font fallback 변경 혼합 금지.
- font 파일 추가/삭제와 경로 이전 혼합 금지.
- `/web` legacy 삭제와 canonical 이전 혼합 금지.
- 확장 `web_accessible_resources` 확대 금지.

## 9. Phase B 이후 — 보류와 재평가

Phase B의 대상은 여전히 중요하다.

- `InputHandler` typed context.
- keyboard/mouse 고복잡도 함수 해체.
- dialog state/apply 분리.
- `WasmBridge` typed adapter.
- `core/types.ts` domain surface 분리.
- `diff-engine` 책임 분리.

하지만 v2 기준에서는 Phase B 실행 이슈를 아직 만들지 않는다.

보류 이유:

- 공식 complexity 도구와 산식이 아직 실행 반영되지 않았다.
- 무변동 게이트가 아직 manifest로 고정되지 않았다.
- font/asset ownership이 `rhwp-studio`, extension, VS Code, npm 계약에 영향을 준다.

Phase B는 Phase 0/A 완료 후 다음 기준으로 재평가한다.

- 위험 낮은 순 × 복잡도 높은 순.
- 한 책임/추출 = 1 PR.
- PR 설명 3요소 필수: responsibility, 무변동 gate, 다음 앵커.
- 의존이 일정 임계를 넘으면 함수 추출보다 struct/context 설계를 먼저 한다.

## 10. 하위 이슈 생성 정책

v2 승인 전에는 실행 하위 이슈를 만들지 않는다.

v2 승인 후 먼저 만들 이슈는 다음 2개로 제한한다.

1. `[프론트] baseline freeze: 계약·렌더·확장·폰트 기준선 고정`
2. `[프론트] assets/fonts canonical 이전 계획 및 경로 갱신`

아래 이슈는 Phase 0/A 완료 후 재평가한다.

- `InputHandler typed context`
- keyboard/mouse event handler 해체
- dialog state/apply 분리
- `WasmBridge` JSON boundary typed adapter
- `core/types.ts` domain surface 분리
- `diff-engine` 책임 분리
- extension shared security helper 확대
- VS Code webview contract 정리
- npm package docs contract 정리

## 11. 리뷰 응답 매핑

| 리뷰 | v2 반영 |
|------|---------|
| edwardkim: 복잡도 도구 확정 | ESLint + sonarjs cognitive complexity를 Phase 0 공식 후보로 고정 |
| edwardkim: 산식·모집단 고정 | 포함/제외군, generated glue, `/web` 별도 group 명시 |
| edwardkim: 행동 고정 게이트 | smoke manifest와 표면별 게이트 추가 |
| edwardkim: stage-gate 준용 | §1에 #1883 v2 원칙 준용 명시 |
| edwardkim: 금지 목록 추가 | 3-browser sync, 혼합 금지, publicDir false/inline script 확정 승격 |
| edwardkim: 실측 수치 선반영 | v1 수치는 heuristic으로 격하, Phase 0 공식 재측정으로 처리 |
| jangster77: Phase 0/A만 먼저 | 하위 이슈 생성 정책에 반영 |
| jangster77: `assets/fonts` 선호 | Phase A canonical 방향으로 반영 |
| jangster77: 확정 금지 규칙 | §5에 후보가 아닌 확정 금지 목록으로 반영 |
| jangster77: WASM schema ownership | frontend 소비자 쪽 advisory ownership으로 반영 |

## 12. 결론

프론트 웹 리팩터링은 진행한다. 다만 지금은 코드 리팩터링 착수가 아니라 **v2 계획 승인**이 다음
관문이다.

최종 방향:

1. v2 재리뷰를 요청한다.
2. v2 승인 후 Phase 0 baseline freeze 이슈를 먼저 만든다.
3. Phase 0에서 공식 metric, 모집단, 무변동 게이트를 실제 산출물로 고정한다.
4. 이어서 Phase A `assets/fonts` canonical 이전 이슈만 분리한다.
5. Phase B 대형 모듈 해체는 Phase 0/A 결과를 보고 다시 승인받는다.

이 문서는 계획 보완본이며 코드 변경은 포함하지 않는다.
