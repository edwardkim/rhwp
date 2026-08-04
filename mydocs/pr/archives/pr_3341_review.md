# PR #3341 검토 기록 — rhwp-vscode webpack 5.109

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 5d94473495e20079e1a90c268d4717b263a94030
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3341](https://github.com/edwardkim/rhwp/pull/3341) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `javascript` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +95/-150, 2 files, 1 commit |
| head | `5d94473495e20079e1a90c268d4717b263a94030` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | 충돌을 해소한 VS Code Route B 통합에 수용 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경 범위와 충돌 해소

- 원 PR은 `rhwp-vscode`의 `webpack`을 5.105.4에서 5.109.0으로 올린다.
- source commit `5d94473495e20079e1a90c268d4717b263a94030`을 `git cherry-pick -x`로 적용해
  integration commit `9b17a601c21d3fc13e43f1b08d253b3b13b335e6`을 만들었다.
- 원 저자 `dependabot[bot]`, `Signed-off-by`, source SHA provenance를 보존했다.
- 원 PR과 #3388이 같은 `devDependencies`와 lockfile을 바꿔 content conflict가 발생했다.
- 충돌 해소에서는 #3341의 `webpack ^5.109.0`을 채택하면서 #3388의
  `typescript: npm:@typescript/typescript6` API alias와 #3339의 `webpack-cli ^7.2.1`을 유지했다.
- 최종 manifest를 기준으로 npm lockfile을 다시 계산했다.

## 렌더 영향 판정

- build tooling manifest와 lockfile만 바꾸며 runtime renderer/layout/paint, fixture, font asset을
  변경하지 않는다.
- visual sweep 대상이 아니다.

## 검증

- 원 PR 최신 head의 `Frontend package gates`, `Build & Test`, CodeQL은 문서 작성 시점 성공이다.
- 통합 tree의 clean `npm ci`와 `npm audit` 취약점 0건을 확인했다.
- TypeScript 7.0.2 CLI typecheck와 TypeScript 6.0.3 compiler API 기반 webpack build가 성공했다.
- extension/webview 두 bundle은 webpack 5.109.0으로 성공했다.
- VS Code font/license contract 3/3과 VSIX package 35 files, 17.35 MB가 성공했다.

## 리스크와 권고

- 원 PR의 녹색 CI는 #3388의 TypeScript alias와 결합된 lockfile을 검증하지 않는다. 최신 #3388 head의
  full `Frontend package gates`와 `Build & Test`가 authoritative gate다.
- **권고**: #3341은 직접 merge하지 않고 충돌 해소 commit을 포함한 Draft
  [#3388](https://github.com/edwardkim/rhwp/pull/3388)로 대체한다. #3388 merge 뒤 별도 승인으로
  원 PR을 close한다.
