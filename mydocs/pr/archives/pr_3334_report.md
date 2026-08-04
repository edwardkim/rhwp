# PR #3334 umbrella 사전 판단 보고서 — VS Code 의존성 6건

> 이 문서는 PR #3334/#3337/#3339/#3341/#3342/#3344를 보정·통합한 Route B candidate의
> pre-merge 판단이다. Draft #3388은 생성됐지만 확장 head CI 성공·merge·원 PR close는 완료되지 않았다.

## 판단 요약

| 항목 | 내용 |
|---|---|
| 대상 | [#3334](https://github.com/edwardkim/rhwp/pull/3334), [#3337](https://github.com/edwardkim/rhwp/pull/3337), [#3339](https://github.com/edwardkim/rhwp/pull/3339), [#3341](https://github.com/edwardkim/rhwp/pull/3341), [#3342](https://github.com/edwardkim/rhwp/pull/3342), [#3344](https://github.com/edwardkim/rhwp/pull/3344) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| route | Route B — source commit cherry-pick + collaborator fix integration PR |
| 기준선 | `upstream/devel@e7dffced399e45685ae746bd2ea21d37542ea95e` |
| local code candidate | `9ef89e64ea5b701407d1b7bebfe67913ceaf10c2` |
| 권고 | 원 PR 직접 merge는 보류하고 보정 integration candidate를 수용 |
| 최종 조건 | 최신 integration PR head full CI 성공과 작업지시자 merge 승인 |

## 원인과 사전 판단

### #3334 TypeScript 7

TypeScript 7은 기존 programmatic compiler API를 제공하지 않는다. 원 PR은 `ts-loader`가 import하는
`typescript`를 곧바로 7.0.2로 바꿔 `fileExists` 접근에서 CI가 실패했다. TypeScript 7 CLI와
`@typescript/typescript6` API alias를 병행하는 공식 migration 형태로 보정했다.

### #3337 VS Code API type

`engines.vscode ^1.82.0`을 유지하면서 `@types/vscode ^1.125.0`으로 compile하면 최소 지원 버전에 없는 API
사용을 build가 허용할 수 있다. 원 PR CI는 성공했지만 지원 계약을 보증하지 않으므로 1.82 type으로 고정했다.

### #3344 Node type

VS Code 1.82 Extension Host는 Node 18.15를 기준으로 하는데 Node 26 type으로 compile하면 더 최신 runtime
API가 노출된다. Node 18.15 type으로 고정하고 같은 범위를 Dependabot 정책에 반영했다.

### #3339/#3341/#3342 webpack build tooling

webpack-cli 7.2.1, webpack 5.109.0, ts-loader 9.6.2를 #3388의 보정 tree 위에 통합했다.
#3339는 자동 적용됐지만 #3341과 #3342는 같은 `devDependencies`와 lockfile 문맥에서 충돌했다.
각 원 version bump를 채택하면서 TypeScript 7 CLI/TypeScript 6 compiler API alias, VS Code 1.82 type,
Node 18.15 type을 유지하고 최종 manifest 기준으로 lockfile을 다시 계산했다.

## 변경 파일

| 파일 | collaborator 보정 |
|---|---|
| `.github/dependabot.yml` | VS Code 1.82/Node 18.15보다 새로운 type update 제한 |
| `rhwp-vscode/package.json` | TypeScript 7 CLI + TypeScript 6 API alias, typecheck gate, runtime type 고정 |
| `rhwp-vscode/package-lock.json` | alias/type 해소 및 `fast-uri` 3.1.4 보안 patch |
| `rhwp-vscode/tsconfig.json` | Node type 명시 |
| `rhwp-vscode/tsconfig.webview.json` | TypeScript 7 config 호환과 공유 source root 명시 |
| `rhwp-vscode/src/webview/viewer.ts` | 최신 DOM `hidden` type의 boolean 정규화 |

review 문서는 이 source/config 보정과 분리된 docs commit으로 추가한다.

## source provenance와 credit

| 원 PR | source commit | integration commit |
|---|---|---|
| #3334 | `9a77d8362d5bd86701c68a4799d434736742c62e` | `3039b4d8e08d4c2a847a601554cf6d6fe508ba02` |
| #3337 | `ea5580b77378d685193aed76c880bb5732af2f79` | `beba924eb4f4e8f64fec55e8f1d6927c9b6d8676` |
| #3339 | `ca9a87e540f159aaa2adcaca9662c935d4b2662e` | `d0ab718ab28b963f318852a043c781e06007550b` |
| #3341 | `5d94473495e20079e1a90c268d4717b263a94030` | `9b17a601c21d3fc13e43f1b08d253b3b13b335e6` |
| #3342 | `5e70de300180d0e1c1c7326bb28d172f254c1516` | `9ef89e64ea5b701407d1b7bebfe67913ceaf10c2` |
| #3344 | `568d7d2932ec98e769175f8bbdf236832b59b7e7` | `22222092feab59ee67fc6784ce0b6ce72f8caf23` |

여섯 commit은 `git cherry-pick -x`로 적용해 `dependabot[bot]` author, `Signed-off-by`, source SHA를 보존했다.
collaborator fix `fdc0af5b5c2d8b266990419fbc87cc0543589717`은 별도 commit이다.

## 로컬 검증

| 검증 | 결과 |
|---|---|
| fresh WASM `wasm-pack build --target web --dev` | 성공 |
| 확장 lockfile clean `npm ci` | 성공, audit 취약점 0건 |
| TypeScript compiler 선택 | CLI 7.0.2 / API 6.0.3 |
| build tooling 선택 | webpack-cli 7.2.1 / webpack 5.109.0 / ts-loader 9.6.2 |
| `npm run typecheck` | TypeScript 7 extension/webview 성공 |
| `npm run compile` | TypeScript 6 API 기반 extension/webview webpack 성공 |
| VS Code font/license contract | 3/3 성공 |
| VSIX package | 성공, 35 files / 17.35 MB |
| Dependabot YAML parse | 성공 |
| `git diff --check` | 성공 |

전체 font contract 실행에서는 이번 검토가 생성하지 않은 `rhwp-studio/dist/fonts`가 없어
Studio/browser distribution 1건이 prerequisite 실패했다. VS Code 관련 3건은 같은 실행 산출물에서 모두
성공했다. full frontend aggregate는 integration PR CI에서 다시 확인해야 한다.

## 원 PR CI 스냅샷

- #3334 run
  [#30177235189](https://github.com/edwardkim/rhwp/actions/runs/30177235189):
  `Frontend package gates`, `Build & Test` 실패.
- #3337 run
  [#30177238171](https://github.com/edwardkim/rhwp/actions/runs/30177238171):
  `Frontend package gates`, `Build & Test`, CodeQL 성공.
- #3344 run
  [#30177250840](https://github.com/edwardkim/rhwp/actions/runs/30177250840):
  `Frontend package gates`, `Build & Test`, CodeQL 성공.
- #3339, #3341, #3342 최신 source head:
  `Frontend package gates`, `Build & Test`, CodeQL 성공.

이 값은 2026-07-26 원 PR head 기준 참고값이다. Route B의 authoritative CI는 향후 integration PR의
최신 head 결과다.

## 렌더·시각 검증

- renderer/layout/paint/WASM API/fixture/font asset 변경이 없다.
- webview source 1줄은 메뉴 open 상태를 boolean으로 정규화하며 페이지 조판·렌더 출력과 무관하다.
- visual sweep과 기준 asset은 필요하지 않다고 판정했다.

## 문서와 remote 반영 계획

- 원 PR별 review:
  [#3334](pr_3334_review.md), [#3337](pr_3337_review.md), [#3339](pr_3339_review.md),
  [#3341](pr_3341_review.md), [#3342](pr_3342_review.md), [#3344](pr_3344_review.md)
- 실행 순서와 승인 gate: [umbrella implementation 계획](pr_3334_review_impl.md)
- 원 PR은 `maintainerCanModify=false`이므로 review/code/docs를 source head에 push하지 않는다.
- 작업지시자 승인에 따라 확장 branch를 `origin`에 push하고 기존 Draft #3388 metadata를 갱신한다.
- PR body는 여섯 원 PR을 `Supersedes`로 표시하고 source/integration mapping과 contributor credit을 기록한다.

## Merge 전 조건

1. docs commit까지 포함한 integration PR 최신 head SHA를 확인한다.
2. `Frontend package gates`, `Build & Test`, CodeQL 등 관련 최신 full CI가 성공해야 한다.
3. review 문서가 integration PR diff에 포함되어야 한다.
4. integration PR이 mergeable 상태여야 한다.
5. 작업지시자가 GitHub review/merge를 별도로 승인해야 한다.

code/config 보정이 포함되므로 review-only fast-pass를 적용하지 않는다.

## Merge 뒤 확인 계획

- integration PR의 merge commit과 merge 시각을 GitHub에서 다시 읽는다.
- 별도 승인 뒤 각 원 PR에 integration PR 번호, merge commit, source/integration mapping,
  CI 요약, Dependabot credit 보존을 설명하고 superseded 상태로 close한다.
- 여섯 원 PR에는 linked issue가 없다. merge 뒤에도 상태를 재확인하되 수동 close할 issue는 현재 없다.
- `upstream/devel` 반영을 확인한 뒤 local branch, fetch branch, 생성물 정리는 별도 종료 gate에서 수행한다.

## 잔여 리스크와 결론

- 로컬 검증은 Node 24에서 수행했으므로 저장소 CI Node 22 결과가 필요하다.
- webpack-cli 7.2.1의 build-time Node 하한은 20.9.0이다. 저장소 CI Node 22는 충족하지만 Node 18
  개발 환경에서 package/compile 명령을 실행할 수 없다.
- 실제 VS Code 1.82 Extension Host 설치 smoke는 수행하지 않았다. minimum API/runtime type compile과
  VSIX package를 이번 local 근거로 사용한다.
- 전체 frontend aggregate는 integration PR에서 아직 실행되지 않았다.

**사전 권고**: 보정·확장된 Route B integration candidate를 Draft #3388의 최신 후보로 수용한다.
원 PR 여섯 건은 직접 merge하지 않는다. 확장 head CI 후 review, merge, 원 PR close는 각 승인 게이트를
통과한 뒤 수행한다.
