# PR #3330 umbrella implementation 계획 — GitHub Actions 의존성 7건

## 목적과 상태

- 대상: Dependabot PR [#3330](https://github.com/edwardkim/rhwp/pull/3330),
  [#3331](https://github.com/edwardkim/rhwp/pull/3331),
  [#3332](https://github.com/edwardkim/rhwp/pull/3332),
  [#3333](https://github.com/edwardkim/rhwp/pull/3333),
  [#3335](https://github.com/edwardkim/rhwp/pull/3335),
  [#3336](https://github.com/edwardkim/rhwp/pull/3336),
  [#3338](https://github.com/edwardkim/rhwp/pull/3338)
- 역할: 일곱 원 PR을 직접 merge하지 않고 최신 `upstream/devel` 기반 Route B integration PR로 대체한다.
- 현재 상태: source cherry-pick과 local workflow 검증을 완료하고 review 문서를 작성하는 단계다.
- 이 문서는 integration PR merge 전 실행 계획이다. remote push, PR 생성, 최신 CI 성공, merge,
  원 PR close를 완료 사실로 기록하지 않는다.

## 라우팅과 기준선

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
integration branch: review/dependabot-actions-20260726
base: upstream/devel@e7dffced399e45685ae746bd2ea21d37542ea95e
verified local code head: 07431b472cfb32536a4b228e591ab86061eb825c
```

일곱 원 PR은 모두 `maintainerCanModify=false`이므로 Dependabot head에는 commit을 push하지 않는다.
원 PR별 reviewer를 source 통합 전에 `@postmelee`로 지정했다. 기존 `dependencies`, `github_actions`
labels를 유지하고 milestone은 만들지 않았다. Dependabot bot은 assignable actor가 아니어서 assignee는
비워 두었다.

## source와 integration commit

| 순서 | 원 PR | source commit | integration commit | 변경 |
|---|---|---|---|---|
| 1 | #3330 | `12f1421ebeb66b4f033ab074a473b5724342f23d` | `f1bcf5f510258e09bd2b30e71c78733683f8b9b1` | actions/github-script v9 |
| 2 | #3331 | `7b6d82dee56b7526fa369646ce08a987b3d50164` | `8977c4c5a1eb80afb0f19923585c193e9314f8f9` | actions/deploy-pages v5 |
| 3 | #3332 | `75e81063cbc670d36ce62e2148cb4454f2057b07` | `84a22e897bc5eb69bb9b8a980a84f3e133194d1b` | softprops/action-gh-release v3 |
| 4 | #3333 | `83a505dd5466e8317730cb42057a5fd3d6272626` | `38733dcf6594fb8235cd9dbaad040f812ac5e2fc` | actions/setup-node v7 |
| 5 | #3335 | `500fb12d8f13a7cbb850b71339bc3752617553be` | `dafaabbde31493cb8b28d02459c7c3199ea727be` | actions/checkout v7 |
| 6 | #3336 | `b9a36c58fee963a6cd281a55477c3ee137ecd564` | `d95a1c37af44cd83c89b3b066b6a21044f848e9e` | actions/upload-pages-artifact v5 |
| 7 | #3338 | `1c821e9e5010a25fbaea73cd780d7015730d20ad` | `07431b472cfb32536a4b228e591ab86061eb825c` | actions/cache v6 |

모든 source commit은 `git cherry-pick -x`로 적용했다. author는 `dependabot[bot]`으로 유지되고
`Signed-off-by`와 source SHA provenance가 commit message에 보존된다. 충돌이나 collaborator code
보정은 없었다.

## 변경 범위

- 8개 `.github/workflows/*.yml`에서 action major version만 교체한다.
- 최종 diff는 41 additions/41 deletions이며 모두 `uses:` 줄이다.
- workflow trigger, permissions, condition, input, cache key/path, script 본문은 바꾸지 않는다.
- source, fixture, renderer, WASM, package manifest는 변경하지 않는다.

## 단계

| 단계 | 상태 | 작업 |
|---|---|---|
| 0. metadata 정렬 | 완료 | 일곱 원 PR reviewer 지정, labels/milestone 확인 |
| 1. Route B branch | 완료 | 최신 `upstream/devel`에서 integration branch 생성 |
| 2. source 통합 | 완료 | 일곱 source commit을 번호순 `-x` cherry-pick |
| 3. local verification | 완료 | 최종 diff, actionlint core, diff check |
| 4. review 문서 | 현재 단계 | 원 PR별 review와 umbrella report를 별도 docs commit으로 작성 |
| 5. remote integration PR | 승인됨·미실행 | origin push와 `devel` 대상 Draft PR 생성 |
| 6. authoritative CI/review | 미실행 | 최신 integration head full CI 확인 |
| 7. merge/후속 | 별도 승인 필요 | integration merge, 원 PR comment/close |

## 검증 결과와 한계

- `git diff --check upstream/devel...HEAD`: 성공.
- 최종 workflow diff 수동 확인: action `uses:` version 외 변경 없음.
- `actionlint -shellcheck=`: 성공.
- 전체 `actionlint`는 기존 release/render script 줄에서 shellcheck info 8건을 반환했다.
  이번 diff는 해당 `run:` script를 변경하지 않는다.
- 일곱 원 PR의 최신 head `Build & Test`와 CodeQL은 문서 작성 시점 성공이다.
- PR 이벤트에서 실행되는 checkout, setup-node, cache restore, github-script는 integration PR CI에서
  결합 상태를 검증한다.
- Pages deploy, release attach, npm publish, cache save, devel issue close는 event/권한 특성상 PR CI가
  실제 mutation까지 실행하지 않는다. merge 뒤 최초 해당 workflow run 확인을 남은 운영 gate로 둔다.

## Remote PR 계획과 승인 경계

작업지시자는 branch push와 Draft PR 생성을 승인했다. PR body에는 다음을 포함한다.

- 일곱 원 PR의 `Supersedes #...`
- source/integration commit mapping과 Dependabot credit 보존
- 변경이 action version에 한정된다는 diff 요약
- local actionlint와 원 PR CI, 최신 integration PR CI gate
- deploy/release/publish 전용 경로의 잔여 확인 계획
- integration PR merge 뒤 별도 승인으로 원 PR을 close한다는 명시

연결된 issue가 없어 `Closes #...`는 넣지 않는다. merge, 원 PR comment/close는 별도 승인이 필요하다.
