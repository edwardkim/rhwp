# PR #3337 검토 기록 — VS Code API type 지원 범위

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: ea5580b77378d685193aed76c880bb5732af2f79
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3337](https://github.com/edwardkim/rhwp/pull/3337) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (local fetch 전 review request) |
| labels / milestone | `dependencies`, `javascript` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +7/-7, 2 files, 1 commit |
| head | `ea5580b77378d685193aed76c880bb5732af2f79` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | 원 PR 직접 merge 보류, 지원 계약을 보정한 Route B 통합으로 대체 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경 범위와 문제

- 원 PR은 `@types/vscode`를 1.110.0에서 1.125.0으로 올리고 manifest 하한도 `^1.125.0`으로 바꾼다.
- 확장 manifest의 `engines.vscode`는 계속 `^1.82.0`이다.
- [VS Code extension manifest 문서](https://code.visualstudio.com/api/references/extension-manifest)는
  `engines.vscode`를 확장이 호환되는 VS Code 버전 범위로 정의한다.
- 1.125 type으로 compile하면서 1.82 설치를 허용하면 1.82에 없는 API 사용을 build가 차단하지 못한다.
  원 PR CI가 성공한 사실은 이 최소 지원 계약을 검증하지 않는다.
- 원 PR 전에도 caret 범위 때문에 lockfile이 1.110.0까지 올라가 있어 선언된 최소 버전과 실제 compile
  type이 이미 어긋나 있었다. 이번 검토는 해당 drift도 함께 닫는다.

## Route B 통합과 collaborator 보정

| 구분 | SHA | 내용 |
|---|---|---|
| 원 commit | `ea5580b77` | Dependabot `@types/vscode` 1.125.0 bump |
| 통합 commit | `beba924eb` | `-x` cherry-pick, 원 저자와 `Signed-off-by` 보존 |
| 보정 commit | `fdc0af5b5` | 최소 지원 API type과 Dependabot 정책 정렬 |

최종 통합 tree에서는 `@types/vscode`를 `~1.82.0`, lockfile을 1.82.0으로 고정한다.
`.github/dependabot.yml`은 `engines.vscode`를 올리기 전 `>=1.83.0` type update를 생성하지 않도록 제한한다.
따라서 원 PR이 요청한 1.125.0 자체는 최종 tree에 남기지 않으며, 원 commit provenance만 보존한 뒤
지원 계약 보정으로 supersede한다.

## 렌더 영향 판정

- API declaration과 lockfile만 바꾸며 renderer·layout·paint·asset을 변경하지 않는다.
- visual sweep 대상이 아니다.

## 검증

- 원 PR CI run
  [#30177238171](https://github.com/edwardkim/rhwp/actions/runs/30177238171)의
  `Build & Test`, `Frontend package gates`, CodeQL은 문서 작성 시점 성공이다.
- 통합 tree에서 `@types/vscode` 1.82.0 해소를 확인했다.
- fresh WASM 뒤 TypeScript 7 extension/webview typecheck와 TypeScript 6 API 기반 webpack compile이
  모두 성공했다.
- VS Code font/license contract 3/3, VSIX package, YAML parse, `git diff --check`가 성공했다.

전체 font contract의 Studio/browser 항목 1건은 `rhwp-studio/dist/fonts` 미생성으로 prerequisite 실패했다.
통합 PR full CI에서 전체 frontend aggregate를 다시 확인한다.

## 리스크와 권고

- 1.82 type으로 compile이 성공했으므로 현재 source가 더 최신 VS Code API를 요구하지 않는 것은 확인했다.
- 실제 VS Code 1.82 Extension Host 설치 smoke는 수행하지 않았다. 이 변경은 type/build 계약 보정이며,
  integration PR의 Node 22 full CI를 merge gate로 둔다.
- **권고**: #3337은 직접 merge하지 않고 보정된 integration PR로 대체한다. integration PR merge 뒤
  별도 승인으로 원 PR에 supersede 설명을 남기고 close한다.
