# task m100-2752 — @rhwp/editor README `getHmlSaveState()` 반환 형태 오문서 수정

## 이슈

- **Issue**: [#2752](https://github.com/edwardkim/rhwp/issues/2752) — `npm/editor/README.md`의
  `editor.getHmlSaveState()` 절이 실재하지 않는 반환 형태 `{ ok, blocker }` 를 문서화

## 분석

`npm/editor/README.md:212`~`219` 이 다음과 같이 적혀 있었다.

```javascript
const state = await editor.getHmlSaveState();
// { ok: true } 또는 { ok: false, blocker: '...' }
```

`ok` / `blocker` 는 이 API 와 관련해 저장소 어디에서도 정의되거나 반환된 적이 없는 키다.
실제 wire DTO 는 `{ sourceFormat, hmlSavable, blockers[] }` 이며 다음 6곳이 이를 못박는다.

| # | 근거 | 위치 |
|---|------|------|
| 1 | canonical Rust DTO `struct HmlSaveState { source_format, hml_savable, blockers }` (serde `camelCase`) | `src/wasm_api.rs:272`~`287` |
| 2 | `source_format_name()` 이 `"hwpx" \| "hml" \| "hwp"` 만 반환 | `src/wasm_api.rs:317`~`323` |
| 3 | studio 브리지가 `HmlSaveState { sourceFormat, hmlSavable, blockers }` 로 파싱·반환 | `rhwp-studio/src/core/wasm-bridge.ts:329`~`336`, `rhwp-studio/src/core/hml-save-capability.ts:23`~`27` |
| 4 | embed 핸들러가 가공 없이 그대로 응답 | `rhwp-studio/src/main.ts:1334`~`1336` |
| 5 | CI 계약 테스트가 `deepEqual` 로 정확한 DTO 고정 | `scripts/frontend-editor-embed.test.mjs:74`~`81` |
| 6 | studio e2e 가 `state.sourceFormat === 'hml' && state.hmlSavable && state.blockers.length === 0` 단언 | `rhwp-studio/e2e/hml-equation-embed.test.mjs:264` |

같은 패키지의 `npm/editor/index.d.ts:35`~`46` 도 `HmlSaveState` / `HmlSaveBlocker` 를 정확히 선언하고
있고, `npm/editor/index.js:202`~`205` 는 studio 응답을 가공 없이 통과시키므로 SDK 계층에 `ok`/`blocker`
어댑터도 없다.

### 왜 의도된 표기가 아닌가

```
$ git log -S "{ ok: false, blocker" -- npm/editor/README.md
a0f839d7 docs(editor): README에 exportHml·getHmlSaveState API 문서 추가 (#2691)
```

해당 절을 도입한 유일한 커밋이며, 그 이전 README 에는 절 자체가 없었다. 즉 과거 형태의 보존도, 의도적
축약 표기도 아니다.

### 사용자 영향

README 는 `package.json` 의 `files` 배열에 포함되어 npm 타르볼로 배포되고 npmjs.com 패키지 첫 화면으로
렌더링된다. README 예제를 그대로 따른 JavaScript 소비자는

- `state.ok` → `undefined` (항상 falsy) → **HML 저장이 가능한 문서에서도 저장 분기에 절대 진입하지 못함**
- `state.blocker` → `undefined` → 사용자에게 `"...: undefined"` 표시

TypeScript 소비자는 `index.d.ts` 덕분에 컴파일 오류로 걸러진다. 즉 이 결함의 유일한 발현 표면이 문서다.

## 변경

### 1. `npm/editor/README.md` — `### editor.getHmlSaveState()` 절 교체

인접한 올바른 형제 절 `editor.exportHwpVerify()` (`README.md:178`~`194`, 필드가 `index.d.ts` 의
`HwpVerifyResult` 와 정확히 일치)의 서술 방식을 따랐다.

- 실제 반환 객체를 주석으로 제시 (`sourceFormat`, `hmlSavable`, `blockers[].{code,xmlPath,message,preserved}`)
- `hmlSavable` 로 분기하고 `blockers` 를 표시하는 사용 예제 추가 — 오류 문자열 파싱을 금지한 SPEC 계약
  (`docs/changelog/2026-07/13-pr-2219-hml-equation-export-spec/SPEC.md:247`, `:321`)과 일치
- 반환값 / `HmlSaveBlocker` 필드 표 추가. `sourceFormat` 값 범위는 근거 #2, `preserved` 가 항상 `false`
  인 점은 `src/wasm_api.rs:305` 및 `index.d.ts:39` 근거

`index.js` / `index.d.ts` / studio / Rust 는 모두 이미 정확하므로 **코드 변경 없음**.

### 2. `npm/editor/tests/hml-save-state-doc.contract.test.mjs` — 신규 계약 테스트

`index.d.ts` 원문을 파싱해 계약을 고정하는 기존 선례
(`npm/editor/tests/renderer-diagnostics-v1.contract.test.mjs`)를 그대로 따른다.

- `HmlSaveState` / `HmlSaveBlocker` 필드 7개를 `index.d.ts` 에서 추출하고, README 절이 그 7개를 모두
  **객체 키 형태**(`\bfield\s*:`)로 보이는지 검사
- 실재하지 않는 `ok:` / `blocker:` 표기가 절에 없는지 검사 (`blockers:` / `blocker.code` 같은 정상 표기는
  걸리지 않음)
- 체크아웃에 따라 CRLF 일 수 있으므로 두 파일 모두 LF 로 정규화 후 비교

`npm/editor/package.json` 의 `test` 스크립트가 `tests/*.test.mjs` 를 실행하므로
`.github/workflows/ci.yml:685` 의 `npm --prefix npm/editor test` 에 자동 포함된다.

## 검증

### RED → GREEN (실제 실행)

테스트를 먼저 추가하고 README 수정 전 상태에서 실행 — **2/2 실패**.

```
$ node --test tests/hml-save-state-doc.contract.test.mjs
✖ README getHmlSaveState 절은 HmlSaveState 선언과 같은 필드를 문서화한다 (8.4593ms)
✖ README getHmlSaveState 절은 실재하지 않는 반환 필드를 문서화하지 않는다 (0.5233ms)
ℹ tests 2
ℹ pass 0
ℹ fail 2

✖ failing tests:

✖ README getHmlSaveState 절은 HmlSaveState 선언과 같은 필드를 문서화한다
  AssertionError [ERR_ASSERTION]: README 가 반환값 필드 sourceFormat 를 객체 키 형태로 보여야 한다
    actual: "### editor.getHmlSaveState()\n\n현재 문서의 HML 저장 가능 여부와 blocker를 반환합니다.\n\n
             ```javascript\nconst state = await editor.getHmlSaveState();\n
             // { ok: true } 또는 { ok: false, blocker: '...' }\n```\n",
    expected: /\bsourceFormat\s*:/,
    operator: 'match',

✖ README getHmlSaveState 절은 실재하지 않는 반환 필드를 문서화하지 않는다
  AssertionError [ERR_ASSERTION]: getHmlSaveState 는 ok 필드를 반환하지 않는다
    actual: "### editor.getHmlSaveState()\n\n... // { ok: true } 또는 { ok: false, blocker: '...' }\n```\n",
    expected: /\bok\s*:/,
    operator: 'doesNotMatch',
```

README 수정 후 재실행 — **2/2 통과**.

```
$ node --test tests/hml-save-state-doc.contract.test.mjs
✔ README getHmlSaveState 절은 HmlSaveState 선언과 같은 필드를 문서화한다 (6.5352ms)
✔ README getHmlSaveState 절은 실재하지 않는 반환 필드를 문서화하지 않는다 (0.6775ms)
ℹ tests 2
ℹ pass 2
ℹ fail 0
```

### CI 게이트

| 명령 | 결과 |
|------|------|
| `npm --prefix npm/editor test --if-present` (ci.yml:685) | **21 pass / 0 fail** (기존 19 + 신규 2) |
| `node --test scripts/frontend-editor-embed.test.mjs` (ci.yml:682) | **2 pass / 0 fail** — 회귀 없음 |

`.rs` 변경이 없으므로 cargo 게이트(clippy / test / rustfmt)는 해당 없음.

### 로컬 환경 한정 실패 (이번 변경과 무관, 사전 존재)

- `scripts/frontend-wasm-bindings.test.mjs` — `pkg/rhwp.d.ts` 부재로 `ENOENT`. `pkg/` 는 wasm-pack
  빌드 산출물이라 로컬 체크아웃에 없다.
- `rhwp-chrome/sw/*.test.mjs`, `rhwp-firefox/sw/*.test.mjs` — `SyntaxError: Unexpected token '.'`.
  `sw/document-url-resolver.js` 등이 `rhwp-shared/sw/` 를 가리키는 **심볼릭 링크**인데 Windows 체크아웃에서
  링크 대상 경로 문자열(`../../rhwp-shared/sw/...`)을 담은 일반 파일로 복원되어 모듈 파싱이 실패한다.
  두 경우 모두 이번 diff(`npm/editor/` 2파일) 밖이며 Linux CI 에서는 발생하지 않는다.

## 결과

- **Branch**: `task/m100-2752-editor-readme-hml-save-state`
- **변경 파일 2개**: `npm/editor/README.md` (수정), `npm/editor/tests/hml-save-state-doc.contract.test.mjs` (신규)
- **Closes**: #2752

## 범위 밖 (잔여)

- README 절 배치 순서(`exportHml`/`getHmlSaveState` 가 `exportHwpVerify` 뒤)와 `index.js`/`index.d.ts`
  선언 순서(앞) 불일치 — 사실 오류가 아니므로 제외.
- `exportHml()` / `getHmlSaveState()` 가 `getRendererDiagnostics()` 와 달리
  `transport.supports('hml-export')` 를 검사하지 않는 비대칭 — legacy postMessage 폴백에서
  `_peerCapabilities` 가 비워지므로(`npm/editor/transport.js:231`) 하드 capability 검사는 legacy 경로를
  깨뜨린다. 의도된 설계로 판단해 제외.
- `rhwp-shared/security/` 의 `url-validator.js` / `filename-sanitizer.js` / `security-log.js` /
  `sender-validator.js` 가 현재 어떤 확장에서도 import 되지 않는 상태 — 별도 사안.
