# PR #3334 검토 기록 — rhwp-vscode TypeScript 7 전환

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, rework_and_exceptions.md
current head: 9a77d8362d5bd86701c68a4799d434736742c62e
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3334](https://github.com/edwardkim/rhwp/pull/3334) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (local fetch 전 review request) |
| labels / milestone | `dependencies`, `javascript` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +373/-12, 2 files, 1 commit |
| head | `9a77d8362d5bd86701c68a4799d434736742c62e` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | 원 PR 직접 merge 보류, 보정된 Route B 통합에 수용 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경 범위

- `rhwp-vscode/package.json`의 `typescript`를 5.9.3에서 7.0.2로 올린다.
- `rhwp-vscode/package-lock.json`에 TypeScript 7 네이티브 배포물과 플랫폼별 optional package를 반영한다.
- 원 PR에는 source, test, runtime asset 변경이 없다.

## 발견한 차단 문제

원 PR의 CI run
[#30177235189](https://github.com/edwardkim/rhwp/actions/runs/30177235189)에서
`Frontend package gates`가 실패했다. `npm --prefix rhwp-vscode run compile` 중 `ts-loader`가
TypeScript compiler API의 `fileExists`를 읽으려다 다음 오류를 냈다.

```text
TypeError: Cannot read properties of undefined (reading 'fileExists')
```

[TypeScript 7 공식 발표](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/)에 따르면
7.0은 programmatic API를 제공하지 않으며, API가 필요한 도구는
`@typescript/typescript6` compatibility package를 npm alias로 함께 사용해야 한다. 따라서
`ts-loader`가 `typescript` 7.0.2를 직접 import하는 원 PR 상태는 source 수정 없이 해소되지 않는다.

## Route B 통합과 collaborator 보정

| 구분 | SHA | 내용 |
|---|---|---|
| 원 commit | `9a77d8362` | Dependabot TypeScript 7.0.2 bump |
| 통합 commit | `3039b4d8e` | `-x` cherry-pick, 원 저자와 `Signed-off-by` 보존 |
| 보정 commit | `fdc0af5b5` | TypeScript 7 CLI와 TypeScript 6 API 병행, 설정·검증 gate 보완 |

보정 결과는 다음과 같다.

- `@typescript/native: npm:typescript@^7.0.2`로 TypeScript 7 CLI를 유지한다.
- `typescript: npm:@typescript/typescript6@^6.0.2`로 `ts-loader`가 사용할 compiler API를 제공한다.
- `typecheck` script에서 TypeScript 7 CLI로 extension/webview 두 config를 검사한 뒤 webpack을 실행한다.
- TypeScript 7 설정 계약에 맞게 Node type을 명시하고 webview의 제거된 `baseUrl` 의존을 없앤다.
- 최신 DOM type에서 `HTMLElement.hidden`이 `"until-found"`를 포함할 수 있으므로 메뉴 토글 입력을
  명시적 boolean으로 정규화한다.

여러 PR의 실행 순서와 전체 보정은 [통합 implementation 계획](pr_3334_review_impl.md), 사전 수용 판단은
[통합 사전 보고서](pr_3334_report.md)에 기록한다.

## 렌더 영향 판정

- renderer, layout, paint, WASM API, fixture, font asset은 변경하지 않는다.
- `viewer.ts` 변경은 메뉴 open/close 입력의 타입 정규화이며 기존 boolean 값에서 동작이 같다.
- 페이지 수·배치·SVG/PDF 산출 주장이 없으므로 visual sweep 대상이 아니다.

## 검증

보정된 통합 branch `review/dependabot-vscode-20260726`에서 다음을 확인했다.

- `CARGO_INCREMENTAL=0 wasm-pack build --target web --dev`: 성공.
- `npm ci`: 성공, 최종 audit 취약점 0건.
- TypeScript 선택 확인: `tsc` 7.0.2, `require("typescript")` compiler API 6.0.3.
- `npm run typecheck`: extension/webview config 모두 성공.
- `npm run compile`: extension/webview webpack bundle 모두 성공.
- `node --test --test-name-pattern='VS Code' scripts/frontend-font-assets.test.mjs`: 3/3 성공.
- `npx --yes @vscode/vsce package`: 35 files, 17.35 MB VSIX 생성 성공.
- `.github/dependabot.yml` YAML parse와 `git diff --check`: 성공.

전체 font contract 스크립트도 실행했으나 이번 범위에서 생성하지 않은 `rhwp-studio/dist/fonts`가 없어
Studio/browser distribution 1건이 prerequisite 실패했다. 같은 스크립트의 VS Code 3건은 모두 성공했으며,
통합 PR의 full CI에서는 Studio build를 포함한 전체 `Frontend package gates`가 성공해야 한다.

## 리스크와 권고

- 원 PR head의 `Build & Test`는 실패 상태이므로 직접 merge할 수 없다.
- 로컬 검증은 Node 24에서 수행했다. 저장소 CI의 Node 22 결과가 authoritative gate다.
- **권고**: 원 PR은 merge하지 않고, 보정 commit을 포함한 Route B integration PR의 최신 head CI가
  성공하고 작업지시자가 승인한 뒤 통합한다. 통합 PR merge 뒤 별도 승인으로 원 PR에 supersede 설명을
  남기고 close한다.
