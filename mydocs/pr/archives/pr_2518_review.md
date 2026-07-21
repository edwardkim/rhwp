# PR #2518 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#2518](https://github.com/edwardkim/rhwp/pull/2518) |
| 관련 이슈 | [#2513](https://github.com/edwardkim/rhwp/issues/2513) |
| 작성자 / base | [@cskwork](https://github.com/cskwork) / `devel` |
| 검토 범위 | `@rhwp/editor.loadFile()`의 대화상자 기본값, transport 계약, npm package surface |
| 원 PR 규모 | +494/-21, 12 files |
| 원 PR merge | 2026-07-21, [`a621ac1`](https://github.com/edwardkim/rhwp/commit/a621ac19c95a5bc2f79086408b17c9a6ae94f442) |
| 최종 판단 | 수용 및 merge 완료. [#2513](https://github.com/edwardkim/rhwp/issues/2513)은 원 PR merge 뒤 자동 close됨 |

## 변경과 판단

이 PR은 `@rhwp/editor`의 `loadFile(data, fileName)`에서 `suppressDialogs`를 생략하면 `true`를 보내도록
변경했다. iframe 내부의 HWPX 검증 또는 로컬 글꼴 안내창이 사용자 입력을 기다려 embed caller의 Promise가
timeout되는 문제를 SDK 경계에서 막는다.

명시적 `{ suppressDialogs: false }`는 대화형 흐름을 유지하고, 명시적 `true`도 기존 비대화형 동작을 유지한다.
raw embed protocol과 Studio 직접 열기의 생략 기본값은 바꾸지 않아 SDK 외 caller의 계약도 보존한다. README와
`index.d.ts`는 새 기본값과 대화형 opt-out을 같은 의미로 설명한다.

`renderer: 'auto'`를 명시한 기존 embed E2E 변경은 이 이슈의 직접 수정 범위는 아니다. 따라서 reviewer는
별도 headless smoke에서 renderer 옵션을 생략한 `createEditor()`와 zero-option `loadFile()`도 확인했다.
`canvas2d` 요청 상태에서 7페이지 문서가 제한 시간 안에 완료되어, 기본 renderer 경로가 테스트에서 빠지지
않았음을 확인했다.

## 검증

- `npm --prefix npm/editor test`: 19/19 통과. 생략, 명시 `false`, 명시 `true`의 `suppressDialogs` 전달 계약을
  포함한다.
- `node --test scripts/frontend-wasm-bindings.test.mjs scripts/frontend-editor-embed.test.mjs`: 3/3 통과.
  public MessageChannel v1, binary transport, legacy fallback, WASM binding 선언을 확인했다.
- `cd rhwp-studio && npx tsc --noEmit --skipLibCheck ../npm/editor/index.d.ts`: 통과.
- `npm --prefix npm/editor pack --dry-run --json`: `index.js`, `index.d.ts`, `transport.js`, README,
  `package.json`의 5개 publish 파일을 확인했다.
- 실제 headless iframe smoke: renderer 옵션을 생략한 `createEditor()`에서 `loadFile()`이 사용자 대화상자
  클릭 없이 완료됐고 `pageCount: 7`, requested/backend 모두 `canvas2d`였다.
- 원 PR 최종 head [`d63d0e2`](https://github.com/edwardkim/rhwp/commit/d63d0e2147b194b01a3f70b1a04b42137491a275)의
  CI, CodeQL, Render Diff, Native Skia, frontend package gates와 default-feature shard가 모두 성공했다.

## 시각 검증 판정

SDK request option과 package 문서·테스트만 변경하며 renderer, layout, typeset, PDF/SVG 출력 경로는 바꾸지
않는다. 따라서 visual sweep과 별도 image asset은 필요하지 않다.

## 후속 상태

원 PR merge 직후 GitHub 상태 전파를 다시 확인한 결과, [#2513](https://github.com/edwardkim/rhwp/issues/2513)은
자동으로 close됐다. 별도 수동 close는 필요하지 않다.
