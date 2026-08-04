# PR #3335 검토 기록 — actions/checkout 7

## 라우팅

```text
base route: collaborator 매개 외부 PR (Route B: 통합 PR)
modifiers: 접수·리뷰 기록, 로컬 검증, 다수 PR·update branch, 재작업·예외(Dependabot)
current head: 500fb12d8f13a7cbb850b71339bc3752617553be
```

## PR metadata

| 항목 | 내용 |
|---|---|
| 원 PR | [#3335](https://github.com/edwardkim/rhwp/pull/3335) |
| 작성자 / base | `dependabot[bot]` / `devel` |
| 검토자 | `@postmelee` (source 통합 전 review request) |
| labels / milestone | `dependencies`, `github_actions` / 없음 |
| assignee | 없음 — GitHub이 Dependabot bot을 assignable actor로 허용하지 않음 |
| 규모 | 2026-07-26 조회: +15/-15, 7 files, 1 commit |
| head | `500fb12d8f13a7cbb850b71339bc3752617553be` |
| 상태 스냅샷 | `MERGEABLE`, `BEHIND`, draft 아님, review decision 없음 |
| 권한 / 관련 issue | `maintainerCanModify=false` / 자동 close 대상 issue 없음 |
| 판단 | Checkout Actions Route B 통합에 수용 |

## 변경과 통합

- CI, CodeQL, Pages, renderer sweep, npm publish, release, Render Diff의 `actions/checkout`을
  v5에서 v7로 올린다.
- source `500fb12d8f13a7cbb850b71339bc3752617553be`를 `git cherry-pick -x`로 적용해
  integration `dafaabbde31493cb8b28d02459c7c3199ea727be`를 만들었다.
- checkout 입력과 fetch 조건은 바꾸지 않았고 원 저자·`Signed-off-by`·source SHA를 보존했다.

## 검증과 리스크

- 원 PR 최신 head의 `Frontend package gates`, `Build & Test`, CodeQL, Render Diff는 문서 작성 시점 성공이다.
- checkout은 대부분의 PR job에서 실제 실행되므로 integration PR full CI가 결합 상태를 직접 검증한다.
- 누적 tree의 diff는 action version 교체만 포함하며 `actionlint -shellcheck=`와 `git diff --check`가 성공했다.
- publish/release/deploy event 전용 checkout 경로는 실제 event에서 추가 확인이 필요하다.
- renderer source나 fixture를 바꾸지 않아 visual sweep 대상이 아니다.

**권고**: #3335는 직접 merge하지 않고 Actions integration PR로 대체한다. 최신 integration head CI와
작업지시자 승인을 merge 조건으로 두고 merge 뒤 별도 승인으로 원 PR을 close한다.
