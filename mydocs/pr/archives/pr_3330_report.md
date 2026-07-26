# PR #3330 umbrella 사전 판단 보고서 — GitHub Actions 의존성 7건

> 이 문서는 PR #3330/#3331/#3332/#3333/#3335/#3336/#3338을 통합한 Route B candidate의
> pre-merge 판단이다. remote PR 생성·최신 CI 성공·merge·원 PR close는 아직 완료되지 않았다.

## 판단 요약

| 항목 | 내용 |
|---|---|
| 대상 | [#3330](https://github.com/edwardkim/rhwp/pull/3330), [#3331](https://github.com/edwardkim/rhwp/pull/3331), [#3332](https://github.com/edwardkim/rhwp/pull/3332), [#3333](https://github.com/edwardkim/rhwp/pull/3333), [#3335](https://github.com/edwardkim/rhwp/pull/3335), [#3336](https://github.com/edwardkim/rhwp/pull/3336), [#3338](https://github.com/edwardkim/rhwp/pull/3338) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| route | Route B — source commit cherry-pick integration PR |
| 기준선 | `upstream/devel@e7dffced399e45685ae746bd2ea21d37542ea95e` |
| local code candidate | `07431b472cfb32536a4b228e591ab86061eb825c` |
| 권고 | 원 PR 직접 merge 대신 축별 integration candidate를 수용 |
| 최종 조건 | 최신 integration PR head full CI 성공과 작업지시자 merge 승인 |

## 변경 축

| action | version | 주요 경로 |
|---|---|---|
| `actions/github-script` | v8 → v9 | CI/CodeQL/Render preflight, devel issue close |
| `actions/deploy-pages` | v4 → v5 | Pages deploy |
| `softprops/action-gh-release` | v2 → v3 | tag release artifact attach |
| `actions/setup-node` | v6 → v7 | frontend, Pages, renderer sweep, npm publish |
| `actions/checkout` | v5 → v7 | CI/CodeQL/Pages/release/publish/render |
| `actions/upload-pages-artifact` | v3 → v5 | Pages artifact |
| `actions/cache` | v5 → v6 | CI/CodeQL/Pages/release/publish/render cache |

8개 workflow의 최종 diff는 41 additions/41 deletions이며 모두 `uses:` version 교체다.
trigger, permission, condition, action input, shell script는 변경하지 않는다.

## source provenance와 credit

| 원 PR | source commit | integration commit |
|---|---|---|
| #3330 | `12f1421ebeb66b4f033ab074a473b5724342f23d` | `f1bcf5f510258e09bd2b30e71c78733683f8b9b1` |
| #3331 | `7b6d82dee56b7526fa369646ce08a987b3d50164` | `8977c4c5a1eb80afb0f19923585c193e9314f8f9` |
| #3332 | `75e81063cbc670d36ce62e2148cb4454f2057b07` | `84a22e897bc5eb69bb9b8a980a84f3e133194d1b` |
| #3333 | `83a505dd5466e8317730cb42057a5fd3d6272626` | `38733dcf6594fb8235cd9dbaad040f812ac5e2fc` |
| #3335 | `500fb12d8f13a7cbb850b71339bc3752617553be` | `dafaabbde31493cb8b28d02459c7c3199ea727be` |
| #3336 | `b9a36c58fee963a6cd281a55477c3ee137ecd564` | `d95a1c37af44cd83c89b3b066b6a21044f848e9e` |
| #3338 | `1c821e9e5010a25fbaea73cd780d7015730d20ad` | `07431b472cfb32536a4b228e591ab86061eb825c` |

일곱 commit은 `git cherry-pick -x`로 적용해 `dependabot[bot]` author, `Signed-off-by`, source SHA를
보존했다. 충돌, squash, contributor commit rewrite는 없었다.

## 검증

| 검증 | 결과 |
|---|---|
| 누적 cherry-pick | 7/7 충돌 없음 |
| 최종 diff 확인 | `uses:` action version 교체만 존재 |
| `actionlint -shellcheck=` | 성공 |
| 전체 `actionlint` | 기존 unchanged script의 shellcheck info 8건 |
| `git diff --check` | 성공 |
| 원 PR 최신 head CI | 7건 모두 `Build & Test`, CodeQL 성공 |

Route B의 authoritative CI는 새 integration PR의 최신 head 결과다. checkout, setup-node, cache restore,
github-script 등 PR 이벤트 경로는 그 CI에서 실제 실행된다.

## 실행되지 않는 event 전용 경로

- `deploy-pages.yml`의 Pages artifact upload/deploy
- `release-binary.yml`의 release artifact attach
- `npm-publish.yml`의 registry publish
- push 조건의 cache save
- `close-issues-on-devel-push.yml`의 issue mutation

이 경로들은 PR 권한과 event 차이 때문에 local/actionlint와 PR CI만으로 side effect를 끝까지 재현하지 않는다.
입력·권한·조건이 바뀌지 않은 것을 확인했으며, merge 뒤 최초 실제 run을 운영 확인 항목으로 남긴다.

## 문서와 remote 반영 계획

- 원 PR별 review:
  [#3330](pr_3330_review.md), [#3331](pr_3331_review.md), [#3332](pr_3332_review.md),
  [#3333](pr_3333_review.md), [#3335](pr_3335_review.md), [#3336](pr_3336_review.md),
  [#3338](pr_3338_review.md)
- 실행 순서와 승인 gate: [umbrella implementation 계획](pr_3330_review_impl.md)
- 원 PR은 `maintainerCanModify=false`이므로 source head에 commit을 push하지 않는다.
- 승인된 `review/dependabot-actions-20260726`을 origin에 push하고 `devel` 대상 Draft PR을 생성한다.
- PR body에 일곱 `Supersedes`, source/integration mapping, credit, 검증과 event 전용 잔여 gate를 기록한다.

## Merge 전 조건

1. docs commit까지 포함한 integration PR 최신 head SHA를 확인한다.
2. `CI preflight`, `Lint`, `Frontend package gates`, `Build & Test`, CodeQL, Render Diff 등
   최신 relevant checks가 성공해야 한다.
3. review 문서가 integration PR diff에 포함되어야 한다.
4. integration PR이 mergeable 상태여야 한다.
5. 작업지시자가 GitHub review/merge를 별도로 승인해야 한다.

workflow 변경이므로 review-only fast-pass를 적용하지 않는다.

## Merge 뒤 확인 계획

- integration PR merge commit과 merge 시각을 다시 읽는다.
- event 전용 workflow의 최초 실제 run 상태를 확인한다.
- 별도 승인 뒤 각 원 PR에 integration PR 번호, merge commit, source/integration mapping, CI 요약,
  Dependabot credit 보존을 설명하고 superseded 상태로 close한다.
- 일곱 원 PR에는 linked issue가 없다.

**사전 권고**: source provenance를 보존하고 action version 교체만 포함한 Route B candidate를 Draft PR
생성 후보로 수용한다. merge와 원 PR 후속 처리는 최신 integration CI와 별도 승인 뒤 수행한다.
