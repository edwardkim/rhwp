# PR #3342 검토 기록 — rhwp-vscode ts-loader 9.6

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 5e70de300180d0e1c1c7326bb28d172f254c1516
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3342](https://github.com/edwardkim/rhwp/pull/3342) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `javascript` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +19/-104, 2 files, 1 commit |
| head | `5e70de300180d0e1c1c7326bb28d172f254c1516` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | TypeScript API bridge를 보존한 VS Code Route B 통합에 수용 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경 범위와 충돌 해소

- 원 PR은 `rhwp-vscode`의 `ts-loader`를 9.5.4에서 9.6.2로 올린다.
- source commit `5e70de300180d0e1c1c7326bb28d172f254c1516`을 `git cherry-pick -x`로 적용해
  integration commit `9ef89e64ea5b701407d1b7bebfe67913ceaf10c2`을 만들었다.
- 원 저자 `dependabot[bot]`, `Signed-off-by`, source SHA provenance를 보존했다.
- 원 PR과 #3388이 같은 `devDependencies`와 lockfile 전이 항목을 바꿔 content conflict가 발생했다.
- 충돌 해소에서는 #3342의 `ts-loader ^9.6.2`를 채택하되, `ts-loader`가 import하는
  `typescript`는 기존 `npm:@typescript/typescript6@^6.0.2` compiler API alias로 유지했다.
- TypeScript 7 CLI, webpack 5.109.0, webpack-cli 7.2.1과 함께 최종 lockfile을 다시 계산했다.

## 렌더 영향 판정

- build tooling manifest와 lockfile만 바꾸며 runtime renderer/layout/paint, fixture, font asset을
  변경하지 않는다.
- visual sweep 대상이 아니다.

## 검증

- 원 PR 최신 head의 `Frontend package gates`, `Build & Test`, CodeQL은 문서 작성 시점 성공이다.
- 통합 tree에서 `ts-loader 9.6.2`가 TypeScript 6.0.3 compiler API와 함께 설치됨을 확인했다.
- TypeScript 7.0.2 CLI의 extension/webview typecheck와 webpack 5.109.0 compile이 성공했다.
- clean `npm ci`, VS Code font/license contract 3/3, VSIX 35 files, 17.35 MB가 성공했다.
- `npm audit`은 취약점 0건이다.

## 리스크와 권고

- TypeScript 7은 programmatic compiler API를 제공하지 않으므로 `typescript` alias를 다시 7.x로
  되돌리면 `ts-loader` build가 재차 실패한다. 이 통합은 CLI와 API package 분리를 merge 조건으로 둔다.
- **권고**: #3342는 직접 merge하지 않고 충돌 해소 commit을 포함한 Draft
  [#3388](https://github.com/edwardkim/rhwp/pull/3388)로 대체한다. #3388 merge 뒤 별도 승인으로
  원 PR을 close한다.
