# PR #3344 검토 기록 — VS Code Extension Host Node type 지원 범위

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 568d7d2932ec98e769175f8bbdf236832b59b7e7
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3344](https://github.com/edwardkim/rhwp/pull/3344) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (local fetch 전 review request) |
| labels / milestone | `dependencies`, `javascript` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +11/-11, 2 files, 1 commit |
| head | `568d7d2932ec98e769175f8bbdf236832b59b7e7` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | 원 PR 직접 merge 보류, runtime type 계약을 보정한 Route B 통합으로 대체 |

metadata와 CI는 문서 작성 시점 참고값이다. 최종 판단 전 최신 integration PR head에서 다시 확인한다.

## 변경 범위와 문제

- 원 PR은 `@types/node`를 20.19.37에서 26.1.1로 올리고 manifest 하한을 `^26.1.1`로 바꾼다.
- 확장은 `engines.vscode: ^1.82.0`을 선언한다.
- [VS Code 1.82 release notes](https://code.visualstudio.com/updates/v1_82)는 Extension Host가 포함된
  Electron 25의 Node.js를 18.15.0으로 갱신했다고 기록한다.
- Node 26 type으로 compile하면서 VS Code 1.82 설치를 허용하면 런타임에 없는 Node API 사용을 build가
  차단하지 못한다. 원 PR의 녹색 CI는 Node 26 type compilation이 성공했다는 뜻이지 1.82 runtime
  호환성을 보증하지 않는다.
- 원 PR 전의 `@types/node ^20.0.0`도 최소 runtime보다 높았으므로 이번 검토에서 함께 정렬한다.

## Route B 통합과 collaborator 보정

| 구분 | SHA | 내용 |
|---|---|---|
| 원 commit | `568d7d293` | Dependabot `@types/node` 26.1.1 bump |
| 통합 commit | `22222092f` | `-x` cherry-pick, 원 저자와 `Signed-off-by` 보존 |
| 보정 commit | `fdc0af5b5` | Node 18.15 type과 Dependabot 정책 정렬 |

#3334와 #3337을 먼저 적용한 상태에서 #3344를 cherry-pick하자 두 manifest의 같은
`devDependencies` 문맥에서 충돌했다. 충돌 해소 단계에서는 `@types/node 26.1.1`과
`@types/vscode 1.125.0` 원 bump를 모두 보존해 원 commit을 완료한 뒤, collaborator 보정 commit에서
지원 계약을 별도로 적용했다.

최종 통합 tree는 `@types/node ~18.15.0`, lockfile 18.15.13을 사용한다.
`.github/dependabot.yml`은 minimum Extension Host를 올리기 전 `>=18.16.0` type update를 생성하지 않도록
제한한다. 원 PR이 요청한 26.1.1은 최종 tree에 남기지 않는다.

## 렌더 영향 판정

- Node declaration과 lockfile만 바꾸며 renderer·layout·paint·asset을 변경하지 않는다.
- visual sweep 대상이 아니다.

## 검증

- 원 PR CI run
  [#30177250840](https://github.com/edwardkim/rhwp/actions/runs/30177250840)의
  `Build & Test`, `Frontend package gates`, CodeQL은 문서 작성 시점 성공이다.
- 통합 tree에서 `@types/node` 18.15.13 해소를 확인했다.
- `types: ["node"]`를 명시한 TypeScript 7 extension typecheck와 TypeScript 6 API 기반 webpack
  extension bundle이 성공했다.
- fresh WASM, webview typecheck/bundle, VS Code font/license contract 3/3, VSIX package,
  YAML parse, `git diff --check`가 성공했다.

전체 font contract의 Studio/browser 항목 1건은 `rhwp-studio/dist/fonts` 미생성으로 prerequisite 실패했다.
통합 PR full CI에서 전체 frontend aggregate를 다시 확인한다.

## 리스크와 권고

- 실제 VS Code 1.82 Extension Host 설치 smoke는 수행하지 않았다. compile type을 최소 runtime에 맞춘
  것이 이번 변경의 핵심 보증이다.
- **권고**: #3344는 직접 merge하지 않고 보정된 integration PR로 대체한다. integration PR의 Node 22
  full CI와 작업지시자 승인을 merge 조건으로 두며, merge 뒤 별도 승인으로 원 PR을 close한다.
