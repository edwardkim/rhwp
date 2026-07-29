# PR #3339 검토 기록 — rhwp-vscode webpack-cli 7

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: ca9a87e540f159aaa2adcaca9662c935d4b2662e
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3339](https://github.com/edwardkim/rhwp/pull/3339) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `javascript` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +35/-90, 2 files, 1 commit |
| head | `ca9a87e540f159aaa2adcaca9662c935d4b2662e` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | 원 PR 직접 merge 대신 VS Code Route B 통합에 수용 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경 범위와 통합

- 원 PR은 `rhwp-vscode`의 `webpack-cli`를 6.0.1에서 7.2.1로 올린다.
- source commit `ca9a87e540f159aaa2adcaca9662c935d4b2662e`를 `git cherry-pick -x`로 적용해
  integration commit `d0ab718ab28b963f318852a043c781e06007550b`을 만들었다.
- 원 저자 `dependabot[bot]`, `Signed-off-by`, source SHA provenance를 보존했다.
- #3388의 TypeScript 7 CLI/TypeScript 6 compiler API alias 위에 충돌 없이 적용됐다.
- 설치된 webpack-cli 7.2.1은 build-time Node `>=20.9.0`을 요구한다. 저장소 CI Node 22와
  로컬 검증 Node 24는 이 조건을 충족하며, dev dependency라 VS Code 1.82 Extension Host에는 포함되지 않는다.

## 렌더 영향 판정

- package manifest와 lockfile만 바꾸며 renderer, layout, paint, WASM API, fixture, font asset을
  변경하지 않는다.
- visual sweep 대상이 아니다.

## 검증

- 원 PR 최신 head의 `Frontend package gates`, `Build & Test`, CodeQL은 문서 작성 시점 성공이다.
- 통합 tree에서 `webpack-cli 7.2.1`, `webpack 5.109.0`, `ts-loader 9.6.2` 해소를 확인했다.
- clean `npm ci`, TypeScript 7 extension/webview `npm run typecheck`, webpack `npm run compile`이 성공했다.
- VS Code font/license contract 3/3과 VSIX package 35 files, 17.35 MB가 성공했다.
- `npm audit`은 취약점 0건이다.

## 리스크와 권고

- release/package 명령의 build-time Node 하한이 올라가므로 Node 18 개발 환경에서는 webpack-cli 7을
  직접 실행할 수 없다. 저장소 CI 계약은 Node 22이며 최신 integration PR CI를 authoritative gate로 둔다.
- **권고**: #3339는 직접 merge하지 않고 확장된 Draft
  [#3388](https://github.com/edwardkim/rhwp/pull/3388)로 대체한다. #3388 merge 뒤 별도 승인으로
  source/integration mapping을 설명하고 원 PR을 close한다.
